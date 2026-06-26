use crate::ast::{BinaryOp, Expr, Function, LogicalOp, Program, Stmt, UnaryOp};
use crate::sema::{ProgramLayout, SymbolTable};

//version 5: the System V AMD64 calling convention passes the first six integer/
//pointer arguments in these registers, in this order
//Return values come back in rax, sema guarantees we never exceed six
const ARG_REGS: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

//version 4: control flow needs unique jump targets. each call hands back a fresh
//integer that I formatted as `.L<n>` so labels never collide
fn next_label(labels: &mut usize) -> usize {
    let id = *labels;
    *labels += 1;
    id
}

//version 5: push/pop helpers that also track how many 8-byte values are currently
//on the stack. After the prologue rsp is 16-byte aligned, so rsp is aligned exactly
// when depth is even
fn push(asm: &mut String, depth: &mut i64) {
    asm.push_str("  push rax\n");
    *depth += 1;
}

fn pop(asm: &mut String, depth: &mut i64, reg: &str) {
    asm.push_str(&format!("  pop {reg}\n"));
    *depth -= 1;
}

pub fn generate(program: &Program, layout: &ProgramLayout) -> String {
    let mut asm = String::new();
    asm.push_str(".intel_syntax noprefix\n");
    asm.push_str(".text\n");

    //version 5: emit each function in turn
    let mut labels = 0usize;
    for func in &program.functions {
        let symbols = &layout.functions[&func.name];
        gen_function(&mut asm, func, symbols, &mut labels);
    }

    asm
}

//version 5: emit one complete function
fn gen_function(asm: &mut String, func: &Function, symbols: &SymbolTable, labels: &mut usize) {
    asm.push_str(&format!(".globl {}\n", func.name));
    asm.push_str(&format!("{}:\n", func.name));

    // prologue: save the caller's frame pointer, set up ours
    asm.push_str("  push rbp\n");
    asm.push_str("  mov rbp, rsp\n");

    //version 3: carve out space for all the local variables up front. sema already
    //worked out how many bytes we need (16-byte aligned), one sub rsp covers
    //every slot we'll address as [rbp-offset]
    if symbols.stack_size > 0 {
        asm.push_str(&format!("  sub rsp, {}\n", symbols.stack_size));
    }

    //version 5: copy each incoming argument register into the slot sema assigned to that parameter
    for (i, param) in func.params.iter().enumerate() {
        let offset = symbols.offsets[&param.name];
        asm.push_str(&format!("  mov [rbp-{offset}], {}\n", ARG_REGS[i]));
    }

    //version 5: each function needs its own return label, otherwise two functions
    //would both define `.Lreturn` and the assembler would reject the duplicate
    let ret_label = format!(".Lreturn_{}", func.name);
    let mut depth = 0i64;

    for stmt in &func.body {
        gen_stmt(asm, stmt, symbols, labels, &mut depth, &ret_label);
    }

    // epilogue: every `return` jumps here after leaving its value in rax, we also
    // fall through to here if execution runs off the end of the body
    asm.push_str(&format!("{ret_label}:\n"));
    asm.push_str("  mov rsp, rbp\n");
    asm.push_str("  pop rbp\n");
    asm.push_str("  ret\n");
}

//version 3: emit assembly for a single statement
fn gen_stmt(
    asm: &mut String,
    stmt: &Stmt,
    symbols: &SymbolTable,
    labels: &mut usize,
    depth: &mut i64,
    ret_label: &str,
) {
    match stmt {
        Stmt::Return(expr) => {
            gen_expr(asm, expr, symbols, labels, depth);
            asm.push_str(&format!("  jmp {ret_label}\n"));
        }


        Stmt::Expr(expr) => {
            gen_expr(asm, expr, symbols, labels, depth);
        }

        // evaluate the initializer (defaulting to 0) into rax, then
        // store it into the variable's stack slot
        Stmt::Decl(decl) => {
            match &decl.init {
                Some(init) => gen_expr(asm, init, symbols, labels, depth),
                None => asm.push_str("  mov rax, 0\n"),
            }
            let offset = symbols.offsets[&decl.name];
            asm.push_str(&format!("  mov [rbp-{offset}], rax\n"));
        }

        //version 4: a block just emits its statements in order
        Stmt::Block(stmts) => {
            for s in stmts {
                gen_stmt(asm, s, symbols, labels, depth, ret_label);
            }
        }

        //version 4: if/else lowers to a conditional jump over the `then` branch
        //   <cond>            ; result in rax
        //   cmp rax, 0
        //   je  .Lelse        ; false -> skip the then-branch
        //   <then>
        //   jmp .Lend
        //  .Lelse:
        //   <else>
        //  .Lend:
        // when there's no else, we jump straight to .Lend on a false condition
        Stmt::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let else_label = next_label(labels);
            let end_label = next_label(labels);

            gen_expr(asm, cond, symbols, labels, depth);
            asm.push_str("  cmp rax, 0\n");

            match else_branch {
                Some(else_branch) => {
                    asm.push_str(&format!("  je .L{else_label}\n"));
                    gen_stmt(asm, then_branch, symbols, labels, depth, ret_label);
                    asm.push_str(&format!("  jmp .L{end_label}\n"));
                    asm.push_str(&format!(".L{else_label}:\n"));
                    gen_stmt(asm, else_branch, symbols, labels, depth, ret_label);
                    asm.push_str(&format!(".L{end_label}:\n"));
                }
                None => {
                    asm.push_str(&format!("  je .L{end_label}\n"));
                    gen_stmt(asm, then_branch, symbols, labels, depth, ret_label);
                    asm.push_str(&format!(".L{end_label}:\n"));
                }
            }
        }

        //version 4: while loops re-test the condition at the top and use a back
        //edge (`jmp .Lstart`) to repeat
        //  .Lstart:
        //   <cond>
        //   cmp rax, 0
        //   je  .Lend         ; exit when the condition is false
        //   <body>
        //   jmp .Lstart       ; back edge
        //  .Lend:
        Stmt::While { cond, body } => {
            let start_label = next_label(labels);
            let end_label = next_label(labels);

            asm.push_str(&format!(".L{start_label}:\n"));
            gen_expr(asm, cond, symbols, labels, depth);
            asm.push_str("  cmp rax, 0\n");
            asm.push_str(&format!("  je .L{end_label}\n"));
            gen_stmt(asm, body, symbols, labels, depth, ret_label);
            asm.push_str(&format!("  jmp .L{start_label}\n"));
            asm.push_str(&format!(".L{end_label}:\n"));
        }

        //version 4: a for loop is a while loop with an init before it and a step
        //appended to the bottom of the body.
        //   <init>
        //  .Lstart:
        //   <cond>            ; omitted clause => no test, loops forever
        //   cmp rax, 0
        //   je  .Lend
        //   <body>
        //   <step>
        //   jmp .Lstart
        //  .Lend:
        Stmt::For {
            init,
            cond,
            step,
            body,
        } => {
            let start_label = next_label(labels);
            let end_label = next_label(labels);

            if let Some(init) = init {
                gen_stmt(asm, init, symbols, labels, depth, ret_label);
            }

            asm.push_str(&format!(".L{start_label}:\n"));
            // an absent condition means "always true": skip the test entirely
            if let Some(cond) = cond {
                gen_expr(asm, cond, symbols, labels, depth);
                asm.push_str("  cmp rax, 0\n");
                asm.push_str(&format!("  je .L{end_label}\n"));
            }
            gen_stmt(asm, body, symbols, labels, depth, ret_label);
            if let Some(step) = step {
                gen_expr(asm, step, symbols, labels, depth);
            }
            asm.push_str(&format!("  jmp .L{start_label}\n"));
            asm.push_str(&format!(".L{end_label}:\n"));
        }
    }
}

//version 6: compute the address of an lvalue into rax (used by Addr and Assign)
fn gen_addr(asm: &mut String, expr: &Expr, symbols: &SymbolTable, labels: &mut usize, depth: &mut i64) {
    match expr {
        Expr::Var(name) => {
            let offset = symbols.offsets[name];
            asm.push_str(&format!("  lea rax, [rbp-{offset}]\n"));
        }
        // *p is an lvalue: the address is whatever p holds
        Expr::Unary(UnaryOp::Deref, inner) => {
            gen_expr(asm, inner, symbols, labels, depth);
        }
        _ => panic!("gen_addr called on non-lvalue"),
    }
}

//new function that recursively emits assembly for expression trees built by parser
fn gen_expr(asm: &mut String, expr: &Expr, symbols: &SymbolTable, labels: &mut usize, depth: &mut i64) {
    match expr {
        Expr::Int(value) => {
            asm.push_str(&format!("  mov rax, {value}\n"));
        }

        //version 3: read a variable by loading through its stack-slot address
        Expr::Var(name) => {
            let offset = symbols.offsets[name];
            asm.push_str(&format!("  lea rax, [rbp-{offset}]\n"));
            asm.push_str("  mov rax, [rax]\n");
        }

        //version 6: compute address of lhs, evaluate rhs, store through the address
        Expr::Assign(lhs, rhs) => {
            gen_addr(asm, lhs, symbols, labels, depth);
            push(asm, depth);
            gen_expr(asm, rhs, symbols, labels, depth);
            pop(asm, depth, "rdi");
            asm.push_str("  mov [rdi], rax\n");
        }

        Expr::Unary(op, inner) => match op {
            UnaryOp::Neg => {
                gen_expr(asm, inner, symbols, labels, depth);
                asm.push_str("  neg rax\n");
            }
            // &x: produce the address of x as a value
            UnaryOp::Addr => {
                gen_addr(asm, inner, symbols, labels, depth);
            }
            // *p: load through the pointer value
            UnaryOp::Deref => {
                gen_expr(asm, inner, symbols, labels, depth);
                asm.push_str("  mov rax, [rax]\n");
            }
        },

        Expr::Binary(op, lhs, rhs) => {
            gen_expr(asm, lhs, symbols, labels, depth);
            push(asm, depth);

            gen_expr(asm, rhs, symbols, labels, depth);
            pop(asm, depth, "rdi");
            // after this: rdi = left operand, rax = right operand

            match op {
                BinaryOp::Add => {
                    asm.push_str("  add rax, rdi\n");
                }

                BinaryOp::Sub => {
                    asm.push_str("  sub rdi, rax\n");
                    asm.push_str("  mov rax, rdi\n");
                }

                BinaryOp::Mul => {
                    asm.push_str("  imul rax, rdi\n");
                }

                BinaryOp::Div | BinaryOp::Mod => {
                    asm.push_str("  mov rcx, rax\n");
                    asm.push_str("  mov rax, rdi\n");
                    asm.push_str("  cqo\n");
                    asm.push_str("  idiv rcx\n");

                    if matches!(op, BinaryOp::Mod) {
                        asm.push_str("  mov rax, rdx\n");
                    }
                }

                //version 4: comparisons. we compare left(rdi) against right(rax),
                //then materialize the boolean flag as an integer 0/1 in rax
                BinaryOp::Eq
                | BinaryOp::Ne
                | BinaryOp::Lt
                | BinaryOp::Le
                | BinaryOp::Gt
                | BinaryOp::Ge => {
                    let setcc = match op {
                        BinaryOp::Eq => "sete",
                        BinaryOp::Ne => "setne",
                        BinaryOp::Lt => "setl",
                        BinaryOp::Le => "setle",
                        BinaryOp::Gt => "setg",
                        BinaryOp::Ge => "setge",
                        _ => unreachable!(),
                    };
                    asm.push_str("  cmp rdi, rax\n");
                    asm.push_str(&format!("  {setcc} al\n"));
                    asm.push_str("  movzx rax, al\n");
                }
            }
        }

        //version 4: short-circuiting `&&` / `||`
        Expr::Logical(op, lhs, rhs) => {
            let short_label = next_label(labels); // where we jump when the result is known early
            let end_label = next_label(labels);

            match op {
                LogicalOp::And => {
                    gen_expr(asm, lhs, symbols, labels, depth);
                    asm.push_str("  cmp rax, 0\n");
                    asm.push_str(&format!("  je .L{short_label}\n"));
                    gen_expr(asm, rhs, symbols, labels, depth);
                    asm.push_str("  cmp rax, 0\n");
                    asm.push_str(&format!("  je .L{short_label}\n"));
                    // both sides truthy -> 1
                    asm.push_str("  mov rax, 1\n");
                    asm.push_str(&format!("  jmp .L{end_label}\n"));
                    asm.push_str(&format!(".L{short_label}:\n"));
                    asm.push_str("  mov rax, 0\n");
                    asm.push_str(&format!(".L{end_label}:\n"));
                }

                LogicalOp::Or => {
                    gen_expr(asm, lhs, symbols, labels, depth);
                    asm.push_str("  cmp rax, 0\n");
                    asm.push_str(&format!("  jne .L{short_label}\n"));
                    gen_expr(asm, rhs, symbols, labels, depth);
                    asm.push_str("  cmp rax, 0\n");
                    asm.push_str(&format!("  jne .L{short_label}\n"));
                    asm.push_str("  mov rax, 0\n");
                    asm.push_str(&format!("  jmp .L{end_label}\n"));
                    asm.push_str(&format!(".L{short_label}:\n"));
                    asm.push_str("  mov rax, 1\n");
                    asm.push_str(&format!(".L{end_label}:\n"));
                }
            }
        }

        //version 5: a function call
        // 1. Evaluate each argument from left to right, pushing each result so a later
        //    argument's evaluation can't clobber an earlier one
        // 2. Pop them back into the argument registers,popping high-index-first 
        //    lands each value in the right register
        // 3. Align rsp to 16 before `call`, as the ABI requires, rsp is aligned if
        //    `depth` is even, so when it's odd we nudge rsp by 8 around the call
        Expr::Call(name, args) => {
            for arg in args {
                gen_expr(asm, arg, symbols, labels, depth);
                push(asm, depth);
            }
            for i in (0..args.len()).rev() {
                pop(asm, depth, ARG_REGS[i]);
            }

            if *depth % 2 != 0 {
                asm.push_str("  sub rsp, 8\n");
                asm.push_str(&format!("  call {name}\n"));
                asm.push_str("  add rsp, 8\n");
            } else {
                asm.push_str(&format!("  call {name}\n"));
            }
        }
    }
}
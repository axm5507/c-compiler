use crate::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};
use crate::sema::SymbolTable;

pub fn generate(program: &Program, symbols: &SymbolTable) -> String {
    let mut asm = String::new();
    asm.push_str(".intel_syntax noprefix\n");
    asm.push_str(".text\n");
    asm.push_str(&format!(".globl {}\n", program.function.name));
    asm.push_str(&format!("{}:\n", program.function.name));
    asm.push_str("  push rbp\n");
    asm.push_str("  mov rbp, rsp\n");

    //version 3: carve out space for all the local variables up front. sema already
    //worked out how many bytes we need (16-byte aligned), one sub rsp covers
    //every slot we'll address as [rbp-offset]
    if symbols.stack_size > 0 {
        asm.push_str(&format!("  sub rsp, {}\n", symbols.stack_size));
    }

    //version 3: the body is now a list of statements rather than a single return
    for stmt in &program.function.body {
        gen_stmt(&mut asm, stmt, symbols);
    }


    asm.push_str(".Lreturn:\n");//version 3 : one return point at the end of the function, all returns jump to it after leaving their value in rax
    asm.push_str("  mov rsp, rbp\n");
    asm.push_str("  pop rbp\n");
    asm.push_str("  ret\n");

    asm
}

//version 3: emit assembly for a single statement
fn gen_stmt(asm: &mut String, stmt: &Stmt, symbols: &SymbolTable) {
    match stmt {
        Stmt::Return(expr) => {
            gen_expr(asm, expr, symbols);
            asm.push_str("  jmp .Lreturn\n");
        }

        // a bare expression statement, evaluate it for its side
        // effect and throw away the result left in rax
        Stmt::Expr(expr) => {
            gen_expr(asm, expr, symbols);
        }

        // evaluate the initializer (defaulting to 0) into rax, then
        // store it into the variable's stack slot
        Stmt::Decl(decl) => {
            match &decl.init {
                Some(init) => gen_expr(asm, init, symbols),
                None => asm.push_str("  mov rax, 0\n"),
            }
            let offset = symbols.offsets[&decl.name];
            asm.push_str(&format!("  mov [rbp-{offset}], rax\n"));
        }
    }
}

//new function that recursively emits assembly for expression trees built by parser
fn gen_expr(asm: &mut String, expr: &Expr, symbols: &SymbolTable) {
    match expr {
        Expr::Int(value) => {
            asm.push_str(&format!("  mov rax, {value}\n"));
        }

        //version 3: read a variable by taking the address of its slot and loading through it 
        Expr::Var(name) => {
            let offset = symbols.offsets[name];
            asm.push_str(&format!("  lea rax, [rbp-{offset}]\n"));
            asm.push_str("  mov rax, [rax]\n");
        }

        //version 3: evaluate the right hand side into rax, then store it into the variable's slot
        Expr::Assign(name, value) => {
            gen_expr(asm, value, symbols);
            let offset = symbols.offsets[name];
            asm.push_str(&format!("  mov [rbp-{offset}], rax\n"));
        }

        Expr::Unary(UnaryOp::Neg, inner) => {
            gen_expr(asm, inner, symbols);
            asm.push_str("  neg rax\n");
        }

        Expr::Binary(op, lhs, rhs) => {
            gen_expr(asm, lhs, symbols);
            asm.push_str("  push rax\n");

            gen_expr(asm, rhs, symbols);
            asm.push_str("  pop rdi\n");

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
            }
        }
    }
}

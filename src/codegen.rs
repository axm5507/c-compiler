use crate::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};

pub fn generate(program: &Program) -> String {
    let mut asm = String::new();
    asm.push_str(".intel_syntax noprefix\n");
    asm.push_str(".text\n");
    asm.push_str(&format!(".globl {}\n", program.function.name));
    asm.push_str(&format!("{}:\n", program.function.name));
    asm.push_str("  push rbp\n");
    asm.push_str("  mov rbp, rsp\n");

    match &program.function.body {
        //Stmt::Return(Expr::Int(value)) => {
          //  asm.push_str(&format!("  mov rax, {}\n", value));
        //}
        //commenting this code out from version 1 to replac it with version 2 code
        Stmt::Return(expr) => {
            gen_expr(&mut asm, expr);
        }
    }

    asm.push_str("  mov rsp, rbp\n");
    asm.push_str("  pop rbp\n");
    asm.push_str("  ret\n");

    asm
}

//new function that recursively emits assembly for expression trees built by parser 
fn gen_expr(asm: &mut String, expr: &Expr) {
    match expr {
        Expr::Int(value) => {
            asm.push_str(&format!("  mov rax, {value}\n"));
        }

        Expr::Unary(UnaryOp::Neg, inner) => {
            gen_expr(asm, inner);
            asm.push_str("  neg rax\n");
        }

        Expr::Binary(op, lhs, rhs) => {
            gen_expr(asm, lhs);
            asm.push_str("  push rax\n");

            gen_expr(asm, rhs);
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

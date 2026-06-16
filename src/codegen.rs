pub fn generate(program: &Program) -> String {
    let mut asm = String::new();
    asm.push_str(".intel_syntax noprefix\n");
    asm.push_str(".text\n");
    asm.push_str(&format!(".globl {}\n", program.function.name));
    asm.push_str(&format!("{}:\n", program.function.name));
    asm.push_str("  push rbp\n");
    asm.push_str("  mov rbp, rsp\n");

    match &program.function.body {
        Stmt::Return(Expr::Int(value)) => {
            asm.push_str(&format!("  mov rax, {}\n", value));
        }
    }

    asm.push_str("  mov rsp, rbp\n");
    asm.push_str("  pop rbp\n");
    asm.push_str("  ret\n");

    asm
}
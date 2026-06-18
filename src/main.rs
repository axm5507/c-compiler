use std::process::ExitCode;

use educational_c_compiler::codegen;
use educational_c_compiler::lexer::Lexer;
use educational_c_compiler::parser::Parser;
use educational_c_compiler::sema;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);

    let source = match args.next() {
        // Read the program from a file path
        Some(path) => match std::fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(err) => {
                eprintln!("error: could not read '{path}': {err}");
                return ExitCode::FAILURE;
            }
        },
        // if there's nothing, it will fall back to a sample so `cargo run` works regardless
        None => {
            eprintln!("usage: ecc <source.c>   (no file given, compiling a sample)");
            "int main() { return 1 + 2 * 3; }".to_string()
        }
    };

    match compile(&source) {
        Ok(asm) => {
            print!("{asm}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn compile(source: &str) -> Result<String, String> {
    let tokens = Lexer::new(source).tokenize()?;
    let program = Parser::new(tokens).parse_program()?;
    // version 3: validate the program and lay out locals before generating code
    let symbols = sema::analyze(&program)?;
    Ok(codegen::generate(&program, &symbols))
}

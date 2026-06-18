//I wrote this for version 3
//this code does 2 jobs right now, validation and storage layout.
//When an undeclared variable is used, or a variable is declared twice in 
//one scope like int x = x, it will reject the program.
//Furthermore, it will assign every local variable a slot on the stack, and
//record it as an offset from rbp. Codegen later handles each variable as 
//[rbp - offset].

use std::collections::HashMap;

use crate::ast::{Expr, Program, Stmt};

// The result of analysis: where each variable lives and how much stack we need.
pub struct SymbolTable {
    pub offsets: HashMap<String, i64>,
    // total stack bytes to reserve for locals, rounded up to a 16-byte boundary
    // (the x86-64 System V ABI wants the stack 16 byte aligned)
    pub stack_size: i64,
}

struct Analyzer {
    //version 3: Right now the grammar only produces one scope, but the
    // stack is here so nested { } blocks can be added later without rework
    scopes: Vec<HashMap<String, i64>>,
    offsets: HashMap<String, i64>,
    next_offset: i64,
}

impl Analyzer {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            offsets: HashMap::new(),
            next_offset: 0,
        }
    }


    fn declare(&mut self, name: &str) -> Result<(), String> {
        let scope = self.scopes.last_mut().expect("there is always one scope");
        if scope.contains_key(name) {
            return Err(format!("duplicate declaration of variable '{name}'"));
        }

        self.next_offset += 8;
        let offset = self.next_offset;
        scope.insert(name.to_string(), offset);
        self.offsets.insert(name.to_string(), offset);
        Ok(())
    }

    // Look a name up from the innermost scope outward, error if never declared
    fn resolve(&self, name: &str) -> Result<i64, String> {
        for scope in self.scopes.iter().rev() {
            if let Some(&offset) = scope.get(name) {
                return Ok(offset);
            }
        }
        Err(format!("use of undeclared variable '{name}'"))
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Return(expr) => self.check_expr(expr),
            Stmt::Expr(expr) => self.check_expr(expr),
            Stmt::Decl(decl) => {
                if let Some(init) = &decl.init {
                    self.check_expr(init)?;
                }
                self.declare(&decl.name)
            }
        }
    }

    fn check_expr(&self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Int(_) => Ok(()),
            Expr::Var(name) => self.resolve(name).map(|_| ()),
            Expr::Assign(name, value) => {
                // both the target and the value have to be valid
                self.resolve(name)?;
                self.check_expr(value)
            }
            Expr::Binary(_, lhs, rhs) => {
                self.check_expr(lhs)?;
                self.check_expr(rhs)
            }
            Expr::Unary(_, inner) => self.check_expr(inner),
        }
    }
}

//validate the program and produce its variable layout
pub fn analyze(program: &Program) -> Result<SymbolTable, String> {
    let mut analyzer = Analyzer::new();

    for stmt in &program.function.body {
        analyzer.check_stmt(stmt)?;
    }

    //round raw total up to the next multiple of 16 for ABI alignment
    let stack_size = (analyzer.next_offset + 15) & !15;

    Ok(SymbolTable {
        offsets: analyzer.offsets,
        stack_size,
    })
}
//I wrote this for version 3
//this code does 2 jobs right now, validation and storage layout.
//When an undeclared variable is used, or a variable is declared twice in
//one scope like int x = x, it will reject the program.
//Furthermore, it will assign every local variable a slot on the stack, and
//record it as an offset from rbp. Codegen later handles each variable as
//[rbp - offset].
//
//version 5: I now made it so there can be there are now multiple functions. 
// We first collect every function's name and parameters so a call can refer to a
//function defined later in the file. Then we check each body in turn, with its
//parameters already declared as locals.

use std::collections::HashMap;

use crate::ast::{Expr, Program, Stmt};

//version 5: the System V AMD64 convention only passes the first 6 integer
//arguments in registers (rdi, rsi, rdx, rcx, r8, r9). I didn't implement the
//stack spill path for the 7th+, so I capped functions at 6 parameters.
// maybe that's something I can add in the future
const MAX_PARAMS: usize = 6;

// This is where each local lives and how much stack the frame needs
pub struct SymbolTable {
    pub offsets: HashMap<String, i64>,
    // total stack bytes to reserve for locals, rounded up to a 16-byte boundary
    // (the x86-64 System V ABI wants the stack 16 byte aligned)
    pub stack_size: i64,
}

//version 5: one SymbolTable per function, looked up by name
pub struct ProgramLayout {
    pub functions: HashMap<String, SymbolTable>,
}

struct Analyzer<'a> {
    //version 3: Right now the grammar only produces one scope, but the
    // stack is here so nested { } blocks can be added later without rework
    scopes: Vec<HashMap<String, i64>>,
    offsets: HashMap<String, i64>,
    next_offset: i64,
    //version 5: name -> parameter count for every function in the program, so we
    //can validate a function existing and having the right number of arguments 
    signatures: &'a HashMap<String, usize>,
}

impl<'a> Analyzer<'a> {
    fn new(signatures: &'a HashMap<String, usize>) -> Self {
        Self {
            scopes: vec![HashMap::new()],
            offsets: HashMap::new(),
            next_offset: 0,
            signatures,
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

            //version 4: a condition is just an expression (C has no separate bool
            //type, any int is truthy if it isnt 0), so we validate it like any other
            //expression and then recurse into the nested statements
            Stmt::Block(stmts) => {
                for s in stmts {
                    self.check_stmt(s)?;
                }
                Ok(())
            }
            Stmt::If {
                cond,
                then_branch,
                else_branch,
            } => {
                self.check_expr(cond)?;
                self.check_stmt(then_branch)?;
                if let Some(else_branch) = else_branch {
                    self.check_stmt(else_branch)?;
                }
                Ok(())
            }
            Stmt::While { cond, body } => {
                self.check_expr(cond)?;
                self.check_stmt(body)
            }
            Stmt::For {
                init,
                cond,
                step,
                body,
            } => {
                if let Some(init) = init {
                    self.check_stmt(init)?;
                }
                if let Some(cond) = cond {
                    self.check_expr(cond)?;
                }
                if let Some(step) = step {
                    self.check_expr(step)?;
                }
                self.check_stmt(body)
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
            //version 4: a logical operator just needs both operands to be valid
            Expr::Logical(_, lhs, rhs) => {
                self.check_expr(lhs)?;
                self.check_expr(rhs)
            }
            //version 5: a call must target a known function and pass exactly as many
            //arguments as that function declares
            // then every argument expression has to check out
            Expr::Call(name, args) => {
                match self.signatures.get(name) {
                    None => return Err(format!("call to undefined function '{name}'")),
                    Some(&arity) => {
                        if arity != args.len() {
                            return Err(format!(
                                "function '{name}' expects {arity} argument(s) but {} were given",
                                args.len()
                            ));
                        }
                    }
                }
                for arg in args {
                    self.check_expr(arg)?;
                }
                Ok(())
            }
        }
    }
}

//version 5: validate the whole program and produce a layout per function
pub fn analyze(program: &Program) -> Result<ProgramLayout, String> {
    // Pass 1: collect every signature up front, doing this before checking any
    // body is what lets a function call another that is defined later in the file
    // and also allows recursion
    let mut signatures = HashMap::new();
    for func in &program.functions {
        if func.params.len() > MAX_PARAMS {
            return Err(format!(
                "function '{}' has {} parameters; at most {MAX_PARAMS} are supported",
                func.name,
                func.params.len()
            ));
        }
        if signatures.insert(func.name.clone(), func.params.len()).is_some() {
            return Err(format!("duplicate definition of function '{}'", func.name));
        }
    }

    // Pass 2: check each function body and lay out its locals
    let mut functions = HashMap::new();
    for func in &program.functions {
        let mut analyzer = Analyzer::new(&signatures);

        // Parameters are locals too: declare them first so they get the lowest
        // stack slots which is the order codegen relies on when spilling the argument registers
        for param in &func.params {
            analyzer.declare(param)?;
        }

        for stmt in &func.body {
            analyzer.check_stmt(stmt)?;
        }

        //round raw total up to the next multiple of 16 for ABI alignment
        let stack_size = (analyzer.next_offset + 15) & !15;

        functions.insert(
            func.name.clone(),
            SymbolTable {
                offsets: analyzer.offsets,
                stack_size,
            },
        );
    }

    Ok(ProgramLayout { functions })
}
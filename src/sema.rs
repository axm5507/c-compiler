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

use crate::ast::{Expr, Program, Stmt, Type, UnaryOp,};

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

//version 6: struct for symbol
pub struct Symbol{
    offset: i64,
    ty: Type,
}

struct Analyzer<'a> {
    //version 3: Right now the grammar only produces one scope, but the
    // stack is here so nested { } blocks can be added later without rework
    scopes: Vec<HashMap<String, Symbol>>,
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

    fn declare(&mut self, name: &str, ty: Type) -> Result<(), String> {
        let scope = self.scopes.last_mut().expect("there is always one scope");
        if scope.contains_key(name) {
            return Err(format!("duplicate declaration of variable '{name}'"));
        }

        self.next_offset += 8;
        let offset = self.next_offset;
        scope.insert(name.to_string(), Symbol { offset, ty });
        self.offsets.insert(name.to_string(), offset);
        Ok(())
    }

    // Look a name up from the innermost scope outward, error if never declared
    fn resolve(&self, name: &str) -> Result<Symbol, String> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.get(name) {
                return Ok(*sym);
            }
        }
        Err(format!("use of undeclared variable '{name}'"))
    }
//version 6: adding lvalue checking
    fn is_lvalue(expr: &Expr) -> bool {
        match expr {
            Expr::Var(_) => true,
            Expr::Unary(UnaryOp::Deref, _) => true,
            _ => false,
        }
    }
    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Return(expr) => self.check_expr(expr),
            Stmt::Expr(expr) => self.check_expr(expr),
            Stmt::Decl(decl) => {
                if let Some(init) = &decl.init {
                    let init_ty = self.check_expr(init)?;
                    if init_ty != decl.ty {
                        return Err(format!(
                            "variable '{}' declared as {:?} but initialized with {:?}",
                            decl.name
                        ));
                    }
                }
                self.declare(&decl.name, decl.ty.clone())
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

    fn check_expr(&self, expr: &Expr) -> Result<Type, String> {
        match expr {
            Expr::Int(_) => Ok(Type::Int),
            Expr::Var(name) => {
                Ok(self.resolve(name)?.ty.clone())
            }
            Expr::Assign(lhs, rhs) => {
                // both the target and the value have to be valid
                if !Self::is_lvalue(lhs) {
                    return Err("assignment requires an lvalue on the left side".to_string());
                }
                let lhs_ty = self.check_expr(lhs)?;
                let rhs_ty = self.check_expr(rhs)?;
                if lhs_ty != rhs_ty {
                    return Err("assignment requires both sides to have the same type".to_string());
                }
                Ok(lhs_ty)
            }
            Expr::Binary(_, lhs, rhs) => {
                let lhs_ty = self.check_expr(lhs)?;
                let rhs_ty = self.check_expr(rhs)?;
                if lhs_ty != Type::Int || rhs_ty != Type::Int {
                    return Err("binary operator requires integer operands".to_string());
                }
                Ok(Type::Int)
            }
            Expr::Unary(op, inner) => {
                let inner_ty = self.check_expr(inner)?;
                match op {
                    UnaryOp::Neg => {
                        if inner_ty != Type::Int {
                            return Err("unary minus requires an integer operand".to_string());
                        }
                        Ok(Type::Int)
                    }
                    UnaryOp::Addr => {
                        //version 6: &x returns a pointer to x's type
                        if !Self::is_lvalue(inner) {
                            return Err("address-of requires an lvalue operand".to_string());
                        }
                        Ok(Type::Ptr(Box::new(inner_ty)))
                    }
                    UnaryOp::Deref => {
                        //version 6: *x requires x to be a pointer and returns the type it points to
                        match inner_ty {
                            Type::Ptr(inner) => Ok(*inner),
                            _ => Err("dereference requires a pointer operand".to_string()),
                        }
                    }
                }
            }
            //version 4: a logical operator just needs both operands to be valid
            Expr::Logical(_, lhs, rhs) => {
                self.check_expr(lhs)?;
                self.check_expr(rhs)?;
                Ok(Type::Int)
            }
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
                Ok(Type::Int)
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
            analyzer.declare(&param.name, param.ty.clone())?;
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

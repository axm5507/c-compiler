//version 3: validation and stack layout
//version 5: multi-function support via two passes (collect signatures, then check bodies)
//version 6: type tracking for pointers (lvalue checks, & and * operators)
//version 7: struct field offsets and array sizes, validate Index/Field expressions

use std::collections::HashMap;

use crate::ast::{Expr, Program, Stmt, StructDecl, Type, UnaryOp};

//version 5: the System V AMD64 convention only passes the first 6 integer
//arguments in registers (rdi, rsi, rdx, rcx, r8, r9). I didn't implement the
//stack spill path for the 7th+, so I capped functions at 6 parameters.
const MAX_PARAMS: usize = 6;

//version 7: one entry per field: its name, byte offset inside the struct, and type
pub struct FieldEntry {
    pub name: String,
    pub offset: i64,
    pub ty: Type,
}

//version 7: the computed layout for a single struct definition
pub struct StructLayout {
    pub fields: Vec<FieldEntry>,
    pub size: i64,
}


// This is where each local lives and how much stack the frame needs
pub struct SymbolTable {
    pub offsets: HashMap<String, i64>,
    //version 7: type of every local/parameter, needed by codegen to distinguish
    //scalars from aggregates
    pub types: HashMap<String, Type>,
    // total stack bytes to reserve for locals, rounded up to a 16-byte boundary
    pub stack_size: i64,
}

//version 5: one SymbolTable per function, looked up by name
//version 7: also carries the struct layouts that codegen needs
pub struct ProgramLayout {
    pub functions: HashMap<String, SymbolTable>,
    pub structs: HashMap<String, StructLayout>,
}


//version 6: struct for symbol
#[derive(Clone)]
pub struct Symbol {
    pub offset: i64,
    pub ty: Type,
}


//version 7: compute the number of bytes a type occupies on the stack
//Every scalar (int, pointer) is 8 bytes because we always use 64-bit slots
//Arrays multiply element size by count, structs look up their pre-computed size
fn type_size(ty: &Type, structs: &HashMap<String, StructLayout>) -> Result<i64, String> {
    match ty {
        Type::Int | Type::Ptr(_) => Ok(8),
        Type::Array(elem, n) => Ok((*n as i64) * type_size(elem, structs)?),
        Type::Struct(name) => structs
            .get(name)
            .map(|l| l.size)
            .ok_or_else(|| format!("unknown struct type 'struct {name}'")),
    }
}

//version 7: this will build the StructLayout for one struct declaration
fn build_struct_layout(
    sd: &StructDecl,
    known: &HashMap<String, StructLayout>,
) -> Result<StructLayout, String> {
    let mut offset = 0i64;
    let mut fields = Vec::new();

    for field in &sd.fields {
        let size = type_size(&field.ty, known)?;
        fields.push(FieldEntry {
            name: field.name.clone(),
            offset,
            ty: field.ty.clone(),
        });
        offset += size;
    }

    // Round the total size up to an 8-byte boundary 
    let size = (offset + 7) & !7;
    Ok(StructLayout { fields, size })
}


struct Analyzer<'a> {
    //version 3: Right now the grammar only produces one scope, but the
    // stack is here so nested { } blocks can be added later without rework
    scopes: Vec<HashMap<String, Symbol>>,
    offsets: HashMap<String, i64>,
    //version 7: type of each declared variable, written into SymbolTable at the end
    types: HashMap<String, Type>,
    next_offset: i64,
    //version 5: name -> parameter count for every function in the program, so we
    //can validate a function existing and having the right number of arguments
    signatures: &'a HashMap<String, usize>,
    //version 7: struct layouts built in the pre-pass, needed to compute variable sizes
    struct_layouts: &'a HashMap<String, StructLayout>,
}

impl<'a> Analyzer<'a> {
    fn new(
        signatures: &'a HashMap<String, usize>,
        struct_layouts: &'a HashMap<String, StructLayout>,
    ) -> Self {
        Self {
            scopes: vec![HashMap::new()],
            offsets: HashMap::new(),
            types: HashMap::new(),
            next_offset: 0,
            signatures,
            struct_layouts,
        }
    }

    fn declare(&mut self, name: &str, ty: Type) -> Result<(), String> {
        let scope = self.scopes.last_mut().expect("there is always one scope");
        if scope.contains_key(name) {
            return Err(format!("duplicate declaration of variable '{name}'"));
        }

        //version 7: allocate exactly as many bytes as the type requires
        //For `int xs[3]` this is 24 bytes, for a scalar it's still 8
        let size = type_size(&ty, self.struct_layouts)?;
        self.next_offset += size;
        let offset = self.next_offset;

        scope.insert(name.to_string(), Symbol { offset, ty: ty.clone() });
        self.offsets.insert(name.to_string(), offset);
        self.types.insert(name.to_string(), ty);
        Ok(())
    }

    // Look a name up from the innermost scope outward, error if never declared
    fn resolve(&self, name: &str) -> Result<Symbol, String> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.get(name) {
                return Ok(sym.clone());
            }
        }
        Err(format!("use of undeclared variable '{name}'"))
    }

    //version 6: lvalue checking
    //version 7: array subscript and field access are also lvalues
    fn is_lvalue(expr: &Expr) -> bool {
        matches!(
            expr,
            Expr::Var(_) | Expr::Unary(UnaryOp::Deref, _) | Expr::Index(_, _) | Expr::Field(_, _)
        )
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Return(expr) => {
                self.check_expr(expr)?;
                Ok(())
            }
            Stmt::Expr(expr) => {
                self.check_expr(expr)?;
                Ok(())
            }
            Stmt::Decl(decl) => {
                if let Some(init) = &decl.init {
                    let init_ty = self.check_expr(init)?;
                    //version 7: aggregate types cannot be initialized with an expressionn
                    match &decl.ty {
                        Type::Array(_, _) | Type::Struct(_) => {
                            return Err(format!(
                                "aggregate variable '{}' cannot be initialized with an expression; \
                                 declare it first and assign individual elements",
                                decl.name
                            ));
                        }
                        _ => {
                            if init_ty != decl.ty {
                                return Err(format!(
                                    "variable '{}' declared as {:?} but initialized with {:?}",
                                    decl.name, decl.ty, init_ty
                                ));
                            }
                        }
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

            Expr::Var(name) => Ok(self.resolve(name)?.ty.clone()),

            Expr::Assign(lhs, rhs) => {
                if !Self::is_lvalue(lhs) {
                    return Err("assignment requires an lvalue on the left side".to_string());
                }
                let lhs_ty = self.check_expr(lhs)?;
                let rhs_ty = self.check_expr(rhs)?;
                //version 7: forbid assigning to whole arrays/structs
                match &lhs_ty {
                    Type::Array(_, _) | Type::Struct(_) => {
                        return Err(format!(
                            "cannot assign to an aggregate type directly; \
                             assign individual elements or fields instead"
                        ));
                    }
                    _ => {}
                }
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

            //version 7: arr must be an array or pointer type; result is the element type
            //Arrays decay automatically, pointer indexing also works
            Expr::Index(arr, idx) => {
                let arr_ty = self.check_expr(arr)?;
                let elem_ty = match arr_ty {
                    Type::Array(elem, _) => *elem,
                    Type::Ptr(elem) => *elem,
                    other => {
                        return Err(format!(
                            "index operator `[]` requires an array or pointer operand, got {other:?}"
                        ));
                    }
                };
                let idx_ty = self.check_expr(idx)?;
                if idx_ty != Type::Int {
                    return Err("array index must be an integer".to_string());
                }
                Ok(elem_ty)
            }

            //version 7: obj must be a struct type; result is the field's type
            Expr::Field(obj, field_name) => {
                let obj_ty = self.check_expr(obj)?;
                match obj_ty {
                    Type::Struct(ref sname) => {
                        let layout = self.struct_layouts.get(sname).ok_or_else(|| {
                            format!("struct '{sname}' is used but has no declared definition")
                        })?;
                        let entry = layout
                            .fields
                            .iter()
                            .find(|f| f.name == *field_name)
                            .ok_or_else(|| {
                                format!("struct '{sname}' has no field named '{field_name}'")
                            })?;
                        Ok(entry.ty.clone())
                    }
                    other => Err(format!(
                        "field access `.{field_name}` requires a struct operand, got {other:?}"
                    )),
                }
            }
        }
    }
}

//version 5: validate the whole program and produce a layout per function
//version 7: also build and return struct layouts
pub fn analyze(program: &Program) -> Result<ProgramLayout, String> {
    //version 7: Pass 0 - build struct layouts before anything else 
    //Process structs in declaration order
    let mut struct_layouts: HashMap<String, StructLayout> = HashMap::new();
    for sd in &program.structs {
        if struct_layouts.contains_key(&sd.name) {
            return Err(format!("duplicate definition of struct '{}'", sd.name));
        }
        let layout = build_struct_layout(sd, &struct_layouts)?;
        struct_layouts.insert(sd.name.clone(), layout);
    }

    // Pass 1: collect every function signature up front so a function can call
    // another that is defined later in the file (and to allow recursion)
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
        let mut analyzer = Analyzer::new(&signatures, &struct_layouts);

        // Parameters are locals too: declare them first so they get the lowest
        // stack slots (the order codegen relies on when spilling the argument registers)
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
                types: analyzer.types,
                stack_size,
            },
        );
    }

    Ok(ProgramLayout {
        functions,
        structs: struct_layouts,
    })
}

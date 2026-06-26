#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    //version 7: programs can now have struct declarations before functions
    pub structs: Vec<StructDecl>,
    pub functions: Vec<Function>,
}

//version 7: struct declaration
#[derive(Debug, Clone, PartialEq)]
pub struct StructDecl {
    pub name: String,
    pub fields: Vec<FieldDecl>,
}

//version 7: single field inside a struct
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDecl {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    //version 5: now funcs need params, so this is for
    // parameter names in order, their position decides
    //which incoming argument register they map to
    pub params: Vec<Param>,
    //version 3: a function body is now a sequence of statements, not just one return
    //this lets us write things like `int x = 3; return x;`
    pub body: Vec<Stmt>,
}

//version 6: add pub struct for param
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Return(Expr),
    //version 3: a local variable declaration, like int x = 3; or int y;
    Decl(VarDecl),
    Expr(Expr),
    //version 4: structured control flow
    Block(Vec<Stmt>),
    //if then else branches are boxed because a Stmt
    //can recursively contain more statements
    If {
        cond: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    //`while (cond) body`
    While {
        cond: Expr,
        body: Box<Stmt>,
    },
    //`for (init; cond; step) body`
    For {
        init: Option<Box<Stmt>>,
        cond: Option<Expr>,
        step: Option<Expr>,
        body: Box<Stmt>,
    },
}

//version 3: everything we need to know about a declared local variable.
#[derive(Debug, Clone, PartialEq)]
pub struct VarDecl {
    pub name: String,
    //version 6: adding type
    pub ty: Type,
    pub init: Option<Expr>,
}

//version 6: Adding type for pointers
//version 7: Adding Array and Struct types
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Ptr(Box<Type>),
    //version 7: a fixed size array with usize as element count
    Array(Box<Type>, usize),
    //version 7: a named struct type
    Struct(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64),
    //version 2: now I'm gonna add unary and binary expressions so the compiler can do math/PEMDAS
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    //version 3: reading a variable by name
    Var(String),
    //version 3: assigning to a variable
    //version 6: changing to include type of variable being assigned to
    Assign(Box<Expr>, Box<Expr>),
    //version 4: short-circuiting `&&` / `||`
    //what that means is the right side doesn't always have to be evaluated
    Logical(LogicalOp, Box<Expr>, Box<Expr>),
    //version 5: a function call, the result comes back in rax
    Call(String, Vec<Expr>),
    //version 7: array subscript (x[i])
    Index(Box<Expr>, Box<Expr>),
    //version 7: struct field access (x.field)
    Field(Box<Expr>, String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    //version 4: comparison operators
    Eq, // go to lexer to see what each of these means
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    //version 6: &x and *x
    Addr,
    Deref,
}

//version 4: the two short circuiting logical operators
#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOp {
    And,
    Or,
}

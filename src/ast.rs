#[derive(Debug, Clone, PartialEq)]
pub struct Program{
    pub function: Function,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function{
    pub name: String,
    //version 3: a function body is now a sequence of statements, not just one return
    //this lets us write things like `int x = 3; return x;`
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt{
    Return(Expr),
    //version 3: a local variable declaration, like `int x = 3;` or `int y;`
    Decl(VarDecl),
    Expr(Expr),
}

//version 3: everything we need to know about a declared local variable.
#[derive(Debug, Clone, PartialEq)]
pub struct VarDecl{
    pub name: String,
    pub init: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr{
    Int(i64),
    //commit 2: now I'm gonna add unary and binary expressions so the compiler can do math/PEMDAS
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    //version 3: reading a variable by name
    Var(String),
    //version 3: assigning to a variable
    Assign(String, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp{
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp{
    Neg,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program{
    pub function: Function,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function{
    pub name: String,
    pub body: Stmt,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt{
    Return(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr{
    Int(i64),
    //commit 2: now I'm gonna add unary and binary expressions so the compiler can do math/PEMDAS
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
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

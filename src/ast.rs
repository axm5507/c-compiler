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
}
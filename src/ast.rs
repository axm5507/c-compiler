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
    //version 3: a local variable declaration, like int x = 3; or int y;
    Decl(VarDecl),
    Expr(Expr),
    //version 4: structured control flow
    Block(Vec<Stmt>),
    //`if (cond) then_branch [else else_branch]`. branches are boxed because a Stmt
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
    //version 4: short-circuiting `&&` / `||`
    //what that means is the right side doesn't always have to be evaluated
    Logical(LogicalOp, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp{
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
pub enum UnaryOp{
    Neg,
}

//version 4: the two short circuiting logical operators
#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOp{
    And, 
    Or,  
}

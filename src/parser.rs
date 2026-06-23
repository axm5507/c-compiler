use crate::ast::{BinaryOp, Expr, Function, LogicalOp, Program, Stmt, UnaryOp, VarDecl};
use crate::lexer::{Token, TokenKind};

pub struct Parser{
    tokens: Vec<Token>,
    pos: usize,
}

//will change this from being so strict and only accepting one thing to accepting a bunch of stuff later
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    //version 5: a program is now zero or more functions one after another up to EOF
    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut functions = Vec::new();
        while !self.at(TokenKind::Eof) {
            functions.push(self.parse_function()?);
        }
        Ok(Program { functions })
    }

    //version 5: int name(params) { body }
    fn parse_function(&mut self) -> Result<Function, String> {
        self.expect(TokenKind::IntKw)?;
        let name = self.expect_ident()?;

        self.expect(TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen)?;

        self.expect(TokenKind::LBrace)?;

        //version 3: parse a sequence of statements until the closing brace, instead of
        //assuming the body is exactly one return statement
        let mut body = Vec::new();
        while !self.at(TokenKind::RBrace) {
            body.push(self.parse_stmt()?);
        }

        self.expect(TokenKind::RBrace)?;

        Ok(Function { name, params, body })
    }

    //version 5: a comma separated list of int name parameters(could be empty)
    fn parse_params(&mut self) -> Result<Vec<String>, String> {
        let mut params = Vec::new();
        if self.at(TokenKind::RParen) {
            return Ok(params); //no parameters
        }
        loop {
            self.expect(TokenKind::IntKw)?; // every parameter is typed int
            params.push(self.expect_ident()?);
            if !self.consume(TokenKind::Comma) {
                break;
            }
        }
        Ok(params)
    }

    //version 3: decide which kind of statement we're looking at and parse it
    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        if self.at(TokenKind::IntKw) {
            return self.parse_decl();
        }

        //version 4: control-flow statements get their own dedicated parsers
        if self.at(TokenKind::IfKw) {
            return self.parse_if();
        }
        if self.at(TokenKind::WhileKw) {
            return self.parse_while();
        }
        if self.at(TokenKind::ForKw) {
            return self.parse_for();
        }
        //version 4: a `{ ... }` block nested inside the function body
        if self.at(TokenKind::LBrace) {
            return self.parse_block();
        }

        // return <expr>;
        if self.consume(TokenKind::ReturnKw) {
            let expr = self.parse_expr()?;
            self.expect(TokenKind::Semi)?;
            return Ok(Stmt::Return(expr));
        }

        // otherwise it's an expression statement
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Semi)?;
        Ok(Stmt::Expr(expr))
    }

    //version 4: parse statements until the matching closing brace
    fn parse_block(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        while !self.at(TokenKind::RBrace) {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(Stmt::Block(stmts))
    }

    //version 4: the `else` binds to the nearest `if`
    //automatically because each branch is just one parsed statement
    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::IfKw)?;
        self.expect(TokenKind::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(TokenKind::RParen)?;

        let then_branch = Box::new(self.parse_stmt()?);

        let else_branch = if self.consume(TokenKind::ElseKw) {
            Some(Box::new(self.parse_stmt()?))
        } else {
            None
        };

        Ok(Stmt::If {
            cond,
            then_branch,
            else_branch,
        })
    }

    //version 4: `while (cond) stmt`
    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::WhileKw)?;
        self.expect(TokenKind::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(TokenKind::RParen)?;
        let body = Box::new(self.parse_stmt()?);
        Ok(Stmt::While { cond, body })
    }

    //version 4: for loop parsing
    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::ForKw)?;
        self.expect(TokenKind::LParen)?;

        // init clause: either a declaration, an expression, or nothing
        let init = if self.at(TokenKind::Semi) {
            self.expect(TokenKind::Semi)?;
            None
        } else if self.at(TokenKind::IntKw) {
            Some(Box::new(self.parse_decl()?))
        } else {
            let expr = self.parse_expr()?;
            self.expect(TokenKind::Semi)?;
            Some(Box::new(Stmt::Expr(expr)))
        };

        // condition clause: optional expression, then `;`
        let cond = if self.at(TokenKind::Semi) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        self.expect(TokenKind::Semi)?;

        // step clause: optional expression, then the closing `)`
        let step = if self.at(TokenKind::RParen) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        self.expect(TokenKind::RParen)?;

        let body = Box::new(self.parse_stmt()?);
        Ok(Stmt::For {
            init,
            cond,
            step,
            body,
        })
    }

    //version 3: parsing a local variable declaration
    fn parse_decl(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::IntKw)?;
        let name = self.expect_ident()?;

        let init = if self.consume(TokenKind::Assign) {
            Some(self.parse_expr()?)
        } else {
            None
        };

        self.expect(TokenKind::Semi)?;
        Ok(Stmt::Decl(VarDecl { name, init }))
    }

    //now adding the parsing of different mathematical stuff for version 2
    fn parse_expr(&mut self) -> Result<Expr, String>{
        //version 3: assignment sits at the bottom (lowest precedence) of the expression grammar
        self.parse_assign()
    }

    //version 3: assignment is lower precedence than math stuff
    //we first parse an ordinary expression; if it's followed by equal sign,
    //the thing we just parsed must be a plain variable name
    fn parse_assign(&mut self) -> Result<Expr, String>{
        //version 4: below assignment sits the whole comparison/logical ladder.
        let expr = self.parse_logical_or()?;

        if self.consume(TokenKind::Assign) {
            let value = self.parse_assign()?;
            if let Expr::Var(name) = expr {
                return Ok(Expr::Assign(name, Box::new(value)));
            }
            return Err(self.error_here("invalid assignment target (left side must be a variable)"));
        }

        Ok(expr)
    }

    //version 4: precedence ladder, lowest binding first. each level parses the
    //next-tighter level and then loops while it sees an operator at its own level,
    //so operators of equal precedence group left to right
    //
    //   `||`  <  `&&`  <  `== !=`  <  `< <= > >=`  <  `+ -`  <  `* / %`  <  unary
    fn parse_logical_or(&mut self) -> Result<Expr, String>{
        let mut expr = self.parse_logical_and()?;
        while self.consume(TokenKind::OrOr) {
            expr = Expr::Logical(
                LogicalOp::Or,
                Box::new(expr),
                Box::new(self.parse_logical_and()?),
            );
        }
        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Expr, String>{
        let mut expr = self.parse_equality()?;
        while self.consume(TokenKind::AndAnd) {
            expr = Expr::Logical(
                LogicalOp::And,
                Box::new(expr),
                Box::new(self.parse_equality()?),
            );
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr, String>{
        let mut expr = self.parse_relational()?;
        loop {
            let op = if self.consume(TokenKind::EqEq) {
                BinaryOp::Eq
            } else if self.consume(TokenKind::Ne) {
                BinaryOp::Ne
            } else {
                break;
            };
            expr = Expr::Binary(op, Box::new(expr), Box::new(self.parse_relational()?));
        }
        Ok(expr)
    }

    fn parse_relational(&mut self) -> Result<Expr, String>{
        let mut expr = self.parse_additive()?;
        loop {
            let op = if self.consume(TokenKind::Lt) {
                BinaryOp::Lt
            } else if self.consume(TokenKind::Le) {
                BinaryOp::Le
            } else if self.consume(TokenKind::Gt) {
                BinaryOp::Gt
            } else if self.consume(TokenKind::Ge) {
                BinaryOp::Ge
            } else {
                break;
            };
            expr = Expr::Binary(op, Box::new(expr), Box::new(self.parse_additive()?));
        }
        Ok(expr)
    }
    // this is for addition and subtraction
    fn parse_additive(&mut self) -> Result<Expr, String>{
        let mut expr = self.parse_multiplicative()?;

        loop {
            if self.consume(TokenKind::Plus) {
                expr = Expr::Binary(
                    BinaryOp::Add,
                    Box::new(expr),
                    Box::new(self.parse_multiplicative()?),
            );
        }     else if self.consume(TokenKind::Minus) {
                expr = Expr::Binary(
                    BinaryOp::Sub,
                    Box::new(expr),
                    Box::new(self.parse_multiplicative()?),
            );
        }     else {
                break;
        }
    }

        Ok(expr)
    }
    //for multiplication, division, and modulus
    fn parse_multiplicative(&mut self) -> Result<Expr, String>{
        let mut expr = self.parse_unary()?;
        loop{
            if self.consume(TokenKind::Star){
                expr = Expr::Binary(
                    BinaryOp::Mul,
                    Box::new(expr),
                    Box::new(self.parse_unary()?),
                    );
            }
            else if self.consume(TokenKind::Slash){
                expr = Expr::Binary(
                    BinaryOp::Div,
                    Box::new(expr),
                    Box::new(self.parse_unary()?),
                    );
            }
            else if self.consume(TokenKind::Percent){
                expr = Expr::Binary(
                    BinaryOp::Mod,
                    Box::new(expr),
                    Box::new(self.parse_unary()?),
                    );
            }
            else{
                break;
            }
        }
        Ok(expr)
    
    }
    //for negative nums
    fn parse_unary(&mut self) -> Result<Expr, String>{
        if self.consume(TokenKind::Minus){
            return Ok(Expr::Unary(
                UnaryOp::Neg,
                Box::new(self.parse_unary()?),
                ));
        }
        self.parse_primary()
    }
    //for numbers and parenthesis
    fn parse_primary(&mut self) -> Result<Expr, String>{
        if self.consume(TokenKind::LParen){
            let expr = self.parse_expr()?;
            self.expect(TokenKind::RParen)?;
            return Ok(expr);
        }
        if self.at(TokenKind::Number){
            return Ok(Expr::Int(self.expect_number()?));
        }
        //version 3: a bare identifier is a variable read, for example `x`
        //version 5: unless it's immediately followed by `(`, which makes it a call
        if self.at(TokenKind::Ident){
            let name = self.expect_ident()?;
            if self.consume(TokenKind::LParen) {
                let args = self.parse_args()?;
                self.expect(TokenKind::RParen)?;
                return Ok(Expr::Call(name, args));
            }
            return Ok(Expr::Var(name));
        }
        Err(self.error_here("expected expression"))
    }

    //version 5: a comma separated list of argument expressions(could be empty)
    fn parse_args(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        if self.at(TokenKind::RParen) {
            return Ok(args); //no arguments
        }
        loop {
            args.push(self.parse_expr()?);
            if !self.consume(TokenKind::Comma) {
                break;
            }
        }
        Ok(args)
    }

    // ---- token-stream helpers ----

    fn peek(&self) -> &Token {
        // There is always an Eof token at the end, so indexing the clamped
        // position is safe even once we've consumed everything
        let idx = self.pos.min(self.tokens.len() - 1);
        &self.tokens[idx]
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    // Advance past the current token and hand it back
    fn advance(&mut self) -> Token {
        let tok = self.peek().clone();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    // If the current token matches `kind`, consume it and return true
    fn consume(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    // Consume a token of the expected kind or produce an error
    fn expect(&mut self, kind: TokenKind) -> Result<Token, String> {
        if self.at(kind.clone()) {
            Ok(self.advance())
        } else {
            Err(self.error_here(&format!("expected {kind:?}")))
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        let tok = self.expect(TokenKind::Ident)?;
        Ok(tok.lexeme)
    }

    fn expect_number(&mut self) -> Result<i64, String> {
        let tok = self.expect(TokenKind::Number)?;
        tok.lexeme
            .parse::<i64>()
            .map_err(|_| format!("invalid integer literal '{}'", tok.lexeme))
    }

    fn error_here(&self, msg: &str) -> String {
        let tok = self.peek();
        format!("{} at byte {} (found {:?})", msg, tok.pos, tok.kind)
    }
}

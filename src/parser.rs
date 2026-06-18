use crate::ast::{BinaryOp, Expr, Function, Program, Stmt, UnaryOp, VarDecl};
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

    pub fn parse_program(&mut self) -> Result<Program, String> {
        self.expect(TokenKind::IntKw)?;
        let name = self.expect_ident()?;

        self.expect(TokenKind::LParen)?;
        self.expect(TokenKind::RParen)?;

        self.expect(TokenKind::LBrace)?;

        //version 3: parse a sequence of statements until the closing brace, instead of
        //assuming the body is exactly one return statement
        let mut body = Vec::new();
        while !self.at(TokenKind::RBrace) {
            body.push(self.parse_stmt()?);
        }

        self.expect(TokenKind::RBrace)?;
        self.expect(TokenKind::Eof)?;

        Ok(Program {
            function: Function { name, body },
        })
    }

    //version 3: decide which kind of statement we're looking at and parse it
    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        if self.at(TokenKind::IntKw) {
            return self.parse_decl();
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
        let expr = self.parse_additive()?;

        if self.consume(TokenKind::Assign) {
            let value = self.parse_assign()?;
            if let Expr::Var(name) = expr {
                return Ok(Expr::Assign(name, Box::new(value)));
            }
            return Err(self.error_here("invalid assignment target (left side must be a variable)"));
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
        //version 3: a bare identifier is a variable read, e.g. `x`
        if self.at(TokenKind::Ident){
            return Ok(Expr::Var(self.expect_ident()?));
        }
        Err(self.error_here("expected expression"))
    }

    // ---- token-stream helpers ----

    fn peek(&self) -> &Token {
        // There is always an Eof token at the end, so indexing the clamped
        // position is safe even once we've consumed everything.
        let idx = self.pos.min(self.tokens.len() - 1);
        &self.tokens[idx]
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    // Advance past the current token and hand it back.
    fn advance(&mut self) -> Token {
        let tok = self.peek().clone();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    // If the current token matches `kind`, consume it and return true.
    fn consume(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    // Consume a token of the expected kind or produce an error.
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

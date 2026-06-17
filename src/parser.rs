pub struct Parser{
    tokens: Vec<Token>,
    pos: usize,
}

//will change this from being so strict and only accepting one thing to accepting a bunch of stuff later
impl Parser {
    pub fn parse_program(&mut self) -> Result<Program, String> {
        self.expect(TokenKind::IntKw)?;
        let name = self.expect_ident()?;

        self.expect(TokenKind::LParen)?;
        self.expect(TokenKind::RParen)?;

        self.expect(TokenKind::LBrace)?;
        self.expect(TokenKind::ReturnKw)?;
        
        //let value = self.expect_number()?;
        //replacing this for version 2
        let expr = self.parse_expr()?;

        self.expect(TokenKind::Semi)?;
        self.expect(TokenKind::RBrace)?;
        self.expect(TokenKind::Eof)?;

        Ok(Program {
            function: Function {
                name,
                //body: Stmt::Return(Expr::Int(value)),
                //I'm also replacing this for version 2
                body: Stmt::Return(expr),
            },
        })
    }
    //now adding the parsing of different mathematical stuff for version 2
    fn parse_expr(&mut self) -> Result<Expr, String>{
        self.parse_additive()
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
        Err(self.error_here("expected expression"))
    }
}

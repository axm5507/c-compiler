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

        let value = self.expect_number()?;

        self.expect(TokenKind::Semi)?;
        self.expect(TokenKind::RBrace)?;
        self.expect(TokenKind::Eof)?;

        Ok(Program {
            function: Function {
                name,
                body: Stmt::Return(Expr::Int(value)),
            },
        })
    }
}
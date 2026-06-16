#[derive(Debug, Clone, PartialEq)]
pub struct Token{
    pub kind: TokenKind,
    pub lexeme: String,
    pub pos: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind{
    IntKw,
    ReturnKw,
    Ident,
    Number,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semi,
    Eof,
}
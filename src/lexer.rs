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
    //version 2 edits
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    //version 3: just adding `=`
    Assign,
    //version 4: control flow keywords
    IfKw,
    ElseKw,
    WhileKw,
    ForKw,
    //version 4: comparison operators
    EqEq, // ==
    Ne,   // !=
    Lt,   // <
    Le,   // <=
    Gt,   // >
    Ge,   // >=
    //version 4: short circuiting logical operators
    AndAnd, // &&
    OrOr,   // ||
}

//version 2 additions
pub struct Lexer<'a>{
    source: &'a str,
    chars: Vec<char>,
    pos: usize,
}
impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().collect(),
            pos: 0,
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek() {
            match ch {
                c if c.is_whitespace() => {
                    self.bump();
                }

                //version 4: skip `//` line comments so example files can be
                //commented on. this code looks ahead to distinguish `//` from a lone
                //`/` (division)
                '/' if self.chars.get(self.pos + 1) == Some(&'/') => {
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.bump();
                    }
                }

                c if c.is_ascii_digit() => {
                    tokens.push(self.number());
                }

                c if is_ident_start(c) => {
                    tokens.push(self.ident_or_keyword());
                }

                _ => {
                    tokens.push(self.symbol()?);
                }
            }
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            lexeme: String::new(),
            pos: self.source.len(),
        });

        Ok(tokens)
    }

    fn number(&mut self) -> Token {
        let start = self.pos;

        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.bump();
        }

        Token {
            kind: TokenKind::Number,
            lexeme: self.slice(start),
            pos: start,
        }
    }

    fn ident_or_keyword(&mut self) -> Token {
        let start = self.pos;

        while matches!(self.peek(), Some(c) if is_ident_continue(c)) {
            self.bump();
        }

        let lexeme = self.slice(start);

        let kind = match lexeme.as_str() {
            "int" => TokenKind::IntKw,
            "return" => TokenKind::ReturnKw,
            //version 4: control flow keywords
            "if" => TokenKind::IfKw,
            "else" => TokenKind::ElseKw,
            "while" => TokenKind::WhileKw,
            "for" => TokenKind::ForKw,
            _ => TokenKind::Ident,
        };

        Token {
            kind,
            lexeme,
            pos: start,
        }
    }

    fn symbol(&mut self) -> Result<Token, String> {
        let start = self.pos;

        let kind = match self.bump().unwrap() {
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,

            //version 4: some operators are two characters long. after bump() the
            //cursor sits on the second character, so we peek and if it forms a
            //two char operator, bump() again to consume it
            '=' if self.peek() == Some('=') => {
                self.bump();
                TokenKind::EqEq
            }
            '=' => TokenKind::Assign,

            '!' if self.peek() == Some('=') => {
                self.bump();
                TokenKind::Ne
            }

            '<' if self.peek() == Some('=') => {
                self.bump();
                TokenKind::Le
            }
            '<' => TokenKind::Lt,

            '>' if self.peek() == Some('=') => {
                self.bump();
                TokenKind::Ge
            }
            '>' => TokenKind::Gt,

            // `&` and `|` will only ever appear doubled 
            '&' if self.peek() == Some('&') => {
                self.bump();
                TokenKind::AndAnd
            }
            '|' if self.peek() == Some('|') => {
                self.bump();
                TokenKind::OrOr
            }

            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            ';' => TokenKind::Semi,

            other => {
                return Err(format!(
                    "unexpected character '{}' at byte {}",
                    other, start
                ));
            }
        };

        Ok(Token {
            kind,
            lexeme: self.slice(start),
            pos: start,
        })
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += 1;
        Some(ch)
    }

    fn slice(&self, start: usize) -> String {
        self.chars[start..self.pos].iter().collect()
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_ident_continue(ch: char) -> bool {
    is_ident_start(ch) || ch.is_ascii_digit()
}

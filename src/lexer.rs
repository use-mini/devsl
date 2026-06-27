#[derive(Debug)]
pub struct Span {
    start: usize,
    end: usize,
}

#[derive(Debug)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

impl LexError {
    fn new(message: String, start: usize, end: usize) -> LexError {
        LexError {
            message,
            span: Span { start, end },
        }
    }
}

#[derive(Debug)]
enum TokenKind {
    Identifier(String),
    Keyword(String),
    StringLiteral(String),

    OParen,
    CParen,
    SemiColon,
    Newline,
    Eof,
}

#[derive(Debug)]
pub struct Token {
    kind: TokenKind,
    span: Span,
}

impl Token {
    fn new(kind: TokenKind, start: usize, end: usize) -> Token {
        Token {
            kind,
            span: Span { start, end },
        }
    }
}

pub struct Lexer<'a> {
    code: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(code: &str) -> Lexer<'_> {
        Lexer { code, pos: 0 }
    }

    pub fn is_at_end(&self) -> bool {
        self.code.len() == self.pos
    }

    pub fn peek(&self) -> Option<char> {
        self.code[self.pos..].chars().next()
    }

    pub fn peek_next(&self) -> Option<char> {
        let c = self.peek()?;
        self.code[self.pos + c.len_utf8()..].chars().next()
    }

    pub fn advance(&mut self) -> Option<char> {
        let c = self.code[self.pos..].chars().next()?;
        self.pos = self.pos + c.len_utf8();
        Some(c)
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        while !self.is_at_end() {
            while let Some(c) = self.peek() {
                if c == ' ' || c == '\t' {
                    self.advance();
                } else {
                    break;
                }
            }
            if self.is_at_end() {
                break;
            }
            tokens.push(self.next_token()?);
        }
        tokens.push(Token::new(TokenKind::Eof, self.pos, self.pos));
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        let Some(c) = self.advance() else {
            unreachable!("lex already checks for end of input");
        };
        match c {
            'a'..='z' | 'A'..='Z' | '_' => self.lex_identifier(start),
            '"' => self.lex_string_literal(start),
            '(' => Ok(Token::new(TokenKind::OParen, start, self.pos)),
            ')' => Ok(Token::new(TokenKind::CParen, start, self.pos)),
            ';' => Ok(Token::new(TokenKind::SemiColon, start, self.pos)),
            '\n' => Ok(Token::new(TokenKind::Newline, start, self.pos)),
            _ => todo!("char {c:?} at pos {}", self.pos),
        }
    }

    fn lex_identifier(&mut self, start: usize) -> Result<Token, LexError> {
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let name = self.code[start..self.pos].to_string();

        match name.as_str() {
            _ => Ok(Token::new(TokenKind::Identifier(name), start, self.pos)),
        }
    }

    fn lex_string_literal(&mut self, start: usize) -> Result<Token, LexError> {
        let mut string_literal = String::new();
        while let Some(c) = self.peek() {
            match c {
                '"' => break,
                '\\' => {
                    self.advance();
                    match self.advance() {
                        Some('"') => string_literal.push('"'),
                        Some(other) => {
                            return Err(LexError::new(
                                format!("invalid escape sequence: {other}"),
                                start,
                                self.pos,
                            ));
                        }
                        None => {
                            return Err(LexError::new(
                                "unterminated escape sequence".into(),
                                start,
                                self.pos,
                            ));
                        }
                    }
                }
                _ => {
                    string_literal.push(c);
                    self.advance();
                }
            }
        }
        if self.peek().is_none() {
            return Err(LexError::new(
                "unterminated string literal".into(),
                start,
                self.pos,
            ));
        }
        self.advance();
        Ok(Token::new(
            TokenKind::StringLiteral(string_literal),
            start,
            self.pos,
        ))
    }
}

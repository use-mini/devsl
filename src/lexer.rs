#[derive(Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
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
pub enum TokenKind {
    Identifier(String),
    ExtendedIdentifier(String),
    StringLiteral(String),
    Int(i64),
    Float(f64),

    OParen,
    CParen,
    OBracket,
    CBracket,
    OCurly,
    CCurly,

    Dot,
    Comma,
    Colon,
    SemiColon,

    Plus,
    Minus,
    Star,
    Slash,
    Eq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    EqEq,
    BangEq,
    Or,
    And,
    Not,

    Arrow,

    Var,
    Const,
    If,
    Else,
    Bool(bool),
    Return,
    For,
    In,
    Null,
    Fn,

    Newline,
    Eof,
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
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

    fn is_at_end(&self) -> bool {
        self.code.len() == self.pos
    }

    fn peek(&self) -> Option<char> {
        self.code[self.pos..].chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        let c = self.peek()?;
        self.code[self.pos + c.len_utf8()..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.code[self.pos..].chars().next()?;
        self.pos = self.pos + c.len_utf8();
        Some(c)
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        while !self.is_at_end() {
            while let Some(c) = self.peek() {
                match c {
                    ' ' | '\t' => {
                        self.advance();
                    }
                    '#' => {
                        while let Some(c) = self.peek() {
                            if c == '\n' {
                                break;
                            }
                            self.advance();
                        }
                    }
                    _ => break,
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
            '0'..='9' => self.lex_number(start),
            'a'..='z' | 'A'..='Z' | '_' => self.lex_identifier(start),
            '"' => self.lex_string_literal(start),
            '(' => Ok(Token::new(TokenKind::OParen, start, self.pos)),
            ')' => Ok(Token::new(TokenKind::CParen, start, self.pos)),
            '[' => Ok(Token::new(TokenKind::OBracket, start, self.pos)),
            ']' => Ok(Token::new(TokenKind::CBracket, start, self.pos)),
            '{' => Ok(Token::new(TokenKind::OCurly, start, self.pos)),
            '}' => Ok(Token::new(TokenKind::CCurly, start, self.pos)),
            '@' => self.lex_extended_identifier(start),
            '.' => Ok(Token::new(TokenKind::Dot, start, self.pos)),
            ',' => Ok(Token::new(TokenKind::Comma, start, self.pos)),
            ':' => Ok(Token::new(TokenKind::Colon, start, self.pos)),
            ';' => Ok(Token::new(TokenKind::SemiColon, start, self.pos)),
            '+' => Ok(Token::new(TokenKind::Plus, start, self.pos)),
            '-' if matches!(self.peek(), Some('>')) => {
                self.advance();
                Ok(Token::new(TokenKind::Arrow, start, self.pos))
            }
            '-' => Ok(Token::new(TokenKind::Minus, start, self.pos)),
            '*' => Ok(Token::new(TokenKind::Star, start, self.pos)),
            '/' => Ok(Token::new(TokenKind::Slash, start, self.pos)),
            '=' if matches!(self.peek(), Some('=')) => {
                self.advance();
                Ok(Token::new(TokenKind::EqEq, start, self.pos))
            }
            '=' => Ok(Token::new(TokenKind::Eq, start, self.pos)),
            '!' if matches!(self.peek(), Some('=')) => {
                self.advance();
                Ok(Token::new(TokenKind::BangEq, start, self.pos))
            }
            '<' if matches!(self.peek(), Some('=')) => {
                self.advance();
                Ok(Token::new(TokenKind::LtEq, start, self.pos))
            }
            '<' => Ok(Token::new(TokenKind::Lt, start, self.pos)),
            '>' if matches!(self.peek(), Some('=')) => {
                self.advance();
                Ok(Token::new(TokenKind::GtEq, start, self.pos))
            }
            '>' => Ok(Token::new(TokenKind::Gt, start, self.pos)),

            '`' => Err(LexError::new(
                "backtick is reserved for future interpolated strings".into(),
                start,
                self.pos,
            )),
            '$' => Err(LexError::new(
                "`$` is reserved for future interpolated strings".into(),
                start,
                self.pos,
            )),

            '\n' => Ok(Token::new(TokenKind::Newline, start, self.pos)),
            _ => Err(LexError::new(
                format!("unexpected character: {c:?}"),
                start,
                self.pos,
            )),
        }
    }

    fn next_char_pos(&self) -> usize {
        match self.peek() {
            Some(c) => self.pos + c.len_utf8(),
            None => self.pos,
        }
    }

    fn lex_number(&mut self, start: usize) -> Result<Token, LexError> {
        let mut has_exp = false;
        let mut is_float = false;
        while let Some(c) = self.peek() {
            match c {
                '0'..='9' => {
                    self.advance();
                }
                '_' => {
                    self.advance();
                    if !matches!(self.peek(), Some('0'..='9')) {
                        let end = self.next_char_pos();
                        return Err(LexError::new(
                            format!(
                                "invalid number literal: `_` must appear between digits: `{}`",
                                &self.code[start..end]
                            ),
                            start,
                            end,
                        ));
                    }
                }
                '.' if !is_float => {
                    self.advance();
                    is_float = true;
                }
                '.' => {
                    return Err(LexError::new(
                        format!(
                            "invalid float number: `{}`. It contains two or more decimal points",
                            &self.code[start..self.next_char_pos()]
                        ),
                        start,
                        self.next_char_pos(),
                    ));
                }
                'e' | 'E' if !has_exp => {
                    self.advance();
                    is_float = true;
                    has_exp = true;
                    if matches!(self.peek(), Some('+' | '-')) {
                        self.advance();
                    }
                    if !matches!(self.peek(), Some('0'..='9')) {
                        let end = self.next_char_pos();
                        return Err(LexError::new(
                            format!("invalid exponent: `{}`", &self.code[start..end]),
                            start,
                            end,
                        ));
                    }
                }
                _ => break,
            }
        }

        if matches!(self.peek(), Some(c) if c.is_ascii_alphabetic() || c == '_') {
            while let Some(c) = self.peek() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    self.advance();
                } else {
                    break;
                }
            }
            return Err(LexError::new(
                format!("invalid number literal: `{}`", &self.code[start..self.pos]),
                start,
                self.pos,
            ));
        }

        let number_string: String = self.code[start..self.pos]
            .chars()
            .filter(|&c| c != '_')
            .collect();

        if is_float {
            let number = number_string
                .parse::<f64>()
                .map_err(|e| LexError::new(format!("invalid float: {e}"), start, self.pos))?;
            Ok(Token::new(TokenKind::Float(number), start, self.pos))
        } else {
            let number = number_string
                .parse::<i64>()
                .map_err(|e| LexError::new(format!("invalid integer: {e}"), start, self.pos))?;
            Ok(Token::new(TokenKind::Int(number), start, self.pos))
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
            "or" => Ok(Token::new(TokenKind::Or, start, self.pos)),
            "and" => Ok(Token::new(TokenKind::And, start, self.pos)),
            "not" => Ok(Token::new(TokenKind::Not, start, self.pos)),
            "var" => Ok(Token::new(TokenKind::Var, start, self.pos)),
            "const" => Ok(Token::new(TokenKind::Const, start, self.pos)),
            "if" => Ok(Token::new(TokenKind::If, start, self.pos)),
            "else" => Ok(Token::new(TokenKind::Else, start, self.pos)),
            "true" => Ok(Token::new(TokenKind::Bool(true), start, self.pos)),
            "false" => Ok(Token::new(TokenKind::Bool(false), start, self.pos)),
            "return" => Ok(Token::new(TokenKind::Return, start, self.pos)),
            "for" => Ok(Token::new(TokenKind::For, start, self.pos)),
            "in" => Ok(Token::new(TokenKind::In, start, self.pos)),
            "null" => Ok(Token::new(TokenKind::Null, start, self.pos)),
            "fn" => Ok(Token::new(TokenKind::Fn, start, self.pos)),
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
                        Some('n') => string_literal.push('\n'),
                        Some('t') => string_literal.push('\t'),
                        Some('\\') => string_literal.push('\\'),
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
                '\n' => {
                    return Err(LexError::new(
                        format!(
                            "string literal cannot span multiple lines: `{}`",
                            &self.code[start..self.pos]
                        ),
                        start,
                        self.pos,
                    ));
                }
                _ => {
                    string_literal.push(c);
                    self.advance();
                }
            }
        }
        if self.peek().is_none() {
            return Err(LexError::new(
                format!(
                    "unterminated string literal: `{}`",
                    &self.code[start..self.pos]
                ),
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

    fn lex_extended_identifier(&mut self, start: usize) -> Result<Token, LexError> {
        let name_start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                self.advance();
            } else {
                break;
            }
        }
        if name_start == self.pos {
            Err(LexError::new(
                "expected identifier after `@`".into(),
                start,
                self.next_char_pos(),
            ))
        } else {
            let name = self.code[name_start..self.pos].to_string();
            Ok(Token::new(
                TokenKind::ExtendedIdentifier(name),
                start,
                self.pos,
            ))
        }
    }
}

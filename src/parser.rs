use crate::lexer::{Span, Token, TokenKind};

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl ParseError {
    fn new(message: String, span: Span) -> ParseError {
        ParseError { message, span }
    }
}

#[derive(Debug)]
pub enum Expr {
    String(String, Span),
    Identifier(String, Span),
    Int(i64, Span),
    Float(f64, Span),
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::String(_, span) => *span,
            Expr::Identifier(_, span) => *span,
            Expr::Int(_, span) => *span,
            Expr::Float(_, span) => *span,
            Expr::Call { span, .. } => *span,
        }
    }
}

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr),
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn advance(&mut self) -> &Token {
        assert!(!self.is_at_end(), "unexpected end of tokens");
        let token = &self.tokens[self.pos];
        self.pos += 1;
        token
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut exprs = Vec::new();
        while !self.is_at_end() {
            while matches!(self.peek().kind, TokenKind::Newline) {
                self.advance();
            }
            if self.is_at_end() {
                break;
            }
            exprs.push(Stmt::Expr(self.parse_expr()?));
            let token = self.peek();
            match token.kind {
                TokenKind::Newline | TokenKind::SemiColon => {
                    self.advance();
                }
                TokenKind::Eof => {}
                _ => {
                    return Err(ParseError::new(
                        format!(
                            "expected new line or `;` after expression, found {:?}",
                            token.kind
                        ),
                        token.span,
                    ));
                }
            }
        }
        Ok(exprs)
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        if self.is_at_end() {
            return Err(ParseError::new(
                "unexpected end of input".into(),
                self.peek().span,
            ));
        }
        let token = self.peek().clone();
        self.advance();
        match token.kind {
            TokenKind::StringLiteral(s) => Ok(Expr::String(s, token.span)),
            TokenKind::Int(i) => Ok(Expr::Int(i, token.span)),
            TokenKind::Float(f) => Ok(Expr::Float(f, token.span)),
            TokenKind::Identifier(ref identifier) => {
                if matches!(&self.peek().kind, TokenKind::OParen) {
                    self.advance();
                    let mut args = Vec::new();
                    let mut cparen_span = self.peek().span;
                    if !matches!(&self.peek().kind, TokenKind::CParen) {
                        loop {
                            if matches!(self.peek().kind, TokenKind::Comma | TokenKind::CParen) {
                                return Err(ParseError::new(
                                    format!("expected argument, found {:?}", self.peek().kind),
                                    self.peek().span,
                                ));
                            }
                            args.push(self.parse_expr()?);
                            let peek = self.peek();
                            match &peek.kind {
                                TokenKind::Comma => {
                                    self.advance();
                                }
                                TokenKind::CParen => {
                                    cparen_span = peek.span;
                                    break;
                                }
                                _ => {
                                    return Err(ParseError::new(
                                        format!(
                                            "expected `,` or `)`, found {:?}",
                                            self.peek().kind
                                        ),
                                        self.peek().span,
                                    ));
                                }
                            }
                        }
                    }
                    self.advance();
                    Ok(Expr::Call {
                        callee: Box::new(Expr::Identifier(identifier.to_string(), token.span)),
                        args,
                        span: Span {
                            start: token.span.start,
                            end: cparen_span.end,
                        },
                    })
                } else {
                    Ok(Expr::Identifier(identifier.to_string(), token.span))
                }
            }
            _ => Err(ParseError::new(
                format!("expected expression, found {:?}", token.kind),
                token.span,
            )),
        }
    }
}

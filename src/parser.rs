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

#[derive(Debug, Copy, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,

    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
}

#[derive(Debug)]
pub enum Expr {
    String(String, Span),
    Identifier(String, Span),
    Int(i64, Span),
    Float(f64, Span),
    Bool(bool, Span),
    Null(Span),
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        span: Span,
    },
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
            Expr::Bool(_, span) => *span,
            Expr::Null(span) => *span,
            Expr::Binary { span, .. } => *span,
            Expr::Call { span, .. } => *span,
        }
    }
    pub fn with_span(self, span: Span) -> Expr {
        match self {
            Expr::String(string, _) => Expr::String(string, span),
            Expr::Identifier(identifier, _) => Expr::Identifier(identifier, span),
            Expr::Int(int, _) => Expr::Int(int, span),
            Expr::Float(float, _) => Expr::Float(float, span),
            Expr::Bool(bool, _) => Expr::Bool(bool, span),
            Expr::Null(_) => Expr::Null(span),
            Expr::Binary { op, lhs, rhs, .. } => Expr::Binary { op, lhs, rhs, span },
            Expr::Call { callee, args, .. } => Expr::Call { callee, args, span },
        }
    }
}

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr),
    Var {
        name: String,
        value: Expr,
        span: Span,
    },
    Const {
        name: String,
        value: Expr,
        span: Span,
    },
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
            exprs.push(self.parse_stmt()?);
            let token = self.peek();
            match token.kind {
                TokenKind::Newline | TokenKind::SemiColon => {
                    self.advance();
                }
                TokenKind::Eof => {}
                _ => {
                    return Err(ParseError::new(
                        format!(
                            "expected new line or `;` after statement, found {:?}",
                            token.kind
                        ),
                        token.span,
                    ));
                }
            }
        }
        Ok(exprs)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek().kind {
            TokenKind::Var => self.parse_binding(true),
            TokenKind::Const => self.parse_binding(false),
            _ => Ok(Stmt::Expr(self.parse_expr()?)),
        }
    }

    fn parse_binding(&mut self, is_var: bool) -> Result<Stmt, ParseError> {
        let kw_span = self.peek().span;
        self.advance();

        let name = match &self.peek().kind {
            TokenKind::Identifier(identifier) => identifier.clone(),
            _ => {
                return Err(ParseError::new(
                    format!(
                        "expected identifier after `{}`, found {:?}",
                        if is_var { "var" } else { "const" },
                        self.peek().kind
                    ),
                    self.peek().span,
                ));
            }
        };
        self.advance();
        if !matches!(self.peek().kind, TokenKind::Eq) {
            return Err(ParseError::new(
                format!(
                    "expected `=` after binding name, found {:?}",
                    self.peek().kind
                ),
                self.peek().span,
            ));
        }
        self.advance();
        let value = self.parse_expr().map_err(|e| {
            ParseError::new(
                format!("expected expression after `=`: {}", e.message),
                e.span,
            )
        })?;
        let span = Span {
            start: kw_span.start,
            end: value.span().end,
        };
        Ok(if is_var {
            Stmt::Var { name, value, span }
        } else {
            Stmt::Const { name, value, span }
        })
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_comparison()
    }
    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let lhs = self.parse_additive()?;
        let op = match self.peek().kind {
            TokenKind::Lt => BinOp::Lt,
            TokenKind::Gt => BinOp::Gt,
            TokenKind::LtEq => BinOp::LtEq,
            TokenKind::GtEq => BinOp::GtEq,
            TokenKind::EqEq => BinOp::Eq,
            TokenKind::BangEq => BinOp::NotEq,
            _ => return Ok(lhs),
        };
        self.advance();
        let rhs = self.parse_additive()?;
        if matches!(
            self.peek().kind,
            TokenKind::Lt
                | TokenKind::Gt
                | TokenKind::LtEq
                | TokenKind::GtEq
                | TokenKind::EqEq
                | TokenKind::BangEq,
        ) {
            return Err(ParseError::new(
                "chained comparison is not allowed; use `and` to combine".into(),
                self.peek().span,
            ));
        }
        let span = Span {
            start: lhs.span().start,
            end: rhs.span().end,
        };
        Ok(Expr::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            span,
        })
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_multiplicative()?;

        loop {
            let op = match self.peek().kind {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_multiplicative()?;
            let span = Span {
                start: lhs.span().start,
                end: rhs.span().end,
            };
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                span,
            };
        }
        Ok(lhs)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_primary()?;

        loop {
            let op = match self.peek().kind {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                _ => break,
            };
            self.advance();
            let rhs = self.parse_primary()?;
            let span = Span {
                start: lhs.span().start,
                end: rhs.span().end,
            };
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                span,
            };
        }
        Ok(lhs)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        if self.is_at_end() {
            return Err(ParseError::new(
                "unexpected end of input".into(),
                self.peek().span,
            ));
        }

        if matches!(self.peek().kind, TokenKind::OParen) {
            let oparen_span = self.peek().span;
            self.advance();
            let inner = self.parse_expr()?;
            if !matches!(self.peek().kind, TokenKind::CParen) {
                return Err(ParseError::new(
                    format!("expected `)`, found {:?}", self.peek().kind),
                    self.peek().span,
                ));
            }
            let cparen_span = self.peek().span;
            self.advance();
            return Ok(inner.with_span(Span {
                start: oparen_span.start,
                end: cparen_span.end,
            }));
        }

        let token = self.peek().clone();
        self.advance();
        match token.kind {
            TokenKind::StringLiteral(s) => Ok(Expr::String(s, token.span)),
            TokenKind::Int(i) => Ok(Expr::Int(i, token.span)),
            TokenKind::Float(f) => Ok(Expr::Float(f, token.span)),
            TokenKind::Bool(b) => Ok(Expr::Bool(b, token.span)),
            TokenKind::Null => Ok(Expr::Null(token.span)),
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

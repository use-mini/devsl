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

    Or,
    And,
}

#[derive(Debug)]
pub enum Expr {
    String(String, Span),
    Identifier(String, Span),
    Int(i64, Span),
    Float(f64, Span),
    Bool(bool, Span),
    Null(Span),
    List {
        items: Vec<Expr>,
        span: Span,
    },
    Object {
        entries: Vec<(String, Expr)>,
        span: Span,
    },
    Not {
        inner: Box<Expr>,
        span: Span,
    },
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
            Expr::List { span, .. } => *span,
            Expr::Object { span, .. } => *span,
            Expr::Not { span, .. } => *span,
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
            Expr::List { items, .. } => Expr::List { items, span },
            Expr::Object { entries, .. } => Expr::Object { entries, span },
            Expr::Not { inner, .. } => Expr::Not { inner, span },
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
    Block {
        stmts: Vec<Stmt>,
        span: Span,
    },
    Reassign {
        name: String,
        value: Expr,
        span: Span,
    },
    If {
        cond: Expr,
        then_block: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
        span: Span,
    },
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Expr(e) => e.span(),
            Stmt::Var { span, .. } => *span,
            Stmt::Const { span, .. } => *span,
            Stmt::Block { span, .. } => *span,
            Stmt::Reassign { span, .. } => *span,
            Stmt::If { span, .. } => *span,
        }
    }
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

    fn peek_nth(&self, idx: usize) -> &Token {
        let idx = self.pos + idx;
        if idx >= self.tokens.len() {
            &self.tokens[self.tokens.len() - 1] // Eof
        } else {
            &self.tokens[idx]
        }
    }

    fn peek_next(&self) -> &Token {
        self.peek_nth(1)
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
            TokenKind::OCurly if !self.is_object_literal() => self.parse_block(),
            TokenKind::If => self.parse_if(),
            TokenKind::Identifier(_) if matches!(self.peek_next().kind, TokenKind::Eq) => {
                self.parse_reassign()
            }
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

    fn parse_block(&mut self) -> Result<Stmt, ParseError> {
        let ocurly_span = self.peek().span;
        self.advance();
        let mut stmts = Vec::new();
        loop {
            while matches!(self.peek().kind, TokenKind::Newline) {
                self.advance();
            }
            if matches!(self.peek().kind, TokenKind::CCurly) {
                break;
            }

            if self.is_at_end() {
                return Err(ParseError::new(
                    "unclosed block: expected `}`, found end of input".into(),
                    self.peek().span,
                ));
            }
            stmts.push(self.parse_stmt()?);
            match self.peek().kind {
                TokenKind::Newline | TokenKind::SemiColon => {
                    self.advance();
                }
                TokenKind::CCurly => break,
                TokenKind::Eof => {
                    return Err(ParseError::new(
                        "unclosed block: expected `}`, found end of input".into(),
                        self.peek().span,
                    ));
                }
                _ => {
                    return Err(ParseError::new(
                        format!(
                            "expected new line, `;` or `}}` after statement, found {:?}",
                            self.peek().kind
                        ),
                        self.peek().span,
                    ));
                }
            }
        }
        let ccurly_span = self.peek().span;
        self.advance();
        Ok(Stmt::Block {
            stmts,
            span: Span {
                start: ocurly_span.start,
                end: ccurly_span.end,
            },
        })
    }

    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        let kw_span = self.peek().span;
        self.advance();

        if matches!(self.peek().kind, TokenKind::OCurly) {
            return Err(ParseError::new(
                "expected condition after `if`, found `{`".into(),
                self.peek().span,
            ));
        }
        let cond = self.parse_expr().map_err(|e| {
            ParseError::new(
                format!("expected condition after `if`: {}", e.message),
                e.span,
            )
        })?;

        if !matches!(self.peek().kind, TokenKind::OCurly) {
            return Err(ParseError::new(
                format!(
                    "expected `{{` after if condition, found {:?}",
                    self.peek().kind
                ),
                self.peek().span,
            ));
        }
        let then_block = self.parse_block()?;
        let mut end = then_block.span().end;

        let saved_pos = self.pos;
        while matches!(self.peek().kind, TokenKind::Newline) {
            self.advance();
        }
        if !matches!(self.peek().kind, TokenKind::Else) {
            // NOTE: restore pos, so newlines can be used as a statement terminator
            self.pos = saved_pos;
        }

        let else_branch = if matches!(self.peek().kind, TokenKind::Else) {
            self.advance();
            let branch = if matches!(self.peek().kind, TokenKind::If) {
                self.parse_if()?
            } else if matches!(self.peek().kind, TokenKind::OCurly) {
                self.parse_block()?
            } else {
                return Err(ParseError::new(
                    format!(
                        "expected `{{` or `if` after `else`, found {:?}",
                        self.peek().kind
                    ),
                    self.peek().span,
                ));
            };
            end = branch.span().end;
            Some(Box::new(branch))
        } else {
            None
        };

        Ok(Stmt::If {
            cond,
            then_block: Box::new(then_block),
            else_branch,
            span: Span {
                start: kw_span.start,
                end,
            },
        })
    }

    fn parse_reassign(&mut self) -> Result<Stmt, ParseError> {
        let (name, span) = match &self.peek().kind {
            TokenKind::Identifier(identifier) => (identifier.clone(), self.peek().span),
            _ => unreachable!("dispatch guarantees Identifier"),
        };
        self.advance(); // identifier
        self.advance(); // `=`
        let value = self.parse_expr().map_err(|e| {
            ParseError::new(
                format!("expected expression after `=`: {}", e.message),
                e.span,
            )
        })?;

        let span = Span {
            start: span.start,
            end: value.span().end,
        };
        Ok(Stmt::Reassign { name, value, span })
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_and()?;

        while matches!(self.peek().kind, TokenKind::Or) {
            self.advance();
            let rhs = self.parse_and()?;
            let span = Span {
                start: lhs.span().start,
                end: rhs.span().end,
            };
            lhs = Expr::Binary {
                op: BinOp::Or,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                span,
            };
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_not()?;
        while matches!(self.peek().kind, TokenKind::And) {
            self.advance();
            let rhs = self.parse_not()?;
            let span = Span {
                start: lhs.span().start,
                end: rhs.span().end,
            };
            lhs = Expr::Binary {
                op: BinOp::And,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                span,
            };
        }
        Ok(lhs)
    }

    fn parse_not(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek().kind, TokenKind::Not) {
            let kw_span = self.peek().span;
            self.advance();
            let inner = self.parse_not()?;
            let span = Span {
                start: kw_span.start,
                end: inner.span().end,
            };
            return Ok(Expr::Not {
                inner: Box::new(inner),
                span,
            });
        }
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

        if matches!(self.peek().kind, TokenKind::OBracket) {
            return self.parse_list_literal();
        }

        if matches!(self.peek().kind, TokenKind::OCurly) {
            return self.parse_object_literal();
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

    fn parse_list_literal(&mut self) -> Result<Expr, ParseError> {
        let obracket_span = self.peek().span;
        self.advance();
        let mut cbracket_span = self.peek().span;
        if matches!(self.peek().kind, TokenKind::CBracket) {
            self.advance();
            return Ok(Expr::List {
                items: Vec::new(),
                span: Span {
                    start: obracket_span.start,
                    end: cbracket_span.end,
                },
            });
        }
        let mut items = Vec::new();
        loop {
            while matches!(self.peek().kind, TokenKind::Newline) {
                self.advance();
            }
            if matches!(self.peek().kind, TokenKind::Comma) {
                self.advance();
                while matches!(self.peek().kind, TokenKind::Newline) {
                    self.advance();
                }
            }

            if matches!(self.peek().kind, TokenKind::CBracket) {
                cbracket_span = self.peek().span;
                self.advance();
                break;
            }

            let item = self.parse_expr()?;
            items.push(item);

            if !matches!(self.peek().kind, TokenKind::Comma | TokenKind::CBracket) {
                return Err(ParseError::new(
                    format!("expected `,` or `]`, found {:?}", self.peek().kind),
                    self.peek().span,
                ));
            }
        }
        Ok(Expr::List {
            items,
            span: Span {
                start: obracket_span.start,
                end: cbracket_span.end,
            },
        })
    }

    fn is_object_literal(&self) -> bool {
        matches!(self.peek_nth(1).kind, TokenKind::CCurly)
            || (matches!(
                self.peek_nth(1).kind,
                TokenKind::Identifier(_)
                    | TokenKind::ExtendedIdentifier(_)
                    | TokenKind::StringLiteral(_)
            ) && matches!(self.peek_nth(2).kind, TokenKind::Colon))
    }

    fn parse_object_literal(&mut self) -> Result<Expr, ParseError> {
        let ocurly_span = self.peek().span;
        self.advance();
        let mut ccurly_span = self.peek().span;
        if matches!(self.peek().kind, TokenKind::CCurly) {
            self.advance();
            return Ok(Expr::Object {
                entries: Vec::new(),
                span: Span {
                    start: ocurly_span.start,
                    end: ccurly_span.end,
                },
            });
        }

        let mut entries = Vec::<(String, Expr)>::new();
        loop {
            while matches!(self.peek().kind, TokenKind::Newline) {
                self.advance();
            }
            if matches!(self.peek().kind, TokenKind::Comma) {
                self.advance();
                while matches!(self.peek().kind, TokenKind::Newline) {
                    self.advance();
                }
            }

            if matches!(self.peek().kind, TokenKind::CCurly) {
                ccurly_span = self.peek().span;
                self.advance();
                break;
            }

            let key_span = self.peek().span;
            let key_name = match &self.peek().kind {
                TokenKind::ExtendedIdentifier(s) => s.clone(),
                TokenKind::StringLiteral(s) => s.clone(),
                TokenKind::Identifier(identifier) => identifier.clone(),
                _ => {
                    return Err(ParseError::new(
                        format!(
                            "expected object key (identifier, @kebab-key or string), found {:?}",
                            self.peek().kind
                        ),
                        self.peek().span,
                    ));
                }
            };
            self.advance();

            for e in entries.iter() {
                if e.0 == key_name {
                    return Err(ParseError::new(
                        format!("`{key_name}` is already defined in the object"),
                        key_span,
                    ));
                }
            }

            if matches!(self.peek().kind, TokenKind::Colon) {
                self.advance();
            } else {
                return Err(ParseError::new(
                    format!(
                        "expected `:` after identifier `{key_name}`, found {:?}",
                        self.peek().kind
                    ),
                    self.peek().span,
                ));
            }

            let value = self.parse_expr()?;

            entries.push((key_name, value));

            if !matches!(self.peek().kind, TokenKind::Comma | TokenKind::CCurly) {
                return Err(ParseError::new(
                    format!("expected `,` or `}}`, found {:?}", self.peek().kind),
                    self.peek().span,
                ));
            }
        }

        Ok(Expr::Object {
            entries,
            span: Span {
                start: ocurly_span.start,
                end: ccurly_span.end,
            },
        })
    }
}

use std::{collections::HashMap, error::Error};

use crate::lexer::Span;
use crate::parser::{Expr, Stmt};

#[derive(Debug, Clone)]
enum Value {
    String(String),
    Int(i64),
    Float(f64),
}

#[derive(Debug, Clone)]
struct Binding {
    value: Value,
    is_const: bool,
}

#[derive(Debug, Default)]
pub struct Env {
    values: HashMap<String, Binding>,
}

impl Env {
    pub fn new() -> Env {
        Env::default()
    }

    fn define(&mut self, name: String, value: Value, is_const: bool) {
        self.values.insert(name, Binding { value, is_const });
    }

    fn lookup(&self, name: &str) -> Option<&Value> {
        self.values.get(name).map(|b| &b.value)
    }
}

pub struct EvalCtx<'w> {
    pub env: Env,
    pub out: &'w mut dyn std::io::Write,
}

fn bind(
    ctx: &mut EvalCtx,
    name: &str,
    value: &Expr,
    span: Span,
    is_const: bool,
) -> Result<(), EvalError> {
    let v = match eval_expr(value, ctx)? {
        Some(v) => v,
        None => {
            return Err(EvalError::new(
                ErrorCategory::Type,
                "cannot bind an expressions that doesn't return a value".into(),
                span,
            ));
        }
    };

    ctx.env.define(name.to_string(), v, is_const);
    Ok(())
}

pub fn eval(stmts: &[Stmt], ctx: &mut EvalCtx) -> Result<(), EvalError> {
    for stmt in stmts {
        eval_stmt(stmt, ctx)?;
    }

    Ok(())
}

fn eval_stmt(stmt: &Stmt, ctx: &mut EvalCtx) -> Result<(), EvalError> {
    match stmt {
        Stmt::Expr(e) => {
            let _ = eval_expr(e, ctx)?;
            Ok(())
        }
        Stmt::Var { name, value, span } => bind(ctx, name, value, *span, false),
        Stmt::Const { name, value, span } => bind(ctx, name, value, *span, true),
    }
}

fn eval_expr(expr: &Expr, ctx: &mut EvalCtx) -> Result<Option<Value>, EvalError> {
    match expr {
        Expr::Int(int, _) => Ok(Some(Value::Int(*int))),
        Expr::Float(float, _) => Ok(Some(Value::Float(*float))),
        Expr::String(string, _) => Ok(Some(Value::String(string.clone()))),
        Expr::Identifier(identifier, span) => match ctx.env.lookup(identifier) {
            Some(v) => Ok(Some(v.clone())),
            None => Err(EvalError::new(
                ErrorCategory::Name,
                format!("unknown identifier `{identifier}`"),
                *span,
            )),
        },
        Expr::Binary { span, .. } | Expr::Call { span, .. } => Err(EvalError::new(
            ErrorCategory::Runtime,
            "expression kind not implemented".into(),
            *span,
        )),
    }
}

#[derive(Debug)]
pub enum ErrorCategory {
    Type,
    Name,
    Runtime,
}

#[derive(Debug)]
pub struct EvalError {
    pub category: ErrorCategory,
    pub message: String,
    pub span: Span,
    pub cause: Option<Box<dyn Error + Send + Sync>>,
}

impl EvalError {
    pub fn new(category: ErrorCategory, message: String, span: Span) -> EvalError {
        EvalError {
            category,
            message,
            span,
            cause: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Expr, Stmt};

    fn no_span() -> Span {
        Span { start: 0, end: 0 }
    }

    fn run_stmts(stmts: &[Stmt]) -> Result<String, EvalError> {
        let mut buf = Vec::<u8>::new();
        let mut ctx = EvalCtx {
            env: Env::new(),
            out: &mut buf,
        };
        eval(stmts, &mut ctx)?;
        Ok(String::from_utf8(buf).unwrap())
    }

    #[test]
    fn env_define_and_lookup() {
        let mut env = Env::new();
        env.define("x".into(), Value::Int(42), false);
        match env.lookup("x") {
            Some(Value::Int(42)) => {}
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn env_lookup_missing() {
        let env = Env::new();
        assert!(env.lookup("nope").is_none());
    }

    #[test]
    fn env_const_flag_is_tracked() {
        let mut env = Env::new();
        env.define("PI".into(), Value::Float(3.14), true);
        assert!(matches!(env.lookup("PI"), Some(Value::Float(_))));
    }

    #[test]
    fn eval_empty_program() {
        assert_eq!(run_stmts(&[]).unwrap(), "");
    }

    #[test]
    fn eval_literal_int_is_discarded() {
        let stmts = vec![Stmt::Expr(Expr::Int(42, no_span()))];
        assert_eq!(run_stmts(&stmts).unwrap(), "");
    }

    #[test]
    fn eval_unknown_identifier_is_name_error() {
        let stmts = vec![Stmt::Expr(Expr::Identifier("nope".into(), no_span()))];
        let err = run_stmts(&stmts).unwrap_err();
        assert!(matches!(err.category, ErrorCategory::Name));
        assert!(err.message.contains("nope"));
    }

    fn ident(name: &str) -> Expr {
        Expr::Identifier(name.into(), no_span())
    }

    fn var_stmt(name: &str, value: Expr) -> Stmt {
        Stmt::Var {
            name: name.into(),
            value,
            span: no_span(),
        }
    }

    fn const_stmt(name: &str, value: Expr) -> Stmt {
        Stmt::Const {
            name: name.into(),
            value,
            span: no_span(),
        }
    }

    #[test]
    fn var_binds_value() {
        let stmts = vec![
            var_stmt("x", Expr::Int(7, no_span())),
            Stmt::Expr(ident("x")),
        ];
        run_stmts(&stmts).unwrap();
    }

    #[test]
    fn const_binds_value() {
        let stmts = vec![
            const_stmt("pi", Expr::Float(3.14, no_span())),
            Stmt::Expr(ident("pi")),
        ];
        run_stmts(&stmts).unwrap();
    }
}

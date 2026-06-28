use std::{collections::HashMap, error::Error};

use crate::lexer::Span;
use crate::parser::{BinOp, Expr, Stmt};

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
    if builtin_is_name(name) {
        return Err(EvalError::new(
            ErrorCategory::Name,
            format!("cannot shadow builtin `{name}`"),
            span,
        ));
    }
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
        Expr::Binary { op, lhs, rhs, span } => {
            let l = require_value(eval_expr(lhs, ctx)?, lhs.span())?;
            let r = require_value(eval_expr(rhs, ctx)?, rhs.span())?;
            eval_binary(*op, l, r, *span).map(Some)
        }
        Expr::Call { callee, args, span } => {
            let name = match callee.as_ref() {
                Expr::Identifier(identifier, _) => identifier,
                _ => {
                    return Err(EvalError::new(
                        ErrorCategory::Type,
                        "callee must be an identifier".into(),
                        callee.span(),
                    ));
                }
            };
            if ctx.env.lookup(name).is_some() {
                return Err(EvalError::new(
                    ErrorCategory::Type,
                    format!("`{name}` is a value, not callable"),
                    callee.span(),
                ));
            }
            let builtin = builtin_lookup(name).ok_or_else(|| {
                EvalError::new(
                    ErrorCategory::Name,
                    format!("unknown identifier `{name}`"),
                    callee.span(),
                )
            })?;

            let mut arg_values = Vec::with_capacity(args.len());
            for a in args {
                let v = require_value(eval_expr(a, ctx)?, a.span())?;
                arg_values.push(v);
            }

            (builtin.func)(ctx, &arg_values, *span)
        }
    }
}

fn require_value(v: Option<Value>, span: Span) -> Result<Value, EvalError> {
    v.ok_or_else(|| EvalError::new(ErrorCategory::Type, "expected a value".into(), span))
}

fn eval_binary(op: BinOp, lhs: Value, rhs: Value, span: Span) -> Result<Value, EvalError> {
    use BinOp::*;
    use Value::*;
    match (op, lhs, rhs) {
        (Add, Int(a), Int(b)) => a
            .checked_add(b)
            .map(Int)
            .ok_or_else(|| EvalError::new(ErrorCategory::Runtime, "integer overflow".into(), span)),
        (Sub, Int(a), Int(b)) => a
            .checked_sub(b)
            .map(Int)
            .ok_or_else(|| EvalError::new(ErrorCategory::Runtime, "integer overflow".into(), span)),
        (Mul, Int(a), Int(b)) => a
            .checked_mul(b)
            .map(Int)
            .ok_or_else(|| EvalError::new(ErrorCategory::Runtime, "integer overflow".into(), span)),
        (Div, Int(a), Int(b)) => {
            if b == 0 {
                Err(EvalError::new(
                    ErrorCategory::Runtime,
                    "division by zero".into(),
                    span,
                ))
            } else {
                Ok(Float(a as f64 / b as f64))
            }
        }
        (Add, Float(a), Float(b)) => Ok(Float(a + b)),
        (Sub, Float(a), Float(b)) => Ok(Float(a - b)),
        (Mul, Float(a), Float(b)) => Ok(Float(a * b)),
        (Div, Float(a), Float(b)) => Ok(Float(a / b)),

        (op, Int(a), Float(b)) => eval_binary(op, Float(a as f64), Float(b), span),
        (op, Float(a), Int(b)) => eval_binary(op, Float(a), Float(b as f64), span),

        (Add, String(a), String(b)) => Ok(String(a + &b)),

        (op, l, r) => Err(EvalError::new(
            ErrorCategory::Type,
            format!("`{op:?}` not defined for {l:?} and {r:?}"),
            span,
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

    pub fn with_cause(
        category: ErrorCategory,
        message: String,
        span: Span,
        cause: Box<dyn Error + Send + Sync>,
    ) -> EvalError {
        EvalError {
            category,
            message,
            span,
            cause: Some(cause),
        }
    }
}

struct Builtin {
    name: &'static str,
    func: fn(&mut EvalCtx, &[Value], Span) -> Result<Option<Value>, EvalError>,
}

const BUILTINS: &[Builtin] = &[Builtin {
    name: "print",
    func: builtin_print,
}];

fn builtin_lookup(name: &str) -> Option<&'static Builtin> {
    BUILTINS.iter().find(|b| b.name == name)
}

fn builtin_is_name(name: &str) -> bool {
    BUILTINS.iter().any(|b| b.name == name)
}

fn builtin_print(
    ctx: &mut EvalCtx,
    args: &[Value],
    span: Span,
) -> Result<Option<Value>, EvalError> {
    let mut first = true;
    for v in args {
        if !first {
            ctx.out.write_all(b" ").map_err(|e| io_err(e, span))?;
        }
        first = false;
        write_value(ctx.out, v).map_err(|e| io_err(e, span))?;
    }
    ctx.out.write_all(b"\n").map_err(|e| io_err(e, span))?;
    Ok(None)
}

fn write_value(w: &mut dyn std::io::Write, v: &Value) -> std::io::Result<()> {
    match v {
        Value::Int(int) => write!(w, "{int}"),
        Value::Float(float) => write!(w, "{float}"),
        Value::String(string) => w.write_all(string.as_bytes()),
    }
}

fn io_err(e: std::io::Error, span: Span) -> EvalError {
    EvalError::with_cause(
        ErrorCategory::Runtime,
        format!("io error: {e}"),
        span,
        Box::new(e),
    )
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

    use crate::parser::BinOp;

    fn bin(op: BinOp, lhs: Expr, rhs: Expr) -> Expr {
        Expr::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            span: no_span(),
        }
    }

    fn eval_to_value(expr: Expr) -> Result<Value, EvalError> {
        let mut buf = Vec::<u8>::new();
        let mut ctx = EvalCtx {
            env: Env::new(),
            out: &mut buf,
        };
        eval_expr(&expr, &mut ctx).map(|o| o.unwrap())
    }

    #[test]
    fn add_int_int() {
        assert!(matches!(
            eval_to_value(bin(
                BinOp::Add,
                Expr::Int(2, no_span()),
                Expr::Int(3, no_span())
            )),
            Ok(Value::Int(5))
        ));
    }

    #[test]
    fn add_int_float_promotes() {
        assert!(matches!(
            eval_to_value(bin(BinOp::Add, Expr::Int(2, no_span()), Expr::Float(1.5, no_span()))),
            Ok(Value::Float(v)) if (v - 3.5).abs() < 1e-9
        ));
    }

    #[test]
    fn div_int_int_is_float() {
        assert!(matches!(
            eval_to_value(bin(BinOp::Div, Expr::Int(1, no_span()), Expr::Int(2, no_span()))),
            Ok(Value::Float(v)) if (v - 0.5).abs() < 1e-9
        ));
    }

    #[test]
    fn string_plus_string_concats() {
        let expr = bin(
            BinOp::Add,
            Expr::String("hello ".into(), no_span()),
            Expr::String("world".into(), no_span()),
        );
        match eval_to_value(expr) {
            Ok(Value::String(s)) => assert_eq!(s, "hello world"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn string_plus_int_is_type_error() {
        let expr = bin(
            BinOp::Add,
            Expr::String("x".into(), no_span()),
            Expr::Int(1, no_span()),
        );
        assert!(matches!(
            eval_to_value(expr).unwrap_err().category,
            ErrorCategory::Type
        ));
    }

    #[test]
    fn int_overflow_traps() {
        let expr = bin(
            BinOp::Add,
            Expr::Int(i64::MAX, no_span()),
            Expr::Int(1, no_span()),
        );
        let err = eval_to_value(expr).unwrap_err();
        assert!(matches!(err.category, ErrorCategory::Runtime));
        assert!(err.message.contains("overflow"));
    }

    #[test]
    fn int_div_by_zero() {
        let expr = bin(BinOp::Div, Expr::Int(1, no_span()), Expr::Int(0, no_span()));
        let err = eval_to_value(expr).unwrap_err();
        assert!(matches!(err.category, ErrorCategory::Runtime));
    }

    #[test]
    fn call_unknown_callee_is_name_error() {
        let stmts = vec![Stmt::Expr(Expr::Call {
            callee: Box::new(ident("nope")),
            args: vec![],
            span: no_span(),
        })];
        let err = run_stmts(&stmts).unwrap_err();
        assert!(matches!(err.category, ErrorCategory::Name));
    }
}

use std::{collections::HashMap, error::Error};

use crate::lexer::Span;
use crate::parser::{BinOp, Expr, Stmt};

#[derive(Debug, Clone)]
enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<Value>),
    Object(Vec<(String, Value)>),
    Null,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(left), Self::String(right)) => left == right,
            (Self::Int(left), Self::Int(right)) => left == right,
            (Self::Float(left), Self::Float(right)) => left == right,
            (Self::Int(left), Self::Float(right)) => (*left as f64) == *right,
            (Self::Float(left), Self::Int(right)) => *left == (*right as f64),
            (Self::Bool(left), Self::Bool(right)) => left == right,
            (Self::Null, Self::Null) => true,
            (Self::List(left), Self::List(right)) => {
                if left.len() != right.len() {
                    return false;
                }
                for (idx, item) in left.iter().enumerate() {
                    if right[idx] != *item {
                        return false;
                    }
                }
                true
            }
            (Self::Object(left), Self::Object(right)) => {
                if left.len() != right.len() {
                    return false;
                }

                for item in left.iter() {
                    if !right.iter().any(|i| i == item) {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
struct Binding {
    value: Value,
    is_const: bool,
}

#[derive(Debug)]
enum ReassignError {
    Unknown,
    Const,
}

#[derive(Debug)]
pub struct Env {
    scopes: Vec<HashMap<String, Binding>>,
}

impl Env {
    pub fn new() -> Env {
        let mut scopes = Vec::new();
        scopes.push(HashMap::new());
        Env { scopes }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() != 1 {
            self.scopes.pop();
        }
    }

    fn define(&mut self, name: String, value: Value, is_const: bool) {
        let top = self.scopes.last_mut().expect("env always has root scope");
        top.insert(name, Binding { value, is_const });
    }

    fn reassign(&mut self, name: &str, value: Value) -> Result<(), ReassignError> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(v) = scope.get_mut(name) {
                if v.is_const {
                    return Err(ReassignError::Const);
                } else {
                    v.value = value;
                    return Ok(());
                }
            }
        }
        Err(ReassignError::Unknown)
    }

    fn lookup(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(&v.value);
            }
        }
        None
    }
}

impl Default for Env {
    fn default() -> Env {
        Env::new()
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
        Stmt::Block { stmts, .. } => {
            ctx.env.push_scope();
            let result = (|| {
                for stmt in stmts {
                    eval_stmt(stmt, ctx)?;
                }
                Ok(())
            })();
            ctx.env.pop_scope();
            result
        }
        Stmt::Reassign { name, value, span } => {
            let v = require_value(eval_expr(value, ctx)?, value.span())?;
            match ctx.env.reassign(name, v) {
                Ok(_) => Ok(()),
                Err(ReassignError::Unknown) => Err(EvalError::new(
                    ErrorCategory::Name,
                    format!("unknown identifier `{name}`"),
                    *span,
                )),
                Err(ReassignError::Const) => Err(EvalError::new(
                    ErrorCategory::Name,
                    format!("cannot reassign const `{name}`"),
                    *span,
                )),
            }
        }
        Stmt::If {
            cond,
            then_block,
            else_branch,
            ..
        } => {
            let v = require_value(eval_expr(cond, ctx)?, cond.span())?;
            match v {
                Value::Bool(true) => eval_stmt(then_block, ctx),
                Value::Bool(false) => match else_branch {
                    Some(branch) => eval_stmt(branch, ctx),
                    None => Ok(()),
                },
                other => Err(EvalError::new(
                    ErrorCategory::Type,
                    format!("`if` condition must be Bool, got {other:?}"),
                    cond.span(),
                )),
            }
        }
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
        Expr::Binary {
            op: BinOp::And,
            lhs,
            rhs,
            ..
        } => {
            let l = require_value(eval_expr(lhs, ctx)?, lhs.span())?;
            match l {
                Value::Bool(false) => {
                    check_names(rhs, ctx)?;
                    Ok(Some(Value::Bool(false)))
                }
                Value::Bool(true) => {
                    let r = require_value(eval_expr(rhs, ctx)?, rhs.span())?;
                    match r {
                        Value::Bool(b) => Ok(Some(Value::Bool(b))),
                        other => Err(EvalError::new(
                            ErrorCategory::Type,
                            format!("`and` requires Bool, got {other:?}"),
                            rhs.span(),
                        )),
                    }
                }
                other => Err(EvalError::new(
                    ErrorCategory::Type,
                    format!("`and` requires Bool, got {other:?}"),
                    lhs.span(),
                )),
            }
        }
        Expr::Binary {
            op: BinOp::Or,
            lhs,
            rhs,
            ..
        } => {
            let l = require_value(eval_expr(lhs, ctx)?, lhs.span())?;
            match l {
                Value::Bool(true) => {
                    check_names(rhs, ctx)?;
                    Ok(Some(Value::Bool(true)))
                }
                Value::Bool(false) => {
                    let r = require_value(eval_expr(rhs, ctx)?, rhs.span())?;
                    match r {
                        Value::Bool(b) => Ok(Some(Value::Bool(b))),
                        other => Err(EvalError::new(
                            ErrorCategory::Type,
                            format!("`or` requires Bool, got {other:?}"),
                            rhs.span(),
                        )),
                    }
                }
                other => Err(EvalError::new(
                    ErrorCategory::Type,
                    format!("`or` requires Bool, got {other:?}"),
                    lhs.span(),
                )),
            }
        }
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
            (builtin.func)(BuiltinCall {
                ctx,
                args: &arg_values,
                span: *span,
            })
        }
        Expr::Bool(b, _) => Ok(Some(Value::Bool(*b))),
        Expr::List { items, .. } => {
            let mut values = Vec::with_capacity(items.len());

            for item in items {
                values.push(require_value(eval_expr(item, ctx)?, item.span())?);
            }

            Ok(Some(Value::List(values)))
        }
        Expr::Object { entries, .. } => {
            let mut values = Vec::with_capacity(entries.len());

            for (k, v) in entries {
                let field = require_value(eval_expr(v, ctx)?, v.span())?;
                values.push((k.clone(), field));
            }

            Ok(Some(Value::Object(values)))
        }
        Expr::Null(_) => Ok(Some(Value::Null)),
        Expr::Not { inner, span } => {
            let v = require_value(eval_expr(inner, ctx)?, inner.span())?;
            match v {
                Value::Bool(b) => Ok(Some(Value::Bool(!b))),
                other => Err(EvalError::new(
                    ErrorCategory::Type,
                    format!("`not` requires Bool, got {other:?}"),
                    *span,
                )),
            }
        }
        Expr::Member { recv, field, span } => {
            let v = require_value(eval_expr(recv, ctx)?, recv.span())?;

            match v {
                Value::Object(entries) => {
                    if let Some(e) = entries.iter().find(|e| e.0 == *field) {
                        Ok(Some(e.1.clone()))
                    } else {
                        Err(EvalError::new(
                            ErrorCategory::Name,
                            format!("object has no field `{field}`"),
                            *span,
                        ))
                    }
                }
                _ => Err(EvalError::new(
                    ErrorCategory::Type,
                    format!("cannot access field `{field}` on {}", type_name(&v)),
                    *span,
                )),
            }
        }
        Expr::Index { recv, key, span } => {
            let r = require_value(eval_expr(recv, ctx)?, recv.span())?;
            let k = require_value(eval_expr(key, ctx)?, key.span())?;
            match (r, k) {
                (Value::Object(entries), Value::String(field)) => {
                    for (k, v) in entries {
                        if k == field {
                            return Ok(Some(v));
                        }
                    }
                    Err(EvalError::new(
                        ErrorCategory::Name,
                        format!("object has no key `{field}`"),
                        *span,
                    ))
                }
                (Value::Object(_), other) => Err(EvalError::new(
                    ErrorCategory::Type,
                    format!("object key must be String, got {}", type_name(&other)),
                    *span,
                )),
                (Value::List(entries), Value::Int(idx)) => {
                    if idx >= 0 && (idx as usize) < entries.len() {
                        Ok(Some(entries[idx as usize].clone()))
                    } else {
                        Err(EvalError::new(
                            ErrorCategory::Runtime,
                            format!("list index {idx} out of bounds (length {})", entries.len()),
                            *span,
                        ))
                    }
                }
                (Value::List(_), other) => Err(EvalError::new(
                    ErrorCategory::Type,
                    format!("list index must be Int, got {}", type_name(&other)),
                    *span,
                )),
                (other, _) => Err(EvalError::new(
                    ErrorCategory::Type,
                    format!("cannot index {}", type_name(&other)),
                    *span,
                )),
            }
        }
    }
}

fn check_names(expr: &Expr, ctx: &EvalCtx) -> Result<(), EvalError> {
    match expr {
        Expr::Identifier(identifier, span) => {
            if ctx.env.lookup(identifier).is_none() && !builtin_is_name(identifier) {
                return Err(EvalError::new(
                    ErrorCategory::Name,
                    format!("unknown identifier `{identifier}`"),
                    *span,
                ));
            }
            Ok(())
        }
        Expr::Call { callee, args, .. } => {
            check_names(callee, ctx)?;
            for a in args {
                check_names(a, ctx)?;
            }
            Ok(())
        }
        Expr::Binary { lhs, rhs, .. } => {
            check_names(lhs, ctx)?;
            check_names(rhs, ctx)
        }
        Expr::Not { inner, .. } => check_names(inner, ctx),
        Expr::Int(_, _)
        | Expr::Float(_, _)
        | Expr::String(_, _)
        | Expr::Bool(_, _)
        | Expr::Null(_) => Ok(()),
        _ => todo!(),
    }
}

fn require_value(v: Option<Value>, span: Span) -> Result<Value, EvalError> {
    v.ok_or_else(|| EvalError::new(ErrorCategory::Type, "expected a value".into(), span))
}

fn eval_binary(op: BinOp, lhs: Value, rhs: Value, span: Span) -> Result<Value, EvalError> {
    use BinOp::*;
    use Value::*;
    match (op, lhs, rhs) {
        (Eq, l, r) => Ok(Bool(l == r)),
        (NotEq, l, r) => Ok(Bool(l != r)),

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

        (Add, String(a), String(b)) => Ok(String(a + &b)),

        (Lt, Int(a), Int(b)) => Ok(Bool(a < b)),
        (Lt, Float(a), Float(b)) => Ok(Bool(a < b)),
        (Lt, String(a), String(b)) => Ok(Bool(a < b)),

        (LtEq, Int(a), Int(b)) => Ok(Bool(a <= b)),
        (LtEq, Float(a), Float(b)) => Ok(Bool(a <= b)),
        (LtEq, String(a), String(b)) => Ok(Bool(a <= b)),

        (Gt, Int(a), Int(b)) => Ok(Bool(a > b)),
        (Gt, Float(a), Float(b)) => Ok(Bool(a > b)),
        (Gt, String(a), String(b)) => Ok(Bool(a > b)),

        (GtEq, Int(a), Int(b)) => Ok(Bool(a >= b)),
        (GtEq, Float(a), Float(b)) => Ok(Bool(a >= b)),
        (GtEq, String(a), String(b)) => Ok(Bool(a >= b)),

        (op, Int(a), Float(b)) => eval_binary(op, Float(a as f64), Float(b), span),
        (op, Float(a), Int(b)) => eval_binary(op, Float(a), Float(b as f64), span),

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

struct BuiltinCall<'a, 'w> {
    ctx: &'a mut EvalCtx<'w>,
    args: &'a [Value],
    span: Span,
}

struct Builtin {
    name: &'static str,
    func: fn(BuiltinCall<'_, '_>) -> Result<Option<Value>, EvalError>,
}

const BUILTINS: &[Builtin] = &[
    Builtin {
        name: "print",
        func: builtin_print,
    },
    Builtin {
        name: "string",
        func: builtin_string,
    },
    Builtin {
        name: "int",
        func: builtin_int,
    },
    Builtin {
        name: "float",
        func: builtin_float,
    },
    Builtin {
        name: "number",
        func: builtin_number,
    },
];

fn builtin_lookup(name: &str) -> Option<&'static Builtin> {
    BUILTINS.iter().find(|b| b.name == name)
}

fn builtin_is_name(name: &str) -> bool {
    BUILTINS.iter().any(|b| b.name == name)
}

fn builtin_print(call: BuiltinCall) -> Result<Option<Value>, EvalError> {
    let mut first = true;
    for v in call.args {
        if !first {
            call.ctx
                .out
                .write_all(b" ")
                .map_err(|e| io_err(e, call.span))?;
        }
        first = false;
        write_value(call.ctx.out, v).map_err(|e| io_err(e, call.span))?;
    }
    call.ctx
        .out
        .write_all(b"\n")
        .map_err(|e| io_err(e, call.span))?;
    Ok(None)
}

fn builtin_string(call: BuiltinCall) -> Result<Option<Value>, EvalError> {
    if call.args.len() != 1 {
        return Err(EvalError::new(
            ErrorCategory::Type,
            format!("`string` takes 1 argument, got {}", call.args.len()),
            call.span,
        ));
    }

    let mut buf = Vec::<u8>::new();
    write_value(&mut buf, &call.args[0]).map_err(|e| io_err(e, call.span))?;
    let s = String::from_utf8(buf).expect("write_value emits valid utf8");
    Ok(Some(Value::String(s)))
}

fn builtin_int(call: BuiltinCall) -> Result<Option<Value>, EvalError> {
    if call.args.len() != 1 {
        return Err(EvalError::new(
            ErrorCategory::Type,
            format!("`int` takes 1 argument, got {}", call.args.len()),
            call.span,
        ));
    }

    let int = match &call.args[0] {
        Value::Int(int) => *int,
        Value::Float(float) => {
            if float.is_infinite() {
                return Err(EvalError::new(
                    ErrorCategory::Runtime,
                    format!("cannot convert `{float}` to int"),
                    call.span,
                ));
            }
            let trunc = float.trunc();
            // NOTE: `i64::MAX as f64` rounds up to i64::MAX + 1
            if trunc >= i64::MAX as f64 || trunc < i64::MIN as f64 {
                return Err(EvalError::new(
                    ErrorCategory::Runtime,
                    format!("float `{float:?}` is out of range for int (after f64 rounding)"),
                    call.span,
                ));
            }
            trunc as i64
        }
        Value::String(string) => string.parse::<i64>().map_err(|e| {
            EvalError::with_cause(
                ErrorCategory::Type,
                format!("cannot parse `{string}` as int"),
                call.span,
                Box::new(e),
            )
        })?,
        other => {
            return Err(EvalError::new(
                ErrorCategory::Type,
                format!("`int` does not accept {other:?}"),
                call.span,
            ));
        }
    };
    Ok(Some(Value::Int(int)))
}

fn builtin_float(call: BuiltinCall) -> Result<Option<Value>, EvalError> {
    if call.args.len() != 1 {
        return Err(EvalError::new(
            ErrorCategory::Type,
            format!("`float` takes 1 argument, got {}", call.args.len()),
            call.span,
        ));
    }

    let float = match &call.args[0] {
        Value::Int(int) => *int as f64,
        Value::Float(float) => *float,
        Value::String(string) => string.parse::<f64>().map_err(|e| {
            EvalError::with_cause(
                ErrorCategory::Type,
                format!("cannot parse `{string}` as float"),
                call.span,
                Box::new(e),
            )
        })?,
        other => {
            return Err(EvalError::new(
                ErrorCategory::Type,
                format!("`float` does not accept {other:?}"),
                call.span,
            ));
        }
    };
    Ok(Some(Value::Float(float)))
}

fn builtin_number(call: BuiltinCall) -> Result<Option<Value>, EvalError> {
    if call.args.len() != 1 {
        return Err(EvalError::new(
            ErrorCategory::Type,
            format!("`float` takes 1 argument, got {}", call.args.len()),
            call.span,
        ));
    }

    let number = match &call.args[0] {
        Value::Int(int) => Value::Int(*int),
        Value::Float(float) => Value::Float(*float),
        Value::String(string) => {
            if let Ok(n) = string.parse::<i64>() {
                Value::Int(n)
            } else if let Ok(n) = string.parse::<f64>() {
                Value::Float(n)
            } else {
                return Err(EvalError::new(
                    ErrorCategory::Type,
                    format!("cannot parse `{string}` as number"),
                    call.span,
                ));
            }
        }
        other => {
            return Err(EvalError::new(
                ErrorCategory::Type,
                format!("`number` does not accept {other:?}"),
                call.span,
            ));
        }
    };
    Ok(Some(number))
}

fn write_value(w: &mut dyn std::io::Write, v: &Value) -> std::io::Result<()> {
    write_value_ctx(w, v, false)
}

fn write_value_ctx(
    w: &mut dyn std::io::Write,
    v: &Value,
    in_compound: bool,
) -> std::io::Result<()> {
    match v {
        Value::Int(int) => write!(w, "{int}"),
        Value::Float(float) => write!(w, "{float:?}"),
        Value::String(string) => {
            if in_compound {
                let s = string.replace('\\', "\\\\").replace('"', "\\\"");
                write!(w, "\"{s}\"")
            } else {
                w.write_all(string.as_bytes())
            }
        }
        Value::Bool(bool) => write!(w, "{bool}"),
        Value::List(items) => {
            write!(w, "[")?;
            for (idx, item) in items.iter().enumerate() {
                if idx > 0 {
                    write!(w, ", ")?;
                }
                write_value_ctx(w, item, true)?;
            }
            write!(w, "]")
        }
        Value::Object(object) => {
            write!(w, "{{")?;
            for (idx, key) in object.iter().enumerate() {
                if idx > 0 {
                    write!(w, ", ")?;
                }
                write_object_key(w, &key.0)?;
                write!(w, ": ")?;
                write_value_ctx(w, &key.1, true)?;
            }
            write!(w, "}}")
        }
        Value::Null => w.write_all(b"null"),
    }
}

fn write_object_key(w: &mut dyn std::io::Write, key: &str) -> std::io::Result<()> {
    let first = key.chars().next();
    let is_bare = !is_keyword(key)
        && first
            .map(|c| c.is_ascii_alphabetic() || c == '_')
            .unwrap_or(false)
        && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if is_bare {
        write!(w, "{key}")
    } else {
        let s = key.replace('\\', "\\\\").replace('"', "\\\"");
        write!(w, "\"{s}\"")
    }
}

fn is_keyword(string: &str) -> bool {
    matches!(
        string,
        "or" | "and"
            | "not"
            | "var"
            | "const"
            | "if"
            | "else"
            | "true"
            | "false"
            | "return"
            | "for"
            | "in"
            | "null"
            | "fn"
    )
}

fn type_name(v: &Value) -> &'static str {
    match v {
        Value::String(_) => "String",
        Value::Int(_) => "Int",
        Value::Float(_) => "Float",
        Value::Bool(_) => "Bool",
        Value::List(_) => "List",
        Value::Object(_) => "Object",
        Value::Null => "Null",
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

    #[test]
    fn eval_bool_literal() {
        let expr = Expr::Bool(true, no_span());
        assert!(matches!(eval_to_value(expr), Ok(Value::Bool(true))));
    }

    #[test]
    fn eval_null_literal() {
        let expr = Expr::Null(no_span());
        assert!(matches!(eval_to_value(expr), Ok(Value::Null)));
    }

    #[test]
    fn eval_not_true_is_false() {
        let expr = Expr::Not {
            inner: Box::new(Expr::Bool(true, no_span())),
            span: no_span(),
        };
        assert!(matches!(eval_to_value(expr), Ok(Value::Bool(false))));
    }

    #[test]
    fn eval_not_non_bool_is_type_error() {
        let expr = Expr::Not {
            inner: Box::new(Expr::Int(1, no_span())),
            span: no_span(),
        };
        assert!(matches!(
            eval_to_value(expr).unwrap_err().category,
            ErrorCategory::Type
        ));
    }

    #[test]
    fn lt_int_int() {
        let expr = bin(BinOp::Lt, Expr::Int(1, no_span()), Expr::Int(2, no_span()));
        assert!(matches!(eval_to_value(expr), Ok(Value::Bool(true))));
    }

    #[test]
    fn eq_null_null() {
        let expr = bin(BinOp::Eq, Expr::Null(no_span()), Expr::Null(no_span()));
        assert!(matches!(eval_to_value(expr), Ok(Value::Bool(true))));
    }

    #[test]
    fn eq_cross_kind_is_false() {
        let expr = bin(
            BinOp::Eq,
            Expr::Int(1, no_span()),
            Expr::String("1".into(), no_span()),
        );
        assert!(matches!(eval_to_value(expr), Ok(Value::Bool(false))));
    }

    #[test]
    fn lt_null_is_type_error() {
        let expr = bin(BinOp::Lt, Expr::Null(no_span()), Expr::Int(1, no_span()));
        assert!(matches!(
            eval_to_value(expr).unwrap_err().category,
            ErrorCategory::Type
        ));
    }

    #[test]
    fn and_bool_bool() {
        let expr = bin(
            BinOp::And,
            Expr::Bool(true, no_span()),
            Expr::Bool(false, no_span()),
        );
        assert!(matches!(eval_to_value(expr), Ok(Value::Bool(false))));
    }

    #[test]
    fn and_non_bool_rhs_is_type_error() {
        let expr = bin(
            BinOp::And,
            Expr::Bool(true, no_span()),
            Expr::Int(1, no_span()),
        );
        assert!(matches!(
            eval_to_value(expr).unwrap_err().category,
            ErrorCategory::Type
        ));
    }

    #[test]
    fn or_short_circuits_skips_rhs_eval() {
        let expr = bin(
            BinOp::Or,
            Expr::Bool(true, no_span()),
            Expr::Int(1, no_span()),
        );
        assert!(matches!(eval_to_value(expr), Ok(Value::Bool(true))));
    }

    #[test]
    fn or_short_circuit_still_checks_names_in_rhs() {
        let expr = bin(BinOp::Or, Expr::Bool(true, no_span()), ident("nope"));
        assert!(matches!(
            eval_to_value(expr).unwrap_err().category,
            ErrorCategory::Name
        ));
    }

    fn block_stmt(stmts: Vec<Stmt>) -> Stmt {
        Stmt::Block {
            stmts,
            span: no_span(),
        }
    }

    #[test]
    fn block_runs_inner_stmts() {
        let stmts = vec![block_stmt(vec![
            var_stmt("x", Expr::Int(1, no_span())),
            Stmt::Expr(ident("x")),
        ])];
        run_stmts(&stmts).unwrap();
    }

    #[test]
    fn block_inner_var_does_not_leak() {
        let stmts = vec![
            block_stmt(vec![var_stmt("x", Expr::Int(1, no_span()))]),
            Stmt::Expr(ident("x")),
        ];
        let err = run_stmts(&stmts).unwrap_err();
        assert!(matches!(err.category, ErrorCategory::Name));
    }

    #[test]
    fn block_shadows_outer() {
        let stmts = vec![
            var_stmt("x", Expr::Int(1, no_span())),
            block_stmt(vec![
                var_stmt("x", Expr::Int(2, no_span())),
                Stmt::Expr(ident("x")),
            ]),
            Stmt::Expr(ident("x")),
        ];
        run_stmts(&stmts).unwrap();
    }

    fn reassign_stmt(name: &str, value: Expr) -> Stmt {
        Stmt::Reassign {
            name: name.into(),
            value,
            span: no_span(),
        }
    }

    #[test]
    fn reassign_existing_var() {
        let stmts = vec![
            var_stmt("x", Expr::Int(1, no_span())),
            reassign_stmt("x", Expr::Int(2, no_span())),
            Stmt::Expr(ident("x")),
        ];
        run_stmts(&stmts).unwrap();
    }

    #[test]
    fn reassign_across_types() {
        let stmts = vec![
            var_stmt("x", Expr::Int(1, no_span())),
            reassign_stmt("x", Expr::String("hi".into(), no_span())),
        ];
        run_stmts(&stmts).unwrap();
    }

    #[test]
    fn reassign_unknown_is_name_error() {
        let stmts = vec![reassign_stmt("nope", Expr::Int(1, no_span()))];
        let err = run_stmts(&stmts).unwrap_err();
        assert!(matches!(err.category, ErrorCategory::Name));
    }

    #[test]
    fn reassign_const_is_name_error() {
        let stmts = vec![
            const_stmt("pi", Expr::Float(3.14, no_span())),
            reassign_stmt("pi", Expr::Float(3.0, no_span())),
        ];
        let err = run_stmts(&stmts).unwrap_err();
        assert!(matches!(err.category, ErrorCategory::Name));
        assert!(err.message.contains("const"));
    }

    #[test]
    fn reassign_in_inner_scope_affects_outer() {
        let stmts = vec![
            var_stmt("x", Expr::Int(1, no_span())),
            block_stmt(vec![reassign_stmt("x", Expr::Int(2, no_span()))]),
            Stmt::Expr(ident("x")),
        ];
        run_stmts(&stmts).unwrap();
    }

    fn if_stmt(cond: Expr, then_block: Stmt, else_branch: Option<Stmt>) -> Stmt {
        Stmt::If {
            cond,
            then_block: Box::new(then_block),
            else_branch: else_branch.map(Box::new),
            span: no_span(),
        }
    }

    #[test]
    fn if_true_runs_then() {
        let stmts = vec![
            var_stmt("hit", Expr::Bool(false, no_span())),
            if_stmt(
                Expr::Bool(true, no_span()),
                block_stmt(vec![reassign_stmt("hit", Expr::Bool(true, no_span()))]),
                None,
            ),
            Stmt::Expr(ident("hit")),
        ];
        run_stmts(&stmts).unwrap();
    }

    #[test]
    fn if_false_skips_then() {
        let stmts = vec![if_stmt(
            Expr::Bool(false, no_span()),
            block_stmt(vec![Stmt::Expr(ident("nope"))]),
            None,
        )];
        run_stmts(&stmts).unwrap();
    }

    #[test]
    fn if_false_runs_else() {
        let stmts = vec![if_stmt(
            Expr::Bool(false, no_span()),
            block_stmt(vec![]),
            Some(block_stmt(vec![var_stmt("x", Expr::Int(1, no_span()))])),
        )];
        run_stmts(&stmts).unwrap();
    }

    #[test]
    fn if_non_bool_cond_is_type_error() {
        let stmts = vec![if_stmt(Expr::Int(1, no_span()), block_stmt(vec![]), None)];
        let err = run_stmts(&stmts).unwrap_err();
        assert!(matches!(err.category, ErrorCategory::Type));
    }

    #[test]
    fn if_then_block_has_its_own_scope() {
        let stmts = vec![
            if_stmt(
                Expr::Bool(true, no_span()),
                block_stmt(vec![var_stmt("x", Expr::Int(1, no_span()))]),
                None,
            ),
            Stmt::Expr(ident("x")),
        ];
        let err = run_stmts(&stmts).unwrap_err();
        assert!(matches!(err.category, ErrorCategory::Name));
    }
}

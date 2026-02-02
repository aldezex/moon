use crate::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};
use crate::error::RuntimeError;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Bool(bool),
    String(String),
    Unit,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{i}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Unit => write!(f, "()"),
        }
    }
}

#[derive(Debug, Default)]
pub struct Env {
    vars: HashMap<String, Value>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.vars.get(name)
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.vars.insert(name, value);
    }
}

pub fn eval_program(program: &Program) -> Result<Value, RuntimeError> {
    let mut env = Env::new();
    let mut last = Value::Unit;

    for stmt in &program.stmts {
        last = eval_stmt(stmt, &mut env)?;
    }

    Ok(last)
}

fn eval_stmt(stmt: &Stmt, env: &mut Env) -> Result<Value, RuntimeError> {
    match stmt {
        Stmt::Let { name, expr, .. } => {
            let value = eval_expr(expr, env)?;
            env.set(name.clone(), value);
            Ok(Value::Unit)
        }
        Stmt::Expr { expr, .. } => eval_expr(expr, env),
    }
}

fn eval_expr(expr: &Expr, env: &mut Env) -> Result<Value, RuntimeError> {
    match expr {
        Expr::Int(i, _) => Ok(Value::Int(*i)),
        Expr::Bool(b, _) => Ok(Value::Bool(*b)),
        Expr::String(s, _) => Ok(Value::String(s.clone())),
        Expr::Ident(name, sp) => env.get(name).cloned().ok_or_else(|| RuntimeError {
            message: format!("undefined variable: {name}"),
            span: *sp,
        }),
        Expr::Group { expr, .. } => eval_expr(expr, env),
        Expr::Unary { op, expr, span } => {
            let v = eval_expr(expr, env)?;
            match (op, v) {
                (UnaryOp::Neg, Value::Int(i)) => Ok(Value::Int(-i)),
                (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                (UnaryOp::Neg, other) => Err(RuntimeError {
                    message: format!("cannot apply unary '-' to {other:?}"),
                    span: *span,
                }),
                (UnaryOp::Not, other) => Err(RuntimeError {
                    message: format!("cannot apply unary '!' to {other:?}"),
                    span: *span,
                }),
            }
        }
        Expr::Binary {
            lhs,
            op,
            rhs,
            span,
        } => match op {
            BinaryOp::And => {
                let left = eval_expr(lhs, env)?;
                let lb = match left {
                    Value::Bool(b) => b,
                    other => {
                        return Err(RuntimeError {
                            message: format!("left side of '&&' must be bool, got {other:?}"),
                            span: *span,
                        })
                    }
                };
                if !lb {
                    return Ok(Value::Bool(false));
                }
                let right = eval_expr(rhs, env)?;
                match right {
                    Value::Bool(b) => Ok(Value::Bool(b)),
                    other => Err(RuntimeError {
                        message: format!("right side of '&&' must be bool, got {other:?}"),
                        span: *span,
                    }),
                }
            }
            BinaryOp::Or => {
                let left = eval_expr(lhs, env)?;
                let lb = match left {
                    Value::Bool(b) => b,
                    other => {
                        return Err(RuntimeError {
                            message: format!("left side of '||' must be bool, got {other:?}"),
                            span: *span,
                        })
                    }
                };
                if lb {
                    return Ok(Value::Bool(true));
                }
                let right = eval_expr(rhs, env)?;
                match right {
                    Value::Bool(b) => Ok(Value::Bool(b)),
                    other => Err(RuntimeError {
                        message: format!("right side of '||' must be bool, got {other:?}"),
                        span: *span,
                    }),
                }
            }
            _ => {
                let l = eval_expr(lhs, env)?;
                let r = eval_expr(rhs, env)?;
                eval_binary(*op, l, r, *span)
            }
        },
    }
}

fn eval_binary(
    op: BinaryOp,
    l: Value,
    r: Value,
    span: crate::span::Span,
) -> Result<Value, RuntimeError> {
    use Value::*;

    let err = |message: std::string::String| RuntimeError { message, span };

    match op {
        BinaryOp::Add => match (l, r) {
            (Int(a), Int(b)) => Ok(Int(a + b)),
            (String(a), String(b)) => Ok(String(format!("{a}{b}"))),
            (a, b) => Err(err(format!("cannot add {a:?} and {b:?}"))),
        },
        BinaryOp::Sub => match (l, r) {
            (Int(a), Int(b)) => Ok(Int(a - b)),
            (a, b) => Err(err(format!("cannot subtract {b:?} from {a:?}"))),
        },
        BinaryOp::Mul => match (l, r) {
            (Int(a), Int(b)) => Ok(Int(a * b)),
            (a, b) => Err(err(format!("cannot multiply {a:?} and {b:?}"))),
        },
        BinaryOp::Div => match (l, r) {
            (Int(_), Int(0)) => Err(err("division by zero".to_string())),
            (Int(a), Int(b)) => Ok(Int(a / b)),
            (a, b) => Err(err(format!("cannot divide {a:?} by {b:?}"))),
        },
        BinaryOp::Mod => match (l, r) {
            (Int(_), Int(0)) => Err(err("modulo by zero".to_string())),
            (Int(a), Int(b)) => Ok(Int(a % b)),
            (a, b) => Err(err(format!("cannot modulo {a:?} by {b:?}"))),
        },
        BinaryOp::Eq => Ok(Bool(l == r)),
        BinaryOp::Ne => Ok(Bool(l != r)),
        BinaryOp::Lt => match (l, r) {
            (Int(a), Int(b)) => Ok(Bool(a < b)),
            (a, b) => Err(err(format!("cannot compare {a:?} < {b:?}"))),
        },
        BinaryOp::Le => match (l, r) {
            (Int(a), Int(b)) => Ok(Bool(a <= b)),
            (a, b) => Err(err(format!("cannot compare {a:?} <= {b:?}"))),
        },
        BinaryOp::Gt => match (l, r) {
            (Int(a), Int(b)) => Ok(Bool(a > b)),
            (a, b) => Err(err(format!("cannot compare {a:?} > {b:?}"))),
        },
        BinaryOp::Ge => match (l, r) {
            (Int(a), Int(b)) => Ok(Bool(a >= b)),
            (a, b) => Err(err(format!("cannot compare {a:?} >= {b:?}"))),
        },
        BinaryOp::And | BinaryOp::Or => unreachable!("handled via short-circuit"),
    }
}

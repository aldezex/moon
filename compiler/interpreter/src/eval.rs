use moon_core::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};
use moon_core::span::Span;

use crate::env::Function;
use crate::{Env, RuntimeError, Value};

pub fn eval_program(program: &Program) -> Result<Value, RuntimeError> {
    let mut env = Env::new();

    // Pre-pass: register functions so they can be called before their definition (Rust-style items).
    for stmt in &program.stmts {
        if let Stmt::Fn {
            name, params, body, ..
        } = stmt
        {
            env.define_fn(
                name.clone(),
                Function {
                    params: params.iter().map(|p| p.name.clone()).collect(),
                    body: body.clone(),
                },
            );
        }
    }

    for stmt in &program.stmts {
        eval_stmt(stmt, &mut env)?;
    }

    match &program.tail {
        Some(expr) => eval_expr(expr, &mut env),
        None => Ok(Value::Unit),
    }
}

fn eval_stmt(stmt: &Stmt, env: &mut Env) -> Result<(), RuntimeError> {
    match stmt {
        Stmt::Let { name, expr, .. } => {
            let value = eval_expr(expr, env)?;
            env.define_var(name.clone(), value);
            Ok(())
        }
        Stmt::Assign { target, expr, span } => {
            let value = eval_expr(expr, env)?;
            match target {
                Expr::Ident(name, _) => env.assign_var(name, value).map_err(|()| RuntimeError {
                    message: format!("undefined variable: {name}"),
                    span: *span,
                })?,
                Expr::Index {
                    target: base,
                    index,
                    ..
                } => {
                    let base_v = eval_expr(base, env)?;
                    let index_v = eval_expr(index, env)?;
                    assign_index(env, base_v, index_v, value, *span)?
                }
                _ => {
                    return Err(RuntimeError {
                        message: "invalid assignment target".to_string(),
                        span: *span,
                    })
                }
            }
            Ok(())
        }
        Stmt::Fn { .. } => Ok(()),
        Stmt::Expr { expr, .. } => {
            // Expression statement always discards its value.
            let _ = eval_expr(expr, env)?;
            Ok(())
        }
    }
}

fn eval_expr(expr: &Expr, env: &mut Env) -> Result<Value, RuntimeError> {
    match expr {
        Expr::Int(i, _) => Ok(Value::Int(*i)),
        Expr::Bool(b, _) => Ok(Value::Bool(*b)),
        Expr::String(s, _) => Ok(Value::String(s.clone())),
        Expr::Ident(name, sp) => env.get_var(name).cloned().ok_or_else(|| RuntimeError {
            message: format!("undefined variable: {name}"),
            span: *sp,
        }),
        Expr::Array { elements, .. } => {
            let mut values = Vec::with_capacity(elements.len());
            for e in elements {
                values.push(eval_expr(e, env)?);
            }
            let handle = env.heap.alloc_array(values);
            Ok(Value::Array(handle))
        }
        Expr::Object { props, .. } => {
            let mut map = std::collections::HashMap::new();
            for (k, vexpr) in props {
                let v = eval_expr(vexpr, env)?;
                map.insert(k.clone(), v);
            }
            let handle = env.heap.alloc_object(map);
            Ok(Value::Object(handle))
        }
        Expr::Group { expr, .. } => eval_expr(expr, env),
        Expr::Block { stmts, tail, .. } => {
            env.push_scope();
            let result = (|| {
                for stmt in stmts {
                    eval_stmt(stmt, env)?;
                }
                match tail {
                    Some(expr) => eval_expr(expr, env),
                    None => Ok(Value::Unit),
                }
            })();
            env.pop_scope();
            result
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
            span,
        } => {
            let v = eval_expr(cond, env)?;
            let b = match v {
                Value::Bool(b) => b,
                other => {
                    return Err(RuntimeError {
                        message: format!("if condition must be bool, got {other:?}"),
                        span: *span,
                    })
                }
            };
            if b {
                eval_expr(then_branch, env)
            } else {
                eval_expr(else_branch, env)
            }
        }
        Expr::Call { callee, args, span } => {
            let name = match &**callee {
                Expr::Ident(name, _) => name.as_str(),
                _ => {
                    return Err(RuntimeError {
                        message: "cannot call non-function value".to_string(),
                        span: *span,
                    })
                }
            };

            // Builtins (minimal):
            // - gc(): triggers a GC cycle for heap-allocated objects.
            if name == "gc" {
                if !args.is_empty() {
                    return Err(RuntimeError {
                        message: "gc() takes no arguments".to_string(),
                        span: *span,
                    });
                }
                let roots = env.roots();
                let _ = env.heap.collect_garbage(&roots);
                return Ok(Value::Unit);
            }

            let func = env.get_fn(name).cloned().ok_or_else(|| RuntimeError {
                message: format!("undefined function: {name}"),
                span: *span,
            })?;

            if func.params.len() != args.len() {
                return Err(RuntimeError {
                    message: format!(
                        "wrong number of arguments for {name}: expected {}, got {}",
                        func.params.len(),
                        args.len()
                    ),
                    span: *span,
                });
            }

            let mut values = Vec::with_capacity(args.len());
            for arg in args {
                values.push(eval_expr(arg, env)?);
            }

            // New call frame: only globals + function locals. Caller locals are not visible.
            let saved_scopes = env.take_scopes();
            env.push_scope();
            for (param, value) in func.params.iter().zip(values) {
                env.define_var(param.clone(), value);
            }

            let result = eval_expr(&func.body, env);
            env.restore_scopes(saved_scopes);
            result
        }
        Expr::Index {
            target,
            index,
            span,
        } => {
            let base_v = eval_expr(target, env)?;
            let index_v = eval_expr(index, env)?;
            eval_index(env, base_v, index_v, *span)
        }
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
        Expr::Binary { lhs, op, rhs, span } => match op {
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

fn eval_index(env: &mut Env, base: Value, index: Value, span: Span) -> Result<Value, RuntimeError> {
    match base {
        Value::Array(h) => {
            let idx = match index {
                Value::Int(i) => usize::try_from(i).map_err(|_| RuntimeError {
                    message: "array index must be >= 0".to_string(),
                    span,
                })?,
                other => {
                    return Err(RuntimeError {
                        message: format!("array index must be int, got {other:?}"),
                        span,
                    })
                }
            };
            env.heap
                .array_get(h, idx)
                .cloned()
                .ok_or_else(|| RuntimeError {
                    message: format!("index out of bounds: {idx}"),
                    span,
                })
        }
        Value::Object(h) => {
            let key = match index {
                Value::String(s) => s,
                other => {
                    return Err(RuntimeError {
                        message: format!("object key must be string, got {other:?}"),
                        span,
                    })
                }
            };
            env.heap
                .object_get(h, &key)
                .cloned()
                .ok_or_else(|| RuntimeError {
                    message: format!("missing key: {key}"),
                    span,
                })
        }
        other => Err(RuntimeError {
            message: format!("cannot index into {other:?}"),
            span,
        }),
    }
}

fn assign_index(
    env: &mut Env,
    base: Value,
    index: Value,
    value: Value,
    span: Span,
) -> Result<(), RuntimeError> {
    match base {
        Value::Array(h) => {
            let idx = match index {
                Value::Int(i) => usize::try_from(i).map_err(|_| RuntimeError {
                    message: "array index must be >= 0".to_string(),
                    span,
                })?,
                other => {
                    return Err(RuntimeError {
                        message: format!("array index must be int, got {other:?}"),
                        span,
                    })
                }
            };
            env.heap
                .array_set(h, idx, value)
                .map_err(|e| RuntimeError { message: e, span })
        }
        Value::Object(h) => {
            let key = match index {
                Value::String(s) => s,
                other => {
                    return Err(RuntimeError {
                        message: format!("object key must be string, got {other:?}"),
                        span,
                    })
                }
            };
            env.heap
                .object_set(h, key, value)
                .map_err(|e| RuntimeError { message: e, span })
        }
        other => Err(RuntimeError {
            message: format!("cannot assign through index on {other:?}"),
            span,
        }),
    }
}

fn eval_binary(op: BinaryOp, l: Value, r: Value, span: Span) -> Result<Value, RuntimeError> {
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

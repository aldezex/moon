use moon_core::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};
use moon_core::span::Span;

use crate::env::Function;
use crate::{Env, RuntimeError, Value};

#[derive(Debug, Clone)]
enum Exec {
    Value(Value),
    Return(Value, Span),
}

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
        match eval_stmt(stmt, &mut env)? {
            Exec::Value(_) => {}
            Exec::Return(_, span) => {
                return Err(RuntimeError {
                    message: "return is only allowed inside functions".to_string(),
                    span,
                })
            }
        }
    }

    let result = match &program.tail {
        Some(expr) => eval_expr(expr, &mut env)?,
        None => Exec::Value(Value::Unit),
    };

    match result {
        Exec::Value(v) => Ok(v),
        Exec::Return(_, span) => Err(RuntimeError {
            message: "return is only allowed inside functions".to_string(),
            span,
        }),
    }
}

fn eval_stmt(stmt: &Stmt, env: &mut Env) -> Result<Exec, RuntimeError> {
    match stmt {
        Stmt::Let { name, expr, .. } => {
            let value = match eval_expr(expr, env)? {
                Exec::Value(v) => v,
                Exec::Return(v, sp) => return Ok(Exec::Return(v, sp)),
            };
            env.define_var(name.clone(), value);
            Ok(Exec::Value(Value::Unit))
        }

        Stmt::Assign { target, expr, span } => {
            match target {
                Expr::Ident(name, _) => {
                    let value = match eval_expr(expr, env)? {
                        Exec::Value(v) => v,
                        Exec::Return(v, sp) => return Ok(Exec::Return(v, sp)),
                    };

                    env.assign_var(name, value).map_err(|()| RuntimeError {
                        message: format!("undefined variable: {name}"),
                        span: *span,
                    })?;

                    Ok(Exec::Value(Value::Unit))
                }

                Expr::Index { target, index, .. } => {
                    // Match VM semantics: evaluate base+index before the RHS.
                    let base_v = match eval_expr(target, env)? {
                        Exec::Value(v) => v,
                        Exec::Return(v, sp) => return Ok(Exec::Return(v, sp)),
                    };

                    let index_v = match eval_expr(index, env)? {
                        Exec::Value(v) => v,
                        Exec::Return(v, sp) => return Ok(Exec::Return(v, sp)),
                    };

                    let value = match eval_expr(expr, env)? {
                        Exec::Value(v) => v,
                        Exec::Return(v, sp) => return Ok(Exec::Return(v, sp)),
                    };

                    assign_index(env, base_v, index_v, value, *span)?;
                    Ok(Exec::Value(Value::Unit))
                }

                _ => Err(RuntimeError {
                    message: "invalid assignment target".to_string(),
                    span: *span,
                }),
            }
        }

        Stmt::Return { expr, span } => {
            if let Some(expr) = expr {
                match eval_expr(expr, env)? {
                    Exec::Value(v) => Ok(Exec::Return(v, *span)),
                    Exec::Return(v, sp) => Ok(Exec::Return(v, sp)),
                }
            } else {
                Ok(Exec::Return(Value::Unit, *span))
            }
        }

        Stmt::Fn { .. } => Ok(Exec::Value(Value::Unit)),

        Stmt::Expr { expr, .. } => match eval_expr(expr, env)? {
            Exec::Value(_) => Ok(Exec::Value(Value::Unit)),
            Exec::Return(v, sp) => Ok(Exec::Return(v, sp)),
        },
    }
}

fn eval_expr(expr: &Expr, env: &mut Env) -> Result<Exec, RuntimeError> {
    match expr {
        Expr::Int(i, _) => Ok(Exec::Value(Value::Int(*i))),
        Expr::Bool(b, _) => Ok(Exec::Value(Value::Bool(*b))),
        Expr::String(s, _) => Ok(Exec::Value(Value::String(s.clone()))),
        Expr::Ident(name, sp) => {
            if let Some(v) = env.get_var(name).cloned() {
                return Ok(Exec::Value(v));
            }

            // Functions are values too (like Rust function items). Vars shadow functions.
            if env.get_fn(name).is_some() || name == "gc" {
                return Ok(Exec::Value(Value::Function(name.clone())));
            }

            Err(RuntimeError {
                message: format!("undefined variable: {name}"),
                span: *sp,
            })
        }

        Expr::Array { elements, .. } => {
            let mut values = Vec::with_capacity(elements.len());
            for e in elements {
                match eval_expr(e, env)? {
                    Exec::Value(v) => values.push(v),
                    Exec::Return(v, sp) => return Ok(Exec::Return(v, sp)),
                }
            }
            let handle = env.heap.alloc_array(values);
            Ok(Exec::Value(Value::Array(handle)))
        }

        Expr::Object { props, .. } => {
            let mut map = std::collections::HashMap::new();
            for (k, vexpr) in props {
                let v = match eval_expr(vexpr, env)? {
                    Exec::Value(v) => v,
                    Exec::Return(v, sp) => return Ok(Exec::Return(v, sp)),
                };
                map.insert(k.clone(), v);
            }
            let handle = env.heap.alloc_object(map);
            Ok(Exec::Value(Value::Object(handle)))
        }

        Expr::Group { expr, .. } => eval_expr(expr, env),

        Expr::Block { stmts, tail, .. } => {
            env.push_scope();
            let result = (|| {
                for stmt in stmts {
                    match eval_stmt(stmt, env)? {
                        Exec::Value(_) => {}
                        Exec::Return(v, sp) => return Ok(Exec::Return(v, sp)),
                    }
                }

                match tail {
                    Some(expr) => eval_expr(expr, env),
                    None => Ok(Exec::Value(Value::Unit)),
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
            let v = match eval_expr(cond, env)? {
                Exec::Value(v) => v,
                Exec::Return(v, sp) => return Ok(Exec::Return(v, sp)),
            };

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
            let callee_v = match eval_expr(callee, env)? {
                Exec::Value(v) => v,
                Exec::Return(v, sp) => return Ok(Exec::Return(v, sp)),
            };

            let Value::Function(name) = callee_v else {
                return Err(RuntimeError {
                    message: "cannot call non-function value".to_string(),
                    span: *span,
                });
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
                return Ok(Exec::Value(Value::Unit));
            }

            let func = env.get_fn(&name).cloned().ok_or_else(|| RuntimeError {
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
                match eval_expr(arg, env)? {
                    Exec::Value(v) => values.push(v),
                    Exec::Return(v, sp) => return Ok(Exec::Return(v, sp)),
                }
            }

            // New call frame: only globals + function locals. Caller locals are not visible.
            let saved_scopes = env.take_scopes();
            env.push_scope();
            for (param, value) in func.params.iter().zip(values) {
                env.define_var(param.clone(), value);
            }

            let result = eval_expr(&func.body, env);
            env.restore_scopes(saved_scopes);

            match result? {
                Exec::Value(v) => Ok(Exec::Value(v)),
                Exec::Return(v, _) => Ok(Exec::Value(v)),
            }
        }

        Expr::Index {
            target,
            index,
            span,
        } => {
            let base_v = match eval_expr(target, env)? {
                Exec::Value(v) => v,
                Exec::Return(v, sp) => return Ok(Exec::Return(v, sp)),
            };
            let index_v = match eval_expr(index, env)? {
                Exec::Value(v) => v,
                Exec::Return(v, sp) => return Ok(Exec::Return(v, sp)),
            };
            Ok(Exec::Value(eval_index(env, base_v, index_v, *span)?))
        }

        Expr::Unary { op, expr, span } => {
            let v = match eval_expr(expr, env)? {
                Exec::Value(v) => v,
                Exec::Return(v, sp) => return Ok(Exec::Return(v, sp)),
            };
            match (op, v) {
                (UnaryOp::Neg, Value::Int(i)) => Ok(Exec::Value(Value::Int(-i))),
                (UnaryOp::Not, Value::Bool(b)) => Ok(Exec::Value(Value::Bool(!b))),
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
                let left = match eval_expr(lhs, env)? {
                    Exec::Value(v) => v,
                    Exec::Return(v, sp) => return Ok(Exec::Return(v, sp)),
                };
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
                    return Ok(Exec::Value(Value::Bool(false)));
                }
                let right = match eval_expr(rhs, env)? {
                    Exec::Value(v) => v,
                    Exec::Return(v, sp) => return Ok(Exec::Return(v, sp)),
                };
                match right {
                    Value::Bool(b) => Ok(Exec::Value(Value::Bool(b))),
                    other => Err(RuntimeError {
                        message: format!("right side of '&&' must be bool, got {other:?}"),
                        span: *span,
                    }),
                }
            }
            BinaryOp::Or => {
                let left = match eval_expr(lhs, env)? {
                    Exec::Value(v) => v,
                    Exec::Return(v, sp) => return Ok(Exec::Return(v, sp)),
                };
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
                    return Ok(Exec::Value(Value::Bool(true)));
                }
                let right = match eval_expr(rhs, env)? {
                    Exec::Value(v) => v,
                    Exec::Return(v, sp) => return Ok(Exec::Return(v, sp)),
                };
                match right {
                    Value::Bool(b) => Ok(Exec::Value(Value::Bool(b))),
                    other => Err(RuntimeError {
                        message: format!("right side of '||' must be bool, got {other:?}"),
                        span: *span,
                    }),
                }
            }
            _ => {
                let l = match eval_expr(lhs, env)? {
                    Exec::Value(v) => v,
                    Exec::Return(v, sp) => return Ok(Exec::Return(v, sp)),
                };
                let r = match eval_expr(rhs, env)? {
                    Exec::Value(v) => v,
                    Exec::Return(v, sp) => return Ok(Exec::Return(v, sp)),
                };
                Ok(Exec::Value(eval_binary(*op, l, r, *span)?))
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

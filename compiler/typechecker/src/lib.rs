mod env;
mod error;
mod types;

use moon_core::ast::{BinaryOp, Expr, Program, Stmt, TypeExpr, UnaryOp};

pub use error::TypeError;
pub use types::Type;

use crate::env::TypeEnv;

pub fn check_program(program: &Program) -> Result<Type, TypeError> {
    let mut env = TypeEnv::new();

    // Pass 1: collect function signatures, so calls work regardless of definition order.
    for stmt in &program.stmts {
        if let Stmt::Fn {
            name,
            params,
            ret_ty,
            span,
            ..
        } = stmt
        {
            if env.get_fn(name).is_some() {
                return Err(TypeError {
                    message: format!("duplicate function: {name}"),
                    span: *span,
                });
            }
            let params = params
                .iter()
                .map(|p| lower_type(&p.ty))
                .collect::<Result<Vec<_>, _>>()?;
            let ret = lower_type(ret_ty)?;
            env.define_fn(name.clone(), params, ret)?;
        }
    }

    // Pass 2: typecheck statements in order (strict: vars must be defined before use).
    for stmt in &program.stmts {
        check_stmt(stmt, &mut env)?;
    }

    match &program.tail {
        Some(expr) => check_expr(expr, &mut env),
        None => Ok(Type::Unit),
    }
}

fn check_stmt(stmt: &Stmt, env: &mut TypeEnv) -> Result<(), TypeError> {
    match stmt {
        Stmt::Let {
            name, ty, expr, ..
        } => {
            let expr_ty = check_expr(expr, env)?;
            if let Some(ann) = ty {
                let ann_ty = lower_type(ann)?;
                if ann_ty != expr_ty {
                    return Err(TypeError {
                        message: format!("type mismatch: expected {ann_ty}, got {expr_ty}"),
                        span: ann.span(),
                    });
                }
            }
            env.define_var(name.clone(), expr_ty);
            Ok(())
        }
        Stmt::Fn {
            name,
            params,
            ret_ty,
            body,
            span,
            ..
        } => {
            // Signature already exists from pass 1.
            let sig = env.get_fn(name).cloned().ok_or_else(|| TypeError {
                message: format!("internal error: missing signature for function {name}"),
                span: *span,
            })?;

            let saved = env.take_scopes();
            env.push_scope();
            for (param, ty) in params.iter().zip(sig.params.iter()) {
                env.define_var(param.name.clone(), ty.clone());
            }

            let body_ty = check_expr(body, env);
            env.restore_scopes(saved);

            let body_ty = body_ty?;
            let expected = sig.ret.clone();
            if body_ty != expected {
                return Err(TypeError {
                    message: format!("type mismatch: expected {expected}, got {body_ty}"),
                    span: *span,
                });
            }

            // Also validate that the declared return type is a known type.
            // (We lowered it in pass 1, but this produces a nicer span for errors in the return type.)
            let _ = lower_type(ret_ty)?;
            Ok(())
        }
        Stmt::Expr { expr, .. } => {
            let _ = check_expr(expr, env)?;
            Ok(())
        }
    }
}

fn check_expr(expr: &Expr, env: &mut TypeEnv) -> Result<Type, TypeError> {
    match expr {
        Expr::Int(_, _) => Ok(Type::Int),
        Expr::Bool(_, _) => Ok(Type::Bool),
        Expr::String(_, _) => Ok(Type::String),
        Expr::Ident(name, sp) => env.get_var(name).cloned().ok_or_else(|| TypeError {
            message: format!("undefined variable: {name}"),
            span: *sp,
        }),
        Expr::Group { expr, .. } => check_expr(expr, env),
        Expr::Block { stmts, tail, .. } => {
            env.push_scope();
            let result = (|| {
                for stmt in stmts {
                    check_stmt(stmt, env)?;
                }
                match tail {
                    Some(expr) => check_expr(expr, env),
                    None => Ok(Type::Unit),
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
            let cond_ty = check_expr(cond, env)?;
            if cond_ty != Type::Bool {
                return Err(TypeError {
                    message: format!("if condition must be Bool, got {cond_ty}"),
                    span: *span,
                });
            }
            let then_ty = check_expr(then_branch, env)?;
            let else_ty = check_expr(else_branch, env)?;
            if then_ty != else_ty {
                return Err(TypeError {
                    message: format!("if branches must have the same type: got {then_ty} and {else_ty}"),
                    span: *span,
                });
            }
            Ok(then_ty)
        }
        Expr::Call { callee, args, span } => {
            let name = match &**callee {
                Expr::Ident(name, _) => name.as_str(),
                _ => {
                    return Err(TypeError {
                        message: "can only call functions by name (for now)".to_string(),
                        span: *span,
                    })
                }
            };

            let sig = env.get_fn(name).cloned().ok_or_else(|| TypeError {
                message: format!("undefined function: {name}"),
                span: *span,
            })?;

            if sig.params.len() != args.len() {
                return Err(TypeError {
                    message: format!(
                        "wrong number of arguments for {name}: expected {}, got {}",
                        sig.params.len(),
                        args.len()
                    ),
                    span: *span,
                });
            }

            for (arg_expr, param_ty) in args.iter().zip(sig.params.iter()) {
                let arg_ty = check_expr(arg_expr, env)?;
                if &arg_ty != param_ty {
                    return Err(TypeError {
                        message: format!("argument type mismatch: expected {param_ty}, got {arg_ty}"),
                        span: arg_expr.span(),
                    });
                }
            }

            Ok(sig.ret.clone())
        }
        Expr::Unary { op, expr, span } => {
            let inner = check_expr(expr, env)?;
            match op {
                UnaryOp::Neg => {
                    if inner != Type::Int {
                        return Err(TypeError {
                            message: format!("cannot apply unary '-' to {inner}"),
                            span: *span,
                        });
                    }
                    Ok(Type::Int)
                }
                UnaryOp::Not => {
                    if inner != Type::Bool {
                        return Err(TypeError {
                            message: format!("cannot apply unary '!' to {inner}"),
                            span: *span,
                        });
                    }
                    Ok(Type::Bool)
                }
            }
        }
        Expr::Binary {
            lhs,
            op,
            rhs,
            span,
        } => {
            let l = check_expr(lhs, env)?;
            let r = check_expr(rhs, env)?;
            check_binary(*op, l, r, *span)
        }
    }
}

fn check_binary(op: BinaryOp, l: Type, r: Type, span: moon_core::span::Span) -> Result<Type, TypeError> {
    let err = |message: std::string::String| TypeError { message, span };

    match op {
        BinaryOp::Add => match (&l, &r) {
            (Type::Int, Type::Int) => Ok(Type::Int),
            (Type::String, Type::String) => Ok(Type::String),
            _ => Err(err(format!("cannot add {l} and {r}"))),
        },
        BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
            if l == Type::Int && r == Type::Int {
                Ok(Type::Int)
            } else {
                Err(err(format!("arithmetic operators require Int, got {l} and {r}")))
            }
        }
        BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
            if l == Type::Int && r == Type::Int {
                Ok(Type::Bool)
            } else {
                Err(err(format!("comparison operators require Int, got {l} and {r}")))
            }
        }
        BinaryOp::Eq | BinaryOp::Ne => {
            if l == r {
                Ok(Type::Bool)
            } else {
                Err(err(format!("cannot compare {l} and {r}")))
            }
        }
        BinaryOp::And | BinaryOp::Or => {
            if l == Type::Bool && r == Type::Bool {
                Ok(Type::Bool)
            } else {
                Err(err(format!("logical operators require Bool, got {l} and {r}")))
            }
        }
    }
}

fn lower_type(ty: &TypeExpr) -> Result<Type, TypeError> {
    match ty {
        TypeExpr::Named(name, sp) => match name.as_str() {
            "Int" => Ok(Type::Int),
            "Bool" => Ok(Type::Bool),
            "String" => Ok(Type::String),
            "Unit" => Ok(Type::Unit),
            _ => Err(TypeError {
                message: format!("unknown type: {name}"),
                span: *sp,
            }),
        },
    }
}

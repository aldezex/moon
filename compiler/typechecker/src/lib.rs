mod env;
mod error;
mod types;

use moon_core::ast::{BinaryOp, Expr, Program, Stmt, TypeExpr, UnaryOp};
use moon_core::span::Span;

pub use error::TypeError;
pub use types::Type;

use crate::env::TypeEnv;

#[derive(Debug, Clone)]
pub struct CheckInfo {
    pub ty: Type,
    pub expr_types: Vec<(Span, Type)>,
}

pub fn check_program(program: &Program) -> Result<Type, TypeError> {
    check_program_with_sink(program, &mut ())
}

pub fn check_program_with_spans(program: &Program) -> Result<CheckInfo, TypeError> {
    let mut expr_types = Vec::new();
    let ty = check_program_with_sink(program, &mut expr_types)?;
    Ok(CheckInfo { ty, expr_types })
}

trait TypeSink {
    fn record(&mut self, span: Span, ty: Type);
}

impl TypeSink for () {
    fn record(&mut self, _: Span, _: Type) {}
}

impl TypeSink for Vec<(Span, Type)> {
    fn record(&mut self, span: Span, ty: Type) {
        self.push((span, ty));
    }
}

fn check_program_with_sink<S: TypeSink>(
    program: &Program,
    sink: &mut S,
) -> Result<Type, TypeError> {
    let mut env = TypeEnv::new();

    // Builtins.
    // `gc()` triggers a garbage collection cycle for heap-allocated objects.
    env.define_fn("gc".to_string(), Vec::new(), Type::Unit)?;

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
        let _ = check_stmt(stmt, &mut env, sink, None)?;
    }

    match &program.tail {
        Some(expr) => check_expr(expr, &mut env, sink, None),
        None => Ok(Type::Unit),
    }
}

fn check_stmt<S: TypeSink>(
    stmt: &Stmt,
    env: &mut TypeEnv,
    sink: &mut S,
    current_ret: Option<&Type>,
) -> Result<bool, TypeError> {
    match stmt {
        Stmt::Let { name, ty, expr, .. } => {
            // Minimal contextual typing for empty literals:
            // `let a: Array<Int> = [];` / `let o: Object<Int> = #{}'
            let expr_ty = match (expr, ty) {
                (Expr::Array { elements, .. }, Some(ann)) if elements.is_empty() => {
                    lower_type(ann)?
                }
                (Expr::Object { props, .. }, Some(ann)) if props.is_empty() => lower_type(ann)?,
                _ => check_expr(expr, env, sink, current_ret)?,
            };

            let mut ann_ty: Option<Type> = None;
            if let Some(ann) = ty {
                let t = lower_type(ann)?;
                if !compatible(&t, &expr_ty) {
                    return Err(TypeError {
                        message: format!("type mismatch: expected {t}, got {expr_ty}"),
                        span: ann.span(),
                    });
                }
                ann_ty = Some(t);
            }

            env.define_var(name.clone(), ann_ty.unwrap_or_else(|| expr_ty.clone()));

            // If the initializer diverges, the statement does too.
            Ok(matches!(expr_ty, Type::Never))
        }

        Stmt::Assign { target, expr, span } => match target {
            Expr::Ident(name, sp) => {
                let rhs_ty = check_expr(expr, env, sink, current_ret)?;

                // The VM evaluates the RHS before setting the variable. If the RHS diverges,
                // the assignment never happens.
                if matches!(rhs_ty, Type::Never) {
                    return Ok(true);
                }

                let var_ty = env.get_var(name).cloned().ok_or_else(|| TypeError {
                    message: format!("undefined variable: {name}"),
                    span: *sp,
                })?;

                if !compatible(&var_ty, &rhs_ty) {
                    return Err(TypeError {
                        message: format!("type mismatch: expected {var_ty}, got {rhs_ty}"),
                        span: *span,
                    });
                }

                Ok(false)
            }
            Expr::Index {
                target: base,
                index,
                ..
            } => {
                // The VM evaluates base+index before the RHS.
                let base_ty = check_expr(base, env, sink, current_ret)?;
                if matches!(base_ty, Type::Never) {
                    return Ok(true);
                }

                let index_ty = check_expr(index, env, sink, current_ret)?;
                if matches!(index_ty, Type::Never) {
                    return Ok(true);
                }

                let rhs_ty = check_expr(expr, env, sink, current_ret)?;
                if matches!(rhs_ty, Type::Never) {
                    return Ok(true);
                }

                match base_ty {
                    Type::Array(inner) => {
                        if index_ty != Type::Int {
                            return Err(TypeError {
                                message: format!("array index must be Int, got {index_ty}"),
                                span: *span,
                            });
                        }
                        let inner = *inner;
                        if !compatible(&inner, &rhs_ty) {
                            return Err(TypeError {
                                message: format!("type mismatch: expected {inner}, got {rhs_ty}"),
                                span: *span,
                            });
                        }
                        Ok(false)
                    }
                    Type::Object(inner) => {
                        if index_ty != Type::String {
                            return Err(TypeError {
                                message: format!("object key must be String, got {index_ty}"),
                                span: *span,
                            });
                        }
                        let inner = *inner;
                        if !compatible(&inner, &rhs_ty) {
                            return Err(TypeError {
                                message: format!("type mismatch: expected {inner}, got {rhs_ty}"),
                                span: *span,
                            });
                        }
                        Ok(false)
                    }
                    other => Err(TypeError {
                        message: format!("cannot assign through index on {other}"),
                        span: *span,
                    }),
                }
            }
            _ => Err(TypeError {
                message: "invalid assignment target".to_string(),
                span: *span,
            }),
        },

        Stmt::Return { expr, span } => {
            let Some(expected) = current_ret else {
                return Err(TypeError {
                    message: "return is only allowed inside functions".to_string(),
                    span: *span,
                });
            };

            let got = match expr {
                Some(expr) => check_expr(expr, env, sink, current_ret)?,
                None => Type::Unit,
            };

            if !compatible(expected, &got) {
                return Err(TypeError {
                    message: format!("type mismatch: expected {expected}, got {got}"),
                    span: *span,
                });
            }

            Ok(true)
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

            let expected = sig.ret.clone();

            let saved = env.take_scopes();
            env.push_scope();
            for (param, ty) in params.iter().zip(sig.params.iter()) {
                env.define_var(param.name.clone(), ty.clone());
            }

            let body_ty = check_expr(body, env, sink, Some(&expected));
            env.restore_scopes(saved);

            let body_ty = body_ty?;
            if !compatible(&expected, &body_ty) {
                return Err(TypeError {
                    message: format!("type mismatch: expected {expected}, got {body_ty}"),
                    span: *span,
                });
            }

            // Also validate that the declared return type is a known type.
            // (We lowered it in pass 1, but this produces a nicer span for errors in the return type.)
            let _ = lower_type(ret_ty)?;

            Ok(false)
        }

        Stmt::Expr { expr, .. } => {
            let ty = check_expr(expr, env, sink, current_ret)?;
            Ok(matches!(ty, Type::Never))
        }
    }
}

fn check_expr<S: TypeSink>(
    expr: &Expr,
    env: &mut TypeEnv,
    sink: &mut S,
    current_ret: Option<&Type>,
) -> Result<Type, TypeError> {
    let ty = match expr {
        Expr::Int(_, _) => Type::Int,
        Expr::Bool(_, _) => Type::Bool,
        Expr::String(_, _) => Type::String,
        Expr::Ident(name, sp) => {
            if let Some(ty) = env.get_var(name).cloned() {
                ty
            } else if let Some(sig) = env.get_fn(name) {
                Type::Function {
                    params: sig.params.clone(),
                    ret: Box::new(sig.ret.clone()),
                }
            } else {
                return Err(TypeError {
                    message: format!("undefined variable: {name}"),
                    span: *sp,
                });
            }
        }
        Expr::Fn {
            params,
            ret_ty,
            body,
            span,
        } => {
            let ret = lower_type(ret_ty)?;
            let mut param_tys = Vec::with_capacity(params.len());
            for p in params {
                param_tys.push(lower_type(&p.ty)?);
            }

            env.push_scope();
            for (p, ty) in params.iter().zip(param_tys.iter()) {
                env.define_var(p.name.clone(), ty.clone());
            }
            let body_ty = check_expr(body, env, sink, Some(&ret))?;
            env.pop_scope();

            if !compatible(&ret, &body_ty) {
                return Err(TypeError {
                    message: format!("type mismatch: expected {ret}, got {body_ty}"),
                    span: *span,
                });
            }

            Type::Function {
                params: param_tys,
                ret: Box::new(ret),
            }
        }

        Expr::Array { elements, span } => {
            if elements.is_empty() {
                return Err(TypeError {
                    message: "cannot infer type of empty array; add an annotation".to_string(),
                    span: *span,
                });
            }

            let first = check_expr(&elements[0], env, sink, current_ret)?;
            if matches!(first, Type::Never) {
                return Ok(Type::Never);
            }

            for elem in &elements[1..] {
                let ty = check_expr(elem, env, sink, current_ret)?;
                if matches!(ty, Type::Never) {
                    return Ok(Type::Never);
                }
                if ty != first {
                    return Err(TypeError {
                        message: format!(
                            "array elements must have the same type: got {first} and {ty}"
                        ),
                        span: *span,
                    });
                }
            }

            Type::Array(Box::new(first))
        }

        Expr::Object { props, span } => {
            if props.is_empty() {
                return Err(TypeError {
                    message: "cannot infer type of empty object; add an annotation".to_string(),
                    span: *span,
                });
            }

            let first = check_expr(&props[0].1, env, sink, current_ret)?;
            if matches!(first, Type::Never) {
                return Ok(Type::Never);
            }

            for (_, value) in &props[1..] {
                let ty = check_expr(value, env, sink, current_ret)?;
                if matches!(ty, Type::Never) {
                    return Ok(Type::Never);
                }
                if ty != first {
                    return Err(TypeError {
                        message: format!(
                            "object values must have the same type: got {first} and {ty}"
                        ),
                        span: *span,
                    });
                }
            }

            Type::Object(Box::new(first))
        }

        Expr::Group { expr, .. } => check_expr(expr, env, sink, current_ret)?,

        Expr::Block { stmts, tail, .. } => {
            env.push_scope();
            let result = (|| {
                for stmt in stmts {
                    let diverges = check_stmt(stmt, env, sink, current_ret)?;
                    if diverges {
                        return Ok(Type::Never);
                    }
                }
                match tail {
                    Some(expr) => check_expr(expr, env, sink, current_ret),
                    None => Ok(Type::Unit),
                }
            })();
            env.pop_scope();
            result?
        }

        Expr::If {
            cond,
            then_branch,
            else_branch,
            span,
        } => {
            let cond_ty = check_expr(cond, env, sink, current_ret)?;
            if matches!(cond_ty, Type::Never) {
                return Ok(Type::Never);
            }
            if cond_ty != Type::Bool {
                return Err(TypeError {
                    message: format!("if condition must be Bool, got {cond_ty}"),
                    span: *span,
                });
            }

            let then_ty = check_expr(then_branch, env, sink, current_ret)?;
            let else_ty = check_expr(else_branch, env, sink, current_ret)?;

            if then_ty == else_ty {
                then_ty
            } else if matches!(then_ty, Type::Never) {
                else_ty
            } else if matches!(else_ty, Type::Never) {
                then_ty
            } else {
                return Err(TypeError {
                    message: format!(
                        "if branches must have the same type: got {then_ty} and {else_ty}"
                    ),
                    span: *span,
                });
            }
        }

        Expr::Call { callee, args, span } => {
            let callee_ty = check_expr(callee, env, sink, current_ret)?;
            if matches!(callee_ty, Type::Never) {
                return Ok(Type::Never);
            }

            let (params, ret) = match callee_ty {
                Type::Function { params, ret } => (params, ret),
                other => {
                    return Err(TypeError {
                        message: format!("cannot call non-function value: {other}"),
                        span: *span,
                    })
                }
            };

            if params.len() != args.len() {
                return Err(TypeError {
                    message: format!(
                        "wrong number of arguments: expected {}, got {}",
                        params.len(),
                        args.len()
                    ),
                    span: *span,
                });
            }

            for (arg_expr, param_ty) in args.iter().zip(params.iter()) {
                let arg_ty = check_expr(arg_expr, env, sink, current_ret)?;
                if matches!(arg_ty, Type::Never) {
                    return Ok(Type::Never);
                }
                if !compatible(param_ty, &arg_ty) {
                    return Err(TypeError {
                        message: format!(
                            "argument type mismatch: expected {param_ty}, got {arg_ty}"
                        ),
                        span: arg_expr.span(),
                    });
                }
            }

            *ret
        }

        Expr::Index {
            target,
            index,
            span,
        } => {
            let base = check_expr(target, env, sink, current_ret)?;
            if matches!(base, Type::Never) {
                return Ok(Type::Never);
            }

            let idx = check_expr(index, env, sink, current_ret)?;
            if matches!(idx, Type::Never) {
                return Ok(Type::Never);
            }

            match base {
                Type::Array(inner) => {
                    if idx != Type::Int {
                        return Err(TypeError {
                            message: format!("array index must be Int, got {idx}"),
                            span: *span,
                        });
                    }
                    *inner
                }
                Type::Object(inner) => {
                    if idx != Type::String {
                        return Err(TypeError {
                            message: format!("object key must be String, got {idx}"),
                            span: *span,
                        });
                    }
                    *inner
                }
                other => {
                    return Err(TypeError {
                        message: format!("cannot index into {other}"),
                        span: *span,
                    })
                }
            }
        }

        Expr::Unary { op, expr, span } => {
            let inner = check_expr(expr, env, sink, current_ret)?;
            if matches!(inner, Type::Never) {
                return Ok(Type::Never);
            }
            match op {
                UnaryOp::Neg => {
                    if inner != Type::Int {
                        return Err(TypeError {
                            message: format!("cannot apply unary '-' to {inner}"),
                            span: *span,
                        });
                    }
                    Type::Int
                }
                UnaryOp::Not => {
                    if inner != Type::Bool {
                        return Err(TypeError {
                            message: format!("cannot apply unary '!' to {inner}"),
                            span: *span,
                        });
                    }
                    Type::Bool
                }
            }
        }

        Expr::Binary { lhs, op, rhs, span } => {
            match op {
                BinaryOp::And | BinaryOp::Or => {
                    let l = check_expr(lhs, env, sink, current_ret)?;
                    if matches!(l, Type::Never) {
                        return Ok(Type::Never);
                    }
                    if l != Type::Bool {
                        return Err(TypeError {
                            message: format!("logical operators require Bool, got {l} and ..."),
                            span: *span,
                        });
                    }

                    let r = check_expr(rhs, env, sink, current_ret)?;
                    if matches!(r, Type::Never) {
                        // Short-circuit means the expression can still evaluate to Bool.
                        Type::Bool
                    } else if r == Type::Bool {
                        Type::Bool
                    } else {
                        return Err(TypeError {
                            message: format!("logical operators require Bool, got {l} and {r}"),
                            span: *span,
                        });
                    }
                }
                _ => {
                    let l = check_expr(lhs, env, sink, current_ret)?;
                    if matches!(l, Type::Never) {
                        return Ok(Type::Never);
                    }

                    let r = check_expr(rhs, env, sink, current_ret)?;
                    if matches!(r, Type::Never) {
                        return Ok(Type::Never);
                    }

                    check_binary(*op, l, r, *span)?
                }
            }
        }
    };

    sink.record(expr.span(), ty.clone());
    Ok(ty)
}

fn check_binary(op: BinaryOp, l: Type, r: Type, span: Span) -> Result<Type, TypeError> {
    let err = |message: String| TypeError { message, span };

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
                Err(err(format!(
                    "arithmetic operators require Int, got {l} and {r}"
                )))
            }
        }
        BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
            if l == Type::Int && r == Type::Int {
                Ok(Type::Bool)
            } else {
                Err(err(format!(
                    "comparison operators require Int, got {l} and {r}"
                )))
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
            // And/Or are handled in `check_expr` to model short-circuit + Never.
            Err(err(
                "internal error: unexpected And/Or in check_binary".to_string()
            ))
        }
    }
}

fn compatible(expected: &Type, got: &Type) -> bool {
    expected == got || matches!(got, Type::Never)
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
        TypeExpr::Generic { base, args, span } => match base.as_str() {
            "Array" => {
                if args.len() != 1 {
                    return Err(TypeError {
                        message: "Array<T> expects exactly one type argument".to_string(),
                        span: *span,
                    });
                }
                let inner = lower_type(&args[0])?;
                Ok(Type::Array(Box::new(inner)))
            }
            "Object" => {
                if args.len() != 1 {
                    return Err(TypeError {
                        message: "Object<T> expects exactly one type argument".to_string(),
                        span: *span,
                    });
                }
                let inner = lower_type(&args[0])?;
                Ok(Type::Object(Box::new(inner)))
            }
            _ => Err(TypeError {
                message: format!("unknown type: {base}"),
                span: *span,
            }),
        },
    }
}

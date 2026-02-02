use std::collections::HashMap;

use moon_core::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};
use moon_core::span::Span;
use moon_runtime::Value;

use crate::instr::{Instr, InstrKind};
use crate::module::{FuncId, Function, Module};

#[derive(Debug, Clone)]
pub struct CompileError {
    pub message: String,
    pub span: Span,
}

pub fn compile(program: &Program) -> Result<Module, CompileError> {
    let mut functions: Vec<Function> = Vec::new();
    let mut by_name: HashMap<String, FuncId> = HashMap::new();

    // Reserve main at 0.
    functions.push(Function {
        name: "<main>".to_string(),
        params: Vec::new(),
        code: Vec::new(),
    });
    let main_id = 0usize;

    // Builtins (implemented in the VM).
    // We treat them as functions in the module so they can be called like normal.
    {
        let id = functions.len();
        by_name.insert("gc".to_string(), id);
        functions.push(Function {
            name: "gc".to_string(),
            params: Vec::new(),
            code: Vec::new(),
        });
    }

    // Collect function ids first so calls can refer to functions declared later.
    for stmt in &program.stmts {
        if let Stmt::Fn { name, params, .. } = stmt {
            if by_name.contains_key(name) {
                return Err(CompileError {
                    message: format!("duplicate function: {name}"),
                    span: stmt.span(),
                });
            }
            let id = functions.len();
            by_name.insert(name.clone(), id);
            functions.push(Function {
                name: name.clone(),
                params: params.iter().map(|p| p.name.clone()).collect(),
                code: Vec::new(),
            });
        }
    }

    // Compile each function body.
    for stmt in &program.stmts {
        let Stmt::Fn { name, body, .. } = stmt else {
            continue;
        };
        let id = *by_name.get(name).expect("function id exists");
        let mut code = Vec::new();
        compile_expr(body, &mut code, &by_name)?;
        emit(&mut code, InstrKind::Return, body.span());
        functions[id].code = code;
    }

    // Compile main.
    {
        let mut code = Vec::new();
        compile_stmts(&program.stmts, &mut code, &by_name)?;

        let end_span = program
            .tail
            .as_ref()
            .map(|e| e.span())
            .or_else(|| program.stmts.last().map(|s| s.span()))
            .unwrap_or(Span::new(0, 0));

        match &program.tail {
            Some(expr) => compile_expr(expr, &mut code, &by_name)?,
            None => emit(&mut code, InstrKind::Push(Value::Unit), end_span),
        }
        emit(&mut code, InstrKind::Return, end_span);
        functions[main_id].code = code;
    }

    Ok(Module {
        functions,
        by_name,
        main: main_id,
    })
}

fn compile_stmts(
    stmts: &[Stmt],
    code: &mut Vec<Instr>,
    funcs: &HashMap<String, FuncId>,
) -> Result<(), CompileError> {
    for stmt in stmts {
        match stmt {
            Stmt::Let {
                name, expr, span, ..
            } => {
                compile_expr(expr, code, funcs)?;
                emit(code, InstrKind::DefineVar(name.clone()), *span);
            }
            Stmt::Assign { target, expr, span } => match target {
                Expr::Ident(name, name_span) => {
                    compile_expr(expr, code, funcs)?;
                    emit(code, InstrKind::SetVar(name.clone()), *name_span);
                }
                Expr::Index { target, index, .. } => {
                    compile_expr(target, code, funcs)?;
                    compile_expr(index, code, funcs)?;
                    compile_expr(expr, code, funcs)?;
                    emit(code, InstrKind::IndexSet, *span);
                }
                _ => {
                    return Err(CompileError {
                        message: "invalid assignment target".to_string(),
                        span: *span,
                    })
                }
            },
            Stmt::Return { expr, span } => {
                match expr {
                    Some(expr) => compile_expr(expr, code, funcs)?,
                    None => emit(code, InstrKind::Push(Value::Unit), *span),
                }
                emit(code, InstrKind::Return, *span);
            }
            Stmt::Fn { .. } => {
                // Functions are top-level items. They don't execute in main.
            }
            Stmt::Expr { expr, .. } => {
                compile_expr(expr, code, funcs)?;
                emit(code, InstrKind::Pop, expr.span());
            }
        }
    }
    Ok(())
}

fn compile_expr(
    expr: &Expr,
    code: &mut Vec<Instr>,
    funcs: &HashMap<String, FuncId>,
) -> Result<(), CompileError> {
    match expr {
        Expr::Int(i, span) => emit(code, InstrKind::Push(Value::Int(*i)), *span),
        Expr::Bool(b, span) => emit(code, InstrKind::Push(Value::Bool(*b)), *span),
        Expr::String(s, span) => emit(code, InstrKind::Push(Value::String(s.clone())), *span),
        Expr::Ident(name, span) => emit(code, InstrKind::LoadVar(name.clone()), *span),
        Expr::Group { expr, .. } => return compile_expr(expr, code, funcs),

        Expr::Array { elements, span } => {
            for e in elements {
                compile_expr(e, code, funcs)?;
            }
            emit(code, InstrKind::MakeArray(elements.len()), *span);
        }
        Expr::Object { props, span } => {
            let mut keys = Vec::with_capacity(props.len());
            for (k, v) in props {
                keys.push(k.clone());
                compile_expr(v, code, funcs)?;
            }
            emit(code, InstrKind::MakeObject(keys), *span);
        }
        Expr::Index {
            target,
            index,
            span,
        } => {
            compile_expr(target, code, funcs)?;
            compile_expr(index, code, funcs)?;
            emit(code, InstrKind::IndexGet, *span);
        }

        Expr::Block { stmts, tail, span } => {
            emit(code, InstrKind::PushScope, *span);
            compile_stmts(stmts, code, funcs)?;
            match tail {
                Some(expr) => compile_expr(expr, code, funcs)?,
                None => emit(code, InstrKind::Push(Value::Unit), *span),
            }
            emit(code, InstrKind::PopScope, *span);
        }

        Expr::If {
            cond,
            then_branch,
            else_branch,
            span,
        } => {
            compile_expr(cond, code, funcs)?;
            let jmp_false_at = code.len();
            emit(code, InstrKind::JumpIfFalse(usize::MAX), *span);
            emit(code, InstrKind::Pop, cond.span()); // pop condition (true)

            compile_expr(then_branch, code, funcs)?;
            let jmp_end_at = code.len();
            emit(code, InstrKind::Jump(usize::MAX), *span);

            // else:
            let else_ip = code.len();
            patch_jump(code, jmp_false_at, else_ip);
            emit(code, InstrKind::Pop, cond.span()); // pop condition (false)
            compile_expr(else_branch, code, funcs)?;

            let end_ip = code.len();
            patch_jump(code, jmp_end_at, end_ip);
        }

        Expr::Unary { op, expr, span } => {
            compile_expr(expr, code, funcs)?;
            match op {
                UnaryOp::Neg => emit(code, InstrKind::Neg, *span),
                UnaryOp::Not => emit(code, InstrKind::Not, *span),
            }
        }

        Expr::Binary { lhs, op, rhs, span } => match op {
            BinaryOp::And => {
                compile_expr(lhs, code, funcs)?;
                let jmp_false_at = code.len();
                emit(code, InstrKind::JumpIfFalse(usize::MAX), *span);
                emit(code, InstrKind::Pop, lhs.span()); // pop true
                compile_expr(rhs, code, funcs)?;
                let end_ip = code.len();
                patch_jump(code, jmp_false_at, end_ip);
            }
            BinaryOp::Or => {
                compile_expr(lhs, code, funcs)?;
                let jmp_true_at = code.len();
                emit(code, InstrKind::JumpIfTrue(usize::MAX), *span);
                emit(code, InstrKind::Pop, lhs.span()); // pop false
                compile_expr(rhs, code, funcs)?;
                let end_ip = code.len();
                patch_jump(code, jmp_true_at, end_ip);
            }
            _ => {
                compile_expr(lhs, code, funcs)?;
                compile_expr(rhs, code, funcs)?;
                let kind = match op {
                    BinaryOp::Add => InstrKind::Add,
                    BinaryOp::Sub => InstrKind::Sub,
                    BinaryOp::Mul => InstrKind::Mul,
                    BinaryOp::Div => InstrKind::Div,
                    BinaryOp::Mod => InstrKind::Mod,
                    BinaryOp::Eq => InstrKind::Eq,
                    BinaryOp::Ne => InstrKind::Ne,
                    BinaryOp::Lt => InstrKind::Lt,
                    BinaryOp::Le => InstrKind::Le,
                    BinaryOp::Gt => InstrKind::Gt,
                    BinaryOp::Ge => InstrKind::Ge,
                    BinaryOp::And | BinaryOp::Or => unreachable!("handled above"),
                };
                emit(code, kind, *span);
            }
        },

        Expr::Call { callee, args, span } => {
            // Evaluate callee first, then args (left-to-right), then call.
            compile_expr(callee, code, funcs)?;
            for arg in args {
                compile_expr(arg, code, funcs)?;
            }
            emit(code, InstrKind::CallValue(args.len()), *span);
        }
    }

    Ok(())
}

fn patch_jump(code: &mut [Instr], at: usize, target: usize) {
    match &mut code[at].kind {
        InstrKind::Jump(dst) | InstrKind::JumpIfFalse(dst) | InstrKind::JumpIfTrue(dst) => {
            *dst = target
        }
        _ => panic!("expected jump at {at}"),
    }
}

fn emit(code: &mut Vec<Instr>, kind: InstrKind, span: Span) {
    code.push(Instr::new(kind, span));
}

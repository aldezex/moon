use std::collections::HashMap;

use moon_core::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};
use moon_core::span::Span;
use moon_runtime::Value;

use crate::instr::Instr;
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
        code.push(Instr::Return);
        functions[id].code = code;
    }

    // Compile main.
    {
        let mut code = Vec::new();
        compile_stmts(&program.stmts, &mut code, &by_name)?;
        match &program.tail {
            Some(expr) => compile_expr(expr, &mut code, &by_name)?,
            None => code.push(Instr::Push(Value::Unit)),
        }
        code.push(Instr::Return);
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
            Stmt::Let { name, expr, .. } => {
                compile_expr(expr, code, funcs)?;
                code.push(Instr::DefineVar(name.clone()));
            }
            Stmt::Assign { target, expr, span } => match target {
                Expr::Ident(name, _) => {
                    compile_expr(expr, code, funcs)?;
                    code.push(Instr::SetVar(name.clone()));
                }
                Expr::Index { target, index, .. } => {
                    compile_expr(target, code, funcs)?;
                    compile_expr(index, code, funcs)?;
                    compile_expr(expr, code, funcs)?;
                    code.push(Instr::IndexSet);
                }
                _ => {
                    return Err(CompileError {
                        message: "invalid assignment target".to_string(),
                        span: *span,
                    })
                }
            },
            Stmt::Fn { .. } => {
                // Functions are top-level items. They don't execute in main.
            }
            Stmt::Expr { expr, .. } => {
                compile_expr(expr, code, funcs)?;
                code.push(Instr::Pop);
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
        Expr::Int(i, _) => code.push(Instr::Push(Value::Int(*i))),
        Expr::Bool(b, _) => code.push(Instr::Push(Value::Bool(*b))),
        Expr::String(s, _) => code.push(Instr::Push(Value::String(s.clone()))),
        Expr::Ident(name, _) => code.push(Instr::LoadVar(name.clone())),
        Expr::Group { expr, .. } => return compile_expr(expr, code, funcs),
        Expr::Array { elements, .. } => {
            for e in elements {
                compile_expr(e, code, funcs)?;
            }
            code.push(Instr::MakeArray(elements.len()));
        }
        Expr::Object { props, .. } => {
            let mut keys = Vec::with_capacity(props.len());
            for (k, v) in props {
                keys.push(k.clone());
                compile_expr(v, code, funcs)?;
            }
            code.push(Instr::MakeObject(keys));
        }
        Expr::Index { target, index, .. } => {
            compile_expr(target, code, funcs)?;
            compile_expr(index, code, funcs)?;
            code.push(Instr::IndexGet);
        }
        Expr::Block { stmts, tail, .. } => {
            code.push(Instr::PushScope);
            compile_stmts(stmts, code, funcs)?;
            match tail {
                Some(expr) => compile_expr(expr, code, funcs)?,
                None => code.push(Instr::Push(Value::Unit)),
            }
            code.push(Instr::PopScope);
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            compile_expr(cond, code, funcs)?;
            let jmp_false_at = code.len();
            code.push(Instr::JumpIfFalse(usize::MAX));
            code.push(Instr::Pop); // pop condition (true)
            compile_expr(then_branch, code, funcs)?;
            let jmp_end_at = code.len();
            code.push(Instr::Jump(usize::MAX));

            // else:
            let else_ip = code.len();
            patch_jump(code, jmp_false_at, else_ip);
            code.push(Instr::Pop); // pop condition (false)
            compile_expr(else_branch, code, funcs)?;

            let end_ip = code.len();
            patch_jump(code, jmp_end_at, end_ip);
        }
        Expr::Unary { op, expr, .. } => {
            compile_expr(expr, code, funcs)?;
            match op {
                UnaryOp::Neg => code.push(Instr::Neg),
                UnaryOp::Not => code.push(Instr::Not),
            }
        }
        Expr::Binary { lhs, op, rhs, .. } => match op {
            BinaryOp::And => {
                compile_expr(lhs, code, funcs)?;
                let jmp_false_at = code.len();
                code.push(Instr::JumpIfFalse(usize::MAX));
                code.push(Instr::Pop); // pop true
                compile_expr(rhs, code, funcs)?;
                let end_ip = code.len();
                patch_jump(code, jmp_false_at, end_ip);
            }
            BinaryOp::Or => {
                compile_expr(lhs, code, funcs)?;
                let jmp_true_at = code.len();
                code.push(Instr::JumpIfTrue(usize::MAX));
                code.push(Instr::Pop); // pop false
                compile_expr(rhs, code, funcs)?;
                let end_ip = code.len();
                patch_jump(code, jmp_true_at, end_ip);
            }
            _ => {
                compile_expr(lhs, code, funcs)?;
                compile_expr(rhs, code, funcs)?;
                code.push(match op {
                    BinaryOp::Add => Instr::Add,
                    BinaryOp::Sub => Instr::Sub,
                    BinaryOp::Mul => Instr::Mul,
                    BinaryOp::Div => Instr::Div,
                    BinaryOp::Mod => Instr::Mod,
                    BinaryOp::Eq => Instr::Eq,
                    BinaryOp::Ne => Instr::Ne,
                    BinaryOp::Lt => Instr::Lt,
                    BinaryOp::Le => Instr::Le,
                    BinaryOp::Gt => Instr::Gt,
                    BinaryOp::Ge => Instr::Ge,
                    BinaryOp::And | BinaryOp::Or => unreachable!("handled above"),
                });
            }
        },
        Expr::Call { callee, args, span } => {
            let name = match &**callee {
                Expr::Ident(name, _) => name.as_str(),
                _ => {
                    return Err(CompileError {
                        message: "can only call functions by name (for now)".to_string(),
                        span: *span,
                    })
                }
            };

            for arg in args {
                compile_expr(arg, code, funcs)?;
            }

            let id = funcs.get(name).copied().ok_or_else(|| CompileError {
                message: format!("undefined function: {name}"),
                span: *span,
            })?;
            code.push(Instr::Call(id, args.len()));
        }
    }

    Ok(())
}

fn patch_jump(code: &mut [Instr], at: usize, target: usize) {
    match &mut code[at] {
        Instr::Jump(dst) | Instr::JumpIfFalse(dst) | Instr::JumpIfTrue(dst) => *dst = target,
        _ => panic!("expected jump at {at}"),
    }
}

use std::collections::{HashMap, HashSet};

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

#[derive(Debug, Default)]
struct FunctionCtx {
    // Locals defined in the current function, split by lexical scopes.
    scopes: Vec<HashSet<String>>,
    // Names available via the current closure environment (if this function is a closure).
    closure_env: HashSet<String>,
}

impl FunctionCtx {
    fn new_main() -> Self {
        Self::default()
    }

    fn new_function(params: &[String], closure_env: Vec<String>) -> Self {
        let mut ctx = Self {
            scopes: vec![HashSet::new()],
            closure_env: closure_env.into_iter().collect(),
        };
        for p in params {
            ctx.define_local(p.clone());
        }
        ctx
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashSet::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn define_local(&mut self, name: String) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name);
        }
    }

    fn visible_names(&self) -> Vec<String> {
        let mut set: HashSet<String> = self.closure_env.clone();
        for scope in &self.scopes {
            for name in scope {
                set.insert(name.clone());
            }
        }
        let mut v: Vec<String> = set.into_iter().collect();
        v.sort();
        v
    }
}

struct Compiler {
    functions: Vec<Function>,
    by_name: HashMap<String, FuncId>,
    next_lambda_id: usize,
}

impl Compiler {
    fn new() -> Self {
        Self {
            functions: Vec::new(),
            by_name: HashMap::new(),
            next_lambda_id: 0,
        }
    }

    fn fresh_lambda_name(&mut self) -> String {
        let id = self.next_lambda_id;
        self.next_lambda_id += 1;
        format!("<lambda#{id}>")
    }

    fn define_stub(&mut self, name: String, params: Vec<String>) -> FuncId {
        let id = self.functions.len();
        self.by_name.insert(name.clone(), id);
        self.functions.push(Function {
            name,
            params,
            code: Vec::new(),
        });
        id
    }

    fn compile_function_body(
        &mut self,
        id: FuncId,
        body: &Expr,
        ctx: &mut FunctionCtx,
    ) -> Result<(), CompileError> {
        let mut code = Vec::new();
        self.compile_expr(body, &mut code, ctx)?;
        emit(&mut code, InstrKind::Return, body.span());
        self.functions[id].code = code;
        Ok(())
    }

    fn compile_stmts(
        &mut self,
        stmts: &[Stmt],
        code: &mut Vec<Instr>,
        ctx: &mut FunctionCtx,
    ) -> Result<(), CompileError> {
        for stmt in stmts {
            match stmt {
                Stmt::Let {
                    name, expr, span, ..
                } => {
                    self.compile_expr(expr, code, ctx)?;
                    emit(code, InstrKind::DefineVar(name.clone()), *span);
                    ctx.define_local(name.clone());
                }
                Stmt::Assign { target, expr, span } => match target {
                    Expr::Ident(name, name_span) => {
                        self.compile_expr(expr, code, ctx)?;
                        emit(code, InstrKind::SetVar(name.clone()), *name_span);
                    }
                    Expr::Index { target, index, .. } => {
                        self.compile_expr(target, code, ctx)?;
                        self.compile_expr(index, code, ctx)?;
                        self.compile_expr(expr, code, ctx)?;
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
                        Some(expr) => self.compile_expr(expr, code, ctx)?,
                        None => emit(code, InstrKind::Push(Value::Unit), *span),
                    }
                    emit(code, InstrKind::Return, *span);
                }
                Stmt::Fn { .. } => {
                    // Functions are top-level items. They don't execute in main.
                }
                Stmt::Expr { expr, .. } => {
                    self.compile_expr(expr, code, ctx)?;
                    emit(code, InstrKind::Pop, expr.span());
                }
            }
        }
        Ok(())
    }

    fn compile_expr(
        &mut self,
        expr: &Expr,
        code: &mut Vec<Instr>,
        ctx: &mut FunctionCtx,
    ) -> Result<(), CompileError> {
        match expr {
            Expr::Int(i, span) => emit(code, InstrKind::Push(Value::Int(*i)), *span),
            Expr::Bool(b, span) => emit(code, InstrKind::Push(Value::Bool(*b)), *span),
            Expr::String(s, span) => emit(code, InstrKind::Push(Value::String(s.clone())), *span),
            Expr::Ident(name, span) => emit(code, InstrKind::LoadVar(name.clone()), *span),
            Expr::Group { expr, .. } => return self.compile_expr(expr, code, ctx),

            Expr::Fn {
                params, body, span, ..
            } => {
                let name = self.fresh_lambda_name();

                // Capture all currently-visible locals (incl. closure env of this function).
                let captures = ctx.visible_names();

                // Compile the function body into a new module function.
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let id = self.define_stub(name.clone(), param_names.clone());

                let mut inner_ctx = FunctionCtx::new_function(&param_names, captures.clone());
                self.compile_function_body(id, body, &mut inner_ctx)?;

                emit(code, InstrKind::MakeClosure(name.clone(), captures), *span);
            }

            Expr::Array { elements, span } => {
                for e in elements {
                    self.compile_expr(e, code, ctx)?;
                }
                emit(code, InstrKind::MakeArray(elements.len()), *span);
            }
            Expr::Object { props, span } => {
                let mut keys = Vec::with_capacity(props.len());
                for (k, v) in props {
                    keys.push(k.clone());
                    self.compile_expr(v, code, ctx)?;
                }
                emit(code, InstrKind::MakeObject(keys), *span);
            }
            Expr::Index {
                target,
                index,
                span,
            } => {
                self.compile_expr(target, code, ctx)?;
                self.compile_expr(index, code, ctx)?;
                emit(code, InstrKind::IndexGet, *span);
            }

            Expr::Block { stmts, tail, span } => {
                emit(code, InstrKind::PushScope, *span);
                ctx.push_scope();
                self.compile_stmts(stmts, code, ctx)?;
                match tail {
                    Some(expr) => self.compile_expr(expr, code, ctx)?,
                    None => emit(code, InstrKind::Push(Value::Unit), *span),
                }
                ctx.pop_scope();
                emit(code, InstrKind::PopScope, *span);
            }

            Expr::If {
                cond,
                then_branch,
                else_branch,
                span,
            } => {
                self.compile_expr(cond, code, ctx)?;
                let jmp_false_at = code.len();
                emit(code, InstrKind::JumpIfFalse(usize::MAX), *span);
                emit(code, InstrKind::Pop, cond.span()); // pop condition (true)

                self.compile_expr(then_branch, code, ctx)?;
                let jmp_end_at = code.len();
                emit(code, InstrKind::Jump(usize::MAX), *span);

                // else:
                let else_ip = code.len();
                patch_jump(code, jmp_false_at, else_ip);
                emit(code, InstrKind::Pop, cond.span()); // pop condition (false)
                self.compile_expr(else_branch, code, ctx)?;

                let end_ip = code.len();
                patch_jump(code, jmp_end_at, end_ip);
            }

            Expr::Unary { op, expr, span } => {
                self.compile_expr(expr, code, ctx)?;
                match op {
                    UnaryOp::Neg => emit(code, InstrKind::Neg, *span),
                    UnaryOp::Not => emit(code, InstrKind::Not, *span),
                }
            }

            Expr::Binary { lhs, op, rhs, span } => match op {
                BinaryOp::And => {
                    self.compile_expr(lhs, code, ctx)?;
                    let jmp_false_at = code.len();
                    emit(code, InstrKind::JumpIfFalse(usize::MAX), *span);
                    emit(code, InstrKind::Pop, lhs.span()); // pop true
                    self.compile_expr(rhs, code, ctx)?;
                    let end_ip = code.len();
                    patch_jump(code, jmp_false_at, end_ip);
                }
                BinaryOp::Or => {
                    self.compile_expr(lhs, code, ctx)?;
                    let jmp_true_at = code.len();
                    emit(code, InstrKind::JumpIfTrue(usize::MAX), *span);
                    emit(code, InstrKind::Pop, lhs.span()); // pop false
                    self.compile_expr(rhs, code, ctx)?;
                    let end_ip = code.len();
                    patch_jump(code, jmp_true_at, end_ip);
                }
                _ => {
                    self.compile_expr(lhs, code, ctx)?;
                    self.compile_expr(rhs, code, ctx)?;
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
                self.compile_expr(callee, code, ctx)?;
                for arg in args {
                    self.compile_expr(arg, code, ctx)?;
                }
                emit(code, InstrKind::CallValue(args.len()), *span);
            }
        }

        Ok(())
    }
}

pub fn compile(program: &Program) -> Result<Module, CompileError> {
    let mut c = Compiler::new();

    // Reserve main at 0.
    c.functions.push(Function {
        name: "<main>".to_string(),
        params: Vec::new(),
        code: Vec::new(),
    });
    let main_id = 0usize;

    // Builtins (implemented in the VM).
    // We treat them as functions in the module so they can be called like normal.
    {
        c.define_stub("gc".to_string(), Vec::new());
    }

    // Collect function ids first so calls can refer to functions declared later.
    for stmt in &program.stmts {
        if let Stmt::Fn { name, params, .. } = stmt {
            if c.by_name.contains_key(name) {
                return Err(CompileError {
                    message: format!("duplicate function: {name}"),
                    span: stmt.span(),
                });
            }
            let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
            c.define_stub(name.clone(), param_names);
        }
    }

    // Compile each function body.
    for stmt in &program.stmts {
        let Stmt::Fn { name, body, .. } = stmt else {
            continue;
        };
        let id = *c.by_name.get(name).expect("function id exists");
        let params = c.functions[id].params.clone();
        let mut ctx = FunctionCtx::new_function(&params, Vec::new());
        c.compile_function_body(id, body, &mut ctx)?;
    }

    // Compile main.
    {
        let mut code = Vec::new();
        let mut ctx = FunctionCtx::new_main();
        c.compile_stmts(&program.stmts, &mut code, &mut ctx)?;

        let end_span = program
            .tail
            .as_ref()
            .map(|e| e.span())
            .or_else(|| program.stmts.last().map(|s| s.span()))
            .unwrap_or(Span::new(0, 0));

        match &program.tail {
            Some(expr) => c.compile_expr(expr, &mut code, &mut ctx)?,
            None => emit(&mut code, InstrKind::Push(Value::Unit), end_span),
        }
        emit(&mut code, InstrKind::Return, end_span);
        c.functions[main_id].code = code;
    }

    Ok(Module {
        functions: c.functions,
        by_name: c.by_name,
        main: main_id,
    })
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

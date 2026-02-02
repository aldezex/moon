use std::collections::HashMap;

use moon_bytecode::{FuncId, InstrKind, Module};
use moon_core::span::Span;
use moon_runtime::{GcRef, Heap, Value};

use crate::error::VmError;

#[derive(Debug, Clone)]
struct Frame {
    func: FuncId,
    ip: usize,
    stack_base: usize,
    scopes: Vec<HashMap<String, Value>>,
    closure: Option<GcRef>,
}

#[derive(Debug)]
pub struct Vm {
    module: Module,
    heap: Heap,
    globals: HashMap<String, Value>,
    stack: Vec<Value>,
    frames: Vec<Frame>,
    current_span: Span,
}

impl Vm {
    pub fn new(module: Module) -> Self {
        Self {
            module,
            heap: Heap::new(),
            globals: HashMap::new(),
            stack: Vec::new(),
            frames: Vec::new(),
            current_span: Span::new(0, 0),
        }
    }

    pub fn run(mut self) -> Result<Value, VmError> {
        // Main executes with no local scopes. Top-level `let` therefore defines globals.
        self.frames.push(Frame {
            func: self.module.main,
            ip: 0,
            stack_base: 0,
            scopes: Vec::new(),
            closure: None,
        });

        loop {
            let frame_idx = self.frames.len() - 1;
            let func_id = self.frames[frame_idx].func;
            let ip = self.frames[frame_idx].ip;

            let func = self.module.get_func(func_id).ok_or_else(|| {
                VmError::new("invalid function id".to_string(), self.current_span)
            })?;

            if ip >= func.code.len() {
                return Err(VmError::new(
                    format!("instruction pointer out of bounds in {}", func.name),
                    self.current_span,
                ));
            }

            let instr = func.code[ip].clone();
            self.frames[frame_idx].ip += 1;
            self.current_span = instr.span;

            match instr.kind {
                InstrKind::Push(v) => self.stack.push(v),
                InstrKind::Pop => {
                    self.stack
                        .pop()
                        .ok_or_else(|| self.err("stack underflow"))?;
                }

                InstrKind::PushScope => self.frames[frame_idx].scopes.push(HashMap::new()),
                InstrKind::PopScope => {
                    self.frames[frame_idx]
                        .scopes
                        .pop()
                        .ok_or_else(|| self.err("scope underflow"))?;
                }

                InstrKind::LoadVar(name) => {
                    if let Some(v) = self.get_var(frame_idx, &name) {
                        self.stack.push(v);
                    } else if self.module.by_name.contains_key(&name) {
                        // Functions are values too. Vars shadow functions.
                        self.stack.push(Value::Function(name));
                    } else {
                        return Err(self.err(format!("undefined variable: {name}")));
                    }
                }
                InstrKind::DefineVar(name) => {
                    let v = self.pop()?;
                    self.define_var(frame_idx, name, v);
                }
                InstrKind::SetVar(name) => {
                    let v = self.pop()?;
                    self.set_var(frame_idx, &name, v)?;
                }

                InstrKind::Neg => {
                    let v = self.pop()?;
                    match v {
                        Value::Int(i) => self.stack.push(Value::Int(-i)),
                        other => {
                            return Err(self.err(format!("cannot apply unary '-' to {other:?}")))
                        }
                    }
                }
                InstrKind::Not => {
                    let v = self.pop()?;
                    match v {
                        Value::Bool(b) => self.stack.push(Value::Bool(!b)),
                        other => {
                            return Err(self.err(format!("cannot apply unary '!' to {other:?}")))
                        }
                    }
                }

                InstrKind::Add => self.bin_add()?,
                InstrKind::Sub => self.bin_int(|a, b| a - b, "subtract")?,
                InstrKind::Mul => self.bin_int(|a, b| a * b, "multiply")?,
                InstrKind::Div => {
                    let (a, b) = self.pop_two_ints()?;
                    if b == 0 {
                        return Err(self.err("division by zero"));
                    }
                    self.stack.push(Value::Int(a / b));
                }
                InstrKind::Mod => {
                    let (a, b) = self.pop_two_ints()?;
                    if b == 0 {
                        return Err(self.err("modulo by zero"));
                    }
                    self.stack.push(Value::Int(a % b));
                }
                InstrKind::Eq => self.bin_eq(true)?,
                InstrKind::Ne => self.bin_eq(false)?,
                InstrKind::Lt => self.bin_cmp(|a, b| a < b, "<")?,
                InstrKind::Le => self.bin_cmp(|a, b| a <= b, "<=")?,
                InstrKind::Gt => self.bin_cmp(|a, b| a > b, ">")?,
                InstrKind::Ge => self.bin_cmp(|a, b| a >= b, ">=")?,

                InstrKind::Jump(dst) => self.frames[frame_idx].ip = dst,
                InstrKind::JumpIfFalse(dst) => {
                    let v = self.peek()?.clone();
                    match v {
                        Value::Bool(false) => self.frames[frame_idx].ip = dst,
                        Value::Bool(true) => {}
                        other => {
                            return Err(self.err(format!("expected bool condition, got {other:?}")))
                        }
                    }
                }
                InstrKind::JumpIfTrue(dst) => {
                    let v = self.peek()?.clone();
                    match v {
                        Value::Bool(true) => self.frames[frame_idx].ip = dst,
                        Value::Bool(false) => {}
                        other => {
                            return Err(self.err(format!("expected bool condition, got {other:?}")))
                        }
                    }
                }

                InstrKind::Call(id, argc) => {
                    let func_obj = self
                        .module
                        .get_func(id)
                        .ok_or_else(|| self.err("invalid function id"))?;

                    // Builtins are treated like normal functions in bytecode, but executed by the VM.
                    if func_obj.name == "gc" {
                        // No args.
                        if argc != 0 {
                            return Err(self.err("gc() takes no arguments"));
                        }

                        let roots = self.roots();
                        let _ = self.heap.collect_garbage(&roots);
                        self.stack.push(Value::Unit);
                        continue;
                    }

                    // Pop arguments from the stack.
                    let mut args = Vec::with_capacity(argc);
                    for _ in 0..argc {
                        args.push(self.pop()?);
                    }
                    args.reverse();

                    let stack_base = self.stack.len();
                    self.push_call_frame(id, stack_base, args, None)?;
                }

                InstrKind::CallValue(argc) => {
                    // Pop arguments from the stack.
                    let mut args = Vec::with_capacity(argc);
                    for _ in 0..argc {
                        args.push(self.pop()?);
                    }
                    args.reverse();

                    let callee = self.pop()?;
                    let (name, closure) = match callee {
                        Value::Function(name) => (name, None),
                        Value::Closure(h) => {
                            let func = self
                                .heap
                                .closure_func_name(h)
                                .ok_or_else(|| self.err("invalid closure handle"))?;
                            (func.to_string(), Some(h))
                        }
                        other => {
                            return Err(
                                self.err(format!("cannot call non-function value: {other:?}"))
                            )
                        }
                    };

                    let id = self
                        .module
                        .by_name
                        .get(&name)
                        .copied()
                        .ok_or_else(|| self.err(format!("undefined function: {name}")))?;

                    let func_obj = self
                        .module
                        .get_func(id)
                        .ok_or_else(|| self.err("invalid function id"))?;

                    // Builtins are treated like normal functions in bytecode, but executed by the VM.
                    if func_obj.name == "gc" {
                        // No args.
                        if argc != 0 {
                            return Err(self.err("gc() takes no arguments"));
                        }

                        let roots = self.roots();
                        let _ = self.heap.collect_garbage(&roots);
                        self.stack.push(Value::Unit);
                        continue;
                    }

                    let stack_base = self.stack.len();
                    self.push_call_frame(id, stack_base, args, closure)?;
                }

                InstrKind::Return => {
                    let ret = self.pop()?;
                    let frame = self.frames.pop().expect("frame exists");
                    self.stack.truncate(frame.stack_base);

                    if self.frames.is_empty() {
                        return Ok(ret);
                    }

                    self.stack.push(ret);
                }

                InstrKind::MakeArray(n) => {
                    let mut elems = Vec::with_capacity(n);
                    for _ in 0..n {
                        elems.push(self.pop()?);
                    }
                    elems.reverse();
                    let h = self.heap.alloc_array(elems);
                    self.stack.push(Value::Array(h));
                }
                InstrKind::MakeObject(keys) => {
                    let n = keys.len();
                    let mut values = Vec::with_capacity(n);
                    for _ in 0..n {
                        values.push(self.pop()?);
                    }
                    values.reverse();
                    let mut map = HashMap::new();
                    for (k, v) in keys.into_iter().zip(values) {
                        map.insert(k, v);
                    }
                    let h = self.heap.alloc_object(map);
                    self.stack.push(Value::Object(h));
                }
                InstrKind::IndexGet => {
                    let index = self.pop()?;
                    let base = self.pop()?;
                    let v = self.index_get(base, index)?;
                    self.stack.push(v);
                }
                InstrKind::IndexSet => {
                    let value = self.pop()?;
                    let index = self.pop()?;
                    let base = self.pop()?;
                    self.index_set(base, index, value)?;
                }

                InstrKind::MakeClosure(name, captures) => {
                    let mut env = HashMap::new();
                    for cap in captures {
                        if let Some(v) = self.get_local(frame_idx, &cap) {
                            env.insert(cap, v);
                        }
                    }
                    let h = self.heap.alloc_closure(name, env);
                    self.stack.push(Value::Closure(h));
                }
            }
        }
    }

    fn err(&self, message: impl Into<String>) -> VmError {
        VmError::new(message, self.current_span)
    }

    fn push_call_frame(
        &mut self,
        func: FuncId,
        stack_base: usize,
        args: Vec<Value>,
        closure: Option<GcRef>,
    ) -> Result<(), VmError> {
        let func_obj = self
            .module
            .get_func(func)
            .ok_or_else(|| self.err("invalid function id"))?;

        let mut scope = HashMap::new();
        for (name, value) in func_obj.params.iter().cloned().zip(args) {
            scope.insert(name, value);
        }

        self.frames.push(Frame {
            func,
            ip: 0,
            stack_base,
            scopes: vec![scope],
            closure,
        });
        Ok(())
    }

    fn get_var(&self, frame_idx: usize, name: &str) -> Option<Value> {
        for scope in self.frames[frame_idx].scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v.clone());
            }
        }
        if let Some(h) = self.frames[frame_idx].closure {
            if let Some(v) = self.heap.closure_get(h, name) {
                return Some(v.clone());
            }
        }
        self.globals.get(name).cloned()
    }

    fn get_local(&self, frame_idx: usize, name: &str) -> Option<Value> {
        for scope in self.frames[frame_idx].scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v.clone());
            }
        }
        if let Some(h) = self.frames[frame_idx].closure {
            if let Some(v) = self.heap.closure_get(h, name) {
                return Some(v.clone());
            }
        }
        None
    }

    fn define_var(&mut self, frame_idx: usize, name: String, value: Value) {
        if let Some(scope) = self.frames[frame_idx].scopes.last_mut() {
            scope.insert(name, value);
        } else {
            self.globals.insert(name, value);
        }
    }

    fn set_var(&mut self, frame_idx: usize, name: &str, value: Value) -> Result<(), VmError> {
        for scope in self.frames[frame_idx].scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Ok(());
            }
        }
        if let Some(h) = self.frames[frame_idx].closure {
            if self.heap.closure_contains(h, name) {
                self.heap
                    .closure_set(h, name.to_string(), value)
                    .map_err(|e| self.err(e))?;
                return Ok(());
            }
        }
        if self.globals.contains_key(name) {
            self.globals.insert(name.to_string(), value);
            return Ok(());
        }
        Err(self.err(format!("undefined variable: {name}")))
    }

    fn peek(&self) -> Result<&Value, VmError> {
        self.stack.last().ok_or_else(|| self.err("stack underflow"))
    }

    fn pop(&mut self) -> Result<Value, VmError> {
        self.stack.pop().ok_or_else(|| self.err("stack underflow"))
    }

    fn pop_two_ints(&mut self) -> Result<(i64, i64), VmError> {
        let b = self.pop()?;
        let a = self.pop()?;
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => Ok((a, b)),
            (a, b) => Err(self.err(format!("expected two ints, got {a:?} and {b:?}"))),
        }
    }

    fn bin_int(&mut self, f: fn(i64, i64) -> i64, _name: &'static str) -> Result<(), VmError> {
        let (a, b) = self.pop_two_ints()?;
        self.stack.push(Value::Int(f(a, b)));
        Ok(())
    }

    fn bin_add(&mut self) -> Result<(), VmError> {
        let b = self.pop()?;
        let a = self.pop()?;
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => {
                self.stack.push(Value::Int(a + b));
                Ok(())
            }
            (Value::String(a), Value::String(b)) => {
                self.stack.push(Value::String(format!("{a}{b}")));
                Ok(())
            }
            (a, b) => Err(self.err(format!("cannot add {a:?} and {b:?}"))),
        }
    }

    fn bin_eq(&mut self, eq: bool) -> Result<(), VmError> {
        let b = self.pop()?;
        let a = self.pop()?;
        let r = if eq { a == b } else { a != b };
        self.stack.push(Value::Bool(r));
        Ok(())
    }

    fn bin_cmp(&mut self, f: fn(i64, i64) -> bool, _name: &'static str) -> Result<(), VmError> {
        let (a, b) = self.pop_two_ints()?;
        self.stack.push(Value::Bool(f(a, b)));
        Ok(())
    }

    fn index_get(&mut self, base: Value, index: Value) -> Result<Value, VmError> {
        match base {
            Value::Array(h) => {
                let idx = match index {
                    Value::Int(i) => {
                        usize::try_from(i).map_err(|_| self.err("array index must be >= 0"))?
                    }
                    other => {
                        return Err(self.err(format!("array index must be int, got {other:?}")))
                    }
                };
                self.heap
                    .array_get(h, idx)
                    .cloned()
                    .ok_or_else(|| self.err(format!("index out of bounds: {idx}")))
            }
            Value::Object(h) => {
                let key = match index {
                    Value::String(s) => s,
                    other => {
                        return Err(self.err(format!("object key must be string, got {other:?}")))
                    }
                };
                self.heap
                    .object_get(h, &key)
                    .cloned()
                    .ok_or_else(|| self.err(format!("missing key: {key}")))
            }
            other => Err(self.err(format!("cannot index into {other:?}"))),
        }
    }

    fn index_set(&mut self, base: Value, index: Value, value: Value) -> Result<(), VmError> {
        match base {
            Value::Array(h) => {
                let idx = match index {
                    Value::Int(i) => {
                        usize::try_from(i).map_err(|_| self.err("array index must be >= 0"))?
                    }
                    other => {
                        return Err(self.err(format!("array index must be int, got {other:?}")))
                    }
                };
                self.heap.array_set(h, idx, value).map_err(|e| self.err(e))
            }
            Value::Object(h) => {
                let key = match index {
                    Value::String(s) => s,
                    other => {
                        return Err(self.err(format!("object key must be string, got {other:?}")))
                    }
                };
                self.heap.object_set(h, key, value).map_err(|e| self.err(e))
            }
            other => Err(self.err(format!("cannot assign through index on {other:?}"))),
        }
    }

    fn roots(&self) -> Vec<Value> {
        let mut roots = Vec::new();
        roots.extend(self.globals.values().cloned());
        for frame in &self.frames {
            if let Some(h) = frame.closure {
                roots.push(Value::Closure(h));
            }
            for scope in &frame.scopes {
                roots.extend(scope.values().cloned());
            }
        }
        roots.extend(self.stack.iter().cloned());
        roots
    }
}

pub fn run(module: Module) -> Result<Value, VmError> {
    Vm::new(module).run()
}

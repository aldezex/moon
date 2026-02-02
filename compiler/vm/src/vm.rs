use std::collections::HashMap;

use moon_bytecode::{FuncId, Instr, Module};
use moon_runtime::{Heap, Value};

use crate::error::VmError;

#[derive(Debug, Clone)]
struct Frame {
    func: FuncId,
    ip: usize,
    stack_base: usize,
    scopes: Vec<HashMap<String, Value>>,
}

#[derive(Debug)]
pub struct Vm {
    module: Module,
    heap: Heap,
    globals: HashMap<String, Value>,
    stack: Vec<Value>,
    frames: Vec<Frame>,
}

impl Vm {
    pub fn new(module: Module) -> Self {
        Self {
            module,
            heap: Heap::new(),
            globals: HashMap::new(),
            stack: Vec::new(),
            frames: Vec::new(),
        }
    }

    pub fn run(mut self) -> Result<Value, VmError> {
        // Main executes with no local scopes. Top-level `let` therefore defines globals.
        self.frames.push(Frame {
            func: self.module.main,
            ip: 0,
            stack_base: 0,
            scopes: Vec::new(),
        });

        loop {
            let frame_idx = self.frames.len() - 1;
            let func_id = self.frames[frame_idx].func;
            let ip = self.frames[frame_idx].ip;

            let func = self.module.get_func(func_id).ok_or_else(|| VmError {
                message: "invalid function id".to_string(),
            })?;

            if ip >= func.code.len() {
                return Err(VmError {
                    message: format!("instruction pointer out of bounds in {}", func.name),
                });
            }

            let instr = func.code[ip].clone();
            self.frames[frame_idx].ip += 1;

            match instr {
                Instr::Push(v) => self.stack.push(v),
                Instr::Pop => {
                    self.stack.pop().ok_or_else(|| VmError {
                        message: "stack underflow".to_string(),
                    })?;
                }
                Instr::PushScope => self.frames[frame_idx].scopes.push(HashMap::new()),
                Instr::PopScope => {
                    self.frames[frame_idx].scopes.pop().ok_or_else(|| VmError {
                        message: "scope underflow".to_string(),
                    })?;
                }
                Instr::LoadVar(name) => {
                    let v = self.get_var(frame_idx, &name).ok_or_else(|| VmError {
                        message: format!("undefined variable: {name}"),
                    })?;
                    self.stack.push(v);
                }
                Instr::DefineVar(name) => {
                    let v = self.stack.pop().ok_or_else(|| VmError {
                        message: "stack underflow".to_string(),
                    })?;
                    self.define_var(frame_idx, name, v);
                }
                Instr::SetVar(name) => {
                    let v = self.stack.pop().ok_or_else(|| VmError {
                        message: "stack underflow".to_string(),
                    })?;
                    self.set_var(frame_idx, &name, v)?;
                }

                Instr::Neg => {
                    let v = self.pop()?;
                    match v {
                        Value::Int(i) => self.stack.push(Value::Int(-i)),
                        other => {
                            return Err(VmError {
                                message: format!("cannot apply unary '-' to {other:?}"),
                            })
                        }
                    }
                }
                Instr::Not => {
                    let v = self.pop()?;
                    match v {
                        Value::Bool(b) => self.stack.push(Value::Bool(!b)),
                        other => {
                            return Err(VmError {
                                message: format!("cannot apply unary '!' to {other:?}"),
                            })
                        }
                    }
                }

                Instr::Add => self.bin_add()?,
                Instr::Sub => self.bin_int(|a, b| a - b, "subtract")?,
                Instr::Mul => self.bin_int(|a, b| a * b, "multiply")?,
                Instr::Div => {
                    let (a, b) = self.pop_two_ints()?;
                    if b == 0 {
                        return Err(VmError {
                            message: "division by zero".to_string(),
                        });
                    }
                    self.stack.push(Value::Int(a / b));
                }
                Instr::Mod => {
                    let (a, b) = self.pop_two_ints()?;
                    if b == 0 {
                        return Err(VmError {
                            message: "modulo by zero".to_string(),
                        });
                    }
                    self.stack.push(Value::Int(a % b));
                }

                Instr::Eq => self.bin_eq(true)?,
                Instr::Ne => self.bin_eq(false)?,
                Instr::Lt => self.bin_cmp(|a, b| a < b, "<")?,
                Instr::Le => self.bin_cmp(|a, b| a <= b, "<=")?,
                Instr::Gt => self.bin_cmp(|a, b| a > b, ">")?,
                Instr::Ge => self.bin_cmp(|a, b| a >= b, ">=")?,

                Instr::Jump(dst) => self.frames[frame_idx].ip = dst,
                Instr::JumpIfFalse(dst) => {
                    let cond = match self.peek()? {
                        Value::Bool(b) => *b,
                        other => {
                            return Err(VmError {
                                message: format!("JumpIfFalse expects bool, got {other:?}"),
                            })
                        }
                    };
                    if !cond {
                        self.frames[frame_idx].ip = dst;
                    }
                }
                Instr::JumpIfTrue(dst) => {
                    let cond = match self.peek()? {
                        Value::Bool(b) => *b,
                        other => {
                            return Err(VmError {
                                message: format!("JumpIfTrue expects bool, got {other:?}"),
                            })
                        }
                    };
                    if cond {
                        self.frames[frame_idx].ip = dst;
                    }
                }

                Instr::Call(id, argc) => {
                    // Evaluate arguments are already on the stack.
                    let func = self.module.get_func(id).ok_or_else(|| VmError {
                        message: "invalid function id".to_string(),
                    })?;

                    // Builtins
                    if func.name == "gc" {
                        if argc != 0 {
                            return Err(VmError {
                                message: "gc() takes no arguments".to_string(),
                            });
                        }
                        let roots = self.roots();
                        let _ = self.heap.collect_garbage(&roots);
                        self.stack.push(Value::Unit);
                        continue;
                    }

                    if argc != func.params.len() {
                        return Err(VmError {
                            message: format!(
                                "wrong number of arguments for {}: expected {}, got {}",
                                func.name,
                                func.params.len(),
                                argc
                            ),
                        });
                    }

                    let mut args = Vec::with_capacity(argc);
                    for _ in 0..argc {
                        args.push(self.pop()?);
                    }
                    args.reverse();

                    let stack_base = self.stack.len();
                    self.push_call_frame(id, stack_base, args)?;
                }

                Instr::Return => {
                    let ret = self.pop()?;
                    let frame = self.frames.pop().expect("frame exists");
                    self.stack.truncate(frame.stack_base);

                    if self.frames.is_empty() {
                        return Ok(ret);
                    }

                    self.stack.push(ret);
                }

                Instr::MakeArray(n) => {
                    let mut elems = Vec::with_capacity(n);
                    for _ in 0..n {
                        elems.push(self.pop()?);
                    }
                    elems.reverse();
                    let h = self.heap.alloc_array(elems);
                    self.stack.push(Value::Array(h));
                }
                Instr::MakeObject(keys) => {
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
                Instr::IndexGet => {
                    let index = self.pop()?;
                    let base = self.pop()?;
                    let v = self.index_get(base, index)?;
                    self.stack.push(v);
                }
                Instr::IndexSet => {
                    let value = self.pop()?;
                    let index = self.pop()?;
                    let base = self.pop()?;
                    self.index_set(base, index, value)?;
                }
            }
        }
    }

    fn push_call_frame(
        &mut self,
        func: FuncId,
        stack_base: usize,
        args: Vec<Value>,
    ) -> Result<(), VmError> {
        let func_obj = self.module.get_func(func).ok_or_else(|| VmError {
            message: "invalid function id".to_string(),
        })?;

        let mut scope = HashMap::new();
        for (name, value) in func_obj.params.iter().cloned().zip(args) {
            scope.insert(name, value);
        }

        self.frames.push(Frame {
            func,
            ip: 0,
            stack_base,
            scopes: vec![scope],
        });
        Ok(())
    }

    fn get_var(&self, frame_idx: usize, name: &str) -> Option<Value> {
        for scope in self.frames[frame_idx].scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v.clone());
            }
        }
        self.globals.get(name).cloned()
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
        if self.globals.contains_key(name) {
            self.globals.insert(name.to_string(), value);
            return Ok(());
        }
        Err(VmError {
            message: format!("undefined variable: {name}"),
        })
    }

    fn peek(&self) -> Result<&Value, VmError> {
        self.stack.last().ok_or_else(|| VmError {
            message: "stack underflow".to_string(),
        })
    }

    fn pop(&mut self) -> Result<Value, VmError> {
        self.stack.pop().ok_or_else(|| VmError {
            message: "stack underflow".to_string(),
        })
    }

    fn pop_two_ints(&mut self) -> Result<(i64, i64), VmError> {
        let b = self.pop()?;
        let a = self.pop()?;
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => Ok((a, b)),
            (a, b) => Err(VmError {
                message: format!("expected two ints, got {a:?} and {b:?}"),
            }),
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
            (a, b) => Err(VmError {
                message: format!("cannot add {a:?} and {b:?}"),
            }),
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
                    Value::Int(i) => usize::try_from(i).map_err(|_| VmError {
                        message: "array index must be >= 0".to_string(),
                    })?,
                    other => {
                        return Err(VmError {
                            message: format!("array index must be int, got {other:?}"),
                        })
                    }
                };
                self.heap.array_get(h, idx).cloned().ok_or_else(|| VmError {
                    message: format!("index out of bounds: {idx}"),
                })
            }
            Value::Object(h) => {
                let key = match index {
                    Value::String(s) => s,
                    other => {
                        return Err(VmError {
                            message: format!("object key must be string, got {other:?}"),
                        })
                    }
                };
                self.heap
                    .object_get(h, &key)
                    .cloned()
                    .ok_or_else(|| VmError {
                        message: format!("missing key: {key}"),
                    })
            }
            other => Err(VmError {
                message: format!("cannot index into {other:?}"),
            }),
        }
    }

    fn index_set(&mut self, base: Value, index: Value, value: Value) -> Result<(), VmError> {
        match base {
            Value::Array(h) => {
                let idx = match index {
                    Value::Int(i) => usize::try_from(i).map_err(|_| VmError {
                        message: "array index must be >= 0".to_string(),
                    })?,
                    other => {
                        return Err(VmError {
                            message: format!("array index must be int, got {other:?}"),
                        })
                    }
                };
                self.heap
                    .array_set(h, idx, value)
                    .map_err(|e| VmError { message: e })
            }
            Value::Object(h) => {
                let key = match index {
                    Value::String(s) => s,
                    other => {
                        return Err(VmError {
                            message: format!("object key must be string, got {other:?}"),
                        })
                    }
                };
                self.heap
                    .object_set(h, key, value)
                    .map_err(|e| VmError { message: e })
            }
            other => Err(VmError {
                message: format!("cannot assign through index on {other:?}"),
            }),
        }
    }

    fn roots(&self) -> Vec<Value> {
        let mut roots = Vec::new();
        roots.extend(self.globals.values().cloned());
        for frame in &self.frames {
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

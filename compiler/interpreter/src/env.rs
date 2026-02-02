use std::collections::HashMap;

use moon_core::ast::Expr;
use moon_runtime::Heap;

use crate::Value;

#[derive(Debug, Clone)]
pub struct Function {
    pub params: Vec<String>,
    pub body: Expr,
}

#[derive(Debug, Default)]
pub struct Env {
    globals: HashMap<String, Value>,
    scopes: Vec<HashMap<String, Value>>,
    funcs: HashMap<String, Function>,
    pub heap: Heap,
}

impl Env {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            scopes: Vec::new(),
            funcs: HashMap::new(),
            heap: Heap::new(),
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn get_var(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v);
            }
        }
        self.globals.get(name)
    }

    pub fn define_var(&mut self, name: String, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        } else {
            self.globals.insert(name, value);
        }
    }

    pub fn assign_var(&mut self, name: &str, value: Value) -> Result<(), ()> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Ok(());
            }
        }
        if self.globals.contains_key(name) {
            self.globals.insert(name.to_string(), value);
            return Ok(());
        }
        Err(())
    }

    pub fn define_fn(&mut self, name: String, func: Function) {
        self.funcs.insert(name, func);
    }

    pub fn get_fn(&self, name: &str) -> Option<&Function> {
        self.funcs.get(name)
    }

    pub fn take_scopes(&mut self) -> Vec<HashMap<String, Value>> {
        std::mem::take(&mut self.scopes)
    }

    pub fn restore_scopes(&mut self, scopes: Vec<HashMap<String, Value>>) {
        self.scopes = scopes;
    }

    pub fn roots(&self) -> Vec<Value> {
        let mut roots = Vec::new();
        roots.extend(self.globals.values().cloned());
        for scope in &self.scopes {
            roots.extend(scope.values().cloned());
        }
        roots
    }
}

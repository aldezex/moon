use std::collections::HashMap;

use moon_core::ast::Expr;

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
}

impl Env {
    pub fn new() -> Self {
        Self::default()
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
}

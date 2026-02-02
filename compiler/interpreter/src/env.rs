use std::collections::HashMap;

use crate::Value;

#[derive(Debug, Default)]
pub struct Env {
    vars: HashMap<String, Value>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.vars.get(name)
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.vars.insert(name, value);
    }
}

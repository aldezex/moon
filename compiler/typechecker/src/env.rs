use std::collections::HashMap;

use crate::error::TypeError;
use crate::types::Type;

#[derive(Debug, Clone)]
pub struct FuncSig {
    pub params: Vec<Type>,
    pub ret: Type,
}

#[derive(Debug, Default)]
pub struct TypeEnv {
    globals: HashMap<String, Type>,
    scopes: Vec<HashMap<String, Type>>,
    funcs: HashMap<String, FuncSig>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn define_var(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        } else {
            self.globals.insert(name, ty);
        }
    }

    pub fn get_var(&self, name: &str) -> Option<&Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        self.globals.get(name)
    }

    pub fn define_fn(
        &mut self,
        name: String,
        params: Vec<Type>,
        ret: Type,
    ) -> Result<(), TypeError> {
        self.funcs.insert(name, FuncSig { params, ret });
        Ok(())
    }

    pub fn get_fn(&self, name: &str) -> Option<&FuncSig> {
        self.funcs.get(name)
    }

    pub fn take_scopes(&mut self) -> Vec<HashMap<String, Type>> {
        std::mem::take(&mut self.scopes)
    }

    pub fn restore_scopes(&mut self, scopes: Vec<HashMap<String, Type>>) {
        self.scopes = scopes;
    }
}

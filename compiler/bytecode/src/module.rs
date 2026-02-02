use std::collections::HashMap;

use crate::Instr;

pub type FuncId = usize;

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub code: Vec<Instr>,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub functions: Vec<Function>,
    pub by_name: HashMap<String, FuncId>,
    pub main: FuncId,
}

impl Module {
    pub fn get_func(&self, id: FuncId) -> Option<&Function> {
        self.functions.get(id)
    }
}

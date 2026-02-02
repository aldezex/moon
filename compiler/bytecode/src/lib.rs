mod compiler;
mod instr;
mod module;

pub use compiler::{compile, CompileError};
pub use instr::{Instr, InstrKind};
pub use module::{FuncId, Function, Module};

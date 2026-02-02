use moon_core::span::Span;
use moon_runtime::Value;

use crate::module::FuncId;

#[derive(Debug, Clone)]
pub struct Instr {
    pub kind: InstrKind,
    pub span: Span,
}

impl Instr {
    pub fn new(kind: InstrKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone)]
pub enum InstrKind {
    // Constants / stack ops
    Push(Value),
    Pop,

    // Scopes (HashMap-based, like the interpreter)
    PushScope,
    PopScope,

    // Variables
    LoadVar(String),
    DefineVar(String),
    SetVar(String),

    // Ops
    Neg,
    Not,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Control flow
    Jump(usize),
    JumpIfFalse(usize),
    JumpIfTrue(usize),

    // Calls
    Call(FuncId, usize),
    CallValue(usize),
    Return,

    // Heap / aggregates
    MakeArray(usize),
    MakeObject(Vec<String>),
    IndexGet,
    IndexSet,
}

impl std::fmt::Display for InstrKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstrKind::Push(v) => write!(f, "Push {v:?}"),
            InstrKind::Pop => write!(f, "Pop"),

            InstrKind::PushScope => write!(f, "PushScope"),
            InstrKind::PopScope => write!(f, "PopScope"),

            InstrKind::LoadVar(name) => write!(f, "LoadVar {name}"),
            InstrKind::DefineVar(name) => write!(f, "DefineVar {name}"),
            InstrKind::SetVar(name) => write!(f, "SetVar {name}"),

            InstrKind::Neg => write!(f, "Neg"),
            InstrKind::Not => write!(f, "Not"),
            InstrKind::Add => write!(f, "Add"),
            InstrKind::Sub => write!(f, "Sub"),
            InstrKind::Mul => write!(f, "Mul"),
            InstrKind::Div => write!(f, "Div"),
            InstrKind::Mod => write!(f, "Mod"),
            InstrKind::Eq => write!(f, "Eq"),
            InstrKind::Ne => write!(f, "Ne"),
            InstrKind::Lt => write!(f, "Lt"),
            InstrKind::Le => write!(f, "Le"),
            InstrKind::Gt => write!(f, "Gt"),
            InstrKind::Ge => write!(f, "Ge"),

            InstrKind::Jump(dst) => write!(f, "Jump {dst}"),
            InstrKind::JumpIfFalse(dst) => write!(f, "JumpIfFalse {dst}"),
            InstrKind::JumpIfTrue(dst) => write!(f, "JumpIfTrue {dst}"),

            InstrKind::Call(id, argc) => write!(f, "Call f{id} argc={argc}"),
            InstrKind::CallValue(argc) => write!(f, "CallValue argc={argc}"),
            InstrKind::Return => write!(f, "Return"),

            InstrKind::MakeArray(n) => write!(f, "MakeArray {n}"),
            InstrKind::MakeObject(keys) => write!(f, "MakeObject keys={keys:?}"),
            InstrKind::IndexGet => write!(f, "IndexGet"),
            InstrKind::IndexSet => write!(f, "IndexSet"),
        }
    }
}

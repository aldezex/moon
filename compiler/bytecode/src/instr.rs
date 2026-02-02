use moon_runtime::Value;

#[derive(Debug, Clone)]
pub enum Instr {
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
    Call(super::module::FuncId, usize),
    Return,

    // Heap / aggregates
    MakeArray(usize),
    MakeObject(Vec<String>),
    IndexGet,
    IndexSet,
}

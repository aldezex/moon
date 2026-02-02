use crate::GcRef;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Bool(bool),
    String(String),
    Unit,
    Function(String),
    Closure(GcRef),

    // Heap-allocated values (traced by the GC).
    Array(GcRef),
    Object(GcRef),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{i}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Unit => write!(f, "()"),
            Value::Function(name) => write!(f, "<fn {name}>"),
            Value::Closure(h) => write!(f, "<closure@{}>", h.0),
            Value::Array(h) => write!(f, "<array@{}>", h.0),
            Value::Object(h) => write!(f, "<object@{}>", h.0),
        }
    }
}

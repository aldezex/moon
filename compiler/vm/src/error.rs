use moon_core::span::Span;

#[derive(Debug, Clone)]
pub struct VmError {
    pub message: String,
    pub span: Span,
}

impl VmError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "vm error: {}", self.message)
    }
}

impl std::error::Error for VmError {}

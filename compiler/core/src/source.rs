use crate::span::Span;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Source {
    pub path: PathBuf,
    pub text: String,
}

impl Source {
    pub fn new(path: impl Into<PathBuf>, text: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            text: text.into(),
        }
    }

    pub fn from_path(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let text = std::fs::read_to_string(&path)?;
        Ok(Self { path, text })
    }

    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        // 1-based line/col.
        let mut line = 1usize;
        let mut col = 1usize;
        for (i, b) in self.text.as_bytes().iter().enumerate() {
            if i >= offset {
                break;
            }
            if *b == b'\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    pub fn render_span(&self, span: Span, message: &str) -> String {
        let start = span.start.min(self.text.len());
        let end = span.end.min(self.text.len());
        let (line, col) = self.line_col(start);

        // Extract the line that contains `start`.
        let line_start = self.text[..start]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let line_end = self.text[start..]
            .find('\n')
            .map(|i| start + i)
            .unwrap_or(self.text.len());

        let line_text = &self.text[line_start..line_end];

        let mut caret = String::new();
        let caret_pos = start.saturating_sub(line_start);
        for _ in 0..caret_pos {
            caret.push(' ');
        }

        let caret_len = (end.saturating_sub(start)).max(1);
        for _ in 0..caret_len {
            caret.push('^');
        }

        format!(
            "{}:{}:{}: {}\n{}\n{}",
            self.path.display(),
            line,
            col,
            message,
            line_text,
            caret
        )
    }
}

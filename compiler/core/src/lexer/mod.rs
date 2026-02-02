use crate::error::LexError;
use crate::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Ident(String),
    Int(i64),
    String(String),

    // Keywords
    Let,
    True,
    False,

    // Operators / punctuation
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    Equal,
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    AndAnd,
    OrOr,

    LParen,
    RParen,
    Semicolon,

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub fn lex(input: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        let b = bytes[i];

        // Whitespace
        if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
            i += 1;
            continue;
        }

        // Line comment: //...
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        // Ident / keyword
        if is_ident_start(b) {
            let start = i;
            i += 1;
            while i < bytes.len() && is_ident_continue(bytes[i]) {
                i += 1;
            }
            let text = &input[start..i];
            let kind = match text {
                "let" => TokenKind::Let,
                "true" => TokenKind::True,
                "false" => TokenKind::False,
                _ => TokenKind::Ident(text.to_string()),
            };
            tokens.push(Token {
                kind,
                span: Span::new(start, i),
            });
            continue;
        }

        // Number (int only for now)
        if b.is_ascii_digit() {
            let start = i;
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            let text = &input[start..i];
            let value = text.parse::<i64>().map_err(|_| LexError {
                message: format!("invalid integer literal: {text}"),
                span: Span::new(start, i),
            })?;
            tokens.push(Token {
                kind: TokenKind::Int(value),
                span: Span::new(start, i),
            });
            continue;
        }

        // String literal: "..."
        if b == b'"' {
            let start = i;
            i += 1; // skip opening quote
            let mut out = String::new();
            let mut closed = false;
            while i < bytes.len() {
                let b = bytes[i];
                if b == b'"' {
                    i += 1; // skip closing quote
                    tokens.push(Token {
                        kind: TokenKind::String(out),
                        span: Span::new(start, i),
                    });
                    closed = true;
                    break;
                }

                if b == b'\\' {
                    // Very small escape set for the MVP.
                    i += 1;
                    if i >= bytes.len() {
                        return Err(LexError {
                            message: "unterminated string literal".to_string(),
                            span: Span::new(start, i),
                        });
                    }
                    let esc = bytes[i];
                    let ch = match esc {
                        b'n' => '\n',
                        b't' => '\t',
                        b'"' => '"',
                        b'\\' => '\\',
                        _ => {
                            return Err(LexError {
                                message: format!("unknown escape: \\{}", esc as char),
                                span: Span::new(i - 1, i + 1),
                            })
                        }
                    };
                    out.push(ch);
                    i += 1;
                    continue;
                }

                if b.is_ascii() {
                    out.push(b as char);
                    i += 1;
                    continue;
                }

                return Err(LexError {
                    message: "non-ascii characters are not supported in string literals yet"
                        .to_string(),
                    span: Span::new(i, i + 1),
                });
            }

            if !closed {
                return Err(LexError {
                    message: "unterminated string literal".to_string(),
                    span: Span::new(start, input.len()),
                });
            }

            continue;
        }

        // Operators / punctuation (including multi-char).
        let start = i;
        let (kind, len) = match b {
            b'+' => (TokenKind::Plus, 1),
            b'-' => (TokenKind::Minus, 1),
            b'*' => (TokenKind::Star, 1),
            b'%' => (TokenKind::Percent, 1),
            b'(' => (TokenKind::LParen, 1),
            b')' => (TokenKind::RParen, 1),
            b';' => (TokenKind::Semicolon, 1),
            b'!' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    (TokenKind::BangEqual, 2)
                } else {
                    (TokenKind::Bang, 1)
                }
            }
            b'=' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    (TokenKind::EqualEqual, 2)
                } else {
                    (TokenKind::Equal, 1)
                }
            }
            b'<' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    (TokenKind::LessEqual, 2)
                } else {
                    (TokenKind::Less, 1)
                }
            }
            b'>' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    (TokenKind::GreaterEqual, 2)
                } else {
                    (TokenKind::Greater, 1)
                }
            }
            b'&' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'&' {
                    (TokenKind::AndAnd, 2)
                } else {
                    return Err(LexError {
                        message: "unexpected '&' (did you mean '&&'?)".to_string(),
                        span: Span::new(i, i + 1),
                    });
                }
            }
            b'|' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'|' {
                    (TokenKind::OrOr, 2)
                } else {
                    return Err(LexError {
                        message: "unexpected '|' (did you mean '||'?)".to_string(),
                        span: Span::new(i, i + 1),
                    });
                }
            }
            b'/' => (TokenKind::Slash, 1),
            _ => {
                return Err(LexError {
                    message: format!("unexpected character: '{}'", b as char),
                    span: Span::new(i, i + 1),
                })
            }
        };

        i += len;
        tokens.push(Token {
            kind,
            span: Span::new(start, i),
        });
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        span: Span::new(input.len(), input.len()),
    });

    Ok(tokens)
}

fn is_ident_start(b: u8) -> bool {
    b == b'_' || (b as char).is_ascii_alphabetic()
}

fn is_ident_continue(b: u8) -> bool {
    is_ident_start(b) || b.is_ascii_digit()
}

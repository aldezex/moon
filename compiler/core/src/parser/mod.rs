use crate::ast::{BinaryOp, Expr, Program, Stmt, UnaryOp};
use crate::error::ParseError;
use crate::lexer::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse_program(mut self) -> Result<Program, ParseError> {
        let mut stmts = Vec::new();
        while !self.is_eof() {
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
        }
        Ok(Program::new(stmts))
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match &self.peek().kind {
            TokenKind::Let => self.parse_let_stmt(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt, ParseError> {
        let let_tok = self.expect(|k| matches!(k, TokenKind::Let), "expected 'let'")?;

        let name_tok = self.next();
        let name = match name_tok.kind {
            TokenKind::Ident(s) => s,
            _ => {
                return Err(ParseError {
                    message: "expected identifier after 'let'".to_string(),
                    span: name_tok.span,
                })
            }
        };

        self.expect(|k| matches!(k, TokenKind::Equal), "expected '=' after identifier")?;
        let expr = self.parse_expr(0)?;

        let semi = self.maybe(|k| matches!(k, TokenKind::Semicolon));
        if semi.is_none() && !self.is_eof() {
            return Err(ParseError {
                message: "expected ';' after let statement".to_string(),
                span: expr.span(),
            });
        }

        let span = let_tok.span.merge(expr.span());
        Ok(Stmt::Let { name, expr, span })
    }

    fn parse_expr_stmt(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.parse_expr(0)?;
        let semi = self.maybe(|k| matches!(k, TokenKind::Semicolon));
        if semi.is_none() && !self.is_eof() {
            return Err(ParseError {
                message: "expected ';' after expression".to_string(),
                span: expr.span(),
            });
        }
        Ok(Stmt::Expr {
            span: expr.span(),
            expr,
        })
    }

    fn parse_expr(&mut self, min_prec: u8) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_prefix()?;

        loop {
            let (op, prec) = match self.peek_infix() {
                Some(x) => x,
                None => break,
            };

            if prec < min_prec {
                break;
            }

            let op_tok = self.next();
            let rhs = self.parse_expr(prec + 1)?;
            let span = lhs.span().merge(rhs.span());
            lhs = Expr::Binary {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
                span: span.merge(op_tok.span),
            };
        }

        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        let tok = self.next();
        match tok.kind {
            TokenKind::Int(i) => Ok(Expr::Int(i, tok.span)),
            TokenKind::True => Ok(Expr::Bool(true, tok.span)),
            TokenKind::False => Ok(Expr::Bool(false, tok.span)),
            TokenKind::String(s) => Ok(Expr::String(s, tok.span)),
            TokenKind::Ident(s) => Ok(Expr::Ident(s, tok.span)),
            TokenKind::Minus => {
                let expr = self.parse_expr(7)?;
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    span: tok.span.merge(expr.span()),
                    expr: Box::new(expr),
                })
            }
            TokenKind::Bang => {
                let expr = self.parse_expr(7)?;
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    span: tok.span.merge(expr.span()),
                    expr: Box::new(expr),
                })
            }
            TokenKind::LParen => {
                let expr = self.parse_expr(0)?;
                let close = self.expect(|k| matches!(k, TokenKind::RParen), "expected ')'")?;
                Ok(Expr::Group {
                    span: tok.span.merge(close.span),
                    expr: Box::new(expr),
                })
            }
            _ => Err(ParseError {
                message: "unexpected token in expression".to_string(),
                span: tok.span,
            }),
        }
    }

    fn peek_infix(&self) -> Option<(BinaryOp, u8)> {
        let op = match &self.peek().kind {
            TokenKind::OrOr => (BinaryOp::Or, 1),
            TokenKind::AndAnd => (BinaryOp::And, 2),
            TokenKind::EqualEqual => (BinaryOp::Eq, 3),
            TokenKind::BangEqual => (BinaryOp::Ne, 3),
            TokenKind::Less => (BinaryOp::Lt, 4),
            TokenKind::LessEqual => (BinaryOp::Le, 4),
            TokenKind::Greater => (BinaryOp::Gt, 4),
            TokenKind::GreaterEqual => (BinaryOp::Ge, 4),
            TokenKind::Plus => (BinaryOp::Add, 5),
            TokenKind::Minus => (BinaryOp::Sub, 5),
            TokenKind::Star => (BinaryOp::Mul, 6),
            TokenKind::Slash => (BinaryOp::Div, 6),
            TokenKind::Percent => (BinaryOp::Mod, 6),
            _ => return None,
        };
        Some(op)
    }

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .unwrap_or_else(|| self.tokens.last().expect("tokens must not be empty"))
    }

    fn next(&mut self) -> Token {
        let tok = self.peek().clone();
        if !matches!(tok.kind, TokenKind::Eof) {
            self.pos += 1;
        }
        tok
    }

    fn is_eof(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn maybe(&mut self, pred: impl FnOnce(&TokenKind) -> bool) -> Option<Token> {
        if pred(&self.peek().kind) {
            Some(self.next())
        } else {
            None
        }
    }

    fn expect(
        &mut self,
        pred: impl FnOnce(&TokenKind) -> bool,
        message: &'static str,
    ) -> Result<Token, ParseError> {
        let tok = self.peek().clone();
        if pred(&tok.kind) {
            Ok(self.next())
        } else {
            Err(ParseError {
                message: message.to_string(),
                span: tok.span,
            })
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Program, ParseError> {
    Parser::new(tokens).parse_program()
}

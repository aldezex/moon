use crate::ast::{BinaryOp, Expr, Param, Program, Stmt, TypeExpr, UnaryOp};
use crate::error::ParseError;
use crate::lexer::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

#[derive(Debug, Copy, Clone)]
enum Terminator {
    Eof,
    RBrace,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse_program(mut self) -> Result<Program, ParseError> {
        let (stmts, tail) = self.parse_sequence(Terminator::Eof)?;
        Ok(Program::new(stmts, tail))
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

        let ty = if self.maybe(|k| matches!(k, TokenKind::Colon)).is_some() {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(
            |k| matches!(k, TokenKind::Equal),
            "expected '=' after identifier",
        )?;
        let expr = self.parse_expr(0)?;

        self.expect(
            |k| matches!(k, TokenKind::Semicolon),
            "expected ';' after let statement",
        )?;

        let span = let_tok.span.merge(expr.span());
        Ok(Stmt::Let {
            name,
            ty,
            expr,
            span,
        })
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt, ParseError> {
        let ret_tok = self.expect(|k| matches!(k, TokenKind::Return), "expected 'return'")?;

        // `return;`
        if let Some(semi) = self.maybe(|k| matches!(k, TokenKind::Semicolon)) {
            let span = ret_tok.span.merge(semi.span);
            return Ok(Stmt::Return { expr: None, span });
        }

        // `return <expr>;`
        let expr = self.parse_expr(0)?;
        let semi = self.expect(
            |k| matches!(k, TokenKind::Semicolon),
            "expected ';' after return",
        )?;

        let span = ret_tok.span.merge(semi.span);
        Ok(Stmt::Return {
            expr: Some(expr),
            span,
        })
    }

    fn parse_fn_stmt(&mut self) -> Result<Stmt, ParseError> {
        let fn_tok = self.expect(|k| matches!(k, TokenKind::Fn), "expected 'fn'")?;

        let name_tok = self.next();
        let name = match name_tok.kind {
            TokenKind::Ident(s) => s,
            _ => {
                return Err(ParseError {
                    message: "expected identifier after 'fn'".to_string(),
                    span: name_tok.span,
                })
            }
        };

        self.expect(
            |k| matches!(k, TokenKind::LParen),
            "expected '(' after fn name",
        )?;
        let mut params = Vec::new();
        if !matches!(self.peek().kind, TokenKind::RParen) {
            loop {
                let param_name_tok = self.next();
                let param_name = match param_name_tok.kind {
                    TokenKind::Ident(s) => s,
                    _ => {
                        return Err(ParseError {
                            message: "expected parameter name".to_string(),
                            span: param_name_tok.span,
                        })
                    }
                };

                self.expect(
                    |k| matches!(k, TokenKind::Colon),
                    "expected ':' after parameter name",
                )?;
                let ty = self.parse_type()?;
                let span = param_name_tok.span.merge(ty.span());
                params.push(Param {
                    name: param_name,
                    ty,
                    span,
                });

                if self.maybe(|k| matches!(k, TokenKind::Comma)).is_some() {
                    if matches!(self.peek().kind, TokenKind::RParen) {
                        break;
                    }
                    continue;
                }
                break;
            }
        }
        self.expect(
            |k| matches!(k, TokenKind::RParen),
            "expected ')' after parameters",
        )?;

        self.expect(
            |k| matches!(k, TokenKind::Arrow),
            "expected '->' after parameters",
        )?;
        let ret_ty = self.parse_type()?;

        let body = self.parse_block_expr()?;
        let span = fn_tok.span.merge(body.span());
        Ok(Stmt::Fn {
            name,
            params,
            ret_ty,
            body,
            span,
        })
    }

    fn parse_expr(&mut self, min_prec: u8) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_prefix()?;
        lhs = self.parse_postfix(lhs)?;

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
            TokenKind::If => self.parse_if_expr(tok),
            TokenKind::LBrace => self.parse_block_expr_from_open(tok),
            TokenKind::LBracket => self.parse_array_expr_from_open(tok),
            TokenKind::Hash => self.parse_object_expr(tok),
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

    fn parse_postfix(&mut self, mut expr: Expr) -> Result<Expr, ParseError> {
        loop {
            if matches!(self.peek().kind, TokenKind::LParen) {
                expr = self.parse_call_expr(expr)?;
                continue;
            }
            if matches!(self.peek().kind, TokenKind::LBracket) {
                expr = self.parse_index_expr(expr)?;
                continue;
            }
            break;
        }
        Ok(expr)
    }

    fn parse_call_expr(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let open = self.expect(|k| matches!(k, TokenKind::LParen), "expected '('")?;
        let mut args = Vec::new();
        if !matches!(self.peek().kind, TokenKind::RParen) {
            loop {
                let arg = self.parse_expr(0)?;
                args.push(arg);

                if self.maybe(|k| matches!(k, TokenKind::Comma)).is_some() {
                    if matches!(self.peek().kind, TokenKind::RParen) {
                        break;
                    }
                    continue;
                }
                break;
            }
        }
        let close = self.expect(|k| matches!(k, TokenKind::RParen), "expected ')'")?;

        let span = callee.span().merge(open.span).merge(close.span);
        Ok(Expr::Call {
            callee: Box::new(callee),
            args,
            span,
        })
    }

    fn parse_index_expr(&mut self, target: Expr) -> Result<Expr, ParseError> {
        let open = self.expect(|k| matches!(k, TokenKind::LBracket), "expected '['")?;
        let index = self.parse_expr(0)?;
        let close = self.expect(|k| matches!(k, TokenKind::RBracket), "expected ']'")?;

        let span = target.span().merge(open.span).merge(close.span);
        Ok(Expr::Index {
            target: Box::new(target),
            index: Box::new(index),
            span,
        })
    }

    fn parse_array_expr_from_open(&mut self, open: Token) -> Result<Expr, ParseError> {
        let mut elements = Vec::new();
        if !matches!(self.peek().kind, TokenKind::RBracket) {
            loop {
                let elem = self.parse_expr(0)?;
                elements.push(elem);

                if self.maybe(|k| matches!(k, TokenKind::Comma)).is_some() {
                    if matches!(self.peek().kind, TokenKind::RBracket) {
                        break;
                    }
                    continue;
                }
                break;
            }
        }
        let close = self.expect(|k| matches!(k, TokenKind::RBracket), "expected ']'")?;
        let span = open.span.merge(close.span);
        Ok(Expr::Array { elements, span })
    }

    fn parse_object_expr(&mut self, hash: Token) -> Result<Expr, ParseError> {
        self.expect(|k| matches!(k, TokenKind::LBrace), "expected '{' after '#'")?;

        let mut props: Vec<(String, Expr)> = Vec::new();
        if !matches!(self.peek().kind, TokenKind::RBrace) {
            loop {
                let key_tok = self.next();
                let key = match key_tok.kind {
                    TokenKind::Ident(s) => s,
                    TokenKind::String(s) => s,
                    _ => {
                        return Err(ParseError {
                            message: "expected object key (identifier or string)".to_string(),
                            span: key_tok.span,
                        })
                    }
                };

                self.expect(|k| matches!(k, TokenKind::Colon), "expected ':' after key")?;
                let value = self.parse_expr(0)?;
                props.push((key, value));

                if self.maybe(|k| matches!(k, TokenKind::Comma)).is_some() {
                    if matches!(self.peek().kind, TokenKind::RBrace) {
                        break;
                    }
                    continue;
                }
                break;
            }
        }

        let close = self.expect(|k| matches!(k, TokenKind::RBrace), "expected '}'")?;
        let span = hash.span.merge(close.span);
        Ok(Expr::Object { props, span })
    }

    fn parse_if_expr(&mut self, if_tok: Token) -> Result<Expr, ParseError> {
        let cond = self.parse_expr(0)?;

        let then_branch = self.parse_block_expr()?;

        self.expect(|k| matches!(k, TokenKind::Else), "expected 'else'")?;

        let else_branch = match self.peek().kind {
            TokenKind::If => {
                // else if ...
                let tok = self.next();
                self.parse_if_expr(tok)?
            }
            TokenKind::LBrace => self.parse_block_expr()?,
            _ => {
                let tok = self.peek().clone();
                return Err(ParseError {
                    message: "expected 'if' or '{' after 'else'".to_string(),
                    span: tok.span,
                });
            }
        };

        let span = if_tok.span.merge(else_branch.span());
        Ok(Expr::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
            span,
        })
    }

    fn parse_block_expr(&mut self) -> Result<Expr, ParseError> {
        let open = self.expect(|k| matches!(k, TokenKind::LBrace), "expected '{'")?;
        self.parse_block_expr_from_open(open)
    }

    fn parse_block_expr_from_open(&mut self, open: Token) -> Result<Expr, ParseError> {
        let (stmts, tail) = self.parse_sequence(Terminator::RBrace)?;
        let close = self.expect(|k| matches!(k, TokenKind::RBrace), "expected '}'")?;

        let span = open.span.merge(close.span);
        Ok(Expr::Block {
            stmts,
            tail: tail.map(Box::new),
            span,
        })
    }

    fn parse_sequence(
        &mut self,
        terminator: Terminator,
    ) -> Result<(Vec<Stmt>, Option<Expr>), ParseError> {
        let mut stmts = Vec::new();
        let mut tail = None;

        while !self.at_terminator(terminator) {
            match &self.peek().kind {
                TokenKind::Let => {
                    stmts.push(self.parse_let_stmt()?);
                    continue;
                }
                TokenKind::Return => {
                    stmts.push(self.parse_return_stmt()?);
                    continue;
                }
                TokenKind::Fn => {
                    if matches!(terminator, Terminator::RBrace) {
                        let tok = self.peek().clone();
                        return Err(ParseError {
                            message:
                                "function declarations are only allowed at top-level (for now)"
                                    .to_string(),
                            span: tok.span,
                        });
                    }
                    stmts.push(self.parse_fn_stmt()?);
                    continue;
                }
                _ => {}
            }

            let expr = self.parse_expr(0)?;

            // Assignment statement: <lvalue> = <expr>;
            if self.maybe(|k| matches!(k, TokenKind::Equal)).is_some() {
                if !is_assignable(&expr) {
                    return Err(ParseError {
                        message: "invalid assignment target".to_string(),
                        span: expr.span(),
                    });
                }

                let rhs = self.parse_expr(0)?;
                self.expect(
                    |k| matches!(k, TokenKind::Semicolon),
                    "expected ';' after assignment",
                )?;

                let span = expr.span().merge(rhs.span());
                stmts.push(Stmt::Assign {
                    target: expr,
                    expr: rhs,
                    span,
                });
                continue;
            }

            if self.maybe(|k| matches!(k, TokenKind::Semicolon)).is_some() {
                stmts.push(Stmt::Expr {
                    span: expr.span(),
                    expr,
                });
                continue;
            }

            if self.at_terminator(terminator) {
                tail = Some(expr);
                break;
            }

            return Err(ParseError {
                message: "expected ';' after expression".to_string(),
                span: expr.span(),
            });
        }

        Ok((stmts, tail))
    }

    fn parse_type(&mut self) -> Result<TypeExpr, ParseError> {
        let tok = self.next();
        match tok.kind {
            TokenKind::Ident(base) => {
                let base_span = tok.span;
                if self.maybe(|k| matches!(k, TokenKind::Less)).is_some() {
                    let mut args = Vec::new();
                    if matches!(self.peek().kind, TokenKind::Greater) {
                        return Err(ParseError {
                            message: "expected type argument".to_string(),
                            span: self.peek().span,
                        });
                    }
                    loop {
                        let ty = self.parse_type()?;
                        args.push(ty);
                        if self.maybe(|k| matches!(k, TokenKind::Comma)).is_some() {
                            continue;
                        }
                        break;
                    }
                    let close = self.expect(
                        |k| matches!(k, TokenKind::Greater),
                        "expected '>' to close type arguments",
                    )?;
                    let span = base_span.merge(close.span);
                    Ok(TypeExpr::Generic { base, args, span })
                } else {
                    Ok(TypeExpr::Named(base, base_span))
                }
            }
            _ => Err(ParseError {
                message: "expected type name".to_string(),
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

    fn at_terminator(&self, terminator: Terminator) -> bool {
        match terminator {
            Terminator::Eof => self.is_eof(),
            Terminator::RBrace => matches!(self.peek().kind, TokenKind::RBrace),
        }
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

fn is_assignable(expr: &Expr) -> bool {
    matches!(expr, Expr::Ident(_, _) | Expr::Index { .. })
}

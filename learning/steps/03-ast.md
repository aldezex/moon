# 03 - AST (Abstract Syntax Tree)

## Que es un AST

Un AST (Abstract Syntax Tree) es una representacion "limpia" del programa:
- El lexer/parser entiende el texto.
- El AST captura la **estructura**: expresiones, operadores, declaraciones.
- El interpreter/typechecker/compilador trabajan sobre el AST, no sobre tokens.

## Nuestro AST (MVP)

Archivo:
- `compiler/core/src/ast/mod.rs`

### Program

`Program` es simplemente:
- `stmts: Vec<Stmt>`

### Stmt (statements)

Por ahora:
- `Let { name, expr, span }`
- `Expr { expr, span }` (una expresion como statement)

Nota:
- Ambos guardan `span`, y tenemos `Stmt::span()` para recuperarlo.

### Expr (expressions)

Soportamos:
- Literales:
  - `Int(i64, Span)`
  - `Bool(bool, Span)`
  - `String(String, Span)`
- Identificadores:
  - `Ident(String, Span)`
- Unarios:
  - `Unary { op, expr, span }` con `UnaryOp::{Neg, Not}`
- Binarios:
  - `Binary { lhs, op, rhs, span }` con `BinaryOp` para `+ - * / ...`
- Agrupacion:
  - `Group { expr, span }` (parentesis)

Todas las expresiones tienen `Expr::span()` para obtener su rango.

## Por que cada nodo lleva Span

Esto nos permite:
- Errores con ubicacion exacta (type errors, runtime errors, etc.)
- Tooling futuro: hover, go-to-def, formatter, etc.

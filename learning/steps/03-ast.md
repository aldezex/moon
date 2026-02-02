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

`Program` representa un archivo/script completo como una secuencia de statements + una posible expresion final (tail expression):
- `stmts: Vec<Stmt>` (siempre "statements" reales)
- `tail: Option<Expr>` (la ultima expresion **sin** `;`, si existe)

Regla clave:
- El valor del programa es `tail` si existe, o `Unit` si no.

### Stmt (statements)

Por ahora:
- `Let { name, ty, expr, span }`
  - `ty` es una anotacion opcional (`let x: Int = 1;`), util para el typechecker.
- `Fn { name, params, ret_ty, body, span }`
  - Declaracion de funcion (solo top-level por ahora).
- `Expr { expr, span }`
  - Expression statement (siempre termina en `;` y **descarta** el valor).

Nota:
- Ambos guardan `span`, y tenemos `Stmt::span()` para recuperarlo.

### TypeExpr y Param

Para tipado estricto, el AST tambien modela anotaciones de tipo:
- `TypeExpr::Named(String, Span)` (ej: `Int`, `Bool`, `String`)
- `Param { name, ty, span }` (parametros de funciones)

### Expr (expressions)

Soportamos:
- Literales:
  - `Int(i64, Span)`
  - `Bool(bool, Span)`
  - `String(String, Span)`
- Identificadores:
  - `Ident(String, Span)`
- Bloques:
  - `Block { stmts, tail, span }`
  - Un bloque introduce un scope nuevo y devuelve:
    - `tail` si existe (ultima expresion sin `;`)
    - `Unit` si no
- If/else:
  - `If { cond, then_branch, else_branch, span }`
- Unarios:
  - `Unary { op, expr, span }` con `UnaryOp::{Neg, Not}`
- Binarios:
  - `Binary { lhs, op, rhs, span }` con `BinaryOp` para `+ - * / ...`
- Llamadas:
  - `Call { callee, args, span }` (por ahora llamamos funciones por nombre)
- Agrupacion:
  - `Group { expr, span }` (parentesis)

Todas las expresiones tienen `Expr::span()` para obtener su rango.

## Por que cada nodo lleva Span

Esto nos permite:
- Errores con ubicacion exacta (type errors, runtime errors, etc.)
- Tooling futuro: hover, go-to-def, formatter, etc.

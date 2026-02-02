# 05 - Parser (Pratt / precedencias)

## Que hace el parser

El parser convierte tokens en AST:

```
[Token] -> Program (AST)
```

Archivo:
- `compiler/core/src/parser/mod.rs`

## Gramatica (aproximada) del MVP

```
Program  := Stmt* EOF
Stmt     := "let" Ident "=" Expr ";"?
          | Expr ";"?
Expr     := (Pratt parser con precedencias)
Primary  := Int | True | False | String | Ident | "(" Expr ")"
Unary    := ("-" | "!") Expr
```

Nota: por simplicidad, el `;` es obligatorio salvo que estemos justo antes de `EOF`.

## Por que Pratt parser

Un Pratt parser es muy practico para expresiones con muchos operadores:
- Es compacto.
- Maneja precedencias y asociatividad sin escribir 10 funciones (`parse_term`, `parse_factor`, ...).

En nuestro caso:
- `parse_expr(min_prec)` parsea una expresion con precedencia minima.
- `parse_prefix()` parsea primarios y unarios.
- `peek_infix()` detecta el operador binario y su precedencia.

## Tabla de precedencias

En `peek_infix()` definimos (de menor a mayor):

1) `||`
2) `&&`
3) `== !=`
4) `< <= > >=`
5) `+ -`
6) `* / %`

Los unarios (`-expr`, `!expr`) tienen precedencia mas alta (se parsean con `parse_expr(7)`).

## Asociatividad (detalle importante)

Este Pratt parser construye operadores binarios como **left-associative** usando:
- `rhs = parse_expr(prec + 1)`

Ejemplo:
- `1 - 2 - 3` se parsea como `(1 - 2) - 3`

## Errores del parser

Cuando un token no cumple lo esperado, devolvemos `ParseError { message, span }`.

Ejemplos:
- despues de `let` no hay identificador
- falta `=`
- falta `)`
- falta `;`

De nuevo: la CLI los imprime con `Source::render_span(...)`.

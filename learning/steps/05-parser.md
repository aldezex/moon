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
Program  := Stmt* TailExpr? EOF
Stmt     := LetStmt | FnStmt | ExprStmt
LetStmt  := "let" Ident (":" Type)? "=" Expr ";"
FnStmt   := "fn" Ident "(" Params? ")" "->" Type Block
ExprStmt := Expr ";"
TailExpr := Expr            // solo si viene antes de EOF (o antes de '}' dentro de un Block)
Expr     := (Pratt parser con precedencias)
Primary  := Int | True | False | String | Ident | "(" Expr ")"
Unary    := ("-" | "!") Expr
Postfix  := Call
Call     := Primary "(" Args? ")"
Block    := "{" Stmt* TailExpr? "}"
IfExpr   := "if" Expr Block "else" (Block | IfExpr)
Type     := Ident
```

Regla clave:
- Un `ExprStmt` siempre termina en `;` y su valor se descarta.
- La ultima expresion sin `;` en un `Program` o `Block` es la **tail expression** y define el valor de esa unidad.

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

Los calls (`f(x)`) se parsean como postfix con precedencia aun mas alta que los binarios.

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
- falta `}`
- falta `;`
- falta `else`
- `fn` fuera de top-level (restriccion actual)

De nuevo: la CLI los imprime con `Source::render_span(...)`.

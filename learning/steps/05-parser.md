# 05 - Parser (Pratt + statements + tipos)

## Que hace el parser

El parser toma tokens y construye un AST:

```
[Token] -> Program (AST)
```

Archivo:
- `compiler/core/src/parser/mod.rs`

## La gramatica del MVP (aproximada)

Moon es "Rust-like" en una cosa clave: los bloques devuelven valor via tail expression.

Esto se puede pensar asi:

```
Program  := Seq(terminator = EOF)

Seq(T)   := (Stmt)* (Expr)?   // Expr solo si viene justo antes del terminador

Stmt     := LetStmt | FnStmt | AssignStmt | ExprStmt
LetStmt  := "let" Ident (":" Type)? "=" Expr ";"
FnStmt   := "fn" Ident "(" Params? ")" "->" Type Block
AssignStmt := Expr "=" Expr ";"     // pero Expr debe ser lvalue (Ident o Index)
ExprStmt := Expr ";"

Block    := "{" Seq(terminator = "}") "}"
IfExpr   := "if" Expr Block "else" (Block | IfExpr)

Expr     := Prefix (Postfix)* (Infix ...)*
Prefix   := literals | Ident | Group | IfExpr | Block | Array | Object | Unary
Group    := "(" Expr ")"
Unary    := ("-" | "!") Expr
Postfix  := Call | Index
Call     := Expr "(" Args? ")"
Index    := Expr "[" Expr "]"
Array    := "[" (Expr ("," Expr)*)? ","? "]"
Object   := "#" "{" (Key ":" Expr ("," Key ":" Expr)*)? ","? "}"
Key      := Ident | String

Type     := Ident ("<" Type ("," Type)* ">")?
```

Notas importantes:
- `FnStmt` solo se permite en top-level (no dentro de blocks) para mantener el modelo simple.
- `AssignStmt` se detecta en el nivel de statements (no como operador infix), para no complicar precedencias.

## Parsing de statements y tail expression

La funcion clave es `parse_sequence(terminator)`:
- itera tokens hasta ver el terminador (`EOF` o `}`)
- parsea statements (`let`, `fn`) cuando corresponden
- si encuentra una expresion:
  - si termina en `;` -> `Stmt::Expr`
  - si le sigue `=` -> `Stmt::Assign`
  - si llega justo antes del terminador -> se vuelve `tail`
  - si no, error ("expected ';'")

Este diseño hace que:
- el parser decida claramente que es statement y que es valor de bloque
- el interpreter/VM tengan un AST mas directo (no necesitan adivinar semicolons)

## Expresiones: Pratt parser

Para expresiones con operadores, usamos Pratt:
- `parse_expr(min_prec)`
- `parse_prefix()`
- `peek_infix()` devuelve (op, prec)

### Tabla de precedencias

De menor a mayor:

1) `||`
2) `&&`
3) `== !=`
4) `< <= > >=`
5) `+ -`
6) `* / %`

Los unarios (`-x`, `!x`) se parsean con precedencia mas alta.

### Postfix (calls e indexing)

Los postfix tienen "precedencia mas alta que todo":
- primero parseamos un prefix (literal/ident/...),
- luego aplicamos postfix en loop:
  - `f(1)(2)[0]` se arma como un arbol encadenado

Esto vive en `parse_postfix`.

## Parsing de tipos

Los tipos se parsean solo en contexto de tipos:
- despues de `:`
- despues de `->`

Esto es importante porque:
- `<` y `>` en expresiones significan comparacion
- `<` y `>` en tipos significan "generics"

En `parse_type()`:
- parsea un Ident base (`Array`, `Object`, `Int`, etc.)
- si viene `<`, parsea argumentos tipo recursivamente hasta `>`

Ejemplos:
- `Int`
- `Array<Int>`
- `Object<Array<Int>>`

El resultado es `TypeExpr` en el AST (no es el tipo "resuelto"; eso lo hace el typechecker).

## Errores del parser

Cuando algo no cumple lo esperado, devolvemos:
- `ParseError { message, span }`

Ejemplos:
- falta `;` despues de un statement
- falta `}` para cerrar block
- `fn` dentro de block (restriccion MVP)
- assignment con target invalido (`(x) = 1;`)

La CLI lo imprime via `Source::render_span`.

## Mini ejercicios

1) Permitir `fn` dentro de blocks (scoping de funciones).
2) Soportar `else if` con una sintaxis distinta (ej: `elif`).
3) Convertir assignment en un operador (tipo Rust) y ajustar Pratt para asociatividad derecha.

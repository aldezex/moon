# 05 - Parser (Pratt + statements + tipos)

Este parser convierte tokens en AST.
En un lenguaje real, el parser es donde se fijan:
- precedencias
- asociatividad
- disambiguaciones (ej: `fn name(...)` vs `fn(...)`)

Archivo:
- `compiler/core/src/parser/mod.rs`

## 0) Output y contrato

Input:
- `Vec<Token>` (del lexer)

Output:
- `Program { stmts, tail }` (AST)

Errores:
- `ParseError { message, span }`

## 1) Gramatica del MVP (aproximada)

Notacion informal (no es una CFG completa, pero sirve como guia):

```
Program  := Seq(terminator = EOF)
Seq(T)   := (Stmt)* (Expr)?   // Expr solo si viene justo antes del terminador

Stmt       := LetStmt | ReturnStmt | FnDecl | AssignStmt | ExprStmt
LetStmt    := "let" Ident (":" Type)? "=" Expr ";"
ReturnStmt := "return" Expr? ";"
FnDecl     := "fn" Ident "(" Params? ")" "->" Type Block
AssignStmt := Expr "=" Expr ";"   // Expr debe ser lvalue (Ident o Index)
ExprStmt   := Expr ";"

// Expresiones
Expr     := Prefix (Postfix)* (Infix ...)*
Prefix   := literals | Ident | Group | IfExpr | Block | Array | Object | Unary | FnExpr
FnExpr   := "fn" "(" Params? ")" "->" Type Block

Postfix  := Call | Index
Call     := Expr "(" Args? ")"
Index    := Expr "[" Expr "]"

Block    := "{" Seq(terminator = "}") "}"
IfExpr   := "if" Expr Block "else" (Block | IfExpr)

Type     := Ident ("<" Type ("," Type)* ">")?
Params   := Param ("," Param)* ","?
Param    := Ident ":" Type
```

Puntos clave:
- `FnDecl` es item top-level: `fn name(...) ...`
- `FnExpr` es expresion anonima: `fn(...) ...`
- `Call` y `Index` son postfix y tienen la precedencia mas alta

## 2) Disambiguacion: `fn name` vs `fn (`

Mismo keyword, dos construcciones.

Decision de diseno:
- `fn <ident>` se parsea como `Stmt::Fn` (solo top-level)
- `fn (` se parsea como `Expr::Fn` (en cualquier expresion)

Implementacion:
- en `parse_sequence`, cuando el token actual es `Fn`, hacemos lookahead:
  - si el siguiente token es `Ident`, intentamos `parse_fn_stmt`
  - si no, dejamos que el Pratt parser consuma `fn` como `parse_fn_expr`

Archivo:
- `compiler/core/src/parser/mod.rs`:
  - `parse_fn_stmt`
  - `parse_fn_expr`
  - `parse_sequence`

Esto permite:

```moon
let f = fn(x: Int) -> Int { x + 1 };
```

sin permitir:

```moon
{ fn named() -> Int { 0 } }
```

## 3) Parsing de statements + tail expression

Funcion clave:
- `parse_sequence(terminator)`

Invariante:
- lee hasta `EOF` o `}`

Algoritmo:
1) mientras no ve terminator:
   - si ve `let`: parsea `Stmt::Let`
   - si ve `return`: parsea `Stmt::Return`
   - si ve `fn` y next token es ident:
     - solo permitido si terminator == EOF
     - parsea `Stmt::Fn`
   - en otro caso: parsea una expresion `Expr`
2) luego decide si esa expresion es:
   - assignment statement (`=` despues de expr)
   - expression statement (`;` despues)
   - tail expression (si llega justo antes del terminator)
   - error (si no hay `;` y tampoco es tail)

Esto fija la semantica estilo Rust:
- `expr;` => statement
- `expr` al final => valor

## 4) Expresiones con Pratt parser

Pratt parser = parseo por precedencias en runtime.

Componentes:
- `parse_prefix()` parsea:
  - literales, ident, block, if, array, object, unary, fn-expr
- `parse_postfix(expr)` aplica repetidamente:
  - call `(...)`
  - index `[...]`
- `parse_expr(min_prec)`:
  - parsea prefix
  - aplica postfix
  - luego consume infix mientras `prec >= min_prec`

Precedencias (baja -> alta):
1) `||`
2) `&&`
3) `== !=`
4) `< <= > >=`
5) `+ -`
6) `* / %`

Unarios (`-x`, `!x`) tienen precedencia mas alta.

Short-circuit:
- El parser solo construye AST.
- Short-circuit se implementa en interpreter/VM.

## 5) Tipos en el parser (TypeExpr)

Los tipos solo se parsean en contexto de tipos:
- despues de `:`
- despues de `->`

Motivo:
- `<` y `>` en expresiones son comparadores
- `<` y `>` en tipos son generics

`parse_type()`:
- parsea Ident base
- si ve `<`, parsea args recursivamente

Ejemplos:
- `Int`
- `Array<Int>`
- `Object<Array<Int>>`

## 6) Errores (ParseError) y spans

Ejemplos de errores:
- `expected ';' after expression`
- `function declarations are only allowed at top-level`
- assignment target invalido

Spans:
- se usan spans de tokens o spans de sub-expresiones

## 7) Practica: lee el parser con un ejemplo

Input:

```moon
let f = { let x = 10; fn(y: Int) -> Int { x + y } };
f(1)
```

Pasos:
- `parse_sequence(EOF)` reconoce `let`.
- RHS del `let` es `Expr::Block`.
- dentro del block, tail es `Expr::Fn` (porque `fn` seguido de `(`)
- `f(1)` es `Expr::Call` con callee `Ident("f")`

Ejercicio:
- agrega un test que asegure que `fn(...)` en tail expression parsea como `Expr::Fn`.

## 8) Ejercicios

1) Agrega `else if` como azucar sintactico (ya soportado via `else` + `IfExpr`).
2) Implementa recuperacion de errores (sync en `;` y `}`) para reportar multiples errores.
3) Agrega sintaxis para function types en `TypeExpr` y discute ambiguedades con `(`.

# 03 - AST (Abstract Syntax Tree)

## AST vs CST (idea rapida)

Cuando parseas texto tienes dos caminos:

- CST (Concrete Syntax Tree): mantiene casi todo "tal cual" esta escrito (incluye muchas cosas de formato).
- AST (Abstract Syntax Tree): representa la estructura semantica del programa.

Moon usa AST porque:
- el interpreter y la VM quieren "estructura", no tokens
- el typechecker quiere reglas sobre expresiones, no sobre caracteres
- el formatter/LSP a futuro tambien se benefician

## Archivo principal

- `compiler/core/src/ast/mod.rs`

## Program: statements + tail expression

Moon sigue una regla estilo Rust:
- `expr;` descarta el valor (es un statement)
- la ultima expresion **sin** `;` es el resultado del bloque/programa

Para modelar esto sin trucos:

- `Program { stmts: Vec<Stmt>, tail: Option<Expr> }`

El valor del programa:
- si `tail` existe, es ese valor
- si no, `Unit`

Esto hace que:
- `let x = 1; x` tenga resultado `1`
- `let x = 1; x;` tenga resultado `Unit`

## Statements (Stmt)

Hoy:

1) `Let`
   - `let name (: Type)? = expr;`
   - modelado como:
     - `name: String`
     - `ty: Option<TypeExpr>`
     - `expr: Expr`
     - `span: Span`

2) `Assign` (assignment statement)
   - `target = expr;`
   - donde `target` por ahora puede ser:
     - un identificador (`x`)
     - un index (`arr[0]`, `obj["k"]`)

3) `Return`
   - `return expr?;`
   - `expr` es opcional (`return;` devuelve `Unit`)
   - semantica: solo permitido dentro de funciones (lo valida el typechecker)
   - modelado como:
     - `expr: Option<Expr>`
     - `span: Span`

4) `Fn` (declaracion de funcion, top-level)
   - `fn name(params...) -> Type { ... }`
   - el body es un `Expr` (normalmente un Block)

5) `Expr` (expression statement)
   - `expr;`
   - se evalua y se descarta

## TypeExpr: tipos en el AST

Moon tiene tipado estricto, asi que necesitamos representar tipos en el AST.

Hoy soportamos:

- `TypeExpr::Named("Int" | "Bool" | "String" | "Unit")`
- `TypeExpr::Generic { base, args }`
  - ejemplo: `Array<Int>`, `Object<String>`

Nota: esto es sintaxis de tipos, no "types ya resueltos".
El typechecker hace "lowering" de `TypeExpr` a su propio enum `Type`.

## Expressions (Expr)

Primitivas:
- `Int(i64, Span)`
- `Bool(bool, Span)`
- `String(String, Span)`
- `Ident(String, Span)`

Agregados (heap objects):
- `Array { elements, span }`
  - `[1, 2, 3]`
- `Object { props, span }`
  - `#{ a: 1, "b": 2 }`

Control:
- `Block { stmts, tail, span }`
  - `{ let x = 1; x }`
- `If { cond, then_branch, else_branch, span }`
  - `if cond { ... } else { ... }`

Operadores:
- `Unary { op, expr, span }`
  - `-x`, `!x`
- `Binary { lhs, op, rhs, span }`
  - `+ - * / %`
  - `== != < <= > >= && ||`

Calls + indexing:
- `Call { callee, args, span }`
  - `f(1, 2)`
  - por ahora, el callee se restringe a un nombre (lo valida el typechecker)
- `Index { target, index, span }`
  - `arr[0]`
  - `obj["k"]`

Grouping:
- `Group { expr, span }`
  - `(expr)`

## Spans: por que cada nodo tiene uno

Span es el pegamento del tooling:
- errores con ubicacion exacta
- en la VM, a futuro, mapear `ip -> span` para errores de runtime
- refactors: agregar features sin perder diagnosticos

Regla de oro:
- si un nodo representa algo que el usuario escribio, debe tener Span.

## Mini ejercicios

1) Agrega un nuevo literal (por ejemplo `null`) al AST.
2) Agrega un `Expr::While` (solo el AST, sin parser/interpreter) para practicar.
3) Cambia `Object` para permitir keys computed (ej: `#{ "a" + "b": 1 }`) y piensa el impacto.

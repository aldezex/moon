# 03 - AST (Abstract Syntax Tree)

El AST es el "IR" humano: una representacion estructural del programa.
En Moon, el AST es el contrato entre:
- parser
- typechecker
- interpreter
- bytecode compiler

Si el AST esta mal diseniado, todo lo demas se vuelve doloroso.

Archivo principal:
- `compiler/core/src/ast/mod.rs`

## 0) AST vs CST (rapido, pero con criterio)

- CST (Concrete Syntax Tree): preserva casi todo lo textual (tokens, parentesis, etc)
- AST (Abstract Syntax Tree): preserva estructura semantica

Moon usa AST porque:
- el evaluator (interpreter/VM) quiere estructura, no puntuacion
- el typechecker quiere reglas sobre nodos semanticos
- tooling (LSP/diagnosticos) cuelga de spans del AST

## 1) Program: statements + tail expression

Moon es expresion-oriented, estilo Rust:
- `expr;` descarta el valor (statement)
- la ultima expresion sin `;` es el valor del bloque/programa

Por eso `Program` es:
- `stmts: Vec<Stmt>`
- `tail: Option<Expr>`

Esto evita hacks (como convertir todo a statement) y hace que el backend sea directo.

## 2) Statements (Stmt)

### 2.1 `Stmt::Let`
- `let name (: Type)? = expr;`

### 2.2 `Stmt::Assign`
- `target = expr;`
- `target` valido (MVP):
  - `Ident`
  - `Index`

### 2.3 `Stmt::Return`
- `return expr?;`
- `return;` equivale a devolver `Unit`

Regla semantica:
- solo permitido dentro de funciones/closures (lo valida el typechecker)

### 2.4 `Stmt::Fn` (item top-level)
- `fn name(params...) -> Type { ... }`

Nota:
- es un item, no una expresion.
- permite call-before-definition via pre-pass.

### 2.5 `Stmt::Expr`
- `expr;`

## 3) TypeExpr: sintaxis de tipos

El parser produce `TypeExpr` (sintaxis):
- `Named("Int"|"Bool"|"String"|"Unit")`
- `Generic { base, args }` (ej: `Array<Int>`, `Object<String>`)

El typechecker hace lowering a `Type`.

Limitacion MVP importante:
- aun no hay sintaxis para tipos de funcion (ej: `(Int)->Int`) en el source
- pero el typechecker si tiene `Type::Function` internamente

## 4) Expressions (Expr)

### 4.1 Literales y variables
- `Int`, `Bool`, `String`, `Ident`

### 4.2 Funciones anonimas (Expr::Fn)
- sintaxis:
  - `fn(params...) -> Type { ... }`
- produce un **valor**:
  - en runtime es una closure (`Value::Closure`) con environment capturado

Nota:
- `Stmt::Fn` define un item top-level (nombre obligatorio)
- `Expr::Fn` es anonima (no nombre) y puede aparecer en cualquier expresion

### 4.3 Agregados
- `Array { elements }`
- `Object { props }`

### 4.4 Control y estructura
- `Block { stmts, tail }`
- `If { cond, then_branch, else_branch }`

### 4.5 Operadores
- `Unary { op, expr }`
- `Binary { lhs, op, rhs }`

### 4.6 Call e Index
- `Call { callee: Expr, args }`
  - el callee es una expresion: puede ser `Ident`, `Expr::Fn`, una variable que contiene closure, etc.
- `Index { target, index }`

## 5) Spans como pegamento

Cada nodo relevante tiene `Span`.
Regla:
- si es algo que el usuario escribio, tiene span.

Los spans habilitan:
- errores con ubicacion
- debug info en bytecode
- hover/definition en LSP

## 6) Ejemplo: AST mental de una closure

Codigo:

```moon
let f = { let x = 10; fn(y: Int) -> Int { x + y } };
f(1)
```

Conceptualmente:
- `Stmt::Let(name=f, expr=Expr::Block(... tail = Expr::Fn(...)))`
- `Expr::Fn` contiene:
  - params: `[y: Int]`
  - ret_ty: `Int`
  - body: `Expr::Block(tail = x + y)`

El runtime capturara `x` dentro de la closure.

## 7) Ejercicios

1) Agrega `Null` al AST (solo AST) y decide si es literal o keyword.
2) Agrega `Expr::While` (solo AST) y discute que spans deberia cubrir.
3) Piensa como representarias function types en `TypeExpr` sin romper el parser (conflicto con `(` en expresiones).

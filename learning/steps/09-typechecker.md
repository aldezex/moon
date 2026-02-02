# 09 - Typechecker estricto (moon check)

## Objetivo

Antes de ejecutar un script queremos validar:
- que las variables existen donde se usan
- que los operadores se aplican a tipos correctos
- que `if` tiene condicion booleana y ramas compatibles
- que las llamadas a funciones tienen aridad y tipos correctos

Esto evita muchos errores de runtime y nos acerca al "feel" de Rust, pero manteniendo la experiencia de scripting.

Crate:
- `compiler/typechecker` (`moon_typechecker`)

Archivos:
- `compiler/typechecker/src/lib.rs`
- `compiler/typechecker/src/env.rs`
- `compiler/typechecker/src/types.rs`
- `compiler/typechecker/src/error.rs`

## Tipos del MVP

Archivo:
- `compiler/typechecker/src/types.rs`

Tipos soportados:
- `Int`
- `Bool`
- `String`
- `Unit`

Nota: en el AST, las anotaciones de tipo son `TypeExpr::Named("Int")`, etc.

## Errores con span

Archivo:
- `compiler/typechecker/src/error.rs`

`TypeError` incluye:
- `message`
- `span`

La CLI renderiza el error con `Source::render_span(span, message)`.

## Entorno de tipos (TypeEnv)

Archivo:
- `compiler/typechecker/src/env.rs`

`TypeEnv` mantiene:
- `globals`: variables definidas en top-level
- `scopes`: stack de scopes para bloques
- `funcs`: firmas de funciones (`FuncSig { params, ret }`)

Regla (importante):
- Typechecking es estricto y **secuencial** para variables: si una variable no fue declarada aun, es error.

## Algoritmo: dos pasadas

Archivo:
- `compiler/typechecker/src/lib.rs`

`check_program(program)` hace:

1) Pass 1: recolectar firmas de funciones
   - Esto permite llamadas a funciones antes de su definicion (estilo Rust items).
   - Tambien habilita recursion (self-call).

2) Pass 2: typecheck de statements en orden
   - `let`: infiere tipo del RHS y valida contra anotacion si existe.
   - `expr;`: valida la expresion y descarta el tipo.
   - `fn`: typecheck del cuerpo con un scope nuevo que contiene los parametros.

3) Tail expression:
   - el tipo del programa es el tipo de `program.tail` si existe, o `Unit` si no.

## Reglas principales (MVP)

- `let x: T = expr;`
  - `expr` debe tener tipo `T`
- `if cond { a } else { b }`
  - `cond` debe ser `Bool`
  - `a` y `b` deben tener el mismo tipo
- Operadores:
  - aritmetica `+ - * / %`: `Int` x `Int` -> `Int`
  - `String + String` -> `String`
  - comparaciones `< <= > >=`: `Int` x `Int` -> `Bool`
  - igualdad `== !=`: requiere tipos iguales -> `Bool`
  - logicos `&& ||`: `Bool` x `Bool` -> `Bool`
- Calls:
  - por ahora solo llamamos funciones por nombre (`f(...)`)
  - aridad y tipos deben matchear exactamente

## Integracion con la CLI

Archivo:
- `src/main.rs`

Comandos:
- `moon check file.moon`: parse + typecheck (no ejecuta)
- `moon run file.moon`: parse + typecheck + ejecuta

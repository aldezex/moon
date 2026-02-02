# 06 - Interpreter (tree-walk)

## Que es un interpreter "tree-walk"

Un tree-walk interpreter ejecuta el AST directamente:
- No genera bytecode.
- No hace optimizaciones grandes.
- Es ideal para un MVP porque permite iterar en sintaxis y semantica rapido.

En Moon, el interpreter vive en un crate separado para no mezclar frontend (core) con semantica.

Archivos:
- `compiler/interpreter/src/lib.rs`
- `compiler/interpreter/src/value.rs`
- `compiler/interpreter/src/env.rs`
- `compiler/interpreter/src/error.rs`
- `compiler/interpreter/src/eval.rs`

## Value: los valores en runtime

Archivo:
- `compiler/interpreter/src/value.rs`

Por ahora:
- `Int(i64)`
- `Bool(bool)`
- `String(String)`
- `Unit` (equivalente a "no valor"; lo usamos como resultado de `let`)

## Env: variables

Archivo:
- `compiler/interpreter/src/env.rs`

`Env` maneja 3 cosas:
- variables globales
- scopes locales (stack) para bloques
- tabla global de funciones

Variables:
- `globals: HashMap<String, Value>`
- `scopes: Vec<HashMap<String, Value>>` (innermost al final)

Funciones:
- `funcs: HashMap<String, Function>`

Reglas:
- `let` define en el scope actual (si estamos dentro de un bloque) o en `globals` (si estamos en top-level).
- lookup de variables busca primero en scopes locales, luego en globales.

Funciones hoy no son valores (todavia):
- Se declaran con `fn` (solo top-level por ahora)
- Se llaman por nombre: `f(1, 2)`

## RuntimeError: errores en ejecucion

Archivo:
- `compiler/interpreter/src/error.rs`

`RuntimeError` incluye:
- `message`
- `span` (importante: seguimos apuntando a la ubicacion exacta del AST)

Ejemplo de error:
- variable no definida
- operador aplicado a tipos incorrectos
- division por 0

## Evaluador (eval)

Archivo:
- `compiler/interpreter/src/eval.rs`

Entrada:
- `Program` de `moon_core::ast`

Salida:
- `Result<Value, RuntimeError>`

### Flujo principal

- `eval_program(program)`:
  1) Hace un pre-pass para registrar todas las funciones (`fn`) antes de ejecutar nada (estilo Rust items).
  2) Ejecuta los statements en orden (lets y expr statements).
  3) Si existe una tail expression (`program.tail`), ese es el valor final.

### Statements

- `let name = expr;`
  - evalua `expr`
  - guarda en `Env`
  - devuelve `Unit`
- `expr;` (expr statement)
  - evalua la expresion y descarta el valor (devuelve `Unit`)
- `fn name(...) -> Type { ... }`
  - se registra en la tabla de funciones (no devuelve valor)

### Expresiones

Soportamos:
- literales / ident
- bloques `{ ... }` con scope y tail expression
- `if ... else ...` como expresion
- llamadas `f(x, y)`
- unarios: `-` para ints, `!` para bools
- binarios:
  - Aritmetica: `+ - * / %` (ints)
  - Concat: `string + string`
  - Comparaciones: `< <= > >=` (ints)
  - Igualdad: `== !=` (cualquier `Value` comparable por igualdad)
  - Logicos: `&& ||` con short-circuit

Short-circuit:
- `a && b`: si `a` es `false`, no evalua `b`
- `a || b`: si `a` es `true`, no evalua `b`

### Scoping (detalle importante)

Los bloques son scopes lexicos:
- `{ let x = 2; x }` devuelve `2` y `x` no existe fuera del bloque.

Las funciones **no capturan** variables locales del caller (no hay closures aun):
- Cuando llamamos una funcion, ejecutamos su cuerpo con:
  - acceso a `globals`
  - un scope local nuevo con sus parametros/variables
  - sin acceso a scopes del caller

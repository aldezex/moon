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

`Env` es un `HashMap<String, Value>`:
- `get(name)` para leer variables
- `set(name, value)` para definir/actualizar

Nota: no tenemos scopes todavia; es un entorno global plano.

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
  - recorre `stmts` en orden
  - guarda el ultimo valor evaluado (como REPL)

### Statements

- `let name = expr;`
  - evalua `expr`
  - guarda en `Env`
  - devuelve `Unit`
- `expr;`
  - devuelve el resultado de evaluar la expresion

### Expresiones

Soportamos:
- literales / ident
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

# 06 - Interpreter (tree-walk) + scoping + heap

## Que es un tree-walk interpreter

Un tree-walk interpreter ejecuta el AST directamente:
- no genera bytecode
- no optimiza demasiado
- es perfecto para iterar rapido en semantica del lenguaje

En Moon:
- `moon run` usa el interpreter (despues de typecheck)
- `moon vm` usa bytecode + VM (tambien despues de typecheck)

Crate:
- `compiler/interpreter` (`moon_interpreter`)

Archivos:
- `compiler/interpreter/src/env.rs`
- `compiler/interpreter/src/eval.rs`
- `compiler/interpreter/src/error.rs`

## Value y Heap viven en moon_runtime

En vez de que cada backend invente su propio `Value`, centralizamos runtime en:
- `compiler/runtime` (`moon_runtime`)

Archivos:
- `compiler/runtime/src/value.rs`
- `compiler/runtime/src/heap.rs`

Esto permite que:
- interpreter y VM compartan el mismo modelo de valores
- el GC sea unico
- el typechecker se alinee con el runtime

## Env: variables, scopes, funciones y heap

Archivo:
- `compiler/interpreter/src/env.rs`

`Env` mantiene:
- `globals: HashMap<String, Value>`
- `scopes: Vec<HashMap<String, Value>>` (stack de scopes; el ultimo es el mas interno)
- `funcs: HashMap<String, Function>` (funciones top-level)
- `heap: Heap` (para arrays/objects)

Reglas:
- `let` define en el scope actual si existe, si no en globals.
- lookup busca primero en scopes (inner -> outer) y luego en globals.
- assignment (`x = ...;`) actualiza el scope mas cercano donde exista `x`.

Funciones (MVP):
- se declaran top-level
- no son valores (todavia)
- no capturan variables locales (todavia)
- soportan `return expr?;` para salir temprano

## eval_program: pipeline del interpreter

Archivo:
- `compiler/interpreter/src/eval.rs`

`eval_program(program)`:

1) Pre-pass de funciones:
   - registra todas las `fn` antes de ejecutar nada
   - permite call-before-definition (estilo Rust items)

2) Ejecuta statements en orden

3) Evalua el `program.tail` si existe
   - ese es el valor final del script

Nota: si un `return` intenta "escapar" al top-level, el interpreter lo convierte en error.

## Statements

1) `let name = expr;`
   - evalua `expr`
   - define `name`
   - resultado del statement: `Unit`

2) `target = expr;`
   - evalua RHS
   - aplica assignment:
     - `x = ...;` -> update variable
     - `arr[i] = ...;` -> muta array en heap
     - `obj["k"] = ...;` -> muta object en heap

3) `return expr?;`
   - `expr` es opcional (`return;` devuelve `Unit`)
   - semantica: solo permitido dentro de funciones (el typechecker lo valida)
   - implementacion: es un "control flow no-local" que se propaga hacia arriba hasta que
     lo consume el call frame de una funcion

4) `expr;`
   - evalua la expresion y descarta su valor

5) `fn ...`
   - se registra; no ejecuta al "pasar"

## Expresiones

Primitivas:
- ints/bools/strings/ident

Blocks:
- `{ ... }` introduce scope nuevo
- ejecuta statements
- devuelve el tail expression si existe, si no `Unit`

If/else:
- `if cond { ... } else { ... }` es una expresion
- `cond` debe ser bool (en runtime, si no, error)

Ops:
- unarios: `-` para Int, `!` para Bool
- binarios:
  - aritmetica para Int
  - concatenacion String + String
  - comparaciones para Int
  - `&&` y `||` con short-circuit

Calls:
- solo por nombre (callee debe ser `Ident`)
- crea un "call frame" logico:
  - guarda scopes del caller
  - crea scope nuevo con params
  - ejecuta body
  - restaura scopes del caller
- si el body devuelve `return`, el call frame lo consume y ese valor pasa a ser el resultado de la llamada

### Control flow no-local: como implementamos `return`

Una leccion clasica en interpretes/VMs: `return` no es un "valor normal".
Es una salida temprana que tiene que cruzar multiples nodos (bloques, ifs, etc.).

En el interpreter MVP, en vez de que `eval_*` devuelva solo `Value`, devolvemos un enum interno:

- `Exec::Value(Value)`
- `Exec::Return(Value, Span)`

Regla:
- cualquier `eval_*` que reciba `Exec::Return` lo re-propaga hacia arriba
- solo el handler de llamadas a funcion lo "consume" (y convierte a `Value`)

Esto evita hacks (como exceptions) y hace muy facil extender despues con `break/continue` para loops.

Arrays/Objects:
- `[1, 2, 3]` -> `Value::Array(handle)` (heap alloc)
- `#{ a: 1 }` -> `Value::Object(handle)` (heap alloc)

Indexing:
- `arr[0]` lee elemento
- `obj["k"]` lee valor

## GC en el interpreter (builtin `gc()`)

El heap vive en `Env.heap`.

En el MVP, el GC se dispara manualmente via builtin:
- `gc()`

Implementacion:
- junta roots (`Env::roots()`) clonando valores de globals + scopes
- llama `heap.collect_garbage(&roots)`

Esto sirve como "herramienta de debug" para validar que el GC no rompe nada.
A futuro, el runtime/VM lo disparan automatico por heuristica (alloc threshold).

## Mini ejercicios

1) Implementa un builtin `heap_stats()` que devuelva `{ live: Int, freed: Int }`.
2) Cambia assignment para que devuelva `Unit` como expresion (en vez de statement).
3) Agrega un `print(x)` builtin (solo para `String` al principio).

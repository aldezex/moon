# 10 - Runtime y memoria (Value + Heap + GC mark/sweep)

Moon tiene heap objects:
- arrays
- objects
- closures (environment)

Este capitulo explica el runtime compartido (`moon_runtime`).

Crate:
- `compiler/runtime` (`moon_runtime`)

Archivos:
- `compiler/runtime/src/value.rs`
- `compiler/runtime/src/heap.rs`

## 0) Por que un runtime compartido

Interpreter y VM deben producir el mismo `Value`.
Si cada backend inventa su runtime:
- divergen facil
- GC se duplica
- tests se vuelven inconsistentes

Por eso:
- `moon_runtime::Value` se comparte
- `moon_runtime::Heap` se comparte

## 1) Value (tagged union)

Archivo:
- `compiler/runtime/src/value.rs`

`Value` (MVP+):
- escalares:
  - `Int(i64)`
  - `Bool(bool)`
  - `String(String)`
  - `Unit`
- referencias a heap:
  - `Array(GcRef)`
  - `Object(GcRef)`
  - `Closure(GcRef)`
- funciones:
  - `Function(String)` (item/builtin por nombre)

Nota:
- `Function(String)` NO es heap (no captura).
- `Closure(GcRef)` SI es heap (captura env).

## 2) Heap: arenas + free list

Archivo:
- `compiler/runtime/src/heap.rs`

Representacion:
- `objects: Vec<Option<HeapObject>>`
- `free_list: Vec<usize>`

`GcRef(usize)` apunta a un indice en `objects`.

Alloc:
- si hay un indice libre, reusa
- si no, push al final

Esto evita usar `Box` por objeto y simplifica GC.

## 3) HeapObjectKind

`HeapObjectKind`:
- `Array(Vec<Value>)`
- `Object(HashMap<String, Value>)`
- `Closure { func_name: String, env: HashMap<String, Value> }`

Los arrays/objects son estructuras dinamicas.
El closure env es el "activation record" heap-alloc de una closure.

## 4) Mark/sweep (GC)

### 4.1 Root set

GC necesita una lista de valores raiz.
En Moon, roots se construyen en los backends:
- interpreter:
  - globals
  - scopes
  - closure activa
- VM:
  - globals
  - scopes de frames
  - operand stack
  - closure activa del frame

### 4.2 Mark phase

`mark_value(v)`:
- si `v` es `Array/Object/Closure`:
  - marca el heap object y recorre sus hijos
- si `v` es escalar o `Function`:
  - no hace nada

`mark_object(handle)`:
- chequea bounds
- evita double-mark
- recorre children:
  - arrays: elementos
  - objects: values
  - closures: values del `env`

### 4.3 Sweep phase

- recorre `objects`
- si `marked == false`:
  - libera slot (`None`)
  - agrega indice a `free_list`
- si `marked == true`:
  - lo desmarca para el proximo ciclo

Tradeoffs:
- mark/sweep es simple pero puede pausar (stop-the-world)
- para MVP esta bien

## 5) Closures: por que el env vive en heap

Una closure debe sobrevivir al scope que la creo.
Ejemplo:

```moon
let c = { let x = 0; fn() -> Int { x = x + 1; x } };
// aca el scope del block termino, pero la closure sigue viva
c()
```

Si `x` viviera solo en stack:
- se perderia al salir del block

Por eso:
- al crear la closure, copiamos los locals visibles a un `env` heap-alloc
- el `Value::Closure` apunta a ese env

## 6) Practica: inspeccion de heap

Hoy el runtime expone:
- `Heap::stats()` (live/freed)

Ejercicio:
1) agrega un builtin `heap_stats()` que devuelva `#{ live: Int, freed: Int }`.
2) agrega auto-GC (heuristica por cantidad de allocs).

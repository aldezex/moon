# 10 - Runtime y memoria (heap + GC mark/sweep)

## Que es el "runtime" en un lenguaje

Cuando hablamos de runtime, hablamos de:
- como representamos valores en ejecucion (`Value`)
- donde viven valores "grandes" (heap)
- como se maneja memoria (GC)
- que operaciones existen (indexing, mutacion, etc.)

En Moon, el runtime es un crate separado para que:
- interpreter y VM compartan `Value`
- el GC sea unico
- el resto del compilador no se acople a un backend

Crate:
- `compiler/runtime` (`moon_runtime`)

Archivos:
- `compiler/runtime/src/value.rs`
- `compiler/runtime/src/heap.rs`

## Value: valores de ejecucion

Archivo:
- `compiler/runtime/src/value.rs`

`Value` incluye:
- `Int(i64)`
- `Bool(bool)`
- `String(String)`
- `Unit`
- `Array(GcRef)`
- `Object(GcRef)`

Idea clave:
- `Array` y `Object` no guardan su contenido inline.
- guardan un handle `GcRef` que apunta al heap.

Esto da:
- identidad (dos variables pueden apuntar al mismo array/object)
- mutabilidad sin copiar estructuras grandes

## Heap: objetos trazables

Archivo:
- `compiler/runtime/src/heap.rs`

### Representacion

El heap es:
- `Vec<Option<HeapObject>>` (slots)
- `free_list: Vec<usize>` (indices libres reutilizables)

`GcRef(usize)` apunta a un slot.

`HeapObject` tiene:
- `marked: bool` (para GC)
- `kind: HeapObjectKind`

`HeapObjectKind` hoy soporta:
- `Array(Vec<Value>)`
- `Object(HashMap<String, Value>)`

### API del heap (MVP)

Alloc:
- `alloc_array(Vec<Value>) -> GcRef`
- `alloc_object(HashMap<String, Value>) -> GcRef`

Acceso/mutacion:
- `array_get(handle, idx) -> Option<&Value>`
- `array_set(handle, idx, value) -> Result<(), String>`
- `object_get(handle, key) -> Option<&Value>`
- `object_set(handle, key, value) -> Result<(), String>`

Nota:
- `array_set` hoy exige `idx < len` (no auto-grow). Es una decision MVP.

## GC mark/sweep (como funciona)

El GC se dispara cuando alguien llama:
- `Heap::collect_garbage(roots)`

Y hace dos fases.

### 1) Mark

Input:
- `roots: &[Value]`

Roots tipicos:
- globals
- scopes actuales
- operand stack (en la VM)

Mark recorre cada root:
- si es primitivo, no hace nada
- si es `Array(GcRef)`/`Object(GcRef)`:
  - marca el objeto en el heap
  - recorre recursivamente sus hijos (`Vec<Value>` o `HashMap<String, Value>`)

### 2) Sweep

Recorre todos los slots:
- si `marked == false`: libera el slot (lo pone en `None`) y lo agrega a `free_list`
- si `marked == true`: lo desmarca para el proximo ciclo

Resultado:
- se devuelve un `HeapStats` con conteos utiles (live/freed).

## Donde aparecen las roots en Moon

### Interpreter

Crate:
- `compiler/interpreter`

El heap vive en `Env.heap`.

El builtin `gc()`:
- junta roots con `Env::roots()` (globals + scopes)
- llama `heap.collect_garbage(&roots)`

Archivo:
- `compiler/interpreter/src/eval.rs`

### VM

Crate:
- `compiler/vm`

El builtin `gc()` existe como funcion builtin del modulo y la VM lo intercepta:
- junta roots:
  - globals
  - scopes de todos los frames
  - operand stack
- corre GC

Archivo:
- `compiler/vm/src/vm.rs`

## Relacion con el lenguaje (sintaxis)

En el lenguaje, arrays/objects se crean con literales:
- arrays: `[1, 2, 3]`
- objects: `#{ a: 1, "b": 2 }`

Indexing:
- `arr[0]`
- `obj["k"]`

Mutacion (assignment statement):
- `arr[0] = 10;`
- `obj["k"] = 10;`

Todo esto se apoya en:
- `Value::Array` / `Value::Object`
- `Heap::{array_get,array_set,object_get,object_set}`

## Limitaciones actuales (a proposito)

El GC y el heap estan listos, pero:
- no hay closures aun (no hay `Value::Closure`)
- no hay objetos con tipos estructurales (solo `Object<T>` homogeneo)
- no hay auto-GC por heuristica (por ahora `gc()` es manual)

Esto es intencional: queremos primero tener el "esqueleto" correcto, y luego sumar features que realmente lo necesiten.

## Mini ejercicios

1) Implementa auto-GC:
   - cada N allocs, disparar GC
   - decide un threshold simple

2) Soporta `arr.push(x)` como builtin:
   - implica agregar parsing de `.` o un builtin `push(arr, x)`

3) Agrega `Value::Null` y define su interaccion con objects/arrays.

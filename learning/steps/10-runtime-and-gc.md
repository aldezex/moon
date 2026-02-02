# 10 - Runtime y memoria (GC) (diseño)

Este capitulo describe el plan para pasar del runtime MVP (solo `Int/Bool/String/Unit`) a un runtime de lenguaje de scripting real con objetos, arrays y closures.

Todavia no esta implementado; es el siguiente bloque grande de trabajo.

## Por que necesitamos heap + GC

En un lenguaje tipo JS/TS queremos:
- arrays y objetos mutables
- closures (funciones que capturan variables)
- grafos de objetos con ciclos

Con valores "por copia" (owned `Vec<Value>` dentro de `Value`) no podemos:
- compartir eficientemente estructuras grandes
- representar ciclos
- tener identidad (dos variables apuntando al mismo objeto) sin duplicar

La solucion tipica: asignar objetos en heap y referenciarlos con handles/pointers:
- `Value::Obj(Handle)`
- `Value::Array(Handle)`
- `Value::Function(Handle)` (closures)

Y limpiar memoria automaticamente via **GC por trazado** (mark/sweep).

## Diseño sugerido (simple y incremental)

### 1) Separar runtime en un crate

Nuevo crate sugerido:
- `compiler/runtime`

Responsabilidades:
- `Value` (incluyendo referencias a heap)
- `Heap` (alloc + GC)
- primitivas builtin (string ops, array ops, etc.)

El interpreter/VM solo "usa" el runtime.

### 2) Representar heap objects

Un MVP de heap se puede modelar asi:

- `Handle(usize)` apunta a un slot en un `Vec<Option<HeapObject>>`
- `HeapObject` tiene:
  - `marked: bool`
  - `kind: ObjectKind`

Donde `ObjectKind` puede ser:
- `Array(Vec<Value>)`
- `Object(HashMap<String, Value>)`
- `Closure { params, body, captured_env }`

### 3) Mark phase (trazado)

Inputs: conjunto de roots (valores vivos).

Roots tipicos:
- variables globales
- scopes locales actuales
- stack frames (si hay llamadas a funciones)

Algoritmo:
- recorrer cada `Value`
  - si es primitivo, no hace nada
  - si es `Handle`, marcar el objeto y recorrer recursivamente sus hijos (sus `Value` internos)

### 4) Sweep phase

Recorrer todos los slots del heap:
- si `marked == false`: liberar (slot = None, agregar a free-list)
- si `marked == true`: desmarcar para la proxima GC

## Interaccion con el lenguaje (cuando exista)

### Mutabilidad

Si agregamos:
- `arr.push(x)`
- `obj.key = v`

Entonces los objetos deben tener identidad y vivir en heap.

### Closures (captura lexica)

Para closures necesitamos capturar un "environment" persistente:
- una estructura tipo `EnvFrame` en heap (o una lista de frames)
- `Value::Closure(handle)` que apunta a `{ func, env }`

La GC debe poder trazar:
- desde una closure hacia su `env`
- desde un `env` hacia sus values

## Integracion con typechecker

Cuando existan objetos/arrays:
- el typechecker debe conocer tipos como:
  - `Array<T>`
  - `Object` (posible: `Record<K,V>` o tipos estructurales)

Esto probablemente requiere extender `TypeExpr` y el parser de tipos.

## Integracion con VM

En una VM, el root set incluye:
- stack de valores (operand stack)
- call frames
- globals

La VM puede disparar GC:
- cuando el heap crece mas de X bytes/objetos
- o cada N allocs

## Siguiente paso practico

Antes de implementar un GC completo, conviene:
1) agregar arrays/objects como valores heap sin mutabilidad (solo literales + lectura)
2) agregar mutabilidad basica
3) recien ahi: GC mark/sweep (cuando aparezcan ciclos/comparticion real)

# 06 - Interpreter (tree-walk) + scoping + closures

El interpreter ejecuta el AST directamente.
Es el backend mas simple y es donde validamos semantica rapidamente.

Crate:
- `compiler/interpreter` (`moon_interpreter`)

Archivos clave:
- `compiler/interpreter/src/env.rs`
- `compiler/interpreter/src/eval.rs`
- `compiler/interpreter/src/error.rs`

## 0) Modelo mental: evaluator por recursion

Cada nodo del AST se evalua con un `match`.
- Expresiones producen un `Value`
- Statements ejecutan efectos y devuelven `Unit` (o control-flow)

En Moon, la semantica importante es:
- scopes lexicos (bloques)
- funciones como valores
- closures (captura)
- `return` como salida temprana

## 1) Runtime compartido: Value + Heap

El interpreter NO define su propio `Value`.
Usa `moon_runtime::Value`:
- `Int`, `Bool`, `String`, `Unit`
- `Array(GcRef)`, `Object(GcRef)`
- `Function(String)`
- `Closure(GcRef)`

Heap + GC tambien vienen de `moon_runtime`:
- `compiler/runtime/src/heap.rs`

Ventaja:
- interpreter y VM comparten semantica de valores/GC.

## 2) Environment (Env) y resolucion de nombres

Archivo:
- `compiler/interpreter/src/env.rs`

`Env` mantiene:
- `globals: HashMap<String, Value>`
- `scopes: Vec<HashMap<String, Value>>` (stack; ultimo = mas interno)
- `funcs: HashMap<String, Function>` (tabla de funciones por nombre)
- `heap: Heap`
- `closure: Option<GcRef>` (environment capturado activo)

Regla de lookup (lexical scoping):
1) scopes locales (inner -> outer)
2) closure env (si existe)
3) globals
4) si no hay var: se intenta resolver a funcion (ver `eval_expr(Ident)`)

Esto evita dynamic scoping:
- una closure NO ve los locals del caller
- solo ve:
  - su propio frame local
  - su closure env
  - globals

## 3) Funciones vs Closures (representacion)

### 3.1 `Value::Function(name)`

Representa un item top-level o builtin.
- no captura locals
- al llamar, el frame se ejecuta con `closure = None`

### 3.2 `Value::Closure(handle)`

Un closure es heap-allocated:
- `HeapObjectKind::Closure { func_name, env }`

El closure env:
- es un `HashMap<String, Value>`
- se crea al evaluar `Expr::Fn`
- captura **locals visibles** (y closure env externo si lo hay)
- NO captura globals (se resuelven en call-time)

Importante:
- el env es mutable: assignments a variables capturadas actualizan el env
- eso permite closures con estado:

```moon
let c = { let x = 0; fn() -> Int { x = x + 1; x } };
c() + c() // 3
```

Semantica de captura (MVP):
- captura por valor (snapshot shallow)
  - Int/Bool/String se copian
  - Array/Object comparten el mismo heap handle

## 4) `return` como control flow no-local

`return` no es un valor normal: corta el flujo.

En vez de exceptions, usamos un enum interno:
- `Exec::Value(Value)`
- `Exec::Return(Value, Span)`

Regla:
- cualquier `eval_*` propaga `Return` hacia arriba
- solo el handler de llamada a funcion/closure lo consume

Eso es el mismo patron que usaras despues para `break/continue`.

## 5) Evaluacion: funciones principales

### 5.1 `eval_program(program)`

Pipeline:
1) pre-pass: registra todas las `Stmt::Fn` en `env.funcs`
   - permite call-before-definition
2) ejecuta `program.stmts`
3) evalua `program.tail` si existe

Guardrail:
- si un `return` llega al top-level, es error.

### 5.2 Statements

- `Stmt::Let`:
  - evalua RHS
  - define var en scope actual o global

- `Stmt::Assign`:
  - `x = expr;` asigna variable
  - `target[index] = expr;` muta heap

Nota de semantica:
- en `a[i] = rhs`, evaluamos `a` y `i` antes de `rhs`.

- `Stmt::Return`:
  - produce `Exec::Return(value)`

- `Stmt::Fn`:
  - no ejecuta (ya fue registrada)

- `Stmt::Expr`:
  - evalua expresion y descarta

### 5.3 Expressions

- Literales: obvio
- `Ident`:
  - busca var
  - si no existe, busca funcion (y produce `Value::Function(name)`)

- `Expr::Fn`:
  - genera un nombre unico `<lambda#N>`
  - registra un `Function { params, body }` en `env.funcs`
  - captura locals visibles y crea `Value::Closure(handle)`

- `Call`:
  - evalua callee (expr)
  - evalua args
  - si callee es:
    - `Function(name)` -> call con `closure=None`
    - `Closure(h)` -> call con `closure=Some(h)`

- `Block`:
  - `env.push_scope()`
  - ejecuta statements
  - evalua tail
  - `env.pop_scope()`

- `If`:
  - evalua cond
  - branch

- Arrays/Objects:
  - allocan en heap

## 6) GC y roots

El builtin `gc()`:
- construye roots via `Env::roots()`:
  - globals
  - scopes
  - closure activa (si existe)
- llama `heap.collect_garbage(&roots)`

Importante:
- incluir closure activa evita que el GC coleccione el environment mientras se ejecuta.

## 7) Practica: debug de closures

Lee estas partes:
- `compiler/interpreter/src/eval.rs`:
  - `Expr::Fn` (creacion)
  - `Expr::Call` (invocacion)
- `compiler/interpreter/src/env.rs`:
  - `get_var` y `assign_var` (lookup/assign con closure env)

Ejercicio:
- cambia la estrategia de captura:
  - captura tambien globals (y observa como cambia el comportamiento)

## 8) Ejercicios

1) Implementa `dbg(x)` como builtin que imprima y devuelva `x`.
2) Implementa `len(x)` para `Array`/`String`/`Object`.
3) Agrega un modo tracing (log de cada `Expr::Call`) con spans.

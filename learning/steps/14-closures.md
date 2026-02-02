# 14 - Closures y funciones anonimas (diseno + implementacion)

Este capitulo es un deep dive.
Una closure es el punto donde:
- parser
- typechecker
- runtime
- VM

se encuentran.

Moon soporta:
- funciones como valores
- funciones anonimas como expresion (`Expr::Fn`)
- closures con captura lexical

## 0) Teoria: que es una closure

En terminos clasicos:
- una funcion es "codigo" (un cuerpo + params)
- una closure es:
  - codigo
  - + environment (los bindings libres que el codigo usa)

Formalmente:
- si en el cuerpo aparece un identificador que no es param ni local del cuerpo, es un free variable.
- el environment de la closure debe proveer un valor para ese binding.

Ejemplo:

```moon
{ let x = 10; fn(y: Int) -> Int { x + y } }
```

En el cuerpo `x + y`:
- `y` es param
- `x` es free variable

La closure debe capturar `x`.

## 1) Lexical scoping vs dynamic scoping

Lexical scoping (lo que queremos):
- el significado de un nombre depende de la estructura del programa
- una closure ve el scope donde fue definida, no el donde fue llamada

Dynamic scoping (lo que NO queremos):
- una funcion ve el scope del caller

Test clasico:

```moon
let f = { let x = 10; fn(y: Int) -> Int { x + y } };
{ let x = 100; f(1) }
```

Con lexical scoping, el resultado es 11.

## 2) Estrategias de implementacion (tradeoffs)

Hay varias estrategias reales:

### 2.1 Lambda lifting

Transformas funciones anidadas en funciones top-level y pasas capturas como parametros extra.
Pros:
- simple para VM
Cons:
- cambia aridad
- complica debugging

### 2.2 Environment objects (lo que hacemos)

Representas closure como:
- `function pointer` (o id)
- `environment object` heap-alloc

Pros:
- aridad estable
- lexical scoping natural
Cons:
- overhead de alloc y lookup

### 2.3 Upvalues por referencia

Capturas por referencia con "cells" compartidos.
Pros:
- replica JS/TS (mutaciones del outer scope se ven)
Cons:
- mas complejo (lifting de storage a heap)

Moon (MVP) implementa 2.2 con captura por valor (snapshot).

## 3) Semantica de captura en Moon (MVP)

Definicion operativa:
- una closure captura los **locals visibles** en el punto de creacion:
  - scopes locales actuales
  - y el closure env activo (si la closure se crea dentro de otra closure)
- globals NO se capturan
- captura por valor (snapshot shallow)
- el env capturado es mutable y persiste entre llamadas

Consecuencia:
- puedes construir closures con estado (contador)
- pero cambios a un local externo despues de crear la closure no se reflejan (porque snapshot)

## 4) AST + parser

### 4.1 AST
Archivo:
- `compiler/core/src/ast/mod.rs`

Nodo nuevo:
- `Expr::Fn { params, ret_ty, body, span }`

### 4.2 Parser
Archivo:
- `compiler/core/src/parser/mod.rs`

Disambiguacion:
- `fn <Ident>` => `Stmt::Fn` (solo top-level)
- `fn (` => `Expr::Fn` (en expresiones)

Implementacion:
- lookahead en `parse_sequence`
- `parse_fn_expr` parsea:
  - params con tipos
  - return type
  - body como `Block`

## 5) Typechecker

Archivo:
- `compiler/typechecker/src/lib.rs`

Reglas:
- `Expr::Fn` produce `Type::Function { params, ret }`
- el body se typecheckea con:
  - un scope nuevo para params
  - el environment externo intacto (captura lexical)

Importante:
- `Stmt::Fn` NO captura: el typechecker borra scopes antes de chequear el body
- `Expr::Fn` SI captura: el typechecker mantiene scopes externos

`return`:
- se modela con `Type::Never`
- dentro de closures funciona igual que en funciones

## 6) Runtime (Value/Heap/GC)

Archivo:
- `compiler/runtime/src/value.rs`
- `compiler/runtime/src/heap.rs`

Representacion:
- `Value::Closure(GcRef)`
- heap object:
  - `Closure { func_name: String, env: HashMap<String, Value> }`

GC:
- `mark_value` marca closures
- `mark_object` recorre `env.values()`

Root sets:
- interpreter y VM incluyen closures activas en roots

## 7) Interpreter: creacion y llamada

Archivo:
- `compiler/interpreter/src/eval.rs`
- `compiler/interpreter/src/env.rs`

Creacion (`Expr::Fn`):
- genera nombre unico `<lambda#N>`
- registra `Function { params, body }` en `env.funcs`
- captura locals visibles:
  - `Env::capture_visible_locals()`
- `heap.alloc_closure(func_name, captured_env)`
- devuelve `Value::Closure(handle)`

Llamada (`Expr::Call`):
- evalua callee
- si `Function(name)`:
  - call con `closure=None`
- si `Closure(h)`:
  - call con `closure=Some(h)`

Lookup/assign:
- `Env::get_var` y `Env::assign_var` consultan:
  - scopes locales
  - closure env (heap)
  - globals

Esto fija lexical scoping.

## 8) Bytecode + VM

### 8.1 Bytecode instructions
Archivo:
- `compiler/bytecode/src/instr.rs`

Nuevas instrucciones:
- `MakeClosure(func_name, captures)`
- `CallValue(argc)` (ya existia para funciones como valores; ahora soporta closures)

### 8.2 Compiler
Archivo:
- `compiler/bytecode/src/compiler.rs`

Estrategia:
- `Expr::Fn` se compila a:
  - crear una nueva `Function` en el `Module` con nombre `<lambda#N>`
  - emitir `MakeClosure(<lambda#N>, captures)`

`captures`:
- se calculan via `FunctionCtx.visible_names()`
- `FunctionCtx` trackea:
  - locals por scope
  - closure env names (para nested closures)

### 8.3 VM
Archivo:
- `compiler/vm/src/vm.rs`

`Frame` tiene:
- `closure: Option<GcRef>`

`MakeClosure`:
- copia valores de locals/closure env del frame
- alloc closure en heap
- push `Value::Closure`

`CallValue`:
- si callee es closure:
  - resuelve `func_name` desde heap
  - push frame con `closure=Some(handle)`

## 9) Tests (contratos de semantica)

- `compiler/interpreter/tests/mvp.rs`
- `compiler/vm/tests/vm.rs`
- `compiler/typechecker/tests/typechecker.rs`

Casos clave:
- funciones anonimas
- captura lexical (no dynamic)
- mutacion de estado capturado

## 10) Limitaciones actuales (y por que)

- captura por valor (no por referencia)
- sin sintaxis para function types
- sin recursion en anon functions (no letrec)

Estas son tradeoffs intencionales para mantener MVP pequeno.

## 11) Ejercicios (siguiente nivel)

1) Implementa upvalues por referencia (cells) y agrega tests que prueben que cambios externos se reflejan.
2) Agrega function types a `TypeExpr` y habilita funciones que retornan closures.
3) Optimiza `captures` para incluir solo free variables (analisis estatico).

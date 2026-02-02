# 11 - Bytecode + VM (AST -> bytecode -> VM)

El interpreter es ideal para iterar, pero no escala.
La VM ejecuta una representacion compacta: bytecode.

En Moon:
- `moon run` => interpreter
- `moon vm` => bytecode compiler + VM

Este capitulo describe el bytecode real del repo y como soportamos closures.

## 0) Componentes

### Bytecode
Crate:
- `compiler/bytecode` (`moon_bytecode`)

Archivos:
- `compiler/bytecode/src/instr.rs`
- `compiler/bytecode/src/module.rs`
- `compiler/bytecode/src/compiler.rs`

### VM
Crate:
- `compiler/vm` (`moon_vm`)

Archivos:
- `compiler/vm/src/vm.rs`
- `compiler/vm/src/error.rs`

### Runtime compartido
- `compiler/runtime` (`moon_runtime`)

## 1) Modelo de VM

VM stack-based:
- operand stack: `Vec<Value>`
- scopes por frame: `Vec<HashMap<String, Value>>`
- frames: `Vec<Frame>`

`Frame` (MVP+):
- `func: FuncId`
- `ip: usize`
- `stack_base: usize` (para truncar stack al retornar)
- `scopes: Vec<HashMap<String, Value>>`
- `closure: Option<GcRef>` (environment capturado activo)

Lookup de variables (lexical scoping):
1) scopes locales (inner -> outer)
2) closure env del frame (si existe)
3) globals

Si no existe variable, `LoadVar` cae a funcion top-level:
- si `module.by_name` contiene el nombre, empuja `Value::Function(name)`.

## 2) IR: Instr y spans

En Moon, cada instruccion incluye debug info:
- `Instr { kind: InstrKind, span: Span }`

Eso permite:
- `moon disasm` mostrar de que parte del source viene cada instruccion
- la VM adjunta `span` en `VmError`

Archivo:
- `compiler/bytecode/src/instr.rs`

## 3) Instrucciones (InstrKind)

Categorias:

### 3.1 Stack
- `Push(Value)`
- `Pop`

### 3.2 Scopes
- `PushScope`
- `PopScope`

### 3.3 Variables
- `LoadVar(name)`
- `DefineVar(name)`
- `SetVar(name)`

### 3.4 Ops
- `Neg`, `Not`
- `Add/Sub/Mul/Div/Mod`
- `Eq/Ne/Lt/Le/Gt/Ge`

### 3.5 Control flow
- `Jump(ip)`
- `JumpIfFalse(ip)` / `JumpIfTrue(ip)`
- `Return`

### 3.6 Calls
- `Call(FuncId, argc)` (directo, casi legacy)
- `CallValue(argc)` (indirecto; callee viene en stack)

### 3.7 Closures
- `MakeClosure(func_name, captures)`

`MakeClosure`:
- crea `Value::Closure(handle)`
- captura locals visibles (por nombre)

### 3.8 Heap
- `MakeArray(n)`
- `MakeObject(keys)`
- `IndexGet` / `IndexSet`

## 4) Module

Archivo:
- `compiler/bytecode/src/module.rs`

`Module`:
- `functions: Vec<Function>`
- `by_name: HashMap<String, FuncId>`
- `main: FuncId`

`Function`:
- `name: String`
- `params: Vec<String>`
- `code: Vec<Instr>`

Nota:
- las funciones anonimas (`Expr::Fn`) se compilan como funciones con nombres sinteticos `<lambda#N>`.

## 5) Compiler: lowering AST -> bytecode

Archivo:
- `compiler/bytecode/src/compiler.rs`

Estrategia:
- un `Compiler` mantiene:
  - `functions` y `by_name`
  - un contador para `<lambda#N>`
- para cada contexto de funcion, usamos `FunctionCtx`:
  - `scopes` (nombres de locals por scope)
  - `closure_env` (nombres accesibles via env capturado)

### 5.1 Por que trackear scopes en el compilador

Para closures necesitamos saber "que nombres estan visibles ahora".

Al ver `Expr::Fn`:
- calculamos `captures = ctx.visible_names()`
- generamos un `Function` nuevo para el body
- emitimos `MakeClosure(name, captures)`

Semantica:
- captura por valor (snapshot) de locals
- globals no se capturan

### 5.2 Blocks y scopes

Cuando compilamos `Expr::Block`:
- emitimos `PushScope`
- `ctx.push_scope()`
- compilamos statements
- compilamos tail
- `ctx.pop_scope()`
- emitimos `PopScope`

Esto alinea:
- scoping del AST
- scoping de la VM
- scoping del tracker de captura

## 6) VM: ejecucion de closures

### 6.1 `MakeClosure`

La VM:
- para cada nombre en `captures`:
  - busca en locals o closure env del frame (NO globals)
  - si existe, lo copia al env capturado
- alloc en heap:
  - `heap.alloc_closure(func_name, env)`
- empuja `Value::Closure(handle)`

### 6.2 `CallValue`

`CallValue(argc)`:
- pop args
- pop callee
- si callee es:
  - `Value::Function(name)` => call con `closure=None`
  - `Value::Closure(h)` => call con `closure=Some(h)` y `func_name` sacado del heap

Eso fija lexical scoping:
- una closure ve su env capturado, no el caller.

## 7) Practica: mira el bytecode

Ejemplo:

```moon
let c = { let x = 0; fn() -> Int { x = x + 1; x } };
c() + c()
```

Usa:
- `cargo run -- disasm <file>`

Busca:
- `MakeClosure <lambda#...>`
- `CallValue argc=0`
- `SetVar x` que actualiza el env capturado

## 8) Ejercicios

1) Implementa slots de variables (`LoadLocal(slot)`), elimina HashMaps.
2) Implementa upvalues por referencia (captura por referencia, no snapshot).
3) Agrega debug stepping (ejecutar una instruccion por vez) usando spans.

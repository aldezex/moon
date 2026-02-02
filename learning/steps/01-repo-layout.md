# 01 - Layout del repo y crates (arquitectura)

Este capitulo explica la arquitectura del workspace.
La idea es que puedas predecir "donde vive" una responsabilidad y evitar dependencias circulares.

## 0) Workspace y binario

- `Cargo.toml` (raiz)
  - define el workspace
  - define el binario `moon` (CLI)
- `src/main.rs`
  - implementa el CLI

El CLI es el "orquestador" del pipeline:
- lee archivo
- corre lexer/parser
- typecheck
- elige backend (interpreter o VM)

## 1) Crates (carpeta `compiler/`)

### 1.1 `compiler/core` (`moon_core`)
Frontend del lenguaje:
- `Span` / `Source` (diagnosticos)
- AST
- lexer
- parser

Regla: `moon_core` NO ejecuta.

### 1.2 `compiler/typechecker` (`moon_typechecker`)
Semantica estatica:
- tipos (`Type`)
- environment de tipos (`TypeEnv`)
- algoritmo de chequeo

Regla: produce errores con `Span`.

### 1.3 `compiler/runtime` (`moon_runtime`)
Runtime compartido:
- `Value`
- `Heap`
- GC mark/sweep

Regla: NO depende de `moon_core`.
Esto permite que interpreter y VM compartan runtime.

### 1.4 `compiler/interpreter` (`moon_interpreter`)
Tree-walk interpreter:
- ejecuta AST directamente
- ideal para iterar semantica

Depende de:
- `moon_core` (AST)
- `moon_runtime` (Value/Heap)

### 1.5 `compiler/bytecode` (`moon_bytecode`)
Compilador AST -> bytecode:
- `InstrKind` (IR)
- `Instr` (IR + `Span`)
- `Module` / `Function`
- lowering desde AST

Depende de:
- `moon_core`
- `moon_runtime` (para `Value` constantes)

### 1.6 `compiler/vm` (`moon_vm`)
VM stack-based:
- ejecuta `Module` + `Instr`
- maneja frames, scopes, operand stack

Depende de:
- `moon_bytecode`
- `moon_runtime`

### 1.7 `compiler/lsp` (`moon_lsp`)
Language Server Protocol:
- diagnostics (lexer/parser/typechecker)
- hover/definition/completion basico

Depende de:
- `moon_core`
- `moon_typechecker`

## 2) Dependencias (grafo mental)

El objetivo del grafo: evitar ciclos.

```
moon_core  <--- moon_typechecker
   ^               ^
   |               |
   |               +--- moon_lsp
   |
moon_runtime <--- moon_interpreter
      ^
      +--- moon_bytecode ---+ 
                            |
                         moon_vm

moon (CLI) depende de todos para orquestar.
```

Invariantes de esta arquitectura:
- El runtime no conoce el AST.
- La VM no conoce el parser.
- El typechecker no conoce el heap.

Eso mantiene separaciones limpias.

## 3) Donde toco cuando agrego un feature

Regla de oro: feature de lenguaje toca capas.

Ejemplo: closures (`fn(...) -> ... { ... }` capturando variables)
- AST/parser: nuevo `Expr::Fn`
- typechecker: typing de `Expr::Fn` y reglas de `Call`
- interpreter: representacion de closures y environment
- bytecode: `MakeClosure` + generar funciones anonimas en el `Module`
- VM: frames con `closure` y lookup/set de variables
- runtime: `Value::Closure` y heap object `Closure`
- tests: interpreter + VM

## 4) Comandos

- interpreter:
  - `cargo run -- run examples/hello.moon`
- typecheck:
  - `cargo run -- check examples/hello.moon`
- VM:
  - `cargo run -- vm examples/hello.moon`
- AST dump:
  - `cargo run -- ast examples/hello.moon`
- disasm:
  - `cargo run -- disasm examples/hello.moon`
- LSP:
  - `cargo run -p moon_lsp --bin moon-lsp`
- tests:
  - `cargo test --workspace`

## 5) Navegacion rapida (diagnostico)

Cuando algo falla:
- "no tokeniza": `compiler/core/src/lexer/*`
- "no parsea": `compiler/core/src/parser/*`
- "deberia ser error de tipos": `compiler/typechecker/src/lib.rs`
- "typecheck ok, runtime mal": `compiler/interpreter/src/eval.rs` o `compiler/vm/src/vm.rs`
- "crashea con arrays/objects/closures": `compiler/runtime/src/heap.rs`

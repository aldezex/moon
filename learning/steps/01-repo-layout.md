# 01 - Layout del repo y crates

## Objetivo

Queremos que el repo sea facil de navegar y que cada crate tenga una responsabilidad clara.

En un compilador/lenguaje "real", mezclar todo en un solo crate suele terminar en:
- dependencias circulares
- APIs acopladas (cada modulo toca todo)
- refactors dolorosos cuando aparece un backend nuevo (VM/JIT), un typechecker, un LSP, etc.

Por eso, desde temprano, separamos por capas.

## Estructura actual

- `Cargo.toml` (raiz)
  - Define el **workspace** y el binario `moon` (CLI).
- `src/main.rs`
  - CLI: `moon run`, `moon ast`, `moon check`, `moon vm`, `moon disasm`.

Crates (carpeta `compiler/`):

- `compiler/core`
  - `moon_core`: frontend (AST, lexer, parser, spans/diagnosticos).
- `compiler/runtime`
  - `moon_runtime`: runtime compartido (Value + Heap + GC mark/sweep).
- `compiler/interpreter`
  - `moon_interpreter`: tree-walk interpreter (ejecuta AST directamente).
- `compiler/typechecker`
  - `moon_typechecker`: typechecker estricto (sin ejecutar).
- `compiler/bytecode`
  - `moon_bytecode`: compilador AST -> bytecode (Module + Instr).
- `compiler/vm`
  - `moon_vm`: VM que ejecuta bytecode.
- `compiler/lsp`
  - `moon_lsp`: language server (LSP) para editor (diagnosticos/hover/definition).

Extras:
- `examples/`: scripts `.moon` para probar.
- `learning/`: esta guia.

## Por que un workspace

Rust workspaces nos dejan:
- Separar crates sin perder un solo `cargo test --workspace`.
- Evitar dependencias circulares.
- Mantener una direccion de dependencias clara (quien importa a quien).

Diagrama (dependencias):

```
moon_core  <--- moon_typechecker
   ^               ^
   |               |
   |          (CLI usa)
   |               |
moon_runtime <--- moon_interpreter
      ^             ^
      |             |
      +--- moon_bytecode ---+ 
                            |
                         moon_vm
                            ^
                            |
                           moon (CLI)

moon_core <--- moon_typechecker <--- moon_lsp
```

Archivo clave:
- `Cargo.toml` (raiz)

## Como ejecutar

- Ejecutar un archivo (interprete):
  - `cargo run -- run examples/hello.moon`
- Ejecutar un archivo (VM):
  - `cargo run -- vm examples/hello.moon`
- Ver el AST:
  - `cargo run -- ast examples/hello.moon`
- Solo typecheck:
  - `cargo run -- check examples/hello.moon`
- Disassembler (bytecode):
  - `cargo run -- disasm examples/hello.moon`
- Tests:
  - `cargo test --workspace`
- Language server (LSP):
  - `cargo run -p moon_lsp --bin moon-lsp`

## Regla mental para navegar el codigo

Cuando algo "no funciona", pregunta primero en que capa estas:

- "No parsea": `moon_core` (lexer/parser)
- "Parsea pero ejecuta mal": `moon_interpreter` o `moon_vm`
- "La sintaxis es valida pero deberia ser error de tipos": `moon_typechecker`
- "Problema con arrays/objects/GC": `moon_runtime`

Ese mapa te evita perderte.

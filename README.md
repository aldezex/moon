# Moon

Moon es un lenguaje de scripting (ejecucion directa tipo JavaScript/TypeScript) con una "vibra" Rust:
tipos, expresiones, bloques, y una base pensada para escalar hacia un compilador/VM mas seria.

Este repo arranca con un MVP pequeno:
- Lexer -> Parser (AST) -> Typechecker -> Evaluador (interprete) -> CLI
- Spans/diagnosticos basicos para ubicar errores en el source

Codigo:
- `compiler/core`: frontend (AST/lexer/parser/spans/diagnosticos)
- `compiler/runtime`: runtime (Value + Heap + GC mark/sweep)
- `compiler/interpreter`: interprete (tree-walk sobre AST)
- `compiler/typechecker`: typechecker estricto (`moon check`)
- `compiler/bytecode`: compilador AST -> bytecode
- `compiler/vm`: VM (bytecode interpreter)
- `compiler/lsp`: language server (LSP) para diagnosticos/hover/definition en el editor
- `src/main.rs`: CLI (`moon run`, `moon ast`, `moon check`, `moon vm`)

## Desarrollo

Requisitos: Rust (cargo).

Guia paso a paso:
- `learning/README.md`

Comandos:
- `cargo run -- run examples/hello.moon`
- `cargo run -- check examples/hello.moon`
- `cargo run -- vm examples/hello.moon`
- `cargo run -p moon_lsp --bin moon-lsp` (language server via stdio)
- `cargo test --workspace`

## Roadmap (alto nivel)

1) Sintaxis + parser con buena recuperacion de errores
2) Interprete (tree-walk) para iterar rapido en el lenguaje
3) Typechecking estricto (sin `any` implicito) + inferencia basica/local (implementado MVP)
4) Runtime (heap + GC) + arrays/objects (implementado MVP)
5) Bytecode + VM para performance y tooling (implementado MVP)
6) Stdlib + FFI / embedding

## Memoria (decision)

- Runtime: heap con GC por trazado (mark/sweep) para objetos/arrays/closures (evita leaks por ciclos y suele rendir mejor que RC en lenguajes dinamicos).

# Moon

Moon es un lenguaje de scripting (ejecucion directa tipo JavaScript/TypeScript) con una "vibra" Rust:
tipos, expresiones, bloques, y una base pensada para escalar hacia un compilador/VM mas seria.

Este repo arranca con un MVP pequeno:
- Lexer -> Parser (AST) -> Evaluador (interprete) -> CLI
- Spans/diagnosticos basicos para ubicar errores en el source

## Desarrollo

Requisitos: Rust (cargo).

Comandos:
- `cargo run -- run examples/hello.moon`
- `cargo test --workspace`

## Roadmap (alto nivel)

1) Sintaxis + parser con buena recuperacion de errores
2) Interprete (tree-walk) para iterar rapido en el lenguaje
3) Typechecking (gradual / estilo TS) + inferencia basica
4) Bytecode + VM para performance y tooling (REPL rapido, cache, etc.)
5) Stdlib + FFI / embedding

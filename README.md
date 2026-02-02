# Moon

Moon es un lenguaje de scripting (ejecucion directa tipo JavaScript/TypeScript) con una "vibra" Rust:
tipos, expresiones, bloques, y una base pensada para escalar hacia un compilador/VM mas seria.

Este repo arranca con un MVP pequeno:
- Lexer -> Parser (AST) -> Typechecker -> Evaluador (interprete) -> CLI
- Spans/diagnosticos basicos para ubicar errores en el source

Codigo:
- `compiler/core`: frontend (AST/lexer/parser/spans/diagnosticos)
- `compiler/interpreter`: interprete (runtime MVP: values/env/eval)
- `compiler/typechecker`: typechecker estricto (`moon check`)
- `src/main.rs`: CLI (`moon run`, `moon ast`, `moon check`)

## Desarrollo

Requisitos: Rust (cargo).

Guia paso a paso:
- `learning/README.md`

Comandos:
- `cargo run -- run examples/hello.moon`
- `cargo run -- check examples/hello.moon`
- `cargo test --workspace`

## Roadmap (alto nivel)

1) Sintaxis + parser con buena recuperacion de errores
2) Interprete (tree-walk) para iterar rapido en el lenguaje
3) Typechecking estricto (sin `any` implicito) + inferencia basica/local (implementado MVP)
4) Bytecode + VM para performance y tooling (REPL rapido, cache, etc.)
5) Stdlib + FFI / embedding

## Memoria (decision)

- Runtime: heap con GC por trazado (mark/sweep) para objetos/arrays/closures (evita leaks por ciclos y suele rendir mejor que RC en lenguajes dinamicos).

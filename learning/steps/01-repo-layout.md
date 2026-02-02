# 01 - Layout del repo y crates

## Objetivo

Queremos que el repo sea facil de navegar y que cada crate tenga una responsabilidad clara.

## Estructura actual

- `Cargo.toml` (raiz)
  - Define el **workspace** y el binario `moon` (CLI).
- `src/main.rs`
  - CLI: `moon run`, `moon ast`, `moon check`.
- `compiler/core`
  - `moon_core`: frontend (AST, lexer, parser, spans/diagnosticos).
- `compiler/interpreter`
  - `moon_interpreter`: interprete (runtime MVP + evaluador).
- `compiler/typechecker`
  - `moon_typechecker`: typechecker estricto (sin ejecutar).
- `examples/`
  - Scripts `.moon` para probar.

## Por que un workspace

Rust workspaces nos dejan:
- Separar crates sin perder un solo `cargo test --workspace`.
- Evitar dependencias circulares.
- Mantener bien definida la direccion de dependencias:
  - `moon_interpreter` depende de `moon_core` (necesita AST).
  - `moon_typechecker` depende de `moon_core` (necesita AST).
  - `moon` (CLI) depende de los tres.

Archivo clave:
- `Cargo.toml` (raiz)

## Como ejecutar

- Ejecutar un archivo:
  - `cargo run -- run examples/hello.moon`
- Ver el AST:
  - `cargo run -- ast examples/hello.moon`
- Tests:
  - `cargo test --workspace`

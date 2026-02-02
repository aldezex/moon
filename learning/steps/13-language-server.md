# 13 - Language Server (LSP)

Un language server permite que el editor entienda tu lenguaje:
- diagnostics
- hover
- go-to-definition
- completion

Moon tiene un LSP MVP:
- simple, pero end-to-end
- basado en `tower-lsp`

Crate:
- `compiler/lsp` (`moon_lsp`)

Archivo:
- `compiler/lsp/src/main.rs`

## 0) Arquitectura

El servidor mantiene un cache de documentos:
- `documents: RwLock<HashMap<Url, Document>>`

`Document`:
- `text`
- `version`

Cada evento de LSP actualiza el cache y re-publica diagnostics.

## 1) Diagnostics

Pipeline:
1) lexer
2) parser
3) typechecker

Si falla en algun punto:
- construye `Diagnostic` con:
  - `range` (UTF-16)
  - `severity`
  - `message`

Conversion clave:
- Moon usa `Span` en bytes.
- LSP usa `Position` (line, col en UTF-16 code units).

Helpers:
- `position_from_offset_utf16(text, offset)`
- `offset_from_position_utf16(text, position)`
- `range_from_span_utf16(text, span)`

Hay tests unitarios para asegurar consistencia con surrogate pairs.

## 2) Hover (type-at-span)

El typechecker expone:
- `check_program_with_spans(program) -> CheckInfo`

`CheckInfo` incluye:
- `expr_types: Vec<(Span, Type)>`

Estrategia MVP:
- dado un offset del cursor:
  - busca el span mas pequeno que contiene ese offset
  - muestra `Type` en hover

Esto es una version minimal de "type-of-expression".

## 3) Go-to-definition

Estrategia MVP:
- parsea el programa
- recolecta definiciones top-level:
  - `let` y `fn`
- si el cursor esta sobre un ident:
  - si coincide con una definicion top-level, devuelve su range

Limitaciones:
- no resuelve locals
- no resuelve shadowing
- no resuelve fields

Es suficiente para validar el pipeline.

## 4) Completion

MVP:
- completions estaticas (keywords/builtins)

A futuro:
- completions context-sensitive:
  - locals
  - members
  - function params

## 5) Practica

Corre el LSP:
- `cargo run -p moon_lsp --bin moon-lsp`

Conecta desde tu editor (ej: VSCode extension custom) via stdio.

Pruebas:
- escribe `fn` / `return` / `if`
- introduce un error de tipos y mira diagnostics

## 6) Ejercicios

1) Agrega soporte de definition para locals (requiere resolver scopes).
2) Agrega semantic tokens (highlights) usando spans del lexer.
3) Agrega hover para `Value::Function` mostrando signature (params/ret).

# 07 - CLI (moon run / moon ast)

## Objetivo

Necesitamos una forma simple de:
- ejecutar un script `.moon`
- inspeccionar el AST para debug

Archivo:
- `src/main.rs`

## Comandos actuales

### `moon run <file>`

Pipeline:
1) Cargar el source:
   - si `<file>` es `-`, lee de stdin
   - si no, lee del filesystem
2) Lexer:
   - `moon_core::lexer::lex(&source.text)`
3) Parser:
   - `moon_core::parser::parse(tokens)`
4) Interpreter:
   - `moon_interpreter::eval_program(&program)`
5) Output:
   - si el valor final no es `Unit`, se imprime

Errores:
- lex/parse/runtime se imprimen usando `source.render_span(span, message)`

### `moon ast <file>`

Hace el mismo pipeline hasta parsear y luego imprime:
- `println!("{program:#?}")`

Esto es util para validar que el parser arma el AST que esperamos.

## Por que la CLI no vive dentro de compiler/

Porque `moon` es "la herramienta" (entrypoint), y `compiler/*` son librerias reutilizables:
- tests
- embedding (a futuro)
- herramientas (formatter, lsp, etc.)

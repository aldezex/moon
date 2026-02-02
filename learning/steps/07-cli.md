# 07 - CLI (moon run / moon ast / moon check / moon vm)

## Objetivo

La CLI es el "entrypoint" del repo:
- lee un archivo (o stdin)
- corre el pipeline (lex/parse/typecheck/ejecucion)
- imprime errores con contexto

Archivo:
- `src/main.rs`

## Convencion clave: ';' y tail expression

Regla estilo Rust:
- `expr;` descarta el valor (statement)
- la ultima expresion **sin** `;` es el valor del programa/bloque

Ejemplo:
- `let x = 1; x` imprime `1`
- `let x = 1; x;` no imprime nada (resultado `Unit`)

## Comandos

### `moon ast <file>`

Pipeline:
- load -> lex -> parse -> print AST (`{program:#?}`)

Sirve para:
- debug del parser
- entender el shape del AST

### `moon check <file>`

Pipeline:
- load -> lex -> parse -> typecheck

Si pasa:
- imprime `ok: <Type>`

Si falla:
- imprime `type error: ...` con span

### `moon run <file>` (interprete)

Pipeline:
- load -> lex -> parse -> typecheck -> interpreter

Si el valor final no es `Unit`, se imprime.

### `moon vm <file>` (bytecode + VM)

Pipeline:
- load -> lex -> parse -> typecheck -> compile(bytecode) -> VM

Nota:
- En esta etapa el "compile error" es raro (la mayoria de cosas ya deberian estar validadas por typecheck/parser).

## Input: file vs stdin

Convencion:
- si `<file>` es `-`, se lee de stdin
- si no, se lee del filesystem

Esto permite:
- `cat examples/hello.moon | cargo run -- run -`

## Errores (como se imprimen)

En la CLI convertimos errores a diagnosticos humanos via:
- `Source::render_span(span, message)`

Tipos de error:
- lex error: `LexError` (span del token/char)
- parse error: `ParseError` (span del token/expr)
- type error: `TypeError` (span del nodo)
- runtime error (interpreter): `RuntimeError` (span del nodo)
- compile error (bytecode): `CompileError` (span del nodo)
- vm error: (por ahora solo message; a futuro tambien span)

## Mini ejercicios

1) Agrega `moon tokens <file>` que imprima la lista de tokens (kind + span).
2) Agrega `moon fmt <file>` (aunque sea un stub) para reservar el comando.
3) En `moon vm`, agrega un flag para imprimir el bytecode antes de ejecutar.

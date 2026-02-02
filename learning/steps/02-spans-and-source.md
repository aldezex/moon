# 02 - Spans y Source (diagnosticos con ubicacion)

Un lenguaje usable no es solo semantica: es diagnostico.
Este capitulo define como representamos ubicaciones en el source y como las convertimos a:
- line/col para CLI
- ranges UTF-16 para LSP

## 0) Definicion: Span

En Moon, un `Span` es un rango **en bytes** dentro del `source.text`:

- `Span { start: usize, end: usize }`
- `start` inclusive, `end` exclusive (convencion estandar)

Archivo:
- `compiler/core/src/span.rs`

Por que bytes y no (line, col)?
- el lexer y parser operan sobre offsets
- merge/union de spans es trivial
- evita ambiguedades con Unicode (codepoints vs graphemes)
- es barato (dos `usize`)

Operacion fundamental:
- `Span::merge(a, b)` produce el rango minimo que contiene ambos

Eso permite que nodos AST grandes (ej: un `if`) tengan un span que cubre toda la expresion.

## 1) Source: path + text + rendering

`Source` es el wrapper que usamos para:
- cargar archivos
- mapear offsets a line/col
- renderizar un span como "error con snippet"

Archivo:
- `compiler/core/src/source.rs`

API relevante:
- `Source::from_path(path) -> Source`
- `Source::line_col(offset) -> (line, col)` (1-based)
- `Source::render_span(span, message) -> String`

Nota tecnica:
- `line_col` actual es O(n) en el offset (itera bytes)
- para un MVP esta bien; a futuro se puede indexar line starts para O(log n)

## 2) Render de errores (CLI)

`render_span`:
- calcula la linea del error
- imprime:
  - `path:line:col: message`
  - la linea de texto
  - un caret `^^^^` con longitud `max(1, end-start)`

Tradeoff:
- el caret usa offsets en bytes; si hay Unicode multibyte puede "desalinearse" visualmente.
- esto se resuelve con un renderer unicode-aware (futuro).

## 3) Spans en el AST

Regla de diseno:
- si el usuario escribio algo, el nodo debe tener span.

Ejemplos:
- `Expr::Int(_, Span)`
- `Stmt::Let { span, .. }`

Esto permite:
- errores del parser/typechecker con ubicacion
- debug info en bytecode (ip -> span)

## 4) LSP y UTF-16 (por que hay que convertir)

LSP define `Position { line, character }` donde:
- `line` es 0-based
- `character` es "UTF-16 code units" (no bytes)

Esto es un clasico punto de friccion:
- Rust `String` indexa por bytes
- LSP usa UTF-16 para compatibilidad historica (VSCode/TS)

Implementacion en Moon:
- `compiler/lsp/src/main.rs`
  - `position_from_offset_utf16(text, offset) -> Position`
  - `offset_from_position_utf16(text, position) -> usize`
  - `range_from_span_utf16(text, span) -> Range`

Tambien hay tests unitarios:
- `utf16_position_roundtrip_ascii`
- `utf16_position_handles_surrogate_pairs`

Eso asegura:
- offsets <-> positions son consistentes
- emojis (surrogate pairs) no rompen ranges

## 5) Practica: follow the span

Toma este programa:

```moon
let f = { let x = 10; fn(y: Int) -> Int { x + y } };
f(1)
```

- El lexer asigna spans a tokens.
- El parser construye:
  - un `Stmt::Let` cuyo span cubre desde `let` hasta `;`
  - un `Expr::Fn` con span desde `fn` hasta `}`
- El typechecker si falla, reporta spans del nodo problematico.
- El bytecode compiler asigna a cada `Instr` el span del AST que la genero.
- La VM pega `span` en errores de runtime.

Ejercicio:
- fuerza un error y mira el span:
  - `let x: Int = true;`
  - `if 1 { 0 } else { 1 }`

## 6) Ejercicios (para reforzar)

1) Optimiza `Source::line_col` precalculando indices de line starts.
2) Implementa un renderer unicode-aware (alinear caret por grapheme clusters).
3) Agrega una funcion helper `Span::len()` y usa `max(1, len)` donde sea necesario.

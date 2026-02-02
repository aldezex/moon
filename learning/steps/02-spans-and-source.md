# 02 - Spans y diagnosticos (Source)

## Por que importan los spans

Cuando hacemos un lenguaje, lo mas frustrante para un usuario es:
- errores sin contexto
- errores sin ubicacion (linea/col)

Por eso, desde el MVP, cada token y cada nodo del AST llevan un **Span**: un rango `[start, end)` en offsets de bytes dentro del source original.

## Span

Archivo:
- `compiler/core/src/span.rs`

Un `Span` es:
- `start`: offset de inicio (bytes)
- `end`: offset de fin (bytes)

Y una operacion clave:
- `merge(a, b)`: crea un span que cubre ambos (usado para spans de expresiones compuestas).

## Source: archivo + texto + render de errores

Archivo:
- `compiler/core/src/source.rs`

`Source` encapsula:
- `path`: para mostrar en errores (ej: `examples/hello.moon` o `<stdin>`)
- `text`: el contenido completo

Funciones importantes:
- `line_col(offset)`: convierte offset -> (linea, columna) 1-based
- `render_span(span, message)`: imprime un mini-diagnostico:
  - `path:line:col: message`
  - la linea del codigo
  - carets `^^^^` marcando el rango

## Errores (frontend)

Archivo:
- `compiler/core/src/error.rs`

En el frontend tenemos:
- `LexError { message, span }`
- `ParseError { message, span }`

La CLI convierte esos errores a diagnosticos humanos via `Source::render_span(...)`.

Ejemplo mental (si faltara un `;`):
- El parser crea `ParseError` con el `Span` de la expresion.
- La CLI llama `render_span` y lo imprime.

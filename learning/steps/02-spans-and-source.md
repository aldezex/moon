# 02 - Spans y diagnosticos (Source)

## Por que importan los spans

Cuando haces un lenguaje, el usuario tolera "errores", pero no tolera:
- errores sin ubicacion
- mensajes vagos
- no saber que parte del codigo esta mal

La diferencia entre "toy language" y "lenguaje usable" suele ser:
- diagnosticos buenos desde el dia 1

Por eso, desde el MVP, Moon usa **Span** en todos lados:
- cada token del lexer
- cada nodo del AST
- cada error (lex/parse/type/runtime/compile)

Un Span es un rango `[start, end)` dentro del texto original.

## Span (rango en bytes)

Archivo:
- `compiler/core/src/span.rs`

`Span`:
- `start: usize`
- `end: usize`

Importante: son offsets en **bytes**, no en "caracteres".

Por que bytes:
- Rust usa UTF-8 en `String`.
- Indexar por "char index" es mas caro y mas ambiguo.
- Para tooling (LSP, resaltado, etc.) lo que importa es un rango en el buffer real.

Operacion clave:
- `Span::merge(a, b)`: produce un Span que cubre ambos.
  - lo usamos cuando construimos nodos compuestos:
    - `a + b` cubre desde el inicio de `a` hasta el fin de `b`

## Source: path + text + render_span

Archivo:
- `compiler/core/src/source.rs`

`Source` encapsula:
- `path`: para mensajes (`examples/hello.moon` o `<stdin>`)
- `text`: el contenido

### line_col(offset)

`line_col` convierte `offset` (bytes) a:
- `(line, col)` 1-based

Hoy es un scan lineal por el texto, suficiente para MVP.
A futuro se puede optimizar con un indice de line starts (vector de offsets).

### render_span(span, message)

`render_span` crea un diagnostico tipo:

```
file.moon:3:15: parse error: expected ';'
let x = 1 + 2
              ^
```

Como lo hace:
1) calcula `line/col` del inicio del span
2) extrae el texto de la linea actual
3) imprime carets `^` desde `span.start` por `len = max(1, span.end - span.start)`

Limitaciones del MVP:
- si el span cruza multiples lineas, solo mostramos la linea del inicio
- tabs se imprimen como `\t` pero el caret puede "desalinearse"

Eso esta bien para arrancar; lo importante es tener el hook (Span) para mejorar.

## Errores y spans (frontend)

Archivo:
- `compiler/core/src/error.rs`

En el frontend:
- `LexError { message, span }`
- `ParseError { message, span }`

La CLI imprime:
- `source.render_span(error.span, "...")`

En el resto del pipeline:
- typechecker: `compiler/typechecker/src/error.rs`
- interpreter: `compiler/interpreter/src/error.rs`
- bytecode compiler: `compiler/bytecode/src/compiler.rs` (CompileError)

## Buenas practicas al propagar spans

1) El lexer debe marcar lo mas "atomicamente" posible
   - ej: `==` tiene su span exacto

2) El parser debe:
   - asignar spans a nodos
   - y usar `merge` para construir spans de expresiones mas grandes

3) Los errores deben apuntar al token o nodo que "explica" el error
   - ej: "undefined variable x" -> span de `x`
   - ej: "expected ';'" -> span de la expresion que necesita `;`

## Mini ejercicios

1) Mejora `render_span` para spans multi-line.
2) Agrega una cache de line starts para que `line_col` sea O(log N) o O(1).
3) Agrega un prefijo con la linea (tipo `  3 | let x = ...`).

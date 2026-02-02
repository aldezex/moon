# 04 - Lexer (tokenizer)

El lexer convierte texto en tokens:

```
source text -> [Token { kind, span }, ... , Eof]
```

Un lexer bueno:
- es determinista
- es total: no paniquea con input arbitrario
- produce spans correctos

Archivo:
- `compiler/core/src/lexer/mod.rs`

## 0) Modelo: Token y TokenKind

`Token`:
- `kind: TokenKind`
- `span: Span` (rangos en bytes dentro del source)

`TokenKind` (resumen):

Literals:
- `Int(i64)`
- `String(String)`
- `Ident(String)`

Keywords:
- `let`, `fn`, `return`
- `if`, `else`
- `true`, `false`

Puntuacion / operadores:
- `(` `)` `{` `}` `[` `]`
- `,` `;` `:`
- `=`
- `->`
- `+ - * / %`
- `! && ||`
- `== != < <= > >=`
- `#` (object literal: `#{ ... }`)

Fin:
- `Eof`

## 1) Diseno: bytes + ASCII first

El lexer opera sobre `source.text.as_bytes()`.

Decisiones MVP:
- Identificadores ASCII: `[A-Za-z_][A-Za-z0-9_]*`
- Strings delimitadas por `"..."` con escapes basicos

Esto simplifica:
- spans (bytes)
- performance
- predictibilidad

A futuro:
- identifiers unicode (requiere decidir normalizacion)
- strings con escapes mas completos

## 2) Algoritmo (loop + cursor)

Estrategia tipica:
- un cursor `i` sobre bytes
- inspeccionas el byte actual
- consumes 1+ bytes segun categoria

Orden recomendado:
1) whitespace
2) comentarios (`// ...`)
3) identifiers/keywords
4) numeros
5) strings
6) operadores de 2 chars (mirando lookahead)
7) operadores de 1 char

El orden importa:
- `==` debe ganar sobre `=`
- `->` debe ganar sobre `-`

## 3) Keywords vs Ident

Regla:
- escaneas un "word" como ident
- luego chequeas si esta en tabla de keywords

Eso permite que:
- `fn` sea keyword
- `foo` sea ident

Importante para el parser:
- `fn name(...)` (declaracion)
- `fn(...)` (expresion anonima)

La distincion final (stmt vs expr) NO la hace el lexer.

## 4) Strings y escapes

Strings:
- empiezan y terminan en `"`
- spans cubren todo el literal

Escapes MVP:
- `\n`, `\t`, `\"`, `\\`

Errores comunes:
- string sin cerrar
- escape desconocido

## 5) Errores (LexError)

Cuando algo no cuadra, devolvemos:
- `LexError { message, span }`

Ejemplos:
- caracter inesperado
- int overflow
- string sin cerrar

La CLI renderiza usando `Source::render_span`.

## 6) Tests y debugging

Si el parser falla, muchas veces el bug esta en tokens.

Estrategia:
- agrega un test que solo corra lexer
- imprime tokens en un debug command (si hace falta)

Ejercicios:
1) soporta `_` en numeros (`1_000_000`)
2) agrega comentarios `/* ... */`
3) agrega escapes hex `\xNN`

# 04 - Lexer (tokenizer)

## Que hace el lexer

El lexer transforma texto crudo en una secuencia de tokens:

```
source text -> [Token { kind, span }, ... , Eof]
```

Esto simplifica el parser: en vez de pelear con caracteres, trabaja con una lista de simbolos ya clasificados.

Archivo:
- `compiler/core/src/lexer/mod.rs`

## Tokens del MVP

`TokenKind` incluye:

- Literales:
  - `Int(i64)`
  - `String(String)`
  - `Ident(String)`
- Keywords:
  - `Let`, `True`, `False`
- Operadores/puntuacion:
  - `+ - * / %`
  - `! = == != < <= > >=`
  - `&& ||`
  - `(` `)` `;`
- `Eof`

Cada `Token` tiene:
- `kind: TokenKind`
- `span: Span`

## Reglas de lexing

1) Whitespace: se ignora (`space/tab/newline`).
2) Comentarios de linea:
   - Si ve `//`, ignora hasta el fin de linea.
3) Identificadores / keywords:
   - `[_A-Za-z][_A-Za-z0-9]*`
   - Si el texto coincide con `let/true/false`, se convierte a keyword.
4) Numeros:
   - Por ahora solo ints decimales: `[0-9]+` -> `i64`
5) Strings:
   - `"..."`
   - Escapes soportados: `\\n`, `\\t`, `\\"`, `\\\\`
   - Nota: por ahora forzamos ASCII (MVP).
6) Operadores y puntuacion:
   - Maneja tokens de 1 y 2 caracteres (ej: `==`, `<=`, `&&`).

## Errores del lexer

Cuando algo no cuadra, devuelve `LexError { message, span }`, por ejemplo:
- caracter inesperado
- string sin cerrar
- escape desconocido
- numero que no entra en `i64`

La CLI imprime estos errores con contexto usando `Source::render_span`.

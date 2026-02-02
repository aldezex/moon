# 04 - Lexer (tokenizer)

## Que hace el lexer

El lexer transforma texto crudo en una secuencia de tokens:

```
source text -> [Token { kind, span }, ... , Eof]
```

La idea es separar responsabilidades:
- el lexer entiende caracteres y "palabras"
- el parser entiende estructura (expresiones, bloques, statements)

Archivo:
- `compiler/core/src/lexer/mod.rs`

## Token y TokenKind

`Token`:
- `kind: TokenKind`
- `span: Span` (rango en bytes dentro del source)

`TokenKind` (MVP actual) incluye:

Literals:
- `Int(i64)`
- `String(String)`
- `Ident(String)`

Keywords:
- `let`
- `fn`
- `if`
- `else`
- `true`
- `false`

Operadores / puntuacion:
- aritmetica: `+ - * / %`
- booleanos: `! && ||`
- comparacion: `< <= > >=`
- igualdad: `== !=`
- asignacion (solo para statements): `=`
- tipos:
  - `:` (anotacion)
  - `->` (return type)
  - `<` `>` (usado en tipos genericos, ej: `Array<Int>`)
- delimitadores:
  - `(` `)` (calls y grouping)
  - `{` `}` (blocks)
  - `[` `]` (arrays + indexing)
  - `,` (listas y props)
  - `;` (expr statement)
- `#` (object literal: `#{ ... }`)

Siempre terminamos con:
- `Eof`

## Algoritmo (alto nivel)

Implementacion MVP: un loop con un cursor `i` sobre bytes.

1) Whitespace
   - se ignora (`space/tab/newline`)

2) Comentarios
   - `// ...` hasta fin de linea

3) Identificadores / keywords
   - `[_A-Za-z][_A-Za-z0-9]*`
   - si coincide con una keyword, produce ese token

4) Numeros
   - por ahora solo ints decimales: `[0-9]+`

5) Strings
   - `" ... "`
   - escapes MVP: `\\n`, `\\t`, `\\"`, `\\\\`

6) Operadores/puntuacion
   - tokens de 1 o 2 caracteres
   - casos de 2 chars importantes:
     - `==`, `!=`, `<=`, `>=`, `&&`, `||`, `->`

## Errores del lexer

Cuando algo no cuadra, el lexer devuelve:
- `LexError { message, span }`

Ejemplos:
- caracter inesperado
- string sin cerrar
- escape desconocido
- numero invalido (overflow de i64)

La CLI imprime el error usando:
- `Source::render_span(error.span, ...)`

## Tips de debugging

Cuando el parser falla, muchas veces conviene:
1) agregar un comando CLI o log para imprimir tokens
2) verificar que el lexer no este "comiéndose" caracteres por error

Un truco: cuando agregas un token nuevo, crea un test pequeno que solo haga lexing.

## Mini ejercicios

1) Agrega soporte para comentarios de bloque `/* ... */`.
2) Soporta `_` en numeros: `1_000_000`.
3) Soporta escapes hex: `\\xNN`.

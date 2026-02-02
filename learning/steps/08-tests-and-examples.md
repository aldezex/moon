# 08 - Tests y ejemplos

## Ejemplo: hello.moon

Archivo:
- `examples/hello.moon`

Hoy el ejemplo solo usa features del MVP:
- `let`
- aritmetica + precedencia
- strings + concatenacion
- tail expression (sin `;` al final) para que la CLI imprima el resultado

Ejecutar:
- `cargo run -- run examples/hello.moon`
- `cargo run -- check examples/hello.moon`

## Tests del interpreter

Archivo:
- `compiler/interpreter/tests/mvp.rs`

La idea de estos tests:
- Usar el pipeline real: `lex -> parse -> eval`.
- Probar reglas importantes de semantica:
  - precedencia aritmetica: `1 + 2 * 3`
  - precedencia logica: `true && false || true`
  - concatenacion de strings: `"a" + "b"`
  - bloques + scopes + tail expression
  - `if/else` como expresion
  - funciones y llamadas (incluyendo call-before-definition)

Ejecutar:
- `cargo test --workspace`

## Como agregar mas tests

Patron sugerido:
1) escribe un snippet `.moon` como string
2) corre `lex/parse/eval`
3) assert del `Value` final o del error esperado

Si estamos agregando una feature nueva (por ejemplo `if`):
- agrega primero tests que describan el comportamiento deseado
- implementa lo minimo para pasarlos

## Tests del typechecker

Archivo:
- `compiler/typechecker/tests/typechecker.rs`

Estos tests validan:
- inferencia basica (`let x = ...`)
- errores de tipo (mismatch en anotaciones, `if` con ramas distintas, argumentos incorrectos, etc.)

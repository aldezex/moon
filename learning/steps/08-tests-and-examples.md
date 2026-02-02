# 08 - Tests y ejemplos

## Ejemplo: hello.moon

Archivo:
- `examples/hello.moon`

Hoy el ejemplo solo usa features del MVP:
- `let`
- aritmetica + precedencia
- strings + concatenacion

Ejecutar:
- `cargo run -- run examples/hello.moon`

## Tests del interpreter

Archivo:
- `compiler/interpreter/tests/mvp.rs`

La idea de estos tests:
- Usar el pipeline real: `lex -> parse -> eval`.
- Probar reglas importantes de semantica:
  - precedencia aritmetica: `1 + 2 * 3`
  - precedencia logica: `true && false || true`
  - concatenacion de strings: `"a" + "b"`

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

# 08 - Tests y ejemplos (como validar el lenguaje)

## Por que tests desde el MVP

En un lenguaje, un cambio pequeno puede romper cosas muy lejos:
- cambiar precedencia rompe cientos de programas
- cambiar scoping rompe funciones/blocks
- cambiar Value rompe interpreter y VM

Por eso:
- tests unitarios del frontend
- tests de pipeline (lex -> parse -> eval)
- tests de VM (lex -> parse -> typecheck -> compile -> run)

La meta no es tener "muchos tests", sino tests que cubran:
- reglas fundamentales
- casos borde
- regressions tipicas

## Ejemplos

Carpeta:
- `examples/`

Ejemplo actual:
- `examples/hello.moon`

Tip: manten los examples chicos, pero representativos. Un ejemplo por feature nueva ayuda muchisimo a onboarding.

## Tests del interpreter

Archivo:
- `compiler/interpreter/tests/mvp.rs`

Que cubre:
- precedencia aritmetica y logica
- tail expression
- blocks + scopes
- if/else
- funciones + call-before-definition
- arrays/objects + indexing + assignment

Patron de test:
1) crear `Source` desde string
2) `lex` -> `parse` -> `eval_program`
3) assert del `Value`

## Tests del typechecker

Archivo:
- `compiler/typechecker/tests/typechecker.rs`

Que cubre:
- inferencia basica en lets
- anotaciones correctas vs mismatch
- reglas de `if` (cond Bool, ramas mismo tipo)
- llamadas a funciones (aridad + tipos)
- arrays/objects:
  - inferencia
  - `[]`/`#{}` vacios requieren anotacion
  - indexing y assignment

Tip:
- los tests de typechecker deberian assert sobre `message` (no sobre formato exacto del diagnostico) para permitir mejorar mensajes sin romper tests.

## Tests de la VM (bytecode)

Archivo:
- `compiler/vm/tests/vm.rs`

Pipeline:
- `lex -> parse -> typecheck -> compile -> vm.run`

Estos tests son cruciales para:
- verificar que compiler+VM preservan semantica del interpreter
- detectar bugs de stack discipline (Pop, Return, etc.)

## Ideas para mejorar tests

1) "Golden tests" de diagnosticos:
   - snapshot de `render_span` para algunos errores comunes

2) Property-based tests (a futuro):
   - generar expresiones random y comparar:
     - interpreter vs VM (deben dar mismo valor)

3) Tests por feature:
   - cada PR que agrega sintaxis nueva deberia traer:
     - parser tests (AST shape)
     - typechecker tests
     - interpreter tests
     - VM tests (si aplica)

# 08 - Tests y ejemplos (calidad y regresiones)

Un lenguaje sin tests se rompe silenciosamente.
Este capitulo describe como testeamos Moon y por que.

## 0) Objetivo

- Capturar semantica como contratos
- Evitar divergencia entre interpreter y VM
- Hacer facil agregar features sin miedo

## 1) Donde estan los tests

### 1.1 Interpreter
- `compiler/interpreter/tests/mvp.rs`

Tests de semantica directa:
- precedencias
- bloques + scopes
- funciones/closures
- arrays/objects
- `return`

### 1.2 Typechecker
- `compiler/typechecker/tests/typechecker.rs`

Tests de errores y reglas:
- mismatches
- llamadas
- `return` y `Never`
- closures y `Expr::Fn`

### 1.3 VM
- `compiler/vm/tests/vm.rs`

Tests del pipeline:
- parse + typecheck + compile + run
- asegura que bytecode/VM coincide con semantica del interpreter

## 2) Filosofia: contratos, no outputs

Preferimos tests de semantica:
- "este programa devuelve 42"
- "este programa debe fallar con error que contiene X"

Evita tests fragiles:
- snapshots de AST/bytecode son utiles, pero cambian facil

## 3) Consistencia interpreter vs VM

Regla:
- cada feature de lenguaje debe tener al menos:
  - 1 test en interpreter
  - 1 test en VM

Ejemplos actuales:
- closures:
  - `anonymous_functions_work`
  - `closures_capture_lexical_variables`
  - `closures_can_mutate_captured_state`

Esto evita que:
- el parser cambie y rompa solo un backend
- la VM tenga orden de evaluacion distinto

## 4) Tests de errores (mensajes)

Los tests no comparan el error completo (fragil).
Comparan substrings:
- `assert!(err.contains("type mismatch"))`

Eso mantiene:
- flexibilidad de mensajes
- pero asegura categoria de error

## 5) Ideas avanzadas (cuando el lenguaje crezca)

### 5.1 Golden tests
- input `.moon` -> output esperado
- ideal para `moon run` y `moon vm`

### 5.2 Differential testing
- correr el mismo programa en ambos backends
- comparar valores finales

### 5.3 Fuzzing
- fuzz del lexer/parser:
  - no panics
  - spans validos

## 6) Ejercicios

1) Agrega un helper que corra el mismo snippet en interpreter y VM y compare `Value`.
2) Crea un directorio `tests/golden/` con programas y outputs.
3) Agrega tests para unicode en strings (y mira spans).

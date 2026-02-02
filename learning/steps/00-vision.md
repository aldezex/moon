# 00 - Vision y principios (Moon)

Este capitulo define el "contrato" del proyecto: objetivos, no-objetivos, y decisiones base.
Si te pierdes en una implementacion, vuelve aqui: casi todo tradeoff importante se deriva de esto.

## 0) Que es Moon (definicion operativa)

Moon es un lenguaje de scripting con:
- tipado estricto (sin `any` implicito)
- semantica expresiva (bloques con tail expression, `if` como expresion)
- tooling serio (spans, diagnosticos, LSP)
- runtime con heap + GC
- dos backends de ejecucion:
  - interpreter (tree-walk) para iterar rapido
  - bytecode + VM para performance/tooling

Si lo quieres comparar:
- UX/iteracion: JS/TS
- disciplina de implementacion: Rust (capas claras, tipos, tests)

## 1) Objetivos (lo que optimizamos)

### 1.1 Strict typing que no estorba

- Todo programa pasa por `moon_typechecker` antes de ejecutar.
- No hay `any` implicito.
- Los errores deben ser:
  - deterministas
  - con span (ubicacion exacta)
  - accionables (mensaje + contexto)

### 1.2 Lenguaje expresion-oriented

La sintaxis favorece construir valores:
- Un bloque `{ ... }` puede producir un valor via tail expression.
- `if ... else ...` es expresion.

Esto reduce "ceremonia" y simplifica la VM (mas expresiones, menos statements especiales).

### 1.3 Scripting real: funciones como valores y closures

Para parecerse a JS/TS, Moon necesita:
- funciones como valores (`let f = add1; f(41)`)
- funciones anonimas (`fn(...) -> ... { ... }` como expresion)
- closures (captura lexical)

En Moon (MVP actual):
- las closures capturan variables locales por valor (snapshot shallow) en un environment heap-alloc
- ese environment es mutable (puedes actualizar estado capturado dentro de la closure)

### 1.4 Una base para escalar

- Empezamos con interpreter para iterar semantica.
- Movemos la misma semantica a bytecode+VM.
- Mantener runtime compartido (Value/Heap/GC) evita divergencias.

## 2) No-objetivos (por ahora)

Importante para no romper ritmo:
- no hay borrow checker, lifetimes, ownership al estilo Rust
- no hay macros
- no hay JIT
- no hay modulos/imports
- no hay loops (`while`/`for`) ni `break/continue`
- no hay sintaxis de tipos de funcion en el source (el typechecker si maneja `Type::Function`)

## 3) Principios de diseno (reglas practicas)

### 3.1 Capas con responsabilidades duras

- `compiler/core`:
  - tokens, spans, AST, parser
  - NO ejecuta
- `compiler/typechecker`:
  - valida semantica estatica
  - produce errores con spans
- backends:
  - `compiler/interpreter`: ejecuta AST
  - `compiler/bytecode` + `compiler/vm`: compila a IR y ejecuta
- `compiler/runtime`:
  - `Value`, heap, GC
  - compartido por interpreter y VM

### 3.2 La semantica debe estar testeada en ambos backends

Cada feature importante debe tener tests:
- interpreter (tree-walk)
- VM (bytecode)

Eso fuerza consistencia.

### 3.3 Spans en todas las capas

Cualquier error que le importe al usuario debe tener ubicacion:
- lexer/parser/typechecker
- runtime (interpreter) y VM

Implementacion:
- `moon_core::span::Span` (rangos en bytes)
- `moon_core::source::Source` renderiza spans a line/col y snippet

### 3.4 Semantica determinista (orden de evaluacion)

Elegimos orden de evaluacion y lo fijamos con tests.
Ejemplo (importante para side effects y `return`):
- en `a[i] = rhs;` evaluamos `a` y `i` antes de `rhs` (VM e interpreter alineados)

## 4) Pipeline (lo que pasa cuando corres un archivo)

Entrada: texto (source)

1) Lexer
- `source text -> Vec<Token>`
- cada `Token` tiene `TokenKind` + `Span`

2) Parser
- `Vec<Token> -> Program (AST)`
- AST mantiene spans

3) Typechecker
- valida tipos, variables, aridad, etc
- errores con span

4) Backend
- interpreter (AST) o bytecode+VM

## 5) Memoria (decision)

Moon tiene heap objects:
- arrays
- objects
- closures (environment)

Decision MVP:
- GC mark/sweep

Razon:
- ciclos no son un problema (a diferencia de RC puro)
- el "root set" es claro:
  - globals
  - scopes (frames)
  - operand stack (VM)
  - closures activas (frame.closure)

Nota:
- el builtin `gc()` dispara un ciclo manualmente para debug.

## 6) Como trabajar en features (workflow)

Para agregar una feature de lenguaje:
1) AST + parser
2) typechecker
3) interpreter
4) bytecode compiler
5) VM
6) tests en interpreter + VM
7) actualizar `learning/*`

Esta disciplina evita que Moon crezca como "demo" y se vuelve una base real.

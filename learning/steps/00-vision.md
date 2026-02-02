# 00 - Vision y principios

## Que es Moon (en una frase)

Moon es un lenguaje de scripting (feedback rapido) con una arquitectura de compilador seria (spans, typechecker estricto, runtime con GC, VM).

Piensalo como:
- ergonomia de "script" (tipo JS/TS)
- disciplina de "engine" (tipo Rust en tooling y claridad)

## Objetivos

1) Scripting de verdad
   - `moon run file.moon` debe ser simple y rapido
   - errores con contexto y ubicacion (line/col)

2) Tipado estricto
   - sin `any` implicito
   - reglas claras para operadores, `if`, llamadas, etc.
   - el typechecker debe fallar temprano y con buen mensaje

3) Base para performance
   - empezamos con interpreter tree-walk para iterar en lenguaje
   - sumamos un backend de bytecode + VM para escalar

4) Runtime real (pensado para lenguaje dinamico)
   - arrays y objects (maps)
   - heap + GC por trazado (mark/sweep)
   - prepara el terreno para closures y ciclos (a futuro)

## No-objetivos (por ahora)

Esto es importante para no romper el ritmo:
- no hay macros, lifetimes, ni borrow checker al estilo Rust
- no hay JIT
- no hay un sistema de tipos estructural estilo TS completo (todavia)
- no hay closures/capturas (todavia)

## Principios de diseño (practicos)

1) Capas con responsabilidades claras
   - frontend (lexer/parser/AST) no "ejecuta"
   - semantica:
     - typechecker (valida)
     - interpreter/VM (ejecutan)
   - runtime (Value/heap/GC) compartido

2) Todo error importante debe tener Span
   - lex/parse/type/runtime deben poder apuntar al codigo que causo el problema

3) MVP primero, generalidad despues
   - implementamos la feature mas simple que desbloquea el siguiente paso
   - refactor cuando el nuevo requisito sea real (no hipotetico)

## El pipeline completo (hoy)

Cuando ejecutas un archivo:

```
source text
  -> lexer (tokens + spans)
  -> parser (AST + spans)
  -> typechecker (ok o TypeError con span)
  -> backend:
       - interpreter (tree-walk)  OR
       - bytecode compiler + VM
```

La CLI expone esto como:
- `moon run file.moon` (typecheck + interpreter)
- `moon vm file.moon` (typecheck + bytecode + VM)
- `moon check file.moon` (solo typecheck)

## Memoria (decision)

En un lenguaje con heap objects (arrays/objects/closures), el enfoque mas practico suele ser:
- heap con GC por trazado (mark/sweep)

Por que:
- evita leaks por ciclos (RC se complica con ciclos)
- la VM/interpreter pueden dar al GC un "root set" claro (globals, scopes, stack)

En Moon ya tenemos un GC mark/sweep simple en `moon_runtime` y un builtin `gc()` para dispararlo.

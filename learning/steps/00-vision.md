# 00 - Vision y principios

## Que es Moon

Moon quiere ser un lenguaje de scripting (ejecucion directa, ciclos cortos de feedback) con varias ideas "heredadas" de Rust:
- Un core pensado para ser **preciso** (buenos mensajes de error, spans, tooling).
- Un diseño que pueda evolucionar a **tipado estricto** (sin `any` implicito) y a un runtime eficiente.

No buscamos copiar Rust literalmente: queremos algo mas cercano a JS/TS en ergonomia (script rapido), pero con una arquitectura de compilador seria por debajo.

## Como lo estamos construyendo (capas)

Separar por capas nos permite iterar sin mezclar responsabilidades:

1) Frontend (sintaxis):
   - Lexer: texto -> tokens
   - Parser: tokens -> AST
   - Diagnosticos: spans + render de errores sobre el source

2) Semantica inicial:
   - Interpreter tree-walk: ejecuta el AST directamente (ideal para MVP)

3) Siguiente etapa (aun no implementada):
   - Typechecker estricto: valida tipos antes de ejecutar (y produce errores con spans)

4) Performance + tooling (a futuro):
   - Bytecode + VM (o JIT mas adelante)

## Decision sobre memoria (a futuro)

Para un lenguaje dinamico con objetos/arrays/closures, lo mas practico suele ser:
- Heap con **GC por trazado** (mark/sweep o mark/compact), porque maneja bien ciclos y reduce overhead de RC en grafos con muchas referencias.

En el MVP actual solo tenemos valores simples (`int/bool/string`) y no necesitamos GC todavia.

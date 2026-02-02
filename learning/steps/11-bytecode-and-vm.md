# 11 - Bytecode + VM (diseño)

Este capitulo describe el salto desde el tree-walk interpreter (ejecutar AST directamente) hacia un backend mas rapido:
- compilar AST a bytecode
- ejecutar bytecode en una VM

Todavia no esta implementado; es el siguiente gran paso despues de estabilizar sintaxis/semantica.

## Por que una VM

El interpreter tree-walk es excelente para MVP, pero:
- re-traversa estructuras del AST todo el tiempo
- hace muchas dispatches por nodo
- es dificil optimizar sin reestructurar todo

Una VM con bytecode:
- ejecuta instrucciones compactas
- permite caches y optimizaciones
- es mejor base para tooling (debugger/step, profiling, etc.)

## Diseño sugerido

### Crates sugeridos

- `compiler/bytecode`
  - define `Instruction`, `Chunk`/`Module`, constant pool
- `compiler/vm`
  - ejecuta bytecode, maneja stack frames, globals, GC roots

### Modelo: stack-based VM (simple)

Instrucciones tipicas:
- `Const k`          -> push const[k]
- `Pop`              -> pop
- `LoadGlobal name`  -> push globals[name]
- `StoreGlobal name` -> globals[name] = pop
- `LoadLocal i` / `StoreLocal i`
- `Add/Sub/Mul/...`
- `Eq/Ne/Lt/...`
- `JumpIfFalse addr`
- `Jump addr`
- `Call fn, argc`
- `Return`

El bytecode suele venir con:
- `constants: Vec<Value>` (o valores del runtime)
- `spans: Vec<Span>` para mapear instruction pointer -> span (diagnosticos)

### Compilador (AST -> bytecode)

Pasos:
1) Lowering:
   - `Program` -> una funcion "main"
   - `fn` -> funciones con su propio chunk
2) Compilar expresiones:
   - emitir instrucciones que dejan el valor en el stack
3) Compilar statements:
   - `let` -> eval expr, store
   - `expr;` -> eval expr, pop
4) Compilar blocks:
   - scopes -> slots de locals (stack slots)
5) Compilar if:
   - compilar cond
   - `JumpIfFalse` al else
   - compilar then
   - `Jump` al end
   - compilar else

### VM (bytecode -> ejecucion)

Estructuras:
- operand stack: `Vec<Value>`
- call stack: frames (ip, locals base, function id)
- globals: map o vector indexado

Errores:
- runtime errors deberian incluir span:
  - usando metadata `ip -> Span`
  - o guardando span en cada instruccion

## Interaccion con typechecker

Pipeline ideal:
- `check`: parse -> typecheck
- `run`:
  - parse -> typecheck -> compile -> vm

Mientras no exista VM:
- `run` usa el interpreter.

## Siguiente paso practico

MVP de VM (para nuestro lenguaje actual):
1) bytecode para literales/ops/let/blocks/if/calls
2) funciones globales sin closures
3) spans por instruccion para diagnosticos

Luego:
4) closures + heap + GC roots

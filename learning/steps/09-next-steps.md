# 09 - Proximos pasos

Este capitulo es un mapa de hacia donde vamos. No todo esta implementado aun.

## 1) Bloques y scopes

Agregar:
- bloques `{ ... }` como expresion (devuelve el ultimo valor)
- scopes anidados (shadowing) para que `let` sea por bloque, no global

Esto requiere:
- extender el AST (`Expr::Block` o similar)
- extender el parser para `{` `}`
- extender el interpreter: `Env` con stack de scopes o una estructura enlazada

## 2) If/else como expresion

Agregar:
- `if cond { ... } else { ... }`

Necesita:
- condiciones booleanas (en runtime y luego en typechecker)
- buen mensaje de error si `cond` no es bool

## 3) Funciones

Agregar:
- `fn` declarations
- llamadas `f(x, y)`
- closures (eventualmente)

Esto empuja la decision de memoria (GC) cuando haya valores heap compartidos.

## 4) Typechecking estricto (la gran meta cercana)

En un lenguaje de scripting con tipado estricto, una estrategia practica es:
- `moon check`: parsea y typecheckea, sin ejecutar
- `moon run`: parsea + typecheck + ejecuta

Ideas iniciales:
- Tipos primitivos: `Int`, `Bool`, `String`, `Unit`
- Reglas estrictas:
  - no permitir `int + string` (a menos que se defina explicitamente)
  - cond de `if` debe ser `Bool`
  - `&& ||` solo para `Bool`

Arquitectura sugerida:
- nuevo crate: `compiler/typechecker`
- produce:
  - o bien un AST anotado con tipos
  - o bien una tabla de tipos por `Span`/NodeId

## 5) Runtime real (objetos/arrays) + GC

Para pasar de "valores simples" a lenguaje de scripting:
- arrays, objetos (maps), strings mas avanzados
- closures, stack frames
- heap + GC (mark/sweep)

Arquitectura sugerida:
- crate `compiler/runtime` o `compiler/vm` (cuando exista bytecode)
- crate `compiler/gc` (si vale la pena aislarlo)

## 6) Bytecode + VM

Cuando el lenguaje crezca, el tree-walk interpreter se queda corto.
El paso natural:
- compiler: AST -> bytecode
- vm: bytecode -> ejecucion

Beneficios:
- performance mas estable
- mejor base para tooling (debugger, stepping, etc.)

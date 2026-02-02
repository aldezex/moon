# 12 - Proximos pasos (roadmap vivo)

Este capitulo es un mapa de hacia donde vamos. Algunas cosas ya estan implementadas (y las referenciamos) y otras quedan como trabajo futuro.

## 1) Bloques y scopes

Estado: implementado.

Ver:
- `learning/steps/03-ast.md` (Expr::Block)
- `learning/steps/05-parser.md` (tail expression y blocks)
- `learning/steps/06-interpreter.md` (scopes)

## 2) If/else como expresion

Estado: implementado.

Ver:
- `learning/steps/05-parser.md` (IfExpr)
- `learning/steps/06-interpreter.md` (runtime)
- `learning/steps/09-typechecker.md` (type rules)

## 3) Funciones

Estado: implementado (version inicial).

Notas:
- Hoy las funciones se declaran en top-level (restriccion actual).
- No hay closures aun (no capturan variables locales).
- Calls son por nombre (`f(...)`).

## 4) Typechecking estricto (la gran meta cercana)

Estado: implementado (MVP).

Ver:
- `learning/steps/09-typechecker.md`

## 5) Runtime real (objetos/arrays) + GC

Pendiente.

Para pasar de "valores simples" a un lenguaje de scripting:
- arrays, objetos (maps), strings mas avanzados
- closures, stack frames / call frames
- heap + GC (mark/sweep)

Ver diseno:
- `learning/steps/10-runtime-and-gc.md`

## 6) Bytecode + VM

Pendiente.

Cuando el lenguaje crezca, el tree-walk interpreter se queda corto. El paso natural:
- compiler: AST -> bytecode
- VM: bytecode -> ejecucion

Beneficios:
- performance mas estable
- mejor base para tooling (debugger, stepping, etc.)

Ver diseno:
- `learning/steps/11-bytecode-and-vm.md`

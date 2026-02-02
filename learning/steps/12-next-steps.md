# 12 - Proximos pasos (roadmap vivo)

Este capitulo es un mapa practico: que ya esta implementado, que queda, y (lo mas importante) como pensar el diseno/implementacion en un lenguaje real.

No es una lista "fija". La idea es que, cada vez que implementemos un feature nuevo, volvamos aqui:
- lo movemos de "Roadmap" a "Implementado"
- anotamos las decisiones y tradeoffs que hicimos
- agregamos tests y links a los archivos tocados

## 0) Checklist de lo que ya tenemos (MVP)

Ya esta implementado y documentado:
- Bloques + scopes + tail expression: `learning/steps/05-parser.md`, `learning/steps/06-interpreter.md`
- `if/else` como expresion: `learning/steps/05-parser.md`, `learning/steps/09-typechecker.md`
- Funciones top-level + llamadas por nombre: `learning/steps/03-ast.md`, `learning/steps/05-parser.md`
- `return expr?;` como statement dentro de funciones (early exit): `learning/steps/05-parser.md`, `learning/steps/06-interpreter.md`, `learning/steps/09-typechecker.md`
- Typechecking estricto: `learning/steps/09-typechecker.md`
- Runtime con heap + GC mark/sweep (arrays/objects): `learning/steps/10-runtime-and-gc.md`
- Bytecode + VM: `learning/steps/11-bytecode-and-vm.md`
- Language Server (LSP): `learning/steps/13-language-server.md`

Eso nos deja una base "completa" para seguir creciendo sin reescribir todo.

## 1) Funciones de verdad: funciones como valores + closures (prioridad alta)

Hoy Moon tiene funciones, pero con restricciones MVP:
- se declaran solo en top-level
- se llaman solo por nombre (el callee debe ser `Ident`)
- no hay closures (no capturan variables locales)

Para que Moon sea realmente "scripting" (estilo JS/TS), el salto grande es:
- tratar funciones como valores: `let f = add1; f(41)`
- permitir funciones anonimas: `let f = fn(x: Int) -> Int { x + 1 };`
- permitir closures: `let make = fn(x: Int) -> fn(Int)->Int { fn(y: Int)->Int { x + y } }`

### Por que es dificil (y por que vale la pena)

Un closure obliga a resolver:
- representacion: como guardo "codigo" + "ambiente capturado"
- type system: cual es el tipo exacto de la funcion (incluyendo args/ret)
- runtime/GC: el ambiente capturado vive en heap y tiene que ser trazable
- VM: como compilo/ejecuto `UpvalueGet` / `UpvalueSet` (o equivalente)

### Plan incremental (recomendado)

Paso A: funciones como valores, sin captures (barato)
- Parser/AST: permitir referenciar funciones por nombre como expresion (ya existe `Expr::Ident`).
- Typechecker: permitir asignar `Ident` de funcion a un `let` y cargar su tipo (ya existe `Type::Function`).
- Interpreter/VM: representar un "function value" (ej: `Value::Function(FuncId)` o `Value::Function(String)`).

Paso B: `fn` como expresion (anonimas), aun sin captures (medio)
- Parser: agregar un `Expr::Fn` o similar (distinto de `Stmt::Fn` top-level).
- Typechecker: generar un tipo de funcion desde params/ret y typecheckear body.
- Bytecode: compilar la funcion a un Function interno y devolver un handle en runtime.

Paso C: closures con captures (caro, pero desbloquea todo)
- Frontend: decidir "que se captura" (solo lectura vs mut, por referencia vs por valor).
- Runtime: agregar `Value::Closure(GcRef)` con un objeto heap que contenga:
  - id de funcion (o pointer al Function en el Module)
  - vector/map de valores capturados (upvalues)
- VM: agregar instrucciones para:
  - crear closure (con capturas)
  - leer/escribir upvalues durante ejecucion
- GC: marcar upvalues como children del closure.

Archivos que probablemente cambian:
- `compiler/core/src/ast/mod.rs` (Expr::Fn / Expr::Call callee general)
- `compiler/core/src/parser/mod.rs` (parse de `fn` como expr)
- `compiler/typechecker/src/lib.rs` (callee ya no es solo ident; closures)
- `compiler/runtime/src/value.rs` y `compiler/runtime/src/heap.rs` (Closure)
- `compiler/bytecode/src/compiler.rs` + `compiler/bytecode/src/instr.rs` (MakeClosure, Upvalue ops)
- `compiler/vm/src/vm.rs` (frames con closure env)

## 2) Control flow: `return`, loops, break/continue

Hoy todo es "expresion" y la salida de una funcion depende del tail expression del block.
Eso funciona, pero en la practica queremos control flow explicito.

Estado:
- `return expr?;` implementado (early exit claro).
- loops (`while`, `loop`, quizas `for`) pendiente.
- `break` / `continue` pendiente.

### `return`: diseno y decisiones (lo que hicimos)

Sintaxis:
- `return expr?;`
  - `return;` devuelve `Unit`

Reglas:
- `return` solo se permite dentro de funciones (lo valida el typechecker).
- Si un `return` "escapa" al top-level, es error (guardrail en interpreter).

Implementacion (patron reusable para loops):
- Typechecker:
  - agregamos `Type::Never` para modelar "diverge" (un bloque/rama que no produce valor).
  - regla clave: `Never` es compatible con cualquier tipo esperado.
- Interpreter:
  - `eval_*` propaga `Return(value)` hacia arriba hasta que el handler de llamadas lo consume.
- Bytecode/VM:
  - el bytecode ya tenia `Instr::Return`; ahora compila `Stmt::Return` a `Return` (con span del statement).

Archivos clave:
- `compiler/core/src/ast/mod.rs`
- `compiler/core/src/parser/mod.rs`
- `compiler/typechecker/src/lib.rs`
- `compiler/interpreter/src/eval.rs`
- `compiler/bytecode/src/compiler.rs`

### Como se implementa sin volverse loco

En interpreter y VM suele aparecer el mismo patron:
- introducis un tipo de control-flow "no local"
  - ej: `EvalResult = Value | ControlFlow`
  - donde `ControlFlow` puede ser `Return(Value)`, `Break(Value?)`, `Continue`
- y propagas eso hacia arriba hasta que alguien lo consume (una funcion consume `Return`, un loop consume `Break/Continue`).

En bytecode:
- `return` compila directo a `Instr::Return` (ya existe)
- `while`/`loop` requiere patching de jumps (como `if`)
- `break`/`continue` necesitan que el compiler lleve una stack de "labels" de loop para saber a donde saltar

## 3) Tipos mas expresivos (sin perder el "estricto")

El typechecker MVP hoy tiene una simplificacion importante:
- `Object<T>` es homogeneo: todas las values deben ser `T`

Eso simplifica arrays/objects, pero limita mucho:
- `#{ a: 1, b: "x" }` no es posible (mezcla Int y String)

### Opciones de diseno (hay que elegir)

Opcion 1: mantener `Object<T>` y agregar "records" estructurales
- `Object<T>` sigue siendo map dinamico (String -> T)
- Agregas `Record{a: Int, b: String}` como tipo distinto (campos fijos)
- Nuevas reglas:
  - literal `#{ a: 1, b: "x" }` puede inferir `Record{a:Int,b:String}`
  - indexing con string literal `"a"` puede resolverse a campo (si es record)
  - (opcional) sumar sintaxis `o.a` (dot access) para records

Opcion 2: union types / sum types (mas potente, mas caro)
- `Object<Int | String>` permitiria mezcla, pero complica toda la semantica (operadores, narrowing, etc.)

Opcion 3: un `Any` (NO recomendado si queremos "sin any implicito")
- rompe el objetivo principal.

Para Moon, la opcion 1 suele ser el mejor paso incremental:
mantienes un map dinamico tipado, y sumas records tipados cuando queres estructura.

## 4) Errores mejorados: spans en VM/bytecode + disassembler

Estado: implementado (MVP).

Lo que agregamos:

1) Debug info: Span por instruccion
- En `compiler/bytecode`, `Instr` es `Instr { kind, span }`.
- El compiler asigna el span del nodo AST que genero esa instruccion.

Archivos:
- `compiler/bytecode/src/instr.rs`
- `compiler/bytecode/src/compiler.rs`

2) Errores de VM con ubicacion
- `VmError` ahora incluye `span`.
- La VM mantiene `current_span` (span de la instruccion actual) y lo pega a los errores.

Archivos:
- `compiler/vm/src/error.rs`
- `compiler/vm/src/vm.rs`

3) `moon disasm <file>`
- Comando de CLI que imprime funciones + bytecode con `ip` y spans.

Archivos:
- `src/main.rs` (cmd `disasm`)

Ver mas detalle en:
- `learning/steps/11-bytecode-and-vm.md`

## 5) Performance en VM: variables por slots (y menos HashMap)

La VM MVP usa HashMaps por scope, como el interpreter. Es simple y correcto, pero no escala bien:
- cada LoadVar/SetVar hace hashing de string
- cada scope crea un HashMap

El upgrade clasico:
- en compilacion, resolves variables a "slots" (indices) dentro de un frame
- el frame guarda `Vec<Value>` en vez de HashMap
- `LoadLocal(slot)` y `StoreLocal(slot)` son O(1) y no asignan strings

Esto tambien te obliga a definir bien:
- locals vs globals
- como se resuelven capturas (closures), que pasan a ser upvalues

## 6) Runtime: auto-GC, strings mejores, builtins utiles

Ya tenemos heap + GC mark/sweep, pero falta "producto":

Auto-GC (heuristica):
- correr GC cada N allocs, o cuando heap slots crecen mas de X%
- expone counters para debug (bytes allocados, objetos vivos)

Strings:
- interning (para keys frecuentes)
- slicing/rope (si apuntamos a performance, no es necesario al principio)

Builtins:
- `print(x)` / `dbg(x)`
- `len(array|string|object)`
- `push(array, x)` (o metodo `array.push(x)` si sumamos dot calls)

Nota: aunque el lenguaje sea estricto, los builtins tambien deben estar tipados en `compiler/typechecker`.

## 7) Modulos, imports y stdlib

Sin modulos, todo vive en un archivo. Para crecer:
- `import "path/to/file.moon"`
- un loader con cache (un modulo se evalua una vez)
- nombres exportados (por ahora podria ser "todo lo top-level es publico")

En una version "seria":
- namespaces
- `export` explicito
- stdlib versionada y testeada

## 8) Calidad: mas tests, fuzzing y golden files

Cuando el lenguaje crece, lo que mas duele son regresiones silenciosas.
Ideas practicas:
- tests "pipeline" que corran tanto `moon run` como `moon vm` con el mismo input y comparen outputs
- golden tests (input -> output esperado)
- fuzzing del lexer/parser (que no panic; que reporte errores con spans validos)

## 9) Que feature hacemos ahora (sugerencia)

Si tenemos que elegir un siguiente paso "con mejor ROI":

Ya completamos:
- spans en VM/bytecode + `moon disasm`
- `return expr?;` (early exit dentro de funciones)

Siguiente trio con mejor impacto:
1) funciones como valores (sin captures) y luego closures
2) loops (`while`/`loop`) + `break`/`continue`
3) performance: variables por slots (cuando empiece a doler)

Cada uno de esos pasos mejora:
- usabilidad del lenguaje
- estructura del runtime/VM
- calidad de los diagnosticos

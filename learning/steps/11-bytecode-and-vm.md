# 11 - Bytecode + VM (AST -> bytecode -> VM)

## Por que agregar bytecode si ya hay interpreter

El tree-walk interpreter es genial para iterar:
- implementas features rapido
- el codigo es directo (match sobre AST)

Pero tiene limites:
- cada ejecucion recorre el AST (mucho overhead)
- los dispatches por nodo son caros
- es mas dificil construir tooling (debugger/step) sobre AST

La VM con bytecode:
- ejecuta instrucciones compactas
- separa "compilar" de "ejecutar"
- prepara el terreno para optimizaciones futuras

En Moon hoy tenemos ambos:
- `moon run` -> interpreter
- `moon vm` -> bytecode compiler + VM

## Componentes

### 1) Bytecode IR

Crate:
- `compiler/bytecode` (`moon_bytecode`)

Archivos:
- `compiler/bytecode/src/instr.rs`
- `compiler/bytecode/src/module.rs`
- `compiler/bytecode/src/compiler.rs`

### 2) VM

Crate:
- `compiler/vm` (`moon_vm`)

Archivos:
- `compiler/vm/src/vm.rs`
- `compiler/vm/src/error.rs`

### 3) Runtime compartido

La VM usa el mismo runtime que el interpreter:
- `compiler/runtime` (`moon_runtime`)

Esto permite:
- mismo `Value`
- mismo heap + GC

## Modelo: stack-based VM con entorno por scopes

Esta VM es deliberadamente simple:
- operand stack: `Vec<Value>`
- variables: HashMaps por scope (igual que interpreter)
- frames: stack de `Frame` con `ip` (instruction pointer) y scopes

Esto no es la VM mas rapida posible, pero es excelente para aprender y para validar semantica.

## Instrucciones (Instr) + debug info (Span)

Archivo:
- `compiler/bytecode/src/instr.rs`

En Moon, una instruccion de bytecode no es solo "que hacer", tambien incluye **de donde viene** en el source.

Por eso `Instr` es una struct:
- `kind: InstrKind` (la instruccion en si)
- `span: Span` (rango en bytes en el source que genero esa instruccion)

`InstrKind` es el enum con la lista de instrucciones.

Categorias (InstrKind):

1) Stack
- `Push(Value)`
- `Pop`

2) Scopes
- `PushScope`
- `PopScope`

3) Variables
- `LoadVar(name)`
- `DefineVar(name)`   // pop value, define en scope actual o globals
- `SetVar(name)`      // pop value, asigna variable existente

4) Ops
- `Neg`, `Not`
- `Add/Sub/Mul/Div/Mod`
- `Eq/Ne/Lt/Le/Gt/Ge`

5) Control flow
- `Jump(ip)`
- `JumpIfFalse(ip)`   // mira bool en top-of-stack, no lo poppea
- `JumpIfTrue(ip)`

6) Calls
- `Call(FuncId, argc)`
- `Return`

7) Heap/aggregates
- `MakeArray(n)`
- `MakeObject(keys)`
- `IndexGet`
- `IndexSet`

## Module y Function

Archivo:
- `compiler/bytecode/src/module.rs`

`Module` contiene:
- `functions: Vec<Function>`
- `by_name: HashMap<String, FuncId>`
- `main: FuncId`

`Function` contiene:
- `name`
- `params`
- `code: Vec<Instr>`

Nota:
- `gc` es un builtin y se inserta como Function en el Module.

## Compiler: AST -> bytecode

Archivo:
- `compiler/bytecode/src/compiler.rs`

Estrategia:

1) Reservar `main` en el modulo.
2) Insertar builtins (por ahora `gc`).
3) Pre-colectar ids de funciones:
   - asi resolvemos calls aunque la funcion se defina despues
4) Compilar cada funcion a su propio `Vec<Instr>`.
5) Compilar `main`:
   - statements en orden
   - tail expr (o `Unit`)
   - `Return`

### Patching de jumps

En bytecode no sabemos el `ip` final de un `if` hasta compilar ramas.
Solucion:
- emitir `JumpIfFalse(placeholder)`
- compilar then
- emitir `Jump(placeholder)`
- patch el primer jump al inicio del else
- compilar else
- patch el segundo jump al final

Esto esta en `patch_jump(...)`.

### Short-circuit de && y ||

El bytecode preserva semantica de corto circuito:

- `a && b`
  - evalua `a`
  - si es false, salta al final dejando `a` en el stack como resultado
  - si es true, lo poppea y evalua `b`

- `a || b`
  - evalua `a`
  - si es true, salta al final dejando `a` en el stack como resultado
  - si es false, lo poppea y evalua `b`

## VM: ejecucion

Archivo:
- `compiler/vm/src/vm.rs`

### Frame

Cada call crea un Frame:
- `func: FuncId`
- `ip`
- `stack_base` (donde truncar el stack al retornar)
- `scopes: Vec<HashMap<String, Value>>`

### Globals vs locals (detalle importante)

Regla que queremos (igual que interpreter):
- top-level `let` define globals
- las funciones ven globals, pero no ven scopes del caller

Implementacion:
- el frame de `main` empieza con `scopes = []` (vacio)
  - por eso `DefineVar` en main va a globals
- al llamar funcion, se crea frame con `scopes = [params_scope]`

### Builtin gc()

La VM intercepta calls a una Function cuyo `name == "gc"`:
- junta roots (globals + scopes de frames + operand stack)
- corre `heap.collect_garbage(roots)`
- empuja `Unit` como retorno

## Disassembler: `moon disasm`

Ahora que el bytecode incluye `span` por instruccion, podemos imprimir el modulo y ver "que genero" el compiler.

Comando:
- `moon disasm <file>`

Pipeline:
- load -> lex -> parse -> typecheck -> compile(bytecode) -> print

El output muestra por funcion:
- `ip` (instruction pointer)
- `InstrKind` (la instruccion)
- `@line:col [start..end]` (origen aproximado en el source)

Archivo:
- `src/main.rs` (cmd `disasm`)

## Tests: VM vs interpreter

Archivo:
- `compiler/vm/tests/vm.rs`

Estos tests validan:
- que la VM respeta semantica del lenguaje
- que el bytecode compiler esta generando instrucciones correctas

Un buen criterio a futuro:
- cualquier feature nueva deberia tener test en interpreter y VM

## Mini ejercicios

1) Mejora `moon disasm` para imprimir un snippet por instruccion usando `Source::render_span`.
2) Agrega "stacktrace" basico en `VmError` (lista de funciones activas).
3) Cambia variables de HashMap a slots (indices) para performance.

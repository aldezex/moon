# 12 - Proximos pasos (roadmap vivo)

Este documento es un roadmap tecnico.
No es una lista fija: cada feature nueva debe mover items de "pendiente" a "implementado".

Formato recomendado por item:
- motivacion
- decision de diseno
- plan incremental
- archivos tocados
- tests agregados

## 0) Checklist (estado actual)

Implementado y con tests:
- Bloques + scopes + tail expression
- `if/else` como expresion
- `return expr?;` como statement (modelado con `Type::Never`)
- Typechecking estricto
- Runtime: heap + GC mark/sweep
- Bytecode + VM con spans por instruccion + errores con spans
- `moon disasm`
- LSP (diagnostics/hover/definition/completion basico)

Funciones y closures:
- funciones top-level (`Stmt::Fn`) + call-before-definition
- funciones como valores (`Value::Function(name)`)
- funciones anonimas (`Expr::Fn`) como expresion
- closures con captura lexical:
  - runtime: `Value::Closure(GcRef)` + `HeapObjectKind::Closure`
  - bytecode: `MakeClosure` + `CallValue`
  - VM frames con `closure: Option<GcRef)`

## 1) Tipos de funcion en la sintaxis (alta prioridad)

### Problema

Hoy el typechecker maneja `Type::Function`, pero el source NO puede escribir:
- el tipo de una variable funcion
- el tipo de retorno de una funcion que retorna otra funcion

Eso bloquea:
- `fn make_adder(x: Int) -> (Int)->Int { ... }`

### Opcion A (recomendada)

Agregar sintaxis de function type:
- `(T1, T2) -> R`

En `TypeExpr`:
- `TypeExpr::Function { params: Vec<TypeExpr>, ret: Box<TypeExpr>, span }`

Parser:
- necesita desambiguar `(` de tipos vs expresiones
- como los tipos solo se parsean en contexto de tipos, es manejable

Typechecker:
- lowering directo a `Type::Function`

Tests:
- anotaciones de let con function type
- funciones que retornan funciones

## 2) Closures "serias": upvalues por referencia

### Problema

Hoy capturamos por valor (snapshot). Eso es util, pero no replica JS/TS:
- mutaciones del outer scope despues de crear la closure no se ven

### Objetivo

Soportar captura por referencia (upvalues):
- una variable capturada se representa como un "cell" heap-alloc
- closures comparten ese cell

### Plan incremental

Paso A: cells en runtime
- agregar `HeapObjectKind::Cell(Value)` o similar
- agregar `Value::Cell(GcRef)` o representar cell solo en heap

Paso B: lifting de locals capturados
- cuando una var es capturada, su storage pasa a ser heap cell
- loads/sets se vuelven deref

Paso C: bytecode
- instrucciones dedicadas:
  - `UpvalueGet(slot)`
  - `UpvalueSet(slot)`
  - o `LoadCell/StoreCell`

Paso D: typechecker
- semantica de mutabilidad (si agregamos `let` inmutable vs `mut`)

Esta es una feature grande; requiere diseno cuidadoso.

## 3) Loops + break/continue

### Objetivo

Agregar:
- `while cond { ... }`
- `loop { ... }`
- `break` / `continue`

### Patron de implementacion

- en interpreter:
  - extender el enum de control flow:
    - `Return`, `Break`, `Continue`
- en bytecode:
  - patching de jumps
  - stack de labels por loop

### Tests

- loops simples
- break/continue
- interaccion con `return`

## 4) Performance: variables por slots

Hoy interpreter y VM usan HashMaps por scope:
- `LoadVar/SetVar` hacen hashing
- scopes allocan HashMaps

Upgrade clasico:
- resolver variables a slots en compilacion
- frame guarda `Vec<Value>`
- instrucciones:
  - `LoadLocal(slot)` / `StoreLocal(slot)`

Esto tambien prepara:
- upvalues (closures por referencia)

## 5) Tipos mas expresivos (records)

Hoy:
- `Object<T>` es homogeneo

Para escribir:
- `#{ a: 1, b: "x" }`

Opcion incremental:
- agregar `Record{a:Int,b:String}` como tipo estructural
- mantener `Object<T>` como map dinamico

Parser:
- sintaxis de tipo record
Typechecker:
- reglas de acceso (index con string literal)

## 6) Modulos/imports

Sin modulos, todo vive en un archivo.
Plan MVP:
- `import "path"`
- loader con cache (un modulo se evalua una vez)

## 7) Runtime: auto-GC + builtins

- auto-GC por heuristica
- builtins tipados:
  - `print`, `dbg`, `len`, `push`

## 8) Calidad

- golden tests
- fuzzing del lexer/parser
- differential tests interpreter vs VM

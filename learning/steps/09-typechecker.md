# 09 - Typechecker estricto (moon check)

## Objetivo

El typechecker de Moon valida programas antes de ejecutar:
- variables deben existir donde se usan
- operadores solo se aplican a tipos compatibles
- `if` debe tener condicion Bool y ramas compatibles
- llamadas a funciones deben matchear aridad y tipos
- arrays/objects deben usarse correctamente (indexing, assignment)

Esto transforma muchos "runtime bugs" en errores claros y tempranos.

Crate:
- `compiler/typechecker` (`moon_typechecker`)

Archivos:
- `compiler/typechecker/src/lib.rs` (algoritmo principal)
- `compiler/typechecker/src/env.rs` (TypeEnv)
- `compiler/typechecker/src/types.rs` (Type)
- `compiler/typechecker/src/error.rs` (TypeError)

## TypeExpr (en el AST) vs Type (del typechecker)

El parser produce anotaciones como `TypeExpr` (sintaxis):
- `Int`
- `Array<Int>`
- `Object<String>`

El typechecker hace "lowering" a su enum `Type`:
- `Type::Int`
- `Type::Array(Box<Type>)`
- etc.

Esto esta en `lower_type(...)` (`compiler/typechecker/src/lib.rs`).

## Tipos del MVP

Archivo:
- `compiler/typechecker/src/types.rs`

Soportamos:
- `Int`
- `Bool`
- `String`
- `Unit`
- `Array<T>`
- `Object<T>` (map String -> T)

Notas:
- `Object<T>` es un "map" con valores homogeneos (todos del mismo tipo).
  - Esto es una simplificacion MVP; objetos estructurales estilo TS vendran despues.

## Errores (TypeError)

Archivo:
- `compiler/typechecker/src/error.rs`

`TypeError` tiene:
- `message`
- `span`

La CLI lo convierte a diagnostico via `Source::render_span`.

## Entorno de tipos (TypeEnv)

Archivo:
- `compiler/typechecker/src/env.rs`

El typechecker necesita saber:
- que variables existen en cada scope
- que funciones existen (firmas)

`TypeEnv` mantiene:
- `globals: HashMap<String, Type>`
- `scopes: Vec<HashMap<String, Type>>`
- `funcs: HashMap<String, FuncSig>`

Busqueda de variables:
- scopes (inner -> outer) y luego globals

## Algoritmo principal: dos pasadas

Archivo:
- `compiler/typechecker/src/lib.rs`

### Paso 0: builtins

Antes de mirar el programa, registramos builtins:
- `gc(): Unit`

Esto permite que el lenguaje use GC como herramienta sin definir una funcion.

### Pass 1: recolectar firmas de funciones

Recorremos `program.stmts` y para cada `Stmt::Fn`:
- parseamos tipos de params y return
- guardamos `FuncSig { params, ret }` en `env.funcs`

Beneficio:
- `f(1); fn f(x: Int) -> Int { x }` es valido
- recursion es posible

### Pass 2: typecheck de statements en orden

Regla de "estricto" para variables:
- una variable debe estar declarada antes de usarse

Se procesan:

1) `let name (: T)? = expr;`
   - typecheck de `expr` -> `expr_ty`
   - si hay anotacion `T`, se valida `T == expr_ty`
   - se define `name` en el scope actual

2) `target = expr;` (assignment statement)
   - se typecheckea RHS
   - el target debe ser un lvalue:
     - `x` (misma type)
     - `arr[i]` (i Int, RHS == T de Array<T>)
     - `obj["k"]` (k String, RHS == T de Object<T>)

3) `expr;`
   - se typecheckea y se descarta

4) `fn ...`
   - el cuerpo se typecheckea en un scope nuevo con los parametros
   - el tipo del cuerpo debe igualar el return type

### Tipo del programa

El tipo final es:
- el tipo del `program.tail` si existe
- o `Unit` si no

## Reglas de expresiones (resumen)

Primitivas:
- literales -> tipos obvios
- ident -> busca en env

Blocks:
- empuja scope
- typecheckea statements
- tail expr -> tipo del bloque
- si no hay tail -> Unit

If:
- `cond` debe ser Bool
- `then` y `else` deben tener el mismo tipo
- el tipo del `if` es el tipo comun

Array literal:
- `[e1, e2, ...]` requiere que todos tengan mismo tipo
- `[]` vacio:
  - no se puede inferir
  - se permite solo con anotacion contextual: `let a: Array<Int> = [];`

Object literal:
- `#{ k: v, ... }` requiere que todos los `v` tengan el mismo tipo
- `#{}` vacio:
  - no se puede inferir
  - se permite con anotacion: `let o: Object<Int> = #{};`

Indexing:
- `Array<T>[Int] -> T`
- `Object<T>[String] -> T`

Ops:
- `Int (+-*/%) Int -> Int`
- `String + String -> String`
- comparaciones `Int < Int -> Bool`, etc.
- `==`/`!=` requieren tipos iguales (-> Bool)
- `&&`/`||` requieren Bool (-> Bool)

Calls:
- por ahora solo por nombre (callee debe ser Ident)
- debe existir firma
- args deben matchear en tipo y cantidad

## Mini ejercicios

1) Agrega un tipo `Null` y reglas para `Object<Null>`.
2) Agrega un builtin `print(x: String) -> Unit` y haz que sea reconocido como builtin.
3) Implementa "contextual typing" mas general (pasar expected type a `check_expr`) para inferir `[]` aun sin let annotation (ej: como arg de funcion).

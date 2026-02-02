# 09 - Typechecker estricto (semantica estatica)

Moon ejecuta solo despues de typecheck.
Este capitulo describe:
- el sistema de tipos MVP
- el algoritmo real en el repo
- como modelamos `return` con `Never`
- como tipamos funciones como valores, funciones anonimas y closures

Crate:
- `compiler/typechecker` (`moon_typechecker`)

Archivos:
- `compiler/typechecker/src/lib.rs` (algoritmo)
- `compiler/typechecker/src/types.rs` (Type)
- `compiler/typechecker/src/env.rs` (TypeEnv)
- `compiler/typechecker/src/error.rs` (TypeError)

## 0) Contrato del typechecker

Input:
- AST (`moon_core::ast::Program`)

Output:
- `Type` del programa (tipo del tail expression o `Unit`)
- o `TypeError { message, span }`

Invariante:
- si typecheck pasa, interpreter/VM pueden asumir:
  - ops se aplican a tipos validos
  - indices correctos
  - llamadas con aridad/tipos correctos

## 1) TypeExpr (parser) vs Type (typechecker)

- `TypeExpr` vive en el AST (sintaxis)
- `Type` vive en el typechecker (semantica)

Lowering:
- `lower_type(TypeExpr) -> Type`

Limitacion MVP:
- no hay sintaxis para function types en `TypeExpr`
- pero `Type` si incluye `Type::Function`

## 2) Tipos soportados (Type)

Archivo:
- `compiler/typechecker/src/types.rs`

```
Int | Bool | String | Unit
Array<T>
Object<T>
Function { params: Vec<Type>, ret: Type }
Never
```

`Never` es especial:
- significa "esta expresion no produce valor porque no continua" (diverge)
- ejemplo: un bloque que ejecuta `return`

## 3) Environment de tipos (TypeEnv)

Archivo:
- `compiler/typechecker/src/env.rs`

`TypeEnv` mantiene:
- `globals: HashMap<String, Type>`
- `scopes: Vec<HashMap<String, Type>>`
- `funcs: HashMap<String, FuncSig>`

Regla de lookup:
- scopes (inner -> outer) y luego globals

## 4) Algoritmo principal (check_program)

Archivo:
- `compiler/typechecker/src/lib.rs`

### 4.1 Builtins

Antes de mirar el programa:
- registramos `gc(): Unit`

Esto hace que `gc()` typecheckee sin declaracion.

### 4.2 Dos pasadas para `Stmt::Fn`

Motivo:
- call-before-definition
- recursion

Pass 1:
- recorre `program.stmts`
- por cada `Stmt::Fn`:
  - lower param types
  - lower return type
  - guarda `FuncSig` en `env.funcs`

Pass 2:
- recorre statements en orden
- regla "strict": una variable debe estar definida antes de usarse

### 4.3 Contexto de return

El typechecker pasa un contexto `current_ret: Option<&Type>`.
- fuera de funciones: `None`
- dentro de una funcion/closure: `Some(expected_ret)`

`Stmt::Return`:
- error si `current_ret` es `None`
- si hay expr, su tipo debe ser compatible con el return type esperado

## 5) Reglas de typing (resumen formal)

Puedes pensarlo como un judgement:

`Gamma |- expr : T`

Donde `Gamma` es el environment (scopes/globals/funcs).

### 5.1 Ident

En Moon, un identificador puede referirse a:
- una variable
- un item de funcion top-level

Implementacion:
- `Expr::Ident`:
  - si `env.get_var(name)` existe => ese tipo
  - si no, si `env.get_fn(name)` existe => `Type::Function { ... }`
  - si no => error

Eso habilita:

```moon
fn add1(x: Int) -> Int { x + 1 }
let f = add1;
f(41)
```

### 5.2 Call

`Expr::Call { callee, args }`:
- se typecheckea `callee`
- debe producir `Type::Function { params, ret }`
- `args.len()` debe igualar `params.len()`
- cada arg debe ser compatible con su param
- tipo del call es `ret`

Nota:
- ya no restringimos call a `Ident`.
- cualquier expresion de tipo funcion es callable.

### 5.3 Funciones anonimas (Expr::Fn)

`Expr::Fn { params, ret_ty, body }`:
- lower param types
- lower return type
- empuja scope con params
- typecheckea `body` con `current_ret = Some(&ret)`
- valida que el tipo del body sea compatible con `ret`
- resultado es `Type::Function { params, ret }`

Punto clave:
- a diferencia de `Stmt::Fn`, aqui NO borramos scopes externos.
- eso habilita captura lexical (closures).

### 5.4 `Never` y control flow

Para modelar `return`, necesitamos diverging control flow.

Reglas practicas:
- `Stmt::Return` hace que el statement "diverge".
- un `Block`:
  - si encuentra un statement divergente, el tipo del bloque es `Never`.
- un `if`:
  - si ramas tienen mismo tipo => ok
  - si una rama es `Never`, el `if` toma el tipo de la otra

Compatibilidad:
- `compatible(expected, got)` es true si:
  - `expected == got` o `got == Never`

Eso permite:

```moon
fn f(x: Int) -> Int {
  if x > 0 { return x; } else { };
  x + 1
}
```

## 6) Arrays y Objects (tipos homogeneos)

`Array<T>`:
- literal `[e1, e2, ...]` requiere todos del mismo tipo
- `[]` no se puede inferir sin anotacion contextual

`Object<T>`:
- literal `#{ k: v, ... }` requiere todos los `v` del mismo tipo
- `#{}` no se puede inferir sin anotacion

Indexing:
- `Array<T>[Int] -> T`
- `Object<T>[String] -> T`

## 7) Practica: lee el typechecker con closures

Ejemplo:

```moon
let c = { let x = 0; fn() -> Int { x = x + 1; x } };
c() + c()
```

Sigue estos puntos en `compiler/typechecker/src/lib.rs`:
- `Expr::Fn` produce `Type::Function`.
- Dentro del body, `Ident("x")` resuelve a la var del scope externo.
- `Stmt::Return` (si existe) produce `Never`.

## 8) Ejercicios

1) Agrega sintaxis para function types en `TypeExpr`.
2) Implementa records estructurales (no homogeneos) como tipo adicional.
3) Agrega un builtin `print(x: String) -> Unit` y tipalo.

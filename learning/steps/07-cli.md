# 07 - CLI (moon)

El CLI es la interfaz de usuario del toolchain.
No es "solo un runner": es un punto de integracion de todas las capas.

Archivo:
- `src/main.rs`

## 0) Filosofia

- Cada comando corre el mismo frontend:
  - load source
  - lex
  - parse
  - typecheck

- Luego, segun comando, elige backend:
  - interpreter (`moon run`)
  - bytecode+VM (`moon vm`)

Esto garantiza:
- consistencia entre backends
- errores antes de ejecutar

## 1) Comandos

### 1.1 `moon run <file>`
Ejecuta con el interpreter.
Pipeline:
1) `Source::from_path` (o stdin si `<file> == "-"`)
2) `moon_core::lexer::lex`
3) `moon_core::parser::parse`
4) `moon_typechecker::check_program`
5) `moon_interpreter::eval_program`

Output:
- imprime el valor final si no es `Unit`

Errores:
- lex/parse/type/runtime se imprimen con `Source::render_span`

### 1.2 `moon vm <file>`
Ejecuta con bytecode+VM.
Pipeline:
1) lex/parse/typecheck (igual)
2) `moon_bytecode::compile`
3) `moon_vm::run`

Output y errores:
- igual que `run`, pero errores vienen de la VM

### 1.3 `moon check <file>`
Solo typecheck.
- imprime `ok: <Type>`

### 1.4 `moon ast <file>`
Imprime el AST (debug `#?`).
Util para:
- validar parseo
- entender spans

### 1.5 `moon disasm <file>`
Imprime el bytecode del modulo.

Pipeline:
1) lex/parse/typecheck
2) compile a `Module`
3) imprime funciones + instrucciones

Nota:
- cada instruccion incluye el `Span` que la genero
- se imprime `@line:col [start..end]` usando `Source::line_col`

Esto es clave para tooling:
- cuando la VM falla, el span te lleva a la expresion origen

## 2) Implementacion (donde mirar)

`src/main.rs` implementa:
- parse manual de args (MVP)
- un handler por comando:
  - `cmd_run`, `cmd_vm`, `cmd_check`, `cmd_ast`, `cmd_disasm`

Cada handler:
- retorna `Result<(), i32>` para manejar exit codes

## 3) Practica: debugging con `disasm`

Ejemplo:

```moon
let c = { let x = 0; fn() -> Int { x = x + 1; x } };
c() + c()
```

- corre `moon disasm` y busca:
  - `MakeClosure` (creacion de closure)
  - `CallValue` (llamada indirecta)
  - `LoadVar`/`SetVar` de `x`

Esto te muestra como el source se transforma en instrucciones.

## 4) Ejercicios

1) Agrega un comando `tokens` que imprima tokens+spans.
2) Agrega un comando `type-at <offset>` (solo para jugar) que use `check_program_with_spans`.
3) Cambia el CLI para aceptar `--backend=vm|interp` y unifica `run`.

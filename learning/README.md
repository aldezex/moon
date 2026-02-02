# Learning Moon

Esta carpeta es un recurso de estudio: documenta, paso a paso, como construimos **Moon**.

Moon es un lenguaje de scripting con una "vibra Rust":
- expresion-oriented (bloques con tail expression)
- tipado estricto (sin `any` implicito)
- tooling desde el dia 1 (spans, errores con ubicacion, LSP)

El objetivo no es solo "que funcione", sino construir una base realista:
- frontend (lexer -> parser -> AST)
- typechecker estricto
- interpreter (tree-walk) para iterar rapido
- bytecode + VM para performance/tooling
- runtime con heap + GC mark/sweep

Regla de lectura recomendada:
1) lee el capitulo
2) abre los paths citados
3) corre `cargo test --workspace`
4) modifica algo pequeno y agrega un test

Requisitos:
- Rust estable (cargo)

## Indice (paso a paso)

0) Vision y principios
   - `learning/steps/00-vision.md`

1) Layout del repo y crates
   - `learning/steps/01-repo-layout.md`

2) Spans y Source: diagnosticos con ubicacion
   - `learning/steps/02-spans-and-source.md`

3) AST: el modelo del lenguaje
   - `learning/steps/03-ast.md`

4) Lexer: texto -> tokens
   - `learning/steps/04-lexer.md`

5) Parser: tokens -> AST (Pratt + statements)
   - `learning/steps/05-parser.md`

6) Interpreter: ejecucion directa sobre AST
   - `learning/steps/06-interpreter.md`

7) CLI: comandos para inspeccionar y ejecutar
   - `learning/steps/07-cli.md`

8) Tests: estrategia y ejemplos
   - `learning/steps/08-tests-and-examples.md`

9) Typechecker: reglas, errores, `Never`, funciones y closures
   - `learning/steps/09-typechecker.md`

10) Runtime: heap + GC + valores compartidos (interpreter/VM)
   - `learning/steps/10-runtime-and-gc.md`

11) Bytecode + VM: AST -> bytecode -> ejecucion
   - `learning/steps/11-bytecode-and-vm.md`

12) Roadmap vivo (next steps)
   - `learning/steps/12-next-steps.md`

13) Language Server (LSP): diagnosticos/hover/definition
   - `learning/steps/13-language-server.md`

14) Closures y funciones anonimas: diseno + implementacion
   - `learning/steps/14-closures.md`

## Estado actual del lenguaje (snapshot)

Sintaxis y semantica (MVP+):
- Variables:
  - `let name = expr;` (anotacion opcional: `let x: Int = 1;`)
  - assignment como statement: `name = expr;` y `target[index] = expr;`
- Control flow:
  - bloques `{ ... }` con scopes
  - tail expression: la ultima expresion sin `;` es el valor del bloque
  - `if cond { ... } else { ... }` es expresion
  - `return expr?;` dentro de funciones/closures (`return;` devuelve `Unit`)
- Funciones:
  - items top-level: `fn name(params...) -> Type { ... }`
  - funciones como valores: `let f = add1; f(41)`
  - funciones anonimas: `let f = fn(x: Int) -> Int { x + 1 };`
  - closures (capturan variables locales): `let f = { let x = 10; fn(y: Int) -> Int { x + y } };`
- Literales:
  - `Int`, `Bool`, `String`, `Unit` (`()` al imprimir)
  - arrays: `[a, b, c]`
  - objects (map literal): `#{ key: value, "key2": value2 }`
- Indexing:
  - `arr[0]` y `arr[0] = 1`
  - `obj["k"]` y `obj["k"] = v`
- Operadores:
  - `+ - * / %`, comparaciones, `== !=`, `&& ||`, `!`, unario `-expr`, parentesis
- Comentarios de linea:
  - `// ...`
- Builtin:
  - `gc()` fuerza un ciclo de GC (debug)

Tooling:
- spans en errores de lexer/parser/typechecker/runtime
- `moon disasm` imprime bytecode con spans
- `moon-lsp` (LSP) expone diagnostics/hover/definition/completion basico

## Comandos utiles

Ejecutar (interpreter):
- `cargo run -- run examples/hello.moon`

Typecheck:
- `cargo run -- check examples/hello.moon`

Ejecutar (VM):
- `cargo run -- vm examples/hello.moon`

Disassembler:
- `cargo run -- disasm examples/hello.moon`

LSP (stdio):
- `cargo run -p moon_lsp --bin moon-lsp`

Tests:
- `cargo test --workspace`

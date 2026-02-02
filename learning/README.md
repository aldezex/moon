# Learning Moon

Esta carpeta documenta, paso a paso, como estamos construyendo **Moon**: un lenguaje de scripting con "vibra Rust" (expresiones, tipado estricto, tooling) pero pensado para iterar rapido como JavaScript/TypeScript.

La idea es que puedas leer estos capitulos en orden y, en cada paso:
- entiendas el concepto (lexer/parser/AST/typechecker/VM/etc)
- veas como se traduce a codigo real en Rust (con paths a los archivos)
- puedas ejecutar el repo en ese punto (tests + CLI)

Si quieres aprender "con las manos", lo mas efectivo es:
1) leer un paso
2) abrir los archivos referenciados
3) correr `cargo test --workspace`
4) intentar una mini mejora (hay sugerencias/exercises dentro de los capitulos)

Requisitos:
- Rust estable (cargo)

## Indice (paso a paso)

0) Vision y principios
   - `learning/steps/00-vision.md`

1) Layout del repo y crates
   - `learning/steps/01-repo-layout.md`

2) Spans y diagnosticos (Source)
   - `learning/steps/02-spans-and-source.md`

3) AST (Abstract Syntax Tree)
   - `learning/steps/03-ast.md`

4) Lexer (tokenizer)
   - `learning/steps/04-lexer.md`

5) Parser (Pratt / precedencias)
   - `learning/steps/05-parser.md`

6) Interpreter (tree-walk)
   - `learning/steps/06-interpreter.md`

7) CLI (moon run / moon ast / moon check / moon vm / moon disasm)
   - `learning/steps/07-cli.md`

8) Tests y ejemplos
   - `learning/steps/08-tests-and-examples.md`

9) Typechecker estricto (moon check)
   - `learning/steps/09-typechecker.md`

10) Runtime y memoria (heap + GC mark/sweep)
   - `learning/steps/10-runtime-and-gc.md`

11) Bytecode + VM (AST -> bytecode -> VM)
   - `learning/steps/11-bytecode-and-vm.md`

12) Proximos pasos (roadmap vivo)
   - `learning/steps/12-next-steps.md`

13) Language Server (LSP)
   - `learning/steps/13-language-server.md`

## Estado actual del lenguaje (MVP)

Soportamos:
- `let name = expr;` (con anotacion opcional: `let x: Int = 1;`)
- Assignment como statement: `name = expr;` y `target[index] = expr;`
- Bloques `{ ... }` con scopes y **tail expression** (la ultima expresion sin `;` es el valor del bloque)
- `if cond { ... } else { ... }` como expresion
- `fn name(params...) -> Type { ... }` + llamadas `name(args...)` y llamadas indirectas (`let f = name; f(args...)`)
- `return expr?;` dentro de funciones (early exit; `return;` devuelve `Unit`)
- Literales:
  - `Int`, `Bool`, `String`
  - Arrays: `[a, b, c]`
  - Objects (map literal): `#{ key: value, "key2": value2 }`
- Indexing:
  - `arr[0]`
  - `obj["key"]`
- Expresiones: `+ - * / %`, comparaciones, `== !=`, `&& ||`, `!`, `-expr`, parentesis `(...)`
- Comentarios de linea `// ...`

Ejecutar:
- `cargo run -- run examples/hello.moon`
- `cargo run -- check examples/hello.moon`
- `cargo run -- vm examples/hello.moon`
- `cargo run -- disasm examples/hello.moon`
- `cargo run -p moon_lsp --bin moon-lsp`
- `cargo test --workspace`

# Learning Moon

Esta carpeta documenta, paso a paso, como estamos construyendo **Moon**: un lenguaje de scripting con "vibra Rust" (expresiones, tipos, tooling) pero pensado para iterar rapido como JavaScript/TypeScript.

La idea es que puedas leer estos capitulos en orden y, en cada paso, abrir el codigo referenciado para ver la implementacion real.

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

7) CLI (moon run / moon ast / moon check)
   - `learning/steps/07-cli.md`

8) Tests y ejemplos
   - `learning/steps/08-tests-and-examples.md`

9) Typechecker estricto (moon check)
   - `learning/steps/09-typechecker.md`

10) Runtime y memoria (GC) (diseño)
   - `learning/steps/10-runtime-and-gc.md`

11) Bytecode + VM (diseño)
   - `learning/steps/11-bytecode-and-vm.md`

12) Proximos pasos (roadmap vivo)
   - `learning/steps/12-next-steps.md`

## Estado actual del lenguaje (MVP)

Soportamos:
- `let name = expr;` (con anotacion opcional: `let x: Int = 1;`)
- Bloques `{ ... }` con scopes y **tail expression** (la ultima expresion sin `;` es el valor del bloque)
- `if cond { ... } else { ... }` como expresion
- `fn name(params...) -> Type { ... }` + llamadas `name(args...)`
- Literales: `Int`, `Bool`, `String`
- Expresiones: `+ - * / %`, comparaciones, `== !=`, `&& ||`, `!`, `-expr`, parentesis `(...)`
- Comentarios de linea `// ...`

Ejecutar:
- `cargo run -- run examples/hello.moon`
- `cargo run -- check examples/hello.moon`
- `cargo test --workspace`

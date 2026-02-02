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

7) CLI (moon run / moon ast)
   - `learning/steps/07-cli.md`

8) Tests y ejemplos
   - `learning/steps/08-tests-and-examples.md`

9) Proximos pasos (typechecker estricto, GC, VM)
   - `learning/steps/09-next-steps.md`

## Estado actual del lenguaje (MVP)

Soportamos:
- `let name = expr;`
- Literales: `int`, `bool`, `string`
- Expresiones: `+ - * / %`, comparaciones, `== !=`, `&& ||`, `!`, `-expr`, y parentesis `(...)`
- Comentarios de linea `// ...`

Ejecutar:
- `cargo run -- run examples/hello.moon`
- `cargo test --workspace`

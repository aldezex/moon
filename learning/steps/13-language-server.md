# 13 - Language Server (LSP)

## Que es un language server (y por que lo queremos)

Hasta ahora, Moon corre bien por CLI:
- `moon check` para typechecking
- `moon run` para ejecutar en interpreter
- `moon vm` para ejecutar en la VM

Eso esta perfecto para aprender el pipeline. Pero en la practica, el 80% del tiempo escribimos codigo dentro de un editor.
Un **Language Server** (LSP) sirve para que el editor tenga:
- diagnosticos en vivo (errores de lex/parse/type) mientras escribes
- hover (ver el tipo de una expresion al pasar el mouse)
- go-to-definition (ir a la definicion de una funcion/let)
- autocompletado (keywords, tipos, builtins)

La magia es que no hace falta escribir plugins distintos para cada editor.
Con LSP, escribes **un solo servidor** que habla un protocolo estandar, y cualquier editor con soporte LSP puede conectarse.

## Donde vive en el repo

Crate:
- `compiler/lsp` (package `moon_lsp`)

Binario:
- `moon-lsp`

Archivo principal:
- `compiler/lsp/src/main.rs`

Este crate depende de:
- `moon_core` (lexer/parser/spans)
- `moon_typechecker` (reglas de tipos)

Y usa una libreria para implementar LSP en Rust:
- `tower-lsp`

## Como se ejecuta

El language server se comunica por **stdin/stdout** (JSON-RPC).
Eso significa que normalmente no lo ejecutas "a mano" sino que tu editor lo lanza.

Para compilar/ejecutar:

- Dev (lento, pero rapido de iterar):
  - `cargo run -p moon_lsp --bin moon-lsp`

- Release (recomendado para usarlo desde un editor):
  - `cargo build -p moon_lsp --bin moon-lsp --release`
  - binario resultante: `target/release/moon-lsp`

## Arquitectura del servidor

El servidor es un proceso con estado:
- recibe notificaciones del cliente (didOpen/didChange/didSave)
- responde requests (hover/definition/completion)
- publica diagnosticos (publishDiagnostics)

En Moon, la estructura clave es:
- `Backend`: implementa el trait `LanguageServer`
- `documents`: cache simple en memoria `HashMap<Url, Document>` con el texto actual por archivo

### Document sync: FULL vs incremental

LSP permite dos modelos:
- incremental (te manda diffs)
- full (te manda el texto completo en cada cambio)

En el MVP elegimos **FULL**:
- implementacion mas simple
- suficiente para archivos chicos/medianos

Esto se ve en `initialize`:
- `TextDocumentSyncKind::FULL`

Y en `did_change`, donde tomamos el ultimo `content_changes` y lo tratamos como texto completo.

## Pipeline reutilizado: lex -> parse -> typecheck

La idea mas importante: el LSP no inventa un pipeline nuevo.
Reutiliza exactamente el mismo que la CLI:

1) `lex(text)`
2) `parse(tokens)`
3) `check_program(program)`

Si cualquiera falla, generamos un `Diagnostic`.

Archivo:
- `compiler/lsp/src/main.rs`

Funcion:
- `diagnostics_for(uri, text) -> Vec<Diagnostic>`

Nota MVP:
- hoy devolvemos 0 o 1 diagnostico (porque lex/parse/typechecker retornan un solo error).
- a futuro, parser/typechecker podrian acumular multiples errores.

## Spans vs Range: el problema de los offsets

Moon usa `Span { start, end }` en **bytes**.
LSP usa `Range { start: Position, end: Position }`:
- `Position.line`: 0-based
- `Position.character`: columna en **UTF-16 code units** (no bytes)

Eso significa que necesitamos conversiones.

### De Span -> Range (para diagnosticos)

Implementado como:
- `range_from_span_utf16(text, span)`

Por dentro:
- `position_from_offset_utf16(text, span.start)`
- `position_from_offset_utf16(text, span.end)`

Y listo.

### De Position -> offset (para hover/definition)

Para hover y go-to-definition, el cliente nos manda un `Position`.
Lo convertimos a byte offset con:
- `offset_from_position_utf16(text, position)`

Importante:
- si el cliente apunta al medio de un caracter que en UTF-16 ocupa 2 unidades (emoji, algunos simbolos), nuestro mapeo cae en el final del caracter. Eso evita offsets invalidos.

## Hover: mostrar el tipo en el cursor

Para hover, necesitamos saber el tipo de *cada expresion*.
El typechecker MVP originalmente solo devolvia el tipo del programa.

Solucion:
- agregamos una API nueva en `moon_typechecker`:
  - `check_program_with_spans(program) -> CheckInfo`

`CheckInfo` contiene:
- `ty`: tipo del programa
- `expr_types: Vec<(Span, Type)>`: tipos por span de expresion

El LSP hace:
1) calcula `offset` desde `Position`
2) busca en `expr_types` el span mas pequeno que contenga ese offset
3) muestra el tipo en hover

Esto es suficiente para un MVP y sirve para validar que nuestro typechecker esta "pegado" al source.

Limitaciones:
- los spans pueden solaparse (ej: una expr grande contiene expr chicas). Por eso elegimos el span mas corto.
- `let`/`fn` hoy no guardan span del nombre en el AST (solo span del statement completo). Hover en nombres definidos puede caer en spans mas grandes.

## Go-to-definition: definiciones top-level

En Moon MVP:
- `fn` solo existe en top-level

Eso permite un go-to-definition simple:
1) leer el ident bajo el cursor (scan simple ASCII: `[a-zA-Z0-9_]`)
2) parsear el programa
3) construir un map de defs top-level:
   - `fn name` -> span del statement
   - `let name` -> span del statement
4) si existe, devolver `Location { uri, range }`

Limitaciones:
- el rango apunta al statement completo (por ahora), no exactamente al nombre.
- no resolvemos scopes (un `let` dentro de un block podria shadowear, pero este MVP solo indexa defs top-level).

## Completion: lista estatica (MVP)

El autocompletado MVP es intencionalmente simple:
- keywords: `let`, `fn`, `if`, `else`, `true`, `false`
- tipos: `Int`, `Bool`, `String`, `Unit`, `Array`, `Object`
- builtins: `gc`

No es context-aware (todavia).
Pero ya da una UX basica y nos deja el hook para mejorar.

## Mini ejercicios (siguientes mejoras)

1) Diagnosticos multiples
   - cambia parser/typechecker para acumular errores
   - publica una lista de `Diagnostic` en vez de 1

2) Hover con mas informacion
   - mostrar tambien: "kind" (var/function), y si es call, mostrar firma

3) Go-to-definition real (scopes)
   - resolver bindings por scope (igual que interpreter/typechecker)
   - para un `Ident`, devolver la definicion mas cercana

4) `moon disasm` + debug info (conectar con VM)
   - cuando existan spans por instruccion, el LSP podria hacer:
     - mostrar span en runtime errors
     - stepping basico

5) Completion contextual
   - si estas despues de `let x:`, sugerir tipos
   - si estas en `#{ ... }`, sugerir keys conocidas


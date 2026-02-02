use std::env;
use std::path::PathBuf;

use moon_bytecode::compile;
use moon_core::lexer::lex;
use moon_core::parser::parse;
use moon_core::source::Source;
use moon_interpreter::{eval_program, Value};
use moon_typechecker::check_program;
use moon_vm::run as run_vm;

fn main() {
    let mut args = env::args().skip(1);
    let cmd = args.next();

    match cmd.as_deref() {
        Some("run") => {
            let path = match args.next() {
                Some(p) => p,
                None => {
                    eprintln!("missing <file> for `moon run`.\n");
                    print_help();
                    std::process::exit(2);
                }
            };
            if let Err(code) = cmd_run(path) {
                std::process::exit(code);
            }
        }
        Some("ast") => {
            let path = match args.next() {
                Some(p) => p,
                None => {
                    eprintln!("missing <file> for `moon ast`.\n");
                    print_help();
                    std::process::exit(2);
                }
            };
            if let Err(code) = cmd_ast(path) {
                std::process::exit(code);
            }
        }
        Some("check") => {
            let path = match args.next() {
                Some(p) => p,
                None => {
                    eprintln!("missing <file> for `moon check`.\n");
                    print_help();
                    std::process::exit(2);
                }
            };
            if let Err(code) = cmd_check(path) {
                std::process::exit(code);
            }
        }
        Some("vm") => {
            let path = match args.next() {
                Some(p) => p,
                None => {
                    eprintln!("missing <file> for `moon vm`.\n");
                    print_help();
                    std::process::exit(2);
                }
            };
            if let Err(code) = cmd_vm(path) {
                std::process::exit(code);
            }
        }
        Some("disasm") => {
            let path = match args.next() {
                Some(p) => p,
                None => {
                    eprintln!("missing <file> for `moon disasm`.\n");
                    print_help();
                    std::process::exit(2);
                }
            };
            if let Err(code) = cmd_disasm(path) {
                std::process::exit(code);
            }
        }
        Some("help") | Some("-h") | Some("--help") | None => {
            print_help();
        }
        Some(other) => {
            eprintln!("unknown command: {other}\n");
            print_help();
            std::process::exit(2);
        }
    }
}

fn cmd_run(path: String) -> Result<(), i32> {
    let source = load_source(&path).map_err(|e| {
        eprintln!("io error: {e}");
        1
    })?;

    let tokens = lex(&source.text).map_err(|e| {
        eprintln!(
            "{}",
            source.render_span(e.span, &format!("lex error: {}", e.message))
        );
        1
    })?;

    let program = parse(tokens).map_err(|e| {
        eprintln!(
            "{}",
            source.render_span(e.span, &format!("parse error: {}", e.message))
        );
        1
    })?;

    let _ = check_program(&program).map_err(|e| {
        eprintln!(
            "{}",
            source.render_span(e.span, &format!("type error: {}", e.message))
        );
        1
    })?;

    let value = eval_program(&program).map_err(|e| {
        eprintln!(
            "{}",
            source.render_span(e.span, &format!("runtime error: {}", e.message))
        );
        1
    })?;

    if value != Value::Unit {
        println!("{value}");
    }

    Ok(())
}

fn cmd_ast(path: String) -> Result<(), i32> {
    let source = load_source(&path).map_err(|e| {
        eprintln!("io error: {e}");
        1
    })?;

    let tokens = lex(&source.text).map_err(|e| {
        eprintln!(
            "{}",
            source.render_span(e.span, &format!("lex error: {}", e.message))
        );
        1
    })?;

    let program = parse(tokens).map_err(|e| {
        eprintln!(
            "{}",
            source.render_span(e.span, &format!("parse error: {}", e.message))
        );
        1
    })?;

    println!("{program:#?}");
    Ok(())
}

fn cmd_check(path: String) -> Result<(), i32> {
    let source = load_source(&path).map_err(|e| {
        eprintln!("io error: {e}");
        1
    })?;

    let tokens = lex(&source.text).map_err(|e| {
        eprintln!(
            "{}",
            source.render_span(e.span, &format!("lex error: {}", e.message))
        );
        1
    })?;

    let program = parse(tokens).map_err(|e| {
        eprintln!(
            "{}",
            source.render_span(e.span, &format!("parse error: {}", e.message))
        );
        1
    })?;

    let ty = check_program(&program).map_err(|e| {
        eprintln!(
            "{}",
            source.render_span(e.span, &format!("type error: {}", e.message))
        );
        1
    })?;

    println!("ok: {ty}");
    Ok(())
}

fn cmd_vm(path: String) -> Result<(), i32> {
    let source = load_source(&path).map_err(|e| {
        eprintln!("io error: {e}");
        1
    })?;

    let tokens = lex(&source.text).map_err(|e| {
        eprintln!(
            "{}",
            source.render_span(e.span, &format!("lex error: {}", e.message))
        );
        1
    })?;

    let program = parse(tokens).map_err(|e| {
        eprintln!(
            "{}",
            source.render_span(e.span, &format!("parse error: {}", e.message))
        );
        1
    })?;

    let _ = check_program(&program).map_err(|e| {
        eprintln!(
            "{}",
            source.render_span(e.span, &format!("type error: {}", e.message))
        );
        1
    })?;

    let module = compile(&program).map_err(|e| {
        eprintln!(
            "{}",
            source.render_span(e.span, &format!("compile error: {}", e.message))
        );
        1
    })?;

    let value = run_vm(module).map_err(|e| {
        eprintln!(
            "{}",
            source.render_span(e.span, &format!("vm error: {}", e.message))
        );
        1
    })?;

    if value != Value::Unit {
        println!("{value}");
    }

    Ok(())
}

fn cmd_disasm(path: String) -> Result<(), i32> {
    let source = load_source(&path).map_err(|e| {
        eprintln!("io error: {e}");
        1
    })?;

    let tokens = lex(&source.text).map_err(|e| {
        eprintln!(
            "{}",
            source.render_span(e.span, &format!("lex error: {}", e.message))
        );
        1
    })?;

    let program = parse(tokens).map_err(|e| {
        eprintln!(
            "{}",
            source.render_span(e.span, &format!("parse error: {}", e.message))
        );
        1
    })?;

    let _ = check_program(&program).map_err(|e| {
        eprintln!(
            "{}",
            source.render_span(e.span, &format!("type error: {}", e.message))
        );
        1
    })?;

    let module = compile(&program).map_err(|e| {
        eprintln!(
            "{}",
            source.render_span(e.span, &format!("compile error: {}", e.message))
        );
        1
    })?;

    println!("main: f{}", module.main);
    for (id, func) in module.functions.iter().enumerate() {
        let params = if func.params.is_empty() {
            String::new()
        } else {
            func.params.join(", ")
        };
        println!("\nfn f{id} {}({})", func.name, params);
        for (ip, instr) in func.code.iter().enumerate() {
            let start = instr.span.start.min(source.text.len());
            let end = instr.span.end.min(source.text.len());
            let (line, col) = source.line_col(start);
            println!(
                "  {:04}  {:<24}  @{}:{}  [{}..{}]",
                ip, instr.kind, line, col, start, end
            );
        }
    }

    Ok(())
}

fn load_source(path: &str) -> std::io::Result<Source> {
    if path == "-" {
        use std::io::Read;
        let mut text = String::new();
        std::io::stdin().read_to_string(&mut text)?;
        Ok(Source::new(PathBuf::from("<stdin>"), text))
    } else {
        Source::from_path(path)
    }
}

fn print_help() {
    println!(
        "moon (prototype)

USAGE:
  moon run <file>
  moon ast <file>
  moon check <file>
  moon vm <file>
  moon disasm <file>

NOTES:
  - Use '-' as <file> to read from stdin.
  - Semicolons discard values; the last expression without ';' is the program result.
  - Current features: let, assignment, blocks, if/else, fn/calls, arrays/objects, and expressions."
    );
}

use std::env;
use std::path::PathBuf;

use moon_core::eval::eval_program;
use moon_core::lexer::lex;
use moon_core::parser::parse;
use moon_core::source::Source;

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
        eprintln!("{}", source.render_span(e.span, &format!("lex error: {}", e.message)));
        1
    })?;

    let program = parse(tokens).map_err(|e| {
        eprintln!(
            "{}",
            source.render_span(e.span, &format!("parse error: {}", e.message))
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

    if value != moon_core::eval::Value::Unit {
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
        eprintln!("{}", source.render_span(e.span, &format!("lex error: {}", e.message)));
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
        "moon (MVP)

USAGE:
  moon run <file>
  moon ast <file>

NOTES:
  - Use '-' as <file> to read from stdin.
  - This is an early prototype: ints/bools/strings, let, and expressions."
    );
}

use moon_core::eval::{eval_program, Value};
use moon_core::lexer::lex;
use moon_core::parser::parse;
use moon_core::source::Source;

fn run(src: &str) -> Value {
    let source = Source::new("<test>", src.to_string());
    let tokens = lex(&source.text).unwrap();
    let program = parse(tokens).unwrap();
    eval_program(&program).unwrap()
}

#[test]
fn arithmetic_precedence() {
    let v = run("let x = 1 + 2 * 3; x + 1;");
    assert_eq!(v, Value::Int(8));
}

#[test]
fn bool_precedence() {
    let v = run("let x = true && false || true; x;");
    assert_eq!(v, Value::Bool(true));
}

#[test]
fn string_concat() {
    let v = run("let s = \"a\" + \"b\"; s;");
    assert_eq!(v, Value::String("ab".to_string()));
}

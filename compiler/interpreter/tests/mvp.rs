use moon_core::lexer::lex;
use moon_core::parser::parse;
use moon_core::source::Source;
use moon_interpreter::{eval_program, Value};

fn run(src: &str) -> Value {
    let source = Source::new("<test>", src.to_string());
    let tokens = lex(&source.text).unwrap();
    let program = parse(tokens).unwrap();
    eval_program(&program).unwrap()
}

#[test]
fn arithmetic_precedence() {
    let v = run("let x = 1 + 2 * 3; x + 1");
    assert_eq!(v, Value::Int(8));
}

#[test]
fn bool_precedence() {
    let v = run("let x = true && false || true; x");
    assert_eq!(v, Value::Bool(true));
}

#[test]
fn string_concat() {
    let v = run("let s = \"a\" + \"b\"; s");
    assert_eq!(v, Value::String("ab".to_string()));
}

#[test]
fn block_scope_and_tail_expr() {
    let v = run("let x = 1; { let x = 2; x } + x");
    assert_eq!(v, Value::Int(3));
}

#[test]
fn if_expression() {
    let v = run("if true { 1 } else { 2 }");
    assert_eq!(v, Value::Int(1));
}

#[test]
fn functions_work_and_do_not_capture_caller_locals() {
    let v = run(
        "let x = 10;
         fn f() -> Int { x }
         { let x = 20; f() }",
    );
    assert_eq!(v, Value::Int(10));
}

#[test]
fn can_call_function_before_its_definition() {
    let v = run(
        "f(1);
         fn f(x: Int) -> Int { x + 1 }
         f(1)",
    );
    assert_eq!(v, Value::Int(2));
}

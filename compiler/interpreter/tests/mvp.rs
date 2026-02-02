use moon_core::lexer::lex;
use moon_core::parser::parse;
use moon_core::source::Source;
use moon_interpreter::{eval_program, RuntimeError, Value};

fn run_result(src: &str) -> Result<Value, RuntimeError> {
    let source = Source::new("<test>", src.to_string());
    let tokens = lex(&source.text).unwrap();
    let program = parse(tokens).unwrap();
    eval_program(&program)
}

fn run(src: &str) -> Value {
    run_result(src).unwrap()
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
    let v = run("let x = 10;
         fn f() -> Int { x }
         { let x = 20; f() }");
    assert_eq!(v, Value::Int(10));
}

#[test]
fn can_call_function_before_its_definition() {
    let v = run("f(1);
         fn f(x: Int) -> Int { x + 1 }
         f(1)");
    assert_eq!(v, Value::Int(2));
}

#[test]
fn array_literal_index_and_assignment() {
    let v = run("let a = [1, 2, 3]; a[0] = 10; a[0] + a[1]");
    assert_eq!(v, Value::Int(12));
}

#[test]
fn object_literal_index_and_assignment() {
    let v = run("let o = #{ a: 1, \"b\": 2 }; o[\"a\"] = 10; o[\"a\"] + o[\"b\"]");
    assert_eq!(v, Value::Int(12));
}

#[test]
fn variable_assignment_updates_nearest_scope() {
    let v = run("let x = 1; { let x = 2; x = 3; x } + x");
    assert_eq!(v, Value::Int(4));
}

#[test]
fn return_is_error_at_top_level() {
    let err = run_result("return 1;").unwrap_err();
    assert!(err.message.contains("return"));
}

#[test]
fn return_exits_function_early() {
    let v =
        run("fn f(x: Int) -> Int { if x > 0 { return x; } else { }; x + 1 }\n         f(0) + f(2)");
    assert_eq!(v, Value::Int(3));
}

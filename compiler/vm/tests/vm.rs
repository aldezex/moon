use moon_bytecode::compile;
use moon_core::lexer::lex;
use moon_core::parser::parse;
use moon_core::source::Source;
use moon_typechecker::check_program;
use moon_vm::run;

fn run_vm(src: &str) -> moon_runtime::Value {
    let source = Source::new("<test>", src.to_string());
    let tokens = lex(&source.text).unwrap();
    let program = parse(tokens).unwrap();
    check_program(&program).unwrap();
    let module = compile(&program).unwrap();
    run(module).unwrap()
}

#[test]
fn arithmetic_and_precedence() {
    let v = run_vm("let x = 1 + 2 * 3; x + 1");
    assert_eq!(v, moon_runtime::Value::Int(8));
}

#[test]
fn blocks_scopes_and_tail_expr() {
    let v = run_vm("let x = 1; { let x = 2; x } + x");
    assert_eq!(v, moon_runtime::Value::Int(3));
}

#[test]
fn if_expression() {
    let v = run_vm("if true { 1 } else { 2 }");
    assert_eq!(v, moon_runtime::Value::Int(1));
}

#[test]
fn functions_and_call_before_definition() {
    let v = run_vm(
        "f(1);
         fn f(x: Int) -> Int { x + 1 }
         f(1)",
    );
    assert_eq!(v, moon_runtime::Value::Int(2));
}

#[test]
fn functions_are_values_and_can_be_called_indirectly() {
    let v = run_vm(
        "fn add1(x: Int) -> Int { x + 1 }
         let f = add1;
         f(41)",
    );
    assert_eq!(v, moon_runtime::Value::Int(42));
}

#[test]
fn anonymous_functions_work() {
    let v = run_vm("let f = fn(x: Int) -> Int { x + 1 }; f(41)");
    assert_eq!(v, moon_runtime::Value::Int(42));
}

#[test]
fn closures_capture_lexical_variables() {
    let v = run_vm(
        "let f = { let x = 10; fn(y: Int) -> Int { x + y } };
         { let x = 100; f(1) }",
    );
    assert_eq!(v, moon_runtime::Value::Int(11));
}

#[test]
fn closures_can_mutate_captured_state() {
    let v = run_vm(
        "let c = { let x = 0; fn() -> Int { x = x + 1; x } };
         c() + c()",
    );
    assert_eq!(v, moon_runtime::Value::Int(3));
}

#[test]
fn arrays_objects_and_assignment() {
    let v = run_vm(
        "let a = [1, 2, 3];
         a[0] = 10;
         let o = #{ a: 1, \"b\": 2 };
         o[\"a\"] = 10;
         a[0] + o[\"b\"]",
    );
    assert_eq!(v, moon_runtime::Value::Int(12));
}

#[test]
fn gc_builtin_keeps_roots_alive() {
    let v = run_vm("let a = [1, 2, 3]; gc(); a[0]");
    assert_eq!(v, moon_runtime::Value::Int(1));
}

#[test]
fn return_statement_exits_function_early() {
    let v = run_vm(
        "fn f(x: Int) -> Int { if x > 0 { return x; } else { }; x + 1 }\n         f(0) + f(2)",
    );
    assert_eq!(v, moon_runtime::Value::Int(3));
}

#[test]
fn function_can_be_implemented_with_only_return() {
    let v = run_vm("fn f() -> Int { return 1; } f()");
    assert_eq!(v, moon_runtime::Value::Int(1));
}

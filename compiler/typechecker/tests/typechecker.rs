use moon_core::lexer::lex;
use moon_core::parser::parse;
use moon_core::source::Source;
use moon_typechecker::{check_program, Type};

fn check(src: &str) -> Result<Type, String> {
    let source = Source::new("<test>", src.to_string());
    let tokens = lex(&source.text).map_err(|e| format!("lex: {}", e.message))?;
    let program = parse(tokens).map_err(|e| format!("parse: {}", e.message))?;
    check_program(&program).map_err(|e| e.message)
}

#[test]
fn infers_let_and_checks_ops() {
    let ty = check("let x = 1 + 2 * 3; x").unwrap();
    assert_eq!(ty, Type::Int);
}

#[test]
fn rejects_mismatched_let_annotation() {
    let err = check("let x: Bool = 1; x").unwrap_err();
    assert!(err.contains("type mismatch"));
}

#[test]
fn rejects_if_branch_type_mismatch() {
    let err = check("if true { 1 } else { false }").unwrap_err();
    assert!(err.contains("if branches"));
}

#[test]
fn rejects_wrong_argument_type() {
    let err = check("fn f(x: Int) -> Int { x } f(true)").unwrap_err();
    assert!(err.contains("argument type mismatch"));
}

#[test]
fn rejects_wrong_return_type() {
    let err = check("fn f() -> Bool { 1 } 0").unwrap_err();
    assert!(err.contains("type mismatch"));
}

#[test]
fn can_typecheck_call_before_definition() {
    let ty = check("f(1); fn f(x: Int) -> Int { x } f(2)").unwrap();
    assert_eq!(ty, Type::Int);
}

#[test]
fn allows_functions_as_values_and_indirect_calls() {
    let ty = check("fn add1(x: Int) -> Int { x + 1 } let f = add1; f(41)").unwrap();
    assert_eq!(ty, Type::Int);
}

#[test]
fn rejects_calling_non_function_value() {
    let err = check("let x = 1; x(2)").unwrap_err();
    assert!(err.contains("cannot call non-function"));
}

#[test]
fn infers_array_types_and_indexing() {
    let ty = check("let a = [1, 2, 3]; a[0]").unwrap();
    assert_eq!(ty, Type::Int);
}

#[test]
fn rejects_mixed_array_element_types() {
    let err = check("let a = [1, true]; a").unwrap_err();
    assert!(err.contains("array elements"));
}

#[test]
fn requires_annotation_for_empty_array_literal() {
    let err = check("let a = []; a").unwrap_err();
    assert!(err.contains("empty array"));
}

#[test]
fn allows_empty_array_with_annotation() {
    let ty = check("let a: Array<Int> = []; a").unwrap();
    assert_eq!(ty, Type::Array(Box::new(Type::Int)));
}

#[test]
fn infers_object_types_and_indexing() {
    let ty = check("let o = #{ a: 1, \"b\": 2 }; o[\"a\"]").unwrap();
    assert_eq!(ty, Type::Int);
}

#[test]
fn allows_empty_object_with_annotation() {
    let ty = check("let o: Object<Int> = #{}; o").unwrap();
    assert_eq!(ty, Type::Object(Box::new(Type::Int)));
}

#[test]
fn rejects_assignment_type_mismatch() {
    let err = check("let x: Int = 1; x = true; x").unwrap_err();
    assert!(err.contains("type mismatch"));
}

#[test]
fn rejects_return_outside_function() {
    let err = check("return 1; 0").unwrap_err();
    assert!(err.contains("return"));
}

#[test]
fn rejects_return_type_mismatch() {
    let err = check("fn f() -> Int { return true; } 0").unwrap_err();
    assert!(err.contains("type mismatch"));
}

#[test]
fn allows_function_with_only_return_statement() {
    let ty = check("fn f() -> Int { return 1; } f()").unwrap();
    assert_eq!(ty, Type::Int);
}

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

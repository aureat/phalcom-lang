use phalcom_ast::error::{RestParameterErrorKind, SyntaxErrorKind};
use phalcom_ast::parse_source;

fn rest_error(source: &str) -> RestParameterErrorKind {
    let error = parse_source(source, 0).expect_err("fixture must be rejected");
    match error.kind {
        SyntaxErrorKind::RestParameter(kind) => kind,
        other => panic!("expected structured rest diagnostic, got {other:?}"),
    }
}

#[test]
fn duplicate_positional_rest_is_structured() {
    assert_eq!(
        rest_error("class C { f(*left, *right) { return 0 } }\n"),
        RestParameterErrorKind::DuplicatePositional
    );
}

#[test]
fn positional_rest_after_label_is_structured() {
    assert_eq!(
        rest_error("class C { f(timeout, *tail) { return 0 } }\n"),
        RestParameterErrorKind::PositionalAfterLabeled
    );
}

#[test]
fn duplicate_labeled_rest_is_structured() {
    assert_eq!(
        rest_error("class C { f(**left, **right) { return 0 } }\n"),
        RestParameterErrorKind::DuplicateLabeled
    );
}

#[test]
fn complete_rest_conflict_is_structured() {
    assert_eq!(
        rest_error("class C { f(*tail, ***remaining) { return 0 } }\n"),
        RestParameterErrorKind::CompleteConflict
    );
}

#[test]
fn parameter_after_labeled_rest_is_structured() {
    assert_eq!(
        rest_error("class C { f(**extra, debug) { return 0 } }\n"),
        RestParameterErrorKind::AfterTerminal
    );
}

#[test]
fn positional_after_positional_rest_is_structured() {
    assert_eq!(
        rest_error("class C { f(*tail, _ later) { return 0 } }\n"),
        RestParameterErrorKind::PositionalAfterLabeledOrRest
    );
}

#[test]
fn subscript_rest_is_structured() {
    assert_eq!(
        rest_error("class C { [*items] { return 0 } }\n"),
        RestParameterErrorKind::UnsupportedInSubscript
    );
}

#[test]
fn split_rest_remains_valid() {
    parse_source(
        "class C { split(_ fixed, *tail, timeout, **extra) { return tail.size + extra.size } }\n",
        0,
    )
    .expect("valid split-rest declaration must continue to parse");
}

use phalcom_ast::{
    ast::{AlgebraicPhase, Statement},
    lexer::Lexer,
    parse_source,
    token::Token,
};

#[test]
fn retired_enum_spelling_is_not_a_keyword() {
    let token = Lexer::new("enum Shape").next().expect("identifier token").expect("lexes");
    assert!(matches!(token.1, Token::Identifier(ref name) if name == "enum"));
}

#[test]
fn enum_declaration_syntax_is_rejected() {
    let source = "enum Shape { Circle(_ radius: Float) }\n";
    assert!(parse_source(source, 0).is_err());
}

#[test]
fn data_retains_sum_and_gadt_syntax() {
    let source = "data Expr<T> {\n  Int(_ value: Int) -> Expr<Int>\n  Bool(_ value: Bool) -> Expr<Bool>\n}\n";
    let program = parse_source(source, 0).expect("data GADT parses");
    let Statement::Data(data) = &program.statements[0] else {
        panic!("expected data declaration");
    };
    assert_eq!(data.phase, AlgebraicPhase::Runtime);
    assert!(data.is_sum());
    assert_eq!(data.cases().expect("cases").len(), 2);
}

#[test]
fn bare_data_cases_support_all_payload_shapes() {
    let source = "data Cases<T> {\n  Singleton\n  Nullary()\n  Positional(_ value: T)\n  Record { value: T }\n}\n";
    let program = parse_source(source, 0).expect("bare sum cases parse");
    let Statement::Data(data) = &program.statements[0] else {
        panic!("expected data declaration");
    };
    let cases = data.cases().expect("cases");
    assert_eq!(cases.len(), 4);
    assert!(cases[0].is_singleton());
    assert!(cases[1].is_explicit_empty());
    assert_eq!(cases[2].tuple_parameters().expect("tuple payload").len(), 1);
    assert_eq!(cases[3].record_components().expect("record payload").len(), 1);
}

#[test]
fn data_variant_marker_is_rejected_at_the_marker() {
    let source = "data Shape { @variant Circle(_ radius: Float) }\n";
    let marker_start = source.find("@variant").expect("marker in source");
    let error = parse_source(source, 0).expect_err("legacy data marker is rejected");
    assert_eq!(error.code(), "syntax.data.variant_marker_retired");
    assert_eq!(error.range, marker_start..marker_start + "@variant".len());
}

#[test]
fn class_variant_marker_is_retired_at_the_marker() {
    let source = "@sealed class Shape { @variant Circle(radius:) }\n";
    let marker_start = source.find("@variant").expect("marker in source");
    let error = parse_source(source, 0).expect_err("class marker is retired");
    assert_eq!(error.code(), "syntax.class.variant_marker_retired");
    assert_eq!(error.range, marker_start..marker_start + "@variant".len());
}

#[test]
fn class_variant_marker_is_rejected_without_a_case_body() {
    let source = "@sealed class Shape { @variant }\n";
    let marker_start = source.find("@variant").expect("marker in source");
    let error = parse_source(source, 0).expect_err("class marker is retired");
    assert_eq!(error.code(), "syntax.class.variant_marker_retired");
    assert_eq!(error.range, marker_start..marker_start + "@variant".len());
}

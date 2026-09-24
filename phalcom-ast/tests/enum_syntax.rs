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
    let source = "enum Shape { @variant Circle(_ radius: Float) }\n";
    assert!(parse_source(source, 0).is_err());
}

#[test]
fn data_retains_sum_and_gadt_syntax() {
    let source = "data Expr<T> {\n  @variant Int(_ value: Int) -> Expr<Int>\n  @variant Bool(_ value: Bool) -> Expr<Bool>\n}\n";
    let program = parse_source(source, 0).expect("data GADT parses");
    let Statement::Data(data) = &program.statements[0] else {
        panic!("expected data declaration");
    };
    assert_eq!(data.phase, AlgebraicPhase::Runtime);
    assert!(data.is_sum());
    assert_eq!(data.cases().expect("cases").len(), 2);
}

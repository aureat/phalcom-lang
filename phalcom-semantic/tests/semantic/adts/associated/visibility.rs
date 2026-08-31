//! Visibility scenarios are kept separate from family identity tests.

#[test]
#[ignore = "GATED: visibility requires the multi-module ADT fixture model"]
fn visibility_scenarios_require_cross_module_fixture_support() {
    let source = "enum Public { @variant Ready }\nclass Test { run() { Public::Ready } }\n";
    let parsed = phalcom_ast::parse_source(source, 0).expect("visibility baseline should parse");
    assert!(!parsed.statements.is_empty(), "gated visibility test must retain an executable source fixture");
}

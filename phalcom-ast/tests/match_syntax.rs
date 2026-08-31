use phalcom_ast::{
    ast::{Expr, MatchArm, MatchExpr, Pattern, Statement, VariantPattern, VariantPatternArgument, VariantPatternMode},
    parse_source,
};
use phalcom_common::range::SourceRange;

#[test]
fn match_expression_ast_shape() {
    let dummy_range = SourceRange::new(0, 10);
    let match_expr = MatchExpr {
        value: Box::new(Expr::Int {
            digits: "42".into(),
            radix: 10,
            range: dummy_range,
        }),
        arms: vec![
            MatchArm {
                pattern: Pattern::Variant(VariantPattern {
                    owner: None,
                    base: "Some".into(),
                    base_range: dummy_range,
                    mode: VariantPatternMode::ExactCall {
                        arguments: vec![VariantPatternArgument {
                            label: None,
                            label_range: None,
                            pattern: Pattern::Name {
                                name: "x".into(),
                                range: dummy_range,
                            },
                            range: dummy_range,
                        }],
                    },
                    range: dummy_range,
                }),
                branch: Box::new(Expr::Var {
                    value: "x".into(),
                    range: dummy_range,
                }),
                arrow_range: dummy_range,
                range: dummy_range,
            },
            MatchArm {
                pattern: Pattern::Wildcard { range: dummy_range },
                branch: Box::new(Expr::Int {
                    digits: "0".into(),
                    radix: 10,
                    range: dummy_range,
                }),
                arrow_range: dummy_range,
                range: dummy_range,
            },
        ],
        range: dummy_range,
    };

    let expr = Expr::Match(match_expr);
    match expr {
        Expr::Match(m) => {
            assert_eq!(m.arms.len(), 2);
            assert!(matches!(m.arms[1].pattern, Pattern::Wildcard { .. }));
        }
        other => panic!("expected match expression, got {other:#?}"),
    }
}

#[test]
fn parse_basic_match_expression() {
    let source = "const value = match result {\n  Result::Ok(x) => x\n  Result::Error(_, reason: message) => {\n    message\n  }\n}";
    let program = parse_source(source, 0).expect("should parse cleanly");
    assert_eq!(program.statements.len(), 1);
    let Statement::Let(let_decl) = &program.statements[0] else {
        panic!("expected let statement");
    };
    let Some(Expr::Match(match_expr)) = &let_decl.value else {
        panic!("expected match expression, got {:?}", let_decl.value);
    };
    assert_eq!(match_expr.arms.len(), 2);

    // Arm 1: Result::Ok(x) => x
    let arm0 = &match_expr.arms[0];
    let Pattern::Variant(var0) = &arm0.pattern else {
        panic!("expected variant pattern");
    };
    assert_eq!(var0.owner.as_ref().unwrap().root, "Result");
    assert_eq!(var0.base, "Ok");
    let VariantPatternMode::ExactCall { arguments: args0 } = &var0.mode else {
        panic!("expected ExactCall");
    };
    assert_eq!(args0.len(), 1);
    assert_eq!(args0[0].label, None);
    assert!(matches!(args0[0].pattern, Pattern::Name { ref name, .. } if name == "x"));

    // Arm 2: Result::Error(_, reason: message) => { message }
    let arm1 = &match_expr.arms[1];
    let Pattern::Variant(var1) = &arm1.pattern else {
        panic!("expected variant pattern");
    };
    assert_eq!(var1.owner.as_ref().unwrap().root, "Result");
    assert_eq!(var1.base, "Error");
    let VariantPatternMode::ExactCall { arguments: args1 } = &var1.mode else {
        panic!("expected ExactCall");
    };
    assert_eq!(args1.len(), 2);
    assert!(matches!(args1[0].pattern, Pattern::Wildcard { .. }));
    assert_eq!(args1[1].label.as_deref(), Some("reason"));
    assert!(matches!(args1[1].pattern, Pattern::Name { ref name, .. } if name == "message"));
    assert!(matches!(*arm1.branch, Expr::Block(_)));
}

#[test]
fn parse_match_with_families_gaps_and_wildcard() {
    let source = "match animal {\n  Animal::Dog* => 1\n  Animal::Cat(...) => 2\n  Animal::Fox(name, ..., named: age) => 3\n  _ => 4\n}";
    let program = parse_source(source, 0).expect("should parse cleanly");
    let Statement::Expr { expr: Expr::Match(m), .. } = &program.statements[0] else {
        panic!("expected match expression");
    };
    assert_eq!(m.arms.len(), 4);

    // Arm 0: Animal::Dog*
    let Pattern::Variant(v0) = &m.arms[0].pattern else {
        panic!("expected variant");
    };
    assert_eq!(v0.base, "Dog");
    assert!(matches!(v0.mode, VariantPatternMode::WholeFamily { .. }));

    // Arm 1: Animal::Cat(...)
    let Pattern::Variant(v1) = &m.arms[1].pattern else {
        panic!("expected variant");
    };
    assert_eq!(v1.base, "Cat");
    let VariantPatternMode::CallablePattern { prefix: p1, suffix: s1, .. } = &v1.mode else {
        panic!("expected CallablePattern");
    };
    assert!(p1.is_empty());
    assert!(s1.is_empty());

    // Arm 2: Animal::Fox(name, ..., named: age)
    let Pattern::Variant(v2) = &m.arms[2].pattern else {
        panic!("expected variant");
    };
    assert_eq!(v2.base, "Fox");
    let VariantPatternMode::CallablePattern { prefix: p2, suffix: s2, .. } = &v2.mode else {
        panic!("expected CallablePattern");
    };
    assert_eq!(p2.len(), 1);
    assert!(matches!(p2[0].pattern, Pattern::Name { ref name, .. } if name == "name"));
    assert_eq!(s2.len(), 1);
    assert_eq!(s2[0].label.as_deref(), Some("named"));
    assert!(matches!(s2[0].pattern, Pattern::Name { ref name, .. } if name == "age"));

    // Arm 3: _
    assert!(matches!(m.arms[3].pattern, Pattern::Wildcard { .. }));
}

#[test]
fn parse_match_with_nested_and_or_patterns() {
    let source = "match value {\n  Some(Ok(x) | Cached(x)) => x\n  Some(Error(_)) => 0\n  None => 0\n}";
    let program = parse_source(source, 0).expect("should parse cleanly");
    let Statement::Expr { expr: Expr::Match(m), .. } = &program.statements[0] else {
        panic!("expected match expression");
    };
    assert_eq!(m.arms.len(), 3);

    // Arm 0: Some(Ok(x) | Cached(x))
    let Pattern::Variant(v0) = &m.arms[0].pattern else {
        panic!("expected variant");
    };
    assert_eq!(v0.base, "Some");
    let VariantPatternMode::ExactCall { arguments: args0 } = &v0.mode else {
        panic!("expected ExactCall");
    };
    assert_eq!(args0.len(), 1);
    let Pattern::Or { alternatives, .. } = &args0[0].pattern else {
        panic!("expected Or pattern");
    };
    assert_eq!(alternatives.len(), 2);
    let Pattern::Variant(alt0) = &alternatives[0] else {
        panic!("expected variant");
    };
    assert_eq!(alt0.base, "Ok");
    let Pattern::Variant(alt1) = &alternatives[1] else {
        panic!("expected variant");
    };
    assert_eq!(alt1.base, "Cached");

    // Arm 1: Some(Error(_))
    let Pattern::Variant(v1) = &m.arms[1].pattern else {
        panic!("expected variant");
    };
    assert_eq!(v1.base, "Some");
    let VariantPatternMode::ExactCall { arguments: args1 } = &v1.mode else {
        panic!("expected ExactCall");
    };
    let Pattern::Variant(inner) = &args1[0].pattern else {
        panic!("expected variant");
    };
    assert_eq!(inner.base, "Error");
    let VariantPatternMode::ExactCall { arguments: inner_args } = &inner.mode else {
        panic!("expected ExactCall");
    };
    assert!(matches!(inner_args[0].pattern, Pattern::Wildcard { .. }));

    // Arm 2: None (bare identifier contextual singleton parsed as Name at AST stage)
    assert!(matches!(m.arms[2].pattern, Pattern::Name { ref name, .. } if name == "None"));
}

#[test]
fn review_ast_01_nested_or_patterns_remain_inside_payload_node() {
    let program = parse_source("match value { Some(Ok(x) | Cached(x)) => x _ => 0 }", 0).expect("match should parse");
    let Statement::Expr { expr: Expr::Match(match_expr), .. } = &program.statements[0] else {
        panic!("expected match expression");
    };
    let Pattern::Variant(outer) = &match_expr.arms[0].pattern else {
        panic!("expected outer variant");
    };
    let VariantPatternMode::ExactCall { arguments } = &outer.mode else {
        panic!("expected exact outer call");
    };
    assert!(matches!(arguments[0].pattern, Pattern::Or { ref alternatives, .. } if alternatives.len() == 2));
}

#[test]
fn review_ast_02_selector_modes_preserve_getter_call_and_family_shape() {
    let program = parse_source("match value { Animal::Dog => 1 Animal::Dog() => 2 Animal::Dog* => 3 }", 0).expect("match should parse");
    let Statement::Expr { expr: Expr::Match(match_expr), .. } = &program.statements[0] else {
        panic!("expected match expression");
    };
    let Pattern::Variant(getter) = &match_expr.arms[0].pattern else { panic!("expected getter pattern") };
    assert!(matches!(getter.mode, VariantPatternMode::Singleton));
    let Pattern::Variant(call) = &match_expr.arms[1].pattern else { panic!("expected call pattern") };
    assert!(matches!(call.mode, VariantPatternMode::ExactCall { ref arguments } if arguments.is_empty()));
    let Pattern::Variant(family) = &match_expr.arms[2].pattern else { panic!("expected family pattern") };
    assert!(matches!(family.mode, VariantPatternMode::WholeFamily { .. }));
}

#[test]
fn review_ast_03_source_ranges_cover_gap_and_label_tokens() {
    let source = "match value { Animal::Dog(x, ..., named: y) => x }";
    let program = parse_source(source, 0).expect("match should parse");
    let Statement::Expr { expr: Expr::Match(match_expr), .. } = &program.statements[0] else {
        panic!("expected match expression");
    };
    let Pattern::Variant(variant) = &match_expr.arms[0].pattern else { panic!("expected variant") };
    assert!(variant.base_range.start < variant.base_range.end);
    let VariantPatternMode::CallablePattern { gap_range, suffix, .. } = &variant.mode else {
        panic!("expected callable pattern");
    };
    assert!(gap_range.start < gap_range.end);
    assert_eq!(suffix[0].label.as_deref(), Some("named"));
    assert!(suffix[0].label_range.expect("label range").start < suffix[0].label_range.expect("label range").end);
}

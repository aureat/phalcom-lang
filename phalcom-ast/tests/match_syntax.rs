use phalcom_ast::ast::{
    Expr, MatchArm, MatchExpr, Pattern, SourceRange, VariantPattern, VariantPatternArgument,
    VariantPatternMode,
};

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

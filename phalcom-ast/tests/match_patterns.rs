use phalcom_ast::ast::{
    Pattern, SourceRange, VariantPattern, VariantPatternArgument, VariantPatternMode,
};

#[test]
fn or_and_wildcard_pattern_ast_shape() {
    let dummy_range = SourceRange::new(0, 10);
    let or_pattern = Pattern::Or {
        alternatives: vec![
            Pattern::Variant(VariantPattern {
                owner: None,
                base: "Ok".into(),
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
            Pattern::Variant(VariantPattern {
                owner: None,
                base: "Cached".into(),
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
        ],
        range: dummy_range,
    };

    assert!(matches!(or_pattern, Pattern::Or { ref alternatives, .. } if alternatives.len() == 2));
}

#[test]
fn variant_pattern_modes_ast_shape() {
    let dummy_range = SourceRange::new(0, 10);

    let singleton = VariantPattern {
        owner: None,
        base: "Dog".into(),
        base_range: dummy_range,
        mode: VariantPatternMode::Singleton,
        range: dummy_range,
    };
    assert!(matches!(singleton.mode, VariantPatternMode::Singleton));

    let whole_family = VariantPattern {
        owner: None,
        base: "Dog".into(),
        base_range: dummy_range,
        mode: VariantPatternMode::WholeFamily {
            star_range: dummy_range,
        },
        range: dummy_range,
    };
    assert!(matches!(whole_family.mode, VariantPatternMode::WholeFamily { .. }));

    let callable_pattern = VariantPattern {
        owner: None,
        base: "Dog".into(),
        base_range: dummy_range,
        mode: VariantPatternMode::CallablePattern {
            prefix: vec![VariantPatternArgument {
                label: None,
                label_range: None,
                pattern: Pattern::Name {
                    name: "name".into(),
                    range: dummy_range,
                },
                range: dummy_range,
            }],
            gap_range: dummy_range,
            suffix: vec![VariantPatternArgument {
                label: Some("named".into()),
                label_range: Some(dummy_range),
                pattern: Pattern::Name {
                    name: "age".into(),
                    range: dummy_range,
                },
                range: dummy_range,
            }],
        },
        range: dummy_range,
    };
    assert!(matches!(callable_pattern.mode, VariantPatternMode::CallablePattern { .. }));
}

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

#[test]
fn selector_projection_from_variant_patterns() {
    let dummy_range = SourceRange::new(0, 10);
    use phalcom_ast::selector::{
        selector_from_exact_variant_pattern, selector_pattern_from_variant_pattern, SelectorSlot,
    };

    // 1. Singleton
    let singleton = VariantPattern {
        owner: None,
        base: "Dog".into(),
        base_range: dummy_range,
        mode: VariantPatternMode::Singleton,
        range: dummy_range,
    };
    let exact_sel = selector_from_exact_variant_pattern(&singleton).unwrap();
    assert_eq!(exact_sel.encode(), "Dog");

    // 2. ExactCall with labels: Dog(_, named: _)
    let exact_call = VariantPattern {
        owner: None,
        base: "Dog".into(),
        base_range: dummy_range,
        mode: VariantPatternMode::ExactCall {
            arguments: vec![
                VariantPatternArgument {
                    label: None,
                    label_range: None,
                    pattern: Pattern::Wildcard { range: dummy_range },
                    range: dummy_range,
                },
                VariantPatternArgument {
                    label: Some("named".into()),
                    label_range: Some(dummy_range),
                    pattern: Pattern::Name {
                        name: "age".into(),
                        range: dummy_range,
                    },
                    range: dummy_range,
                },
            ],
        },
        range: dummy_range,
    };
    let exact_call_sel = selector_from_exact_variant_pattern(&exact_call).unwrap();
    assert_eq!(exact_call_sel.encode(), "Dog(_,named)");

    // 3. CallablePattern with gap: Dog(x, ..., named: y)
    let pattern_call = VariantPattern {
        owner: None,
        base: "Dog".into(),
        base_range: dummy_range,
        mode: VariantPatternMode::CallablePattern {
            prefix: vec![VariantPatternArgument {
                label: None,
                label_range: None,
                pattern: Pattern::Name {
                    name: "x".into(),
                    range: dummy_range,
                },
                range: dummy_range,
            }],
            gap_range: dummy_range,
            suffix: vec![VariantPatternArgument {
                label: Some("named".into()),
                label_range: Some(dummy_range),
                pattern: Pattern::Name {
                    name: "y".into(),
                    range: dummy_range,
                },
                range: dummy_range,
            }],
        },
        range: dummy_range,
    };
    let pat_sel = selector_pattern_from_variant_pattern(&pattern_call).unwrap();
    assert!(pat_sel.has_gap);
    assert_eq!(pat_sel.prefix.as_ref(), &[SelectorSlot::Positional]);
    assert_eq!(pat_sel.suffix.as_ref(), &[SelectorSlot::Label("named".into())]);
}


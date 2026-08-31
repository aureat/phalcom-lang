use phalcom_ast::ast::{
    MapPatternEntry, MapPatternKey, Pattern, RecordPatternEntry, SourceRange, VariantPattern,
    VariantPatternArgument, VariantPatternMode,
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
        mode: VariantPatternMode::WholeFamily { star_range: dummy_range },
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
    use phalcom_ast::selector::{SelectorSlot, selector_from_exact_variant_pattern, selector_pattern_from_variant_pattern};

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

#[test]
fn review_ast_selector_projection_is_canonical_for_all_variant_modes() {
    let dummy_range = SourceRange::new(0, 10);
    use phalcom_ast::selector::{selector_from_exact_variant_pattern, selector_pattern_from_variant_pattern};

    let singleton = VariantPattern {
        owner: None,
        base: "State".into(),
        base_range: dummy_range,
        mode: VariantPatternMode::Singleton,
        range: dummy_range,
    };
    assert_eq!(selector_from_exact_variant_pattern(&singleton).expect("singleton selector").encode(), "State");

    let family = VariantPattern {
        owner: None,
        base: "State".into(),
        base_range: dummy_range,
        mode: VariantPatternMode::CallablePattern {
            prefix: vec![],
            gap_range: dummy_range,
            suffix: vec![],
        },
        range: dummy_range,
    };
    let projected = selector_pattern_from_variant_pattern(&family).expect("family selector pattern");
    assert!(projected.has_gap);
    assert!(selector_from_exact_variant_pattern(&family).is_err());
}

#[test]
fn review_ast_02_record_and_map_patterns_keep_distinct_discriminants() {
    let range = SourceRange::new(0, 1);
    let record = Pattern::Record {
        entries: vec![RecordPatternEntry {
            label: "name".into(),
            pattern: Pattern::Wildcard { range },
            range,
        }],
        range,
    };
    let map = Pattern::Map {
        entries: vec![MapPatternEntry {
            key: MapPatternKey::Symbol("name".into()),
            pattern: Pattern::Wildcard { range },
            range,
        }],
        range,
    };

    assert!(matches!(record, Pattern::Record { .. }));
    assert!(matches!(map, Pattern::Map { .. }));
}

#[test]
fn review_m3_01_method_callable_pattern_projects_method_slots() {
    use phalcom_ast::selector::selector_pattern_from_variant_pattern;
    let range = SourceRange::new(0, 1);
    let pattern = VariantPattern {
        owner: None,
        base: "Dog".into(),
        base_range: range,
        mode: VariantPatternMode::CallablePattern {
            prefix: vec![VariantPatternArgument {
                label: None,
                label_range: None,
                pattern: Pattern::Wildcard { range },
                range,
            }],
            gap_range: range,
            suffix: vec![],
        },
        range,
    };
    let projected = selector_pattern_from_variant_pattern(&pattern).expect("method pattern");
    assert_eq!(projected.prefix.as_ref(), &[phalcom_ast::selector::SelectorSlot::Positional]);
}

#[test]
fn review_m3_02_getter_singleton_does_not_project_as_callable_method() {
    use phalcom_ast::selector::{selector_from_exact_variant_pattern, selector_pattern_from_variant_pattern};
    let range = SourceRange::new(0, 1);
    let pattern = VariantPattern {
        owner: None,
        base: "Dog".into(),
        base_range: range,
        mode: VariantPatternMode::Singleton,
        range,
    };
    assert_eq!(selector_from_exact_variant_pattern(&pattern).expect("getter selector").kind, phalcom_common::selector::SelectorKind::Getter);
    assert!(selector_pattern_from_variant_pattern(&pattern).is_err());
}

#[test]
fn review_m3_03_selector_projection_does_not_invent_kind_for_family_gap() {
    use phalcom_ast::selector::selector_pattern_from_variant_pattern;
    let range = SourceRange::new(0, 1);
    let pattern = VariantPattern {
        owner: None,
        base: "Dog".into(),
        base_range: range,
        mode: VariantPatternMode::CallablePattern { prefix: vec![], gap_range: range, suffix: vec![] },
        range,
    };
    let projected = selector_pattern_from_variant_pattern(&pattern).expect("family gap projection");
    assert!(projected.has_gap);
}

#[test]
#[ignore = "GATED: non-method callable variant syntax is not admitted"]
fn review_m3_04_non_method_callable_variant_kind_is_preserved() {
    use phalcom_ast::selector::selector_pattern_from_variant_pattern;
    let range = SourceRange::new(0, 1);
    let pattern = VariantPattern {
        owner: None,
        base: "Dog".into(),
        base_range: range,
        mode: VariantPatternMode::CallablePattern {
            prefix: vec![VariantPatternArgument {
                label: Some("name".into()),
                label_range: Some(range),
                pattern: Pattern::Wildcard { range },
                range,
            }],
            gap_range: range,
            suffix: vec![],
        },
        range,
    };
    let projected = selector_pattern_from_variant_pattern(&pattern).expect("callable selector pattern");
    assert_eq!(projected.prefix.as_ref(), &[phalcom_ast::selector::SelectorSlot::Label("name".into())]);
}

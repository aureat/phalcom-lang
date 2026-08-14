use phalcom_common::selector::{Selector, SelectorKind, SelectorKindPattern, SelectorPattern, SelectorSlot};

#[test]
fn exact_selector_round_trips_all_kinds() {
    let cases = [
        Selector::getter("name").unwrap(),
        Selector::method("name", Vec::<SelectorSlot>::new().into_boxed_slice()).unwrap(),
        Selector::method("move", vec![SelectorSlot::Positional, SelectorSlot::Label("to".into())].into_boxed_slice()).unwrap(),
        Selector::setter("name").unwrap(),
        Selector::subscript_get(vec![SelectorSlot::Positional].into_boxed_slice()).unwrap(),
        Selector::subscript_set(vec![SelectorSlot::Label("ключ".into())].into_boxed_slice()).unwrap(),
    ];

    for selector in cases {
        let encoded = selector.encode();
        assert_eq!(Selector::try_decode_exact(&encoded).unwrap(), selector);
    }
}

#[test]
fn escaped_and_unicode_labels_round_trip() {
    let selector = Selector::method(
        "move",
        vec![SelectorSlot::Label("a,b".into()), SelectorSlot::Label("λ".into())].into_boxed_slice(),
    )
    .unwrap();

    assert_eq!(Selector::try_decode_exact(&selector.encode()).unwrap(), selector);
}

#[test]
fn method_gap_pattern_matches_prefix_and_suffix() {
    let pattern = SelectorPattern::named_method(
        "foo",
        vec![SelectorSlot::Positional].into_boxed_slice(),
        vec![SelectorSlot::Label("bar".into())].into_boxed_slice(),
        true,
    )
    .unwrap();

    assert!(
        pattern.matches(
            &Selector::method(
                "foo",
                vec![SelectorSlot::Positional, SelectorSlot::Positional, SelectorSlot::Label("bar".into())].into_boxed_slice(),
            )
            .unwrap()
        )
    );
    assert!(!pattern.matches(&Selector::getter("foo").unwrap()));
}

#[test]
fn broad_named_pattern_matches_named_selector_kinds_only() {
    let empty = Vec::<SelectorSlot>::new().into_boxed_slice();
    let pattern = SelectorPattern::named("foo", SelectorKindPattern::AnyNamed, empty.clone(), empty.clone(), true).unwrap();
    assert!(pattern.matches(&Selector::getter("foo").unwrap()));
    assert!(pattern.matches(&Selector::method("foo", empty.clone()).unwrap()));
    assert!(pattern.matches(&Selector::setter("foo").unwrap()));
    assert!(!pattern.matches(&Selector::subscript_get(empty.clone()).unwrap()));
}

#[test]
fn rejects_invalid_structural_construction() {
    assert!(Selector::method("foo", vec![SelectorSlot::Label("x".into()), SelectorSlot::Positional].into_boxed_slice()).is_err());
    let empty = Vec::<SelectorSlot>::new().into_boxed_slice();
    assert!(SelectorPattern::named_method("foo", empty.clone(), empty.clone(), false).is_err());
    assert!(
        SelectorPattern::new(
            phalcom_common::selector::SelectorBase::Subscript,
            SelectorKindPattern::Exact(SelectorKind::Getter),
            empty.clone(),
            empty,
            false,
        )
        .is_err()
    );
}

#[test]
fn malformed_runtime_selector_decode_is_total() {
    for text in ["", "foo(", "foo)", "[", "[x", "foo(~zz)", "foo(~f)", "foo(~ff)", "foo(,)"] {
        let decoded = std::panic::catch_unwind(|| Selector::decode(text));
        assert!(decoded.is_ok(), "decode panicked for {text:?}");
    }
}

#[test]
fn runtime_decode_preserves_rest_family_markers_after_labels() {
    let selector = Selector::decode("labeled(timeout,**)");
    assert_eq!(selector.base, phalcom_common::selector::SelectorBase::Named("labeled".into()));
    assert_eq!(selector.kind, SelectorKind::Method);
    assert_eq!(selector.slots.as_ref(), &[SelectorSlot::Label("timeout".into()), SelectorSlot::Positional]);
}

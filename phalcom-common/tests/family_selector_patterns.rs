use phalcom_common::selector::{Selector, SelectorBase, SelectorKind, SelectorKindPattern, SelectorPattern, SelectorSlot};

fn method(name: &str, slots: Vec<SelectorSlot>) -> Selector {
    Selector::method(name, slots.into_boxed_slice()).unwrap()
}

fn empty_slots() -> Box<[SelectorSlot]> {
    Vec::new().into_boxed_slice()
}

fn method_pattern(prefix: Vec<SelectorSlot>, suffix: Vec<SelectorSlot>) -> SelectorPattern {
    SelectorPattern::named_method("route", prefix.into_boxed_slice(), suffix.into_boxed_slice(), true).unwrap()
}

#[test]
fn gap_is_zero_or_more_slots_not_one_or_more() {
    let pattern = method_pattern(vec![SelectorSlot::Positional], vec![SelectorSlot::Label("foo".into())]);

    assert!(pattern.matches(&method("route", vec![SelectorSlot::Positional, SelectorSlot::Label("foo".into())],)));
    assert!(pattern.matches(&method(
        "route",
        vec![SelectorSlot::Positional, SelectorSlot::Positional, SelectorSlot::Label("foo".into()),],
    )));
    assert!(pattern.matches(&method(
        "route",
        vec![
            SelectorSlot::Positional,
            SelectorSlot::Positional,
            SelectorSlot::Label("mid".into()),
            SelectorSlot::Label("foo".into()),
        ],
    )));
}

#[test]
fn prefix_suffix_and_base_are_all_structural_constraints() {
    let pattern = method_pattern(vec![SelectorSlot::Positional], vec![SelectorSlot::Label("foo".into())]);

    assert!(!pattern.matches(&method("other", vec![SelectorSlot::Positional, SelectorSlot::Label("foo".into())])));
    assert!(!pattern.matches(&method("route", vec![SelectorSlot::Label("foo".into())])));
    assert!(!pattern.matches(&method("route", vec![SelectorSlot::Positional, SelectorSlot::Label("bar".into())],)));
    assert!(!pattern.matches(&method("route", vec![SelectorSlot::Positional])));
}

#[test]
fn method_gap_pattern_never_matches_getter_or_setter() {
    let empty = Vec::<SelectorSlot>::new().into_boxed_slice();
    let pattern = SelectorPattern::named_method("value", empty.clone(), empty, true).unwrap();

    assert!(pattern.matches(&Selector::method("value", empty_slots()).unwrap()));
    assert!(!pattern.matches(&Selector::getter("value").unwrap()));
    assert!(!pattern.matches(&Selector::setter("value").unwrap()));
}

#[test]
fn any_named_pattern_covers_getter_setter_and_methods_but_not_subscripts() {
    let empty = Vec::<SelectorSlot>::new().into_boxed_slice();
    let pattern = SelectorPattern::named("value", SelectorKindPattern::AnyNamed, empty.clone(), empty.clone(), true).unwrap();

    assert!(pattern.matches(&Selector::getter("value").unwrap()));
    assert!(pattern.matches(&Selector::setter("value").unwrap()));
    assert!(pattern.matches(&Selector::method("value", empty_slots()).unwrap()));
    assert!(pattern.matches(&method("value", vec![SelectorSlot::Positional])));
    assert!(!pattern.matches(&Selector::getter("other").unwrap()));
    assert!(!pattern.matches(&Selector::subscript_get(empty_slots()).unwrap()));
    assert!(!pattern.matches(&Selector::subscript_set(empty_slots()).unwrap()));
}

#[test]
fn setter_pattern_matches_only_the_named_setter() {
    let pattern = SelectorPattern::named("value", SelectorKindPattern::Exact(SelectorKind::Setter), empty_slots(), empty_slots(), true).unwrap();

    assert!(pattern.matches(&Selector::setter("value").unwrap()));
    assert!(!pattern.matches(&Selector::getter("value").unwrap()));
    assert!(!pattern.matches(&Selector::method("value", empty_slots()).unwrap()));
    assert!(!pattern.matches(&Selector::setter("other").unwrap()));
}

#[test]
fn subscript_patterns_distinguish_read_and_write_kinds() {
    let get = SelectorPattern::new(
        SelectorBase::Subscript,
        SelectorKindPattern::Exact(SelectorKind::SubscriptGet),
        vec![SelectorSlot::Positional].into_boxed_slice(),
        Vec::<SelectorSlot>::new().into_boxed_slice(),
        true,
    )
    .unwrap();
    let set = SelectorPattern::new(
        SelectorBase::Subscript,
        SelectorKindPattern::Exact(SelectorKind::SubscriptSet),
        vec![SelectorSlot::Positional].into_boxed_slice(),
        Vec::<SelectorSlot>::new().into_boxed_slice(),
        true,
    )
    .unwrap();

    let one = vec![SelectorSlot::Positional].into_boxed_slice();
    let two = vec![SelectorSlot::Positional, SelectorSlot::Positional].into_boxed_slice();
    assert!(get.matches(&Selector::subscript_get(one.clone()).unwrap()));
    assert!(get.matches(&Selector::subscript_get(two.clone()).unwrap()));
    assert!(!get.matches(&Selector::subscript_set(one.clone()).unwrap()));
    assert!(set.matches(&Selector::subscript_set(one.clone()).unwrap()));
    assert!(set.matches(&Selector::subscript_set(two).unwrap()));
    assert!(!set.matches(&Selector::subscript_get(one).unwrap()));
}

#[test]
fn impossible_pattern_lane_orderings_are_rejected() {
    assert!(
        SelectorPattern::named_method(
            "route",
            vec![SelectorSlot::Label("foo".into())].into_boxed_slice(),
            vec![SelectorSlot::Positional].into_boxed_slice(),
            true,
        )
        .is_err()
    );

    assert!(
        SelectorPattern::named(
            "value",
            SelectorKindPattern::Exact(SelectorKind::Getter),
            vec![SelectorSlot::Positional].into_boxed_slice(),
            empty_slots(),
            true,
        )
        .is_err()
    );
    assert!(
        SelectorPattern::named(
            "value",
            SelectorKindPattern::Exact(SelectorKind::Setter),
            empty_slots(),
            vec![SelectorSlot::Label("foo".into())].into_boxed_slice(),
            true,
        )
        .is_err()
    );
    assert!(SelectorPattern::new(SelectorBase::Subscript, SelectorKindPattern::AnyNamed, empty_slots(), empty_slots(), true,).is_err());
}

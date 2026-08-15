use phalcom_ast::{
    ast::{BinaryOp, Expr, MethodRefExpr, SelectorPatternSyntax, SelectorSpecSyntax, Statement, SymbolLiteralKind},
    parse_source,
};
use phalcom_common::selector::{SelectorKind, SelectorKindPattern, SelectorSlot};

fn method_ref(source: &str) -> MethodRefExpr {
    let program = parse_source(source, 0).unwrap_or_else(|err| panic!("{source:?} should parse: {err:?}"));
    let statement = program.statements.into_iter().next().expect("fixture should contain one statement");
    let Statement::Expr {
        expr: Expr::MethodRef(reference),
        ..
    } = statement
    else {
        panic!("{source:?} should parse as a method reference");
    };
    *reference
}

fn pattern(source: &str) -> SelectorPatternSyntax {
    match method_ref(source).spec {
        SelectorSpecSyntax::Pattern(pattern) => pattern,
        other => panic!("{source:?} should be a selector pattern, got {other:?}"),
    }
}

fn slot_values(slots: &[phalcom_ast::ast::SelectorSlotSyntax]) -> Vec<SelectorSlot> {
    slots.iter().map(|slot| slot.slot.clone()).collect()
}

#[test]
fn colon_colon_exact_forms_are_hashless_and_structurally_distinct() {
    let cases = [
        ("receiver::name", SelectorKind::Getter, vec![]),
        ("receiver::name()", SelectorKind::Method, vec![]),
        ("receiver::name(_)", SelectorKind::Method, vec![SelectorSlot::Positional]),
        (
            "receiver::name(_, foo)",
            SelectorKind::Method,
            vec![SelectorSlot::Positional, SelectorSlot::Label("foo".into())],
        ),
        (
            "receiver::name(foo, bar)",
            SelectorKind::Method,
            vec![SelectorSlot::Label("foo".into()), SelectorSlot::Label("bar".into())],
        ),
        ("receiver::name=(put)", SelectorKind::Setter, vec![]),
    ];

    let operator = method_ref("receiver::+");
    let SelectorSpecSyntax::Exact(operator) = operator.spec else {
        panic!("operator selector should be exact");
    };
    assert_eq!(operator.base, "+");
    assert_eq!(operator.kind, SelectorKind::Method);
    assert_eq!(slot_values(&operator.slots), vec![SelectorSlot::Positional]);

    for (source, expected_kind, expected_slots) in cases {
        let reference = method_ref(source);
        let SelectorSpecSyntax::Exact(exact) = reference.spec else {
            panic!("{source:?} should be exact");
        };
        assert_eq!(exact.base, "name", "{source:?}");
        assert_eq!(exact.kind, expected_kind, "{source:?}");
        assert_eq!(slot_values(&exact.slots), expected_slots, "{source:?}");
    }
}

#[test]
fn colon_colon_rejects_legacy_hash_prefixed_selector_specs() {
    // `#` remains the first-class selector/pattern sigil. It is deliberately
    // not part of `::` selector syntax: the Family operator already establishes
    // selector-spec context, so `receiver::#name` is redundant legacy syntax.
    for source in [
        "receiver::#name",
        "receiver::#name()",
        "receiver::#name(_)",
        "receiver::#name(foo)",
        "receiver::#name...",
        "receiver::#name(...) ",
        "receiver::#name(_, ..., foo)",
        "receiver::#+",
    ] {
        assert!(parse_source(source, 0).is_err(), "legacy syntax must be rejected: {source:?}");
    }
}

#[test]
fn colon_colon_accepts_every_single_gap_position() {
    let any_named = pattern("receiver::name...");
    assert_eq!(any_named.kind, SelectorKindPattern::AnyNamed);
    assert!(any_named.prefix.is_empty());
    assert!(any_named.suffix.is_empty());

    let whole_method = pattern("receiver::name(...)");
    assert_eq!(whole_method.kind, SelectorKindPattern::Exact(SelectorKind::Method));
    assert!(whole_method.prefix.is_empty());
    assert!(whole_method.suffix.is_empty());

    let prefix = pattern("receiver::name(_, ...)");
    assert_eq!(slot_values(&prefix.prefix), vec![SelectorSlot::Positional]);
    assert!(prefix.suffix.is_empty());

    let suffix = pattern("receiver::name(..., foo)");
    assert!(suffix.prefix.is_empty());
    assert_eq!(slot_values(&suffix.suffix), vec![SelectorSlot::Label("foo".into())]);

    let sandwich = pattern("receiver::name(_, ..., foo)");
    assert_eq!(slot_values(&sandwich.prefix), vec![SelectorSlot::Positional]);
    assert_eq!(slot_values(&sandwich.suffix), vec![SelectorSlot::Label("foo".into())]);

    let wide = pattern("receiver::name(_, _, ..., foo, bar)");
    assert_eq!(
        slot_values(&wide.prefix),
        vec![SelectorSlot::Positional, SelectorSlot::Positional]
    );
    assert_eq!(
        slot_values(&wide.suffix),
        vec![SelectorSlot::Label("foo".into()), SelectorSlot::Label("bar".into())]
    );
}

#[test]
fn colon_colon_accepts_setter_pattern_without_hash() {
    let setter = pattern("receiver::name=...");
    assert_eq!(setter.base, "name");
    assert_eq!(setter.kind, SelectorKindPattern::Exact(SelectorKind::Setter));
    assert!(setter.prefix.is_empty());
    assert!(setter.suffix.is_empty());
}

#[test]
fn malformed_selector_patterns_are_rejected_at_parse_time() {
    for source in [
        "receiver::name(..., ...)",
        "receiver::name(_, ..., foo, ...)",
        "receiver::name(foo, ..., _)",
        "receiver::name(_, , foo)",
        "receiver::name(...",
        "receiver::name(_, ... foo)",
    ] {
        assert!(parse_source(source, 0).is_err(), "malformed pattern unexpectedly parsed: {source:?}");
    }
}

#[test]
fn first_class_selector_specs_keep_hash_for_reflection() {
    // Banning `#` after `::` must not accidentally retire selector Symbols or
    // SelectorPattern values. Those are exactly what ordinary `Behavior#>>(_:)`
    // consumes on its RHS.
    for source in [
        "#name",
        "#name()",
        "#name(_)",
        "#name...",
        "#name(...) ",
        "#name(_, ...)",
        "#name(..., foo)",
        "#name(_, ..., foo)",
        "#name=...",
        "#name=(put)",
    ] {
        assert!(parse_source(source, 0).is_ok(), "first-class selector spec should parse: {source:?}");
    }
}

#[test]
fn shift_right_with_selector_pattern_remains_an_ordinary_binary_send() {
    let program = parse_source("Behavior >> #name(_, ..., foo)", 0).expect("reflection expression parses");
    let Statement::Expr {
        expr: Expr::Binary(binary),
        ..
    } = &program.statements[0]
    else {
        panic!("expected binary >> expression");
    };
    assert_eq!(binary.op, BinaryOp::ShiftRight);
    let Expr::Symbol(symbol) = &binary.right else {
        panic!(">> RHS should remain a first-class selector-pattern value");
    };
    let SymbolLiteralKind::Pattern(pattern) = &symbol.kind else {
        panic!(">> RHS should be a SelectorPattern");
    };
    assert_eq!(pattern.kind, SelectorKindPattern::Exact(SelectorKind::Method));
    assert_eq!(slot_values(&pattern.prefix), vec![SelectorSlot::Positional]);
    assert_eq!(slot_values(&pattern.suffix), vec![SelectorSlot::Label("foo".into())]);
}

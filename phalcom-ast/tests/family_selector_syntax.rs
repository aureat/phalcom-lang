use phalcom_ast::{
    ast::{
        AssociatedInvokeExpr, AssociatedLookupExpr, AssociatedMemberSyntax, AssociatedNamedMode,
        AssociatedResidualSelectorSyntax, BinaryOp, Expr, Statement, SymbolLiteralKind,
    },
    error::SyntaxErrorKind,
    parse_source,
};
use phalcom_common::selector::{SelectorKind, SelectorKindPattern, SelectorSlot};

fn slot_values(slots: &[phalcom_ast::ast::SelectorSlotSyntax]) -> Vec<SelectorSlot> {
    slots.iter().map(|slot| slot.slot.clone()).collect()
}

fn parse_expr_stmt(source: &str) -> Expr {
    let program = parse_source(source, 0).unwrap_or_else(|err| panic!("{source:?} should parse: {err:?}"));
    let statement = program.statements.into_iter().next().expect("fixture should contain one statement");
    let Statement::Expr { expr, .. } = statement else {
        panic!("{source:?} should parse as an expression statement");
    };
    expr
}

fn associated_lookup(source: &str) -> AssociatedLookupExpr {
    match parse_expr_stmt(source) {
        Expr::AssociatedLookup(lookup) => *lookup,
        other => panic!("{source:?} should parse as AssociatedLookup, got {other:?}"),
    }
}

fn associated_invoke(source: &str) -> AssociatedInvokeExpr {
    match parse_expr_stmt(source) {
        Expr::AssociatedInvoke(invoke) => *invoke,
        other => panic!("{source:?} should parse as AssociatedInvoke, got {other:?}"),
    }
}

#[test]
fn associated_named_modes_parse_correctly() {
    // 1. Implicit getter: receiver::name
    let lookup = associated_lookup("receiver::name");
    let AssociatedMemberSyntax::Named(named) = lookup.member else {
        panic!("expected Named member");
    };
    assert_eq!(named.base, "name");
    assert!(matches!(named.mode, AssociatedNamedMode::Getter { explicit_separator_range: None }));

    // 2. Explicit getter: receiver::name::
    let lookup = associated_lookup("receiver::name::");
    let AssociatedMemberSyntax::Named(named) = lookup.member else {
        panic!("expected Named member");
    };
    assert_eq!(named.base, "name");
    assert!(matches!(named.mode, AssociatedNamedMode::Getter { explicit_separator_range: Some(_) }));

    // 3. Family lookup: receiver::name::*
    let lookup = associated_lookup("receiver::name::*");
    let AssociatedMemberSyntax::Named(named) = lookup.member else {
        panic!("expected Named member");
    };
    assert_eq!(named.base, "name");
    assert!(matches!(named.mode, AssociatedNamedMode::Family { .. }));

    // 4. Exact zero-arg method: receiver::name::()
    let lookup = associated_lookup("receiver::name::()");
    let AssociatedMemberSyntax::Named(named) = lookup.member else {
        panic!("expected Named member");
    };
    assert_eq!(named.base, "name");
    let AssociatedNamedMode::Exact { residual: AssociatedResidualSelectorSyntax::Method { slots, .. }, .. } = named.mode else {
        panic!("expected Exact method mode");
    };
    assert!(slots.is_empty());

    // 5. Exact method with slots: receiver::name::(_, foo)
    let lookup = associated_lookup("receiver::name::(_, foo)");
    let AssociatedMemberSyntax::Named(named) = lookup.member else {
        panic!("expected Named member");
    };
    assert_eq!(named.base, "name");
    let AssociatedNamedMode::Exact { residual: AssociatedResidualSelectorSyntax::Method { slots, .. }, .. } = named.mode else {
        panic!("expected Exact method mode");
    };
    assert_eq!(slots.len(), 2);
    assert_eq!(slots[0].slot, SelectorSlot::Positional);
    assert_eq!(slots[1].slot, SelectorSlot::Label("foo".to_string()));

    // 6. Exact setter: receiver::name::=(put)
    let lookup = associated_lookup("receiver::name::=(put)");
    let AssociatedMemberSyntax::Named(named) = lookup.member else {
        panic!("expected Named member");
    };
    assert_eq!(named.base, "name");
    assert!(matches!(named.mode, AssociatedNamedMode::Exact { residual: AssociatedResidualSelectorSyntax::Setter { .. }, .. }));
}

#[test]
fn associated_operator_and_subscript_parse_correctly() {
    let lookup = associated_lookup("receiver::+");
    let AssociatedMemberSyntax::Operator(exact) = lookup.member else {
        panic!("expected Operator member");
    };
    assert_eq!(exact.base, "+");
    assert_eq!(exact.kind, SelectorKind::Method);

    let lookup = associated_lookup("receiver::[]");
    let AssociatedMemberSyntax::Subscript(exact) = lookup.member else {
        panic!("expected Subscript member");
    };
    assert_eq!(exact.kind, SelectorKind::SubscriptGet);

    let lookup = associated_lookup("receiver::[_, foo]");
    let AssociatedMemberSyntax::Subscript(exact) = lookup.member else {
        panic!("expected Subscript member");
    };
    assert_eq!(exact.slots.len(), 2);
}

#[test]
fn associated_direct_invoke_parses_correctly() {
    let invoke = associated_invoke("receiver::name()");
    assert_eq!(invoke.base, "name");
    assert!(invoke.args.is_empty());

    let invoke = associated_invoke("receiver::name(1, label: 2)");
    assert_eq!(invoke.base, "name");
    assert_eq!(invoke.args.len(), 2);
}

#[test]
fn legacy_family_and_shape_errors_are_diagnosed() {
    // Single-colon-colon exact shape error
    let err = parse_source("receiver::name(_)", 0).unwrap_err();
    assert_eq!(err.kind, SyntaxErrorKind::AssociatedExactShapeRequiresSecondSeparator);

    // Legacy ellipsis errors
    let err = parse_source("receiver::name...", 0).unwrap_err();
    assert_eq!(err.kind, SyntaxErrorKind::AssociatedLegacyFamilyEllipsis);

    let err = parse_source("receiver::name(...)", 0).unwrap_err();
    assert_eq!(err.kind, SyntaxErrorKind::AssociatedLegacyFamilyEllipsis);

    let err = parse_source("receiver::...", 0).unwrap_err();
    assert_eq!(err.kind, SyntaxErrorKind::AssociatedLegacyFamilyEllipsis);

    let err = parse_source("receiver::name::(...)", 0).unwrap_err();
    assert_eq!(err.kind, SyntaxErrorKind::AssociatedLegacyFamilyEllipsis);
}

#[test]
fn colon_colon_rejects_legacy_hash_prefixed_selector_specs() {
    for source in [
        "receiver::#name",
        "receiver::#name()",
        "receiver::#name(_)",
        "receiver::#name(foo)",
        "receiver::#name...",
        "receiver::#+",
    ] {
        assert!(parse_source(source, 0).is_err(), "legacy syntax must be rejected: {source:?}");
    }
}

#[test]
fn first_class_selector_specs_keep_hash_for_reflection() {
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
    assert!(matches!(binary.op, BinaryOp::ShiftRight));
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

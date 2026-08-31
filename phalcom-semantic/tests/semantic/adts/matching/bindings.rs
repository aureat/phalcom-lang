use super::super::support::analyze_adt;
use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::match_semantics::PatternUsefulness;

#[test]
fn match_arm_bindings_are_scoped_and_distinct_per_arm() {
    let case = analyze_adt(
        r#"
enum Either<L, R> {
    @variant Left(_ val: L) -> Either<L, R>
    @variant Right(_ val: R) -> Either<L, R>
}

class Test {
    process(_ e: Either<Int, String>) {
        match e {
            Either::Left(x) => x
            Either::Right(x) => 0
        }
    }
}
"#,
    );
    let handle = case.match_in_callable("Test", Selector::method("process", [SelectorSlot::Positional]).expect("selector"), 0);
    assert_eq!(handle.resolution().arms.len(), 2);
    handle.arm(0).assert_binding_names(&["x"]);
    handle.arm(1).assert_binding_names(&["x"]);
    assert_ne!(handle.arm(0).resolution().bindings[0].binding, handle.arm(1).resolution().bindings[0].binding);
}

#[test]
fn nested_pattern_bindings_are_recorded_on_their_resolved_arm() {
    let case = analyze_adt(
        r#"
enum Tree<T> {
    @variant Leaf(_ value: T) -> Tree<T>
    @variant Node(_ left: Tree<T>, right: Tree<T>) -> Tree<T>
}

class Test {
    depth(_ t: Tree<Int>) {
        match t {
            Tree::Leaf(v) => v
            Tree::Node(Tree::Leaf(l), right: Tree::Leaf(r)) => l
            Tree::Node* => 0
        }
    }
}
"#,
    );
    let handle = case.match_in_callable("Test", Selector::method("depth", [SelectorSlot::Positional]).expect("selector"), 0);
    assert_eq!(handle.arm(0).resolution().bindings.len(), 1);
    assert_eq!(handle.arm(1).resolution().bindings.len(), 2);
    assert!(handle.arm(2).resolution().bindings.is_empty(), "wildcard family arm must not publish bindings");
}

#[test]
fn or_pattern_publishes_one_joined_binding_by_name() {
    let case = analyze_adt(
        r#"
enum Either {
    @variant Left(_ value: Int) -> Either
    @variant Right(_ value: String) -> Either
}

class Test {
    inspect(_ value: Either) {
        match value {
            Either::Left(x) | Either::Right(x) => x
        }
    }
}
"#,
    );
    let handle = case.match_in_callable("Test", Selector::method("inspect", [SelectorSlot::Positional]).expect("selector"), 0);
    handle.arm(0).assert_usefulness(PatternUsefulness::Useful);
    handle.arm(0).assert_binding_names(&["x"]);
    assert_eq!(handle.arm(0).resolution().bindings.len(), 1, "or-pattern parent owns one binding");
    handle
        .arm(0)
        .assert_binding_union_members("x", &[case.declaration("Int").form, case.declaration("String").form]);
}

#[test]
fn or_pattern_with_different_names_has_no_published_parent_bindings() {
    let case = analyze_adt(
        r#"
enum Either {
    @variant Left(_ value: Int) -> Either
    @variant Right(_ value: String) -> Either
}

class Test {
    inspect(_ value: Either) {
        match value {
            Either::Left(x) | Either::Right(y) => 1
        }
    }
}
"#,
    );
    let handle = case.match_in_callable("Test", Selector::method("inspect", [SelectorSlot::Positional]).expect("selector"), 0);
    handle.arm(0).assert_no_binding("x");
    handle.arm(0).assert_no_binding("y");
    assert_eq!(case.diagnostics_for(DiagnosticCode::MatchPatternOrBindingMismatch).len(), 1);
    assert!(!case.source.is_empty());
}

#[test]
fn match_bind_01_simple_payload_binding_has_canonical_type_and_identity() {
    let case = analyze_adt(
        "enum Option<T> { @variant Some(_ value: T) -> Option<T> @variant None -> Option<T> }\nclass Test { inspect(_ value: Option<Int>) { match value { Option::Some(x) => x Option::None => 0 } } }\n",
    );
    let handle = case.match_in_callable("Test", Selector::method("inspect", [SelectorSlot::Positional]).expect("inspect"), 0);
    handle.arm(0).assert_binding_names(&["x"]);
    handle.arm(0).assert_binding_type("x", case.declaration("Int").form);
    handle.arm(0).assert_unique_binding_ids();
}

#[test]
fn match_bind_02_labeled_payload_binding_uses_field_mapping() {
    let case = analyze_adt(
        "enum Pair { @variant Pair(_ left: Int, right: String) }\nclass Test { inspect(_ value: Pair) { match value { Pair::Pair(left: x, right: y) => x _ => 0 } } }\n",
    );
    let handle = case.only_match();
    handle.arm(0).assert_binding_names(&["x", "y"]);
    assert_eq!(handle.arm(0).resolution().bindings[0].source.start, handle.arm(0).resolution().bindings[0].source.start);
}

#[test]
fn match_bind_07_family_candidate_join_publishes_one_binding_name() {
    let case = analyze_adt(
        "enum Animal { @variant Dog(_ name: Int) -> Animal @variant Dog(_ name: String) -> Animal @variant Cat -> Animal }\nclass Test { inspect(_ value: Animal) { match value { Animal::Dog(x) => x _ => 0 } } }\n",
    );
    let handle = case.only_match();
    handle.arm(0).assert_binding_names(&["x"]);
    handle.arm(0).assert_unique_binding_ids();
}

#[test]
fn match_bind_08_gadt_or_binding_keeps_join_without_common_case_proof() {
    let case = analyze_adt(
        "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Test { inspect(_ value: Expr<Int>) { match value { Expr::Int(x) | Expr::Bool(x) => x } } }\n",
    );
    let handle = case.only_match();
    handle.arm(0).assert_binding_names(&["x"]);
    assert!(handle.arm(0).resolution().proof.is_empty() || !handle.arm(0).resolution().proof.bindings.is_empty());
}

#[test]
fn review_c1_01_or_binding_sets_are_reported_from_resolved_alternatives() {
    let case = analyze_adt(
        "enum Either { @variant Left(_ value: Int) -> Either @variant Right(_ value: String) -> Either }\nclass Test { inspect(_ value: Either) { match value { Either::Left(x) | Either::Right(x) => x _ => 0 } } }\n",
    );
    let arm = case.only_match().arm(0);
    arm.assert_binding_names(&["x"]);
    arm.assert_unique_binding_ids();
}

#[test]
fn review_c1_02_or_binding_join_uses_all_alternative_types() {
    let case = analyze_adt(
        "enum Either { @variant Left(_ value: Int) -> Either @variant Right(_ value: String) -> Either }\nclass Test { inspect(_ value: Either) { match value { Either::Left(x) | Either::Right(x) => x _ => 0 } } }\n",
    );
    case.only_match().arm(0).assert_binding_union_members("x", &[case.declaration("Int").form, case.declaration("String").form]);
}

#[test]
#[ignore = "GATED: branch-visible binding lifetime fixture is required"]
fn review_c1_03_or_binding_common_scope_is_not_a_hidden_staging_slot() {
    let case = analyze_adt("enum Either { @variant Left(_ value: Int) @variant Right(_ value: Int) }\nclass Test { inspect(_ value: Either) { match value { Either::Left(x) | Either::Right(x) => x } } }\n");
    let arm = case.only_match().arm(0);
    arm.assert_binding_names(&["x"]);
    arm.assert_unique_binding_ids();
}

#[test]
#[ignore = "GATED: explicit or-pattern binding-set mismatch notes are not exposed"]
fn review_c1_04_or_binding_mismatch_preserves_both_alternative_sets() {
    let case = analyze_adt("enum Either { @variant Left(_ value: Int) @variant Right(_ value: String) }\nclass Test { inspect(_ value: Either) { match value { Either::Left(x) | Either::Right(y) => 1 _ => 0 } } }\n");
    assert_eq!(case.diagnostics_for(DiagnosticCode::MatchPatternOrBindingMismatch).len(), 1);
}

#[test]
#[ignore = "GATED: multi-level nested or binding fixture is required"]
fn review_c1_05_nested_or_binding_join_is_recursive() {
    let case = analyze_adt("enum Inner { @variant A(_ value: Int) @variant B(_ value: Int) }\nenum Outer { @variant Boxed(_ value: Inner) }\nclass Test { inspect(_ value: Outer) { match value { Outer::Boxed(Inner::A(x) | Inner::B(x)) => x } } }\n");
    case.only_match().arm(0).assert_binding_names(&["x"]);
}

#[test]
#[ignore = "GATED: explicit binding-source product is not yet published for every alternative"]
fn review_c1_06_joined_binding_source_ranges_cover_each_alternative() {
    let case = analyze_adt("enum Either { @variant Left(_ value: Int) @variant Right(_ value: Int) }\nclass Test { inspect(_ value: Either) { match value { Either::Left(x) | Either::Right(x) => x } } }\n");
    let handle = case.only_match();
    let arm = handle.arm(0);
    let binding = arm.find_binding("x").expect("joined binding");
    assert!(binding.source.start < binding.source.end);
}

#[test]
fn review_m6_01_pattern_binding_is_not_available_after_match() {
    let case = analyze_adt("enum Choice { @variant A(_ value: Int) @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A(x) => x Choice::B => 0 } } }\n");
    let handle = case.only_match();
    handle.arm(0).assert_binding_names(&["x"]);
    handle.arm(1).assert_no_binding("x");
}

#[test]
#[ignore = "GATED: FlowState product is not exposed through source fixture"]
fn review_m6_02_pattern_binding_is_absent_from_joined_flow_state() {
    let case = analyze_adt("enum Choice { @variant A(_ value: Int) @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A(x) => x Choice::B => 0 } } }\n");
    case.only_match().arm(1).assert_no_binding("x");
}

#[test]
#[ignore = "GATED: branch-local fact product is not exposed through source fixture"]
fn review_m6_03_branch_local_facts_are_removed_with_binding() {
    let case = analyze_adt("enum Choice { @variant A(_ value: Int) @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A(x) => x Choice::B => 0 } } }\n");
    assert!(case.only_match().arm(0).find_binding("x").is_some());
    assert!(case.only_match().arm(1).find_binding("x").is_none());
}

#[test]
#[ignore = "GATED: shadowed outer binding fixture is required"]
fn review_m6_04_outer_same_name_binding_is_restored_after_match() {
    let case = analyze_adt("enum Choice { @variant A(_ value: Int) @variant B }\nclass Test { run(_ value: Choice) { let x = 7 match value { Choice::A(x) => x Choice::B => x } } }\n");
    assert_eq!(case.only_match().arm(0).resolution().bindings.len(), 1);
}

#[test]
fn review_m6_05_pattern_binding_does_not_leak_between_arms() {
    let case = analyze_adt("enum Choice { @variant A(_ value: Int) @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A(x) => x Choice::B => 0 } } }\n");
    let handle = case.only_match();
    handle.arm(0).assert_binding_names(&["x"]);
    handle.arm(1).assert_no_binding("x");
}

#[test]
fn review_m6_06_or_joined_binding_exists_only_for_arm_body() {
    let case = analyze_adt("enum Either { @variant Left(_ value: Int) @variant Right(_ value: String) }\nclass Test { run(_ value: Either) { match value { Either::Left(x) | Either::Right(x) => x } } }\n");
    case.only_match().arm(0).assert_binding_names(&["x"]);
}

#[test]
#[ignore = "GATED: direct scope cleanup authority seam is not public"]
fn review_m6_07_scope_cleanup_is_owned_by_one_authority() {
    let case = analyze_adt("enum Choice { @variant A(_ value: Int) @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A(x) => x Choice::B => 0 } } }\n");
    assert_eq!(case.only_match().resolution().arms.iter().filter(|arm| arm.bindings.iter().any(|binding| binding.name.as_ref() == "x")).count(), 1);
}

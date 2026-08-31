use super::super::support::analyze_adt;
use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_semantic::match_semantics::PatternUsefulness;

#[test]
fn gadt_branch_proof_refines_type_parameter_and_omits_refuted_cases() {
    let case = analyze_adt(
        r#"
enum Expr<T> {
    @variant LitInt(_ value: Int) -> Expr<Int>
    @variant LitBool(_ value: Bool) -> Expr<Bool>
}

class Test {
    evalInt(_ e: Expr<Int>) {
        match e {
            Expr::LitInt(n) => n
        }
    }
}
"#,
    );
    let handle = case.match_in_callable("Test", Selector::method("evalInt", [SelectorSlot::Positional]).expect("selector"), 0);
    handle.assert_exhaustive();
    handle.arm(0).assert_usefulness(PatternUsefulness::Useful);
    handle
        .arm(0)
        .assert_candidate_variants(&[case.variant_id("Expr", Selector::method("LitInt", [SelectorSlot::Positional]).expect("selector"))]);
    let int_ty = case.declaration("Int").form;
    assert_eq!(handle.arm(0).resolution().proof.bindings.len(), 1, "case environment should retain T = Int");
    assert_eq!(handle.arm(0).resolution().proof.bindings.values().next(), Some(&int_ty));
}

#[test]
fn match_gadt_01_generic_evaluator_keeps_both_candidates_and_branch_proofs() {
    let case = analyze_adt(
        r#"
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
    @variant Bool(_ value: Bool) -> Expr<Bool>
}

class Eval {
    eval<T>(_ e: Expr<T>) -> T {
        match e {
            Expr::Int(x) => x
            Expr::Bool(x) => x
        }
    }
}
"#,
    );
    let handle = case.match_in_callable("Eval", Selector::method("eval", [SelectorSlot::Positional]).expect("selector"), 0);
    handle.assert_exhaustive();
    assert_eq!(handle.resolution().arms.len(), 2);
    handle
        .arm(0)
        .assert_candidate_variants(&[case.variant_id("Expr", Selector::method("Int", [SelectorSlot::Positional]).expect("selector"))]);
    handle
        .arm(1)
        .assert_candidate_variants(&[case.variant_id("Expr", Selector::method("Bool", [SelectorSlot::Positional]).expect("selector"))]);
    assert!(!handle.arm(0).resolution().proof.is_empty(), "Int arm must retain T = Int proof");
    assert!(!handle.arm(1).resolution().proof.is_empty(), "Bool arm must retain T = Bool proof");
    handle.arm(0).assert_binding_type("x", case.declaration("Int").form);
    handle.arm(1).assert_binding_type("x", case.declaration("Bool").form);
}


#[test]
#[ignore = "RED-REVIEW: direct free-parameter proof harness is not yet source-owned"]
fn review_c3_01_free_parameter_vs_concrete_case_is_satisfiable() {
    let case = analyze_adt("enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { eval<U>(_ value: Expr<U>) -> U { match value { Expr::Int(x) => x } } }\n");
    let arm = case.only_match().arm(0);
    arm.assert_usefulness(PatternUsefulness::Useful);
    assert!(!arm.resolution().proof.is_empty());
}

#[test]
fn match_gadt_02_expr_int_excludes_bool_case() {
    let case = analyze_adt("enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { run(_ value: Expr<Int>) { match value { Expr::Bool(x) => x Expr::Int(x) => x } } }\n");
    case.only_match().arm(0).assert_usefulness(PatternUsefulness::Impossible);
}

#[test]
fn match_gadt_03_expr_bool_excludes_int_case() {
    let case = analyze_adt("enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { run(_ value: Expr<Bool>) { match value { Expr::Int(x) => x Expr::Bool(x) => x } } }\n");
    case.only_match().arm(0).assert_usefulness(PatternUsefulness::Impossible);
}

#[test]
#[ignore = "RED-REVIEW: direct free-parameter proof harness is not yet source-owned"]
fn review_c3_02_concrete_vs_free_parameter_is_satisfiable() {
    let case = analyze_adt("enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { eval(_ value: Expr<Int>) { match value { Expr::Int(x) => x } } }\n");
    case.only_match().arm(0).assert_usefulness(PatternUsefulness::Useful);
}

#[test]
#[ignore = "RED-REVIEW: distinct free-parameter IDs need explicit solver fixture"]
fn review_c3_03_distinct_free_parameters_are_not_refuted_by_id_inequality() {
    let case = analyze_adt("enum Pair<T, U> { @variant Left(_ value: T) -> Pair<T, U> @variant Right(_ value: U) -> Pair<T, U> }\nclass Eval { eval<A, B>(_ value: Pair<A, B>) { match value { Pair::Left(x) => x Pair::Right(x) => x } } }\n");
    let handle = case.only_match();
    assert!(handle.arm(0).resolution().usefulness != PatternUsefulness::Impossible);
    assert!(handle.arm(1).resolution().usefulness != PatternUsefulness::Impossible);
}

#[test]
#[ignore = "RED-REVIEW: direct incompatible-concrete proof fixture is not yet source-owned"]
fn review_c3_04_incompatible_concrete_types_remain_refuted() {
    let case = analyze_adt("enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { eval(_ value: Expr<Bool>) { match value { Expr::Int(x) => x } } }\n");
    case.only_match().arm(0).assert_usefulness(PatternUsefulness::Impossible);
}

#[test]
#[ignore = "GATED: nominal-subtype proof policy needs explicit fixture"]
fn review_c3_05_nominal_subtype_policy_is_explicit() {
    let case = analyze_adt("enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { eval(_ value: Expr<Int>) { match value { Expr::Bool(x) => x Expr::Int(x) => x } } }\n");
    case.only_match().arm(0).assert_usefulness(PatternUsefulness::Impossible);
}

#[test]
fn review_c3_06_generic_evaluator_keeps_both_specialized_candidates() {
    let case = super::super::support::analyze_adt(
        "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { run<T>(_ value: Expr<T>) -> T { match value { Expr::Int(x) => x Expr::Bool(x) => x } } }\n",
    );
    let handle = case.only_match();
    assert_eq!(handle.resolution().arms.len(), 2);
    assert!(!handle.arm(0).resolution().proof.is_empty());
    assert!(!handle.arm(1).resolution().proof.is_empty());
}

#[test]
#[ignore = "RED-REVIEW: generic exhaustive proof depends on free-parameter solver correction"]
fn review_c3_07_generic_exhaustiveness_retains_satisfiable_cases() {
    let case = analyze_adt("enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { eval<U>(_ value: Expr<U>) -> U { match value { Expr::Int(x) => x Expr::Bool(x) => x } } }\n");
    case.only_match().assert_exhaustive();
}

#[test]
#[ignore = "RED-REVIEW: mixed open/concrete multi-parameter solver fixture is not yet source-owned"]
fn review_c3_08_multi_parameter_mixed_open_concrete_gadt_is_satisfiable() {
    let case = analyze_adt("enum Pair<T, U> { @variant Left(_ value: Int) -> Pair<Int, U> @variant Right(_ value: String) -> Pair<T, String> }\nclass Eval { eval<A, B>(_ value: Pair<A, B>) { match value { Pair::Left(x) => x Pair::Right(x) => x } } }\n");
    assert!(case.only_match().resolution().arms.iter().all(|arm| !arm.proof.is_empty()));
}

#[test]
#[ignore = "RED: additional branch proof rows await generic solver correction"]
fn match_gadt_04_generic_root_keeps_all_compatible_cases() {
    let case = analyze_adt("enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { eval<U>(_ value: Expr<U>) { match value { Expr::Int(x) => x Expr::Bool(x) => x } } }\n");
    assert_eq!(case.only_match().resolution().arms.len(), 2);
}

#[test]
#[ignore = "GATED: multi-parameter indexed GADT source fixture is required"]
fn match_gadt_05_multi_parameter_proof_is_complete() {
    let case = analyze_adt("enum Pair<T, U> { @variant Left(_ value: Int) -> Pair<Int, U> @variant Right(_ value: String) -> Pair<T, String> }\nclass Eval { eval<A, B>(_ value: Pair<A, B>) { match value { Pair::Left(x) => x Pair::Right(x) => x } } }\n");
    assert!(case.only_match().resolution().arms.iter().all(|arm| !arm.proof.is_empty()));
}

#[test]
#[ignore = "GATED: nested GADT source fixture is required"]
fn match_gadt_06_nested_gadt_proof_is_branch_local() {
    let case = analyze_adt("enum Inner<T> { @variant Int(_ value: Int) -> Inner<Int> }\nenum Outer<T> { @variant Boxed(_ value: Inner<T>) -> Outer<T> }\nclass Eval { eval<U>(_ value: Outer<U>) { match value { Outer::Boxed(Inner::Int(x)) => x } } }\n");
    assert!(!case.only_match().arm(0).resolution().proof.is_empty());
}

#[test]
#[ignore = "GATED: GADT-in-union source fixture is required"]
fn match_gadt_07_gadt_case_in_union_keeps_specialized_space() {
    let case = analyze_adt("enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { eval<U>(_ value: Expr<U>) { match value { Expr::Int(x) => x Expr::Bool(x) => x _ => 0 } } }\n");
    assert_eq!(case.only_match().resolution().arms.len(), 3);
}

#[test]
#[ignore = "RED: or-proof common-fact product is not yet exposed"]
fn match_gadt_08_or_proof_keeps_only_common_facts() {
    let case = analyze_adt("enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { eval<U>(_ value: Expr<U>) { match value { Expr::Int(x) | Expr::Bool(x) => x } } }\n");
    assert!(!case.only_match().resolution().arms.is_empty());
}

#[test]
fn match_gadt_09_branch_proof_does_not_leak_to_sibling_arm() {
    let case = super::super::support::analyze_adt(
        "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { run<T>(_ value: Expr<T>) -> T { match value { Expr::Int(x) => x Expr::Bool(x) => x } } }\n",
    );
    let handle = case.only_match();
    assert_ne!(handle.arm(0).resolution().proof, handle.arm(1).resolution().proof);
}

#[test]
#[ignore = "GATED: post-match flow environment fixture is required"]
fn match_gadt_10_branch_proof_does_not_leak_after_match() {
    let case = analyze_adt("enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { eval<U>(_ value: Expr<U>) { match value { Expr::Int(x) => x Expr::Bool(x) => x } } }\n");
    assert_ne!(case.only_match().arm(0).resolution().proof, Default::default());
}

#[test]
#[ignore = "GATED: blocked compatibility boundary fixture is required"]
fn match_gadt_11_blocked_is_not_impossible() {
    let case = analyze_adt("enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { eval<U>(_ value: Expr<U>) { match value { Expr::Int(x) => x _ => 0 } } }\n");
    assert_ne!(case.only_match().arm(0).resolution().usefulness, PatternUsefulness::Impossible);
}

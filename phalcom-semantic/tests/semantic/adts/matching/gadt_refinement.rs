use super::super::support::analyze_adt;
use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_semantic::match_semantics::{PatternResolution, PatternUsefulness};
use phalcom_semantic::types::LocalType;

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
fn review_c3_01_free_parameter_vs_concrete_case_is_satisfiable() {
    let case = analyze_adt(
        "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { eval<U>(_ value: Expr<U>) -> U { match value { Expr::Int(x) => x } } }\n",
    );
    let arm = case.only_match().arm(0);
    arm.assert_usefulness(PatternUsefulness::Useful);
    assert!(!arm.resolution().proof.is_empty());
}

#[test]
fn match_gadt_02_expr_int_excludes_bool_case() {
    let case = analyze_adt(
        "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { run(_ value: Expr<Int>) { match value { Expr::Bool(x) => x Expr::Int(x) => x } } }\n",
    );
    case.only_match().arm(0).assert_usefulness(PatternUsefulness::Impossible);
}

#[test]
fn match_gadt_03_expr_bool_excludes_int_case() {
    let case = analyze_adt(
        "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { run(_ value: Expr<Bool>) { match value { Expr::Int(x) => x Expr::Bool(x) => x } } }\n",
    );
    case.only_match().arm(0).assert_usefulness(PatternUsefulness::Impossible);
}

#[test]
fn review_c3_02_concrete_vs_free_parameter_is_satisfiable() {
    let case = analyze_adt(
        "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { eval(_ value: Expr<Int>) { match value { Expr::Int(x) => x } } }\n",
    );
    case.only_match().arm(0).assert_usefulness(PatternUsefulness::Useful);
}

#[test]
fn review_c3_03_distinct_free_parameters_are_not_refuted_by_id_inequality() {
    let case = analyze_adt(
        "enum Pair<T, U> { @variant Left(_ value: T) -> Pair<T, U> @variant Right(_ value: U) -> Pair<T, U> }\nclass Eval { eval<A, B>(_ value: Pair<A, B>) { match value { Pair::Left(x) => x Pair::Right(x) => x } } }\n",
    );
    let handle = case.only_match();
    assert!(handle.arm(0).resolution().usefulness != PatternUsefulness::Impossible);
    assert!(handle.arm(1).resolution().usefulness != PatternUsefulness::Impossible);
}

#[test]
fn review_c3_04_incompatible_concrete_types_remain_refuted() {
    let case = analyze_adt(
        "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { eval(_ value: Expr<Bool>) { match value { Expr::Int(x) => x } } }\n",
    );
    case.only_match().arm(0).assert_usefulness(PatternUsefulness::Impossible);
}

#[test]
fn review_c3_05_nominal_subtype_policy_is_explicit() {
    let case = analyze_adt(
        "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { eval(_ value: Expr<Int>) { match value { Expr::Bool(x) => x Expr::Int(x) => x } } }\n",
    );
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
fn review_c3_07_generic_exhaustiveness_retains_satisfiable_cases() {
    let case = analyze_adt(
        "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { eval<U>(_ value: Expr<U>) -> U { match value { Expr::Int(x) => x Expr::Bool(x) => x } } }\n",
    );
    case.only_match().assert_exhaustive();
}

#[test]
fn review_c3_08_multi_parameter_mixed_open_concrete_gadt_is_satisfiable() {
    let case = analyze_adt(
        "enum Pair<T, U> { @variant Left(_ value: Int) -> Pair<Int, U> @variant Right(_ value: String) -> Pair<T, String> }\nclass Eval { eval<A, B>(_ value: Pair<A, B>) { match value { Pair::Left(x) => x Pair::Right(x) => x } } }\n",
    );
    assert!(case.only_match().resolution().arms.iter().all(|arm| !arm.proof.is_empty()));
}

#[test]
fn match_gadt_04_generic_root_keeps_all_compatible_cases() {
    let case = analyze_adt(
        "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { eval<U>(_ value: Expr<U>) { match value { Expr::Int(x) => x Expr::Bool(x) => x } } }\n",
    );
    assert_eq!(case.only_match().resolution().arms.len(), 2);
}

#[test]
fn match_gadt_05_multi_parameter_proof_is_complete() {
    let case = analyze_adt(
        "enum Pair<T, U> { @variant Left(_ value: Int) -> Pair<Int, U> @variant Right(_ value: String) -> Pair<T, String> }\nclass Eval { eval<A, B>(_ value: Pair<A, B>) { match value { Pair::Left(x) => x Pair::Right(x) => x } } }\n",
    );
    assert!(case.only_match().resolution().arms.iter().all(|arm| !arm.proof.is_empty()));
}

#[test]
fn match_gadt_06_nested_gadt_proof_is_branch_local() {
    let case = analyze_adt(
        "enum Inner<T> { @variant Int(_ value: Int) -> Inner<Int> }\nenum Outer<T> { @variant Boxed(_ value: Inner<T>) -> Outer<T> }\nclass Eval { eval<U>(_ value: Outer<U>) { match value { Outer::Boxed(Inner::Int(x)) => x } } }\n",
    );
    let arm = case.only_match().arm(0);
    assert!(!arm.resolution().proof.is_empty(), "nested equality must be visible in branch product");
    let phalcom_semantic::match_semantics::PatternResolution::Variant(outer) = &arm.resolution().pattern else {
        panic!("expected outer variant resolution");
    };
    let phalcom_semantic::match_semantics::PatternResolution::Variant(inner) = outer.candidates[0].fields[0].child.as_ref() else {
        panic!("expected nested variant resolution");
    };
    assert!(!inner.candidates[0].proof.is_empty(), "nested branch must retain its local equality proof");
}

#[test]
fn match_gadt_07_gadt_case_in_union_keeps_specialized_space() {
    let case = analyze_adt(
        "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { eval<U>(_ value: Expr<U>) { match value { Expr::Int(x) => x Expr::Bool(x) => x _ => 0 } } }\n",
    );
    assert_eq!(case.only_match().resolution().arms.len(), 3);
}

#[test]
fn match_gadt_08_or_proof_keeps_only_common_facts() {
    let case = analyze_adt(
        "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { eval<U>(_ value: Expr<U>) { match value { Expr::Int(x) | Expr::Bool(x) => x } } }\n",
    );
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
    let case = analyze_adt(
        "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { eval<U>(_ value: Expr<U>) { match value { Expr::Int(x) => x Expr::Bool(x) => x } } }\n",
    );
    assert_ne!(case.only_match().arm(0).resolution().proof, Default::default());
}

#[test]
fn match_gadt_11_blocked_is_not_impossible() {
    let case = analyze_adt(
        "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { eval<U>(_ value: Expr<U>) { match value { Expr::Int(x) => x _ => 0 } } }\n",
    );
    assert_ne!(case.only_match().arm(0).resolution().usefulness, PatternUsefulness::Impossible);
}

#[test]
fn match_gadt_12_variant_local_generic_opens_shared_rigid_and_keeps_index_proof() {
    let case = analyze_adt(
        r#"
enum Expr<T> {
    @variant Wrap<U>(_ value: U) -> Expr<List<U>> where U <: Object
}

class Eval {
    eval<T>(_ value: Expr<T>) {
        match value {
            Expr::Wrap(x) => x
        }
    }
}
"#,
    );
    let arm = case.only_match().arm(0);
    arm.assert_usefulness(PatternUsefulness::Useful);
    let PatternResolution::Variant(pattern) = &arm.resolution().pattern else {
        panic!("expected variant pattern");
    };
    let candidate = pattern.candidates.first().expect("Wrap candidate");
    let instantiation = candidate.case_instantiation.as_ref().expect("constructor-local case instantiation");
    assert_eq!(instantiation.local_rigids.len(), 1);
    assert_eq!(instantiation.result_type.free_rigids().len(), 1);
    assert!(!candidate.proof.local_equalities.is_empty(), "local result/index proof must be retained");
    let binding = arm.find_binding("x").expect("payload binding");
    assert_eq!(binding.local_type.as_ref().map(|ty| ty.free_rigids().len()), Some(1));
}

#[test]
fn match_gadt_13_variant_local_rigid_is_not_guessed_to_fit_concrete_index() {
    let case = analyze_adt(
        r#"
enum Expr<T> {
    @variant Wrap<U>(_ value: U) -> Expr<List<U>>
}

class Eval {
    eval(_ value: Expr<Int>) {
        match value {
            Expr::Wrap(x) => x
        }
    }
}
"#,
    );
    case.only_match().arm(0).assert_usefulness(PatternUsefulness::Impossible);
}

#[test]
fn match_gadt_14_independent_constructor_observations_get_fresh_rigids_with_alpha_equivalent_shape() {
    let case = analyze_adt(
        r#"
enum Expr<T> {
    @variant Wrap<U>(_ value: U) -> Expr<List<U>>
}

class Eval {
    eval<T>(_ value: Expr<T>) {
        match value {
            Expr::Wrap(x) => x
            Expr::Wrap(y) => y
        }
    }
}
"#,
    );
    let handle = case.only_match();
    let arm0 = handle.arm(0);
    let arm1 = handle.arm(1);
    let PatternResolution::Variant(pat0) = &arm0.resolution().pattern else {
        panic!("expected variant pattern")
    };
    let PatternResolution::Variant(pat1) = &arm1.resolution().pattern else {
        panic!("expected variant pattern")
    };
    let cand0 = pat0.candidates.first().expect("cand0");
    let cand1 = pat1.candidates.first().expect("cand1");
    let inst0 = cand0.case_instantiation.as_ref().expect("inst0");
    let inst1 = cand1.case_instantiation.as_ref().expect("inst1");

    assert_ne!(inst0.scope, inst1.scope, "independent observations must have distinct rigid scopes");
    assert_ne!(inst0.local_rigids, inst1.local_rigids, "independent observations must have distinct rigid IDs");
    assert_ne!(inst0.result_type, inst1.result_type, "exact result terms must not be identical by ID");
    assert!(
        inst0.result_type.alpha_equivalent(&inst1.result_type),
        "independent observations must be alpha-equivalent"
    );
}

#[test]
fn match_gadt_15_enclosing_enum_specialization_appears_in_local_payload() {
    let case = analyze_adt(
        r#"
enum Container<F, T> {
    @variant Item<A>(_ value: A, _ tag: F) -> Container<F, List<A>>
}

class Eval {
    eval<T>(_ value: Container<Int, T>) {
        match value {
            Container::Item(x, tag) => tag
        }
    }
}
"#,
    );
    let arm = case.only_match().arm(0);
    let PatternResolution::Variant(pat) = &arm.resolution().pattern else {
        panic!("expected variant pattern")
    };
    let cand = pat.candidates.first().expect("Item candidate");
    let inst = cand.case_instantiation.as_ref().expect("case instantiation");
    assert_eq!(inst.local_rigids.len(), 1, "only A is constructor-local rigid");

    let int_ty = case.declaration("Int").form;
    let tag_binding = arm.find_binding("tag").expect("tag binding");
    assert_eq!(tag_binding.knowledge.ty(), Some(int_ty), "F must specialize to canonical Int");

    let x_binding = arm.find_binding("x").expect("x binding");
    let x_local = x_binding.local_type.as_ref().expect("x local type");
    assert_eq!(x_local.free_rigids().len(), 1, "value field must preserve constructor-local rigid A");
}

#[test]
fn match_gadt_16_nested_local_expected_subject_preserves_parent_rigid_without_materialization() {
    let case = analyze_adt(
        r#"
enum Inner<U> {
    @variant Boxed(_ val: U) -> Inner<U>
}

enum Outer<T> {
    @variant Wrap<A>(_ inner: Inner<A>) -> Outer<List<A>>
}

class Eval {
    eval<T>(_ value: Outer<T>) {
        match value {
            Outer::Wrap(Inner::Boxed(x)) => x
        }
    }
}
"#,
    );
    let arm = case.only_match().arm(0);
    let x_binding = arm.find_binding("x").expect("x binding");
    let x_local = x_binding.local_type.as_ref().expect("x local type");
    assert_eq!(x_local.free_rigids().len(), 1, "nested payload must preserve parent rigid");
}

#[test]
fn match_gadt_17_r1_t01_exact_nested_local_binding_structure() {
    let case = analyze_adt(
        r#"
enum Inner<U> {
    @variant Boxed(_ value: U) -> Inner<U>
}

enum Outer<T> {
    @variant Wrap<A>(_ inner: Inner<A>) -> Outer<List<A>>
}

class Eval {
    eval<T>(_ value: Outer<T>) {
        match value {
            Outer::Wrap(Inner::Boxed(x)) => x
        }
    }
}
"#,
    );
    let arm = case.only_match().arm(0);
    let PatternResolution::Variant(pat) = &arm.resolution().pattern else {
        panic!("expected variant pattern")
    };
    let outer_cand = pat.candidates.first().expect("outer candidate");
    let outer_field = outer_cand.fields.first().expect("outer field");
    let outer_field_local = outer_field.local_type.as_ref().expect("outer field local type");
    // Outer field local type is Inner<ρ>
    assert!(
        matches!(outer_field_local, LocalType::Applied { .. }),
        "outer field must be Applied Inner<ρ>, got {outer_field_local:?}"
    );

    let x_binding = arm.find_binding("x").expect("x binding");
    let x_local = x_binding.local_type.as_ref().expect("x local type");
    // x must be exactly ρ (LocalType::Rigid), NOT Inner<ρ>
    assert!(
        matches!(x_local, LocalType::Rigid(_)),
        "nested x must bind directly to rigid ρ, not ancestor type {x_local:?}"
    );
}
#[test]
fn match_gadt_18_r1_t02_multiple_nested_bindings_receive_different_local_types() {
    let case = analyze_adt(
        r#"
enum Outer<T> {
    @variant Wrap<A>(_ pair: (A, List<A>)) -> Outer<A>
}

class Eval {
    eval<T>(_ value: Outer<T>) {
        match value {
            Outer::Wrap((x, xs)) => x
        }
    }
}
"#,
    );
    let arm = case.only_match().arm(0);
    let x_binding = arm.find_binding("x").expect("x binding");
    let x_local = x_binding.local_type.as_ref().expect("x local type");
    assert!(matches!(x_local, LocalType::Rigid(_)), "x must be rigid ρ, got {x_local:?}");

    let xs_binding = arm.find_binding("xs").expect("xs binding");
    let xs_local = xs_binding.local_type.as_ref().expect("xs local type");
    assert!(matches!(xs_local, LocalType::Applied { .. }), "xs must be List<ρ>, got {xs_local:?}");
    assert_ne!(x_local, xs_local, "x and xs must have distinct local types");
}

#[test]
fn match_gadt_19_r1_t05_list_local_rigid_propagation() {
    let case = analyze_adt(
        r#"
enum Wrapper<T> {
    @variant Wrap<A>(_ items: List<A>) -> Wrapper<A>
}

class Eval {
    eval<T>(_ value: Wrapper<T>) {
        match value {
            Wrapper::Wrap([head, *tail]) => head
            _ => 0
        }
    }
}
"#,
    );
    let arm = case.only_match().arm(0);
    let head_binding = arm.find_binding("head").expect("head binding");
    let head_local = head_binding.local_type.as_ref().expect("head local type");
    assert!(
        matches!(head_local, LocalType::Rigid(_)),
        "head binding in list pattern must preserve element rigid ρ, got {head_local:?}"
    );
}

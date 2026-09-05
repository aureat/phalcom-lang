use super::super::support::analyze_adt;
use phalcom_common::selector::Selector;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::match_semantics::{ExhaustivenessResult, PatternSpaceSummary, PatternUsefulness};

#[test]
fn exhaustive_match_proves_full_coverage() {
    let case = analyze_adt(
        r#"
enum Option<T> {
    @variant Some(_ value: T) -> Option<T>
    @variant None -> Option<T>
}

class Test {
    check(_ opt: Option<Int>) {
        match opt {
            Option::Some(x) => x
            Option::None => 0
        }
    }
}
"#,
    );
    case.only_match().assert_exhaustive();
}

#[test]
fn non_exhaustive_match_retains_structured_missing_witness() {
    let case = analyze_adt(
        r#"
enum Option<T> {
    @variant Some(_ value: T) -> Option<T>
    @variant None -> Option<T>
}

class Test {
    check(_ opt: Option<Int>) {
        match opt {
            Option::Some(x) => x
        }
    }
}
"#,
    );
    let handle = case.only_match();
    let ExhaustivenessResult::Missing(witnesses) = &handle.resolution().exhaustiveness else {
        panic!("expected structured missing witness, got {:#?}", handle.resolution().exhaustiveness);
    };
    assert!(!witnesses.is_empty());
    assert_eq!(case.diagnostics_for(DiagnosticCode::MatchNonExhaustive).len(), 1);
}

#[test]
fn redundant_arm_is_recorded_in_match_product_before_diagnostic_rendering() {
    let case = analyze_adt(
        r#"
enum Choice {
    @variant A -> Choice
    @variant B -> Choice
}

class Test {
    inspect(_ value: Choice) {
        match value {
            _ => 1
            Choice::A => 2
        }
    }
}
"#,
    );
    let handle = case.only_match();
    assert_eq!(handle.arm(0).resolution().usefulness, PatternUsefulness::Useful);
    assert_eq!(handle.arm(1).resolution().usefulness, PatternUsefulness::Redundant);
    assert_eq!(case.diagnostics_for(DiagnosticCode::MatchArmRedundant).len(), 1);
}

#[test]
fn gadt_impossible_case_is_not_reported_as_missing_coverage() {
    let case = analyze_adt(
        r#"
enum Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
    @variant Bool(_ value: Bool) -> Expr<Bool>
}

class Test {
    inspect(_ value: Expr<Int>) {
        match value {
            Expr::Int(x) => x
        }
    }
}
"#,
    );
    let handle = case.only_match();
    handle.assert_exhaustive();
    assert_eq!(handle.resolution().arms[0].usefulness, PatternUsefulness::Useful);
}

#[test]
fn redundant_arm_does_not_widen_match_result_product() {
    let case = analyze_adt(
        r#"
enum Choice {
    @variant A -> Choice
    @variant B -> Choice
}

class Test {
    inspect(_ value: Choice) {
        match value {
            _ => 1
            Choice::A => "unreachable"
        }
    }
}
"#,
    );
    let handle = case.only_match();
    assert_eq!(handle.arm(1).resolution().usefulness, PatternUsefulness::Redundant);
    assert_eq!(handle.resolution().result, handle.arm(0).resolution().branch_result);
}

#[test]
fn match_exh_03_singleton_coverage_does_not_cover_nullary_constructor() {
    let case = analyze_adt("enum Animal { @variant Dog @variant Dog() }\nclass Test { run(_ value: Animal) { match value { Animal::Dog => 1 } } }\n");
    let handle = case.only_match();
    handle.assert_not_exhaustive();
    assert!(!case.diagnostics_for(DiagnosticCode::MatchNonExhaustive).is_empty());
}

#[test]
fn match_exh_04_distinct_payload_constructors_are_each_required() {
    let case = analyze_adt(
        "enum Result { @variant Ok(_ value: Int) @variant Error(_ value: String) }\nclass Test { run(_ value: Result) { match value { Result::Ok(x) => x } } }\n",
    );
    case.only_match().assert_not_exhaustive();
}

#[test]
fn match_exh_05_family_pattern_covers_family_but_not_sibling() {
    let case =
        analyze_adt("enum Animal { @variant Dog @variant Dog() @variant Cat }\nclass Test { run(_ value: Animal) { match value { Dog* => 1 Cat => 2 } } }\n");
    case.only_match().assert_exhaustive();
}

#[test]
fn match_exh_06_callable_family_leaves_singleton_residual() {
    let case = analyze_adt("enum Animal { @variant Dog @variant Dog() @variant Cat }\nclass Test { run(_ value: Animal) { match value { Dog(...) => 1 } } }\n");
    case.only_match().assert_not_exhaustive();
}

#[test]
fn match_exh_07_exact_case_scrutinee_requires_only_that_case() {
    let case = analyze_adt(
        "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Test { run(_ value: Expr<Int>) { match value { Expr::Int(x) => x } } }\n",
    );
    case.only_match().assert_exhaustive();
}

#[test]
#[ignore = "GATED: transparent alias union fixture is required"]
fn match_exh_08_alias_union_exhaustiveness_is_not_root_widened() {
    let case = analyze_adt(
        "enum Choice { @variant A @variant B }\ntype ChoiceAlias = Choice\nclass Test { run(_ value: ChoiceAlias) { match value { Choice::A => 1 Choice::B => 2 } } }\n",
    );
    case.only_match().assert_exhaustive();
}

#[test]
fn match_exh_09_mixed_closed_and_opaque_union_retains_opaque_witness() {
    let case = analyze_adt("enum Choice { @variant A @variant B }\nclass Test { run(_ value: Object) { match value { Choice::A => 1 } } }\n");
    case.only_match().assert_not_exhaustive();
}

#[test]
fn match_exh_10_wildcard_closes_opaque_residual() {
    let case = analyze_adt("class Test { run(_ value: Object) { match value { _ => 1 } } }\n");
    case.only_match().assert_exhaustive();
}

#[test]
fn match_exh_11_nested_totality_preserves_child_coverage() {
    let case = analyze_adt(
        "enum Option<T> { @variant Some(_ value: T) -> Option<T> @variant None -> Option<T> }\nenum Choice { @variant Left @variant Right }\nclass Test { run(_ value: Option<Choice>) { match value { Some(Choice::Left) => 1 Some(Choice::Right) => 2 None => 0 } } }\n",
    );
    case.only_match().assert_exhaustive();
}

#[test]
fn match_exh_12_nested_missing_witness_preserves_shape() {
    let case = analyze_adt(
        "enum Option<T> { @variant Some(_ value: T) -> Option<T> @variant None -> Option<T> }\nenum Choice { @variant Left @variant Right }\nclass Test { run(_ value: Option<Choice>) { match value { Some(Choice::Left) => 1 None => 0 } } }\n",
    );
    let handle = case.only_match();
    let ExhaustivenessResult::Missing(witnesses) = &handle.resolution().exhaustiveness else {
        panic!("expected nested witness")
    };
    assert!(!witnesses.is_empty());
}

#[test]
fn match_exh_13_tuple_product_is_exhaustive_when_all_products_are_covered() {
    let case = analyze_adt(
        r#"
enum Choice { @variant Left @variant Right }
class Test {
    run(_ value: (Choice, Choice)) {
        match value {
            (Choice::Left, Choice::Left) => 1
            (Choice::Left, Choice::Right) => 2
            (Choice::Right, Choice::Left) => 3
            (Choice::Right, Choice::Right) => 4
        }
    }
}
"#,
    );
    case.only_match().assert_exhaustive();
}

#[test]
fn match_exh_14_list_partition_is_exhaustive() {
    let case = analyze_adt(
        r#"
class Test {
    run(_ value: List<Int>) {
        match value {
            [] => 0
            [head, *tail] => 1
        }
    }
}
"#,
    );
    case.only_match().assert_exhaustive();
}

#[test]
#[ignore = "GATED: open object domain fixture is required"]
fn match_exh_15_open_object_requires_wildcard() {
    let case = analyze_adt("class Test { run(_ value: Object) { match value { #{name: value} => value } } }\n");
    case.only_match().assert_not_exhaustive();
}

#[test]
fn match_use_02_exact_duplicate_arm_is_redundant() {
    let case =
        analyze_adt("enum Choice { @variant A @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A => 1 Choice::A => 2 _ => 0 } } }\n");
    assert_eq!(case.only_match().arm(1).resolution().usefulness, PatternUsefulness::Redundant);
}

#[test]
fn match_use_03_family_subsumes_exact_member() {
    let case = analyze_adt(
        "enum Animal { @variant Dog @variant Dog() @variant Cat }\nclass Test { run(_ value: Animal) { match value { Dog* => 1 Dog() => 2 _ => 0 } } }\n",
    );
    assert_eq!(case.only_match().arm(1).resolution().usefulness, PatternUsefulness::Redundant);
}

#[test]
fn match_use_04_duplicate_or_alternative_is_redundant() {
    let case = analyze_adt(
        "enum Choice { @variant Left @variant Right }\nclass Test { run(_ value: Choice) { match value { Choice::Left | Choice::Left => 1 _ => 0 } } }\n",
    );
    assert!(!case.only_match().resolution().arms.is_empty());
}

#[test]
fn match_use_05_family_alternative_subsumes_exact_member() {
    let case = analyze_adt(
        "enum Animal { @variant Dog @variant Dog() @variant Cat }\nclass Test { run(_ value: Animal) { match value { Dog* => 1 Dog => 2 _ => 0 } } }\n",
    );
    assert_eq!(case.only_match().arm(1).resolution().usefulness, PatternUsefulness::Redundant);
}

#[test]
fn match_imp_01_gadt_impossible_arm_is_classified_impossible() {
    let case = analyze_adt(
        "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Test { run(_ value: Expr<Int>) { match value { Expr::Bool(x) => x Expr::Int(x) => x } } }\n",
    );
    assert_eq!(case.only_match().arm(0).resolution().usefulness, PatternUsefulness::Impossible);
}

#[test]
fn match_imp_02_disjoint_union_pattern_is_impossible() {
    let case = analyze_adt(
        "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Test { run(_ value: Expr<Int>) { match value { Expr::Bool(x) => x } } }\n",
    );
    assert_eq!(case.only_match().arm(0).resolution().usefulness, PatternUsefulness::Impossible);
}

#[test]
fn review_m5_01_two_field_witness_keeps_both_fields() {
    let case = analyze_adt("enum Pair { @variant Both(_ left: Int, right: String) }\nclass Test { run(_ value: Pair) { match value { } } }\n");
    let handle = case.only_match();
    let phalcom_semantic::match_semantics::ExhaustivenessResult::Missing(witnesses) = &handle.resolution().exhaustiveness else {
        panic!("expected missing witness")
    };
    assert!(
        witnesses
            .iter()
            .any(|witness| matches!(witness, phalcom_semantic::match_semantics::CoverageWitness::Variant { fields, .. } if fields.len() == 2))
    );
}

#[test]
#[ignore = "RED: multi-field residual witness generation remains incomplete"]
fn review_m5_02_missing_two_field_combination_is_complete() {
    let case = analyze_adt(
        "enum Pair { @variant Both(_ left: Int, right: String) }\nclass Test { run(_ value: Pair) { match value { Pair::Both(_, right: \"ok\") => 1 } } }\n",
    );
    let handle = case.only_match();
    let ExhaustivenessResult::Missing(witnesses) = &handle.resolution().exhaustiveness else {
        panic!("expected pair witness")
    };
    assert!(!witnesses.is_empty());
}

#[test]
#[ignore = "RED: nested multi-field witness generation remains incomplete"]
fn review_m5_03_nested_multi_field_witness_preserves_child_tree() {
    let case = analyze_adt(
        "enum Pair { @variant Both(_ left: Int, right: String) }\nenum Outer { @variant Boxed(_ value: Pair) }\nclass Test { run(_ value: Outer) { match value { Outer::Boxed(Pair::Both(_, \"ok\")) => 1 } } }\n",
    );
    let handle = case.only_match();
    let ExhaustivenessResult::Missing(witnesses) = &handle.resolution().exhaustiveness else {
        panic!("expected nested pair witness")
    };
    assert!(!witnesses.is_empty());
}

#[test]
#[ignore = "GATED: labeled witness renderer fixture is required"]
fn review_m5_04_labeled_multi_field_witness_maps_external_labels() {
    let case = analyze_adt(
        "enum Pair { @variant Both(left: Int, right: String) }\nclass Test { run(_ value: Pair) { match value { Pair::Both(left: _, right: \"ok\") => 1 } } }\n",
    );
    let handle = case.only_match();
    let ExhaustivenessResult::Missing(witnesses) = &handle.resolution().exhaustiveness else {
        panic!("expected labeled witness")
    };
    assert!(!witnesses.is_empty());
}

#[test]
fn review_m5_05_witness_generation_is_deterministic() {
    let source = "enum Choice { @variant A @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A => 1 } } }\n";
    let first = analyze_adt(source);
    let second = analyze_adt(source);
    assert_eq!(first.only_match().resolution().exhaustiveness, second.only_match().resolution().exhaustiveness);
}

#[test]
fn review_m5_06_witness_is_representative_while_residual_product_stays_structured() {
    let case = analyze_adt("enum Choice { @variant A @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A => 1 } } }\n");
    let handle = case.only_match();
    let phalcom_semantic::match_semantics::ExhaustivenessResult::Missing(witnesses) = &handle.resolution().exhaustiveness else {
        panic!("expected witness")
    };
    assert!(!witnesses.is_empty());
    assert!(!matches!(
        handle.resolution().arms[0].residual_after,
        phalcom_semantic::match_semantics::PatternSpaceSummary::Empty
    ));
}

#[test]
fn r2_t01_residual_after_first_arm_is_prior_matrix_residual() {
    let case = analyze_adt(
        "enum Choice { @variant A @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A => 1 Choice::B => 2 } } }\n",
    );
    let handle = case.only_match();
    let residual = &handle.resolution().arms[0].residual_after;
    let expected_b = case.variant_id("Choice", Selector::getter("B").expect("selector"));
    assert!(matches!(residual, PatternSpaceSummary::Variant { variant, .. } if *variant == expected_b));
    assert!(!matches!(residual, PatternSpaceSummary::Variant { variant, .. } if variant.selector == Selector::getter("A").expect("selector")));
}

#[test]
fn r2_t02_exhaustive_match_has_empty_final_residual() {
    let case = analyze_adt(
        "enum Choice { @variant A @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A => 1 Choice::B => 2 } } }\n",
    );
    let handle = case.only_match();
    assert_eq!(handle.resolution().arms.last().unwrap().residual_after, PatternSpaceSummary::Empty);
}

#[test]
fn r2_t06_empty_match_witnesses_are_bounded() {
    let variants = (0..12).map(|index| format!("@variant C{index}")).collect::<Vec<_>>().join(" ");
    let source = format!(
        "enum Wide {{ {variants} }}\nclass Test {{ run(_ value: Wide) {{ match value {{ }} }} }}\n"
    );
    let case = analyze_adt(&source);
    let handle = case.only_match();
    let ExhaustivenessResult::Missing(witnesses) = &handle.resolution().exhaustiveness else {
        panic!("expected missing witnesses")
    };
    assert!(witnesses.len() <= 8, "witness bound exceeded: {}", witnesses.len());
}

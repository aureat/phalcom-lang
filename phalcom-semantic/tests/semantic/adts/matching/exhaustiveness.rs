use super::super::support::analyze_adt;
use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::match_semantics::{ExhaustivenessResult, PatternSpaceSummary, PatternUsefulness};

#[test]
fn exhaustive_match_proves_full_coverage() {
    let case = analyze_adt(
        r#"
data Option<T> {
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
fn canonical_result_name_self_match_proves_contextual_and_qualified_cases_equivalently() {
    let case = analyze_adt(
        r#"
native data Result<T, E> {
    @variant Ok(_ value: T) -> Result<T, E>
    @variant Error(_ error: E) -> Result<T, E>
}

native data Option<T> {
    @variant Some(_ value: T) -> Option<T>
    @variant None -> Option<T>
}

impl<T, E> Result<T, E> {
    match<R>(
        ok: (value: T) -> R,
        err: (error: E) -> R
    ) -> R {
        match self {
            Ok(value) => ok.call(value)
            Error(error) => err.call(error)
        }
    }

    matchQualified<R>(
        ok: (value: T) -> R,
        err: (error: E) -> R
    ) -> R {
        match self {
            Result::Ok(value) => ok.call(value)
            Result::Error(error) => err.call(error)
        }
    }

    transpose<U>() -> Option<Result<U, E>> where T == Option<U> {
        match self {
            Result::Ok(option) => {
                match option {
                    Option::Some(value) => Option::Some(Result::Ok(value))
                    Option::None => Option::None
                }
            }
            Result::Error(error) => Option::Some(Result::Error(error))
        }
    }

    directConstrainedOption<U>(_ option: T) where T == Option<U> {
        match option {
            Option::Some(_) => 1
            Option::None => 0
        }
    }

    reverseConstrainedOption<U>(_ option: T) where Option<U> == T {
        match option {
            Option::Some(_) => 1
            Option::None => 0
        }
    }

    directChainedConstrainedOption<U, V>(_ option: T) where T == V, V == Option<U> {
        match option {
            Option::Some(_) => 1
            Option::None => 0
        }
    }
}
"#,
    );

    let slots = [SelectorSlot::Label("ok".into()), SelectorSlot::Label("err".into())];
    let contextual = case.match_in_callable("Result", Selector::method("match", slots.clone()).expect("match selector"), 0);
    let qualified = case.match_in_callable(
        "Result",
        Selector::method("matchQualified", slots).expect("matchQualified selector"),
        0,
    );

    contextual.assert_exhaustive();
    qualified.assert_exhaustive();

    let expected = [
        case.variant_id("Result", Selector::method("Ok", [SelectorSlot::Positional]).expect("Ok selector")),
        case.variant_id("Result", Selector::method("Error", [SelectorSlot::Positional]).expect("Error selector")),
    ];
    contextual.assert_arm_candidate_variants(0, &expected[..1]);
    contextual.assert_arm_candidate_variants(1, &expected[1..]);
    qualified.assert_arm_candidate_variants(0, &expected[..1]);
    qualified.assert_arm_candidate_variants(1, &expected[1..]);

    let transpose_outer = case.match_in_callable("Result", Selector::method("transpose", []).expect("transpose selector"), 0);
    let transpose_inner = case.match_in_callable("Result", Selector::method("transpose", []).expect("transpose selector"), 1);
    let direct = case.match_in_callable(
        "Result",
        Selector::method("directConstrainedOption", [SelectorSlot::Positional]).expect("directConstrainedOption selector"),
        0,
    );
    let chained_direct = case.match_in_callable(
        "Result",
        Selector::method("directChainedConstrainedOption", [SelectorSlot::Positional])
            .expect("directChainedConstrainedOption selector"),
        0,
    );
    let reverse_direct = case.match_in_callable(
        "Result",
        Selector::method("reverseConstrainedOption", [SelectorSlot::Positional])
            .expect("reverseConstrainedOption selector"),
        0,
    );
    transpose_outer.assert_exhaustive();
    direct.assert_exhaustive();
    chained_direct.assert_exhaustive();
    reverse_direct.assert_exhaustive();
    let transpose_ok_arm = transpose_outer.arm(0);
    let option_binding = transpose_ok_arm.find_binding("option").expect("Ok arm binds option");
    let option_ty = option_binding.knowledge.ty().expect("option has a canonical type");
    assert_eq!(
        case.analysis
            .snapshot
            .store
            .nominal_origin_declaration(option_ty)
            .map(|declaration| declaration.name.as_ref()),
        Some("Option"),
        "the Result::Ok payload must be normalized under transpose's callable equality before it enters the arm scope",
    );
    transpose_inner.assert_exhaustive();
    transpose_inner.assert_arm_candidate_variants(
        0,
        &[case.variant_id("Option", Selector::method("Some", [SelectorSlot::Positional]).expect("Some selector"))],
    );
    transpose_inner.assert_arm_candidate_variants(
        1,
        &[case.variant_id("Option", Selector::getter("None").expect("None selector"))],
    );
}

#[test]
fn non_exhaustive_match_retains_structured_missing_witness() {
    let case = analyze_adt(
        r#"
data Option<T> {
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
data Choice {
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
data Expr<T> {
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
data Choice {
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
    let case = analyze_adt("data Animal { @variant Dog @variant Dog() }\nclass Test { run(_ value: Animal) { match value { Animal::Dog => 1 } } }\n");
    let handle = case.only_match();
    handle.assert_not_exhaustive();
    assert!(!case.diagnostics_for(DiagnosticCode::MatchNonExhaustive).is_empty());
}

#[test]
fn match_exh_04_distinct_payload_constructors_are_each_required() {
    let case = analyze_adt(
        "data Result { @variant Ok(_ value: Int) @variant Error(_ value: String) }\nclass Test { run(_ value: Result) { match value { Result::Ok(x) => x } } }\n",
    );
    case.only_match().assert_not_exhaustive();
}

#[test]
fn match_exh_05_family_pattern_covers_family_but_not_sibling() {
    let case =
        analyze_adt("data Animal { @variant Dog @variant Dog() @variant Cat }\nclass Test { run(_ value: Animal) { match value { Dog* => 1 Cat => 2 } } }\n");
    case.only_match().assert_exhaustive();
}

#[test]
fn match_exh_06_callable_family_leaves_singleton_residual() {
    let case = analyze_adt("data Animal { @variant Dog @variant Dog() @variant Cat }\nclass Test { run(_ value: Animal) { match value { Dog(...) => 1 } } }\n");
    case.only_match().assert_not_exhaustive();
}

#[test]
fn match_exh_07_exact_case_scrutinee_requires_only_that_case() {
    let case = analyze_adt(
        "data Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Test { run(_ value: Expr<Int>) { match value { Expr::Int(x) => x } } }\n",
    );
    case.only_match().assert_exhaustive();
}

#[test]
#[ignore = "GATED: transparent alias union fixture is required"]
fn match_exh_08_alias_union_exhaustiveness_is_not_root_widened() {
    let case = analyze_adt(
        "data Choice { @variant A @variant B }\ntype ChoiceAlias = Choice\nclass Test { run(_ value: ChoiceAlias) { match value { Choice::A => 1 Choice::B => 2 } } }\n",
    );
    case.only_match().assert_exhaustive();
}

#[test]
fn match_exh_09_mixed_closed_and_opaque_union_retains_opaque_witness() {
    let case = analyze_adt("data Choice { @variant A @variant B }\nclass Test { run(_ value: Object) { match value { Choice::A => 1 } } }\n");
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
        "data Option<T> { @variant Some(_ value: T) -> Option<T> @variant None -> Option<T> }\ndata Choice { @variant Left @variant Right }\nclass Test { run(_ value: Option<Choice>) { match value { Some(Choice::Left) => 1 Some(Choice::Right) => 2 None => 0 } } }\n",
    );
    case.only_match().assert_exhaustive();
}

#[test]
fn match_exh_12_nested_missing_witness_preserves_shape() {
    let case = analyze_adt(
        "data Option<T> { @variant Some(_ value: T) -> Option<T> @variant None -> Option<T> }\ndata Choice { @variant Left @variant Right }\nclass Test { run(_ value: Option<Choice>) { match value { Some(Choice::Left) => 1 None => 0 } } }\n",
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
data Choice { @variant Left @variant Right }
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
    let case = analyze_adt("class Test { run(_ value: Object) { match value { {name: value} => value } } }\n");
    case.only_match().assert_not_exhaustive();
}

#[test]
fn match_use_02_exact_duplicate_arm_is_redundant() {
    let case =
        analyze_adt("data Choice { @variant A @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A => 1 Choice::A => 2 _ => 0 } } }\n");
    assert_eq!(case.only_match().arm(1).resolution().usefulness, PatternUsefulness::Redundant);
}

#[test]
fn match_use_03_family_subsumes_exact_member() {
    let case = analyze_adt(
        "data Animal { @variant Dog @variant Dog() @variant Cat }\nclass Test { run(_ value: Animal) { match value { Dog* => 1 Dog() => 2 _ => 0 } } }\n",
    );
    assert_eq!(case.only_match().arm(1).resolution().usefulness, PatternUsefulness::Redundant);
}

#[test]
fn match_use_04_duplicate_or_alternative_is_redundant() {
    let case = analyze_adt(
        "data Choice { @variant Left @variant Right }\nclass Test { run(_ value: Choice) { match value { Choice::Left | Choice::Left => 1 _ => 0 } } }\n",
    );
    assert!(!case.only_match().resolution().arms.is_empty());
}

#[test]
fn match_use_05_family_alternative_subsumes_exact_member() {
    let case = analyze_adt(
        "data Animal { @variant Dog @variant Dog() @variant Cat }\nclass Test { run(_ value: Animal) { match value { Dog* => 1 Dog => 2 _ => 0 } } }\n",
    );
    assert_eq!(case.only_match().arm(1).resolution().usefulness, PatternUsefulness::Redundant);
}

#[test]
fn match_imp_01_gadt_impossible_arm_is_classified_impossible() {
    let case = analyze_adt(
        "data Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Test { run(_ value: Expr<Int>) { match value { Expr::Bool(x) => x Expr::Int(x) => x } } }\n",
    );
    assert_eq!(case.only_match().arm(0).resolution().usefulness, PatternUsefulness::Impossible);
}

#[test]
fn match_imp_02_disjoint_union_pattern_is_impossible() {
    let case = analyze_adt(
        "data Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Test { run(_ value: Expr<Int>) { match value { Expr::Bool(x) => x } } }\n",
    );
    assert_eq!(case.only_match().arm(0).resolution().usefulness, PatternUsefulness::Impossible);
}

#[test]
fn review_m5_01_two_field_witness_keeps_both_fields() {
    let case = analyze_adt("data Pair { @variant Both(_ left: Int, right: String) }\nclass Test { run(_ value: Pair) { match value { } } }\n");
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
        "data Pair { @variant Both(_ left: Int, right: String) }\nclass Test { run(_ value: Pair) { match value { Pair::Both(_, right: \"ok\") => 1 } } }\n",
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
        "data Pair { @variant Both(_ left: Int, right: String) }\ndata Outer { @variant Boxed(_ value: Pair) }\nclass Test { run(_ value: Outer) { match value { Outer::Boxed(Pair::Both(_, \"ok\")) => 1 } } }\n",
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
        "data Pair { @variant Both(left: Int, right: String) }\nclass Test { run(_ value: Pair) { match value { Pair::Both(left: _, right: \"ok\") => 1 } } }\n",
    );
    let handle = case.only_match();
    let ExhaustivenessResult::Missing(witnesses) = &handle.resolution().exhaustiveness else {
        panic!("expected labeled witness")
    };
    assert!(!witnesses.is_empty());
}

#[test]
fn review_m5_05_witness_generation_is_deterministic() {
    let source = "data Choice { @variant A @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A => 1 } } }\n";
    let first = analyze_adt(source);
    let second = analyze_adt(source);
    assert_eq!(first.only_match().resolution().exhaustiveness, second.only_match().resolution().exhaustiveness);
}

#[test]
fn review_m5_06_witness_is_representative_while_residual_product_stays_structured() {
    let case = analyze_adt("data Choice { @variant A @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A => 1 } } }\n");
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
    let case = analyze_adt("data Choice { @variant A @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A => 1 Choice::B => 2 } } }\n");
    let handle = case.only_match();
    let residual = &handle.resolution().arms[0].residual_after;
    let expected_b = case.variant_id("Choice", Selector::getter("B").expect("selector"));
    assert!(matches!(residual, PatternSpaceSummary::Variant { variant, .. } if *variant == expected_b));
    assert!(!matches!(residual, PatternSpaceSummary::Variant { variant, .. } if variant.selector == Selector::getter("A").expect("selector")));
}

#[test]
fn r2_t02_exhaustive_match_has_empty_final_residual() {
    let case = analyze_adt("data Choice { @variant A @variant B }\nclass Test { run(_ value: Choice) { match value { Choice::A => 1 Choice::B => 2 } } }\n");
    let handle = case.only_match();
    assert_eq!(handle.resolution().arms.last().unwrap().residual_after, PatternSpaceSummary::Empty);
}

#[test]
fn r2_t06_empty_match_witnesses_are_bounded() {
    let variants = (0..12).map(|index| format!("@variant C{index}")).collect::<Vec<_>>().join(" ");
    let source = format!("data Wide {{ {variants} }}\nclass Test {{ run(_ value: Wide) {{ match value {{ }} }} }}\n");
    let case = analyze_adt(&source);
    let handle = case.only_match();
    let ExhaustivenessResult::Missing(witnesses) = &handle.resolution().exhaustiveness else {
        panic!("expected missing witnesses")
    };
    assert!(witnesses.len() <= 8, "witness bound exceeded: {}", witnesses.len());
}

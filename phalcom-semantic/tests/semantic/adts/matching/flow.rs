//! Match result typing and branch-flow scenarios.

use super::super::support::analyze_adt;
use phalcom_common::selector::Selector;
use phalcom_semantic::match_semantics::PatternSpaceSummary;

#[test]
fn match_product_exposes_result_knowledge_separately_from_arm_products() {
    let case = analyze_adt(
        r#"
enum Choice {
    @variant Left(_ value: Int) -> Choice
    @variant Right(_ value: String) -> Choice
}

class Test {
    inspect(_ value: Choice) {
        match value {
            Choice::Left(x) => x
            Choice::Right(y) => y
        }
    }
}
"#,
    );
    let handle = case.match_in_callable(
        "Test",
        Selector::method("inspect", [phalcom_common::selector::SelectorSlot::Positional]).expect("selector"),
        0,
    );
    assert_eq!(handle.resolution().arms.len(), 2);
    assert!(handle.resolution().result.ty().is_some(), "match result must retain structured type knowledge");
}

#[test]
fn match_flow_01_homogeneous_arms_join_to_one_type() {
    let case = analyze_adt(
        "enum Choice { @variant Left @variant Right }\nclass Test { run(_ value: Choice) { match value { Choice::Left => 1 Choice::Right => 2 } } }\n",
    );
    assert_eq!(case.only_match().resolution().result.ty(), Some(case.declaration("Int").form));
}

#[test]
fn match_flow_02_heterogeneous_arms_publish_canonical_union() {
    let case = analyze_adt(
        "enum Choice { @variant Left @variant Right }\nclass Test { run(_ value: Choice) { match value { Choice::Left => 1 Choice::Right => \"two\" } } }\n",
    );
    let result = case.only_match().resolution().result.ty().expect("match result type");
    assert!(matches!(case.type_data(result), phalcom_semantic::types::store::TypeData::Union(_)));
}

#[test]
fn match_flow_07_variant_arm_publishes_stable_binding_refinement() {
    let case = analyze_adt(
        "enum Option<T> { @variant Some(_ value: T) -> Option<T> @variant None -> Option<T> }\nclass Test { run(_ value: Option<Int>) { match value { Option::Some(x) => x Option::None => 0 } } }\n",
    );
    let handle = case.only_match();
    handle.arm(0).assert_binding_type("x", case.declaration("Int").form);
    assert!(!matches!(
        handle.arm(0).resolution().reachable_space,
        phalcom_semantic::match_semantics::PatternSpaceSummary::Empty
    ));
}

#[test]
fn match_flow_08_family_arm_keeps_candidate_space_in_match_product() {
    let case =
        analyze_adt("enum Animal { @variant Dog @variant Dog() @variant Cat }\nclass Test { run(_ value: Animal) { match value { Dog* => 1 Cat => 2 } } }\n");
    let handle = case.only_match();
    assert!(!matches!(
        handle.arm(0).resolution().reachable_space,
        phalcom_semantic::match_semantics::PatternSpaceSummary::Empty
    ));
    assert!(!matches!(
        handle.arm(0).resolution().residual_after,
        phalcom_semantic::match_semantics::PatternSpaceSummary::Empty
    ));
}

#[test]
fn match_flow_09_bindings_are_arm_local() {
    let case = analyze_adt(
        "enum Choice { @variant Left(_ value: Int) @variant Right }\nclass Test { run(_ value: Choice) { match value { Choice::Left(x) => x Choice::Right => 0 } } }\n",
    );
    let handle = case.only_match();
    handle.arm(0).assert_binding_names(&["x"]);
    handle.arm(1).assert_no_binding("x");
}

#[test]
fn match_flow_11_later_arm_residual_excludes_earlier_case() {
    let case = analyze_adt(
        "enum Choice { @variant Left @variant Right }\nclass Test { run(_ value: Choice) { match value { Choice::Left => 1 Choice::Right => 2 } } }\n",
    );
    let handle = case.only_match();
    assert!(handle.arm(1).resolution().reachable_space != handle.arm(0).resolution().reachable_space);
}

#[test]
fn match_flow_12_nested_residual_is_not_forced_to_root_type() {
    let case = analyze_adt(
        "enum Option<T> { @variant Some(_ value: T) -> Option<T> @variant None -> Option<T> }\nenum Choice { @variant Left @variant Right }\nclass Test { run(_ value: Option<Choice>) { match value { Some(Choice::Left) => 1 None => 0 } } }\n",
    );
    assert!(matches!(
        case.only_match().resolution().exhaustiveness,
        phalcom_semantic::match_semantics::ExhaustivenessResult::Missing(_)
    ));
}

#[test]
fn r2_t03_nested_reachable_summary_uses_payload_subject() {
    let case = analyze_adt(
        "enum Option<T> { @variant Some(_ value: T) -> Option<T> @variant None -> Option<T> }\nclass Test { run(_ value: Option<Int>) { match value { Some(_) => 1 None => 0 } } }\n",
    );
    let handle = case.only_match();
    let reachable = &handle.resolution().arms[0].reachable_space;
    let PatternSpaceSummary::Variant { fields, .. } = reachable else {
        panic!("expected variant reachable summary, got {reachable:?}")
    };
    assert_eq!(fields.as_ref(), &[PatternSpaceSummary::Opaque(case.declaration("Int").form)]);
}

#[test]
#[ignore = "GATED: abrupt branch fixture is required"]
fn match_flow_03_abrupt_arm_is_excluded_from_normal_result_join() {
    let case = analyze_adt(
        "enum Choice { @variant Left @variant Right }\nclass Test { run(_ value: Choice) -> Int { match value { Choice::Left => return 1 Choice::Right => 2 } } }\n",
    );
    assert_eq!(case.only_match().resolution().result.ty(), Some(case.declaration("Int").form));
}

#[test]
#[ignore = "GATED: all-abrupt branch fixture is required"]
fn match_flow_04_all_abrupt_arms_have_never_result() {
    let case = analyze_adt(
        "enum Choice { @variant Left @variant Right }\nclass Test { run(_ value: Choice) -> Int { match value { Choice::Left => return 1 Choice::Right => return 2 } } }\n",
    );
    assert!(case.only_match().resolution().result.ty().is_some());
}

#[test]
fn match_flow_05_expected_type_is_checked_per_reachable_arm() {
    let case = analyze_adt(
        "enum Choice { @variant Left @variant Right }\nclass Test { run(_ value: Choice) -> Int { match value { Choice::Left => 1 Choice::Right => 2 } } }\n",
    );
    assert_eq!(case.only_match().resolution().result.ty(), Some(case.declaration("Int").form));
}

#[test]
fn match_flow_06_wrong_branch_result_points_to_offending_arm() {
    let case = analyze_adt(
        "enum Choice { @variant Left @variant Right }\nclass Test { run(_ value: Choice) -> Int { match value { Choice::Left => 1 Choice::Right => \"wrong\" } } }\n",
    );
    assert!(!case.diagnostics().collect::<Vec<_>>().is_empty());
}

#[test]
#[ignore = "GATED: outer variable write/join fixture is required"]
fn match_flow_10_branch_writes_join_after_match() {
    let case = analyze_adt(
        "enum Choice { @variant Left @variant Right }\nclass Test { run(_ value: Choice) { let total = match value { Choice::Left => 1 Choice::Right => 2 } total } }\n",
    );
    assert_eq!(case.only_match().resolution().result.ty(), Some(case.declaration("Int").form));
}

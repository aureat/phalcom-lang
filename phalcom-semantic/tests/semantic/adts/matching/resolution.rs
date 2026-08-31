use std::collections::BTreeMap;
use std::sync::Arc;

use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::analyze_single_module;
use super::super::support::analyze_adt;
use phalcom_semantic::identity::{BindingId, BodyId, DeclarationId, ExpressionId, LocalExpressionId, VariantFamilyId, VariantId};
use phalcom_semantic::match_semantics::{
    BranchProofEnvironment, ExhaustivenessResult, MatchArmResolution, MatchResolution, PatternBindingResolution, PatternResolution, PatternSpaceSummary,
    PatternUsefulness, ResolvedVariantCandidate, ResolvedVariantPattern, VariantSelectorConstraint,
};
use phalcom_semantic::types::evidence::{EvidenceOrigin, TypeKnowledge};
use phalcom_semantic::types::id::TypeId;

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

#[test]
fn match_product_model_shape() {
    let dummy_range = SourceRange::new(0, 10);
    let module = test_module();
    let decl = DeclarationId::new(module.clone(), "Option".into());
    let var_id = VariantId::new(decl.clone(), Selector::getter("Some").expect("selector"));
    let fam_id = VariantFamilyId::new(decl.clone(), "Some");

    let match_res = MatchResolution {
        expression: ExpressionId::new(BodyId(1), LocalExpressionId(1)),
        scrutinee: TypeKnowledge::established(TypeId(10), EvidenceOrigin::Flow),
        initial_space: PatternSpaceSummary::Variant {
            variant: var_id.clone(),
            exact_case: TypeId(11),
            fields: Box::new([PatternSpaceSummary::Opaque(TypeId(12))]),
        },
        arms: Box::new([MatchArmResolution {
            arm_index: 0,
            pattern: PatternResolution::Variant(ResolvedVariantPattern {
                owner: decl.clone(),
                family: fam_id,
                selector: VariantSelectorConstraint::WholeFamily,
                candidates: Box::new([ResolvedVariantCandidate {
                    variant: var_id,
                    exact_case: TypeId(11),
                    fields: Box::new([]),
                    proof: BranchProofEnvironment::default(),
                }]),
            }),
            reachable_space: PatternSpaceSummary::Empty,
            residual_after: PatternSpaceSummary::Empty,
            bindings: Box::new([PatternBindingResolution {
                binding: BindingId(1),
                name: "x".into(),
                knowledge: TypeKnowledge::established(TypeId(12), EvidenceOrigin::Flow),
                source: dummy_range,
            }]),
            proof: BranchProofEnvironment {
                bindings: BTreeMap::new(),
                equalities: Box::new([]),
            },
            usefulness: PatternUsefulness::Useful,
            branch_result: TypeKnowledge::established(TypeId(12), EvidenceOrigin::Flow),
        }]),
        result: TypeKnowledge::established(TypeId(12), EvidenceOrigin::Flow),
        exhaustiveness: ExhaustivenessResult::Proven,
    };

    assert_eq!(match_res.arms.len(), 1);
    assert_eq!(match_res.exhaustiveness, ExhaustivenessResult::Proven);
    assert!(match_res.proof_for_arm(0).is_some());
    assert!(match_res.arms[0].proof.is_empty());
}

#[test]
fn match_resolution_recorded_in_callable_analysis() {
    let source = r#"
class Test {
    run() {
        const x = 42
        const y = match x {
            _ => 1
        }
        y
    }
}
"#;
    let module = test_module();
    let parsed = phalcom_ast::parse_source(source, 0).expect("should parse cleanly");
    let analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed));
    let callable_id = phalcom_semantic::identity::CallableId::new(
        DeclarationId::new(module, "Test".into()),
        Selector::method("run", vec![]).expect("selector"),
        phalcom_semantic::identity::DispatchSide::Instance,
    );
    let callable_analysis = analysis.snapshot.callable_analyses.get(&callable_id).expect("should analyze Test::run");
    assert_eq!(callable_analysis.match_resolutions.len(), 1, "should record match resolution");
}

#[test]
fn match_res_01_wildcard_root_records_total_resolution() {
    let case = analyze_adt("enum Status { @variant Ready @variant Done }\nclass Test { run(_ value: Status) { match value { _ => 1 } } }\n");
    let handle = case.only_match();
    handle.assert_exhaustive();
    assert!(handle.resolution().arms[0].bindings.is_empty());
}

#[test]
fn match_res_02_qualified_singleton_has_one_exact_candidate() {
    let case = analyze_adt("enum Status { @variant Ready @variant Done }\nclass Test { run(_ value: Status) { match value { Status::Ready => 1 _ => 0 } } }\n");
    let handle = case.only_match();
    handle.assert_arm_candidate_variants(0, &[case.variant_id("Status", phalcom_common::selector::Selector::getter("Ready").expect("Ready"))]);
}

#[test]
fn match_res_03_qualified_nullary_has_constructor_identity() {
    let case = analyze_adt("enum Status { @variant Ready() @variant Done }\nclass Test { run(_ value: Status) { match value { Status::Ready() => 1 _ => 0 } } }\n");
    let handle = case.only_match();
    handle.assert_arm_candidate_variants(0, &[case.variant_id("Status", phalcom_common::selector::Selector::method("Ready", []).expect("Ready"))]);
}

#[test]
fn match_res_04_exact_positional_constructor_projects_field() {
    let case = analyze_adt("enum Status { @variant Ready(_ code: Int) @variant Done }\nclass Test { run(_ value: Status) { match value { Status::Ready(code) => code _ => 0 } } }\n");
    let handle = case.only_match();
    assert_eq!(handle.arm(0).resolution().bindings.len(), 1);
    handle.arm(0).assert_binding_names(&["code"]);
}

#[test]
fn match_res_05_exact_labeled_constructor_uses_external_label() {
    let case = analyze_adt("enum Status { @variant Ready(named code: Int) @variant Done }\nclass Test { run(_ value: Status) { match value { Status::Ready(named: code) => code _ => 0 } } }\n");
    let handle = case.only_match();
    handle.arm(0).assert_binding_names(&["code"]);
}

#[test]
fn match_res_06_contextual_some_uses_scrutinee_owner() {
    let case = analyze_adt("enum Option<T> { @variant Some(_ value: T) -> Option<T> @variant None -> Option<T> }\nclass Test { run(_ value: Option<Int>) { match value { Some(x) => x None => 0 } } }\n");
    let handle = case.only_match();
    handle.arm(0).assert_binding_names(&["x"]);
    handle.arm(0).assert_binding_type("x", case.declaration("Int").form);
}

#[test]
fn match_res_07_contextual_none_resolves_singleton_member() {
    let case = analyze_adt("enum Option<T> { @variant Some(_ value: T) -> Option<T> @variant None -> Option<T> }\nclass Test { run(_ value: Option<Int>) { match value { Some(_) => 1 None => 0 } } }\n");
    let handle = case.only_match();
    assert_eq!(handle.arm(1).resolution().bindings.len(), 0);
}

#[test]
#[ignore = "GATED: ambiguous contextual owner fixture requires multi-module/domain union support"]
fn match_res_08_ambiguous_contextual_owner_reports_no_arbitrary_candidate() {
    let case = analyze_adt("enum Left { @variant Same }\nenum Right { @variant Same }\nclass Test { run(_ value: Object) { match value { Same => 1 _ => 0 } } }\n");
    assert!(case.diagnostics().any(|diagnostic| diagnostic.code == phalcom_semantic::diagnostic::DiagnosticCode::MatchPatternUnresolved));
}

#[test]
fn match_res_09_nested_contextual_resolution_uses_specialized_payload_domain() {
    let case = analyze_adt("enum Result<T, E> { @variant Ok(_ value: T) -> Result<T, E> @variant Error(_ value: E) -> Result<T, E> }\nenum Option<T> { @variant Some(_ value: T) -> Option<T> @variant None -> Option<T> }\nclass Test { run(_ value: Option<Result<Int, String>>) { match value { Some(Ok(x)) => x Some(Error(x)) => 0 None => 0 } } }\n");
    let handle = case.only_match();
    assert_eq!(handle.arm(0).resolution().bindings.len(), 1);
}

#[test]
#[ignore = "RED: selector-family gap projection remains incomplete"]
fn match_res_10_callable_family_includes_only_callable_candidates() {
    let case = analyze_adt("enum Animal { @variant Dog @variant Dog(_ name: String) @variant Cat }\nclass Test { run(_ value: Animal) { match value { Dog(...) => 1 _ => 0 } } }\n");
    assert!(case.only_match().arm(0).resolution().bindings.is_empty());
}

#[test]
#[ignore = "RED: selector-family gap projection remains incomplete"]
fn match_res_11_prefix_gap_records_candidate_specific_projection() {
    let case = analyze_adt("enum Animal { @variant Dog(_ name: String) @variant Dog(_ name: String, age: Int, breed: String) }\nclass Test { run(_ value: Animal) { match value { Dog(name, ..., breed: b) => 1 _ => 0 } } }\n");
    assert!(matches!(case.only_match().arm(0).resolution().pattern, phalcom_semantic::match_semantics::PatternResolution::Variant(_)));
}

#[test]
#[ignore = "RED: selector-family gap projection remains incomplete"]
fn match_res_12_suffix_label_gap_matches_canonical_selector_pattern() {
    let case = analyze_adt("enum Animal { @variant Dog(_ name: String) @variant Dog(_ name: String, age: Int, breed: String) }\nclass Test { run(_ value: Animal) { match value { Dog(..., breed: b) => 1 _ => 0 } } }\n");
    assert!(matches!(case.only_match().arm(0).resolution().pattern, phalcom_semantic::match_semantics::PatternResolution::Variant(_)));
}

#[test]
#[ignore = "RED: selector-family gap projection remains incomplete"]
fn match_res_13_prefix_and_suffix_gap_join_field_projections() {
    let case = analyze_adt("enum Animal { @variant Dog(_ name: String) @variant Dog(_ name: String, age: Int, breed: String) }\nclass Test { run(_ value: Animal) { match value { Dog(name, ..., breed: b) => name _ => \"unknown\" } } }\n");
    assert!(matches!(case.only_match().arm(0).resolution().pattern, phalcom_semantic::match_semantics::PatternResolution::Variant(_)));
}

#[test]
fn match_res_14_whole_family_candidate_set_is_explicit() {
    let case = analyze_adt("enum Animal { @variant Dog @variant Dog() @variant Dog(_ name: String) @variant Cat }\nclass Test { run(_ value: Animal) { match value { Dog* => 1 Cat => 2 } } }\n");
    let handle = case.only_match();
    let candidates = match &handle.arm(0).resolution().pattern {
        phalcom_semantic::match_semantics::PatternResolution::Variant(pattern) => pattern.candidates.len(),
        pattern => panic!("expected family variant pattern, got {pattern:?}"),
    };
    assert_eq!(candidates, 3);
}

#[test]
fn match_res_15_candidate_order_is_stable_across_repeated_analysis() {
    let source = "enum Animal { @variant Dog @variant Dog() @variant Dog(_ name: String) }\nclass Test { run(_ value: Animal) { match value { Dog* => 1 } } }\n";
    let first = analyze_adt(source);
    let second = analyze_adt(source);
    let first_ids = match &first.only_match().arm(0).resolution().pattern {
        phalcom_semantic::match_semantics::PatternResolution::Variant(pattern) => pattern.candidates.iter().map(|candidate| candidate.variant.clone()).collect::<Vec<_>>(),
        pattern => panic!("expected variant pattern, got {pattern:?}"),
    };
    let second_ids = match &second.only_match().arm(0).resolution().pattern {
        phalcom_semantic::match_semantics::PatternResolution::Variant(pattern) => pattern.candidates.iter().map(|candidate| candidate.variant.clone()).collect::<Vec<_>>(),
        pattern => panic!("expected variant pattern, got {pattern:?}"),
    };
    assert_eq!(first_ids, second_ids);
}

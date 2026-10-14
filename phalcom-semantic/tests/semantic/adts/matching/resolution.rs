use std::collections::BTreeMap;
use std::sync::Arc;

use super::super::support::analyze_adt;
use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::analyze_single_module;
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
                owner: Some(decl.clone()),
                family: Some(fam_id),
                owner_candidates: Box::new([decl.clone()]),
                selector: VariantSelectorConstraint::WholeFamily,
                candidates: Box::new([ResolvedVariantCandidate {
                    variant: var_id,
                    exact_case: TypeId(11),
                    fields: Box::new([]),
                    proof: BranchProofEnvironment::default(),
                    case_instantiation: None,
                }]),
                blocked: None,
            }),
            reachable_space: PatternSpaceSummary::Empty,
            residual_after: PatternSpaceSummary::Empty,
            bindings: Box::new([PatternBindingResolution {
                binding: BindingId(1),
                name: "x".into(),
                knowledge: TypeKnowledge::established(TypeId(12), EvidenceOrigin::Flow),
                local_type: None,
                source: dummy_range,
            }]),
            proof: BranchProofEnvironment {
                bindings: BTreeMap::new(),
                equalities: Box::new([]),
                local_bindings: BTreeMap::new(),
                local_equalities: Box::new([]),
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
    let analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed)).expect("semantic test analysis should succeed");
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
    let case = analyze_adt("data Status { Ready Done }\nclass Test { run(_ value: Status) { match value { _ => 1 } } }\n");
    let handle = case.only_match();
    handle.assert_exhaustive();
    assert!(handle.resolution().arms[0].bindings.is_empty());
}

#[test]
fn match_res_02_qualified_singleton_has_one_exact_candidate() {
    let case = analyze_adt("data Status { Ready Done }\nclass Test { run(_ value: Status) { match value { Status::Ready => 1 _ => 0 } } }\n");
    let handle = case.only_match();
    handle.assert_arm_candidate_variants(
        0,
        &[case.variant_id("Status", phalcom_common::selector::Selector::getter("Ready").expect("Ready"))],
    );
}

#[test]
fn match_res_03_qualified_nullary_has_constructor_identity() {
    let case = analyze_adt("data Status { Ready() Done }\nclass Test { run(_ value: Status) { match value { Status::Ready() => 1 _ => 0 } } }\n");
    let handle = case.only_match();
    handle.assert_arm_candidate_variants(
        0,
        &[case.variant_id("Status", phalcom_common::selector::Selector::method("Ready", []).expect("Ready"))],
    );
}

#[test]
fn match_res_04_exact_positional_constructor_projects_field() {
    let case =
        analyze_adt("data Status { Ready(_ code: Int) Done }\nclass Test { run(_ value: Status) { match value { Status::Ready(code) => code _ => 0 } } }\n");
    let handle = case.only_match();
    assert_eq!(handle.arm(0).resolution().bindings.len(), 1);
    handle.arm(0).assert_binding_names(&["code"]);
}

#[test]
fn match_res_05_exact_labeled_constructor_uses_external_label() {
    let case = analyze_adt(
        "data Status { Ready(named code: Int) Done }\nclass Test { run(_ value: Status) { match value { Status::Ready(named: code) => code _ => 0 } } }\n",
    );
    let handle = case.only_match();
    handle.arm(0).assert_binding_names(&["code"]);
}

#[test]
fn match_res_06_contextual_some_uses_scrutinee_owner() {
    let case = analyze_adt(
        "data Option<T> { Some(_ value: T) -> Option<T> None -> Option<T> }\nclass Test { run(_ value: Option<Int>) { match value { Some(x) => x None => 0 } } }\n",
    );
    let handle = case.only_match();
    handle.arm(0).assert_binding_names(&["x"]);
    handle.arm(0).assert_binding_type("x", case.declaration("Int").form);
}

#[test]
fn match_res_07_contextual_none_resolves_singleton_member() {
    let case = analyze_adt(
        "data Option<T> { Some(_ value: T) -> Option<T> None -> Option<T> }\nclass Test { run(_ value: Option<Int>) { match value { Some(_) => 1 None => 0 } } }\n",
    );
    let handle = case.only_match();
    assert_eq!(handle.arm(1).resolution().bindings.len(), 0);
}

#[test]
fn match_res_08_ambiguous_contextual_owner_reports_no_arbitrary_candidate() {
    let case = analyze_adt("data Left { Same }\ndata Right { Same }\nclass Test { run(_ value: Left | Right) { match value { Same => 1 _ => 0 } } }\n");
    case.assert_diagnostic_primary_contains(phalcom_semantic::diagnostic::DiagnosticCode::MatchPatternUnresolved, "Same");
    let handle = case.only_match();
    let arm = handle.arm(0);
    let PatternResolution::Variant(pattern) = &arm.resolution().pattern else {
        panic!("expected contextual variant resolution");
    };
    assert!(pattern.owner.is_none(), "ambiguous contextual owner must remain absent");
    assert!(pattern.family.is_none(), "ambiguous contextual family must remain absent");
    assert_eq!(pattern.owner_candidates.len(), 2);
    assert_eq!(pattern.candidates.len(), 2);
    assert_eq!(
        pattern.candidates[0].variant.owner,
        case.variant_id("Left", Selector::getter("Same").expect("selector")).owner
    );
    assert_eq!(
        pattern.candidates[1].variant.owner,
        case.variant_id("Right", Selector::getter("Same").expect("selector")).owner
    );
}

#[test]
fn match_res_09_nested_contextual_resolution_uses_specialized_payload_domain() {
    let case = analyze_adt(
        "data Result<T, E> { Ok(_ value: T) -> Result<T, E> Error(_ value: E) -> Result<T, E> }\ndata Option<T> { Some(_ value: T) -> Option<T> None -> Option<T> }\nclass Test { run(_ value: Option<Result<Int, String>>) { match value { Some(Ok(x)) => x Some(Error(x)) => 0 None => 0 } } }\n",
    );
    let handle = case.only_match();
    assert_eq!(handle.arm(0).resolution().bindings.len(), 1);
}

#[test]
fn match_res_10_callable_family_includes_only_callable_candidates() {
    let case = analyze_adt("data Animal { Dog Dog(_ name: String) Cat }\nclass Test { run(_ value: Animal) { match value { Dog(...) => 1 _ => 0 } } }\n");
    let arm = case.only_match().arm(0);
    assert!(arm.resolution().bindings.is_empty());
    let PatternResolution::Variant(pattern) = &arm.resolution().pattern else {
        panic!("expected callable variant pattern");
    };
    assert!(
        pattern
            .candidates
            .iter()
            .all(|candidate| candidate.variant.selector.kind == phalcom_common::selector::SelectorKind::Method)
    );
}

#[test]
fn match_res_11_prefix_gap_records_candidate_specific_projection() {
    let case = analyze_adt(
        "data Animal { Dog(_ name: String) Dog(_ name: String, age: Int, breed: String) }\nclass Test { run(_ value: Animal) { match value { Dog(name, ..., breed: b) => 1 _ => 0 } } }\n",
    );
    assert!(matches!(
        case.only_match().arm(0).resolution().pattern,
        phalcom_semantic::match_semantics::PatternResolution::Variant(_)
    ));
}

#[test]
fn match_res_12_suffix_label_gap_matches_canonical_selector_pattern() {
    let case = analyze_adt(
        "data Animal { Dog(_ name: String) Dog(_ name: String, age: Int, breed: String) }\nclass Test { run(_ value: Animal) { match value { Dog(..., breed: b) => 1 _ => 0 } } }\n",
    );
    assert!(matches!(
        case.only_match().arm(0).resolution().pattern,
        phalcom_semantic::match_semantics::PatternResolution::Variant(_)
    ));
}

#[test]
fn match_res_13_prefix_and_suffix_gap_join_field_projections() {
    let case = analyze_adt(
        "data Animal { Dog(_ name: String) Dog(_ name: String, age: Int, breed: String) }\nclass Test { run(_ value: Animal) { match value { Dog(name, ..., breed: b) => name _ => \"unknown\" } } }\n",
    );
    assert!(matches!(
        case.only_match().arm(0).resolution().pattern,
        phalcom_semantic::match_semantics::PatternResolution::Variant(_)
    ));
}

#[test]
fn match_res_14_whole_family_candidate_set_is_explicit() {
    let case = analyze_adt("data Animal { Dog Dog() Dog(_ name: String) Cat }\nclass Test { run(_ value: Animal) { match value { Dog* => 1 Cat => 2 } } }\n");
    let handle = case.only_match();
    let candidates = match &handle.arm(0).resolution().pattern {
        phalcom_semantic::match_semantics::PatternResolution::Variant(pattern) => pattern.candidates.len(),
        pattern => panic!("expected family variant pattern, got {pattern:?}"),
    };
    assert_eq!(candidates, 3);
}

#[test]
fn match_res_15_candidate_order_is_stable_across_repeated_analysis() {
    let source = "data Animal { Dog Dog() Dog(_ name: String) }\nclass Test { run(_ value: Animal) { match value { Dog* => 1 } } }\n";
    let first = analyze_adt(source);
    let second = analyze_adt(source);
    let first_ids = match &first.only_match().arm(0).resolution().pattern {
        phalcom_semantic::match_semantics::PatternResolution::Variant(pattern) => {
            pattern.candidates.iter().map(|candidate| candidate.variant.clone()).collect::<Vec<_>>()
        }
        pattern => panic!("expected variant pattern, got {pattern:?}"),
    };
    let second_ids = match &second.only_match().arm(0).resolution().pattern {
        phalcom_semantic::match_semantics::PatternResolution::Variant(pattern) => {
            pattern.candidates.iter().map(|candidate| candidate.variant.clone()).collect::<Vec<_>>()
        }
        pattern => panic!("expected variant pattern, got {pattern:?}"),
    };
    assert_eq!(first_ids, second_ids);
}

#[test]
fn contextual_enum_union_constructor_uses_member_specialization() {
    let case = analyze_adt(
        r#"
data Left<T> { Take(_ value: T) Empty }
data Right { Other }
class Test {
  run(_ value: Left<Int> | Right) -> Int {
    match value { Take(item) => item _ => 0 }
  }
}
"#,
    );
    case.assert_no_diagnostics();
    let arm = case.only_match().arm(0);
    arm.assert_binding_names(&["item"]);
    let binding = &arm.resolution().bindings[0];
    let ty = binding.knowledge.ty().expect("payload type");
    assert_eq!(case.analysis.snapshot.store.format_type(ty), "Int");
    case.only_match().assert_arm_candidate_variants(
        0,
        &[case.variant_id("Left", Selector::method("Take", [phalcom_common::selector::SelectorSlot::Positional]).unwrap())],
    );
}

#[test]
fn contextual_enum_union_constructor_ambiguity_requires_qualification() {
    let case = analyze_adt(
        r#"
data Left { Take(_ value: Int) }
data Right { Take(_ value: Int) }
class Test {
  run(_ value: Left | Right) -> Int {
    match value { Take(item) => item _ => 0 }
  }
}
"#,
    );
    let diagnostic = case.diagnostic(phalcom_semantic::diagnostic::DiagnosticCode::MatchPatternUnresolved);
    assert!(diagnostic.message.contains("ambiguous"), "{diagnostic:?}");
    assert!(diagnostic.message.contains("qualify"), "{diagnostic:?}");
}

#[test]
fn contextual_enum_gadt_union_pattern_excludes_refuted_owners() {
    let case = analyze_adt(
        r#"
data Restricted<T> { Take(_ value: Bool) -> Restricted<Bool> }
data Plain { Take(_ value: Int) Other }
class Test {
  run(_ value: Restricted<Int> | Plain) -> Int {
    match value { Take(item) => item _ => 0 }
  }
}
"#,
    );
    case.assert_no_diagnostics();
    case.only_match().assert_arm_candidate_variants(
        0,
        &[case.variant_id("Plain", Selector::method("Take", [phalcom_common::selector::SelectorSlot::Positional]).unwrap())],
    );
}

#[test]
fn contextual_enum_repeated_owner_union_pattern_preserves_each_specialization() {
    let case = analyze_adt(
        r#"
data Restricted<T> { Take(_ value: Bool) -> Restricted<Bool> Other }
class Test {
  run(_ value: Restricted<Int> | Restricted<Bool>) -> Bool {
    match value { Take(item) => item _ => false }
  }
}
"#,
    );
    case.assert_no_diagnostics();
    let arm = case.only_match().arm(0);
    arm.assert_binding_names(&["item"]);
    let ty = arm.resolution().bindings[0].knowledge.ty().expect("payload type");
    assert_eq!(case.analysis.snapshot.store.format_type(ty), "Bool");
    case.only_match().assert_arm_candidate_variants(
        0,
        &[case.variant_id(
            "Restricted",
            Selector::method("Take", [phalcom_common::selector::SelectorSlot::Positional]).unwrap(),
        )],
    );
}

#[test]
fn contextual_enum_repeated_owner_union_pattern_joins_payload_specializations() {
    let case = analyze_adt(
        r#"
data Boxed<T> { Take(_ value: T) Empty }
class Test {
  run(_ value: Boxed<Int> | Boxed<String>) -> Int {
    match value { Take(item) => 0 _ => 0 }
  }
}
"#,
    );
    case.assert_no_diagnostics();
    let arm = case.only_match().arm(0);
    arm.assert_binding_names(&["item"]);
    let binding = &arm.resolution().bindings[0];
    let ty = binding.knowledge.ty().expect("payload type");
    assert_eq!(case.analysis.snapshot.store.format_type(ty), "Int | String");
}

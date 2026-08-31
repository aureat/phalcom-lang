use std::collections::BTreeMap;
use std::sync::Arc;

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

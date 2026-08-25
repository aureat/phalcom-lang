use phalcom_common::range::SourceRange;
use phalcom_semantic::checker::analysis::{AnalysisStatus, ExpressionAnalysis};
use phalcom_semantic::checker::flow::FlowState;
use phalcom_semantic::identity::{BindingId, BodyId, DiagnosticCauseId, ExpressionId, LocalExpressionId, TypeId};
use phalcom_semantic::types::evidence::{EvidenceOrigin, TypeKnowledge};
use phalcom_semantic::types::store::TypeStore;

#[test]
fn test_expression_analysis_ready_and_invalid() {
    let expr_id = ExpressionId::new(BodyId(1), LocalExpressionId(0));
    let range = SourceRange { start: 0, end: 5 };
    let ty = TypeId(10);
    let knowledge = TypeKnowledge::established(ty, EvidenceOrigin::Syntax);

    let ready_analysis = ExpressionAnalysis::ready(expr_id, range, knowledge);
    assert!(ready_analysis.status.is_ready());
    assert_eq!(ready_analysis.id, expr_id);
    assert_eq!(ready_analysis.knowledge.ty(), Some(ty));

    let cause_id = DiagnosticCauseId(42);
    let invalid_analysis = ExpressionAnalysis::invalid(expr_id, range, cause_id);
    assert!(invalid_analysis.status.is_invalid());
    assert_eq!(invalid_analysis.status, AnalysisStatus::Invalid(cause_id));
}

#[test]
fn test_binding_state_and_flow_state_operations() {
    let mut store = TypeStore::new();
    let module = phalcom_modules::identity::ModuleId::core();
    let int_decl = phalcom_modules::DeclarationId::new(module.clone(), "Int".into());
    let float_decl = phalcom_modules::DeclarationId::new(module.clone(), "Float".into());
    let num_decl = phalcom_modules::DeclarationId::new(module.clone(), "Number".into());

    let int_ty = store.nominal(int_decl);
    let float_ty = store.nominal(float_decl);
    let num_ty = store.nominal(num_decl);

    let b1 = BindingId(1);

    let mut state = FlowState::new();
    state.declare(
        b1,
        "b1",
        SourceRange::default(),
        Some(num_ty),
        TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax),
        true,
    );

    assert_eq!(state.get_declared_type(b1), Some(num_ty));
    assert_eq!(state.get_current_type(b1).and_then(|k| k.ty()), Some(int_ty));

    // Sequential assignment replaces current fact
    state.assign(b1, TypeKnowledge::established(float_ty, EvidenceOrigin::Syntax));
    assert_eq!(state.get_current_type(b1).and_then(|k| k.ty()), Some(float_ty));
    assert_eq!(state.get_declared_type(b1), Some(num_ty));
    assert_eq!(state.get_binding(b1).unwrap().version, 1);

    // Branch fork & join
    let mut branch1 = state.fork();
    let mut branch2 = state.fork();

    branch1.assign(b1, TypeKnowledge::established(int_ty, EvidenceOrigin::Flow));
    branch2.assign(b1, TypeKnowledge::established(float_ty, EvidenceOrigin::Flow));

    let joined = FlowState::join(&[branch1, branch2], &mut store);
    assert!(joined.is_reachable());
    let joined_binding = joined.get_binding(b1).unwrap();
    assert_eq!(joined_binding.declared_type(), Some(num_ty));
    // Joined type should be a union of int and float
    let joined_ty = joined_binding.current.ty().unwrap();
    assert!(matches!(store.get(joined_ty), phalcom_semantic::types::store::TypeData::Union(_)));
}

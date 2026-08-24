use phalcom_semantic::TypeId;
use phalcom_semantic::identity::{DeclarationId, ModuleId};
use phalcom_semantic::types::evidence::{DynamicReason, EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason, join_type_knowledge};
use phalcom_semantic::types::store::TypeStore;

#[test]
fn formal_knowledge_exposes_status_and_origin_without_changing_type_identity() {
    let established = TypeKnowledge::established(TypeId(7), EvidenceOrigin::Syntax);
    assert_eq!(established.ty(), Some(TypeId(7)));
    assert_eq!(established.status(), Some(EvidenceStatus::Established));
    assert_eq!(established.origin(), Some(EvidenceOrigin::Syntax));
    assert!(established.is_established());
    assert!(!established.is_assumed());

    let assumed = TypeKnowledge::assumed(TypeId(7), EvidenceOrigin::DeveloperAnnotation);
    assert_eq!(assumed.ty(), Some(TypeId(7)));
    assert_eq!(assumed.status(), Some(EvidenceStatus::Assumed));
    assert_eq!(assumed.origin(), Some(EvidenceOrigin::DeveloperAnnotation));
    assert!(!assumed.is_established());
    assert!(assumed.is_assumed());

    let unknown = TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence);
    assert_eq!(unknown.ty(), None);
    assert_eq!(unknown.status(), None);
    assert_eq!(unknown.origin(), None);
}

#[test]
fn type_transformation_preserves_epistemic_status_origin_and_provenance() {
    let knowledge = TypeKnowledge::assumed(TypeId(3), EvidenceOrigin::CallableSignature).with_range(Default::default());
    let transformed = knowledge.map_type(|_| TypeId(11));

    assert_eq!(transformed.ty(), Some(TypeId(11)));
    assert_eq!(transformed.status(), Some(EvidenceStatus::Assumed));
    assert_eq!(transformed.origin(), Some(EvidenceOrigin::CallableSignature));
}

#[test]
fn flow_join_is_fail_closed_and_preserves_epistemic_status() {
    let mut store = TypeStore::new();
    let left = store.nominal(DeclarationId::new(ModuleId::core(), "JoinLeft".into()));
    let right = store.nominal(DeclarationId::new(ModuleId::core(), "JoinRight".into()));
    let established = TypeKnowledge::established(left, EvidenceOrigin::Syntax);
    let assumed = TypeKnowledge::assumed(right, EvidenceOrigin::DeveloperAnnotation);

    let joined = join_type_knowledge(&mut store, [established.clone(), assumed]);
    assert_eq!(joined.status(), Some(EvidenceStatus::Assumed));
    assert_eq!(joined.origin(), Some(EvidenceOrigin::Flow));
    assert!(joined.ty().is_some());

    let unknown = TypeKnowledge::Unknown(UnknownReason::InferenceBlocked);
    assert_eq!(join_type_knowledge(&mut store, [established.clone(), unknown]).status(), None);
    assert!(matches!(
        join_type_knowledge(&mut store, [established, TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape)]),
        TypeKnowledge::Dynamic(_)
    ));
}

use phalcom_semantic::TypeId;
use phalcom_semantic::checker::{AssumptionBasis, BindingConsistency, BindingContract, BindingContractOrigin, reconcile_binding_contract};
use phalcom_semantic::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
use phalcom_semantic::types::relation::MapTypeHierarchy;
use phalcom_semantic::types::store::TypeStore;

#[test]
fn established_actual_is_retained_and_validated_against_contract() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let actual_ty = store.nominal(phalcom_modules::DeclarationId::new(phalcom_modules::identity::ModuleId::core(), "Int".into()));
    let contract_ty = actual_ty;
    let actual = TypeKnowledge::established(actual_ty, EvidenceOrigin::Syntax);
    let contract = BindingContract {
        ty: contract_ty,
        origin: BindingContractOrigin::SourceAnnotation,
        source: None,
    };

    let result = reconcile_binding_contract(&store, &hierarchy, Some(&contract), &actual);
    assert_eq!(result.current, actual);
    assert_eq!(result.consistency, BindingConsistency::Validated);
}

#[test]
fn eligible_no_evidence_receives_assumption_but_coverage_gap_does_not() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let int_ty = store.nominal(phalcom_modules::DeclarationId::new(phalcom_modules::identity::ModuleId::core(), "Int".into()));
    let contract = BindingContract {
        ty: int_ty,
        origin: BindingContractOrigin::SourceAnnotation,
        source: None,
    };

    let eligible = reconcile_binding_contract(&store, &hierarchy, Some(&contract), &TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence));
    assert_eq!(eligible.current.status(), Some(phalcom_semantic::EvidenceStatus::Assumed));
    assert_eq!(
        eligible.consistency,
        BindingConsistency::Assumed {
            basis: AssumptionBasis::MissingValueEvidence(UnknownReason::NoTypeEvidence),
        }
    );

    let blocked = reconcile_binding_contract(&store, &hierarchy, Some(&contract), &TypeKnowledge::Unknown(UnknownReason::UncheckedExpression));
    assert_eq!(blocked.current.ty(), None);
    assert!(matches!(blocked.consistency, BindingConsistency::Blocked(_)));
}

#[test]
fn refuted_contract_does_not_replace_actual_knowledge() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let int_ty = store.nominal(phalcom_modules::DeclarationId::new(phalcom_modules::identity::ModuleId::core(), "Int".into()));
    let string_ty = store.nominal(phalcom_modules::DeclarationId::new(
        phalcom_modules::identity::ModuleId::core(),
        "String".into(),
    ));
    let actual = TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax);
    let contract = BindingContract {
        ty: string_ty,
        origin: BindingContractOrigin::SourceAnnotation,
        source: None,
    };

    let result = reconcile_binding_contract(&store, &hierarchy, Some(&contract), &actual);
    assert_eq!(result.current.ty(), Some(int_ty));
    assert!(matches!(result.consistency, BindingConsistency::Refuted { actual, expected, .. } if actual == int_ty && expected == string_ty));
}

#[test]
fn no_contract_keeps_current_fact_unconstrained() {
    let store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let actual = TypeKnowledge::assumed(TypeId(0), EvidenceOrigin::CallableSignature);
    let result = reconcile_binding_contract(&store, &hierarchy, None, &actual);
    assert_eq!(result.current, actual);
    assert_eq!(result.consistency, BindingConsistency::Unconstrained);
}

//! Foundation tests for field flow state and validity lattice.

use phalcom_semantic::checker::causal::CausalInvalidity;
use phalcom_semantic::checker::flow::{FieldContractValidity, FieldInitialization, FieldState, FlowState, join_field_validity};
use phalcom_semantic::identity::{DeclarationId, DispatchSide, FieldId, ModuleId};
use phalcom_semantic::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
use phalcom_semantic::types::outcome::{BlockReason, DynamicBoundaryObligation};
use phalcom_semantic::types::store::TypeStore;

fn make_field(name: &str) -> FieldId {
    FieldId::new(DeclarationId::new(ModuleId::core(), "Cell".into()), name, DispatchSide::Instance)
}

#[test]
fn field_validity_join_never_strengthens_a_reachable_path() {
    use FieldContractValidity::*;

    assert_eq!(join_field_validity([Validated, Validated]), Validated);
    assert_eq!(join_field_validity([Validated, Assumed]), Assumed);
    assert_eq!(join_field_validity([Assumed, Assumed]), Assumed);
    assert_eq!(join_field_validity([Validated, Unchecked]), Unchecked);
    assert_eq!(join_field_validity([Validated, Refuted]), Refuted);
    assert_eq!(join_field_validity([Assumed, Refuted]), Refuted);

    let blocked = Blocked(BlockReason::UnknownType(UnknownReason::MissingInitializer));
    assert_eq!(join_field_validity([Validated, blocked.clone()]), blocked);
    assert_eq!(join_field_validity([blocked.clone(), Refuted]), Refuted);

    let dynamic = DynamicBoundary(DynamicBoundaryObligation { reason: "dynamic call".into() });
    assert_eq!(join_field_validity([Validated, dynamic.clone()]), dynamic);
    assert_eq!(join_field_validity([dynamic, Refuted]), Refuted);
}

#[test]
fn field_join_with_incompatible_reachable_paths_records_refuted_and_unioned_knowledge() {
    let mut store = TypeStore::new();
    let int_ty = store.nominal_type(DeclarationId::new(ModuleId::core(), "Int".into()));
    let string_ty = store.nominal_type(DeclarationId::new(ModuleId::core(), "String".into()));
    let id = make_field("_value");
    let contract = TypeKnowledge::assumed(int_ty, EvidenceOrigin::DeveloperAnnotation);

    let mut left = FlowState::new();
    left.seed_field(FieldState {
        field: id.clone(),
        contract: contract.clone(),
        current: TypeKnowledge::established(int_ty, EvidenceOrigin::Flow),
        initialization: FieldInitialization::DefinitelyInitialized,
        validity: FieldContractValidity::Validated,
        causal_invalidity: CausalInvalidity::Clean,
        version: 0,
    });

    let mut right = FlowState::new();
    right.seed_field(FieldState {
        field: id.clone(),
        contract: contract.clone(),
        current: TypeKnowledge::established(string_ty, EvidenceOrigin::Flow),
        initialization: FieldInitialization::DefinitelyInitialized,
        validity: FieldContractValidity::Refuted,
        causal_invalidity: CausalInvalidity::Clean,
        version: 0,
    });

    let joined = FlowState::join(&[left, right], &mut store);
    let field = joined.get_field(&id).expect("joined field");

    assert_eq!(field.initialization, FieldInitialization::DefinitelyInitialized);
    assert_eq!(field.validity, FieldContractValidity::Refuted);
    assert_eq!(field.causal_invalidity, CausalInvalidity::Clean);
    assert_ne!(field.current.ty(), Some(int_ty));
    assert_ne!(field.current.ty(), Some(string_ty));
}

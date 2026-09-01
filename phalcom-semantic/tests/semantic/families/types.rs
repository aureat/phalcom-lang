use phalcom_common::selector::SelectorSlot;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::types::family::{FamilyMemberType, FamilyMemberTypeKind, FamilyOperationShape, FamilyTypeError};
use phalcom_semantic::types::store::{CallableParameterType, CallableType, TypeData, TypeStore};

#[test]
fn structural_family_type_canonicalizes_member_order() {
    let mut store = TypeStore::new();
    let value_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "Int".into()));
    let callable_type = store.callable(CallableType {
        parameters: Box::new([CallableParameterType::new(value_type)]),
        return_type: value_type,
    });
    let getter = FamilyMemberType::value(FamilyOperationShape::getter(), value_type);
    let method = FamilyMemberType::callable(FamilyOperationShape::method(vec![SelectorSlot::Positional].into_boxed_slice()), callable_type);

    let first = store.family_type([getter.clone(), method.clone()]).expect("family type");
    let second = store.family_type([method, getter]).expect("family type");

    assert_eq!(first, second);
    let TypeData::Family(family_id) = store.get(first) else {
        panic!("expected structural family type");
    };
    let family = store.get_family(*family_id);
    assert_eq!(family.members.len(), 2);
    assert_eq!(family.members[0].member_kind, FamilyMemberTypeKind::Value);
}

#[test]
fn structural_family_type_rejects_conflicting_duplicate_operation_shapes() {
    let mut store = TypeStore::new();
    let int_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "Int".into()));
    let string_type = store.nominal_type(DeclarationId::new(ModuleId::universe_root(), "String".into()));
    let operation = FamilyOperationShape::getter();

    let error = store
        .family_type([
            FamilyMemberType::value(operation.clone(), int_type),
            FamilyMemberType::value(operation, string_type),
        ])
        .expect_err("conflicting duplicate operation");

    assert!(matches!(error, FamilyTypeError::DuplicateOperationShape { .. }));
}

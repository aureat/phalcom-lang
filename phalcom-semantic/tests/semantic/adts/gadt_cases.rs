use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::types::case_environment::derive_case_environment;
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner};
use phalcom_semantic::types::store::TypeStore;

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

#[test]
fn gadt_case_result_binds_multiple_enum_parameters_in_declaration_order() {
    let mut store = TypeStore::new();
    let module = test_module();
    let owner = DeclarationId::new(module.clone(), "Pair".into());
    let first = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(owner.clone()), 0, "A", KindId::TYPE));
    let second = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(owner.clone()), 1, "B", KindId::TYPE));
    let int_ty = store.nominal_type(DeclarationId::new(module.clone(), "Int".into()));
    let string_ty = store.nominal_type(DeclarationId::new(module, "String".into()));
    let pair_kind = store.arrow_kind(vec![KindId::TYPE, KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let pair_form = store.nominal_form(owner.clone(), pair_kind);
    let result = store.apply_type_form(pair_form, &[int_ty, string_ty]).expect("Pair<Int, String>");

    let environment = derive_case_environment(&mut store, &owner, &[first, second], Some(result)).expect("GADT environment");

    assert_eq!(environment.bindings.get(&first), Some(&int_ty));
    assert_eq!(environment.bindings.get(&second), Some(&string_ty));
}

#[test]
fn default_gadt_result_keeps_case_environment_empty() {
    let mut store = TypeStore::new();
    let module = test_module();
    let owner = DeclarationId::new(module, "Expr".into());
    let parameter = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(owner.clone()), 0, "T", KindId::TYPE));

    let environment = derive_case_environment(&mut store, &owner, &[parameter], None).expect("default GADT environment");

    assert!(environment.is_empty());
}

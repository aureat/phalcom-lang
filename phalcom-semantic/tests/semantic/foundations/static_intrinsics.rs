use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::static_intrinsic::{StaticIntrinsicOperation, StaticIntrinsicRegistry};

#[test]
fn static_intrinsic_authority_uses_the_exact_ready_callable_identity() {
    let registry = StaticIntrinsicRegistry::from_canonical_core().unwrap();
    let selector = Selector::try_decode_exact("+(_)").unwrap();
    let core_number_add = CallableId::method(
        phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Number),
        selector.clone(),
        DispatchSide::Instance,
    );
    assert_eq!(registry.operation_for(&core_number_add), Some(StaticIntrinsicOperation::NumberAdd));

    let user_same_spelling = CallableId::method(
        DeclarationId::new(ModuleId::universe_root(), "Number".into()),
        selector.clone(),
        DispatchSide::Instance,
    );
    assert_eq!(registry.operation_for(&user_same_spelling), None);

    let string_add = CallableId::method(
        phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::String),
        selector,
        DispatchSide::Instance,
    );
    assert_eq!(
        registry.operation_for(&string_add),
        None,
        "native surface membership alone does not grant static authority"
    );
}

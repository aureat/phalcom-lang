use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::identity::{DeclarationId, VariantId};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::relation::{MapTypeHierarchy, is_subtype};
use phalcom_semantic::types::store::{TypeData, TypeStore};

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

#[test]
fn exact_case_identity_is_canonical_for_same_variant_and_specialization() {
    let mut store = TypeStore::new();
    let module = test_module();
    let owner = DeclarationId::new(module.clone(), "Option".into());
    let int_decl = DeclarationId::new(module.clone(), "Int".into());
    let option_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let option_form = store.nominal_form(owner.clone(), option_kind);
    let int_ty = store.nominal_type(int_decl);
    let option_int = store.apply_type_form(option_form, &[int_ty]).expect("Option<Int>");
    let some = VariantId::new(
        owner,
        Selector::method("Some", [phalcom_common::selector::SelectorSlot::Positional]).expect("Some"),
    );

    let first = store.exact_case_type(&some, option_int).expect("first exact case");
    let second = store.exact_case_type(&some, option_int).expect("second exact case");

    assert_eq!(first, second);
    assert!(matches!(store.get(first), TypeData::ExactCase { .. }));
}

#[test]
fn exact_case_subtyping_is_reflexive_enclosing_and_variant_specific() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let module = test_module();
    let owner = DeclarationId::new(module.clone(), "Option".into());
    let option = store.nominal_type(owner.clone());
    let some = VariantId::new(owner.clone(), Selector::getter("Some").expect("Some"));
    let none = VariantId::new(owner, Selector::getter("None").expect("None"));
    let some_case = store.exact_case_type(&some, option).expect("Some case");
    let none_case = store.exact_case_type(&none, option).expect("None case");

    assert!(is_subtype(&mut store, &hierarchy, some_case, some_case));
    assert!(is_subtype(&mut store, &hierarchy, some_case, option));
    assert!(!is_subtype(&mut store, &hierarchy, some_case, none_case));
    assert!(!is_subtype(&mut store, &hierarchy, option, some_case));
}

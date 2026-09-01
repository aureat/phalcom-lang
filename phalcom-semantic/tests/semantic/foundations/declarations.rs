use phalcom_modules::identity::ModuleId;
use phalcom_native_meta::universe::UniverseKey;
use phalcom_semantic::declarations::bootstrap_universe_declarations;
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::kind::KindData;
use phalcom_semantic::types::parameter::TypeParameterOwner;
use phalcom_semantic::types::store::TypeStore;

fn test_universe_resolver(key: UniverseKey) -> DeclarationId {
    DeclarationId::new(ModuleId::core(), key.name().into())
}

#[test]
fn declaration_kinds_and_forms_are_correct() {
    let mut store = TypeStore::new();
    let table = bootstrap_universe_declarations(&mut store, &test_universe_resolver);

    let int_decl = test_universe_resolver(UniverseKey::Int);
    let list_decl = test_universe_resolver(UniverseKey::List);
    let set_decl = test_universe_resolver(UniverseKey::Set);
    let map_decl = test_universe_resolver(UniverseKey::Map);
    let opt_decl = test_universe_resolver(UniverseKey::Option);

    // Int :: Type
    assert_eq!(table.kind(&int_decl), Some(KindId::TYPE));
    let int_form = table.form(&int_decl).unwrap();
    assert_eq!(store.kind_of(int_form), KindId::TYPE);
    assert!(store.is_proper_type(int_form));

    // List :: Type -> Type
    let list_kind = table.kind(&list_decl).unwrap();
    assert_eq!(
        store.get_kind(list_kind),
        &KindData::Arrow {
            parameters: vec![KindId::TYPE].into_boxed_slice(),
            result: KindId::TYPE,
        }
    );

    // Set :: Type -> Type
    let set_kind = table.kind(&set_decl).unwrap();
    assert_eq!(
        store.get_kind(set_kind),
        &KindData::Arrow {
            parameters: vec![KindId::TYPE].into_boxed_slice(),
            result: KindId::TYPE,
        }
    );

    // Map :: Type -> Type -> Type
    let map_kind = table.kind(&map_decl).unwrap();
    assert_eq!(
        store.get_kind(map_kind),
        &KindData::Arrow {
            parameters: vec![KindId::TYPE, KindId::TYPE].into_boxed_slice(),
            result: KindId::TYPE,
        }
    );

    // Option :: Type -> Type
    let opt_kind = table.kind(&opt_decl).unwrap();
    assert_eq!(
        store.get_kind(opt_kind),
        &KindData::Arrow {
            parameters: vec![KindId::TYPE].into_boxed_slice(),
            result: KindId::TYPE,
        }
    );

    // ClassObject proper types all have kind Type and are distinct from form
    let int_class_obj = table.class_object_type(&int_decl).unwrap();
    assert_ne!(int_class_obj, int_form);
    assert_eq!(store.kind_of(int_class_obj), KindId::TYPE);

    let list_form = table.form(&list_decl).unwrap();
    let list_class_obj = table.class_object_type(&list_decl).unwrap();
    assert_ne!(list_class_obj, list_form);
    assert_eq!(store.kind_of(list_class_obj), KindId::TYPE);

    // Generic signature parameters have stable owner and indices
    let list_sig = table.generic_signature(&list_decl).unwrap();
    assert_eq!(list_sig.parameters.len(), 1);
    let p_info = store.type_parameter(list_sig.parameters[0]);
    assert_eq!(p_info.owner, TypeParameterOwner::Declaration(list_decl));
    assert_eq!(p_info.index, 0);
    assert_eq!(p_info.name.as_ref(), "T");
    assert_eq!(p_info.kind, KindId::TYPE);
}

#[test]
fn option_case_behavior_classes_are_not_semantic_declarations() {
    let mut store = TypeStore::new();
    let declarations = bootstrap_universe_declarations(
        &mut store,
        &|key| DeclarationId::new(ModuleId::core(), key.name().into()),
    );

    let option = DeclarationId::new(ModuleId::core(), "Option".into());
    let some = DeclarationId::new(ModuleId::core(), "Some".into());
    let none = DeclarationId::new(ModuleId::core(), "None".into());

    assert!(declarations.get(&option).is_some());
    assert!(declarations.get(&some).is_none());
    assert!(declarations.get(&none).is_none());
}

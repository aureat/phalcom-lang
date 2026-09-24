use phalcom_modules::identity::ModuleId;
use phalcom_native_meta::universe::UniverseKey;
use phalcom_semantic::core_surface::universe_declaration;
use phalcom_semantic::declarations::bootstrap_universe_declarations;
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::kind::KindData;
use phalcom_semantic::types::parameter::TypeParameterOwner;
use phalcom_semantic::types::store::TypeStore;

fn test_universe_resolver(key: UniverseKey) -> DeclarationId {
    DeclarationId::new(ModuleId::universe_root(), key.name().into())
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

    assert_eq!(table.kind(&int_decl), Some(KindId::TYPE));
    let int_form = table.form(&int_decl).unwrap();
    assert_eq!(store.kind_of(int_form), KindId::TYPE);
    assert!(store.is_proper_type(int_form));

    let list_kind = table.kind(&list_decl).unwrap();
    assert_eq!(
        store.get_kind(list_kind),
        &KindData::Arrow {
            parameters: vec![KindId::TYPE].into_boxed_slice(),
            result: KindId::TYPE,
        }
    );

    let set_kind = table.kind(&set_decl).unwrap();
    assert_eq!(
        store.get_kind(set_kind),
        &KindData::Arrow {
            parameters: vec![KindId::TYPE].into_boxed_slice(),
            result: KindId::TYPE,
        }
    );

    let map_kind = table.kind(&map_decl).unwrap();
    assert_eq!(
        store.get_kind(map_kind),
        &KindData::Arrow {
            parameters: vec![KindId::TYPE, KindId::TYPE].into_boxed_slice(),
            result: KindId::TYPE,
        }
    );

    let opt_kind = table.kind(&opt_decl).unwrap();
    assert_eq!(
        store.get_kind(opt_kind),
        &KindData::Arrow {
            parameters: vec![KindId::TYPE].into_boxed_slice(),
            result: KindId::TYPE,
        }
    );

    let int_class_obj = table.class_object_type(&int_decl).unwrap();
    assert_ne!(int_class_obj, int_form);
    assert_eq!(store.kind_of(int_class_obj), KindId::TYPE);

    let list_form = table.form(&list_decl).unwrap();
    let list_class_obj = table.class_object_type(&list_decl).unwrap();
    assert_ne!(list_class_obj, list_form);
    assert_eq!(store.kind_of(list_class_obj), KindId::TYPE);

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
    let declarations = bootstrap_universe_declarations(&mut store, &|key| DeclarationId::new(ModuleId::universe_root(), key.name().into()));

    let option = DeclarationId::new(ModuleId::universe_root(), "Option".into());
    let some = DeclarationId::new(ModuleId::universe_root(), "Some".into());
    let none = DeclarationId::new(ModuleId::universe_root(), "None".into());

    assert!(declarations.get(&option).is_some());
    assert!(declarations.get(&some).is_none());
    assert!(declarations.get(&none).is_none());
}

#[test]
fn source_declared_runtime_support_class_remains_semantic_declaration() {
    let mut store = TypeStore::new();
    let declarations = bootstrap_universe_declarations(&mut store, &universe_declaration);
    let unit = universe_declaration(UniverseKey::Unit);

    assert!(
        declarations.get(&unit).is_some(),
        "Unit is declared in canonical Universe source and must not disappear merely because native metadata classifies it as runtime support"
    );
    assert_eq!(
        declarations.intrinsic_runtime_owner(&unit),
        Some(phalcom_semantic::declarations::IntrinsicRuntimeOwner::Unit),
    );
}

#[test]
fn runtime_structural_declarations_bind_canonical_platform_owners() {
    use phalcom_semantic::declarations::IntrinsicRuntimeOwner;

    let mut store = TypeStore::new();
    let declarations = bootstrap_universe_declarations(&mut store, &universe_declaration);

    for (key, owner) in [
        (UniverseKey::Unit, IntrinsicRuntimeOwner::Unit),
        (UniverseKey::Tuple, IntrinsicRuntimeOwner::Tuple),
        (UniverseKey::Record, IntrinsicRuntimeOwner::Record),
    ] {
        let declaration = universe_declaration(key);
        assert_eq!(declarations.intrinsic_runtime_owner(&declaration), Some(owner));
        assert!(
            declarations.contains(&declaration),
            "runtime source facade retains a semantic declaration projection"
        );
    }
    assert_eq!(declarations.form(&universe_declaration(UniverseKey::Unit)), Some(store.unit()));
    let unit = universe_declaration(UniverseKey::Unit);
    let unit_runtime_form = store.nominal_type(unit.clone());
    assert_eq!(declarations.runtime_form(&mut store, &unit), Some(unit_runtime_form));
}

#[test]
fn production_universe_declarations_keep_source_owned_identity() {
    let mut store = TypeStore::new();
    let declarations = bootstrap_universe_declarations(&mut store, &universe_declaration);

    for key in [
        UniverseKey::Int,
        UniverseKey::List,
        UniverseKey::Option,
        UniverseKey::Result,
        UniverseKey::Ordering,
        UniverseKey::Unit,
    ] {
        let declaration = universe_declaration(key);
        let info = declarations
            .get(&declaration)
            .unwrap_or_else(|| panic!("missing source-owned semantic declaration for {key:?}: {declaration:?}"));
        assert_eq!(info.declaration, declaration);
        assert_ne!(
            info.declaration.module,
            ModuleId::universe_root(),
            "{key:?} must be owned by its canonical source module, not fabricated in the Universe root"
        );
    }
}

#[test]
fn intrinsic_structural_source_facades_bind_existing_authorities_without_nominal_forms() {
    use phalcom_modules::identity::{ModuleComponent, ModulePath};
    use phalcom_semantic::types::store::StaticDomainData;
    use phalcom_semantic::types::type_member::IntrinsicTypeOwner;

    let mut store = TypeStore::new();
    let declarations = bootstrap_universe_declarations(&mut store, &universe_declaration);
    let source_module = |parts: &[&str]| {
        ModuleId::universe(ModulePath::from_components(
            parts
                .iter()
                .map(|part| ModuleComponent::from_identifier(part).expect("canonical component"))
                .collect::<Vec<_>>(),
        ))
    };
    let facades = [
        (
            DeclarationId::new(source_module(&["collections", "tuple"]), "TypePack".into()),
            KindId::TYPE_PACK,
        ),
        (
            DeclarationId::new(source_module(&["collections", "tuple"]), "LabeledTypePack".into()),
            KindId::LABELED_TYPE_PACK,
        ),
        (
            DeclarationId::new(source_module(&["collections", "tuple"]), "TupleEntries".into()),
            KindId::COMPLETE_TYPE_PACK,
        ),
        (
            DeclarationId::new(source_module(&["collections", "record"]), "RecordRow".into()),
            KindId::RECORD_ROW,
        ),
    ];

    for (declaration, expected_kind) in &facades {
        assert_eq!(declarations.intrinsic_static_domain_kind(declaration), Some(*expected_kind));
        let expected_owner = match *expected_kind {
            KindId::TYPE_PACK => IntrinsicTypeOwner::TypePack,
            KindId::LABELED_TYPE_PACK => IntrinsicTypeOwner::LabeledTypePack,
            KindId::COMPLETE_TYPE_PACK => IntrinsicTypeOwner::CompleteTypePack,
            KindId::RECORD_ROW => IntrinsicTypeOwner::RecordRow,
            _ => unreachable!(),
        };
        assert_eq!(declarations.intrinsic_type_owner(declaration), Some(expected_owner));
        assert!(declarations.form(declaration).is_none(), "facades do not acquire TypeId forms");
        assert!(declarations.get(declaration).is_none(), "facades do not acquire nominal declaration records");
        assert!(
            store
                .static_domain_ids()
                .all(|id| !matches!(store.static_domain(id), Some(StaticDomainData::Nominal { declaration: owner }) if owner == declaration)),
            "facades do not allocate nominal static domains"
        );
    }
    assert_eq!(
        declarations.intrinsic_type_owner(&universe_declaration(UniverseKey::Tuple)),
        Some(IntrinsicTypeOwner::Tuple)
    );
    assert_eq!(
        declarations.intrinsic_type_owner(&universe_declaration(UniverseKey::Record)),
        Some(IntrinsicTypeOwner::Record)
    );
}

#[test]
fn tuple_entries_kind_resolution_uses_its_canonical_facade_declaration() {
    use phalcom_semantic::types::annotation::{GenericBinderSite, SimpleTypeResolver, TypeFormationOutcome, TypeFormationSite, resolve_generic_signature};

    let mut store = TypeStore::new();
    let declarations = bootstrap_universe_declarations(&mut store, &universe_declaration);
    let tuple_module = ModuleId::universe(phalcom_modules::identity::ModulePath::from_components(
        ["collections", "tuple"]
            .into_iter()
            .map(|part| phalcom_modules::identity::ModuleComponent::from_identifier(part).unwrap())
            .collect::<Vec<_>>(),
    ));
    let tuple_entries = DeclarationId::new(tuple_module, "TupleEntries".into());
    let parsed = phalcom_ast::parse_source("class Carrier<T: TupleEntries> {}\n", 0).expect("carrier parses");
    let phalcom_ast::ast::Statement::Class(class) = &parsed.statements[0] else {
        panic!("expected class declaration");
    };
    let mut resolver = SimpleTypeResolver::new();
    resolver.insert("TupleEntries", tuple_entries);
    let owner = DeclarationId::new(ModuleId::universe_root(), "Carrier".into());
    let mut diagnostics = Vec::new();
    let signature = resolve_generic_signature(
        &mut store,
        &declarations,
        &resolver,
        &TypeFormationSite::module(ModuleId::universe_root()),
        TypeParameterOwner::Declaration(owner),
        GenericBinderSite::NominalDeclaration,
        &class.generic_parameters,
        None,
        &mut diagnostics,
    );
    let TypeFormationOutcome::Ready(signature) = signature else {
        panic!("TupleEntries binder kind must resolve: {signature:?}; diagnostics={diagnostics:?}");
    };
    assert_eq!(store.type_parameter(signature.parameters[0]).kind, KindId::COMPLETE_TYPE_PACK);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

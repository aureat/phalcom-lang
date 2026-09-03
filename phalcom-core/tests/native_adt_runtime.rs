use phalcom_common::selector::Selector;
use phalcom_core::adt::{RuntimeAdtRegistrationError, RuntimeAdtRepresentation};
use phalcom_core::modules::semantic_lowering::{EnumLoweringSpec, VariantFieldLoweringSpec, VariantLoweringSpec};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use phalcom_modules::{DeclarationId, ModuleComponent, ModuleId, ModulePath, SyntheticProjectIdAllocator};
use phalcom_native_meta::UniverseKey;
use phalcom_semantic::core_surface::universe_declaration;
use phalcom_semantic::enum_semantics::VariantShape;
use phalcom_semantic::identity::{VariantFieldId, VariantId};

fn empty_general_enum(owner: DeclarationId) -> EnumLoweringSpec {
    EnumLoweringSpec {
        owner,
        representation: RuntimeAdtRepresentation::General,
        variants: Box::new([]),
    }
}

fn synthetic_declaration(name: &str) -> DeclarationId {
    let mut ids = SyntheticProjectIdAllocator;
    DeclarationId::new(ModuleId::synthetic(ids.allocate(), ModulePath::root()), name.into())
}

#[test]
fn test_native_option_runtime_representation_seam() {
    let mut vm = VM::new();
    let option_decl = universe_declaration(UniverseKey::Option);

    let some_sel = Selector::method("Some", vec![phalcom_common::selector::SelectorSlot::Positional]).unwrap();
    let some_var_id = VariantId::new(option_decl.clone(), some_sel);
    let some_field_id = VariantFieldId::new(some_var_id.clone(), 0);

    let none_sel = Selector::getter("None").unwrap();
    let none_var_id = VariantId::new(option_decl.clone(), none_sel);

    let option_spec = EnumLoweringSpec {
        owner: option_decl.clone(),
        representation: RuntimeAdtRepresentation::NativeOption,
        variants: Box::new([
            VariantLoweringSpec {
                id: some_var_id.clone(),
                shape: VariantShape::Constructor,
                payload_fields: Box::new([VariantFieldLoweringSpec {
                    id: some_field_id,
                    local_name: "value".into(),
                    slot: 0,
                }]),
            },
            VariantLoweringSpec {
                id: none_var_id.clone(),
                shape: VariantShape::Singleton,
                payload_fields: Box::new([]),
            },
        ]),
    };

    let expected_root = vm.universe.classes.resolve(UniverseKey::Option);
    let root_class = vm.register_enum_from_spec(&option_spec).expect("register Option enum");
    assert_eq!(root_class, expected_root);
    assert_eq!(vm.heap.class(root_class).name.as_str(), "Option");

    let enum_id = vm.adt_registry.enum_by_declaration(&option_decl).expect("Option ADT descriptor");
    assert_eq!(
        vm.adt_registry.enum_descriptor(enum_id).expect("Option enum descriptor").root_class,
        expected_root
    );

    let none_val = Value::none();
    let some_val = Value::int(42).wrap_some().unwrap();

    let none_rid = vm.runtime_variant_of(none_val).expect("none variant id");
    let some_rid = vm.runtime_variant_of(some_val).expect("some variant id");

    assert_ne!(none_rid, some_rid);
    assert!(vm.value_is_variant(none_val, none_rid));
    assert!(vm.value_is_variant(some_val, some_rid));

    assert_eq!(vm.case_payload_len(none_val), Some(0));
    assert_eq!(vm.case_payload_len(some_val), Some(1));

    let extracted = vm.case_payload_at(some_val, 0).expect("payload at 0");
    assert_eq!(extracted, Value::int(42));

    let none_case_class = vm.case_behavior_class(none_val).expect("none case class");
    let some_case_class = vm.case_behavior_class(some_val).expect("some case class");
    assert_ne!(none_case_class, some_case_class);
    assert_eq!(vm.heap.class(none_case_class).superclass, Some(expected_root));
    assert_eq!(vm.heap.class(some_case_class).superclass, Some(expected_root));
}

#[test]
fn canonical_result_reuses_primordial_runtime_root() {
    let mut vm = VM::new();
    let result_decl = universe_declaration(UniverseKey::Result);
    let expected = vm.universe.classes.resolve(UniverseKey::Result);

    let actual = vm
        .register_enum_from_spec(&empty_general_enum(result_decl.clone()))
        .expect("register canonical Result");

    assert_eq!(actual, expected);
    let enum_id = vm.adt_registry.enum_by_declaration(&result_decl).expect("Result ADT descriptor");
    let descriptor = vm.adt_registry.enum_descriptor(enum_id).expect("Result enum descriptor");
    assert_eq!(descriptor.root_class, expected);
    assert_eq!(descriptor.representation, RuntimeAdtRepresentation::General);
}

#[test]
fn canonical_ordering_reuses_primordial_root_for_hidden_case_superclass() {
    let mut vm = VM::new();
    let ordering_decl = universe_declaration(UniverseKey::Ordering);
    let expected = vm.universe.classes.resolve(UniverseKey::Ordering);
    let less = VariantId::new(ordering_decl.clone(), Selector::getter("Less").unwrap());
    let spec = EnumLoweringSpec {
        owner: ordering_decl.clone(),
        representation: RuntimeAdtRepresentation::General,
        variants: Box::new([VariantLoweringSpec {
            id: less.clone(),
            shape: VariantShape::Singleton,
            payload_fields: Box::new([]),
        }]),
    };

    let actual = vm.register_enum_from_spec(&spec).expect("register canonical Ordering");
    assert_eq!(actual, expected);

    let enum_id = vm.adt_registry.enum_by_declaration(&ordering_decl).expect("Ordering ADT descriptor");
    let descriptor = vm.adt_registry.enum_descriptor(enum_id).expect("Ordering enum descriptor");
    assert_eq!(descriptor.root_class, expected);
    assert_eq!(descriptor.representation, RuntimeAdtRepresentation::General);

    let runtime_less = vm.adt_registry.variant_by_semantic(&less).expect("Less runtime identity");
    let behavior = vm.adt_registry.variant_descriptor(runtime_less).expect("Less descriptor").behavior_class;
    assert_eq!(vm.heap.class(behavior).superclass, Some(expected));
}

#[test]
fn user_result_does_not_reuse_universe_result_root() {
    let mut vm = VM::new();
    let owner = synthetic_declaration("Result");
    let universe_result = vm.universe.classes.resolve(UniverseKey::Result);

    let user_root = vm.register_enum_from_spec(&empty_general_enum(owner)).expect("register user Result");

    assert_ne!(user_root, universe_result);
}

#[test]
fn conflicting_duplicate_registry_registration_is_rejected() {
    let vm = VM::new();
    let mut registry = phalcom_core::adt::RuntimeAdtRegistry::new();
    let owner = synthetic_declaration("Choice");
    let first_root = vm.universe.classes.result_class;
    let conflicting_root = vm.universe.classes.ordering_class;

    let first = registry
        .register_enum_with_representation(owner.clone(), first_root, RuntimeAdtRepresentation::General)
        .expect("initial registration");
    let same = registry
        .register_enum_with_representation(owner.clone(), first_root, RuntimeAdtRepresentation::General)
        .expect("identical registration is idempotent");
    assert_eq!(first, same);

    let conflict = registry
        .register_enum_with_representation(owner.clone(), conflicting_root, RuntimeAdtRepresentation::General)
        .expect_err("different root must be rejected");
    assert!(matches!(conflict, RuntimeAdtRegistrationError::ConflictingEnumRegistration { .. }));

    let representation_conflict = registry
        .register_enum_with_representation(owner.clone(), first_root, RuntimeAdtRepresentation::NativeOption)
        .expect_err("different representation must be rejected");
    assert!(matches!(
        representation_conflict,
        RuntimeAdtRegistrationError::ConflictingEnumRegistration { .. }
    ));

    let other_owner = synthetic_declaration("OtherChoice");
    let root_conflict = registry
        .register_enum_with_representation(other_owner, first_root, RuntimeAdtRepresentation::General)
        .expect_err("one root class cannot back two semantic enum declarations");
    assert!(matches!(root_conflict, RuntimeAdtRegistrationError::RootClassAlreadyRegistered { .. }));
}

#[test]
fn universe_leaf_name_cannot_reconstruct_a_different_declaration_owner() {
    let vm = VM::new();
    let wrong_module = ModuleId::universe(ModulePath::from_components(vec![
        ModuleComponent::from_identifier("object").expect("valid component"),
    ]));
    let forged_result = DeclarationId::new(wrong_module, "Result".into());

    assert!(
        vm.resolve_declaration_class(&forged_result).is_err(),
        "a Universe leaf name must not resolve unless the full DeclarationId is canonical"
    );
}

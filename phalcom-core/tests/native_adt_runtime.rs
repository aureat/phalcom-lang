use phalcom_common::selector::Selector;
use phalcom_core::modules::semantic_lowering::{EnumLoweringSpec, VariantFieldLoweringSpec, VariantLoweringSpec};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use phalcom_modules::{DeclarationId, ModuleId};
use phalcom_semantic::enum_semantics::VariantShape;
use phalcom_semantic::identity::{VariantFieldId, VariantId};

#[test]
fn test_native_option_runtime_representation_seam() {
    let mut vm = VM::new();
    let module = ModuleId::core();
    let option_decl = DeclarationId::new(module.clone(), "Option".into());

    let some_sel = Selector::method("Some", vec![phalcom_common::selector::SelectorSlot::Positional]).unwrap();
    let some_var_id = VariantId::new(option_decl.clone(), some_sel);
    let some_field_id = VariantFieldId::new(some_var_id.clone(), 0);

    let none_sel = Selector::getter("None").unwrap();
    let none_var_id = VariantId::new(option_decl.clone(), none_sel);

    let option_spec = EnumLoweringSpec {
        owner: option_decl.clone(),
        representation: phalcom_core::adt::RuntimeAdtRepresentation::NativeOption,
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

    let root_class = vm.register_enum_from_spec(&option_spec).expect("register Option enum");
    assert_eq!(vm.heap.class(root_class).name.as_str(), "Option");

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

    // .class / case_behavior_class
    let none_case_class = vm.case_behavior_class(none_val).expect("none case class");
    let some_case_class = vm.case_behavior_class(some_val).expect("some case class");
    assert_ne!(none_case_class, some_case_class);
}

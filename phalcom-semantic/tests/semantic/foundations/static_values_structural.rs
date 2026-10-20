use phalcom_common::modifier::Visibility;
use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::data_semantics::{
    DataComponentSemantic, DataConstructorParameter, DataConstructorSignature, DataInfo, DataProductContribution, DataProductShapeTemplate, DataSemanticTable,
    DataShape,
};
use phalcom_semantic::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact};
use phalcom_semantic::enum_semantics::{AlgebraicCasePayloadKind, EnumSemanticTable, VariantFieldSemantic, VariantInfo, VariantShape, VariantVisibility};
use phalcom_semantic::field_key::FieldKey;
use phalcom_semantic::identity::{DataComponentId, DataConstructorId, DeclarationId, VariantFieldId, VariantId};
use phalcom_semantic::static_value::{StaticValueData, StaticValueStore, StaticValueStoreError};
use phalcom_semantic::types::TypeId;
use phalcom_semantic::types::case_environment::CaseTypeEnvironment;
use phalcom_semantic::types::evidence::{EvidenceOrigin, TypeKnowledge};
use phalcom_semantic::types::id::VariantTypeId;
use phalcom_semantic::types::relation::{Assignability, MapTypeHierarchy};
use phalcom_semantic::types::store::TypeStore;
use std::sync::Arc;

fn declared(ty: TypeId) -> DeclaredTypeFact {
    DeclaredTypeFact::from_knowledge_with_basis(
        &TypeKnowledge::established(ty, EvidenceOrigin::DeclarationSemantics),
        DeclaredTypeBasis::DeclarationSemantics,
    )
}

#[test]
fn structural_static_values_are_canonical_typed_and_acyclic() {
    let module = ModuleId::universe_root();
    let data_owner = DeclarationId::new(module.clone(), "Pair".into());
    let enum_owner = DeclarationId::new(module, "Choice".into());
    let mut types = TypeStore::new();
    let int_ty = types.nominal_type(phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Int));
    let string_ty = types.nominal_type(phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::String));
    let pair_ty = types.nominal_type(data_owner.clone());
    let enum_ty = types.nominal_type(enum_owner.clone());

    let left_component = DataComponentSemantic {
        id: DataComponentId::new(data_owner.clone(), 0),
        local_name: "left".into(),
        external_label: None,
        declared_type: declared(int_ty),
        source: None,
    };
    let right_component = DataComponentSemantic {
        id: DataComponentId::new(data_owner.clone(), 1),
        local_name: "right".into(),
        external_label: None,
        declared_type: declared(int_ty),
        source: None,
    };
    let data_info = DataInfo {
        owner: data_owner.clone(),
        root_form: pair_ty,
        generic_signature: None,
        static_data: None,
        shape: DataShape::Tuple,
        components: vec![left_component.clone(), right_component.clone()].into_boxed_slice(),
        constructor: DataConstructorSignature {
            constructor: DataConstructorId::new(data_owner.clone()),
            visibility: Visibility::Public,
            parameters: vec![
                DataConstructorParameter {
                    component: left_component.id.clone(),
                    external_label: None,
                    local_name: left_component.local_name.clone(),
                    declared_type: declared(int_ty),
                },
                DataConstructorParameter {
                    component: right_component.id.clone(),
                    external_label: None,
                    local_name: right_component.local_name.clone(),
                    declared_type: declared(int_ty),
                },
            ]
            .into_boxed_slice(),
            result_type_template: pair_ty,
            source: None,
        },
        shape_template: DataProductShapeTemplate {
            shape: DataShape::Tuple,
            contributions: vec![
                DataProductContribution::Component(left_component.clone()),
                DataProductContribution::Component(right_component.clone()),
            ]
            .into_boxed_slice(),
        },
        constructor_template: None,
        source: None,
    };
    let mut data_table = DataSemanticTable::new();
    data_table.insert_data(Arc::new(data_info.clone()));

    let variant_a = VariantId::new(enum_owner.clone(), Selector::method("Same", [SelectorSlot::Positional]).unwrap());
    let variant_b = VariantId::new(
        enum_owner.clone(),
        Selector::method("Same", [SelectorSlot::Positional, SelectorSlot::Positional]).unwrap(),
    );
    let make_variant = |id: VariantId, variant_type: VariantTypeId, exact_case_template: TypeId| VariantInfo {
        id,
        type_handle: variant_type,
        family: None,
        shape: VariantShape::Singleton,
        payload_kind: AlgebraicCasePayloadKind::Singleton,
        fields: Box::new([]),
        result_type_template: enum_ty,
        exact_case_template,
        case_environment: CaseTypeEnvironment::default(),
        constructor: None,
        visibility: VariantVisibility::default(),
        source: None,
    };
    let variant_type_a = types.intern_variant_identity(variant_a.clone());
    let variant_type_b = types.intern_variant_identity(variant_b.clone());
    let exact_a = types.exact_case_type(&variant_a, enum_ty).unwrap();
    let exact_b = types.exact_case_type(&variant_b, enum_ty).unwrap();
    let info_a = make_variant(variant_a.clone(), variant_type_a, exact_a);
    let info_b = make_variant(variant_b.clone(), variant_type_b, exact_b);
    let mut enum_table = EnumSemanticTable::new();
    enum_table.insert_variant(Arc::new(info_a.clone()));
    enum_table.insert_variant(Arc::new(info_b.clone()));

    let mut values = StaticValueStore::new();
    let one = values.intern_int(1);
    let two = values.intern_int(2);
    let tuple = values.intern_tuple(vec![one, two]).unwrap();
    let reversed_tuple = values.intern_tuple(vec![two, one]).unwrap();
    assert!(!values.semantic_eq(tuple, reversed_tuple).unwrap());
    assert_ne!(values.content_fingerprint(tuple).unwrap(), values.content_fingerprint(reversed_tuple).unwrap());
    assert!(matches!(values.get(tuple), Ok(StaticValueData::Tuple(_))));

    let record_ab = values.intern_record(vec![(FieldKey::symbol("b"), two), (FieldKey::symbol("a"), one)]).unwrap();
    let record_ba = values.intern_record(vec![(FieldKey::symbol("a"), one), (FieldKey::symbol("b"), two)]).unwrap();
    assert_eq!(record_ab, record_ba, "record construction order is not semantic");
    assert!(matches!(
        values.intern_record(vec![(FieldKey::symbol("x"), one), (FieldKey::symbol("x"), two)]),
        Err(StaticValueStoreError::DuplicateRecordKey(_))
    ));

    let shared = values.intern_tuple(vec![one, two]).unwrap();
    let nested = values.intern_tuple(vec![shared, shared]).unwrap();
    assert!(values.semantic_eq(nested, nested).unwrap());
    assert_eq!(values.content_fingerprint(nested).unwrap(), values.content_fingerprint(nested).unwrap());
    let foreign = StaticValueStore::new().intern_bool(true);
    assert!(matches!(values.intern_tuple(vec![foreign]), Err(StaticValueStoreError::StoreMismatch { .. })));
    assert_eq!(values.intern_tuple(Vec::new()).unwrap(), values.intern_unit());

    let pair = values
        .intern_data(
            &data_info,
            pair_ty,
            vec![(right_component.id.clone(), two), (left_component.id.clone(), one)],
            &types,
        )
        .unwrap();
    let pair_reordered = values
        .intern_data(
            &data_info,
            pair_ty,
            vec![(left_component.id.clone(), one), (right_component.id.clone(), two)],
            &types,
        )
        .unwrap();
    assert!(
        values.semantic_eq(pair, pair_reordered).unwrap(),
        "payloads map through canonical component identities"
    );
    assert!(matches!(
        values.intern_data(&data_info, pair_ty, vec![(left_component.id.clone(), one)], &types),
        Err(StaticValueStoreError::InvalidDataIdentity)
    ));

    let value_a = values.intern_variant(&info_a, exact_a, vec![], &types).unwrap();
    let value_b = values.intern_variant(&info_b, exact_b, vec![], &types).unwrap();
    assert!(
        !values.semantic_eq(value_a, value_b).unwrap(),
        "same family name does not erase exact VariantId"
    );
    let field_id = VariantFieldId::new(variant_a.clone(), 0);
    let field = VariantFieldSemantic {
        id: field_id.clone(),
        local_name: "payload".into(),
        external_label: None,
        declared_type: declared(int_ty),
        source: None,
    };
    let mut payload_info = info_a.clone();
    payload_info.fields = vec![field].into_boxed_slice();
    assert!(matches!(
        values.intern_variant(&payload_info, exact_a, vec![], &types),
        Err(StaticValueStoreError::InvalidVariantPayload)
    ));

    let derived_tuple = values.static_value_type(tuple, &mut types, &data_table, &enum_table).unwrap();
    assert!(matches!(types.get(derived_tuple), phalcom_semantic::types::TypeData::Tuple(_)));
    let derived_record = values.static_value_type(record_ab, &mut types, &data_table, &enum_table).unwrap();
    assert!(matches!(types.get(derived_record), phalcom_semantic::types::TypeData::Record(_)));
    assert_eq!(values.static_value_type(pair, &mut types, &data_table, &enum_table).unwrap(), pair_ty);
    assert_eq!(values.static_value_type(value_a, &mut types, &data_table, &enum_table).unwrap(), exact_a);
    assert!(matches!(
        values
            .check_value_type(tuple, string_ty, &mut types, &data_table, &enum_table, &MapTypeHierarchy::new())
            .unwrap(),
        Assignability::Refuted { .. }
    ));
}

#[test]
fn structural_constant_descriptors_preserve_nested_values_and_labels() {
    use phalcom_semantic::materialization::{MaterializationCapability, MaterializationDescriptor};
    let mut values = StaticValueStore::new();
    let number = values.intern_int(7);
    let text = values.intern_string("retained");
    let tuple = values.intern_tuple(vec![number, text]).unwrap();
    let key = FieldKey::symbol("pair");
    let record = values.intern_record(vec![(key.clone(), tuple)]).unwrap();
    let label = values.intern_symbol_id("named");
    let pack = values.intern_pack(vec![tuple], vec![(label, record)]).unwrap();
    let descriptor = MaterializationDescriptor::from_static_value(&values, pack).unwrap();
    drop(values);
    let pair = MaterializationDescriptor::Tuple(Arc::from([
        MaterializationDescriptor::Int(phalcom_semantic::static_value::StaticInt::Small(7)),
        MaterializationDescriptor::String(Arc::from("retained")),
    ]));
    assert_eq!(
        descriptor,
        MaterializationDescriptor::Pack {
            positionals: Arc::from([pair.clone()]),
            labeled: Arc::from([(Arc::from("named"), MaterializationDescriptor::Record(Arc::from([(key, pair)])))]),
        }
    );
    assert_eq!(descriptor.capability(), MaterializationCapability::Sequence);
}

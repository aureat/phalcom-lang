use phalcom_common::modifier::Visibility;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::data_semantics::{DataConstructorSignature, DataInfo, DataProductShapeTemplate, DataSemanticTable, DataShape};
use phalcom_semantic::enum_semantics::{EnumInfo, EnumSemanticTable};
use phalcom_semantic::identity::{DataConstructorId, DeclarationId};
use phalcom_semantic::structural::{OrderedProductForm, StructuralClassification, StructuralClassifier};
use phalcom_semantic::types::store::{AnonymousTupleRemainder, PackTypeElement, TupleType, TupleTypeElement, TypeData, TypeStore};
use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner};
use std::sync::Arc;

fn data_info(owner: DeclarationId, root_form: phalcom_semantic::types::TypeId, shape: DataShape) -> DataInfo {
    DataInfo {
        owner: owner.clone(),
        root_form,
        generic_signature: None,
        static_data: None,
        shape,
        components: Box::new([]),
        constructor: DataConstructorSignature {
            constructor: DataConstructorId::new(owner),
            visibility: Visibility::Public,
            parameters: Box::new([]),
            result_type_template: root_form,
            source: None,
        },
        shape_template: DataProductShapeTemplate {
            shape,
            contributions: Box::new([]),
        },
        constructor_template: None,
        source: None,
    }
}

#[test]
fn classifier_uses_canonical_forms_and_declaration_products() {
    let module = ModuleId::universe_root();
    let tuple_owner = DeclarationId::new(module.clone(), "Pair".into());
    let record_owner = DeclarationId::new(module.clone(), "RecordLike".into());
    let sum_owner = DeclarationId::new(module.clone(), "Choice".into());
    let int_owner = DeclarationId::new(module.clone(), "Int".into());
    let misleading_name = DeclarationId::new(module, "Tuple".into());

    let mut types = TypeStore::new();
    let int_type = types.nominal_type(int_owner);
    let unit_type = types.unit();
    let tuple_type = types.tuple(vec![TupleTypeElement::unlabeled(int_type)].into_boxed_slice());
    let pack_type = types.pack(vec![PackTypeElement::labeled("label".into(), int_type)].into_boxed_slice()).unwrap();
    let pair_type = types.nominal_type(tuple_owner.clone());
    let record_type = types.nominal_type(record_owner.clone());
    let sum_type = types.nominal_type(sum_owner.clone());
    let name_only_tuple_type = types.nominal_type(misleading_name.clone());

    let mut data = DataSemanticTable::new();
    data.insert_data(Arc::new(data_info(tuple_owner.clone(), pair_type, DataShape::Tuple)));
    data.insert_data(Arc::new(data_info(record_owner, record_type, DataShape::Record)));

    let mut enums = EnumSemanticTable::new();
    enums.insert_enum(Arc::new(EnumInfo {
        owner: sum_owner.clone(),
        root_form: sum_type,
        generic_signature: None,
        static_data: None,
        default_result_type: sum_type,
        variants: Box::new([]),
        variant_families: Box::new([]),
        source: None,
    }));

    let classifier = StructuralClassifier::new(&data, &enums);
    assert_eq!(
        classifier.classify(&types, int_type),
        StructuralClassification::Unsupported,
        "ordinary scalar types do not acquire structural status"
    );
    assert_eq!(
        classifier.classify(&types, unit_type),
        StructuralClassification::OrderedProduct(OrderedProductForm::Unit)
    );
    assert_eq!(
        classifier.classify(&types, tuple_type),
        StructuralClassification::OrderedProduct(OrderedProductForm::Tuple)
    );
    assert_eq!(
        classifier.classify(&types, pack_type),
        StructuralClassification::OrderedProduct(OrderedProductForm::Pack)
    );
    assert_eq!(
        classifier.classify(&types, pair_type),
        StructuralClassification::OrderedProduct(OrderedProductForm::Data { owner: tuple_owner })
    );
    assert_eq!(
        classifier.classify(&types, sum_type),
        StructuralClassification::Sum { owner: sum_owner }
    );
    assert_eq!(
        classifier.classify(&types, record_type),
        StructuralClassification::Unsupported,
        "Record-shaped nominal data are not ordered products"
    );
    assert_eq!(
        classifier.classify(&types, name_only_tuple_type),
        StructuralClassification::Unsupported,
        "source spelling does not substitute for canonical declaration products"
    );
}

#[test]
fn ordered_descriptor_keeps_lanes_remainders_and_symbolic_pack_terms() {
    let module = ModuleId::universe_root();
    let int_owner = DeclarationId::new(module.clone(), "Int".into());
    let data = DataSemanticTable::new();
    let enums = EnumSemanticTable::new();
    let mut types = TypeStore::new();
    let int_type = types.nominal_type(int_owner);
    let string_owner = DeclarationId::new(module, "String".into());
    let string_type = types.nominal_type(string_owner);
    let classifier = StructuralClassifier::new(&data, &enums);

    let pack = types
        .open_pack(
            vec![
                PackTypeElement::unlabeled(int_type),
                PackTypeElement::labeled("name".into(), string_type),
            ]
            .into_boxed_slice(),
            AnonymousTupleRemainder::Typed(int_type),
            AnonymousTupleRemainder::Unconstrained,
        )
        .unwrap();
    let descriptor = classifier.ordered_product(&mut types, pack).unwrap().unwrap();
    assert!(matches!(descriptor.origin, phalcom_semantic::structural::OrderedProductOrigin::Pack));
    assert_eq!(descriptor.positional_entries.len(), 1);
    assert_eq!(descriptor.positional_entries[0].ty, Some(int_type));
    assert_eq!(descriptor.labeled_entries.len(), 1);
    assert_eq!(descriptor.labeled_entries[0].label.as_deref(), Some("name"));
    assert_eq!(descriptor.labeled_entries[0].ty, Some(string_type));
    assert_eq!(descriptor.positional_remainder, AnonymousTupleRemainder::Typed(int_type));
    assert_eq!(descriptor.labeled_remainder, AnonymousTupleRemainder::Unconstrained);

    let open_tuple = types.intern_with_kind(
        TypeData::Tuple(TupleType {
            elements: vec![TupleTypeElement::labeled("tag".into(), string_type)].into_boxed_slice(),
            positional_remainder: AnonymousTupleRemainder::Unconstrained,
            labeled_remainder: AnonymousTupleRemainder::Typed(int_type),
        }),
        phalcom_semantic::types::id::KindId::TYPE,
    );
    let descriptor = classifier.ordered_product(&mut types, open_tuple).unwrap().unwrap();
    assert_eq!(descriptor.origin, phalcom_semantic::structural::OrderedProductOrigin::Tuple);
    assert!(descriptor.positional_entries.is_empty());
    assert_eq!(descriptor.labeled_entries[0].label.as_deref(), Some("tag"));
    assert_eq!(descriptor.positional_remainder, AnonymousTupleRemainder::Unconstrained);
    assert_eq!(descriptor.labeled_remainder, AnonymousTupleRemainder::Typed(int_type));

    let parameter_owner = TypeParameterOwner::Declaration(DeclarationId::new(ModuleId::universe_root(), "SymbolicTuple".into()));
    let positional_parameter = types.intern_type_parameter(TypeParameterData::new(
        parameter_owner.clone(),
        0,
        "Ps",
        phalcom_semantic::types::id::KindId::TYPE_PACK,
    ));
    let labeled_parameter = types.intern_type_parameter(TypeParameterData::new(
        parameter_owner,
        1,
        "Ls",
        phalcom_semantic::types::id::KindId::LABELED_TYPE_PACK,
    ));
    let positionals = types.parameter_form(positional_parameter);
    let labeled = types.parameter_form(labeled_parameter);
    let symbolic_tuple = types.tuple_from_packs(positionals, labeled).unwrap();
    let descriptor = classifier.ordered_product(&mut types, symbolic_tuple).unwrap().unwrap();
    assert_eq!(
        descriptor.origin,
        phalcom_semantic::structural::OrderedProductOrigin::TupleFromPacks
    );
    assert_eq!(descriptor.symbolic_lanes, Some((positionals, labeled)));
}

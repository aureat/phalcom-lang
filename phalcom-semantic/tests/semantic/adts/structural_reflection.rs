use phalcom_native_meta::UniverseKey;
use phalcom_semantic::core_surface::universe_declaration;
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::structural::{OrderedProductOrigin, StructuralClassifier};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::store::TypeData;

#[test]
fn ordered_data_reflection_specializes_components_and_keeps_nominal_owners() {
    let case = super::support::analyze_adt(
        r#"
data Pair<T>(_ first: T, _ second: String)
data Twin<T>(_ first: T, _ second: String)
"#,
    );
    case.assert_no_diagnostics();

    let snapshot = &case.analysis.snapshot;
    let mut types = (*snapshot.store).clone();
    let int = types.nominal_type(universe_declaration(UniverseKey::Int));
    let string = types.nominal_type(universe_declaration(UniverseKey::String));
    let classifier = StructuralClassifier::new(&snapshot.data_semantics, &snapshot.enum_semantics);

    for name in ["Pair", "Twin"] {
        let owner = DeclarationId::new(case.module.clone(), name.into());
        let info = snapshot.data_semantics.data_info(&owner).expect("data semantic product");
        assert_eq!(classifier.classify(&types, info.root_form), phalcom_semantic::StructuralClassification::Unsupported);
        let specialized = types.apply_type_form(info.root_form, &[int]).expect("generic data specialization");
        let structure = classifier.ordered_product(&mut types, specialized).unwrap().expect("ordered data structure");
        assert_eq!(structure.origin, OrderedProductOrigin::Data { owner });
        assert_eq!(structure.state, phalcom_semantic::data_semantics::DataProductShapeState::Concrete);
        assert_eq!(structure.positional_entries.len(), 2);
        assert_eq!(structure.positional_entries[0].ty, Some(int));
        assert_eq!(structure.positional_entries[1].ty, Some(string));
        assert!(structure.labeled_entries.is_empty());
    }

    let pair = DeclarationId::new(case.module.clone(), "Pair".into());
    let twin = DeclarationId::new(case.module, "Twin".into());
    let pair_info = snapshot.data_semantics.data_info(&pair).unwrap();
    let twin_info = snapshot.data_semantics.data_info(&twin).unwrap();
    let pair_type = types.apply_type_form(pair_info.root_form, &[int]).unwrap();
    let twin_type = types.apply_type_form(twin_info.root_form, &[int]).unwrap();
    assert_ne!(pair_type, twin_type, "identical payloads do not erase nominal identity");
    assert!(matches!(types.get(pair_type), TypeData::Applied { .. }));
    assert!(matches!(types.get(twin_type), TypeData::Applied { .. }));
}

#[test]
fn sum_reflection_is_ordered_shallow_and_specializes_variant_fields() {
    let case = super::support::analyze_adt(
        r#"
data Result<T, E> {
    @variant Ok(_ value: T) -> Result<T, E>
    @variant Err(_ error: E) -> Result<T, E>
}
data Node {
    @variant Next(_ child: Node) -> Node
}
data Family {
    @variant Same
    @variant Same()
    @variant Same(_ value: Int)
}
data Expr<T> {
    @variant Int(_ value: Int) -> Expr<Int>
    @variant Bool(_ value: Bool) -> Expr<Bool>
}
"#,
    );
    case.assert_no_diagnostics();

    let snapshot = &case.analysis.snapshot;
    let mut types = (*snapshot.store).clone();
    let classifier = StructuralClassifier::new(&snapshot.data_semantics, &snapshot.enum_semantics);
    let int = types.nominal_type(universe_declaration(UniverseKey::Int));
    let string = types.nominal_type(universe_declaration(UniverseKey::String));

    let result_owner = DeclarationId::new(case.module.clone(), "Result".into());
    let result_info = snapshot.enum_semantics.enum_info(&result_owner).expect("Result EnumInfo");
    let result_type = types.apply_type_form(result_info.root_form, &[int, string]).expect("specialized Result");
    let result = classifier.sum(&mut types, result_type).unwrap().expect("Sum descriptor");
    assert_eq!(result.ty, result_type);
    assert_eq!(result.owner, result_owner);
    assert_eq!(
        result.variants.iter().map(|variant| &variant.declaration.variant).collect::<Vec<_>>(),
        result_info.variants.iter().collect::<Vec<_>>(),
        "exact Variant identities retain declaration order"
    );
    assert_eq!(result.variants[0].exact_case.fields[0].specialized_type, Some(int));
    assert_eq!(result.variants[1].exact_case.fields[0].specialized_type, Some(string));
    assert!(result
        .variants
        .iter()
        .all(|variant| variant.exact_case.result_type == result_type));

    let node_owner = DeclarationId::new(case.module.clone(), "Node".into());
    let node_type = types.nominal_type(node_owner.clone());
    let node = classifier.sum(&mut types, node_type).unwrap().expect("recursive Node Sum");
    assert_eq!(node.variants.len(), 1);
    assert_eq!(node.variants[0].exact_case.fields[0].specialized_type, Some(node_type));
    assert_eq!(node.variants[0].exact_case.enum_type, node_type, "reflection stores a reference instead of expanding recursively");

    let family_owner = DeclarationId::new(case.module.clone(), "Family".into());
    let family_type = types.nominal_type(family_owner.clone());
    let family_info = snapshot.enum_semantics.enum_info(&family_owner).unwrap();
    let family = classifier.sum(&mut types, family_type).unwrap().expect("same-name Variant family");
    assert_eq!(family.variants.len(), 3);
    assert_eq!(
        family.variants.iter().map(|variant| &variant.declaration.variant).collect::<Vec<_>>(),
        family_info.variants.iter().collect::<Vec<_>>()
    );
    let family_id = family.variants[0].declaration.family.clone().expect("family identity");
    assert!(family.variants.iter().all(|variant| variant.declaration.family.as_ref() == Some(&family_id)));

    let expr_owner = DeclarationId::new(case.module, "Expr".into());
    let expr_info = snapshot.enum_semantics.enum_info(&expr_owner).unwrap();
    assert!(expr_info
        .variants
        .iter()
        .any(|variant| snapshot.enum_semantics.variant_info(variant).unwrap().case_environment.bindings.len() > 0));
    let expr_type = types.apply_type_form(expr_info.root_form, &[int]).expect("specialized Expr");
    let expr = classifier.sum(&mut types, expr_type).unwrap().expect("GADT Sum structure");
    assert_eq!(expr.variants.len(), 2);
    assert_eq!(expr.variants[0].exact_case.fields[0].specialized_type, Some(int));
    assert_eq!(expr.variants[1].exact_case.fields[0].specialized_type, Some(types.nominal_type(universe_declaration(UniverseKey::Bool))));
    assert_eq!(types.kind_of(expr_type), KindId::TYPE);
}

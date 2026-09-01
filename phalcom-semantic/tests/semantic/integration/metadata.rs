use phalcom_modules::{DeclarationId, ModuleComponent, ModuleId, ModulePath};
use phalcom_semantic::declarations::{DeclarationTypeInfo, DeclarationTypeTable, GenericSupertypeTemplate};
use phalcom_semantic::metadata::MetadataExporter;
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::parameter::{GenericConstraint, GenericSignature, TypeParameterData, TypeParameterOwner, TypeTerm};
use phalcom_semantic::types::store::TypeStore;
use phalcom_semantic::types::variance::Variance;
use phalcom_type_meta::header::MetadataProfile;
use phalcom_type_meta::validate::{ValidationLimits, validate_metadata_bundle};

fn dummy_module() -> ModuleId {
    ModuleId::universe(
        ModulePath::from_components(vec![
            ModuleComponent::from_identifier("collections").unwrap(),
            ModuleComponent::from_identifier("list").unwrap(),
        ]),
    )
}

fn dummy_decl(name: &str) -> DeclarationId {
    DeclarationId::new(dummy_module(), name.into())
}

#[test]
fn test_metadata_fresh_store_determinism() {
    let mut store1 = TypeStore::new();
    let mut store2 = TypeStore::new();

    let int1 = store1.nominal(dummy_decl("Int"));
    let list_kind1 = store1.arrow_kind(Box::new([KindId::TYPE]), KindId::TYPE);
    let list1 = store1.nominal_form(dummy_decl("List"), list_kind1);
    let list_int1 = store1.apply_type_form(list1, &[int1]).unwrap();

    let int2 = store2.nominal(dummy_decl("Int"));
    let list_kind2 = store2.arrow_kind(Box::new([KindId::TYPE]), KindId::TYPE);
    let list2 = store2.nominal_form(dummy_decl("List"), list_kind2);
    let list_int2 = store2.apply_type_form(list2, &[int2]).unwrap();

    let exporter1 = MetadataExporter::new(&store1, None, None, None, MetadataProfile::RuntimePublic);
    let mod_id = dummy_module();
    let bundle1 = exporter1.build_bundle(&[(&mod_id, "root", list_int1)]).unwrap();

    let exporter2 = MetadataExporter::new(&store2, None, None, None, MetadataProfile::RuntimePublic);
    let bundle2 = exporter2.build_bundle(&[(&mod_id, "root", list_int2)]).unwrap();

    assert_eq!(bundle1, bundle2);
    validate_metadata_bundle(&bundle1, &ValidationLimits::default()).unwrap();
}

#[test]
fn test_metadata_lambda_alpha_equivalence() {
    let mut store1 = TypeStore::new();
    let mut store2 = TypeStore::new();

    let list1 = store1.nominal(dummy_decl("List"));
    let bound1 = store1
        .arena_mut()
        .intern_scoped(phalcom_semantic::types::ScopedTypeData::Bound { depth: 0, index: 0 });
    let free_list1 = store1.arena_mut().intern_scoped(phalcom_semantic::types::ScopedTypeData::Free(list1));
    let applied1 = store1.arena_mut().intern_scoped(phalcom_semantic::types::ScopedTypeData::Applied {
        origin: free_list1,
        arguments: Box::new([bound1]),
    });
    let lam1 = store1.lambda(Box::new([KindId::TYPE]), applied1, KindId::TYPE);

    let list2 = store2.nominal(dummy_decl("List"));
    let bound2 = store2
        .arena_mut()
        .intern_scoped(phalcom_semantic::types::ScopedTypeData::Bound { depth: 0, index: 0 });
    let free_list2 = store2.arena_mut().intern_scoped(phalcom_semantic::types::ScopedTypeData::Free(list2));
    let applied2 = store2.arena_mut().intern_scoped(phalcom_semantic::types::ScopedTypeData::Applied {
        origin: free_list2,
        arguments: Box::new([bound2]),
    });
    let lam2 = store2.lambda(Box::new([KindId::TYPE]), applied2, KindId::TYPE);

    let exporter1 = MetadataExporter::new(&store1, None, None, None, MetadataProfile::RuntimePublic);
    let mod_id = dummy_module();
    let bundle1 = exporter1.build_bundle(&[(&mod_id, "lam", lam1)]).unwrap();

    let exporter2 = MetadataExporter::new(&store2, None, None, None, MetadataProfile::RuntimePublic);
    let bundle2 = exporter2.build_bundle(&[(&mod_id, "lam", lam2)]).unwrap();

    assert_eq!(bundle1.types, bundle2.types);
    assert_eq!(bundle1.scoped_types, bundle2.scoped_types);
    validate_metadata_bundle(&bundle1, &ValidationLimits::default()).unwrap();
}

#[test]
fn test_metadata_generic_signature_and_supertype_template_export() {
    let mut store = TypeStore::new();
    let decl = dummy_decl("Names");
    let seq = dummy_decl("Sequence");

    let p0 = store
        .intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(decl.clone()), 0, "T", KindId::TYPE).with_variance(Variance::Covariant));
    let p0_form = store.parameter_form(p0);

    let seq_kind = store.arrow_kind(Box::new([KindId::TYPE]), KindId::TYPE);
    let seq_form = store.nominal_form(seq.clone(), seq_kind);
    let seq_t = store.apply_type_form(seq_form, &[p0_form]).unwrap();

    let names_form = store.nominal(decl.clone());

    let mut decl_table = DeclarationTypeTable::new();
    decl_table.insert(DeclarationTypeInfo {
        declaration: decl.clone(),
        form: names_form,
        class_object_type: store.class_object_type(decl.clone()),
        kind: store.kind_of(names_form),
        generic_signature: Some(GenericSignature::with_constraints(
            TypeParameterOwner::Declaration(decl.clone()),
            Box::new([p0]),
            Box::new([GenericConstraint::Subtype {
                lower: TypeTerm::Canonical(p0_form),
                upper: TypeTerm::Canonical(store.unit()),
            }]),
        )),
        supertype_template: Some(GenericSupertypeTemplate {
            declaration: decl.clone(),
            supertype: seq_t,
        }),
    });

    let exporter = MetadataExporter::new(&store, Some(&decl_table), None, None, MetadataProfile::RuntimePublic);
    let bundle = exporter.build_bundle(&[]).unwrap();

    validate_metadata_bundle(&bundle, &ValidationLimits::default()).unwrap();
    assert_eq!(bundle.declarations.len(), 1);
    assert!(bundle.declarations[0].superclass_template.is_some());
    assert!(bundle.declarations[0].generic_signature.is_some());
    assert_eq!(bundle.generic_signatures.len(), 1);
    assert_eq!(bundle.generic_signatures[0].constraints.len(), 1);
    assert_eq!(bundle.parameters.len(), 1);
    assert_eq!(bundle.parameters[0].variance, phalcom_type_meta::generic::VarianceRef::Covariant);
}

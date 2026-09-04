use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_modules::{DeclarationId, ModuleComponent, ModuleId, ModulePath, ProjectIdentity, ProjectUniverse};
use phalcom_semantic::declarations::{DeclarationTypeInfo, DeclarationTypeTable, GenericSupertypeTemplate};
use phalcom_semantic::diagnostic::SemanticSourceSpan;
use phalcom_semantic::identity::{CallableId, DispatchSide};
use phalcom_semantic::metadata::MetadataExportError;
use phalcom_semantic::metadata::MetadataExporter;
use phalcom_semantic::metadata::stable_identity::StableIdentityContext;
use phalcom_semantic::signature::{CallableParameterSemantic, CallableSemanticSignature, ReturnContractValidation};
use phalcom_semantic::type_alias::{TypeAliasInfo, TypeAliasTable};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::parameter::{GenericConstraint, GenericSignature, SelfRole, SelfTypeTerm, TypeParameterData, TypeParameterOwner, TypeTerm};
use phalcom_semantic::types::row::{RecordRowField, RecordRowTail};
use phalcom_semantic::types::store::TypeStore;
use phalcom_semantic::types::variance::Variance;
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::header::MetadataProfile;
use phalcom_type_meta::identity::StableProjectRef;
use phalcom_type_meta::validate::{ValidationLimits, validate_metadata_bundle};

fn dummy_module() -> ModuleId {
    ModuleId::universe(ModulePath::from_components(vec![
        ModuleComponent::from_identifier("collections").unwrap(),
        ModuleComponent::from_identifier("list").unwrap(),
    ]))
}

fn dummy_decl(name: &str) -> DeclarationId {
    DeclarationId::new(dummy_module(), name.into())
}

#[test]
fn stable_project_identity_ignores_graph_allocation_order() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("main.ph"), "value").unwrap();
    let other = tempfile::tempdir().unwrap();

    let mut first = ProjectUniverse::new();
    let first_id = first.load_synthetic_root("sample", root.path(), "main").unwrap();

    let mut second = ProjectUniverse::new();
    second.load_synthetic_root("other", other.path(), "main").unwrap();
    let second_id = second.load_synthetic_root("sample", root.path(), "main").unwrap();

    let first_decl = DeclarationId::new(ModuleId::resolved(first_id, ModulePath::root()), "Thing".into());
    let second_decl = DeclarationId::new(ModuleId::resolved(second_id, ModulePath::root()), "Thing".into());
    let first_ref = phalcom_semantic::metadata::stable_identity::to_stable_declaration_with_context(&first_decl, &StableIdentityContext::new(&first));
    let second_ref = phalcom_semantic::metadata::stable_identity::to_stable_declaration_with_context(&second_decl, &StableIdentityContext::new(&second));
    assert_eq!(first_ref, second_ref);
}

#[test]
fn stable_project_identity_distinguishes_roots_and_revisions() {
    let left = tempfile::tempdir().unwrap();
    let right = tempfile::tempdir().unwrap();
    std::fs::write(left.path().join("main.ph"), "value").unwrap();
    std::fs::write(right.path().join("main.ph"), "value").unwrap();

    let mut projects = ProjectUniverse::new();
    let left_id = projects.load_synthetic_root("sample", left.path(), "main").unwrap();
    let right_id = projects.load_synthetic_root("sample", right.path(), "main").unwrap();
    let left_decl = DeclarationId::new(ModuleId::resolved(left_id, ModulePath::root()), "Thing".into());
    let right_decl = DeclarationId::new(ModuleId::resolved(right_id, ModulePath::root()), "Thing".into());
    let context = StableIdentityContext::new(&projects);
    assert_ne!(
        phalcom_semantic::metadata::stable_identity::to_stable_declaration_with_context(&left_decl, &context),
        phalcom_semantic::metadata::stable_identity::to_stable_declaration_with_context(&right_decl, &context)
    );

    let before = projects.get_project(left_id).unwrap().revision_fingerprint();
    std::fs::write(left.path().join("main.ph"), "changed").unwrap();
    let after = projects.get_project(left_id).unwrap().revision_fingerprint();
    assert_ne!(before, after);
}

#[test]
fn stable_builtin_and_synthetic_projects_remain_explicitly_scoped() {
    let builtin = phalcom_semantic::metadata::stable_identity::to_stable_project(&ProjectIdentity::Universe);
    assert!(matches!(builtin, StableProjectRef::Builtin { namespace, .. } if namespace.as_ref() == "universe"));

    let mut projects = ProjectUniverse::new();
    let synthetic = projects.allocate_synthetic_id();
    let session = phalcom_semantic::metadata::stable_identity::to_stable_project(&ProjectIdentity::Synthetic(synthetic));
    assert!(matches!(session, StableProjectRef::Session { session_fingerprint } if session_fingerprint != Fingerprint128::ZERO));
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
fn metadata_export_rejects_query_local_rigid_types() {
    let store = TypeStore::new();
    let mut arena = phalcom_semantic::types::rigid::RigidArena::new();
    let scope = arena.fresh_scope(None);
    let rigid = arena.fresh(scope, KindId::TYPE, phalcom_semantic::types::rigid::RigidOrigin::Synthetic);
    let local = phalcom_semantic::types::rigid::LocalType::Rigid(rigid);
    let exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    assert!(matches!(exporter.validate_local_type_export(&local), Err(MetadataExportError::ScopedLocalType)));
}

#[test]
fn metadata_round_trip_preserves_callable_generic_owner_and_constraints() {
    let mut store = TypeStore::new();
    let owner = dummy_decl("NativeEquivalent");
    let selector = Selector::method("identity", [phalcom_common::selector::SelectorSlot::Positional]).unwrap();
    let callable = CallableId::new(owner.clone(), selector.clone(), DispatchSide::Instance);
    let parameter = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Callable(callable.clone()), 0, "T", KindId::TYPE));
    let parameter_form = store.parameter_form(parameter);
    let signature = GenericSignature::with_constraints(
        TypeParameterOwner::Callable(callable.clone()),
        Box::new([parameter]),
        Box::new([GenericConstraint::Subtype {
            lower: TypeTerm::Canonical(parameter_form),
            upper: TypeTerm::Canonical(store.unit()),
        }]),
    );
    let mut callables = phalcom_semantic::signature::CallableSignatureTable::new();
    callables.insert(CallableSemanticSignature {
        callable: callable.clone(),
        owner,
        side: DispatchSide::Instance,
        selector,
        generics: Some(signature),
        parameters: Box::new([CallableParameterSemantic::new(
            phalcom_semantic::identity::CallableParameterId::new(callable.clone(), 0),
            "value",
            phalcom_semantic::declaration_type::DeclaredTypeFact::known(
                TypeTerm::Canonical(parameter_form),
                phalcom_semantic::declaration_type::DeclaredTypeBasis::NativeSignature,
            ),
        )]),
        declared_return: phalcom_semantic::declaration_type::DeclaredTypeFact::known(
            TypeTerm::Canonical(parameter_form),
            phalcom_semantic::declaration_type::DeclaredTypeBasis::NativeSignature,
        ),
        return_validation: ReturnContractValidation::NotApplicable,
        inferred_return: None,
        source: None,
        implementation: phalcom_native_meta::ImplementationKind::NativePrimitive,
        native_id: None,
        effects: phalcom_native_meta::EffectSpec::Pure,
        raises: phalcom_native_meta::RaisesSpec::Unknown,
        flow: phalcom_native_meta::ReturnFlowSpec::Value,
        lifecycle: phalcom_native_meta::NativeLifecycleSpec::UNKNOWN,
    });

    let exporter = MetadataExporter::new(&store, None, Some(&callables), None, MetadataProfile::RuntimePublic);
    let bundle = exporter.build_bundle(&[]).expect("callable metadata");
    validate_metadata_bundle(&bundle, &ValidationLimits::default()).unwrap();
    let callable_record = bundle.callables.first().expect("callable record");
    let generic_id = callable_record.generic_signature.expect("callable generic signature");
    let generic_record = &bundle.generic_signatures[generic_id.0 as usize];
    assert!(matches!(
        generic_record.owner,
        phalcom_type_meta::generic::StableTypeParameterOwnerRef::Callable(_)
    ));
    assert_eq!(generic_record.parameters.len(), 1);
    assert_eq!(generic_record.constraints.len(), 1);
}

#[test]
fn open_record_metadata_is_tail_sensitive_and_enables_row_feature() {
    let mut store = TypeStore::new();
    let int = store.nominal(dummy_decl("Int"));
    let owner_one = CallableId::new(dummy_decl("Owner"), Selector::getter("row-one").unwrap(), DispatchSide::Instance);
    let owner_two = CallableId::new(dummy_decl("Owner"), Selector::getter("row-two").unwrap(), DispatchSide::Instance);
    let row_one = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Callable(owner_one), 0, "R", KindId::RECORD_ROW));
    let row_two = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Callable(owner_two), 0, "R", KindId::RECORD_ROW));
    let fields = vec![RecordRowField { name: "value".into(), ty: int }];
    let closed = store.record_row_type_checked(fields.clone(), RecordRowTail::Closed).unwrap();
    let open_one = store.record_row_type_checked(fields.clone(), RecordRowTail::Parameter(row_one)).unwrap();
    let open_two = store.record_row_type_checked(fields, RecordRowTail::Parameter(row_two)).unwrap();

    let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let closed_id = exporter.export_type_form(closed).unwrap();
    let open_one_id = exporter.export_type_form(open_one).unwrap();
    let open_two_id = exporter.export_type_form(open_two).unwrap();
    let bundle = exporter.build_bundle(&[]).unwrap();

    assert!(matches!(
        bundle.types[closed_id.0 as usize].form,
        phalcom_type_meta::type_node::TypeNode::Record(_)
    ));
    let phalcom_type_meta::type_node::TypeNode::OpenRecord(open_ref) = &bundle.types[open_one_id.0 as usize].form else {
        panic!("open record must use OpenRecord metadata node")
    };
    assert_eq!(open_ref.fields.len(), 1);
    assert_eq!(
        open_ref.tail,
        bundle.parameters.iter().find(|parameter| parameter.id == open_ref.tail).unwrap().id
    );
    assert_ne!(
        bundle.types[open_one_id.0 as usize].structural_fingerprint, bundle.types[open_two_id.0 as usize].structural_fingerprint,
        "stable callable owner must contribute to open-row fingerprint",
    );
    assert_ne!(
        bundle.types[closed_id.0 as usize].structural_fingerprint,
        bundle.types[open_one_id.0 as usize].structural_fingerprint,
    );
    assert!(bundle.header.features.record_rows);
    validate_metadata_bundle(&bundle, &ValidationLimits::default()).unwrap();
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
            structural_form: None,
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

#[test]
fn test_metadata_exports_transparent_alias_records_and_generic_forms() {
    let mut store = TypeStore::new();
    let alias = dummy_decl("UserId");
    let int = store.nominal(dummy_decl("Int"));
    let generic_alias = dummy_decl("ListAlias");
    let parameter = store.intern_type_parameter(TypeParameterData::new(
        TypeParameterOwner::Declaration(generic_alias.clone()),
        0,
        "T",
        KindId::TYPE,
    ));
    let list_kind = store.arrow_kind(Box::new([KindId::TYPE]), KindId::TYPE);
    let list = store.nominal_form(dummy_decl("List"), list_kind);
    let bound = store
        .arena_mut()
        .intern_scoped(phalcom_semantic::types::ScopedTypeData::Bound { depth: 0, index: 0 });
    let free_list = store.arena_mut().intern_scoped(phalcom_semantic::types::ScopedTypeData::Free(list));
    let applied = store.arena_mut().intern_scoped(phalcom_semantic::types::ScopedTypeData::Applied {
        origin: free_list,
        arguments: Box::new([bound]),
    });
    let generic_form = store.lambda(Box::new([KindId::TYPE]), applied, KindId::TYPE);
    let generic_signature = GenericSignature::new(TypeParameterOwner::Declaration(generic_alias.clone()), Box::new([parameter]));
    let aliases = {
        let mut table = TypeAliasTable::new();
        table.insert(TypeAliasInfo {
            declaration: alias.clone(),
            kind: KindId::TYPE,
            kind_shape: "Type".into(),
            generic_signature: None,
            form: int,
            structural_form: "Int".into(),
            dependencies: Box::new([]),
            source: SemanticSourceSpan::new(dummy_module(), SourceRange { start: 4, end: 10 }),
        });
        table.insert(TypeAliasInfo {
            declaration: generic_alias,
            kind: list_kind,
            kind_shape: "(Type) -> Type".into(),
            generic_signature: Some(generic_signature),
            form: generic_form,
            structural_form: "<T> =>> List<T>".into(),
            dependencies: Box::new([]),
            source: SemanticSourceSpan::new(dummy_module(), SourceRange { start: 11, end: 30 }),
        });
        table
    };

    let exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic).with_aliases(&aliases);
    let bundle = exporter.build_bundle(&[]).expect("valid alias metadata");
    assert_eq!(bundle.aliases.len(), 2);
    let user_id = bundle
        .aliases
        .iter()
        .find(|record| record.declaration.path[0].as_ref() == "UserId")
        .expect("UserId alias");
    let target = &bundle.types[user_id.target.0 as usize].form;
    assert!(matches!(target, phalcom_type_meta::type_node::TypeNode::Nominal { declaration } if declaration.path[0].as_ref() == "Int"));
    let list_alias = bundle
        .aliases
        .iter()
        .find(|record| record.declaration.path[0].as_ref() == "ListAlias")
        .expect("generic alias");
    assert!(list_alias.generic_signature.is_some());
    assert!(matches!(
        &bundle.types[list_alias.target.0 as usize].form,
        phalcom_type_meta::type_node::TypeNode::TypeLambda(_)
    ));
    assert_eq!(bundle.generic_signatures.len(), 1);
    assert_eq!(bundle.parameters.len(), 1);
    validate_metadata_bundle(&bundle, &ValidationLimits::default()).unwrap();
}

#[test]
fn test_metadata_rejects_stale_alias_form_before_publication() {
    let store = TypeStore::new();
    let alias = dummy_decl("BrokenAlias");
    let mut aliases = TypeAliasTable::new();
    aliases.insert(TypeAliasInfo {
        declaration: alias,
        kind: KindId::TYPE,
        kind_shape: "Type".into(),
        generic_signature: None,
        form: phalcom_semantic::types::id::TypeId::DUMMY,
        structural_form: "<stale>".into(),
        dependencies: Box::new([]),
        source: SemanticSourceSpan::new(dummy_module(), SourceRange { start: 0, end: 1 }),
    });

    let exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic).with_aliases(&aliases);
    assert!(matches!(
        exporter.build_bundle(&[]),
        Err(phalcom_semantic::metadata::MetadataExportError::NonExportableForm(form))
            if form == phalcom_semantic::types::id::TypeId::DUMMY
    ));
}

#[test]
fn test_metadata_rejects_inference_variable_signature_before_publication() {
    let mut store = TypeStore::new();
    let owner = TypeParameterOwner::Declaration(dummy_decl("SolverLeak"));
    let parameter = store.intern_type_parameter(TypeParameterData::new(owner.clone(), 0, "T", KindId::TYPE));
    let signature = GenericSignature::with_constraints(
        owner,
        Box::new([parameter]),
        Box::new([GenericConstraint::Equivalent {
            left: TypeTerm::Infer(phalcom_semantic::types::id::InferVarId(11)),
            right: TypeTerm::Canonical(store.unit()),
        }]),
    );
    let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    assert!(matches!(
        exporter.export_generic_signature(&signature),
        Err(phalcom_semantic::metadata::MetadataExportError::InvalidGenericSignature(
            phalcom_semantic::types::parameter::GenericSignaturePublicationError::InferenceVariable { .. }
        ))
    ));
}

#[test]
fn test_metadata_preserves_owner_relative_self_terms() {
    let mut store = TypeStore::new();
    let owner = dummy_decl("Owner");
    let instance = store.self_type(SelfTypeTerm {
        owner: owner.clone(),
        side: DispatchSide::Instance,
        role: SelfRole::InstanceType,
    });
    let class = store.self_type(SelfTypeTerm {
        owner,
        side: DispatchSide::Class,
        role: SelfRole::ReceiverValue,
    });
    let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let instance_id = exporter.export_type_form(instance).expect("instance Self export");
    let class_id = exporter.export_type_form(class).expect("class Self export");
    assert_ne!(instance_id, class_id);
    let bundle = exporter.build_bundle(&[]).expect("Self terms export");
    assert!(matches!(
        &bundle.types[instance_id.0 as usize].form,
        phalcom_type_meta::type_node::TypeNode::SelfType(self_ref)
            if matches!(self_ref.side, phalcom_type_meta::identity::StableDispatchSide::Instance)
    ));
    assert!(matches!(
        &bundle.types[class_id.0 as usize].form,
        phalcom_type_meta::type_node::TypeNode::SelfType(self_ref)
            if matches!(self_ref.side, phalcom_type_meta::identity::StableDispatchSide::Class)
    ));
}

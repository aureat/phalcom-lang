use phalcom_common::range::SourceRange;
use phalcom_common::selector::Selector;
use phalcom_modules::{DeclarationId, ModuleComponent, ModuleId, ModulePath, ProjectIdentity, ProjectUniverse};
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::declarations::{DeclarationTypeInfo, DeclarationTypeTable, GenericSupertypeTemplate};
use phalcom_semantic::diagnostic::SemanticSourceSpan;
use phalcom_semantic::identity::{CallableId, DispatchSide};
use phalcom_semantic::metadata::MetadataExportError;
use phalcom_semantic::metadata::MetadataExporter;
use phalcom_semantic::metadata::stable_identity::StableIdentityContext;
use phalcom_semantic::signature::{CallableParameterSemantic, CallableSemanticSignature, ReturnContractValidation};
use phalcom_semantic::traits::{AssociatedTypeRequirementId, TraitRef};
use phalcom_semantic::type_alias::{TypeAliasInfo, TypeAliasTable};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::parameter::{
    GenericConstraint, GenericParameterMode, GenericSignature, SelfRole, SelfTypeTerm, TypeParameterData, TypeParameterOwner, TypeTerm,
};
use phalcom_semantic::types::row::{RecordRowField, RecordRowTail};
use phalcom_semantic::types::store::{StaticArgument, TypeStore};
use phalcom_semantic::types::variance::Variance;
use phalcom_type_meta::declaration::{AlgebraicPhaseRef, AlgebraicShapeRef};
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::header::MetadataProfile;
use phalcom_type_meta::identity::StableProjectRef;
use phalcom_type_meta::kind::{KindNode, StaticDomainRef};
use phalcom_type_meta::type_node::{StaticArgumentRef, TypeNode};
use phalcom_type_meta::validate::{ValidationLimits, validate_metadata_bundle};
use std::sync::Arc;

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
fn metadata_exports_and_validates_static_domain_and_type_data_identity() {
    let mut store = TypeStore::new();
    let owner = dummy_decl("Shape");
    let domain = store.nominal_static_domain(owner.clone());
    let const_int = store.const_int(3);
    let const_int_kind = store.kind_of(const_int);
    let constructor = store.intern_static_constructor(domain, owner, "Point", vec![const_int_kind].into_boxed_slice());
    let static_data = store
        .static_data(domain, constructor, vec![StaticArgument::Term(const_int)].into_boxed_slice())
        .expect("registered static constructor should accept matching static data");
    let shape_kind = store.static_domain_kind(domain);
    let tensor_kind = store.arrow_kind(vec![KindId::TYPE, shape_kind].into_boxed_slice(), KindId::TYPE);
    let tensor = store.nominal_form(dummy_decl("Tensor"), tensor_kind);
    let float = store.nominal_type(dummy_decl("Float"));
    let tensor_value = store
        .apply_static_form(tensor, &[StaticArgument::Term(float), StaticArgument::Term(static_data)])
        .expect("heterogeneous Tensor application");

    let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let static_data_id = exporter.export_type_form(static_data).unwrap();
    let const_int_id = exporter.export_type_form(const_int).unwrap();
    let tensor_value_id = exporter.export_type_form(tensor_value).unwrap();
    let bundle = exporter.build_bundle(&[]).unwrap();

    assert!(bundle.header.features.static_domains);
    assert!(
        bundle
            .kinds
            .iter()
            .any(|entry| matches!(entry.node, KindNode::StaticDomain(StaticDomainRef::Nominal { .. })))
    );
    assert!(
        bundle
            .kinds
            .iter()
            .any(|entry| matches!(entry.node, KindNode::StaticDomain(StaticDomainRef::ConstInt)))
    );
    assert!(matches!(bundle.types[const_int_id.0 as usize].form, TypeNode::ConstInt(3)));
    assert!(matches!(bundle.types[static_data_id.0 as usize].form, TypeNode::StaticData(_)));
    assert!(
        matches!(&bundle.types[tensor_value_id.0 as usize].form, TypeNode::Applied { arguments, .. } if matches!(arguments.as_ref(), [StaticArgumentRef::Term(_), StaticArgumentRef::Term(_)]))
    );
    validate_metadata_bundle(&bundle, &ValidationLimits::default()).unwrap();
}

#[test]
fn metadata_projection_retains_selected_source_type_member_identity() {
    let mut store = TypeStore::new();
    let owner = dummy_decl("Geometry");
    let subject = store.nominal_type(owner.clone());
    let member =
        phalcom_semantic::types::type_member::TypeMemberDeclarationId::new(owner, phalcom_semantic::types::type_member::TypeMemberSelector::named("Transform"));
    let projection = store.type_member_projection(
        phalcom_semantic::types::type_member::TypeLevelProjection {
            subject: phalcom_semantic::types::type_member::ProjectionSubject::Type(subject),
            selector: member.selector.clone(),
            arguments: Box::new([]),
            source_declaration: Some(member),
        },
        KindId::TYPE,
    );
    let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let projection_id = exporter.export_type_form(projection).unwrap();
    let bundle = exporter.build_bundle(&[]).unwrap();
    let TypeNode::TypeMemberProjection(projected) = &bundle.types[projection_id.0 as usize].form else {
        panic!("expected source type-member projection metadata");
    };
    let stable_member = projected.source_declaration.as_ref().expect("stable source member identity");
    assert_eq!(stable_member.selector.as_ref(), "Transform");
    assert_eq!(stable_member.owner.path.last().map(AsRef::as_ref), Some("Geometry"));
    validate_metadata_bundle(&bundle, &ValidationLimits::default()).unwrap();
}

#[test]
fn metadata_exports_associated_static_domain_identity_and_owner_environment() {
    use phalcom_type_meta::encode::{decode_metadata_json, encode_metadata_json};

    use phalcom_semantic::identity::AssociatedStaticDomainDeclarationId;

    let mut store = TypeStore::new();
    let owner = dummy_decl("Packet");
    store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(owner.clone()), 0, "T", KindId::TYPE));
    let declaration = AssociatedStaticDomainDeclarationId::new(owner.clone(), "State");
    let int = store.nominal_type(dummy_decl("Int"));
    let term_domain = store.associated_static_domain(declaration.clone(), vec![StaticArgument::Term(int)].into_boxed_slice());
    let term_constructor = store.intern_static_constructor(term_domain, owner.clone(), "Unread", Box::new([]));
    let term = store.static_data(term_domain, term_constructor, Box::new([])).expect("associated singleton");

    let row_owner = dummy_decl("RowPacket");
    store.intern_type_parameter(TypeParameterData::new(
        TypeParameterOwner::Declaration(row_owner.clone()),
        0,
        "R",
        KindId::RECORD_ROW,
    ));
    let row_declaration = AssociatedStaticDomainDeclarationId::new(row_owner.clone(), "State");
    let row = store
        .record_row_checked(Vec::new(), phalcom_semantic::types::row::RecordRowTail::Closed)
        .unwrap();
    let row_domain = store.associated_static_domain(row_declaration, vec![StaticArgument::RecordRow(row)].into_boxed_slice());
    let row_constructor = store.intern_static_constructor(row_domain, row_owner, "Parsed", Box::new([]));
    let row_term = store
        .static_data(row_domain, row_constructor, Box::new([]))
        .expect("row-specialized associated singleton");

    let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let term_id = exporter.export_type_form(term).unwrap();
    let row_term_id = exporter.export_type_form(row_term).unwrap();
    let bundle = exporter.build_bundle(&[]).unwrap();
    let TypeNode::StaticData(term_data) = &bundle.types[term_id.0 as usize].form else {
        panic!("expected associated static data node");
    };
    let StaticDomainRef::Associated {
        declaration,
        owner_environment,
    } = &term_data.domain
    else {
        panic!("expected associated static-domain reference");
    };
    assert_eq!(declaration.name.as_ref(), "State");
    assert!(matches!(owner_environment.as_ref(), [StaticArgumentRef::Term(_)]));
    let TypeNode::StaticData(row_data) = &bundle.types[row_term_id.0 as usize].form else {
        panic!("expected row-specialized associated static data node");
    };
    assert!(matches!(&row_data.domain, StaticDomainRef::Associated { owner_environment, .. }
        if matches!(owner_environment.as_ref(), [StaticArgumentRef::RecordRow(row)] if row.fields.is_empty() && row.tail.is_none())));
    validate_metadata_bundle(&bundle, &ValidationLimits::default()).unwrap();

    let json = encode_metadata_json(&bundle).unwrap();
    let decoded = decode_metadata_json(&json, &ValidationLimits::default()).unwrap();
    assert_eq!(
        decoded, bundle,
        "associated identity and row owner environment must survive metadata round-trip"
    );
}

#[test]
fn metadata_publishes_stable_algebraic_phase_and_shape() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        "data MetaProduct(_ value: Int)\n\
         data MetaSum { A; B(_ value: Int) }\n\
         type data MetaShape { Scalar; Vector(_ size: Int) }\n",
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let analysis = analyze_single_module(module, source, Arc::new(parsed.program));
    assert!(!analysis.snapshot.has_errors(), "semantic diagnostics: {:?}", analysis.snapshot.diagnostics);

    let exporter = MetadataExporter::new(
        &analysis.snapshot.store,
        Some(&analysis.snapshot.declarations),
        None,
        None,
        MetadataProfile::RuntimePublic,
    )
    .with_algebraic_semantics(&analysis.snapshot.data_semantics, &analysis.snapshot.enum_semantics);
    let bundle = exporter.build_bundle(&[]).unwrap();
    let record = |name: &str| {
        bundle
            .declarations
            .iter()
            .find(|record| record.declaration.path.last().is_some_and(|component| component.as_ref() == name))
            .unwrap_or_else(|| panic!("missing metadata declaration {name}"))
    };

    assert_eq!(
        record("MetaProduct").algebraic,
        Some(phalcom_type_meta::declaration::AlgebraicDeclarationRecord {
            phase: AlgebraicPhaseRef::Runtime,
            shape: AlgebraicShapeRef::Product,
        })
    );
    assert_eq!(
        record("MetaSum").algebraic,
        Some(phalcom_type_meta::declaration::AlgebraicDeclarationRecord {
            phase: AlgebraicPhaseRef::Runtime,
            shape: AlgebraicShapeRef::Sum,
        })
    );
    assert_eq!(
        record("MetaShape").algebraic,
        Some(phalcom_type_meta::declaration::AlgebraicDeclarationRecord {
            phase: AlgebraicPhaseRef::TypeOnly,
            shape: AlgebraicShapeRef::Sum,
        })
    );
    validate_metadata_bundle(&bundle, &ValidationLimits::default()).unwrap();
}

#[test]
fn snapshot_metadata_round_trip_preserves_static_alias_and_associated_term_identity() {
    use phalcom_type_meta::encode::{decode_metadata_json, encode_metadata_json};
    use phalcom_type_meta::identity::StableStaticDomainDeclarationRef;

    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        "type data Shape { Scalar; Empty() }\n\
         type data ShapeSingleton { Empty }\n\
         type Current: Shape = Shape::Scalar\n\
         type EmptyShape: Shape = Shape::Empty()\n\
         type SingletonShape: ShapeSingleton = ShapeSingleton::Empty\n\
         class Container<T: Type> { type data State { Ready; Done } }\n\
         type CurrentState: Container<Int>::State = Container<Int>::State::Ready\n",
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let analysis = analyze_single_module(module, source, Arc::new(parsed.program));
    assert!(!analysis.snapshot.has_errors(), "semantic diagnostics: {:?}", analysis.snapshot.diagnostics);

    let bundle = MetadataExporter::new(
        &analysis.snapshot.store,
        Some(&analysis.snapshot.declarations),
        None,
        None,
        MetadataProfile::RuntimePublic,
    )
    .with_algebraic_semantics(&analysis.snapshot.data_semantics, &analysis.snapshot.enum_semantics)
    .with_associated_static_domains(analysis.snapshot.associated_static_domains.as_ref())
    .with_aliases(&analysis.snapshot.type_aliases)
    .build_bundle(&[])
    .expect("snapshot-backed static metadata");

    let shape_catalog = bundle
        .static_domains
        .iter()
        .find(|record| {
            matches!(&record.declaration,
            StableStaticDomainDeclarationRef::Nominal(declaration)
                if declaration.path.last().is_some_and(|name| name.as_ref() == "Shape"))
        })
        .expect("Shape catalog");
    assert!(shape_catalog.cases.iter().any(|case| case.name.as_ref() == "Empty()"));
    assert!(!shape_catalog.cases.iter().any(|case| case.name.as_ref() == "Empty"));
    let singleton_catalog = bundle
        .static_domains
        .iter()
        .find(|record| {
            matches!(&record.declaration,
            StableStaticDomainDeclarationRef::Nominal(declaration)
                if declaration.path.last().is_some_and(|name| name.as_ref() == "ShapeSingleton"))
        })
        .expect("singleton Shape catalog");
    assert!(singleton_catalog.cases.iter().any(|case| case.name.as_ref() == "Empty"));
    assert_ne!(
        singleton_catalog.cases[0].name,
        shape_catalog.cases.iter().find(|case| case.name.as_ref() == "Empty()").unwrap().name,
        "singleton Empty and explicit-empty Empty() need distinct stable selectors",
    );

    let alias_target = |name: &str| {
        let alias = bundle
            .aliases
            .iter()
            .find(|alias| alias.declaration.path.last().is_some_and(|component| component.as_ref() == name))
            .unwrap_or_else(|| panic!("missing alias metadata for {name}"));
        &bundle.types[alias.target.0 as usize].form
    };
    assert!(matches!(alias_target("Current"), TypeNode::StaticData(data)
        if matches!(&data.domain, StaticDomainRef::Nominal { declaration } if declaration.path.last().is_some_and(|part| part.as_ref() == "Shape"))
            && data.constructor.case_name.as_ref() == "Scalar"));
    assert!(matches!(alias_target("EmptyShape"), TypeNode::StaticData(data)
        if data.constructor.case_name.as_ref() == "Empty()"));
    assert!(matches!(alias_target("CurrentState"), TypeNode::StaticData(data)
        if matches!(&data.domain, StaticDomainRef::Associated { declaration, owner_environment }
            if declaration.name.as_ref() == "State"
                && declaration.owner.path.last().is_some_and(|part| part.as_ref() == "Container")
                && owner_environment.len() == 1)
            && data.constructor.case_name.as_ref() == "Ready"));

    validate_metadata_bundle(&bundle, &ValidationLimits::default()).expect("snapshot metadata validates");
    let json = encode_metadata_json(&bundle).expect("encode snapshot metadata");
    let decoded = decode_metadata_json(&json, &ValidationLimits::default()).expect("decode snapshot metadata");
    assert_eq!(decoded, bundle, "static alias and associated identities round-trip");
}

#[test]
fn metadata_round_trip_preserves_heterogeneous_application_and_scoped_static_domain_bound() {
    use phalcom_semantic::types::type_lambda::ScopedTypeData;
    use phalcom_type_meta::encode::{decode_metadata_json, encode_metadata_json};
    use phalcom_type_meta::scoped_type::ScopedTypeNode;
    use phalcom_type_meta::type_node::StaticArgumentRef;

    let mut store = TypeStore::new();
    let shape_owner = dummy_decl("Shape");
    let shape_domain = store.nominal_static_domain(shape_owner.clone());
    let shape_kind = store.static_domain_kind(shape_domain);
    store.intern_static_constructor(shape_domain, shape_owner, "Scalar", Box::new([]));

    let int = store.nominal_type(dummy_decl("Int"));
    let row = store
        .record_row_checked(vec![RecordRowField::symbol_required("value", int)], RecordRowTail::Closed)
        .expect("closed row argument");
    let tensor_kind = store.arrow_kind(vec![KindId::TYPE, KindId::RECORD_ROW].into_boxed_slice(), KindId::TYPE);
    let tensor = store.nominal_form(dummy_decl("Tensor"), tensor_kind);
    let applied = store
        .apply_static_form(tensor, &[StaticArgument::Term(int), StaticArgument::RecordRow(row)])
        .expect("heterogeneous Tensor application");

    let bound = store.arena_mut().intern_scoped(ScopedTypeData::Bound { depth: 0, index: 0 });
    let lambda = store.lambda(vec![shape_kind].into_boxed_slice(), bound, shape_kind);

    let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let applied_id = exporter.export_type_form(applied).expect("heterogeneous application export");
    let lambda_id = exporter.export_type_form(lambda).expect("static-domain lambda export");
    let bundle = exporter.build_bundle(&[]).expect("canonical metadata bundle");

    assert!(matches!(&bundle.types[applied_id.0 as usize].form, TypeNode::Applied { arguments, .. }
        if matches!(arguments.as_ref(), [StaticArgumentRef::Term(_), StaticArgumentRef::RecordRow(row)]
            if row.fields.len() == 1 && row.fields[0].name.as_ref() == "value")));
    let TypeNode::TypeLambda(lambda) = &bundle.types[lambda_id.0 as usize].form else {
        panic!("expected type lambda metadata");
    };
    assert!(matches!(
        bundle.scoped_types[lambda.body.0 as usize].form,
        ScopedTypeNode::Bound { depth: 0, index: 0 }
    ));
    assert!(matches!(
        bundle.kinds[lambda.parameter_kinds[0].0 as usize].node,
        KindNode::StaticDomain(StaticDomainRef::Nominal { .. })
    ));

    validate_metadata_bundle(&bundle, &ValidationLimits::default()).expect("heterogeneous scoped metadata validates");
    let json = encode_metadata_json(&bundle).expect("encode heterogeneous scoped metadata");
    let decoded = decode_metadata_json(&json, &ValidationLimits::default()).expect("decode heterogeneous scoped metadata");
    assert_eq!(decoded, bundle, "RecordRow arguments and scoped static-domain binder round-trip");
}

#[test]
fn metadata_keeps_legacy_enum_and_equivalent_data_identity_neutral() {
    fn export(source: &str) -> phalcom_type_meta::bundle::SemanticMetadataBundle {
        let module = ModuleId::universe_root();
        let source: Arc<str> = Arc::from(source);
        let parsed = phalcom_ast::parse(&source, 0);
        assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
        let analysis = analyze_single_module(module, source, Arc::new(parsed.program));
        assert!(!analysis.snapshot.has_errors(), "semantic diagnostics: {:?}", analysis.snapshot.diagnostics);
        MetadataExporter::new(
            &analysis.snapshot.store,
            Some(&analysis.snapshot.declarations),
            None,
            None,
            MetadataProfile::RuntimePublic,
        )
        .with_algebraic_semantics(&analysis.snapshot.data_semantics, &analysis.snapshot.enum_semantics)
        .build_bundle(&[])
        .unwrap()
    }

    let legacy = export("data MetaCompat { A; B(_ value: Int) }\n");
    let canonical = export("data MetaCompat { A; B(_ value: Int) }\n");
    assert_eq!(legacy.header.interface_fingerprint, canonical.header.interface_fingerprint);
    let legacy_record = legacy
        .declarations
        .iter()
        .find(|record| record.declaration.path.last().is_some_and(|component| component.as_ref() == "MetaCompat"))
        .expect("legacy metadata declaration");
    let canonical_record = canonical
        .declarations
        .iter()
        .find(|record| record.declaration.path.last().is_some_and(|component| component.as_ref() == "MetaCompat"))
        .expect("canonical metadata declaration");
    assert_eq!(legacy_record.algebraic, canonical_record.algebraic);
    validate_metadata_bundle(&canonical, &ValidationLimits::default()).unwrap();
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
        implementation: phalcom_common::modifier::ImplementationKind::Native,
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
fn cross_module_inferred_callable_result_round_trips_as_compiler_inferred_metadata() {
    use phalcom_modules::interface::{InterfaceBuilder, LinkedExport, LinkedExportTarget, LinkedModuleInterface};
    use phalcom_modules::linker::{GlobalBindingId, ImportBindingId, LinkedModule, LinkedProgram, LinkedReadSpec, ModuleBindingLayout, SymbolId};
    use phalcom_modules::source::ModuleKind;
    use phalcom_semantic::session::SemanticWorkspaceSession;
    use phalcom_semantic::source::ParsedModuleUnit;
    use phalcom_semantic::workspace::SemanticWorkspaceInput;
    use phalcom_type_meta::declaration::{PublishedTypeAuthority, PublishedTypeSlot};
    use phalcom_type_meta::encode::{decode_metadata_json, encode_metadata_json};
    use phalcom_type_meta::identity::StableDispatchSide;
    use std::collections::BTreeMap;

    let api_module = ModuleId::universe(ModulePath::from_components(vec![ModuleComponent::from_identifier("metadata_api").unwrap()]));
    let client_module = ModuleId::universe(ModulePath::from_components(vec![ModuleComponent::from_identifier("metadata_client").unwrap()]));
    let api_source = "class Api { @class value() { \"result\" } }\nexport Api\n";
    let client_source = "import metadata_api.Api\nclass Client { @class value() { Api.value() } }\nexport Client\n";
    let parse_unit = |module: &ModuleId, source: &str| {
        let parsed = phalcom_ast::parse(source, 0);
        assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
        Arc::new(ParsedModuleUnit::new(
            module.clone(),
            ModuleKind::Module,
            None,
            Arc::from(source),
            Arc::new(parsed.program),
        ))
    };
    let api_unit = parse_unit(&api_module, api_source);
    let client_unit = parse_unit(&client_module, client_source);
    let export_api = LinkedExport {
        public_name: "Api".into(),
        target: LinkedExportTarget::Binding(SymbolId {
            module: api_module.clone(),
            name: "Api".into(),
        }),
        range: SourceRange::default(),
    };
    let api_program = api_unit.program.clone();
    let client_program = client_unit.program.clone();
    let api_globals = InterfaceBuilder::build(api_module.clone(), ModuleKind::Module, &api_program)
        .unwrap()
        .declarations
        .keys()
        .enumerate()
        .map(|(index, name)| (name.clone().into_boxed_str(), GlobalBindingId(index as u32)))
        .collect();
    let client_globals = InterfaceBuilder::build(client_module.clone(), ModuleKind::Module, &client_program)
        .unwrap()
        .declarations
        .keys()
        .enumerate()
        .map(|(index, name)| (name.clone().into_boxed_str(), GlobalBindingId(index as u32)))
        .collect();
    let linked_modules = BTreeMap::from([
        (
            api_module.clone(),
            LinkedModule {
                interface: LinkedModuleInterface {
                    module: api_module.clone(),
                    kind: ModuleKind::Module,
                    exports: BTreeMap::from([("Api".into(), export_api)]),
                    metadata: phalcom_modules::metadata::ModuleMetadata::default(),
                },
                bindings: ModuleBindingLayout {
                    local_globals: api_globals,
                    ..ModuleBindingLayout::default()
                },
                linked_reads: Vec::new(),
                runtime_dependencies: Vec::new(),
            },
        ),
        (
            client_module.clone(),
            LinkedModule {
                interface: LinkedModuleInterface {
                    module: client_module.clone(),
                    kind: ModuleKind::Module,
                    exports: BTreeMap::new(),
                    metadata: phalcom_modules::metadata::ModuleMetadata::default(),
                },
                bindings: ModuleBindingLayout {
                    local_globals: client_globals,
                    imports: BTreeMap::from([("Api".into(), ImportBindingId(0))]),
                },
                linked_reads: vec![LinkedReadSpec::Binding(SymbolId {
                    module: api_module.clone(),
                    name: "Api".into(),
                })],
                runtime_dependencies: vec![api_module.clone()],
            },
        ),
    ]);
    let input = SemanticWorkspaceInput::new(
        Arc::new(LinkedProgram {
            universe: Arc::new(ProjectUniverse::new()),
            modules: linked_modules,
            graphs: phalcom_modules::graph::ModuleGraphs::default(),
            entry: client_module.clone(),
            initialization_order: vec![api_module.clone(), client_module.clone()],
        }),
        BTreeMap::from([(api_module, api_unit), (client_module, client_unit)]),
        1,
    );
    let mut session = SemanticWorkspaceSession::new();
    let analysis = session.update(input);
    assert!(
        !analysis.snapshot.has_errors(),
        "cross-module semantic diagnostics: {:?}",
        analysis.snapshot.diagnostics
    );

    let bundle = MetadataExporter::new(
        &analysis.snapshot.store,
        Some(&analysis.snapshot.declarations),
        Some(&analysis.snapshot.callable_signatures),
        Some(&analysis.snapshot.field_signatures),
        MetadataProfile::RuntimePublic,
    )
    .build_bundle(&[])
    .expect("cross-module inferred callable metadata export");
    let record = bundle
        .callables
        .iter()
        .find(|record| {
            record.callable.owner.path.last().is_some_and(|name| name.as_ref() == "Api")
                && record.callable.selector.as_ref() == "value()"
                && record.callable.side == StableDispatchSide::Class
        })
        .expect("Api.value metadata record");
    let (form_id, authority) = match &record.return_type {
        PublishedTypeSlot::Known { form, authority } => (*form, *authority),
        other => panic!("expected published inferred result, got {other:?}"),
    };
    assert_eq!(authority, PublishedTypeAuthority::CompilerInferred);
    assert!(matches!(
        bundle.types[form_id.0 as usize].form,
        TypeNode::Nominal { ref declaration }
            if declaration.path.last().is_some_and(|name| name.as_ref() == "String")
    ));
    validate_metadata_bundle(&bundle, &ValidationLimits::default()).expect("cross-module inferred callable metadata validates");
    let json = encode_metadata_json(&bundle).expect("encode inferred callable metadata");
    let decoded = decode_metadata_json(&json, &ValidationLimits::default()).expect("decode inferred callable metadata");
    assert_eq!(decoded, bundle, "CompilerInferred result survives metadata round-trip");
}

#[test]
fn open_record_metadata_is_tail_sensitive_and_enables_row_feature() {
    let mut store = TypeStore::new();
    let int = store.nominal(dummy_decl("Int"));
    let owner_one = CallableId::new(dummy_decl("Owner"), Selector::getter("row-one").unwrap(), DispatchSide::Instance);
    let owner_two = CallableId::new(dummy_decl("Owner"), Selector::getter("row-two").unwrap(), DispatchSide::Instance);
    let row_one = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Callable(owner_one), 0, "R", KindId::RECORD_ROW));
    let row_two = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Callable(owner_two), 0, "R", KindId::RECORD_ROW));
    let fields = vec![RecordRowField::symbol_required("value", int)];
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

    let list_kind1 = store1.arrow_kind(Box::new([KindId::TYPE]), KindId::TYPE);
    let list1 = store1.nominal_form(dummy_decl("List"), list_kind1);
    let bound1 = store1
        .arena_mut()
        .intern_scoped(phalcom_semantic::types::ScopedTypeData::Bound { depth: 0, index: 0 });
    let free_list1 = store1.arena_mut().intern_scoped(phalcom_semantic::types::ScopedTypeData::Free(list1));
    let applied1 = store1.arena_mut().intern_scoped(phalcom_semantic::types::ScopedTypeData::Applied {
        origin: free_list1,
        arguments: Box::new([bound1]),
    });
    let lam1 = store1.lambda(Box::new([KindId::TYPE]), applied1, KindId::TYPE);

    let list_kind2 = store2.arrow_kind(Box::new([KindId::TYPE]), KindId::TYPE);
    let list2 = store2.nominal_form(dummy_decl("List"), list_kind2);
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
        binding: phalcom_semantic::declarations::DeclarationTypeBinding::ordinary(),
        generic_binder_header: None,
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
fn unit_and_empty_record_alias_metadata_round_trip_as_structural_zero_forms() {
    use phalcom_type_meta::encode::{decode_metadata_json, encode_metadata_json};

    let unit_declaration = phalcom_semantic::PreludeTypeMap::canonical_universe()
        .get("Unit")
        .cloned()
        .expect("Unit keeps its source-owned prelude declaration");
    let record_module = ModuleId::universe(ModulePath::from_components(vec![
        ModuleComponent::from_identifier("collections").unwrap(),
        ModuleComponent::from_identifier("record").unwrap(),
    ]));
    let empty_record_declaration = DeclarationId::new(record_module.clone(), "EmptyRecord".into());

    let mut store = TypeStore::new();
    let unit_form = store.unit();
    assert_eq!(store.tuple(Box::new([])), unit_form, "the empty Tuple specialization is canonical Unit");
    let empty_record_form = store.record(Box::new([]));
    assert!(matches!(store.get(unit_form), phalcom_semantic::types::store::TypeData::Unit));
    assert!(matches!(store.get(empty_record_form), phalcom_semantic::types::store::TypeData::Record(row) if store.record_row(*row).fields.is_empty()));

    let mut aliases = TypeAliasTable::new();
    for (declaration, form) in [(unit_declaration.clone(), unit_form), (empty_record_declaration.clone(), empty_record_form)] {
        aliases.insert(TypeAliasInfo {
            declaration: declaration.clone(),
            kind: KindId::TYPE,
            kind_shape: "Type".into(),
            generic_signature: None,
            form,
            structural_form: if declaration.name.as_ref() == "Unit" { "Tuple<>" } else { "Record<>" }.into(),
            dependencies: Box::new([]),
            source: SemanticSourceSpan::new(declaration.module.clone(), SourceRange { start: 0, end: 1 }),
        });
    }

    let bundle = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic)
        .with_aliases(&aliases)
        .build_bundle(&[])
        .expect("zero-product aliases export as transparent alias records");
    assert!(bundle.declarations.is_empty(), "the aliases are not runtime nominal declarations or data roots");
    assert_eq!(bundle.aliases.len(), 2);

    let unit_alias = bundle
        .aliases
        .iter()
        .find(|alias| alias.declaration.path.last().is_some_and(|part| part.as_ref() == "Unit"))
        .expect("Unit alias record");
    assert_eq!(
        unit_alias.declaration,
        phalcom_semantic::metadata::stable_identity::to_stable_declaration(&unit_declaration),
        "Unit retains its exact prelude/source declaration identity"
    );
    assert!(matches!(&bundle.types[unit_alias.target.0 as usize].form, TypeNode::Unit));

    let empty_record_alias = bundle
        .aliases
        .iter()
        .find(|alias| alias.declaration.path.last().is_some_and(|part| part.as_ref() == "EmptyRecord"))
        .expect("EmptyRecord alias record");
    assert_eq!(
        empty_record_alias.declaration,
        phalcom_semantic::metadata::stable_identity::to_stable_declaration(&empty_record_declaration),
        "EmptyRecord retains its exact Record source declaration identity"
    );
    assert!(matches!(&bundle.types[empty_record_alias.target.0 as usize].form, TypeNode::Record(fields) if fields.is_empty()));
    assert!(bundle.types.iter().all(|entry| !matches!(
        &entry.form,
        TypeNode::Nominal { declaration }
            if declaration.path.last().is_some_and(|part| matches!(part.as_ref(), "Unit" | "EmptyRecord"))
    )));

    validate_metadata_bundle(&bundle, &ValidationLimits::default()).expect("transparent alias metadata validates");
    let json = encode_metadata_json(&bundle).expect("encode alias metadata");
    let imported = decode_metadata_json(&json, &ValidationLimits::default()).expect("import alias metadata");
    assert_eq!(imported, bundle, "import preserves both source alias identities and structural targets");
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
fn metadata_exports_associated_projection_with_stable_identity() {
    let mut store = TypeStore::new();
    let subject = store.nominal(dummy_decl("Box"));
    let trait_declaration = dummy_decl("Cell");
    let projection = store.associated_projection(
        subject,
        TraitRef::new(trait_declaration.clone(), Vec::<phalcom_semantic::types::id::TypeId>::new().into_boxed_slice()),
        AssociatedTypeRequirementId::new(trait_declaration, 0),
    );
    let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let node = exporter.export_type_form(projection).expect("associated projection is exportable");
    let bundle = exporter.build_bundle(&[]).expect("associated projection metadata bundle");
    let entry = &bundle.types[node.0 as usize];
    let phalcom_type_meta::type_node::TypeNode::AssociatedProjection {
        subject,
        trait_application,
        requirement,
    } = &entry.form
    else {
        panic!("expected associated projection metadata node, got {:?}", entry.form);
    };
    assert!(matches!(
        &bundle.types[subject.0 as usize].form,
        phalcom_type_meta::type_node::TypeNode::Nominal { declaration } if declaration.path.last().map(|value| value.as_ref()) == Some("Box")
    ));
    assert_eq!(trait_application.declaration.path.last().map(|value| value.as_ref()), Some("Cell"));
    assert!(trait_application.arguments.is_empty());
    assert_eq!(requirement.owner, trait_application.declaration);
    assert_eq!(requirement.index, 0);
    validate_metadata_bundle(&bundle, &ValidationLimits::default()).expect("associated projection bundle validates");

    let first_fingerprint = entry.structural_fingerprint;
    let mut second_exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let second_node = second_exporter.export_type_form(projection).expect("associated projection remains exportable");
    let second_bundle = second_exporter.build_bundle(&[]).expect("second associated projection metadata bundle");
    assert_eq!(first_fingerprint, second_bundle.types[second_node.0 as usize].structural_fingerprint);
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

#[test]
fn metadata_exports_complete_and_concat_pack_terms_with_binder_modes() {
    let mut store = TypeStore::new();
    let int = store.nominal(dummy_decl("Int"));
    let str_ty = store.nominal(dummy_decl("String"));
    let positionals = store.type_pack(Box::new([int])).unwrap();
    let labeled = store
        .labeled_type_pack(Box::new([phalcom_semantic::types::store::LabeledTypePackElement {
            label: "name".into(),
            element: str_ty,
        }]))
        .unwrap();
    let complete = store.complete_type_pack(positionals, labeled).unwrap();
    let owner = TypeParameterOwner::Declaration(dummy_decl("Variadic"));
    let parameter =
        store.intern_type_parameter(TypeParameterData::new(owner.clone(), 0, "P", KindId::TYPE_PACK).with_mode(GenericParameterMode::PositionalPack));
    let symbolic = store.parameter_form(parameter);
    let concat = store.type_pack_concat(&[positionals, symbolic]).unwrap();
    let signature = GenericSignature::new(owner, Box::new([parameter])).with_parameter_modes(Box::new([GenericParameterMode::PositionalPack]));

    let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let complete_id = exporter.export_type_form(complete).unwrap();
    let concat_id = exporter.export_type_form(concat).unwrap();
    exporter.export_generic_signature(&signature).unwrap();
    let bundle = exporter.build_bundle(&[]).unwrap();

    assert_eq!(bundle.header.schema_version, phalcom_type_meta::header::TYPE_METADATA_SCHEMA_VERSION);
    assert!(matches!(
        bundle.types[complete_id.0 as usize].form,
        phalcom_type_meta::type_node::TypeNode::CompleteTypePack { .. }
    ));
    assert!(matches!(
        bundle.types[concat_id.0 as usize].form,
        phalcom_type_meta::type_node::TypeNode::TypePackConcat(_)
    ));
    assert_eq!(bundle.parameters[0].mode, phalcom_type_meta::generic::GenericParameterModeRef::PositionalPack);
    validate_metadata_bundle(&bundle, &ValidationLimits::default()).unwrap();
}

#[test]
fn binder_mode_changes_metadata_interface_fingerprint() {
    fn bundle_for_mode(mode: GenericParameterMode) -> phalcom_type_meta::bundle::SemanticMetadataBundle {
        let mut store = TypeStore::new();
        let owner = TypeParameterOwner::Declaration(dummy_decl("VariadicApi"));
        let parameter = store.intern_type_parameter(TypeParameterData::new(owner.clone(), 0, "P", KindId::TYPE_PACK).with_mode(mode));
        let signature = GenericSignature::new(owner, Box::new([parameter])).with_parameter_modes(Box::new([mode]));
        let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
        exporter.export_generic_signature(&signature).unwrap();
        exporter.build_bundle(&[]).unwrap()
    }

    let single = bundle_for_mode(GenericParameterMode::Single);
    let positional = bundle_for_mode(GenericParameterMode::PositionalPack);
    assert_ne!(single.header.interface_fingerprint, positional.header.interface_fingerprint);
}

#[test]
fn parameter_mode_and_kind_metadata_round_trip_independently() {
    use phalcom_type_meta::encode::{decode_metadata_json, encode_metadata_json};
    use phalcom_type_meta::generic::GenericParameterModeRef;

    fn exported(kind: KindId, mode: GenericParameterMode) -> phalcom_type_meta::bundle::SemanticMetadataBundle {
        let mut store = TypeStore::new();
        let owner = TypeParameterOwner::Declaration(dummy_decl("ModeKindMatrix"));
        let parameter = store.intern_type_parameter(TypeParameterData::new(owner.clone(), 0, "P", kind).with_mode(mode));
        let signature = GenericSignature::new(owner, Box::new([parameter])).with_parameter_modes(Box::new([mode]));
        let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
        exporter.export_generic_signature(&signature).unwrap();
        let bundle = exporter.build_bundle(&[]).unwrap();
        validate_metadata_bundle(&bundle, &ValidationLimits::default()).unwrap();

        let encoded = encode_metadata_json(&bundle).unwrap();
        let decoded = decode_metadata_json(&encoded, &ValidationLimits::default()).unwrap();
        assert_eq!(decoded.parameters[0].kind, bundle.parameters[0].kind);
        assert_eq!(decoded.parameters[0].mode, bundle.parameters[0].mode);
        decoded
    }

    let labeled_pack = exported(KindId::LABELED_TYPE_PACK, GenericParameterMode::LabeledPack);
    let labeled_row = exported(KindId::RECORD_ROW, GenericParameterMode::LabeledPack);
    let single_row = exported(KindId::RECORD_ROW, GenericParameterMode::Single);

    for (bundle, expected_kind, expected_mode) in [
        (
            &labeled_pack,
            phalcom_type_meta::kind::KindNode::LabeledTypePack,
            GenericParameterModeRef::LabeledPack,
        ),
        (&labeled_row, phalcom_type_meta::kind::KindNode::RecordRow, GenericParameterModeRef::LabeledPack),
        (&single_row, phalcom_type_meta::kind::KindNode::RecordRow, GenericParameterModeRef::Single),
    ] {
        assert_eq!(bundle.kinds[bundle.parameters[0].kind.0 as usize].node, expected_kind);
        assert_eq!(bundle.parameters[0].mode, expected_mode);
    }

    assert_ne!(
        labeled_row.header.interface_fingerprint, single_row.header.interface_fingerprint,
        "same RecordRow kind with different expansion modes must remain distinct in the interface fingerprint"
    );
    assert_ne!(
        labeled_pack.header.interface_fingerprint, labeled_row.header.interface_fingerprint,
        "same LabeledPack mode with different semantic kinds must remain distinct in the interface fingerprint"
    );
}

#[test]
fn scoped_record_row_lambda_keeps_kind_and_mode_separate_through_fingerprinting() {
    use phalcom_semantic::types::type_lambda::ScopedTypeData;
    use phalcom_type_meta::encode::{decode_metadata_json, encode_metadata_json};
    use phalcom_type_meta::generic::GenericParameterModeRef;
    use phalcom_type_meta::scoped_type::ScopedTypeNode;

    fn exported(mode: GenericParameterMode) -> phalcom_type_meta::bundle::SemanticMetadataBundle {
        let mut store = TypeStore::new();
        let body = store.arena_mut().intern_scoped(ScopedTypeData::Bound { depth: 0, index: 0 });
        let lambda = store
            .arena_mut()
            .intern_lambda_with_modes(Box::new([KindId::RECORD_ROW]), Box::new([mode]), body, KindId::RECORD_ROW, None);
        let kind = store.arrow_kind(Box::new([KindId::RECORD_ROW]), KindId::RECORD_ROW);
        let ty = store.intern_with_kind(phalcom_semantic::types::store::TypeData::Lambda(lambda), kind);
        let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
        exporter.export_type_form(ty).unwrap();
        let bundle = exporter.build_bundle(&[]).unwrap();
        validate_metadata_bundle(&bundle, &ValidationLimits::default()).unwrap();

        let encoded = encode_metadata_json(&bundle).unwrap();
        decode_metadata_json(&encoded, &ValidationLimits::default()).unwrap()
    }

    let labeled = exported(GenericParameterMode::LabeledPack);
    let single = exported(GenericParameterMode::Single);
    for (bundle, expected_mode) in [(&labeled, GenericParameterModeRef::LabeledPack), (&single, GenericParameterModeRef::Single)] {
        let phalcom_type_meta::type_node::TypeNode::TypeLambda(lambda) = &bundle.types[0].form else {
            panic!("expected exported type lambda")
        };
        assert_eq!(lambda.parameter_modes, Some(vec![expected_mode].into_boxed_slice()));
        assert_eq!(
            bundle.kinds[lambda.parameter_kinds[0].0 as usize].node,
            phalcom_type_meta::kind::KindNode::RecordRow
        );
        assert!(
            bundle
                .scoped_types
                .iter()
                .any(|entry| matches!(entry.form, ScopedTypeNode::Bound { depth: 0, index: 0 }))
        );
    }
    assert_ne!(
        labeled.types[0].structural_fingerprint, single.types[0].structural_fingerprint,
        "scoped lambda fingerprints must include mode independently of the shared RecordRow kind"
    );
}

#[test]
fn p3_metadata_exports_scoped_pack_aliases_under_schema_v4() {
    use phalcom_semantic::types::type_lambda::ScopedTypeData;
    use phalcom_type_meta::encode::{decode_metadata_json, encode_metadata_json};

    let mut store = TypeStore::new();

    // 1. type AddHead<*Ts> = (Int, *Ts)
    let int_ty = store.nominal(dummy_decl("Int"));
    let bound_ts = store.arena_mut().intern_scoped(ScopedTypeData::Bound { depth: 0, index: 0 });
    let free_int = store.arena_mut().intern_scoped(ScopedTypeData::Free(int_ty));
    let int_pack = store.arena_mut().intern_scoped(ScopedTypeData::TypePack(vec![free_int].into_boxed_slice()));
    let concat_p = store
        .arena_mut()
        .intern_scoped(ScopedTypeData::TypePackConcat(vec![int_pack, bound_ts].into_boxed_slice()));
    let empty_lab = store.arena_mut().intern_scoped(ScopedTypeData::LabeledTypePack(Box::new([])));
    let tuple_body = store.arena_mut().intern_scoped(ScopedTypeData::TupleFromPacks {
        positionals: concat_p,
        labeled: empty_lab,
    });

    let lambda_id = store.arena_mut().intern_lambda_with_modes(
        vec![KindId::TYPE_PACK].into_boxed_slice(),
        vec![GenericParameterMode::PositionalPack].into_boxed_slice(),
        tuple_body,
        KindId::TYPE,
        None,
    );
    let lambda_ty = store.intern_with_kind(phalcom_semantic::types::store::TypeData::Lambda(lambda_id), KindId::TYPE);

    let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let exported_id = exporter.export_type_form(lambda_ty).unwrap();
    let bundle = exporter.build_bundle(&[]).unwrap();

    assert_eq!(bundle.header.schema_version, phalcom_type_meta::header::TYPE_METADATA_SCHEMA_VERSION);
    validate_metadata_bundle(&bundle, &ValidationLimits::default()).unwrap();

    // JSON round trip
    let json = encode_metadata_json(&bundle).unwrap();
    let decoded = decode_metadata_json(&json, &ValidationLimits::default()).unwrap();
    assert_eq!(decoded.header.schema_version, phalcom_type_meta::header::TYPE_METADATA_SCHEMA_VERSION);
    assert_eq!(decoded.header.interface_fingerprint, bundle.header.interface_fingerprint);

    let exported_node = &decoded.types[exported_id.0 as usize].form;
    assert!(matches!(
        exported_node,
        phalcom_type_meta::type_node::TypeNode::TypeLambda(lam)
            if lam.parameter_modes == Some(Box::new([phalcom_type_meta::generic::GenericParameterModeRef::PositionalPack]))
    ));
}

#[test]
fn p3_lambda_mode_only_changes_structural_and_interface_fingerprints() {
    use phalcom_semantic::types::type_lambda::ScopedTypeData;

    let mut store = TypeStore::new();
    let bound_0 = store.arena_mut().intern_scoped(ScopedTypeData::Bound { depth: 0, index: 0 });

    let lam_single = store.arena_mut().intern_lambda_with_modes(
        vec![KindId::TYPE_PACK].into_boxed_slice(),
        vec![GenericParameterMode::Single].into_boxed_slice(),
        bound_0,
        KindId::TYPE_PACK,
        None,
    );
    let ty_single = store.intern_with_kind(phalcom_semantic::types::store::TypeData::Lambda(lam_single), KindId::TYPE_PACK);

    let lam_pack = store.arena_mut().intern_lambda_with_modes(
        vec![KindId::TYPE_PACK].into_boxed_slice(),
        vec![GenericParameterMode::PositionalPack].into_boxed_slice(),
        bound_0,
        KindId::TYPE_PACK,
        None,
    );
    let ty_pack = store.intern_with_kind(phalcom_semantic::types::store::TypeData::Lambda(lam_pack), KindId::TYPE_PACK);

    let mut exporter1 = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let id1 = exporter1.export_type_form(ty_single).unwrap();
    let b1 = exporter1.build_bundle(&[]).unwrap();

    let mut exporter2 = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let id2 = exporter2.export_type_form(ty_pack).unwrap();
    let b2 = exporter2.build_bundle(&[]).unwrap();

    assert_ne!(b1.types[id1.0 as usize].structural_fingerprint, b2.types[id2.0 as usize].structural_fingerprint);
}

#[test]
fn schema_v10_exports_open_tuple_and_anonymous_record_with_presence_and_remainders() {
    use phalcom_semantic::types::row::{AnonymousRecordRemainder, RecordRowField, RecordRowTail};
    use phalcom_semantic::types::store::{AnonymousTupleRemainder, TupleTypeElement};
    use phalcom_type_meta::encode::{decode_metadata_json, encode_metadata_json};
    use phalcom_type_meta::type_node::{AnonymousRecordRemainderRef, AnonymousTupleRemainderRef, StructuralMemberPresenceRef, TypeNode};

    let mut store = TypeStore::new();
    let int_ty = store.nominal(dummy_decl("Int"));
    let str_ty = store.nominal(dummy_decl("String"));

    // 1. Open tuple with typed remainder: (Int, opt: String, ...Int)
    let open_tuple_ty = store.open_tuple(
        Box::new([
            TupleTypeElement { label: None, ty: int_ty },
            TupleTypeElement {
                label: Some("opt".into()),
                ty: str_ty,
            },
        ]),
        AnonymousTupleRemainder::Typed(int_ty),
        AnonymousTupleRemainder::None,
    );

    // The v10 name-only Record remainder is valid only for a Symbol key domain.
    let symbol_key_ty = store.nominal(phalcom_semantic::core_surface::CoreDeclarationIds::default().symbol);

    // 2. Anonymous record with unconstrained remainder: { a: Int, b?: String, **... }
    let row_id = store
        .record_row_checked(
            vec![RecordRowField::symbol_required("a", int_ty), RecordRowField::symbol_optional("b", str_ty)],
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type: symbol_key_ty }),
        )
        .unwrap();
    let anon_record_ty = store.intern_with_kind(
        phalcom_semantic::types::store::TypeData::Record(row_id),
        phalcom_semantic::types::id::KindId::TYPE,
    );

    let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let tuple_node_id = exporter.export_type_form(open_tuple_ty).unwrap();
    let record_node_id = exporter.export_type_form(anon_record_ty).unwrap();
    let bundle = exporter.build_bundle(&[]).unwrap();

    assert_eq!(bundle.header.schema_version, 10);
    validate_metadata_bundle(&bundle, &ValidationLimits::default()).unwrap();

    let json = encode_metadata_json(&bundle).unwrap();
    let decoded = decode_metadata_json(&json, &ValidationLimits::default()).unwrap();

    let decoded_tuple = &decoded.types[tuple_node_id.0 as usize].form;
    let TypeNode::OpenTuple(open_t) = decoded_tuple else {
        panic!("Expected OpenTuple, found {:?}", decoded_tuple);
    };
    assert_eq!(open_t.elements.len(), 2);
    assert_eq!(open_t.elements[0].presence, StructuralMemberPresenceRef::Required);
    assert_eq!(open_t.elements[1].presence, StructuralMemberPresenceRef::Required);
    assert_eq!(open_t.elements[1].label.as_deref(), Some("opt"));
    assert!(matches!(open_t.positional_remainder, AnonymousTupleRemainderRef::Typed(_)));
    assert_eq!(open_t.labeled_remainder, AnonymousTupleRemainderRef::None);

    let decoded_record = &decoded.types[record_node_id.0 as usize].form;
    let TypeNode::AnonymousRecord(anon_r) = decoded_record else {
        panic!("Expected AnonymousRecord, found {:?}", decoded_record);
    };
    assert_eq!(anon_r.fields.len(), 2);
    assert_eq!(anon_r.fields[0].presence, StructuralMemberPresenceRef::Required);
    assert_eq!(anon_r.fields[1].presence, StructuralMemberPresenceRef::Optional);
    assert_eq!(anon_r.remainder, AnonymousRecordRemainderRef::Unconstrained);
}

#[test]
fn legacy_record_metadata_fails_closed_for_non_symbol_keys_and_domains() {
    use num_bigint::BigInt;
    use phalcom_semantic::field_key::{FieldKey, FieldKeyKind};
    use phalcom_semantic::metadata::MetadataExportError;
    use phalcom_semantic::scalar::StaticInt;
    use phalcom_semantic::types::row::{AnonymousRecordRemainder, RecordRowField, RecordRowTail};
    use phalcom_semantic::types::store::TypeData;

    let mut store = TypeStore::new();
    let value_type = store.nominal(dummy_decl("Int"));
    let non_symbol_keys = [FieldKey::string("text"), FieldKey::int(StaticInt::new(BigInt::from(42))), FieldKey::bool(true)];

    for key in non_symbol_keys {
        let row = store
            .record_row_checked(vec![RecordRowField::required(key, value_type)], RecordRowTail::Closed)
            .expect("unique FieldKey row");
        let record = store.intern_with_kind(TypeData::Record(row), KindId::TYPE);
        let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
        assert!(matches!(
            exporter.export_type_form(record),
            Err(MetadataExportError::NonExportableRecordKey(kind)) if kind != FieldKeyKind::Symbol
        ));
    }

    let field_key_type = store.field_key_type_from_core();
    for remainder in [
        AnonymousRecordRemainder::Unconstrained { key_type: field_key_type },
        AnonymousRecordRemainder::Typed {
            key_type: store.nominal(phalcom_semantic::core_surface::CoreDeclarationIds::default().int),
            value_type,
        },
    ] {
        let row = store
            .record_row_checked(Vec::new(), RecordRowTail::Anonymous(remainder))
            .expect("proper key/value types");
        let record = store.intern_with_kind(TypeData::Record(row), KindId::TYPE);
        let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
        assert_eq!(exporter.export_type_form(record), Err(MetadataExportError::NonExportableRecordKeyDomain));
    }

    let symbol_key_type = store.nominal(phalcom_semantic::core_surface::CoreDeclarationIds::default().symbol);
    let typed_symbol_row = store
        .record_row_checked(
            Vec::new(),
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Typed {
                key_type: symbol_key_type,
                value_type,
            }),
        )
        .expect("Symbol key and typed value domains");
    let typed_symbol_record = store.intern_with_kind(TypeData::Record(typed_symbol_row), KindId::TYPE);
    let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let record_id = exporter.export_type_form(typed_symbol_record).expect("legacy typed Symbol remainder");
    let value_id = exporter.export_type_form(value_type).expect("typed remainder value type");
    let bundle = exporter.build_bundle(&[]).expect("metadata bundle");
    let TypeNode::AnonymousRecord(record) = &bundle.types[record_id.0 as usize].form else {
        panic!("expected anonymous Record metadata")
    };
    assert_eq!(
        record.remainder,
        phalcom_type_meta::type_node::AnonymousRecordRemainderRef::Typed(value_id),
        "legacy export must preserve the typed remainder value type"
    );
}

#[test]
fn structural_member_presence_and_remainder_affect_fingerprints() {
    use phalcom_semantic::types::row::{AnonymousRecordRemainder, RecordRowField, RecordRowTail};

    let mut store = TypeStore::new();
    let int_ty = store.nominal(dummy_decl("Int"));
    let field_key_ty = store.field_key_type_from_core();

    // Record Required vs Optional
    let row_req = store
        .record_row_checked(vec![RecordRowField::symbol_required("x", int_ty)], RecordRowTail::Closed)
        .unwrap();
    let row_opt = store
        .record_row_checked(vec![RecordRowField::symbol_optional("x", int_ty)], RecordRowTail::Closed)
        .unwrap();

    let rec_req = store.intern_with_kind(
        phalcom_semantic::types::store::TypeData::Record(row_req),
        phalcom_semantic::types::id::KindId::TYPE,
    );
    let rec_opt = store.intern_with_kind(
        phalcom_semantic::types::store::TypeData::Record(row_opt),
        phalcom_semantic::types::id::KindId::TYPE,
    );

    let mut exp1 = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let id_req = exp1.export_type_form(rec_req).unwrap();
    let id_opt = exp1.export_type_form(rec_opt).unwrap();
    let b1 = exp1.build_bundle(&[]).unwrap();

    assert_ne!(
        b1.types[id_req.0 as usize].structural_fingerprint,
        b1.types[id_opt.0 as usize].structural_fingerprint
    );

    // Record Closed vs Anonymous Remainder
    let row_closed = store
        .record_row_checked(vec![RecordRowField::symbol_required("x", int_ty)], RecordRowTail::Closed)
        .unwrap();
    let row_anon = store
        .record_row_checked(
            vec![RecordRowField::symbol_required("x", int_ty)],
            RecordRowTail::Anonymous(AnonymousRecordRemainder::Unconstrained { key_type: field_key_ty }),
        )
        .unwrap();

    let rec_closed = store.intern_with_kind(
        phalcom_semantic::types::store::TypeData::Record(row_closed),
        phalcom_semantic::types::id::KindId::TYPE,
    );
    let rec_anon = store.intern_with_kind(
        phalcom_semantic::types::store::TypeData::Record(row_anon),
        phalcom_semantic::types::id::KindId::TYPE,
    );

    let mut exp2 = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let id_closed = exp2.export_type_form(rec_closed).unwrap();
    let id_anon = exp2.export_type_form(rec_anon).unwrap();
    let b2 = exp2.build_bundle(&[]).unwrap();

    assert_ne!(
        b2.types[id_closed.0 as usize].structural_fingerprint,
        b2.types[id_anon.0 as usize].structural_fingerprint
    );
}

#[test]
fn metadata_rejects_optional_tuple_presence() {
    use phalcom_semantic::types::store::TupleTypeElement;
    use phalcom_type_meta::type_node::{OpenTupleTypeRef, StructuralMemberPresenceRef, TupleElementRef, TypeNode};

    let mut store = TypeStore::new();
    let int_ty = store.nominal(dummy_decl("Int"));
    let tuple_ty = store.tuple(vec![TupleTypeElement::unlabeled(int_ty)].into_boxed_slice());

    let mut exporter = MetadataExporter::new(&store, None, None, None, MetadataProfile::RuntimePublic);
    let _ = exporter.export_type_form(tuple_ty).unwrap();
    let mut bundle = exporter.build_bundle(&[]).unwrap();

    // Mutate bundle to have Optional Tuple element in TypeNode::Tuple
    for entry in bundle.types.iter_mut() {
        if let TypeNode::Tuple(elements) = &mut entry.form {
            elements[0].presence = StructuralMemberPresenceRef::Optional;
        }
    }
    assert!(validate_metadata_bundle(&bundle, &ValidationLimits::default()).is_err());

    // Also test OpenTuple with Optional element
    let mut bundle2 = bundle.clone();
    for entry in bundle2.types.iter_mut() {
        if let TypeNode::Tuple(elements) = &mut entry.form {
            entry.form = TypeNode::OpenTuple(OpenTupleTypeRef {
                elements: Box::new([TupleElementRef {
                    label: None,
                    ty: elements[0].ty,
                    presence: StructuralMemberPresenceRef::Optional,
                }]),
                positional_remainder: phalcom_type_meta::type_node::AnonymousTupleRemainderRef::None,
                labeled_remainder: phalcom_type_meta::type_node::AnonymousTupleRemainderRef::None,
            });
        }
    }
    assert!(validate_metadata_bundle(&bundle2, &ValidationLimits::default()).is_err());
}

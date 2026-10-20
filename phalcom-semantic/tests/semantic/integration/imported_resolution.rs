use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_modules::diagnostic::{ModuleDiagnostic, ModuleDiagnosticKind};
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::{LinkedExport, LinkedExportTarget, LinkedModuleInterface};
use phalcom_modules::linker::{GlobalBindingId, ImportBindingId, LinkedModule, LinkedProgram, LinkedReadSpec, ModuleBindingLayout, SymbolId};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::source::{ModuleKind, ParsedModuleUnit};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, SemanticTargetId};
use phalcom_semantic::resolver::LinkedTypeResolver;
use phalcom_semantic::snapshot::SnapshotStatus;
use phalcom_semantic::types::annotation::TypeResolver;
use phalcom_semantic::{OccurrenceIndex, SemanticWorkspaceInput, SourceIndexContext, analyze_workspace, build_source_scope_index};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::Arc;

fn module(project: ResolvedProjectId, path: &[&str]) -> ModuleId {
    ModuleId::resolved(
        project,
        ModulePath::from_components(
            path.iter()
                .map(|component| ModuleComponent::from_identifier(component).expect("valid module component"))
                .collect::<Vec<_>>(),
        ),
    )
}

#[test]
fn imported_binding_use_resolves_to_exported_declaration_not_local_import_site() {
    let source = "from .shapes import Circle\nCircle\n";
    let parsed = phalcom_ast::parse(source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);

    let project = ResolvedProjectId::from_raw(1);
    let importer = module(project, &["main"]);
    let shapes = module(project, &["shapes"]);
    let circle = DeclarationId::new(shapes.clone(), "Circle".into());
    let context = SourceIndexContext::default()
        .with_resolved_import(importer.clone(), ".shapes", shapes.clone())
        .with_target(shapes, "Circle", SemanticTargetId::Declaration(circle.clone()));

    let mut scopes = build_source_scope_index(importer.clone(), &parsed.program, &context);
    let occurrences = OccurrenceIndex::from_program_with_context(&mut scopes, &parsed.program, Some(&context));
    let use_offset = source.rfind("Circle").expect("Circle use") + 1;
    let occurrence = occurrences.occurrence_at(use_offset).expect("Circle occurrence");

    let binding = scopes
        .bindings
        .values()
        .find(|binding| binding.name.as_ref() == "Circle")
        .expect("import binding");
    assert_eq!(occurrence.target, Some(&SemanticTargetId::Binding(binding.declaration_site.clone())));
    assert_eq!(
        scopes.import_origin(&binding.declaration_site).map(|origin| &origin.remote_target),
        Some(&SemanticTargetId::Declaration(circle.clone()))
    );
    assert!(occurrences.exact_targets().iter().any(|(site, target)| site != &binding.declaration_site
        && site.owner == phalcom_semantic::SourceOwner::Module(importer.clone())
        && target == &SemanticTargetId::Declaration(circle.clone())));
}

#[test]
fn imported_alias_keeps_local_binding_and_separate_external_origin() {
    let source = "from .shapes import Circle as C\nC\n";
    let parsed = phalcom_ast::parse(source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let project = ResolvedProjectId::from_raw(1);
    let importer = module(project, &["main"]);
    let shapes = module(project, &["shapes"]);
    let circle = DeclarationId::new(shapes.clone(), "Circle".into());
    let context = SourceIndexContext::default()
        .with_resolved_import(importer.clone(), ".shapes", shapes.clone())
        .with_target(shapes, "Circle", SemanticTargetId::Declaration(circle.clone()));
    let mut scopes = build_source_scope_index(importer, &parsed.program, &context);
    let alias = scopes.bindings.values().find(|binding| binding.name.as_ref() == "C").expect("alias binding");
    assert_eq!(alias.kind, phalcom_semantic::source_index::SourceBindingKind::Import);
    let alias_site = alias.declaration_site.clone();
    assert_eq!(scopes.target_for(&alias_site), Some(&SemanticTargetId::Binding(alias_site.clone())));
    assert_eq!(
        scopes.import_origin(&alias_site).map(|origin| &origin.remote_target),
        Some(&SemanticTargetId::Declaration(circle.clone()))
    );
    let occurrences = OccurrenceIndex::from_program(&mut scopes, &parsed.program);
    let use_offset = source.rfind('C').expect("alias use");
    assert_eq!(
        occurrences.occurrence_at(use_offset).and_then(|occurrence| occurrence.target),
        Some(&SemanticTargetId::Binding(alias_site))
    );
}

#[test]
fn imported_enum_remote_item_keeps_declaring_declaration_identity() {
    let source = "from .options import Option as ImportedOption\nImportedOption\n";
    let parsed = phalcom_ast::parse(source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let project = ResolvedProjectId::from_raw(1);
    let importer = module(project, &["consumer"]);
    let options = module(project, &["options"]);
    let option = DeclarationId::new(options.clone(), "Option".into());
    let target = SemanticTargetId::Declaration(option.clone());
    let context = SourceIndexContext::default()
        .with_resolved_import(importer.clone(), ".options", options.clone())
        .with_target(options, "Option", target.clone());
    let mut scopes = build_source_scope_index(importer.clone(), &parsed.program, &context);
    let occurrences = OccurrenceIndex::from_program_with_context(&mut scopes, &parsed.program, Some(&context));
    let remote_offset = source.find("Option").expect("remote enum name") + 1;
    assert_eq!(occurrences.occurrence_at(remote_offset).and_then(|occurrence| occurrence.target), Some(&target));
    let alias = scopes
        .bindings
        .values()
        .find(|binding| binding.name.as_ref() == "ImportedOption")
        .expect("enum alias binding");
    assert_eq!(scopes.import_origin(&alias.declaration_site).map(|origin| &origin.remote_target), Some(&target));
}

#[test]
fn module_import_read_resolves_to_canonical_module_target() {
    let source = "import .shapes as S\nS\n";
    let parsed = phalcom_ast::parse(source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let project = ResolvedProjectId::from_raw(1);
    let importer = module(project, &["main"]);
    let shapes = module(project, &["shapes"]);
    let context = SourceIndexContext::default().with_resolved_import(importer.clone(), ".shapes", shapes.clone());
    let mut scopes = build_source_scope_index(importer, &parsed.program, &context);
    let occurrences = OccurrenceIndex::from_program(&mut scopes, &parsed.program);
    let use_offset = source.rfind('S').expect("module alias use");
    let binding = scopes
        .bindings
        .values()
        .find(|binding| binding.name.as_ref() == "S")
        .expect("module alias binding");
    assert_eq!(
        occurrences.occurrence_at(use_offset).and_then(|occurrence| occurrence.target),
        Some(&SemanticTargetId::Binding(binding.declaration_site.clone()))
    );
    assert_eq!(
        scopes.import_origin(&binding.declaration_site).map(|origin| &origin.remote_target),
        Some(&SemanticTargetId::Module(shapes))
    );
}

#[test]
fn imported_class_participates_in_expression_type_inference_with_declaring_module_identity() {
    let project = ResolvedProjectId::from_raw(1);
    let point_module = module(project, &["shapes", "point"]);
    let consumer_module = module(project, &["consumer"]);

    let point_source: Arc<str> = Arc::from("class Point {}\nexport Point\n");
    let consumer_source: Arc<str> = Arc::from("import app.shapes.point.Point\nclass Consumer {\n  probe() { Point }\n}\n");
    let point_program = Arc::new(phalcom_ast::parse(&point_source, 0).program);
    let consumer_program = Arc::new(phalcom_ast::parse(&consumer_source, 0).program);

    let mut sources = BTreeMap::new();
    sources.insert(
        point_module.clone(),
        Arc::new(ParsedModuleUnit::new(
            point_module.clone(),
            ModuleKind::Module,
            None,
            point_source,
            point_program,
        )),
    );
    sources.insert(
        consumer_module.clone(),
        Arc::new(ParsedModuleUnit::new(
            consumer_module.clone(),
            ModuleKind::Module,
            None,
            consumer_source.clone(),
            consumer_program,
        )),
    );

    let point_symbol = SymbolId {
        module: point_module.clone(),
        name: "Point".into(),
    };
    let point_export = LinkedExport {
        public_name: "Point".into(),
        target: LinkedExportTarget::Binding(point_symbol.clone()),
        range: Default::default(),
    };

    let point_linked = LinkedModule {
        interface: LinkedModuleInterface {
            module: point_module.clone(),
            kind: ModuleKind::Module,
            exports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([("Point".into(), point_export)])),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([("Point".into(), GlobalBindingId(0))])),
            imports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::new()),
        },
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };

    let consumer_linked = LinkedModule {
        interface: LinkedModuleInterface {
            module: consumer_module.clone(),
            kind: ModuleKind::Module,
            exports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::new()),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([("Consumer".into(), GlobalBindingId(0))])),
            imports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([("Point".into(), ImportBindingId(0))])),
        },
        linked_reads: vec![LinkedReadSpec::Binding(point_symbol)],
        runtime_dependencies: vec![point_module.clone()],
    };

    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([
            (point_module.clone(), point_linked),
            (consumer_module.clone(), consumer_linked),
        ])),
        graphs: Default::default(),
        entry: consumer_module.clone(),
        initialization_order: vec![point_module.clone(), consumer_module.clone()],
    });

    let analysis = analyze_workspace(SemanticWorkspaceInput::new(linked, sources, 1)).expect("semantic test analysis should succeed");
    assert!(!analysis.snapshot.has_errors(), "diagnostics: {:#?}", analysis.snapshot.diagnostics);

    let point_decl = DeclarationId::new(point_module, "Point".into());
    let point_class_object = analysis
        .snapshot
        .declarations
        .get(&point_decl)
        .expect("Point declaration metadata")
        .class_object_type;
    let consumer_decl = DeclarationId::new(consumer_module, "Consumer".into());
    let probe = CallableId::new(
        consumer_decl,
        Selector::method("probe", Vec::new()).expect("probe selector"),
        DispatchSide::Instance,
    );
    let callable = analysis.snapshot.callable_analyses.get(&probe).expect("Consumer.probe analysis");
    let point_expression = callable
        .expressions
        .values()
        .find(|expression| consumer_source.get(expression.range.start..expression.range.end) == Some("Point"))
        .expect("Point expression in Consumer.probe");

    assert_eq!(
        point_expression.knowledge.ty(),
        Some(point_class_object),
        "imported class expression must infer the class object belonging to shapes.point.Point"
    );
}

#[test]
fn imported_static_domain_case_publishes_provider_identity_before_runtime_rejection() {
    let project = ResolvedProjectId::from_raw(707);
    let shapes_module = module(project, &["app", "shapes"]);
    let consumer_module = module(project, &["app", "consumer"]);
    let shape_source: Arc<str> = Arc::from("type data Shape { Scalar }\nexport Shape\n");
    let consumer_source: Arc<str> = Arc::from("import app.shapes.Shape\nclass Probe { @class status() { Shape::Scalar } }\n");
    let shape_parsed = phalcom_ast::parse(&shape_source, 0);
    let consumer_parsed = phalcom_ast::parse(&consumer_source, 0);
    assert!(shape_parsed.errors.is_empty(), "provider parse errors: {:#?}", shape_parsed.errors);
    assert!(consumer_parsed.errors.is_empty(), "consumer parse errors: {:#?}", consumer_parsed.errors);
    let shape_program = Arc::new(shape_parsed.program);
    let consumer_program = Arc::new(consumer_parsed.program);
    let shape_symbol = SymbolId {
        module: shapes_module.clone(),
        name: "Shape".into(),
    };
    let shape_export = LinkedExport {
        public_name: "Shape".into(),
        target: LinkedExportTarget::Binding(shape_symbol.clone()),
        range: Default::default(),
    };
    let shape_linked = LinkedModule {
        interface: LinkedModuleInterface {
            module: shapes_module.clone(),
            kind: ModuleKind::Module,
            exports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([("Shape".into(), shape_export)])),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([("Shape".into(), GlobalBindingId(0))])),
            imports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::new()),
        },
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };
    let consumer_linked = LinkedModule {
        interface: LinkedModuleInterface {
            module: consumer_module.clone(),
            kind: ModuleKind::Module,
            exports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::new()),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([("Probe".into(), GlobalBindingId(0))])),
            imports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([("Shape".into(), ImportBindingId(0))])),
        },
        linked_reads: vec![LinkedReadSpec::Binding(shape_symbol)],
        runtime_dependencies: vec![shapes_module.clone()],
    };
    let sources = BTreeMap::from([
        (
            shapes_module.clone(),
            Arc::new(ParsedModuleUnit::new(
                shapes_module.clone(),
                ModuleKind::Module,
                None,
                shape_source,
                shape_program,
            )),
        ),
        (
            consumer_module.clone(),
            Arc::new(ParsedModuleUnit::new(
                consumer_module.clone(),
                ModuleKind::Module,
                None,
                consumer_source.clone(),
                consumer_program,
            )),
        ),
    ]);
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([
            (shapes_module.clone(), shape_linked),
            (consumer_module.clone(), consumer_linked),
        ])),
        graphs: Default::default(),
        entry: consumer_module.clone(),
        initialization_order: vec![shapes_module.clone(), consumer_module.clone()],
    });

    let analysis = analyze_workspace(SemanticWorkspaceInput::new(linked, sources, 1)).expect("semantic test analysis should succeed");
    assert!(
        analysis
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == phalcom_semantic::diagnostic::DiagnosticCode::TypeOnlyRuntimeValue),
        "type-data case remains invalid as a runtime value: {:#?}",
        analysis.snapshot.diagnostics
    );
    let shape = DeclarationId::new(shapes_module.clone(), "Shape".into());
    let index = analysis.snapshot.source_index.module(&consumer_module).expect("consumer source index");
    let case_offset = consumer_source.rfind("Scalar").expect("case reference") + 1;
    assert_eq!(
        index.occurrences.occurrence_at(case_offset).and_then(|occurrence| occurrence.target.cloned()),
        Some(SemanticTargetId::StaticCase(phalcom_semantic::identity::StaticCaseId::new(
            phalcom_semantic::identity::StaticDomainDeclarationId::Nominal(shape.clone()),
            "Scalar",
        )))
    );

    let status = CallableId::new(
        DeclarationId::new(consumer_module, "Probe".into()),
        Selector::method("status", []).expect("status selector"),
        DispatchSide::Class,
    );
    let callable = analysis.snapshot.callable_analyses.get(&status).expect("status analysis");
    assert!(callable.associated_resolutions.values().any(|resolution| matches!(
        &resolution.kind,
        phalcom_semantic::checker::associated::AssociatedResolutionKind::StaticTerm { domain, .. }
            if matches!(analysis.snapshot.store.static_domain(*domain), Some(phalcom_semantic::types::store::StaticDomainData::Nominal { declaration }) if declaration == &shape)
    )));
}

fn static_domain_provider_input(provider_name: &str, provider_source: &str, consumer_source: &str, generation: u64) -> SemanticWorkspaceInput {
    let shapes_module = ModuleId::universe(ModulePath::from_components(vec![
        ModuleComponent::from_identifier("app").unwrap(),
        ModuleComponent::from_identifier("shapes").unwrap(),
    ]));
    let consumer_module = ModuleId::universe(ModulePath::from_components(vec![
        ModuleComponent::from_identifier("app").unwrap(),
        ModuleComponent::from_identifier("consumer").unwrap(),
    ]));
    let provider_source: Arc<str> = Arc::from(provider_source);
    let consumer_source: Arc<str> = Arc::from(consumer_source);
    let provider_parsed = phalcom_ast::parse(&provider_source, 0);
    let consumer_parsed = phalcom_ast::parse(&consumer_source, 0);
    assert!(provider_parsed.errors.is_empty(), "provider parse errors: {:#?}", provider_parsed.errors);
    assert!(consumer_parsed.errors.is_empty(), "consumer parse errors: {:#?}", consumer_parsed.errors);
    let provider_symbol = SymbolId {
        module: shapes_module.clone(),
        name: provider_name.into(),
    };
    let provider_export = LinkedExport {
        public_name: provider_name.into(),
        target: LinkedExportTarget::Binding(provider_symbol.clone()),
        range: Default::default(),
    };
    let provider_linked = LinkedModule {
        interface: LinkedModuleInterface {
            module: shapes_module.clone(),
            kind: ModuleKind::Module,
            exports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([(provider_name.into(), provider_export)])),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([(provider_name.into(), GlobalBindingId(0))])),
            imports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::new()),
        },
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };
    let consumer_linked = LinkedModule {
        interface: LinkedModuleInterface {
            module: consumer_module.clone(),
            kind: ModuleKind::Module,
            exports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::new()),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([
                ("Selected".into(), GlobalBindingId(0)),
                ("Probe".into(), GlobalBindingId(1)),
            ])),
            imports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([(provider_name.into(), ImportBindingId(0))])),
        },
        linked_reads: vec![LinkedReadSpec::Binding(provider_symbol)],
        runtime_dependencies: vec![shapes_module.clone()],
    };
    let sources = BTreeMap::from([
        (
            shapes_module.clone(),
            Arc::new(ParsedModuleUnit::new(
                shapes_module.clone(),
                ModuleKind::Module,
                None,
                provider_source,
                Arc::new(provider_parsed.program),
            )),
        ),
        (
            consumer_module.clone(),
            Arc::new(ParsedModuleUnit::new(
                consumer_module.clone(),
                ModuleKind::Module,
                None,
                consumer_source,
                Arc::new(consumer_parsed.program),
            )),
        ),
    ]);
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([
            (shapes_module.clone(), provider_linked),
            (consumer_module.clone(), consumer_linked),
        ])),
        graphs: Default::default(),
        entry: consumer_module.clone(),
        initialization_order: vec![shapes_module, consumer_module],
    });
    SemanticWorkspaceInput::new(linked, sources, generation)
}

#[test]
fn imported_static_domain_provider_edits_recompute_exact_consumer_and_match_cold() {
    use phalcom_semantic::db::QueryKey;
    use phalcom_semantic::identity::{DeclarationId, SemanticTargetId, StaticCaseId, StaticDomainDeclarationId};
    use phalcom_semantic::metadata::MetadataExporter;
    use phalcom_semantic::session::SemanticWorkspaceSession;
    use phalcom_type_meta::header::MetadataProfile;
    use phalcom_type_meta::identity::StableStaticDomainDeclarationRef;
    use phalcom_type_meta::validate::{ValidationLimits, validate_metadata_bundle};

    let provider = |cases: &str| format!("type data Shape {{ {cases} }}\nexport Shape\n");
    let consumer = "import app.shapes.Shape\n\
                   type Selected: Shape = Shape::Vector<Shape::Scalar>(_)\n\
                   class Probe { @class status() { Shape::Scalar } }\n";
    let source_a = provider("Scalar; Vector(_ size: Shape)");
    let mut session = SemanticWorkspaceSession::new();
    let revision_a = session
        .update(static_domain_provider_input("Shape", &source_a, consumer, 1))
        .expect("semantic test update should succeed");
    assert!(
        revision_a
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == phalcom_semantic::diagnostic::DiagnosticCode::TypeOnlyRuntimeValue)
    );
    assert!(revision_a.snapshot.internal_incidents.is_empty());

    let shapes_module = ModuleId::universe(ModulePath::from_components(vec![
        ModuleComponent::from_identifier("app").unwrap(),
        ModuleComponent::from_identifier("shapes").unwrap(),
    ]));
    let consumer_module = ModuleId::universe(ModulePath::from_components(vec![
        ModuleComponent::from_identifier("app").unwrap(),
        ModuleComponent::from_identifier("consumer").unwrap(),
    ]));
    let shape = DeclarationId::new(shapes_module.clone(), "Shape".into());
    let selected_alias = DeclarationId::new(consumer_module.clone(), "Selected".into());
    let provider_key = QueryKey::EnumDeclaration(shape.clone());
    let vector_case_key = QueryKey::StaticCaseSignature(StaticCaseId::with_selector(
        StaticDomainDeclarationId::Nominal(shape.clone()),
        Selector::method("Vector", [SelectorSlot::Positional]).expect("Vector constructor selector"),
    ));
    let provider_fp_a = session.db().ready_product_fingerprint(&provider_key).expect("provider vA product");
    let case_fp_a = session.db().ready_product_fingerprint(&vector_case_key).expect("provider Vector signature vA");

    let index_a = revision_a.snapshot.source_index.module(&consumer_module).expect("consumer source index");
    let scalar_offset = consumer.rfind("Scalar").expect("Scalar source occurrence") + 1;
    assert_eq!(
        index_a
            .occurrences
            .occurrence_at(scalar_offset)
            .and_then(|occurrence| occurrence.target.cloned()),
        Some(SemanticTargetId::StaticCase(StaticCaseId::new(
            StaticDomainDeclarationId::Nominal(shape.clone()),
            "Scalar"
        )))
    );

    let bundle = MetadataExporter::new(
        &revision_a.snapshot.store,
        Some(&revision_a.snapshot.declarations),
        None,
        None,
        MetadataProfile::RuntimePublic,
    )
    .with_algebraic_semantics(&revision_a.snapshot.data_semantics, &revision_a.snapshot.enum_semantics)
    .with_associated_static_domains(&revision_a.snapshot.associated_static_domains)
    .with_aliases(&revision_a.snapshot.type_aliases)
    .build_bundle(&[])
    .expect("multi-module provider metadata");
    let provider_catalog = bundle
        .static_domains
        .iter()
        .find(|record| {
            matches!(&record.declaration,
            StableStaticDomainDeclarationRef::Nominal(declaration)
                if declaration.path.last().is_some_and(|name| name.as_ref() == "Shape")
                    && declaration.module.path.iter().any(|part| part.as_ref() == "shapes"))
        })
        .expect("provider Shape catalog");
    assert!(provider_catalog.cases.iter().any(|case| case.name.as_ref() == "Vector(_)"));
    let selected = bundle
        .aliases
        .iter()
        .find(|alias| alias.declaration.path.last().is_some_and(|name| name.as_ref() == "Selected"))
        .expect("consumer alias metadata");
    assert!(matches!(&bundle.types[selected.target.0 as usize].form,
        phalcom_type_meta::type_node::TypeNode::StaticData(data)
            if matches!(&data.domain, phalcom_type_meta::kind::StaticDomainRef::Nominal { declaration }
                if declaration.module.path.iter().any(|part| part.as_ref() == "shapes"))
                && data.constructor.case_name.as_ref() == "Vector(_)"));
    validate_metadata_bundle(&bundle, &ValidationLimits::default()).expect("composed provider metadata validates");

    let source_b = provider("Scalar; Vector(_ size: Int)");
    let revision_b = session
        .update(static_domain_provider_input("Shape", &source_b, consumer, 2))
        .expect("semantic test update should succeed");
    assert!(revision_b.snapshot.internal_incidents.is_empty());
    let provider_fp_b = session.db().ready_product_fingerprint(&provider_key).expect("provider payload product");
    let case_fp_b = session.db().ready_product_fingerprint(&vector_case_key).expect("provider Vector signature vB");
    assert_ne!(provider_fp_a, provider_fp_b, "provider payload-kind edit changes its declaration product");
    assert_ne!(case_fp_a, case_fp_b, "provider Vector payload edit changes its checked signature");

    let mut cold_b = SemanticWorkspaceSession::new();
    let cold_revision_b = cold_b
        .update(static_domain_provider_input("Shape", &source_b, consumer, 1))
        .expect("semantic test update should succeed");
    let diagnostic_codes = |snapshot: &phalcom_semantic::SemanticSnapshot| {
        let mut codes = snapshot
            .all_diagnostics()
            .map(|diagnostic| (format!("{:?}", diagnostic.code), diagnostic.primary_range.start))
            .collect::<Vec<_>>();
        codes.sort();
        codes
    };
    assert_eq!(diagnostic_codes(&revision_b.snapshot), diagnostic_codes(&cold_revision_b.snapshot));
    let alias_form = |snapshot: &phalcom_semantic::SemanticSnapshot| snapshot.type_aliases.form(&selected_alias).map(|form| snapshot.store.format_type(form));
    assert_ne!(
        alias_form(&revision_a.snapshot),
        alias_form(&revision_b.snapshot),
        "provider signature edit changes or rejects the imported consumer alias form",
    );
    assert_eq!(alias_form(&revision_b.snapshot), alias_form(&cold_revision_b.snapshot));

    let source_c = provider("Atom; Sized(_ size: Int)");
    let revision_c = session
        .update(static_domain_provider_input("Shape", &source_c, consumer, 3))
        .expect("semantic test update should succeed");
    assert!(revision_c.snapshot.internal_incidents.is_empty());
    let mut cold_c = SemanticWorkspaceSession::new();
    let cold_revision_c = cold_c
        .update(static_domain_provider_input("Shape", &source_c, consumer, 1))
        .expect("semantic test update should succeed");
    assert_eq!(diagnostic_codes(&revision_c.snapshot), diagnostic_codes(&cold_revision_c.snapshot));
    assert_eq!(alias_form(&revision_c.snapshot), alias_form(&cold_revision_c.snapshot));
    let index_c = revision_c
        .snapshot
        .source_index
        .module(&consumer_module)
        .expect("updated consumer source index");
    assert!(
        index_c
            .occurrences
            .occurrence_at(scalar_offset)
            .is_none_or(|occurrence| occurrence.target.is_none()),
        "renamed provider singleton no longer has a current source target"
    );
    assert!(
        !revision_c
            .snapshot
            .enum_semantics
            .enum_info(&shape)
            .unwrap()
            .static_data
            .as_ref()
            .unwrap()
            .constructors
            .is_empty()
    );
}

#[test]
fn imported_associated_static_domain_retains_provider_owner_specialization() {
    use phalcom_semantic::identity::DeclarationId;
    use phalcom_semantic::metadata::MetadataExporter;
    use phalcom_semantic::session::SemanticWorkspaceSession;
    use phalcom_type_meta::header::MetadataProfile;
    use phalcom_type_meta::identity::StableStaticDomainDeclarationRef;
    use phalcom_type_meta::kind::StaticDomainRef;
    use phalcom_type_meta::type_node::TypeNode;
    use phalcom_type_meta::validate::{ValidationLimits, validate_metadata_bundle};

    let provider_source = "class Container<T: Type> { type data State { Ready } }\nexport Container\n";
    let consumer_source = "import app.shapes.Container\ntype Selected: Container<Int>::State = Container<Int>::State::Ready\n";
    let mut session = SemanticWorkspaceSession::new();
    let update = session
        .update(static_domain_provider_input("Container", provider_source, consumer_source, 1))
        .expect("semantic test update should succeed");
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);
    assert!(update.snapshot.internal_incidents.is_empty());

    let shapes_module = ModuleId::universe(ModulePath::from_components(vec![
        ModuleComponent::from_identifier("app").unwrap(),
        ModuleComponent::from_identifier("shapes").unwrap(),
    ]));
    let consumer_module = ModuleId::universe(ModulePath::from_components(vec![
        ModuleComponent::from_identifier("app").unwrap(),
        ModuleComponent::from_identifier("consumer").unwrap(),
    ]));
    let owner = DeclarationId::new(shapes_module, "Container".into());
    let selected = DeclarationId::new(consumer_module, "Selected".into());
    let form = update.snapshot.type_aliases.form(&selected).expect("consumer static alias");
    let phalcom_semantic::types::store::TypeData::StaticData(term) = update.snapshot.store.get(form) else {
        panic!("associated case alias should retain static data: {}", update.snapshot.store.format_type(form));
    };
    let phalcom_semantic::types::store::StaticDomainData::Associated { declaration, owner_arguments } =
        update.snapshot.store.static_domain(term.domain).expect("associated domain")
    else {
        panic!("associated static domain specialization expected");
    };
    assert_eq!(declaration.owner, owner);
    assert_eq!(declaration.name.as_ref(), "State");
    assert_eq!(owner_arguments.len(), 1);
    assert_eq!(update.snapshot.store.static_constructor(term.constructor).unwrap().case_name.as_ref(), "Ready");

    let bundle = MetadataExporter::new(
        &update.snapshot.store,
        Some(&update.snapshot.declarations),
        None,
        None,
        MetadataProfile::RuntimePublic,
    )
    .with_algebraic_semantics(&update.snapshot.data_semantics, &update.snapshot.enum_semantics)
    .with_associated_static_domains(&update.snapshot.associated_static_domains)
    .with_aliases(&update.snapshot.type_aliases)
    .build_bundle(&[])
    .expect("cross-module associated static metadata");
    let alias = bundle
        .aliases
        .iter()
        .find(|alias| alias.declaration.path.last().is_some_and(|name| name.as_ref() == "Selected"))
        .expect("Selected metadata alias");
    let TypeNode::StaticData(metadata_term) = &bundle.types[alias.target.0 as usize].form else {
        panic!("expected associated static term metadata");
    };
    assert_eq!(metadata_term.constructor.case_name.as_ref(), "Ready");
    assert!(matches!(&metadata_term.domain, StaticDomainRef::Associated { declaration, owner_environment }
        if declaration.name.as_ref() == "State"
            && declaration.owner.path.last().is_some_and(|name| name.as_ref() == "Container")
            && declaration.owner.module.path.iter().any(|name| name.as_ref() == "shapes")
            && matches!(owner_environment.as_ref(), [phalcom_type_meta::type_node::StaticArgumentRef::Term(_)])));
    assert!(bundle.static_domains.iter().any(|record| matches!(&record.declaration,
        StableStaticDomainDeclarationRef::Associated(declaration)
            if declaration.name.as_ref() == "State"
                && declaration.owner.path.last().is_some_and(|name| name.as_ref() == "Container")
                && declaration.owner.module.path.iter().any(|name| name.as_ref() == "shapes")
                && record.cases.iter().any(|case| case.name.as_ref() == "Ready"))));
    validate_metadata_bundle(&bundle, &ValidationLimits::default()).expect("provider associated metadata validates");
}

#[test]
fn editor_definition_sites_exclude_local_import_declaration_for_external_target() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("package.ph"), "").unwrap();
    let main_path = root.path().join("main.ph");
    let shapes_path = root.path().join("shapes.ph");
    let location = |path: &std::path::Path| phalcom_modules::SourceLocation {
        source_id: phalcom_modules::SourceId(path.to_string_lossy().into()),
        display_path: path.to_path_buf(),
    };
    let main_text = "from .shapes import Circle as C\nC\n";

    let mut session = phalcom_semantic::SemanticWorkspaceSession::new();
    let publication = session
        .apply_module_mutations([
            phalcom_modules::WorkspaceSourceBatchMutation::SetOverlay {
                source: location(&main_path),
                text: Arc::from(main_text),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
            phalcom_modules::WorkspaceSourceBatchMutation::SetOverlay {
                source: location(&shapes_path),
                text: Arc::from("class Circle {}\nexport Circle\n"),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("workspace publication should succeed");

    let queries = publication.snapshot.module_queries();
    let main_module = queries
        .module_for_display_path(&main_path)
        .cloned()
        .expect("main source must map to a canonical module");
    let shapes_module = queries
        .module_for_display_path(&shapes_path)
        .cloned()
        .expect("shapes source must map to a canonical module");
    let circle = SemanticTargetId::Declaration(DeclarationId::new(shapes_module.clone(), "Circle".into()));
    let editor = publication.snapshot.editor();
    let import_offset = main_text.find("Circle").expect("imported Circle token") + 1;

    assert_eq!(editor.target_at(&main_module, import_offset), Some(circle.clone()));

    let definitions = editor.definition_sites(&circle);
    assert_eq!(definitions.len(), 1, "only the exported class declaration is a canonical definition");
    assert!(
        matches!(&definitions[0].owner, phalcom_semantic::SourceOwner::Module(module) if module == &shapes_module),
        "the local import declaration must not masquerade as the external class definition"
    );

    let references = editor.reference_sites(&circle);
    assert!(
        references
            .iter()
            .any(|site| matches!(&site.owner, phalcom_semantic::SourceOwner::Module(module) if module == &main_module)),
        "the import declaration/use must remain references to the external target"
    );
}

#[test]
fn imported_nominal_binder_header_supports_variadic_alias_forwarding() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("package.ph"), "").unwrap();
    let main_path = root.path().join("main.ph");
    let provider_path = root.path().join("provider.ph");
    let location = |path: &std::path::Path| phalcom_modules::SourceLocation {
        source_id: phalcom_modules::SourceId(path.to_string_lossy().into()),
        display_path: path.to_path_buf(),
    };
    let main_source = "from .provider import Target\ntype Forward<*Ts> = Target<*Ts>\n";

    let mut session = phalcom_semantic::SemanticWorkspaceSession::new();
    let publication = session
        .apply_module_mutations([
            phalcom_modules::WorkspaceSourceBatchMutation::SetOverlay {
                source: location(&main_path),
                text: Arc::from(main_source),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
            phalcom_modules::WorkspaceSourceBatchMutation::SetOverlay {
                source: location(&provider_path),
                text: Arc::from("class Target<*Ts> {}\nexport Target\n"),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("workspace publication should succeed");

    let main_module = publication
        .snapshot
        .module_queries()
        .module_for_display_path(&main_path)
        .cloned()
        .expect("main source must map to a canonical module");
    let forward = DeclarationId::new(main_module, "Forward".into());
    assert!(
        publication.snapshot.type_aliases.form(&forward).is_some(),
        "{:?}",
        publication.snapshot.diagnostics
    );

    let invalidated = session
        .apply_module_mutations([phalcom_modules::WorkspaceSourceBatchMutation::SetOverlay {
            source: location(&provider_path),
            text: Arc::from("class Target<T> {}\nexport Target\n"),
            revision: phalcom_modules::SourceRevision(2),
            recovered_program: None,
        }])
        .expect("provider-only update should publish");
    assert!(
        invalidated.snapshot.type_aliases.form(&forward).is_none(),
        "an unchanged alias must be recomputed when its imported nominal binder contract changes"
    );
}

#[test]
fn workspace_partial_snapshot_publishes_module_diagnostic_and_valid_product() {
    let project = ResolvedProjectId::from_raw(1);
    let valid_module = module(project, &["valid"]);
    let blocked_module = module(project, &["blocked"]);
    let valid_source: Arc<str> = Arc::from("class Product {}\n");
    let blocked_source: Arc<str> = Arc::from("let blocked = 1\n");

    let valid_program = Arc::new(phalcom_ast::parse(&valid_source, 0).program);
    let blocked_program = Arc::new(phalcom_ast::parse(&blocked_source, 0).program);
    let sources = BTreeMap::from([
        (
            valid_module.clone(),
            Arc::new(ParsedModuleUnit::new(
                valid_module.clone(),
                ModuleKind::Module,
                None,
                valid_source,
                valid_program,
            )),
        ),
        (
            blocked_module.clone(),
            Arc::new(ParsedModuleUnit::new(
                blocked_module.clone(),
                ModuleKind::Module,
                None,
                blocked_source,
                blocked_program,
            )),
        ),
    ]);
    let linked_module = |module: ModuleId| LinkedModule {
        interface: LinkedModuleInterface {
            module,
            kind: ModuleKind::Module,
            exports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::new()),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout::default(),
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([
            (valid_module.clone(), linked_module(valid_module.clone())),
            (blocked_module.clone(), linked_module(blocked_module.clone())),
        ])),
        graphs: Default::default(),
        entry: valid_module.clone(),
        initialization_order: vec![valid_module.clone(), blocked_module.clone()],
    });
    let diagnostic = ModuleDiagnostic::new(
        blocked_module.clone(),
        ModuleDiagnosticKind::ModuleNotFound("missing".into()),
        SourceRange::new(4, 11),
        "blocked module import is unavailable",
    );

    let mut session = phalcom_semantic::SemanticWorkspaceSession::new();
    let update = session
        .update(
            SemanticWorkspaceInput::new(linked, sources, 7)
                .with_diagnostics(BTreeMap::from([(blocked_module.clone(), vec![diagnostic.clone()])]))
                .with_blocked_modules(BTreeSet::from([blocked_module.clone()])),
        )
        .expect("semantic test update should succeed");
    let snapshot = update.snapshot;

    assert_eq!(snapshot.generation, 7);
    assert_eq!(snapshot.status(), &SnapshotStatus::Partial { blocked_modules: 1 });
    let diagnostics = snapshot.diagnostics_for(&blocked_module).expect("blocked module diagnostics");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, phalcom_semantic::diagnostic::DiagnosticCode::ModuleImportUnresolved);
    assert_eq!(diagnostics[0].message, diagnostic.message);
    assert_eq!(diagnostics[0].primary.module, blocked_module);
    assert_eq!(diagnostics[0].primary_range, diagnostic.range);

    assert!(snapshot.sources.contains_key(&valid_module));
    assert!(snapshot.source_index().module(&valid_module).is_some());
    assert!(snapshot.module_products.linked.contains_key(&valid_module));
    assert_eq!(snapshot.module_products.linked[&valid_module].module, valid_module);
}

#[test]
fn qualified_module_type_lookup_uses_public_linked_exports_and_fails_closed() {
    let project = ResolvedProjectId::from_raw(1);
    let models = module(project, &["models"]);
    let consumer = module(project, &["consumer"]);
    let public = DeclarationId::new(models.clone(), "Public".into());
    let hidden = DeclarationId::new(models.clone(), "Hidden".into());
    let public_export = LinkedExport {
        public_name: "Public".into(),
        target: LinkedExportTarget::Binding(SymbolId {
            module: models.clone(),
            name: "Public".into(),
        }),
        range: Default::default(),
    };
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([
            (
                models.clone(),
                LinkedModule {
                    interface: LinkedModuleInterface {
                        module: models.clone(),
                        kind: ModuleKind::Module,
                        exports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([("Public".into(), public_export)])),
                        metadata: ModuleMetadata::default(),
                    },
                    bindings: ModuleBindingLayout::default(),
                    linked_reads: Vec::new(),
                    runtime_dependencies: Vec::new(),
                },
            ),
            (
                consumer.clone(),
                LinkedModule {
                    interface: LinkedModuleInterface {
                        module: consumer.clone(),
                        kind: ModuleKind::Module,
                        exports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::new()),
                        metadata: ModuleMetadata::default(),
                    },
                    bindings: ModuleBindingLayout {
                        local_globals: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::new()),
                        imports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([("models".into(), ImportBindingId(0))])),
                    },
                    linked_reads: vec![LinkedReadSpec::Module(models.clone())],
                    runtime_dependencies: vec![models.clone()],
                },
            ),
        ])),
        graphs: Default::default(),
        entry: consumer.clone(),
        initialization_order: vec![models.clone(), consumer.clone()],
    });
    let resolver = LinkedTypeResolver::new(linked, HashSet::from([public.clone(), hidden]), ModuleId::universe_root());

    assert_eq!(resolver.resolve_type_name(&consumer, "models", &["Public".into()]), Some(public));
    assert_eq!(
        resolver.resolve_type_name(&consumer, "models", &["Hidden".into()]),
        None,
        "private declarations must not become visible through a module alias"
    );
    assert_eq!(
        resolver.resolve_type_name(&consumer, "models", &["nested".into(), "Public".into()]),
        None,
        "unsupported deep qualification must fail closed"
    );
}

#[test]
fn exported_global_and_top_level_binding_use_module_binding_target() {
    let project = ResolvedProjectId::from_raw(1);
    let provider = module(project, &["provider"]);
    let consumer = module(project, &["consumer"]);
    let symbol = SymbolId {
        module: provider.clone(),
        name: "version".into(),
    };
    let provider_source: Arc<str> = Arc::from("const version = 1\nexport version\n");
    let consumer_source: Arc<str> = Arc::from("from provider import version\nversion\n");
    let provider_program = Arc::new(phalcom_ast::parse(&provider_source, 0).program);
    let consumer_program = Arc::new(phalcom_ast::parse(&consumer_source, 0).program);
    let sources = BTreeMap::from([
        (
            provider.clone(),
            Arc::new(ParsedModuleUnit::new(
                provider.clone(),
                ModuleKind::Module,
                None,
                provider_source,
                provider_program,
            )),
        ),
        (
            consumer.clone(),
            Arc::new(ParsedModuleUnit::new(
                consumer.clone(),
                ModuleKind::Module,
                None,
                consumer_source,
                consumer_program,
            )),
        ),
    ]);
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([
            (
                provider.clone(),
                LinkedModule {
                    interface: LinkedModuleInterface {
                        module: provider.clone(),
                        kind: ModuleKind::Module,
                        exports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([(
                            "version".into(),
                            LinkedExport {
                                public_name: "version".into(),
                                target: LinkedExportTarget::Binding(symbol.clone()),
                                range: Default::default(),
                            },
                        )])),
                        metadata: ModuleMetadata::default(),
                    },
                    bindings: ModuleBindingLayout {
                        local_globals: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([("version".into(), GlobalBindingId(0))])),
                        imports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::new()),
                    },
                    linked_reads: Vec::new(),
                    runtime_dependencies: Vec::new(),
                },
            ),
            (
                consumer.clone(),
                LinkedModule {
                    interface: LinkedModuleInterface {
                        module: consumer.clone(),
                        kind: ModuleKind::Module,
                        exports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::new()),
                        metadata: ModuleMetadata::default(),
                    },
                    bindings: ModuleBindingLayout {
                        local_globals: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::new()),
                        imports: phalcom_modules::authority::ModuleAuthorityMap::Source(BTreeMap::from([("version".into(), ImportBindingId(0))])),
                    },
                    linked_reads: vec![LinkedReadSpec::Binding(symbol.clone())],
                    runtime_dependencies: vec![provider.clone()],
                },
            ),
        ])),
        graphs: Default::default(),
        entry: consumer.clone(),
        initialization_order: vec![provider.clone(), consumer.clone()],
    });
    let mut session = phalcom_semantic::SemanticWorkspaceSession::new();
    let snapshot = session
        .update(SemanticWorkspaceInput::new(linked, sources, 1))
        .expect("semantic test update should succeed")
        .snapshot;
    let target = SemanticTargetId::ModuleBinding(symbol.clone());

    let provider_binding = snapshot
        .source_index()
        .module(&provider)
        .expect("provider source index")
        .structure
        .bindings
        .values()
        .find(|binding| binding.name.as_ref() == "version")
        .expect("provider top-level binding");
    assert_eq!(snapshot.source_index().target_for(&provider_binding.declaration_site), Some(&target));
    let consumer_binding = snapshot
        .source_index()
        .module(&consumer)
        .expect("consumer source index")
        .structure
        .bindings
        .values()
        .find(|binding| binding.name.as_ref() == "version")
        .expect("consumer import binding");
    let consumer_binding_target = SemanticTargetId::Binding(consumer_binding.declaration_site.clone());
    assert_eq!(
        snapshot.source_index().target_for(&consumer_binding.declaration_site),
        Some(&consumer_binding_target)
    );
    assert_eq!(
        snapshot
            .source_index()
            .module(&consumer)
            .and_then(|module| module.structure.import_origin(&consumer_binding.declaration_site))
            .map(|origin| &origin.remote_target),
        Some(&target)
    );

    let definitions = snapshot.editor().definition_sites(&target);
    assert_eq!(definitions.len(), 1, "only provider top-level binding is a definition");
    assert!(matches!(
        &definitions[0].owner,
        phalcom_semantic::SourceOwner::Module(module) if module == &provider
    ));
}

#[test]
fn p3_cross_module_provider_pack_alias_survives_and_recomputes_cold_parity() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("package.ph"), "").unwrap();
    let main_path = root.path().join("main.ph");
    let provider_path = root.path().join("provider.ph");
    let location = |path: &std::path::Path| phalcom_modules::SourceLocation {
        source_id: phalcom_modules::SourceId(path.to_string_lossy().into()),
        display_path: path.to_path_buf(),
    };
    let main_source = "from .provider import Wrap\ntype MyAlias<*Ts> = Wrap<*Ts>\n";

    let mut session = phalcom_semantic::SemanticWorkspaceSession::new();
    let publication = session
        .apply_module_mutations([
            phalcom_modules::WorkspaceSourceBatchMutation::SetOverlay {
                source: location(&main_path),
                text: Arc::from(main_source),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
            phalcom_modules::WorkspaceSourceBatchMutation::SetOverlay {
                source: location(&provider_path),
                text: Arc::from("type Wrap<*Ts> = (*Ts,)\nexport Wrap\n"),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("workspace publication should succeed");

    let main_module = publication
        .snapshot
        .module_queries()
        .module_for_display_path(&main_path)
        .cloned()
        .expect("main source must map to a canonical module");
    let alias_id = DeclarationId::new(main_module.clone(), "MyAlias".into());
    let initial_form = publication.snapshot.type_aliases.form(&alias_id).expect("initial MyAlias");

    // Provider edit: change Wrap from (*Ts,) to (Int, *Ts)
    let updated = session
        .apply_module_mutations([phalcom_modules::WorkspaceSourceBatchMutation::SetOverlay {
            source: location(&provider_path),
            text: Arc::from("type Wrap<*Ts> = (Int, *Ts)\nexport Wrap\n"),
            revision: phalcom_modules::SourceRevision(2),
            recovered_program: None,
        }])
        .expect("update should publish");

    let inc_form = updated.snapshot.type_aliases.form(&alias_id).expect("updated MyAlias");
    assert_ne!(
        publication
            .snapshot
            .store
            .arena()
            .get_lambda(match publication.snapshot.store.get(initial_form) {
                phalcom_semantic::types::store::TypeData::Lambda(id) => *id,
                _ => panic!("expected lambda"),
            })
            .body,
        updated
            .snapshot
            .store
            .arena()
            .get_lambda(match updated.snapshot.store.get(inc_form) {
                phalcom_semantic::types::store::TypeData::Lambda(id) => *id,
                _ => panic!("expected lambda"),
            })
            .body,
    );

    // Cold parity check
    let mut cold_session = phalcom_semantic::SemanticWorkspaceSession::new();
    let cold_pub = cold_session
        .apply_module_mutations([
            phalcom_modules::WorkspaceSourceBatchMutation::SetOverlay {
                source: location(&main_path),
                text: Arc::from(main_source),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
            phalcom_modules::WorkspaceSourceBatchMutation::SetOverlay {
                source: location(&provider_path),
                text: Arc::from("type Wrap<*Ts> = (Int, *Ts)\nexport Wrap\n"),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("cold publication should succeed");
    let cold_form = cold_pub.snapshot.type_aliases.form(&alias_id).expect("cold MyAlias");

    assert_eq!(updated.snapshot.store.format_type(inc_form), cold_pub.snapshot.store.format_type(cold_form));
}

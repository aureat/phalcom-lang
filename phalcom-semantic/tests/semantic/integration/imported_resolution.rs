use phalcom_common::selector::Selector;
use phalcom_common::range::SourceRange;
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
use phalcom_semantic::{OccurrenceIndex, SemanticWorkspaceInput, SourceIndexContext, analyze_workspace, build_source_scope_index};
use phalcom_semantic::types::annotation::TypeResolver;
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

    let mut scopes = build_source_scope_index(importer, &parsed.program, &context);
    let occurrences = OccurrenceIndex::from_program(&mut scopes, &parsed.program);
    let use_offset = source.rfind("Circle").expect("Circle use") + 1;
    let occurrence = occurrences.occurrence_at(use_offset).expect("Circle occurrence");

    assert_eq!(
        occurrence.target,
        Some(&SemanticTargetId::Declaration(circle)),
        "a read through an import binding must preserve the imported declaration's canonical identity"
    );
}

#[test]
fn imported_alias_keeps_local_declaration_metadata_and_external_read_identity() {
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
    assert_eq!(scopes.target_for(&alias_site), Some(&SemanticTargetId::Declaration(circle.clone())));
    let occurrences = OccurrenceIndex::from_program(&mut scopes, &parsed.program);
    let use_offset = source.rfind('C').expect("alias use");
    assert_eq!(
        occurrences.occurrence_at(use_offset).and_then(|occurrence| occurrence.target),
        Some(&SemanticTargetId::Declaration(circle))
    );
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
    assert_eq!(
        occurrences.occurrence_at(use_offset).and_then(|occurrence| occurrence.target),
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
            exports: BTreeMap::from([("Point".into(), point_export)]),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: BTreeMap::from([("Point".into(), GlobalBindingId(0))]),
            imports: BTreeMap::new(),
        },
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };

    let consumer_linked = LinkedModule {
        interface: LinkedModuleInterface {
            module: consumer_module.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::new(),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: BTreeMap::from([("Consumer".into(), GlobalBindingId(0))]),
            imports: BTreeMap::from([("Point".into(), ImportBindingId(0))]),
        },
        linked_reads: vec![LinkedReadSpec::Binding(point_symbol)],
        runtime_dependencies: vec![point_module.clone()],
    };

    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules: BTreeMap::from([(point_module.clone(), point_linked), (consumer_module.clone(), consumer_linked)]),
        graphs: Default::default(),
        entry: consumer_module.clone(),
        initialization_order: vec![point_module.clone(), consumer_module.clone()],
    });

    let analysis = analyze_workspace(SemanticWorkspaceInput::new(
        linked,
        sources,
        1,
    ));
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
fn editor_definition_sites_exclude_local_import_declaration_for_external_target() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("package.ph"), "").unwrap();
    let main_path = root.path().join("main.ph");
    let shapes_path = root.path().join("shapes.ph");
    let location = |path: &std::path::Path| phalcom_modules::SourceLocation {
        source_id: phalcom_modules::SourceId(path.to_string_lossy().into()),
        display_path: path.to_path_buf(),
    };
    let main_text = "from .shapes import Circle\nCircle\n";

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
            exports: BTreeMap::new(),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout::default(),
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules: BTreeMap::from([
            (valid_module.clone(), linked_module(valid_module.clone())),
            (blocked_module.clone(), linked_module(blocked_module.clone())),
        ]),
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
    let update = session.update(
        SemanticWorkspaceInput::new(linked, sources, 7)
            .with_diagnostics(BTreeMap::from([(blocked_module.clone(), vec![diagnostic.clone()])]))
            .with_blocked_modules(BTreeSet::from([blocked_module.clone()])),
    );
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
        modules: BTreeMap::from([
            (
                models.clone(),
                LinkedModule {
                    interface: LinkedModuleInterface {
                        module: models.clone(),
                        kind: ModuleKind::Module,
                        exports: BTreeMap::from([("Public".into(), public_export)]),
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
                        exports: BTreeMap::new(),
                        metadata: ModuleMetadata::default(),
                    },
                    bindings: ModuleBindingLayout {
                        local_globals: BTreeMap::new(),
                        imports: BTreeMap::from([("models".into(), ImportBindingId(0))]),
                    },
                    linked_reads: vec![LinkedReadSpec::Module(models.clone())],
                    runtime_dependencies: vec![models.clone()],
                },
            ),
        ]),
        graphs: Default::default(),
        entry: consumer.clone(),
        initialization_order: vec![models.clone(), consumer.clone()],
    });
    let resolver = LinkedTypeResolver::new(linked, HashSet::from([public.clone(), hidden]), ModuleId::universe_root());

    assert_eq!(
        resolver.resolve_type_name(&consumer, "models", &["Public".into()]),
        Some(public)
    );
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
            Arc::new(ParsedModuleUnit::new(provider.clone(), ModuleKind::Module, None, provider_source, provider_program)),
        ),
        (
            consumer.clone(),
            Arc::new(ParsedModuleUnit::new(consumer.clone(), ModuleKind::Module, None, consumer_source, consumer_program)),
        ),
    ]);
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules: BTreeMap::from([
            (
                provider.clone(),
                LinkedModule {
                    interface: LinkedModuleInterface {
                        module: provider.clone(),
                        kind: ModuleKind::Module,
                        exports: BTreeMap::from([(
                            "version".into(),
                            LinkedExport {
                                public_name: "version".into(),
                                target: LinkedExportTarget::Binding(symbol.clone()),
                                range: Default::default(),
                            },
                        )]),
                        metadata: ModuleMetadata::default(),
                    },
                    bindings: ModuleBindingLayout {
                        local_globals: BTreeMap::from([("version".into(), GlobalBindingId(0))]),
                        imports: BTreeMap::new(),
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
                        exports: BTreeMap::new(),
                        metadata: ModuleMetadata::default(),
                    },
                    bindings: ModuleBindingLayout {
                        local_globals: BTreeMap::new(),
                        imports: BTreeMap::from([("version".into(), ImportBindingId(0))]),
                    },
                    linked_reads: vec![LinkedReadSpec::Binding(symbol.clone())],
                    runtime_dependencies: vec![provider.clone()],
                },
            ),
        ]),
        graphs: Default::default(),
        entry: consumer.clone(),
        initialization_order: vec![provider.clone(), consumer.clone()],
    });
    let mut session = phalcom_semantic::SemanticWorkspaceSession::new();
    let snapshot = session
        .update(SemanticWorkspaceInput::new(linked, sources, 1))
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
    assert_eq!(
        snapshot.source_index().target_for(&provider_binding.declaration_site),
        Some(&target)
    );
    let consumer_binding = snapshot
        .source_index()
        .module(&consumer)
        .expect("consumer source index")
        .structure
        .bindings
        .values()
        .find(|binding| binding.name.as_ref() == "version")
        .expect("consumer import binding");
    assert_eq!(snapshot.source_index().target_for(&consumer_binding.declaration_site), Some(&target));

    let definitions = snapshot.editor().definition_sites(&target);
    assert_eq!(definitions.len(), 1, "only provider top-level binding is a definition");
    assert!(matches!(
        &definitions[0].owner,
        phalcom_semantic::SourceOwner::Module(module) if module == &provider
    ));
}

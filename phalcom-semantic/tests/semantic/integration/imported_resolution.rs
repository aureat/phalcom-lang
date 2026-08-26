use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::{LinkedExport, LinkedExportTarget, LinkedModuleInterface};
use phalcom_modules::linker::{GlobalBindingId, ImportBindingId, LinkedModule, LinkedProgram, LinkedReadSpec, ModuleBindingLayout, SymbolId};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::source::{ModuleKind, ParsedModuleUnit};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, SemanticTargetId};
use phalcom_semantic::{OccurrenceIndex, SemanticWorkspaceInput, SourceIndexContext, analyze_workspace, build_source_scope_index};
use std::collections::BTreeMap;
use std::sync::Arc;

fn module(project: ResolvedProjectId, path: &[&str]) -> ModuleId {
    ModuleId::resolved(
        project,
        ModulePath::from_components(
            path.iter()
                .map(|component| ModuleComponent::from_identifier(component).expect("valid module component"))
                .collect(),
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
fn imported_class_participates_in_expression_type_inference_with_declaring_module_identity() {
    let project = ResolvedProjectId::from_raw(1);
    let point_module = module(project, &["shapes", "point"]);
    let consumer_module = module(project, &["consumer"]);

    let point_source: Arc<str> = Arc::from("class Point {}\nexport Point\n");
    let consumer_source: Arc<str> = Arc::from(
        "import app.shapes.point.Point\nclass Consumer {\n  probe() { Point }\n}\n",
    );
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
        modules: BTreeMap::from([
            (point_module.clone(), point_linked),
            (consumer_module.clone(), consumer_linked),
        ]),
        graphs: Default::default(),
        entry: consumer_module.clone(),
        initialization_order: vec![point_module.clone(), consumer_module.clone()],
    });

    let analysis = analyze_workspace(SemanticWorkspaceInput {
        linked,
        sources,
        generation: 1,
    });
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

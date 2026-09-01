use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::{LinkedExport, LinkedExportTarget, LinkedModuleInterface};
use phalcom_modules::linker::{GlobalBindingId, ImportBindingId, LinkedModule, LinkedProgram, LinkedReadSpec, ModuleBindingLayout, SymbolId};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::project::ProjectUniverse;
use phalcom_modules::source::{ModuleKind, ParsedModuleUnit};
use phalcom_modules::{SourceId, SourceLocation, SourceRevision, WorkspaceSourceBatchMutation};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::types::environment::{TypeEnvironment, TypeView};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::store::TypeData;
use phalcom_semantic::{analyze_single_module, analyze_workspace, SemanticWorkspaceInput, SemanticWorkspaceSession, TypeHierarchy};
use std::collections::BTreeMap;
use std::fs;
use std::sync::Arc;
use tempfile::TempDir;

fn batch_source(path: &str) -> SourceLocation {
    SourceLocation {
        source_id: SourceId(path.into()),
        display_path: path.into(),
    }
}

#[test]
fn semantic_batch_publishes_once_and_retains_workspace_store_and_modules() {
    let left = batch_source("/tmp/phalcom-semantic-batch-left.ph");
    let right = batch_source("/tmp/phalcom-semantic-batch-right.ph");
    let mut session = SemanticWorkspaceSession::new();

    let first = session
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
            source: left.clone(),
            text: Arc::from("class Left { value() {} }\n"),
            revision: SourceRevision(1),
            recovered_program: None,
        }])
        .expect("initial semantic publication");
    let left_module = first.snapshot.module_for_source(&left.source_id).cloned().expect("left module");
    let workspace = first.snapshot.id.workspace();
    let store = first.snapshot.store.id();

    let second = session
        .apply_module_mutations([
            WorkspaceSourceBatchMutation::SetOverlay {
                source: left.clone(),
                text: Arc::from("class Left { value() {} changed() {} }\n"),
                revision: SourceRevision(2),
                recovered_program: None,
            },
            WorkspaceSourceBatchMutation::SetOverlay {
                source: right.clone(),
                text: Arc::from("class Right { value() {} }\n"),
                revision: SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("coalesced semantic publication");

    assert_eq!(second.snapshot.generation, 2);
    assert_eq!(second.snapshot.id.workspace(), workspace);
    assert_eq!(second.snapshot.store.id(), store);
    assert_eq!(session.module_session().generation(), 2);
    assert_eq!(second.snapshot.module_for_source(&left.source_id), Some(&left_module));
    assert!(second.snapshot.module_for_source(&right.source_id).is_some());
}

#[test]
fn single_module_analysis_succeeds() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from("class Point { getX() -> Int { 42 } }");
    let parse_res = phalcom_ast::parse(&source, 0);
    let program = Arc::new(parse_res.program);

    let analysis = analyze_single_module(module.clone(), source, program);
    assert!(!analysis.snapshot.has_errors());
    assert!(analysis.snapshot.sources.contains_key(&module));
    assert!(analysis.snapshot.surfaces.contains_key(&DeclarationId::new(module.clone(), "Point".into())));
}

#[test]
fn generic_declaration_kind_matches_published_signature() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
class Box<T> {}
class Transformer<F: Type -> Type, T> {}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let mut store = (*analysis.snapshot.store).clone();
    for (name, expected_parameter_kinds) in [
        ("Box", vec![KindId::TYPE]),
        ("Transformer", vec![store.arrow_kind(Box::new([KindId::TYPE]), KindId::TYPE), KindId::TYPE]),
    ] {
        let declaration = DeclarationId::new(module.clone(), name.into());
        let info = analysis.snapshot.declarations.get(&declaration).expect("declaration header");
        let signature = info.generic_signature.as_ref().expect("generic signature");
        let parameter_kinds = signature
            .parameters
            .iter()
            .map(|&parameter| analysis.snapshot.store.type_parameter(parameter).kind)
            .collect::<Vec<_>>();
        assert_eq!(parameter_kinds, expected_parameter_kinds);
        let derived_kind = store.arrow_kind(parameter_kinds.into_boxed_slice(), KindId::TYPE);
        assert_eq!(info.kind, derived_kind, "kind mismatch for {name}");
    }
}

#[test]
fn invalid_generic_kind_does_not_publish_ready_declaration_header() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from("class Broken<T: ?> {}\n");
    let parsed = phalcom_ast::parse(&source, 0);
    let phalcom_ast::ast::Statement::Class(class_def) = &parsed.program.statements[0] else {
        panic!("expected class declaration")
    };
    assert!(matches!(
        class_def.generic_parameters[0].kind,
        Some(phalcom_ast::ast::KindSyntax::Invalid { .. })
    ));
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let declaration = DeclarationId::new(module, "Broken".into());
    assert!(analysis.snapshot.declarations.get(&declaration).is_none());
}

#[test]
fn written_invalid_superclass_does_not_publish_ready_declaration() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        "class ConstructorBase<T> {}\nclass ConstructorKind is ConstructorBase {}\nclass MissingBase is MissingBaseType {}\n",
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    assert_eq!(parsed.program.statements.len(), 3, "parsed source statements");
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    assert!(analysis
        .snapshot
        .all_diagnostics()
        .any(|diagnostic| diagnostic.code == phalcom_semantic::diagnostic::DiagnosticCode::KindExpectedType),
        "diagnostics: {:?}, declarations: {:?}",
        analysis.snapshot.diagnostics,
        analysis.snapshot.declarations.iter().map(|(id, _)| id).collect::<Vec<_>>());
    assert!(analysis
        .snapshot
        .all_diagnostics()
        .any(|diagnostic| diagnostic.code == phalcom_semantic::diagnostic::DiagnosticCode::AnnotationUnresolved));
    assert!(analysis
        .snapshot
        .declarations
        .get(&DeclarationId::new(module.clone(), "ConstructorKind".into()))
        .is_none());
    assert!(analysis
        .snapshot
        .surfaces
        .get(&DeclarationId::new(module.clone(), "ConstructorKind".into()))
        .is_none());
    assert!(analysis
        .snapshot
        .declarations
        .get(&DeclarationId::new(module, "MissingBase".into()))
        .is_none());
}

#[test]
fn transparent_aliases_preserve_canonical_forms_and_kinds() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        "type UserId = Int\ntype Pair<T> = (T, T)\ntype ListAlias = List\nclass Uses { id(_ value: UserId) -> UserId { value } }\n",
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    assert!(!analysis.snapshot.has_errors(), "diagnostics: {:?}", analysis.snapshot.diagnostics);

    let user_id = DeclarationId::new(module.clone(), "UserId".into());
    let int = phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Int);
    assert_eq!(analysis.snapshot.type_aliases.get(&user_id).unwrap().kind, KindId::TYPE);
    assert_eq!(analysis.snapshot.type_aliases.form(&user_id), analysis.snapshot.declarations.form(&int));

    let pair = analysis
        .snapshot
        .type_aliases
        .get(&DeclarationId::new(module.clone(), "Pair".into()))
        .unwrap();
    assert_ne!(pair.kind, KindId::TYPE);
    assert!(matches!(analysis.snapshot.store.get(pair.form), TypeData::Lambda(_)));

    let list_alias = analysis
        .snapshot
        .type_aliases
        .get(&DeclarationId::new(module.clone(), "ListAlias".into()))
        .unwrap();
    assert_ne!(list_alias.kind, KindId::TYPE);
    assert_eq!(analysis.snapshot.store.format_kind(list_alias.kind), "(Type) -> Type");
}

/// COMPOSED: an exported class method remains callable through an imported class identity.
#[test]
fn exported_constructor_and_method_feed_importing_client_summary() {
    let project = ResolvedProjectId::from_raw(1);
    let api_module = ModuleId::resolved(project, ModulePath::from_components(vec![ModuleComponent::from_identifier("api").unwrap()]));
    let client_module = ModuleId::resolved(project, ModulePath::from_components(vec![ModuleComponent::from_identifier("client").unwrap()]));
    let api_source: Arc<str> = Arc::from(
        r#"
class Service {
  @constructor new() {}

  @class
  serve() -> Int { 7 }
}
export Service
"#,
    );
    let client_source: Arc<str> = Arc::from(
        r#"
import app.api.Service

class Client {
  @class
  run() { Service.serve() }
}
export Client
"#,
    );
    let api_program = Arc::new(phalcom_ast::parse(&api_source, 0).program);
    let client_program = Arc::new(phalcom_ast::parse(&client_source, 0).program);

    let mut sources = BTreeMap::new();
    sources.insert(
        api_module.clone(),
        Arc::new(ParsedModuleUnit::new(api_module.clone(), ModuleKind::Module, None, api_source, api_program)),
    );
    sources.insert(
        client_module.clone(),
        Arc::new(ParsedModuleUnit::new(
            client_module.clone(),
            ModuleKind::Module,
            None,
            client_source.clone(),
            client_program,
        )),
    );

    let service_symbol = SymbolId {
        module: api_module.clone(),
        name: "Service".into(),
    };
    let client_symbol = SymbolId {
        module: client_module.clone(),
        name: "Client".into(),
    };
    let service_export = LinkedExport {
        public_name: "Service".into(),
        target: LinkedExportTarget::Binding(service_symbol.clone()),
        range: Default::default(),
    };
    let client_export = LinkedExport {
        public_name: "Client".into(),
        target: LinkedExportTarget::Binding(client_symbol),
        range: Default::default(),
    };
    let mut modules = BTreeMap::new();
    modules.insert(
        api_module.clone(),
        LinkedModule {
            interface: LinkedModuleInterface {
                module: api_module.clone(),
                kind: ModuleKind::Module,
                exports: BTreeMap::from([("Service".into(), service_export)]),
                metadata: ModuleMetadata::default(),
            },
            bindings: ModuleBindingLayout {
                local_globals: BTreeMap::from([("Service".into(), GlobalBindingId(0))]),
                imports: BTreeMap::new(),
            },
            linked_reads: Vec::new(),
            runtime_dependencies: Vec::new(),
        },
    );
    modules.insert(
        client_module.clone(),
        LinkedModule {
            interface: LinkedModuleInterface {
                module: client_module.clone(),
                kind: ModuleKind::Module,
                exports: BTreeMap::from([("Client".into(), client_export)]),
                metadata: ModuleMetadata::default(),
            },
            bindings: ModuleBindingLayout {
                local_globals: BTreeMap::from([("Client".into(), GlobalBindingId(0))]),
                imports: BTreeMap::from([("Service".into(), ImportBindingId(0))]),
            },
            linked_reads: vec![LinkedReadSpec::Binding(service_symbol)],
            runtime_dependencies: vec![api_module.clone()],
        },
    );

    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules,
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: client_module.clone(),
        initialization_order: vec![api_module.clone(), client_module.clone()],
    });
    let analysis = analyze_workspace(SemanticWorkspaceInput {
        linked,
        sources,
        generation: 1,
    });

    assert!(!analysis.snapshot.has_errors(), "diagnostics: {:#?}", analysis.snapshot.diagnostics);
    let client_decl = DeclarationId::new(client_module.clone(), "Client".into());
    let run_id = CallableId::new(client_decl, Selector::method("run", Vec::new()).unwrap(), DispatchSide::Class);
    let run = analysis.snapshot.callable_analyses.get(&run_id).expect("Client.run analysis");
    let serve = run
        .expressions
        .values()
        .find(|expression| client_source.get(expression.range.start..expression.range.end) == Some("Service.serve()"))
        .expect("imported Service.serve() expression");
    let int_ty = analysis
        .snapshot
        .declarations
        .form(&phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Int))
        .expect("Int type");
    assert_eq!(serve.knowledge.ty(), Some(int_ty));
    assert_eq!(
        serve.callable.as_ref(),
        Some(&CallableId::new(
            DeclarationId::new(api_module.clone(), "Service".into()),
            Selector::method("serve", Vec::new()).unwrap(),
            DispatchSide::Class,
        ))
    );
    assert!(run.dependencies.iter().any(|dependency| dependency.owner.module == api_module));
    assert!(
        run.semantic_dependencies.iter().any(|dependency| {
            matches!(
                dependency,
                phalcom_semantic::checker::analysis::SemanticDependency::CallableSignature(callable)
                    if callable.owner.module == api_module
            )
        }),
        "client must record API callable-signature dependency: {run:#?}"
    );
    assert!(
        run.semantic_dependencies.iter().any(|dependency| {
            matches!(
                dependency,
                phalcom_semantic::checker::analysis::SemanticDependency::LinkedInterface(module)
                    if module == &client_module
            )
        }),
        "client must record its linked interface dependency: {run:#?}"
    );
}

#[test]
fn workspace_multi_module_linking_resolution_and_cycles() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    let proj_dir = root.join("app");
    fs::create_dir_all(proj_dir.join("src/shapes")).unwrap();
    fs::write(
        proj_dir.join("project.toml"),
        "[project]\nname = \"app\"\nnamespace = \"app\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(proj_dir.join("src/package.ph"), "expose .shapes\n").unwrap();
    fs::write(proj_dir.join("src/point.ph"), "class Point { get() -> Int { 1 } }\nexport Point\n").unwrap();
    fs::write(
        proj_dir.join("src/shapes/circle.ph"),
        "import app.point.Point\nclass Circle is Point { radius() -> Int { 5 } }\nexport Circle\n",
    )
    .unwrap();

    let mut universe = ProjectUniverse::new();
    let root_id = universe.load_root(proj_dir.join("project.toml")).expect("universe load succeeds");

    let point_mod = ModuleId::resolved(root_id, ModulePath::from_components(vec![ModuleComponent::from_identifier("point").unwrap()]));
    let circle_mod = ModuleId::resolved(
        root_id,
        ModulePath::from_components(vec![
            ModuleComponent::from_identifier("shapes").unwrap(),
            ModuleComponent::from_identifier("circle").unwrap(),
        ]),
    );

    let point_src: Arc<str> = Arc::from(fs::read_to_string(proj_dir.join("src/point.ph")).unwrap());
    let circle_src: Arc<str> = Arc::from(fs::read_to_string(proj_dir.join("src/shapes/circle.ph")).unwrap());

    let point_prog = Arc::new(phalcom_ast::parse(&point_src, 0).program);
    let circle_prog = Arc::new(phalcom_ast::parse(&circle_src, 0).program);

    let mut sources = BTreeMap::new();
    sources.insert(
        point_mod.clone(),
        Arc::new(ParsedModuleUnit::new(point_mod.clone(), ModuleKind::Module, None, point_src, point_prog)),
    );
    sources.insert(
        circle_mod.clone(),
        Arc::new(ParsedModuleUnit::new(circle_mod.clone(), ModuleKind::Module, None, circle_src, circle_prog)),
    );

    let mut modules = BTreeMap::new();
    let mut point_exports = BTreeMap::new();
    point_exports.insert(
        "Point".into(),
        LinkedExport {
            public_name: "Point".into(),
            target: LinkedExportTarget::Binding(SymbolId {
                module: point_mod.clone(),
                name: "Point".into(),
            }),
            range: phalcom_common::range::SourceRange::default(),
        },
    );
    modules.insert(
        point_mod.clone(),
        LinkedModule {
            interface: LinkedModuleInterface {
                module: point_mod.clone(),
                kind: ModuleKind::Module,
                exports: point_exports,
                metadata: ModuleMetadata::default(),
            },
            bindings: ModuleBindingLayout {
                local_globals: BTreeMap::from([("Point".into(), GlobalBindingId(0))]),
                imports: BTreeMap::new(),
            },
            linked_reads: Vec::new(),
            runtime_dependencies: Vec::new(),
        },
    );

    let mut circle_imports = BTreeMap::new();
    circle_imports.insert("Point".into(), ImportBindingId(0));
    let mut circle_exports = BTreeMap::new();
    circle_exports.insert(
        "Circle".into(),
        LinkedExport {
            public_name: "Circle".into(),
            target: LinkedExportTarget::Binding(SymbolId {
                module: circle_mod.clone(),
                name: "Circle".into(),
            }),
            range: phalcom_common::range::SourceRange::default(),
        },
    );
    modules.insert(
        circle_mod.clone(),
        LinkedModule {
            interface: LinkedModuleInterface {
                module: circle_mod.clone(),
                kind: ModuleKind::Module,
                exports: circle_exports,
                metadata: ModuleMetadata::default(),
            },
            bindings: ModuleBindingLayout {
                local_globals: BTreeMap::from([("Circle".into(), GlobalBindingId(0))]),
                imports: circle_imports,
            },
            linked_reads: vec![LinkedReadSpec::Binding(SymbolId {
                module: point_mod.clone(),
                name: "Point".into(),
            })],
            runtime_dependencies: vec![point_mod.clone()],
        },
    );

    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(universe),
        modules,
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: circle_mod.clone(),
        initialization_order: vec![point_mod.clone(), circle_mod.clone()],
    });

    let analysis = analyze_workspace(SemanticWorkspaceInput {
        linked,
        sources,
        generation: 1,
    });

    assert!(!analysis.snapshot.has_errors());

    // 1. Cross-module superclass resolved
    let circle_decl = DeclarationId::new(circle_mod.clone(), "Circle".into());
    let point_decl = DeclarationId::new(point_mod.clone(), "Point".into());
    assert_eq!(analysis.snapshot.hierarchy.superclass(&circle_decl), Some(&point_decl));

    // 2. Point in both modules resolves canonically and has same TypeId
    let point_ty1 = analysis.snapshot.declarations.form(&point_decl).unwrap();
    assert_eq!(KindId::TYPE, analysis.snapshot.store.kind_of(point_ty1));
}

#[test]
fn inheritance_cycle_is_rejected_in_workspace() {
    let universe = ProjectUniverse::new();
    let mod_a = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("a").unwrap()]),
    );
    let mod_b = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("b").unwrap()]),
    );

    let a_src: Arc<str> = Arc::from("import b.B\nclass A is B {}\nexport A\n");
    let b_src: Arc<str> = Arc::from("import a.A\nclass B is A {}\nexport B\n");

    let a_prog = Arc::new(phalcom_ast::parse(&a_src, 0).program);
    let b_prog = Arc::new(phalcom_ast::parse(&b_src, 0).program);

    let mut sources = BTreeMap::new();
    sources.insert(
        mod_a.clone(),
        Arc::new(ParsedModuleUnit::new(mod_a.clone(), ModuleKind::Module, None, a_src, a_prog)),
    );
    sources.insert(
        mod_b.clone(),
        Arc::new(ParsedModuleUnit::new(mod_b.clone(), ModuleKind::Module, None, b_src, b_prog)),
    );

    let mut modules = BTreeMap::new();
    modules.insert(
        mod_a.clone(),
        LinkedModule {
            interface: LinkedModuleInterface {
                module: mod_a.clone(),
                kind: ModuleKind::Module,
                exports: BTreeMap::from([(
                    "A".into(),
                    LinkedExport {
                        public_name: "A".into(),
                        target: LinkedExportTarget::Binding(SymbolId {
                            module: mod_a.clone(),
                            name: "A".into(),
                        }),
                        range: phalcom_common::range::SourceRange::default(),
                    },
                )]),
                metadata: ModuleMetadata::default(),
            },
            bindings: ModuleBindingLayout {
                local_globals: BTreeMap::from([("A".into(), GlobalBindingId(0))]),
                imports: BTreeMap::from([("B".into(), ImportBindingId(0))]),
            },
            linked_reads: vec![LinkedReadSpec::Binding(SymbolId {
                module: mod_b.clone(),
                name: "B".into(),
            })],
            runtime_dependencies: vec![mod_b.clone()],
        },
    );

    modules.insert(
        mod_b.clone(),
        LinkedModule {
            interface: LinkedModuleInterface {
                module: mod_b.clone(),
                kind: ModuleKind::Module,
                exports: BTreeMap::from([(
                    "B".into(),
                    LinkedExport {
                        public_name: "B".into(),
                        target: LinkedExportTarget::Binding(SymbolId {
                            module: mod_b.clone(),
                            name: "B".into(),
                        }),
                        range: phalcom_common::range::SourceRange::default(),
                    },
                )]),
                metadata: ModuleMetadata::default(),
            },
            bindings: ModuleBindingLayout {
                local_globals: BTreeMap::from([("B".into(), GlobalBindingId(0))]),
                imports: BTreeMap::from([("A".into(), ImportBindingId(0))]),
            },
            linked_reads: vec![LinkedReadSpec::Binding(SymbolId {
                module: mod_a.clone(),
                name: "A".into(),
            })],
            runtime_dependencies: vec![mod_a.clone()],
        },
    );

    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(universe),
        modules,
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: mod_a.clone(),
        initialization_order: vec![mod_a.clone(), mod_b.clone()],
    });

    let analysis = analyze_workspace(SemanticWorkspaceInput {
        linked,
        sources,
        generation: 1,
    });

    assert!(analysis.snapshot.has_errors(), "inheritance cycle must be detected and rejected");
}

#[test]
fn same_leaf_name_in_two_modules_stays_distinct() {
    let mod_x = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("x").unwrap()]),
    );
    let mod_y = ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("y").unwrap()]),
    );

    let x_src: Arc<str> = Arc::from("class Item { xVal() -> Int { 1 } }\n");
    let y_src: Arc<str> = Arc::from("class Item { yVal() -> String { \"y\" } }\n");

    let x_prog = Arc::new(phalcom_ast::parse(&x_src, 0).program);
    let y_prog = Arc::new(phalcom_ast::parse(&y_src, 0).program);

    let mut sources = BTreeMap::new();
    sources.insert(
        mod_x.clone(),
        Arc::new(ParsedModuleUnit::new(mod_x.clone(), ModuleKind::Module, None, x_src, x_prog)),
    );
    sources.insert(
        mod_y.clone(),
        Arc::new(ParsedModuleUnit::new(mod_y.clone(), ModuleKind::Module, None, y_src, y_prog)),
    );

    let mut modules = BTreeMap::new();
    modules.insert(
        mod_x.clone(),
        LinkedModule {
            interface: LinkedModuleInterface {
                module: mod_x.clone(),
                kind: ModuleKind::Module,
                exports: BTreeMap::new(),
                metadata: ModuleMetadata::default(),
            },
            bindings: ModuleBindingLayout::default(),
            linked_reads: Vec::new(),
            runtime_dependencies: Vec::new(),
        },
    );
    modules.insert(
        mod_y.clone(),
        LinkedModule {
            interface: LinkedModuleInterface {
                module: mod_y.clone(),
                kind: ModuleKind::Module,
                exports: BTreeMap::new(),
                metadata: ModuleMetadata::default(),
            },
            bindings: ModuleBindingLayout::default(),
            linked_reads: Vec::new(),
            runtime_dependencies: Vec::new(),
        },
    );

    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(ProjectUniverse::new()),
        modules,
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: mod_x.clone(),
        initialization_order: vec![mod_x.clone(), mod_y.clone()],
    });

    let analysis = analyze_workspace(SemanticWorkspaceInput {
        linked,
        sources,
        generation: 1,
    });

    let decl_x = DeclarationId::new(mod_x, "Item".into());
    let decl_y = DeclarationId::new(mod_y, "Item".into());

    let form_x = analysis.snapshot.declarations.form(&decl_x).unwrap();
    let form_y = analysis.snapshot.declarations.form(&decl_y).unwrap();

    assert_ne!(form_x, form_y, "declarations in different modules must have distinct TypeIds");
    assert!(analysis.snapshot.surfaces.contains_key(&decl_x));
    assert!(analysis.snapshot.surfaces.contains_key(&decl_y));
}

#[test]
fn generation_retains_clean_snapshot_and_removes_stale_declarations() {
    let module = ModuleId::universe_root();
    let source_v1: Arc<str> = Arc::from("class OldName { val() -> Int { 1 } }");
    let analysis_v1 = analyze_single_module(
        module.clone(),
        source_v1,
        Arc::new(phalcom_ast::parse("class OldName { val() -> Int { 1 } }", 0).program),
    );

    assert!(analysis_v1
        .snapshot
        .surfaces
        .contains_key(&DeclarationId::new(module.clone(), "OldName".into())));

    let source_v2: Arc<str> = Arc::from("class NewName { val() -> Int { 2 } }");
    let analysis_v2 = analyze_single_module(
        module.clone(),
        source_v2,
        Arc::new(phalcom_ast::parse("class NewName { val() -> Int { 2 } }", 0).program),
    );

    assert!(analysis_v2
        .snapshot
        .surfaces
        .contains_key(&DeclarationId::new(module.clone(), "NewName".into())));
    assert!(!analysis_v2
        .snapshot
        .surfaces
        .contains_key(&DeclarationId::new(module.clone(), "OldName".into())));
}

#[test]
fn deterministic_fresh_store_analysis_matches_structurally() {
    use phalcom_semantic::export::{export_kind, export_type_form};

    let make_input = |generation| {
        let mod_a = ModuleId::resolved(
            ResolvedProjectId::from_raw(1),
            ModulePath::from_components(vec![ModuleComponent::from_identifier("a").unwrap()]),
        );
        let mod_b = ModuleId::resolved(
            ResolvedProjectId::from_raw(1),
            ModulePath::from_components(vec![ModuleComponent::from_identifier("b").unwrap()]),
        );

        let a_src: Arc<str> = Arc::from("class Base { val() -> Int { 10 } }\nexport Base\n");
        let b_src: Arc<str> = Arc::from("import a.Base\nclass Sub is Base { val() -> String { \"mismatch\" } }\nexport Sub\n");

        let a_prog = Arc::new(phalcom_ast::parse(&a_src, 0).program);
        let b_prog = Arc::new(phalcom_ast::parse(&b_src, 0).program);

        let mut sources = BTreeMap::new();
        sources.insert(
            mod_a.clone(),
            Arc::new(ParsedModuleUnit::new(mod_a.clone(), ModuleKind::Module, None, a_src, a_prog)),
        );
        sources.insert(
            mod_b.clone(),
            Arc::new(ParsedModuleUnit::new(mod_b.clone(), ModuleKind::Module, None, b_src, b_prog)),
        );

        let mut modules = BTreeMap::new();
        modules.insert(
            mod_a.clone(),
            LinkedModule {
                interface: LinkedModuleInterface {
                    module: mod_a.clone(),
                    kind: ModuleKind::Module,
                    exports: BTreeMap::from([(
                        "Base".into(),
                        LinkedExport {
                            public_name: "Base".into(),
                            target: LinkedExportTarget::Binding(SymbolId {
                                module: mod_a.clone(),
                                name: "Base".into(),
                            }),
                            range: phalcom_common::range::SourceRange::default(),
                        },
                    )]),
                    metadata: ModuleMetadata::default(),
                },
                bindings: ModuleBindingLayout {
                    local_globals: BTreeMap::from([("Base".into(), GlobalBindingId(0))]),
                    imports: BTreeMap::new(),
                },
                linked_reads: Vec::new(),
                runtime_dependencies: Vec::new(),
            },
        );

        modules.insert(
            mod_b.clone(),
            LinkedModule {
                interface: LinkedModuleInterface {
                    module: mod_b.clone(),
                    kind: ModuleKind::Module,
                    exports: BTreeMap::from([(
                        "Sub".into(),
                        LinkedExport {
                            public_name: "Sub".into(),
                            target: LinkedExportTarget::Binding(SymbolId {
                                module: mod_b.clone(),
                                name: "Sub".into(),
                            }),
                            range: phalcom_common::range::SourceRange::default(),
                        },
                    )]),
                    metadata: ModuleMetadata::default(),
                },
                bindings: ModuleBindingLayout {
                    local_globals: BTreeMap::from([("Sub".into(), GlobalBindingId(0))]),
                    imports: BTreeMap::from([("Base".into(), ImportBindingId(0))]),
                },
                linked_reads: vec![LinkedReadSpec::Binding(SymbolId {
                    module: mod_a.clone(),
                    name: "Base".into(),
                })],
                runtime_dependencies: vec![mod_a.clone()],
            },
        );

        let linked = Arc::new(LinkedProgram {
            universe: Arc::new(ProjectUniverse::new()),
            modules,
            graphs: phalcom_modules::graph::ModuleGraphs::default(),
            entry: mod_b.clone(),
            initialization_order: vec![mod_a.clone(), mod_b.clone()],
        });

        SemanticWorkspaceInput { linked, sources, generation }
    };

    let run1 = analyze_workspace(make_input(1));
    let run2 = analyze_workspace(make_input(2));

    // Compare diagnostics: module ordering, codes, ranges, severities
    let diag_keys1: Vec<_> = run1.snapshot.diagnostics.keys().collect();
    let diag_keys2: Vec<_> = run2.snapshot.diagnostics.keys().collect();
    assert_eq!(diag_keys1, diag_keys2);

    for (k1, diags1) in run1.snapshot.diagnostics.iter() {
        let diags2 = run2.snapshot.diagnostics.get(k1).expect("matching module key");
        assert_eq!(diags1.len(), diags2.len());
        for (d1, d2) in diags1.iter().zip(diags2.iter()) {
            assert_eq!(d1.code, d2.code);
            assert_eq!(d1.severity, d2.severity);
            assert_eq!(d1.primary_range, d2.primary_range);
            assert_eq!(d1.message, d2.message);
        }
    }

    // Compare structural export descriptors of all declared forms
    for (decl_id, info1) in run1.snapshot.declarations.iter() {
        let ty2 = run2.snapshot.declarations.form(decl_id).expect("matching decl");
        let exp1 = export_type_form(&run1.snapshot.store, info1.form).expect("valid export 1");
        let exp2 = export_type_form(&run2.snapshot.store, ty2).expect("valid export 2");
        assert_eq!(exp1, exp2);

        let kind1 = export_kind(&run1.snapshot.store, run1.snapshot.store.kind_of(info1.form));
        let kind2 = export_kind(&run2.snapshot.store, run2.snapshot.store.kind_of(ty2));
        assert_eq!(kind1, kind2);
    }
}

#[test]
fn workspace_generic_class_and_callable_signature_publication() {
    let module = ModuleId::universe_root();
    let source: Arc<str> =
        Arc::from("class Container<T> {\n  value(_ v: T) -> T { v }\n}\nclass Box<U> is Container<U> {\n  unbox() -> U { self.value(1) }\n}\n");
    let parse_res = phalcom_ast::parse(&source, 0);
    let program = Arc::new(parse_res.program);

    let analysis = analyze_single_module(module.clone(), source, program);
    assert!(!analysis.snapshot.has_errors(), "diagnostics: {:?}", analysis.snapshot.diagnostics);

    let container_id = DeclarationId::new(module.clone(), "Container".into());
    let container_info = analysis.snapshot.declarations.get(&container_id).expect("Container declaration info");
    assert_ne!(container_info.kind, KindId::TYPE, "generic class Container must have arrow kind");
    assert!(container_info.generic_signature.is_some(), "Container must have generic signature");
    let container_sig = container_info.generic_signature.as_ref().unwrap();
    assert_eq!(container_sig.parameters.len(), 1);

    let box_id = DeclarationId::new(module.clone(), "Box".into());
    let box_info = analysis.snapshot.declarations.get(&box_id).expect("Box declaration info");
    assert_ne!(box_info.kind, KindId::TYPE);
    assert!(box_info.supertype_template.is_some(), "Box must have supertype template");

    let box_parameter = box_info.generic_signature.as_ref().unwrap().parameters[0];
    let template = box_info.supertype_template.as_ref().unwrap();
    let TypeData::Applied { origin, arguments } = analysis.snapshot.store.get(template.supertype) else {
        panic!("Box superclass should be applied Container<U>");
    };
    assert_eq!(*origin, container_info.form);
    assert!(matches!(analysis.snapshot.store.get(arguments[0]), TypeData::Parameter(parameter) if *parameter == box_parameter));
    let int = analysis
        .snapshot
        .declarations
        .form(&phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Int))
        .expect("Int declaration form");
    let mut environment = TypeEnvironment::new();
    environment.bind_param(box_parameter, int);
    let mut store = (*analysis.snapshot.store).clone();
    let specialized = TypeView::new(template.supertype, environment).materialize(&mut store);
    let TypeData::Applied { arguments, .. } = store.get(specialized) else {
        panic!("specialized superclass should remain applied");
    };
    assert_eq!(arguments[0], int);

    let sel = phalcom_common::selector::Selector::method("value", vec![phalcom_common::selector::SelectorSlot::Positional]).unwrap();
    let callable_id = phalcom_semantic::identity::CallableId::new(container_id, sel, phalcom_semantic::identity::DispatchSide::Instance);
    let callable_sig = analysis
        .snapshot
        .callable_signatures
        .get(&callable_id)
        .expect("value(_) callable signature published");
    assert_eq!(callable_sig.parameters.len(), 1);
}

#[test]
fn workspace_constructor_and_cross_module_dispatch_inference() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    let proj_dir = root.join("app");
    fs::create_dir_all(proj_dir.join("src/domain")).unwrap();
    fs::create_dir_all(proj_dir.join("src/service")).unwrap();
    fs::write(
        proj_dir.join("project.toml"),
        "[project]\nname = \"app\"\nnamespace = \"app\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    fs::write(proj_dir.join("src/package.ph"), "expose .domain\nexpose .service\nexpose .main\n").unwrap();
    fs::write(
        proj_dir.join("src/domain/package.ph"),
        "expose .point\nexpose .weight\nexpose .shipment\nexpose .parcel\n",
    )
    .unwrap();
    fs::write(proj_dir.join("src/service/package.ph"), "expose .planner\n").unwrap();

    fs::write(
        proj_dir.join("src/domain/point.ph"),
        "class Point {\n  _x: Int = 0\n  _y: Int = 0\n  @constructor\n  new(_ x: Int, y: Int) {\n    _x = x\n    _y = y\n  }\n}\nexport Point\n",
    )
    .unwrap();

    fs::write(
        proj_dir.join("src/domain/weight.ph"),
        "class Weight {\n  _units: Int = 0\n  @constructor\n  new(_ units: Int) {\n    _units = units\n  }\n}\nexport Weight\n",
    )
    .unwrap();

    fs::write(
        proj_dir.join("src/domain/shipment.ph"),
        "class Shipment {\n  @constructor\n  new() {}\n}\nexport Shipment\n",
    )
    .unwrap();

    fs::write(
        proj_dir.join("src/domain/parcel.ph"),
        "from .point import Point\nfrom .weight import Weight\nclass Parcel {\n  _id: String\n  _destination: Point\n  _weight: Weight\n  @constructor\n  new(_ id: String, destination: Point, weight: Weight) -> () {\n    _id = id\n    _destination = destination\n    _weight = weight\n  }\n}\nexport Parcel\n",
    )
    .unwrap();

    fs::write(
        proj_dir.join("src/service/planner.ph"),
        "from ..domain.parcel import Parcel\nfrom ..domain.shipment import Shipment\nfrom ..domain.point import Point\nclass Planner {\n  @class\n  plan(_ parcel: Parcel, origin: Point) -> Shipment {\n    Shipment.new()\n  }\n}\nexport Planner\n",
    )
    .unwrap();

    fs::write(
        proj_dir.join("src/main.ph"),
        "from .domain.point import Point\nfrom .domain.weight import Weight\nfrom .domain.parcel import Parcel\nfrom .service.planner import Planner\nclass Main {\n  @class\n  main {\n    const origin = Point.new(0, y: 0)\n    const destination = Point.new(3, y: 4)\n    const parcel = Parcel.new(\"PKG-001\", destination: destination, weight: Weight.new(12))\n    const shipment = Planner.plan(parcel, origin: origin)\n  }\n}\n",
    )
    .unwrap();

    let mut universe = ProjectUniverse::new();
    let root_id = universe.load_root(proj_dir.join("project.toml")).expect("universe load succeeds");

    let mut sources = BTreeMap::new();
    let mut interfaces = BTreeMap::new();
    let file_map = [
        ("domain/point", proj_dir.join("src/domain/point.ph"), ModuleKind::Module),
        ("domain/weight", proj_dir.join("src/domain/weight.ph"), ModuleKind::Module),
        ("domain/shipment", proj_dir.join("src/domain/shipment.ph"), ModuleKind::Module),
        ("domain/parcel", proj_dir.join("src/domain/parcel.ph"), ModuleKind::Module),
        ("service/planner", proj_dir.join("src/service/planner.ph"), ModuleKind::Module),
        ("main", proj_dir.join("src/main.ph"), ModuleKind::Module),
        ("", proj_dir.join("src/package.ph"), ModuleKind::Package),
        ("domain", proj_dir.join("src/domain/package.ph"), ModuleKind::Package),
        ("service", proj_dir.join("src/service/package.ph"), ModuleKind::Package),
    ];

    for (mod_name, path, kind) in file_map {
        let components: Vec<ModuleComponent> = if mod_name.is_empty() {
            Vec::new()
        } else {
            mod_name.split('/').map(|s| ModuleComponent::from_identifier(s).unwrap()).collect()
        };
        let mod_id = ModuleId::resolved(root_id, ModulePath::from_components(components));
        let src: Arc<str> = Arc::from(fs::read_to_string(&path).unwrap());
        let prog = Arc::new(phalcom_ast::parse(&src, 0).program);
        sources.insert(mod_id.clone(), Arc::new(ParsedModuleUnit::new(mod_id.clone(), kind, None, src, prog.clone())));
        let iface = phalcom_modules::InterfaceBuilder::build(mod_id.clone(), kind, &prog).unwrap();
        interfaces.insert(mod_id.clone(), iface);
    }

    let provider = phalcom_modules::FilesystemSourceProvider::new();
    let mut resolver = phalcom_modules::ModuleResolver::new(&universe, &provider);
    let mut resolved = BTreeMap::new();
    for (mod_id, iface) in &interfaces {
        for import in &iface.imports {
            let path = match import {
                phalcom_modules::ImportSurface::Module(m) => &m.path,
                phalcom_modules::ImportSurface::Selective(s) => &s.path,
                phalcom_modules::ImportSurface::ReExport(r) => &r.path,
            };
            if let Ok(target) = resolver.resolve_import(mod_id, path) {
                resolved.insert((mod_id.clone(), path.to_string()), target.id);
            }
        }
    }

    let main_mod = ModuleId::resolved(root_id, ModulePath::from_components(vec![ModuleComponent::from_identifier("main").unwrap()]));
    let linker = phalcom_modules::ModuleLinker::new(Arc::new(universe), interfaces);
    let linked = Arc::new(linker.link(main_mod.clone(), &resolved).unwrap());

    let analysis = analyze_workspace(SemanticWorkspaceInput {
        linked,
        sources,
        generation: 1,
    });

    assert!(!analysis.snapshot.has_errors(), "diagnostics: {:?}", analysis.snapshot.diagnostics);

    // Find Main.main callable analysis
    let main_decl = DeclarationId::new(main_mod.clone(), "Main".into());
    let main_sel = phalcom_common::selector::Selector::getter("main").unwrap();
    let main_cid = phalcom_semantic::identity::CallableId::new(main_decl, main_sel, phalcom_semantic::identity::DispatchSide::Class);

    let main_analysis = analysis.snapshot.callable_analyses.get(&main_cid).expect("Main.main analysis must exist");
    assert_eq!(main_analysis.status, phalcom_semantic::checker::CallableAnalysisStatus::Complete);

    // Verify formal types for the local bindings in Main.main
    let point_decl = DeclarationId::new(
        ModuleId::resolved(
            root_id,
            ModulePath::from_components(vec![
                ModuleComponent::from_identifier("domain").unwrap(),
                ModuleComponent::from_identifier("point").unwrap(),
            ]),
        ),
        "Point".into(),
    );
    let parcel_decl = DeclarationId::new(
        ModuleId::resolved(
            root_id,
            ModulePath::from_components(vec![
                ModuleComponent::from_identifier("domain").unwrap(),
                ModuleComponent::from_identifier("parcel").unwrap(),
            ]),
        ),
        "Parcel".into(),
    );
    let shipment_decl = DeclarationId::new(
        ModuleId::resolved(
            root_id,
            ModulePath::from_components(vec![
                ModuleComponent::from_identifier("domain").unwrap(),
                ModuleComponent::from_identifier("shipment").unwrap(),
            ]),
        ),
        "Shipment".into(),
    );

    let point_form = analysis.snapshot.declarations.form(&point_decl).expect("Point form");
    let parcel_form = analysis.snapshot.declarations.form(&parcel_decl).expect("Parcel form");
    let shipment_form = analysis.snapshot.declarations.form(&shipment_decl).expect("Shipment form");

    let origin_binding = main_analysis.bindings.values().find(|b| b.name == "origin").expect("origin binding");
    let destination_binding = main_analysis.bindings.values().find(|b| b.name == "destination").expect("destination binding");
    let parcel_binding = main_analysis.bindings.values().find(|b| b.name == "parcel").expect("parcel binding");
    let shipment_binding = main_analysis.bindings.values().find(|b| b.name == "shipment").expect("shipment binding");

    // All bindings must have distinct identities
    assert_ne!(origin_binding.binding, destination_binding.binding);
    assert_ne!(origin_binding.binding, parcel_binding.binding);
    assert_ne!(origin_binding.binding, shipment_binding.binding);
    assert_ne!(parcel_binding.binding, shipment_binding.binding);

    // Inferred formal types
    assert_eq!(origin_binding.current.ty(), Some(point_form), "origin must be formally known as Point");
    assert_eq!(
        destination_binding.current.ty(),
        Some(point_form),
        "destination must be formally known as Point"
    );
    assert_eq!(parcel_binding.current.ty(), Some(parcel_form), "parcel must be formally known as Parcel");
    assert_eq!(
        shipment_binding.current.ty(),
        Some(shipment_form),
        "shipment must be formally known as Shipment"
    );
}

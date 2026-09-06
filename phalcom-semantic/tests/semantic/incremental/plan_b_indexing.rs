//! Incremental indexing, reference publication, formal projection retention,
//! and workspace-symbol verification for Phalcom LSP Module Architecture Plan B (PB-1 through PB-12).

use super::support::multi_module_input;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::{LinkedExport, LinkedExportTarget, LinkedModuleInterface};
use phalcom_modules::linker::{GlobalBindingId, ImportBindingId, LinkedModule, LinkedProgram, LinkedReadSpec, ModuleBindingLayout, SymbolId};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::editor::ReferenceDomain;
use phalcom_semantic::identity::{DeclarationId, SemanticTargetId};
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::source::ParsedModuleUnit;
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use std::collections::BTreeMap;
use std::sync::Arc;

fn module(name: &str) -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(808),
        ModulePath::from_components(vec![ModuleComponent::from_identifier(name).unwrap()]),
    )
}

fn two_module_input(
    provider_source: &str,
    consumer_source: &str,
    has_alias: bool,
    generation: u64,
) -> SemanticWorkspaceInput {
    let provider = module("provider");
    let consumer = module("consumer");
    let mut sources = BTreeMap::new();
    let p_src: Arc<str> = Arc::from(provider_source.to_owned());
    let c_src: Arc<str> = Arc::from(consumer_source.to_owned());
    let p_prog = Arc::new(phalcom_ast::parse(&p_src, 0).program);
    let c_prog = Arc::new(phalcom_ast::parse(&c_src, 0).program);

    sources.insert(
        provider.clone(),
        Arc::new(ParsedModuleUnit::new(provider.clone(), ModuleKind::Module, None, p_src, p_prog)),
    );
    sources.insert(
        consumer.clone(),
        Arc::new(ParsedModuleUnit::new(consumer.clone(), ModuleKind::Module, None, c_src, c_prog)),
    );

    let export_foo = LinkedExport {
        public_name: "Foo".into(),
        target: LinkedExportTarget::Binding(SymbolId {
            module: provider.clone(),
            name: "Foo".into(),
        }),
        range: phalcom_common::range::SourceRange::default(),
    };

    let p_mod = LinkedModule {
        interface: LinkedModuleInterface {
            module: provider.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::from([("Foo".into(), export_foo)]),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: BTreeMap::from([("Foo".into(), GlobalBindingId(0))]),
            imports: BTreeMap::new(),
        },
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };

    let imported_name = if has_alias { "Bar" } else { "Foo" };
    let c_mod = LinkedModule {
        interface: LinkedModuleInterface {
            module: consumer.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::new(),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: BTreeMap::from([("Consumer".into(), GlobalBindingId(0))]),
            imports: BTreeMap::from([(imported_name.into(), ImportBindingId(0))]),
        },
        linked_reads: vec![LinkedReadSpec::Binding(SymbolId {
            module: provider.clone(),
            name: "Foo".into(),
        })],
        runtime_dependencies: vec![provider.clone()],
    };

    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
        modules: BTreeMap::from([(provider.clone(), p_mod), (consumer.clone(), c_mod)]),
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: consumer,
        initialization_order: vec![provider, module("consumer")],
    });

    SemanticWorkspaceInput::new(linked, sources, generation)
}

// -----------------------------------------------------------------------------
// PB-1 — Presentation-only movement
// -----------------------------------------------------------------------------
#[test]
fn pb_1_presentation_only_movement() {
    let app = module("app");
    let unrelated = module("unrelated");
    let mut session = SemanticWorkspaceSession::new();

    let initial = session.update(multi_module_input(
        vec![
            (app.clone(), "class App {\n  run() -> Int { 42 }\n}\n".into()),
            (unrelated.clone(), "class Other {\n  value() -> Int { 1 }\n}\n".into()),
        ],
        1,
    ));

    let app_initial_source_shard = initial.snapshot.source_index().module_arc(&app).unwrap();
    let unrelated_initial_source_shard = initial.snapshot.source_index().module_arc(&unrelated).unwrap();
    let initial_app_fps = app_initial_source_shard.fingerprints();

    // Mutate with leading comments and whitespace, meaning unchanged
    let updated = session.update(multi_module_input(
        vec![
            (app.clone(), "// Leading comment\n\nclass App {\n  run() -> Int { 42 }\n}\n".into()),
            (unrelated.clone(), "class Other {\n  value() -> Int { 1 }\n}\n".into()),
        ],
        2,
    ));

    let updated_app_shard = updated.snapshot.source_index().module_arc(&app).unwrap();
    let updated_app_fps = updated_app_shard.fingerprints();

    // Semantic fingerprint is unchanged, presentation fingerprint changed
    assert_eq!(initial_app_fps.semantic, updated_app_fps.semantic);
    assert_ne!(initial_app_fps.presentation, updated_app_fps.presentation);

    // Unrelated source index shard Arc retained
    let updated_unrelated_shard = updated.snapshot.source_index().module_arc(&unrelated).unwrap();
    assert!(Arc::ptr_eq(&unrelated_initial_source_shard, &updated_unrelated_shard));

    // App source shard updated
    assert!(!Arc::ptr_eq(&app_initial_source_shard, &updated_app_shard));

    // Ranges updated for App declaration
    let app_decl = updated.snapshot.source_index().declaration_source(&DeclarationId::new(app.clone(), "App".into())).unwrap();
    assert!(app_decl.declaration_range.start > 0);

    // Stats assert zero prohibited scans
    let src_stats = updated.snapshot.source_index().stats();
    assert_eq!(src_stats.source_workspace_scan_units, 0);
    assert_eq!(src_stats.reference_workspace_scan_units, 0);
    assert_eq!(src_stats.formal_workspace_scan_units, 0);
}

// -----------------------------------------------------------------------------
// PB-2 — Local references
// -----------------------------------------------------------------------------
#[test]
fn pb_2_local_references() {
    let app = module("app");
    let mut session = SemanticWorkspaceSession::new();

    let initial = session.update(multi_module_input(
        vec![(
            app.clone(),
            r#"class Demo {
  compute(x: Int) -> Int {
    let y = x + 1
    let z = y * 2
    z + x
  }
}
"#.into(),
        )],
        1,
    ));

    let query = initial.snapshot.editor();
    let app_shard = initial.snapshot.source_index().module(&app).unwrap();
    let x_binding = app_shard.structure.bindings.values().find(|b| b.name.as_ref() == "x").expect("x binding");
    let x_target = SemanticTargetId::Binding(x_binding.declaration_site.clone());

    let defs = query.definition_sites(&x_target);
    assert_eq!(defs.len(), 1);
    assert_eq!(defs[0], x_binding.declaration_site);

    let lexical_refs = query.reference_sites_in_domain(&x_target, ReferenceDomain::Lexical);
    assert_eq!(lexical_refs.len(), 2, "x is read twice in compute");

    let y_binding = app_shard.structure.bindings.values().find(|b| b.name.as_ref() == "y").expect("y binding");
    let y_target = SemanticTargetId::Binding(y_binding.declaration_site.clone());
    let y_lexical_refs = query.reference_sites_in_domain(&y_target, ReferenceDomain::Lexical);
    assert_eq!(y_lexical_refs.len(), 1, "y is read once");
}

// -----------------------------------------------------------------------------
// PB-3 — Imported binding reference indexing
// -----------------------------------------------------------------------------
#[test]
fn pb_3_imported_binding() {
    let mut session = SemanticWorkspaceSession::new();
    let input = two_module_input(
        "class Foo { @class make() -> Int { 1 } }\nexport Foo\n",
        "from provider import Foo\nclass Consumer { run() -> Int { Foo.make() } }\n",
        false,
        1,
    );
    let publication = session.update(input);
    let query = publication.snapshot.editor();

    let provider = module("provider");
    let consumer = module("consumer");
    let foo_target = SemanticTargetId::Declaration(DeclarationId::new(provider.clone(), "Foo".into()));

    let consumer_shard = publication.snapshot.source_index().module(&consumer).unwrap();
    let import_binding = consumer_shard.structure.bindings.values().find(|b| b.name.as_ref() == "Foo").expect("imported Foo binding");
    let import_target = SemanticTargetId::Binding(import_binding.declaration_site.clone());

    // Import declaration is local binding definition
    assert_eq!(query.definition_sites(&import_target), &[import_binding.declaration_site.clone()]);

    // Downstream use is local lexical ref to the binding
    let lexical_refs = query.reference_sites_in_domain(&import_target, ReferenceDomain::Lexical);
    assert_eq!(lexical_refs.len(), 1);

    // Downstream use is semantic ref to upstream Foo
    let semantic_refs = query.reference_sites_in_domain(&foo_target, ReferenceDomain::Semantic);
    assert!(semantic_refs.contains(&lexical_refs[0]));

    // Definition locations follows provenance to upstream Foo
    let def_locs = query.definition_locations(&import_target);
    assert!(!def_locs.is_empty());
    assert_eq!(
        def_locs[0],
        phalcom_semantic::editor::SemanticDefinitionLocation::SourceSite(
            publication.snapshot.source_index().declaration_source(&DeclarationId::new(provider, "Foo".into())).unwrap().declaration_site.clone()
        )
    );
}

// -----------------------------------------------------------------------------
// PB-4 — Alias dual relation
// -----------------------------------------------------------------------------
#[test]
fn pb_4_alias_dual_relation() {
    let mut session = SemanticWorkspaceSession::new();
    let input = two_module_input(
        "class Foo { @class make() -> Int { 1 }\n@class new() -> Foo { Foo.new() } }\nexport Foo\n",
        "from provider import Foo as Bar\nclass Consumer { run() -> Int { Bar.make(); Bar.new(); 0 } }\n",
        true,
        1,
    );
    let publication = session.update(input);
    let query = publication.snapshot.editor();

    let provider = module("provider");
    let consumer = module("consumer");
    let foo_target = SemanticTargetId::Declaration(DeclarationId::new(provider.clone(), "Foo".into()));

    let consumer_shard = publication.snapshot.source_index().module(&consumer).unwrap();
    let bar_binding = consumer_shard.structure.bindings.values().find(|b| b.name.as_ref() == "Bar").expect("Bar alias binding");
    let bar_target = SemanticTargetId::Binding(bar_binding.declaration_site.clone());

    // Bar declaration is local Binding definition
    assert_eq!(query.definition_sites(&bar_target), &[bar_binding.declaration_site.clone()]);

    // Bar uses are local lexical references to Bar
    let bar_lexical_refs = query.reference_sites_in_domain(&bar_target, ReferenceDomain::Lexical);
    assert_eq!(bar_lexical_refs.len(), 2, "Bar.make() and Bar.new()");

    // Bar uses are semantic references to Foo from consumer module (1 import site + 2 call sites)
    let foo_semantic_refs = query.reference_sites_in_domain(&foo_target, ReferenceDomain::Semantic);
    let consumer_foo_semantic_refs = foo_semantic_refs.iter().filter(|s| match &s.owner {
        phalcom_semantic::identity::SourceOwner::Module(m) => m == &consumer,
        phalcom_semantic::identity::SourceOwner::Callable(c) => c.module() == &consumer,
    }).count();
    assert_eq!(consumer_foo_semantic_refs, 3, "three semantic references to Foo in consumer (1 import + 2 call sites)");

    // Foo upstream definition does not include Bar declaration
    let foo_defs = query.definition_sites(&foo_target);
    assert!(!foo_defs.contains(&bar_binding.declaration_site));

    // RenameTargetView contains both
    let rename_view = query.rename_target_view(&bar_binding.declaration_site).expect("rename view for Bar");
    assert_eq!(rename_view.lexical_target, &bar_target);
    assert_eq!(rename_view.semantic_target, Some(&foo_target));
    assert_eq!(rename_view.lexical_references.len(), 2);
    assert!(rename_view.semantic_references.len() >= 3);
}

// -----------------------------------------------------------------------------
// PB-5 — Missing target appears
// -----------------------------------------------------------------------------
#[test]
fn pb_5_missing_target_appears() {
    let mut session = SemanticWorkspaceSession::new();
    let provider = module("provider");

    // Initially provider only has class Other, not Foo
    let initial_input = two_module_input(
        "class Other {}\n",
        "import provider.Foo\nclass Consumer { run() -> Int { Foo.make() } }\n",
        false,
        1,
    );
    let initial = session.update(initial_input);
    let initial_query = initial.snapshot.editor();
    let foo_target = SemanticTargetId::Declaration(DeclarationId::new(provider.clone(), "Foo".into()));
    assert_eq!(initial_query.definition_sites(&foo_target).len(), 0);

    // Provider adds Foo
    let updated_input = two_module_input(
        "class Foo { @class make() -> Int { 1 } }\nexport Foo\n",
        "import provider.Foo\nclass Consumer { run() -> Int { Foo.make() } }\n",
        false,
        2,
    );
    let updated = session.update(updated_input);
    let updated_query = updated.snapshot.editor();
    assert_eq!(updated_query.definition_sites(&foo_target).len(), 1);
    assert_eq!(updated_query.reference_sites_in_domain(&foo_target, ReferenceDomain::Semantic).len(), 1);
}

// -----------------------------------------------------------------------------
// PB-6 — Target disappears
// -----------------------------------------------------------------------------
#[test]
fn pb_6_target_disappears() {
    let mut session = SemanticWorkspaceSession::new();
    let provider = module("provider");

    let initial_input = two_module_input(
        "class Foo { @class make() -> Int { 1 } }\nexport Foo\n",
        "import provider.Foo\nclass Consumer { run() -> Int { Foo.make() } }\n",
        false,
        1,
    );
    let initial = session.update(initial_input);
    let foo_target = SemanticTargetId::Declaration(DeclarationId::new(provider.clone(), "Foo".into()));
    assert_eq!(initial.snapshot.editor().definition_sites(&foo_target).len(), 1);
    assert_eq!(initial.snapshot.editor().workspace_symbols("Foo", 10).len(), 1);

    // Provider removes Foo
    let updated_input = two_module_input(
        "class Gone {}\n",
        "import provider.Foo\nclass Consumer { run() -> Int { 0 } }\n",
        false,
        2,
    );
    let updated = session.update(updated_input);
    let updated_query = updated.snapshot.editor();
    assert_eq!(updated_query.definition_sites(&foo_target).len(), 0);
    assert_eq!(updated_query.reference_sites_in_domain(&foo_target, ReferenceDomain::Semantic).len(), 0);
    assert_eq!(updated_query.workspace_symbols("Foo", 10).len(), 0);
}

// -----------------------------------------------------------------------------
// PB-7 — Rename and delete/re-add
// -----------------------------------------------------------------------------
#[test]
fn pb_7_rename_and_delete_readd() {
    let app = module("app");
    let mut session = SemanticWorkspaceSession::new();

    // 1. Foo
    let s1 = session.update(multi_module_input(vec![(app.clone(), "class Foo {}\n".into())], 1));
    let foo_target = SemanticTargetId::Declaration(DeclarationId::new(app.clone(), "Foo".into()));
    assert_eq!(s1.snapshot.editor().definition_sites(&foo_target).len(), 1);

    // 2. Rename to Bar
    let s2 = session.update(multi_module_input(vec![(app.clone(), "class Bar {}\n".into())], 2));
    let bar_target = SemanticTargetId::Declaration(DeclarationId::new(app.clone(), "Bar".into()));
    assert_eq!(s2.snapshot.editor().definition_sites(&foo_target).len(), 0);
    assert_eq!(s2.snapshot.editor().definition_sites(&bar_target).len(), 1);
    assert_eq!(s2.snapshot.editor().workspace_symbols("Foo", 10).len(), 0);
    assert_eq!(s2.snapshot.editor().workspace_symbols("Bar", 10).len(), 1);

    // 3. Delete all
    let s3 = session.update(multi_module_input(vec![(app.clone(), "// empty\n".into())], 3));
    assert_eq!(s3.snapshot.editor().definition_sites(&bar_target).len(), 0);
    assert_eq!(s3.snapshot.editor().workspace_symbols("Bar", 10).len(), 0);

    // 4. Foo again
    let s4 = session.update(multi_module_input(vec![(app.clone(), "class Foo {}\n".into())], 4));
    assert_eq!(s4.snapshot.editor().definition_sites(&foo_target).len(), 1);
    assert_eq!(s4.snapshot.editor().workspace_symbols("Foo", 10).len(), 1);
}

// -----------------------------------------------------------------------------
// PB-8 — Structural sharing
// -----------------------------------------------------------------------------
#[test]
fn pb_8_structural_sharing() {
    let m1 = module("m1");
    let m2 = module("m2");
    let m3 = module("m3");
    let mut session = SemanticWorkspaceSession::new();

    let initial = session.update(multi_module_input(
        vec![
            (m1.clone(), "class C1 {}\n".into()),
            (m2.clone(), "class C2 {}\n".into()),
            (m3.clone(), "class C3 {}\n".into()),
        ],
        1,
    ));

    let m2_shard = initial.snapshot.source_index().module_arc(&m2).unwrap();
    let m3_shard = initial.snapshot.source_index().module_arc(&m3).unwrap();
    let m2_formal = initial.snapshot.formal_projection().module_arc(&m2).unwrap();
    let m3_formal = initial.snapshot.formal_projection().module_arc(&m3).unwrap();

    // Edit only m1
    let updated = session.update(multi_module_input(
        vec![
            (m1.clone(), "class C1 { run() { 1 } }\n".into()),
            (m2.clone(), "class C2 {}\n".into()),
            (m3.clone(), "class C3 {}\n".into()),
        ],
        2,
    ));

    // Pointer equality for m2 and m3 shards
    assert!(Arc::ptr_eq(&m2_shard, &updated.snapshot.source_index().module_arc(&m2).unwrap()));
    assert!(Arc::ptr_eq(&m3_shard, &updated.snapshot.source_index().module_arc(&m3).unwrap()));
    assert!(Arc::ptr_eq(&m2_formal, &updated.snapshot.formal_projection().module_arc(&m2).unwrap()));
    assert!(Arc::ptr_eq(&m3_formal, &updated.snapshot.formal_projection().module_arc(&m3).unwrap()));
}

// -----------------------------------------------------------------------------
// PB-9a / PB-9b / PB-9c — High Fanout
// -----------------------------------------------------------------------------
fn make_fanout_input(provider_source: &str, consumer_sources: Vec<(usize, String)>, generation: u64) -> SemanticWorkspaceInput {
    const CONSUMER_COUNT: usize = 50;
    let provider = module("provider");
    let mut sources = BTreeMap::new();
    let p_src: Arc<str> = Arc::from(provider_source.to_owned());
    let p_prog = Arc::new(phalcom_ast::parse(&p_src, 0).program);
    sources.insert(
        provider.clone(),
        Arc::new(ParsedModuleUnit::new(provider.clone(), ModuleKind::Module, None, p_src, p_prog)),
    );

    let export_foo = LinkedExport {
        public_name: "Foo".into(),
        target: LinkedExportTarget::Binding(SymbolId {
            module: provider.clone(),
            name: "Foo".into(),
        }),
        range: phalcom_common::range::SourceRange::default(),
    };
    let p_mod = LinkedModule {
        interface: LinkedModuleInterface {
            module: provider.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::from([("Foo".into(), export_foo)]),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout {
            local_globals: BTreeMap::from([("Foo".into(), GlobalBindingId(0))]),
            imports: BTreeMap::new(),
        },
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };
    let mut linked_modules = BTreeMap::from([(provider.clone(), p_mod)]);
    let mut init_order = vec![provider.clone()];

    let consumer_map: BTreeMap<usize, String> = consumer_sources.into_iter().collect();
    for i in 0..CONSUMER_COUNT {
        let name = format!("c{i:02}");
        let c_mod_id = module(&name);
        let src_str = consumer_map.get(&i).cloned().unwrap_or_else(|| {
            format!("import provider.Foo\nclass C{i:02} {{ run() -> Int {{ Foo.value() }} }}\n")
        });
        let c_src: Arc<str> = Arc::from(src_str);
        let c_prog = Arc::new(phalcom_ast::parse(&c_src, 0).program);
        sources.insert(
            c_mod_id.clone(),
            Arc::new(ParsedModuleUnit::new(c_mod_id.clone(), ModuleKind::Module, None, c_src, c_prog)),
        );
        let c_mod = LinkedModule {
            interface: LinkedModuleInterface {
                module: c_mod_id.clone(),
                kind: ModuleKind::Module,
                exports: BTreeMap::new(),
                metadata: ModuleMetadata::default(),
            },
            bindings: ModuleBindingLayout {
                local_globals: BTreeMap::from([(format!("C{i:02}").into(), GlobalBindingId(0))]),
                imports: BTreeMap::from([("Foo".into(), ImportBindingId(0))]),
            },
            linked_reads: vec![LinkedReadSpec::Binding(SymbolId {
                module: provider.clone(),
                name: "Foo".into(),
            })],
            runtime_dependencies: vec![provider.clone()],
        };
        linked_modules.insert(c_mod_id.clone(), c_mod);
        init_order.push(c_mod_id);
    }

    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
        modules: linked_modules,
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: module("c00"),
        initialization_order: init_order,
    });
    SemanticWorkspaceInput::new(linked, sources, generation)
}

#[test]
fn pb_9a_high_fanout_provider_body_only_edit() {
    let mut session = SemanticWorkspaceSession::new();
    let initial = session.update(make_fanout_input("class Foo { @class value() -> Int { 1 } }\nexport Foo\n", Vec::new(), 1));
    assert!(initial.snapshot.source_index().modules().count() >= 51);

    // Capture consumer shards from initial snapshot
    let initial_consumers = (0..50)
        .map(|i| {
            let m = module(&format!("c{:02}", i));
            (m.clone(), initial.snapshot.source_index().module_arc(&m).unwrap())
        })
        .collect::<Vec<_>>();

    // Body-only edit on Provider
    let updated = session.update(make_fanout_input("class Foo { @class value() -> Int { 2 } }\nexport Foo\n", Vec::new(), 2));

    // Prohibited scan invariants: zero whole-workspace scans
    let src_stats = updated.snapshot.source_index().stats();
    assert_eq!(src_stats.source_workspace_scan_units, 0);
    assert_eq!(src_stats.reference_workspace_scan_units, 0);
    assert_eq!(src_stats.formal_workspace_scan_units, 0);
    assert!(src_stats.source_modules_rebuilt <= 2, "at most provider and entry rebuilt");
    assert_eq!(src_stats.source_modules_retired, 0);

    // Consumers (c01..c49) are structurally shared and retained
    for (m, initial_shard) in &initial_consumers[1..] {
        let updated_shard = updated.snapshot.source_index().module_arc(m).unwrap();
        assert!(Arc::ptr_eq(initial_shard, &updated_shard), "consumer shard {} retained", m);
    }
}

#[test]
fn pb_9b_high_fanout_one_consumer_reference_edit() {
    let mut session = SemanticWorkspaceSession::new();
    let initial = session.update(make_fanout_input("class Foo { @class value() -> Int { 1 } }\nexport Foo\n", Vec::new(), 1));

    let c05_shard = initial.snapshot.source_index().module_arc(&module("c05")).unwrap();
    let c06_shard = initial.snapshot.source_index().module_arc(&module("c06")).unwrap();

    // Edit only c05
    let updated = session.update(make_fanout_input(
        "class Foo { @class value() -> Int { 1 } }\nexport Foo\n",
        vec![(5, "import provider.Foo\nclass C05 { run() -> Int { Foo.value() + 10 } }\n".into())],
        2,
    ));

    // Exactly c05 changed, c06 retained
    let updated_c05_shard = updated.snapshot.source_index().module_arc(&module("c05")).unwrap();
    let updated_c06_shard = updated.snapshot.source_index().module_arc(&module("c06")).unwrap();
    assert!(!Arc::ptr_eq(&c05_shard, &updated_c05_shard));
    assert!(Arc::ptr_eq(&c06_shard, &updated_c06_shard));

    let src_stats = updated.snapshot.source_index().stats();
    assert_eq!(src_stats.source_workspace_scan_units, 0);
    assert_eq!(src_stats.reference_workspace_scan_units, 0);
}

// -----------------------------------------------------------------------------
// PB-10 — Cold/incremental editor parity
// -----------------------------------------------------------------------------
#[test]
fn pb_10_cold_incremental_parity() {
    let mut session = SemanticWorkspaceSession::new();
    let provider = module("provider");

    let input_v1 = two_module_input(
        "class Foo { @class make() -> Int { 1 } }\nexport Foo\n",
        "import provider.Foo\nclass Consumer { run() -> Int { Foo.make() } }\n",
        false,
        1,
    );
    let _inc_v1 = session.update(input_v1.clone());

    let input_v2 = two_module_input(
        "class Foo { @class make() -> Int { 2 }\n@class another() -> Int { 3 } }\nexport Foo\n",
        "import provider.Foo\nclass Consumer { run() -> Int { Foo.make(); Foo.another(); 0 } }\n",
        false,
        2,
    );
    let inc_v2 = session.update(input_v2.clone());

    // Compare with fresh cold session on v2
    let mut cold_session = SemanticWorkspaceSession::new();
    let cold_v2 = cold_session.update(input_v2);

    let inc_query = inc_v2.snapshot.editor();
    let cold_query = cold_v2.snapshot.editor();

    let foo_target = SemanticTargetId::Declaration(DeclarationId::new(provider.clone(), "Foo".into()));
    assert_eq!(inc_query.definition_sites(&foo_target).len(), cold_query.definition_sites(&foo_target).len());
    assert_eq!(
        inc_query.reference_sites_in_domain(&foo_target, ReferenceDomain::Semantic).len(),
        cold_query.reference_sites_in_domain(&foo_target, ReferenceDomain::Semantic).len()
    );
    assert_eq!(
        inc_query.workspace_symbols("Foo", 10).len(),
        cold_query.workspace_symbols("Foo", 10).len()
    );
}

// -----------------------------------------------------------------------------
// PB-11 — Old snapshot immutability
// -----------------------------------------------------------------------------
#[test]
fn pb_11_old_snapshot_immutability() {
    let mut session = SemanticWorkspaceSession::new();
    let app = module("app");

    let s1 = session.update(multi_module_input(vec![(app.clone(), "class Foo { run() { 1 } }\n".into())], 1));
    let snap1 = s1.snapshot.clone();

    let s2 = session.update(multi_module_input(vec![(app.clone(), "class Bar { run() { 2 } }\n".into())], 2));
    let snap2 = s2.snapshot.clone();

    let s3 = session.update(multi_module_input(vec![(app.clone(), "// empty\n".into())], 3));
    let snap3 = s3.snapshot.clone();

    let foo_target = SemanticTargetId::Declaration(DeclarationId::new(app.clone(), "Foo".into()));
    let bar_target = SemanticTargetId::Declaration(DeclarationId::new(app.clone(), "Bar".into()));

    // Query old snapshot 1: still has Foo
    let q1 = snap1.editor();
    assert_eq!(q1.definition_sites(&foo_target).len(), 1);
    assert_eq!(q1.definition_sites(&bar_target).len(), 0);
    assert_eq!(q1.workspace_symbols("Foo", 10).len(), 1);

    // Query old snapshot 2: still has Bar
    let q2 = snap2.editor();
    assert_eq!(q2.definition_sites(&foo_target).len(), 0);
    assert_eq!(q2.definition_sites(&bar_target).len(), 1);
    assert_eq!(q2.workspace_symbols("Bar", 10).len(), 1);

    // Query snapshot 3: empty
    let q3 = snap3.editor();
    assert_eq!(q3.definition_sites(&foo_target).len(), 0);
    assert_eq!(q3.definition_sites(&bar_target).len(), 0);
    assert_eq!(q3.workspace_symbols("Foo", 10).len(), 0);
    assert_eq!(q3.workspace_symbols("Bar", 10).len(), 0);
}

// -----------------------------------------------------------------------------
// PB-12 — Zero prohibited workspace scans
// -----------------------------------------------------------------------------
#[test]
fn pb_12_zero_prohibited_workspace_scans() {
    let mut session = SemanticWorkspaceSession::new();
    let _initial = session.update(make_fanout_input("class Foo { @class value() -> Int { 1 } }\nexport Foo\n", Vec::new(), 1));

    // Perform an ordinary edit to one consumer
    let update = session.update(make_fanout_input(
        "class Foo { @class value() -> Int { 1 } }\nexport Foo\n",
        vec![(0, "import provider.Foo\nclass C00 { run() -> Int { Foo.value() + 42 } }\n".into())],
        2,
    ));

    let src_stats = update.snapshot.source_index().stats();
    assert_eq!(src_stats.source_workspace_scan_units, 0);
    assert_eq!(src_stats.reference_workspace_scan_units, 0);
    assert_eq!(src_stats.formal_workspace_scan_units, 0);
}

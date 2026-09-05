use phalcom_common::range::SourceRange;
use phalcom_modules::interface::{LinkedExport, LinkedExportTarget, LinkedModuleInterface, UnlinkedModuleInterface};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::{ModuleComponent, ModuleId, ModuleKind, ModulePath, ModuleQueryFacade, ProjectUniverse, SourceId, SourceLocation};
use std::collections::{BTreeMap, BTreeSet};

fn component(name: &str) -> ModuleComponent {
    ModuleComponent::from_identifier(name).expect("valid module component")
}

fn path(name: &str) -> ModulePath {
    ModulePath::from_components(vec![component(name)])
}

#[test]
fn facade_exposes_canonical_roots_children_exports_and_provenance() {
    let universe = ProjectUniverse::new();
    let root = ModuleId::universe(ModulePath::root());
    let child = ModuleId::universe(path("math"));

    let mut unlinked = BTreeMap::new();
    unlinked.insert(
        root.clone(),
        UnlinkedModuleInterface {
            id: root.clone(),
            kind: ModuleKind::Package,
            declarations: BTreeMap::new(),
            exports: BTreeMap::new(),
            imports: Vec::new(),
            exposed_children: BTreeSet::from([component("math")]),
            metadata: ModuleMetadata::default(),
        },
    );

    let mut linked = BTreeMap::new();
    linked.insert(
        child.clone(),
        LinkedModuleInterface {
            module: child.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::from([(
                "answer".into(),
                LinkedExport {
                    public_name: "answer".into(),
                    target: LinkedExportTarget::Module(child.clone()),
                    range: SourceRange { start: 0, end: 6 },
                },
            )]),
            metadata: ModuleMetadata::default(),
        },
    );

    let mut resolved = BTreeMap::new();
    resolved.insert((root.clone(), "universe.math".to_string()), child.clone());
    let mut sources = BTreeMap::new();
    sources.insert(
        child.clone(),
        SourceLocation {
            source_id: SourceId("/workspace/src/math.ph".into()),
            display_path: "/workspace/src/math.ph".into(),
        },
    );

    let source_modules = BTreeMap::from([(SourceId("/workspace/src/math.ph".into()), child.clone())]);
    let display_path_modules = BTreeMap::from([(std::path::PathBuf::from("/workspace/src/math.ph"), child.clone())]);
    let facade = ModuleQueryFacade::new(&universe, &unlinked, &linked, &resolved, &sources, &source_modules, &display_path_modules);
    let roots = facade.import_roots(&root);
    assert!(roots.contains_key(&component("universe")));
    assert_eq!(facade.import_children(&root, &ModulePath::root()), vec![child.clone()]);
    assert!(facade.public_exports(&child).unwrap().contains_key("answer"));
    assert_eq!(facade.resolved_import_target(&root, "universe.math"), Some(&child));
    assert_eq!(facade.reverse_importers(&child), vec![root.clone()]);
    assert_eq!(
        facade.definition_source(&child).unwrap().display_path.to_string_lossy(),
        "/workspace/src/math.ph"
    );
    assert_eq!(facade.module_for_source(&SourceId("/workspace/src/math.ph".into())), Some(&child));
    assert_eq!(facade.module_for_display_path(std::path::Path::new("/workspace/src/math.ph")), Some(&child));
}

#[test]
fn facade_rejects_unexposed_package_children() {
    let universe = ProjectUniverse::new();
    let root = ModuleId::universe(ModulePath::root());
    let child = ModuleId::universe(path("private"));
    let unlinked = BTreeMap::from([(
        root.clone(),
        UnlinkedModuleInterface {
            id: root.clone(),
            kind: ModuleKind::Package,
            declarations: BTreeMap::new(),
            exports: BTreeMap::new(),
            imports: Vec::new(),
            exposed_children: BTreeSet::new(),
            metadata: ModuleMetadata::default(),
        },
    )]);
    let linked = BTreeMap::from([(
        child.clone(),
        LinkedModuleInterface {
            module: child,
            kind: ModuleKind::Module,
            exports: BTreeMap::new(),
            metadata: ModuleMetadata::default(),
        },
    )]);
    let resolved = BTreeMap::new();
    let sources = BTreeMap::new();

    let source_modules = BTreeMap::new();
    let display_path_modules = BTreeMap::new();
    let facade = ModuleQueryFacade::new(&universe, &unlinked, &linked, &resolved, &sources, &source_modules, &display_path_modules);
    assert!(facade.import_children(&root, &ModulePath::root()).is_empty());
}

#[test]
fn facade_with_topology_and_reverse_imports_accelerates_lookups() {
    use phalcom_modules::stabilization::ResolverGeneration;
    use phalcom_modules::topology::ModuleTopology;

    let universe = ProjectUniverse::new();
    let root = ModuleId::universe(ModulePath::root());
    let child = ModuleId::universe(path("math"));

    let unlinked = BTreeMap::from([(
        root.clone(),
        UnlinkedModuleInterface {
            id: root.clone(),
            kind: ModuleKind::Package,
            declarations: BTreeMap::new(),
            exports: BTreeMap::new(),
            imports: Vec::new(),
            exposed_children: BTreeSet::from([component("math")]),
            metadata: ModuleMetadata::default(),
        },
    )]);
    let linked = BTreeMap::from([(
        child.clone(),
        LinkedModuleInterface {
            module: child.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::new(),
            metadata: ModuleMetadata::default(),
        },
    )]);
    let resolved = BTreeMap::from([((root.clone(), "universe.math".to_string()), child.clone())]);
    let sources = BTreeMap::from([(
        child.clone(),
        SourceLocation {
            source_id: SourceId("/workspace/src/math.ph".into()),
            display_path: "/workspace/src/math.ph".into(),
        },
    )]);
    let source_modules = BTreeMap::new();
    let display_path_modules = BTreeMap::new();

    let topology = ModuleTopology::from_parts(ResolverGeneration(1), &universe, &unlinked, &sources);
    let reverse_imports = BTreeMap::from([(child.clone(), BTreeSet::from([root.clone()]))]);

    use std::sync::atomic::{AtomicU64, Ordering};

    let counter = AtomicU64::new(0);
    let facade = ModuleQueryFacade::new(&universe, &unlinked, &linked, &resolved, &sources, &source_modules, &display_path_modules)
        .with_topology(&topology)
        .with_reverse_imports(&reverse_imports)
        .with_fallback_counter(&counter);

    assert!(facade.has_topology());
    assert!(facade.has_reverse_imports());
    assert!(facade.is_fully_indexed());

    // Verified queries route through topology and precomputed reverse index
    assert_eq!(facade.module_children(root.project, &ModulePath::root()), vec![child.clone()]);
    assert_eq!(facade.import_children(&root, &ModulePath::root()), vec![child.clone()]);
    assert_eq!(facade.module_for_source(&SourceId("/workspace/src/math.ph".into())), Some(&child));
    assert_eq!(facade.reverse_importers(&child), vec![root.clone()]);

    // Zero fallback scans occurred on the fully-indexed facade
    assert_eq!(counter.load(Ordering::Relaxed), 0);
}

#[test]
fn unindexed_facade_records_fallback_scans_while_indexed_records_zero() {
    use phalcom_modules::stabilization::ResolverGeneration;
    use phalcom_modules::topology::ModuleTopology;
    use std::sync::atomic::{AtomicU64, Ordering};

    let universe = ProjectUniverse::new();
    let root = ModuleId::universe(ModulePath::root());
    let child = ModuleId::universe(path("math"));

    let unlinked = BTreeMap::from([(
        root.clone(),
        UnlinkedModuleInterface {
            id: root.clone(),
            kind: ModuleKind::Package,
            declarations: BTreeMap::new(),
            exports: BTreeMap::new(),
            imports: Vec::new(),
            exposed_children: BTreeSet::from([component("math")]),
            metadata: ModuleMetadata::default(),
        },
    )]);
    let linked = BTreeMap::from([(
        child.clone(),
        LinkedModuleInterface {
            module: child.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::new(),
            metadata: ModuleMetadata::default(),
        },
    )]);
    let resolved = BTreeMap::from([((root.clone(), "universe.math".to_string()), child.clone())]);
    let sources = BTreeMap::from([(
        child.clone(),
        SourceLocation {
            source_id: SourceId("/workspace/src/math.ph".into()),
            display_path: "/workspace/src/math.ph".into(),
        },
    )]);
    let source_modules = BTreeMap::from([(SourceId("/workspace/src/math.ph".into()), child.clone())]);
    let display_path_modules = BTreeMap::new();

    // 1. Un-indexed facade records fallback scans on every fallback-eligible query
    let unindexed_counter = AtomicU64::new(0);
    let unindexed = ModuleQueryFacade::new(&universe, &unlinked, &linked, &resolved, &sources, &source_modules, &display_path_modules)
        .with_fallback_counter(&unindexed_counter);

    assert!(!unindexed.has_topology());
    assert!(!unindexed.has_reverse_imports());
    assert!(!unindexed.is_fully_indexed());

    let _ = unindexed.module_children(root.project, &ModulePath::root());
    assert_eq!(unindexed_counter.load(Ordering::Relaxed), 1);

    let _ = unindexed.import_children(&root, &ModulePath::root());
    // external_import_children calls record_fallback_scan, and its inner module_children also calls it
    assert!(unindexed_counter.load(Ordering::Relaxed) >= 2);

    let _ = unindexed.module_for_source(&SourceId("/workspace/src/math.ph".into()));
    assert!(unindexed_counter.load(Ordering::Relaxed) >= 3);

    let _ = unindexed.reverse_importers(&child);
    assert!(unindexed_counter.load(Ordering::Relaxed) >= 4);

    // 2. Fully indexed facade executes same queries with 0 fallback scans
    let topology = ModuleTopology::from_parts(ResolverGeneration(1), &universe, &unlinked, &sources);
    let reverse_imports = BTreeMap::from([(child.clone(), BTreeSet::from([root.clone()]))]);
    let indexed_counter = AtomicU64::new(0);
    let indexed = ModuleQueryFacade::new(&universe, &unlinked, &linked, &resolved, &sources, &source_modules, &display_path_modules)
        .with_topology(&topology)
        .with_reverse_imports(&reverse_imports)
        .with_fallback_counter(&indexed_counter);

    assert!(indexed.is_fully_indexed());
    assert_eq!(indexed.module_children(root.project, &ModulePath::root()), vec![child.clone()]);
    assert_eq!(indexed.import_children(&root, &ModulePath::root()), vec![child.clone()]);
    assert_eq!(indexed.module_for_source(&SourceId("/workspace/src/math.ph".into())), Some(&child));
    assert_eq!(indexed.reverse_importers(&child), vec![root.clone()]);

    assert_eq!(indexed_counter.load(Ordering::Relaxed), 0, "indexed queries must perform zero fallback scans");
}

#[test]
fn synthetic_large_scale_topology_query_work_count() {
    use phalcom_modules::stabilization::ResolverGeneration;
    use phalcom_modules::topology::ModuleTopology;
    use std::sync::atomic::{AtomicU64, Ordering};

    let universe = ProjectUniverse::new();
    let root = ModuleId::universe(ModulePath::root());

    // Build 1,000 synthetic nodes: 10 top-level packages, each with 10 subpackages, each with 10 modules
    let mut unlinked = BTreeMap::new();
    let mut sources = BTreeMap::new();
    let mut reverse_imports: BTreeMap<ModuleId, BTreeSet<ModuleId>> = BTreeMap::new();

    let mut root_exposed = BTreeSet::new();
    for p in 0..10 {
        let p_name = format!("pkg_{p}");
        root_exposed.insert(component(&p_name));
    }
    unlinked.insert(
        root.clone(),
        UnlinkedModuleInterface {
            id: root.clone(),
            kind: ModuleKind::Package,
            declarations: BTreeMap::new(),
            exports: BTreeMap::new(),
            imports: Vec::new(),
            exposed_children: root_exposed,
            metadata: ModuleMetadata::default(),
        },
    );

    let mut total_nodes = 1usize; // root
    for p in 0..10 {
        let p_name = format!("pkg_{p}");
        let p_path = ModulePath::from_components(vec![component(&p_name)]);
        let p_id = ModuleId::universe(p_path.clone());
        let mut p_exposed = BTreeSet::new();
        for sp in 0..10 {
            p_exposed.insert(component(&format!("sub_{sp}")));
        }
        unlinked.insert(
            p_id.clone(),
            UnlinkedModuleInterface {
                id: p_id.clone(),
                kind: ModuleKind::Package,
                declarations: BTreeMap::new(),
                exports: BTreeMap::new(),
                imports: Vec::new(),
                exposed_children: p_exposed,
                metadata: ModuleMetadata::default(),
            },
        );
        total_nodes += 1;

        for sp in 0..10 {
            let sp_name = format!("sub_{sp}");
            let sp_path = p_path.join(component(&sp_name));
            let sp_id = ModuleId::universe(sp_path.clone());
            let mut sp_exposed = BTreeSet::new();
            for m in 0..10 {
                sp_exposed.insert(component(&format!("mod_{m}")));
            }
            unlinked.insert(
                sp_id.clone(),
                UnlinkedModuleInterface {
                    id: sp_id.clone(),
                    kind: ModuleKind::Package,
                    declarations: BTreeMap::new(),
                    exports: BTreeMap::new(),
                    imports: Vec::new(),
                    exposed_children: sp_exposed,
                    metadata: ModuleMetadata::default(),
                },
            );
            total_nodes += 1;

            for m in 0..10 {
                let m_name = format!("mod_{m}");
                let m_path = sp_path.join(component(&m_name));
                let m_id = ModuleId::universe(m_path);
                unlinked.insert(
                    m_id.clone(),
                    UnlinkedModuleInterface {
                        id: m_id.clone(),
                        kind: ModuleKind::Module,
                        declarations: BTreeMap::new(),
                        exports: BTreeMap::new(),
                        imports: Vec::new(),
                        exposed_children: BTreeSet::new(),
                        metadata: ModuleMetadata::default(),
                    },
                );
                let src_path = format!("/workspace/src/{p_name}/{sp_name}/{m_name}.ph");
                sources.insert(
                    m_id.clone(),
                    SourceLocation {
                        source_id: SourceId(src_path.clone().into()),
                        display_path: src_path.into(),
                    },
                );
                // Each module is imported by root
                reverse_imports.entry(m_id.clone()).or_default().insert(root.clone());
                total_nodes += 1;
            }
        }
    }

    assert_eq!(total_nodes, 1111); // 1 root + 10 pkgs + 100 subpkgs + 1000 modules

    let topology = ModuleTopology::from_parts(ResolverGeneration(1), &universe, &unlinked, &sources);
    assert_eq!(topology.nodes.len(), 1111);

    let linked = BTreeMap::new();
    let resolved = BTreeMap::new();
    let source_modules = BTreeMap::new();
    let display_path_modules = BTreeMap::new();
    let counter = AtomicU64::new(0);

    let facade = ModuleQueryFacade::new(&universe, &unlinked, &linked, &resolved, &sources, &source_modules, &display_path_modules)
        .with_topology(&topology)
        .with_reverse_imports(&reverse_imports)
        .with_fallback_counter(&counter);

    // 1. Root children: exactly 10 packages
    let root_children = facade.module_children(root.project, &ModulePath::root());
    assert_eq!(root_children.len(), 10);

    // 2. Querying children of each package: exactly 10 subpackages each
    for child in &root_children {
        let subs = facade.module_children(child.project, &child.path);
        assert_eq!(subs.len(), 10);
        for sub in &subs {
            let mods = facade.module_children(sub.project, &sub.path);
            assert_eq!(mods.len(), 10);
            for m in &mods {
                // Reverse importers lookup: O(1) indexed
                let importers = facade.reverse_importers(m);
                assert_eq!(importers, vec![root.clone()]);
            }
        }
    }

    // 3. Source lookup: exactly resolves without scanning
    let sample_source = SourceId("/workspace/src/pkg_3/sub_7/mod_2.ph".into());
    let found_module = facade.module_for_source(&sample_source);
    assert!(found_module.is_some());
    assert_eq!(found_module.unwrap().path.to_string(), "pkg_3.sub_7.mod_2");

    // Zero fallback scans across thousands of node operations on 1k+ graph
    assert_eq!(counter.load(Ordering::Relaxed), 0, "large synthetic topology queries must perform exactly zero fallback scans");
}

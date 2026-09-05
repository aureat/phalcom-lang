use phalcom_modules::fingerprint::{
    interface_fingerprint, linked_interface_fingerprint, unlinked_interface_input_fingerprint,
};
use phalcom_modules::interface::{
    DeclarationSurface, ExportSurface, LinkedExport, LinkedExportTarget, LinkedModuleInterface,
    UnlinkedExportTarget, UnlinkedModuleInterface,
};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::resolver::{
    ImportPathIdentity, ImportResolutionProduct, ResolutionFingerprint, ResolutionTopologyDependencies,
};
use phalcom_modules::source::ModuleKind;
use phalcom_modules::stabilization::ResolverGeneration;
use phalcom_modules::topology::{ModuleTopology, TopologyFingerprint, TopologyNode};
use phalcom_modules::{ModuleComponent, ModuleId, ModulePath, ProjectUniverse, SourceId, SourceLocation};
use std::collections::{BTreeMap, BTreeSet};

fn component(name: &str) -> ModuleComponent {
    ModuleComponent::from_identifier(name).expect("valid module component")
}

fn path(name: &str) -> ModulePath {
    ModulePath::from_components(vec![component(name)])
}

#[test]
fn topology_fingerprint_ignores_symbol_only_export_and_declaration_changes() {
    let universe = ProjectUniverse::new();
    let root = ModuleId::universe(ModulePath::root());
    let child = ModuleId::universe(path("service"));

    let mut unlinked_base = BTreeMap::new();
    unlinked_base.insert(
        root.clone(),
        UnlinkedModuleInterface {
            id: root.clone(),
            kind: ModuleKind::Package,
            declarations: BTreeMap::new(),
            exports: BTreeMap::new(),
            imports: Vec::new(),
            exposed_children: BTreeSet::from([component("service")]),
            metadata: ModuleMetadata::default(),
        },
    );
    unlinked_base.insert(
        child.clone(),
        UnlinkedModuleInterface {
            id: child.clone(),
            kind: ModuleKind::Module,
            declarations: BTreeMap::from([(
                "hello".to_string(),
                DeclarationSurface {
                    name: "hello".to_string(),
                    is_const: true,
                    range: phalcom_common::range::SourceRange { start: 0, end: 10 },
                },
            )]),
            exports: BTreeMap::new(),
            imports: Vec::new(),
            exposed_children: BTreeSet::new(),
            metadata: ModuleMetadata::default(),
        },
    );

    let sources = BTreeMap::from([
        (
            root.clone(),
            SourceLocation {
                source_id: SourceId("/universe/package.ph".into()),
                display_path: "/universe/package.ph".into(),
            },
        ),
        (
            child.clone(),
            SourceLocation {
                source_id: SourceId("/universe/service.ph".into()),
                display_path: "/universe/service.ph".into(),
            },
        ),
    ]);

    let topo_base = ModuleTopology::from_parts(ResolverGeneration(1), &universe, &unlinked_base, &sources);

    // Modify declarations in child interface (symbol change)
    let mut unlinked_symbol_change = unlinked_base.clone();
    unlinked_symbol_change.get_mut(&child).unwrap().declarations.insert(
        "goodbye".to_string(),
        DeclarationSurface {
            name: "goodbye".to_string(),
            is_const: false,
            range: phalcom_common::range::SourceRange { start: 15, end: 30 },
        },
    );

    // Interface fingerprint must change
    assert_ne!(
        interface_fingerprint(&unlinked_base[&child]),
        interface_fingerprint(&unlinked_symbol_change[&child])
    );

    // Topology fingerprint MUST remain unchanged
    let topo_symbol_change =
        ModuleTopology::from_parts(ResolverGeneration(1), &universe, &unlinked_symbol_change, &sources);
    assert_eq!(topo_base.fingerprint, topo_symbol_change.fingerprint);
}

#[test]
fn topology_fingerprint_changes_on_package_exposure_change() {
    let universe = ProjectUniverse::new();
    let root = ModuleId::universe(ModulePath::root());

    let mut unlinked_base = BTreeMap::new();
    unlinked_base.insert(
        root.clone(),
        UnlinkedModuleInterface {
            id: root.clone(),
            kind: ModuleKind::Package,
            declarations: BTreeMap::new(),
            exports: BTreeMap::new(),
            imports: Vec::new(),
            exposed_children: BTreeSet::from([component("service")]),
            metadata: ModuleMetadata::default(),
        },
    );
    let sources = BTreeMap::from([(
        root.clone(),
        SourceLocation {
            source_id: SourceId("/universe/package.ph".into()),
            display_path: "/universe/package.ph".into(),
        },
    )]);

    let topo_base = ModuleTopology::from_parts(ResolverGeneration(1), &universe, &unlinked_base, &sources);

    // Modify exposure: expose "math" as well
    let mut unlinked_exposed_change = unlinked_base.clone();
    unlinked_exposed_change
        .get_mut(&root)
        .unwrap()
        .exposed_children
        .insert(component("math"));

    let topo_exposed =
        ModuleTopology::from_parts(ResolverGeneration(1), &universe, &unlinked_exposed_change, &sources);

    assert_ne!(topo_base.fingerprint, topo_exposed.fingerprint);
}

#[test]
fn topology_fingerprint_changes_on_module_addition_and_removal() {
    let universe = ProjectUniverse::new();
    let root = ModuleId::universe(ModulePath::root());
    let child = ModuleId::universe(path("worker"));

    let mut unlinked_one = BTreeMap::new();
    unlinked_one.insert(
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
    );
    let sources = BTreeMap::new();

    let topo_one = ModuleTopology::from_parts(ResolverGeneration(1), &universe, &unlinked_one, &sources);

    let mut unlinked_two = unlinked_one.clone();
    unlinked_two.insert(
        child.clone(),
        UnlinkedModuleInterface {
            id: child.clone(),
            kind: ModuleKind::Module,
            declarations: BTreeMap::new(),
            exports: BTreeMap::new(),
            imports: Vec::new(),
            exposed_children: BTreeSet::new(),
            metadata: ModuleMetadata::default(),
        },
    );

    let topo_two = ModuleTopology::from_parts(ResolverGeneration(1), &universe, &unlinked_two, &sources);

    assert_ne!(topo_one.fingerprint, topo_two.fingerprint);
}

#[test]
fn topology_fingerprint_changes_on_module_kind_change() {
    let universe = ProjectUniverse::new();
    let mod_id = ModuleId::universe(path("target"));

    let mut unlinked_as_mod = BTreeMap::new();
    unlinked_as_mod.insert(
        mod_id.clone(),
        UnlinkedModuleInterface {
            id: mod_id.clone(),
            kind: ModuleKind::Module,
            declarations: BTreeMap::new(),
            exports: BTreeMap::new(),
            imports: Vec::new(),
            exposed_children: BTreeSet::new(),
            metadata: ModuleMetadata::default(),
        },
    );
    let sources = BTreeMap::new();

    let topo_mod = ModuleTopology::from_parts(ResolverGeneration(1), &universe, &unlinked_as_mod, &sources);

    let mut unlinked_as_pkg = unlinked_as_mod.clone();
    unlinked_as_pkg.get_mut(&mod_id).unwrap().kind = ModuleKind::Package;

    let topo_pkg = ModuleTopology::from_parts(ResolverGeneration(1), &universe, &unlinked_as_pkg, &sources);

    assert_ne!(topo_mod.fingerprint, topo_pkg.fingerprint);
}

#[test]
fn topology_detect_cycle_and_descendants() {
    let universe = ProjectUniverse::new();
    let root = ModuleId::universe(ModulePath::root());
    let a = ModuleId::universe(path("a"));
    let b = ModuleId::universe(path("b"));
    let c = ModuleId::universe(ModulePath::from_components(vec![component("a"), component("c")]));

    let unlinked = BTreeMap::from([
        (
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
        ),
        (
            a.clone(),
            UnlinkedModuleInterface {
                id: a.clone(),
                kind: ModuleKind::Package,
                declarations: BTreeMap::new(),
                exports: BTreeMap::new(),
                imports: Vec::new(),
                exposed_children: BTreeSet::new(),
                metadata: ModuleMetadata::default(),
            },
        ),
        (
            b.clone(),
            UnlinkedModuleInterface {
                id: b.clone(),
                kind: ModuleKind::Module,
                declarations: BTreeMap::new(),
                exports: BTreeMap::new(),
                imports: Vec::new(),
                exposed_children: BTreeSet::new(),
                metadata: ModuleMetadata::default(),
            },
        ),
        (
            c.clone(),
            UnlinkedModuleInterface {
                id: c.clone(),
                kind: ModuleKind::Module,
                declarations: BTreeMap::new(),
                exports: BTreeMap::new(),
                imports: Vec::new(),
                exposed_children: BTreeSet::new(),
                metadata: ModuleMetadata::default(),
            },
        ),
    ]);
    let sources = BTreeMap::new();

    let topo = ModuleTopology::from_parts(ResolverGeneration(1), &universe, &unlinked, &sources);

    // Test descendants
    let root_descendants = topo.descendants(&root);
    assert!(root_descendants.contains(&a));
    assert!(root_descendants.contains(&b));
    assert!(root_descendants.contains(&c));

    let a_descendants = topo.descendants(&a);
    assert_eq!(a_descendants, BTreeSet::from([c.clone()]));

    // Test cycle detection
    // Case 1: DAG (no cycle)
    let dag_edges = BTreeMap::from([
        (a.clone(), BTreeSet::from([b.clone()])),
        (b.clone(), BTreeSet::from([c.clone()])),
    ]);
    assert_eq!(topo.detect_cycle(&dag_edges), None);

    // Case 2: Cycle (a -> b -> c -> a)
    let cycle_edges = BTreeMap::from([
        (a.clone(), BTreeSet::from([b.clone()])),
        (b.clone(), BTreeSet::from([c.clone()])),
        (c.clone(), BTreeSet::from([a.clone()])),
    ]);
    let detected = topo.detect_cycle(&cycle_edges);
    assert!(detected.is_some());
    let cycle_path = detected.unwrap();
    assert_eq!(cycle_path.first(), cycle_path.last());
}

#[test]
fn import_resolution_product_deterministic_fingerprint() {
    let importer = ModuleId::universe(path("client"));
    let target = ModuleId::universe(path("server"));
    let written = ImportPathIdentity {
        written: "universe.server".to_string(),
        is_relative: false,
    };
    let consulted = BTreeSet::from([ModuleId::universe(ModulePath::root())]);
    let deps = ResolutionTopologyDependencies {
        consulted_packages: consulted,
        target_project: Some(target.project),
        target_module: Some(target.clone()),
    };

    let prod1 = ImportResolutionProduct::new(importer.clone(), written.clone(), Ok(target.clone()), deps.clone());
    let prod2 = ImportResolutionProduct::new(importer.clone(), written.clone(), Ok(target.clone()), deps.clone());

    assert_eq!(prod1.fingerprint, prod2.fingerprint);

    // Different target gives different fingerprint
    let other_target = ModuleId::universe(path("other"));
    let other_deps = ResolutionTopologyDependencies {
        consulted_packages: deps.consulted_packages.clone(),
        target_project: Some(other_target.project),
        target_module: Some(other_target.clone()),
    };
    let prod3 = ImportResolutionProduct::new(importer, written, Ok(other_target), other_deps);
    assert_ne!(prod1.fingerprint, prod3.fingerprint);
}

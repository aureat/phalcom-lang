use phalcom_common::range::SourceRange;
use phalcom_modules::interface::{LinkedExport, LinkedExportTarget, LinkedModuleInterface, UnlinkedModuleInterface};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::{BuiltinProject, ModuleComponent, ModuleId, ModuleKind, ModulePath, ModuleQueryFacade, ProjectUniverse, SourceId, SourceLocation};
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
    let root = ModuleId::builtin(BuiltinProject::Std, ModulePath::root());
    let child = ModuleId::builtin(BuiltinProject::Std, path("math"));

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
    resolved.insert((root.clone(), "std.math".to_string()), child.clone());
    let mut sources = BTreeMap::new();
    sources.insert(
        child.clone(),
        SourceLocation {
            source_id: SourceId("/workspace/src/math.ph".into()),
            display_path: "/workspace/src/math.ph".into(),
        },
    );

    let facade = ModuleQueryFacade::new(&universe, &unlinked, &linked, &resolved, &sources);
    let roots = facade.import_roots(&root);
    assert!(roots.contains_key(&component("std")));
    assert!(roots.contains_key(&component("universe")));
    assert_eq!(facade.import_children(&root, &ModulePath::root()), vec![child.clone()]);
    assert!(facade.public_exports(&child).unwrap().contains_key("answer"));
    assert_eq!(facade.resolved_import_target(&root, "std.math"), Some(&child));
    assert_eq!(facade.reverse_importers(&child), vec![root.clone()]);
    assert_eq!(
        facade.definition_source(&child).unwrap().display_path.to_string_lossy(),
        "/workspace/src/math.ph"
    );
}

#[test]
fn facade_rejects_unexposed_package_children() {
    let universe = ProjectUniverse::new();
    let root = ModuleId::builtin(BuiltinProject::Std, ModulePath::root());
    let child = ModuleId::builtin(BuiltinProject::Std, path("private"));
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

    let facade = ModuleQueryFacade::new(&universe, &unlinked, &linked, &resolved, &sources);
    assert!(facade.import_children(&root, &ModulePath::root()).is_empty());
}

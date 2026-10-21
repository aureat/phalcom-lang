use phalcom_modules::{ModuleId, ModulePath, ProjectIdentity, SyntheticProjectIdAllocator, UniverseSourceProvider};

#[test]
fn universe_functional_sources_parse_and_export_typed_computations() {
    let provider = UniverseSourceProvider::new();
    let cases: &[(&[&str], &[&str])] = &[
        (
            &["functional"],
            &[
                "State",
                "StateStep",
                "Reader",
                "Validation",
                "Lens",
                "Chain",
                "ChainIterator",
                "NonEmpty",
                "Partition",
                "Semigroup",
                "Monoid",
                "Additive",
                "Multiplicative",
                "AllTrue",
                "AnyTrue",
                "First",
                "Last",
                "Dual",
                "Endo",
                "Writer",
                "Journal",
                "StateResult",
                "ReaderResult",
                "Functor",
                "Applicative",
                "Monad",
                "Identity",
                "IdentityContext",
                "OptionContext",
                "ChainContext",
                "NonEmptyContext",
                "Effects",
                "Folds",
                "Fold",
                "FoldFactory",
                "Iso",
                "Optional",
                "Prism",
                "Either",
                "Arrow",
                "ArrowContext",
                "Profunctor",
            ],
        ),
        (&["functional", "state"], &["State", "StateStep"]),
        (&["functional", "reader"], &["Reader"]),
        (&["functional", "validation"], &["Validation"]),
        (&["functional", "lens"], &["Lens"]),
        (&["functional", "chain"], &["Chain", "ChainIterator", "NonEmpty", "Partition"]),
        (
            &["functional", "algebra"],
            &[
                "Semigroup",
                "Monoid",
                "Additive",
                "Multiplicative",
                "AllTrue",
                "AnyTrue",
                "First",
                "Last",
                "Dual",
                "Endo",
            ],
        ),
        (&["functional", "writer"], &["Writer"]),
        (&["functional", "journal"], &["Journal"]),
        (&["functional", "state_result"], &["StateResult"]),
        (&["functional", "reader_result"], &["ReaderResult"]),
        (
            &["functional", "higher_kinded"],
            &[
                "Functor",
                "Applicative",
                "Monad",
                "Identity",
                "IdentityContext",
                "OptionContext",
                "ChainContext",
                "NonEmptyContext",
            ],
        ),
        (&["functional", "effects"], &["Effects"]),
        (&["functional", "folds"], &["Folds"]),
        (&["functional", "fold"], &["Fold", "FoldFactory"]),
        (&["functional", "optics"], &["Iso", "Optional", "Prism"]),
        (&["functional", "either"], &["Either"]),
        (&["functional", "arrow"], &["Arrow", "ArrowContext", "Profunctor"]),
    ];
    for (components, exports) in cases {
        let path = ModulePath::from_components(
            components
                .iter()
                .map(|name| phalcom_modules::ModuleComponent::from_identifier(name).unwrap())
                .collect::<Vec<_>>(),
        );
        let id = ModuleId::universe(path);
        provider.load_parsed(&id).unwrap_or_else(|error| panic!("{id} must parse: {error}"));
        let interface = provider.load_interface(&id).unwrap_or_else(|error| panic!("{id} interface must load: {error}"));
        for name in *exports {
            assert!(interface.exports.contains_key(*name), "{id} must export {name}");
        }
    }
    // Exercise the modified prelude declarations without executing Universe.
    for components in [&["option", "option"][..], &["errors", "result"][..]] {
        let path = ModulePath::from_components(
            components
                .iter()
                .map(|name| phalcom_modules::ModuleComponent::from_identifier(name).unwrap())
                .collect::<Vec<_>>(),
        );
        provider.load_parsed(&ModuleId::universe(path)).expect("combinator source parses");
    }
}

#[test]
fn builtin_universe_provider_has_virtual_identity_and_exports() {
    let id = ModuleId::universe(ModulePath::root());
    let provider = UniverseSourceProvider::new();
    assert_eq!(provider.source_id(&id).unwrap().0.as_ref(), "phalcom://universe/");
    let interface = provider.load_interface(&id).unwrap();
    assert!(interface.exports.contains_key("Object"));
    assert_eq!(id.project, ProjectIdentity::Universe);
}

#[test]
fn universe_identity_is_disjoint_from_resolved_and_synthetic() {
    let mut allocator = SyntheticProjectIdAllocator;
    assert_ne!(
        ModuleId::universe_root().project,
        ProjectIdentity::from(phalcom_modules::ResolvedProjectId::from_raw(1))
    );
    assert_ne!(ModuleId::universe_root().project, ProjectIdentity::from(allocator.allocate()));
}

#[test]
fn builtin_universe_reflection_children_load() {
    let provider = UniverseSourceProvider::new();
    let children = [
        "module",
        "package_object",
        "project",
        "project_manifest",
        "package_info",
        "package_author",
        "package_requirement",
        "resolved_project_dependency",
        "module_dependency",
        "export_table",
        "export",
        "export_kind",
        "child_module_table",
        "module_identity",
        "package_identity",
        "project_identity",
        "uri",
        "selector",
        "message",
        "attribute",
    ];

    for child in children {
        let path = ModulePath::from_components(vec![
            phalcom_modules::ModuleComponent::from_identifier("reflection").unwrap(),
            phalcom_modules::ModuleComponent::from_identifier(child).unwrap(),
        ]);
        let id = ModuleId::universe(path);
        assert!(provider.contains(&id.path), "module path {id} should exist in provider");
        let iface = provider.load_interface(&id).unwrap_or_else(|_| panic!("interface for {id} should load"));
        assert_eq!(iface.kind, phalcom_modules::ModuleKind::Module);
        let src = provider.source_text(&id).unwrap_or_else(|_| panic!("source text for {id} should load"));
        assert!(!src.is_empty(), "source for {id} should not be empty");
    }
}

#[test]
fn reflection_leaf_interfaces_export_their_public_native_declarations() {
    let provider = UniverseSourceProvider::new();
    let cases: &[(&[&str], &[&str])] = &[
        (&["reflection", "selector"], &["Selector", "SelectorPattern"]),
        (&["reflection", "message"], &["Message"]),
    ];

    for (components, expected_exports) in cases {
        let path = ModulePath::from_components(
            components
                .iter()
                .map(|component| phalcom_modules::ModuleComponent::from_identifier(component).unwrap())
                .collect::<Vec<_>>(),
        );
        let id = ModuleId::universe(path);
        let interface = provider.load_interface(&id).unwrap_or_else(|_| panic!("interface for {id} should load"));

        for expected in *expected_exports {
            let export = interface
                .exports
                .get(*expected)
                .unwrap_or_else(|| panic!("{id} must publicly export {expected}"));
            assert_eq!(export.exported_name, *expected);
            assert_eq!(export.internal_name, *expected);
            assert_eq!(export.target, phalcom_modules::UnlinkedExportTarget::Local((*expected).to_owned()));
        }
    }
}

#[test]
fn universe_random_provider_registers_facade_and_source_modules() {
    let provider = UniverseSourceProvider::new();
    let random_path = ModulePath::from_components(vec![phalcom_modules::ModuleComponent::from_identifier("random").unwrap()]);
    let random = ModuleId::universe(random_path);
    assert!(provider.contains(&random.path));
    let interface = provider.load_interface(&random).expect("random facade interface loads");
    assert!(interface.kind.is_package_like());
    for export in [
        "Generator",
        "Xoshiro256PlusPlus",
        "Distribution",
        "UniformInt",
        "Bernoulli",
        "SecureRandom",
        "EntropyError",
    ] {
        assert!(interface.exports.contains_key(export), "random facade exports {export}");
    }

    for child in ["generator", "xoshiro256pp", "distribution", "uniform_int", "bernoulli", "secure"] {
        let child_path = ModulePath::from_components(vec![
            phalcom_modules::ModuleComponent::from_identifier("random").unwrap(),
            phalcom_modules::ModuleComponent::from_identifier(child).unwrap(),
        ]);
        let child_id = ModuleId::universe(child_path);
        assert!(provider.contains(&child_id.path), "random child {child_id} exists");
        assert!(!provider.source_text(&child_id).expect("random child source loads").is_empty());
    }
}


#[test]
fn canonical_universe_sources_parse_interfaces_and_link() {
    let provider = UniverseSourceProvider::new();
    let universe = std::sync::Arc::new(phalcom_modules::ProjectUniverse::new());
    let filesystem = phalcom_modules::FilesystemSourceProvider::new();
    let mut resolver =
        phalcom_modules::ModuleResolver::with_universe_source_provider(&universe, &filesystem, provider.clone());
    let mut interfaces = std::collections::BTreeMap::new();

    for node in provider.nodes() {
        let path = ModulePath::from_components(
            node.path
                .iter()
                .map(|component| phalcom_modules::ModuleComponent::from_identifier(component).expect("canonical Universe component"))
                .collect::<Vec<_>>(),
        );
        let id = ModuleId::universe(path);
        provider
            .load_parsed(&id)
            .unwrap_or_else(|error| panic!("{id} canonical source must parse: {error}"));
        let interface = provider
            .load_interface(&id)
            .unwrap_or_else(|error| panic!("{id} canonical interface must load: {error}"));
        interfaces.insert(id, interface);
    }

    let mut resolved = std::collections::BTreeMap::new();
    for (module, interface) in &interfaces {
        for import in &interface.imports {
            let path = match import {
                phalcom_modules::ImportSurface::Module(decl) => &decl.path,
                phalcom_modules::ImportSurface::Selective(decl) => &decl.path,
                phalcom_modules::ImportSurface::ReExport(decl) => &decl.path,
            };
            let target = resolver
                .resolve_import(module, path)
                .unwrap_or_else(|error| panic!("{module}::{path} canonical import must resolve: {error}"));
            resolved.insert((module.clone(), path.to_string()), target.id);
        }
    }

    phalcom_modules::ModuleLinker::new(universe, interfaces)
        .link_all(ModuleId::universe_root(), &resolved)
        .expect("canonical Universe interfaces must link");
}

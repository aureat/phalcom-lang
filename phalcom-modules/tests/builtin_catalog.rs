use phalcom_ast::ast::DependencyDecl;
use phalcom_ast::parser::parse;
use phalcom_modules::{
UniverseSourceProvider, ModuleComponent, ModuleId, ModuleKind, ModuleLoadError, ModulePath, ModuleResolutionError, UNIVERSE_NODES,
};

fn make_universe_id(path: &[&str]) -> ModuleId {
    let components: Vec<ModuleComponent> = path.iter().map(|s| ModuleComponent::from_identifier(s).expect("valid identifier")).collect();
    ModuleId::universe( ModulePath::from_components(components))
}

/// BCAT-01 — Every path in UNIVERSE_NODES has a source_text arm (catalog/source parity)
#[test]
fn bcat_01_universe_nodes_source_parity() {
    let provider = UniverseSourceProvider::new();
    for node in UNIVERSE_NODES {
        let id = make_universe_id(node.path);
        let src = provider.source_text(&id);
        assert!(src.is_ok(), "source_text failed for node with path {:?}: {:?}", node.path, src.err());
        let text = src.unwrap();
        assert!(!text.is_empty(), "empty source text for path {:?}", node.path);
    }
}

/// BCAT-02 — Every path exposed in reflection/package.ph has a node in UNIVERSE_NODES
#[test]
fn bcat_02_reflection_exposed_children_in_nodes() {
    let provider = UniverseSourceProvider::new();
    let reflection_pkg_id = make_universe_id(&["reflection"]);
    let src = provider.source_text(&reflection_pkg_id).expect("reflection package source");
    let parse_result = parse(&src, 0);
    assert!(
        parse_result.errors.is_empty(),
        "parse errors in reflection/package.ph: {:?}",
        parse_result.errors
    );

    let mut exposed = Vec::new();
    for dep in &parse_result.program.preamble.dependencies {
        if let DependencyDecl::Expose(expose_decl) = dep {
            exposed.push(expose_decl.child.name.clone());
        }
    }

    assert_eq!(exposed.len(), 22, "expected 22 expose declarations in reflection/package.ph");

    for child in &exposed {
        let path = &["reflection", child.as_str()];
        let node = UNIVERSE_NODES.iter().find(|n| n.path == path);
        assert!(
            node.is_some(),
            "exposed child {:?} has no corresponding UniverseNodeSpec in UNIVERSE_NODES",
            path
        );
        let node = node.unwrap();
        let expected_kind = if child == "typing" { ModuleKind::Package } else { ModuleKind::Module };
        assert_eq!(node.kind, expected_kind, "expected {:?} for exposed child {:?}", expected_kind, path);
    }
}

/// BCAT-03 — contains returns false for a path not in the catalog
#[test]
fn bcat_03_contains_false_for_missing_path() {
    let provider = UniverseSourceProvider::new();
    let path = ModulePath::from_components(vec![
        ModuleComponent::from_identifier("reflection").unwrap(),
        ModuleComponent::from_identifier("nonexistent").unwrap(),
    ]);
    assert!(!provider.contains(&path));
}

/// BCAT-04 — load_interface for a missing path returns ModuleNotFound
#[test]
fn bcat_04_load_interface_missing_path_returns_module_not_found() {
    let provider = UniverseSourceProvider::new();
    let id = make_universe_id(&["reflection", "nonexistent"]);
    let result = provider.load_interface(&id);
    assert!(
        matches!(result, Err(ModuleLoadError::Resolution(ModuleResolutionError::ModuleNotFound(_)))),
        "expected ModuleNotFound error, got {:?}",
        result
    );
}

/// BCAT-05 — load_interface for universe.errors.unsupported returns an interface with unsupported in exports
#[test]
fn bcat_05_load_interface_selector_exports_selector() {
    let provider = UniverseSourceProvider::new();
    let id = make_universe_id(&["errors", "unsupported"]);
    let iface = provider.load_interface(&id).expect("load unsupported interface");
    assert!(
        iface.exports.contains_key("unsupported"),
        "unsupported missing from exports of universe.errors.unsupported: {:?}",
        iface.exports.keys().collect::<Vec<_>>()
    );
}

/// BCAT-06 — load_interface for universe (root) has all native bindings from UNIVERSE_BINDINGS in exports
#[test]
fn bcat_06_load_interface_root_exports_universe_bindings() {
    let provider = UniverseSourceProvider::new();
    let id = make_universe_id(&[]);
    let iface = provider.load_interface(&id).expect("load universe root interface");

    for binding in phalcom_native_meta::UNIVERSE_BINDINGS.iter().filter(|b| b.exported) {
        assert!(
            iface.exports.contains_key(binding.name),
            "native binding {:?} missing from universe root exports",
            binding.name
        );
    }
}

/// BCAT-07 — source_text for a pure-native package root returns Ok (no source panic)
#[test]
fn bcat_07_source_text_root_ok() {
    let provider = UniverseSourceProvider::new();
    let id = make_universe_id(&[]);
    let src = provider.source_text(&id).expect("root source text");
    assert!(src.contains("expose"), "root source text should contain expose declarations");
}

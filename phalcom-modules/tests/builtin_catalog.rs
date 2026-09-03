use phalcom_ast::ast::DependencyDecl;
use phalcom_ast::parser::parse;
use phalcom_modules::{
    ModuleComponent, ModuleId, ModuleKind, ModuleLinker, ModuleLoadError, ModulePath, ModuleResolutionError, ProjectUniverse, SymbolId, UNIVERSE_NODES,
    UniverseSourceProvider,
};
use std::collections::BTreeMap;
use std::sync::Arc;

fn make_universe_id(path: &[&str]) -> ModuleId {
    let components: Vec<ModuleComponent> = path.iter().map(|s| ModuleComponent::from_identifier(s).expect("valid identifier")).collect();
    ModuleId::universe(ModulePath::from_components(components))
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

/// BCAT-08 — Root convenience aliases preserve their defining source owner.
#[test]
fn bcat_08_root_alias_does_not_create_synthetic_declaration() {
    let provider = UniverseSourceProvider::new();
    let mut root = provider.load_interface(&make_universe_id(&[])).expect("root interface");
    let root_id = root.id.clone();
    let aliases = [
        ("Int", make_universe_id(&["scalar", "number"])),
        ("List", make_universe_id(&["collections", "list"])),
        ("Option", make_universe_id(&["option", "option"])),
        ("Result", make_universe_id(&["errors", "result"])),
        ("Ordering", make_universe_id(&["object", "ordering"])),
    ];
    root.imports.clear();
    root.exports.retain(|name, _| aliases.iter().any(|(alias, _)| name == alias));

    let mut interfaces = BTreeMap::from([(root_id.clone(), root)]);
    for (name, owner) in &aliases {
        let source = provider.load_interface(owner).expect("canonical source interface");
        assert!(source.declarations.contains_key(*name), "{name} must be declared by {owner}");
        interfaces.insert(owner.clone(), source);
    }

    let root = &interfaces[&root_id];
    for (name, owner) in &aliases {
        assert!(root.exports.contains_key(*name), "root convenience alias {name} must remain exported");
        assert!(
            !root.declarations.contains_key(*name),
            "root convenience alias {name} must not fabricate universe::<root>::{name}"
        );
        assert!(
            matches!(
                root.exports[*name].target,
                phalcom_modules::UnlinkedExportTarget::CanonicalDeclaration { ref module, name: ref target_name }
                    if module == owner && target_name == *name
            ),
            "root {name} export must target its canonical source declaration rather than a local root binding"
        );
    }

    let linked = ModuleLinker::new(Arc::new(ProjectUniverse::new()), interfaces)
        .link(root_id.clone(), &BTreeMap::new())
        .expect("root aliases and source owners should link");
    for (name, owner) in aliases {
        assert_eq!(
            linked.modules[&root_id].interface.exports[name].symbol(),
            Some(&SymbolId {
                module: owner,
                name: name.into(),
            }),
            "root convenience lookup must preserve its source-owner identity"
        );
    }
}

/// BCAT-09 — A Universe module's source declarations are private until source exports them.
#[test]
fn bcat_09_non_root_source_declarations_are_not_implicitly_exported() {
    let provider = UniverseSourceProvider::new();
    let iface = provider
        .load_interface(&make_universe_id(&["scalar", "number"]))
        .expect("canonical number interface");

    assert!(iface.declarations.contains_key("Int"), "source declaration must remain discoverable");
    assert!(
        !iface.exports.contains_key("Int"),
        "universe.scalar.number::Int is not public without an ordinary source export"
    );
}

use phalcom_modules::{
    ModuleComponent, ModuleId, ModulePath, ProjectIdentity, ResolvedProjectId, UNIVERSE_NODES, universe_module_from_uri, universe_module_uri,
};

fn node_id(path: &[&str]) -> ModuleId {
    let components = path
        .iter()
        .map(|component| ModuleComponent::from_identifier(component).expect("catalog component is canonical"))
        .collect::<Vec<_>>();
    ModuleId::universe(ModulePath::from_components(components))
}

#[test]
fn universe_uri_round_trips_every_catalog_node() {
    for node in UNIVERSE_NODES {
        let id = node_id(node.path);
        let uri = universe_module_uri(&id).expect("Universe node has canonical URI");
        assert_eq!(universe_module_from_uri(&uri), Some(id), "URI did not round-trip: {uri}");
    }
}

#[test]
fn universe_uri_uses_one_canonical_root_form() {
    let root = ModuleId::universe_root();
    assert_eq!(universe_module_uri(&root).as_deref(), Some("phalcom://universe/"));
    assert_eq!(universe_module_from_uri("phalcom://universe/"), Some(root));
    assert_eq!(universe_module_from_uri("phalcom://universe"), None);
}

#[test]
fn universe_uri_rejects_legacy_and_malformed_forms() {
    for uri in [
        "phalcom://core/",
        "phalcom://std/",
        "phalcom://other/option",
        "phalcom://universe//option",
        "phalcom://universe/option/",
        "phalcom://universe/Option",
        "phalcom://universe/option?query",
        "phalcom://universe/option#fragment",
        "phalcom://user@universe/option",
        "phalcom://universe:443/option",
    ] {
        assert_eq!(universe_module_from_uri(uri), None, "accepted malformed URI: {uri}");
    }
}

#[test]
fn universe_uri_encoder_rejects_non_universe_projects() {
    let id = ModuleId::resolved(ResolvedProjectId::from_raw(1), ModulePath::root());
    assert_eq!(id.project, ProjectIdentity::Resolved(ResolvedProjectId::from_raw(1)));
    assert_eq!(universe_module_uri(&id), None);
}

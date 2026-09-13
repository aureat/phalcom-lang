use phalcom_ast::parser::parse;
use phalcom_modules::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::identity::{DataComponentId, DeclarationId, SemanticTargetId};
use phalcom_semantic::source_index::{SourceIndexContext, SourceSiteKind, build_source_scope_index};

fn module() -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(9910),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("shapes").unwrap()]),
    )
}

fn assert_component_targets(source: &str, names: &[&str]) {
    let parsed = parse(source, 0);
    assert!(parsed.errors.is_empty(), "{:#?}", parsed.errors);

    let module = module();
    let owner = DeclarationId::new(module.clone(), "Point".into());
    let index = build_source_scope_index(module, &parsed.program, &SourceIndexContext::default());

    for (component_index, name) in names.iter().enumerate() {
        let id = DataComponentId::new(owner.clone(), component_index as u32);
        let declaration_site = index
            .sites
            .values()
            .find(|site| matches!(&site.kind, SourceSiteKind::DataComponent(candidate) if candidate == &id))
            .unwrap_or_else(|| panic!("missing source site for data component {name}"));

        assert_eq!(
            index.targets.get(&declaration_site.id),
            Some(&SemanticTargetId::DataComponent(id)),
            "data component {name} must publish its canonical DataComponentId"
        );
        assert_eq!(
            &source[declaration_site.range.start..declaration_site.range.end],
            *name,
            "component source range must identify its authored name"
        );
    }
}

#[test]
fn tuple_data_components_publish_canonical_source_targets() {
    assert_component_targets("data Point(_ x: Int, _ y: Int)\n", &["x", "y"]);
}

#[test]
fn record_data_components_publish_canonical_source_targets() {
    assert_component_targets("data Point { x: Int, y: Int }\n", &["x", "y"]);
}

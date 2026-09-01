use phalcom_native_surface::NATIVE_SURFACES;
use phalcom_semantic::core_surface::*;
use phalcom_semantic::declarations::bootstrap_universe_declarations;
use phalcom_semantic::identity::{DeclarationId, ModuleId};
use phalcom_semantic::types::SimpleTypeResolver;
use phalcom_semantic::types::store::TypeStore;

#[test]
fn test_native_surface_conformance() {
    let mut store = TypeStore::new();
    let core_mod = ModuleId::universe_root();
    let universe_resolver = |key: phalcom_native_meta::UniverseKey| -> DeclarationId { DeclarationId::new(ModuleId::universe_root(), key.name().into()) };
    let declarations = bootstrap_universe_declarations(&mut store, &universe_resolver);

    let resolver = SimpleTypeResolver::new();

    let report = validate_native_surface_conformance(&mut store, &declarations, &resolver, &core_mod);
    assert_eq!(
        report.failures,
        Vec::<String>::new(),
        "Native surface conformance failed: {:?}",
        report.failures
    );
    assert_eq!(report.total_surfaces, report.resolved_surfaces);
    assert!(
        report.total_surfaces >= 30,
        "Expected at least 30 native surfaces, got {}",
        report.total_surfaces
    );
}

#[test]
fn test_presentation_ir_and_virtual_source() {
    let empty_sources = Vec::new();
    let merged = merge_surfaces(&empty_sources, NATIVE_SURFACES);
    assert!(!merged.is_empty());

    let bool_merged = merged.iter().find(|c| c.name == "Bool").expect("Bool class in merged surfaces");
    let presentation = ClassPresentation::from_merged(bool_merged);

    let md = presentation.render_markdown();
    assert!(md.contains("## class Bool"));
    assert!(md.contains("- not · `native · intrinsic · pure`"));

    let virt = presentation.render_virtual_source();
    assert!(virt.contains("class Bool {"));
    assert!(virt.contains("@native"));
    assert!(virt.contains("@intrinsic(BoolNot)"));
    assert!(virt.contains("@pure"));
}

#[test]
fn test_native_enum_merge_preserves_declarations() {
    let source = r#"
@native
enum Option<T> {
    @variant Some(_ value: T)
    @variant None

    @native
    match(some: Dynamic, none: Dynamic) -> Dynamic
}
"#;
    let program = phalcom_ast::parser::parse_source(source, 0).expect("parse option source");
    let core_mod = ModuleId::universe_root();
    let decls = extract_source_declarations(&core_mod, &program);
    let merged = merge_surfaces(&decls, NATIVE_SURFACES);

    let opt_merged = merged
        .iter()
        .find(|d| d.name == "Option")
        .expect("Option enum in merged surfaces");
    assert!(opt_merged.source_enum().is_some());
    assert_eq!(opt_merged.source_enum().unwrap().variants.len(), 2);
}

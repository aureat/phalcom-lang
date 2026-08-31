use std::sync::Arc;

use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::enum_semantics::VariantShape;
use phalcom_semantic::identity::{DeclarationId, VariantId};

#[test]
fn singleton_and_constructor_variants_have_distinct_identities_in_one_family() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
enum Weird {
  @variant Marker
  @variant Marker()
  @variant Marker(_ value: Int)
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let owner = DeclarationId::new(module, "Weird".into());

    let singleton = VariantId::new(owner.clone(), Selector::getter("Marker").expect("singleton selector"));
    let nullary = VariantId::new(owner.clone(), Selector::method("Marker", []).expect("nullary selector"));
    let unary = VariantId::new(owner.clone(), Selector::method("Marker", [SelectorSlot::Positional]).expect("unary selector"));

    assert_ne!(singleton, nullary);
    assert_ne!(nullary, unary);
    assert_ne!(singleton, unary);
    assert_eq!(singleton.family(), nullary.family());
    assert_eq!(nullary.family(), unary.family());

    let singleton_info = analysis.snapshot.enum_semantics.variant_info(&singleton).expect("singleton metadata");
    assert_eq!(singleton_info.shape, VariantShape::Singleton);
    assert!(singleton_info.constructor.is_none());

    let nullary_info = analysis.snapshot.enum_semantics.variant_info(&nullary).expect("nullary metadata");
    assert_eq!(nullary_info.shape, VariantShape::Constructor);
    assert_eq!(nullary_info.constructor.as_ref().expect("nullary constructor").parameters.len(), 0);

    let unary_info = analysis.snapshot.enum_semantics.variant_info(&unary).expect("unary metadata");
    assert_eq!(unary_info.shape, VariantShape::Constructor);
    assert_eq!(unary_info.constructor.as_ref().expect("unary constructor").parameters.len(), 1);
    assert_eq!(unary_info.fields[0].local_name.as_ref(), "value");
}

#[test]
fn variant_payload_field_identity_uses_declaration_order() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
enum Pair {
  @variant Pair(_ first: Int, _ second: String)
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let owner = DeclarationId::new(module, "Pair".into());
    let variant = VariantId::new(
        owner,
        Selector::method("Pair", [SelectorSlot::Positional, SelectorSlot::Positional]).expect("pair selector"),
    );
    let info = analysis.snapshot.enum_semantics.variant_info(&variant).expect("pair metadata");

    assert_eq!(info.fields.len(), 2);
    assert_eq!(info.fields[0].id.index, 0);
    assert_eq!(info.fields[1].id.index, 1);
    assert_eq!(info.fields[0].local_name.as_ref(), "first");
    assert_eq!(info.fields[1].local_name.as_ref(), "second");
}

use std::sync::Arc;

use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::enum_semantics::VariantShape;
use phalcom_semantic::identity::{DeclarationId, VariantId};

#[test]
fn singleton_and_constructor_variants_have_distinct_identities_in_one_family() {
    let module = ModuleId::universe_root();
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
    let module = ModuleId::universe_root();
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

#[test]
fn adt_variant_01_singleton_is_getter_shaped_without_payload() {
    let case = super::support::analyze_adt("enum Animal { @variant Dog }\n");
    let dog = case.variant("Animal", Selector::getter("Dog").expect("Dog getter"));
    assert_eq!(dog.shape, VariantShape::Singleton);
    assert!(dog.fields.is_empty());
    assert!(dog.constructor.is_none());
}

#[test]
fn adt_variant_02_nullary_is_callable_with_zero_parameters() {
    let case = super::support::analyze_adt("enum Animal { @variant Dog() }\n");
    let dog = case.variant("Animal", Selector::method("Dog", []).expect("Dog method"));
    assert_eq!(dog.shape, VariantShape::Constructor);
    assert_eq!(dog.fields.len(), 0);
    assert_eq!(dog.constructor.as_ref().expect("constructor").parameters.len(), 0);
}

#[test]
fn adt_variant_03_singleton_and_nullary_have_distinct_exact_cases() {
    let case = super::support::analyze_adt("enum Animal { @variant Dog @variant Dog() }\n");
    let singleton = case.variant("Animal", Selector::getter("Dog").expect("Dog getter"));
    let nullary = case.variant("Animal", Selector::method("Dog", []).expect("Dog method"));
    assert_ne!(singleton.id, nullary.id);
    assert_ne!(singleton.exact_case_template, nullary.exact_case_template);
    assert_eq!(singleton.family, nullary.family);
}

#[test]
fn adt_variant_04_positional_payload_keeps_local_name_out_of_selector_identity() {
    let case = super::support::analyze_adt("enum Animal { @variant Dog(_ name: String) }\n");
    let dog = case.variant("Animal", Selector::method("Dog", [SelectorSlot::Positional]).expect("Dog selector"));
    assert_eq!(dog.fields[0].local_name.as_ref(), "name");
    assert_eq!(dog.fields[0].external_label, None);
    assert_eq!(dog.id.selector, Selector::method("Dog", [SelectorSlot::Positional]).expect("Dog selector"));
}

#[test]
fn adt_variant_05_labeled_payload_uses_external_label_in_selector() {
    let case = super::support::analyze_adt("enum Animal { @variant Dog(named age: Int) }\n");
    let dog = case.variant("Animal", Selector::method("Dog", [SelectorSlot::Label("named".into())]).expect("Dog selector"));
    assert_eq!(dog.fields[0].local_name.as_ref(), "age");
    assert_eq!(dog.fields[0].external_label.as_deref(), Some("named"));
}

#[test]
fn adt_variant_06_mixed_payload_field_ids_follow_selector_slots() {
    let case = super::support::analyze_adt("enum Animal { @variant Dog(_ name: String, named age: Int) }\n");
    let dog = case.variant(
        "Animal",
        Selector::method("Dog", [SelectorSlot::Positional, SelectorSlot::Label("named".into())]).expect("Dog selector"),
    );
    assert_eq!(dog.fields.len(), 2);
    assert_eq!(dog.fields[0].id.index, 0);
    assert_eq!(dog.fields[1].id.index, 1);
    assert_eq!(dog.fields[1].external_label.as_deref(), Some("named"));
}

#[test]
fn adt_variant_07_all_same_base_variants_share_one_family_identity() {
    let case = super::support::analyze_adt("enum Animal { @variant Dog @variant Dog() @variant Dog(_ name: String) }\n");
    let family = case.family_id("Animal", "Dog");
    let info = case.enum_info("Animal");
    assert_eq!(info.variant_families.as_ref(), [family.clone()]);
    assert!(info.variants.iter().all(|variant| variant.family().as_ref() == Some(&family)));
}

#[test]
#[ignore = "GATED: external-module visibility fixture is required"]
fn adt_variant_08_private_variant_name_is_not_explicitly_acquirable() {
    let _ = super::support::analyze_adt("enum Animal { @variant _Dog }\n");
    panic!("external visibility fixture must assert inaccessible explicit acquisition");
}

#[test]
#[ignore = "GATED: cross-module construction and match visibility fixture is required"]
fn adt_variant_09_construction_visibility_is_independent_from_match_universe() {
    let _ = super::support::analyze_adt("enum Animal { @variant Dog }\n");
    panic!("cross-module visibility fixture must assert construction and elimination separately");
}

#[test]
#[ignore = "GATED: payload visibility fixture is required"]
fn adt_variant_10_private_payload_rejects_projection_but_allows_wildcard_ignore() {
    let _ = super::support::analyze_adt("enum Animal { @variant Dog(_ secret: Int) }\n");
    panic!("payload visibility fixture must assert explicit projection and wildcard behavior");
}

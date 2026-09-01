//! Closed-enum behavior and exact-case dispatch scenarios.

use super::support::analyze_adt;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::identity::{CallableOwnerId, DeclarationId, VariantId};

#[test]
fn adt_beh_01_enum_wide_method_is_published_as_root_default_and_requirement() {
    let case = analyze_adt(
        r#"
enum Shape {
    describe() -> String { "shape" }
    area() -> Float
    @variant Circle {
        area() -> Float { 3.14 }
    }
    @variant Square {
        area() -> Float { 1.0 }
    }
}
"#,
    );
    let owner = DeclarationId::new(ModuleId::universe_root(), "Shape".into());
    let requirements = case
        .analysis
        .snapshot
        .enum_requirements
        .requirements
        .get(&owner)
        .expect("enum requirement table entry");
    let req_selector = Selector::method("area", []).expect("area selector");
    assert!(requirements.iter().any(|requirement| requirement.id.selector == req_selector));

    let default_selector = Selector::method("describe", []).expect("describe selector");
    let default_sig = case
        .analysis
        .snapshot
        .callable_signatures
        .iter()
        .find(|(_, sig)| sig.owner == owner && sig.callable.selector == default_selector)
        .map(|(_, sig)| sig)
        .expect("root default callable signature");
    assert_eq!(default_sig.callable.declaration_owner(), &owner);
}

#[test]
fn adt_beh_02_variant_override_has_exact_case_dispatch_target() {
    let case = analyze_adt(
        r#"
enum Shape {
    describe() -> String { "shape" }
    @variant Circle {
        describe() -> String { "circle" }
    }
    @variant Square
}
"#,
    );
    let owner = DeclarationId::new(ModuleId::universe_root(), "Shape".into());
    let circle_variant = VariantId::new(owner.clone(), Selector::getter("Circle").unwrap());
    let circle = case
        .analysis
        .snapshot
        .callable_analyses
        .keys()
        .find(|callable| callable.owner == CallableOwnerId::Variant(circle_variant.clone()) && callable.selector == Selector::method("describe", []).unwrap())
        .expect("exact-case override callable");
    assert_eq!(circle.owner, CallableOwnerId::Variant(circle_variant));
}

#[test]
fn adt_beh_03_case_only_method_is_not_promised_by_root_type() {
    let case = analyze_adt(
        r#"
enum Shape {
    @variant Circle {
        draw() -> String { "circle" }
    }
    @variant Square
}
"#,
    );
    let owner = DeclarationId::new(ModuleId::universe_root(), "Shape".into());
    let requirements = case
        .analysis
        .snapshot
        .enum_requirements
        .requirements
        .get(&owner)
        .expect("enum requirement table entry");
    assert!(
        !requirements
            .iter()
            .any(|requirement| requirement.id.selector == Selector::method("draw", []).unwrap())
    );
}

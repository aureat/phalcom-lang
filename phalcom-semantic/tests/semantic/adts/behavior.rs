//! Closed-enum behavior and exact-case dispatch scenarios.

use super::support::analyze_adt;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::identity::DeclarationId;

#[test]
fn adt_beh_01_enum_wide_method_is_published_as_root_requirement() {
    let case = analyze_adt(
        r#"
enum Shape {
    describe() -> String { "shape" }
    @variant Circle
    @variant Square
}
"#,
    );
    let owner = DeclarationId::new(ModuleId::core(), "Shape".into());
    let requirements = case
        .analysis
        .snapshot
        .enum_requirements
        .requirements
        .get(&owner)
        .expect("enum requirement table entry");
    let selector = Selector::method("describe", []).expect("describe selector");
    assert!(requirements.iter().any(|requirement| requirement.id.selector == selector));
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
    let owner = DeclarationId::new(ModuleId::core(), "Shape".into());
    let circle = case
        .analysis
        .snapshot
        .callable_analyses
        .keys()
        .find(|callable| callable.owner == owner && callable.selector == Selector::method("describe", []).unwrap())
        .expect("exact-case override callable");
    assert_eq!(circle.owner, owner);
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
    let owner = DeclarationId::new(ModuleId::core(), "Shape".into());
    let requirements = case
        .analysis
        .snapshot
        .enum_requirements
        .requirements
        .get(&owner)
        .expect("enum requirement table entry");
    assert!(!requirements.iter().any(|requirement| requirement.id.selector == Selector::method("draw", []).unwrap()));
}

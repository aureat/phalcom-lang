//! Explicit associated lookup and family capability scenarios.

use super::super::support::analyze_adt;
use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_semantic::checker::AssociatedResolutionKind;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};

fn resolution<'a>(case: &'a super::super::support::AdtCase, text: &str) -> &'a phalcom_semantic::checker::AssociatedResolution {
    case.analysis
        .snapshot
        .callable_analyses
        .values()
        .find_map(|callable| {
            callable.associated_resolutions.iter().find_map(|(expression_id, associated)| {
                let expression = callable.expressions.get(expression_id)?;
                (case.source.get(expression.range.start..expression.range.end) == Some(text)).then_some(associated)
            })
        })
        .unwrap_or_else(|| panic!("missing associated resolution for {text:?}"))
}

#[test]
fn adt_assoc_01_exact_singleton_lookup_resolves_variant_id() {
    let case = analyze_adt("enum Animal { @variant Dog }\nclass Test { run() { Animal::Dog } }\n");
    assert!(matches!(resolution(&case, "Animal::Dog").kind, AssociatedResolutionKind::ExactValue { .. }));
}

#[test]
fn adt_assoc_02_exact_nullary_lookup_keeps_callable_selector_kind() {
    let case = analyze_adt("enum Animal { @variant Dog() }\n");
    let dog = case.variant("Animal", Selector::method("Dog", []).expect("Dog"));
    assert_eq!(dog.id.selector.kind, phalcom_common::selector::SelectorKind::Method);
}

#[test]
fn adt_assoc_03_exact_payload_constructor_resolves_one_member() {
    let case = analyze_adt("enum Animal { @variant Dog(_ name: String) }\nclass Test { run() { Animal::Dog(\"rex\") } }\n");
    assert!(matches!(
        resolution(&case, "Animal::Dog(\"rex\")").kind,
        AssociatedResolutionKind::StaticInvoke { .. }
    ));
}

#[test]
fn adt_assoc_04_whole_family_capture_publishes_exact_member_ids() {
    let case = analyze_adt("enum Animal { @variant Dog @variant Dog() @variant Cat }\nclass Test { run() { Animal::Dog::* } }\n");
    let AssociatedResolutionKind::Family { members, .. } = &resolution(&case, "Animal::Dog::*").kind else {
        panic!("expected family resolution");
    };
    assert_eq!(members.len(), 2);
    assert!(
        members
            .iter()
            .all(|member| matches!(member.member, phalcom_semantic::associated::AssociatedMemberId::Variant(_)))
    );
}

#[test]
#[ignore = "RED: callable-family selector-kind filtering remains incomplete"]
fn adt_assoc_05_callable_family_excludes_singleton_member() {
    let case = analyze_adt("enum Animal { @variant Dog @variant Dog() @variant Cat }\nclass Test { run() { Animal::Dog::* } }\n");
    let AssociatedResolutionKind::Family { members, .. } = &resolution(&case, "Animal::Dog::*").kind else {
        panic!("expected family resolution");
    };
    assert_eq!(members.len(), 2);
}

#[test]
#[ignore = "RED: static family invocation lowering remains incomplete"]
fn adt_assoc_06_static_family_invocation_selects_exact_member() {
    let case = analyze_adt("enum Animal { @variant Dog(_ name: String) @variant Dog() }\nclass Test { run() { Animal::Dog(\"rex\") } }\n");
    assert!(matches!(
        resolution(&case, "Animal::Dog(\"rex\")").kind,
        AssociatedResolutionKind::StaticInvoke { .. }
    ));
}

#[test]
#[ignore = "RED: dynamic family routing remains incomplete"]
fn adt_assoc_07_dynamic_family_pack_routes_by_shape() {
    let case = analyze_adt("enum Animal { @variant Dog @variant Dog() @variant Cat }\nclass Test { run() { Animal::Dog::* } }\n");
    let AssociatedResolutionKind::Family { members, .. } = &resolution(&case, "Animal::Dog::*").kind else {
        panic!("expected dynamic family surface");
    };
    assert!(!members.is_empty());
}

#[test]
#[ignore = "RED: positional rest family routing remains incomplete"]
fn adt_assoc_08_positional_rest_routes_matching_candidate_shapes() {
    let case = analyze_adt("enum Animal { @variant Dog(_ age: Int) @variant Dog(_ age: Int, breed: String) }\nclass Test { run() { Animal::Dog::* } }\n");
    let AssociatedResolutionKind::Family { members, .. } = &resolution(&case, "Animal::Dog::*").kind else {
        panic!("expected positional-rest family surface");
    };
    assert_eq!(members.len(), 2);
}

#[test]
#[ignore = "RED: labeled rest family routing remains incomplete"]
fn adt_assoc_09_labeled_rest_routes_matching_candidate_shapes() {
    let case =
        analyze_adt("enum Animal { @variant Dog(named age: Int) @variant Dog(named age: Int, breed: String) }\nclass Test { run() { Animal::Dog::* } }\n");
    let AssociatedResolutionKind::Family { members, .. } = &resolution(&case, "Animal::Dog::*").kind else {
        panic!("expected labeled-rest family surface");
    };
    assert_eq!(members.len(), 2);
}

#[test]
fn adt_assoc_10_generic_family_metadata_keeps_owner_specialization() {
    let case =
        analyze_adt("enum Option<T> { @variant Some(_ value: T) -> Option<T> @variant None -> Option<T> }\nclass Test { run() { Option<Int>::Some::* } }\n");
    let family = case.associated_family("Option", "Some");
    assert!(!family.members.is_empty());
}

#[test]
#[ignore = "GATED: cross-module inherited ADT family fixture is required"]
fn adt_assoc_11_inherited_family_keeps_lookup_and_definition_owners() {
    let case = analyze_adt("enum Animal { @variant Dog @variant Cat }\nclass Test { run() { Animal::Dog::* } }\n");
    let family = case.associated_family("Animal", "Dog");
    assert!(
        family
            .members
            .iter()
            .all(|member| matches!(member, phalcom_semantic::associated::AssociatedMemberId::Variant(_)))
    );
}

#[test]
fn adt_assoc_12_exact_selector_miss_does_not_select_nearest_shape() {
    let case = analyze_adt("enum Animal { @variant Dog(_ name: String) }\nclass Test { run() { Animal::Dog() } }\n");
    assert!(
        case.diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::AssociatedCallShapeMissing || diagnostic.code == DiagnosticCode::AssociatedMemberMissing)
    );
}

#[test]
fn adt_assoc_13_wrong_call_shape_reports_diagnostic() {
    let case = analyze_adt("enum Animal { @variant Dog(_ name: String) }\nclass Test { run() { Animal::Dog(1, 2) } }\n");
    assert!(
        case.diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::AssociatedCallShapeMissing || diagnostic.code == DiagnosticCode::AssociatedMemberMissing)
    );
}

#[test]
#[ignore = "GATED: cross-module associated visibility fixture is required"]
fn adt_assoc_14_private_member_is_not_explicitly_acquirable() {
    let case = analyze_adt("enum Animal { @variant Dog @variant Cat }\nclass Test { run() { Animal::Dog } }\n");
    assert!(matches!(resolution(&case, "Animal::Dog").kind, AssociatedResolutionKind::ExactValue { .. }));
}

#[test]
#[ignore = "GATED: frozen-capability hierarchy fixture is required"]
fn adt_assoc_15_frozen_family_does_not_acquire_later_members() {
    let case = analyze_adt("enum Animal { @variant Dog @variant Cat }\nclass Test { run() { Animal::Dog::* } }\n");
    let family = case.associated_family("Animal", "Dog");
    assert_eq!(family.members.len(), 1);
}

#[test]
#[ignore = "GATED: capability escape visibility fixture is required"]
fn adt_assoc_16_family_value_does_not_escape_member_visibility() {
    let case = analyze_adt("enum Animal { @variant Dog @variant Cat }\nclass Test { run() { Animal::Dog::* } }\n");
    let family = case.associated_family("Animal", "Dog");
    assert!(!family.members.is_empty());
}

#[allow(dead_code)]
fn _canonical_ids() -> (DeclarationId, CallableId, Selector) {
    let owner = DeclarationId::new(phalcom_modules::identity::ModuleId::universe_root(), "Animal".into());
    let callable = CallableId::new(owner.clone(), Selector::method("run", []).expect("run"), DispatchSide::Instance);
    let selector = Selector::method("Dog", [SelectorSlot::Positional]).expect("Dog");
    (owner, callable, selector)
}

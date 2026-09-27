//! Explicit associated lookup and family capability scenarios.

use super::super::support::analyze_adt;
use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_semantic::checker::{AssociatedResolutionKind, CallableReferenceResolutionKind};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{AssociatedFamilyId, CallableId, DataConstructorId, DeclarationId, DispatchSide, InvocationTargetId};

fn resolution<'a>(case: &'a super::super::support::AdtCase, text: &str) -> &'a phalcom_semantic::checker::AssociatedResolution {
    case.analysis
        .snapshot
        .callable_analyses
        .values()
        .find_map(|callable| {
            callable
                .associated_resolutions
                .iter()
                .find_map(|(expression_id, associated)| {
                    let expression = callable.expressions.get(expression_id)?;
                    (case.source.get(expression.range.start..expression.range.end) == Some(text)).then_some(associated)
                })
                .or_else(|| {
                    callable.callable_reference_resolutions.iter().find_map(|(expression_id, reference)| {
                        let expression = callable.expressions.get(expression_id)?;
                        if case.source.get(expression.range.start..expression.range.end) != Some(text) {
                            return None;
                        }
                        match &reference.kind {
                            CallableReferenceResolutionKind::Associated(associated) => Some(associated.as_ref()),
                            CallableReferenceResolutionKind::BoundFamily { .. } => None,
                        }
                    })
                })
        })
        .unwrap_or_else(|| panic!("missing associated resolution for {text:?}"))
}

#[test]
fn adt_assoc_01_exact_singleton_lookup_resolves_variant_id() {
    let case = analyze_adt("data Animal { @variant Dog }\nclass Test { run() { Animal::Dog } }\n");
    assert!(matches!(resolution(&case, "Animal::Dog").kind, AssociatedResolutionKind::ExactValue { .. }));
}

#[test]
fn adt_assoc_02_exact_nullary_lookup_keeps_callable_selector_kind() {
    let case = analyze_adt("data Animal { @variant Dog() }\n");
    let dog = case.variant("Animal", Selector::method("Dog", []).expect("Dog"));
    assert_eq!(dog.id.selector.kind, phalcom_common::selector::SelectorKind::Method);
}

#[test]
fn adt_assoc_03_exact_payload_constructor_resolves_one_member() {
    let case = analyze_adt("data Animal { @variant Dog(_ name: String) }\nclass Test { run() { Animal::Dog(\"rex\") } }\n");
    assert!(matches!(
        resolution(&case, "Animal::Dog(\"rex\")").kind,
        AssociatedResolutionKind::StaticInvoke { .. }
    ));
}

#[test]
fn adt_assoc_04_whole_family_capture_publishes_exact_member_ids() {
    let case = analyze_adt("data Animal { @variant Dog @variant Dog() @variant Cat }\nclass Test { run() { &Animal::Dog... } }\n");
    let AssociatedResolutionKind::Family { members, .. } = &resolution(&case, "&Animal::Dog...").kind else {
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
fn adt_assoc_05_callable_family_excludes_singleton_member() {
    let case = analyze_adt("data Animal { @variant Dog @variant Dog() @variant Cat }\nclass Test { run() { &Animal::Dog... } }\n");
    let AssociatedResolutionKind::Family { members, .. } = &resolution(&case, "&Animal::Dog...").kind else {
        panic!("expected family resolution");
    };
    assert_eq!(members.len(), 2);
}

#[test]
fn adt_assoc_06_static_family_invocation_selects_exact_member() {
    let case = analyze_adt("data Animal { @variant Dog(_ name: String) @variant Dog() }\nclass Test { run() { Animal::Dog(\"rex\") } }\n");
    assert!(matches!(
        resolution(&case, "Animal::Dog(\"rex\")").kind,
        AssociatedResolutionKind::StaticInvoke { .. }
    ));
}

#[test]
fn adt_assoc_07_dynamic_family_pack_routes_by_shape() {
    let case = analyze_adt("data Animal { @variant Dog @variant Dog() @variant Cat }\nclass Test { run() { &Animal::Dog... } }\n");
    let AssociatedResolutionKind::Family { members, .. } = &resolution(&case, "&Animal::Dog...").kind else {
        panic!("expected dynamic family surface");
    };
    assert!(!members.is_empty());
}

#[test]
fn adt_assoc_08_positional_rest_routes_matching_candidate_shapes() {
    let case = analyze_adt("data Animal { @variant Dog(_ age: Int) @variant Dog(_ age: Int, breed: String) }\nclass Test { run() { &Animal::Dog... } }\n");
    let AssociatedResolutionKind::Family { members, .. } = &resolution(&case, "&Animal::Dog...").kind else {
        panic!("expected positional-rest family surface");
    };
    assert_eq!(members.len(), 2);
}

#[test]
fn adt_assoc_09_labeled_rest_routes_matching_candidate_shapes() {
    let case =
        analyze_adt("data Animal { @variant Dog(named age: Int) @variant Dog(named age: Int, breed: String) }\nclass Test { run() { &Animal::Dog... } }\n");
    let AssociatedResolutionKind::Family { members, .. } = &resolution(&case, "&Animal::Dog...").kind else {
        panic!("expected labeled-rest family surface");
    };
    assert_eq!(members.len(), 2);
}

#[test]
fn adt_assoc_10_generic_family_metadata_keeps_owner_specialization() {
    let case =
        analyze_adt("data Option<T> { @variant Some(_ value: T) -> Option<T> @variant None -> Option<T> }\nclass Test { run() { &Option<Int>::Some... } }\n");
    let family = case.associated_family("Option", "Some");
    assert!(!family.members.is_empty());
}

#[test]
#[ignore = "GATED: cross-module inherited ADT family fixture is required"]
fn adt_assoc_11_inherited_family_keeps_lookup_and_definition_owners() {
    let case = analyze_adt("data Animal { @variant Dog @variant Cat }\nclass Test { run() { &Animal::Dog... } }\n");
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
    let case = analyze_adt("data Animal { @variant Dog(_ name: String) }\nclass Test { run() { Animal::Dog() } }\n");
    assert!(
        case.diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::AssociatedCallShapeMissing || diagnostic.code == DiagnosticCode::AssociatedMemberMissing)
    );
}

#[test]
fn adt_assoc_13_wrong_call_shape_reports_diagnostic() {
    let case = analyze_adt("data Animal { @variant Dog(_ name: String) }\nclass Test { run() { Animal::Dog(1, 2) } }\n");
    assert!(
        case.diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::AssociatedCallShapeMissing || diagnostic.code == DiagnosticCode::AssociatedMemberMissing)
    );
}

#[test]
#[ignore = "GATED: cross-module associated visibility fixture is required"]
fn adt_assoc_14_private_member_is_not_explicitly_acquirable() {
    let case = analyze_adt("data Animal { @variant Dog @variant Cat }\nclass Test { run() { Animal::Dog } }\n");
    assert!(matches!(resolution(&case, "Animal::Dog").kind, AssociatedResolutionKind::ExactValue { .. }));
}

#[test]
#[ignore = "GATED: frozen-capability hierarchy fixture is required"]
fn adt_assoc_15_frozen_family_does_not_acquire_later_members() {
    let case = analyze_adt("data Animal { @variant Dog @variant Cat }\nclass Test { run() { &Animal::Dog... } }\n");
    let family = case.associated_family("Animal", "Dog");
    assert_eq!(family.members.len(), 1);
}

#[test]
#[ignore = "GATED: capability escape visibility fixture is required"]
fn adt_assoc_16_family_value_does_not_escape_member_visibility() {
    let case = analyze_adt("data Animal { @variant Dog @variant Cat }\nclass Test { run() { &Animal::Dog... } }\n");
    let family = case.associated_family("Animal", "Dog");
    assert!(!family.members.is_empty());
}

#[test]
fn data_associated_exact_callable_reference_resolves_data_constructor() {
    let case = analyze_adt("data Point(_ x: Int, _ y: Int)\nclass Test { run() { &Point::Construct(_, _) } }\n");
    let res = resolution(&case, "&Point::Construct(_, _)");
    assert!(matches!(
        res.kind,
        AssociatedResolutionKind::ExactCallable {
            member: phalcom_semantic::associated::AssociatedMemberId::DataConstructor(_),
            target: phalcom_semantic::identity::InvocationTargetId::DataConstructor(_),
            ..
        }
    ));
}

#[test]
fn ergonomic_primary_constructor_references_resolve_the_same_capabilities() {
    let case = analyze_adt(
        "data Tagged<T>(_ value: Int)\ndata Person<T> { name: String, payload: T }\nclass Test { run() { &Tagged<Int>::Construct(_); &Tagged<Int>(_); &Person<Int>::Construct(name: _, payload: _); &Person<Int>(name: _, payload: _) } }\n",
    );
    case.assert_no_diagnostics();

    for (canonical_source, ergonomic_source, expected_owner) in [
        ("&Tagged<Int>::Construct(_)" , "&Tagged<Int>(_)" , "Tagged"),
        (
            "&Person<Int>::Construct(name: _, payload: _)",
            "&Person<Int>(name: _, payload: _)",
            "Person",
        ),
    ] {
        let canonical = resolution(&case, canonical_source);
        let ergonomic = resolution(&case, ergonomic_source);
        let (canonical_member, canonical_target) = match &canonical.kind {
            AssociatedResolutionKind::ExactCallable { member, target, .. } => (member, target),
            other => panic!("expected exact canonical constructor capability, got {other:?}"),
        };
        let (ergonomic_member, ergonomic_target) = match &ergonomic.kind {
            AssociatedResolutionKind::ExactCallable { member, target, .. } => (member, target),
            other => panic!("expected exact ergonomic constructor capability, got {other:?}"),
        };
        assert_eq!(canonical.family, ergonomic.family);
        assert_eq!(canonical_member, ergonomic_member);
        assert_eq!(canonical_target, ergonomic_target);
        assert!(matches!(
            canonical_target,
            InvocationTargetId::DataConstructor(constructor) if constructor.owner.name.as_ref() == expected_owner
        ));
    }
}

#[test]
fn data_associated_whole_family_capture_publishes_data_constructor() {
    let case = analyze_adt("data Point(_ x: Int, _ y: Int)\nclass Test { run() { &Point::Construct... } }\n");
    let res = resolution(&case, "&Point::Construct...");
    let AssociatedResolutionKind::Family { members, .. } = &res.kind else {
        panic!("expected family resolution");
    };
    assert_eq!(members.len(), 1);
    assert!(matches!(
        members[0].member,
        phalcom_semantic::associated::AssociatedMemberId::DataConstructor(_)
    ));
    assert!(matches!(
        members[0].target,
        Some(phalcom_semantic::identity::InvocationTargetId::DataConstructor(_))
    ));
}

#[test]
fn product_construct_invocation_and_callable_reference_share_primary_constructor() {
    let case = analyze_adt("data Point(_ x: Int, _ y: Int)\nclass Test { run() { Point::Construct(1, 2); &Point::Construct(_, _) } }\n");
    case.assert_no_diagnostics();

    let owner = DeclarationId::new(case.module.clone(), "Point".into());
    let constructor = DataConstructorId::new(owner.clone());
    let family = case.associated_family("Point", "Construct");
    assert_eq!(
        family.id,
        AssociatedFamilyId::new(owner.clone(), phalcom_common::selector::SelectorBase::Named("Construct".into()))
    );
    assert_eq!(
        family.members.as_ref(),
        &[phalcom_semantic::associated::AssociatedMemberId::DataConstructor(constructor.clone())]
    );

    let invocation = resolution(&case, "Point::Construct(1, 2)");
    assert_eq!(invocation.family.as_ref(), Some(&family.id));
    assert!(matches!(
        &invocation.kind,
        AssociatedResolutionKind::StaticInvoke {
            member: phalcom_semantic::associated::AssociatedMemberId::DataConstructor(id),
            target: InvocationTargetId::DataConstructor(target),
            ..
        } if id == &constructor && target == &constructor
    ));

    let reference = resolution(&case, "&Point::Construct(_, _)");
    assert_eq!(reference.family.as_ref(), Some(&family.id));
    assert!(matches!(
        &reference.kind,
        AssociatedResolutionKind::ExactCallable {
            member: phalcom_semantic::associated::AssociatedMemberId::DataConstructor(id),
            target: InvocationTargetId::DataConstructor(target),
            ..
        } if id == &constructor && target == &constructor
    ));
}

#[test]
fn inaccessible_data_constructor_is_not_acquirable_by_exact_reference() {
    let case = analyze_adt("data Secret private(_ value: Int)\nclass Test { run() { &Secret::Construct(_) } }\n");
    assert!(
        case.analysis
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::AssociatedMemberInaccessible),
        "expected inaccessible associated constructor diagnostic, got: {:?}",
        case.analysis.snapshot.diagnostics
    );
}

#[test]
fn inaccessible_data_constructor_is_not_captured_by_family_reference() {
    let case = analyze_adt("data Secret private(_ value: Int)\nclass Test { run() { &Secret::Construct... } }\n");
    assert!(
        case.analysis
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::AssociatedMemberInaccessible),
        "expected inaccessible associated family diagnostic, got: {:?}",
        case.analysis.snapshot.diagnostics
    );
}

#[test]
fn inaccessible_data_constructor_is_rejected_by_associated_invocation() {
    let case = analyze_adt("data Secret private(_ value: Int)\nclass Test { run() { Secret::Construct(42) } }\n");
    assert!(
        case.analysis
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::MemberInaccessible),
        "expected inaccessible associated invocation diagnostic, got: {:?}",
        case.analysis.snapshot.diagnostics
    );
}

#[test]
fn authorized_data_constructor_capability_can_escape_its_owner() {
    let case = analyze_adt(
        "data Secret private(_ value: Int)\nimpl Secret { @class factory() -> (Int) -> Secret { &Secret::Construct(_) } }\nclass Caller { @class use() -> Secret { let f = Secret.factory(); f(42) } }\n",
    );
    assert!(
        !case
            .analysis
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::AssociatedMemberInaccessible),
        "authorized constructor capability must remain usable after escape: {:?}",
        case.analysis.snapshot.diagnostics
    );
}

#[test]
fn data_associated_surface_kind_is_data_constructor() {
    let case = analyze_adt("data Point(_ x: Int, _ y: Int)\n");
    let family = case.associated_family("Point", "Construct");
    assert_eq!(family.kind, phalcom_semantic::associated::AssociatedFamilyKind::DataConstructor);
    assert_eq!(family.members.len(), 1);
    assert!(matches!(
        family.members[0],
        phalcom_semantic::associated::AssociatedMemberId::DataConstructor(_)
    ));
    assert!(
        case.analysis
            .snapshot
            .associated_surfaces
            .surfaces
            .get(&DeclarationId::new(case.module.clone(), "Point".into()))
            .is_some_and(|surface| !surface.families.contains_key(&phalcom_common::selector::SelectorBase::Named("Point".into())))
    );
}

#[allow(dead_code)]
fn _canonical_ids() -> (DeclarationId, CallableId, Selector) {
    let owner = DeclarationId::new(phalcom_modules::identity::ModuleId::universe_root(), "Animal".into());
    let callable = CallableId::new(owner.clone(), Selector::method("run", []).expect("run"), DispatchSide::Instance);
    let selector = Selector::method("Dog", [SelectorSlot::Positional]).expect("Dog");
    (owner, callable, selector)
}

//! Tests for trait generic contracts, derived conformance, and associated projection normalization.

use crate::semantic::support::Fixture;
use phalcom_semantic::DiagnosticCode;
use phalcom_semantic::types::parameter::{GenericConstraint, TypeTerm};
use phalcom_semantic::types::store::TypeData;

#[test]
fn trait_contract_where_self_forms() {
    let fixture = Fixture::new(
        r#"
trait Base {
    type Item
}

trait Sub
    where Self |= Base
{
    type Item
}
"#,
    );

    let sub_decl = fixture.decl("Sub");
    let base_decl = fixture.decl("Base");
    let sub_info = fixture.analysis.snapshot.trait_headers.get(&sub_decl).expect("sub trait header");
    let generics = sub_info.generic_signature.as_ref().expect("generics on Sub");

    assert_eq!(generics.constraints.len(), 1);
    match &generics.constraints[0] {
        GenericConstraint::Conforms { subject, trait_ref } => {
            assert!(matches!(subject, TypeTerm::Canonical(ty) if matches!(fixture.analysis.snapshot.store.get(*ty), TypeData::SelfType(_))));
            assert_eq!(trait_ref.declaration, base_decl);
        }
        other => panic!("expected Conforms constraint, got {:?}", other),
    }
}

#[test]
fn trait_contract_where_self_associated_projection_forms() {
    let fixture = Fixture::new(
        r#"
trait Stepper {
    type Item
}

trait Stream
    where Self::Iterator |= Stepper
{
    type Item
    type Iterator
}
"#,
    );

    let stream_decl = fixture.decl("Stream");
    let stepper_decl = fixture.decl("Stepper");
    let stream_info = fixture.analysis.snapshot.trait_headers.get(&stream_decl).expect("stream trait header");
    let generics = stream_info.generic_signature.as_ref().expect("generics on Stream");

    assert_eq!(generics.constraints.len(), 1);
    match &generics.constraints[0] {
        GenericConstraint::Conforms { subject, trait_ref } => {
            match subject {
                TypeTerm::Canonical(ty) => match fixture.analysis.snapshot.store.get(*ty) {
                    TypeData::AssociatedProjection(proj) => {
                        assert_eq!(proj.trait_ref.declaration, stream_decl);
                        assert_eq!(proj.requirement.owner, stream_decl);
                        assert_eq!(proj.requirement.index, 1); // Iterator is index 1
                        assert!(matches!(fixture.analysis.snapshot.store.get(proj.subject), TypeData::SelfType(_)));
                    }
                    other => panic!("expected AssociatedProjection on Self, got {:?}", other),
                },
                _ => panic!("expected canonical term"),
            }
            assert_eq!(trait_ref.declaration, stepper_decl);
        }
        other => panic!("expected Conforms constraint, got {:?}", other),
    }
}

#[test]
fn trait_contract_qualified_nested_projection_forms() {
    let fixture = Fixture::new(
        r#"
trait Stepper {
    type Item
}

trait Stream
    where
        Self::Iterator |= Stepper,
        <Self::Iterator as Stepper>::Item == Self::Item
{
    type Item
    type Iterator
}
"#,
    );

    let stream_decl = fixture.decl("Stream");
    let stepper_decl = fixture.decl("Stepper");
    let stream_info = fixture.analysis.snapshot.trait_headers.get(&stream_decl).expect("stream trait header");
    let generics = stream_info.generic_signature.as_ref().expect("generics on Stream");

    assert_eq!(generics.constraints.len(), 2);
    let mut found_conforms = false;
    let mut found_equiv = false;

    for constraint in &generics.constraints {
        match constraint {
            GenericConstraint::Conforms { subject: _, trait_ref } => {
                assert_eq!(trait_ref.declaration, stepper_decl);
                found_conforms = true;
            }
            GenericConstraint::Equivalent { left, right } => {
                // One side should be < <Self as Stream>::Iterator as Stepper>::Item
                // The other side should be <Self as Stream>::Item
                let mut found_nested = false;
                let mut found_stream_item = false;
                for term in [left, right] {
                    if let TypeTerm::Canonical(ty) = term {
                        match fixture.analysis.snapshot.store.get(*ty) {
                            TypeData::AssociatedProjection(proj) => {
                                if proj.trait_ref.declaration == stepper_decl {
                                    assert_eq!(proj.requirement.owner, stepper_decl);
                                    assert_eq!(proj.requirement.index, 0); // Item is index 0 in Stepper
                                    // Subject of stepper projection is <Self as Stream>::Iterator
                                    match fixture.analysis.snapshot.store.get(proj.subject) {
                                        TypeData::AssociatedProjection(inner_proj) => {
                                            assert_eq!(inner_proj.trait_ref.declaration, stream_decl);
                                            assert_eq!(inner_proj.requirement.owner, stream_decl);
                                            assert_eq!(inner_proj.requirement.index, 1); // Iterator is index 1 in Stream
                                            found_nested = true;
                                        }
                                        other => panic!("expected inner AssociatedProjection, got {:?}", other),
                                    }
                                } else if proj.trait_ref.declaration == stream_decl {
                                    assert_eq!(proj.requirement.owner, stream_decl);
                                    assert_eq!(proj.requirement.index, 0); // Item is index 0 in Stream
                                    assert!(matches!(fixture.analysis.snapshot.store.get(proj.subject), TypeData::SelfType(_)));
                                    found_stream_item = true;
                                }
                            }
                            other => panic!("unexpected term: {:?}", other),
                        }
                    }
                }
                assert!(found_nested, "expected nested projection term");
                assert!(found_stream_item, "expected Self::Item projection term");
                assert!(left <= right, "Equality must be canonically oriented");
                found_equiv = true;
            }
            other => panic!("unexpected constraint: {:?}", other),
        }
    }

    assert!(found_conforms);
    assert!(found_equiv);
}

#[test]
fn trait_contract_constraint_order_is_canonical() {
    let fixture1 = Fixture::new(
        r#"
trait Stepper {
    type Item
}

trait Stream
    where
        Self::Iterator |= Stepper,
        <Self::Iterator as Stepper>::Item == Self::Item
{
    type Item
    type Iterator
}
"#,
    );

    let fixture2 = Fixture::new(
        r#"
trait Stepper {
    type Item
}

trait Stream
    where
        <Self::Iterator as Stepper>::Item == Self::Item,
        Self::Iterator |= Stepper
{
    type Item
    type Iterator
}
"#,
    );

    let s1 = fixture1.analysis.snapshot.trait_headers.get(&fixture1.decl("Stream")).unwrap();
    let s2 = fixture2.analysis.snapshot.trait_headers.get(&fixture2.decl("Stream")).unwrap();

    let sig1 = s1.generic_signature.as_ref().unwrap();
    let sig2 = s2.generic_signature.as_ref().unwrap();

    assert_eq!(sig1.constraints, sig2.constraints);
}

#[test]
fn trait_contract_unknown_own_associated_projection_diagnoses() {
    let fixture = Fixture::new(
        r#"
trait Stream
    where Self::UnknownType |= Stream
{
    type Item
}
"#,
    );

    let diags = fixture.diagnostics(DiagnosticCode::AssociatedTypeUnknown);
    assert!(
        !diags.is_empty(),
        "expected AssociatedTypeUnknown diagnostic, got: {:?}",
        fixture.analysis.snapshot.diagnostics
    );
}

#[test]
fn trait_contract_does_not_require_associated_declaration_bound_syntax() {
    let fixture = Fixture::new(
        r#"
trait Stepper {
    type Item
}

trait Iterable
    where
        Self::Iterator |= Stepper,
        <Self::Iterator as Stepper>::Item == Self::Item
{
    type Item
    type Iterator
}
"#,
    );

    let diags = &fixture.analysis.snapshot.diagnostics;
    assert!(diags.is_empty(), "expected clean analysis without inline bound syntax, got: {:?}", diags);
}

#[test]
fn explicit_trait_contract_validates_successfully() {
    let fixture = Fixture::new(
        r#"
trait Stepper {
    type Item
}

trait Stream
    where
        Self::Iterator |= Stepper,
        <Self::Iterator as Stepper>::Item == Self::Item
{
    type Item
    type Iterator
}

class IntStepper {}

impl Stepper for IntStepper {
    type Item = Int
}

class IntStream {}

impl Stream for IntStream {
    type Item = Int
    type Iterator = IntStepper
}
"#,
    );

    let diags = &fixture.analysis.snapshot.diagnostics;
    assert!(diags.is_empty(), "expected valid explicit contract to have no diagnostics, got: {:?}", diags);
}

#[test]
fn trait_contract_rejects_nonconforming_associated_iterator() {
    let fixture = Fixture::new(
        r#"
trait Stepper {
    type Item
}

trait Stream
    where
        Self::Iterator |= Stepper,
        <Self::Iterator as Stepper>::Item == Self::Item
{
    type Item
    type Iterator
}

class NotStepper {}

class BadStream {}

impl Stream for BadStream {
    type Item = Int
    type Iterator = NotStepper
}
"#,
    );

    let diags = fixture.diagnostics(DiagnosticCode::ImplConformanceIncomplete);
    assert!(
        !diags.is_empty(),
        "expected ImplConformanceIncomplete diagnostic for non-conforming associated iterator, got: {:?}",
        fixture.analysis.snapshot.diagnostics
    );
}

#[test]
fn trait_contract_rejects_associated_item_mismatch() {
    let fixture = Fixture::new(
        r#"
trait Stepper {
    type Item
}

trait Stream
    where
        Self::Iterator |= Stepper,
        <Self::Iterator as Stepper>::Item == Self::Item
{
    type Item
    type Iterator
}

class StringStepper {}

impl Stepper for StringStepper {
    type Item = String
}

class BadStream {}

impl Stream for BadStream {
    type Item = Int
    type Iterator = StringStepper
}
"#,
    );

    let diags = fixture.diagnostics(DiagnosticCode::ImplConformanceIncomplete);
    assert!(
        !diags.is_empty(),
        "expected ImplConformanceIncomplete diagnostic for mismatched Item, got: {:?}",
        fixture.analysis.snapshot.diagnostics
    );
}

#[test]
fn assumed_trait_evidence_entails_associated_conformance() {
    let fixture = Fixture::new(
        r#"
trait Stepper {
    type Item
}

trait Stream
    where
        Self::Iterator |= Stepper,
        <Self::Iterator as Stepper>::Item == Self::Item
{
    type Item
    type Iterator
}

class Inspector {
    @class
    check<S>(_ stream: S)
        where S |= Stream
    {
        nil
    }
}
"#,
    );

    let inspector = fixture.callable("Inspector", "check", phalcom_semantic::identity::DispatchSide::Class);
    let signature = fixture.analysis.snapshot.callable_signatures.get(&inspector.callable).expect("signature");
    let generics = signature.generics.as_ref().expect("generics");
    let stream_decl = fixture.decl("Stream");
    let stepper_decl = fixture.decl("Stepper");

    let env = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
        &generics.constraints,
        phalcom_semantic::types::parameter::TypeParameterOwner::Callable(inspector.callable.clone()),
        &fixture.analysis.snapshot.trait_surfaces,
    )
    .expect("env");

    let GenericConstraint::Conforms { subject: s_term, .. } = &generics.constraints[0] else {
        panic!("expected Conforms constraint");
    };
    let TypeTerm::Canonical(s_ty) = *s_term else {
        panic!("expected canonical term")
    };

    let mut store = fixture.analysis.snapshot.store.as_ref().clone();
    let s_iter_proj = store.associated_projection(
        s_ty,
        phalcom_semantic::traits::TraitRef::new(stream_decl.clone(), Vec::new()),
        phalcom_semantic::traits::AssociatedTypeRequirementId::new(stream_decl, 1),
    );

    let key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(
        TypeTerm::Canonical(s_iter_proj),
        phalcom_semantic::traits::TraitRef::new(stepper_decl, Vec::new()),
    );

    let mut context = phalcom_semantic::generic_evidence::ConformanceProofContext::new(
        Some(&env),
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    );

    let proof = phalcom_semantic::generic_evidence::prove_conformance_with_context(&key, &mut context);
    assert!(proof.is_proven(), "expected assumed Stream to entail Iterator |= Stepper, got: {:?}", proof);
}

#[test]
fn assumed_trait_evidence_entails_associated_equality() {
    let fixture = Fixture::new(
        r#"
trait Stepper {
    type Item
}

trait Stream
    where
        Self::Iterator |= Stepper,
        <Self::Iterator as Stepper>::Item == Self::Item
{
    type Item
    type Iterator
}

class Inspector {
    @class
    check<S>(_ stream: S)
        where S |= Stream
    {
        nil
    }
}
"#,
    );

    let inspector = fixture.callable("Inspector", "check", phalcom_semantic::identity::DispatchSide::Class);
    let signature = fixture.analysis.snapshot.callable_signatures.get(&inspector.callable).expect("signature");
    let generics = signature.generics.as_ref().expect("generics");
    let stream_decl = fixture.decl("Stream");
    let stepper_decl = fixture.decl("Stepper");

    let env = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
        &generics.constraints,
        phalcom_semantic::types::parameter::TypeParameterOwner::Callable(inspector.callable.clone()),
        &fixture.analysis.snapshot.trait_surfaces,
    )
    .expect("env");

    let GenericConstraint::Conforms { subject: s_term, .. } = &generics.constraints[0] else {
        panic!("expected Conforms constraint");
    };
    let TypeTerm::Canonical(s_ty) = *s_term else {
        panic!("expected canonical term")
    };

    let mut store = fixture.analysis.snapshot.store.as_ref().clone();

    // Left: <<S as Stream>::Iterator as Stepper>::Item
    let s_iter_proj = store.associated_projection(
        s_ty,
        phalcom_semantic::traits::TraitRef::new(stream_decl.clone(), Vec::new()),
        phalcom_semantic::traits::AssociatedTypeRequirementId::new(stream_decl.clone(), 1),
    );
    let nested_item_proj = store.associated_projection(
        s_iter_proj,
        phalcom_semantic::traits::TraitRef::new(stepper_decl.clone(), Vec::new()),
        phalcom_semantic::traits::AssociatedTypeRequirementId::new(stepper_decl, 0),
    );

    // Right: <S as Stream>::Item
    let stream_item_proj = store.associated_projection(
        s_ty,
        phalcom_semantic::traits::TraitRef::new(stream_decl.clone(), Vec::new()),
        phalcom_semantic::traits::AssociatedTypeRequirementId::new(stream_decl, 0),
    );

    let constraint = GenericConstraint::Equivalent {
        left: TypeTerm::Canonical(nested_item_proj),
        right: TypeTerm::Canonical(stream_item_proj),
    };

    let mut context = phalcom_semantic::generic_evidence::ConformanceProofContext::new(
        Some(&env),
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    );

    let proof =
        phalcom_semantic::generic_evidence::prove_constraint(&constraint, &phalcom_semantic::types::substitution::TypeSubstitution::new(), &mut context);
    assert!(
        proof.is_proven(),
        "expected assumed Stream to entail Item equality constraint, got: {:?}",
        proof
    );
}

#[test]
fn dynamic_does_not_satisfy_trait_contract() {
    let fixture = Fixture::new(
        r#"
trait Stepper {
    type Item
}

trait Stream
    where
        Self::Iterator |= Stepper,
        <Self::Iterator as Stepper>::Item == Self::Item
{
    type Item
    type Iterator
}
"#,
    );

    let stepper_decl = fixture.decl("Stepper");
    let int_ty = fixture.ty("Int");
    let mut store = fixture.analysis.snapshot.store.as_ref().clone();

    // Query on an unconstrained Nominal type that doesn't conform
    let key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(
        TypeTerm::Canonical(int_ty),
        phalcom_semantic::traits::TraitRef::new(stepper_decl, Vec::new()),
    );

    let mut context = phalcom_semantic::generic_evidence::ConformanceProofContext::new(
        None,
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    );

    let proof = phalcom_semantic::generic_evidence::prove_conformance_with_context(&key, &mut context);
    assert!(!proof.is_proven(), "Int must not statically satisfy Stepper proof");
}

#[test]
fn trait_contract_recursive_implication_terminates_without_success() {
    let fixture = Fixture::new(
        r#"
trait LoopNode
    where Self |= LoopNode
{
    type Item
}

class Probe<T> {
    @class
    check(_ node: T) {
        nil
    }
}
"#,
    );

    let loop_decl = fixture.decl("LoopNode");
    let probe_decl = fixture.decl("Probe");
    let probe_info = fixture.analysis.snapshot.declarations.get(&probe_decl).expect("Probe declaration");
    let t_param = probe_info.generic_signature.as_ref().unwrap().parameters[0];

    let mut store = fixture.analysis.snapshot.store.as_ref().clone();
    let t_ty = store.parameter_form(t_param);

    let key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(
        TypeTerm::Canonical(t_ty),
        phalcom_semantic::traits::TraitRef::new(loop_decl, Vec::new()),
    );

    let mut context = phalcom_semantic::generic_evidence::ConformanceProofContext::new(
        None,
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    );

    let proof = phalcom_semantic::generic_evidence::prove_conformance_with_context(&key, &mut context);
    assert!(!proof.is_proven(), "Recursive trait contract must terminate as non-proven, got: {:?}", proof);
}

#[test]
fn guarded_generic_target_conformance_forms_rule() {
    let fixture = Fixture::new(
        r#"
trait Iterator {
    type Item
}

trait Iterable {
    type Item
    type Iterator
    iterator() -> Self::Iterator
}

impl<T> Iterable for T
    where T |= Iterator
{
    type Item = T::Item
    type Iterator = T

    iterator() -> T {
        self
    }
}
"#,
    );

    let diags = fixture.analysis.snapshot.all_diagnostics().collect::<Vec<_>>();
    assert!(diags.is_empty(), "unexpected diagnostics: {:?}", diags);

    let iterable_decl = fixture.decl("Iterable");
    let rules = fixture.analysis.snapshot.derived_conformance_rules.rules_for_trait(&iterable_decl);
    assert_eq!(rules.len(), 1, "expected 1 derived conformance rule for Iterable");
    let rule = &rules[0];
    assert_eq!(rule.trait_ref.declaration, iterable_decl);
    assert_eq!(rule.requirements.len(), 1);
    assert_eq!(rule.associated_type_plan.bindings.len(), 2);
}

#[test]
fn unguarded_generic_target_conformance_is_rejected() {
    let fixture = Fixture::new(
        r#"
trait Target {
    ping() -> Int
}

impl<T> Target for T {
    ping() -> Int {
        42
    }
}
"#,
    );

    let codes: Vec<_> = fixture.analysis.snapshot.all_diagnostics().map(|d| d.code).collect();
    assert!(
        codes.contains(&DiagnosticCode::DerivedConformanceUnguarded),
        "expected DerivedConformanceUnguarded, got: {:?}",
        codes
    );
}

#[test]
fn generic_target_rule_requires_consequent_trait_ownership() {
    use crate::semantic::support::WorkspaceFixture;

    let fixture = WorkspaceFixture::new()
        .module(
            "upstream",
            r#"
trait OwnedTrait {
    ping() -> Int
}
export OwnedTrait

trait GuardTrait {}
export GuardTrait
"#,
        )
        .module(
            "downstream",
            r#"
from upstream import OwnedTrait
from upstream import GuardTrait

impl<T> OwnedTrait for T
    where T |= GuardTrait
{
    ping() -> Int {
        0
    }
}
"#,
        )
        .analyze();

    let codes: Vec<_> = fixture.analysis.snapshot.all_diagnostics().map(|d| d.code).collect();
    assert!(
        codes.contains(&DiagnosticCode::DerivedConformanceUnauthorized),
        "expected DerivedConformanceUnauthorized for external trait rule, got: {:?}",
        codes
    );
}

#[test]
fn generic_target_rule_parameter_renaming_is_semantically_equal() {
    let fixture1 = Fixture::new(
        r#"
trait Guard {}
trait Target {
    type Assoc
}

impl<U> Target for U
    where U |= Guard
{
    type Assoc = U
}
"#,
    );

    let fixture2 = Fixture::new(
        r#"
trait Guard {}
trait Target {
    type Assoc
}

impl<V> Target for V
    where V |= Guard
{
    type Assoc = V
}
"#,
    );

    let target_decl1 = fixture1.decl("Target");
    let target_decl2 = fixture2.decl("Target");

    let rule1 = &fixture1.analysis.snapshot.derived_conformance_rules.rules_for_trait(&target_decl1)[0];
    let rule2 = &fixture2.analysis.snapshot.derived_conformance_rules.rules_for_trait(&target_decl2)[0];

    assert_eq!(
        rule1.target_template, rule2.target_template,
        "target templates should be structurally equal regardless of parameter name"
    );
}

#[test]
fn generic_target_rule_constraint_order_is_fingerprint_stable() {
    let fixture1 = Fixture::new(
        r#"
trait GuardA {}
trait GuardB {}
trait Target {}

impl<T> Target for T
    where T |= GuardA, T |= GuardB
{}
"#,
    );

    let fixture2 = Fixture::new(
        r#"
trait GuardA {}
trait GuardB {}
trait Target {}

impl<T> Target for T
    where T |= GuardB, T |= GuardA
{}
"#,
    );

    let target_decl1 = fixture1.decl("Target");
    let target_decl2 = fixture2.decl("Target");

    let rule1 = &fixture1.analysis.snapshot.derived_conformance_rules.rules_for_trait(&target_decl1)[0];
    let rule2 = &fixture2.analysis.snapshot.derived_conformance_rules.rules_for_trait(&target_decl2)[0];

    assert_eq!(
        rule1.fingerprint, rule2.fingerprint,
        "fingerprints should be stable under constraint order permutation"
    );
}

#[test]
fn generic_target_rule_associated_binding_uses_trait_owned_requirement() {
    let fixture = Fixture::new(
        r#"
trait Iterator {
    type Item
}

trait Iterable {
    type Item
    type Iterator
}

impl<T> Iterable for T
    where T |= Iterator
{
    type Item = T::Item
    type Iterator = T
}
"#,
    );

    let iterable_decl = fixture.decl("Iterable");
    let rule = &fixture.analysis.snapshot.derived_conformance_rules.rules_for_trait(&iterable_decl)[0];

    for req in rule.associated_type_plan.bindings.keys() {
        assert_eq!(req.owner, iterable_decl, "associated type requirement must be owned by the consequent trait");
    }
}

#[test]
fn generic_target_rule_witness_does_not_enter_inherent_surface() {
    let fixture = Fixture::new(
        r#"
trait Iterator {
    type Item
}

trait Iterable {
    iterator() -> Self
}

impl<T> Iterable for T
    where T |= Iterator
{
    iterator() -> T {
        self
    }
}

class NumberStream {}
"#,
    );

    let stream_decl = fixture.decl("NumberStream");
    let stream_surface = fixture.analysis.snapshot.surfaces.get(&stream_decl).expect("stream surface");
    assert!(
        !stream_surface
            .instance
            .callables_by_selector
            .keys()
            .any(|s| matches!(&s.base, phalcom_common::selector::SelectorBase::Named(name) if name == "iterator")),
        "rule witness must not leak into inherent surface of arbitrary types"
    );
}

#[test]
fn overlapping_generic_target_rules_are_rejected() {
    let fixture = Fixture::new(
        r#"
trait GuardA {}
trait GuardB {}
trait Target {}

impl<T> Target for T
    where T |= GuardA
{}

impl<T> Target for T
    where T |= GuardB
{}
"#,
    );

    let codes: Vec<_> = fixture.analysis.snapshot.all_diagnostics().map(|d| d.code).collect();
    assert!(
        codes.contains(&DiagnosticCode::DerivedConformanceOverlap),
        "expected DerivedConformanceOverlap, got: {:?}",
        codes
    );
}

#[test]
fn generic_target_rule_order_does_not_select_winner() {
    let fixture1 = Fixture::new(
        r#"
trait GuardA {}
trait GuardB {}
trait Target {}

impl<T> Target for T
    where T |= GuardA
{}

impl<T> Target for T
    where T |= GuardB
{}
"#,
    );

    let fixture2 = Fixture::new(
        r#"
trait GuardA {}
trait GuardB {}
trait Target {}

impl<T> Target for T
    where T |= GuardB
{}

impl<T> Target for T
    where T |= GuardA
{}
"#,
    );

    let codes1: Vec<_> = fixture1.analysis.snapshot.all_diagnostics().map(|d| d.code).collect();
    let codes2: Vec<_> = fixture2.analysis.snapshot.all_diagnostics().map(|d| d.code).collect();

    assert!(codes1.contains(&DiagnosticCode::DerivedConformanceOverlap));
    assert!(codes2.contains(&DiagnosticCode::DerivedConformanceOverlap));
}

#[test]
fn exact_iterator_derives_iterable() {
    let fixture = Fixture::new(
        r#"
trait Iterator {
    type Item
}

trait Iterable {
    type Iterator
    type Item
}

impl<T> Iterable for T
    where T |= Iterator
{
    type Iterator = T
    type Item = T::Item
}

class ItemType {}
class Counter {}

impl Iterator for Counter {
    type Item = ItemType
}
"#,
    );

    fixture.assert_no_error_diagnostics();

    let counter_ty = fixture.ty("Counter");
    let iterable_decl = fixture.decl("Iterable");
    let trait_ref = phalcom_semantic::traits::TraitRef::new(iterable_decl, vec![]);

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let mut context = phalcom_semantic::generic_evidence::ConformanceProofContext::new(
        None,
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    )
    .with_derived_rules(Some(&fixture.analysis.snapshot.derived_conformance_rules));

    let key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(phalcom_semantic::types::parameter::TypeTerm::Canonical(counter_ty), trait_ref);

    let proof = phalcom_semantic::generic_evidence::prove_conformance_with_context(&key, &mut context);
    match proof {
        phalcom_semantic::generic_evidence::GenericConformanceProof::Proven(phalcom_semantic::generic_evidence::GenericConformanceEvidence::Derived(
            derived,
        )) => {
            match &derived.origin {
                phalcom_semantic::generic_evidence::DerivedConformanceOrigin::GuardedRule { exact_subject, .. } => {
                    assert_eq!(*exact_subject, counter_ty);
                }
                _ => panic!("expected GuardedRule origin"),
            }
            assert_eq!(derived.nested.len(), 1);
        }
        other => panic!("expected Proven(Derived) proof, got {:?}", other),
    }
}

#[test]
fn non_iterator_does_not_derive_iterable() {
    let fixture = Fixture::new(
        r#"
trait Iterator {
    type Item
}

trait Iterable {
    type Iterator
    type Item
}

impl<T> Iterable for T
    where T |= Iterator
{
    type Iterator = T
    type Item = T::Item
}

class User {}
"#,
    );

    fixture.assert_no_error_diagnostics();

    let user_ty = fixture.ty("User");
    let iterable_decl = fixture.decl("Iterable");
    let trait_ref = phalcom_semantic::traits::TraitRef::new(iterable_decl, vec![]);

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let mut context = phalcom_semantic::generic_evidence::ConformanceProofContext::new(
        None,
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    )
    .with_derived_rules(Some(&fixture.analysis.snapshot.derived_conformance_rules));

    let key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(phalcom_semantic::types::parameter::TypeTerm::Canonical(user_ty), trait_ref);

    let proof = phalcom_semantic::generic_evidence::prove_conformance_with_context(&key, &mut context);
    assert_eq!(proof, phalcom_semantic::generic_evidence::GenericConformanceProof::Missing);
}

#[test]
fn assumed_iterator_derives_iterable() {
    let fixture = Fixture::new(
        r#"
trait Iterator {
    type Item
}

trait Iterable {
    type Iterator
    type Item
}

impl<T> Iterable for T
    where T |= Iterator
{
    type Iterator = T
    type Item = T::Item
}

class Container<T>
    where T |= Iterator
{}
"#,
    );

    fixture.assert_no_error_diagnostics();

    let container_decl = fixture.decl("Container");
    let iterable_decl = fixture.decl("Iterable");
    let iterator_decl = fixture.decl("Iterator");

    let container_info = fixture.analysis.snapshot.declarations.get(&container_decl).expect("container decl");
    let generics = container_info.generic_signature.as_ref().expect("generics on Container");
    let t_param = generics.parameters[0];

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let t_param_ty = store.parameter_form(t_param);

    let env = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
        &generics.constraints,
        generics.owner.clone(),
        &fixture.analysis.snapshot.trait_surfaces,
    )
    .expect("evidence env");

    let trait_ref = phalcom_semantic::traits::TraitRef::new(iterable_decl, vec![]);
    let mut context = phalcom_semantic::generic_evidence::ConformanceProofContext::new(
        Some(&env),
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    )
    .with_derived_rules(Some(&fixture.analysis.snapshot.derived_conformance_rules));

    let key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(phalcom_semantic::types::parameter::TypeTerm::Canonical(t_param_ty), trait_ref);

    let proof = phalcom_semantic::generic_evidence::prove_conformance_with_context(&key, &mut context);
    match proof {
        phalcom_semantic::generic_evidence::GenericConformanceProof::Proven(phalcom_semantic::generic_evidence::GenericConformanceEvidence::Derived(
            derived,
        )) => {
            assert_eq!(derived.nested.len(), 1);
            match &derived.nested[0] {
                phalcom_semantic::generic_evidence::NestedGenericEvidence::Conformance(
                    phalcom_semantic::generic_evidence::GenericConformanceEvidence::Assumed(assumed),
                ) => {
                    assert_eq!(assumed.key.trait_ref.declaration, iterator_decl);
                }
                other => panic!("expected nested Assumed evidence, got {:?}", other),
            }
        }
        other => panic!("expected Proven(Derived) proof, got {:?}", other),
    }
}

#[test]
fn conditional_iterator_feeds_derived_iterable() {
    let fixture = Fixture::new(
        r#"
trait Element {}

trait Iterator {
    type Item
}

trait Iterable {
    type Iterator
    type Item
}

impl<T> Iterable for T
    where T |= Iterator
{
    type Iterator = T
    type Item = T::Item
}

class Box<T> {}

impl<T> Iterator for Box<T>
    where T |= Element
{
    type Item = T
}

class GoodElem {}

impl Element for GoodElem {}
"#,
    );

    fixture.assert_no_error_diagnostics();

    let box_form = fixture.ty("Box");
    let good_elem_ty = fixture.ty("GoodElem");
    let iterable_decl = fixture.decl("Iterable");
    let trait_ref = phalcom_semantic::traits::TraitRef::new(iterable_decl, vec![]);

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let box_elem_ty = store.apply_type_form(box_form, &[good_elem_ty]).unwrap();

    let mut context = phalcom_semantic::generic_evidence::ConformanceProofContext::new(
        None,
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    )
    .with_derived_rules(Some(&fixture.analysis.snapshot.derived_conformance_rules));

    let key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(phalcom_semantic::types::parameter::TypeTerm::Canonical(box_elem_ty), trait_ref);

    let proof = phalcom_semantic::generic_evidence::prove_conformance_with_context(&key, &mut context);
    match proof {
        phalcom_semantic::generic_evidence::GenericConformanceProof::Proven(phalcom_semantic::generic_evidence::GenericConformanceEvidence::Derived(
            derived,
        )) => {
            assert_eq!(derived.nested.len(), 1);
            match &derived.nested[0] {
                phalcom_semantic::generic_evidence::NestedGenericEvidence::Conformance(
                    phalcom_semantic::generic_evidence::GenericConformanceEvidence::Derived(cond_derived),
                ) => {
                    assert!(cond_derived.exact_source().is_some());
                }
                other => panic!("expected nested conditional Derived evidence, got {:?}", other),
            }
        }
        other => panic!("expected Proven(Derived), got {:?}", other),
    }
}

#[test]
fn nested_derived_chain_works() {
    let fixture = Fixture::new(
        r#"
trait Base {}
trait Middle {}
trait Top {}

impl<T> Middle for T
    where T |= Base
{}

impl<T> Top for T
    where T |= Middle
{}

class Leaf {}

impl Base for Leaf {}
"#,
    );

    fixture.assert_no_error_diagnostics();

    let leaf_ty = fixture.ty("Leaf");
    let top_decl = fixture.decl("Top");
    let top_ref = phalcom_semantic::traits::TraitRef::new(top_decl, vec![]);

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let mut context = phalcom_semantic::generic_evidence::ConformanceProofContext::new(
        None,
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    )
    .with_derived_rules(Some(&fixture.analysis.snapshot.derived_conformance_rules));

    let key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(phalcom_semantic::types::parameter::TypeTerm::Canonical(leaf_ty), top_ref);

    let proof = phalcom_semantic::generic_evidence::prove_conformance_with_context(&key, &mut context);
    match proof {
        phalcom_semantic::generic_evidence::GenericConformanceProof::Proven(phalcom_semantic::generic_evidence::GenericConformanceEvidence::Derived(
            top_derived,
        )) => {
            assert_eq!(top_derived.nested.len(), 1);
            match &top_derived.nested[0] {
                phalcom_semantic::generic_evidence::NestedGenericEvidence::Conformance(
                    phalcom_semantic::generic_evidence::GenericConformanceEvidence::Derived(mid_derived),
                ) => {
                    assert_eq!(mid_derived.nested.len(), 1);
                    match &mid_derived.nested[0] {
                        phalcom_semantic::generic_evidence::NestedGenericEvidence::Conformance(
                            phalcom_semantic::generic_evidence::GenericConformanceEvidence::Exact(_),
                        ) => {}
                        other => panic!("expected innermost Exact evidence, got {:?}", other),
                    }
                }
                other => panic!("expected nested Middle Derived evidence, got {:?}", other),
            }
        }
        other => panic!("expected Proven(Derived) proof, got {:?}", other),
    }
}

#[test]
fn self_cycle_blocks_proof() {
    let fixture = Fixture::new(
        r#"
trait SelfLoop {}

impl<T> SelfLoop for T
    where T |= SelfLoop
{}

class Candidate {}
"#,
    );

    let candidate_ty = fixture.ty("Candidate");
    let loop_decl = fixture.decl("SelfLoop");
    let loop_ref = phalcom_semantic::traits::TraitRef::new(loop_decl, vec![]);

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let mut context = phalcom_semantic::generic_evidence::ConformanceProofContext::new(
        None,
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    )
    .with_derived_rules(Some(&fixture.analysis.snapshot.derived_conformance_rules));

    let key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(phalcom_semantic::types::parameter::TypeTerm::Canonical(candidate_ty), loop_ref);

    let proof = phalcom_semantic::generic_evidence::prove_conformance_with_context(&key, &mut context);
    match proof {
        phalcom_semantic::generic_evidence::GenericConformanceProof::Blocked(phalcom_semantic::types::outcome::BlockReason::RecursiveFixpoint) => {}
        other => panic!("expected Blocked(RecursiveFixpoint), got {:?}", other),
    }
}

#[test]
fn mutual_cycle_blocks_proof() {
    let fixture = Fixture::new(
        r#"
trait TraitA {}
trait TraitB {}

impl<T> TraitA for T
    where T |= TraitB
{}

impl<T> TraitB for T
    where T |= TraitA
{}

class Candidate {}
"#,
    );

    let candidate_ty = fixture.ty("Candidate");
    let a_decl = fixture.decl("TraitA");
    let a_ref = phalcom_semantic::traits::TraitRef::new(a_decl, vec![]);

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let mut context = phalcom_semantic::generic_evidence::ConformanceProofContext::new(
        None,
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    )
    .with_derived_rules(Some(&fixture.analysis.snapshot.derived_conformance_rules));

    let key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(phalcom_semantic::types::parameter::TypeTerm::Canonical(candidate_ty), a_ref);

    let proof = phalcom_semantic::generic_evidence::prove_conformance_with_context(&key, &mut context);
    match proof {
        phalcom_semantic::generic_evidence::GenericConformanceProof::Blocked(phalcom_semantic::types::outcome::BlockReason::RecursiveFixpoint) => {}
        other => panic!("expected Blocked(RecursiveFixpoint), got {:?}", other),
    }
}

#[test]
fn budget_and_cancellation_preserved_in_rule_proof() {
    let fixture = Fixture::new(
        r#"
trait TraitA {}
trait TraitB {}

impl<T> TraitA for T
    where T |= TraitB
{}

class Candidate {}

impl TraitB for Candidate {}
"#,
    );

    let candidate_ty = fixture.ty("Candidate");
    let a_decl = fixture.decl("TraitA");
    let a_ref = phalcom_semantic::traits::TraitRef::new(a_decl, vec![]);

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(phalcom_semantic::types::parameter::TypeTerm::Canonical(candidate_ty), a_ref);

    // 1. Budget exceeded
    let budget = phalcom_semantic::types::outcome::QueryBudget::new(0);
    let cancel = phalcom_semantic::types::outcome::CancellationToken::new();
    let mut context = phalcom_semantic::generic_evidence::ConformanceProofContext::new(
        None,
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    )
    .with_derived_rules(Some(&fixture.analysis.snapshot.derived_conformance_rules))
    .with_budget_and_cancellation(budget, cancel);

    let proof = phalcom_semantic::generic_evidence::prove_conformance_with_context(&key, &mut context);
    assert!(matches!(proof, phalcom_semantic::generic_evidence::GenericConformanceProof::BudgetExceeded(_)));

    // 2. Cancellation
    let budget = phalcom_semantic::types::outcome::QueryBudget::default();
    let cancel = phalcom_semantic::types::outcome::CancellationToken::new();
    cancel.cancel();
    let mut context = phalcom_semantic::generic_evidence::ConformanceProofContext::new(
        None,
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    )
    .with_derived_rules(Some(&fixture.analysis.snapshot.derived_conformance_rules))
    .with_budget_and_cancellation(budget, cancel);

    let proof = phalcom_semantic::generic_evidence::prove_conformance_with_context(&key, &mut context);
    assert_eq!(proof, phalcom_semantic::generic_evidence::GenericConformanceProof::Cancelled);
}

#[test]
fn derived_rule_failed_has_no_partial_evidence() {
    let fixture = Fixture::new(
        r#"
trait Req1 {}
trait Req2 {}
trait DerivedTarget {}

impl<T> DerivedTarget for T
    where
        T |= Req1,
        T |= Req2
{}

class HalfCandidate {}

impl Req1 for HalfCandidate {}
"#,
    );

    let half_ty = fixture.ty("HalfCandidate");
    let target_decl = fixture.decl("DerivedTarget");
    let target_ref = phalcom_semantic::traits::TraitRef::new(target_decl, vec![]);

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let mut context = phalcom_semantic::generic_evidence::ConformanceProofContext::new(
        None,
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    )
    .with_derived_rules(Some(&fixture.analysis.snapshot.derived_conformance_rules));

    let key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(phalcom_semantic::types::parameter::TypeTerm::Canonical(half_ty), target_ref);

    let proof = phalcom_semantic::generic_evidence::prove_conformance_with_context(&key, &mut context);
    assert_eq!(proof, phalcom_semantic::generic_evidence::GenericConformanceProof::Missing);
}

#[test]
fn explicit_and_derived_exact_relation_conflict() {
    let fixture = Fixture::new(
        r#"
trait Iterator {}
trait Iterable {}

impl<T> Iterable for T
    where T |= Iterator
{}

class MyType {}

impl Iterator for MyType {}
impl Iterable for MyType {}
"#,
    );

    let my_ty = fixture.ty("MyType");
    let iterable_decl = fixture.decl("Iterable");
    let iterable_ref = phalcom_semantic::traits::TraitRef::new(iterable_decl, vec![]);

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let mut context = phalcom_semantic::generic_evidence::ConformanceProofContext::new(
        None,
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    )
    .with_derived_rules(Some(&fixture.analysis.snapshot.derived_conformance_rules));

    let key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(phalcom_semantic::types::parameter::TypeTerm::Canonical(my_ty), iterable_ref);

    let proof = phalcom_semantic::generic_evidence::prove_conformance_with_context(&key, &mut context);
    match proof {
        phalcom_semantic::generic_evidence::GenericConformanceProof::CoherenceConflict(impls) => {
            assert_eq!(impls.len(), 2);
        }
        other => panic!("expected CoherenceConflict proof result, got {:?}", other),
    }
}

#[test]
fn derived_associated_item_normalizes_through_antecedent_evidence() {
    let fixture = Fixture::new(
        r#"
trait Iterator {
    type Item
}

trait Iterable {
    type Iterator
    type Item
}

impl<T> Iterable for T
    where T |= Iterator
{
    type Iterator = T
    type Item = T::Item
}

class ItemType {}
class Counter {}

impl Iterator for Counter {
    type Item = ItemType
}
"#,
    );

    fixture.assert_no_error_diagnostics();

    let counter_ty = fixture.ty("Counter");
    let item_ty = fixture.ty("ItemType");
    let iterable_decl = fixture.decl("Iterable");
    let surface = fixture.analysis.snapshot.trait_surfaces.get(&iterable_decl).expect("Iterable surface");
    let item_req = surface.associated_type_by_name("Item").expect("Item requirement").requirement.clone();

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let trait_ref = phalcom_semantic::traits::TraitRef::new(iterable_decl, vec![]);
    let proj = store.associated_projection(counter_ty, trait_ref, item_req);

    let mut budget = phalcom_semantic::types::outcome::QueryBudget::default();
    let cancel = phalcom_semantic::types::outcome::CancellationToken::new();
    let mut context = phalcom_semantic::types::ProjectionNormalizationContext::new(
        phalcom_semantic::types::ProjectionNormalizationMode::Exact {
            conformance_index: &fixture.analysis.snapshot.conformance_index,
            witness_plans: &fixture.analysis.snapshot.conformance_witness_plans,
            trait_surfaces: &fixture.analysis.snapshot.trait_surfaces,
            derived_rules: Some(&fixture.analysis.snapshot.derived_conformance_rules),
            declarations: &fixture.analysis.snapshot.declarations,
            hierarchy: fixture.analysis.snapshot.hierarchy.as_ref(),
        },
        &mut budget,
        &cancel,
    );

    let norm_res = phalcom_semantic::types::normalize_type(&mut store, proj, &mut context);
    assert_eq!(norm_res, phalcom_semantic::types::ProjectionNormalizationResult::Normalized(item_ty));
}

#[test]
fn derived_associated_iterator_is_exact_subject() {
    let fixture = Fixture::new(
        r#"
trait Iterator {
    type Item
}

trait Iterable {
    type Iterator
    type Item
}

impl<T> Iterable for T
    where T |= Iterator
{
    type Iterator = T
    type Item = T::Item
}

class ItemType {}
class Counter {}

impl Iterator for Counter {
    type Item = ItemType
}
"#,
    );

    fixture.assert_no_error_diagnostics();

    let counter_ty = fixture.ty("Counter");
    let iterable_decl = fixture.decl("Iterable");
    let surface = fixture.analysis.snapshot.trait_surfaces.get(&iterable_decl).expect("Iterable surface");
    let iter_req = surface.associated_type_by_name("Iterator").expect("Iterator requirement").requirement.clone();

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let trait_ref = phalcom_semantic::traits::TraitRef::new(iterable_decl, vec![]);
    let proj = store.associated_projection(counter_ty, trait_ref, iter_req);

    let mut budget = phalcom_semantic::types::outcome::QueryBudget::default();
    let cancel = phalcom_semantic::types::outcome::CancellationToken::new();
    let mut context = phalcom_semantic::types::ProjectionNormalizationContext::new(
        phalcom_semantic::types::ProjectionNormalizationMode::Exact {
            conformance_index: &fixture.analysis.snapshot.conformance_index,
            witness_plans: &fixture.analysis.snapshot.conformance_witness_plans,
            trait_surfaces: &fixture.analysis.snapshot.trait_surfaces,
            derived_rules: Some(&fixture.analysis.snapshot.derived_conformance_rules),
            declarations: &fixture.analysis.snapshot.declarations,
            hierarchy: fixture.analysis.snapshot.hierarchy.as_ref(),
        },
        &mut budget,
        &cancel,
    );

    let norm_res = phalcom_semantic::types::normalize_type(&mut store, proj, &mut context);
    assert_eq!(norm_res, phalcom_semantic::types::ProjectionNormalizationResult::Normalized(counter_ty));
}

#[test]
fn derived_projection_normalizes_inside_nested_type() {
    let fixture = Fixture::new(
        r#"
trait Iterator {
    type Item
}

trait Iterable {
    type Iterator
    type Item
}

impl<T> Iterable for T
    where T |= Iterator
{
    type Iterator = T
    type Item = T::Item
}

class ItemType {}
class Counter {}

impl Iterator for Counter {
    type Item = ItemType
}
"#,
    );

    fixture.assert_no_error_diagnostics();

    let counter_ty = fixture.ty("Counter");
    let item_ty = fixture.ty("ItemType");
    let iterable_decl = fixture.decl("Iterable");
    let surface = fixture.analysis.snapshot.trait_surfaces.get(&iterable_decl).expect("Iterable surface");
    let item_req = surface.associated_type_by_name("Item").expect("Item requirement").requirement.clone();

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let trait_ref = phalcom_semantic::traits::TraitRef::new(iterable_decl, vec![]);
    let proj = store.associated_projection(counter_ty, trait_ref, item_req);

    let tuple_ty = store.tuple(
        vec![
            phalcom_semantic::types::store::TupleTypeElement::unlabeled(counter_ty),
            phalcom_semantic::types::store::TupleTypeElement::unlabeled(proj),
        ]
        .into_boxed_slice(),
    );

    let expected_tuple = store.tuple(
        vec![
            phalcom_semantic::types::store::TupleTypeElement::unlabeled(counter_ty),
            phalcom_semantic::types::store::TupleTypeElement::unlabeled(item_ty),
        ]
        .into_boxed_slice(),
    );

    let mut budget = phalcom_semantic::types::outcome::QueryBudget::default();
    let cancel = phalcom_semantic::types::outcome::CancellationToken::new();
    let mut context = phalcom_semantic::types::ProjectionNormalizationContext::new(
        phalcom_semantic::types::ProjectionNormalizationMode::Exact {
            conformance_index: &fixture.analysis.snapshot.conformance_index,
            witness_plans: &fixture.analysis.snapshot.conformance_witness_plans,
            trait_surfaces: &fixture.analysis.snapshot.trait_surfaces,
            derived_rules: Some(&fixture.analysis.snapshot.derived_conformance_rules),
            declarations: &fixture.analysis.snapshot.declarations,
            hierarchy: fixture.analysis.snapshot.hierarchy.as_ref(),
        },
        &mut budget,
        &cancel,
    );

    let norm_res = phalcom_semantic::types::normalize_type(&mut store, tuple_ty, &mut context);
    assert_eq!(norm_res, phalcom_semantic::types::ProjectionNormalizationResult::Normalized(expected_tuple));
}

#[test]
fn generic_derived_associated_projection_remains_symbolic() {
    let fixture = Fixture::new(
        r#"
trait Iterator {
    type Item
}

trait Iterable {
    type Iterator
    type Item
}

impl<T> Iterable for T
    where T |= Iterator
{
    type Iterator = T
    type Item = T::Item
}

class Container<T>
    where T |= Iterator
{}
"#,
    );

    fixture.assert_no_error_diagnostics();

    let container_decl = fixture.decl("Container");
    let iterable_decl = fixture.decl("Iterable");
    let iterator_decl = fixture.decl("Iterator");

    let container_info = fixture.analysis.snapshot.declarations.get(&container_decl).expect("container decl");
    let generics = container_info.generic_signature.as_ref().expect("generics on Container");
    let t_param = generics.parameters[0];

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let t_param_ty = store.parameter_form(t_param);

    let env = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
        &generics.constraints,
        generics.owner.clone(),
        &fixture.analysis.snapshot.trait_surfaces,
    )
    .expect("evidence env");

    let iter_surface = fixture.analysis.snapshot.trait_surfaces.get(&iterable_decl).expect("Iterable surface");
    let iterator_surface = fixture.analysis.snapshot.trait_surfaces.get(&iterator_decl).expect("Iterator surface");

    let item_req = iter_surface.associated_type_by_name("Item").expect("Item requirement").requirement.clone();
    let iter_req = iter_surface
        .associated_type_by_name("Iterator")
        .expect("Iterator requirement")
        .requirement
        .clone();
    let iter_item_req = iterator_surface.associated_type_by_name("Item").expect("Item requirement").requirement.clone();

    let iterable_ref = phalcom_semantic::traits::TraitRef::new(iterable_decl, vec![]);
    let iterator_ref = phalcom_semantic::traits::TraitRef::new(iterator_decl, vec![]);

    let proj_item = store.associated_projection(t_param_ty, iterable_ref.clone(), item_req);
    let proj_iter = store.associated_projection(t_param_ty, iterable_ref, iter_req);

    let mut budget = phalcom_semantic::types::outcome::QueryBudget::default();
    let cancel = phalcom_semantic::types::outcome::CancellationToken::new();
    let mut context = phalcom_semantic::types::ProjectionNormalizationContext::new(
        phalcom_semantic::types::ProjectionNormalizationMode::Generic {
            evidence: &env,
            conformance_index: Some(&fixture.analysis.snapshot.conformance_index),
            witness_plans: Some(&fixture.analysis.snapshot.conformance_witness_plans),
            trait_surfaces: &fixture.analysis.snapshot.trait_surfaces,
            derived_rules: Some(&fixture.analysis.snapshot.derived_conformance_rules),
            declarations: &fixture.analysis.snapshot.declarations,
            hierarchy: fixture.analysis.snapshot.hierarchy.as_ref(),
        },
        &mut budget,
        &cancel,
    );

    // <T as Iterable>::Iterator normalizes to T
    let iter_res = phalcom_semantic::types::normalize_type(&mut store, proj_iter, &mut context);
    assert_eq!(iter_res, phalcom_semantic::types::ProjectionNormalizationResult::Normalized(t_param_ty));

    // <T as Iterable>::Item normalizes to <T as Iterator>::Item (symbolic)
    let expected_sym = store.associated_projection(t_param_ty, iterator_ref, iter_item_req);
    let item_res = phalcom_semantic::types::normalize_type(&mut store, proj_item, &mut context);
    assert_eq!(item_res, phalcom_semantic::types::ProjectionNormalizationResult::Symbolic(expected_sym));
}

#[test]
fn derived_equality_solver_consumes_rule_binding() {
    let fixture = Fixture::new(
        r#"
trait Iterator {
    type Item
}

trait Iterable {
    type Iterator
    type Item
}

impl<T> Iterable for T
    where T |= Iterator
{
    type Iterator = T
    type Item = T::Item
}

class ItemType {}
class Counter {}

impl Iterator for Counter {
    type Item = ItemType
}
"#,
    );

    fixture.assert_no_error_diagnostics();

    let counter_ty = fixture.ty("Counter");
    let item_ty = fixture.ty("ItemType");
    let iterable_decl = fixture.decl("Iterable");
    let surface = fixture.analysis.snapshot.trait_surfaces.get(&iterable_decl).expect("Iterable surface");
    let item_req = surface.associated_type_by_name("Item").expect("Item requirement").requirement.clone();

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let trait_ref = phalcom_semantic::traits::TraitRef::new(iterable_decl, vec![]);
    let proj = store.associated_projection(counter_ty, trait_ref, item_req);

    let mut context = phalcom_semantic::generic_evidence::ConformanceProofContext::new(
        None,
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    )
    .with_derived_rules(Some(&fixture.analysis.snapshot.derived_conformance_rules));

    let constraint = phalcom_semantic::types::parameter::GenericConstraint::Equivalent {
        left: phalcom_semantic::types::parameter::TypeTerm::Canonical(proj),
        right: phalcom_semantic::types::parameter::TypeTerm::Canonical(item_ty),
    };
    let subst = phalcom_semantic::types::substitution::TypeSubstitution::new();
    let proof = phalcom_semantic::generic_evidence::prove_constraint(&constraint, &subst, &mut context);
    assert!(proof.is_proven(), "expected equivalence proof to succeed, got {:?}", proof);
}

#[test]
fn derived_iterator_rule_satisfies_iterable_contract_using_memoized_iterator_proof() {
    let fixture = Fixture::new(
        r#"
trait Iterator {
    type Item
}

trait Iterable
    where
        Self::Iterator |= Iterator,
        <Self::Iterator as Iterator>::Item == Self::Item
{
    type Iterator
    type Item
}

impl<T> Iterable for T
    where T |= Iterator
{
    type Iterator = T
    type Item = T::Item
}

class ItemType {}
class Counter {}

impl Iterator for Counter {
    type Item = ItemType
}
"#,
    );

    fixture.assert_no_error_diagnostics();

    let counter_ty = fixture.ty("Counter");
    let iterable_decl = fixture.decl("Iterable");
    let trait_ref = phalcom_semantic::traits::TraitRef::new(iterable_decl, vec![]);

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let mut context = phalcom_semantic::generic_evidence::ConformanceProofContext::new(
        None,
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    )
    .with_derived_rules(Some(&fixture.analysis.snapshot.derived_conformance_rules));

    let key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(phalcom_semantic::types::parameter::TypeTerm::Canonical(counter_ty), trait_ref);

    let proof = phalcom_semantic::generic_evidence::prove_conformance_with_context(&key, &mut context);
    match proof {
        phalcom_semantic::generic_evidence::GenericConformanceProof::Proven(phalcom_semantic::generic_evidence::GenericConformanceEvidence::Derived(
            derived,
        )) => {
            assert_eq!(derived.nested.len(), 1);
        }
        other => panic!("expected Proven(Derived) proof, got {:?}", other),
    }
}

//! Canonical conditional conformance source contract formation, impl-body assumptions, and coherence tests.

use crate::semantic::support::Fixture;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::types::parameter::{GenericConstraint, TypeTerm};
use phalcom_semantic::types::store::TypeData;

#[test]
fn cv01_source_formation_single_bound() {
    let fixture = Fixture::new(
        r#"
trait Printable {
    print() -> String
}

class Box<T> {}

impl<T> Printable for Box<T>
    where T |= Printable
{
    print() -> String {
        "box"
    }
}
"#,
    );

    fixture.assert_no_error_diagnostics();
    let printable_trait = fixture.decl("Printable");
    let box_decl = fixture.decl("Box");

    let conformance_index = &fixture.analysis.snapshot.conformance_index;
    let contribution = conformance_index
        .iter()
        .find(|(_, c)| {
            c.trait_ref.declaration == printable_trait && matches!(c.target, phalcom_semantic::impls::ConformanceTarget::Declaration(ref d) if d == &box_decl)
        })
        .expect("conformance contribution")
        .1;

    let sig = contribution.generic_signature.as_ref().expect("impl generic signature");
    assert_eq!(sig.parameters.len(), 1);
    assert_eq!(sig.constraints.len(), 1);

    match &sig.constraints[0] {
        GenericConstraint::Conforms { subject, trait_ref } => {
            assert_eq!(trait_ref.declaration, printable_trait);
            assert!(
                matches!(subject, TypeTerm::Canonical(ty) if matches!(fixture.analysis.snapshot.store.get(*ty), TypeData::Parameter(p) if *p == sig.parameters[0]))
            );
        }
        other => panic!("expected Conforms, got {:?}", other),
    }
}

#[test]
fn cv02_source_formation_multiple_bounds() {
    let fixture = Fixture::new(
        r#"
trait Alpha {}
trait Beta {}

class Box<T> {}

impl<T> Alpha for Box<T>
    where T |= Beta + Alpha
{}
"#,
    );

    fixture.assert_no_error_diagnostics();
    let alpha_trait = fixture.decl("Alpha");
    let beta_trait = fixture.decl("Beta");
    let box_decl = fixture.decl("Box");

    let conformance_index = &fixture.analysis.snapshot.conformance_index;
    let contribution = conformance_index
        .iter()
        .find(|(_, c)| {
            c.trait_ref.declaration == alpha_trait && matches!(c.target, phalcom_semantic::impls::ConformanceTarget::Declaration(ref d) if d == &box_decl)
        })
        .expect("conformance contribution")
        .1;

    let sig = contribution.generic_signature.as_ref().expect("impl generic signature");
    assert_eq!(sig.parameters.len(), 1);
    assert_eq!(sig.constraints.len(), 2);

    // Canonicalized: Alpha sorted before Beta
    match &sig.constraints[0] {
        GenericConstraint::Conforms { trait_ref, .. } => assert_eq!(trait_ref.declaration, alpha_trait),
        other => panic!("expected Conforms Alpha, got {:?}", other),
    }
    match &sig.constraints[1] {
        GenericConstraint::Conforms { trait_ref, .. } => assert_eq!(trait_ref.declaration, beta_trait),
        other => panic!("expected Conforms Beta, got {:?}", other),
    }
}

#[test]
fn cv03_source_formation_conditional_equivalent() {
    let fixture = Fixture::new(
        r#"
trait Marker {}
class Box<T> {}

impl<T> Marker for Box<T>
    where T == String
{}
"#,
    );

    fixture.assert_no_error_diagnostics();
    let marker_trait = fixture.decl("Marker");
    let box_decl = fixture.decl("Box");

    let contribution = fixture
        .analysis
        .snapshot
        .conformance_index
        .iter()
        .find(|(_, c)| {
            c.trait_ref.declaration == marker_trait && matches!(c.target, phalcom_semantic::impls::ConformanceTarget::Declaration(ref d) if d == &box_decl)
        })
        .expect("conformance contribution")
        .1;

    let sig = contribution.generic_signature.as_ref().expect("impl generic signature");
    assert_eq!(sig.constraints.len(), 1);
    assert!(matches!(&sig.constraints[0], GenericConstraint::Equivalent { .. }));
}

#[test]
fn cv04_source_formation_conditional_subtype() {
    let fixture = Fixture::new(
        r#"
trait Marker {}
class Base {}
class Box<T> {}

impl<T> Marker for Box<T>
    where T <: Base
{}
"#,
    );

    fixture.assert_no_error_diagnostics();
    let marker_trait = fixture.decl("Marker");
    let box_decl = fixture.decl("Box");

    let contribution = fixture
        .analysis
        .snapshot
        .conformance_index
        .iter()
        .find(|(_, c)| {
            c.trait_ref.declaration == marker_trait && matches!(c.target, phalcom_semantic::impls::ConformanceTarget::Declaration(ref d) if d == &box_decl)
        })
        .expect("conformance contribution")
        .1;

    let sig = contribution.generic_signature.as_ref().expect("impl generic signature");
    assert_eq!(sig.constraints.len(), 1);
    assert!(matches!(&sig.constraints[0], GenericConstraint::Subtype { .. }));
}

#[test]
fn cv05_specialization_conforms_and_equivalent() {
    let fixture = Fixture::new(
        r#"
trait Iterator {
    type Item
}
trait Printable {}

class Stream<I> {}

impl<I, T> Printable for Stream<I>
    where I |= Iterator<Item = T>, T |= Printable
{}
"#,
    );

    fixture.assert_no_error_diagnostics();
    let printable_trait = fixture.decl("Printable");
    let iterator_trait = fixture.decl("Iterator");
    let stream_decl = fixture.decl("Stream");

    let contribution = fixture
        .analysis
        .snapshot
        .conformance_index
        .iter()
        .find(|(_, c)| {
            c.trait_ref.declaration == printable_trait
                && matches!(c.target, phalcom_semantic::impls::ConformanceTarget::Declaration(ref d) if d == &stream_decl)
        })
        .expect("conformance contribution")
        .1;

    let sig = contribution.generic_signature.as_ref().expect("impl generic signature");
    assert_eq!(sig.parameters.len(), 2);
    // 3 constraints: Conforms(I, Iterator), Conforms(T, Printable), Equivalent(Item == T)
    assert_eq!(sig.constraints.len(), 3);

    let has_iter_conforms = sig
        .constraints
        .iter()
        .any(|c| matches!(c, GenericConstraint::Conforms { trait_ref, .. } if trait_ref.declaration == iterator_trait));
    let has_printable_conforms = sig
        .constraints
        .iter()
        .any(|c| matches!(c, GenericConstraint::Conforms { trait_ref, .. } if trait_ref.declaration == printable_trait));
    let has_equivalent = sig.constraints.iter().any(|c| matches!(c, GenericConstraint::Equivalent { .. }));

    assert!(has_iter_conforms);
    assert!(has_printable_conforms);
    assert!(has_equivalent);
}

#[test]
fn cv06_body_assumptions_witness_body_uses_assumed_behavior() {
    let fixture = Fixture::new(
        r#"
trait Printable {
    render() -> String
}

class Box<T> {
    _value: T

    @constructor new(v: T) {
        _value = v
    }

    get() -> T {
        _value
    }
}

impl<T> Printable for Box<T>
    where T |= Printable
{
    render() -> String {
        self.get().render()
    }
}
"#,
    );

    fixture.assert_no_error_diagnostics();
    let callable_analysis = fixture
        .analysis
        .snapshot
        .callable_analyses
        .values()
        .find(|c| matches!(c.callable.owner, phalcom_semantic::identity::CallableOwnerId::Conformance(_)))
        .expect("conformance callable analysis");
    let has_assumed = callable_analysis
        .expressions
        .values()
        .any(|expr| matches!(expr.trait_dispatch, Some(phalcom_semantic::trait_dispatch::TraitDispatchSite::Assumed(_))));
    assert!(has_assumed, "expected assumed trait dispatch for self.get().render()");
}

#[test]
fn cv07_negative_body_unjustified_behavior_rejected() {
    let fixture = Fixture::new_allowing_internal_incidents(
        r#"
trait Printable {
    render() -> String
}

trait Serializable {
    serialize() -> String
}

class Box<T> {
    _value: T

    @constructor new(v: T) {
        _value = v
    }

    get() -> T {
        _value
    }
}

impl<T> Printable for Box<T>
    where T |= Printable
{
    render() -> String {
        self.get().serialize()
    }
}
"#,
    );

    let callable_analysis = fixture
        .analysis
        .snapshot
        .callable_analyses
        .values()
        .find(|c| matches!(c.callable.owner, phalcom_semantic::identity::CallableOwnerId::Conformance(_)))
        .expect("conformance callable analysis");
    for expr in callable_analysis.expressions.values() {
        assert!(expr.trait_dispatch.is_none(), "unconstrained serialize must not have trait dispatch");
    }
}

#[test]
fn cv08_declaration_validity_future_dependent_not_required_globally() {
    let fixture = Fixture::new(
        r#"
trait Printable {}
class Unprintable {}
class Box<T> {}

impl<T> Printable for Box<T>
    where T |= Printable
{}
"#,
    );

    fixture.assert_no_error_diagnostics();
}

#[test]
fn cv09_formation_negative_malformed_condition_diagnosed() {
    let fixture = Fixture::new(
        r#"
trait Printable {}
class Box<T> {}

impl<T> Printable for Box<T>
    where NonExistent |= Printable
{}
"#,
    );

    assert!(
        !fixture.diagnostics(DiagnosticCode::AnnotationUnresolved).is_empty(),
        "expected AnnotationUnresolved for non-existent subject"
    );
}

#[test]
fn cv10_canonicalization_order_independent() {
    let fixture1 = Fixture::new(
        r#"
trait Alpha {}
trait Beta {}
class Box<T> {}

impl<T> Alpha for Box<T>
    where T |= Beta + Alpha
{}
"#,
    );

    let fixture2 = Fixture::new(
        r#"
trait Alpha {}
trait Beta {}
class Box<T> {}

impl<T> Alpha for Box<T>
    where T |= Alpha + Beta
{}
"#,
    );

    let alpha1 = fixture1.decl("Alpha");
    let alpha2 = fixture2.decl("Alpha");

    let sig1 = fixture1
        .analysis
        .snapshot
        .conformance_index
        .iter()
        .find(|(_, c)| c.trait_ref.declaration == alpha1)
        .unwrap()
        .1
        .generic_signature
        .as_ref()
        .unwrap();
    let sig2 = fixture2
        .analysis
        .snapshot
        .conformance_index
        .iter()
        .find(|(_, c)| c.trait_ref.declaration == alpha2)
        .unwrap()
        .1
        .generic_signature
        .as_ref()
        .unwrap();

    assert_eq!(sig1.constraints.len(), sig2.constraints.len());
}

#[test]
fn cv40_coherence_duplicate_conditional_heads_rejected() {
    let fixture = Fixture::new(
        r#"
trait Printable {}
class Box<T> {}

impl<T> Printable for Box<T>
    where T |= Printable
{}

impl<U> Printable for Box<U>
    where U |= Printable
{}
"#,
    );

    assert!(
        !fixture.diagnostics(DiagnosticCode::ImplConformanceOverlap).is_empty(),
        "expected ImplConformanceOverlap for identical conditional heads"
    );
}

#[test]
fn cv41_coherence_same_head_different_conditions_rejected() {
    let fixture = Fixture::new(
        r#"
trait Printable {}
trait Serializable {}
class Box<T> {}

impl<T> Printable for Box<T>
    where T |= Printable
{}

impl<T> Printable for Box<T>
    where T |= Serializable
{}
"#,
    );

    assert!(
        !fixture.diagnostics(DiagnosticCode::ImplConformanceOverlap).is_empty(),
        "expected ImplConformanceOverlap for same head with different conditions"
    );
}

#[test]
fn cv42_coherence_mixed_unconditional_and_conditional_rejected() {
    let fixture = Fixture::new(
        r#"
trait Printable {}
class Box<T> {}

impl<T> Printable for Box<T> {}

impl<T> Printable for Box<T>
    where T |= Printable
{}
"#,
    );

    assert!(
        !fixture.diagnostics(DiagnosticCode::ImplConformanceOverlap).is_empty(),
        "expected ImplConformanceOverlap for unconditional + conditional overlapping head"
    );
}

#[test]
fn cv43_coherence_generic_concrete_overlap_rejected() {
    let fixture = Fixture::new(
        r#"
trait Printable {}
class Box<T> {}
class User {}

impl<T> Printable for Box<T>
    where T |= Printable
{}

impl Printable for Box<User> {}
"#,
    );

    assert!(
        !fixture.diagnostics(DiagnosticCode::ImplConformanceOverlap).is_empty(),
        "expected ImplConformanceOverlap for generic Box<T> and concrete Box<User>"
    );
}

#[test]
fn cv44_coherence_declaration_order_independent() {
    let fixture = Fixture::new(
        r#"
trait Printable {}
class Box<T> {}
class User {}

impl Printable for Box<User> {}

impl<T> Printable for Box<T>
    where T |= Printable
{}
"#,
    );

    assert!(
        !fixture.diagnostics(DiagnosticCode::ImplConformanceOverlap).is_empty(),
        "expected ImplConformanceOverlap regardless of declaration order"
    );
}

#[test]
fn cv11_exact_nested_proof_one_level_succeeds() {
    let fixture = Fixture::new(
        r#"
trait Printable {
    render() -> String
}

class User {}
impl Printable for User {
    render() -> String { "user" }
}

class Box<T> {
    _value: T
    @constructor new(v: T) {
        _value = v
    }
}

impl<T> Printable for Box<T>
    where T |= Printable
{
    render() -> String { "boxed" }
}

class Consumer {
    @class
    run<T>(_ item: T) -> String
        where T |= Printable
    {
        item.render()
    }
}

class Probe {
    @class
    run() -> String {
        let b = Box.new(User.new())
        Consumer.run(b)
    }
}
"#,
    );

    fixture.assert_no_error_diagnostics();
}

#[test]
fn cv12_exact_nested_failure_unsatisfied_condition_rejected() {
    let fixture = Fixture::new_allowing_internal_incidents(
        r#"
trait Printable {
    render() -> String
}

class Unprintable {}

class Box<T> {
    _value: T
    @constructor new(_ v: T) {
        _value = v
    }
}

impl<T> Printable for Box<T>
    where T |= Printable
{
    render() -> String { "boxed" }
}

class Consumer {
    @class
    run<T>(_ item: T) -> String
        where T |= Printable
    {
        item.render()
    }
}

class Probe {
    @class
    run() -> String {
        let b = Box.new(Unprintable.new())
        Consumer.run(b)
    }
}
"#,
    );

    fixture.assert_diagnostic(DiagnosticCode::GenericConstraintUnsatisfied, 1);
}

#[test]
fn cv13_two_level_finite_recursive_success() {
    let fixture = Fixture::new(
        r#"
trait Printable {
    render() -> String
}

class User {}
impl Printable for User {
    render() -> String { "user" }
}

class Box<T> {
    _value: T
    @constructor new(_ v: T) {
        _value = v
    }
}

impl<T> Printable for Box<T>
    where T |= Printable
{
    render() -> String { "boxed" }
}

class Consumer {
    @class
    run<T>(_ item: T) -> String
        where T |= Printable
    {
        item.render()
    }
}

class Probe {
    @class
    run() -> String {
        let b = Box.new(Box.new(User.new()))
        Consumer.run(b)
    }
}
"#,
    );

    fixture.assert_no_error_diagnostics();
}

#[test]
fn cv14_deep_finite_nested_success() {
    let fixture = Fixture::new(
        r#"
trait Printable {
    render() -> String
}

class User {}
impl Printable for User {
    render() -> String { "user" }
}

class Box<T> {
    _value: T
    @constructor new(_ v: T) {
        _value = v
    }
}

impl<T> Printable for Box<T>
    where T |= Printable
{
    render() -> String { "boxed" }
}

class Consumer {
    @class
    run<T>(_ item: T) -> String
        where T |= Printable
    {
        item.render()
    }
}

class Probe {
    @class
    run() -> String {
        let b = Box.new(Box.new(Box.new(User.new())))
        Consumer.run(b)
    }
}
"#,
    );

    fixture.assert_no_error_diagnostics();
}

#[test]
fn cv19_assumed_nested_proof_in_generic_caller() {
    let fixture = Fixture::new(
        r#"
trait Printable {
    render() -> String
}

class Box<T> {
    _value: T
    @constructor new(_ v: T) {
        _value = v
    }
}

impl<T> Printable for Box<T>
    where T |= Printable
{
    render() -> String { "boxed" }
}

class Consumer {
    @class
    run<T>(_ item: T) -> String
        where T |= Printable
    {
        item.render()
    }
}

class Forwarder {
    @class
    forward<T>(_ item: Box<T>) -> String
        where T |= Printable
    {
        Consumer.run(item)
    }
}
"#,
    );

    fixture.assert_no_error_diagnostics();
}

#[test]
fn cv21_multiple_nested_conditions_succeed() {
    let fixture = Fixture::new(
        r#"
trait Printable {
    render() -> String
}

class User1 {}
impl Printable for User1 {
    render() -> String { "u1" }
}

class User2 {}
impl Printable for User2 {
    render() -> String { "u2" }
}

class Pair<A, B> {
    _a: A
    _b: B
    @constructor new(_ a: A, _ b: B) {
        _a = a
        _b = b
    }
}

impl<A, B> Printable for Pair<A, B>
    where A |= Printable, B |= Printable
{
    render() -> String { "pair" }
}

class Consumer {
    @class
    run<T>(_ item: T) -> String
        where T |= Printable
    {
        item.render()
    }
}

class Probe {
    @class
    run() -> String {
        let p = Pair.new(User1.new(), User2.new())
        Consumer.run(p)
    }
}
"#,
    );

    fixture.assert_no_error_diagnostics();
}

#[test]
fn cv30_self_cycle_terminates_non_success() {
    let fixture = Fixture::new_allowing_internal_incidents(
        r#"
trait Marker {}
class Box<T> {}
class User {}

impl<T> Marker for Box<T>
    where Box<T> |= Marker
{}
"#,
    );

    let marker_trait = fixture.decl("Marker");
    let trait_ref = phalcom_semantic::traits::TraitRef::new(marker_trait.clone(), vec![]);
    let box_decl = fixture.decl("Box");
    let user_decl = fixture.decl("User");
    let box_form = fixture.analysis.snapshot.declarations.form(&box_decl).expect("Box form");
    let user_form = fixture.analysis.snapshot.declarations.form(&user_decl).expect("User form");

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let target = store.apply_type_form(box_form, &[user_form]).unwrap();

    let mut context = phalcom_semantic::generic_evidence::ConformanceProofContext::new(
        None,
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    );

    let key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(phalcom_semantic::types::parameter::TypeTerm::Canonical(target), trait_ref);

    let proof = phalcom_semantic::generic_evidence::prove_conformance_with_context(&key, &mut context);
    assert!(
        matches!(
            proof,
            phalcom_semantic::generic_evidence::GenericConformanceProof::Blocked(phalcom_semantic::types::outcome::BlockReason::RecursiveFixpoint)
        ),
        "expected Blocked(RecursiveFixpoint) for self-cycle, got {:?}",
        proof
    );
}

#[test]
fn cv31_mutual_cycle_terminates_non_success() {
    let fixture = Fixture::new_allowing_internal_incidents(
        r#"
trait Alpha {}
trait Beta {}
class Box<T> {}
class User {}

impl<T> Alpha for Box<T>
    where Box<T> |= Beta
{}

impl<T> Beta for Box<T>
    where Box<T> |= Alpha
{}
"#,
    );

    let alpha_trait = fixture.decl("Alpha");
    let trait_ref = phalcom_semantic::traits::TraitRef::new(alpha_trait.clone(), vec![]);
    let box_decl = fixture.decl("Box");
    let user_decl = fixture.decl("User");
    let box_form = fixture.analysis.snapshot.declarations.form(&box_decl).expect("Box form");
    let user_form = fixture.analysis.snapshot.declarations.form(&user_decl).expect("User form");

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let target = store.apply_type_form(box_form, &[user_form]).unwrap();

    let mut context = phalcom_semantic::generic_evidence::ConformanceProofContext::new(
        None,
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    );

    let key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(phalcom_semantic::types::parameter::TypeTerm::Canonical(target), trait_ref);

    let proof = phalcom_semantic::generic_evidence::prove_conformance_with_context(&key, &mut context);
    assert!(
        matches!(
            proof,
            phalcom_semantic::generic_evidence::GenericConformanceProof::Blocked(phalcom_semantic::types::outcome::BlockReason::RecursiveFixpoint)
        ),
        "expected Blocked(RecursiveFixpoint) for mutual cycle, got {:?}",
        proof
    );
}

#[test]
fn cv33_memoization_reuses_completed_proof() {
    let fixture = Fixture::new(
        r#"
trait Printable {}
class User {}
impl Printable for User {}
class Box<T> {}
impl<T> Printable for Box<T> where T |= Printable {}
"#,
    );

    let printable_trait = fixture.decl("Printable");
    let trait_ref = phalcom_semantic::traits::TraitRef::new(printable_trait, vec![]);
    let user_decl = fixture.decl("User");
    let mut store = (*fixture.analysis.snapshot.store).clone();
    let user_ty = store.nominal(user_decl);

    let mut context = phalcom_semantic::generic_evidence::ConformanceProofContext::new(
        None,
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    );

    let key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(phalcom_semantic::types::parameter::TypeTerm::Canonical(user_ty), trait_ref);

    let proof1 = phalcom_semantic::generic_evidence::prove_conformance_with_context(&key, &mut context);
    assert!(matches!(proof1, phalcom_semantic::generic_evidence::GenericConformanceProof::Proven(_)));
    let steps_after_1 = context.budget.steps_taken;

    let proof2 = phalcom_semantic::generic_evidence::prove_conformance_with_context(&key, &mut context);
    assert_eq!(proof1, proof2);
    // Memo hit should not charge new proof step
    assert_eq!(context.budget.steps_taken, steps_after_1);
}

#[test]
fn cv35_budget_exhaustion_terminates_gracefully() {
    let fixture = Fixture::new(
        r#"
trait Printable {}
class User {}
impl Printable for User {}
class Box<T> {}
impl<T> Printable for Box<T> where T |= Printable {}
"#,
    );

    let printable_trait = fixture.decl("Printable");
    let trait_ref = phalcom_semantic::traits::TraitRef::new(printable_trait, vec![]);
    let user_decl = fixture.decl("User");
    let mut store = (*fixture.analysis.snapshot.store).clone();
    let user_ty = store.nominal(user_decl);

    let budget = phalcom_semantic::types::outcome::QueryBudget::new(0);
    let mut context = phalcom_semantic::generic_evidence::ConformanceProofContext::new(
        None,
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    )
    .with_budget_and_cancellation(budget, phalcom_semantic::types::outcome::CancellationToken::new());

    let key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(phalcom_semantic::types::parameter::TypeTerm::Canonical(user_ty), trait_ref);

    let proof = phalcom_semantic::generic_evidence::prove_conformance_with_context(&key, &mut context);
    assert!(matches!(proof, phalcom_semantic::generic_evidence::GenericConformanceProof::BudgetExceeded(_)));
}

#[test]
fn cv36_cancellation_terminates_proof() {
    let fixture = Fixture::new(
        r#"
trait Printable {}
class User {}
impl Printable for User {}
"#,
    );

    let printable_trait = fixture.decl("Printable");
    let trait_ref = phalcom_semantic::traits::TraitRef::new(printable_trait, vec![]);
    let user_decl = fixture.decl("User");
    let mut store = (*fixture.analysis.snapshot.store).clone();
    let user_ty = store.nominal(user_decl);

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
    .with_budget_and_cancellation(phalcom_semantic::types::outcome::QueryBudget::default(), cancel);

    let key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(phalcom_semantic::types::parameter::TypeTerm::Canonical(user_ty), trait_ref);

    let proof = phalcom_semantic::generic_evidence::prove_conformance_with_context(&key, &mut context);
    assert!(matches!(proof, phalcom_semantic::generic_evidence::GenericConformanceProof::Cancelled));
}

#[test]
fn cv24_conditional_equivalent_succeeds() {
    let fixture = Fixture::new(
        r#"
trait Printable {
    render() -> String
}

class Box<T> {
    _value: T
    @constructor new(_ v: T) {
        _value = v
    }
}

impl<T> Printable for Box<T>
    where T == String
{
    render() -> String { "string-box" }
}

class Consumer {
    @class
    run<T>(_ item: T) -> String
        where T |= Printable
    {
        item.render()
    }
}

class Probe {
    @class
    run() -> String {
        let b = Box.new("hello")
        Consumer.run(b)
    }
}
"#,
    );

    fixture.assert_no_error_diagnostics();
}

#[test]
fn cv25_conditional_equivalent_fails() {
    let fixture = Fixture::new_allowing_internal_incidents(
        r#"
trait Printable {
    render() -> String
}

class Box<T> {
    _value: T
    @constructor new(_ v: T) {
        _value = v
    }
}

impl<T> Printable for Box<T>
    where T == String
{
    render() -> String { "string-box" }
}

class Consumer {
    @class
    run<T>(_ item: T) -> String
        where T |= Printable
    {
        item.render()
    }
}

class Probe {
    @class
    run() -> String {
        let b = Box.new(42)
        Consumer.run(b)
    }
}
"#,
    );

    fixture.assert_diagnostic(DiagnosticCode::GenericConstraintUnsatisfied, 1);
}

#[test]
fn cv26_projection_equality_condition() {
    let fixture = Fixture::new(
        r#"
trait Printable {
    render() -> String
}

trait Sequence {
    type Item
}

class IntSeq {}
impl Sequence for IntSeq {
    type Item = String
}

class Box<T> {
    _value: T
    @constructor new(_ v: T) {
        _value = v
    }
}

impl<T> Printable for Box<T>
    where
        T |= Sequence,
        <T as Sequence>::Item == String
{
    render() -> String { "seq-box" }
}

class Consumer {
    @class
    run<T>(_ item: T) -> String
        where T |= Printable
    {
        item.render()
    }
}

class Probe {
    @class
    run() -> String {
        let b = Box.new(IntSeq.new())
        Consumer.run(b)
    }
}
"#,
    );

    fixture.assert_no_error_diagnostics();
}

#[test]
fn cv27_conditional_subtype_succeeds() {
    let fixture = Fixture::new(
        r#"
trait Printable {
    render() -> String
}

class Animal {}
class Dog is Animal {}

class Box<T> {
    _value: T
    @constructor new(_ v: T) {
        _value = v
    }
}

impl<T> Printable for Box<T>
    where T <: Animal
{
    render() -> String { "animal-box" }
}

class Consumer {
    @class
    run<T>(_ item: T) -> String
        where T |= Printable
    {
        item.render()
    }
}

class Probe {
    @class
    run() -> String {
        let b = Box.new(Dog.new())
        Consumer.run(b)
    }
}
"#,
    );

    fixture.assert_no_error_diagnostics();
}

#[test]
fn cv28_conditional_subtype_fails() {
    let fixture = Fixture::new_allowing_internal_incidents(
        r#"
trait Printable {
    render() -> String
}

class Animal {}
class Rock {}

class Box<T> {
    _value: T
    @constructor new(_ v: T) {
        _value = v
    }
}

impl<T> Printable for Box<T>
    where T <: Animal
{
    render() -> String { "animal-box" }
}

class Consumer {
    @class
    run<T>(_ item: T) -> String
        where T |= Printable
    {
        item.render()
    }
}

class Probe {
    @class
    run() -> String {
        let b = Box.new(Rock.new())
        Consumer.run(b)
    }
}
"#,
    );

    fixture.assert_diagnostic(DiagnosticCode::GenericConstraintUnsatisfied, 1);
}

#[test]
fn cv29_mixed_conditions() {
    let fixture = Fixture::new(
        r#"
trait Printable {
    render() -> String
}

trait Named {
    name() -> String
}

class Animal {}
class Dog is Animal {}

impl Named for Dog {
    name() -> String { "dog" }
}

class Box<T> {
    _value: T
    @constructor new(_ v: T) {
        _value = v
    }
}

impl<T> Printable for Box<T>
    where
        T |= Named,
        T <: Animal
{
    render() -> String { "named-animal-box" }
}

class Consumer {
    @class
    run<T>(_ item: T) -> String
        where T |= Printable
    {
        item.render()
    }
}

class Probe {
    @class
    run() -> String {
        let b = Box.new(Dog.new())
        Consumer.run(b)
    }
}
"#,
    );

    fixture.assert_no_error_diagnostics();
}

#[test]
fn cv53_conditional_projection_normalizes() {
    let fixture = Fixture::new(
        r#"
trait Printable {}
class User {}
impl Printable for User {}

trait Container {
    type Element
}

class Box<T> {}

impl<T> Container for Box<T>
    where T |= Printable
{
    type Element = T
}
"#,
    );

    fixture.assert_no_error_diagnostics();

    let container_trait = fixture.decl("Container");
    let trait_ref = phalcom_semantic::traits::TraitRef::new(container_trait, vec![]);
    let user_ty = fixture.ty("User");
    let box_form = fixture.ty("Box");

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let box_user_ty = store.apply_type_form(box_form, &[user_ty]).unwrap();

    let mut context = phalcom_semantic::generic_evidence::ConformanceProofContext::new(
        None,
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    );

    let key = phalcom_semantic::generic_evidence::GenericConformanceKey::new(phalcom_semantic::types::parameter::TypeTerm::Canonical(box_user_ty), trait_ref);

    let proof = phalcom_semantic::generic_evidence::prove_conformance_with_context(&key, &mut context);
    match proof {
        phalcom_semantic::generic_evidence::GenericConformanceProof::Proven(phalcom_semantic::generic_evidence::GenericConformanceEvidence::Derived(
            derived,
        )) => {
            let exact_source = derived.exact_source().unwrap();
            assert_eq!(exact_source.associated_types.len(), 1);
            let (_req, binding) = exact_source.associated_types.iter().next().unwrap();
            assert_eq!(binding.value, phalcom_semantic::impls::AssociatedTypeBindingValue::Type(user_ty));
        }
        other => panic!("expected Derived conformance evidence, got {:?}", other),
    }
}

#[test]
fn cv54_associated_specialization_in_condition() {
    let fixture = Fixture::new(
        r#"
trait Printable {
    render() -> String
}

trait Sequence {
    type Item
}

class StrSeq {}
impl Sequence for StrSeq {
    type Item = String
}

class Box<T> {
    _value: T
    @constructor new(_ v: T) {
        _value = v
    }
}

impl<T> Printable for Box<T>
    where T |= Sequence<Item = String>
{
    render() -> String { "str-seq-box" }
}

class Consumer {
    @class
    run<T>(_ item: T) -> String
        where T |= Printable
    {
        item.render()
    }
}

class Probe {
    @class
    run() -> String {
        let b = Box.new(StrSeq.new())
        Consumer.run(b)
    }
}
"#,
    );

    fixture.assert_no_error_diagnostics();
}

#[test]
fn cv56_proof_normalization_cycle_terminates() {
    let fixture = Fixture::new_allowing_internal_incidents(
        r#"
trait Printable {
    render() -> String
}

class Box<T> {
    _value: T
    @constructor new(_ v: T) {
        _value = v
    }
}

impl<T> Printable for Box<T>
    where <Box<T> as Printable>::Item == String
{
    type Item = String
    render() -> String { "cycle" }
}
"#,
    );

    // Should not infinite loop or panic
    let _ = fixture.analysis.snapshot.conformance_index;
}

#[test]
fn cv55_imported_conditional_associated_projection_normalizes() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("package.ph"), "").unwrap();
    let provider_path = root.path().join("provider.ph");
    let consumer_path = root.path().join("consumer.ph");

    let provider = phalcom_modules::SourceLocation {
        source_id: phalcom_modules::SourceId(provider_path.to_string_lossy().into()),
        display_path: provider_path.to_path_buf(),
    };
    let consumer = phalcom_modules::SourceLocation {
        source_id: phalcom_modules::SourceId(consumer_path.to_string_lossy().into()),
        display_path: consumer_path.to_path_buf(),
    };

    let mut session = phalcom_semantic::session::SemanticWorkspaceSession::new();
    let published = session
        .apply_module_mutations([
            phalcom_modules::WorkspaceSourceBatchMutation::SetOverlay {
                source: provider,
                text: std::sync::Arc::from(
                    r#"
trait Printable {}
class User {}
impl Printable for User {}

trait Container {
    type Element
}

class Box<T> {}

impl<T> Container for Box<T>
    where T |= Printable
{
    type Element = T
}

export Printable, User, Container, Box
"#,
                ),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
            phalcom_modules::WorkspaceSourceBatchMutation::SetOverlay {
                source: consumer,
                text: std::sync::Arc::from(
                    r#"
import .provider as P

class Consumer {
    @class
    element_type() {
        let mut b: P.Box<P.User>
    }
}
"#,
                ),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("workspace setup should succeed");

    assert!(!published.snapshot.has_errors(), "diagnostics: {:?}", published.snapshot.diagnostics);
}

#[test]
fn cv58_cross_module_conditional_proof_proves_consumer_goal() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("package.ph"), "").unwrap();
    let provider_path = root.path().join("provider.ph");
    let consumer_path = root.path().join("consumer.ph");

    let provider = phalcom_modules::SourceLocation {
        source_id: phalcom_modules::SourceId(provider_path.to_string_lossy().into()),
        display_path: provider_path.to_path_buf(),
    };
    let consumer = phalcom_modules::SourceLocation {
        source_id: phalcom_modules::SourceId(consumer_path.to_string_lossy().into()),
        display_path: consumer_path.to_path_buf(),
    };

    let mut session = phalcom_semantic::session::SemanticWorkspaceSession::new();
    let published = session
        .apply_module_mutations([
            phalcom_modules::WorkspaceSourceBatchMutation::SetOverlay {
                source: provider,
                text: std::sync::Arc::from(
                    r#"
trait Printable {
    render() -> String
}

class User {}
impl Printable for User {
    render() -> String { "user" }
}

class Box<T> {
    _value: T
    @constructor new(_ v: T) {
        _value = v
    }
}

impl<T> Printable for Box<T>
    where T |= Printable
{
    render() -> String { "box" }
}

export Printable, User, Box
"#,
                ),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
            phalcom_modules::WorkspaceSourceBatchMutation::SetOverlay {
                source: consumer,
                text: std::sync::Arc::from(
                    r#"
import .provider as P

class Discharger {
    @class
    display<T>(_ item: T) -> String
        where T |= P.Printable
    {
        item.render()
    }
}

class Main {
    @class
    run() -> String {
        let b = P.Box.new(P.User.new())
        Discharger.display(b)
    }
}
"#,
                ),
                revision: phalcom_modules::SourceRevision(1),
                recovered_program: None,
            },
        ])
        .expect("cross-module conditional proof should compile");

    assert!(!published.snapshot.has_errors(), "diagnostics: {:?}", published.snapshot.diagnostics);
}

fn make_input(module: phalcom_modules::identity::ModuleId, source: &str, generation: u64) -> phalcom_semantic::workspace::SemanticWorkspaceInput {
    let parsed = phalcom_ast::parse(source, 0);
    let program = std::sync::Arc::new(parsed.program);
    let local_globals = phalcom_modules::interface::InterfaceBuilder::build(module.clone(), phalcom_modules::source::ModuleKind::Module, &program)
        .map(|interface| {
            interface
                .declarations
                .keys()
                .enumerate()
                .map(|(index, name)| (name.clone().into_boxed_str(), phalcom_modules::linker::GlobalBindingId(index as u32)))
                .collect()
        })
        .unwrap_or_default();

    let mut linked_modules = std::collections::BTreeMap::new();
    linked_modules.insert(
        module.clone(),
        phalcom_modules::linker::LinkedModule {
            interface: phalcom_modules::interface::LinkedModuleInterface {
                module: module.clone(),
                kind: phalcom_modules::source::ModuleKind::Module,
                exports: std::collections::BTreeMap::new(),
                metadata: phalcom_modules::metadata::ModuleMetadata::default(),
            },
            bindings: phalcom_modules::linker::ModuleBindingLayout {
                local_globals,
                ..phalcom_modules::linker::ModuleBindingLayout::default()
            },
            linked_reads: Vec::new(),
            runtime_dependencies: Vec::new(),
        },
    );
    let mut sources = std::collections::BTreeMap::new();
    sources.insert(
        module.clone(),
        std::sync::Arc::new(phalcom_semantic::source::ParsedModuleUnit::new(
            module.clone(),
            phalcom_modules::source::ModuleKind::Module,
            None,
            std::sync::Arc::from(source),
            program,
        )),
    );
    let linked = std::sync::Arc::new(phalcom_modules::linker::LinkedProgram {
        universe: std::sync::Arc::new(phalcom_modules::project::ProjectUniverse::new()),
        modules: linked_modules,
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: module.clone(),
        initialization_order: vec![module],
    });
    phalcom_semantic::workspace::SemanticWorkspaceInput::new(linked, sources, generation)
}

#[test]
fn cv60_incremental_nested_base_conformance_change_invalidates_outer_consumer() {
    let module = phalcom_modules::identity::ModuleId::resolved(
        phalcom_modules::identity::ResolvedProjectId::from_raw(960),
        phalcom_modules::identity::ModulePath::from_components(vec![phalcom_modules::identity::ModuleComponent::from_identifier("main").unwrap()]),
    );

    let source_a = r#"
trait Printable {
    render() -> String
}

class User {
    @constructor new() {}
}
impl Printable for User {
    render() -> String { "user" }
}

class Box<T> {
    _value: T
    @constructor new(_ v: T) {
        _value = v
    }
}

impl<T> Printable for Box<T>
    where T |= Printable
{
    render() -> String { "box" }
}

class Discharger {
    @class
    display<T>(_ item: T) -> String
        where T |= Printable
    {
        item.render()
    }
}

class Main {
    @class
    run() -> String {
        let b = Box.new(User.new())
        Discharger.display(b)
    }
}
"#;

    let source_b = r#"
trait Printable {
    render() -> String
}

class User {
    @constructor new() {}
}

class Box<T> {
    _value: T
    @constructor new(_ v: T) {
        _value = v
    }
}

impl<T> Printable for Box<T>
    where T |= Printable
{
    render() -> String { "box" }
}

class Discharger {
    @class
    display<T>(_ item: T) -> String
        where T |= Printable
    {
        item.render()
    }
}

class Main {
    @class
    run() -> String {
        let b = Box.new(User.new())
        Discharger.display(b)
    }
}
"#;

    let mut incremental = phalcom_semantic::session::SemanticWorkspaceSession::new();
    let first = incremental.update(make_input(module.clone(), source_a, 1));
    assert!(!first.snapshot.has_errors());

    let second = incremental.update(make_input(module.clone(), source_b, 2));
    assert!(
        second.snapshot.has_errors(),
        "expected GenericConstraintUnsatisfied after removing base conformance"
    );
}

#[test]
fn cv61_incremental_condition_change_invalidates_consumer() {
    let module = phalcom_modules::identity::ModuleId::resolved(
        phalcom_modules::identity::ResolvedProjectId::from_raw(961),
        phalcom_modules::identity::ModulePath::from_components(vec![phalcom_modules::identity::ModuleComponent::from_identifier("main").unwrap()]),
    );

    let source_a = r#"
trait Printable {
    render() -> String
}

trait Extra {}

class User {
    @constructor new() {}
}
impl Printable for User {
    render() -> String { "user" }
}

class Box<T> {
    _value: T
    @constructor new(_ v: T) {
        _value = v
    }
}

impl<T> Printable for Box<T>
    where T |= Printable
{
    render() -> String { "box" }
}

class Discharger {
    @class
    display<T>(_ item: T) -> String
        where T |= Printable
    {
        item.render()
    }
}

class Main {
    @class
    run() -> String {
        let b = Box.new(User.new())
        Discharger.display(b)
    }
}
"#;

    let source_b = r#"
trait Printable {
    render() -> String
}

trait Extra {}

class User {
    @constructor new() {}
}
impl Printable for User {
    render() -> String { "user" }
}

class Box<T> {
    _value: T
    @constructor new(_ v: T) {
        _value = v
    }
}

impl<T> Printable for Box<T>
    where
        T |= Printable,
        T |= Extra
{
    render() -> String { "box" }
}

class Discharger {
    @class
    display<T>(_ item: T) -> String
        where T |= Printable
    {
        item.render()
    }
}

class Main {
    @class
    run() -> String {
        let b = Box.new(User.new())
        Discharger.display(b)
    }
}
"#;

    let mut incremental = phalcom_semantic::session::SemanticWorkspaceSession::new();
    let first = incremental.update(make_input(module.clone(), source_a, 1));
    assert!(!first.snapshot.has_errors());

    let second = incremental.update(make_input(module.clone(), source_b, 2));
    assert!(
        second.snapshot.has_errors(),
        "expected GenericConstraintUnsatisfied after strengthening condition"
    );
}

#[test]
fn cv62_incremental_canonical_reorder_preserves_fingerprint() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("package.ph"), "").unwrap();
    let provider_path = root.path().join("provider.ph");

    let provider = phalcom_modules::SourceLocation {
        source_id: phalcom_modules::SourceId(provider_path.to_string_lossy().into()),
        display_path: provider_path.to_path_buf(),
    };

    let mut session = phalcom_semantic::session::SemanticWorkspaceSession::new();

    let pub1 = session
        .apply_module_mutations([phalcom_modules::WorkspaceSourceBatchMutation::SetOverlay {
            source: provider.clone(),
            text: std::sync::Arc::from(
                r#"
trait Printable {}
trait Extra {}
class Box<T> {}

impl<T> Printable for Box<T>
    where
        T |= Printable,
        T |= Extra
{}
"#,
            ),
            revision: phalcom_modules::SourceRevision(1),
            recovered_program: None,
        }])
        .expect("initial setup");

    let pub2 = session
        .apply_module_mutations([phalcom_modules::WorkspaceSourceBatchMutation::SetOverlay {
            source: provider,
            text: std::sync::Arc::from(
                r#"
trait Printable {}
trait Extra {}
class Box<T> {}

impl<T> Printable for Box<T>
    where
        T |= Extra,
        T |= Printable
{}
"#,
            ),
            revision: phalcom_modules::SourceRevision(2),
            recovered_program: None,
        }])
        .expect("reorder setup");

    // The witness plan fingerprints should match
    let impl_id = pub1.snapshot.conformance_witness_plans.keys().next().unwrap();
    let fp1 = pub1.snapshot.conformance_witness_plans[impl_id].fingerprint;
    let fp2 = pub2.snapshot.conformance_witness_plans[impl_id].fingerprint;
    assert_eq!(
        fp1, fp2,
        "canonical reordering of where-clause conditions must preserve witness plan fingerprint"
    );
}

#[test]
fn cv63_cold_and_incremental_parity() {
    let source_a = "trait Printable {\n  render() -> String\n}\nclass User {}\nimpl Printable for User {\n  render() -> String { \"user\" }\n}\nclass Box<T> {\n  _value: T\n  @constructor new(_ v: T) {\n    _value = v\n  }\n}\nimpl<T> Printable for Box<T>\n  where T |= Printable\n{\n  render() -> String { \"box\" }\n}\nclass Discharger {\n  @class\n  display<T>(_ item: T) -> String\n    where T |= Printable\n  {\n    item.render()\n  }\n}\nclass Main {\n  @class\n  run() -> String {\n    let b = Box.new(User.new())\n    Discharger.display(b)\n  }\n}\n";

    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("package.ph"), "").unwrap();
    let path = root.path().join("main.ph");
    let loc = phalcom_modules::SourceLocation {
        source_id: phalcom_modules::SourceId(path.to_string_lossy().into()),
        display_path: path.to_path_buf(),
    };

    // Incremental session (2 steps)
    let mut inc_session = phalcom_semantic::session::SemanticWorkspaceSession::new();
    let _ = inc_session
        .apply_module_mutations([phalcom_modules::WorkspaceSourceBatchMutation::SetOverlay {
            source: loc.clone(),
            text: std::sync::Arc::from("class Dummy {}\n"),
            revision: phalcom_modules::SourceRevision(1),
            recovered_program: None,
        }])
        .unwrap();
    let inc_pub = inc_session
        .apply_module_mutations([phalcom_modules::WorkspaceSourceBatchMutation::SetOverlay {
            source: loc.clone(),
            text: std::sync::Arc::from(source_a),
            revision: phalcom_modules::SourceRevision(2),
            recovered_program: None,
        }])
        .unwrap();

    // Cold session (1 step)
    let mut cold_session = phalcom_semantic::session::SemanticWorkspaceSession::new();
    let cold_pub = cold_session
        .apply_module_mutations([phalcom_modules::WorkspaceSourceBatchMutation::SetOverlay {
            source: loc,
            text: std::sync::Arc::from(source_a),
            revision: phalcom_modules::SourceRevision(1),
            recovered_program: None,
        }])
        .unwrap();

    assert_eq!(inc_pub.snapshot.has_errors(), cold_pub.snapshot.has_errors());
    assert_eq!(
        inc_pub.snapshot.conformance_index.iter().count(),
        cold_pub.snapshot.conformance_index.iter().count()
    );
}

#[test]
fn cv64_diagnostic_identifies_nested_failing_obligation() {
    let fixture = Fixture::new_allowing_internal_incidents(
        r#"
trait Printable {
    render() -> String
}

class Unprintable {}

class Box<T> {
    _value: T
    @constructor new(_ v: T) {
        _value = v
    }
}

impl<T> Printable for Box<T>
    where T |= Printable
{
    render() -> String { "boxed" }
}

class Consumer {
    @class
    run<T>(_ item: T) -> String
        where T |= Printable
    {
        item.render()
    }
}

class Probe {
    @class
    run() -> String {
        let b = Box.new(Unprintable.new())
        Consumer.run(b)
    }
}
"#,
    );

    let diags = fixture.diagnostics(DiagnosticCode::GenericConstraintUnsatisfied);
    assert_eq!(diags.len(), 1, "expected GenericConstraintUnsatisfied diagnostic");
}

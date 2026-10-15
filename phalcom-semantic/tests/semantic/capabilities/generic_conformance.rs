//! Canonical generic conformance constraint ownership, representation, and canonicalization tests.

use crate::semantic::support::Fixture;
use phalcom_common::selector::SelectorBase;
use phalcom_semantic::db::fingerprint::callable_signature_product_fingerprint;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::metadata::MetadataExporter;
use phalcom_semantic::types::parameter::{GenericConstraint, TypeParameterOwner, TypeTerm};
use phalcom_semantic::types::store::TypeData;
use phalcom_type_meta::generic::GenericConstraintRef;
use phalcom_type_meta::header::MetadataProfile;
use phalcom_type_meta::validate::{ValidationLimits, validate_metadata_bundle};

#[test]
fn generic_conformance_constraint_lowering_and_ordering() {
    let fixture = Fixture::new(
        r#"
trait Alpha {}
trait Beta {}

class Probe {
  @class
  run<T>(_ value: T) -> T
    where T |= Beta + Alpha
  {
    value
  }
}

"#,
    );

    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let run_sig = fixture.analysis.snapshot.callable_signatures.get(&run.callable).expect("run signature");
    let generics = run_sig.generics.as_ref().expect("generics");
    assert!(matches!(generics.owner, TypeParameterOwner::Callable(ref owner) if owner == &run.callable));
    assert_eq!(generics.parameters.len(), 1);
    // T |= Beta + Alpha should be canonicalized and sorted into Alpha then Beta!
    assert_eq!(generics.constraints.len(), 2);

    let t_param = generics.parameters[0];
    let alpha_trait = fixture.decl("Alpha");
    let beta_trait = fixture.decl("Beta");

    match &generics.constraints[0] {
        GenericConstraint::Conforms { subject, trait_ref } => {
            assert!(matches!(subject, TypeTerm::Canonical(ty) if matches!(fixture.analysis.snapshot.store.get(*ty), TypeData::Parameter(p) if *p == t_param)));
            assert_eq!(trait_ref.declaration, alpha_trait);
        }
        other => panic!("expected Conforms Alpha, got {:?}", other),
    }

    match &generics.constraints[1] {
        GenericConstraint::Conforms { subject, trait_ref } => {
            assert!(matches!(subject, TypeTerm::Canonical(ty) if matches!(fixture.analysis.snapshot.store.get(*ty), TypeData::Parameter(p) if *p == t_param)));
            assert_eq!(trait_ref.declaration, beta_trait);
        }
        other => panic!("expected Conforms Beta, got {:?}", other),
    }
}

#[test]
fn stable_metadata_preserves_direct_conformance_constraint() {
    let fixture = Fixture::new(
        r#"
trait Printable {}

class Probe {
  @class
  run<T>(_ value: T) -> T
    where T |= Printable
  {
    value
  }
}
"#,
    );

    let exporter = MetadataExporter::new(
        &fixture.analysis.snapshot.store,
        None,
        Some(&fixture.analysis.snapshot.callable_signatures),
        None,
        MetadataProfile::RuntimePublic,
    );
    let bundle = exporter.build_complete_test_bundle(&[]).expect("generic callable metadata");
    validate_metadata_bundle(&bundle, &ValidationLimits::default()).expect("valid generic callable metadata");

    let constraint = bundle
        .generic_signatures
        .iter()
        .flat_map(|record| record.constraints.iter())
        .find(|constraint| {
            matches!(
                constraint,
                GenericConstraintRef::Conforms { trait_application, .. }
                    if trait_application.declaration.path.last().map(|name| name.as_ref()) == Some("Printable")
            )
        })
        .expect("direct conformance constraint");
    let GenericConstraintRef::Conforms { subject, trait_application } = constraint else {
        panic!("expected stable conformance relation, got {constraint:?}");
    };
    assert!(bundle.types.get(subject.0 as usize).is_some(), "subject type node must be present");
    assert_eq!(trait_application.declaration.path.last().map(|name| name.as_ref()), Some("Printable"));
    assert!(trait_application.arguments.is_empty());
}

#[test]
fn direct_conformance_proof_preserves_exact_and_terminal_outcomes() {
    let fixture = Fixture::new(
        r#"
trait Alpha {}
trait Beta {}

class User {}

impl Alpha for User {}

class Probe {
  @class
  run<T>(_ value: T) -> T
    where T |= Beta
  {
    value
  }
}
"#,
    );

    let user_ty = fixture.ty("User");
    let alpha_trait = fixture.decl("Alpha");
    let beta_trait = fixture.decl("Beta");

    let alpha_ref = phalcom_semantic::traits::TraitRef::new(alpha_trait.clone(), vec![]);
    let beta_ref = phalcom_semantic::traits::TraitRef::new(beta_trait.clone(), vec![]);

    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let run_sig = fixture.analysis.snapshot.callable_signatures.get(&run.callable).expect("run signature");
    let generics = run_sig.generics.as_ref().expect("generics");
    let t_param = generics.parameters[0];

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let t_param_ty = store.parameter_form(t_param);

    // Build evidence env from the callable generics
    let env = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
        &generics.constraints,
        generics.owner.clone(),
        &fixture.analysis.snapshot.trait_surfaces,
    )
    .expect("valid evidence layout");

    // 1. Exact proof for User |= Alpha
    let exact_proof = phalcom_semantic::generic_evidence::query_generic_conformance_proof(
        &TypeTerm::Canonical(user_ty),
        &alpha_ref,
        Some(&env),
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        None,
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    );
    match exact_proof {
        phalcom_semantic::generic_evidence::GenericConformanceProof::Proven(phalcom_semantic::generic_evidence::GenericConformanceEvidence::Exact(
            evidence,
        )) => {
            assert_eq!(evidence.exact_target, user_ty);
            assert_eq!(evidence.exact_trait_ref, alpha_ref);
        }
        other => panic!("expected Exact proof for User |= Alpha, got {:?}", other),
    }

    // 2. Assumed proof for T |= Beta
    let assumed_proof = phalcom_semantic::generic_evidence::query_generic_conformance_proof(
        &TypeTerm::Canonical(t_param_ty),
        &beta_ref,
        Some(&env),
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        None,
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    );
    match assumed_proof {
        phalcom_semantic::generic_evidence::GenericConformanceProof::Proven(phalcom_semantic::generic_evidence::GenericConformanceEvidence::Assumed(
            assumed,
        )) => {
            assert_eq!(assumed.key.subject, TypeTerm::Canonical(t_param_ty));
            assert_eq!(assumed.key.trait_ref, beta_ref);
        }
        other => panic!("expected Assumed proof for T |= Beta, got {:?}", other),
    }

    // 3. Missing proof for User |= Beta
    let missing_proof = phalcom_semantic::generic_evidence::query_generic_conformance_proof(
        &TypeTerm::Canonical(user_ty),
        &beta_ref,
        Some(&env),
        Some(&fixture.analysis.snapshot.conformance_index),
        Some(&fixture.analysis.snapshot.conformance_witness_plans),
        Some(&fixture.analysis.snapshot.trait_surfaces),
        None,
        Some(&fixture.analysis.snapshot.declarations),
        Some(fixture.analysis.snapshot.hierarchy.as_ref()),
        Some(&mut store),
    );
    assert_eq!(missing_proof, phalcom_semantic::generic_evidence::GenericConformanceProof::Missing);
}

#[test]
fn owner_and_callable_conformance_assumptions_compose_lexically() {
    let fixture = Fixture::new(
        r#"
trait Alpha {
  alpha_call -> String
}

trait Beta {
  beta_call -> String
}

class Box<T> where T |= Alpha {
  run<U>(_ value: T, _ other: U) -> String
    where U |= Beta
  {
    let a = value.alpha_call
    let b = other.beta_call
    a
  }
}
"#,
    );

    let run = fixture.callable("Box", "run", DispatchSide::Instance);

    // Check that both alpha_call and beta_call expressions resolved to TraitDispatchSite::Assumed
    let mut assumed_dispatches = Vec::new();
    for expr in run.expressions.values() {
        if let Some(phalcom_semantic::trait_dispatch::TraitDispatchSite::Assumed(selection)) = &expr.trait_dispatch {
            assumed_dispatches.push(selection.clone());
        }
    }

    assert_eq!(
        assumed_dispatches.len(),
        2,
        "expected 2 assumed trait dispatches in body, found {:?}",
        assumed_dispatches
    );

    let alpha_trait = fixture.decl("Alpha");
    let beta_trait = fixture.decl("Beta");

    assert!(assumed_dispatches.iter().any(
        |d| d.conformance.trait_ref.declaration == alpha_trait && matches!(&d.requirement.selector.base, SelectorBase::Named(name) if name == "alpha_call")
    ));
    assert!(
        assumed_dispatches
            .iter()
            .any(|d| d.conformance.trait_ref.declaration == beta_trait
                && matches!(&d.requirement.selector.base, SelectorBase::Named(name) if name == "beta_call"))
    );
}

#[test]
fn generic_evidence_layout_is_order_independent_and_subject_sensitive() {
    let fixture1 = Fixture::new(
        r#"
trait Alpha {
  first -> String
}
trait Beta {
  second -> String
}

class Probe {
  @class
  run<T, U>(_ t: T, _ u: U)
    where T |= Beta + Alpha, U |= Alpha
  {}
}
"#,
    );

    let fixture2 = Fixture::new(
        r#"
trait Alpha {
  first -> String
}
trait Beta {
  second -> String
}

class Probe {
  @class
  run<T, U>(_ t: T, _ u: U)
    where U |= Alpha, T |= Alpha, T |= Beta
  {}
}
"#,
    );

    let run1 = fixture1.callable("Probe", "run", DispatchSide::Class);
    let sig1 = fixture1.analysis.snapshot.callable_signatures.get(&run1.callable).expect("sig1");
    let gen1 = sig1.generics.as_ref().expect("gen1");
    let env1 = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
        &gen1.constraints,
        gen1.owner.clone(),
        &fixture1.analysis.snapshot.trait_surfaces,
    )
    .expect("valid evidence layout");

    let run2 = fixture2.callable("Probe", "run", DispatchSide::Class);
    let sig2 = fixture2.analysis.snapshot.callable_signatures.get(&run2.callable).expect("sig2");
    let gen2 = sig2.generics.as_ref().expect("gen2");
    let env2 = phalcom_semantic::generic_evidence::GenericEvidenceEnvironment::try_from_constraints(
        &gen2.constraints,
        gen2.owner.clone(),
        &fixture2.analysis.snapshot.trait_surfaces,
    )
    .expect("valid evidence layout");

    // Both layouts must have 3 slots: (T, Alpha.first), (T, Beta.second), (U, Alpha.first)
    assert_eq!(env1.layout().len(), 3);
    assert_eq!(env2.layout().len(), 3);

    // Slots must be identical
    assert_eq!(env1.layout().slots, env2.layout().slots);
    assert_eq!(
        callable_signature_product_fingerprint(sig1),
        callable_signature_product_fingerprint(sig2),
        "reordering direct conformance bounds must preserve the semantic signature fingerprint"
    );
}

#[test]
fn generic_conformance_bound_source_index_targets_trait_declaration() {
    let source = "trait Printable {}\nclass Probe { @class run<T>(_ value: T) -> T where T |= Printable { value } }\n";
    let fixture = Fixture::new(source);
    let start = source.match_indices("Printable").nth(1).expect("bound trait reference").0;
    let range = (start..start + "Printable".len()).into();
    let module_index = fixture.analysis.snapshot.source_index().module(&fixture.module).expect("source index");
    let occurrence = module_index
        .occurrences
        .all()
        .iter()
        .find(|occurrence| occurrence.range == range)
        .expect("bound trait occurrence");
    assert_eq!(
        module_index.occurrences.target_for(&occurrence.site),
        Some(&phalcom_semantic::SemanticTargetId::Declaration(fixture.decl("Printable")))
    );
}

#[test]
fn generic_method_assumed_trait_dispatch_positive_and_structural_negative() {
    let positive = Fixture::new(
        r#"
trait Printable {
  print -> String
}

class Renderer {
  @class
  render<T>(_ value: T) -> String
    where T |= Printable
  {
    value.print
  }
}
"#,
    );

    let render = positive.callable("Renderer", "render", DispatchSide::Class);
    let mut found_assumed = false;
    for expr in render.expressions.values() {
        if let Some(phalcom_semantic::trait_dispatch::TraitDispatchSite::Assumed(selection)) = &expr.trait_dispatch {
            assert!(matches!(&selection.requirement.selector.base, SelectorBase::Named(name) if name == "print"));
            found_assumed = true;
        }
    }
    assert!(found_assumed, "expected assumed trait dispatch for value.print");

    // Negative: without bound, value.print on unconstrained T cannot resolve to assumed trait dispatch
    let negative = Fixture::new_allowing_internal_incidents(
        r#"
class BadRenderer {
  @class
  render<T>(_ value: T) -> String {
    value.print
  }
}
"#,
    );

    let render_bad = negative.callable("BadRenderer", "render", DispatchSide::Class);
    for expr in render_bad.expressions.values() {
        assert!(expr.trait_dispatch.is_none(), "unconstrained T must not have trait dispatch");
    }
}

#[test]
fn generic_method_assumed_trait_dispatch_ambiguity() {
    let fixture = Fixture::new_allowing_internal_incidents(
        r#"
trait Left {
  name -> String
}

trait Right {
  name -> String
}

class AmbiguousProbe {
  @class
  run<T>(_ value: T) -> String
    where T |= Left + Right
  {
    value.name
  }
}
"#,
    );

    let ambiguous = fixture.callable("AmbiguousProbe", "run", DispatchSide::Class);
    // Two conflicting assumed traits expose the same selector: no single assumed dispatch site can be formed
    for expr in ambiguous.expressions.values() {
        assert!(
            expr.trait_dispatch.is_none(),
            "ambiguous assumed traits must not resolve to a single dispatch site"
        );
    }
}

#[test]
fn conforming_concrete_argument_discharges_direct_bound() {
    let fixture = Fixture::new(
        r#"
trait Printable {
  print -> String
}

class User {
  @constructor
  new() {}
}

impl Printable for User {
  print -> String { "user" }
}

class Discharger {
  @class
  display<T>(_ item: T) -> String
    where T |= Printable
  {
    item.print
  }

  @class
  run -> String {
    let u = User.new()
    Discharger.display(u)
  }
}
"#,
    );

    let run = fixture.callable("Discharger", "run", DispatchSide::Class);
    let mut found_call = false;
    for expr in run.expressions.values() {
        if let Some(spec) = &expr.call_specialization {
            assert_eq!(spec.conformances.len(), 1);
            match &spec.conformances[0].source {
                phalcom_semantic::checker::analysis::GenericCallConformanceSource::Exact { evidence } => {
                    let user_ty = fixture.ty("User");
                    assert_eq!(evidence.exact_target, user_ty);
                    found_call = true;
                }
                other => panic!("expected exact evidence for User |= Printable, got {:?}", other),
            }
        }
    }
    assert!(found_call, "expected CallSpecialization on Discharger.display(u)");
}

#[test]
fn conditional_selected_call_capability_publishes_impl_binding_and_proof() {
    let fixture = Fixture::new(
        r#"
trait Printable {
  print -> String
}

class User {
  @constructor
  new() {}
}

impl Printable for User {
  print -> String { "user" }
}

class Box<T> {}

impl<T> Box<T>
  where T |= Printable
{
  render(_ suffix: String) -> String {
    suffix
  }
}

class Probe {
  @class
  direct(_ box: Box<User>) -> String {
    box.render("!")
  }

  @class
  capture(_ box: Box<User>) -> String {
    let render = &box.render(_)
    render("?")
  }

  @class
  forwarded<T>(_ box: Box<T>) -> String
    where T |= Printable
  {
    box.render("!")
  }
}
"#,
    );
    assert!(
        !fixture.analysis.snapshot.has_errors(),
        "semantic diagnostics: {:?}",
        fixture.analysis.snapshot.diagnostics
    );

    fn assert_capability(
        capability: &phalcom_semantic::dispatch::ConditionalSelectedCallCapability,
        expected_callable: &phalcom_semantic::identity::CallableId,
        expected_impl: &phalcom_semantic::identity::ImplId,
        impl_parameters: &[phalcom_semantic::types::id::TypeParameterId],
        user_ty: phalcom_semantic::types::id::TypeId,
    ) {
        assert_eq!(&capability.selection.callable, expected_callable);
        assert_eq!(&capability.selection.impl_id, expected_impl);
        assert_eq!(capability.applicability.impl_id, capability.selection.impl_id);
        assert!(!impl_parameters.is_empty(), "fixture must expose impl-owned generic parameters");
        for parameter in impl_parameters {
            assert_eq!(
                capability.applicability.bindings.get(parameter),
                Some(&user_ty),
                "selected impl applicability must retain its exact impl-owned `T = User` binding"
            );
        }
        let provenance = capability
            .applicability
            .conformance_provenance
            .as_ref()
            .expect("selected applicability must publish proof provenance");
        assert_eq!(provenance.len(), 1, "every selected impl conformance constraint must be represented");
        match &provenance[0].source {
            phalcom_semantic::checker::analysis::GenericCallConformanceSource::Exact { evidence } => {
                assert_eq!(evidence.exact_target, user_ty);
            }
            other => panic!("expected exact impl-bound evidence for User, got {other:?}"),
        }
    }

    let user_ty = fixture.ty("User");
    let box_target = phalcom_semantic::impls::InherentImplTarget::Declaration(fixture.decl("Box"));
    let impl_member = fixture
        .analysis
        .snapshot
        .dispatch
        .get_conditional_members(&box_target)
        .expect("conditional Box impl")
        .members
        .first()
        .expect("conditional Box member");
    let impl_signature = impl_member.domain.generic_signature.as_ref().expect("impl generic signature");
    assert_eq!(impl_signature.owner, TypeParameterOwner::Impl(impl_member.impl_id.clone()));
    let expected_impl = impl_member.impl_id.clone();
    let impl_parameters = impl_signature.parameters.to_vec();
    let direct = fixture.callable("Probe", "direct", DispatchSide::Class);
    let selected_expression = direct
        .expressions
        .values()
        .find(|expression| expression.conditional_dispatch.is_some())
        .expect("direct conditional send must publish its selected dispatch");
    let direct_capability = selected_expression
        .conditional_capability
        .as_ref()
        .unwrap_or_else(|| panic!("selected expression lost its capability: {selected_expression:?}"));
    let conditional_callable = direct_capability.selection.callable.clone();
    assert_capability(direct_capability, &conditional_callable, &expected_impl, &impl_parameters, user_ty);

    let capture = fixture.callable("Probe", "capture", DispatchSide::Class);
    let captured_capability = capture
        .callable_reference_resolutions
        .values()
        .find_map(|resolution| match &resolution.kind {
            phalcom_semantic::checker::associated::CallableReferenceResolutionKind::BoundFamily { members, .. } => {
                members.iter().find_map(|member| member.conditional_capability.as_ref())
            }
            _ => None,
        })
        .expect("conditional bound family must carry the canonical selected-call capability");
    assert_capability(captured_capability, &conditional_callable, &expected_impl, &impl_parameters, user_ty);
    assert_eq!(
        direct_capability, captured_capability,
        "direct and captured routes must publish the same semantic capability"
    );

    let forwarded = fixture.callable("Probe", "forwarded", DispatchSide::Class);
    let forwarded_capability = forwarded
        .expressions
        .values()
        .find_map(|expression| expression.conditional_capability.as_ref())
        .expect("generic conditional call must publish its selected-call capability");
    let forwarded_provenance = forwarded_capability
        .applicability
        .conformance_provenance
        .as_ref()
        .expect("forwarded selected call must publish provenance");
    assert_eq!(forwarded_provenance.len(), 1);
    assert!(matches!(
        &forwarded_provenance[0].source,
        phalcom_semantic::checker::analysis::GenericCallConformanceSource::Forwarded { caller_conformance }
            if caller_conformance.trait_ref.declaration == fixture.decl("Printable")
    ));
}

#[test]
fn conditional_selected_call_capability_publishes_derived_impl_proof() {
    let fixture = Fixture::new(
        r#"
trait BaseText {
  base_text -> String
}

trait DerivedText {
  derived_text -> String
}

impl<T> DerivedText for T
  where T |= BaseText
{
  derived_text -> String { self.base_text }
}

class User {
  @constructor
  new() {}
}

impl BaseText for User {
  base_text -> String { "user" }
}

class Box<T> {}

impl<T> Box<T>
  where T |= DerivedText
{
  render(_ suffix: String) -> String { suffix }
}

class Probe {
  @class
  render(_ box: Box<User>) -> String { box.render("!") }
}
"#,
    );
    assert!(
        !fixture.analysis.snapshot.has_errors(),
        "semantic diagnostics: {:?}",
        fixture.analysis.snapshot.diagnostics
    );
    let render = fixture.callable("Probe", "render", DispatchSide::Class);
    let capability = render
        .expressions
        .values()
        .find_map(|expression| expression.conditional_capability.as_ref())
        .expect("derived conditional invocation must publish a selected-call capability");
    let provenance = capability
        .applicability
        .conformance_provenance
        .as_ref()
        .expect("selected applicability must retain derived provenance");
    assert_eq!(provenance.len(), 1);
    let phalcom_semantic::checker::analysis::GenericCallConformanceSource::Derived { evidence } = &provenance[0].source else {
        panic!("expected derived conditional proof provenance, got {:?}", provenance[0].source);
    };
    assert!(!evidence.nested.is_empty(), "derived proof should retain its nested BaseText evidence");
}

#[test]
fn conditional_selected_call_capability_keeps_same_trait_subjects_distinct() {
    let fixture = Fixture::new(
        r#"
trait Printable {
  print -> String
}

class User {
  @constructor
  new() {}
}

class Guest {
  @constructor
  new() {}
}

impl Printable for User { print -> String { "user" } }
impl Printable for Guest { print -> String { "guest" } }

class Duo<A, B> {}

impl<A, B> Duo<A, B>
  where A |= Printable, B |= Printable
{
  render(_ suffix: String) -> String { suffix }
}

class Probe {
  @class
  render(_ duo: Duo<User, Guest>) -> String { duo.render("!") }
}
"#,
    );
    assert!(
        !fixture.analysis.snapshot.has_errors(),
        "semantic diagnostics: {:?}",
        fixture.analysis.snapshot.diagnostics
    );
    let render = fixture.callable("Probe", "render", DispatchSide::Class);
    let capability = render
        .expressions
        .values()
        .find_map(|expression| expression.conditional_capability.as_ref())
        .expect("two-obligation conditional call must publish a capability");
    let provenance = capability
        .applicability
        .conformance_provenance
        .as_ref()
        .expect("selected applicability must publish proof sources");
    assert_eq!(provenance.len(), 2, "both impl constraints need provenance");
    let subjects = provenance
        .iter()
        .map(|binding| match binding.callee_conformance.subject {
            TypeTerm::Canonical(subject) => subject,
            ref other => panic!("expected canonical selected impl subject, got {other:?}"),
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(subjects, [fixture.ty("User"), fixture.ty("Guest")].into_iter().collect());
    assert!(
        provenance
            .iter()
            .all(|binding| matches!(binding.source, phalcom_semantic::checker::analysis::GenericCallConformanceSource::Exact { .. }))
    );
}

#[test]
fn generic_helper_binds_fresh_callee_parameter_to_rigid_caller_parameter() {
    let fixture = Fixture::new(
        r#"
class Forwarder {
  @class inner<T>(_ value: T) -> T { value }
  @class outer<U>(_ value: U) -> U {
    let forwarded = Forwarder.inner(value)
    forwarded
  }
}
"#,
    );
    fixture.assert_no_error_diagnostics();
    let outer = fixture.callable("Forwarder", "outer", DispatchSide::Class);
    let inner = fixture.callable("Forwarder", "inner", DispatchSide::Class);
    let signatures = &fixture.analysis.snapshot.callable_signatures;
    let caller = signatures
        .get(&outer.callable)
        .expect("caller signature")
        .generics
        .as_ref()
        .expect("caller generics")
        .parameters[0];
    let callee = signatures
        .get(&inner.callable)
        .expect("callee signature")
        .generics
        .as_ref()
        .expect("callee generics")
        .parameters[0];
    assert_ne!(caller, callee);
    let call = fixture.expression(outer, "Forwarder.inner(value)");
    let specialization = call.call_specialization.as_ref().expect("generic helper specialization");
    assert_eq!(specialization.bindings.len(), 1);
    let (parameter, argument) = specialization.bindings[0];
    assert_eq!(parameter, callee);
    let phalcom_semantic::types::store::StaticArgument::Term(ty) = argument else {
        panic!("forwarded type parameter must remain a proper type term")
    };
    assert!(matches!(fixture.analysis.snapshot.store.get(ty), TypeData::Parameter(parameter) if *parameter == caller));
    assert_eq!(call.knowledge.ty(), Some(ty));
}

#[test]
fn associated_equality_caller_forwards_evidence_to_generic_helper() {
    for bounds in ["I |= Iterable, <I as Iterable>::Item == Float", "I |= Iterable<Item = Float>"] {
        let fixture = Fixture::new(&format!(
            r#"
trait Iterable {{ type Item }}
class FloatReduction {{
  @class snapshot<J>(_ input: J) -> Float
    where J |= Iterable, <J as Iterable>::Item == Float {{ 0.0 }}
  @class reduce<I>(_ input: I) -> Float where {bounds} {{
    let values = FloatReduction.snapshot(input)
    values
  }}
}}
"#
        ));
        fixture.assert_no_error_diagnostics();
        let reduce = fixture.callable("FloatReduction", "reduce", DispatchSide::Class);
        let call = fixture.expression(reduce, "FloatReduction.snapshot(input)");
        let specialization = call.call_specialization.as_ref().expect("helper specialization");
        let caller_parameter = fixture
            .analysis
            .snapshot
            .callable_signatures
            .get(&reduce.callable)
            .expect("caller signature")
            .generics
            .as_ref()
            .expect("caller generics")
            .parameters[0];
        let (_, argument) = specialization.bindings[0];
        let phalcom_semantic::types::store::StaticArgument::Term(bound_type) = argument else {
            panic!("forwarded associated-equality subject must remain a proper type term")
        };
        assert!(matches!(fixture.analysis.snapshot.store.get(bound_type), TypeData::Parameter(parameter) if *parameter == caller_parameter));
        assert_eq!(specialization.conformances.len(), 1);
        assert!(matches!(
            specialization.conformances[0].source,
            phalcom_semantic::checker::analysis::GenericCallConformanceSource::Forwarded { .. }
        ));
        assert_eq!(call.knowledge.ty(), Some(fixture.ty("Float")));
    }
}

#[test]
fn associated_equality_rejects_insufficient_or_contradictory_caller_evidence() {
    use phalcom_semantic::diagnostic::DiagnosticCode;
    let mut failures = Vec::new();
    for bounds in ["I |= Iterable", "I |= Iterable<Item = Int>"] {
        let fixture = Fixture::new(&format!(
            r#"
trait Iterable {{ type Item }}
class FloatReduction {{
  @class snapshot<J>(_ input: J) -> Float
    where J |= Iterable<Item = Float> {{ 0.0 }}
  @class reduce<I>(_ input: I) -> Float where {bounds} {{
    let values = FloatReduction.snapshot(input)
    0.0
  }}
}}
"#
        ));
        if fixture.diagnostics(DiagnosticCode::GenericConstraintUnsatisfied).len() != 1
            || !fixture.diagnostics(DiagnosticCode::GenericInferenceUnderconstrained).is_empty()
        {
            failures.push((
                bounds,
                fixture
                    .analysis
                    .snapshot
                    .all_diagnostics()
                    .map(|diagnostic| diagnostic.code)
                    .collect::<Vec<_>>(),
            ));
        }
    }
    assert!(failures.is_empty(), "incorrect associated-equality rejection diagnostics: {failures:#?}");
}

#[test]
fn bounded_generic_caller_forwards_evidence_to_bounded_callee() {
    let fixture = Fixture::new(
        r#"
trait Printable {
  print -> String
}

class Forwarder {
  @class
  inner<T>(_ value: T) -> String
    where T |= Printable
  {
    value.print
  }

  @class
  outer<T>(_ value: T) -> String
    where T |= Printable
  {
    Forwarder.inner(value)
  }
}
"#,
    );

    let outer = fixture.callable("Forwarder", "outer", DispatchSide::Class);
    let mut found_forwarded = false;
    for expr in outer.expressions.values() {
        if let Some(spec) = &expr.call_specialization {
            assert_eq!(spec.conformances.len(), 1);
            match &spec.conformances[0].source {
                phalcom_semantic::checker::analysis::GenericCallConformanceSource::Forwarded { caller_conformance } => {
                    let printable_decl = fixture.decl("Printable");
                    assert_eq!(caller_conformance.trait_ref.declaration, printable_decl);
                    found_forwarded = true;
                }
                other => panic!("expected forwarded evidence for T |= Printable, got {:?}", other),
            }
        }
    }
    assert!(found_forwarded, "expected forwarded evidence on Forwarder.inner(value)");
}

#[test]
fn dynamic_does_not_discharge_static_conformance_bound() {
    let fixture = Fixture::new_allowing_internal_incidents(
        r#"
trait Printable {
  print -> String
}

class DynamicCaller {
  @class
  consume<T>(_ value: T) -> String
    where T |= Printable
  {
    value.print
  }

  @class
  run(_ dyn_val: Dynamic) -> String {
    DynamicCaller.consume(dyn_val)
  }
}
"#,
    );

    fixture.assert_diagnostic(phalcom_semantic::diagnostic::DiagnosticCode::GenericConstraintUnsatisfied, 1);
    fixture.assert_no_internal_incidents();
    let run = fixture.callable("DynamicCaller", "run", DispatchSide::Class);
    let call = fixture.expression(run, "DynamicCaller.consume(dyn_val)");
    assert!(
        call.call_specialization.is_none(),
        "a Dynamic-dependent constraint must not publish a call specialization or runtime witness recipe"
    );
}

#[test]
fn generic_evidence_layout_overflow_is_a_checked_failure() {
    use phalcom_semantic::generic_evidence::{AssumedConformanceEvidence, GenericConformanceKey, GenericEvidenceLayout, GenericEvidenceLayoutError};
    use std::collections::BTreeMap;

    let fixture = Fixture::new(
        r#"
trait Alpha {
  alpha_req -> String
}
"#,
    );

    let alpha_decl = fixture.decl("Alpha");
    let mut conformances = BTreeMap::new();
    let owner = TypeParameterOwner::Declaration(fixture.decl("Alpha"));

    // Create 65537 conformances with this trait
    for i in 0..=65536 {
        let key = GenericConformanceKey::new(
            TypeTerm::Canonical(phalcom_semantic::types::id::TypeId(i as u32)),
            phalcom_semantic::traits::TraitRef::new(alpha_decl.clone(), vec![]),
        );
        conformances.insert(key.clone(), AssumedConformanceEvidence { key, owner: owner.clone() });
    }

    let constraints = conformances
        .keys()
        .map(|key| GenericConstraint::Conforms {
            subject: key.subject.clone(),
            trait_ref: key.trait_ref.clone(),
        })
        .collect::<Vec<_>>();
    let surfaces = &fixture.analysis.snapshot.trait_surfaces;
    use phalcom_semantic::generic_evidence::GenericEvidenceEnvironment;
    assert!(matches!(
        GenericEvidenceEnvironment::try_from_constraints(&constraints, owner.clone(), surfaces),
        Err(GenericEvidenceLayoutError::SlotCapacityExceeded(65536))
    ));
    let left = GenericEvidenceEnvironment::try_from_constraints(&constraints[..32768], owner.clone(), surfaces).unwrap();
    let right = GenericEvidenceEnvironment::try_from_constraints(&constraints[32768..], owner, surfaces).unwrap();
    assert!(matches!(
        left.try_compose(&right, surfaces),
        Err(GenericEvidenceLayoutError::SlotCapacityExceeded(65536))
    ));

    let layout_res = GenericEvidenceLayout::build(&conformances, &fixture.analysis.snapshot.trait_surfaces);
    assert!(
        matches!(layout_res, Err(GenericEvidenceLayoutError::SlotCapacityExceeded(_))),
        "expected SlotCapacityExceeded, got {:?}",
        layout_res
    );
}

#[test]
fn concrete_missing_conformance_is_rejected_at_call_site() {
    let fixture = Fixture::new_allowing_internal_incidents(
        r#"
trait Printable {
  print -> String
}

class NonPrintable {}

class Discharger {
  @class
  display<T>(_ item: T) -> String
    where T |= Printable
  {
    item.print
  }
}

class Probe {
  @class
  run() -> String {
    let item = NonPrintable.new()
    Discharger.display(item)
  }
}
"#,
    );

    fixture.assert_diagnostic(phalcom_semantic::diagnostic::DiagnosticCode::GenericConstraintUnsatisfied, 1);
}

#[test]
fn failed_bounded_call_does_not_contaminate_later_analysis() {
    let fixture = Fixture::new_allowing_internal_incidents(
        r#"
trait Printable {
  print -> String
}

class NonPrintable {}
class ValidPrintable {}

impl Printable for ValidPrintable {
  print -> String { "valid" }
}

class Discharger {
  @class
  display<T>(_ item: T) -> String
    where T |= Printable
  {
    item.print
  }
}

class Probe {
  @class
  run() -> String {
    let bad = NonPrintable.new()
    let _ = Discharger.display(bad)
    let good = ValidPrintable.new()
    Discharger.display(good)
  }
}
"#,
    );

    // Only the bad call diagnoses unsatisfied generic constraint; good call proceeds cleanly
    fixture.assert_diagnostic(phalcom_semantic::diagnostic::DiagnosticCode::GenericConstraintUnsatisfied, 1);
}

#[test]
fn receiver_owner_evidence_is_published_without_owner_inference() {
    let fixture = Fixture::new(
        r#"
trait Printable { print -> String }
class User { @constructor new() {} }
impl Printable for User { print -> String { "user" } }
class Container<T> where T |= Printable {
  _val: T
  @constructor new(_ val: T) { _val = val }
  show() -> String { _val.print }
  combine<U>(_ other: U) -> String where U |= Printable { _val.print + other.print }
}
class Runner {
  @class run() -> String {
    const c = Container.new(User.new())
    let first = c.show()
    c.combine(User.new())
  }
  @class forward<U>(_ c: Container<U>) -> String where U |= Printable { c.show() }
}
"#,
    );
    fixture.assert_no_error_diagnostics();
    let show = fixture.callable("Container", "show", DispatchSide::Instance);
    assert!(
        show.expressions
            .values()
            .any(|expression| matches!(expression.trait_dispatch, Some(phalcom_semantic::trait_dispatch::TraitDispatchSite::Assumed(_)))),
        "owner body retains assumed dispatch"
    );
    let run = fixture.callable("Runner", "run", DispatchSide::Class);
    for (name, expected) in [("show", 1), ("combine", 2)] {
        let specialization = run
            .expressions
            .values()
            .filter_map(|expression| expression.call_specialization.as_ref())
            .find(|specialization| matches!(&specialization.callable.selector.base, SelectorBase::Named(selector) if selector == name))
            .expect("receiver-specialized call");
        assert_eq!(specialization.conformances.len(), expected);
        assert_eq!(specialization.bindings.len(), expected);
        assert!(
            specialization
                .conformances
                .iter()
                .all(|binding| matches!(binding.source, phalcom_semantic::checker::analysis::GenericCallConformanceSource::Exact { .. }))
        );
    }
    let forward = fixture.callable("Runner", "forward", DispatchSide::Class);
    let specialization = forward
        .expressions
        .values()
        .filter_map(|expression| expression.call_specialization.as_ref())
        .next()
        .expect("forwarded owner call");
    assert_eq!(specialization.conformances.len(), 1);
    assert!(matches!(
        specialization.conformances[0].source,
        phalcom_semantic::checker::analysis::GenericCallConformanceSource::Forwarded { .. }
    ));
}

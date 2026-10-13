//! Tests for generic associated constraints, associated specialization elaboration, kind facts, and equality orientation.

use crate::semantic::support::Fixture;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::parameter::{GenericConstraint, TypeTerm};
use phalcom_semantic::types::store::TypeData;

#[test]
fn associated_specialization_elaborates_to_conforms_and_equivalent() {
    let fixture = Fixture::new(
        r#"
trait Iterator {
  type Item
}

class Tools {
  @class
  first<I, T>(_ iterator: I) -> Option<T>
    where I |= Iterator<Item = T>
  {
    nil
  }
}
"#,
    );

    let first = fixture.callable("Tools", "first", DispatchSide::Class);
    let first_sig = fixture.analysis.snapshot.callable_signatures.get(&first.callable).expect("first signature");
    let generics = first_sig.generics.as_ref().expect("generics");

    assert_eq!(generics.parameters.len(), 2);
    let i_param = generics.parameters[0];
    let t_param = generics.parameters[1];
    let iter_trait = fixture.decl("Iterator");

    // We expect: Conforms(I, Iterator) and Equivalent(<I as Iterator>::Item, T)
    let mut has_conforms = false;
    let mut has_equivalent = false;

    for constraint in generics.constraints.iter() {
        match constraint {
            GenericConstraint::Conforms { subject, trait_ref } => {
                assert!(
                    matches!(subject, TypeTerm::Canonical(ty) if matches!(fixture.analysis.snapshot.store.get(*ty), TypeData::Parameter(p) if *p == i_param))
                );
                assert_eq!(trait_ref.declaration, iter_trait);
                has_conforms = true;
            }
            GenericConstraint::Equivalent { left, right } => {
                // One side should be projection <I as Iterator>::Item, the other side should be T
                let check_term = |term: &TypeTerm| -> (&str, bool) {
                    match term {
                        TypeTerm::Canonical(ty) => match fixture.analysis.snapshot.store.get(*ty) {
                            TypeData::AssociatedProjection(proj) => {
                                assert_eq!(proj.trait_ref.declaration, iter_trait);
                                ("proj", true)
                            }
                            TypeData::Parameter(p) if *p == t_param => ("t", true),
                            other => panic!("unexpected term data: {:?}", other),
                        },
                        _ => panic!("unexpected non-canonical term"),
                    }
                };
                let (l_kind, _) = check_term(left);
                let (r_kind, _) = check_term(right);
                assert_ne!(l_kind, r_kind);
                assert!(left <= right, "Equality must be canonically oriented (left <= right)");
                has_equivalent = true;
            }
            other => panic!("unexpected constraint: {:?}", other),
        }
    }

    assert!(has_conforms, "Expected Conforms(I, Iterator)");
    assert!(has_equivalent, "Expected Equivalent(<I as Iterator>::Item, T)");
}

#[test]
fn where_clause_kind_fact_unifies_with_generic_parameter() {
    let fixture = Fixture::new(
        r#"
class Container<F: Type -> Type>
{
}
"#,
    );

    let container_decl = fixture.decl("Container");
    let container_info = fixture.analysis.snapshot.declarations.get(&container_decl).expect("container declaration");
    let generics = container_info.generic_signature.as_ref().expect("generics");
    assert_eq!(generics.parameters.len(), 1);
    let f_param_id = generics.parameters[0];
    let f_param = fixture.analysis.snapshot.store.type_parameter(f_param_id);
    assert_eq!(fixture.analysis.snapshot.store.format_kind(f_param.kind), "(Type) -> Type");
}

#[test]
fn canonical_equality_orientation_is_symmetric() {
    let fixture_1 = Fixture::new(
        r#"
class Probe {
  @class
  run<A, B>(_ a: A, _ b: B) -> A
    where A == B
  {
    a
  }
}
"#,
    );

    let fixture_2 = Fixture::new(
        r#"
class Probe {
  @class
  run<A, B>(_ a: A, _ b: B) -> A
    where B == A
  {
    a
  }
}
"#,
    );

    let run_1 = fixture_1.callable("Probe", "run", DispatchSide::Class);
    let sig_1 = fixture_1.analysis.snapshot.callable_signatures.get(&run_1.callable).expect("sig 1");
    let generics_1 = sig_1.generics.as_ref().expect("generics 1");

    let run_2 = fixture_2.callable("Probe", "run", DispatchSide::Class);
    let sig_2 = fixture_2.analysis.snapshot.callable_signatures.get(&run_2.callable).expect("sig 2");
    let generics_2 = sig_2.generics.as_ref().expect("generics 2");

    assert_eq!(generics_1.constraints.len(), 1);
    assert_eq!(generics_2.constraints.len(), 1);

    // Both should have identical canonical constraint representations
    match (&generics_1.constraints[0], &generics_2.constraints[0]) {
        (GenericConstraint::Equivalent { left: l1, right: r1 }, GenericConstraint::Equivalent { left: l2, right: r2 }) => {
            assert!(l1 <= r1, "Constraint 1 must be canonically oriented (left <= right)");
            assert!(l2 <= r2, "Constraint 2 must be canonically oriented (left <= right)");
        }
        _ => panic!("Expected Equivalent constraints"),
    }
}

#[test]
fn generic_evidence_environment_tracks_equivalences_and_refinements() {
    use phalcom_semantic::generic_evidence::{GenericEquivalenceProof, GenericEvidenceEnvironment};
    use phalcom_semantic::traits::{AssociatedTypeRequirementId, TraitRef};
    use phalcom_semantic::types::outcome::{CancellationToken, QueryBudget};
    use phalcom_semantic::types::projection::{ProjectionNormalizationContext, ProjectionNormalizationMode, ProjectionNormalizationResult, normalize_type};

    let fixture = Fixture::new(
        r#"
trait Iterator {
  type Item
}

class Tools {
  @class
  first<I, T>(_ iterator: I) -> Option<T>
    where I |= Iterator<Item = T>
  {
    nil
  }
}
"#,
    );

    let first = fixture.callable("Tools", "first", DispatchSide::Class);
    let first_sig = fixture.analysis.snapshot.callable_signatures.get(&first.callable).expect("first signature");
    let generics = first_sig.generics.as_ref().expect("generics");

    let env = GenericEvidenceEnvironment::try_from_constraints(&generics.constraints, generics.owner.clone(), &fixture.analysis.snapshot.trait_surfaces)
        .expect("valid evidence env");

    let i_param = generics.parameters[0];
    let t_param = generics.parameters[1];
    let mut store = (*fixture.analysis.snapshot.store).clone();
    let i_ty = store.parameter_form(i_param);
    let t_ty = store.parameter_form(t_param);
    let iter_trait = fixture.decl("Iterator");
    let trait_ref = TraitRef::new(iter_trait.clone(), vec![].into_boxed_slice());
    let req_id = AssociatedTypeRequirementId::new(iter_trait, 0);

    let proj_ty = store.associated_projection(i_ty, trait_ref.clone(), req_id);

    let proj_term = TypeTerm::Canonical(proj_ty);
    let t_term = TypeTerm::Canonical(t_ty);

    assert_eq!(env.are_equivalent(&proj_term, &t_term), GenericEquivalenceProof::Proven);
    assert_eq!(env.unique_refinement(&proj_term), Some(&t_term));

    // Test normalize_type in Generic mode
    let mut budget = QueryBudget::new(100);
    let cancel = CancellationToken::new();
    let mut context = ProjectionNormalizationContext::new(
        ProjectionNormalizationMode::Generic {
            evidence: &env,
            conformance_index: Some(&fixture.analysis.snapshot.conformance_index),
            witness_plans: Some(&fixture.analysis.snapshot.conformance_witness_plans),
            trait_surfaces: &fixture.analysis.snapshot.trait_surfaces,
            derived_rules: None,
            declarations: &fixture.analysis.snapshot.declarations,
            hierarchy: &*fixture.analysis.snapshot.hierarchy,
        },
        &mut budget,
        &cancel,
    );

    let norm_res = normalize_type(&mut store, proj_ty, &mut context);
    assert_eq!(norm_res, ProjectionNormalizationResult::Normalized(t_ty));
}

#[test]
fn contextual_projection_in_method_signature_forms_with_generic_evidence() {
    let fixture = Fixture::new(
        r#"
trait Iterator {
  type Item
}

class Tools {
  @class
  copy<I>(_ iterator: I, _ value: I::Item) -> I::Item
    where I |= Iterator
  {
    value
  }
}
"#,
    );

    let copy = fixture.callable("Tools", "copy", DispatchSide::Class);
    let copy_sig = fixture.analysis.snapshot.callable_signatures.get(&copy.callable).expect("copy signature");
    assert_eq!(copy_sig.parameters.len(), 2);

    let val_ty = copy_sig.parameters[1].declared_type.to_knowledge().ty().expect("value type");
    let ret_ty = copy_sig.declared_return.to_knowledge().ty().expect("return type");

    assert_eq!(val_ty, ret_ty);
    match fixture.analysis.snapshot.store.get(val_ty) {
        TypeData::AssociatedProjection(proj) => {
            assert_eq!(proj.trait_ref.declaration, fixture.decl("Iterator"));
        }
        other => panic!("expected AssociatedProjection, got {:?}", other),
    }
}

#[test]
fn owner_and_method_evidence_compose_for_projection() {
    let fixture = Fixture::new(
        r#"
trait Iterator {
  type Item
}

class Pipeline<I>
  where I |= Iterator
{
  @instance
  transform<U>(_ value: I::Item, _ mapper: (I::Item) -> U) -> U {
    mapper(value)
  }
}
"#,
    );

    let transform = fixture.callable("Pipeline", "transform", DispatchSide::Instance);
    let transform_sig = fixture
        .analysis
        .snapshot
        .callable_signatures
        .get(&transform.callable)
        .expect("transform signature");
    assert_eq!(transform_sig.parameters.len(), 2);

    let val_ty = transform_sig.parameters[0].declared_type.to_knowledge().ty().expect("value type");
    match fixture.analysis.snapshot.store.get(val_ty) {
        TypeData::AssociatedProjection(proj) => {
            assert_eq!(proj.trait_ref.declaration, fixture.decl("Iterator"));
        }
        other => panic!("expected AssociatedProjection, got {:?}", other),
    }
}

#[test]
fn infer_generic_method_call_with_associated_specialization() {
    let fixture = Fixture::new(
        r#"
trait Iterator {
  type Item
}

class User {}

class UserIterator {}

impl Iterator for UserIterator {
  type Item = User
}

class Tools {
  @class
  first<I, T>(_ iter: I) -> Option<T>
    where I |= Iterator<Item = T>
  {
    nil
  }

  @class
  testCall() {
    let it = UserIterator.new()
    let res = Tools.first(it)
  }
}
"#,
    );

    let diags = fixture.analysis.snapshot.all_diagnostics().collect::<Vec<_>>();
    assert!(diags.is_empty(), "expected 0 diagnostics, got: {:?}", diags);
}

#[test]
fn projection_in_parameter_participates_in_inference() {
    let fixture = Fixture::new(
        r#"
trait Iterator {
  type Item
}

class User {}

class UserIterator {}

impl Iterator for UserIterator {
  type Item = User
}

class Tools {
  @class
  consume<I>(_ iter: I, _ item: I::Item) -> I::Item
    where I |= Iterator
  {
    item
  }

  @class
  testCall() {
    let it = UserIterator.new()
    let u = User.new()
    let res = Tools.consume(it, u)
  }
}
"#,
    );

    let diags = fixture.analysis.snapshot.all_diagnostics().collect::<Vec<_>>();
    assert!(diags.is_empty(), "expected 0 diagnostics, got: {:?}", diags);
}

#[test]
fn unconstrained_projection_subject_does_not_reverse_search() {
    let fixture = Fixture::new(
        r#"
trait Iterator {
  type Item
}

class User {}

class UserIterator {}

impl Iterator for UserIterator {
  type Item = User
}

class Tools {
  @class
  produce<I>(_ dummy: Int) -> I::Item
    where I |= Iterator
  {
    User.new()
  }

  @class
  testCall() {
    let res = Tools.produce(42)
  }
}
"#,
    );

    let diags = fixture.analysis.snapshot.all_diagnostics().collect::<Vec<_>>();
    assert!(!diags.is_empty(), "expected underconstrained diagnostic for unconstrained I");
}

#[test]
fn stable_metadata_roundtrips_associated_projections() {
    use phalcom_semantic::metadata::MetadataExporter;
    use phalcom_type_meta::header::MetadataProfile;
    use phalcom_type_meta::type_node::TypeNode;
    use phalcom_type_meta::validate::{ValidationLimits, validate_metadata_bundle};

    let fixture = Fixture::new(
        r#"
trait Iterator {
  type Item
}

class Tools {
  @class
  first<I, T>(_ iterator: I) -> Option<T>
    where I |= Iterator<Item = T>
  {
    nil
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
    validate_metadata_bundle(&bundle, &ValidationLimits::default()).expect("valid metadata bundle");

    let has_associated_projection = bundle.types.iter().any(|entry| matches!(entry.form, TypeNode::AssociatedProjection { .. }));
    assert!(has_associated_projection, "bundle must contain an AssociatedProjection TypeNode");
}

#[test]
fn projection_equality_is_non_injective_for_type_member_projections() {
    let mut store = phalcom_semantic::types::store::TypeStore::new();
    let mod_id = phalcom_modules::ModuleId::universe_root();
    let decl_p = phalcom_modules::DeclarationId::new(mod_id.clone(), "P".into());
    let decl_q = phalcom_modules::DeclarationId::new(mod_id, "Q".into());
    let p = store.intern_type_parameter(
        phalcom_semantic::types::parameter::TypeParameterData::new(
            phalcom_semantic::types::parameter::TypeParameterOwner::Declaration(decl_p),
            0,
            "P",
            phalcom_semantic::KindId::COMPLETE_TYPE_PACK,
        )
        .with_mode(phalcom_semantic::types::parameter::GenericParameterMode::CompletePack),
    );
    let q = store.intern_type_parameter(
        phalcom_semantic::types::parameter::TypeParameterData::new(
            phalcom_semantic::types::parameter::TypeParameterOwner::Declaration(decl_q),
            0,
            "Q",
            phalcom_semantic::KindId::COMPLETE_TYPE_PACK,
        )
        .with_mode(phalcom_semantic::types::parameter::GenericParameterMode::CompletePack),
    );
    let p_ty = store.parameter_form(p);
    let q_ty = store.parameter_form(q);

    let p_pos = store.project_positionals(p_ty).unwrap();
    let q_pos = store.project_positionals(q_ty).unwrap();

    // In a solver or unification context, P.Positionals == Q.Positionals must NOT unify P and Q.
    assert_ne!(p_ty, q_ty);
    assert_ne!(p_pos, q_pos);
    assert_eq!(store.canonical_complete_pack_subject(p_pos, q_pos), None);
}

#[test]
fn canonical_complete_pack_subject_roundtrips_only_congruent_lanes() {
    let mut store = phalcom_semantic::types::store::TypeStore::new();
    let mod_id = phalcom_modules::ModuleId::universe_root();
    let decl_p = phalcom_modules::DeclarationId::new(mod_id.clone(), "P".into());
    let decl_q = phalcom_modules::DeclarationId::new(mod_id, "Q".into());
    let p = store.intern_type_parameter(
        phalcom_semantic::types::parameter::TypeParameterData::new(
            phalcom_semantic::types::parameter::TypeParameterOwner::Declaration(decl_p),
            0,
            "P",
            phalcom_semantic::KindId::COMPLETE_TYPE_PACK,
        )
        .with_mode(phalcom_semantic::types::parameter::GenericParameterMode::CompletePack),
    );
    let q = store.intern_type_parameter(
        phalcom_semantic::types::parameter::TypeParameterData::new(
            phalcom_semantic::types::parameter::TypeParameterOwner::Declaration(decl_q),
            0,
            "Q",
            phalcom_semantic::KindId::COMPLETE_TYPE_PACK,
        )
        .with_mode(phalcom_semantic::types::parameter::GenericParameterMode::CompletePack),
    );
    let p_ty = store.parameter_form(p);
    let q_ty = store.parameter_form(q);

    let p_pos = store.project_positionals(p_ty).unwrap();
    let p_lab = store.project_labeled(p_ty).unwrap();
    let q_lab = store.project_labeled(q_ty).unwrap();

    // Complete(Positionals(P), Labeled(P)) ≡ P
    assert_eq!(store.canonical_complete_pack_subject(p_pos, p_lab), Some(p_ty));

    // Complete(Positionals(P), Labeled(Q)) with P != Q must NOT collapse to P or Q
    assert_eq!(store.canonical_complete_pack_subject(p_pos, q_lab), None);
}

#[test]
fn match_impl_domain_head_matches_tuple_and_unit_against_tuple_from_packs() {
    let mut store = phalcom_semantic::types::store::TypeStore::new();
    let mod_id = phalcom_modules::ModuleId::universe_root();
    let impl_id = phalcom_semantic::identity::ImplId::new(mod_id.clone(), phalcom_semantic::identity::ImplLocalId(0));
    let es = store.intern_type_parameter(
        phalcom_semantic::types::parameter::TypeParameterData::new(
            phalcom_semantic::types::parameter::TypeParameterOwner::Impl(impl_id.clone()),
            0,
            "Es",
            phalcom_semantic::KindId::COMPLETE_TYPE_PACK,
        )
        .with_mode(phalcom_semantic::types::parameter::GenericParameterMode::CompletePack),
    );
    let es_ty = store.parameter_form(es);
    let es_pos = store.project_positionals(es_ty).unwrap();
    let es_lab = store.project_labeled(es_ty).unwrap();
    let tuple_head = store.tuple_from_packs(es_pos, es_lab).unwrap();

    let int_decl = phalcom_modules::DeclarationId::new(mod_id.clone(), "Int".into());
    let str_decl = phalcom_modules::DeclarationId::new(mod_id, "String".into());
    let int_ty = store.intern_with_kind(TypeData::Nominal { declaration: int_decl }, phalcom_semantic::KindId::TYPE);
    let str_ty = store.intern_with_kind(TypeData::Nominal { declaration: str_decl }, phalcom_semantic::KindId::TYPE);
    let concrete_tuple = store.tuple(Box::new([
        phalcom_semantic::types::store::TupleTypeElement::unlabeled(int_ty),
        phalcom_semantic::types::store::TupleTypeElement::unlabeled(str_ty),
    ]));
    let unit_ty = store.unit();

    let mut bindings_tuple = std::collections::HashMap::new();
    assert!(phalcom_semantic::impls::match_impl_domain_head(
        &mut store,
        tuple_head,
        concrete_tuple,
        &impl_id,
        &mut bindings_tuple
    ));
    assert_eq!(bindings_tuple.len(), 1);
    let bound_complete = bindings_tuple.get(&es).copied().unwrap();
    let expected_complete = store.project_complete(concrete_tuple).unwrap();
    assert_eq!(bound_complete, expected_complete);

    let mut bindings_unit = std::collections::HashMap::new();
    assert!(phalcom_semantic::impls::match_impl_domain_head(
        &mut store,
        tuple_head,
        unit_ty,
        &impl_id,
        &mut bindings_unit
    ));
    assert_eq!(bindings_unit.len(), 1);
    let bound_unit_complete = bindings_unit.get(&es).copied().unwrap();
    let expected_unit_complete = store.project_complete(unit_ty).unwrap();
    assert_eq!(bound_unit_complete, expected_unit_complete);
}

#[test]
fn deferred_projection_with_generic_substitution_normalizes_to_symbolic() {
    use phalcom_semantic::generic_evidence::GenericEvidenceEnvironment;
    use phalcom_semantic::traits::{AssociatedTypeRequirementId, TraitRef};
    use phalcom_semantic::types::outcome::{CancellationToken, QueryBudget};
    use phalcom_semantic::types::projection::{ProjectionNormalizationContext, ProjectionNormalizationMode, ProjectionNormalizationResult, normalize_type};

    let fixture = Fixture::new(
        r#"
trait Iterator {
  type Item
}

class ListIterator<T> {}

impl<T> Iterator for ListIterator<T> {
  type Item = T
}

class TestBox<T> {
  @class
  testProj<I>(_ it: I, _ x: I::Item) -> I::Item
    where I |= Iterator
  {
    x
  }
}
"#,
    );

    let diags = fixture.analysis.snapshot.all_diagnostics().collect::<Vec<_>>();
    assert!(diags.is_empty(), "expected 0 diagnostics, got: {:?}", diags);

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let mod_id = phalcom_modules::ModuleId::universe_root();
    let decl_t = phalcom_modules::DeclarationId::new(mod_id, "T".into());
    let t_param = store.intern_type_parameter(phalcom_semantic::types::parameter::TypeParameterData::new(
        phalcom_semantic::types::parameter::TypeParameterOwner::Declaration(decl_t),
        0,
        "T",
        phalcom_semantic::KindId::TYPE,
    ));
    let t_ty = store.parameter_form(t_param);
    let list_iter_decl = fixture.decl("ListIterator");
    let list_iter_kind = fixture.analysis.snapshot.declarations.get(&list_iter_decl).unwrap().kind;
    let list_iter_nominal = store.intern_with_kind(TypeData::Nominal { declaration: list_iter_decl }, list_iter_kind);
    let applied_list_iter = store.apply_type_form(list_iter_nominal, &[t_ty]).unwrap();

    let iter_trait = fixture.decl("Iterator");
    let trait_ref = TraitRef::new(iter_trait.clone(), vec![].into_boxed_slice());
    let req_id = AssociatedTypeRequirementId::new(iter_trait, 0);

    let proj_ty = store.associated_projection(applied_list_iter, trait_ref, req_id);

    let env = GenericEvidenceEnvironment::empty();
    let mut budget = QueryBudget::new(100);
    let cancel = CancellationToken::new();
    let mut context = ProjectionNormalizationContext::new(
        ProjectionNormalizationMode::Generic {
            evidence: &env,
            conformance_index: Some(&fixture.analysis.snapshot.conformance_index),
            witness_plans: Some(&fixture.analysis.snapshot.conformance_witness_plans),
            trait_surfaces: &fixture.analysis.snapshot.trait_surfaces,
            derived_rules: None,
            declarations: &fixture.analysis.snapshot.declarations,
            hierarchy: &*fixture.analysis.snapshot.hierarchy,
        },
        &mut budget,
        &cancel,
    );

    let norm_res = normalize_type(&mut store, proj_ty, &mut context);
    assert_eq!(norm_res, ProjectionNormalizationResult::Normalized(t_ty));
}

#[test]
fn exact_associated_binding_survives_a_concrete_generic_conformance_assumption() {
    use phalcom_semantic::generic_evidence::GenericEvidenceEnvironment;
    use phalcom_semantic::traits::{AssociatedTypeRequirementId, TraitRef};
    use phalcom_semantic::types::outcome::{CancellationToken, QueryBudget};
    use phalcom_semantic::types::parameter::TypeParameterOwner;
    use phalcom_semantic::types::projection::{ProjectionNormalizationContext, ProjectionNormalizationMode, ProjectionNormalizationResult, normalize_type};
    use phalcom_semantic::types::store::TypeData;

    let fixture = Fixture::new(
        r#"
trait Iterable {
  type Item
}

class Item {}
class Iter<T> {}
class Owner {}

impl<T> Iterable for Iter<T> {
  type Item = T
}
"#,
    );
    let snapshot = &fixture.analysis.snapshot;
    let mut store = (*snapshot.store).clone();
    let iter_decl = fixture.decl("Iter");
    let iter_kind = snapshot.declarations.get(&iter_decl).expect("Iter declaration").kind;
    let iter_form = store.intern_with_kind(TypeData::Nominal { declaration: iter_decl }, iter_kind);
    let item_ty = store.nominal(fixture.decl("Item"));
    let iter_ty = store.apply_type_form(iter_form, &[item_ty]).expect("Iter<Item>");
    let trait_ref = TraitRef::new(fixture.decl("Iterable"), Vec::<phalcom_semantic::types::id::TypeId>::new().into_boxed_slice());
    let item_requirement = AssociatedTypeRequirementId::new(trait_ref.declaration.clone(), 0);
    let projection = store.associated_projection(iter_ty, trait_ref.clone(), item_requirement);
    let conformance = GenericConstraint::Conforms {
        subject: TypeTerm::Canonical(iter_ty),
        trait_ref,
    };
    let evidence =
        GenericEvidenceEnvironment::try_from_constraints(&[conformance], TypeParameterOwner::Declaration(fixture.decl("Owner")), &snapshot.trait_surfaces)
            .expect("generic evidence layout");
    let mut budget = QueryBudget::new(100);
    let cancel = CancellationToken::new();
    let mut context = ProjectionNormalizationContext::new(
        ProjectionNormalizationMode::Generic {
            evidence: &evidence,
            conformance_index: Some(&snapshot.conformance_index),
            witness_plans: Some(&snapshot.conformance_witness_plans),
            trait_surfaces: &snapshot.trait_surfaces,
            derived_rules: Some(&snapshot.derived_conformance_rules),
            declarations: &snapshot.declarations,
            hierarchy: &*snapshot.hierarchy,
        },
        &mut budget,
        &cancel,
    );

    assert_eq!(
        normalize_type(&mut store, projection, &mut context),
        ProjectionNormalizationResult::Normalized(item_ty)
    );
}

//! LANG007.C3.P2 T4 integration coverage for structural static-data inference.

use phalcom_modules::ModuleId;
use phalcom_semantic::checker::inference::{ConstraintOrigin, InferenceRelation, InferenceSession, InferenceTerm};
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::types::relation::MapTypeHierarchy;
use phalcom_semantic::types::store::TypeStore;

fn declaration(name: &str) -> DeclarationId {
    DeclarationId::new(ModuleId::universe_root(), name.into())
}

#[test]
fn static_constructor_payload_inference_solves_canonical_integer_term() {
    let mut store = TypeStore::new();
    let shape_declaration = declaration("Shape");
    let shape = store.nominal_static_domain(shape_declaration.clone());
    let width = store.const_int(3);
    let vector = store.intern_static_constructor(shape, shape_declaration, "Vector", vec![store.kind_of(width)].into_boxed_slice());
    let mut inference = InferenceSession::new();
    let width_var = inference.fresh_variable(store.kind_of(width));

    inference.add_constraint(
        InferenceRelation::Equivalent(
            InferenceTerm::StaticData {
                domain: shape,
                constructor: vector,
                arguments: vec![InferenceTerm::Var(width_var)].into_boxed_slice(),
            },
            InferenceTerm::StaticData {
                domain: shape,
                constructor: vector,
                arguments: vec![InferenceTerm::Canonical(width)].into_boxed_slice(),
            },
        ),
        ConstraintOrigin::Explicit,
        None,
    );

    assert!(inference.solve(&mut store, &MapTypeHierarchy::new()).is_solved());
    assert_eq!(inference.materialize(&InferenceTerm::Var(width_var), &mut store), Ok(width));
}

#[test]
fn static_constructor_inference_refutes_domain_and_case_mismatches() {
    let mut store = TypeStore::new();
    let shape_declaration = declaration("Shape");
    let other_declaration = declaration("OtherShape");
    let shape = store.nominal_static_domain(shape_declaration.clone());
    let other = store.nominal_static_domain(other_declaration.clone());
    let vector = store.intern_static_constructor(shape, shape_declaration.clone(), "Vector", Box::new([]));
    let scalar = store.intern_static_constructor(shape, shape_declaration, "Scalar", Box::new([]));
    let other_vector = store.intern_static_constructor(other, other_declaration, "Vector", Box::new([]));
    let vector_term = InferenceTerm::StaticData {
        domain: shape,
        constructor: vector,
        arguments: Box::new([]),
    };

    for mismatch in [
        InferenceTerm::StaticData {
            domain: shape,
            constructor: scalar,
            arguments: Box::new([]),
        },
        InferenceTerm::StaticData {
            domain: other,
            constructor: other_vector,
            arguments: Box::new([]),
        },
    ] {
        let mut inference = InferenceSession::new();
        inference.add_constraint(InferenceRelation::Equivalent(vector_term.clone(), mismatch), ConstraintOrigin::Explicit, None);
        assert!(
            !inference.solve(&mut store, &MapTypeHierarchy::new()).is_solved(),
            "distinct static domains or constructors must not unify"
        );
    }
}

#[test]
fn static_domain_variable_rejects_a_proper_type_candidate() {
    let mut store = TypeStore::new();
    let shape = store.nominal_static_domain(declaration("Shape"));
    let integer = store.nominal(declaration("Int"));
    let mut inference = InferenceSession::new();
    let shape_var = inference.fresh_variable(store.static_domain_kind(shape));
    inference.add_constraint(
        InferenceRelation::Equivalent(InferenceTerm::Var(shape_var), InferenceTerm::Canonical(integer)),
        ConstraintOrigin::Explicit,
        None,
    );

    assert!(!inference.solve(&mut store, &MapTypeHierarchy::new()).is_solved());
}

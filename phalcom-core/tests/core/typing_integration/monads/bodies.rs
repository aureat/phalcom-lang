use super::super::support::{Fixture, monads_source};
use phalcom_semantic::explain::{GenericConstraintOrigin, GenericConstraintRelation};
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::EvidenceStatus;
use phalcom_semantic::types::id::{TypeId, TypeParameterId};
use phalcom_semantic::types::store::TypeData;

fn parameter_form(f: &Fixture, parameter: TypeParameterId) -> TypeId {
    let mut cloned = (*f.analysis.snapshot.store).clone();
    let form = cloned.parameter_form(parameter);
    assert!(
        form.index() < f.analysis.snapshot.store.type_count(),
        "symbolic parameter form was interned only in cloned store: {form:?}"
    );
    assert!(
        matches!(f.analysis.snapshot.store.get(form), TypeData::Parameter(found) if *found == parameter),
        "original store does not contain the same canonical parameter form for {parameter:?}"
    );
    form
}

/// MON-BODY-01: the generic bind implementation type-checks its internal
/// `Monad<F>.flatMap` call with the inner method generics solved to the outer
/// symbolic A/B parameters rather than Dynamic or fresh unrelated types.
#[test]
fn bind_body_proves_symbolic_flat_map_application() {
    let f = Fixture::new(&monads_source());
    f.assert_no_errors();

    let body = f.callable("MonadAlgorithms", "bind", DispatchSide::Class);
    let call = f.expression_containing(body, "monad.flatMap(value, next)");
    let target = f.callable_id("Monad", "flatMap", DispatchSide::Instance);
    let return_ty = f
        .analysis
        .snapshot
        .callable_signatures
        .get(&body.callable)
        .expect("bind signature")
        .declared_return
        .to_knowledge()
        .ty()
        .expect("bind return type");
    f.assert_expression_call(call, &target, return_ty);

    let receiver_ty = f.binding(body, "monad").current.ty().expect("monad type");
    f.assert_callable_selection(body, call, &target, receiver_ty, &f.decl("Monad"), &[f.decl("Monad")]);

    let inner_a = f.callable_generic_parameter("Monad", "flatMap", DispatchSide::Instance, 0);
    let inner_b = f.callable_generic_parameter("Monad", "flatMap", DispatchSide::Instance, 1);
    let outer_a = f.callable_generic_parameter("MonadAlgorithms", "bind", DispatchSide::Class, 1);
    let outer_b = f.callable_generic_parameter("MonadAlgorithms", "bind", DispatchSide::Class, 2);
    f.assert_generic_solution_exact(body, call, inner_a, parameter_form(&f, outer_a), EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(body, call, inner_b, parameter_form(&f, outer_b), EvidenceStatus::Assumed);

    let value_ty = f.binding(body, "value").current.ty().expect("value type");
    let next_ty = f.binding(body, "next").current.ty().expect("next type");
    f.assert_generic_constraint_exact(
        body,
        call,
        inner_a,
        GenericConstraintOrigin::Argument { parameter_index: 0 },
        GenericConstraintRelation::SupertypeOf(value_ty),
    );
    f.assert_generic_constraint_exact(
        body,
        call,
        inner_b,
        GenericConstraintOrigin::Argument { parameter_index: 1 },
        GenericConstraintRelation::SupertypeOf(next_ty),
    );
}

/// MON-BODY-02: Kleisli's internal flatMap call maps the inner Monad method's
/// A/B exactly onto the outer Kleisli B/C parameters.
#[test]
fn kleisli_body_proves_symbolic_callable_composition() {
    let f = Fixture::new(&monads_source());
    f.assert_no_errors();

    let body = f.callable("MonadAlgorithms", "kleisli", DispatchSide::Class);
    let call = f.expression_containing(body, "monad.flatMap(first.call(value), second)");
    let target = f.callable_id("Monad", "flatMap", DispatchSide::Instance);
    let receiver_ty = f.binding(body, "monad").current.ty().expect("monad type");
    f.assert_callable_selection(body, call, &target, receiver_ty, &f.decl("Monad"), &[f.decl("Monad")]);

    let inner_a = f.callable_generic_parameter("Monad", "flatMap", DispatchSide::Instance, 0);
    let inner_b = f.callable_generic_parameter("Monad", "flatMap", DispatchSide::Instance, 1);
    let outer_b = f.callable_generic_parameter("MonadAlgorithms", "kleisli", DispatchSide::Class, 2);
    let outer_c = f.callable_generic_parameter("MonadAlgorithms", "kleisli", DispatchSide::Class, 3);
    f.assert_generic_solution_exact(body, call, inner_a, parameter_form(&f, outer_b), EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(body, call, inner_b, parameter_form(&f, outer_c), EvidenceStatus::Assumed);
}

/// MON-BODY-03: traverse resolves all three capability calls through the generic
/// hierarchy while F/A/B are still symbolic: pure via Applicative, flatMap via
/// Monad, and map via Functor.
#[test]
fn traverse_body_resolves_symbolic_inherited_capabilities() {
    let f = Fixture::new(&monads_source());
    f.assert_no_errors();

    let body = f.callable("MonadAlgorithms", "traverse", DispatchSide::Class);
    let state_ty = f.binding(body, "state").current.ty().expect("state type");
    let receiver_ty = f.binding(body, "monad").current.ty().expect("monad type");

    let pure_call = f.expression_containing(body, "monad.pure(empty)");
    let pure_target = f.callable_id("Applicative", "pure", DispatchSide::Instance);
    f.assert_expression_call(pure_call, &pure_target, state_ty);
    f.assert_callable_selection(
        body,
        pure_call,
        &pure_target,
        receiver_ty,
        &f.decl("Applicative"),
        &[f.decl("Monad"), f.decl("Applicative")],
    );

    let flat_map_call = f.expression_containing(body, "monad.flatMap(state");
    let flat_map_target = f.callable_id("Monad", "flatMap", DispatchSide::Instance);
    f.assert_expression_call(flat_map_call, &flat_map_target, state_ty);
    f.assert_callable_selection(body, flat_map_call, &flat_map_target, receiver_ty, &f.decl("Monad"), &[f.decl("Monad")]);

    let map_call = f.expression_containing(body, "monad.map(transform.call(value)");
    let map_target = f.callable_id("Functor", "map", DispatchSide::Instance);
    f.assert_expression_call(map_call, &map_target, state_ty);
    f.assert_callable_selection(
        body,
        map_call,
        &map_target,
        receiver_ty,
        &f.decl("Functor"),
        &[f.decl("Monad"), f.decl("Applicative"), f.decl("Functor")],
    );
}

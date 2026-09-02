use super::support::Fixture;
use phalcom_semantic::explain::{GenericConstraintOrigin, GenericConstraintRelation};
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::EvidenceStatus;

/// MON-CALL-01/03/04: inherited Functor.map specializes the receiver constructor,
/// then method inference solves the exact callable-owned A/B identities from
/// arguments, with the selected callable and receiver path recorded formally.
#[test]
fn inherited_map_specializes_constructor_and_proves_method_generics() {
    let f = Fixture::new(&super::support::semantic_source());
    f.assert_no_errors();

    let run = f.callable("MonadSemanticProbe", "inheritedMap", DispatchSide::Class);
    let mapped = f.binding(run, "mapped").current.ty().expect("mapped type");
    f.assert_either(mapped, f.ty("String"), f.ty("Bool"));

    let call = f.expression_containing(run, "monad.map(");
    let target = f.callable_id("Functor", "map", DispatchSide::Instance);
    f.assert_expression_call(call, &target, mapped);
    f.assert_callable_selection(
        run,
        call,
        &target,
        f.ty("StringContractEitherMonad"),
        &f.decl("Functor"),
        &[
            f.decl("StringContractEitherMonad"),
            f.decl("ContractEitherMonad"),
            f.decl("Monad"),
            f.decl("Applicative"),
            f.decl("Functor"),
        ],
    );

    let a = f.callable_generic_parameter("Functor", "map", DispatchSide::Instance, 0);
    let b = f.callable_generic_parameter("Functor", "map", DispatchSide::Instance, 1);
    f.assert_generic_solution_exact(run, call, a, f.ty("Int"), EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(run, call, b, f.ty("Bool"), EvidenceStatus::Established);

    let source_ty = f.binding(run, "source").current.ty().expect("source type");
    let closure = f.expression_containing(run, "|value| { value > 0 }");
    let closure_ty = closure.knowledge.ty().expect("closure type");
    f.assert_generic_constraint_exact(
        run,
        call,
        a,
        GenericConstraintOrigin::Argument { parameter_index: 0 },
        GenericConstraintRelation::SupertypeOf(source_ty),
    );
    f.assert_generic_constraint_exact(
        run,
        call,
        b,
        GenericConstraintOrigin::Argument { parameter_index: 1 },
        GenericConstraintRelation::SupertypeOf(closure_ty),
    );
}

/// MON-CALL-05: an inherited Applicative method sees the same specialized F and
/// performs fresh method-generic inference with an exact callable identity.
#[test]
fn inherited_pure_specializes_through_applicative() {
    let f = Fixture::new(&super::support::semantic_source());
    f.assert_no_errors();

    let run = f.callable("MonadSemanticProbe", "inheritedPure", DispatchSide::Class);
    let lifted = f.binding(run, "lifted").current.ty().expect("lifted type");
    f.assert_either(lifted, f.ty("String"), f.ty("Int"));

    let call = f.expression_containing(run, "monad.pure(42)");
    let target = f.callable_id("Applicative", "pure", DispatchSide::Instance);
    f.assert_expression_call(call, &target, lifted);
    f.assert_callable_selection(
        run,
        call,
        &target,
        f.ty("StringContractEitherMonad"),
        &f.decl("Applicative"),
        &[
            f.decl("StringContractEitherMonad"),
            f.decl("ContractEitherMonad"),
            f.decl("Monad"),
            f.decl("Applicative"),
        ],
    );

    let a = f.callable_generic_parameter("Applicative", "pure", DispatchSide::Instance, 0);
    f.assert_generic_solution_exact(run, call, a, f.ty("Int"), EvidenceStatus::Established);
    f.assert_generic_constraint_exact(
        run,
        call,
        a,
        GenericConstraintOrigin::Argument { parameter_index: 0 },
        GenericConstraintRelation::SupertypeOf(f.ty("Int")),
    );
}

/// MON-CALL-02/05: Monad.flatMap combines class-level F specialization with
/// exact callable-owned A/B identities without conflating either binder scope.
#[test]
fn inherited_flat_map_keeps_class_and_method_generic_scopes_distinct() {
    let f = Fixture::new(&super::support::semantic_source());
    f.assert_no_errors();

    let run = f.callable("MonadSemanticProbe", "inheritedFlatMap", DispatchSide::Class);
    let chained = f.binding(run, "chained").current.ty().expect("chained type");
    f.assert_either(chained, f.ty("String"), f.ty("Bool"));

    let call = f.expression_containing(run, "monad.flatMap(source, next)");
    let target = f.callable_id("Monad", "flatMap", DispatchSide::Instance);
    f.assert_expression_call(call, &target, chained);
    f.assert_callable_selection(
        run,
        call,
        &target,
        f.ty("StringContractEitherMonad"),
        &f.decl("Monad"),
        &[f.decl("StringContractEitherMonad"), f.decl("ContractEitherMonad"), f.decl("Monad")],
    );

    let a = f.callable_generic_parameter("Monad", "flatMap", DispatchSide::Instance, 0);
    let b = f.callable_generic_parameter("Monad", "flatMap", DispatchSide::Instance, 1);
    f.assert_generic_solution_exact(run, call, a, f.ty("Int"), EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(run, call, b, f.ty("Bool"), EvidenceStatus::Assumed);

    let source_ty = f.binding(run, "source").current.ty().expect("source type");
    let next_ty = f.binding(run, "next").current.ty().expect("next type");
    f.assert_generic_constraint_exact(
        run,
        call,
        a,
        GenericConstraintOrigin::Argument { parameter_index: 0 },
        GenericConstraintRelation::SupertypeOf(source_ty),
    );
    f.assert_generic_constraint_exact(
        run,
        call,
        b,
        GenericConstraintOrigin::Argument { parameter_index: 1 },
        GenericConstraintRelation::SupertypeOf(next_ty),
    );

    assert!(matches!(f.analysis.snapshot.store.type_parameter(a).owner, phalcom_semantic::types::parameter::TypeParameterOwner::Callable(ref owner) if owner == &target));
    assert!(matches!(f.analysis.snapshot.store.type_parameter(b).owner, phalcom_semantic::types::parameter::TypeParameterOwner::Callable(ref owner) if owner == &target));
}

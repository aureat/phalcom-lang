use super::support::{Fixture, with_monads};
use phalcom_semantic::explain::{GenericConstraintOrigin, GenericConstraintRelation};
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::EvidenceStatus;

/// MON-OVERRIDE-01: the nearest concrete `EitherMonad.map` implementation wins
/// over the inherited Functor contract after receiver specialization.
#[test]
fn specialized_map_override_wins_and_keeps_generic_inference() {
    let source = with_monads(
        r#"
class OverrideSelectionProbe {
    @class
    run(
        _ monad: StringEitherMonad,
        _ source: Either<String, Int>
    ) {
        let mapped = monad.map(source, |value| { value > 0 })
    }
}
"#,
    );
    let f = Fixture::new(&source);
    f.assert_no_errors();

    let run = f.callable("OverrideSelectionProbe", "run", DispatchSide::Class);
    let mapped = f.binding(run, "mapped").current.ty().expect("mapped type");
    f.assert_either(mapped, f.ty("String"), f.ty("Bool"));

    let call = f.expression_containing(run, "monad.map(source");
    let target = f.callable_id("EitherMonad", "map", DispatchSide::Instance);
    f.assert_expression_call(call, &target, mapped);
    f.assert_callable_selection(
        run,
        call,
        &target,
        f.ty("StringEitherMonad"),
        &f.decl("EitherMonad"),
        &[f.decl("StringEitherMonad"), f.decl("EitherMonad")],
    );

    let a = f.callable_generic_parameter("EitherMonad", "map", DispatchSide::Instance, 0);
    let b = f.callable_generic_parameter("EitherMonad", "map", DispatchSide::Instance, 1);
    f.assert_generic_solution_exact(run, call, a, f.ty("Int"), EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(run, call, b, f.ty("Bool"), EvidenceStatus::Established);

    let source_ty = f.binding(run, "source").current.ty().expect("source type");
    f.assert_generic_constraint_exact(
        run,
        call,
        a,
        GenericConstraintOrigin::Argument { parameter_index: 0 },
        GenericConstraintRelation::SupertypeOf(source_ty),
    );
}

/// MON-OVERRIDE-02: the specialized Applicative override `map2` remains the
/// selected implementation and independently solves all method-owned generics.
#[test]
fn specialized_map2_override_wins_and_solves_three_generics() {
    let source = with_monads(
        r#"
class OverrideMap2Probe {
    @class
    run(
        _ monad: StringEitherMonad,
        _ left: Either<String, Int>,
        _ right: Either<String, Int>
    ) {
        let combined = monad.map2(left, right, |a, b| { a < b })
    }
}
"#,
    );
    let f = Fixture::new(&source);
    f.assert_no_errors();

    let run = f.callable("OverrideMap2Probe", "run", DispatchSide::Class);
    let combined = f.binding(run, "combined").current.ty().expect("combined type");
    f.assert_either(combined, f.ty("String"), f.ty("Bool"));

    let call = f.expression_containing(run, "monad.map2(left, right");
    let target = f.callable_id("EitherMonad", "map2", DispatchSide::Instance);
    f.assert_expression_call(call, &target, combined);
    f.assert_callable_selection(
        run,
        call,
        &target,
        f.ty("StringEitherMonad"),
        &f.decl("EitherMonad"),
        &[f.decl("StringEitherMonad"), f.decl("EitherMonad")],
    );

    let a = f.callable_generic_parameter("EitherMonad", "map2", DispatchSide::Instance, 0);
    let b = f.callable_generic_parameter("EitherMonad", "map2", DispatchSide::Instance, 1);
    let c = f.callable_generic_parameter("EitherMonad", "map2", DispatchSide::Instance, 2);
    f.assert_generic_solution_exact(run, call, a, f.ty("Int"), EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(run, call, b, f.ty("Int"), EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(run, call, c, f.ty("Bool"), EvidenceStatus::Established);
}

/// MON-OVERRIDE-03: concrete override selection is semantically distinct from
/// the contract-only inheritance probes; inherited tests must continue to target
/// Functor/Applicative/Monad while executable tests target EitherMonad.
#[test]
fn contract_and_concrete_hierarchies_resolve_to_distinct_callables() {
    let f = Fixture::new(super::support::monads_source());
    f.assert_no_errors();

    let contract_map = f.callable_id("Functor", "map", DispatchSide::Instance);
    let concrete_map = f.callable_id("EitherMonad", "map", DispatchSide::Instance);
    assert_ne!(contract_map, concrete_map);
}

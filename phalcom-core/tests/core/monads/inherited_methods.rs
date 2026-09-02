use super::support::Fixture;
use phalcom_semantic::explain::GenericConstraintOrigin;
use phalcom_semantic::identity::DispatchSide;

/// MON-CALL-01/03/04: inherited Functor.map specializes the receiver constructor,
/// then method inference solves A/B from arguments, with both facts present in
/// the explanation graph.
#[test]
fn inherited_map_specializes_constructor_and_proves_method_generics() {
    let f = Fixture::new(&super::support::semantic_source());
    f.assert_no_errors();

    let run = f.callable("MonadSemanticProbe", "inheritedMap", DispatchSide::Class);
    let mapped = f.binding(run, "mapped").current.ty().expect("mapped type");
    f.assert_either(mapped, f.ty("String"), f.ty("Bool"));

    let call = f.expression_containing(run, "monad.map(");
    f.assert_callable_selection_path(
        run,
        call,
        "Functor",
        &["StringEitherMonad", "EitherMonad", "Monad", "Applicative", "Functor"],
    );
    f.assert_generic_solution(run, call, "A", f.ty("Int"));
    f.assert_generic_solution(run, call, "B", f.ty("Bool"));
    f.assert_generic_constraint_origin(
        run,
        call,
        "A",
        GenericConstraintOrigin::Argument { parameter_index: 0 },
    );
    f.assert_generic_constraint_origin(
        run,
        call,
        "B",
        GenericConstraintOrigin::Argument { parameter_index: 1 },
    );
}

/// MON-CALL-05: an inherited Applicative method sees the same specialized F and
/// still performs fresh method-generic inference.
#[test]
fn inherited_pure_specializes_through_applicative() {
    let f = Fixture::new(&super::support::semantic_source());
    f.assert_no_errors();

    let run = f.callable("MonadSemanticProbe", "inheritedPure", DispatchSide::Class);
    let lifted = f.binding(run, "lifted").current.ty().expect("lifted type");
    f.assert_either(lifted, f.ty("String"), f.ty("Int"));

    let call = f.expression_containing(run, "monad.pure(42)");
    f.assert_callable_selection_path(
        run,
        call,
        "Applicative",
        &["StringEitherMonad", "EitherMonad", "Monad", "Applicative"],
    );
    f.assert_generic_solution(run, call, "A", f.ty("Int"));
    f.assert_generic_constraint_origin(
        run,
        call,
        "A",
        GenericConstraintOrigin::Argument { parameter_index: 0 },
    );
}

/// MON-CALL-02/05: Monad.flatMap combines class-level F specialization with
/// callable-owned A/B without conflating either binder scope.
#[test]
fn inherited_flat_map_keeps_class_and_method_generic_scopes_distinct() {
    let f = Fixture::new(&super::support::semantic_source());
    f.assert_no_errors();

    let run = f.callable("MonadSemanticProbe", "inheritedFlatMap", DispatchSide::Class);
    let chained = f.binding(run, "chained").current.ty().expect("chained type");
    f.assert_either(chained, f.ty("String"), f.ty("Bool"));

    let call = f.expression_containing(run, "monad.flatMap(source, next)");
    f.assert_callable_selection_path(
        run,
        call,
        "Monad",
        &["StringEitherMonad", "EitherMonad", "Monad"],
    );
    f.assert_generic_solution(run, call, "A", f.ty("Int"));
    f.assert_generic_solution(run, call, "B", f.ty("Bool"));
    f.assert_solution_parameter_is_callable_owned(run, call, "A");
    f.assert_solution_parameter_is_callable_owned(run, call, "B");
}

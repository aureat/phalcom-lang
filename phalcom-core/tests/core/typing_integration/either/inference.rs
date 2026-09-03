use super::super::support::{Fixture, either, nominal, either_semantic_source as semantic_source};
use phalcom_semantic::explain::GenericConstraintOrigin;
use phalcom_semantic::identity::DispatchSide;

/// GEN-06/07/08: constructor payload and expected type jointly solve `Either<L, R>`.
#[test]
fn left_constructor_combines_argument_and_expected_result_constraints() {
    let f = Fixture::new(&semantic_source());
    let run = f.callable("EitherInferenceProbe", "contextualLeft", DispatchSide::Class);
    f.assert_known_generic_binding(run, "contextualLeft", &either(nominal("String"), nominal("Int")));

    let call = f.expression_containing(run, "Either::Left(\"failure\")");
    f.assert_ready(call);
    f.assert_generic_constraint_origin(run, call, "L", GenericConstraintOrigin::Argument { parameter_index: 0 });
    f.assert_generic_constraint_origin(run, call, "R", GenericConstraintOrigin::ExpectedResult);
    f.assert_generic_solution(run, call, "L", f.ty("String"));
    f.assert_generic_solution(run, call, "R", f.ty("Int"));
}

/// GEN-06/07/08: the symmetric constructor path solves `R` from payload and `L` from context.
#[test]
fn right_constructor_combines_argument_and_expected_result_constraints() {
    let f = Fixture::new(&semantic_source());
    let run = f.callable("EitherInferenceProbe", "contextualRight", DispatchSide::Class);
    f.assert_known_generic_binding(run, "contextualRight", &either(nominal("String"), nominal("Int")));

    let call = f.expression_containing(run, "Either::Right(42)");
    f.assert_ready(call);
    f.assert_generic_constraint_origin(run, call, "R", GenericConstraintOrigin::Argument { parameter_index: 0 });
    f.assert_generic_constraint_origin(run, call, "L", GenericConstraintOrigin::ExpectedResult);
    f.assert_generic_solution(run, call, "L", f.ty("String"));
    f.assert_generic_solution(run, call, "R", f.ty("Int"));
}

/// GEN-15: distinct constructor paths canonicalize to the same applied family type.
#[test]
fn left_and_right_paths_converge_on_one_canonical_applied_type() {
    let f = Fixture::new(&semantic_source());
    let run = f.callable("EitherInferenceProbe", "canonicalPaths", DispatchSide::Class);
    let left = f.assert_binding_applied(run, "fromLeft", "Either", &[nominal("String"), nominal("Int")]);
    let right = f.assert_binding_applied(run, "fromRight", "Either", &[nominal("String"), nominal("Int")]);
    assert_eq!(left, right, "canonical applied types must have identical TypeId identity");
    f.assert_no_errors();
}

use super::super::support::{Fixture, either, nominal, either_semantic_source as semantic_source};
use phalcom_semantic::identity::DispatchSide;

/// GEN-12/13/14/33/34: nested applied types recursively solve and reuse the same `L` parameter.
#[test]
fn nested_applied_type_recursively_solves_flatten() {
    let f = Fixture::new(&semantic_source());
    let run = f.callable("EitherNestedProbe", "flattenNested", DispatchSide::Class);
    f.assert_known_generic_binding(run, "flattened", &either(nominal("String"), nominal("Int")));

    let call = f.expression_containing(run, "EitherGenericProbe.flatten(outer)");
    f.assert_generic_solution(run, call, "L", f.ty("String"));
    f.assert_generic_solution(run, call, "R", f.ty("Int"));
}

/// GEN-04/05: repeated occurrences of one generic parameter impose one consistent substitution.
#[test]
fn repeated_generic_variable_unifies_to_one_type() {
    let f = Fixture::new(&semantic_source());
    let run = f.callable("EitherNestedProbe", "repeatedVariable", DispatchSide::Class);
    f.assert_binding_nominal(run, "leftValue", "Int");
    f.assert_binding_nominal(run, "rightValue", "Int");

    let left_call = f.expression_containing(run, "EitherGenericProbe.merge(left)");
    f.assert_generic_solution(run, left_call, "T", f.ty("Int"));
    let right_call = f.expression_containing(run, "EitherGenericProbe.merge(right)");
    f.assert_generic_solution(run, right_call, "T", f.ty("Int"));
}

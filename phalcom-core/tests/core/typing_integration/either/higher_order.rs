use super::super::support::{Fixture, either, nominal, either_semantic_source as semantic_source};
use phalcom_semantic::identity::DispatchSide;

/// GEN-21/22/23/24: generic variables are solved across an applied argument and a callable argument.
#[test]
fn higher_order_lift_solves_across_multiple_arguments() {
    let f = Fixture::new(&semantic_source());
    let run = f.callable("EitherHigherOrderProbe", "inferAcrossArguments", DispatchSide::Class);
    f.assert_known_generic_binding(run, "lifted", &either(nominal("String"), nominal("Bool")));

    let call = f.expression_containing(run, "EitherGenericProbe.lift");
    f.assert_generic_solution(run, call, "L", f.ty("String"));
    f.assert_generic_solution(run, call, "A", f.ty("Int"));
    f.assert_generic_solution(run, call, "B", f.ty("Bool"));
}

/// GEN-02/26/27: the same generic callable gets a fresh substitution environment per invocation.
#[test]
fn repeated_higher_order_calls_do_not_leak_substitutions() {
    let f = Fixture::new(&semantic_source());
    let run = f.callable("EitherHigherOrderProbe", "freshCalls", DispatchSide::Class);
    f.assert_known_generic_binding(run, "first", &either(nominal("String"), nominal("Bool")));
    f.assert_known_generic_binding(run, "second", &either(nominal("Int"), nominal("String")));

    let first_call = f.expression_containing(run, "EitherGenericProbe.lift(\n            firstInput");
    f.assert_generic_solution(run, first_call, "L", f.ty("String"));
    f.assert_generic_solution(run, first_call, "A", f.ty("Int"));
    f.assert_generic_solution(run, first_call, "B", f.ty("Bool"));

    let second_call = f.expression_containing(run, "EitherGenericProbe.lift(\n            secondInput");
    f.assert_generic_solution(run, second_call, "L", f.ty("Int"));
    f.assert_generic_solution(run, second_call, "A", f.ty("Bool"));
    f.assert_generic_solution(run, second_call, "B", f.ty("String"));
}

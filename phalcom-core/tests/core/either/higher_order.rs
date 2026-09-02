use super::support::{Fixture, either, nominal, semantic_source};
use phalcom_semantic::identity::DispatchSide;

/// GEN-21/22/23/24: generic variables are solved across an applied argument and a callable argument.
#[test]
fn higher_order_lift_solves_across_multiple_arguments() {
    let f = Fixture::new(&semantic_source());
    let run = f.callable("EitherHigherOrderProbe", "inferAcrossArguments", DispatchSide::Class);
    f.assert_known_generic_binding(run, "lifted", &either(nominal("String"), nominal("Bool")));

    let call = f.expression_containing(run, "EitherGenericProbe.lift");
    f.assert_generic_solution(run, call, "L", &nominal("String"));
    f.assert_generic_solution(run, call, "A", &nominal("Int"));
    f.assert_generic_solution(run, call, "B", &nominal("Bool"));
}

/// GEN-02/26/27: the same generic callable gets a fresh substitution environment per invocation.
#[test]
fn repeated_higher_order_calls_do_not_leak_substitutions() {
    let f = Fixture::new(&semantic_source());
    let run = f.callable("EitherHigherOrderProbe", "freshCalls", DispatchSide::Class);
    f.assert_known_generic_binding(run, "first", &either(nominal("String"), nominal("Bool")));
    f.assert_known_generic_binding(run, "second", &either(nominal("Int"), nominal("String")));

    let first_call = f.expression_containing(run, "EitherGenericProbe.lift(\n            firstInput");
    f.assert_generic_solution(run, first_call, "L", &nominal("String"));
    f.assert_generic_solution(run, first_call, "A", &nominal("Int"));
    f.assert_generic_solution(run, first_call, "B", &nominal("Bool"));

    let second_call = f.expression_containing(run, "EitherGenericProbe.lift(\n            secondInput");
    f.assert_generic_solution(run, second_call, "L", &nominal("Int"));
    f.assert_generic_solution(run, second_call, "A", &nominal("Bool"));
    f.assert_generic_solution(run, second_call, "B", &nominal("String"));
}

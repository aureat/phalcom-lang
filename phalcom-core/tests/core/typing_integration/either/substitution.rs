use super::super::support::{Fixture, either, nominal, either_semantic_source as semantic_source};
use phalcom_semantic::identity::DispatchSide;

/// GEN-16/17/18/23/24: receiver specialization fixes `L`/`R`; closure result solves fresh `R2`.
#[test]
fn map_preserves_left_and_solves_new_right_from_closure() {
    let f = Fixture::new(&semantic_source());
    let run = f.callable("EitherInferenceProbe", "mapInference", DispatchSide::Class);
    f.assert_known_generic_binding(run, "mapped", &either(nominal("String"), nominal("Bool")));

    let call = f.expression_containing(run, "source.map");
    f.assert_receiver_selection(run, call, &either(nominal("String"), nominal("Int")));
    f.assert_generic_solution(run, call, "R2", f.ty("Bool"));
}

/// GEN-18: `mapLeft` changes `L` and preserves `R` exactly.
#[test]
fn map_left_replaces_only_left_parameter() {
    let f = Fixture::new(&semantic_source());
    let run = f.callable("EitherInferenceProbe", "mapLeftInference", DispatchSide::Class);
    f.assert_known_generic_binding(run, "mappedLeft", &either(nominal("Bool"), nominal("Int")));

    let call = f.expression_containing(run, "source.mapLeft");
    f.assert_receiver_selection(run, call, &either(nominal("String"), nominal("Int")));
    f.assert_generic_solution(run, call, "L2", f.ty("Bool"));
}

/// GEN-19: `bimap` solves two independent method-owned generic parameters.
#[test]
fn bimap_replaces_both_generic_parameters() {
    let f = Fixture::new(&semantic_source());
    let run = f.callable("EitherInferenceProbe", "bimapInference", DispatchSide::Class);
    f.assert_known_generic_binding(run, "bimapped", &either(nominal("Bool"), nominal("Bool")));

    let call = f.expression_containing(run, "source.bimap");
    f.assert_generic_solution(run, call, "L2", f.ty("Bool"));
    f.assert_generic_solution(run, call, "R2", f.ty("Bool"));
}

/// GEN-20: non-generic member specialization may permute receiver-owned parameters.
#[test]
fn swap_permutes_receiver_generic_arguments() {
    let f = Fixture::new(&semantic_source());
    let run = f.callable("EitherInferenceProbe", "swapInference", DispatchSide::Class);
    f.assert_known_generic_binding(run, "swapped", &either(nominal("Int"), nominal("String")));
}

/// GEN-18/19: `orElse` replaces the left parameter but preserves the receiver's right parameter.
#[test]
fn or_else_replaces_left_and_preserves_right() {
    let f = Fixture::new(&semantic_source());
    let run = f.callable("EitherInferenceProbe", "orElseInference", DispatchSide::Class);
    f.assert_known_generic_binding(run, "recovered", &either(nominal("Bool"), nominal("Int")));
}

/// GEN-31/32: specialized applied types must compose across a chain of generic operations.
#[test]
fn chained_substitutions_compose_without_reverting_to_generic_forms() {
    let f = Fixture::new(&semantic_source());
    let run = f.callable("EitherInferenceProbe", "chainedInference", DispatchSide::Class);
    f.assert_known_generic_binding(run, "mapped", &either(nominal("String"), nominal("Bool")));
    f.assert_known_generic_binding(run, "leftMapped", &either(nominal("Int"), nominal("Bool")));
    f.assert_known_generic_binding(run, "swapped", &either(nominal("Bool"), nominal("Int")));
    f.assert_no_errors();
}

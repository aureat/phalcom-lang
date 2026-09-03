use super::super::support::{Fixture, either, nominal, either_semantic_source as semantic_source, with_either};
use phalcom_semantic::identity::DispatchSide;

fn isolated_fixture() -> Fixture {
    Fixture::new(&format!(
        "{}\n{}",
        semantic_source(),
        r#"
class EitherIsolationProbe {
    @class
    mapOnParameter(_ source: Either<String, Int>) {
        let mapped = source.map(|value| { value > 0 })
    }

    @class
    swapOnParameter(_ source: Either<String, Int>) {
        let swapped = source.swap
    }

    @class
    liftOnParameter(_ source: Either<String, Int>) {
        let lifted = EitherGenericProbe.lift(
            source,
            |value| { value > 0 }
        )
    }

    @class
    mergeOnParameter(_ source: Either<Int, Int>) {
        let merged = EitherGenericProbe.merge(source)
    }

    @class
    flattenOnParameter(_ source: Either<String, Either<String, Int>>) {
        let flattened = EitherGenericProbe.flatten(source)
    }
}
"#
    ))
}

/// Diagnostic isolation: the user-space `Either` declaration and its method bodies are valid on their own.
#[test]
fn either_definition_is_semantically_well_formed() {
    let f = Fixture::new(&with_either(""));
    f.assert_no_errors();
}

/// Diagnostic isolation: receiver specialization without any variant-constructor inference.
#[test]
fn receiver_generic_method_specialization_works_from_typed_parameter() {
    let f = isolated_fixture();
    let run = f.callable("EitherIsolationProbe", "mapOnParameter", DispatchSide::Class);
    f.assert_known_generic_binding(run, "mapped", &either(nominal("String"), nominal("Bool")));
}

/// Diagnostic isolation: non-generic receiver-owned return substitution without constructor inference.
#[test]
fn receiver_non_generic_member_substitution_works_from_typed_parameter() {
    let f = isolated_fixture();
    let run = f.callable("EitherIsolationProbe", "swapOnParameter", DispatchSide::Class);
    f.assert_known_generic_binding(run, "swapped", &either(nominal("Int"), nominal("String")));
}

/// Diagnostic isolation: higher-order generic call inference from an already-specialized argument.
#[test]
fn static_higher_order_inference_works_from_typed_parameter() {
    let f = isolated_fixture();
    let run = f.callable("EitherIsolationProbe", "liftOnParameter", DispatchSide::Class);
    f.assert_known_generic_binding(run, "lifted", &either(nominal("String"), nominal("Bool")));
}

/// Diagnostic isolation: repeated generic variable constraints from one applied argument.
#[test]
fn repeated_generic_variable_inference_works_from_typed_parameter() {
    let f = isolated_fixture();
    let run = f.callable("EitherIsolationProbe", "mergeOnParameter", DispatchSide::Class);
    f.assert_binding_nominal(run, "merged", "Int");
}

/// Diagnostic isolation: nested applied generic inference without constructor-produced inputs.
#[test]
fn nested_applied_inference_works_from_typed_parameter() {
    let f = isolated_fixture();
    let run = f.callable("EitherIsolationProbe", "flattenOnParameter", DispatchSide::Class);
    f.assert_known_generic_binding(run, "flattened", &either(nominal("String"), nominal("Int")));
}

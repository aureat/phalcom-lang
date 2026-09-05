use super::super::support::{Fixture, expression_semantic_source as semantic_source};
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::TypeKnowledge;
use phalcom_semantic::types::store::TypeData;

fn assert_higher_order_result(method: &str, expected: &str) {
    let f = Fixture::new(&semantic_source());
    f.assert_no_errors();

    let run = f.callable("ExpressionSemanticProbe", method, DispatchSide::Class);
    let result = f.binding(run, "result");
    let result_ty = result.current.ty().expect("higher-order evaluation result type");
    f.assert_either(result_ty, f.ty("String"), f.ty(expected));
    assert!(matches!(result.current, TypeKnowledge::Known(_)), "result must remain known: {result:#?}");

    let call = f.expression_containing(run, "ExpressionEvaluation.eval(monad, expression)");
    let target = f.callable_id("ExpressionEvaluation", "eval", DispatchSide::Class);
    f.assert_expression_call(call, &target, result_ty);
}

/// GEX-02/GEX-04: Map preserves A -> B through its function-valued field and
/// returns the exact indexed effect result.
#[test]
fn map_preserves_callable_input_and_result_indices() {
    assert_higher_order_result("mapEvaluation", "Bool");
}

/// GEX-02/GEX-05: FlatMap keeps constructor-local A/B fresh and composes the
/// continuation's indexed expression with Monad.flatMap.
#[test]
fn flat_map_preserves_continuation_result_index() {
    assert_higher_order_result("flatMapEvaluation", "Bool");
}

/// GEX-04: Apply relates Expression<F,(A)->B> and Expression<F,A> to the
/// exact Expression<F,B> result before evaluation.
#[test]
fn apply_preserves_function_argument_relationship() {
    let f = Fixture::new(&semantic_source());
    f.assert_no_errors();

    let run = f.callable("ExpressionSemanticProbe", "applyEvaluation", DispatchSide::Class);
    let function = f.binding(run, "function").current.ty().expect("function expression type");
    let function_args = f.assert_applied(f.family_type(function), "Expression", 2);
    let function_type = function_args[1];
    let TypeData::Callable(signature) = f.analysis.snapshot.store.get(function_type) else {
        panic!(
            "Apply function payload must be callable: {}",
            f.analysis.snapshot.store.format_type(function_type)
        );
    };
    assert_eq!(signature.parameters.len(), 1);
    assert_eq!(signature.parameters[0].ty, f.ty("Int"));
    assert_eq!(signature.return_type, f.ty("Bool"));

    let argument = f.binding(run, "argument").current.ty().expect("argument expression type");
    let argument_args = f.assert_applied(f.family_type(argument), "Expression", 2);
    assert_eq!(argument_args[0], function_args[0], "Apply must use one effect constructor");
    assert_eq!(argument_args[1], f.ty("Int"));

    let expression = f.binding(run, "expression").current.ty().expect("Apply expression type");
    let expression_args = f.assert_applied(f.family_type(expression), "Expression", 2);
    assert_eq!(expression_args[0], function_args[0]);
    assert_eq!(expression_args[1], f.ty("Bool"));

    let call = f.expression_containing(run, "ExpressionEvaluation.eval(monad, expression)");
    let target = f.callable_id("ExpressionEvaluation", "eval", DispatchSide::Class);
    f.assert_expression_call(
        call,
        &target,
        f.assert_binding_applied(
            run,
            "result",
            "Either",
            &[super::super::support::nominal("String"), super::super::support::nominal("Bool")],
        ),
    );
}

/// A concrete outer `Expression<F, (Int) -> Bool>` result must specialize the
/// callable generic inside `Expression::Pure` before its closure body is
/// checked.
#[test]
fn pure_callable_payload_keeps_contextual_signature() {
    let f = Fixture::new(&semantic_source());
    f.assert_no_errors();

    let run = f.callable("ExpressionSemanticProbe", "applyEvaluation", DispatchSide::Class);
    let function = f.binding(run, "function").current.ty().expect("function expression type");
    let function_args = f.assert_applied(f.family_type(function), "Expression", 2);
    let TypeData::Callable(signature) = f.analysis.snapshot.store.get(function_args[1]) else {
        panic!(
            "Pure callable payload must be callable: {}",
            f.analysis.snapshot.store.format_type(function_args[1])
        );
    };
    assert_eq!(signature.parameters.len(), 1);
    assert_eq!(signature.parameters[0].ty, f.ty("Int"));
    assert_eq!(signature.return_type, f.ty("Bool"));
}

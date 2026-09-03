use super::super::support::{expression_semantic_source as semantic_source, Fixture};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::TypeKnowledge;

fn assert_evaluation_result(method: &str, expected: &str) {
    let f = Fixture::new(&semantic_source());
    f.assert_no_errors();

    let run = f.callable("ExpressionSemanticProbe", method, DispatchSide::Class);
    let result = f.binding(run, "result");
    let result_ty = result.current.ty().expect("evaluation result type");
    f.assert_either(result_ty, f.ty("String"), f.ty(expected));
    assert!(matches!(result.current, TypeKnowledge::Known(_)), "result must remain known: {result:#?}");

    let call = f.expression_containing(run, "ExpressionEvaluation.eval(monad, expression)");
    let target = f.callable_id("ExpressionEvaluation", "eval", DispatchSide::Class);
    f.assert_expression_call(call, &target, result_ty);
    assert!(matches!(call.status, AnalysisStatus::Ready));
}

/// GEX-01/GEX-03: result-indexed GADT constructors propagate branch-local
/// equality through the outer HKT application without widening T.
#[test]
fn gadt_literal_branches_preserve_exact_effect_index() {
    assert_evaluation_result("intEvaluation", "Int");
    assert_evaluation_result("boolEvaluation", "Bool");
}

/// GEX-01: recursive evaluator branches for Add and If preserve their exact
/// result index while invoking the same generic evaluator.
#[test]
fn gadt_recursive_add_and_if_preserve_exact_effect_index() {
    assert_evaluation_result("addEvaluation", "Int");
    assert_evaluation_result("ifEvaluation", "Int");
}

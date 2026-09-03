use super::super::support::{expression_semantic_source as semantic_source, Fixture};
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::EvidenceStatus;
use phalcom_semantic::types::store::TypeData;

/// INT-01/02: existing MonadAlgorithms.traverse constructs
/// Expression<Either<String,_>,List<Int>>, then generic eval returns the exact
/// Either<String,List<Int>> without a parallel Expression algorithm.
#[test]
fn traverse_expression_then_evaluate_preserves_exact_intermediate_and_final_types() {
    let f = Fixture::new(&semantic_source());
    f.assert_no_errors();

    let run = f.callable("ExpressionIntegrationProbe", "traverseAndEvaluate", DispatchSide::Class);
    let expression = f.binding(run, "expression").current.ty().expect("intermediate expression type");
    let expression_args = f.assert_applied(expression, "Expression", 2);
    let effect = expression_args[0];
    assert!(matches!(f.analysis.snapshot.store.get(effect), TypeData::Lambda(_)));
    let list = f.assert_applied(expression_args[1], "List", 1);
    assert_eq!(list, [f.ty("Int")]);

    let traverse = f.expression_containing(run, "MonadAlgorithms.traverse(");
    let traverse_target = f.callable_id("MonadAlgorithms", "traverse", DispatchSide::Class);
    f.assert_expression_call(traverse, &traverse_target, expression);
    let traverse_constructor = f.callable_generic_parameter("MonadAlgorithms", "traverse", DispatchSide::Class, 0);
    let traverse_solution = f.generic_solution_type_for(run, traverse, traverse_constructor);
    f.assert_unary_constructor_kind(f.analysis.snapshot.store.kind_of(traverse_solution));
    f.assert_generic_solution_exact(run, traverse, traverse_constructor, traverse_solution, EvidenceStatus::Assumed);

    let result = f.binding(run, "result").current.ty().expect("final evaluation type");
    f.assert_either(result, f.ty("String"), expression_args[1]);
    let evaluate = f.expression_containing(run, "ExpressionEvaluation.eval(evaluationMonad, expression)");
    let evaluate_target = f.callable_id("ExpressionEvaluation", "eval", DispatchSide::Class);
    f.assert_expression_call(evaluate, &evaluate_target, result);
}

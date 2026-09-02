use super::support::{Fixture, semantic_source};
use phalcom_semantic::explain::GenericConstraintOrigin;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::store::TypeData;

/// MON-COMP-02/05: F must propagate through both callable return positions in
/// Kleisli composition and the proof graph must retain the argument evidence.
#[test]
fn kleisli_composition_preserves_higher_kinded_callable_shape_and_proof() {
    let f = Fixture::new(&semantic_source());
    f.assert_no_errors();

    let run = f.callable("MonadSemanticProbe", "kleisliEvidence", DispatchSide::Class);
    let composed = f.binding(run, "composed").current.ty().expect("composed callable type");
    let TypeData::Callable(signature) = f.analysis.snapshot.store.get(composed) else {
        panic!("expected callable, got {}", f.analysis.snapshot.store.format_type(composed))
    };
    assert_eq!(signature.parameters.len(), 1);
    assert_eq!(signature.parameters[0].ty, f.ty("String"));
    f.assert_either(signature.return_type, f.ty("String"), f.ty("Bool"));

    let call = f.expression_containing(run, "MonadAlgorithms.kleisli(monad, first, second)");
    f.assert_generic_solution(run, call, "A", f.ty("String"));
    f.assert_generic_solution(run, call, "B", f.ty("Int"));
    f.assert_generic_solution(run, call, "C", f.ty("Bool"));
    f.assert_generic_constraint_origin(
        run,
        call,
        "B",
        GenericConstraintOrigin::Argument { parameter_index: 1 },
    );
    f.assert_generic_constraint_origin(
        run,
        call,
        "C",
        GenericConstraintOrigin::Argument { parameter_index: 2 },
    );
}

/// MON-COMP-03/04/05: traverse reconciles Monad<F>, List<A>, and
/// (A) -> F<B>, then beta-reduces F<List<B>> to the concrete Either result.
#[test]
fn traverse_specializes_to_either_of_list_and_records_independent_evidence() {
    let f = Fixture::new(&semantic_source());
    f.assert_no_errors();

    let run = f.callable("MonadSemanticProbe", "traverseEvidence", DispatchSide::Class);
    let traversed = f.binding(run, "traversed").current.ty().expect("traversed type");
    let either_args = f.assert_applied(traversed, "Either", 2);
    assert_eq!(either_args[0], f.ty("String"));
    let list_args = f.assert_applied(either_args[1], "List", 1);
    assert_eq!(list_args[0], f.ty("Bool"));

    let call = f.expression_containing(run, "MonadAlgorithms.traverse(monad, values, transform)");
    f.assert_generic_solution(run, call, "A", f.ty("Int"));
    f.assert_generic_solution(run, call, "B", f.ty("Bool"));
    f.assert_generic_constraint_origin(
        run,
        call,
        "A",
        GenericConstraintOrigin::Argument { parameter_index: 1 },
    );
    f.assert_generic_constraint_origin(
        run,
        call,
        "B",
        GenericConstraintOrigin::Argument { parameter_index: 2 },
    );
}

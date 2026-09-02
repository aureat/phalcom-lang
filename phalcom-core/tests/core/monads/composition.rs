use super::support::{Fixture, semantic_source};
use phalcom_semantic::explain::{GenericConstraintOrigin, GenericConstraintRelation};
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::EvidenceStatus;
use phalcom_semantic::types::store::TypeData;

/// MON-COMP-01: a real generic sequence implementation consumes List<F<A>>
/// and returns F<List<A>> with constructor and element evidence preserved.
#[test]
fn sequence_specializes_nested_effects_to_either_of_list() {
    let f = Fixture::new(&semantic_source());
    f.assert_no_errors();

    let run = f.callable("MonadSemanticProbe", "sequenceEvidence", DispatchSide::Class);
    let sequenced = f.binding(run, "sequenced").current.ty().expect("sequenced type");
    let either_args = f.assert_applied(sequenced, "Either", 2);
    assert_eq!(either_args[0], f.ty("String"));
    let list_args = f.assert_applied(either_args[1], "List", 1);
    assert_eq!(list_args[0], f.ty("Int"));

    let call = f.expression_containing(run, "MonadAlgorithms.sequence(monad, values)");
    let target = f.callable_id("MonadAlgorithms", "sequence", DispatchSide::Class);
    f.assert_expression_call(call, &target, sequenced);

    let constructor_parameter = f.callable_generic_parameter("MonadAlgorithms", "sequence", DispatchSide::Class, 0);
    let a = f.callable_generic_parameter("MonadAlgorithms", "sequence", DispatchSide::Class, 1);
    let constructor = f.generic_solution_type_for(run, call, constructor_parameter);
    f.assert_unary_constructor_kind(f.analysis.snapshot.store.kind_of(constructor));
    f.assert_generic_solution_exact(run, call, constructor_parameter, constructor, EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(run, call, a, f.ty("Int"), EvidenceStatus::Assumed);

    let monad_ty = f.binding(run, "monad").current.ty().expect("monad type");
    let values_ty = f.binding(run, "values").current.ty().expect("values type");
    f.assert_generic_constraint_exact(
        run,
        call,
        constructor_parameter,
        GenericConstraintOrigin::Argument { parameter_index: 0 },
        GenericConstraintRelation::SupertypeOf(monad_ty),
    );
    f.assert_generic_constraint_exact(
        run,
        call,
        constructor_parameter,
        GenericConstraintOrigin::Argument { parameter_index: 1 },
        GenericConstraintRelation::SupertypeOf(values_ty),
    );
    f.assert_generic_constraint_exact(
        run,
        call,
        a,
        GenericConstraintOrigin::Argument { parameter_index: 1 },
        GenericConstraintRelation::SupertypeOf(values_ty),
    );
    assert!(f.generic_constraint_count(run, call, constructor_parameter) >= 2);
}

/// MON-COMP-02/05: F must propagate through both callable return positions in
/// Kleisli composition and independent constructor evidence must converge on
/// one canonical solution.
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
    let target = f.callable_id("MonadAlgorithms", "kleisli", DispatchSide::Class);
    f.assert_expression_call(call, &target, composed);

    let constructor_parameter = f.callable_generic_parameter("MonadAlgorithms", "kleisli", DispatchSide::Class, 0);
    let a = f.callable_generic_parameter("MonadAlgorithms", "kleisli", DispatchSide::Class, 1);
    let b = f.callable_generic_parameter("MonadAlgorithms", "kleisli", DispatchSide::Class, 2);
    let c = f.callable_generic_parameter("MonadAlgorithms", "kleisli", DispatchSide::Class, 3);
    let constructor = f.generic_solution_type_for(run, call, constructor_parameter);
    f.assert_unary_constructor_kind(f.analysis.snapshot.store.kind_of(constructor));
    f.assert_generic_solution_exact(run, call, constructor_parameter, constructor, EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(run, call, a, f.ty("String"), EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(run, call, b, f.ty("Int"), EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(run, call, c, f.ty("Bool"), EvidenceStatus::Assumed);

    let monad_ty = f.binding(run, "monad").current.ty().expect("monad type");
    let first_ty = f.binding(run, "first").current.ty().expect("first type");
    let second_ty = f.binding(run, "second").current.ty().expect("second type");
    f.assert_generic_constraint_exact(
        run,
        call,
        constructor_parameter,
        GenericConstraintOrigin::Argument { parameter_index: 0 },
        GenericConstraintRelation::SupertypeOf(monad_ty),
    );
    f.assert_generic_constraint_exact(
        run,
        call,
        constructor_parameter,
        GenericConstraintOrigin::Argument { parameter_index: 1 },
        GenericConstraintRelation::SupertypeOf(first_ty),
    );
    f.assert_generic_constraint_exact(
        run,
        call,
        constructor_parameter,
        GenericConstraintOrigin::Argument { parameter_index: 2 },
        GenericConstraintRelation::SupertypeOf(second_ty),
    );
    assert!(f.generic_constraint_count(run, call, constructor_parameter) >= 3);
}

/// MON-COMP-03/04/05: traverse reconciles Monad<F>, List<A>, and
/// (A) -> F<B>, beta-reduces F<List<B>>, and records both independent F evidence
/// paths instead of relying on a late Dynamic fallback.
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
    let target = f.callable_id("MonadAlgorithms", "traverse", DispatchSide::Class);
    f.assert_expression_call(call, &target, traversed);

    let constructor_parameter = f.callable_generic_parameter("MonadAlgorithms", "traverse", DispatchSide::Class, 0);
    let a = f.callable_generic_parameter("MonadAlgorithms", "traverse", DispatchSide::Class, 1);
    let b = f.callable_generic_parameter("MonadAlgorithms", "traverse", DispatchSide::Class, 2);
    let constructor = f.generic_solution_type_for(run, call, constructor_parameter);
    f.assert_unary_constructor_kind(f.analysis.snapshot.store.kind_of(constructor));
    f.assert_generic_solution_exact(run, call, constructor_parameter, constructor, EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(run, call, a, f.ty("Int"), EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(run, call, b, f.ty("Bool"), EvidenceStatus::Assumed);

    let monad_ty = f.binding(run, "monad").current.ty().expect("monad type");
    let values_ty = f.binding(run, "values").current.ty().expect("values type");
    let transform_ty = f.binding(run, "transform").current.ty().expect("transform type");
    f.assert_generic_constraint_exact(
        run,
        call,
        constructor_parameter,
        GenericConstraintOrigin::Argument { parameter_index: 0 },
        GenericConstraintRelation::SupertypeOf(monad_ty),
    );
    f.assert_generic_constraint_exact(
        run,
        call,
        constructor_parameter,
        GenericConstraintOrigin::Argument { parameter_index: 2 },
        GenericConstraintRelation::SupertypeOf(transform_ty),
    );
    f.assert_generic_constraint_exact(
        run,
        call,
        a,
        GenericConstraintOrigin::Argument { parameter_index: 1 },
        GenericConstraintRelation::SupertypeOf(values_ty),
    );
    f.assert_generic_constraint_exact(
        run,
        call,
        b,
        GenericConstraintOrigin::Argument { parameter_index: 2 },
        GenericConstraintRelation::SupertypeOf(transform_ty),
    );
    assert!(f.generic_constraint_count(run, call, constructor_parameter) >= 2);
}

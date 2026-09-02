use super::support::{Fixture, semantic_source};
use phalcom_semantic::explain::{GenericConstraintOrigin, GenericConstraintRelation};
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::EvidenceStatus;
use phalcom_semantic::types::store::TypeData;

/// MON-SOLVE-09: two F<X> arguments can establish one common partially-applied
/// constructor without a Monad<F> argument pre-solving F.
#[test]
fn two_partial_either_arguments_infer_one_shared_constructor() {
    let f = Fixture::new(&semantic_source());
    f.assert_no_errors();

    let run = f.callable("MonadSemanticProbe", "constructorAgreement", DispatchSide::Class);
    let agreed = f.binding(run, "agreed").current.ty().expect("agreed type");
    f.assert_either(agreed, f.ty("String"), f.ty("Bool"));

    let call = f.expression_containing(run, "MonadAlgorithms.sameConstructor(left, right)");
    let target = f.callable_id("MonadAlgorithms", "sameConstructor", DispatchSide::Class);
    f.assert_expression_call(call, &target, agreed);

    let constructor_parameter = f.callable_generic_parameter("MonadAlgorithms", "sameConstructor", DispatchSide::Class, 0);
    let a = f.callable_generic_parameter("MonadAlgorithms", "sameConstructor", DispatchSide::Class, 1);
    let b = f.callable_generic_parameter("MonadAlgorithms", "sameConstructor", DispatchSide::Class, 2);
    let constructor = f.generic_solution_type_for(run, call, constructor_parameter);
    f.assert_unary_constructor_kind(f.analysis.snapshot.store.kind_of(constructor));
    let TypeData::Lambda(lambda_id) = f.analysis.snapshot.store.get(constructor) else {
        panic!("shared partial Either constructor must be represented as a lambda")
    };
    let lambda = f.analysis.snapshot.store.arena().get_lambda(*lambda_id);
    let mut free = Vec::new();
    f.analysis.snapshot.store.arena().collect_free_types(lambda.body, &mut free);
    assert!(free.contains(&f.ty("String")));

    f.assert_generic_solution_exact(run, call, constructor_parameter, constructor, EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(run, call, a, f.ty("Int"), EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(run, call, b, f.ty("Bool"), EvidenceStatus::Assumed);

    let left_ty = f.binding(run, "left").current.ty().expect("left type");
    let right_ty = f.binding(run, "right").current.ty().expect("right type");
    f.assert_generic_constraint_exact(
        run,
        call,
        constructor_parameter,
        GenericConstraintOrigin::Argument { parameter_index: 0 },
        GenericConstraintRelation::SupertypeOf(left_ty),
    );
    f.assert_generic_constraint_exact(
        run,
        call,
        constructor_parameter,
        GenericConstraintOrigin::Argument { parameter_index: 1 },
        GenericConstraintRelation::SupertypeOf(right_ty),
    );
    assert!(f.generic_constraint_count(run, call, constructor_parameter) >= 2);

    let mut store = (*f.analysis.snapshot.store).clone();
    let applied = store.apply_type_form(constructor, &[f.ty("Bool")]).expect("shared F<Bool> must apply");
    let TypeData::Applied { origin, arguments } = store.get(applied) else {
        panic!("expected Either<String, Bool>, got {:?}", store.get(applied))
    };
    assert_eq!(*origin, f.ty("Either"));
    assert_eq!(arguments.as_ref(), [f.ty("String"), f.ty("Bool")]);
}

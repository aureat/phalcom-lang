use super::support::{Fixture, semantic_source, with_monads};
use phalcom_semantic::explain::{GenericConstraintOrigin, GenericConstraintRelation};
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::EvidenceStatus;
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::store::TypeData;

/// MON-SOLVE-01/02/03/05: constructor evidence from Monad<F>, F<A>, and
/// (A) -> F<B> must converge on one exact constructor while solving A and B.
#[test]
fn generic_bind_reconciles_all_constructor_evidence() {
    let f = Fixture::new(&semantic_source());
    f.assert_no_errors();

    let run = f.callable("MonadSemanticProbe", "algorithmBind", DispatchSide::Class);
    let bound = f.binding(run, "bound").current.ty().expect("bound type");
    f.assert_either(bound, f.ty("String"), f.ty("Bool"));

    let call = f.expression_containing(run, "MonadAlgorithms.bind(monad, source, next)");
    let target = f.callable_id("MonadAlgorithms", "bind", DispatchSide::Class);
    f.assert_expression_call(call, &target, bound);

    let constructor_parameter = f.callable_generic_parameter("MonadAlgorithms", "bind", DispatchSide::Class, 0);
    let a = f.callable_generic_parameter("MonadAlgorithms", "bind", DispatchSide::Class, 1);
    let b = f.callable_generic_parameter("MonadAlgorithms", "bind", DispatchSide::Class, 2);
    let constructor = f.generic_solution_type_for(run, call, constructor_parameter);

    assert_ne!(f.analysis.snapshot.store.kind_of(constructor), KindId::TYPE);
    f.assert_unary_constructor_kind(f.analysis.snapshot.store.kind_of(constructor));
    let TypeData::Lambda(lambda_id) = f.analysis.snapshot.store.get(constructor) else {
        panic!(
            "F must solve to a unary type lambda, got {}",
            f.analysis.snapshot.store.format_type(constructor)
        )
    };
    let lambda = f.analysis.snapshot.store.arena().get_lambda(*lambda_id);
    let mut free = Vec::new();
    f.analysis.snapshot.store.arena().collect_free_types(lambda.body, &mut free);
    assert!(free.contains(&f.ty("String")), "F must capture String: {free:#?}");

    f.assert_generic_solution_exact(run, call, constructor_parameter, constructor, EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(run, call, a, f.ty("Int"), EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(run, call, b, f.ty("Bool"), EvidenceStatus::Assumed);

    let monad_ty = f.binding(run, "monad").current.ty().expect("monad type");
    let source_ty = f.binding(run, "source").current.ty().expect("source type");
    let next_ty = f.binding(run, "next").current.ty().expect("next type");
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
        GenericConstraintRelation::SupertypeOf(source_ty),
    );
    f.assert_generic_constraint_exact(
        run,
        call,
        constructor_parameter,
        GenericConstraintOrigin::Argument { parameter_index: 2 },
        GenericConstraintRelation::SupertypeOf(next_ty),
    );
    assert!(
        f.generic_constraint_count(run, call, constructor_parameter) >= 3,
        "MON-SOLVE-05 requires independent F constraints from monad, value, and callable arguments"
    );

    f.assert_generic_constraint_exact(
        run,
        call,
        a,
        GenericConstraintOrigin::Argument { parameter_index: 1 },
        GenericConstraintRelation::SupertypeOf(source_ty),
    );
    f.assert_generic_constraint_exact(
        run,
        call,
        b,
        GenericConstraintOrigin::Argument { parameter_index: 2 },
        GenericConstraintRelation::SupertypeOf(next_ty),
    );
}

/// MON-SOLVE-04: F<A> nested beneath List must still contribute exact
/// constructor and element constraints.
#[test]
fn nested_list_of_f_a_contributes_constructor_and_element_evidence() {
    let f = Fixture::new(&semantic_source());
    f.assert_no_errors();

    let run = f.callable("MonadSemanticProbe", "nestedSequenceEvidence", DispatchSide::Class);
    let sequenced = f.binding(run, "sequenced").current.ty().expect("sequenced type");
    let arguments = f.assert_applied(sequenced, "Either", 2);
    assert_eq!(arguments[0], f.ty("String"));
    let list_args = f.assert_applied(arguments[1], "List", 1);
    assert_eq!(list_args[0], f.ty("Int"));

    let call = f.expression_containing(run, "MonadAlgorithms.sequenceSeed(monad, values, initial)");
    let target = f.callable_id("MonadAlgorithms", "sequenceSeed", DispatchSide::Class);
    f.assert_expression_call(call, &target, sequenced);

    let constructor_parameter = f.callable_generic_parameter("MonadAlgorithms", "sequenceSeed", DispatchSide::Class, 0);
    let a = f.callable_generic_parameter("MonadAlgorithms", "sequenceSeed", DispatchSide::Class, 1);
    let constructor = f.generic_solution_type_for(run, call, constructor_parameter);
    f.assert_unary_constructor_kind(f.analysis.snapshot.store.kind_of(constructor));
    f.assert_generic_solution_exact(run, call, a, f.ty("Int"), EvidenceStatus::Assumed);

    let values_ty = f.binding(run, "values").current.ty().expect("values type");
    f.assert_generic_constraint_exact(
        run,
        call,
        a,
        GenericConstraintOrigin::Argument { parameter_index: 1 },
        GenericConstraintRelation::SupertypeOf(values_ty),
    );
    assert!(f.generic_constraint_count(run, call, constructor_parameter) >= 3);
}

/// MON-SOLVE-06: direct higher-order decomposition must abstract the unary
/// constructor from an applied binary type, solve A, and preserve the fixed
/// left argument in the synthesized lambda.
#[test]
fn direct_f_a_constraint_synthesizes_exact_partial_either_constructor() {
    let source = with_monads(
        r#"
class ConstructorInferenceProbe {
    @class
    run(_ source: Either<String, Int>) {
        let result = MonadAlgorithms.constructorIdentity(source)
    }
}
"#,
    );
    let f = Fixture::new(&source);
    f.assert_no_errors();

    let run = f.callable("ConstructorInferenceProbe", "run", DispatchSide::Class);
    let result = f.binding(run, "result").current.ty().expect("result type");
    f.assert_either(result, f.ty("String"), f.ty("Int"));

    let call = f.expression_containing(run, "MonadAlgorithms.constructorIdentity(source)");
    let target = f.callable_id("MonadAlgorithms", "constructorIdentity", DispatchSide::Class);
    f.assert_expression_call(call, &target, result);

    let constructor_parameter = f.callable_generic_parameter("MonadAlgorithms", "constructorIdentity", DispatchSide::Class, 0);
    let a = f.callable_generic_parameter("MonadAlgorithms", "constructorIdentity", DispatchSide::Class, 1);
    let constructor = f.generic_solution_type_for(run, call, constructor_parameter);
    f.assert_unary_constructor_kind(f.analysis.snapshot.store.kind_of(constructor));
    let TypeData::Lambda(lambda_id) = f.analysis.snapshot.store.get(constructor) else {
        panic!("F must be synthesized as a unary lambda")
    };
    let lambda = f.analysis.snapshot.store.arena().get_lambda(*lambda_id);
    let mut free = Vec::new();
    f.analysis.snapshot.store.arena().collect_free_types(lambda.body, &mut free);
    assert!(free.contains(&f.ty("String")), "synthesized F must preserve fixed String argument: {free:#?}");
    assert!(
        f.analysis.snapshot.store.arena().has_free_bound(lambda.body, 0),
        "synthesized F must retain its bound argument"
    );

    f.assert_generic_solution_exact(run, call, constructor_parameter, constructor, EvidenceStatus::Assumed);
    f.assert_generic_solution_exact(run, call, a, f.ty("Int"), EvidenceStatus::Assumed);
    let source_ty = f.binding(run, "source").current.ty().expect("source type");
    f.assert_generic_constraint_exact(
        run,
        call,
        constructor_parameter,
        GenericConstraintOrigin::Argument { parameter_index: 0 },
        GenericConstraintRelation::SupertypeOf(source_ty),
    );
    f.assert_generic_constraint_exact(
        run,
        call,
        a,
        GenericConstraintOrigin::Argument { parameter_index: 0 },
        GenericConstraintRelation::SupertypeOf(source_ty),
    );

    let mut store = (*f.analysis.snapshot.store).clone();
    let applied = store.apply_type_form(constructor, &[f.ty("Bool")]).expect("synthesized F<Bool> must apply");
    let TypeData::Applied { origin, arguments } = store.get(applied) else {
        panic!("expected Either<String, Bool>, got {:?}", store.get(applied))
    };
    assert_eq!(*origin, f.ty("Either"));
    assert_eq!(arguments.as_ref(), [f.ty("String"), f.ty("Bool")]);
}

/// MON-SOLVE-07: the same direct constructor inference works for an ordinary
/// unary nominal constructor and does not unnecessarily synthesize a lambda.
#[test]
fn direct_f_a_constraint_recovers_nominal_box_constructor() {
    let source = with_monads(
        r#"
class NominalConstructorInferenceProbe {
    @class
    run(_ source: Box<Int>) {
        let result = MonadAlgorithms.constructorIdentity(source)
    }
}
"#,
    );
    let f = Fixture::new(&source);
    f.assert_no_errors();

    let run = f.callable("NominalConstructorInferenceProbe", "run", DispatchSide::Class);
    let result = f.binding(run, "result").current.ty().expect("result type");
    let args = f.assert_applied(result, "Box", 1);
    assert_eq!(args, [f.ty("Int")]);

    let call = f.expression_containing(run, "MonadAlgorithms.constructorIdentity(source)");
    let constructor_parameter = f.callable_generic_parameter("MonadAlgorithms", "constructorIdentity", DispatchSide::Class, 0);
    let a = f.callable_generic_parameter("MonadAlgorithms", "constructorIdentity", DispatchSide::Class, 1);
    let constructor = f.generic_solution_type_for(run, call, constructor_parameter);
    assert_eq!(constructor, f.ty("Box"));
    f.assert_generic_solution_exact(run, call, a, f.ty("Int"), EvidenceStatus::Assumed);
}

/// MON-SOLVE-08: constructor abstraction may capture a caller-owned generic
/// parameter without confusing it with the synthesized lambda binder.
#[test]
fn direct_constructor_abstraction_preserves_outer_generic_capture() {
    let source = with_monads(
        r#"
class CapturedConstructorInferenceProbe<E> {
    run(_ source: Either<E, Int>) {
        let result = MonadAlgorithms.constructorIdentity(source)
    }
}
"#,
    );
    let f = Fixture::new(&source);
    f.assert_no_errors();

    let run = f.callable("CapturedConstructorInferenceProbe", "run", DispatchSide::Instance);
    let call = f.expression_containing(run, "MonadAlgorithms.constructorIdentity(source)");
    let constructor_parameter = f.callable_generic_parameter("MonadAlgorithms", "constructorIdentity", DispatchSide::Class, 0);
    let constructor = f.generic_solution_type_for(run, call, constructor_parameter);
    let TypeData::Lambda(lambda_id) = f.analysis.snapshot.store.get(constructor) else {
        panic!("captured partial Either constructor must be represented as a lambda")
    };
    let lambda = f.analysis.snapshot.store.arena().get_lambda(*lambda_id);
    let mut free = Vec::new();
    f.analysis.snapshot.store.arena().collect_free_types(lambda.body, &mut free);
    assert!(free.contains(&f.type_parameter_form("CapturedConstructorInferenceProbe", 0)));
    assert!(f.analysis.snapshot.store.arena().has_free_bound(lambda.body, 0));
}

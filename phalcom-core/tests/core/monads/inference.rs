use super::support::{Fixture, semantic_source, with_monads};
use phalcom_semantic::diagnostic::DiagnosticSeverity;
use phalcom_semantic::explain::GenericConstraintOrigin;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::store::TypeData;

/// MON-SOLVE-01/02/03/05: constructor evidence from Monad<F>, F<A>, and
/// (A) -> F<B> must converge on one constructor while solving A and B.
#[test]
fn generic_bind_reconciles_receiver_value_and_callable_constructor_evidence() {
    let f = Fixture::new(&semantic_source());
    f.assert_no_errors();

    let run = f.callable("MonadSemanticProbe", "algorithmBind", DispatchSide::Class);
    let bound = f.binding(run, "bound").current.ty().expect("bound type");
    f.assert_either(bound, f.ty("String"), f.ty("Bool"));

    let call = f.expression_containing(run, "MonadAlgorithms.bind(monad, source, next)");
    let constructor = f.generic_solution_type(run, call, "F");
    assert_ne!(f.analysis.snapshot.store.kind_of(constructor), KindId::TYPE);
    f.assert_unary_constructor_kind(f.analysis.snapshot.store.kind_of(constructor));
    let TypeData::Lambda(lambda_id) = f.analysis.snapshot.store.get(constructor) else {
        panic!("F must solve to a unary type lambda, got {}", f.analysis.snapshot.store.format_type(constructor))
    };
    let lambda = f.analysis.snapshot.store.arena().get_lambda(*lambda_id);
    let mut free = Vec::new();
    f.analysis.snapshot.store.arena().collect_free_types(lambda.body, &mut free);
    assert!(free.contains(&f.ty("String")), "F must capture String: {free:#?}");

    f.assert_generic_solution(run, call, "A", f.ty("Int"));
    f.assert_generic_solution(run, call, "B", f.ty("Bool"));
    f.assert_generic_constraint_origin(
        run,
        call,
        "F",
        GenericConstraintOrigin::Argument { parameter_index: 0 },
    );
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

/// MON-SOLVE-04: F<A> nested beneath List must still contribute constraints.
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
    f.assert_generic_solution(run, call, "A", f.ty("Int"));
    f.assert_generic_constraint_origin(
        run,
        call,
        "A",
        GenericConstraintOrigin::Argument { parameter_index: 1 },
    );
}

/// MON-SOLVE-06: direct higher-order decomposition should abstract the common
/// unary constructor from an applied binary type rather than losing the fixed
/// left argument or inventing Dynamic.
#[test]
fn direct_f_a_constraint_synthesizes_partial_either_constructor() {
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

    let errors = f
        .analysis
        .snapshot
        .all_diagnostics()
        .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
        .collect::<Vec<_>>();
    assert!(errors.is_empty(), "constructor abstraction must be supported: {errors:#?}");

    let run = f.callable("ConstructorInferenceProbe", "run", DispatchSide::Class);
    let result = f.binding(run, "result").current.ty().expect("result type");
    f.assert_either(result, f.ty("String"), f.ty("Int"));

    let call = f.expression_containing(run, "MonadAlgorithms.constructorIdentity(source)");
    let constructor = f.generic_solution_type(run, call, "F");
    f.assert_unary_constructor_kind(f.analysis.snapshot.store.kind_of(constructor));
    assert!(matches!(f.analysis.snapshot.store.get(constructor), TypeData::Lambda(_)));
}

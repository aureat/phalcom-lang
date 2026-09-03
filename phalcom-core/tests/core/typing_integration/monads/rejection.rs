use super::super::support::{Fixture, with_monads};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::explain::ExplanationStep;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::{TypeKnowledge, UnknownReason};

/// MON-REJECT-01: constructor evidence fixed by Monad<F> conflicts with an
/// unrelated unary constructor value instead of degrading to Dynamic/Unknown.
#[test]
fn monad_constructor_conflicts_with_unrelated_value_constructor() {
    let source = with_monads(
        r#"
class ConstructorConflictProbe {
    @class
    run(
        _ monad: StringEitherMonad,
        _ value: Box<Int>,
        _ next: (Int) -> Box<Bool>
    ) {
        let bad = MonadAlgorithms.bind(monad, value, next)
    }
}
"#,
    );
    let f = Fixture::new(&source);
    f.assert_only_error_codes(&[DiagnosticCode::GenericInferenceConflict]);

    let run = f.callable("ConstructorConflictProbe", "run", DispatchSide::Class);
    let call = f.expression_containing(run, "MonadAlgorithms.bind(monad, value, next)");
    assert!(
        matches!(call.status, AnalysisStatus::Invalid(_)),
        "conflicting HKT call must be invalid: {call:#?}"
    );
    let trace = f.generic_trace(run, call);
    assert!(
        trace.iter().any(|node| matches!(&node.step, ExplanationStep::GenericConflict { .. })),
        "missing formal generic-conflict proof: {trace:#?}"
    );
}

/// MON-REJECT-02: two partially-applied Either constructors with different
/// fixed arguments cannot be unified as one F.
#[test]
fn differing_fixed_either_arguments_cannot_unify_as_one_constructor() {
    let source = with_monads(
        r#"
class ConstructorAgreement {
    @class
    same<F: Type -> Type, A, B>(
        _ left: F<A>,
        _ right: F<B>
    ) -> F<B> {
        right
    }
}

class FixedArgumentConflictProbe {
    @class
    run(
        _ left: Either<String, Int>,
        _ right: Either<Bool, Bool>
    ) {
        let bad = ConstructorAgreement.same(left, right)
    }
}
"#,
    );
    let f = Fixture::new(&source);
    f.assert_only_error_codes(&[DiagnosticCode::GenericInferenceConflict]);

    let run = f.callable("FixedArgumentConflictProbe", "run", DispatchSide::Class);
    let call = f.expression_containing(run, "ConstructorAgreement.same(left, right)");
    assert!(matches!(call.status, AnalysisStatus::Invalid(_)));
    let trace = f.generic_trace(run, call);
    assert!(
        trace.iter().any(|node| matches!(&node.step, ExplanationStep::GenericConflict { .. })),
        "missing constructor conflict proof: {trace:#?}"
    );
}

/// MON-REJECT-03: a genuinely unconstrained constructor parameter remains
/// underconstrained and is never fabricated as Dynamic.
#[test]
fn unconstrained_constructor_parameter_is_reported_not_invented() {
    let source = with_monads(
        r#"
class UnderconstrainedConstructor {
    @class
    fabricate<F: Type -> Type, A>(_ value: A) -> F<A> {
        throw Error.new("unreachable")
    }
}

class UnderconstrainedConstructorProbe {
    @class
    run() {
        let bad = UnderconstrainedConstructor.fabricate(42)
    }
}
"#,
    );
    let f = Fixture::new(&source);
    f.assert_only_error_codes(&[DiagnosticCode::GenericInferenceUnderconstrained]);

    let run = f.callable("UnderconstrainedConstructorProbe", "run", DispatchSide::Class);
    let binding = f.binding(run, "bad");
    assert!(
        matches!(binding.current, TypeKnowledge::Unknown(UnknownReason::UnderconstrainedTypeVariable)),
        "underconstrained F must remain formally unknown: {binding:#?}"
    );
    assert!(!binding.current.is_dynamic(), "underconstrained F must not become Dynamic");

    let call = f.expression_containing(run, "UnderconstrainedConstructor.fabricate(42)");
    assert!(
        matches!(call.status, AnalysisStatus::Blocked(_)),
        "underconstrained call should be blocked: {call:#?}"
    );
}

// MON-REJECT-04/05 are the kind-level rejection laws exercised in kinds.rs:
// a proper Type and a binary constructor cannot inhabit F: Type -> Type.

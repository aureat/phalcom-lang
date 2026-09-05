use super::super::support::{with_expression, Fixture};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::TypeKnowledge;

fn assert_rejected(extra: &str, owner: &str, needle: &str, accepted: &[DiagnosticCode]) {
    let source = with_expression(extra);
    let probe_start = source.len() - extra.len();
    let f = Fixture::new(&source);
    let run = f.callable(owner, "run", DispatchSide::Class);
    let expression = f.expression_containing(run, needle);
    assert!(
        matches!(expression.status, AnalysisStatus::Invalid(_) | AnalysisStatus::Blocked(_)),
        "hostile Expression path must be formally rejected: {expression:#?}"
    );
    assert!(!matches!(expression.knowledge, TypeKnowledge::Dynamic(_)), "hostile path escaped through Dynamic: {expression:#?}");
    assert!(
        f.analysis.snapshot.all_diagnostics().any(|diagnostic| {
            accepted.contains(&diagnostic.code) && diagnostic.primary_range.start >= probe_start
        }),
        "missing probe-local conflict/mismatch diagnostic; accepted={accepted:?}, all={:#?}",
        f.analysis.snapshot.all_diagnostics().collect::<Vec<_>>()
    );
}

/// GEX-REJECT-01: Add only accepts Int-indexed operands.
#[test]
fn add_rejects_bool_operand_without_dynamic_escape() {
    assert_rejected(
        r#"
class WrongAddProbe {
    @class
    run() {
        let bad: Expression<<X> =>> Either<String, X>, Int> = Expression::Add(
            Expression::BoolLiteral(true),
            Expression::IntLiteral(1)
        )
    }
}
"#,
        "WrongAddProbe",
        "Expression::Add(",
        &[DiagnosticCode::TypeMismatch, DiagnosticCode::GenericInferenceConflict, DiagnosticCode::BindingInitializerMismatch],
    );
}

/// GEX-REJECT-02: If requires a Bool-indexed condition.
#[test]
fn if_rejects_int_condition_without_dynamic_escape() {
    assert_rejected(
        r#"
class WrongIfConditionProbe {
    @class
    run() {
        let bad: Expression<<X> =>> Either<String, X>, Int> = Expression::If(
            Expression::IntLiteral(1),
            Expression::IntLiteral(10),
            Expression::IntLiteral(20)
        )
    }
}
"#,
        "WrongIfConditionProbe",
        "Expression::If(",
        &[DiagnosticCode::TypeMismatch, DiagnosticCode::GenericInferenceConflict, DiagnosticCode::BindingInitializerMismatch],
    );
}

/// GEX-REJECT-03: If branch indices must agree through one constructor-local A.
#[test]
fn if_rejects_incompatible_branch_indices_without_dynamic_escape() {
    assert_rejected(
        r#"
class WrongIfBranchProbe {
    @class
    run() {
        let bad: Expression<<X> =>> Either<String, X>, Int> = Expression::If(
            Expression::BoolLiteral(true),
            Expression::IntLiteral(10),
            Expression::Pure("twenty")
        )
    }
}
"#,
        "WrongIfBranchProbe",
        "Expression::If(",
        &[DiagnosticCode::TypeMismatch, DiagnosticCode::GenericInferenceConflict, DiagnosticCode::BindingInitializerMismatch],
    );
}

/// GEX-REJECT-04: Apply must agree on callable input A and argument index.
#[test]
fn apply_rejects_callable_argument_mismatch_without_dynamic_escape() {
    assert_rejected(
        r#"
class WrongApplyProbe {
    @class
    run() {
        let bad: Expression<<X> =>> Either<String, X>, Int> = Expression::Apply(
            Expression::Pure(|value: String| { value.size }),
            Expression::IntLiteral(42)
        )
    }
}
"#,
        "WrongApplyProbe",
        "Expression::Apply(",
        &[
            DiagnosticCode::TypeMismatch,
            DiagnosticCode::GenericInferenceConflict,
            DiagnosticCode::GenericConstraintUnsatisfied,
            DiagnosticCode::BindingInitializerMismatch,
        ],
    );
}

/// GEX-REJECT-05: FlatMap continuation input must match source index.
#[test]
fn flat_map_rejects_continuation_input_mismatch_without_dynamic_escape() {
    assert_rejected(
        r#"
class WrongFlatMapProbe {
    @class
    run() {
        let bad: Expression<<X> =>> Either<String, X>, Bool> = Expression::FlatMap(
            Expression::IntLiteral(42),
            |value: String| { Expression::Pure(value) }
        )
    }
}
"#,
        "WrongFlatMapProbe",
        "Expression::FlatMap(",
        &[
            DiagnosticCode::TypeMismatch,
            DiagnosticCode::GenericInferenceConflict,
            DiagnosticCode::GenericConstraintUnsatisfied,
            DiagnosticCode::BindingInitializerMismatch,
        ],
    );
}

/// INT-REJECT-01: an Either-indexed expression cannot be evaluated by a
/// BoxMonad merely because both constructors have unary kind.
#[test]
fn evaluator_rejects_effect_constructor_mismatch_without_dynamic_escape() {
    assert_rejected(
        r#"
class WrongEffectProbe {
    @class
    run(_ expression: Expression<<X> =>> Either<String, X>, Int>, _ box: BoxMonad) {
        let bad = ExpressionEvaluation.eval(box, expression)
    }
}
"#,
        "WrongEffectProbe",
        "ExpressionEvaluation.eval(box, expression)",
        &[DiagnosticCode::TypeMismatch, DiagnosticCode::GenericInferenceConflict, DiagnosticCode::GenericConstraintUnsatisfied],
    );
}

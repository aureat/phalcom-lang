use super::super::support::{with_rows, Fixture};
use phalcom_semantic::checker::analysis::{AnalysisStatus, ExpressionAnalysis};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::TypeKnowledge;

#[test]
fn compatible_repeated_remainder_is_one_canonical_row() {
    let fixture = Fixture::new(&with_rows(include_str!("../sources/rows/correlation.ph")));
    let run = fixture.callable("RowCorrelationProbe", "compatible", DispatchSide::Class);
    let result = fixture.binding(run, "result").current.ty().expect("correlated Record type");
    fixture.assert_closed_record(result, &[("id", fixture.ty("Int")), ("label", fixture.ty("String"))]);
    let call = fixture.expression_containing(run, "RowCalculus.sameRemainder(");
    fixture.assert_expression_call(call, &fixture.callable_id("RowCalculus", "sameRemainder", DispatchSide::Class), result);
    fixture.assert_record_row_parameter(fixture.callable_generic_parameter("RowCalculus", "sameRemainder", DispatchSide::Class, 0));
    assert!(matches!(call.status, AnalysisStatus::Ready));
    fixture.assert_no_errors();
}

#[test]
fn incompatible_repeated_remainders_fail_closed() {
    let fixture = Fixture::new(&with_rows(include_str!("../sources/rows/invalid/repeated_remainder_conflict.ph")));
    let run = fixture.callable("RowCorrelationConflictProbe", "incompatible", DispatchSide::Class);
    let call = fixture.expression_containing(run, "RowCalculus.sameRemainder(");
    assert_row_rejection(&fixture, call);
    assert_eq!(fixture.diagnostics(DiagnosticCode::RecordRowInferenceConflict).len(), 1);
}

pub(crate) fn assert_row_rejection(fixture: &Fixture, expression: &ExpressionAnalysis) {
    assert!(
        matches!(expression.status, AnalysisStatus::Invalid(_) | AnalysisStatus::Blocked(_))
            || (matches!(expression.status, AnalysisStatus::Ready) && expression.knowledge.is_unknown()),
        "row rejection must be invalid or formally blocked: {expression:#?}"
    );
    assert!(!matches!(expression.knowledge, TypeKnowledge::Dynamic(_)), "row rejection must not escape through Dynamic: {expression:#?}");
    let row_diagnostics = [
        DiagnosticCode::RecordRowInferenceConflict,
        DiagnosticCode::RecordRowLacksViolation,
        DiagnosticCode::GenericConstraintUnsatisfied,
    ];
    assert!(
        fixture.analysis.snapshot.all_diagnostics().any(|diagnostic| row_diagnostics.contains(&diagnostic.code)),
        "missing row rejection diagnostic: {:#?}",
        fixture.analysis.snapshot.all_diagnostics().collect::<Vec<_>>()
    );
}

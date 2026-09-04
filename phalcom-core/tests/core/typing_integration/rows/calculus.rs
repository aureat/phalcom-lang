use super::super::support::{row_calculus_source, with_rows, Fixture};
use phalcom_semantic::checker::analysis::{AnalysisStatus, BindingState};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::{EvidenceStatus, TypeKnowledge, UnknownReason};

fn probe_fixture() -> Fixture {
    Fixture::new(&row_calculus_source())
}

fn result<'a>(fixture: &'a Fixture, method: &str) -> (&'a phalcom_semantic::checker::analysis::CallableAnalysis, &'a BindingState, &'a phalcom_semantic::checker::analysis::ExpressionAnalysis) {
    let callable = fixture.callable("RowCalculusProbe", method, DispatchSide::Class);
    let binding = fixture.binding(callable, "result");
    let expression = fixture.expression_containing(callable, "RowCalculus.");
    (callable, binding, expression)
}

#[test]
fn preserve_infers_closed_remainder_exactly() {
    let fixture = probe_fixture();
    let (probe, binding, call) = result(&fixture, "preserveRemainder");
    let ty = binding.current.ty().expect("preserved Record type");
    fixture.assert_closed_record(
        ty,
        &[("name", fixture.ty("String")), ("stable", fixture.ty("Bool")), ("version", fixture.ty("Int"))],
    );
    fixture.assert_expression_call(call, &fixture.callable_id("RowCalculus", "preserve", DispatchSide::Class), ty);
    fixture.assert_record_row_parameter(fixture.callable_generic_parameter("RowCalculus", "preserve", DispatchSide::Class, 0));
    assert!(matches!(call.status, AnalysisStatus::Ready));
    assert!(matches!(binding.current, TypeKnowledge::Known(_)));
    fixture.assert_no_errors();
    let _ = probe;
}

#[test]
fn preserve_value_solves_ordinary_type_and_row_together() {
    let fixture = probe_fixture();
    let (probe, binding, call) = result(&fixture, "preserveValueAndRow");
    let ty = binding.current.ty().expect("preserved value Record type");
    fixture.assert_closed_record(
        ty,
        &[("cached", fixture.ty("Bool")), ("label", fixture.ty("String")), ("value", fixture.ty("Int"))],
    );
    fixture.assert_expression_call(call, &fixture.callable_id("RowCalculus", "preserveValue", DispatchSide::Class), ty);
    fixture.assert_record_row_parameter(fixture.callable_generic_parameter("RowCalculus", "preserveValue", DispatchSide::Class, 1));
    fixture.assert_generic_solution(probe, call, "T", fixture.ty("Int"));
    assert!(matches!(call.status, AnalysisStatus::Ready));
    fixture.assert_no_errors();
}

#[test]
fn annotate_solves_higher_order_a_b_and_row_together() {
    let fixture = probe_fixture();
    let (probe, binding, call) = result(&fixture, "annotateHigherOrder");
    let ty = binding.current.ty().expect("annotated Record type");
    fixture.assert_closed_record(
        ty,
        &[("cached", fixture.ty("Bool")), ("mapped", fixture.ty("Bool")), ("name", fixture.ty("String")), ("value", fixture.ty("Int"))],
    );
    fixture.assert_expression_call(call, &fixture.callable_id("RowCalculus", "annotate", DispatchSide::Class), ty);
    fixture.assert_generic_solution(probe, call, "A", fixture.ty("Int"));
    fixture.assert_generic_solution(probe, call, "B", fixture.ty("Bool"));
    fixture.assert_record_row_parameter(fixture.callable_generic_parameter("RowCalculus", "annotate", DispatchSide::Class, 2));
    assert!(matches!(call.status, AnalysisStatus::Ready));
    fixture.assert_no_errors();
}

#[test]
fn preserve_accepts_proven_empty_remainder() {
    let fixture = probe_fixture();
    let (probe, binding, call) = result(&fixture, "preserveEmptyRemainder");
    let ty = binding.current.ty().expect("closed one-field Record type");
    fixture.assert_closed_record(ty, &[("name", fixture.ty("String"))]);
    fixture.assert_expression_call(call, &fixture.callable_id("RowCalculus", "preserve", DispatchSide::Class), ty);
    assert!(matches!(call.status, AnalysisStatus::Ready));
    fixture.assert_no_errors();
    let _ = probe;
}

#[test]
fn result_only_row_remains_underconstrained_without_context() {
    let fixture = Fixture::new(&with_rows(include_str!("../sources/rows/invalid/underconstrained.ph")));
    let callable = fixture.callable("RowCalculusUnderconstrainedProbe", "resultOnlyUnderconstrained", DispatchSide::Class);
    let binding = fixture.binding(callable, "result");
    assert!(matches!(binding.current, TypeKnowledge::Unknown(UnknownReason::InferenceBlocked)), "{binding:#?}");
    let call = fixture.expression_containing(callable, "RowCalculus.make()");
    assert!(matches!(call.status, AnalysisStatus::Blocked(_)), "{call:#?}");
    assert!(call.knowledge.ty().is_none(), "underconstrained row must not publish a type: {call:#?}");
    assert!(!matches!(call.knowledge, TypeKnowledge::Dynamic(_)), "underconstrained row must not become Dynamic: {call:#?}");
    assert_eq!(fixture.diagnostics(DiagnosticCode::RecordRowInferenceUnderconstrained).len(), 1);
}

#[test]
fn expected_result_selects_underconstrained_row() {
    let fixture = probe_fixture();
    let (probe, binding, call) = result(&fixture, "expectedResultSelectsRow");
    let ty = binding.current.ty().expect("contextual Record type");
    fixture.assert_closed_record(
        ty,
        &[("label", fixture.ty("String")), ("value", fixture.ty("Int"))],
    );
    fixture.assert_expression_call(call, &fixture.callable_id("RowCalculus", "make", DispatchSide::Class), ty);
    assert!(matches!(call.status, AnalysisStatus::Ready));
    assert_eq!(call.knowledge.status(), Some(EvidenceStatus::Established));
    fixture.assert_no_errors();
    let _ = probe;
}

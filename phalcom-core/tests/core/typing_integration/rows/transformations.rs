use super::super::support::{Fixture, with_rows};
use super::correlation::assert_row_rejection;
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;

#[test]
fn tagged_adds_disjoint_field_and_preserves_remainder() {
    let fixture = Fixture::new(&with_rows(include_str!("../sources/rows/transformations.ph")));
    let run = fixture.callable("RowTransformationsProbe", "taggedPreservesRemainder", DispatchSide::Class);
    let result = fixture.binding(run, "result").current.ty().expect("tagged Record type");
    fixture.assert_closed_record(
        result,
        &[
            ("age", fixture.ty("Int")),
            ("enabled", fixture.ty("Bool")),
            ("name", fixture.ty("String")),
            ("tag", fixture.ty("String")),
        ],
    );
    let call = fixture.expression_containing(run, "RowCalculus.tagged(");
    fixture.assert_expression_call(call, &fixture.callable_id("RowCalculus", "tagged", DispatchSide::Class), result);
    assert!(matches!(call.status, AnalysisStatus::Ready));
    fixture.assert_no_errors();
}

#[test]
fn tagged_collision_is_rejected_without_dynamic_escape() {
    let fixture = Fixture::new(&with_rows(include_str!("../sources/rows/invalid/duplicate_extension.ph")));
    let run = fixture.callable("RowDuplicateExtensionProbe", "collides", DispatchSide::Class);
    let call = fixture.expression_containing(run, "RowCalculus.tagged(");
    assert_row_rejection(&fixture, call);
    assert_eq!(fixture.diagnostics(DiagnosticCode::RecordRowLacksViolation).len(), 1);
}

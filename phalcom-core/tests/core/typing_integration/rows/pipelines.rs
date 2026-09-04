use super::super::support::{with_rows, Fixture};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::identity::DispatchSide;

#[test]
fn each_pipeline_call_redecomposes_canonical_record() {
    let fixture = Fixture::new(&with_rows(include_str!("../sources/rows/pipelines.ph")));
    let run = fixture.callable("RowPipelineProbe", "redecompose", DispatchSide::Class);
    let tagged = fixture.binding(run, "tagged").current.ty().expect("first pipeline Record type");
    let result = fixture.binding(run, "result").current.ty().expect("second pipeline Record type");
    let expected = [
        ("age", fixture.ty("Int")),
        ("enabled", fixture.ty("Bool")),
        ("name", fixture.ty("String")),
        ("tag", fixture.ty("String")),
    ];
    fixture.assert_closed_record(tagged, &expected);
    fixture.assert_closed_record(result, &expected);

    let tagged_call = fixture.expression_containing(run, "RowCalculus.tagged(input)");
    let consume_call = fixture.expression_containing(run, "RowCalculus.consumeTagged(tagged)");
    fixture.assert_expression_call(tagged_call, &fixture.callable_id("RowCalculus", "tagged", DispatchSide::Class), tagged);
    fixture.assert_expression_call(consume_call, &fixture.callable_id("RowCalculus", "consumeTagged", DispatchSide::Class), result);
    assert!(matches!(tagged_call.status, AnalysisStatus::Ready));
    assert!(matches!(consume_call.status, AnalysisStatus::Ready));
    assert_eq!(tagged, result, "canonical Record result should be reusable across fresh decomposition");
    fixture.assert_no_errors();
}

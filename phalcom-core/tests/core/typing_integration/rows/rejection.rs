use super::super::support::{Fixture, with_rows};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::TypeKnowledge;

/// ROW-REJECT-03: nominal class shape does not satisfy Record structurally.
#[test]
fn nominal_class_does_not_satisfy_record() {
    let structural_source = include_str!("../sources/rows/structural_protocols.ph");
    let invalid_source = include_str!("../sources/rows/invalid/nominal_is_not_record.ph");
    let full_source = format!("{structural_source}\n{invalid_source}");
    let fixture = Fixture::new(&with_rows(&full_source));
    let run = fixture.callable("RowNominalInvalidProbe", "run", DispatchSide::Class);
    let call = fixture.expression_containing(run, "RowCapabilities.port");
    assert!(
        matches!(call.status, AnalysisStatus::Invalid(_) | AnalysisStatus::Blocked(_)),
        "nominal class cannot satisfy Record structurally: {call:#?}"
    );
    assert!(
        !matches!(call.knowledge, TypeKnowledge::Dynamic(_)),
        "must not escape to dynamic: {call:#?}"
    );
}

/// ROW-REJECT-04: Map key sets are not Record rows.
#[test]
fn map_does_not_satisfy_record() {
    let structural_source = include_str!("../sources/rows/structural_protocols.ph");
    let invalid_source = include_str!("../sources/rows/invalid/map_is_not_record.ph");
    let full_source = format!("{structural_source}\n{invalid_source}");
    let fixture = Fixture::new(&with_rows(&full_source));
    let run = fixture.callable("RowMapInvalidProbe", "run", DispatchSide::Class);
    let call = fixture.expression_containing(run, "RowCapabilities.port");
    assert!(
        matches!(call.status, AnalysisStatus::Invalid(_) | AnalysisStatus::Blocked(_)),
        "Map cannot satisfy Record structurally: {call:#?}"
    );
    assert!(
        !matches!(call.knowledge, TypeKnowledge::Dynamic(_)),
        "must not escape to dynamic: {call:#?}"
    );
}

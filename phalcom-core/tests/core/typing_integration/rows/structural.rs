use super::super::support::{row_structural_source, Fixture};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::TypeKnowledge;

fn probe_fixture() -> Fixture {
    Fixture::new(&row_structural_source())
}

/// ROW-REL-01: immutable width subtyping allows wider record to satisfy narrower prefix.
#[test]
fn width_subtyping_accepts_extra_fields() {
    let fixture = probe_fixture();
    let probe = fixture.callable("RowStructuralProbe", "widthOnly", DispatchSide::Class);
    let call = fixture.expression_containing(probe, "RowCapabilities.port");
    assert!(matches!(call.status, AnalysisStatus::Ready));
    assert_eq!(call.knowledge.ty(), Some(fixture.ty("Int")));
    fixture.assert_no_errors();
}

/// ROW-REL-02 & ROW-REL-03: width and covariant depth compose in capability pattern.
#[test]
fn width_and_covariant_depth_compose_in_capability_pattern() {
    let fixture = probe_fixture();
    let probe = fixture.callable("RowStructuralProbe", "widthAndDepth", DispatchSide::Class);

    let port_call = fixture.expression_containing(probe, "RowCapabilities.port");
    assert!(matches!(port_call.status, AnalysisStatus::Ready));
    assert_eq!(port_call.knowledge.ty(), Some(fixture.ty("Int")));

    let host_call = fixture.expression_containing(probe, "RowCapabilities.host");
    assert!(matches!(host_call.status, AnalysisStatus::Ready));
    assert_eq!(host_call.knowledge.ty(), Some(fixture.ty("String")));

    let port_binding = fixture.binding(probe, "port");
    assert!(matches!(port_binding.current, TypeKnowledge::Known(_)));
    assert_eq!(port_binding.current.ty(), Some(fixture.ty("Int")));

    let host_binding = fixture.binding(probe, "host");
    assert!(matches!(host_binding.current, TypeKnowledge::Known(_)));
    assert_eq!(host_binding.current.ty(), Some(fixture.ty("String")));

    fixture.assert_no_errors();
}

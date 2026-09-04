use super::super::support::{row_patterns_source, Fixture};
use phalcom_semantic::checker::analysis::CallableAnalysisStatus;
use phalcom_semantic::identity::DispatchSide;

/// ROW-PATTERN-01 & ROW-PATTERN-02: open record pattern binds known prefix without fabricating tail fields.
#[test]
fn open_record_pattern_binds_known_prefix_without_fabricating_tail_fields() {
    let fixture = Fixture::new(&row_patterns_source());
    let probe = fixture.callable("RowPatternProbe", "inspect", DispatchSide::Class);
    assert!(matches!(probe.status, CallableAnalysisStatus::Complete));

    let resolution = probe.match_resolutions.values().next().expect("record match resolution");
    assert_eq!(
        resolution.arms[0].bindings[0].knowledge.ty(),
        Some(fixture.ty("Int")),
        "known prefix field receives exact type"
    );
    assert_ne!(
        resolution.arms[1].bindings[0].knowledge.ty(),
        Some(fixture.ty("Int")),
        "missing tail field must not fabricate type"
    );
}

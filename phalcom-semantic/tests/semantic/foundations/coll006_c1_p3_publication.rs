//! Construction products commit only after their contextual owners accept.

use crate::semantic::support::Fixture;
use phalcom_semantic::DispatchSide;
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::types::evidence::EvidenceStatus;

#[test]
fn late_generic_failure_discards_dependent_sequence_and_preserves_independent_sibling() {
    let f = Fixture::new(
        r#"
class Consumer {
  @class reject<T>(_ values: Vector<T, 1>) -> Int where T == String { 0 }
  @class run() {
    let independent = <Int, 1>[9]
    let rejected = Consumer.reject(<Int, 1>[1])
  }
}
"#,
    );
    let run = f.callable("Consumer", "run", DispatchSide::Class);
    let call = f.expression(run, "Consumer.reject(<Int, 1>[1])");
    assert!(matches!(call.status, AnalysisStatus::Invalid(_)), "{call:#?}");
    assert_eq!(call.knowledge.ty(), Some(f.ty("Int")));
    assert_eq!(call.knowledge.status(), Some(EvidenceStatus::Established));
    let rejected = f.expression(run, "<Int, 1>[1]");
    let independent = f.expression(run, "<Int, 1>[9]");
    assert!(!run.sequence_literals.contains_key(&rejected.id), "rejected application retained executable argument facts");
    assert!(run.sequence_literals.contains_key(&independent.id), "independent sibling was discarded");
}

#[test]
fn successful_generic_and_partial_annotations_commit_sequences() {
    let f = Fixture::new(
        r#"
class Consumer {
  @class accept<T>(_ values: Vector<T, 1>) -> Int where T == Int { 0 }
  @class run() {
    let accepted = Consumer.accept(<Int, 1>[1])
    let canonical: Vector<Int, 1> = [2]
    let partial: Vector<_, _> = [3]
    let explicit = <_, 1>[4]
  }
}
"#,
    );
    f.assert_no_error_diagnostics();
    let run = f.callable("Consumer", "run", DispatchSide::Class);
    assert_eq!(run.sequence_literals.len(), 4);
    for source in ["<Int, 1>[1]", "[2]", "[3]", "<_, 1>[4]"] {
        assert!(run.sequence_literals.contains_key(&f.expression(run, source).id), "{source}");
    }
}

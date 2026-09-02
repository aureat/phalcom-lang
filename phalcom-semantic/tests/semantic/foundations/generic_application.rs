//! SC-2 generic application and expected-context laws.

use crate::semantic::support::Fixture;
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::{EvidenceOrigin, EvidenceStatus, UnknownReason};

#[test]
fn result_only_generic_uses_expected_context_as_assumed_selection() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  make<T>() -> T { 42 }

  @class
  run() {
    let value: Int = Probe.make()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.make()");
    assert_eq!(call.knowledge.ty(), Some(fixture.ty("Int")));
    assert_eq!(call.knowledge.status(), Some(EvidenceStatus::Assumed));
    assert_eq!(call.knowledge.origin(), Some(EvidenceOrigin::GenericInference));
    assert!(matches!(call.status, AnalysisStatus::Ready));
    fixture.assert_no_diagnostic(DiagnosticCode::GenericInferenceUnderconstrained);
}

#[test]
fn result_only_generic_without_context_remains_underconstrained() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  make<T>() -> T { 42 }

  @class
  run() {
    let value = Probe.make()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.make()");
    assert_eq!(
        call.knowledge,
        phalcom_semantic::types::evidence::TypeKnowledge::Unknown(UnknownReason::UnderconstrainedTypeVariable)
    );
    assert!(matches!(call.status, AnalysisStatus::Blocked(_)));
    fixture.assert_diagnostic(DiagnosticCode::GenericInferenceUnderconstrained, 1);
}

#[test]
#[ignore = "SC-2 Task 5: declaration upper-bound defaulting is not closed yet"]
fn declaration_upper_bound_does_not_default_result_only_generic() {
    let fixture = Fixture::new(
        r#"
class Number {}
class Probe {
  @class
  make<T>() -> T where T <: Number { 42 }

  @class
  run() {
    let value = Probe.make()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.make()");
    assert!(call.knowledge.ty().is_none());
    assert!(matches!(call.status, AnalysisStatus::Blocked(_)));
    fixture.assert_diagnostic(DiagnosticCode::GenericInferenceUnderconstrained, 1);
}

#[test]
fn candidate_is_checked_against_declaration_upper_bound() {
    let fixture = Fixture::new(
        r#"
class Number {}
class Allowed is Number { @constructor new() {} }
class Probe {
  @class
  keep<T>(_ value: T) -> T where T <: Number { value }

  @class
  run(_ value: Allowed) {
    let result = Probe.keep(value)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.keep(value)");
    assert_eq!(call.knowledge.ty(), Some(fixture.ty("Allowed")));
    assert!(matches!(call.status, AnalysisStatus::Ready));
}

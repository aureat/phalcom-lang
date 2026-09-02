//! SC-2 union-receiver call matrix.

use crate::semantic::support::Fixture;
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::explain::{ExplanationStep, UnionArmOutcome};
use phalcom_semantic::identity::DispatchSide;

#[test]
fn union_receiver_same_method_same_result_is_ready() {
    let fixture = Fixture::new(
        r#"
class Left {
  value() -> Int { 1 }
}
class Right {
  value() -> Int { 2 }
}
class Probe {
  @class
  run(_ value: Left | Right) {
    let result = value.value()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "value.value()");
    assert_eq!(call.knowledge.ty(), Some(fixture.ty("Int")));
    assert!(matches!(call.status, AnalysisStatus::Ready));
    fixture.assert_no_error_diagnostics();
}

#[test]
fn union_receiver_different_results_join() {
    let fixture = Fixture::new(
        r#"
class Left {
  value() -> Int { 1 }
}
class Right {
  value() -> String { "right" }
}
class Probe {
  @class
  run(_ value: Left | Right) {
    let result = value.value()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "value.value()");
    fixture.assert_union_members(call.knowledge.ty().expect("joined result"), &[fixture.ty("Int"), fixture.ty("String")]);
    assert!(matches!(call.status, AnalysisStatus::Ready));
    fixture.assert_no_error_diagnostics();
}

#[test]
fn union_receiver_missing_arm_is_invalid() {
    let fixture = Fixture::new(
        r#"
class Left {
  value() -> Int { 1 }
}
class Right {}
class Probe {
  @class
  run(_ value: Left | Right) {
    let result = value.value()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "value.value()");
    assert!(matches!(call.status, AnalysisStatus::Invalid(_)), "{call:#?}");
    fixture.assert_diagnostic(DiagnosticCode::TypeMismatch, 1);
    fixture.assert_trace_has(run, call, |step| {
        matches!(step, ExplanationStep::UnionArm { outcome: UnionArmOutcome::Missing { .. }, .. })
    });
}

#[test]
fn union_receiver_per_arm_generic_solutions_join() {
    let fixture = Fixture::new(
        r#"
class LeftResult<T> {}
class RightResult<T> {}
class Left {
  wrap<T>(_ value: T) -> LeftResult<T>
}
class Right {
  wrap<T>(_ value: T) -> RightResult<T>
}
class Probe {
  @class
  run(_ value: Left | Right) {
    let result = value.wrap(1)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "value.wrap(1)");
    let mut store = (*fixture.analysis.snapshot.store).clone();
    let left_result = store
        .apply_type_form(fixture.ty("LeftResult"), &[fixture.ty("Int")])
        .expect("left result");
    let right_result = store
        .apply_type_form(fixture.ty("RightResult"), &[fixture.ty("Int")])
        .expect("right result");
    fixture.assert_union_members(call.knowledge.ty().expect("joined generic result"), &[left_result, right_result]);
    assert!(matches!(call.status, AnalysisStatus::Ready));
    fixture.assert_no_error_diagnostics();
}

#[test]
fn union_receiver_contextual_closure_is_analyzed_once() {
    let fixture = Fixture::new(
        r#"
class Left {
  apply(_ f: (Int) -> Bool) -> Bool { true }
}
class Right {
  apply(_ f: (Int) -> Bool) -> Bool { false }
}
class Probe {
  @class
  run(_ value: Left | Right) {
    let result = value.apply(|x| { x == 1 })
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "value.apply(|x| { x == 1 })");
    assert_eq!(call.knowledge.ty(), Some(fixture.ty("Bool")));
    assert!(matches!(call.status, AnalysisStatus::Ready), "{call:#?}");
    assert_eq!(
        run.expressions
            .values()
            .filter(|expression| fixture.source.get(expression.range.start..expression.range.end) == Some("|x| { x == 1 }"))
            .count(),
        1
    );
    fixture.assert_no_error_diagnostics();
}

#[test]
fn union_receiver_incompatible_contextual_closure_is_explicit() {
    let fixture = Fixture::new(
        r#"
class Left {
  apply(_ f: (Int) -> Bool) -> Bool { true }
}
class Right {
  apply(_ f: (String) -> Bool) -> Bool { false }
}
class Probe {
  @class
  run(_ value: Left | Right) {
    let result = value.apply(|x| { x == 1 })
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "value.apply(|x| { x == 1 })");
    assert!(matches!(call.status, AnalysisStatus::Invalid(_)), "{call:#?}");
    fixture.assert_diagnostic(DiagnosticCode::TypeMismatch, 1);
    assert_eq!(
        run.expressions
            .values()
            .filter(|expression| fixture.source.get(expression.range.start..expression.range.end) == Some("|x| { x == 1 }"))
            .count(),
        1
    );
    fixture.assert_trace_has(run, call, |step| {
        matches!(step, ExplanationStep::UnionArm { outcome: UnionArmOutcome::ContextConflict, .. })
    });
}

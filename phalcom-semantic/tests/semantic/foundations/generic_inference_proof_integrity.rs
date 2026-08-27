use crate::semantic::support::Fixture;
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::{DynamicReason, EvidenceStatus, TypeKnowledge, UnknownReason};

#[test]
fn unresolved_required_generic_premise_prevents_known_dependent_result() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  first<T>(_ first: T, _ second: T) -> T {
    first
  }

  @class
  run() {
    let result = Probe.first(1, missing)
  }
}
"#,
    );

    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "Probe.first(1, missing)");

    assert_eq!(
        call.knowledge,
        TypeKnowledge::Unknown(UnknownReason::UnresolvedName("missing".into())),
        "a second required generic premise may not disappear merely because the first argument solves T",
    );
    assert!(matches!(call.status, AnalysisStatus::Blocked(_)), "{call:#?}");
}

#[test]
fn unresolved_generic_premise_blocks_call_without_erasing_fixed_return() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  fixed<T>(_ first: T, _ second: T) -> Int {
    1
  }

  @class
  run() {
    let result = Probe.fixed(1, missing)
  }
}
"#,
    );

    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "Probe.fixed(1, missing)");

    assert_eq!(call.knowledge.ty(), Some(int_ty));
    assert_eq!(call.knowledge.status(), Some(EvidenceStatus::Established));
    assert!(matches!(call.status, AnalysisStatus::Blocked(_)));
}

#[test]
fn expected_result_cannot_upgrade_unresolved_generic_premise() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  identity<T>(_ value: T) -> T {
    value
  }

  @class
  run() {
    let result: Int = Probe.identity(missing)
  }
}
"#,
    );

    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "Probe.identity(missing)");

    assert_eq!(call.knowledge, TypeKnowledge::Unknown(UnknownReason::UnresolvedName("missing".into())),);
}

#[test]
fn dynamic_generic_premise_produces_dynamic_dependent_result() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  identity<T>(_ value: T) -> T {
    value
  }

  @class
  run(value: Dynamic) {
    let result = Probe.identity(value)
  }
}
"#,
    );

    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "Probe.identity(value)");

    assert!(matches!(call.knowledge, TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape)), "{call:#?}");
    assert!(matches!(call.status, AnalysisStatus::DynamicBoundary(_)), "{call:#?}");
}

#[test]
fn dynamic_non_return_generic_premise_keeps_fixed_return_known() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  fixed<T>(_ value: T) -> Int {
    1
  }

  @class
  run(value: Dynamic) {
    let result = Probe.fixed(value)
  }
}
"#,
    );

    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "Probe.fixed(value)");
    assert_eq!(call.knowledge.ty(), Some(f.ty("Int")));
    assert_eq!(call.knowledge.status(), Some(EvidenceStatus::Established));
    assert!(matches!(call.status, AnalysisStatus::DynamicBoundary(_)), "{call:#?}");
}

#[test]
fn generic_known_argument_still_publishes_established_result() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  identity<T>(_ value: T) -> T { value }

  @class
  run() {
    let result = Probe.identity(1)
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "Probe.identity(1)");
    assert_eq!(call.knowledge.status(), Some(EvidenceStatus::Established));
    assert_eq!(call.knowledge.ty(), Some(f.ty("Int")));
}

#[test]
fn dependent_generic_conflict_does_not_publish_partial_specialization() {
    let f = Fixture::new(
        r#"
class Allowed {}
class Bad { @constructor new() {} }

class Probe {
  @class
  constrained<T>(_ value: T) -> T where T <: Allowed {
    value
  }

  @class
  constrainedFixed<T>(_ value: T) -> Int where T <: Allowed {
    1
  }

  @class
  run() {
    let dependent = Probe.constrained(Bad.new())
    let fixed = Probe.constrainedFixed(Bad.new())
  }
}
"#,
    );

    let run = f.callable("Probe", "run", DispatchSide::Class);
    let dependent = f.expression(run, "Probe.constrained(Bad.new())");
    assert!(matches!(dependent.status, AnalysisStatus::Invalid(_)), "{dependent:#?}");
    assert_eq!(dependent.knowledge, TypeKnowledge::Unknown(UnknownReason::InferenceConflict));

    let fixed = f.expression(run, "Probe.constrainedFixed(Bad.new())");
    assert!(matches!(fixed.status, AnalysisStatus::Invalid(_)), "{fixed:#?}");
    assert_eq!(fixed.knowledge.ty(), Some(f.ty("Int")));
    assert_eq!(fixed.knowledge.status(), Some(EvidenceStatus::Established));
}

#[test]
fn generic_argument_conflict_targets_failing_argument_range() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  constrained<T>(_ value: T) -> T where T == Int {
    value
  }

  @class
  run() {
    let result = Probe.constrained("bad")
  }
}
"#,
    );

    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "Probe.constrained(\"bad\")");
    assert!(matches!(call.status, AnalysisStatus::Invalid(_)), "{call:#?}");
    assert_eq!(call.knowledge, TypeKnowledge::Unknown(UnknownReason::InferenceConflict));

    let diagnostics = f.diagnostics(DiagnosticCode::ArgumentMismatch);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(diagnostics[0].primary_range, f.expression(run, "\"bad\"").range);
}

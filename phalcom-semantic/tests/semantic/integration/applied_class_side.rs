use crate::semantic::support::{Fixture, applied};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::{TypeKnowledge, UnknownReason};

#[test]
fn applied_class_side_members_and_constructors_retain_receiver_arguments() {
    let fixture = Fixture::new(
        r#"
class Box<T> {
  @class
  const _instances: List<Box<T>> = []

  @class
  instances -> List<Box<T>> { _instances }

  @constructor
  new(_ value: T) {}
}

class Probe {
  @class
  run() {
    let int_instances = Box<Int>.instances
    let string_instances = Box<String>.instances
    let inferred = Box.new(10)
    let explicit = Box<Int>.new(10)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);

    fixture.assert_type(
        fixture.binding(run, "int_instances").current.ty().expect("Box<Int> getter result"),
        applied("List", [applied("Box", [fixture.ty("Int").into()])]),
    );
    fixture.assert_type(
        fixture.binding(run, "string_instances").current.ty().expect("Box<String> getter result"),
        applied("List", [applied("Box", [fixture.ty("String").into()])]),
    );
    fixture.assert_type(
        fixture.binding(run, "inferred").current.ty().expect("inferred constructor result"),
        applied("Box", [fixture.ty("Int").into()]),
    );
    fixture.assert_type(
        fixture.binding(run, "explicit").current.ty().expect("explicit constructor result"),
        applied("Box", [fixture.ty("Int").into()]),
    );

    let inferred_call = fixture.expression(run, "Box.new(10)");
    let explicit_call = fixture.expression(run, "Box<Int>.new(10)");
    assert!(
        run.associated_resolutions.get(&inferred_call.id).is_none(),
        "ordinary class-side calls are not associated lookups"
    );
    assert!(
        run.associated_resolutions.get(&explicit_call.id).is_none(),
        "ordinary class-side calls are not associated lookups"
    );
    assert!(matches!(inferred_call.status, AnalysisStatus::Ready), "{inferred_call:#?}");
    fixture.assert_no_diagnostic(DiagnosticCode::GenericInferenceConflict);
}

#[test]
fn raw_generic_class_side_member_is_underconstrained() {
    let fixture = Fixture::new(
        r#"
class Box<T> {
  @class
  instances -> List<Box<T>> { 0 }
}

class Probe {
  @class
  run() {
    let raw = Box.instances
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let raw = fixture.expression(run, "Box.instances");
    assert_eq!(raw.knowledge, TypeKnowledge::Unknown(UnknownReason::UnderconstrainedTypeVariable));
    assert!(!fixture.diagnostics(DiagnosticCode::GenericInferenceUnderconstrained).is_empty());
}

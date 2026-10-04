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
  _instances: List<Box<T>> = []

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

    let int_instances = applied("List", [applied("Box", [fixture.ty("Int").into()])]);
    let string_instances = applied("List", [applied("Box", [fixture.ty("String").into()])]);
    let boxed_int = applied("Box", [fixture.ty("Int").into()]);

    fixture.assert_type(
        fixture.expression(run, "Box<Int>.instances").knowledge.ty().expect("Box<Int> getter result"),
        int_instances,
    );
    fixture.assert_type(
        fixture
            .expression(run, "Box<String>.instances")
            .knowledge
            .ty()
            .expect("Box<String> getter result"),
        string_instances,
    );
    fixture.assert_type(
        fixture.expression(run, "Box.new(10)").knowledge.ty().expect("inferred constructor result"),
        boxed_int.clone(),
    );
    fixture.assert_type(
        fixture.expression(run, "Box<Int>.new(10)").knowledge.ty().expect("explicit constructor result"),
        boxed_int.clone(),
    );

    for (expression_text, expected_ty) in [
        ("Box<Int>.instances", fixture.ty("Int")),
        ("Box<String>.instances", fixture.ty("String")),
        ("Box.new(10)", fixture.ty("Int")),
        ("Box<Int>.new(10)", fixture.ty("Int")),
    ] {
        let expression = fixture.expression(run, expression_text);
        fixture.assert_expression_ready(expression);
        let specialization = expression.call_specialization.as_ref().expect("class-side call specialization");
        assert_eq!(specialization.bindings.len(), 1, "{expression_text}: {specialization:#?}");
        assert_eq!(specialization.bindings[0].1, expected_ty, "{expression_text}: {specialization:#?}");
    }

    assert!(
        fixture.binding(run, "int_instances").current.is_unknown(),
        "opaque calls intentionally invalidate mutable local current knowledge"
    );
    assert!(
        fixture.binding(run, "string_instances").current.is_unknown(),
        "opaque calls intentionally invalidate mutable local current knowledge"
    );
    assert!(
        fixture.binding(run, "inferred").current.is_unknown(),
        "opaque calls intentionally invalidate mutable local current knowledge"
    );
    fixture.assert_type(fixture.binding(run, "explicit").current.ty().expect("final constructor result"), boxed_int);

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

#[test]
fn native_generic_constructor_uses_contextual_result_type() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let ints: Set<Int> = Set.new()
    let strings: Set<String> = Set.new()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);

    fixture.assert_type(
        fixture.binding(run, "ints").current.ty().expect("contextual Set<Int> result"),
        applied("Set", [fixture.ty("Int").into()]),
    );
    fixture.assert_type(
        fixture.binding(run, "strings").current.ty().expect("contextual Set<String> result"),
        applied("Set", [fixture.ty("String").into()]),
    );
    fixture.assert_no_diagnostic(DiagnosticCode::GenericInferenceConflict);
}

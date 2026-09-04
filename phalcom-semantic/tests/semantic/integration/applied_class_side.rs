use crate::semantic::support::{Fixture, applied};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::checker::associated::AssociatedResolutionKind;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{DispatchSide, InvocationTargetId};
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
    let inferred_resolution = run
        .associated_resolutions
        .get(&inferred_call.id)
        .expect("inferred class-side invocation resolution");
    let explicit_resolution = run
        .associated_resolutions
        .get(&explicit_call.id)
        .expect("explicit class-side invocation resolution");
    let int_get = fixture.expression(run, "Box<Int>.instances");
    let string_get = fixture.expression(run, "Box<String>.instances");
    let invocation = |resolution: &phalcom_semantic::checker::associated::AssociatedResolution| {
        let AssociatedResolutionKind::BoundBehavioralInvoke { target: InvocationTargetId::Behavioral(callable), .. } = &resolution.kind else {
            panic!("expected bound behavioral invocation, got {:?}", resolution.kind);
        };
        (callable.clone(), resolution.owner_form)
    };
    let (inferred_target, inferred_owner) = invocation(inferred_resolution);
    let (explicit_target, explicit_owner) = invocation(explicit_resolution);
    assert_eq!(inferred_target, explicit_target, "constructor callable identity must be shared");
    assert_eq!(inferred_owner, explicit_owner, "inferred and explicit Int construction must converge");
    assert_eq!(explicit_owner, fixture.binding(run, "explicit").current.ty().expect("explicit receiver type"));
    let (_, int_get_owner) = invocation(run.associated_resolutions.get(&int_get.id).expect("Int getter resolution"));
    let (_, string_get_owner) = invocation(run.associated_resolutions.get(&string_get.id).expect("String getter resolution"));
    assert_ne!(int_get_owner, string_get_owner, "applied class-side receiver applications remain distinct");
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

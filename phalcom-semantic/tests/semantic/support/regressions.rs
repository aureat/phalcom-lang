use super::{Fixture, tuple, union};
use phalcom_common::range::SourceRange;
use phalcom_semantic::checker::{CheckingContext, InternalFailurePolicy, InternalSemanticIncidentDetails, InternalSemanticIncidentKind};
use phalcom_semantic::declarations::bootstrap_universe_declarations;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::annotation::SimpleTypeResolver;
use phalcom_semantic::types::relation::MapTypeHierarchy;
use phalcom_semantic::types::store::TypeStore;

fn fail_fast_context() -> CheckingContext<'static> {
    let module = phalcom_semantic::identity::ModuleId::universe_root();
    let store = Box::leak(Box::new(TypeStore::new()));
    let declarations = Box::leak(Box::new(bootstrap_universe_declarations(
        store,
        &phalcom_semantic::core_surface::universe_declaration,
    )));
    let resolver = Box::leak(Box::new(SimpleTypeResolver::new()));
    let hierarchy = Box::leak(Box::new(MapTypeHierarchy::new()));
    CheckingContext::new(store, hierarchy, resolver, declarations, module)
}

#[test]
fn fail_fast_policy_panics_only_after_recording_incident() {
    let mut ctx = fail_fast_context();
    let before = ctx.analysis_incidents.len();
    let incident = ctx.record_internal_incident(
        InternalSemanticIncidentKind::RelationInvariantViolation,
        InternalSemanticIncidentDetails::Message {
            message: "contained test".into(),
        },
        None,
    );

    assert_eq!(before, 0);
    assert_eq!(incident.0, 0);
    assert_eq!(ctx.analysis_incidents.len(), 1);
    assert!(matches!(
        ctx.terminal_status,
        Some(phalcom_semantic::checker::analysis::AnalysisStatus::InternalFailure(_))
    ));

    let child = std::process::Command::new(std::env::current_exe().expect("semantic test executable"))
        .args([
            "--exact",
            "semantic::support::regressions::fail_fast_policy_panics_only_after_recording_incident_child",
            "--ignored",
            "--nocapture",
        ])
        .output()
        .expect("spawn fail-fast semantic child test");
    assert!(!child.status.success(), "fail-fast child unexpectedly completed successfully");
    let stderr = String::from_utf8_lossy(&child.stderr);
    assert!(stderr.contains("INTERNAL SEMANTIC INVARIANT FAILURE"), "child stderr: {stderr}");
    assert!(stderr.contains("RelationInvariantViolation"), "child stderr: {stderr}");
}

#[test]
#[ignore = "spawned by fail_fast_policy_panics_only_after_recording_incident"]
fn fail_fast_policy_panics_only_after_recording_incident_child() {
    let module = phalcom_semantic::identity::ModuleId::universe_root();
    let mut store = TypeStore::new();
    let declarations = bootstrap_universe_declarations(&mut store, &phalcom_semantic::core_surface::universe_declaration);
    let resolver = SimpleTypeResolver::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module);
    ctx.set_internal_failure_policy(InternalFailurePolicy::FailFast);

    ctx.record_internal_incident(
        InternalSemanticIncidentKind::RelationInvariantViolation,
        InternalSemanticIncidentDetails::Message { message: "test".into() },
        None,
    );
}

/// The assertion DSL is part of the semantic oracle. Complex expectations
/// inside unions must be matched structurally rather than acting as wildcards.
#[test]
fn union_expectation_rejects_wrong_structural_members() {
    let f = Fixture::new(
        r#"
class Cat { @constructor new() {} }
class Dog { @constructor new() {} }
class Probe {
  @class
  run(_ flag: Bool) {
    let value = if flag { Cat.new() } else { Dog.new() }
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let actual = f.binding(run, "value").current.ty().expect("branch union");

    let rejected = f.check_type(actual, union([tuple([int_ty.into()]), tuple([string_ty.into()])]));

    assert!(rejected.is_err(), "wrong structural union members were accepted by the test oracle");
}

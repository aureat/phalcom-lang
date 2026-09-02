use phalcom_modules::{SourceId, SourceLocation, SourceRevision, WorkspaceSourceBatchMutation};
use phalcom_semantic::{DiagnosticCode, SemanticWorkspaceSession};
use std::sync::Arc;

fn analyze(source: &str) -> Arc<phalcom_semantic::SemanticSnapshot> {
    let location = SourceLocation {
        source_id: SourceId("/tmp/phalcom-option-typing.ph".into()),
        display_path: "/tmp/phalcom-option-typing.ph".into(),
    };
    let mut session = SemanticWorkspaceSession::new();
    session
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
            source: location,
            text: Arc::from(source),
            revision: SourceRevision(1),
            recovered_program: None,
        }])
        .expect("Option typing workspace publication")
        .snapshot
}

#[test]
fn option_map_specializes_result_from_callable_return() {
    let snapshot = analyze(
        r#"
class Probe {
  @class
  run() {
    let x: Option<Int> = Option::Some(1)
    let y: Option<String> = x.map(|value| { value.toString })
  }
}
"#,
    );

    assert!(!snapshot.has_errors(), "diagnostics: {:#?}", snapshot.diagnostics);
}

#[test]
fn option_flat_map_specializes_nested_option_result() {
    let snapshot = analyze(
        r#"
class Probe {
  @class
  run() {
    let x: Option<Int> = Option::Some(1)
    let y: Option<String> = x.flatMap(|value| { Option::Some(value.toString) })
  }
}
"#,
    );

    assert!(!snapshot.has_errors(), "diagnostics: {:#?}", snapshot.diagnostics);
}

#[test]
fn option_unwrap_or_rejects_default_of_unrelated_type() {
    let snapshot = analyze(
        r#"
class Probe {
  @class
  run() {
    let x: Option<Int> = Option::Some(1)
    let y: String = x.unwrapOr("missing")
  }
}
"#,
    );

    assert!(
        snapshot
            .all_diagnostics()
            .any(|diagnostic| matches!(diagnostic.code, DiagnosticCode::ArgumentMismatch | DiagnosticCode::BindingInitializerMismatch)),
        "unwrapOr must reject a default that is not T; diagnostics: {:#?}",
        snapshot.diagnostics
    );
}

#[test]
fn option_ok_or_rejects_error_argument_incompatible_with_contextual_result() {
    let snapshot = analyze(
        r#"
class Probe {
  @class
  run() {
    let x: Option<Int> = Option::Some(1)
    let y: Result<Int, String> = x.okOr(42)
  }
}
"#,
    );

    assert!(
        snapshot
            .all_diagnostics()
            .any(|diagnostic| matches!(diagnostic.code, DiagnosticCode::ArgumentMismatch | DiagnosticCode::BindingInitializerMismatch)),
        "okOr must relate the error argument to E; diagnostics: {:#?}",
        snapshot.diagnostics
    );
}

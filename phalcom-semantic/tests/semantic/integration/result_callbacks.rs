use phalcom_modules::{SourceId, SourceLocation, SourceRevision, WorkspaceSourceBatchMutation};
use phalcom_semantic::{DiagnosticCode, SemanticWorkspaceSession};
use std::sync::Arc;

fn analyze(source: &str) -> Arc<phalcom_semantic::SemanticSnapshot> {
    let location = SourceLocation {
        source_id: SourceId("/tmp/phalcom-result-callbacks.ph".into()),
        display_path: "/tmp/phalcom-result-callbacks.ph".into(),
    };
    let mut session = SemanticWorkspaceSession::new();
    session
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
            source: location,
            text: Arc::from(source),
            revision: SourceRevision(1),
            recovered_program: None,
        }])
        .expect("Result callback workspace publication")
        .snapshot
}

#[test]
fn result_callback_apis_accept_positional_closures() {
    let snapshot = analyze(
        r#"
class Probe {
  @class
  run() {
    let ok: Result<Int, String> = Result<Int, String>::Ok(5)
    let error: Result<Int, String> = Result<Int, String>::Error("bad")

    let matched: String = ok.match(
      ok: |value| { value.toString },
      err: |message| { message }
    )
    let mapped: Result<String, String> = ok.map(|value| { value.toString })
    let mappedError: Result<Int, String> = error.mapErr(|message| { message + "!" })
    let mappedBoth: Result<String, Int> = error.mapBoth(
      ok: |value| { value.toString },
      err: |message| { message.size }
    )
    let chained: Result<String, String> = ok.andThen(|value| { Result<String, String>::Ok(value.toString) })
    let recovered: Result<Int, Int> = error.orElse(|message| { Result<Int, Int>::Ok(message.size) })
    let isOk: Bool = ok.isOkAnd(|value| { value > 0 })
    let isError: Bool = error.isErrAnd(|message| { message == "bad" })
    ok.inspect(|value| { let ignored = value })
    error.inspectErr(|message| { let ignored = message })
    let unwrapped: Int = error.unwrapOrElse(|message| { message.size })
  }
}
"#,
    );

    assert!(!snapshot.has_errors(), "diagnostics: {:#?}", snapshot.diagnostics);
}

#[test]
fn callable_shape_validation_still_rejects_label_and_arity_mismatches() {
    let snapshot = analyze(
        r#"
class Probe {
  labeled(_ callback: (Int) -> String) -> String { callback.call(value: 1) }
  missing(_ callback: (Int) -> String) -> String { callback.call() }
  extra(_ callback: () -> String) -> String { callback.call(1) }
}
"#,
    );

    assert!(
        snapshot.all_diagnostics().any(|diagnostic| matches!(diagnostic.code, DiagnosticCode::CallShapeMismatch)),
        "callable labels and arity must remain exact; diagnostics: {:#?}",
        snapshot.diagnostics
    );
}

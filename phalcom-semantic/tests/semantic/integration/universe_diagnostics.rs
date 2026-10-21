use phalcom_modules::{SourceId, SourceLocation, SourceRevision, WorkspaceSourceBatchMutation};
use phalcom_semantic::{DiagnosticCode, SemanticWorkspaceSession};
use std::sync::Arc;

fn analyze(source: &str) -> Arc<phalcom_semantic::SemanticSnapshot> {
    let location = SourceLocation {
        source_id: SourceId("/tmp/phalcom-universe-diagnostics.ph".into()),
        display_path: "/tmp/phalcom-universe-diagnostics.ph".into(),
    };
    let mut session = SemanticWorkspaceSession::new();
    session
        .apply_module_mutations([WorkspaceSourceBatchMutation::SetOverlay {
            source: location,
            text: Arc::from(source),
            revision: SourceRevision(1),
            recovered_program: None,
        }])
        .expect("Universe diagnostic regression workspace publication")
        .snapshot
}

#[test]
fn universe_tail_while_true_without_normal_exit_does_not_return_unit() {
    let snapshot = analyze(
        r#"
class Probe {
  run() -> Int {
    while true {
      return 1
    }
  }
}
"#,
    );

    assert!(
        !snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::ReturnMismatch),
        "non-fallthrough while true must not be checked as an implicit Unit return; diagnostics: {:#?}",
        snapshot.diagnostics
    );
}

#[test]
fn universe_tail_while_true_with_break_still_checks_normal_completion() {
    let snapshot = analyze(
        r#"
class Probe {
  run() -> Int {
    while true {
      break
    }
  }
}
"#,
    );

    assert!(
        snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::ReturnMismatch),
        "a reachable break makes the loop complete normally, so the Unit tail must still be rejected; diagnostics: {:#?}",
        snapshot.diagnostics
    );
}

#[test]
fn universe_equality_constrained_constructors_accept_specialized_owners() {
    let snapshot = analyze(
        r#"
data ProbeArrow<A, B> private(_ run: (A) -> B)
impl<A, B> ProbeArrow<A, B> {
  @class
  new(_ run: (A) -> B) -> ProbeArrow<A, B> { ProbeArrow<A, B>(run) }

  @class
  identity() -> ProbeArrow<A, B> where A == B {
    ProbeArrow<A, A>.new(|value| { value })
  }
}

data ProbeReader<R, T> private(_ read: (R) -> T)
impl<R, T> ProbeReader<R, T> {
  @class
  new(_ read: (R) -> T) -> ProbeReader<R, T> { ProbeReader<R, T>(read) }

  @class
  ask() -> ProbeReader<R, T> where T == R {
    ProbeReader<R, R>.new(|environment| { environment })
  }
}

data ProbeStateStep<S, T>(state: S, value: T)
data ProbeState<S, T> private(_ run: (S) -> ProbeStateStep<S, T>)
impl<S, T> ProbeState<S, T> {
  @class
  new(_ run: (S) -> ProbeStateStep<S, T>) -> ProbeState<S, T> {
    ProbeState<S, T>(run)
  }

  @class
  get() -> ProbeState<S, T> where T == S {
    ProbeState<S, S>.new(|state| {
      ProbeStateStep<S, S>(state: state, value: state)
    })
  }

  @class
  put(_ replacement: S) -> ProbeState<S, T> where T == Unit {
    ProbeState<S, Unit>.new(|_| {
      ProbeStateStep<S, Unit>(state: replacement, value: ())
    })
  }
}

data ProbeWriter<W, T>(value: T, log: W)
impl<W, T> ProbeWriter<W, T> {
  @class
  tell(_ log: W) -> ProbeWriter<W, T> where T == Unit {
    ProbeWriter<W, Unit>(value: (), log: log)
  }
}
"#,
    );

    assert!(
        !snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::GenericInferenceConflict),
        "equality-specialized constructors must not create generic inference conflicts; diagnostics: {:#?}",
        snapshot.diagnostics
    );
    assert!(!snapshot.has_errors(), "diagnostics: {:#?}", snapshot.diagnostics);
}

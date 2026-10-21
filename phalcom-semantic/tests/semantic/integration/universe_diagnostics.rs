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


#[test]
fn universe_iterable_associated_contract_accepts_matching_iterator_evidence() {
    let snapshot = analyze(
        r#"
trait ProbeIterator {
  type Item
  next -> Option<Self::Item>
}

trait ProbeIterable
  where
    Self::Iterator |= ProbeIterator,
    <Self::Iterator as ProbeIterator>::Item == Self::Item
{
  type Item
  type Iterator
  iterator -> Self::Iterator
}

data ProbeChainIterator<T>()
impl<T> ProbeChainIterator<T> {
  next -> Option<T> { Option<T>::None }
}

impl<T> ProbeIterator for ProbeChainIterator<T> {
  type Item = T
}

data ProbeChain<T>()
impl<T> ProbeIterable for ProbeChain<T> {
  type Item = T
  type Iterator = ProbeChainIterator<T>

  iterator -> ProbeChainIterator<T> { ProbeChainIterator<T>() }
}
"#,
    );

    assert!(
        !snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::ImplConformanceIncomplete),
        "matching associated iterator evidence must satisfy the Iterable-style trait contract; diagnostics: {:#?}",
        snapshot.diagnostics
    );
    assert!(!snapshot.has_errors(), "diagnostics: {:#?}", snapshot.diagnostics);
}


#[test]
fn constructor_kinded_trait_arguments_are_accepted_in_conformance_heads() {
    let snapshot = analyze(
        r#"
trait ProbeConstructorWitness<F: Type -> Type> {}

data ProbeBox<T>(_ value: T)
class ProbeBoxContext {}

impl ProbeConstructorWitness<ProbeBox> for ProbeBoxContext {}
"#,
    );

    assert!(
        !snapshot.all_diagnostics().any(|diagnostic| {
            matches!(
                diagnostic.code,
                DiagnosticCode::AnnotationUnsaturatedConstructor | DiagnosticCode::AnnotationUnresolved
            )
        }),
        "constructor-kinded trait arguments must be formed at their declared kind; diagnostics: {:#?}",
        snapshot.diagnostics
    );
    assert!(!snapshot.has_errors(), "diagnostics: {:#?}", snapshot.diagnostics);
}


#[test]
fn same_owner_class_generic_dispatch_resolves_current_class() {
    let snapshot = analyze(
        r#"
class ProbeStatics {
  @class
  identity<T>(_ value: T) -> T { value }

  @class
  forward<T>(_ value: T) -> T {
    ProbeStatics.identity(value)
  }
}
"#,
    );

    assert!(
        !snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::AnnotationUnresolved),
        "same-owner class generic dispatch must resolve the current class; diagnostics: {:#?}",
        snapshot.diagnostics
    );
    assert!(!snapshot.has_errors(), "diagnostics: {:#?}", snapshot.diagnostics);
}


#[test]
fn canonical_universe_diagnostic_census_probe() {
    use std::collections::BTreeMap;

    let image = phalcom_semantic::canonical_image::canonical_universe_formal_image()
        .expect("canonical Universe formal image must build for diagnostic census");
    let diagnostics = image.snapshot().all_diagnostics().collect::<Vec<_>>();
    let mut counts = BTreeMap::<String, usize>::new();

    for diagnostic in &diagnostics {
        *counts.entry(format!("{:?}", diagnostic.code)).or_default() += 1;
    }

    eprintln!("UNIVERSE_CENSUS_TOTAL|{}", diagnostics.len());
    for (code, count) in counts {
        eprintln!("UNIVERSE_CENSUS_COUNT|{code}|{count}");
    }
    for diagnostic in diagnostics {
        eprintln!(
            "UNIVERSE_CENSUS_ROW|{:?}|{:?}|{:?}|{}",
            diagnostic.code,
            diagnostic.primary.module,
            diagnostic.primary.range,
            diagnostic.message.replace('\n', " ")
        );
    }
}

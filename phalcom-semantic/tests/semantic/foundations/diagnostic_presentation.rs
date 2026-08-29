use crate::semantic::support::Fixture;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::{DerivationRule, DiagnosticCode, DiagnosticDetail, DiagnosticPresenter, PresentedLabelRole};

#[test]
fn binding_mismatch_presentation_is_specialized_and_trace_is_deterministic() {
    let fixture = Fixture::new(
        r#"
class CellNum { @constructor new() {} }
class Probe {
  @class
  run() {
    let value: Int = CellNum.new()
  }
}
"#,
    );
    let diagnostic = fixture
        .diagnostics(DiagnosticCode::BindingInitializerMismatch)
        .into_iter()
        .next()
        .expect("binding mismatch diagnostic");
    let presenter = DiagnosticPresenter::new(&fixture.analysis.snapshot);

    let explained = presenter.present(diagnostic, DiagnosticDetail::Explain);
    assert_eq!(explained.headline, "initializer conflicts with declared type");
    assert!(explained.labels.iter().any(|label| label.role == PresentedLabelRole::Required));
    assert!(explained.labels.iter().any(|label| label.role == PresentedLabelRole::Established));
    assert!(explained.explanation.iter().any(|line| line.text.contains("@constructor")));
    assert!(
        explained
            .explanation
            .iter()
            .any(|line| line.text.contains("specialized") || line.text.contains("not assignable"))
    );
    assert!(explained.guidance.iter().any(|line| line.text.contains("CellNum")));

    let first = presenter.present(diagnostic, DiagnosticDetail::Trace);
    let second = presenter.present(diagnostic, DiagnosticDetail::Trace);
    assert_eq!(first.trace, second.trace);
    assert!(first.trace.iter().any(|node| node.rule == DerivationRule::TypeRelation));
    assert!(first.trace.iter().any(|node| node.rule == DerivationRule::CallableSelection));
}

#[test]
fn underconstrained_presentation_does_not_turn_unavailable_evidence_into_a_contradiction() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  make<T>() -> T { 42 }

  @class
  run() {
    let value: Int = Probe.make()
  }
}
"#,
    );
    let diagnostic = fixture
        .diagnostics(DiagnosticCode::GenericInferenceUnderconstrained)
        .into_iter()
        .next()
        .expect("underconstrained diagnostic");
    assert!(
        diagnostic.root_cause.is_none(),
        "presentation diagnostic must not manufacture formal invalidity"
    );

    let presented = DiagnosticPresenter::new(&fixture.analysis.snapshot).present(diagnostic, DiagnosticDetail::Trace);
    assert_eq!(presented.headline, "generic parameter is underconstrained");
    assert!(!presented.headline.contains("found Unknown"));
    assert!(!presented.explanation.iter().any(|line| line.text.contains("not assignable")));
    assert!(presented.trace.iter().any(|node| node.rule == DerivationRule::UnknownPropagation));
}

#[test]
fn compact_and_explain_are_projections_of_the_same_semantic_diagnostic() {
    let fixture = Fixture::new(
        r#"
class CellNum {
  @class
  fromInt(_ value: Int) -> CellNum { CellNum.new() }
  @constructor new() {}
}
class Probe {
  @class
  run() {
    let value = CellNum.fromInt("bad")
  }
}
"#,
    );
    let diagnostic = fixture
        .diagnostics(DiagnosticCode::ArgumentMismatch)
        .into_iter()
        .next()
        .expect("argument mismatch diagnostic");
    let presenter = DiagnosticPresenter::new(&fixture.analysis.snapshot);
    let compact = presenter.present(diagnostic, DiagnosticDetail::Compact);
    let explain = presenter.present(diagnostic, DiagnosticDetail::Explain);

    assert_eq!(compact.code, explain.code);
    assert_eq!(compact.severity, explain.severity);
    assert_eq!(compact.headline, explain.headline);
    assert_eq!(compact.primary, explain.primary);
    assert!(compact.explanation.len() <= 2);
    assert!(explain.explanation.len() >= compact.explanation.len());
}

#[test]
fn machine_names_are_stable() {
    assert_eq!(DerivationRule::TypeRelation.as_str(), "type_relation");
    assert_eq!(phalcom_semantic::EvidenceStatus::Established.as_str(), "established");
    assert_eq!(phalcom_semantic::EvidenceOrigin::ConstructorSemantics.as_str(), "constructor_semantics");
    assert_eq!(PresentedLabelRole::Required.as_str(), "required");
}

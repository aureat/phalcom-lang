use crate::semantic::support::Fixture;
use phalcom_common::range::SourceRange;
use phalcom_semantic::diagnostic::{DiagnosticCode, DiagnosticFix, DiagnosticSeverity, ExplanationRef, SemanticDiagnostic};
use phalcom_semantic::identity::{DiagnosticCauseId, DispatchSide, ExplanationId, ModuleId};

#[test]
fn test_structured_diagnostic_extensions() {
    let range = SourceRange { start: 5, end: 15 };
    let callable = phalcom_semantic::identity::CallableId::new(
        phalcom_semantic::identity::DeclarationId::new(ModuleId::universe_root(), "Probe".into()),
        phalcom_common::selector::Selector::getter("run").unwrap(),
        DispatchSide::Class,
    );
    let explanation = ExplanationRef::new(callable, ExplanationId(42));
    let diag = SemanticDiagnostic::error_in(ModuleId::universe_root(), DiagnosticCode::TypeMismatch, "type mismatch occurred", range)
        .with_note("expected Int, found String")
        .with_help("try converting with asInt")
        .with_explanation(explanation.clone())
        .with_fix(DiagnosticFix::replacement("replace with asInt", range, "x.asInt"))
        .with_root_cause(DiagnosticCauseId(1));

    assert_eq!(diag.severity, DiagnosticSeverity::Error);
    assert_eq!(diag.notes.len(), 1);
    assert_eq!(diag.helps.len(), 1);
    assert_eq!(diag.explanations, vec![explanation]);
    assert_eq!(diag.fixes.len(), 1);
    assert_eq!(diag.fixes[0].message, "replace with asInt");
    assert_eq!(diag.root_cause, Some(DiagnosticCauseId(1)));

    let rendered = diag.render(Some("let x = \"hello\""), Some("test.ph"));
    assert!(rendered.contains("type mismatch occurred"));
}

#[test]
fn explanation_refs_disambiguate_callable_local_ids() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  first() {
    1
  }

  @class
  second() {
    "two"
  }
}
"#,
    );

    let first = fixture.callable("Probe", "first", DispatchSide::Class);
    let second = fixture.callable("Probe", "second", DispatchSide::Class);
    let first_expression = fixture.expression(first, "1");
    let second_expression = fixture.expression(second, "\"two\"");

    let first_id = first_expression.explanation.expect("first expression explanation");
    let second_id = second_expression.explanation.expect("second expression explanation");
    assert_eq!(first_id, ExplanationId(0));
    assert_eq!(second_id, ExplanationId(0));

    let first_ref = ExplanationRef::new(first.callable.clone(), first_id);
    let second_ref = ExplanationRef::new(second.callable.clone(), second_id);

    let first_node = fixture.analysis.snapshot.explanation_node(&first_ref).expect("first explanation node");
    let second_node = fixture.analysis.snapshot.explanation_node(&second_ref).expect("second explanation node");

    assert_ne!(first_ref.callable, second_ref.callable);
    assert_ne!(first_node.step, second_node.step);
    assert!(std::ptr::eq(
        fixture.analysis.snapshot.explanation_arena(&first_ref.callable).unwrap(),
        first.explanations.as_ref()
    ));
    assert!(std::ptr::eq(
        fixture.analysis.snapshot.explanation_arena(&second_ref.callable).unwrap(),
        second.explanations.as_ref()
    ));
}

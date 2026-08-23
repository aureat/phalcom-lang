use phalcom_common::range::SourceRange;
use phalcom_semantic::diagnostic::{DiagnosticCode, DiagnosticFix, DiagnosticSeverity, SemanticDiagnostic};
use phalcom_semantic::identity::{DiagnosticCauseId, ExplanationId, ModuleId};

#[test]
fn test_structured_diagnostic_extensions() {
    let range = SourceRange { start: 5, end: 15 };
    let diag = SemanticDiagnostic::error_in(ModuleId::core(), DiagnosticCode::TypeMismatch, "type mismatch occurred", range)
        .with_note("expected Int, found String")
        .with_help("try converting with asInt")
        .with_explanation(ExplanationId(42))
        .with_fix(DiagnosticFix::replacement("replace with asInt", range, "x.asInt"))
        .with_root_cause(DiagnosticCauseId(1));

    assert_eq!(diag.severity, DiagnosticSeverity::Error);
    assert_eq!(diag.notes.len(), 1);
    assert_eq!(diag.helps.len(), 1);
    assert_eq!(diag.explanations, vec![ExplanationId(42)]);
    assert_eq!(diag.fixes.len(), 1);
    assert_eq!(diag.fixes[0].message, "replace with asInt");
    assert_eq!(diag.root_cause, Some(DiagnosticCauseId(1)));

    let rendered = diag.render(Some("let x = \"hello\""), Some("test.ph"));
    assert!(rendered.contains("type mismatch occurred"));
}

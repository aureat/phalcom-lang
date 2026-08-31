use std::sync::Arc;

use phalcom_modules::identity::ModuleId;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::diagnostic::DiagnosticCode;

#[test]
fn generic_associated_owner_conflict_reports_gadt_mismatch_only() {
    let source: Arc<str> = Arc::from(
        r#"
enum Expr<T> {
  @variant IntLit(_ value: Int) -> Expr<Int>
  @variant BoolLit(_ value: Bool) -> Expr<Bool>
}
class Probe {
  @class run() { Expr<String>::IntLit(1) }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(ModuleId::core(), source, Arc::new(parsed.program));
    let diagnostics = analysis.snapshot.all_diagnostics().collect::<Vec<_>>();

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::AssociatedGadtOwnerConflict)
    );
    assert!(!diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::AssociatedFamilyMissing));
}

use std::sync::Arc;

use phalcom_modules::identity::ModuleId;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::diagnostic::DiagnosticCode;

#[test]
fn associated_owner_rejects_known_runtime_value_without_type_form_denotation() {
    let source: Arc<str> = Arc::from(
        r#"
class Probe {
  @class run() {
    let value = 42
    value::missing
  }
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
            .any(|diagnostic| diagnostic.code == DiagnosticCode::AssociatedOwnerNotTypeForm)
    );
    assert!(!diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::AssociatedFamilyMissing));
}

#[test]
fn associated_owner_generic_parameter_is_declaration_independent() {
    let source: Arc<str> = Arc::from(
        r#"
class Box<T> {
  @class run() {
    T::missing
  }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(ModuleId::core(), source, Arc::new(parsed.program));

    assert!(
        analysis
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::AssociatedOwnerNotDeclarationBacked)
    );
}

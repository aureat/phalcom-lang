use std::sync::Arc;

use phalcom_modules::identity::ModuleId;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::diagnostic::DiagnosticCode;

#[test]
fn getter_lookup_does_not_fall_back_to_zero_argument_constructor() {
    let source: Arc<str> = Arc::from(
        r#"
enum State {
  @variant Ready()
}
class Probe {
  @class run() { State::Ready }
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
            .any(|diagnostic| diagnostic.code == DiagnosticCode::AssociatedMemberMissing)
    );
}

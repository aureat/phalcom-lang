use std::sync::Arc;

use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::checker::AssociatedResolutionKind;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};

#[test]
fn gadt_constructor_resolution_refines_owner_and_keeps_runtime_result_known() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
enum Expr<T> {
  @variant IntLit(_ value: Int) -> Expr<Int>
  @variant BoolLit(_ value: Bool) -> Expr<Bool>
}
class Probe {
  @class run() { Expr<Int>::IntLit(42) }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    assert!(
        !analysis
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::AssociatedGadtOwnerConflict)
    );

    let probe = DeclarationId::new(module, "Probe".into());
    let run = CallableId::new(probe, Selector::method("run", []).expect("run selector"), DispatchSide::Class);
    let callable = analysis.snapshot.callable_analyses.get(&run).expect("Probe.run analysis");
    let expression = callable
        .expressions
        .values()
        .find(|candidate| source.get(candidate.range.start..candidate.range.end) == Some("Expr<Int>::IntLit(42)"))
        .expect("GADT constructor expression");
    let resolution = callable.associated_resolutions.get(&expression.id).expect("associated resolution");

    assert!(expression.knowledge.is_known());
    assert!(matches!(resolution.kind, AssociatedResolutionKind::StaticInvoke { .. }));
}

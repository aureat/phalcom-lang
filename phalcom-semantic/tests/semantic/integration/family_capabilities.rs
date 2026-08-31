use std::sync::Arc;

use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::types::denotation::SemanticDenotation;

#[test]
fn stored_family_capture_preserves_its_denotation_through_local_flow() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
enum State {
  @variant Ready
}
class Probe {
  @class run() {
    let first = State::Ready::*;
    let second = first;
    second
  }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let probe = DeclarationId::new(module, "Probe".into());
    let run = CallableId::new(probe, Selector::method("run", []).expect("run selector"), DispatchSide::Class);
    let callable = analysis.snapshot.callable_analyses.get(&run).expect("Probe.run analysis");
    let capture = callable
        .expressions
        .values()
        .find(|candidate| source.get(candidate.range.start..candidate.range.end) == Some("State::Ready::*"))
        .expect("family capture");
    let stored = callable
        .expressions
        .values()
        .find(|candidate| source.get(candidate.range.start..candidate.range.end) == Some("first"))
        .expect("stored family read");

    assert!(matches!(capture.denotation, Some(SemanticDenotation::AssociatedValue(_))));
    assert_eq!(capture.denotation, stored.denotation);
}

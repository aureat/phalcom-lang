//! Formal fixed-return regressions for canonical native callables.

use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::checker::analysis::CallableAnalysis;
use phalcom_semantic::types::{EvidenceOrigin, EvidenceStatus, TypeKnowledge};
use phalcom_semantic::workspace::analyze_single_module;
use std::sync::Arc;

fn module_id() -> ModuleId {
    ModuleId::resolved(
        ResolvedProjectId::from_raw(1),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("main").expect("module component")]),
    )
}

fn demo_run<'a>(analyses: &'a std::collections::HashMap<phalcom_semantic::CallableId, Arc<CallableAnalysis>>) -> &'a CallableAnalysis {
    let selector = Selector::method("run", []).expect("run selector");
    analyses
        .iter()
        .find(|(callable, _)| callable.owner.name.as_ref() == "Demo" && callable.selector == selector)
        .map(|(_, analysis)| analysis.as_ref())
        .expect("Demo.run analysis")
}

#[test]
fn system_print_call_and_tail_return_are_established_unit() {
    let module = module_id();
    let source = Arc::from(
        r#"
class Demo {
  run() {
    System.print("hello")
  }
}
"#,
    );
    let program = Arc::new(phalcom_ast::parse(&source, 0).program);
    let snapshot = analyze_single_module(module, source, program).snapshot;
    let analysis = demo_run(&snapshot.callable_analyses);

    let call = analysis
        .expressions
        .values()
        .filter(|expression| expression.knowledge.ty() == Some(snapshot.store.unit()))
        .max_by_key(|expression| expression.range.end.saturating_sub(expression.range.start))
        .expect("System.print call expression");
    assert!(matches!(call.knowledge, TypeKnowledge::Known(ref evidence)
        if evidence.ty == snapshot.store.unit()
            && evidence.status == EvidenceStatus::Established
            && evidence.origin == EvidenceOrigin::CallableSignature));

    assert_eq!(analysis.exits.normal_return_values.len(), 1);
    assert!(matches!(&analysis.exits.normal_return_values[0], TypeKnowledge::Known(evidence)
        if evidence.ty == snapshot.store.unit() && evidence.status == EvidenceStatus::Established));
}

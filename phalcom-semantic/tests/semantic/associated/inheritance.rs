use std::sync::Arc;

use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::checker::AssociatedResolutionKind;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, InvocationTargetId};

#[test]
fn inherited_associated_lookup_keeps_descendant_lookup_and_ancestor_definition() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
class Base {
  @class make() { 1 }
}
class Child is Base {
}
class Probe {
  @class run() { Child::make() }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let probe = DeclarationId::new(module.clone(), "Probe".into());
    let run = CallableId::new(probe, Selector::method("run", []).expect("run selector"), DispatchSide::Class);
    let callable = analysis.snapshot.callable_analyses.get(&run).expect("Probe.run analysis");
    let expression = callable
        .expressions
        .values()
        .find(|candidate| source.get(candidate.range.start..candidate.range.end) == Some("Child::make()"))
        .expect("Child::make() expression");
    let resolution = callable.associated_resolutions.get(&expression.id).expect("associated resolution");

    assert_eq!(resolution.lookup_owner, DeclarationId::new(module.clone(), "Child".into()));
    let AssociatedResolutionKind::StaticInvoke {
        target: InvocationTargetId::Behavioral(target),
        ..
    } = &resolution.kind
    else {
        panic!("expected inherited behavioral invocation, got {:?}", resolution.kind);
    };
    assert_eq!(target.declaration_owner(), &DeclarationId::new(module, "Base".into()));
}

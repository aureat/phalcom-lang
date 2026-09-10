use std::sync::Arc;

use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::checker::CallableReferenceResolutionKind;
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
  @class run() {
    let make = &Child.make()
    make()
  }
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
        .find(|candidate| source.get(candidate.range.start..candidate.range.end) == Some("&Child.make()"))
        .expect("&Child.make expression");
    let resolution = callable
        .callable_reference_resolutions
        .get(&expression.id)
        .expect("callable reference resolution");
    let CallableReferenceResolutionKind::BoundFamily { members, .. } = &resolution.kind else {
        panic!("expected bound inherited behavioral family, got {:?}", resolution.kind);
    };
    assert!(members.iter().any(|member| {
        matches!(
            &member.target,
            InvocationTargetId::Behavioral(target)
                if target.declaration_owner() == &DeclarationId::new(module.clone(), "Base".into())
        )
    }));
}

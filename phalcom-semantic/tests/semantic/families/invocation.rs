use std::sync::Arc;

use phalcom_common::selector::{SelectorKind, SelectorSlot};
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::checker::{FamilyApplicationResolution, FamilyApplicationSelection};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, InvocationTargetId};

#[test]
fn immediate_family_call_publishes_static_application_resolution() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
enum Weird {
  @variant Marker
  @variant Marker()
  @variant Marker(_ value: Int)
}

class Probe {
  @class run() { (Weird::Marker::*)(1) }
}
"#,
    );
    let analysis = analyze_source(module.clone(), source.clone());
    let probe = DeclarationId::new(module, "Probe".into());
    let resolution = find_application(&analysis, &source, &probe, "run", "(Weird::Marker::*)(1)");

    let FamilyApplicationSelection::Static { operation, target, .. } = &resolution.selection else {
        panic!("expected static family application, got {:?}", resolution.selection);
    };
    assert_eq!(operation.kind, SelectorKind::Method);
    assert_eq!(operation.slots.as_ref(), [SelectorSlot::Positional]);
    assert!(matches!(target, Some(InvocationTargetId::VariantConstructor(_))));
}

#[test]
fn stored_family_call_publishes_static_application_resolution() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
enum Weird {
  @variant Marker
  @variant Marker()
  @variant Marker(_ value: Int)
}

class Probe {
  @class run() {
    let make = Weird::Marker::*;
    make(1)
  }
}
"#,
    );
    let analysis = analyze_source(module.clone(), source.clone());
    let probe = DeclarationId::new(module, "Probe".into());
    let resolution = find_application(&analysis, &source, &probe, "run", "make(1)");

    let FamilyApplicationSelection::Static { operation, target, .. } = &resolution.selection else {
        panic!("expected static family application, got {:?}", resolution.selection);
    };
    assert_eq!(operation.kind, SelectorKind::Method);
    assert_eq!(operation.slots.as_ref(), [SelectorSlot::Positional]);
    assert!(matches!(target, Some(InvocationTargetId::VariantConstructor(_))));
}

#[test]
fn stored_family_dynamic_pack_publishes_frozen_candidates() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
enum Weird {
  @variant Marker
  @variant Marker()
  @variant Marker(_ value: Int)
}

class Probe {
  @class run() {
    let make = Weird::Marker::*;
    let args = [1];
    make(*args)
  }
}
"#,
    );
    let analysis = analyze_source(module.clone(), source.clone());
    let probe = DeclarationId::new(module, "Probe".into());
    let resolution = find_application(&analysis, &source, &probe, "run", "make(*args)");

    let FamilyApplicationSelection::Dynamic { candidates, .. } = &resolution.selection else {
        panic!("expected dynamic family application, got {:?}", resolution.selection);
    };
    assert_eq!(candidates.len(), 2);
    assert!(candidates.iter().all(|candidate| candidate.operation.kind == SelectorKind::Method));
    assert!(candidates.iter().all(|candidate| candidate.target.is_some()));
    assert!(candidates[0].operation.slots.is_empty());
    assert_eq!(candidates[1].operation.slots.as_ref(), [SelectorSlot::Positional]);
    assert!(matches!(
        &candidates[0].target,
        Some(InvocationTargetId::VariantConstructor(constructor)) if constructor.variant.selector.slots.is_empty()
    ));
    assert!(matches!(
        &candidates[1].target,
        Some(InvocationTargetId::VariantConstructor(constructor))
            if constructor.variant.selector.slots.as_ref() == [SelectorSlot::Positional]
    ));
}

fn analyze_source(module: ModuleId, source: Arc<str>) -> phalcom_semantic::workspace::SemanticAnalysis {
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    analyze_single_module(module, source, Arc::new(parsed.program))
}

fn find_application<'a>(
    analysis: &'a phalcom_semantic::workspace::SemanticAnalysis,
    source: &str,
    owner: &DeclarationId,
    callable: &str,
    expression: &str,
) -> &'a FamilyApplicationResolution {
    let callable_id = CallableId::new(
        owner.clone(),
        phalcom_common::selector::Selector::method(callable, []).expect("callable selector"),
        DispatchSide::Class,
    );
    let callable = analysis.snapshot.callable_analyses.get(&callable_id).expect("callable analysis");
    let expression = callable
        .expressions
        .values()
        .find(|candidate| source.get(candidate.range.start..candidate.range.end) == Some(expression))
        .expect("family application expression");
    callable.family_applications.get(&expression.id).expect("family application resolution")
}

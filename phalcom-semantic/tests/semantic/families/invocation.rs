use std::sync::Arc;

use phalcom_common::selector::{SelectorKind, SelectorSlot};
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::checker::{FamilyApplicationResolution, FamilyApplicationSelection};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, InvocationTargetId};

#[test]
fn immediate_family_call_publishes_static_application_resolution() {
    let module = ModuleId::universe_root();
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
    let module = ModuleId::universe_root();
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
    let module = ModuleId::universe_root();
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

#[test]
fn generic_family_failure_does_not_publish_fallback_result() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
class Box<T> {
  convert<U>() -> U { 42 }
}

class Probe {
  @class
  run(_ box: Box<Int>) {
    let family = box::convert::*;
    let value = family()
  }
}
"#,
    );
    let analysis = analyze_source(module.clone(), source.clone());
    let probe = DeclarationId::new(module, "Probe".into());
    let callable_id = CallableId::new(
        probe,
        phalcom_common::selector::Selector::method("run", [SelectorSlot::Positional]).expect("callable selector"),
        DispatchSide::Class,
    );
    let callable = analysis.snapshot.callable_analyses.get(&callable_id).expect("callable analysis");
    let family_capture = callable
        .expressions
        .values()
        .find(|candidate| source.get(candidate.range.start..candidate.range.end) == Some("box::convert::*"))
        .expect("generic family capture expression");
    assert!(
        family_capture.knowledge.is_known(),
        "generic family capture must retain a callable family, got {:?}",
        family_capture.knowledge
    );

    let expression = callable
        .expressions
        .values()
        .find(|candidate| source.get(candidate.range.start..candidate.range.end) == Some("family()"))
        .expect("family application expression");

    assert!(
        expression.knowledge.ty().is_none(),
        "expected failed generic application, got {:?}",
        expression.knowledge
    );
    assert!(
        !callable.family_applications.contains_key(&expression.id),
        "failed generic application must not publish fallback family result: {:?}",
        callable.family_applications.get(&expression.id)
    );
}

#[test]
fn generic_behavioral_family_recovers_canonical_signature_for_expected_result() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
class Box<T> {
  convert<U>() -> U { 42 }
}

class Probe {
  @class
  run(_ box: Box<Int>) {
    let family = box::convert::*;
    let value: Int = family();
  }
}
"#,
    );
    let analysis = analyze_source(module.clone(), source.clone());
    let probe = DeclarationId::new(module, "Probe".into());
    let callable_id = CallableId::new(
        probe,
        phalcom_common::selector::Selector::method("run", [SelectorSlot::Positional]).expect("callable selector"),
        DispatchSide::Class,
    );
    let callable = analysis.snapshot.callable_analyses.get(&callable_id).expect("callable analysis");
    let expression = callable
        .expressions
        .values()
        .find(|candidate| source.get(candidate.range.start..candidate.range.end) == Some("family()"))
        .expect("family application expression");
    let result_type = expression.knowledge.ty().expect("expected generic family result");
    assert_eq!(analysis.snapshot.store.format_type(result_type), "Int");
}

#[test]
fn generic_associated_variant_family_recovers_constructor_signature() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
enum Option<T> {
  @variant Some(_ value: T) -> Option<T>
}

class Probe {
  @class
  run() {
    let family = Option<Int>::Some::*;
    let value = family(1);
  }
}
"#,
    );
    let analysis = analyze_source(module.clone(), source.clone());
    let probe = DeclarationId::new(module, "Probe".into());
    let callable_id = CallableId::new(
        probe,
        phalcom_common::selector::Selector::method("run", []).expect("callable selector"),
        DispatchSide::Class,
    );
    let callable = analysis.snapshot.callable_analyses.get(&callable_id).expect("callable analysis");
    let expression = callable
        .expressions
        .values()
        .find(|candidate| source.get(candidate.range.start..candidate.range.end) == Some("family(1)"))
        .expect("family application expression");
    let result_type = expression.knowledge.ty().expect("associated family result");
    assert!(matches!(
        analysis.snapshot.store.get(result_type),
        phalcom_semantic::types::store::TypeData::ExactCase { .. }
    ));

    let resolution = callable.family_applications.get(&expression.id).expect("family application resolution");
    let FamilyApplicationSelection::Static { target, .. } = &resolution.selection else {
        panic!("expected static family application, got {:?}", resolution.selection);
    };
    assert!(matches!(target, Some(InvocationTargetId::VariantConstructor(_))));
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

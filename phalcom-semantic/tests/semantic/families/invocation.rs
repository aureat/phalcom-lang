use std::sync::Arc;

use phalcom_common::selector::{SelectorKind, SelectorSlot};
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::checker::{AssociatedResolutionKind, FamilyApplicationResolution, FamilyApplicationSelection};
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide, InvocationTargetId};
use phalcom_semantic::types::denotation::SemanticDenotation;

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

#[test]
fn family_wrong_shape_reports_associated_call_shape() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
enum Weird {
  @variant Marker(_ value: Int)
}

class Probe {
  @class
  run() { Weird::Marker(1, 2) }
}
"#,
    );
    let analysis = analyze_source(module, source);
    assert!(
        analysis
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.code == phalcom_semantic::diagnostic::DiagnosticCode::AssociatedCallShapeMissing)
    );
}

#[test]
fn behavioral_family_source_preserves_pattern_storage_and_dispatch_side() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
class Service {
  @class
  make(_ value: Int) -> Int { value }

  take(_ value: Int) -> Int { value }
}

class Probe {
  @class
  run(_ service: Service) {
    let instance = service::take::*;
    let class_side = Service::make::*;
    let instance_result = instance(1)
    let class_result = class_side(1)
  }
}
"#,
    );
    let analysis = analyze_source(module.clone(), source.clone());
    assert!(!analysis.snapshot.has_errors(), "diagnostics: {:?}", analysis.snapshot.diagnostics);
    let probe = DeclarationId::new(module, "Probe".into());
    let run_id = CallableId::new(
        probe,
        phalcom_common::selector::Selector::method("run", [SelectorSlot::Positional]).expect("run selector"),
        DispatchSide::Class,
    );
    let callable = analysis.snapshot.callable_analyses.get(&run_id).expect("run analysis");

    let capture_for = |text: &str| {
        callable
            .expressions
            .values()
            .find(|expression| source.get(expression.range.start..expression.range.end) == Some(text))
            .unwrap_or_else(|| panic!("missing family capture {text}"))
    };
    for (text, expected_side) in [("service::take::*", DispatchSide::Instance), ("Service::make::*", DispatchSide::Class)] {
        let capture = capture_for(text);
        let resolution = callable.associated_resolutions.get(&capture.id).expect("family resolution");
        let AssociatedResolutionKind::BoundBehavioralFamily { members, .. } = &resolution.kind else {
            panic!("expected behavioral family resolution for {text}, got {:?}", resolution.kind);
        };
        assert_eq!(members.len(), 1, "one exact source method should populate one family member");
        assert!(matches!(&members[0].target, InvocationTargetId::Behavioral(callable) if callable.side == expected_side));
    }

    for (text, expected_side) in [("instance(1)", DispatchSide::Instance), ("class_side(1)", DispatchSide::Class)] {
        let application = callable
            .expressions
            .values()
            .find(|expression| source.get(expression.range.start..expression.range.end) == Some(text))
            .expect("family application expression");
        let resolution = callable.family_applications.get(&application.id).expect("family application resolution");
        let FamilyApplicationSelection::Static { target, .. } = &resolution.selection else {
            panic!("expected static family invocation for {text}, got {:?}", resolution.selection);
        };
        assert!(matches!(target, Some(InvocationTargetId::Behavioral(callable)) if callable.side == expected_side));
    }
}

#[test]
fn structurally_equal_behavioral_families_retain_distinct_denotations_and_targets() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
class Left {
  @class
  make(_ value: Int) -> Int { value }
}
class Right {
  @class
  make(_ value: Int) -> Int { value }
}
class Probe {
  @class
  run() {
    let left = Left::make::*;
    let right = Right::make::*;
  }
}
"#,
    );
    let analysis = analyze_source(module.clone(), source.clone());
    assert!(!analysis.snapshot.has_errors(), "diagnostics: {:?}", analysis.snapshot.diagnostics);
    let probe = DeclarationId::new(module, "Probe".into());
    let run_id = CallableId::new(
        probe,
        phalcom_common::selector::Selector::method("run", []).expect("run selector"),
        DispatchSide::Class,
    );
    let callable = analysis.snapshot.callable_analyses.get(&run_id).expect("run analysis");
    let capture = |text: &str| {
        callable
            .expressions
            .values()
            .find(|expression| source.get(expression.range.start..expression.range.end) == Some(text))
            .unwrap_or_else(|| panic!("missing family capture {text}"))
    };
    let left = capture("Left::make::*");
    let right = capture("Right::make::*");
    assert_eq!(
        left.knowledge.ty(),
        right.knowledge.ty(),
        "same callable shape should intern one structural family type"
    );
    assert_ne!(left.denotation, right.denotation, "receiver declaration remains family provenance");

    let target = |expression: &phalcom_semantic::checker::analysis::ExpressionAnalysis| {
        let SemanticDenotation::AssociatedValue(denotation) = expression.denotation.as_ref().expect("family denotation") else {
            panic!("expected associated family denotation");
        };
        let phalcom_semantic::types::denotation::AssociatedValueDenotation::BehavioralFamily { members, .. } = denotation.as_ref() else {
            panic!("expected behavioral family denotation");
        };
        members[0].target.clone()
    };
    let left_target = target(left);
    let right_target = target(right);
    assert_ne!(left_target, right_target, "same family shape must retain distinct callable targets");
    assert!(matches!(left_target, InvocationTargetId::Behavioral(ref callable) if callable.owner.declaration().name.as_ref() == "Left"));
    assert!(matches!(right_target, InvocationTargetId::Behavioral(ref callable) if callable.owner.declaration().name.as_ref() == "Right"));
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

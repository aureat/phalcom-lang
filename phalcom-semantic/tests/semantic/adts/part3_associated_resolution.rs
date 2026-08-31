use std::sync::Arc;

use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::checker::AssociatedResolutionKind;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};

#[test]
fn test_associated_owner_not_type_form_error() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
class Probe {
  @class
  run() {
    let x = 42
    x::m
  }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module, source, Arc::new(parsed.program));

    assert!(
        analysis.snapshot.diagnostics.values().flat_map(|d| d.iter()).any(|d| d.code == DiagnosticCode::AssociatedOwnerNotTypeForm),
        "Expected AssociatedOwnerNotTypeForm diagnostic, got: {:#?}",
        analysis.snapshot.diagnostics
    );
}

#[test]
fn test_associated_owner_generic_parameter_error() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
class Box<T> {
  @class
  run() {
    T::m
  }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module, source, Arc::new(parsed.program));

    assert!(
        analysis.snapshot.diagnostics.values().flat_map(|d| d.iter()).any(|d| d.code == DiagnosticCode::AssociatedOwnerNotDeclarationBacked),
        "Expected AssociatedOwnerNotDeclarationBacked diagnostic, got: {:#?}",
        analysis.snapshot.diagnostics
    );
}

#[test]
fn test_associated_direct_variant_invocation_and_getter() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
enum Option<T> {
  @variant Some(_ value: T)
  @variant None
}

class Test {
  @class
  test_some() {
    Option<Int>::Some(42)
  }
  @class
  test_none() {
    Option<Int>::None
  }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));

    let test_decl = DeclarationId::new(module.clone(), "Test".into());

    let test_some_id = CallableId::new(test_decl.clone(), Selector::method("test_some", []).unwrap(), DispatchSide::Class);
    let some_analysis = analysis.snapshot.callable_analyses.get(&test_some_id).expect("test_some callable");
    let some_expr = some_analysis
        .expressions
        .values()
        .find(|e| source.get(e.range.start..e.range.end) == Some("Option<Int>::Some(42)"))
        .expect("Option<Int>::Some(42) expression");

    assert!(some_expr.knowledge.is_known(), "Some(42) should have known type");
    let resolution = some_analysis.associated_resolutions.get(&some_expr.id).expect("some resolution");
    assert!(matches!(&resolution.kind, AssociatedResolutionKind::StaticInvoke { .. }));

    let test_none_id = CallableId::new(test_decl, Selector::method("test_none", []).unwrap(), DispatchSide::Class);
    let none_analysis = analysis.snapshot.callable_analyses.get(&test_none_id).expect("test_none callable");
    let none_expr = none_analysis
        .expressions
        .values()
        .find(|e| source.get(e.range.start..e.range.end) == Some("Option<Int>::None"))
        .expect("Option<Int>::None expression");

    assert!(none_expr.knowledge.is_known(), "None should have known type");
    let none_res = none_analysis.associated_resolutions.get(&none_expr.id).expect("none resolution");
    assert!(matches!(&none_res.kind, AssociatedResolutionKind::ExactValue { .. }));
}

#[test]
fn test_associated_behavioral_inheritance_and_shadowing() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
class Animal {
  @class kind() { 1 }
}

class Dog is Animal {
  @class kind() { 2 }
}

class Cat is Animal {
}

class Probe {
  @class run() {
    let a = Dog::kind()
    let b = Cat::kind()
  }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let program = Arc::new(parsed.program);
    let analysis = analyze_single_module(module.clone(), source.clone(), program.clone());

    let probe_decl = DeclarationId::new(module.clone(), "Probe".into());
    let run_id = CallableId::new(probe_decl, Selector::method("run", []).unwrap(), DispatchSide::Class);
    let run_analysis = analysis.snapshot.callable_analyses.get(&run_id).expect("Probe.run callable");

    let dog_call = run_analysis
        .expressions
        .values()
        .find(|e| source.get(e.range.start..e.range.end) == Some("Dog::kind()"))
        .expect("Dog::kind() expr");
    let dog_res = run_analysis.associated_resolutions.get(&dog_call.id).expect("Dog resolution");
    let dog_decl = DeclarationId::new(module.clone(), "Dog".into());
    assert_eq!(dog_res.lookup_owner, dog_decl);

    let cat_call = run_analysis
        .expressions
        .values()
        .find(|e| source.get(e.range.start..e.range.end) == Some("Cat::kind()"))
        .expect("Cat::kind() expr");
    let cat_res = run_analysis.associated_resolutions.get(&cat_call.id).expect("Cat resolution");
    let cat_decl = DeclarationId::new(module.clone(), "Cat".into());
    let animal_decl = DeclarationId::new(module.clone(), "Animal".into());
    assert_eq!(cat_res.lookup_owner, cat_decl);
    match &cat_res.kind {
        AssociatedResolutionKind::StaticInvoke { target, .. } => {
            if let phalcom_semantic::identity::InvocationTargetId::Behavioral(c) = target {
                assert_eq!(c.declaration_owner(), &animal_decl);
            } else {
                panic!("Expected behavioral target");
            }
        }
        _ => panic!("Expected StaticInvoke"),
    }
}

#[test]
fn test_associated_gadt_owner_conflict() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
enum Expr<T> {
  @variant IntLit(_ value: Int) -> Expr<Int>
  @variant BoolLit(_ value: Bool) -> Expr<Bool>
}

class Test {
  @class
  run_ok() {
    Expr<Int>::IntLit(42)
  }
  @class
  run_bad() {
    Expr<String>::IntLit(42)
  }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module, source, Arc::new(parsed.program));

    assert!(
        analysis.snapshot.diagnostics.values().flat_map(|d| d.iter()).any(|d| d.code == DiagnosticCode::AssociatedGadtOwnerConflict),
        "Expected AssociatedGadtOwnerConflict diagnostic, got: {:#?}",
        analysis.snapshot.diagnostics
    );
}

#[test]
fn test_part3_5_option_a_underconstrained_reification() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
enum Option<T> {
  @variant Some(_ value: T)
  @variant None
}

class Test {
  @class
  bad_reify_some() {
    Option::Some::(_)
  }
  @class
  bad_reify_none() {
    Option::None
  }
  @class
  bad_reify_family() {
    Option::Some::*
  }
  @class
  good_reify_concrete() {
    Option<Int>::Some::(_)
  }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module, source, Arc::new(parsed.program));

    let underconstrained_count = analysis
        .snapshot
        .diagnostics
        .values()
        .flat_map(|d| d.iter())
        .filter(|d| d.code == DiagnosticCode::AssociatedGenericUnderconstrained)
        .count();

    assert_eq!(
        underconstrained_count, 3,
        "Expected exactly 3 AssociatedGenericUnderconstrained diagnostics (for bare Some, None, *), got: {:#?}",
        analysis.snapshot.diagnostics
    );
}

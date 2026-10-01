use std::sync::Arc;

use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::checker::associated::BehavioralFamilySpec;
use phalcom_semantic::checker::{AssociatedResolutionKind, CallableReferenceResolutionKind};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::types::store::TypeData;

#[test]
fn associated_owner_rejects_runtime_value_type_form() {
    let module = ModuleId::universe_root();
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
        analysis
            .snapshot
            .diagnostics
            .values()
            .flat_map(|d| d.iter())
            .any(|d| d.code == DiagnosticCode::AssociatedOwnerNotTypeForm),
        "Expected AssociatedOwnerNotTypeForm diagnostic, got: {:#?}",
        analysis.snapshot.diagnostics
    );
}

#[test]
fn associated_owner_rejects_generic_parameter() {
    let module = ModuleId::universe_root();
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
        analysis
            .snapshot
            .diagnostics
            .values()
            .flat_map(|d| d.iter())
            .any(|d| d.code == DiagnosticCode::AssociatedOwnerNotDeclarationBacked),
        "Expected AssociatedOwnerNotDeclarationBacked diagnostic, got: {:#?}",
        analysis.snapshot.diagnostics
    );
}

#[test]
fn associated_lookup_resolves_variant_invocation_and_singleton_getter() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
data Option<T> {
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
fn nullary_variant_lookup_uses_generic_expected_result_without_accepting_unconstrained_owner() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
data Option<T> {
  @variant Some(_ value: T)
  @variant None
}

data Result<T, E> {
  @variant Ok(_ value: T)
  @variant Error(_ error: E)
}

impl<T, E> Result<T, E> {
  transpose<U>() -> Option<Result<U, E>> where T == Option<U> {
    match self {
      Result::Ok(option) => {
        match option {
          Option::Some(item) => Option::Some(Result::Ok(item))
          Option::None => Option::None
        }
      }
      Result::Error(error) => Option::Some(Result::Error(error))
    }
  }
}

class Probe {
  @class
  contextual<U, E>() -> Option<Result<U, E>> {
    Option::None
  }

  @class
  conditional(_ flag: Bool) -> Option<Int> {
    if flag {
      Option::None
    } else {
      Option::None
    }
  }

  @class
  conflicting_owner() -> Option<String> {
    Option<Int>::None
  }

  @class
  no_context() {
    Option::None
  }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));

    let probe = DeclarationId::new(module.clone(), "Probe".into());
    let contextual = CallableId::new(probe.clone(), Selector::method("contextual", []).unwrap(), DispatchSide::Class);
    let contextual_analysis = analysis.snapshot.callable_analyses.get(&contextual).expect("contextual callable");
    let contextual_expr = contextual_analysis
        .expressions
        .values()
        .find(|expression| source.get(expression.range.start..expression.range.end) == Some("Option::None"))
        .expect("contextual Option::None expression");
    let resolution = contextual_analysis.associated_resolutions.get(&contextual_expr.id).unwrap_or_else(|| {
        panic!(
            "generic expected result must publish the canonical singleton resolution; fact={:?}; diagnostics={:#?}",
            contextual_expr.knowledge, analysis.snapshot.diagnostics
        )
    });
    let AssociatedResolutionKind::ExactValue { value_type, .. } = resolution.kind else {
        panic!("expected exact singleton resolution, got {:?}", resolution.kind);
    };
    let TypeData::ExactCase { enum_type, .. } = analysis.snapshot.store.get(value_type) else {
        panic!("expected exact enum case, got {}", analysis.snapshot.store.format_type(value_type));
    };
    let TypeData::Applied { arguments, .. } = analysis.snapshot.store.get(*enum_type) else {
        panic!(
            "expected specialized applied enum owner, got {}",
            analysis.snapshot.store.format_type(*enum_type)
        );
    };
    let Some(specialized_result) = arguments.first().and_then(|argument| argument.as_type_id()) else {
        panic!("expected the contextual enum argument to be a proper type: {arguments:?}");
    };
    let TypeData::Applied { .. } = analysis.snapshot.store.get(specialized_result) else {
        panic!(
            "expected Result<U, E> to remain the exact owner argument, got {}",
            analysis.snapshot.store.format_type(specialized_result)
        );
    };

    let conditional = CallableId::new(
        probe.clone(),
        Selector::method("conditional", [SelectorSlot::Positional]).unwrap(),
        DispatchSide::Class,
    );
    let conditional_analysis = analysis.snapshot.callable_analyses.get(&conditional).expect("conditional callable");
    let conditional_cases = conditional_analysis
        .expressions
        .values()
        .filter(|expression| source.get(expression.range.start..expression.range.end) == Some("Option::None"))
        .collect::<Vec<_>>();
    assert_eq!(conditional_cases.len(), 2, "both if branches retain their exact-case expressions");
    for expression in conditional_cases {
        assert!(
            conditional_analysis.associated_resolutions.contains_key(&expression.id),
            "the if branch must receive expected Option<Int>; fact={:?}, diagnostics={:#?}",
            expression.knowledge,
            analysis.snapshot.diagnostics
        );
    }

    let transpose = CallableId::new(
        DeclarationId::new(module.clone(), "Result".into()),
        Selector::method("transpose", []).unwrap(),
        DispatchSide::Instance,
    );
    let nested_analysis = analysis.snapshot.callable_analyses.get(&transpose).expect("Result.transpose callable");
    let nested_none = nested_analysis
        .expressions
        .values()
        .find(|expression| source.get(expression.range.start..expression.range.end) == Some("Option::None"))
        .expect("transpose nested match Option::None expression");
    assert!(
        nested_analysis.associated_resolutions.contains_key(&nested_none.id),
        "nested match branch should receive the same expected owner application; fact={:?}, diagnostics={:#?}, matches={:#?}",
        nested_none.knowledge,
        analysis.snapshot.diagnostics,
        nested_analysis.match_resolutions
    );

    let conflict = CallableId::new(
        DeclarationId::new(module.clone(), "Probe".into()),
        Selector::method("conflicting_owner", []).unwrap(),
        DispatchSide::Class,
    );
    let conflict_analysis = analysis.snapshot.callable_analyses.get(&conflict).expect("conflicting_owner callable");
    let conflict_expr = conflict_analysis
        .expressions
        .values()
        .find(|expression| source.get(expression.range.start..expression.range.end) == Some("Option<Int>::None"))
        .expect("explicitly specialized Option<Int>::None expression");
    let conflict_resolution = conflict_analysis
        .associated_resolutions
        .get(&conflict_expr.id)
        .expect("explicit owner remains a canonical associated resolution");
    let AssociatedResolutionKind::ExactValue { value_type, .. } = conflict_resolution.kind else {
        panic!("expected exact singleton resolution, got {:?}", conflict_resolution.kind);
    };
    let TypeData::ExactCase { enum_type, .. } = analysis.snapshot.store.get(value_type) else {
        panic!("expected exact enum case, got {}", analysis.snapshot.store.format_type(value_type));
    };
    let TypeData::Applied { arguments, .. } = analysis.snapshot.store.get(*enum_type) else {
        panic!("expected explicit applied enum owner, got {}", analysis.snapshot.store.format_type(*enum_type));
    };
    let Some(explicit_owner_argument) = arguments.first().and_then(|argument| argument.as_type_id()) else {
        panic!("expected explicit Option<Int> type argument: {arguments:?}");
    };
    assert_eq!(
        analysis.snapshot.store.format_type(explicit_owner_argument),
        "Int",
        "the expected Option<String> result must not overwrite the explicit Option<Int> owner"
    );

    let no_context = CallableId::new(probe, Selector::method("no_context", []).unwrap(), DispatchSide::Class);
    let no_context_analysis = analysis.snapshot.callable_analyses.get(&no_context).expect("no_context callable");
    let no_context_expr = no_context_analysis
        .expressions
        .values()
        .find(|expression| source.get(expression.range.start..expression.range.end) == Some("Option::None"))
        .expect("uncontextual Option::None expression");
    assert!(no_context_analysis.associated_resolutions.get(&no_context_expr.id).is_none());
    let underconstrained = analysis
        .snapshot
        .diagnostics
        .values()
        .flat_map(|diagnostics| diagnostics.iter())
        .filter(|diagnostic| diagnostic.code == DiagnosticCode::AssociatedGenericUnderconstrained)
        .collect::<Vec<_>>();
    assert_eq!(
        underconstrained.len(),
        1,
        "only the no-context owner should remain underconstrained: {:#?}",
        analysis.snapshot.diagnostics
    );
    assert_eq!(
        underconstrained[0].primary_range, no_context_expr.range,
        "the no-context case must remain underconstrained: {:#?}",
        analysis.snapshot.diagnostics
    );
}

#[test]
fn associated_inheritance_keeps_lookup_owner_and_defining_owner_distinct() {
    let module = ModuleId::universe_root();
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
    let a = &Dog.kind...
    let b = &Cat.kind...
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
        .find(|e| source.get(e.range.start..e.range.end) == Some("&Dog.kind..."))
        .expect("&Dog.kind expr");
    let dog_res = run_analysis.callable_reference_resolutions.get(&dog_call.id).expect("Dog resolution");
    let CallableReferenceResolutionKind::BoundFamily { members: dog_members, .. } = &dog_res.kind else {
        panic!("expected Dog behavioral family, got {:?}", dog_res.kind);
    };
    assert!(dog_members.iter().any(|member| {
        matches!(&member.target, phalcom_semantic::identity::InvocationTargetId::Behavioral(target) if target.declaration_owner().name.as_ref() == "Dog")
    }));

    let cat_call = run_analysis
        .expressions
        .values()
        .find(|e| source.get(e.range.start..e.range.end) == Some("&Cat.kind..."))
        .expect("&Cat.kind expr");
    let cat_res = run_analysis.callable_reference_resolutions.get(&cat_call.id).expect("Cat resolution");
    let CallableReferenceResolutionKind::BoundFamily { members: cat_members, .. } = &cat_res.kind else {
        panic!("expected Cat behavioral family, got {:?}", cat_res.kind);
    };
    assert!(cat_members.iter().any(|member| {
        matches!(&member.target, phalcom_semantic::identity::InvocationTargetId::Behavioral(target) if target.declaration_owner().name.as_ref() == "Animal")
    }));
}

#[test]
fn bound_trait_reference_retains_conformance_selection_evidence() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
trait Marked { marker -> String }
class User {}
impl Marked for User { marker -> String { "witness-user" } }
class Caller { run(_ user: User) -> String { let f = &user.marker; f() } }
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    assert!(!analysis.snapshot.has_errors(), "semantic diagnostics: {:#?}", analysis.snapshot.diagnostics);

    let caller = DeclarationId::new(module, "Caller".into());
    let run = CallableId::new(
        caller,
        Selector::method("run", [SelectorSlot::Positional]).expect("run selector"),
        DispatchSide::Instance,
    );
    let callable = analysis.snapshot.callable_analyses.get(&run).expect("Caller.run analysis");
    let reference = callable
        .expressions
        .values()
        .find(|expression| source.get(expression.range.start..expression.range.end) == Some("&user.marker"))
        .expect("bound trait reference expression");
    let resolution = callable
        .callable_reference_resolutions
        .get(&reference.id)
        .expect("bound trait reference resolution");
    let CallableReferenceResolutionKind::BoundFamily { spec, members, .. } = &resolution.kind else {
        panic!("expected bound family, got {:?}", resolution.kind);
    };
    assert!(matches!(spec, BehavioralFamilySpec::Exact(_)), "expected exact family, got {spec:?}");
    assert_eq!(
        members.len(),
        1,
        "exact trait reference should retain exactly one selected member: {members:#?}"
    );
    assert!(
        members[0].trait_dispatch.is_some(),
        "bound family member lost trait dispatch evidence: member={:#?}, expression_trait_dispatch={:#?}",
        members[0],
        reference.trait_dispatch
    );
    assert!(
        reference.trait_dispatch.is_some(),
        "callable-reference expression lost canonical trait dispatch evidence: member={:#?}",
        members[0]
    );
}

#[test]
fn associated_owner_rejects_conflicting_gadt_specialization() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
data Expr<T> {
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
        analysis
            .snapshot
            .diagnostics
            .values()
            .flat_map(|d| d.iter())
            .any(|d| d.code == DiagnosticCode::AssociatedGadtOwnerConflict),
        "Expected AssociatedGadtOwnerConflict diagnostic, got: {:#?}",
        analysis.snapshot.diagnostics
    );
}

#[test]
fn associated_reification_rejects_unconstrained_generic_owner() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
data Option<T> {
  @variant Some(_ value: T)
  @variant None
}

class Test {
  @class
  bad_reify_some() {
    &Option::Some(_)
  }
  @class
  bad_reify_none() {
    Option::None
  }
  @class
  bad_reify_family() {
    &Option::Some...
  }
  @class
  good_reify_concrete() {
    &Option<Int>::Some(_)
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

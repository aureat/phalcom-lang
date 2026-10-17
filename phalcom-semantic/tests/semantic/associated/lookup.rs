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
    let analysis = analyze_single_module(module, source, Arc::new(parsed.program)).expect("semantic test analysis should succeed");

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
    let analysis = analyze_single_module(module, source, Arc::new(parsed.program)).expect("semantic test analysis should succeed");

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
  Some(_ value: T)
  None
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
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program)).expect("semantic test analysis should succeed");

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
  Some(_ value: T)
  None
}

data Result<T, E> {
  Ok(_ value: T)
  Error(_ error: E)
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
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program)).expect("semantic test analysis should succeed");

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
    let analysis = analyze_single_module(module.clone(), source.clone(), program.clone()).expect("semantic test analysis should succeed");

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
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program)).expect("semantic test analysis should succeed");
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
  IntLit(_ value: Int) -> Expr<Int>
  BoolLit(_ value: Bool) -> Expr<Bool>
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
    let analysis = analyze_single_module(module, source, Arc::new(parsed.program)).expect("semantic test analysis should succeed");

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
  Some(_ value: T)
  None
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
    let analysis = analyze_single_module(module, source, Arc::new(parsed.program)).expect("semantic test analysis should succeed");

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

#[test]
fn contextual_enum_expressions_use_expected_owner_and_reject_union_ambiguity() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
data Choice<T> { Some(_ value: T) None }
data Other { None }
class Probe {
  @class
  singleton() -> Choice<Int> { None }
  @class
  payload() -> Choice<Int> { Some(42) }
  @class
  ambiguous() -> Choice<Int> | Other { None }
  @class
  no_context() { let x = None }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program)).expect("analysis");
    let probe = DeclarationId::new(module, "Probe".into());
    for name in ["singleton", "payload"] {
        let callable = CallableId::new(probe.clone(), Selector::method(name, []).unwrap(), DispatchSide::Class);
        let body = analysis.snapshot.callable_analyses.get(&callable).expect("callable body");
        assert!(
            body.associated_resolutions.values().any(|resolution| matches!(
                resolution.kind,
                AssociatedResolutionKind::ExactValue { .. } | AssociatedResolutionKind::StaticInvoke { .. }
            )),
            "contextual variant must publish canonical resolution: {:?}",
            analysis.snapshot.diagnostics
        );
    }
    let diagnostics = analysis
        .snapshot
        .diagnostics
        .values()
        .flat_map(|diagnostics| diagnostics.iter())
        .collect::<Vec<_>>();
    assert_eq!(
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == DiagnosticCode::AssociatedCallAmbiguous)
            .count(),
        1,
        "only the union context is ambiguous: {diagnostics:?}"
    );
}

#[test]
fn contextual_enum_expression_candidates_check_specialized_payloads() {
    let case = crate::semantic::adts::support::analyze_adt(
        r#"
data Boxed<T> { some(_ value: T) none }
data Text { some(_ value: String) }
data Restricted<T> { take(_ value: Bool) -> Restricted<Bool> other }
class Probe {
  @class
  forward() -> Boxed<Int> | Boxed<String> { some(1) }
  @class
  reverse() -> Boxed<String> | Boxed<Int> { some(1) }
  @class
  distinct() -> Text | Boxed<Int> { some(1) }
  @class
  gadt() -> Restricted<Int> | Restricted<Bool> { take(true) }
}
"#,
    );
    case.assert_no_diagnostics();
    let probe = DeclarationId::new(case.module.clone(), "Probe".into());
    for name in ["forward", "reverse", "distinct", "gadt"] {
        let body = &case.analysis.snapshot.callable_analyses[&CallableId::new(probe.clone(), Selector::method(name, []).unwrap(), DispatchSide::Class)];
        assert_eq!(
            body.associated_resolutions
                .values()
                .filter(|resolution| matches!(resolution.kind, AssociatedResolutionKind::StaticInvoke { .. }))
                .count(),
            1,
            "only the selected constructor may publish: {name}"
        );
    }
}

#[test]
fn contextual_enum_repeated_specialization_singleton_remains_ambiguous() {
    let case = crate::semantic::adts::support::analyze_adt(
        r#"
data Boxed<T> { some(_ value: T) none }
class Probe {
  @class
  run() -> Boxed<Int> | Boxed<String> { none }
}
"#,
    );
    let diagnostics = case.analysis.snapshot.diagnostics.values().flat_map(|items| items.iter()).collect::<Vec<_>>();
    assert_eq!(
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == DiagnosticCode::AssociatedCallAmbiguous)
            .count(),
        1
    );
    let message = &diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == DiagnosticCode::AssociatedCallAmbiguous)
        .unwrap()
        .message;
    assert!(
        message.contains("Boxed<Int>") && message.contains("Boxed<String>"),
        "specializations must be visible: {message}"
    );
}

#[test]
fn contextual_enum_candidate_probes_preserve_nested_payloads_and_flow() {
    let case = crate::semantic::adts::support::analyze_adt(
        r#"
data Boxed<T> { some(_ value: T) none }
class Probe {
  @class
  nested() -> Boxed<Boxed<Int>> | Boxed<Boxed<String>> { some(some(1)) }
  @class
  mutable() -> Boxed<Int> | Boxed<String> {
    let mut value: Boxed<Int> | Boxed<String> = some(0)
    value = some(1)
    value
  }
}
"#,
    );
    case.assert_no_diagnostics();
}

#[test]
fn contextual_enum_unknown_payload_does_not_refute_an_alternative() {
    let case = crate::semantic::adts::support::analyze_adt(
        r#"
data Boxed<T> { some(_ value: T) none }
class Probe {
  @class
  run() -> Boxed<Int> | Boxed<String> { some(missing) }
}
"#,
    );
    let diagnostics = case.analysis.snapshot.diagnostics.values().flat_map(|items| items.iter()).collect::<Vec<_>>();
    assert_eq!(
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == DiagnosticCode::AssociatedCallAmbiguous)
            .count(),
        1,
        "unknown arguments must not select a specialization: {diagnostics:?}"
    );
}

#[test]
fn contextual_enum_symbolic_candidate_keeps_its_inference_context() {
    let case = crate::semantic::adts::support::analyze_adt(
        r#"
data Boxed<T> { some(_ value: T) none }
data Text { some(_ value: String) }
class Probe {
  @class
  run() -> Boxed<Int> {
    let value: Boxed<_> | Text = some(1)
    match value { Boxed::some(item) => Boxed<Int>::some(item) }
  }
}
"#,
    );
    case.assert_no_diagnostics();
    let callable = CallableId::new(
        DeclarationId::new(case.module.clone(), "Probe".into()),
        Selector::method("run", []).unwrap(),
        DispatchSide::Class,
    );
    let body = &case.analysis.snapshot.callable_analyses[&callable];
    let item = body.bindings.values().find(|binding| binding.name == "item").unwrap();
    assert_eq!(case.analysis.snapshot.store.format_type(item.current.ty().unwrap()), "Int");
    let construction = body
        .expressions
        .values()
        .find(|expression| case.source.get(expression.range.start..expression.range.end) == Some("some(1)"))
        .unwrap();
    assert!(matches!(
        body.associated_resolutions[&construction.id].kind,
        AssociatedResolutionKind::StaticInvoke { .. }
    ));
}

#[test]
fn contextual_enum_all_payload_candidates_refuted_report_argument_mismatch() {
    let case = crate::semantic::adts::support::analyze_adt(
        r#"
data Text { make(_ value: String) }
data Boolean { make(_ value: Bool) }
class Probe { @class run() -> Text | Boolean { make(1) } }
"#,
    );
    let diagnostics = case.analysis.snapshot.diagnostics.values().flat_map(|items| items.iter()).collect::<Vec<_>>();
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::ArgumentMismatch),
        "{diagnostics:?}"
    );
    assert!(
        !diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::AssociatedGadtOwnerConflict),
        "payload mismatch must not be mislabeled as GADT conflict: {diagnostics:?}"
    );
}

#[test]
fn contextual_enum_symbolic_union_rejects_nonproper_members() {
    let case = crate::semantic::adts::support::analyze_adt(
        r#"
data Boxed<T> { some(_ value: T) none }
class Probe { @class run() { let value: Boxed<_> | Boxed = some(1) } }
"#,
    );
    assert!(
        case.diagnostics()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::AnnotationUnsaturatedConstructor),
        "a union member must be a proper type: {:?}",
        case.diagnostics().collect::<Vec<_>>()
    );
}

#[test]
fn contextual_enum_candidates_respect_private_construction() {
    let case = crate::semantic::adts::support::analyze_adt(
        r#"
data Secret { @private make(_ value: Int) @private none }
data Public { make(_ value: Int) none }
impl Secret { @class factory() -> Secret { make(1) } }
class Probe {
  @class
  constructor() -> Secret | Public { make(1) }
  @class
  singleton() -> Secret | Public { none }
}
"#,
    );
    case.assert_no_diagnostics();
}

#[test]
fn contextual_enum_private_variants_reject_qualified_acquisition() {
    let case = crate::semantic::adts::support::analyze_adt(
        r#"
data Secret { @private make(_ value: Int) @private none }
class Probe {
  @class
  constructor() { Secret::make(1) }
  @class
  singleton() { Secret::none }
  @class
  reference() { &Secret::make(_) }
}
"#,
    );
    assert_eq!(
        case.diagnostics()
            .filter(|diagnostic| diagnostic.code == DiagnosticCode::AssociatedMemberInaccessible)
            .count(),
        3,
        "all acquisition paths must enforce access: {:?}",
        case.diagnostics().collect::<Vec<_>>()
    );
}

#[test]
fn contextual_enum_canonical_contexts_preserve_qualified_identity() {
    let source = r#"
data Boxed<T> { some(_ value: T) none }
class Probe {
  @class
  accept(_ value: Boxed<Int>) -> Boxed<Int> { value }
  @class
  contextual() -> Boxed<Int> {
    let mut value: Boxed<Int> = none
    value = some(1)
    Probe.accept(some(2))
    some(3)
  }
  @class
  qualified() -> Boxed<Int> { Boxed<Int>::some(3) }
  @class
  nested() -> Boxed<Boxed<Int>> { some(some(4)) }
  @class
  shadow() -> Boxed<Int> {
    let some = |value: Int| { Boxed<Int>::some(value) }
    some(5)
  }
}
"#;
    let case = crate::semantic::adts::support::analyze_adt(source);
    case.assert_no_diagnostics();
    let probe = DeclarationId::new(case.module.clone(), "Probe".into());
    let body = |name| {
        case.analysis
            .snapshot
            .callable_analyses
            .get(&CallableId::new(probe.clone(), Selector::method(name, []).unwrap(), DispatchSide::Class))
            .expect("body")
    };
    let target = |name| {
        body(name)
            .associated_resolutions
            .values()
            .find_map(|resolution| match &resolution.kind {
                AssociatedResolutionKind::StaticInvoke { target, .. } => Some(target.clone()),
                _ => None,
            })
            .expect("constructor target")
    };
    assert_eq!(target("contextual"), target("qualified"));
    let shadow = body("shadow");
    let lexical_call = shadow
        .expressions
        .values()
        .find(|expression| case.source.get(expression.range.start..expression.range.end) == Some("some(5)"))
        .expect("lexical call expression");
    assert!(
        !shadow.associated_resolutions.contains_key(&lexical_call.id),
        "lexical callable must take precedence"
    );
}

#[test]
fn contextual_enum_reports_missing_shapes_payload_conflicts_and_no_context() {
    let case = crate::semantic::adts::support::analyze_adt(
        r#"
data Boxed<T> { some(_ value: T) none }
class Probe {
  @class
  missing() -> Boxed<Int> { absent }
  @class
  wrong_shape() -> Boxed<Int> { some(1, 2) }
  @class
  wrong_payload() -> Boxed<Int> { some("wrong") }
  @class
  no_context() { let x = none; let y = some(1) }
}
"#,
    );
    case.assert_diagnostic_primary_contains(DiagnosticCode::AssociatedMemberMissing, "absent");
    case.assert_diagnostic_primary_contains(DiagnosticCode::AssociatedCallShapeMissing, "some(1, 2)");
    case.assert_diagnostic_primary_contains(DiagnosticCode::GenericInferenceConflict, "\"wrong\"");
    let callable = CallableId::new(
        DeclarationId::new(case.module.clone(), "Probe".into()),
        Selector::method("no_context", []).unwrap(),
        DispatchSide::Class,
    );
    let body = case.analysis.snapshot.callable_analyses.get(&callable).expect("body");
    assert!(body.associated_resolutions.is_empty(), "absent context must not guess an enum");
    for text in ["none", "some(1)"] {
        let expression = body
            .expressions
            .values()
            .find(|expression| case.source.get(expression.range.start..expression.range.end) == Some(text))
            .expect("unresolved expression");
        assert!(
            matches!(
                &expression.knowledge,
                phalcom_semantic::types::evidence::TypeKnowledge::Unknown(phalcom_semantic::types::evidence::UnknownReason::UnresolvedName(_))
            ),
            "{expression:?}"
        );
    }
}

#[test]
fn contextual_enum_symbolic_owner_infers_payload_and_rejects_empty_evidence() {
    let case = crate::semantic::adts::support::analyze_adt(
        r#"
data Boxed<T> { some(_ value: T) none }
class Probe {
  @class
  payload() -> Boxed<Int> { let x: Boxed<_> = some(1); x }
  @class
  singleton() { let x: Boxed<_> = none }
}
"#,
    );
    let callable = CallableId::new(
        DeclarationId::new(case.module.clone(), "Probe".into()),
        Selector::method("payload", []).unwrap(),
        DispatchSide::Class,
    );
    let body = case.analysis.snapshot.callable_analyses.get(&callable).expect("body");
    let expression = body
        .expressions
        .values()
        .find(|expression| case.source.get(expression.range.start..expression.range.end) == Some("some(1)"))
        .expect("payload expression");
    assert!(
        body.associated_resolutions.contains_key(&expression.id),
        "symbolic owner must resolve: {:?}",
        case.analysis.snapshot.diagnostics
    );
    let TypeData::ExactCase { enum_type, .. } = case.analysis.snapshot.store.get(expression.knowledge.ty().expect("inferred payload")) else {
        panic!("constructor result must retain its exact-case type");
    };
    assert_eq!(case.analysis.snapshot.store.format_type(*enum_type), "Boxed<Int>");
    case.assert_diagnostic_primary_contains(DiagnosticCode::AssociatedGenericUnderconstrained, "none");
}

#[test]
fn contextual_enum_gadt_union_excludes_refuted_owners() {
    let case = crate::semantic::adts::support::analyze_adt(
        r#"
data Restricted<T> { take(_ value: Bool) -> Restricted<Bool> }
data Plain { take(_ value: Int) }
class Probe {
  @class
  run() -> Restricted<Int> | Plain { take(42) }
}
"#,
    );
    case.assert_no_diagnostics();
    let callable = CallableId::new(
        DeclarationId::new(case.module.clone(), "Probe".into()),
        Selector::method("run", []).unwrap(),
        DispatchSide::Class,
    );
    let body = case.analysis.snapshot.callable_analyses.get(&callable).expect("body");
    assert!(
        body.associated_resolutions
            .values()
            .any(|resolution| resolution.lookup_owner.name.as_ref() == "Plain")
    );
}

#[test]
fn contextual_enum_if_and_while_patterns_preserve_annotations_and_assignments() {
    let case = crate::semantic::adts::support::analyze_adt(
        r#"
data Boxed<T> { some(_ value: T) none }
class Probe {
  @class
  conditional(_ value: Boxed<Int>) -> Int {
    if let some(item) = value { item } else { 0 }
  }
  @class
  loop(_ value: Boxed<Int>) -> Int {
    let mut remaining: Boxed<Int> = value
    let mut result: Int = 0
    while let some(item) = remaining {
      result = item
      remaining = none
    }
    result
  }
}
"#,
    );
    case.assert_no_diagnostics();
}

#[test]
fn contextual_enum_impossible_gadt_case_reports_conflicting_specialization() {
    let case = crate::semantic::adts::support::analyze_adt(
        r#"
data Expr<T> { boolean(_ value: Bool) -> Expr<Bool> integer(_ value: Int) -> Expr<Int> }
class Probe { @class run() -> Expr<Int> { boolean(true) } }
"#,
    );
    case.assert_diagnostic_primary_contains(DiagnosticCode::AssociatedGadtOwnerConflict, "boolean(true)");
    assert!(case.diagnostics_for(DiagnosticCode::AssociatedCallAmbiguous).is_empty());
}

#[test]
fn contextual_enum_generic_owner_preserves_parameterized_expectation() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
data Wrapped<T> {
  wrap(_ value: T)
  empty
}

class Holder<T> {
  mut _current: Wrapped<T>

  @constructor
  new(_ value: T) {
    _current = wrap(value)
  }

  take(_ value: T) -> Wrapped<T> {
    return wrap(value)
  }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "generic contextual fixture must parse: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program)).expect("semantic generic contextual fixture publishes");
    let diagnostics = analysis.snapshot.diagnostics_for(&module).unwrap_or_default();
    assert!(
        !diagnostics.iter().any(|d| {
            matches!(
                d.code,
                DiagnosticCode::GenericInferenceConflict | DiagnosticCode::AssociatedGenericOwnerConflict | DiagnosticCode::ArgumentMismatch
            )
        }),
        "contextual construction within a generic owner must preserve T: {diagnostics:#?}"
    );
}

#[test]
fn contextual_enum_completion_generic_trait_projection_survives_while_let() {
    let case = crate::semantic::adts::support::analyze_adt(
        r#"
data Envelope<T> { some(_ value: T) none }
trait Stream {
  type Item
  next -> Envelope<Self::Item>
}
class Probe {
  @class
  consume<I>(_ source: I) -> Int
    where I |= Stream
  {
    let mut cursor: I = source
    let mut count: Int = 0
    while let some(item) = cursor.next {
      let preserved: <I as Stream>::Item = item
      count = count + 1
    }
    count
  }
}
"#,
    );
    case.assert_no_diagnostics();
}

#[test]
fn contextual_enum_completion_mutable_generic_flow_preserves_case_owner() {
    let case = crate::semantic::adts::support::analyze_adt(
        r#"
data Envelope<T> { some(_ value: T) none }
class Carrier<T> {
  @class
  advance(_ input: T) -> Envelope<T> {
    let mut state: Envelope<T> = some(input)
    if let some(value) = state {
      state = none
    }
    state = some(input)
    while let some(value) = state {
      let preserved: T = value
      state = none
    }
    state
  }
}
"#,
    );
    case.assert_no_diagnostics();
}

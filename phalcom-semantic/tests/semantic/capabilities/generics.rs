use crate::semantic::support::{Fixture, binding, known};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;

/// LAW: one argument-derived substitution specializes the generic return.
#[test]
fn generic_identity_solves_parameter_from_argument_and_specializes_return() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  identity<T>(_ value: T) -> T {
    value
  }

  @class
  run() {
    let x = Probe.identity(42)
    let y = Probe.identity("hello")
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_expression_knowledge(
        f.expression(run, "Probe.identity(42)"),
        known(int_ty).established().origin(phalcom_semantic::EvidenceOrigin::GenericInference),
    );
    f.assert_expression_knowledge(
        f.expression(run, "Probe.identity(\"hello\")"),
        known(string_ty).established().origin(phalcom_semantic::EvidenceOrigin::GenericInference),
    );
    f.assert_binding_established(run, "x", int_ty);
    f.assert_binding_established(run, "y", string_ty);
}

/// LAW: independent generic variables retain independent argument evidence.
#[test]
fn generic_pair_solves_two_independent_variables() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  pair<A, B>(_ a: A, _ b: B) -> (A, B) {
    (a, b)
  }

  @class
  run() {
    let x = Probe.pair(1, "hello")
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_tuple_types(f.binding(run, "x").current.ty().expect("pair result"), &[int_ty, string_ty]);
    f.assert_expression_knowledge(
        f.expression(run, "Probe.pair(1, \"hello\")"),
        known(f.binding(run, "x").current.ty().expect("pair result"))
            .established()
            .origin(phalcom_semantic::EvidenceOrigin::GenericInference),
    );
}

/// LAW: expected context constrains a call without overwriting its precise fact.
#[test]
fn expected_result_context_constrains_generic_without_merely_overwriting_call_fact() {
    let f = Fixture::new(
        r#"
class Factory {
  @class
  choose<T>(_ value: T) -> T {
    value
  }
}

class Probe {
  @class
  run() {
    let x: Number = Factory.choose(42)
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let number = f.ty("Number");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "Factory.choose(42)");
    f.assert_expression_knowledge(call, known(int_ty).assumed().origin(phalcom_semantic::EvidenceOrigin::GenericInference));
    let x = f.binding(run, "x");
    assert_eq!(x.declared_type(), Some(number));
    assert_eq!(x.current.ty(), Some(int_ty));
    f.assert_no_error_diagnostics();
}

/// LAW: conflicting generic constraints retain actual evidence and diagnose.
#[test]
fn conflicting_generic_constraints_are_refuted_instead_of_using_expected_annotation_as_fact() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  identity<T>(_ value: T) -> T {
    value
  }

  @class
  run() {
    let x: Int = Probe.identity("wrong")
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "Probe.identity(\"wrong\")");
    assert_eq!(call.knowledge.ty(), None, "conflicting constraints must not publish a call result");
    assert!(
        matches!(call.status, AnalysisStatus::Invalid(_)),
        "expected-result contradiction must own call invalidity"
    );
    assert!(
        !f.diagnostics(DiagnosticCode::BindingInitializerMismatch).is_empty() || !f.diagnostics(DiagnosticCode::GenericInferenceConflict).is_empty(),
        "conflicting constraints should produce an owning diagnostic"
    );
    f.assert_only_error_codes(&[DiagnosticCode::GenericInferenceConflict]);
}

/// LAW: assumed input evidence yields an assumed generic return.
#[test]
fn assumed_generic_argument_yields_assumed_generic_return() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  identity<T>(_ value: T) -> T {
    value
  }

  @class
  run(_ value: Int) {
    let result = Probe.identity(value)
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let value = f.binding(run, "value");
    assert_eq!(value.current.ty(), Some(int_ty));
    assert_eq!(value.current.status(), Some(phalcom_semantic::EvidenceStatus::Assumed));
    let result = f.binding(run, "result");
    assert_eq!(result.current.ty(), Some(int_ty));
    assert_eq!(result.current.status(), Some(phalcom_semantic::EvidenceStatus::Assumed));
    assert_eq!(result.current.origin(), Some(phalcom_semantic::EvidenceOrigin::GenericInference));
}

/// LAW: a composite generic result takes weakest supporting evidence.
#[test]
fn mixed_generic_return_uses_weakest_value_support() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  pair<A, B>(_ first: A, _ second: B) -> (A, B) {
    (first, second)
  }

  @class
  run(_ value: Int) {
    let result = Probe.pair(value, 42)
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let result = f.binding(run, "result");
    let result_ty = result.current.ty().expect("mixed generic result");
    f.assert_tuple_types(result_ty, &[int_ty, int_ty]);
    assert_eq!(result.current.status(), Some(phalcom_semantic::EvidenceStatus::Assumed));
}

/// LAW: fixed return evidence stays established despite assumed generic input.
#[test]
fn independent_fixed_generic_return_stays_established() {
    let f = Fixture::new(
        r#"
class Result {
  @constructor new() {}
}
class Probe {
  @class
  fixed<T>(_ value: T) -> Result {
    Result.new()
  }

  @class
  run(_ value: Int) {
    let result: Int = Probe.fixed(value)
  }
}
"#,
    );
    let result_ty = f.ty("Result");
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "Probe.fixed(value)");
    assert_eq!(call.knowledge.ty(), Some(result_ty));
    assert!(
        matches!(call.status, AnalysisStatus::Invalid(_)),
        "expected-result conflict must retain fixed return invalidity"
    );
    let result = f.binding(run, "result");
    assert_eq!(result.current.ty(), Some(result_ty));
    assert_eq!(result.current.status(), Some(phalcom_semantic::EvidenceStatus::Established));
    assert_eq!(result.declared_type(), Some(int_ty));
}

/// LAW: expected context selects a result-only generic but contributes only assumed evidence.
#[test]
fn expected_context_selects_but_does_not_establish_result_only_generic() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  make<T>() -> T {
    42
  }

  @class
  run() {
    let result: Int = Probe.make()
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "Probe.make()");
    f.assert_expression_knowledge(call, known(f.ty("Int")).assumed().origin(phalcom_semantic::EvidenceOrigin::GenericInference));
    assert!(matches!(call.status, AnalysisStatus::Ready), "{call:#?}");
    f.assert_no_diagnostic(DiagnosticCode::GenericInferenceUnderconstrained);
}

/// G03: repeated calls in one body solve independent substitutions independently.
#[test]
fn generic_calls_in_one_body_do_not_share_type_variables() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  identity<T>(_ value: T) -> T { value }

  @class
  run() {
    let number = Probe.identity(1)
    let text = Probe.identity("text")
    let copied = Probe.identity(number)
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_established(run, "number", int_ty);
    f.assert_binding_established(run, "text", string_ty);
    f.assert_binding_established(run, "copied", int_ty);
}

/// G06: a broad expected contract validates a precise generic result without widening it.
#[test]
fn generic_expected_contract_keeps_narrow_current_after_multiple_uses() {
    let f = Fixture::new(
        r#"
class Animal {}
class Cat is Animal { @constructor new() {} }
class Probe {
  @class
  identity<T>(_ value: T) -> T { value }

  @class
  run() {
    let value: Animal = Probe.identity(Cat.new())
    let observed = value
  }
}
"#,
    );
    let animal = f.ty("Animal");
    let cat = f.ty("Cat");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let value = f.binding(run, "value");
    assert_eq!(value.declared_type(), Some(animal));
    assert_eq!(value.current.ty(), Some(cat));
    f.assert_binding_type(run, "observed", cat);
}

/// G07: a conflict in one generic call does not corrupt another call's solution.
#[test]
fn generic_conflict_is_local_to_call_and_sibling_remains_established() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  identity<T>(_ value: T) -> T { value }

  @class
  run() {
    let bad: Int = Probe.identity("wrong")
    let good = Probe.identity(7)
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_established(run, "good", int_ty);
    assert!(f.binding(run, "bad").current.ty().is_none(), "conflicting call must not publish a result type");
}

/// E03: a parameter contract remains assumed when it drives generic inference.
#[test]
fn generic_result_retains_assumed_parameter_support() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  identity<T>(_ value: T) -> T { value }

  @class
  run(_ value: String) {
    let result = Probe.identity(value)
  }
}
"#,
    );
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_expectation(run, "result", binding().current(known(string_ty).assumed()));
}

#[test]
fn higher_kinded_parameter_infers_list_constructor_and_element() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  keep<F: Type -> Type, A>(_ value: F<A>) -> F<A> { value }

  @class
  run() {
    let result = Probe.keep([1])
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let result_ty = f.binding(run, "result").current.ty().expect("HKT result");
    f.assert_type(result_ty, crate::semantic::support::applied("List", [crate::semantic::support::nominal("Int")]));
    let call = f.expression(run, "Probe.keep([1])");
    assert!(matches!(call.status, AnalysisStatus::Ready), "{call:#?}");
    assert_eq!(f.binding(run, "result").current.ty(), Some(result_ty));
    f.assert_no_error_diagnostics();
}

#[test]
fn wrong_kind_higher_kinded_argument_is_rejected_structurally() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  keep<F: Type -> Type, A>(_ value: F<A>) -> F<A> { value }

  @class
  run(_ value: Int) {
    let bad = Probe.keep(value)
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "Probe.keep(value)");
    assert!(!matches!(call.status, AnalysisStatus::Ready), "wrong-kind HKT call must not pass: {call:#?}");
    assert!(
        f.analysis
            .snapshot
            .all_diagnostics()
            .any(|diagnostic| diagnostic.severity == phalcom_semantic::diagnostic::DiagnosticSeverity::Error)
    );
}

#[test]
fn closed_tuple_shape_exposes_nested_generic_argument() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  keep<T>(_ value: (T, String)) -> (T, String) { value }

  @class
  run(_ value: (Int, String)) {
    let result = Probe.keep(value)
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let result = f.binding(run, "result").current.ty().expect("tuple result");
    f.assert_tuple_types(result, &[f.ty("Int"), f.ty("String")]);
    f.assert_expression_ready(f.expression(run, "Probe.keep(value)"));
    f.assert_no_error_diagnostics();
}

#[test]
fn universe_future_preserves_payload_types_through_async_map_then_and_await() {
    let source = format!(
        "{}\n{}",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../phalcom-core/core/universe/src/concurrency/fiber.ph")),
        r#"
class GenericFutureProbe {
  @class
  run() {
    const created = Future.async(|| { 42 })
    const fromValue = Future.value(42)
    const pending: Future<Int> = Future.new()
    const awaited = created.await
    const mapped = created.map(|value| { "mapped" })
    const chained = created.then(|value| { Future.async(|| { "chained" }) })
    const nested = created.map(|value| { Future.async(|| { 7 }) })
    const unit = Future.async(|| { () })
    const cs: CompletionSource<Int> = CompletionSource.new()
    const csFuture = cs.future
    const resolved = cs.tryResolve(100)
    const allProbe = Future.all([created, fromValue])
    const allSettledProbe = Future.allSettled([created, fromValue])
    const raceProbe = Future.race([created, fromValue])
    const timedProbe = created.timeout(100)
  }
}
"#
    );
    let f = Fixture::new(&source);
    f.assert_no_internal_incidents();
    let run = f.callable("GenericFutureProbe", "run", DispatchSide::Class);
    use crate::semantic::support::{applied, nominal};
    for (name, expected) in [
        ("created", applied("Future", [nominal("Int")])),
        ("fromValue", applied("Future", [nominal("Int")])),
        ("pending", applied("Future", [nominal("Int")])),
        ("mapped", applied("Future", [nominal("String")])),
        ("chained", applied("Future", [nominal("String")])),
        ("nested", applied("Future", [applied("Future", [nominal("Int")])])),
        ("cs", applied("CompletionSource", [nominal("Int")])),
        ("csFuture", applied("Future", [nominal("Int")])),
        ("resolved", nominal("Bool")),
        ("allProbe", applied("Future", [applied("List", [nominal("Int")])])),
        (
            "allSettledProbe",
            applied("Future", [applied("List", [applied("Result", [nominal("Int"), nominal("Error")])])]),
        ),
        ("raceProbe", applied("Future", [nominal("Int")])),
        ("timedProbe", applied("Future", [nominal("Int")])),
    ] {
        let ty = f.binding(run, name).current.ty().unwrap_or_else(|| panic!("{name} must retain a formal type"));
        f.assert_type(ty, expected);
    }
    assert_eq!(f.binding(run, "awaited").current.ty(), Some(f.ty("Int")));
    let unit = f.binding(run, "unit").current.ty().expect("Future<Unit>");
    let phalcom_semantic::types::TypeData::Applied { arguments, .. } = f.analysis.snapshot.store.get(unit) else {
        panic!("expected applied Future<Unit>");
    };
    assert_eq!(arguments.as_ref(), &[f.analysis.snapshot.store.unit()]);
    f.assert_no_error_diagnostics();
}

#[test]
fn universe_future_rejects_incompatible_settlement_chaining_and_recovery() {
    let source = format!(
        "{}\n{}",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../phalcom-core/core/universe/src/concurrency/fiber.ph")),
        r#"
class InvalidFutureProbe {
  @class
  run(_ input: Future<Int>, _ cs: CompletionSource<Int>) {
    input.settleValue("wrong")
    input.then(|value| { 42 })
    input.catch(|error| { "wrong" })
    cs.tryResolve("wrong")
  }
}
"#,
    );
    let f = Fixture::new(&source);
    for expression in [
        "input.settleValue(\"wrong\")",
        "input.then(|value| { 42 })",
        "input.catch(|error| { \"wrong\" })",
        "cs.tryResolve(\"wrong\")",
    ] {
        let start = source.find(expression).expect("probe expression");
        let end = start + expression.len();
        assert!(
            f.analysis.snapshot.all_diagnostics().any(|diagnostic| {
                diagnostic.severity == phalcom_semantic::diagnostic::DiagnosticSeverity::Error
                    && diagnostic.primary_range.start < end
                    && diagnostic.primary_range.end > start
            }),
            "{expression} must be rejected, diagnostics: {:?}",
            f.analysis.snapshot.diagnostics,
        );
    }
}

/// LAW: unsaturated generic constructors in proper-type positions emit clean
/// diagnostics (`AnnotationUnsaturatedConstructor`) without compiler or type-store panic.
#[test]
fn unsaturated_generic_constructors_are_cleanly_rejected_in_proper_type_positions() {
    let source = format!(
        "{}\n{}",
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../phalcom-core/core/universe/src/concurrency/fiber.ph")),
        r#"
class ProperTypeProbe {
  @class
  run(_ bareFuture: Future, _ unionWithBare: Future | Int) {
  }
}
"#,
    );
    let f = Fixture::new(&source);
    let diagnostics: Vec<_> = f
        .analysis
        .snapshot
        .all_diagnostics()
        .filter(|d| d.severity == phalcom_semantic::diagnostic::DiagnosticSeverity::Error)
        .collect();
    assert!(!diagnostics.is_empty(), "bare generic constructor must emit diagnostics");
    let codes: Vec<_> = diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&DiagnosticCode::KindExpectedType),
        "bare parameter annotation must emit KindExpectedType; codes: {:?}",
        codes
    );
    assert!(
        codes.contains(&DiagnosticCode::AnnotationUnsaturatedConstructor),
        "union member position must emit AnnotationUnsaturatedConstructor; codes: {:?}",
        codes
    );
}

/// LAW: generic callable return inference successfully solves `R` from exact-domain
/// callables, establishing why arbitrary parameter packs require full pack polymorphism.
#[test]
fn generic_callable_return_inference_requires_exact_parameter_domain() {
    let f = Fixture::new(
        r#"
class CallableProbe {
  @class
  spawnZero<R>(_ body: () -> R) -> R {
    body()
  }

  @class
  spawnUnary<A, R>(_ body: (A) -> R, _ arg: A) -> R {
    body(arg)
  }

  @class
  run() {
    let a = CallableProbe.spawnZero(|| { 42 })
    let b = CallableProbe.spawnUnary(|x| { "res:" + x.toString }, 10)
  }
}
"#,
    );
    let run = f.callable("CallableProbe", "run", DispatchSide::Class);
    assert_eq!(f.binding(run, "a").current.ty(), Some(f.ty("Int")));
    assert_eq!(f.binding(run, "b").current.ty(), Some(f.ty("String")));
    f.assert_no_error_diagnostics();
}

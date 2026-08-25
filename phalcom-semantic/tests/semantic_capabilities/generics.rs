use crate::support::Fixture;
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;

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
    f.assert_binding_established(run, "x", int_ty);
    f.assert_binding_established(run, "y", string_ty);
}

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
}

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
    f.assert_expression_established(call, int_ty);
    let x = f.binding(run, "x");
    assert_eq!(x.declared, Some(number));
    assert_eq!(x.current.ty(), Some(int_ty));
}

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
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "Probe.identity(\"wrong\")");
    assert_eq!(
        call.knowledge.ty(),
        Some(string_ty),
        "argument-derived generic fact must survive expected-result contradiction"
    );
    assert!(
        matches!(call.status, AnalysisStatus::Invalid(_)),
        "expected-result contradiction must own call invalidity"
    );
    assert!(
        !f.diagnostics(DiagnosticCode::BindingInitializerMismatch).is_empty() || !f.diagnostics(DiagnosticCode::ArgumentMismatch).is_empty(),
        "conflicting constraints should produce an owning diagnostic"
    );
}

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
    assert_eq!(result.declared, Some(int_ty));
}

#[test]
fn expected_context_cannot_fabricate_missing_generic_return() {
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
    assert_eq!(call.knowledge.ty(), None, "expected context cannot fabricate generic return evidence");
    assert!(matches!(call.status, AnalysisStatus::Ready));
}

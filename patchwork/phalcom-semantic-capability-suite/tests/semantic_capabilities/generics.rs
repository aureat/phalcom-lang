use crate::support::Fixture;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;

#[test]
fn generic_identity_solves_parameter_from_argument_and_specializes_return() {
    let f = Fixture::new(r#"
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
"#);
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_established(run, "x", int_ty);
    f.assert_binding_established(run, "y", string_ty);
}

#[test]
fn generic_pair_solves_two_independent_variables() {
    let f = Fixture::new(r#"
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
"#);
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_tuple_types(f.binding(run, "x").current.ty().expect("pair result"), &[int_ty, string_ty]);
}

#[test]
fn expected_result_context_constrains_generic_without_merely_overwriting_call_fact() {
    let f = Fixture::new(r#"
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
"#);
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
    let f = Fixture::new(r#"
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
"#);
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "Probe.identity(\"wrong\")");
    assert_eq!(call.knowledge.ty(), Some(string_ty), "argument-derived generic fact must survive expected-result contradiction");
    assert!(
        !f.diagnostics(DiagnosticCode::BindingInitializerMismatch).is_empty()
            || !f.diagnostics(DiagnosticCode::ArgumentMismatch).is_empty(),
        "conflicting constraints should produce an owning diagnostic"
    );
}

use crate::support::Fixture;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;

#[test]
fn chained_dispatch_preserves_constructor_specialization_without_binding_storage() {
    let f = Fixture::new(r#"
class Base {}
class Derived is Base {
  @constructor new() {}
  derivedOnly() -> Int { 1 }
}
class Probe {
  @class
  run() {
    let y = Derived.new().derivedOnly()
  }
}
"#);
    let int_ty = f.ty("Int");
    let derived = f.ty("Derived");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_expression_established(f.expression(run, "Derived.new()"), derived);
    f.assert_expression_established(f.expression(run, "Derived.new().derivedOnly()"), int_ty);
    f.assert_binding_established(run, "y", int_ty);
}

#[test]
fn multiple_hop_call_chain_preserves_each_intermediate_result() {
    let f = Fixture::new(r#"
class C {
  @constructor new() {}
  value() -> Int { 1 }
}
class B {
  @constructor new() {}
  makeC() -> C { C.new() }
}
class A {
  @constructor new() {}
  makeB() -> B { B.new() }
}
class Probe {
  @class
  run() {
    let x = A.new().makeB().makeC().value()
  }
}
"#);
    let a = f.ty("A");
    let b = f.ty("B");
    let c = f.ty("C");
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_expression_established(f.expression(run, "A.new()"), a);
    f.assert_expression_established(f.expression(run, "A.new().makeB()"), b);
    f.assert_expression_established(f.expression(run, "A.new().makeB().makeC()"), c);
    f.assert_expression_established(f.expression(run, "A.new().makeB().makeC().value()"), int_ty);
}

#[test]
fn wrong_class_instance_dispatch_side_is_not_laundered_into_dynamic_unknown() {
    let f = Fixture::new(r#"
class CellNum {
  @constructor new() {}
  instanceOnly() -> Int { 1 }
  @class
  classOnly() -> Int { 1 }
}
class Probe {
  @class
  run() {
    let okClass = CellNum.classOnly()
    let okInstance = CellNum.new().instanceOnly()
    let badClass = CellNum.instanceOnly()
    let badInstance = CellNum.new().classOnly()
  }
}
"#);
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_type(run, "okClass", int_ty);
    f.assert_binding_type(run, "okInstance", int_ty);
    let bad_class = f.binding(run, "badClass");
    let bad_instance = f.binding(run, "badInstance");
    assert!(bad_class.current.is_unknown() || bad_class.causal_invalidity.suppression_cause().is_some());
    assert!(bad_instance.current.is_unknown() || bad_instance.causal_invalidity.suppression_cause().is_some());
}

#[test]
fn selector_label_mismatch_is_distinguished_from_argument_type_mismatch() {
    let f = Fixture::new(r#"
class Probe {
  @class
  consume(value: Int) -> Int { value }

  @class
  run() {
    let good = Probe.consume(value: 1)
    let wrongShape = Probe.consume(1)
  }
}
"#);
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_type(run, "good", int_ty);
    let wrong = f.binding(run, "wrongShape");
    assert!(wrong.current.is_unknown() || wrong.causal_invalidity.suppression_cause().is_some());
    assert!(
        f.diagnostics(DiagnosticCode::ArgumentMismatch).is_empty(),
        "selector-shape failure should not be misreported as an argument type mismatch"
    );
}

#[test]
fn argument_refutation_preserves_independently_known_call_return_type() {
    let f = Fixture::new(r#"
class CellNum {
  @constructor new() {}

  @class
  fromInt(_ value: Int) -> CellNum {
    CellNum.new()
  }
}
class Probe {
  @class
  run() {
    let x = CellNum.fromInt("bad")
  }
}
"#);
    let cell_num = f.ty("CellNum");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_expression_established(f.expression(run, "CellNum.fromInt(\"bad\")"), cell_num);
    f.assert_binding_type(run, "x", cell_num);
    assert_eq!(f.diagnostics(DiagnosticCode::ArgumentMismatch).len(), 1);
}

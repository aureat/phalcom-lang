use crate::semantic::support::{Fixture, known};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;

/// LAW: constructor specialization feeds the next instance-side dispatch.
#[test]
fn chained_dispatch_preserves_constructor_specialization_without_binding_storage() {
    let f = Fixture::new(
        r#"
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
"#,
    );
    let int_ty = f.ty("Int");
    let derived = f.ty("Derived");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let constructor = f.expression(run, "Derived.new()");
    let method = f.expression(run, "Derived.new().derivedOnly()");
    f.assert_expression_knowledge(
        constructor,
        known(derived).established().origin(phalcom_semantic::EvidenceOrigin::ConstructorSemantics),
    );
    f.assert_expression_knowledge(method, known(int_ty).established().origin(phalcom_semantic::EvidenceOrigin::CallableSignature));
    assert!(
        constructor.callable.is_some(),
        "constructor call must resolve to a canonical callable: {constructor:#?}"
    );
    f.assert_expression_call_target(method, &f.callable_id("Derived", "derivedOnly", DispatchSide::Instance));
    f.assert_expression_established(constructor, derived);
    f.assert_expression_established(method, int_ty);
    f.assert_binding_established(run, "y", int_ty);
    f.assert_no_error_diagnostics();
}

/// LAW: every resolved hop preserves its intermediate receiver/result type.
#[test]
fn multiple_hop_call_chain_preserves_each_intermediate_result() {
    let f = Fixture::new(
        r#"
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
"#,
    );
    let a = f.ty("A");
    let b = f.ty("B");
    let c = f.ty("C");
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_expression_established(f.expression(run, "A.new()"), a);
    f.assert_expression_established(f.expression(run, "A.new().makeB()"), b);
    f.assert_expression_established(f.expression(run, "A.new().makeB().makeC()"), c);
    f.assert_expression_established(f.expression(run, "A.new().makeB().makeC().value()"), int_ty);
    f.assert_no_error_diagnostics();
}

/// LAW: side mismatch remains unresolved and cannot become dynamic certainty.
#[test]
fn wrong_class_instance_dispatch_side_is_not_laundered_into_dynamic_unknown() {
    let f = Fixture::new(
        r#"
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
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_type(run, "okClass", int_ty);
    f.assert_binding_type(run, "okInstance", int_ty);
    let bad_class = f.binding(run, "badClass");
    let bad_instance = f.binding(run, "badInstance");
    assert!(bad_class.current.is_unknown() || bad_class.causal_invalidity.suppression_cause().is_some());
    assert!(bad_instance.current.is_unknown() || bad_instance.causal_invalidity.suppression_cause().is_some());
    f.assert_diagnostic(DiagnosticCode::ArgumentMismatch, 0);
}

/// LAW: selector shape errors stay distinct from argument type errors.
#[test]
fn selector_label_mismatch_is_distinguished_from_argument_type_mismatch() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  consume(value: Int) -> Int { value }

  @class
  run() {
    let good = Probe.consume(value: 1)
    let wrongShape = Probe.consume(1)
  }
}
"#,
    );
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

/// LAW: argument refutation does not erase an independently known return.
#[test]
fn argument_refutation_preserves_independently_known_call_return_type() {
    let f = Fixture::new(
        r#"
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
"#,
    );
    let cell_num = f.ty("CellNum");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_expression_established(f.expression(run, "CellNum.fromInt(\"bad\")"), cell_num);
    f.assert_binding_type(run, "x", cell_num);
    assert_eq!(f.diagnostics(DiagnosticCode::ArgumentMismatch).len(), 1);
}

/// D04: inherited instance dispatch resolves the defining callable and retains its result.
#[test]
fn inherited_instance_dispatch_keeps_defining_callable_identity() {
    let f = Fixture::new(
        r#"
class Base {
  value() -> Int { 1 }
}
class Derived is Base {
  @constructor new() {}
}
class Probe {
  @class
  run() {
    let result = Derived.new().value()
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "Derived.new().value()");
    f.assert_expression_established(call, int_ty);
    f.assert_expression_call_target(call, &f.callable_id("Base", "value", DispatchSide::Instance));
}

/// D01: receiver specialization survives an intermediate call before final dispatch.
#[test]
fn receiver_specialization_survives_two_instance_dispatch_hops() {
    let f = Fixture::new(
        r#"
class Leaf {
  @constructor new() {}
  value() -> String { "leaf" }
}
class Wrapper {
  @constructor new() {}
  leaf() -> Leaf { Leaf.new() }
}
class Probe {
  @class
  run() {
    let result = Wrapper.new().leaf().value()
  }
}
"#,
    );
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_established(run, "result", string_ty);
    f.assert_expression_call_target(
        f.expression(run, "Wrapper.new().leaf().value()"),
        &f.callable_id("Leaf", "value", DispatchSide::Instance),
    );
}

/// D08: one invalid call argument leaves a sibling call fully analyzable.
#[test]
fn invalid_dispatch_argument_does_not_suppress_sibling_dispatch() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  consume(_ value: Int) -> String { "ok" }

  @class
  run() {
    let bad = Probe.consume("wrong")
    let good = Probe.consume(1)
  }
}
"#,
    );
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_type(run, "good", string_ty);
    assert_eq!(f.binding(run, "bad").current.ty(), Some(string_ty));
}

/// COMPOSED: constructor specialization and both `super` dispatch sides retain identity.
#[test]
fn constructor_super_chain_preserves_instance_and_class_side_results() {
    let f = Fixture::new(
        r#"
class Base {
  @constructor new() {}

  value(_ n: Int) -> Int { n }

  @class
  label() -> String { "base" }
}

class Derived is Base {
  value(_ n: Int) -> Int { super.value(n) + 1 }

  @class
  label() -> String { super.label() }
}

class Probe {
  @class
  run() {
    let object = Derived.new()
    let number = object.value(4)
    let text = Derived.label()
  }
}
"#,
    );
    let derived = f.ty("Derived");
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);

    f.assert_binding_established(run, "object", derived);
    f.assert_binding_established(run, "number", int_ty);
    f.assert_binding_established(run, "text", string_ty);

    let derived_value = f.callable("Derived", "value", DispatchSide::Instance);
    f.assert_expression_call_target(
        f.expression(derived_value, "super.value(n)"),
        &f.callable_id("Base", "value", DispatchSide::Instance),
    );
    let derived_label = f.callable("Derived", "label", DispatchSide::Class);
    f.assert_expression_call_target(
        f.expression(derived_label, "super.label()"),
        &f.callable_id("Base", "label", DispatchSide::Class),
    );
    f.assert_no_error_diagnostics();
}

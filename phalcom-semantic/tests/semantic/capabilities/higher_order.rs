//! Higher-order callable capability probes.

use crate::semantic::support::{Fixture, binding, known};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::EvidenceOrigin;

/// COMPOSED: a callback invocation should publish its body result, not only its closure object type.
#[test]
fn higher_order_block_call_propagates_captured_result() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  apply(_ value: Int) {
    const increment = || { value + 1 }
    increment.call()
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let apply = f.callable("Probe", "apply", DispatchSide::Class);
    f.assert_expression_established(f.expression(apply, "increment.call()"), int_ty);
    f.assert_normal_return(
        apply,
        crate::semantic::support::known(int_ty)
            .established()
            .origin(phalcom_semantic::EvidenceOrigin::Flow),
    );
    f.assert_no_error_diagnostics();
}

#[test]
fn direct_and_explicit_call_on_same_block_publish_same_result() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ value: Int) {
    const direct = || { value + 1 }
    const explicit = || { value + 1 }
    let a = direct()
    let b = explicit.call()
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_established(run, "a", int_ty);
    f.assert_binding_established(run, "b", int_ty);
}

#[test]
fn nominal_call_method_does_not_use_structural_callable_shortcut() {
    let f = Fixture::new(
        r#"
class Fun {
  call() -> String { "nominal" }
}

class Probe {
  @class
  run() {
    let fun = Fun()
    fun.call()
  }
}
"#,
    );
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_expression_established(f.expression(run, "fun.call()"), string_ty);
}

#[test]
fn generic_call_contextualizes_closure_input_and_result() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  apply<T, U>(_ value: T, _ transform: (T) -> U) -> U {
    transform.call(value)
  }

  @class
  run() {
    let result = Probe.apply(1, |value| { value + 1 })
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "Probe.apply(1, |value| { value + 1 })");
    assert_eq!(call.knowledge.ty(), Some(int_ty));
    assert_eq!(call.knowledge.origin(), Some(EvidenceOrigin::GenericInference));
    assert!(matches!(call.status, AnalysisStatus::Ready), "{call:#?}");
    f.assert_binding_expectation(run, "result", binding().current(known(int_ty)));
    f.assert_no_error_diagnostics();
}

#[test]
fn nested_generic_call_uses_isolated_inner_application() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  identity<T>(_ value: T) -> T { value }

  @class
  apply<T, U>(_ value: T, _ transform: (T) -> U) -> U {
    transform.call(value)
  }

  @class
  run() {
    let result = Probe.apply(1, |value| { Probe.identity(value) })
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let outer = f.expression(run, "Probe.apply(1, |value| { Probe.identity(value) })");
    let inner = f.expression(run, "Probe.identity(value)");
    assert_eq!(outer.knowledge.ty(), Some(int_ty));
    assert_eq!(inner.knowledge.ty(), Some(int_ty));
    assert!(matches!(outer.status, AnalysisStatus::Ready), "{outer:#?}");
    assert!(matches!(inner.status, AnalysisStatus::Ready), "{inner:#?}");
    f.assert_no_error_diagnostics();
}

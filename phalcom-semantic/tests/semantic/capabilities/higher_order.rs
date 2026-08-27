//! Higher-order callable capability probes.

use crate::semantic::support::Fixture;
use phalcom_semantic::identity::DispatchSide;

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

#[path = "dispatch_capabilities.rs"]
mod capability_dispatch;
#[path = "dispatch_class_side.rs"]
mod class_side_dispatch;

use crate::semantic::support::Fixture;
use phalcom_semantic::identity::DispatchSide;

/// LAW: an ordinary user-defined `ifTrue(_:ifFalse:)` send keeps dynamic
/// dispatch semantics even when both arguments are literal blocks.
#[test]
fn ordinary_if_true_selector_on_non_bool_receiver_is_not_control_flow() {
    let f = Fixture::new(
        r#"
class Strange {
  @constructor new() {}

  ifTrue(_ yes, ifFalse no) -> String {
    "ordinary"
  }
}

class Probe {
  @class
  run() {
    let result = Strange.new().ifTrue(|| { 1 }, ifFalse: || { 2 })
  }
}
"#,
    );
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let target = f.callable_id("Strange", "ifTrue", DispatchSide::Instance);
    let call = f.expression(run, "Strange.new().ifTrue(|| { 1 }, ifFalse: || { 2 })");

    f.assert_expression_call_target(call, &target);
    f.assert_binding_established(run, "result", string_ty);
    f.assert_no_error_diagnostics();
}

/// LAW: `whileTrue(_)` is still an ordinary selector on a non-block receiver;
/// literal block arguments alone do not turn the send into semantic loop flow.
#[test]
fn ordinary_while_true_selector_on_non_block_receiver_is_not_control_flow() {
    let f = Fixture::new(
        r#"
class Strange {
  @constructor new() {}

  whileTrue(_ body) -> String {
    "ordinary"
  }
}

class Probe {
  @class
  run() {
    let result = Strange.new().whileTrue(|| { 1 })
  }
}
"#,
    );
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let target = f.callable_id("Strange", "whileTrue", DispatchSide::Instance);
    let call = f.expression(run, "Strange.new().whileTrue(|| { 1 })");

    f.assert_expression_call_target(call, &target);
    f.assert_binding_established(run, "result", string_ty);
    f.assert_no_error_diagnostics();
}

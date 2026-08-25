#[path = "callable_publication_capabilities.rs"]
mod publication;
#[path = "callable_publication_trusted.rs"]
mod trusted_returns;

use crate::semantic::support::{Fixture, binding, known};
use phalcom_semantic::identity::DispatchSide;

/// P02: closure bodies publish a precise result when their literal tail is known.
#[test]
fn standalone_closure_publishes_literal_tail_type() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let producer = || { 42 }
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let producer = f.binding(run, "producer");
    assert!(producer.current.ty().is_some(), "closure should publish callable type: {producer:#?}");
}

/// P04: closure parameter capture does not replace the outer binding identity.
#[test]
fn closure_capture_keeps_outer_binding_identity() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let value = 1
    let read = || { value }
    let copied = value
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let values = f.bindings_named(run, "value");
    assert_eq!(values.len(), 1);
    f.assert_binding_expectation(run, "copied", binding().current(known(int_ty)));
    assert!(f.binding(run, "read").current.ty().is_some());
}

/// P06: a published method result can feed a second caller without annotation.
#[test]
fn published_method_tail_flows_through_two_callers() {
    let f = Fixture::new(
        r#"
class Source {
  @class
  make() {
    1
  }
}
class Probe {
  @class
  forward() {
    Source.make()
  }

  @class
  run() {
    let result = Probe.forward()
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_established(run, "result", int_ty);
}

/// P07: recursive publication remains incomplete instead of inventing a return type.
#[test]
fn recursive_publication_does_not_fabricate_a_result() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  loop() {
    Probe.loop()
  }
}
"#,
    );
    let loop_call = f.expression(f.callable("Probe", "loop", DispatchSide::Class), "Probe.loop()");
    assert!(loop_call.knowledge.is_unknown() || loop_call.knowledge.is_dynamic());
}

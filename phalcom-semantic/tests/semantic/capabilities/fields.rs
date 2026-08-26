//! Field capability scenarios.

use crate::semantic::support::Fixture;
use phalcom_semantic::identity::DispatchSide;

/// COMPOSED: field initializer, constructor write, mutation, and read should share one formal field fact.
#[test]
#[ignore = "GATED: formal field read/write publication is staged in the capability ledger"]
fn field_facts_survive_constructor_and_general_writes() {
    let f = Fixture::new(
        r#"
class Counter {
  _value: Int = 0

  @constructor new(_ initial: Int) { _value = initial }

  increment() -> Int {
    _value = _value + 1
    _value
  }

  read() -> Int { _value }
}

class Probe {
  @class
  run() {
    let counter = Counter.new(4)
    let after = counter.increment()
    counter.read()
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let increment = f.callable("Counter", "increment", DispatchSide::Instance);
    let read = f.callable("Counter", "read", DispatchSide::Instance);
    let run = f.callable("Probe", "run", DispatchSide::Class);

    f.assert_expression_established(f.expression(increment, "_value"), int_ty);
    f.assert_expression_established(f.expression(read, "_value"), int_ty);
    f.assert_binding_established(run, "counter", f.ty("Counter"));
    f.assert_binding_established(run, "after", int_ty);
    f.assert_no_error_diagnostics();
}

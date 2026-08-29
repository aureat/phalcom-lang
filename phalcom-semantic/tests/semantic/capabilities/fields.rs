//! Field capability scenarios.

use crate::semantic::support::Fixture;
use phalcom_semantic::identity::DispatchSide;

/// COMPOSED: field initializer, constructor write, mutation, and read should share one formal field fact.
#[test]
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

#[test]
fn default_initializer_establishes_instance_field_lifecycle_read() {
    let f = Fixture::new(
        r#"
class Counter {
  _value: Int = 0
  read() -> Int { _value }
}
"#,
    );
    let read = f.callable("Counter", "read", DispatchSide::Instance);
    f.assert_expression_established(f.expression(read, "_value"), f.ty("Int"));
}

#[test]
fn constructor_only_field_requires_all_normal_paths_to_initialize() {
    let positive = Fixture::new(
        r#"
class Cell {
  _value: Int
  @constructor new(_ value: Int) { _value = value }
  read() -> Int { _value }
}
"#,
    );
    let read = positive.callable("Cell", "read", DispatchSide::Instance);
    positive.assert_expression_established(positive.expression(read, "_value"), positive.ty("Int"));

    let negative = Fixture::new(
        r#"
class Cell {
  _value: Int
  @constructor new(_ flag: Bool, _ value: Int) {
    if flag { _value = value }
  }
  read() { _value }
}
"#,
    );
    let read = negative.callable("Cell", "read", DispatchSide::Instance);
    assert!(negative.expression(read, "_value").knowledge.is_unknown());
}

#[test]
fn branch_field_writes_join_current_fact_without_rewriting_contract() {
    let f = Fixture::new(
        r#"
class Counter {
  _value: Number = 0

  choose(_ flag: Bool) {
    if flag { _value = 1 } else { _value = 2.5 }
    _value
  }
}
"#,
    );
    let choose = f.callable("Counter", "choose", DispatchSide::Instance);
    let read = f.expression(choose, "_value");
    f.assert_union_members(read.knowledge.ty().expect("joined field type"), &[f.ty("Int"), f.ty("Float")]);
    let field = choose.entry_flow.fields.values().next().expect("field flow summary");
    assert_eq!(field.contract.ty(), Some(f.ty("Number")));
}

#[test]
fn wrong_constructor_write_is_initialized_but_never_certifies_field_contract() {
    let f = Fixture::new(
        r#"
class Cell {
  _value: Int

  @constructor
  new() {
    _value = "wrong"
  }

  read() { _value }
}
"#,
    );

    let constructor = f.callable("Cell", "new", DispatchSide::Instance);
    let read = f.callable("Cell", "read", DispatchSide::Instance);
    let string_ty = f.ty("String");
    let int_ty = f.ty("Int");

    let exit = constructor
        .exits
        .normal_returns
        .first()
        .expect("constructor normal exit");
    let field = exit.flow.fields.values().next().expect("field exit state");

    assert_eq!(field.initialization, phalcom_semantic::checker::flow::FieldInitialization::DefinitelyInitialized);
    assert_eq!(field.current.ty(), Some(string_ty));
    assert_eq!(field.validity, phalcom_semantic::checker::flow::FieldContractValidity::Refuted);
    assert!(
        !f.expression(read, "_value").knowledge.is_established()
            || f.expression(read, "_value").knowledge.ty() != Some(int_ty),
        "a refuted constructor write must not establish the declared field contract"
    );
    assert!(!f.diagnostics(phalcom_semantic::diagnostic::DiagnosticCode::FieldMismatch).is_empty());
}

#[test]
fn assumed_constructor_write_remains_assumed_in_field_state() {
    let f = Fixture::new(
        r#"
class Cell {
  _value: Number

  @constructor
  new(_ value: Int) {
    _value = value
  }
}
"#,
    );

    let constructor = f.callable("Cell", "new", DispatchSide::Instance);
    let int_ty = f.ty("Int");

    let exit = constructor
        .exits
        .normal_returns
        .first()
        .expect("constructor normal exit");
    let field = exit.flow.fields.values().next().expect("field exit state");

    assert_eq!(field.initialization, phalcom_semantic::checker::flow::FieldInitialization::DefinitelyInitialized);
    assert_eq!(field.current.ty(), Some(int_ty));
    assert_eq!(field.current.status(), Some(phalcom_semantic::types::evidence::EvidenceStatus::Assumed));
    assert_eq!(field.validity, phalcom_semantic::checker::flow::FieldContractValidity::Assumed);
}

#[test]
fn wrong_default_initializer_never_establishes_declared_field_contract() {
    let f = Fixture::new(
        r#"
class Cell {
  _value: Int = "wrong"
  read() { _value }
}
"#,
    );

    let read = f.callable("Cell", "read", DispatchSide::Instance);
    let value = f.expression(read, "_value");
    let int_ty = f.ty("Int");
    assert!(!value.knowledge.is_established() || value.knowledge.ty() != Some(int_ty));
    assert!(!f.diagnostics(phalcom_semantic::diagnostic::DiagnosticCode::FieldMismatch).is_empty());
}

#[test]
fn constructor_lifecycle_is_independent_of_constructor_source_order() {
    let order_a = Fixture::new(
        r#"
class Cell {
  _value: Int

  @constructor
  initA(_ value: Int) {
    _value = value
  }

  @constructor
  initB() {
  }

  read() { _value }
}
"#,
    );

    let order_b = Fixture::new(
        r#"
class Cell {
  _value: Int

  @constructor
  initB() {
  }

  @constructor
  initA(_ value: Int) {
    _value = value
  }

  read() { _value }
}
"#,
    );

    let read_a = order_a.callable("Cell", "read", DispatchSide::Instance);
    let read_b = order_b.callable("Cell", "read", DispatchSide::Instance);

    let val_a = order_a.expression(read_a, "_value");
    let val_b = order_b.expression(read_b, "_value");

    assert_eq!(val_a.knowledge.is_unknown(), val_b.knowledge.is_unknown());
    assert_eq!(val_a.knowledge.status(), val_b.knowledge.status());
    assert!(!val_a.knowledge.is_established());
    assert!(!val_b.knowledge.is_established());
}





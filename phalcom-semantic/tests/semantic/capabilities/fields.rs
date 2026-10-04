//! Field capability scenarios.

use crate::semantic::support::Fixture;
use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_semantic::identity::DispatchSide;

/// COMPOSED: field initializer, constructor write, mutation, and read should share one formal field fact.
#[test]
fn field_facts_survive_constructor_and_general_writes() {
    let f = Fixture::new(
        r#"
class Counter {
  mut _value: Int = 0

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

    assert_eq!(f.expression(increment, "_value").knowledge.ty(), Some(int_ty));
    assert_eq!(f.expression(read, "_value").knowledge.ty(), Some(int_ty));
    assert_eq!(
        f.expression(read, "_value").knowledge.status(),
        Some(phalcom_semantic::types::evidence::EvidenceStatus::Assumed)
    );
    f.assert_binding_established(run, "counter", f.ty("Counter"));
    f.assert_binding_type(run, "after", int_ty);
    assert_eq!(
        f.binding(run, "after").current.status(),
        Some(phalcom_semantic::types::evidence::EvidenceStatus::Assumed)
    );
    f.assert_no_error_diagnostics();
}

#[test]
fn subclass_constructor_resolves_explicit_parent_initializer_product() {
    let f = Fixture::new(
        r#"
class ParentThing {
  _base: Int
  @constructor new(_ value: Int) { _base = value }
}

class Child is ParentThing {
  _child: Int
  @constructor new(_ value: Int) {
    super.new(value)
    _child = value
  }
}
"#,
    );
    let child = f.callable("Child", "new", DispatchSide::Instance);
    let base_factory = phalcom_semantic::identity::CallableId::new(
        f.decl("ParentThing"),
        Selector::method("new", [SelectorSlot::Positional]).expect("factory selector"),
        DispatchSide::Class,
    );
    let base_initializer = phalcom_semantic::identity::CallableId::new(
        f.decl("ParentThing"),
        Selector::method("init new", [SelectorSlot::Positional]).expect("initializer selector"),
        DispatchSide::Instance,
    );

    let super_call = child.constructor_super_initializer.as_ref().expect("resolved parent initializer");
    assert_eq!(super_call.initializer, base_initializer);
    assert_eq!(f.expression(child, "super.new(value)").range, super_call.call_range);
    f.assert_expression_call_target(f.expression(child, "super.new(value)"), &base_factory);
    f.assert_no_diagnostic(phalcom_semantic::diagnostic::DiagnosticCode::ConstructorSuperRequired);
    f.assert_no_diagnostic(phalcom_semantic::diagnostic::DiagnosticCode::ConstructorSuperInvalid);
    f.assert_no_error_diagnostics();
}

#[test]
fn subclass_constructor_can_chain_to_parent_default_only_path() {
    let f = Fixture::new(
        r#"
class ParentDefaults {
  _base: Int = 1
}

class Child is ParentDefaults {
  @constructor new() { super.new() }
}
"#,
    );
    let child = f.callable("Child", "new", DispatchSide::Instance);
    let parent_initializer = phalcom_semantic::identity::CallableId::new(
        f.decl("ParentDefaults"),
        Selector::method("init new", []).expect("initializer selector"),
        DispatchSide::Instance,
    );

    assert_eq!(child.constructor_super_initializer.as_ref().map(|call| &call.initializer), Some(&parent_initializer));
    f.assert_no_error_diagnostics();
}

#[test]
fn subclass_constructor_must_explicitly_chain_parent_lifecycle() {
    let f = Fixture::new(
        r#"
class ParentThing {
  @constructor new() {}
}

class Child is ParentThing {
  @constructor new() {}
}
"#,
    );
    let child = f.callable("Child", "new", DispatchSide::Instance);
    assert_eq!(child.constructor_super_initializer, None);
    f.assert_diagnostic(phalcom_semantic::diagnostic::DiagnosticCode::ConstructorSuperRequired, 1);
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
    let val = positive.expression(read, "_value");
    assert_eq!(val.knowledge.ty(), Some(positive.ty("Int")));
    assert_eq!(val.knowledge.status(), Some(phalcom_semantic::types::evidence::EvidenceStatus::Assumed));

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
  mut _value: Number = 0

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

    let exit = constructor.exits.normal_returns.first().expect("constructor normal exit");
    let field = exit.flow.fields.values().next().expect("field exit state");

    assert_eq!(
        field.initialization,
        phalcom_semantic::checker::flow::FieldInitialization::DefinitelyInitialized
    );
    assert_eq!(field.current.ty(), Some(string_ty));
    assert_eq!(field.validity, phalcom_semantic::checker::flow::FieldContractValidity::Refuted);
    assert!(
        !f.expression(read, "_value").knowledge.is_established() || f.expression(read, "_value").knowledge.ty() != Some(int_ty),
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

    let exit = constructor.exits.normal_returns.first().expect("constructor normal exit");
    let field = exit.flow.fields.values().next().expect("field exit state");

    assert_eq!(
        field.initialization,
        phalcom_semantic::checker::flow::FieldInitialization::DefinitelyInitialized
    );
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

#[test]
fn field_read_propagates_causal_invalidity_from_mismatching_write() {
    let f = Fixture::new(
        r#"
class Cell {
  mut _value: Int

  test() {
    _value = "wrong"
    _value
  }
}
"#,
    );

    let test_fn = f.callable("Cell", "test", DispatchSide::Instance);
    let read_expr = f.expression(test_fn, "_value");
    assert_ne!(read_expr.causal_invalidity, phalcom_semantic::checker::causal::CausalInvalidity::Clean);
}

#[test]
fn immutable_constructor_write_is_exactly_once() {
    let first_write = Fixture::new(
        r#"
class Cell {
  _value: Int
  @constructor new() { _value = 1 }
}
"#,
    );
    first_write.assert_no_diagnostic(phalcom_semantic::diagnostic::DiagnosticCode::ImmutableFieldWriteOutsideConstruction);
    first_write.assert_no_diagnostic(phalcom_semantic::diagnostic::DiagnosticCode::ImmutableFieldAlreadyInitialized);
    first_write.assert_no_diagnostic(phalcom_semantic::diagnostic::DiagnosticCode::ImmutableFieldNotInitialized);

    let second_write = Fixture::new(
        r#"
class Cell {
  _value: Int
  @constructor new() { _value = 1; _value = 2 }
}
"#,
    );
    second_write.assert_diagnostic(phalcom_semantic::diagnostic::DiagnosticCode::ImmutableFieldAlreadyInitialized, 1);
}

#[test]
fn immutable_field_write_outside_constructor_is_rejected_and_mutable_reassignment_remains_valid() {
    let immutable = Fixture::new(
        r#"
class Cell {
  _value: Int = 0
  update() { _value = 1 }
}
"#,
    );
    immutable.assert_diagnostic(phalcom_semantic::diagnostic::DiagnosticCode::ImmutableFieldWriteOutsideConstruction, 1);

    let mutable = Fixture::new(
        r#"
class Cell {
  mut _value: Int = 0
  update() { _value = _value + 1 }
}
"#,
    );
    mutable.assert_no_error_diagnostics();
}

#[test]
fn immutable_constructor_exit_requires_every_normal_path_to_initialize() {
    let partial = Fixture::new(
        r#"
class Cell {
  _value: Int
  @constructor new(_ flag: Bool) { if flag { _value = 1 } }
}
"#,
    );
    partial.assert_diagnostic(phalcom_semantic::diagnostic::DiagnosticCode::ImmutableFieldNotInitialized, 1);

    let complete = Fixture::new(
        r#"
class Cell {
  _value: Int
  @constructor new(_ flag: Bool) { if flag { _value = 1 } else { _value = 2 } }
}
"#,
    );
    complete.assert_no_diagnostic(phalcom_semantic::diagnostic::DiagnosticCode::ImmutableFieldNotInitialized);
}

#[test]
fn immutable_class_side_field_requires_initializer() {
    let f = Fixture::new(
        r#"
class Cell {
  @class _count: Int
}
"#,
    );
    f.assert_diagnostic(phalcom_semantic::diagnostic::DiagnosticCode::ImmutableClassFieldRequiresInitializer, 1);
}

#[test]
fn assignment_does_not_declare_a_field() {
    let f = Fixture::new(
        r#"
class Cell {
  @constructor new() { _missing = 1 }
}
"#,
    );
    f.assert_diagnostic(phalcom_semantic::diagnostic::DiagnosticCode::FieldNotDeclared, 1);
}

#[test]
fn uninitialized_field_read_through_receiver_is_not_established() {
    let f = Fixture::new(
        r#"
class Box {
  _value: Int

  @constructor
  new() {
    let dummy = 0
  }

  get() { _value }
}

class Reader {
  @class
  read(_ b: Box) {
    b.get()
  }
}
"#,
    );

    let reader = f.callable("Reader", "read", DispatchSide::Class);
    let call_expr = f.expression(reader, "b.get()");
    assert!(!call_expr.knowledge.is_established());
}

#[test]
fn valid_constructor_literal_establishes_field_contract() {
    let f = Fixture::new(
        r#"
class Box {
  _value: Int

  @constructor
  new() {
    _value = 1
  }

  read() { _value }
}
"#,
    );

    let constructor = f.callable("Box", "new", DispatchSide::Instance);
    let exit = constructor.exits.normal_returns.first().expect("constructor exit");
    let field = exit.flow.fields.values().next().expect("field flow");
    assert_eq!(
        field.initialization,
        phalcom_semantic::checker::flow::FieldInitialization::DefinitelyInitialized
    );
    assert_eq!(field.validity, phalcom_semantic::checker::flow::FieldContractValidity::Validated);

    let read = f.callable("Box", "read", DispatchSide::Instance);
    f.assert_expression_established(f.expression(read, "_value"), f.ty("Int"));
}

#[test]
fn both_branches_valid_initializes_and_validates_field() {
    let f = Fixture::new(
        r#"
class Cell {
  _value: Int

  @constructor
  new(_ flag: Bool) {
    if flag {
      _value = 1
    } else {
      _value = 2
    }
  }

  read() { _value }
}
"#,
    );

    let constructor = f.callable("Cell", "new", DispatchSide::Instance);
    let exit = constructor.exits.normal_returns.first().expect("constructor exit");
    let field = exit.flow.fields.values().next().expect("field flow");
    assert_eq!(
        field.initialization,
        phalcom_semantic::checker::flow::FieldInitialization::DefinitelyInitialized
    );
    assert_eq!(field.validity, phalcom_semantic::checker::flow::FieldContractValidity::Validated);

    let read = f.callable("Cell", "read", DispatchSide::Instance);
    f.assert_expression_established(f.expression(read, "_value"), f.ty("Int"));
}

#[test]
fn mixed_valid_and_refuted_branches_leaves_field_refuted() {
    let f = Fixture::new(
        r#"
class Cell {
  _value: Int

  @constructor
  new(_ flag: Bool) {
    if flag {
      _value = 1
    } else {
      _value = "bad"
    }
  }

  read() { _value }
}
"#,
    );

    let constructor = f.callable("Cell", "new", DispatchSide::Instance);
    let exit = constructor.exits.normal_returns.first().expect("constructor exit");
    let field = exit.flow.fields.values().next().expect("field flow");
    assert_eq!(
        field.initialization,
        phalcom_semantic::checker::flow::FieldInitialization::DefinitelyInitialized
    );
    assert_eq!(field.validity, phalcom_semantic::checker::flow::FieldContractValidity::Refuted);

    let read = f.callable("Cell", "read", DispatchSide::Instance);
    assert!(!f.expression(read, "_value").knowledge.is_established());
}

#[test]
fn multiple_constructors_all_valid_establishes_field_contract() {
    let f = Fixture::new(
        r#"
class Cell {
  _value: Int

  @constructor
  initA() {
    _value = 1
  }

  @constructor
  initB() {
    _value = 2
  }

  read() { _value }
}
"#,
    );

    let read = f.callable("Cell", "read", DispatchSide::Instance);
    f.assert_expression_established(f.expression(read, "_value"), f.ty("Int"));
}

#[test]
fn multiple_constructors_one_missing_does_not_establish_field_contract() {
    let f = Fixture::new(
        r#"
class Cell {
  _value: Int

  @constructor
  initA() {
    _value = 1
  }

  @constructor
  initB() {
    let dummy = 0
  }

  read() { _value }
}
"#,
    );

    let read = f.callable("Cell", "read", DispatchSide::Instance);
    assert!(!f.expression(read, "_value").knowledge.is_established());
}

#[test]
fn multiple_constructors_one_refuted_does_not_establish_field_contract() {
    let f = Fixture::new(
        r#"
class Cell {
  _value: Int

  @constructor
  initA() {
    _value = 1
  }

  @constructor
  initB() {
    _value = "bad"
  }

  read() { _value }
}
"#,
    );

    let read = f.callable("Cell", "read", DispatchSide::Instance);
    assert!(!f.expression(read, "_value").knowledge.is_established());
}

#[test]
fn return_field_call_composition_propagates_established_field_tail() {
    let f = Fixture::new(
        r#"
class Cell {
  _value: Int

  @constructor
  new() {
    _value = 1
  }

  get() { _value }
}

class Probe {
  @class
  run() {
    Cell.new().get()
  }
}
"#,
    );

    let run = f.callable("Probe", "run", DispatchSide::Class);
    let get_call = f.expression(run, "Cell.new().get()");
    f.assert_expression_established(get_call, f.ty("Int"));
}

#[test]
fn refuted_field_tail_cannot_establish_downstream_caller() {
    let f = Fixture::new(
        r#"
class Cell {
  _value: Int

  @constructor
  new() {
    _value = "wrong"
  }

  get() { _value }
}

class Probe {
  @class
  run() {
    Cell.new().get()
  }
}
"#,
    );

    let run = f.callable("Probe", "run", DispatchSide::Class);
    let get_call = f.expression(run, "Cell.new().get()");
    assert!(!get_call.knowledge.is_established());
}

#[test]
fn constructor_initialization_products_cover_source_and_generated_paths() {
    let f = Fixture::new(
        r#"
@construct
class Constructed {
  _required: Int
  _defaulted: String = "default"
}

@data
class Datum {
  _first: Int
  _defaulted: Int = 1
}

class Defaults {
  _first: Int = 1
  _second: String = "two"
}

class Source {
  _defaulted: Int = 0
  @constructor custom(_ value: Int) { _defaulted = value }
}
"#,
    );
    let table = &f.analysis.snapshot.constructor_initializations;
    let signatures = &f.analysis.snapshot.callable_signatures;

    let constructed_owner = f.decl("Constructed");
    let construct_factory = phalcom_semantic::CallableId::new(
        constructed_owner.clone(),
        Selector::method("new", [SelectorSlot::Label("required".into())]).unwrap(),
        DispatchSide::Class,
    );
    let construct = table.get(&construct_factory).expect("@construct product");
    assert_eq!(construct.origin, phalcom_semantic::ConstructorOrigin::Construct);
    assert_eq!(
        construct.default_fields.as_ref(),
        &[phalcom_semantic::FieldId::new(constructed_owner.clone(), "_defaulted", DispatchSide::Instance)]
    );
    assert_eq!(
        construct.parameter_fields.as_ref(),
        &[phalcom_semantic::ConstructorParameterField {
            parameter: phalcom_semantic::identity::CallableParameterId::new(construct_factory.clone(), 0),
            field: phalcom_semantic::FieldId::new(constructed_owner, "_required", DispatchSide::Instance),
        }]
    );
    assert!(signatures.get(&construct_factory).is_some(), "factory signature is canonical");
    assert!(signatures.get(&construct.initializer).is_some(), "paired initializer signature is canonical");

    let datum_owner = f.decl("Datum");
    let data_factory = phalcom_semantic::CallableId::new(
        datum_owner,
        Selector::method("new", [SelectorSlot::Label("first".into())]).unwrap(),
        DispatchSide::Class,
    );
    let data = table.get(&data_factory).expect("@data product");
    assert_eq!(data.origin, phalcom_semantic::ConstructorOrigin::Data);
    assert_eq!(data.parameter_fields[0].field.name.as_ref(), "_first");
    assert_eq!(data.default_fields[0].name.as_ref(), "_defaulted");

    let defaults_owner = f.decl("Defaults");
    let default_factory = phalcom_semantic::CallableId::new(defaults_owner, Selector::method("new", Vec::new()).unwrap(), DispatchSide::Class);
    let default_only = table.get(&default_factory).expect("default-only path");
    assert_eq!(default_only.origin, phalcom_semantic::ConstructorOrigin::DefaultOnly);
    assert_eq!(
        default_only.default_fields.iter().map(|field| field.name.as_ref()).collect::<Vec<_>>(),
        ["_first", "_second"]
    );

    let source_owner = f.decl("Source");
    let source_factory = phalcom_semantic::CallableId::new(
        source_owner,
        Selector::method("custom", [SelectorSlot::Positional]).unwrap(),
        DispatchSide::Class,
    );
    let source = table.get(&source_factory).expect("source constructor product");
    assert_eq!(source.origin, phalcom_semantic::ConstructorOrigin::Source);
    assert_eq!(source.default_fields[0].name.as_ref(), "_defaulted");
    assert!(source.parameter_fields.is_empty());
    assert!(signatures.get(&source.initializer).is_some(), "source initializer signature is canonical");
}

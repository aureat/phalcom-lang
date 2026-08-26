use crate::semantic::support::{Fixture, binding, known};
use phalcom_semantic::identity::DispatchSide;

/// S05: tuple patterns recursively expose nested leaves.
#[test]
fn nested_tuple_pattern_recursively_establishes_each_leaf() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let ((number, text), flag) = ((1, "hello"), true)
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let bool_ty = f.ty("Bool");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_expectation(run, "number", binding().current(known(int_ty)));
    f.assert_binding_expectation(run, "text", binding().current(known(string_ty)));
    f.assert_binding_expectation(run, "flag", binding().current(known(bool_ty)));
}

/// S08: generic result structure survives immediate destructuring.
#[test]
fn generic_pair_result_can_be_destructured_without_losing_components() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  pair<A, B>(_ first: A, _ second: B) -> (A, B) {
    (first, second)
  }

  @class
  run() {
    let (number, text) = Probe.pair(1, "hello")
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_expectation(run, "number", binding().current(known(int_ty)));
    f.assert_binding_expectation(run, "text", binding().current(known(string_ty)));
}

/// S03: nested records retain independent source fields.
#[test]
fn nested_record_literal_retains_field_structure() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let value = #{user: #{name: "A"}, count: 2}
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_record(f.binding(run, "value").current.ty().expect("record type"));
    f.assert_no_error_diagnostics();
}

/// S04: collection element joins remain a union at nested depth.
#[test]
fn nested_heterogeneous_collection_does_not_widen_to_object() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let values = [[1], ["hello"]]
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let values = f.binding(run, "values").current.ty().expect("nested list type");
    assert!(
        !matches!(f.analysis.snapshot.store.get(values), phalcom_semantic::types::store::TypeData::Nominal { declaration } if declaration.name.as_ref() == "Object")
    );
}

/// COMPOSED: collection rest capture must preserve head precision and product structure.
#[test]
#[ignore = "GATED: list/rest pattern lowering is not formal yet"]
fn collection_and_destructure_facts_preserve_element_shapes() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let source = [1, 2, 3]
    let [head, *tail] = source
    let pair = (head, tail)
    let record = #{first: head, remaining: tail}
    record
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let tail = f.binding(run, "tail").current.ty().expect("rest binding type");

    f.assert_binding_established(run, "head", int_ty);
    assert!(!f.binding(run, "tail").current.is_dynamic(), "rest capture must retain structural knowledge");
    f.assert_tuple_types(f.binding(run, "pair").current.ty().expect("pair type"), &[int_ty, tail]);
    f.assert_record(f.binding(run, "record").current.ty().expect("record type"));
}

use crate::semantic::support::{Fixture, assert_source_contract, binding, known};
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::EvidenceStatus;

/// LAW: iteration element type comes from protocol return specialization.
#[test]
fn custom_iterable_element_type_comes_from_protocol_not_first_generic_argument() {
    let f = Fixture::new(
        r#"
class Weird<A, B> {
  iteratorValue(_ cursor: Int) -> B {
    mystery()
  }
}
class Probe {
  @class
  run(_ weird: Weird<String, Int>) {
    for value in weird {
      let observed = value
    }
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_expectation(run, "value", binding().current(known(int_ty)));
    f.assert_binding_expectation(run, "observed", binding().current(known(int_ty)));
    f.assert_binding_type(run, "value", int_ty);
    f.assert_binding_type(run, "observed", int_ty);
}

/// LAW: branch joins compose nested collection element precision.
#[test]
fn constructor_branch_nested_inside_collection_preserves_composed_specific_type() {
    let f = Fixture::new(
        r#"
class Animal {}
class Cat is Animal { @constructor new() {} }
class Dog is Animal { @constructor new() {} }
class Probe {
  @class
  run(_ flag: Bool) {
    let xs = if flag {
      [Cat.new()]
    } else {
      [Dog.new()]
    }
  }
}
"#,
    );
    let cat = f.ty("Cat");
    let dog = f.ty("Dog");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let xs_ty = f.binding(run, "xs").current.ty().expect("branch collection type");
    match f.analysis.snapshot.store.get(xs_ty) {
        phalcom_semantic::types::store::TypeData::Applied { arguments, .. } => {
            assert_eq!(arguments.len(), 1);
            f.assert_union_members(arguments[0], &[cat, dog]);
        }
        phalcom_semantic::types::store::TypeData::Union(members) => {
            assert_eq!(members.len(), 2, "alternative normalization should retain both collection branches");
        }
        other => panic!("expected joined collection type, got {other:?}"),
    }
}

/// LAW: unknown branch evidence weakens a declared result instead of becoming established.
#[test]
fn formal_unknown_branch_with_declared_contract_remains_assumed_not_established() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let x: Number = if flag {
      42
    } else {
      mystery()
    }
  }
}
"#,
    );
    let number = f.ty("Number");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let x = f.binding(run, "x");
    assert_source_contract(x, number);
    assert_ne!(
        x.current.status(),
        Some(EvidenceStatus::Established),
        "reachable unknown evidence must not be laundered into an established Number fact: {x:#?}"
    );
    if x.current.ty() == Some(number) {
        assert_eq!(x.current.status(), Some(EvidenceStatus::Assumed));
    }
    f.assert_no_error_diagnostics();
}

/// I02: changing the receiver application changes the protocol element type.
#[test]
fn generic_iterator_receiver_substitution_selects_second_parameter() {
    let f = Fixture::new(
        r#"
class Weird<A, B> {
  iteratorValue(_ cursor: Int) -> B { mystery() }
}
class Probe {
  @class
  run(_ weird: Weird<Int, String>) {
    for value in weird {
      let observed = value
    }
  }
}
"#,
    );
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_type(run, "value", string_ty);
    f.assert_binding_type(run, "observed", string_ty);
}

/// I03: structured `for` patterns decompose the protocol element fact into
/// independent pattern bindings rather than losing the tuple components.
#[test]
fn for_tuple_pattern_decomposes_protocol_element_type() {
    let f = Fixture::new(
        r#"
class Entries {
  iteratorValue(_ cursor: Int) -> (String, Int) { mystery() }
}
class Probe {
  @class
  run(_ entries: Entries) {
    for (key, value) in entries {
      let observedKey = key
      let observedValue = value
    }
  }
}
"#,
    );
    let string_ty = f.ty("String");
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);

    f.assert_binding_expectation(
        run,
        "key",
        binding().current(known(string_ty).origin(phalcom_semantic::EvidenceOrigin::PatternDecomposition)),
    );
    f.assert_binding_expectation(
        run,
        "value",
        binding().current(known(int_ty).origin(phalcom_semantic::EvidenceOrigin::PatternDecomposition)),
    );
    f.assert_binding_type(run, "observedKey", string_ty);
    f.assert_binding_type(run, "observedValue", int_ty);
}

/// I05: a collection protocol without formal element evidence stays incomplete.
#[test]
fn heterogeneous_collection_iteration_preserves_element_union() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let values = [1, "text"]
    for value in values {
      let observed = value
    }
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let value = f.binding(run, "value");
    assert!(value.current.is_unknown() || value.current.is_dynamic());
    let observed = f.binding(run, "observed");
    assert!(observed.current.is_unknown() || observed.current.is_dynamic());
}

use crate::semantic::support::{Fixture, assert_refuted, assert_source_contract, assert_validated, binding, known};
use phalcom_semantic::identity::DispatchSide;

/// B01/B03: declared and current knowledge remain separate across mutation.
#[test]
fn mutable_assignment_updates_current_without_rewriting_contract() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let value: Number = 1
    value = 2
    let observed = value
  }
}
"#,
    );
    let number = f.ty("Number");
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let value = f.binding(run, "value");
    assert_source_contract(value, number);
    assert_eq!(value.current.ty(), Some(int_ty));
    assert_validated(value);
    f.assert_binding_type(run, "observed", int_ty);
}

/// B04: invalid later evidence remains visible for recovery and downstream reads.
#[test]
fn invalid_assignment_preserves_actual_evidence_for_downstream_reads() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let value: Number = 1
    value = "bad"
    let observed = value
  }
}
"#,
    );
    let number = f.ty("Number");
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let value = f.binding(run, "value");
    assert_source_contract(value, number);
    assert_refuted(value, string_ty, number);
    f.assert_binding_type(run, "observed", string_ty);
    assert_ne!(value.current.ty(), Some(number));
}

/// B05: shadowing creates a new identity while preserving the outer fact.
#[test]
fn nested_shadowing_keeps_outer_and_inner_binding_identities_distinct() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let value = 1
    if flag {
      let value = "shadow"
      let inner = value
    }
    let outer = value
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let values = f.bindings_named(run, "value");
    assert_eq!(values.len(), 2);
    assert_ne!(values[0].binding, values[1].binding);
    assert_eq!(values[0].current.ty(), Some(int_ty));
    assert_eq!(values[1].current.ty(), Some(string_ty));
    f.assert_binding_type(run, "inner", string_ty);
    f.assert_binding_type(run, "outer", int_ty);
}

/// B07: unknown causal evidence does not suppress an independent sibling.
#[test]
fn unknown_binding_does_not_suppress_independent_sibling_analysis() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let missing = mystery()
    let independent = 42
    let copied = independent
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    assert!(f.binding(run, "missing").current.is_unknown());
    f.assert_binding_established(run, "independent", int_ty);
    f.assert_binding_established(run, "copied", int_ty);
}

/// E01: syntax evidence stays established even beside an unknown expression.
#[test]
fn literal_evidence_is_not_weakened_by_unrelated_unknown_flow() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let missing = mystery()
    let literal = 7
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_expectation(
        run,
        "literal",
        binding().current(known(int_ty).established().origin(phalcom_semantic::EvidenceOrigin::Syntax)),
    );
}

/// E08: a branch join uses weakest support across established and assumed arms.
#[test]
fn branch_join_weakens_to_assumed_when_one_arm_is_assumed() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool, _ value: Int) {
    let result = if flag { value } else { 1 }
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_expectation(
        run,
        "result",
        binding().current(known(int_ty).assumed().origin(phalcom_semantic::EvidenceOrigin::Flow)),
    );
}

/// B06: destructuring a union keeps independent leaf facts rather than widening them.
#[test]
fn destructuring_branch_products_preserves_leaf_union_precision() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let pair = if flag { (1, "a") } else { (2, "b") }
    let (number, text) = pair
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

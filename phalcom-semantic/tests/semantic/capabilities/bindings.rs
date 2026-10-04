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

/// C9/T36: a later write updates path current knowledge while preserving the
/// broad source contract, including when the new value refutes that contract.
#[test]
fn broad_contract_survives_narrow_initializer_and_later_refuted_write() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let mut value: Number = 1
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

/// B04: invalid later evidence remains visible for recovery and downstream reads.
#[test]
fn invalid_assignment_preserves_actual_evidence_for_downstream_reads() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let mut value: Number = 1
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

#[test]
fn let_binding_is_immutable() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let x = 10
    x = 20
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let x = f.binding(run, "x");
    assert!(!x.mutable);
    assert_eq!(f.diagnostics(phalcom_semantic::DiagnosticCode::AssignmentToImmutable).len(), 1);
}

#[test]
fn let_mut_binding_is_mutable() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let mut x = 10
    x = 20
    let y = x
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let x = f.binding(run, "x");
    assert!(x.mutable);
    f.assert_no_error_diagnostics();
    let int_ty = f.ty("Int");
    f.assert_binding_type(run, "y", int_ty);
}

#[test]
fn definitely_uninitialized_binding_read_is_diagnostic() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let mut x: Int
    let observed = x
  }
}
"#,
    );

    assert_eq!(f.diagnostics(phalcom_semantic::DiagnosticCode::UninitializedBindingRead).len(), 1);
}

#[test]
fn source_index_reports_let_mut_only_as_mutable() {
    let source = r#"
let top_imm = 1
let mut top_mut = 2
const top_const = 3

class Foo {
  method(param: Int) {
    let loc_imm = 4
    let mut loc_mut = 5
    const loc_const = 6
    for i in [1, 2] {
      let inner = i
    }
  }
}
"#;
    let parsed = phalcom_ast::parser::parse(source, 0);
    assert!(parsed.errors.is_empty());
    let index = phalcom_semantic::source_index::build_source_scope_index(
        phalcom_semantic::identity::ModuleId::universe_root(),
        &parsed.program,
        &phalcom_semantic::source_index::SourceIndexContext::default(),
    );

    let find = |name: &str| {
        index
            .bindings
            .values()
            .find(|b| b.name.as_ref() == name)
            .unwrap_or_else(|| panic!("binding {} not found", name))
    };

    let top_imm = find("top_imm");
    assert_eq!(top_imm.kind, phalcom_semantic::source_index::SourceBindingKind::TopLevelLet);
    assert!(!top_imm.mutable);

    let top_mut = find("top_mut");
    assert_eq!(top_mut.kind, phalcom_semantic::source_index::SourceBindingKind::TopLevelLetMut);
    assert!(top_mut.mutable);

    let top_const = find("top_const");
    assert_eq!(top_const.kind, phalcom_semantic::source_index::SourceBindingKind::TopLevelConst);
    assert!(!top_const.mutable);

    let loc_imm = find("loc_imm");
    assert_eq!(loc_imm.kind, phalcom_semantic::source_index::SourceBindingKind::LocalLet);
    assert!(!loc_imm.mutable);

    let loc_mut = find("loc_mut");
    assert_eq!(loc_mut.kind, phalcom_semantic::source_index::SourceBindingKind::LocalLetMut);
    assert!(loc_mut.mutable);

    let loc_const = find("loc_const");
    assert_eq!(loc_const.kind, phalcom_semantic::source_index::SourceBindingKind::LocalConst);
    assert!(!loc_const.mutable);

    let param = find("param");
    assert_eq!(param.kind, phalcom_semantic::source_index::SourceBindingKind::MethodParameter);
    assert!(!param.mutable);

    let loop_var = find("i");
    assert_eq!(loop_var.kind, phalcom_semantic::source_index::SourceBindingKind::ForBinding);
    assert!(!loop_var.mutable);
}

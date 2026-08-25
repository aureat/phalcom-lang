use crate::semantic::support::{Fixture, binding, known, unknown, union};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::{EvidenceOrigin, UnknownReason};

/// LAW: the join itself owns Flow provenance even when all reachable writes
/// retain the same concrete type.
#[test]
fn same_type_branch_writes_publish_flow_provenance() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let value = 1
    if flag {
      value = 2
    } else {
      value = 3
    }
    let observed = value
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);

    f.assert_binding_expectation(run, "value", binding().current(known(int_ty).established().origin(EvidenceOrigin::Flow)));
    f.assert_binding_expectation(run, "observed", binding().current(known(int_ty).established().origin(EvidenceOrigin::Flow)));
    f.assert_no_error_diagnostics();
}

/// LAW: an unresolved call in one reachable branch remains that exact Unknown
/// reason after the join; it is not downgraded to dynamic or laundered by a
/// known sibling branch.
#[test]
fn branch_join_preserves_exact_unknown_reason() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let value = if flag { 42 } else { mystery() }
    let observed = value
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let expected = UnknownReason::UnresolvedName("mystery".into());

    f.assert_knowledge(&f.binding(run, "value").current, &unknown(expected.clone()));
    f.assert_knowledge(&f.binding(run, "observed").current, &unknown(expected));
}

/// LAW: a refuted branch write remains concrete recovery evidence, while the
/// owning assignment diagnostic is emitted exactly once.
#[test]
fn refuted_branch_write_preserves_recovery_union_and_diagnostic_owner() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let value: Number = 1
    if flag {
      value = "bad"
    } else {
      value = 2
    }
    let observed = value
  }
}
"#,
    );
    let number = f.ty("Number");
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let run = f.callable("Probe", "run", DispatchSide::Class);

    f.assert_binding_expectation(
        run,
        "value",
        binding()
            .declared(number)
            .current(known(union([int_ty.into(), string_ty.into()])).established().origin(EvidenceOrigin::Flow)),
    );
    f.assert_binding_expectation(
        run,
        "observed",
        binding().current(known(union([int_ty.into(), string_ty.into()])).established().origin(EvidenceOrigin::Flow)),
    );
    f.assert_diagnostic(DiagnosticCode::AssignmentMismatch, 1);
    f.assert_only_error_codes(&[DiagnosticCode::AssignmentMismatch]);
}

/// LAW: a loop join carries zero-iteration and body-write evidence as a Flow
/// fact while preserving the broad source contract separately.
#[test]
fn loop_join_publishes_flow_provenance_without_widening_to_contract() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let value: Number = 1
    while flag {
      value = 2.5
    }
    let observed = value
  }
}
"#,
    );
    let number = f.ty("Number");
    let int_ty = f.ty("Int");
    let float_ty = f.ty("Float");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let expected = known(union([int_ty.into(), float_ty.into()])).established().origin(EvidenceOrigin::Flow);

    f.assert_binding_expectation(run, "value", binding().declared(number).current(expected.clone()));
    f.assert_binding_expectation(run, "observed", binding().current(expected));
    f.assert_no_error_diagnostics();
}

/// LAW: checking an escaping closure body must not execute its captured write
/// in the enclosing flow. Both the source binding and its later read retain the
/// original syntax-established fact.
#[test]
fn closure_construction_preserves_outer_flow_provenance() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let value = 1
    let action = || {
      value = "changed"
    }
    let observed = value
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let expected = known(int_ty).established().origin(EvidenceOrigin::Syntax);

    f.assert_binding_expectation(run, "value", binding().current(expected.clone()));
    f.assert_binding_expectation(run, "observed", binding().current(expected));
    assert!(f.binding(run, "action").current.ty().is_some(), "closure value must still be analyzed");
    f.assert_no_error_diagnostics();
}

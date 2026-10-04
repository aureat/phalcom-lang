//! Low-level and semantic tests for binding definite initialization and read safety (T4).

use crate::semantic::support::Fixture;
use phalcom_common::range::SourceRange;
use phalcom_semantic::checker::flow::{BindingReadSafety, FlowState, InitializationState};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{BindingId, DispatchSide};
use phalcom_semantic::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
use phalcom_semantic::types::store::TypeStore;

#[test]
fn binding_init_join_uninitialized() {
    let mut store = TypeStore::new();
    let binding = BindingId(1);

    let mut left = FlowState::new();
    left.declare_with_initialization(
        binding,
        "x",
        SourceRange::default(),
        None,
        TypeKnowledge::Unknown(UnknownReason::MissingInitializer),
        true,
        InitializationState::Uninitialized,
    );

    let mut right = FlowState::new();
    right.declare_with_initialization(
        binding,
        "x",
        SourceRange::default(),
        None,
        TypeKnowledge::Unknown(UnknownReason::MissingInitializer),
        true,
        InitializationState::Uninitialized,
    );

    let joined = FlowState::join(&[left, right], &mut store);
    assert_eq!(
        joined.get_binding(binding).expect("binding").initialization,
        InitializationState::Uninitialized
    );
}

#[test]
fn binding_init_join_definite() {
    let mut store = TypeStore::new();
    let int_ty = store.nominal_type(phalcom_semantic::identity::DeclarationId::new(
        phalcom_modules::identity::ModuleId::universe_root(),
        "Int".into(),
    ));
    let binding = BindingId(1);

    let mut left = FlowState::new();
    left.declare_with_initialization(
        binding,
        "x",
        SourceRange::default(),
        None,
        TypeKnowledge::established(int_ty, EvidenceOrigin::Flow),
        true,
        InitializationState::DefinitelyInitialized,
    );

    let mut right = FlowState::new();
    right.declare_with_initialization(
        binding,
        "x",
        SourceRange::default(),
        None,
        TypeKnowledge::established(int_ty, EvidenceOrigin::Flow),
        true,
        InitializationState::DefinitelyInitialized,
    );

    let joined = FlowState::join(&[left, right], &mut store);
    assert_eq!(
        joined.get_binding(binding).expect("binding").initialization,
        InitializationState::DefinitelyInitialized
    );
}

#[test]
fn binding_init_join_mixed_is_maybe() {
    let mut store = TypeStore::new();
    let int_ty = store.nominal_type(phalcom_semantic::identity::DeclarationId::new(
        phalcom_modules::identity::ModuleId::universe_root(),
        "Int".into(),
    ));
    let binding = BindingId(1);

    let mut left = FlowState::new();
    left.declare_with_initialization(
        binding,
        "x",
        SourceRange::default(),
        None,
        TypeKnowledge::established(int_ty, EvidenceOrigin::Flow),
        true,
        InitializationState::DefinitelyInitialized,
    );

    let mut right = FlowState::new();
    right.declare_with_initialization(
        binding,
        "x",
        SourceRange::default(),
        None,
        TypeKnowledge::Unknown(UnknownReason::MissingInitializer),
        true,
        InitializationState::Uninitialized,
    );

    let joined = FlowState::join(&[left, right], &mut store);
    assert_eq!(
        joined.get_binding(binding).expect("binding").initialization,
        InitializationState::MaybeInitialized
    );
}

#[test]
fn binding_init_join_ignores_unreachable() {
    let mut store = TypeStore::new();
    let int_ty = store.nominal_type(phalcom_semantic::identity::DeclarationId::new(
        phalcom_modules::identity::ModuleId::universe_root(),
        "Int".into(),
    ));
    let binding = BindingId(1);

    let mut left = FlowState::new();
    left.declare_with_initialization(
        binding,
        "x",
        SourceRange::default(),
        None,
        TypeKnowledge::established(int_ty, EvidenceOrigin::Flow),
        true,
        InitializationState::DefinitelyInitialized,
    );

    let unreachable = FlowState::unreachable();

    let joined = FlowState::join(&[left, unreachable], &mut store);
    assert_eq!(
        joined.get_binding(binding).expect("binding").initialization,
        InitializationState::DefinitelyInitialized
    );
}

#[test]
fn assignment_marks_binding_definitely_initialized() {
    let mut store = TypeStore::new();
    let int_ty = store.nominal_type(phalcom_semantic::identity::DeclarationId::new(
        phalcom_modules::identity::ModuleId::universe_root(),
        "Int".into(),
    ));
    let binding = BindingId(1);

    let mut flow = FlowState::new();
    flow.declare_with_initialization(
        binding,
        "x",
        SourceRange::default(),
        None,
        TypeKnowledge::Unknown(UnknownReason::MissingInitializer),
        true,
        InitializationState::Uninitialized,
    );
    assert_eq!(
        flow.get_binding(binding).expect("binding").initialization,
        InitializationState::Uninitialized
    );

    flow.assign(binding, TypeKnowledge::established(int_ty, EvidenceOrigin::Flow));
    assert_eq!(
        flow.get_binding(binding).expect("binding").initialization,
        InitializationState::DefinitelyInitialized
    );
}

#[test]
fn loop_may_initialize_binding_is_maybe() {
    let mut store = TypeStore::new();
    let int_ty = store.nominal_type(phalcom_semantic::identity::DeclarationId::new(
        phalcom_modules::identity::ModuleId::universe_root(),
        "Int".into(),
    ));
    let binding = BindingId(1);

    let mut header = FlowState::new();
    header.declare_with_initialization(
        binding,
        "x",
        SourceRange::default(),
        None,
        TypeKnowledge::Unknown(UnknownReason::MissingInitializer),
        true,
        InitializationState::Uninitialized,
    );

    let mut next_header = FlowState::new();
    next_header.declare_with_initialization(
        binding,
        "x",
        SourceRange::default(),
        None,
        TypeKnowledge::established(int_ty, EvidenceOrigin::Flow),
        true,
        InitializationState::DefinitelyInitialized,
    );

    let widened = FlowState::widen_loop_state(&header, &next_header, &mut store).expect("loop widening succeeded");
    assert_eq!(
        widened.get_binding(binding).expect("binding").initialization,
        InitializationState::MaybeInitialized
    );
}

#[test]
fn loop_all_exits_initialize_binding_is_definite() {
    let mut store = TypeStore::new();
    let int_ty = store.nominal_type(phalcom_semantic::identity::DeclarationId::new(
        phalcom_modules::identity::ModuleId::universe_root(),
        "Int".into(),
    ));
    let binding = BindingId(1);

    let mut exit_a = FlowState::new();
    exit_a.declare_with_initialization(
        binding,
        "x",
        SourceRange::default(),
        None,
        TypeKnowledge::established(int_ty, EvidenceOrigin::Flow),
        true,
        InitializationState::DefinitelyInitialized,
    );

    let mut exit_b = FlowState::new();
    exit_b.declare_with_initialization(
        binding,
        "x",
        SourceRange::default(),
        None,
        TypeKnowledge::established(int_ty, EvidenceOrigin::Flow),
        true,
        InitializationState::DefinitelyInitialized,
    );

    let joined = FlowState::join(&[exit_a, exit_b], &mut store);
    assert_eq!(
        joined.get_binding(binding).expect("binding").initialization,
        InitializationState::DefinitelyInitialized
    );
}

#[test]
fn read_definitely_uninitialized_is_diagnostic() {
    let f = Fixture::new_allowing_internal_incidents(
        r#"
class Probe {
  @class
  run() {
    let mut x: Int
    let y = x
  }
}
"#,
    );
    let diags = f.diagnostics(DiagnosticCode::UninitializedBindingRead);
    assert_eq!(diags.len(), 1, "expected 1 uninitialized read diagnostic, found {diags:#?}");
    assert!(
        diags[0].message.contains("binding `x` is read before it is initialized"),
        "unexpected message: {}",
        diags[0].message
    );
}

#[test]
fn read_after_assignment_is_proven() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let mut x: Int
    x = 42
    let y = x
  }
}
"#,
    );
    f.assert_no_diagnostic(DiagnosticCode::UninitializedBindingRead);
    let run = f.callable("Probe", "run", DispatchSide::Class);
    // Find the read expression for `x` (last expression read)
    let read_safety = run
        .expressions
        .values()
        .filter_map(|expr| expr.binding_read_safety)
        .collect::<Vec<_>>();
    assert!(
        read_safety.contains(&BindingReadSafety::ProvenInitialized),
        "expected ProvenInitialized read, got {read_safety:#?}"
    );
}

#[test]
fn conditional_assignment_requires_runtime_check() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ cond: Bool) {
    let mut x: Int
    if cond {
      x = 42
    }
    let y = x
  }
}
"#,
    );
    f.assert_no_diagnostic(DiagnosticCode::UninitializedBindingRead);
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let read_safety = run
        .expressions
        .values()
        .filter_map(|expr| expr.binding_read_safety)
        .collect::<Vec<_>>();
    assert!(
        read_safety.contains(&BindingReadSafety::RuntimeCheckRequired),
        "expected RuntimeCheckRequired read, got {read_safety:#?}"
    );
}

#[test]
fn explicit_none_is_definitely_initialized() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let mut x: Option[Int] = None
    let y = x
  }
}
"#,
    );
    f.assert_no_diagnostic(DiagnosticCode::UninitializedBindingRead);
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let read_safety = run
        .expressions
        .values()
        .filter_map(|expr| expr.binding_read_safety)
        .collect::<Vec<_>>();
    assert!(
        read_safety.contains(&BindingReadSafety::ProvenInitialized),
        "expected ProvenInitialized read for explicit None, got {read_safety:#?}"
    );
}

#[test]
fn immutable_let_reassignment_is_diagnostic() {
    let f = Fixture::new_allowing_internal_incidents(
        r#"
class Probe {
  @class
  run() {
    let x = 1
    x = 2
  }
}
"#,
    );
    let all_diags = f.analysis.snapshot.all_diagnostics().collect::<Vec<_>>();
    let immutable_diag = all_diags
        .iter()
        .find(|d| d.message.contains("cannot reassign immutable binding `x`; declare it with `let mut` to allow mutation"));
    assert!(
        immutable_diag.is_some(),
        "expected immutable reassignment diagnostic, got: {all_diags:#?}"
    );
}

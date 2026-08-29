//! Intentional RED regressions for Phalcom Semantic Correctness Part 1.
//!
//! Intended repository path:
//!   phalcom-semantic/tests/semantic/foundations/semantic_correctness_regressions.rs
//!
//! These tests encode normative Part 1 semantics that are known to be violated
//! by the reviewed implementation. Do not weaken the assertions to match
//! current behavior. See the companion handoff markdown for integration order
//! and expected failure signatures.

use crate::semantic::support::Fixture;
use phalcom_common::range::SourceRange;
use phalcom_modules::DeclarationId;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::checker::analysis::{AnalysisStatus, normal_return_summary};
use phalcom_semantic::checker::causal::CausalInvalidity;
use phalcom_semantic::checker::flow::state::FlowState;
use phalcom_semantic::checker::inference::{ConstraintOrigin, InferenceFailureReason, InferenceOutcome, InferenceRelation, InferenceSession, InferenceTerm};
use phalcom_semantic::checker::{AssumptionBasis, BindingConsistency, BindingContractOrigin};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{BindingId, DispatchSide};
use phalcom_semantic::types::evidence::{DynamicReason, EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason, join_type_knowledge};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::outcome::BlockReason;
use phalcom_semantic::types::relation::{MapTypeHierarchy, RefutationReason};
use phalcom_semantic::types::store::TypeStore;

fn test_decl(name: &str) -> DeclarationId {
    DeclarationId::new(ModuleId::core(), name.into())
}

/// Part 1 invariant:
/// Causal invalidity is not expression suppression.
///
/// A refuted binding contract does not erase an independently established
/// current type. Reading that binding and dispatching on the established type
/// must remain analyzable, while carrying the upstream invalidity.
#[test]
fn causal_invalidity_does_not_suppress_analyzable_downstream_dispatch() {
    let f = Fixture::new(
        r#"
class CellNum {
  @constructor
  new() {}

  cellOnly() -> Int { 1 }
}

class Probe {
  @class
  run() {
    let x: Int = CellNum.new()
    let y = x.cellOnly()
  }
}
"#,
    );

    let cell_num = f.ty("CellNum");
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);

    let x = f.binding(run, "x");
    assert_eq!(x.current.ty(), Some(cell_num), "actual constructor fact must survive");
    assert_eq!(
        x.current.status(),
        Some(EvidenceStatus::Established),
        "refuted annotation must not weaken the independently established current fact",
    );
    assert!(
        matches!(
            x.consistency,
            BindingConsistency::Refuted {
                actual,
                expected,
                ..
            } if actual == cell_num && expected == int_ty
        ),
        "binding must retain the contradiction instead of overwriting current knowledge: {x:#?}",
    );
    assert!(
        !matches!(x.causal_invalidity, CausalInvalidity::Clean),
        "the refuted binding must retain its owning invalidity",
    );

    let downstream = f.expression(run, "x.cellOnly()");
    assert_eq!(
        downstream.knowledge.ty(),
        Some(int_ty),
        "dispatch remains resolvable from established CellNum recovery knowledge",
    );
    assert_eq!(
        downstream.knowledge.status(),
        Some(EvidenceStatus::Established),
        "the independently resolved call result remains established",
    );
    assert!(
        matches!(downstream.status, AnalysisStatus::Ready),
        "an analyzable expression with upstream invalid dependence is Ready, not Suppressed: {downstream:#?}",
    );
    assert!(
        !matches!(downstream.causal_invalidity, CausalInvalidity::Clean),
        "downstream value must still carry the upstream invalid dependency",
    );

    assert_eq!(f.binding(run, "y").current.ty(), Some(int_ty));
    f.assert_diagnostic(DiagnosticCode::BindingInitializerMismatch, 1);
}

/// Required-premise suppression is a local operation rule, not a projection
/// of every upstream causal invalidity. A failed generic result with no usable
/// receiver type suppresses the dependent dispatch operation.
#[test]
fn invalid_receiver_premise_produces_real_suppression() {
    let f = Fixture::new(
        r#"
class Allowed {
  allowedOnly() -> Int { 1 }
}

class Bad {
  @constructor
  new() {}
}

class Generic {
  @class
  id<T>(_ value: T) -> T where T <: Allowed {
    value
  }
}

class Probe {
  @class
  run() {
    let result = Generic.id(Bad.new()).allowedOnly()
  }
}
"#,
    );

    let run = f.callable("Probe", "run", DispatchSide::Class);
    let inner = f.expression(run, "Generic.id(Bad.new())");
    assert!(
        matches!(inner.status, AnalysisStatus::Invalid(_)),
        "generic constraint failure must own invalidity: {inner:#?}"
    );
    assert!(
        inner.knowledge.ty().is_none(),
        "failed generic result must not fabricate receiver type: {inner:#?}"
    );

    let outer = f.expression(run, "Generic.id(Bad.new()).allowedOnly()");
    assert!(
        matches!(outer.status, AnalysisStatus::Suppressed(_)),
        "receiver-dependent dispatch must be suppressed only after losing its required premise: {outer:#?}",
    );
    assert!(matches!(outer.knowledge, TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause)));
    assert!(!matches!(outer.causal_invalidity, CausalInvalidity::Clean));
}

/// Part 1 invariant:
/// Dynamic is a first-class knowledge state. A callable return summary must not
/// translate "no concrete TypeId" into "checker coverage gap".
#[test]
fn normal_return_summary_preserves_dynamic_reason() {
    let mut store = TypeStore::new();
    let dynamic = TypeKnowledge::Dynamic(DynamicReason::RuntimeReflection);
    let exit = phalcom_semantic::checker::analysis::NormalReturnFact {
        knowledge: dynamic.clone(),
        flow: Default::default(),
        status: phalcom_semantic::checker::analysis::AnalysisStatus::Ready,
        causal_invalidity: phalcom_semantic::checker::causal::CausalInvalidity::Clean,
    };

    let summary = normal_return_summary(&mut store, std::slice::from_ref(&exit));

    assert_eq!(summary, dynamic, "return summarization must preserve Dynamic and its reason",);
}

/// Part 1 invariant:
/// Existing Unknown reasons are semantic evidence. Return summarization may
/// join them, but may not rewrite every non-concrete result to
/// Unknown(UncheckedExpression).
#[test]
fn normal_return_summary_preserves_existing_unknown_reason() {
    let mut store = TypeStore::new();
    let unknown = TypeKnowledge::Unknown(UnknownReason::UnresolvedName("missing".into()));
    let exit = phalcom_semantic::checker::analysis::NormalReturnFact {
        knowledge: unknown.clone(),
        flow: Default::default(),
        status: phalcom_semantic::checker::analysis::AnalysisStatus::Ready,
        causal_invalidity: phalcom_semantic::checker::causal::CausalInvalidity::Clean,
    };

    let summary = normal_return_summary(&mut store, std::slice::from_ref(&exit));

    assert_eq!(summary, unknown, "return summarization must preserve the actual reason analysis is unknown",);
}

/// Part 1 invariant:
/// Flow joins are semantic algebra, not predecessor-order selection.
///
/// When multiple reachable paths are Unknown for different reasons, their
/// deterministic merged result cannot depend on predecessor order.
#[test]
fn knowledge_join_unknown_reason_is_order_independent() {
    let mut store = TypeStore::new();
    let unresolved = TypeKnowledge::Unknown(UnknownReason::UnresolvedName("left".into()));
    let blocked = TypeKnowledge::Unknown(UnknownReason::InferenceBlocked);

    let forward = join_type_knowledge(&mut store, [unresolved.clone(), blocked.clone()]);
    let reverse = join_type_knowledge(&mut store, [blocked, unresolved]);

    assert_eq!(
        forward, reverse,
        "reachable-flow merge must be permutation-stable; first predecessor cannot choose the Unknown reason",
    );
    assert!(
        matches!(forward, TypeKnowledge::Unknown(_)),
        "two unknown predecessors must remain honestly unknown",
    );
}

/// Amendment A invariant:
/// Final lower/upper reconciliation must report the upper bound that actually
/// failed, not `uppers[0]`.
///
/// The first upper (`Number`) is valid for `Int`; the second (`String`) is the
/// real conflict. This specifically exercises the solver's final bound
/// reconciliation rather than direct canonical/canonical structural failure.
#[test]
fn generic_conflict_reports_actual_failed_upper_bound() {
    let mut store = TypeStore::new();
    let mut hierarchy = MapTypeHierarchy::new();

    let int_decl = test_decl("Int");
    let number_decl = test_decl("Number");
    let string_decl = test_decl("String");

    hierarchy.insert(int_decl.clone(), number_decl.clone());

    let int_ty = store.nominal(int_decl);
    let number_ty = store.nominal(number_decl);
    let string_ty = store.nominal(string_decl);

    let mut session = InferenceSession::new();
    let var = session.fresh_variable(KindId::TYPE);

    // Lower bound: Int <: T
    session.add_constraint(
        InferenceRelation::Subtype(InferenceTerm::Canonical(int_ty), InferenceTerm::Var(var)),
        ConstraintOrigin::Explicit,
        None,
    );

    // Compatible upper first: T <: Number.
    session.add_constraint(
        InferenceRelation::Subtype(InferenceTerm::Var(var), InferenceTerm::Canonical(number_ty)),
        ConstraintOrigin::Explicit,
        None,
    );

    // Actual failed upper second: T <: String.
    session.add_constraint(
        InferenceRelation::Subtype(InferenceTerm::Var(var), InferenceTerm::Canonical(string_ty)),
        ConstraintOrigin::Explicit,
        None,
    );

    let outcome = session.solve(&mut store, &hierarchy);
    let InferenceOutcome::Conflicting(conflict) = outcome else {
        panic!("expected final conflicting-bound reconciliation");
    };

    match conflict.failure {
        InferenceFailureReason::ConflictingBounds { var: failed_var, lower, upper } => {
            assert_eq!(failed_var, var);
            assert_eq!(lower, int_ty);
            assert_eq!(
                upper, string_ty,
                "conflict payload must identify the upper bound whose subtype judgment actually failed",
            );
            assert_ne!(upper, number_ty, "the compatible first upper bound is not valid conflict evidence",);
        }
        other => panic!("expected ConflictingBounds, got {other:#?}"),
    }
}

/// Part 1 invariant:
/// Callable parameters enter the body as assumptions supplied by the callable
/// contract, not as raw developer-annotation evidence.
#[test]
fn callable_parameter_body_entry_uses_signature_assumption_provenance() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ value: Int) {
    let copy = value
  }
}
"#,
    );

    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let value = f.binding(run, "value");

    assert_eq!(value.current.ty(), Some(int_ty));
    assert_eq!(
        value.current.status(),
        Some(EvidenceStatus::Assumed),
        "callable entry is a usable contract-backed assumption",
    );
    assert_eq!(
        value.current.origin(),
        Some(EvidenceOrigin::CallableSignature),
        "body-entry evidence is justified by the callable signature, not merely by source annotation syntax",
    );

    let contract = value.contract.as_ref().expect("typed callable parameter must retain a persistent contract");
    assert_eq!(contract.ty, int_ty);
    assert_eq!(contract.origin, BindingContractOrigin::CallableParameter);

    assert!(
        matches!(
            value.consistency,
            BindingConsistency::Assumed {
                basis: AssumptionBasis::CallableParameterContract
            }
        ),
        "parameter consistency must explain that its assumption basis is the callable parameter contract: {value:#?}",
    );

    let read = f.expression(run, "value");
    assert_eq!(read.knowledge.ty(), Some(int_ty));
    assert_eq!(read.knowledge.status(), Some(EvidenceStatus::Assumed));
    assert_eq!(read.knowledge.origin(), Some(EvidenceOrigin::CallableSignature),);
}

/// Part 1 invariant:
/// Exact constructor semantics are a distinct derivation origin. Constructor
/// results must not be flattened into ordinary CallableSignature evidence.
#[test]
fn constructor_result_uses_constructor_semantics_origin() {
    let f = Fixture::new(
        r#"
class CellNum {
  @constructor
  new() {}
}

class Probe {
  @class
  run() {
    let x = CellNum.new()
  }
}
"#,
    );

    let cell_num = f.ty("CellNum");
    let run = f.callable("Probe", "run", DispatchSide::Class);

    let call = f.expression(run, "CellNum.new()");
    assert_eq!(call.knowledge.ty(), Some(cell_num));
    assert_eq!(call.knowledge.status(), Some(EvidenceStatus::Established));
    assert_eq!(
        call.knowledge.origin(),
        Some(EvidenceOrigin::ConstructorSemantics),
        "constructor Self result must record the semantic rule that established it",
    );

    let x = f.binding(run, "x");
    assert_eq!(x.current.ty(), Some(cell_num));
    assert_eq!(x.current.status(), Some(EvidenceStatus::Established));
    assert_eq!(x.current.origin(), Some(EvidenceOrigin::ConstructorSemantics),);
}

/// Part 1 invariant:
/// An inferred initializer contract preserves monomorphic reassignment policy,
/// but it is not an explicit developer declaration.
///
/// This test intentionally targets the compatibility mirror because publishing
/// `declared = Some(Int)` for `let x = 1` lies to downstream semantic consumers.
#[test]
fn inferred_initializer_contract_is_not_published_as_explicit_declaration() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let x = 1
  }
}
"#,
    );

    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let x = f.binding(run, "x");

    assert_eq!(x.current.ty(), Some(int_ty));
    assert_eq!(x.current.status(), Some(EvidenceStatus::Established));

    let contract = x
        .contract
        .as_ref()
        .expect("known unannotated initializer retains an inferred monomorphic contract");
    assert_eq!(contract.ty, int_ty);
    assert_eq!(contract.origin, BindingContractOrigin::InferredInitializer);

    assert_eq!(
        x.declared_type(),
        Some(int_ty),
        "compatibility reads derive from the persistent contract rather than a declaration mirror",
    );
    assert_eq!(
        x.contract.as_ref().map(|contract| contract.origin),
        Some(BindingContractOrigin::InferredInitializer),
        "the contract origin keeps inferred initializer evidence distinct from an explicit declaration",
    );
}

/// Part 1 invariant:
/// Loop widening changes current knowledge and must then reconcile that joined
/// current fact against the invariant persistent contract.
///
/// Merely noticing predecessor consistency disagreement and replacing the
/// semantic relation with RecursiveFixpoint loses available evidence.
#[test]
fn loop_widening_reconciles_joined_current_against_persistent_contract() {
    let mut store = TypeStore::new();

    let int_ty = store.nominal(test_decl("Int"));
    let string_ty = store.nominal(test_decl("String"));
    let number_ty = store.nominal(test_decl("Number"));
    let binding = BindingId(77);

    let mut header = FlowState::new();
    header.declare(
        binding,
        "value",
        SourceRange::default(),
        Some(number_ty),
        TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax),
        true,
    );
    header.bindings.get_mut(&binding).expect("header binding").consistency = BindingConsistency::Validated;

    let mut next = header.clone();
    {
        let next_binding = next.bindings.get_mut(&binding).expect("next binding");
        next_binding.current = TypeKnowledge::established(string_ty, EvidenceOrigin::Syntax);
        next_binding.consistency = BindingConsistency::Refuted {
            actual: string_ty,
            expected: number_ty,
            reason: RefutationReason::IncompatibleNominal,
        };
    }

    let widened = FlowState::widen_loop_state(&header, &next, &mut store).expect("matching loop invariants");
    let widened_binding = widened.get_binding(binding).expect("widened binding");
    let joined_ty = store.union(&[int_ty, string_ty]);

    assert_eq!(widened_binding.current.ty(), Some(joined_ty));
    assert_eq!(
        widened_binding.contract.as_ref().map(|contract| contract.ty),
        Some(number_ty),
        "loop widening must retain the persistent contract",
    );
    assert!(
        matches!(
            widened_binding.consistency,
            BindingConsistency::Refuted {
                actual,
                expected,
                ..
            } if actual == joined_ty && expected == number_ty
        ),
        "widened knowledge must be re-reconciled against the persistent contract instead of becoming an unrelated RecursiveFixpoint block: {widened_binding:#?}",
    );
}

/// Part 1 invariant:
/// Expected-type checking affects AnalysisStatus without overwriting knowledge.
///
/// An unresolved actual under an `Int` binding contract remains unknown and the
/// checking judgment is Blocked; it is not Ready merely because no refutation
/// could be produced.
#[test]
fn expected_contract_blockage_reaches_expression_status() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let x: Int = missing
  }
}
"#,
    );

    let run = f.callable("Probe", "run", DispatchSide::Class);
    let missing = f.expression(run, "missing");

    assert!(
        matches!(missing.knowledge, TypeKnowledge::Unknown(UnknownReason::UnresolvedName(_))),
        "unresolved source name must remain honest unknown knowledge: {missing:#?}",
    );
    assert!(
        matches!(
            missing.status,
            AnalysisStatus::Blocked(BlockReason::UnknownType(UnknownReason::UnresolvedName(_)))
        ),
        "the expected-type relation could not be judged; expression status must expose Blocked rather than Ready: {missing:#?}",
    );

    let x = f.binding(run, "x");
    assert!(
        matches!(x.current, TypeKnowledge::Unknown(UnknownReason::UnresolvedName(_))),
        "the declaration contract must not launder an unresolved name into assumed Int",
    );
    assert!(
        matches!(
            x.consistency,
            BindingConsistency::Blocked(BlockReason::UnknownType(UnknownReason::UnresolvedName(_)))
        ),
        "binding reconciliation must preserve the actual blocked reason",
    );
}

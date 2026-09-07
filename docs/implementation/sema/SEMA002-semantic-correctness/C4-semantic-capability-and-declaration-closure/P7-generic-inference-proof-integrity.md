# Phalcom Generic Inference and Proof Integrity Implementation Plan

> **For agentic workers:** Implement this plan task-by-task using RED → minimal implementation → focused GREEN verification → commit. Do not fold Technical Specification 04 receiver/class specialization into this work.

**Goal:** Make generic inference proof-complete: a substitution may solve without authorizing a known result, every required generic value premise remains represented even when Unknown/Dynamic, expected context remains non-evidentiary, generic calls reuse Technical 02 argument binding, inference-variable subtype relations remain directed, and solver terminal outcomes participate in the checker's real cancellation/budget/status domain.

**Architecture:** Keep `checker/inference.rs` as the owner of solver-local inference terms, substitutions, relation solving, support and new proof-state tracking. Keep `checker/call.rs` as the owner of generic callable application and publication. Reuse Technical 02's `ArgumentBindingPlan`; do not create a new binder. Reuse `types/evidence.rs` reason-join policy; do not create a second Unknown/Dynamic precedence table. Reuse `CheckingContext::CheckerControl` for solver budget/cancellation and `InternalSemanticIncidentKind::InferenceInvariantViolation` for internal solver failures.

**Tech stack:** Rust 2024, Cargo workspace, `phalcom-semantic`, semantic integration binary `tests/semantic.rs`, existing `Fixture` harness.

**Spec:** `docs/impl/semantic/semantic-correctness/part-4/phalcom-semantic-correctness-technical-03-generic-inference-proof-integrity-spec.md`

**Verified baseline:** `aureat/phalcom-lang` `main` at `4599dad282c014669f39e5f42d382e48f89aca9b`.

---

# Global constraints

- Technical Specification 01 is a prerequisite: required-premise completeness and exact Unknown/Dynamic reason preservation are already canonical laws.
- Technical Specification 02 is a prerequisite: `CallableApplicationTarget`, `CallPremise`, `ApplicationArgument`, `ArgumentBindingPlan`, `bind_static_arguments(...)`, and `apply_resolved_callable(...)` remain the outer application architecture.
- `checker/inference.rs` owns solver-local substitution/proof mechanics.
- `checker/call.rs` owns generic call argument analysis and result publication.
- Expected type/context is control information, never value evidence.
- Unknown/Dynamic generic arguments are not converted to fake `TypeId`s.
- Assumed evidence never upgrades to Established.
- A generic call may have known independent return knowledge and a non-Ready sibling status.
- Do not implement Technical 04 receiver/class specialization.
- Do not broaden rest/spread semantics. Unsupported shapes fail closed through Technical 02.
- Every supplied argument must be analyzed exactly once in source order.
- Do not weaken existing tests to match current behavior.
- Every task ends with focused tests and a reviewable commit.
- Use the repository's existing semantic test entry point:
  `cargo test -p phalcom-semantic --test semantic <test-filter>`.
- Current hosted CI has unrelated toolchain/runner problems. Local/fresh command evidence remains required; document infrastructure failures rather than calling them semantic failures.

---

# 1. Pre-execution gate

## Files to inspect

```text
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expected.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/incident.rs
phalcom-semantic/src/types/evidence.rs
phalcom-semantic/src/types/outcome.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/tests/semantic/foundations/inference.rs
phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs
phalcom-semantic/tests/semantic/capabilities/generics.rs
```

## Verify Technical 02 API exists

Run:

```bash
rg -n \
  'CallableApplicationTarget|ArgumentBindingPlan|bind_static_arguments|apply_resolved_callable' \
  phalcom-semantic/src/checker/call.rs
```

Expected: all anchors exist.

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application
```

Expected: PASS before Technical 03 changes.

## Verify current generic baseline

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::inference

cargo test -p phalcom-semantic --test semantic \
  semantic::capabilities::generics
```

Record the exact baseline.

Do not proceed if Technical 02's canonical application suite is already failing for an unexplained reason.

**Commit boundary:** none; investigation only.

---

# 2. Current baseline map

| File | Anchor | Current behavior | Technical 03 action |
|---|---|---|---|
| `checker/call.rs` | `apply_generic_callable_inner` | Manual generic argument matching; Unknown/Dynamic premise omission | Replace mapping and add proof accounting |
| `checker/call.rs` | `terminal_generic_return` | Accepts fallback knowledge not explicitly classified by independence phase | Restrict fallback semantics |
| `checker/call.rs` | `apply_resolved_callable` | Correct outer premise authority cap | Preserve |
| `checker/call.rs` | `bind_static_arguments` | Canonical Technical 02 binder | Reuse in generic path |
| `checker/inference.rs` | `InferenceSupport` | Established/Assumed monotone support | Preserve |
| `checker/inference.rs` | `term_support` | Cannot represent Unknown/Dynamic proof premises | Add parallel proof-state domain |
| `checker/inference.rs` | `subtype_terms` | Unresolved `Var <: Var` can fall back to unification | Introduce directed subtype edge |
| `checker/inference.rs` | `solve` | Structured outcomes but no shared checker cancellation/budget input | Add controlled solve path |
| `checker/expected.rs` | `ExpectedType` | Explicitly contextual/non-evidentiary | Preserve |
| `checker/context.rs` | `CheckerControl` | Shared `QueryBudget` + `CancellationToken` | Reuse |
| `checker/context.rs` | `record_internal_incident` | Existing invariant incident publication | Reuse for inference invariant failures |
| `types/evidence.rs` | Unknown/Dynamic reason join | Canonical deterministic reason policy | Expose crate-local helper for proof-state meet |
| `types/substitution.rs` | `TypeSubstitution` | Canonical structural substitution | Consume only; no Spec04 redesign |
| `foundations/inference.rs` | solver tests | Kinds/occurs/bounds/support | Extend |
| `capabilities/generics.rs` | call-level generic tests | basic support/context/fixed returns | Extend proof-integrity regressions |

---

# 3. Task decomposition

```text
Task 1   Add RED call-level proof-integrity regressions
Task 2   Add InferenceProofState and shared reason-meet policy
Task 3   Record required inference premises and representative proof state
Task 4   Reuse Technical 02 ArgumentBindingPlan in generic application
Task 5   Propagate generic argument status, causality, Unknown, and Dynamic honestly
Task 6   Separate value/declaration solving from expected-result selection
Task 7   Publish generic returns from proof state, not substitution alone
Task 8   Restrict terminal fallback and prove fixed-result independence
Task 9   Preserve directed Var <: Var relations and make solving order-stable
Task 10  Wire generic solving to shared cancellation and query budget
Task 11  Route inference invariant failures to analyzer incidents
Task 12  Improve generic conflict provenance/diagnostic targeting
Task 13  Delete duplicate/legacy generic application shortcuts and audit structure
Task 14  Full regression matrix and closure gate
```

---

# 4. Task 1 — Add RED call-level proof-integrity regressions

**Files:**
- Create: `phalcom-semantic/tests/semantic/foundations/generic_inference_proof_integrity.rs`
- Modify: `phalcom-semantic/tests/semantic/foundations/mod.rs`

**Consumes:**
- `Fixture`
- `AnalysisStatus`
- `TypeKnowledge`
- `UnknownReason`
- `DynamicReason`
- `EvidenceStatus`
- `DispatchSide`

**Produces:** a focused suite that demonstrates the current correctness failures before production changes.

## Step 1.1 — Register the module

Add to `foundations/mod.rs`:

```rust
mod generic_inference_proof_integrity;
```

## Step 1.2 — RED: unresolved required premise cannot disappear

Add:

```rust
#[test]
fn unresolved_required_generic_premise_prevents_known_dependent_result() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  first<T>(_ first: T, _ second: T) -> T {
    first
  }

  @class
  run() {
    let result = Probe.first(1, missing)
  }
}
"#,
    );

    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "Probe.first(1, missing)");

    assert_eq!(
        call.knowledge,
        TypeKnowledge::Unknown(
            UnknownReason::UnresolvedName("missing".into())
        ),
        "a second required generic premise may not disappear merely because the first argument solves T",
    );
    assert!(
        matches!(call.status, AnalysisStatus::Blocked(_)),
        "{call:#?}",
    );
}
```

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  unresolved_required_generic_premise_prevents_known_dependent_result
```

Expected before implementation: FAIL because current code can solve `T = Int` from the first argument and omit the unresolved second premise.

## Step 1.3 — RED: fixed return survives unresolved generic argument

Add:

```rust
#[test]
fn unresolved_generic_premise_blocks_call_without_erasing_fixed_return() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  fixed<T>(_ first: T, _ second: T) -> Int {
    1
  }

  @class
  run() {
    let result = Probe.fixed(1, missing)
  }
}
"#,
    );

    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "Probe.fixed(1, missing)");

    assert_eq!(call.knowledge.ty(), Some(int_ty));
    assert_eq!(
        call.knowledge.status(),
        Some(EvidenceStatus::Established),
    );
    assert!(matches!(call.status, AnalysisStatus::Blocked(_)));
}
```

This may partially pass already; keep it as an independence invariant.

## Step 1.4 — RED: expected context cannot rescue unresolved premise

Add:

```rust
#[test]
fn expected_result_cannot_upgrade_unresolved_generic_premise() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  identity<T>(_ value: T) -> T {
    value
  }

  @class
  run() {
    let result: Int = Probe.identity(missing)
  }
}
"#,
    );

    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "Probe.identity(missing)");

    assert_eq!(
        call.knowledge,
        TypeKnowledge::Unknown(
            UnknownReason::UnresolvedName("missing".into())
        ),
    );
}
```

Run the three tests.

Expected: at least the dependent-result tests fail on the baseline.

## Step 1.5 — RED: Dynamic required premise remains Dynamic

Source `Dynamic` is already a recognized type annotation.

Add:

```rust
#[test]
fn dynamic_generic_premise_produces_dynamic_dependent_result() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  identity<T>(_ value: T) -> T {
    value
  }

  @class
  run(value: Dynamic) {
    let result = Probe.identity(value)
  }
}
"#,
    );

    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "Probe.identity(value)");

    assert!(matches!(call.knowledge, TypeKnowledge::Dynamic(_)));
    assert!(matches!(
        call.status,
        AnalysisStatus::DynamicBoundary(_)
    ));
}
```

Do not over-specify a DynamicReason in this first RED unless the body-entry annotation path is confirmed to preserve `ExplicitEscape`; add the exact-reason assertion after the focused test shows the current source behavior.

**GREEN criteria for Task 1:** none; tests are intentionally RED.

**Commit:**

```text
test(semantic): add generic proof integrity regressions
```

---

# 5. Task 2 — Add `InferenceProofState` and shared reason-meet policy

**Files:**
- Modify: `phalcom-semantic/src/types/evidence.rs`
- Modify: `phalcom-semantic/src/checker/inference.rs`
- Modify: `phalcom-semantic/tests/semantic/foundations/inference.rs`

**Consumes:**
- `UnknownReason`
- `DynamicReason`
- canonical reason precedence already used by `compose_required_knowledge`

**Produces:**
- `InferenceProofState`
- one deterministic proof-state meet law
- no duplicated Unknown/Dynamic precedence table

## Step 2.1 — RED proof-state meet tests

In `foundations/inference.rs`, add:

```rust
#[test]
fn inference_proof_state_preserves_unknown_over_known_support() {
    use phalcom_semantic::checker::inference::InferenceProofState;
    use phalcom_semantic::types::evidence::UnknownReason;

    let state = InferenceProofState::Established.meet(
        InferenceProofState::Unknown(
            UnknownReason::UnresolvedName("missing".into())
        )
    );

    assert_eq!(
        state,
        InferenceProofState::Unknown(
            UnknownReason::UnresolvedName("missing".into())
        )
    );
}
```

Add tests for:

```text
Established + Assumed -> Assumed
Established + Dynamic(D) -> Dynamic(D)
Unknown(A) + Dynamic(D) -> Unknown(A)
Unknown(A) + Unknown(B) -> deterministic exact joined reason
```

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::inference
```

Expected: FAIL because `InferenceProofState` does not exist.

## Step 2.2 — Expose existing reason-join helpers crate-locally

In `types/evidence.rs`, change only visibility:

```rust
pub(crate) fn join_unknown_reason(...)
pub(crate) fn join_dynamic_reason(...)
```

or add one crate-local helper that delegates to the same current logic.

Do not copy `unknown_reason_rank(...)` into `inference.rs`.

## Step 2.3 — Add proof-state enum

In `checker/inference.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InferenceProofState {
    Established,
    Assumed,
    Unknown(UnknownReason),
    Dynamic(DynamicReason),
}
```

Add:

```rust
impl InferenceProofState {
    pub fn from_knowledge(knowledge: &TypeKnowledge) -> Self {
        match knowledge {
            TypeKnowledge::Known(evidence) => match evidence.status() {
                EvidenceStatus::Established => Self::Established,
                EvidenceStatus::Assumed => Self::Assumed,
            },
            TypeKnowledge::Unknown(reason) => Self::Unknown(reason.clone()),
            TypeKnowledge::Dynamic(reason) => Self::Dynamic(reason.clone()),
        }
    }

    pub fn meet(self, other: Self) -> Self {
        // Unknown first, Dynamic second, Assumed weakens Established.
        // Use evidence.rs canonical reason joins.
    }
}
```

Do not attach `TypeId` to this enum.

## Step 2.4 — GREEN

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::inference
```

Expected: PASS.

**Commit:**

```text
feat(semantic): add generic inference proof-state domain
```

---

# 6. Task 3 — Record required inference premises and representative proof state

**Files:**
- Modify: `phalcom-semantic/src/checker/inference.rs`
- Modify: `phalcom-semantic/tests/semantic/foundations/inference.rs`

**Produces:**

```rust
RequiredInferencePremise
InferenceSession::record_required_premise(...)
InferenceSession::proof_state_for_term(...)
```

## Step 3.1 — RED: missing premise survives another solvable constraint

Add a solver-level test:

```rust
#[test]
fn return_proof_remembers_unknown_required_premise_after_substitution_solves() {
    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();
    let int_ty = store.nominal(test_decl("Int"));

    let mut session = InferenceSession::new();
    let t = session.fresh_variable(KindId::TYPE);
    let term = InferenceTerm::Var(t);

    session.record_required_premise(
        &term,
        ConstraintOrigin::Explicit,
        &TypeKnowledge::established(
            int_ty,
            EvidenceOrigin::Syntax,
        ),
        None,
    );
    session.add_constraint_with_support(
        InferenceRelation::Subtype(
            InferenceTerm::Canonical(int_ty),
            term.clone(),
        ),
        ConstraintOrigin::Explicit,
        None,
        InferenceSupport::Established,
    );

    session.record_required_premise(
        &term,
        ConstraintOrigin::Explicit,
        &TypeKnowledge::Unknown(
            UnknownReason::UnresolvedName("missing".into())
        ),
        None,
    );

    let outcome = session.solve(&mut store, &hier);
    assert!(outcome.is_solved());
    assert_eq!(
        session.proof_state_for_term(&term),
        InferenceProofState::Unknown(
            UnknownReason::UnresolvedName("missing".into())
        )
    );
}
```

Run and confirm RED.

## Step 3.2 — Add `RequiredInferencePremise`

In `inference.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequiredInferencePremise {
    pub term: InferenceTerm,
    pub origin: ConstraintOrigin,
    pub proof: InferenceProofState,
    pub explanation: Option<ExplanationId>,
}
```

## Step 3.3 — Add representative proof storage

Preferred minimal implementation: add to `InferenceVariable`:

```rust
pub proof: Option<InferenceProofState>,
```

Initialize to `None` in `fresh_variable`.

Do **not** initialize it to Established. `None` means:

```text
no value proof has yet justified this variable
```

which is different from established support.

## Step 3.4 — Add session premise storage

Add:

```rust
required_premises: Vec<RequiredInferencePremise>,
```

to `InferenceSession`.

## Step 3.5 — Implement recursive proof recording

Add private helper:

```rust
fn record_term_proof_state(
    &mut self,
    term: &InferenceTerm,
    proof: InferenceProofState,
)
```

Traverse the same inference-term forms already supported by `record_term_support`:

```text
Var
Applied
Union
Tuple
Callable
Canonical
```

For each variable representative, meet the new state into its existing state.

## Step 3.6 — Merge proof during aliases

In `alias_variables(...)`, beside support merging:

```rust
let merged_proof = meet_optional_proof(
    self.variables[rep_a].proof.clone(),
    self.variables[rep_b].proof.clone(),
);
self.variables[rep_b].proof = merged_proof;
```

Do not allow the alias to drop Unknown/Dynamic.

## Step 3.7 — Implement `record_required_premise`

```rust
pub fn record_required_premise(
    &mut self,
    term: &InferenceTerm,
    origin: ConstraintOrigin,
    knowledge: &TypeKnowledge,
    explanation: Option<ExplanationId>,
) {
    let proof = InferenceProofState::from_knowledge(knowledge);
    self.record_term_proof_state(term, proof.clone());
    self.required_premises.push(RequiredInferencePremise {
        term: term.clone(),
        origin,
        proof,
        explanation,
    });
}
```

## Step 3.8 — Implement `proof_state_for_term`

Collect representatives for the term.

Rules:

```text
no variables
    -> Established

variable proof Some(state)
    -> state

variable proof None
    -> Unknown(UnderconstrainedTypeVariable)

multiple variables
    -> meet
```

Do not use `term_support(...)` as a proxy.

## Step 3.9 — Add alias and compound tests

Add tests:

- Established + Assumed premises alias -> Assumed proof;
- tuple return `(T, U)` meets both states;
- a variable solved only from an ordinary no-support constraint returns `Unknown(UnderconstrainedTypeVariable)` proof.

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::inference
```

Expected: PASS.

**Commit:**

```text
feat(semantic): track required generic proof premises
```

---

# 7. Task 4 — Reuse Technical 02 `ArgumentBindingPlan` in generic application

**Files:**
- Modify: `phalcom-semantic/src/checker/call.rs`
- Modify: `phalcom-semantic/tests/semantic/foundations/generic_inference_proof_integrity.rs`
- Modify: `phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs` only if a shared binder test belongs there

**Consumes:**
- `bind_static_arguments(...)`
- `ArgumentBindingPlan`
- `emit_shape_failures(...)`
- `ApplicationArgument`

## Step 4.1 — Add RED generic-shape regression

Choose a source form already accepted by the parser and callable selector model. Add a wrong-arity generic call:

```rust
#[test]
fn generic_shape_failure_does_not_partial_specialize_return() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  pair<T>(_ left: T, _ right: T) -> T {
    left
  }

  @class
  run() {
    let value = Probe.pair(1)
  }
}
"#,
    );

    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "Probe.pair(1)");

    assert!(matches!(call.status, AnalysisStatus::Invalid(_)));
    assert!(
        call.knowledge.ty().is_none(),
        "invalid static shape must not publish T = Int from the partial binding: {call:#?}",
    );
}
```

If selector resolution rejects the source before a callable target is obtained, instead use a labeled mismatch form that reaches the current generic binder. Keep the semantic assertion: no partial specialization after shape failure.

## Step 4.2 — Build source-index mapping from the canonical plan

Inside `apply_generic_callable_inner(...)`:

```rust
let plan = bind_static_arguments(
    arguments,
    &target.signature.parameters,
);
```

Create:

```rust
let mut parameter_for_source = vec![None; arguments.len()];
for binding in &plan.bindings {
    parameter_for_source[binding.source_index] =
        Some(binding.parameter_index);
}
```

## Step 4.3 — Delete manual generic matching

Delete the generic-only:

```text
positional_idx
label position search
parameter_index = ...
continue
```

Do not leave it as fallback.

## Step 4.4 — Analyze source arguments from the plan

Iterate `arguments.iter().enumerate()` in source order.

For a safe canonical binding, construct the parameter inference expectation.

For an unbound source argument, analyze with `ExpectedType::None`.

All arguments are analyzed once.

## Step 4.5 — Shape-failure termination

After all supplied arguments have been analyzed, if:

```rust
!plan.failures.is_empty()
```

then:

1. call `emit_shape_failures(...)`;
2. do not run partial generic solving for publication;
3. compute the return term only to determine whether the return is fixed;
4. fixed return may survive;
5. dependent return becomes `Unknown(InferenceBlocked)`.

## Step 4.6 — GREEN

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  generic_shape_failure_does_not_partial_specialize_return

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application
```

Expected: PASS.

**Commit:**

```text
refactor(semantic): use canonical binding plan for generic calls
```

---

# 8. Task 5 — Propagate generic argument status, causality, Unknown, and Dynamic honestly

**Files:**
- Modify: `phalcom-semantic/src/checker/call.rs`
- Modify: `phalcom-semantic/tests/semantic/foundations/generic_inference_proof_integrity.rs`

## Step 5.1 — Mirror non-generic dependency capture

Immediately after each generic argument is analyzed, perform the same call-capture operations as the non-generic engine:

```rust
ctx.record_call_dependency(
    argument_typed.causal_invalidity,
    argument_typed.explanation,
);

if !argument_typed.status.is_ready() {
    ctx.record_call_status(argument_typed.status.clone());
}
```

Use the actual existing readiness helper/`matches!` style used by the non-generic path.

## Step 5.2 — Record required premise before `TypeId` filtering

For each canonically bound parameter inference term:

```rust
let origin = ConstraintOrigin::Argument {
    call: call_id,
    argument: argument_id,
    parameter_index,
};

session.record_required_premise(
    &parameter_term,
    origin.clone(),
    &argument_typed.knowledge,
    argument_typed.explanation,
);
```

## Step 5.3 — Add solver relation only for Known evidence

Replace the old `if let Some(ty)` omission pattern with:

```rust
match &argument_typed.knowledge {
    TypeKnowledge::Known(evidence) => {
        session.add_constraint_with_support(
            InferenceRelation::Subtype(
                InferenceTerm::Canonical(evidence.ty()),
                parameter_term,
            ),
            origin,
            argument_typed.explanation,
            match evidence.status() {
                EvidenceStatus::Established =>
                    InferenceSupport::Established,
                EvidenceStatus::Assumed =>
                    InferenceSupport::Assumed,
            },
        );
    }

    TypeKnowledge::Unknown(reason) => {
        ctx.record_call_status(
            AnalysisStatus::Blocked(
                BlockReason::UnknownType(reason.clone())
            )
        );
    }

    TypeKnowledge::Dynamic(reason) => {
        ctx.record_call_status(
            AnalysisStatus::DynamicBoundary(reason.clone())
        );
    }
}
```

Do not invent a canonical type for Unknown/Dynamic.

## Step 5.4 — GREEN missing-premise tests

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  unresolved_required_generic_premise_prevents_known_dependent_result

cargo test -p phalcom-semantic --test semantic \
  unresolved_generic_premise_blocks_call_without_erasing_fixed_return

cargo test -p phalcom-semantic --test semantic \
  dynamic_generic_premise_produces_dynamic_dependent_result
```

The first test may still fail until result publication changes in Task 7. At this task gate, verify at least that call status/proof premise is captured using a focused solver assertion or temporary debugging assertion; do not weaken the final call assertion.

**Commit:**

```text
fix(semantic): preserve unavailable generic argument premises
```

---

# 9. Task 6 — Separate value/declaration solving from expected-result selection

**Files:**
- Modify: `phalcom-semantic/src/checker/call.rs`
- Modify: `phalcom-semantic/tests/semantic/foundations/generic_inference_proof_integrity.rs`
- Modify: `phalcom-semantic/tests/semantic/capabilities/generics.rs`

## Step 6.1 — Preserve explicit two-phase solving

Refactor `apply_generic_callable_inner(...)` into clearly named stages:

```text
argument/declaration constraints
        ↓
value_outcome = solve
        ↓
value_result = derive result/proof if possible
        ↓
expected-result constraint, if allowed
        ↓
context_outcome = solve
        ↓
final result
```

Do not add expected context before the initial value/declaration solve.

## Step 6.2 — Add a private phase carrier

Use a small internal struct or locals equivalent to:

```rust
struct PreContextGenericResult {
    knowledge: TypeKnowledge,
    proof: InferenceProofState,
}
```

Only populate it if the result proposition was complete before expected-result context was added.

## Step 6.3 — Expected result constraint gets no support/proof

The expected constraint remains:

```rust
session.add_constraint(
    InferenceRelation::Subtype(
        return_term.clone(),
        InferenceTerm::Canonical(expected_ty),
    ),
    ConstraintOrigin::ExpectedResult,
    None,
);
```

It must **not** call:

```rust
add_constraint_with_support(...)
record_required_premise(...)
```

## Step 6.4 — Do not add expected constraints after terminal value solve

If the first outcome is:

```text
Conflicting
Blocked
Cancelled
BudgetExceeded
InternalFailure
```

return/project that terminal outcome.

Do not attempt a second solve.

## Step 6.5 — RED/verify expected-only result

The existing capability test:

```text
expected_context_cannot_fabricate_missing_generic_return
```

must remain.

Add or strengthen:

```rust
assert_eq!(
    call.knowledge,
    TypeKnowledge::Unknown(
        UnknownReason::UnderconstrainedTypeVariable
    )
);
```

if current end-state reason policy is reached directly.

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  expected_context_cannot_fabricate_missing_generic_return

cargo test -p phalcom-semantic --test semantic \
  expected_result_cannot_upgrade_unresolved_generic_premise
```

Expected after Task 7: PASS.

## Step 6.6 — Preserve precise pre-context result

Keep/strengthen:

```text
expected_result_context_constrains_generic_without_merely_overwriting_call_fact
```

Expected:

```text
identity(42) under Number context
call result = Established(Int)
```

not `Number`.

**Commit:**

```text
refactor(semantic): separate generic value proof from contextual selection
```

---

# 10. Task 7 — Publish generic returns from proof state, not substitution alone

**Files:**
- Modify: `phalcom-semantic/src/checker/call.rs`
- Modify: `phalcom-semantic/src/checker/inference.rs`
- Modify: `phalcom-semantic/tests/semantic/foundations/generic_inference_proof_integrity.rs`
- Modify: `phalcom-semantic/tests/semantic/capabilities/generics.rs`

## Step 7.1 — Add one result-publication helper

Add a helper equivalent to:

```rust
fn materialize_proven_generic_return(
    ctx: &mut CheckingContext<'_>,
    session: &InferenceSession,
    return_term: &InferenceTerm,
    outcome: &InferenceOutcome,
    call_range: SourceRange,
) -> TypeKnowledge
```

Behavior for `Solved`:

```rust
let proof = session.proof_state_for_term(return_term);

match proof {
    InferenceProofState::Established => {
        let ty = session.materialize(return_term, ctx.store)
            .map_err(internal incident path)?;
        TypeKnowledge::established(
            ty,
            EvidenceOrigin::GenericInference,
        )
    }

    InferenceProofState::Assumed => {
        let ty = session.materialize(return_term, ctx.store)
            .map_err(internal incident path)?;
        TypeKnowledge::assumed(
            ty,
            EvidenceOrigin::GenericInference,
        )
    }

    InferenceProofState::Unknown(reason) =>
        TypeKnowledge::Unknown(reason),

    InferenceProofState::Dynamic(reason) =>
        TypeKnowledge::Dynamic(reason),
}
```

Do not materialize merely to discover proof afterward.

## Step 7.2 — Fixed return bypass

If the return term contains no inference variables and the exact callable contract has a fixed return, use the existing fixed-return promotion path.

Do not force it through generic proof-state weakening from unrelated variables.

## Step 7.3 — Underconstrained dependent result

Return:

```rust
TypeKnowledge::Unknown(
    UnknownReason::UnderconstrainedTypeVariable
)
```

and record Blocked status when the return itself remains statically unprovable.

## Step 7.4 — GREEN core regressions

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  unresolved_required_generic_premise_prevents_known_dependent_result

cargo test -p phalcom-semantic --test semantic \
  expected_result_cannot_upgrade_unresolved_generic_premise

cargo test -p phalcom-semantic --test semantic \
  dynamic_generic_premise_produces_dynamic_dependent_result

cargo test -p phalcom-semantic --test semantic \
  semantic::capabilities::generics
```

Expected: PASS.

## Step 7.5 — Verify existing authority tests

Specifically verify:

```text
assumed_generic_argument_yields_assumed_generic_return
mixed_generic_return_uses_weakest_value_support
independent_fixed_generic_return_stays_established
generic_call_on_assumed_receiver_is_capped
```

Run the relevant foundations/capability filters.

**Commit:**

```text
fix(semantic): publish generic returns from complete proof state
```

---

# 11. Task 8 — Restrict terminal fallback and prove fixed-result independence

**Files:**
- Modify: `phalcom-semantic/src/checker/call.rs`
- Modify: `phalcom-semantic/tests/semantic/foundations/generic_inference_proof_integrity.rs`
- Modify: `phalcom-semantic/tests/semantic/foundations/semantic_correctness_regressions.rs`

## Step 8.1 — Refactor `terminal_generic_return`

Change the helper so the fallback parameter means exactly:

```text
inference-independent fixed return
```

Rename if helpful:

```rust
terminal_generic_return_with_fixed_fallback(...)
```

Do not pass partially specialized dependent returns into it.

## Step 8.2 — Context-phase terminal behavior

When the value/declaration phase has already produced a complete `PreContextGenericResult`, a later expected-context terminal event may preserve that pre-context knowledge because its proof does not depend on the failed contextual selection.

Make this an explicit branch.

Do not generalize this to arbitrary partial solver state.

## Step 8.3 — RED dependent conflict

Add:

```rust
#[test]
fn dependent_generic_conflict_does_not_publish_partial_specialization() {
    let f = Fixture::new(
        r#"
class Allowed {}
class Bad { @constructor new() {} }

class Probe {
  @class
  constrained<T>(_ value: T) -> T
  where T <: Allowed {
    value
  }

  @class
  run() {
    let value = Probe.constrained(Bad.new())
  }
}
"#,
    );

    let run = f.callable("Probe", "run", DispatchSide::Class);
    let call = f.expression(run, "Probe.constrained(Bad.new())");

    assert!(matches!(call.status, AnalysisStatus::Invalid(_)));
    assert_eq!(
        call.knowledge,
        TypeKnowledge::Unknown(UnknownReason::InferenceConflict),
    );
}
```

Adjust exact syntax only if current parser requires the `where` clause on the same line/form used elsewhere. Do not alter the semantic assertion.

## Step 8.4 — Fixed conflict survivor

Add a parallel callable:

```phalcom
constrainedFixed<T>(_ value: T) -> Int
where T <: Allowed
```

with `Bad.new()`.

Expected:

```text
knowledge = Established(Int)
status = Invalid
```

## Step 8.5 — GREEN existing suppression regression

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  invalid_receiver_premise_produces_real_suppression
```

It must remain PASS: a failed dependent generic result cannot fabricate a receiver type for downstream dispatch.

**Commit:**

```text
fix(semantic): restrict generic terminal result fallback
```

---

# 12. Task 9 — Preserve directed `Var <: Var` relations and make solving order-stable

**Files:**
- Modify: `phalcom-semantic/src/checker/inference.rs`
- Modify: `phalcom-semantic/tests/semantic/foundations/inference.rs`

**Produces:**

```rust
InferenceSubtypeEdge
directed bound propagation
constraint-order regression
```

## Step 9.1 — RED: subtype relation must not alias variables

Add:

```rust
#[test]
fn variable_subtype_constraint_is_not_strengthened_to_equivalence() {
    let mut store = TypeStore::new();
    let mut hier = MapTypeHierarchy::new();

    let int_decl = test_decl("Int");
    let number_decl = test_decl("Number");
    hier.insert(int_decl.clone(), number_decl.clone());

    let int_ty = store.nominal(int_decl);
    let number_ty = store.nominal(number_decl);

    let mut session = InferenceSession::new();
    let t = session.fresh_variable(KindId::TYPE);
    let u = session.fresh_variable(KindId::TYPE);

    // Insert the directed relation first, while both vars are unresolved.
    session.add_constraint(
        InferenceRelation::Subtype(
            InferenceTerm::Var(t),
            InferenceTerm::Var(u),
        ),
        ConstraintOrigin::Explicit,
        None,
    );

    session.add_constraint(
        InferenceRelation::Equivalent(
            InferenceTerm::Var(t),
            InferenceTerm::Canonical(int_ty),
        ),
        ConstraintOrigin::Explicit,
        None,
    );

    session.add_constraint(
        InferenceRelation::Equivalent(
            InferenceTerm::Var(u),
            InferenceTerm::Canonical(number_ty),
        ),
        ConstraintOrigin::Explicit,
        None,
    );

    let outcome = session.solve(&mut store, &hier);
    assert!(
        matches!(outcome, InferenceOutcome::Solved(_)),
        "Int <: Number satisfies T <: U without requiring T == U: {outcome:#?}",
    );
}
```

Current solver is expected to fail/order-depend because the first constraint aliases T/U.

## Step 9.2 — RED permutation test

Build the same semantic constraint set in two insertion orders:

```text
order A: T <: U, T == Int, U == Number
order B: T == Int, U == Number, T <: U
```

Assert both solve and materialize:

```text
T = Int
U = Number
```

## Step 9.3 — Add directed edge storage

In `InferenceSession` add:

```rust
subtype_edges: Vec<InferenceSubtypeEdge>,
```

Deduplicate by representative pair.

## Step 9.4 — Change unresolved `Var <: Var`

In `subtype_terms(...)`:

```rust
(InferenceTerm::Var(sub), InferenceTerm::Var(sup))
```

when neither side has a solved substitution:

1. find representatives;
2. if same representative, return unchanged;
3. record `sub -> sup`;
4. return Changed only on new edge;
5. do not call `unify_terms`.

## Step 9.5 — Propagate bounds along edges

During each solve pass, for `sub -> sup`:

```text
lower(sub) -> lower(sup)
upper(sup) -> upper(sub)
```

If an endpoint is solved, validate/propgate the concrete endpoint using the real subtype relation.

Keep propagation monotone and deduplicated.

## Step 9.6 — Propagate support/proof only along used dependency

When bound propagation from one variable materially constrains another, join the source variable's support/proof state into the target.

Do not seed proof from the subtype edge itself.

The `where T <: U` relation is a contract, not value evidence.

## Step 9.7 — GREEN

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  variable_subtype_constraint_is_not_strengthened_to_equivalence

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::inference
```

Expected: PASS.

**Commit:**

```text
fix(semantic): preserve directed inference subtype relations
```

---

# 13. Task 10 — Wire generic solving to shared cancellation and query budget

**Files:**
- Modify: `phalcom-semantic/src/checker/inference.rs`
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/checker/call.rs`
- Modify: `phalcom-semantic/tests/semantic/foundations/inference.rs`

**Consumes:**
- `CheckerControl`
- `QueryBudget`
- `CancellationToken`
- `BudgetReport`

## Step 10.1 — Add one controlled solver implementation

Preferred shape:

```rust
pub fn solve_with_control(
    &mut self,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    control: &CheckerControl,
) -> InferenceOutcome
```

Keep:

```rust
pub fn solve(...)
```

for solver unit tests only if it delegates to the same implementation using `CheckerControl::default()`.

Do not fork solver logic.

## Step 10.2 — Add context wrapper

In `CheckingContext` add:

```rust
pub(crate) fn solve_inference(
    &mut self,
    session: &mut InferenceSession,
) -> InferenceOutcome {
    session.solve_with_control(
        self.store,
        &self.hierarchy,
        &self.control,
    )
}
```

Resolve borrow syntax exactly against current field ownership; if the compiler rejects simultaneous borrows, make `solve_with_control` take the store/hierarchy/control pieces through a small context helper rather than cloning any semantic state.

## Step 10.3 — Charge deterministic solver work

At minimum:

- check cancellation at entry to every fixed-point pass;
- charge one step before each stored constraint application;
- charge one step for each subtype-edge propagation;
- charge one step for each final variable-bound reconciliation.

If a charge fails:

```rust
return InferenceOutcome::BudgetExceeded(report);
```

If cancelled:

```rust
return InferenceOutcome::Cancelled;
```

Do not map either to Blocked.

## Step 10.4 — RED cancellation test

Add:

```rust
#[test]
fn inference_solver_observes_cancellation() {
    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();

    let token = CancellationToken::new();
    token.cancel();

    let control = CheckerControl::new(
        QueryBudget::default(),
        &token,
    );

    let mut session = InferenceSession::new();
    session.fresh_variable(KindId::TYPE);

    let outcome =
        session.solve_with_control(&mut store, &hier, &control);

    assert!(matches!(outcome, InferenceOutcome::Cancelled));
}
```

Use the real `CancellationToken` cancellation method name from `types/outcome.rs`.

## Step 10.5 — RED budget test

Construct a deliberately tiny `QueryBudget` using the existing constructor/field API.

Add more constraints than the budget allows.

Assert:

```rust
matches!(
    outcome,
    InferenceOutcome::BudgetExceeded(_)
)
```

Do not inspect a fake/sentinel report.

## Step 10.6 — Production call path

Replace both generic solves in `call.rs` with:

```rust
ctx.solve_inference(&mut session)
```

so first and expected-context solve share the same query budget/cancellation state as all other body relations.

## Step 10.7 — GREEN

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  inference_solver_observes_cancellation

cargo test -p phalcom-semantic --test semantic \
  inference_solver_observes_budget_exhaustion
```

Then all inference tests.

**Commit:**

```text
feat(semantic): bound generic inference by checker control
```

---

# 14. Task 11 — Route inference invariant failures to analyzer incidents

**Files:**
- Modify: `phalcom-semantic/src/checker/inference.rs`
- Modify: `phalcom-semantic/src/checker/call.rs`
- Modify: `phalcom-semantic/tests/semantic/foundations/inference.rs`
- Modify: `phalcom-semantic/tests/semantic/foundations/generic_inference_proof_integrity.rs`

## Step 11.1 — Add explicit internal outcome

Extend:

```rust
pub enum InferenceOutcome {
    Solved(InferenceSolution),
    Underconstrained(Vec<InferVarId>),
    Conflicting(InferenceConflict),
    Blocked(BlockReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(InferenceFailureReason),
}
```

If a dedicated `InferenceInternalFailure` type makes the compiler clearer, use it, but do not use free-form string-only failure for ordinary solver invariants.

## Step 11.2 — Classify missing variable metadata as internal

When solver machinery encounters:

```rust
InferenceFailureReason::MissingVariableMetadata { .. }
```

return:

```text
InternalFailure
```

not `Conflicting`.

User source did not create an impossible solver ID.

## Step 11.3 — Post-solved materialization failure

If:

```text
outcome == Solved
```

but a complete return term cannot be materialized, call:

```rust
ctx.record_internal_incident(
    InternalSemanticIncidentKind::InferenceInvariantViolation,
    InternalSemanticIncidentDetails::Message { ... },
    Some(call_range),
)
```

and record:

```text
AnalysisStatus::InternalFailure(id)
```

Return dependent knowledge as:

```text
Unknown(InferenceBlocked)
```

unless an independent fixed return exists.

## Step 11.4 — Test the solver-level classification

Where possible, construct a malformed solver reference only inside a `#[cfg(test)]` helper or a focused unit test with private module access.

Do not add public production APIs solely to fabricate invalid IDs for tests.

If integration tests cannot construct malformed metadata without unsafe/public API expansion, add a `#[cfg(test)]` unit test inside `checker/inference.rs`.

## Step 11.5 — GREEN

Run:

```bash
cargo test -p phalcom-semantic --lib \
  inference

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::inference
```

**Commit:**

```text
fix(semantic): classify inference invariant failures internally
```

---

# 15. Task 12 — Improve generic conflict provenance and diagnostic targeting

**Files:**
- Modify: `phalcom-semantic/src/checker/call.rs`
- Modify: `phalcom-semantic/src/checker/context.rs` only if a read-only expression-range accessor is needed
- Modify: `phalcom-semantic/tests/semantic/foundations/generic_inference_proof_integrity.rs`
- Modify: `phalcom-semantic/tests/semantic/foundations/semantic_correctness_regressions.rs`

## Step 12.1 — Keep real `InferenceConflict`

Do not replace:

```rust
InferenceConflict {
    constraint_index,
    origin,
    failure,
}
```

with a generic boolean or string.

## Step 12.2 — Resolve argument origin to source range

For:

```rust
ConstraintOrigin::Argument {
    argument,
    parameter_index,
    ..
}
```

prefer the recorded `ExpressionAnalysis.range` for that `argument`.

If `CheckingContext` lacks a read-only accessor, add:

```rust
pub(crate) fn expression_analysis(
    &self,
    id: ExpressionId,
) -> Option<&ExpressionAnalysis>
```

Do not expose the entire expression map publicly.

## Step 12.3 — Generic-where origin

For:

```rust
ConstraintOrigin::GenericWhere {
    callable,
    constraint_index,
}
```

keep those identities in the explanation.

If the generic constraint lacks canonical source-site attachment, use the call range as primary diagnostic range.

Do not synthesize a declaration range from selector/name heuristics; that is later identity work.

## Step 12.4 — Expected-result origin

If conflict occurs only after expected-result context was added:

- preserve any complete pre-context result;
- mark the contextual operation invalid according to existing call/checking policy;
- do not overwrite result knowledge with expected type;
- do not claim the expected type supplied value support.

## Step 12.5 — Regression

Strengthen existing:

```text
generic_conflict_reports_actual_failed_upper_bound
```

with origin checks where real origin is available.

Add a call-level argument-origin test verifying that the diagnostic primary range corresponds to the failing argument, not an unrelated first argument.

Use existing `Fixture` diagnostic range APIs; do not add test-only semantic behavior.

**Commit:**

```text
fix(semantic): preserve generic conflict provenance
```

---

# 16. Task 13 — Delete duplicate/legacy generic application shortcuts and audit structure

**Files:**
- Modify: `phalcom-semantic/src/checker/call.rs`
- Modify: any test file needed to keep public behavior covered

## Step 13.1 — Remove legacy non-generic branch inside generic helper

`apply_generic_callable_inner(...)` currently contains an old section labelled approximately:

```text
2. Non-generic Callable Resolution
```

after generic handling.

Technical 02 already owns non-generic application.

Delete that branch once all generic callers are proven to enter only with a non-empty generic signature.

If a compatibility caller still depends on it, route that caller through:

```rust
apply_resolved_callable(...)
```

instead of retaining a second implementation.

## Step 13.2 — Remove obsolete helpers

After compilation/test proof, remove helpers that exist solely for:

```text
manual generic parameter matching
old generic return fallback
duplicate non-generic resolution
```

Do not delete `InferenceSupport`; it remains the known-evidence subset used by the solver.

## Step 13.3 — Structural scans

Run:

```bash
rg -n \
  'positional_idx|position\(\|p\|.*external_label' \
  phalcom-semantic/src/checker/call.rs
```

Expected: no generic production argument binder.

Run:

```bash
rg -n \
  'knowledge\.ty\(\)' \
  phalcom-semantic/src/checker/call.rs
```

Review every match. Any generic argument constraint insertion must be preceded by full required-premise recording.

Run:

```bash
rg -n \
  'ExpectedResult' \
  phalcom-semantic/src/checker/inference.rs \
  phalcom-semantic/src/checker/call.rs
```

Verify there is no support/proof seeding from expected context.

Run:

```bash
rg -n \
  'Subtype|unify_terms' \
  phalcom-semantic/src/checker/inference.rs
```

Verify unresolved `Var <: Var` does not alias.

Run:

```bash
rg -n \
  'terminal_generic_return' \
  phalcom-semantic/src/checker/call.rs
```

Review each fallback source.

## Step 13.4 — GREEN focused suites

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::generic_inference_proof_integrity

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::inference

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application

cargo test -p phalcom-semantic --test semantic \
  semantic::capabilities::generics
```

Expected: PASS.

**Commit:**

```text
refactor(semantic): remove legacy generic inference shortcuts
```

---

# 17. Task 14 — Full regression matrix and closure gate

**Files:**
- Modify: `phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md` if this repository ledger is actively maintained for landed semantic coverage
- Create: `docs/work/logs/YYYY-MM-DD-technical-03-slice-stable.md` only after fresh verification
- No production changes unless a failing regression reveals a Technical 03 correctness defect

## Step 14.1 — Focused solver suite

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::inference
```

Expected: all PASS.

## Step 14.2 — Focused generic proof suite

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::generic_inference_proof_integrity
```

Expected: all PASS.

## Step 14.3 — Generic capability suite

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::capabilities::generics
```

Expected: all PASS.

## Step 14.4 — Technical 02 non-regression

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application
```

Expected: all PASS.

Generic repair must not reopen call-path divergence.

## Step 14.5 — Semantic correctness regressions

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::semantic_correctness_regressions
```

Expected: all PASS, subject only to separately documented tests intentionally owned by later slices.

## Step 14.6 — Bidirectional calls

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::bidirectional_calls
```

Expected: PASS.

## Step 14.7 — Full semantic integration binary

Run:

```bash
cargo test -p phalcom-semantic --test semantic
```

Expected: no new Technical 03 failures.

If the previously documented unrelated imported-identity failure remains, record it exactly; do not change identity code in this slice merely to make the suite green.

## Step 14.8 — Semantic library tests

Run:

```bash
cargo test -p phalcom-semantic --lib
```

Expected: PASS.

## Step 14.9 — Workspace build/check where environment permits

Run:

```bash
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```

If the repository-pinned nightly/hosted-runner component problem or `target-cpu=native` SIGILL remains, record it as infrastructure. Do not claim a clean workspace gate without a command that actually exits 0.

## Step 14.10 — Diff integrity

Run:

```bash
git diff --check
git status --short
```

Review only Technical 03 changes.

Do not stage or overwrite unrelated user work.

## Step 14.11 — Acceptance checklist

Verify explicitly:

- [ ] Unknown required generic premises are recorded before type filtering.
- [ ] Dynamic required generic premises are recorded before type filtering.
- [ ] Exact Unknown/Dynamic reasons survive return proof.
- [ ] Generic path consumes `ArgumentBindingPlan`.
- [ ] Every argument is analyzed once in source order.
- [ ] Expected result never seeds value support/proof.
- [ ] Value/declaration solve precedes expected selection.
- [ ] Return proof uses return-influencing variables only.
- [ ] Assumed support never upgrades.
- [ ] Fixed return survives unrelated generic terminal states.
- [ ] Dependent terminal failure never publishes a partial specialization.
- [ ] `Var <: Var` stays directed.
- [ ] Constraint insertion order does not change subtype semantics.
- [ ] Production solver observes cancellation.
- [ ] Production solver observes query budget.
- [ ] Missing solver metadata is an inference invariant incident.
- [ ] Generic conflicts retain real origin/failure data.
- [ ] Technical 02 canonical call suite remains green.
- [ ] Structural scans show no old generic-only binder/fast path.

## Step 14.12 — Work log

Only after fresh verification, create a work log containing:

```text
baseline SHA
commits landed
focused test counts
full semantic result
workspace/build result
known unrelated failures
structural scan result
Technical 04 handoff notes
```

Do not write “stable” or “complete” if required commands were not run successfully.

**Final commit:**

```text
docs(semantic): record Technical 03 verification status
```

---

# 18. Expected commit sequence

The implementation should normally land as these independently reviewable commits:

```text
1. test(semantic): add generic proof integrity regressions
2. feat(semantic): add generic inference proof-state domain
3. feat(semantic): track required generic proof premises
4. refactor(semantic): use canonical binding plan for generic calls
5. fix(semantic): preserve unavailable generic argument premises
6. refactor(semantic): separate generic value proof from contextual selection
7. fix(semantic): publish generic returns from complete proof state
8. fix(semantic): restrict generic terminal result fallback
9. fix(semantic): preserve directed inference subtype relations
10. feat(semantic): bound generic inference by checker control
11. fix(semantic): classify inference invariant failures internally
12. fix(semantic): preserve generic conflict provenance
13. refactor(semantic): remove legacy generic inference shortcuts
14. docs(semantic): record Technical 03 verification status
```

Do not squash these during implementation unless repository policy explicitly requires it; the sequence is designed to make correctness review and regression bisection practical.

---

# 19. Implementation notes for the agent

## Do not confuse proof state with solver state

These are all valid distinct combinations:

```text
Solved + Unknown(UnresolvedName)
Solved + Dynamic(ExplicitEscape)
Solved + Assumed
Conflicting + fixed Established return
Cancelled + fixed Established return
```

If code shape makes these impossible to express, the abstraction is too weak.

## Do not solve Unknown by assigning its expected type

For:

```phalcom
identity<T>(missing) -> T
```

under expected `Int`, the analyzer may select `T = Int` for contextual exploration, but the result remains Unknown with the original required-premise reason.

## Do not weaken fixed returns because generic arguments are assumed

For:

```phalcom
fixed<T>(value: T) -> Int
```

an assumed `value` affects validity/support of `T`, not the independent fixed return proposition.

The Technical 02 receiver/callee premise can still cap the fixed result if the callable target itself is only assumed.

## Do not use causal cleanliness as proof strength

A result may be:

```text
Established(Int)
+
non-clean causal invalidity
```

That is legal.

## Do not repair Spec04 here

If class-side receiver specialization reveals:

```text
Box<Number>.convert<U>(...) where U <: T
```

with enclosing `T` unspecialized inside the method constraint, fail closed or preserve the current honest outcome. Do not redesign substitution in this plan.

---

# 20. Definition of done

Technical 03 is done when the repository demonstrates all of the following with fresh evidence:

```text
1. substitution solvability and result proof are separate products;
2. every required generic argument premise is represented;
3. Unknown and Dynamic premises never disappear because they lack TypeId;
4. exact Unknown/Dynamic reasons survive generic return dependency;
5. Established/Assumed support is monotone;
6. expected context selects but never proves;
7. generic calls use the Technical 02 argument-binding plan;
8. invalid generic shape cannot yield partial specialization;
9. directed subtype constraints are never silently strengthened to equality;
10. solver meaning is stable under constraint insertion order;
11. fixed generic returns preserve independent knowledge through terminal states;
12. dependent terminal failures publish Unknown/Dynamic rather than partial types;
13. cancellation and budget outcomes are reachable in production solving;
14. inference invariant failures use the internal-incident domain;
15. conflict evidence remains attached to the actual origin/failure;
16. the canonical call suite remains green;
17. focused and broad semantic regressions show no Technical 03 correctness regressions.
```

Only after this gate should Technical Specification 04 assume method-local generic inference is proof-safe and focus on receiver/class substitution and specialization integrity.

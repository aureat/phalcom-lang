# Part 01 — Evidence Authority and Callable Contract Certification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make interprocedural type proof sound by preventing accidental evidence strengthening, separating dispatch certainty from callable-result authority, certifying source return contracts from body evidence, and preventing invalid recovery facts from becoming canonical callable results.

**Architecture:** Keep Phalcom's existing declaration/evidence split. `DeclaredTypeFact` remains the declaration-owned contract, `TypeKnowledge` remains value knowledge, and `CallableSemanticSignature` remains the canonical callable publication. Add only the missing proof boundary: structured normal-return facts plus a small `ReturnContractValidation` state on the canonical signature. Body analysis computes evidence; the session publishes validated/inferred results; dispatch only projects those results; call analysis may preserve or weaken authority but never strengthen it.

**Tech Stack:** Rust, `phalcom-semantic`, `DeclaredTypeFact`, `TypeKnowledge`, `EvidenceStatus`, `AnalysisStatus`, `CausalInvalidity`, `CallableSemanticSignature`, `CallableAnalysis`, `SurfaceDispatchResolver`, `SemanticWorkspaceSession`, deterministic semantic fingerprints, semantic integration fixtures.

**Spec:** This plan implements Part 1 of the ratified six-part typing-correctness architecture described in the project requirements analysis. It builds on the existing semantic-correctness work under `docs/impl/semantic/semantic-correctness/part-4/`, especially `2026-08-27-semantic-capability-gap-closure-implementation-plan.md`. The implementation source of truth for this plan is repository `main` at the grounding revision below.

## Repository grounding

This plan was freshly grounded against `aureat/phalcom-lang` `main` at:

```text
24fc9fd98f3c3c534c4d52b613962a39b9374185
feat(semantic): add rich type diagnostics tests and polish presentation
```

Important current anchors at that revision:

- `phalcom-semantic/src/declaration_type.rs`
  - `DeclaredTypeBasis::SourceAnnotation` projects to `TypeKnowledge::assumed(...)`.
  - native, declaration, and constructor semantics project to established knowledge.
- `phalcom-semantic/src/signature.rs`
  - `CallableSemanticSignature` already separates `declared_return` from `inferred_return`.
  - `published_return_knowledge()` currently selects `inferred_return` when available, otherwise the declaration projection.
- `phalcom-semantic/src/checker/declaration_signature.rs`
  - source annotations become `DeclaredTypeBasis::SourceAnnotation`.
  - constructors publish compiler-owned `Self` with `ConstructorSemantics`.
  - setters publish compiler-owned `Unit` with `DeclarationSemantics`.
- `phalcom-semantic/src/checker/call.rs`
  - `CallTargetAuthority::ExactDispatch` currently has established base authority.
  - `derive_fixed_return()` reconstructs a known signature return as established before capping it, which can launder an assumed source return into established knowledge.
- `phalcom-semantic/src/checker/analysis.rs`
  - `BodyExitFacts` currently splits exit flow (`returns`) from `normal_return_values`.
  - `normal_return_summary()` receives only `TypeKnowledge`, so expression status and causal invalidity are lost before publication.
- `phalcom-semantic/src/checker/body.rs`
  - body analysis collects `Vec<TypeKnowledge>` for normal returns.
  - the tail path can push `typed.knowledge` even when the typed expression is invalid/suppressed.
- `phalcom-semantic/src/checker/statement.rs`
  - direct `return` returns only `Option<TypeKnowledge>` to body analysis.
- `phalcom-semantic/src/checker/context.rs`
  - `finalize_with_normal_returns()` creates `BodyExitFacts.returns` from `entry_flow`, not the actual flow at each normal return.
- `phalcom-semantic/src/session.rs`
  - `refresh_inferred_callable_results()` publishes body summaries only for signatures whose current published result is unknown.
  - this means annotated source contracts are never body-certified as a distinct semantic operation.
- `phalcom-semantic/src/types/evidence.rs`
  - evidence joins already preserve the weakest contributing status.
  - several local pieces of code reimplement “minimum evidence status”; this should be canonicalized.
- `phalcom-semantic/src/db/fingerprint.rs`
  - callable analyses and exit facts are fingerprinted explicitly and must be updated when their semantic shape changes.
- `phalcom-semantic/tests/semantic/capabilities/authority.rs`
  - already contains useful authority and constructor proof tests.
- `phalcom-semantic/tests/semantic/capabilities/callable_publication.rs`
  - already covers inferred return propagation and recursive non-fabrication.
- `phalcom-semantic/tests/semantic/capabilities/callable_publication_trusted.rs`
  - already protects canonical native established returns.

All symbol names in this plan are exact implementation anchors at the grounding revision. Re-resolve line numbers after rebasing; do not mechanically apply stale line offsets.

## Problem statement

The current semantic model contains the right concepts but does not enforce their transitions consistently. Four propositions must remain distinct:

```text
Declaration contract:
    callable C declares return T

Value evidence:
    expression E is Established<T> / Assumed<T> / Unknown / Dynamic

Relation proof:
    given the supplied premises, A <: B is Proven

Contract certification:
    callable C's implementation has enough formal evidence to uphold its declared return contract
```

The current exact-call path conflates dispatch certainty with contract certification. Separately, callable publication drops `AnalysisStatus` and `CausalInvalidity` before body return summaries are published. Both allow facts to become stronger than their supporting evidence.

## Required semantic laws

The implementation MUST satisfy these laws after every task in this plan:

### Law E1 — Ordinary derivations never strengthen evidence

```text
result authority <= weakest required premise authority
```

unless the derivation itself is an independent authoritative observation or compiler-owned semantic fact.

Examples:

```text
Established<Int> projected structurally       -> Established<...>
Assumed<List<Int>> projected to element       -> Assumed<Int>
Established<Int> + Assumed<Int> composition   -> Assumed<...>
Exact dispatch to Assumed<String> return      -> Assumed<String>
```

### Law E2 — Relation proof is not premise proof

`RelationOutcome::Proven` proves a relation given its premises. It MUST NOT by itself upgrade `Assumed<T>` to `Established<T>`.

### Law E3 — Dispatch certainty is not return-contract authority

Resolving the exact `CallableId` establishes which callable is selected. The result authority comes from the callable's published return knowledge, not from exact dispatch itself.

### Law E4 — Source declarations begin as assumptions

A source return annotation is a contract. Before body certification it remains `Assumed`.

### Law E5 — Validated source contracts may become established at the public boundary

For a source declaration `f() -> T`, if every reachable normal return is clean, the body analysis is complete, every normal result is proven assignable to `T`, and the weakest supporting body evidence is established, the callable may publish `Established<T>` as a validated callable contract.

The public type remains the declared `T`, even when the body proves a narrower subtype.

### Law E6 — Assumed implementation evidence cannot certify an established contract

If the body satisfies the relation only from assumed premises, publication remains `Assumed<T>`.

### Law E7 — Invalid recovery knowledge is quarantined

An invalid/suppressed expression may retain useful `TypeKnowledge` for editor recovery, but that knowledge MUST NOT establish an inferred callable return or an established source contract.

### Law E8 — Trusted compiler/native contracts retain authority

Constructor semantics, declaration semantics, and trusted native signatures remain established according to their existing declaration basis. This plan MUST NOT weaken `Cell.new()`, `System.print`, setter `Unit`, or other compiler-owned results merely because source contract certification is introduced.

### Law E9 — No normal exit means `Never`

A complete callable body with no normal return path publishes `Never` for unannotated inference and vacuously satisfies a source return contract, provided the absence of normal exits is not the result of blocked/cancelled/internal analysis.

### Law E10 — Publication is deterministic and incremental-safe

Changing source traversal order or re-running a stable revision MUST NOT change return authority. Contract-validation state and structured return facts must participate in semantic fingerprints so dependent queries refresh only when semantic meaning changes.

## Explicit non-goals

Do NOT broaden Part 1 into any of the following:

- branch reachability redesign;
- `if let` repair;
- loop fixed-point analysis;
- field lifecycle repair beyond exposing actual normal-exit flow snapshots needed by Part 2;
- comparison or membership semantics;
- builtin identity cleanup;
- general theorem-proof objects such as `Proof<T>` threaded through all semantic APIs;
- a second callable signature table;
- CFG-driven body checking;
- advisory inference changes.

Part 1 is complete when callable evidence publication is sound, even if later control-flow plans improve which normal exits are discovered.

---

# Target data model

## 1. Canonical evidence meet

Add one canonical operation to `EvidenceStatus`:

```rust
impl EvidenceStatus {
    pub const fn meet(self, other: Self) -> Self {
        match (self, other) {
            (Self::Established, Self::Established) => Self::Established,
            _ => Self::Assumed,
        }
    }
}
```

For more than two premises, fold from `Established`:

```rust
let support = statuses.into_iter().fold(EvidenceStatus::Established, EvidenceStatus::meet);
```

Do not define an ordering-dependent `min()` policy outside the evidence module.

## 2. Structured normal-return fact

Replace the split `returns + normal_return_values` representation with one fact per normal exit:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalReturnFact {
    pub knowledge: TypeKnowledge,
    pub flow: FlowStateSummary,
    pub status: AnalysisStatus,
    pub causal_invalidity: CausalInvalidity,
}
```

Then:

```rust
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BodyExitFacts {
    pub normal_returns: Vec<NormalReturnFact>,
    pub throws: Vec<FlowStateSummary>,
    pub unreachable: bool,
}
```

Do not retain a second authoritative `normal_return_values` collection. Tests and session publication must project `normal_returns.iter().map(|exit| &exit.knowledge)` only where a view is needed.

`NormalReturnFact.flow` is the flow at the actual normal exit point. It is not callable entry flow and not the final mutable checker flow after later syntactic statements.

## 3. Public-return contract validation

Add to `signature.rs`:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ReturnContractValidation {
    /// No source-authored return contract requires body certification.
    NotApplicable,
    /// Source contract exists but no complete body proof has been published yet.
    Unchecked,
    /// Every normal return satisfies the contract. The payload is the weakest
    /// implementation evidence supporting the contract.
    Satisfied(EvidenceStatus),
    /// At least one normal return is proven incompatible with the contract.
    Refuted,
    /// Analysis could not decide the contract soundly.
    Blocked,
    /// Validation crosses an explicit Dynamic boundary.
    DynamicBoundary,
}
```

Extend `CallableSemanticSignature`:

```rust
pub return_validation: ReturnContractValidation,
```

Initialization rule:

```text
Source implementation + DeclaredTypeBasis::SourceAnnotation -> Unchecked
all other declaration bases                          -> NotApplicable
```

`ReturnContractValidation` is semantic publication state, not diagnostic state. Diagnostics remain attached to `CallableAnalysis`.

## 4. Callable analysis validation result

Add to `CallableAnalysis`:

```rust
pub return_validation: ReturnContractValidation,
```

The body checker computes this result from its actual normal returns and declared source contract. The session copies it into the canonical signature during publication/fixed-point refresh.

## 5. Public return projection

`CallableSemanticSignature::published_return_knowledge()` becomes the single public authority rule:

```text
Known source annotation:
  Satisfied(Established) -> Established<declared T>, origin CallableSignature
  otherwise              -> declared projection (Assumed<T>)

Known native/declaration/constructor semantic return:
  -> existing established declaration projection

Dynamic declaration:
  -> Dynamic

Unknown declaration + inferred_return:
  -> inferred_return

Unknown declaration + no inference:
  -> original Unknown
```

For annotated callables, `inferred_return` MUST NOT replace the declared public type. The body may prove `Dog`; if the declaration says `Animal`, callers see validated `Animal`.

## 6. Return-summary admissibility

Add a small helper on `NormalReturnFact` or in `analysis.rs`:

```rust
pub fn publication_knowledge(&self) -> TypeKnowledge
```

Required behavior:

```text
status Ready + causal Clean                 -> original knowledge
Invalid or Suppressed                       -> Unknown(SuppressedByInvalidCause)
Blocked                                     -> Unknown(InferenceBlocked)
Cancelled                                   -> Unknown(InferenceCancelled)
BudgetExceeded                              -> Unknown(InferenceBudgetExceeded)
InternalFailure                             -> Unknown(InferenceBlocked)
DynamicBoundary                             -> preserve Dynamic if present; otherwise Unknown(InferenceBlocked)
non-clean causal invalidity with Ready      -> Unknown(SuppressedByInvalidCause)
```

`normal_return_summary()` MUST summarize `publication_knowledge()`, not raw recovery knowledge.

---

# File/ownership map

| Area | Current owner | Part 1 responsibility |
|---|---|---|
| Evidence authority | `types/evidence.rs` | Canonical `EvidenceStatus::meet`; eliminate local strengthening helpers |
| Declaration basis | `declaration_type.rs` | Preserve current source-vs-trusted projection semantics |
| Canonical callable signature | `signature.rs` | Add `ReturnContractValidation`; own public return projection |
| Declaration signature construction | `checker/declaration_signature.rs` | Initialize validation state correctly for source/trusted callables |
| Normal return product | `checker/analysis.rs` | Add `NormalReturnFact`; replace split exit representation; compute admissible summary |
| Return statement transfer | `checker/statement.rs` | Preserve typed status/causal state and actual exit flow in return fact |
| Tail/implicit returns | `checker/body.rs` | Create structured return facts instead of bare knowledge |
| Checker finalization | `checker/context.rs` | Aggregate contract validation and publish actual normal-exit facts |
| Call application | `checker/call.rs` | Preserve signature return authority; exact dispatch does not promote it |
| Canonical publication/fixed point | `session.rs` | Publish validation + inference and recheck dependents when public authority changes |
| Incremental identity | `db/fingerprint.rs` | Hash validation and structured exits deterministically |
| Tests | `tests/semantic/capabilities/*`, `tests/semantic/foundations/*` | Lock authority laws at helper, checker, and composed-call levels |

---

# Execution order

```text
Task 0  Freeze baseline and add RED soundness probes
   |
Task 1  Canonicalize EvidenceStatus meet/weaken rules
   |
Task 2  Introduce structured NormalReturnFact and migrate body collection
   |
Task 3  Add ReturnContractValidation to signatures and analyses
   |
Task 4  Compute source-contract validation from complete body exits
   |
Task 5  Refactor session publication/fixed-point refresh
   |
Task 6  Remove exact-dispatch return promotion in call.rs
   |
Task 7  Update fingerprints, explanations, and public test helpers
   |
Task 8  Complete authority/publication regression matrix
   |
Task 9  Full semantic closure gate and cleanup audit
```

Tasks are deliberately sequential. Do not fix `call.rs` first and leave the repository in a state where all source returns are permanently assumed; canonical publication must exist before the exact-dispatch shortcut is removed.

---

## Task 0: Freeze the baseline and install RED soundness probes

**Files:**
- Modify: `phalcom-semantic/tests/semantic/capabilities/authority.rs`
- Modify: `phalcom-semantic/tests/semantic/capabilities/callable_publication.rs`
- Modify: `phalcom-semantic/tests/semantic/capabilities/callable_publication_trusted.rs`
- Read: `phalcom-semantic/tests/semantic/support/*`
- Read: `phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md`

**Interfaces:**
- Consumes: existing `Fixture`, `CallableAnalysis`, `ExpressionAnalysis`, `TypeKnowledge`, `EvidenceStatus`, and snapshot helpers.
- Produces: failing tests that encode the proof-authority bugs before implementation.

- [ ] **Step 1: Confirm a clean revision and record the implementation base.**

Run:

```sh
git status --short
git rev-parse HEAD
git log -1 --oneline
```

Expected: the branch starts from the intended semantic revision with no unrelated edits. Record the SHA in the work log if it differs from the grounding SHA in this plan.

- [ ] **Step 2: Add a RED test proving exact dispatch cannot establish an assumed source result.**

Add a source method whose body depends only on an assumed parameter contract:

```rust
#[test]
fn exact_dispatch_does_not_upgrade_assumed_source_return() {
    let f = Fixture::new(
        r#"
class Echo {
  @class
  echo(_ value: String) -> String {
    value
  }
}

class Probe {
  @class
  run() {
    let result = Echo.echo("hello")
  }
}
"#,
    );

    let run = f.callable("Probe", "run", DispatchSide::Class);
    let result = f.binding(run, "result");
    assert_eq!(result.current.status(), Some(EvidenceStatus::Assumed));
}
```

Why this case is authoritative: the source annotation says `String`, but the body uses the callable parameter whose formal knowledge is itself an assumption. Exact selector resolution cannot create stronger return evidence.

- [ ] **Step 3: Run the new test and verify the current bug.**

Run:

```sh
cargo test -p phalcom-semantic --test semantic exact_dispatch_does_not_upgrade_assumed_source_return -- --nocapture
```

Expected before the fix: FAIL because the call result is currently promoted to established by exact dispatch.

- [ ] **Step 4: Add a RED test proving a fully established body can certify the declared public supertype.**

Use a subtype body result with a broader declared contract:

```rust
#[test]
fn established_body_certifies_declared_public_return_without_narrowing_api() {
    let f = Fixture::new(
        r#"
class Animal {}
class Dog is Animal {
  @constructor
  new() {}
}
class Factory {
  @class
  make() -> Animal {
    Dog.new()
  }
}
class Probe {
  @class
  run() {
    let result = Factory.make()
  }
}
"#,
    );

    let animal = f.ty("Animal");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let result = f.binding(run, "result");
    assert_eq!(result.current.ty(), Some(animal));
    assert_eq!(result.current.status(), Some(EvidenceStatus::Established));
}
```

The test must assert the public type is `Animal`, not the narrower `Dog`.

- [ ] **Step 5: Add a RED test proving invalid recovery knowledge cannot establish an inferred return.**

Use a body where an invalid binding retains useful recovery knowledge and becomes the tail:

```rust
#[test]
fn invalid_tail_recovery_knowledge_is_not_published_as_inferred_return() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  broken() {
    let value: Int = "wrong"
    value
  }

  @class
  run() {
    let result = Probe.broken()
  }
}
"#,
    );

    let broken = f.callable("Probe", "broken", DispatchSide::Class);
    let analysis = f.callable_analysis(broken);
    assert!(analysis.exits.normal_returns.iter().all(|exit| !exit.publication_knowledge().is_established()));
}
```

The exact helper names in this test intentionally depend on Task 2. Before Task 2, install an equivalent assertion against the existing product (for example, assert the caller must not become established) and migrate it when `NormalReturnFact` lands.

- [ ] **Step 6: Add a source-contract refutation test.**

```rust
#[test]
fn incompatible_body_refutes_source_contract_without_establishing_it() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  broken() -> Int {
    "wrong"
  }
}
"#,
    );

    let broken = f.callable("Probe", "broken", DispatchSide::Class);
    let analysis = f.callable_analysis(broken);
    assert!(f.diagnostics(DiagnosticCode::ReturnMismatch).len() >= 1);
    assert_ne!(analysis.return_validation, ReturnContractValidation::Satisfied(EvidenceStatus::Established));
}
```

Migrate the assertion to the final enum after Task 3.

- [ ] **Step 7: Preserve trusted baselines explicitly.**

Ensure the existing constructor/native tests continue asserting:

```text
CellNum.new()       -> Established<CellNum>, ConstructorSemantics
System.print(...)   -> Established<Unit>, NativeSignature
System.gc           -> Established<Unit>, NativeSignature
```

Do not weaken or delete those assertions to make later changes pass.

- [ ] **Step 8: Commit only the baseline probes.**

Suggested commit:

```sh
git add phalcom-semantic/tests/semantic/capabilities
git commit -m "test(semantic): expose callable proof authority gaps"
```

---

## Task 1: Canonicalize evidence-strength composition

**Files:**
- Modify: `phalcom-semantic/src/types/evidence.rs`
- Modify: `phalcom-semantic/src/checker/call.rs`
- Modify as discovered by audit: any file containing local `minimum_evidence_status`, manual `Established`/`Assumed` pair logic, or status folding.
- Test: unit tests in `types/evidence.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/*` where appropriate.

**Interfaces:**
- Consumes: `EvidenceStatus::{Established, Assumed}`.
- Produces: `EvidenceStatus::meet(self, other) -> EvidenceStatus` and one canonical status-fold pattern used by all derived evidence.

- [ ] **Step 1: Add failing algebra tests for `EvidenceStatus::meet`.**

Add:

```rust
#[test]
fn evidence_status_meet_is_weakest_support() {
    use EvidenceStatus::{Assumed, Established};
    assert_eq!(Established.meet(Established), Established);
    assert_eq!(Established.meet(Assumed), Assumed);
    assert_eq!(Assumed.meet(Established), Assumed);
    assert_eq!(Assumed.meet(Assumed), Assumed);
}

#[test]
fn evidence_status_meet_is_commutative_and_idempotent() {
    use EvidenceStatus::{Assumed, Established};
    for left in [Established, Assumed] {
        for right in [Established, Assumed] {
            assert_eq!(left.meet(right), right.meet(left));
        }
        assert_eq!(left.meet(left), left);
    }
}
```

- [ ] **Step 2: Run the tests and verify they fail to compile before the API exists.**

Run:

```sh
cargo test -p phalcom-semantic evidence_status_meet --lib -- --nocapture
```

Expected: compile failure because `meet` is not defined.

- [ ] **Step 3: Implement `EvidenceStatus::meet`.**

Add exactly one canonical implementation in `types/evidence.rs`:

```rust
impl EvidenceStatus {
    pub const fn meet(self, other: Self) -> Self {
        match (self, other) {
            (Self::Established, Self::Established) => Self::Established,
            _ => Self::Assumed,
        }
    }

    pub const fn as_str(self) -> &'static str {
        // existing implementation unchanged
        match self {
            Self::Established => "established",
            Self::Assumed => "assumed",
        }
    }
}
```

- [ ] **Step 4: Replace local binary minimum helpers.**

In `checker/call.rs`, delete:

```rust
fn minimum_evidence_status(left: EvidenceStatus, right: EvidenceStatus) -> EvidenceStatus
```

and replace every use with:

```rust
left.meet(right)
```

Do not change call-result semantics yet; Task 6 will repair `derive_fixed_return()` after canonical publication exists.

- [ ] **Step 5: Audit the crate for duplicate strength logic.**

Run:

```sh
rg "minimum_evidence_status|all\(.*EvidenceStatus::Established|EvidenceStatus::Assumed" phalcom-semantic/src
```

For each match, classify it in the work log as one of:

```text
JOIN          — legitimately computing weakest support; use meet/fold
CONSTRUCTION  — authoritative new evidence; may construct Established directly
PRESENTATION  — only rendering/status labels; no semantic change
UNRELATED     — pattern does not combine evidence strength
```

Do not mechanically replace authoritative constructors such as literal syntax, constructor semantics, or trusted native signatures.

- [ ] **Step 6: Convert `join_type_knowledge()` and `compose_required_knowledge()` to the same fold without changing behavior.**

Use:

```rust
let status = known
    .iter()
    .map(TypeEvidence::status)
    .fold(EvidenceStatus::Established, EvidenceStatus::meet);
```

and the equivalent for required composition. Preserve existing Unknown/Dynamic behavior and provenance merging.

- [ ] **Step 7: Run evidence and semantic authority tests.**

Run:

```sh
cargo test -p phalcom-semantic --lib types::evidence -- --nocapture
cargo test -p phalcom-semantic --test semantic authority -- --nocapture
```

Expected: existing authority tests stay green; the Task 0 RED call tests may remain red until Tasks 3–6.

- [ ] **Step 8: Commit.**

```sh
git add phalcom-semantic/src/types/evidence.rs phalcom-semantic/src/checker/call.rs
git commit -m "refactor(semantic): centralize evidence authority meet"
```

---

## Task 2: Replace split return collections with structured normal-return facts

**Files:**
- Modify: `phalcom-semantic/src/checker/analysis.rs`
- Modify: `phalcom-semantic/src/checker/statement.rs`
- Modify: `phalcom-semantic/src/checker/body.rs`
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: tests referencing `exits.returns` or `exits.normal_return_values`
- Later in this task: `phalcom-semantic/src/db/fingerprint.rs` only enough to restore compilation; semantic hashing is finalized in Task 7.

**Interfaces:**
- Consumes: `TypedExpression`, `AnalysisStatus`, `CausalInvalidity`, `FlowStateSummary`, `TypeKnowledge`.
- Produces:

```rust
pub struct NormalReturnFact {
    pub knowledge: TypeKnowledge,
    pub flow: FlowStateSummary,
    pub status: AnalysisStatus,
    pub causal_invalidity: CausalInvalidity,
}

pub struct BodyExitFacts {
    pub normal_returns: Vec<NormalReturnFact>,
    pub throws: Vec<FlowStateSummary>,
    pub unreachable: bool,
}
```

- [ ] **Step 1: Add low-level tests for publication admissibility.**

In `checker/analysis.rs` tests, construct return facts directly and assert:

```rust
#[test]
fn invalid_normal_return_fact_suppresses_recovery_knowledge_for_publication() {
    let mut store = TypeStore::new();
    let int_ty = /* canonical test nominal */;
    let fact = NormalReturnFact {
        knowledge: TypeKnowledge::established(int_ty, EvidenceOrigin::Flow),
        flow: FlowStateSummary::default(),
        status: AnalysisStatus::Invalid(DiagnosticCauseId(1)),
        causal_invalidity: CausalInvalidity::One(DiagnosticCauseId(1)),
    };
    assert!(matches!(
        fact.publication_knowledge(),
        TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause)
    ));
}
```

Also add clean established, clean assumed, blocked, cancelled, budget-exceeded, and dynamic-boundary cases.

- [ ] **Step 2: Introduce `NormalReturnFact` and `publication_knowledge()`.**

Implement the target struct in `analysis.rs`. Keep the mapping explicit:

```rust
impl NormalReturnFact {
    pub fn publication_knowledge(&self) -> TypeKnowledge {
        if self.causal_invalidity != CausalInvalidity::Clean {
            return TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause);
        }
        match &self.status {
            AnalysisStatus::Ready => self.knowledge.clone(),
            AnalysisStatus::Invalid(_) | AnalysisStatus::Suppressed(_) => {
                TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause)
            }
            AnalysisStatus::Blocked(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBlocked),
            AnalysisStatus::DynamicBoundary(reason) => match &self.knowledge {
                TypeKnowledge::Dynamic(_) => self.knowledge.clone(),
                _ => TypeKnowledge::Dynamic(reason.clone()),
            },
            AnalysisStatus::Cancelled => TypeKnowledge::Unknown(UnknownReason::InferenceCancelled),
            AnalysisStatus::BudgetExceeded(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBudgetExceeded),
            AnalysisStatus::InternalFailure(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBlocked),
        }
    }
}
```

Do not erase the original `knowledge`; it remains available for diagnostics/editor recovery.

- [ ] **Step 3: Replace `BodyExitFacts` split collections.**

Change:

```rust
pub returns: Vec<FlowStateSummary>,
pub normal_return_values: Vec<TypeKnowledge>,
```

to:

```rust
pub normal_returns: Vec<NormalReturnFact>,
```

Keep `throws` and `unreachable` unchanged in Part 1.

- [ ] **Step 4: Change `normal_return_summary()` to consume structured exits.**

Use:

```rust
pub fn normal_return_summary(store: &mut TypeStore, exits: &[NormalReturnFact]) -> TypeKnowledge {
    if exits.is_empty() {
        return TypeKnowledge::established(store.never(), EvidenceOrigin::Flow);
    }
    join_type_knowledge(store, exits.iter().map(NormalReturnFact::publication_knowledge))
}
```

This function no longer accepts bare `TypeKnowledge`.

- [ ] **Step 5: Change `check_statement()` return type.**

Change:

```rust
pub fn check_statement(...) -> Option<TypeKnowledge>
```

to:

```rust
pub fn check_statement(...) -> Option<NormalReturnFact>
```

Only `Statement::Return` returns `Some(...)`.

For the return path, make `val_typed` mutable, apply the relation result to its status/causal invalidity, call `ctx.sync_expression_outcome(&val_typed)`, then capture the exit flow at that exact point:

```rust
Some(NormalReturnFact {
    knowledge: val_typed.knowledge.clone(),
    flow: ctx.current_flow_summary(),
    status: val_typed.status.clone(),
    causal_invalidity: val_typed.causal_invalidity,
})
```

If `RelationApplication.cause` exists, the returned fact must be invalid/causally tainted even if its recovery knowledge is known.

- [ ] **Step 6: Preserve relation application on direct returns.**

Do not merely emit `ReturnMismatch`. Update `val_typed` using the same semantics as other typed relation applications:

```text
cause present            -> invalidate with cause
Blocked                   -> AnalysisStatus::Blocked
DynamicBoundary           -> AnalysisStatus::DynamicBoundary
Cancelled                 -> AnalysisStatus::Cancelled
BudgetExceeded            -> AnalysisStatus::BudgetExceeded
InternalFailure           -> AnalysisStatus::InternalFailure
Proven                     -> preserve existing typed status
```

If an existing reusable helper can be moved from `expression.rs` to `typed_expr.rs` without creating a dependency cycle, do that. Otherwise implement the small mapping in `statement.rs` and leave a comment naming the shared semantic rule. Do not create a new general error framework.

- [ ] **Step 7: Migrate `body.rs` normal-return collection.**

Replace:

```rust
let mut normal_return_values = Vec::new();
```

with:

```rust
let mut normal_returns = Vec::new();
```

For an explicit return, push the `NormalReturnFact` returned by `check_statement()` unchanged.

For a tail expression, create:

```rust
normal_returns.push(NormalReturnFact {
    knowledge: typed.knowledge.clone(),
    flow: ctx.current_flow_summary(),
    status: typed.status.clone(),
    causal_invalidity: typed.causal_invalidity,
});
```

For implicit `Unit` completion, create a ready/clean `NormalReturnFact` with the current flow snapshot.

Do not change Part 3's `can_fall_through` architecture here; Part 1 only ensures any exit that is collected is represented honestly.

- [ ] **Step 8: Rewrite `CheckingContext::finalize_with_normal_returns`.**

Change the input to:

```rust
normal_returns: Vec<NormalReturnFact>
```

and build:

```rust
let exits = BodyExitFacts {
    normal_returns,
    throws: self.throw_exit_flows,
    unreachable: false,
};
```

Delete the current synthetic behavior that creates `returns: vec![entry_flow.clone()]` whenever any normal value exists. The actual flow now travels with each return fact.

- [ ] **Step 9: Migrate call sites and tests mechanically.**

Run:

```sh
rg "exits\.returns|exits\.normal_return_values|normal_return_summary\(" phalcom-semantic
```

Migrate every match. Typical conversions:

```rust
analysis.exits.normal_returns.len()
analysis.exits.normal_returns[0].knowledge.ty()
analysis.exits.normal_returns[0].flow.fields.get(...)
```

No compatibility alias should remain after the task. A second exit representation would reintroduce synchronization risk.

- [ ] **Step 10: Restore compilation in fingerprint code with a temporary exact hash of the new fields.**

In `db/fingerprint.rs`, hash each normal return's:

```text
knowledge
flow
status semantic shape
causal-invalidity shape
```

Task 7 will review the fingerprint semantics; this step simply keeps the repository coherent.

- [ ] **Step 11: Run focused tests.**

```sh
cargo test -p phalcom-semantic --lib checker::analysis -- --nocapture
cargo test -p phalcom-semantic --test semantic callable_publication -- --nocapture
cargo test -p phalcom-semantic --test semantic authority -- --nocapture
```

Expected: existing return behavior is preserved except that invalid return recovery facts are no longer admissible publication inputs.

- [ ] **Step 12: Commit.**

```sh
git add phalcom-semantic/src/checker phalcom-semantic/src/db/fingerprint.rs phalcom-semantic/tests
git commit -m "refactor(semantic): structure callable normal return facts"
```

---

## Task 3: Add explicit source return-contract validation state

**Files:**
- Modify: `phalcom-semantic/src/signature.rs`
- Modify: `phalcom-semantic/src/checker/analysis.rs`
- Modify: `phalcom-semantic/src/checker/declaration_signature.rs`
- Modify: any native/bootstrap construction site of `CallableSemanticSignature`
- Test: `phalcom-semantic/tests/semantic/capabilities/authority.rs`
- Test: unit tests for signature projection if present.

**Interfaces:**
- Consumes: `DeclaredTypeBasis`, `ImplementationKind`, `EvidenceStatus`.
- Produces: `ReturnContractValidation` and `CallableSemanticSignature::return_validation`.

- [ ] **Step 1: Add enum-level tests for initial validation state.**

Write tests covering:

```text
ordinary source annotated return -> Unchecked
source unannotated return         -> NotApplicable
constructor Self                  -> NotApplicable
setter Unit semantics             -> NotApplicable
native trusted return             -> NotApplicable
```

The test may exercise `semantic_signature_for_member()` and `canonical_core_class_new_signature()` rather than a private initializer helper.

- [ ] **Step 2: Define `ReturnContractValidation` in `signature.rs`.**

Use the exact target enum from this plan:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ReturnContractValidation {
    NotApplicable,
    Unchecked,
    Satisfied(EvidenceStatus),
    Refuted,
    Blocked,
    DynamicBoundary,
}
```

- [ ] **Step 3: Add `return_validation` to `CallableSemanticSignature`.**

Add:

```rust
pub return_validation: ReturnContractValidation,
```

beside `declared_return`/`inferred_return`, because all three describe the callable's return publication.

- [ ] **Step 4: Centralize initial validation selection.**

In `declaration_signature.rs`, add:

```rust
fn initial_return_validation(
    declared_return: &DeclaredTypeFact,
    implementation: ImplementationKind,
) -> ReturnContractValidation {
    if implementation == ImplementationKind::Source
        && declared_return.basis == DeclaredTypeBasis::SourceAnnotation
        && declared_return.is_known()
    {
        ReturnContractValidation::Unchecked
    } else {
        ReturnContractValidation::NotApplicable
    }
}
```

If `ImplementationKind` is not `PartialEq`, match it explicitly rather than changing that external type only for this helper.

- [ ] **Step 5: Initialize every signature constructor explicitly.**

Update:

- `semantic_signature_for_member()`;
- `canonical_core_class_new_signature()`;
- native/base signature builders found by:

```sh
rg "CallableSemanticSignature \{" phalcom-semantic/src
```

Never rely on a default for trusted/native signatures; make the authority source visible at construction.

- [ ] **Step 6: Add the same field to `CallableAnalysis`.**

Initialize it temporarily to `NotApplicable`/`Unchecked` according to the declared signature so compilation succeeds. Task 4 will compute the final body result.

- [ ] **Step 7: Change `published_return_knowledge()` to respect validation.**

Implement this ordering explicitly:

```rust
pub fn published_return_knowledge(&self) -> TypeKnowledge {
    if self.declared_return.is_known() {
        let declared = self.declared_return.to_knowledge();
        if self.declared_return.basis == DeclaredTypeBasis::SourceAnnotation
            && self.return_validation == ReturnContractValidation::Satisfied(EvidenceStatus::Established)
        {
            if let Some(ty) = declared.ty() {
                return TypeKnowledge::established(ty, EvidenceOrigin::CallableSignature);
            }
        }
        return declared;
    }

    if self.declared_return.is_dynamic() {
        return self.declared_return.to_knowledge();
    }

    self.inferred_return
        .as_ref()
        .filter(|knowledge| knowledge.is_known() || knowledge.is_dynamic())
        .cloned()
        .unwrap_or_else(|| self.declared_return.to_knowledge())
}
```

`Satisfied(Assumed)` deliberately falls through to the source annotation's assumed projection.

- [ ] **Step 8: Keep public term selection declaration-first.**

Review `published_return_term()`. For a known declared return, it must return the declared term even if an internal body summary is narrower. For an unannotated/unknown declaration, it may use `inferred_return`.

Target ordering:

```text
known declared term -> declared term
otherwise inferred known type -> canonical inferred term
otherwise none
```

- [ ] **Step 9: Run signature and trusted-return tests.**

```sh
cargo test -p phalcom-semantic --test semantic callable_publication_trusted -- --nocapture
cargo test -p phalcom-semantic --test semantic direct_constructor_result_is_proven_and_records_constructor_dependency -- --nocapture
```

Expected: constructors and trusted natives remain established.

- [ ] **Step 10: Commit.**

```sh
git add phalcom-semantic/src/signature.rs phalcom-semantic/src/checker/declaration_signature.rs phalcom-semantic/src/checker/analysis.rs phalcom-semantic/src
git commit -m "feat(semantic): model callable return contract validation"
```

---

## Task 4: Compute source-contract validation from body exits

**Files:**
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/checker/body.rs`
- Modify: `phalcom-semantic/src/checker/analysis.rs`
- Modify: `phalcom-semantic/src/checker/context.rs` `CallableReturnContract`
- Test: `phalcom-semantic/tests/semantic/capabilities/authority.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/callable_publication.rs`

**Interfaces:**
- Consumes: `NormalReturnFact`, `CallableAnalysisStatus`, `CallableReturnContract`, bounded relation checker, `EvidenceStatus::meet`.
- Produces: final `CallableAnalysis.return_validation`.

- [ ] **Step 1: Extend `CallableReturnContract` with declaration basis.**

Change:

```rust
pub struct CallableReturnContract {
    pub ty: TypeId,
    pub basis: DeclaredTypeBasis,
    pub origin: EvidenceOrigin,
    pub source: Option<SourceRange>,
}
```

In `body.rs`, populate `basis` directly from `signature.declared_return.basis` when binding the declaration-owned return requirement.

- [ ] **Step 2: Add aggregation tests before implementation.**

Add low-level tests for a helper with the semantic shape:

```rust
fn validate_return_contract(
    ctx: &CheckingContext<'_>,
    contract: Option<&CallableReturnContract>,
    body_status: CallableAnalysisStatus,
    exits: &[NormalReturnFact],
) -> ReturnContractValidation
```

Required table:

| Contract | Body status | Normal exits | Result |
|---|---|---|---|
| none / non-source | Complete | any | `NotApplicable` |
| source | non-Complete | any | `Blocked` |
| source | Complete | none | `Satisfied(Established)` |
| source | Complete | all clean established + Proven | `Satisfied(Established)` |
| source | Complete | any clean assumed + all Proven | `Satisfied(Assumed)` |
| source | Complete | any Refuted | `Refuted` |
| source | Complete | Unknown/blocked admissible knowledge | `Blocked` |
| source | Complete | Dynamic | `DynamicBoundary` |
| source | Complete | invalid recovery known but relation would otherwise prove | `Blocked` |

- [ ] **Step 3: Implement contract aggregation in `CheckingContext` finalization.**

Use bounded relation checks against the existing shared checker budget/cancellation state. For each exit:

1. Obtain `publication_knowledge()`.
2. If it is known, check it against `contract.ty`.
3. On `Proven`, meet the knowledge's `EvidenceStatus` into the support accumulator.
4. On `Refuted`, return `Refuted` immediately.
5. On blocked/cancelled/budget/internal outcomes, return `Blocked`.
6. On Dynamic, return `DynamicBoundary`.

Important ordering for an invalid recovery value:

```text
raw knowledge may be Known<T>
publication_knowledge is Unknown(SuppressedByInvalidCause)
therefore validation is Blocked, never Satisfied
```

- [ ] **Step 4: Preserve explicit type-mismatch refutation.**

A direct/tail expression that is genuinely incompatible with the source contract must produce `Refuted`, not merely `Blocked` because the diagnostic made the expression invalid.

To achieve this without trusting invalid recovery facts:

- perform the compatibility relation against the raw `knowledge` first when it is Known;
- if the relation is `Refuted`, classify as `Refuted`;
- only if the raw relation is compatible should causal/status admissibility decide whether the contract is `Satisfied` or `Blocked`.

This ordering distinguishes:

```text
Known<String> vs Int + ReturnMismatch -> Refuted
Known<Int> with unrelated invalid child cause -> Blocked
```

Do not use a diagnostic-code search to infer proof state.

- [ ] **Step 5: Store the result on `CallableAnalysis`.**

Before constructing `CallableAnalysis` in `finalize_with_normal_returns`, compute:

```rust
let return_validation = self.validate_return_contract(status, &normal_returns);
```

and publish it into the analysis product.

- [ ] **Step 6: Add integration tests for established and assumed certification.**

The `Dog -> Animal` test from Task 0 must become:

```text
Factory.make analysis.return_validation == Satisfied(Established)
Factory.make canonical public type       == Animal
caller result status                      == Established
```

The `echo(value: String) -> String { value }` case must become:

```text
Echo.echo analysis.return_validation == Satisfied(Assumed)
caller result status                  == Assumed
```

- [ ] **Step 7: Add vacuous `Never` contract validation test.**

Use a complete callable that always throws and has a source return annotation. Assert:

```text
normal_returns.is_empty()
return_validation == Satisfied(Established)
```

Do not add this assertion for cancelled/budget/blocked analyses.

- [ ] **Step 8: Run focused tests.**

```sh
cargo test -p phalcom-semantic --test semantic authority -- --nocapture
cargo test -p phalcom-semantic --test semantic callable_publication -- --nocapture
```

At this point analyses should carry correct validation, but call sites may still use stale dispatch/publication until Tasks 5–6.

- [ ] **Step 9: Commit.**

```sh
git add phalcom-semantic/src/checker phalcom-semantic/tests/semantic/capabilities
git commit -m "feat(semantic): certify source return contracts from body evidence"
```

---

## Task 5: Publish validation and inference through one session fixed point

**Files:**
- Modify: `phalcom-semantic/src/session.rs`
- Modify: `phalcom-semantic/src/signature.rs` if a small publication helper is useful.
- Test: `phalcom-semantic/tests/semantic/capabilities/callable_publication.rs`
- Test: `phalcom-semantic/tests/semantic/incremental/*` relevant to callable dependency reuse.

**Interfaces:**
- Consumes: `CallableAnalysis.return_validation`, `NormalReturnFact`, `normal_return_summary()`, `CallableSemanticSignature::published_return_knowledge()`.
- Produces: canonical signature updates plus dispatch projection updates and dependent-body refresh.

- [ ] **Step 1: Add a RED interprocedural authority-propagation chain.**

Create:

```phalcom
class Source {
  @class
  make() -> Int { 1 }
}
class Middle {
  @class
  forward() { Source.make() }
}
class Probe {
  @class
  run() { let result = Middle.forward() }
}
```

Expected final facts:

```text
Source.make validation       = Satisfied(Established)
Source.make public return    = Established<Int>
Middle.forward inferred      = Established<Int>
Probe.result                 = Established<Int>
```

The test proves that a validation-status-only change can trigger dependent reanalysis.

- [ ] **Step 2: Rename/refactor `refresh_inferred_callable_results()` around all callable return publication.**

Recommended name:

```rust
fn refresh_callable_return_publications(...)
```

It must process two independent changes:

```text
A. source contract validation changes
B. unannotated inferred return changes
```

Do not create two independent fixed-point loops.

- [ ] **Step 3: Remove the current candidate gate based solely on `published_return_knowledge().is_unknown()`.**

Current logic skips annotated callables. Replace it with per-analysis publication computation:

```rust
let old_public = signature.published_return_knowledge();

if signature.return_validation != analysis.return_validation {
    signature.return_validation = analysis.return_validation;
}

if signature.declared_return.is_unknown() {
    let summary = normal_return_summary(store, &analysis.exits.normal_returns);
    signature.inferred_return = publishable_inferred_return(analysis.status, summary);
}

let new_public = signature.published_return_knowledge();
if new_public != old_public {
    changed_callables.insert(signature.callable.clone());
    let _ = dispatch.update_callable_return_type(&signature.callable, new_public);
}
```

- [ ] **Step 4: Gate inferred publication on complete analysis.**

Define a small helper:

```rust
fn publishable_inferred_return(
    status: CallableAnalysisStatus,
    summary: TypeKnowledge,
) -> Option<TypeKnowledge>
```

Rules:

```text
Complete + Known/Dynamic -> Some(summary)
Complete + Unknown       -> None (leave canonical inference unavailable)
Partial/Blocked/Cancelled/Budget/Internal -> None
```

Do not retain a stale previous inferred return when the current revision becomes incomplete. Clear `signature.inferred_return` when current complete proof is no longer available.

- [ ] **Step 5: Treat evidence-status changes as semantic changes even when `TypeId` is unchanged.**

Example:

```text
old public = Assumed<Int>
new public = Established<Int>
```

must add the callable to `changed_callables` and update dispatch. Compare full `TypeKnowledge`, not only `ty()`.

- [ ] **Step 6: Recheck only semantic dependents, as today.**

Preserve the existing dependency-filtered body refresh. Do not broaden to a workspace-wide recheck.

When a dependent body is recomputed, its own `return_validation`/inference may change, causing the next fixed-point iteration.

- [ ] **Step 7: Preserve iteration bound and cancellation.**

Keep a deterministic bound at least as strong as the existing:

```rust
let max_iterations = callable_analyses.len().saturating_add(1).max(1);
```

If the loop cannot stabilize within the bound, do not invent a result. Preserve existing recursive/unknown behavior and add a focused regression if behavior changes.

- [ ] **Step 8: Update dispatch from canonical signatures, never vice versa.**

The session comment should remain explicit:

```text
CallableSemanticSignature is canonical publication.
SurfaceDispatchResolver is a derived lookup projection.
```

No call path may write inferred authority back into the signature.

- [ ] **Step 9: Run publication and incremental tests.**

```sh
cargo test -p phalcom-semantic --test semantic callable_publication -- --nocapture
cargo test -p phalcom-semantic --test semantic incremental -- --nocapture
```

Also run the new three-callable chain test alone to inspect failure output if needed.

- [ ] **Step 10: Commit.**

```sh
git add phalcom-semantic/src/session.rs phalcom-semantic/src/signature.rs phalcom-semantic/tests
git commit -m "fix(semantic): publish callable return authority through canonical fixed point"
```

---

## Task 6: Remove exact-dispatch return proof laundering

**Files:**
- Modify: `phalcom-semantic/src/checker/call.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/authority.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/callable_publication_trusted.rs`
- Test: generic-call tests that exercise fixed/generic return derivation.

**Interfaces:**
- Consumes: `CallableApplicationTarget.signature.return_type`, target selection authority, receiver/callable premise authority, `EvidenceStatus::meet`.
- Produces: call result authority no stronger than the published signature return and required call premises.

- [ ] **Step 1: Add a unit regression directly around fixed-return derivation if the helper is testable.**

Construct a target with:

```text
authority          = ExactDispatch
signature return   = Assumed<String>
premise            = Established<Receiver>
```

Expected result:

```text
Assumed<String>
```

Then construct the same target with an established signature return and expect established.

- [ ] **Step 2: Rewrite `derive_fixed_return()` to start from signature knowledge, not reconstructed established knowledge.**

Delete the current pattern:

```rust
let return_type = match &target.signature.return_type {
    TypeKnowledge::Known(evidence) => TypeKnowledge::established(evidence.ty(), origin),
    other => other.clone(),
};
```

Replace it with authority-preserving logic:

```rust
fn derive_fixed_return(
    target: &CallableApplicationTarget,
    premise: &CallPremise,
    range: SourceRange,
) -> TypeKnowledge {
    let mut result = target.signature.return_type.clone();
    let Some(return_status) = result.status() else {
        return result.with_range(range);
    };
    let Some(premise_status) = premise.knowledge.status() else {
        return premise.knowledge.clone();
    };

    let maximum = return_status
        .meet(target_base_authority(target))
        .meet(premise_status);

    result = weaken_known_to_status(
        result,
        maximum,
        target_fixed_return_origin(target),
        range,
    );
    result
}
```

The signature return status is now an explicit cap.

- [ ] **Step 3: Verify `weaken_known_to_status()` cannot strengthen.**

Change its contract so a caller cannot pass `Established` and upgrade assumed input. Either:

```rust
let status = evidence.status().meet(maximum);
```

as today after Task 1, or rename the parameter to `maximum` and add a unit test:

```text
input Assumed + maximum Established -> Assumed
```

- [ ] **Step 4: Audit generic return derivation for the same bug class.**

Run:

```sh
rg "TypeKnowledge::established\(|return_type|GenericInference" phalcom-semantic/src/checker/call.rs phalcom-semantic/src/checker/inference.rs
```

For each call-result constructor, record the authority premises. Generic inference already tracks proof state; preserve that architecture. Do not replace solver-local proof tracking with call-target authority.

- [ ] **Step 5: Ensure callable-value invocation remains capped by callable-value evidence.**

`callable_value_target()` currently creates assumed parameter/return signature facts and carries `CallTargetAuthority::CallableValue(status)`. Confirm both direct invocation and explicit `.call()` remain no stronger than:

```text
meet(callable-value status, signature-return status, receiver/call premise status)
```

- [ ] **Step 6: Run the Task 0 exact-dispatch test.**

```sh
cargo test -p phalcom-semantic --test semantic exact_dispatch_does_not_upgrade_assumed_source_return -- --nocapture
```

Expected: GREEN with `Assumed<String>`.

- [ ] **Step 7: Run established-contract and trusted-return tests.**

```sh
cargo test -p phalcom-semantic --test semantic established_body_certifies_declared_public_return_without_narrowing_api -- --nocapture
cargo test -p phalcom-semantic --test semantic callable_publication_trusted -- --nocapture
cargo test -p phalcom-semantic --test semantic direct_constructor_result_is_proven_and_records_constructor_dependency -- --nocapture
```

Expected: validated source literal returns, constructors, and native returns remain established.

- [ ] **Step 8: Commit.**

```sh
git add phalcom-semantic/src/checker/call.rs phalcom-semantic/src/checker/inference.rs phalcom-semantic/tests
git commit -m "fix(semantic): preserve callable return evidence through exact dispatch"
```

---

## Task 7: Make new proof products deterministic, explainable, and incrementally visible

**Files:**
- Modify: `phalcom-semantic/src/db/fingerprint.rs`
- Modify if required: `phalcom-semantic/src/explain.rs` or explanation enums/modules.
- Modify: tests asserting explanation status/origin.
- Modify: incremental tests for callable semantic-product stability.

**Interfaces:**
- Consumes: `NormalReturnFact`, `ReturnContractValidation`, `CallableAnalysis`.
- Produces: deterministic fingerprints and explanations reflecting actual authority.

- [ ] **Step 1: Add explicit hash helpers.**

Add:

```rust
fn hash_return_contract_validation(
    validation: ReturnContractValidation,
    hasher: &mut impl Hasher,
) {
    match validation {
        ReturnContractValidation::NotApplicable => 0u8.hash(hasher),
        ReturnContractValidation::Unchecked => 1u8.hash(hasher),
        ReturnContractValidation::Satisfied(EvidenceStatus::Established) => 2u8.hash(hasher),
        ReturnContractValidation::Satisfied(EvidenceStatus::Assumed) => 3u8.hash(hasher),
        ReturnContractValidation::Refuted => 4u8.hash(hasher),
        ReturnContractValidation::Blocked => 5u8.hash(hasher),
        ReturnContractValidation::DynamicBoundary => 6u8.hash(hasher),
    }
}
```

Use stable explicit discriminants, not `Debug` text.

- [ ] **Step 2: Finalize `hash_exit_facts()`.**

For each `NormalReturnFact`, hash:

```text
knowledge semantic content
flow summary semantic content
analysis-status semantic shape
causal-invalidity shape (not allocator-local cause ID)
```

Do not hash `DiagnosticCauseId` identity. Existing `hash_analysis_status()` and `hash_causal_invalidity()` already demonstrate the correct allocator-independent policy.

- [ ] **Step 3: Hash `CallableAnalysis.return_validation`.**

In `callable_body_product_fingerprint()`, include the validation state because dependent semantic publication changes when it changes.

- [ ] **Step 4: Hash canonical signature validation where signature product fingerprints are computed.**

A change from:

```text
Unchecked -> Satisfied(Established)
```

must change the signature semantic fingerprint even when the declared `TypeId` is identical.

- [ ] **Step 5: Correct method-call explanation authority assertions.**

Existing helper `assert_method_call_evidence()` in `authority.rs` assumes every known method call is established. Split it into an explicit expectation API, for example:

```rust
fn assert_method_call_evidence(
    analysis: &CallableAnalysis,
    expression: &ExpressionAnalysis,
    expected_type: TypeId,
    expected_status: EvidenceStatus,
    expected_origin: EvidenceOrigin,
)
```

Then assert `node.status == expected_status` rather than hard-coding established.

- [ ] **Step 6: Ensure validation has an explanation parent or inspectable analysis state without inventing a parallel DAG.**

Minimum requirement: callers can inspect `CallableAnalysis.return_validation`, and existing return relation explanations still show each return proof/refutation. If a new explanation step is added, keep it small:

```text
CallableContractValidation {
    callable,
    declared,
    result
}
```

Do not duplicate every return relation node; reference them as parents.

- [ ] **Step 7: Add incremental test for status-only public change.**

Construct a revision where a callee's public type stays `Int` but its authority changes from assumed to established due to body evidence. Assert:

- callee signature fingerprint changes;
- dependent callable recomputes;
- unrelated callable product remains reusable/pointer-stable according to existing incremental conventions.

- [ ] **Step 8: Run fingerprint and incremental suites.**

```sh
cargo test -p phalcom-semantic --lib db::fingerprint -- --nocapture
cargo test -p phalcom-semantic --test semantic incremental -- --nocapture
cargo test -p phalcom-semantic --test semantic authority -- --nocapture
```

- [ ] **Step 9: Commit.**

```sh
git add phalcom-semantic/src/db phalcom-semantic/src/explain.rs phalcom-semantic/tests
git commit -m "test(semantic): fingerprint callable proof authority"
```

---

## Task 8: Complete the Part 1 authority regression matrix

**Files:**
- Modify: `phalcom-semantic/tests/semantic/capabilities/authority.rs`
- Modify: `phalcom-semantic/tests/semantic/capabilities/callable_publication.rs`
- Modify: `phalcom-semantic/tests/semantic/capabilities/callable_publication_trusted.rs`
- Optionally create: `phalcom-semantic/tests/semantic/foundations/evidence_authority.rs` and register it in the existing module tree if the low-level matrix would make `authority.rs` unwieldy.
- Modify: `phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md` if that ledger tracks these capabilities.

**Interfaces:**
- Consumes: completed Part 1 APIs.
- Produces: durable acceptance coverage for every proof-strength transition changed by this plan.

- [ ] **Step 1: Add the declaration/body/publication matrix.**

Cover at least:

```text
A. annotated T + Established<T> body              -> Satisfied(Established), public Established<T>
B. annotated super T + Established<Subtype> body -> Satisfied(Established), public Established<T>
C. annotated T + Assumed<T> body                  -> Satisfied(Assumed), public Assumed<T>
D. annotated T + Unknown body                     -> Blocked, public Assumed<T> contract only
E. annotated T + incompatible Established<U>      -> Refuted, diagnostic, never public Established<T>
F. unannotated + clean Established<T>             -> inferred Established<T>
G. unannotated + clean Assumed<T>                 -> inferred Assumed<T>
H. unannotated + invalid recovery Known<T>        -> no established inference
I. complete abrupt-only annotated body             -> Satisfied(Established)
J. complete abrupt-only unannotated body           -> inferred Established<Never>
```

Each case must inspect both the callable analysis and a downstream call/binding where applicable.

- [ ] **Step 2: Add call-target authority cases.**

Cover:

```text
ExactDispatch + Assumed signature return        -> Assumed
ExactDispatch + Established signature return    -> Established
CallableValue(Assumed) + Established return     -> Assumed
CallableValue(Established) + Assumed return     -> Assumed
Structural builtin with compiler-owned return   -> Established only when signature itself is established
```

- [ ] **Step 3: Add relation-proof non-laundering case.**

Use a source parameter/body relation where `String <: Object` is Proven but the value is assumed. Assert the result remains assumed.

- [ ] **Step 4: Add recovery-causality composition case.**

A callee with an invalid child and known recovery tail must not cause a clean established return in its caller. Assert:

```text
callee normal exit raw knowledge may be Known
callee publication knowledge is Unknown/blocked
callee inferred/public established result absent
caller does not receive Established<T>
```

- [ ] **Step 5: Add trusted-regression table.**

Keep native/constructor/compiler semantic cases table-driven where possible:

```text
constructor Self
System.print Unit
System.gc Unit
setter Unit
```

- [ ] **Step 6: Add determinism check.**

Where the fixture supports source variants, reorder independent methods/classes without changing semantics and assert equivalent public return knowledge/validation. Do not require allocator IDs or source ranges to be identical.

- [ ] **Step 7: Run the complete semantic test target.**

```sh
cargo test -p phalcom-semantic --test semantic -- --nocapture
```

No ignored test may be introduced to hide a Part 1 failure.

- [ ] **Step 8: Commit.**

```sh
git add phalcom-semantic/tests/semantic
git commit -m "test(semantic): lock callable proof authority invariants"
```

---

## Task 9: Full Part 1 closure gate and proof-construction audit

**Files:**
- Audit: all `phalcom-semantic/src/**/*.rs`
- Audit: `phalcom-semantic/tests/semantic/**/*`
- Modify only findings that violate Part 1 laws.

**Interfaces:**
- Consumes: all prior tasks.
- Produces: a clean, reviewable Part 1 milestone on which Part 2 may depend.

- [ ] **Step 1: Audit all established-knowledge constructors.**

Run:

```sh
rg -n "TypeKnowledge::established\(" phalcom-semantic/src
```

Classify every match as:

```text
AUTHORITATIVE NEW EVIDENCE
  literal syntax
  constructor semantics
  trusted native/declaration semantics
  validated source callable boundary
  trusted flow observation

DERIVATION FROM PREMISES
  must preserve/meet premise authority

SUSPICIOUS
  cannot identify an independent proof source
```

Fix every `SUSPICIOUS` match that is in Part 1's callable/publication surface. Record out-of-scope branch/loop/field findings for their later plans rather than silently broadening this plan.

- [ ] **Step 2: Audit every `RelationOutcome::Proven` consumer in callable/publication paths.**

Run:

```sh
rg -n "RelationOutcome::Proven" phalcom-semantic/src/checker phalcom-semantic/src/session.rs
```

For each Part 1 consumer, verify it proves only relation satisfaction and does not upgrade premise evidence without the explicit source-contract certification rule.

- [ ] **Step 3: Audit obsolete exit-field names and duplicate return authority state.**

Run:

```sh
rg "normal_return_values|exits\.returns|minimum_evidence_status" phalcom-semantic
```

Expected: zero semantic-code matches. Historical docs may contain old names and do not block closure.

- [ ] **Step 4: Format and lint.**

```sh
cargo fmt --all -- --check
cargo clippy -p phalcom-semantic --all-targets -- -D warnings
```

Fix only warnings introduced or exposed by this work; do not launch unrelated cleanup.

- [ ] **Step 5: Run focused Part 1 suites.**

```sh
cargo test -p phalcom-semantic --lib types::evidence -- --nocapture
cargo test -p phalcom-semantic --test semantic authority -- --nocapture
cargo test -p phalcom-semantic --test semantic callable_publication -- --nocapture
cargo test -p phalcom-semantic --test semantic callable_publication_trusted -- --nocapture
cargo test -p phalcom-semantic --test semantic incremental -- --nocapture
```

- [ ] **Step 6: Run the full crate suite.**

```sh
cargo test -p phalcom-semantic
```

Expected: GREEN.

- [ ] **Step 7: Run workspace checks required by the repository CI contract.**

Use the repository's current CI commands if they differ from the following baseline:

```sh
cargo test --workspace
cargo fmt --all -- --check
```

- [ ] **Step 8: Verify no accidental public API narrowing.**

Inspect at least one `Dog -> Animal` validated method and assert:

```text
body exit knowledge = Dog
published callable return = Animal
call-site knowledge = Animal
```

This is the key declaration-vs-proof boundary.

- [ ] **Step 9: Commit closure/audit fixes.**

```sh
git add -A
git commit -m "fix(semantic): close callable proof authority milestone"
```

If no closure edits are needed, do not create an empty commit; record the successful gate in the PR/work log.

---

# Acceptance criteria

Part 1 is complete only when all of the following are true:

## Evidence authority

- `EvidenceStatus` has one canonical meet operation.
- No exact dispatch path promotes an assumed signature return to established.
- Relation success cannot independently strengthen value evidence.
- Generic/callable-value fixed returns obey the same weakest-premise law.

## Declaration vs proof

- Source return annotations remain declaration contracts, initially assumed.
- Body validation is represented separately by `ReturnContractValidation`.
- Validated public calls retain the declared return type, not a narrower body type.
- Source contracts supported only by assumed body evidence remain assumed.
- Refuted/blocked contracts never become established public results.

## Recovery quarantine

- Normal returns carry `knowledge`, `status`, `causal_invalidity`, and their real exit `flow` together.
- Invalid/suppressed recovery knowledge cannot establish inferred returns.
- Incomplete/cancelled/budget/internal body analyses cannot publish fresh inferred or certified established results.

## Trusted semantics

- Constructors remain established `Self`/instance results.
- trusted native returns remain established.
- declaration-owned compiler semantics such as setter `Unit` remain established.

## Interprocedural publication

- Canonical signatures, not dispatch tables, own return publication.
- Dispatch is refreshed as a derived projection.
- status-only changes such as `Assumed<Int> -> Established<Int>` invalidate semantic dependents.
- recursive/unavailable inference remains unknown rather than fabricated.

## Incremental and diagnostics

- structured normal exits and return validation are included in deterministic product fingerprints.
- diagnostic/explanation status matches actual evidence authority.
- unrelated callable products retain existing incremental reuse behavior.

---

# Part 2 dependency contract

Part 2 — Field Contracts and Constructor Lifecycle Correctness — may assume all of the following APIs/invariants exist after this plan:

```rust
EvidenceStatus::meet(...)

NormalReturnFact {
    knowledge,
    flow,
    status,
    causal_invalidity,
}

BodyExitFacts {
    normal_returns,
    throws,
    unreachable,
}

CallableAnalysis.return_validation
CallableSemanticSignature.return_validation
CallableSemanticSignature::published_return_knowledge()
```

Most importantly, Part 2 may use:

```rust
constructor_analysis.exits.normal_returns[i].flow
```

as the actual field/binding flow snapshot at that normal constructor exit. It MUST NOT reconstruct constructor lifecycle from `entry_flow`, final checker state, or raw return knowledge.

# Final implementation principle

The review criterion for every Part 1 change is simple:

> If a line creates `Established<T>`, the implementation must be able to name the independent proof that justifies that strength. Exact selector resolution, a successful subtype relation, or the presence of a source annotation is not by itself such a proof.

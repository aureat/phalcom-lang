# Part 02 — Field Contracts and Constructor Lifecycle Correctness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make instance-field typing and constructor lifecycle publication sound by separating initialization from contract validity, preserving the actual value written to a field, certifying field contracts only from admissible normal constructor exits, and publishing field reads with the correct evidence authority.

**Architecture:** Reuse the existing `FieldState`, `FieldLifecycleTable`, `TypeKnowledge`, relation engine, and constructor-first session ordering. Extend field flow state with the two pieces it currently drops—contract validity and causal invalidity—and make field writes perform the same kind of explicit reconciliation already used for local bindings. Part 1's structured `NormalReturnFact` becomes the authoritative source of constructor exit flow; lifecycle finalization reduces those exit facts into one public `FieldLifecycleFact`. No parallel object-state solver, alias analysis, or new proof system is introduced.

**Tech Stack:** Rust, `phalcom-semantic`, `FieldId`, `FieldSemanticSignature`, `FieldState`, `FieldLifecycleTable`, `TypeKnowledge`, `EvidenceStatus`, `CausalInvalidity`, `RelationOutcome`, `FlowState`, `NormalReturnFact` from Part 1, `SemanticWorkspaceSession`, deterministic semantic fingerprints, semantic capability/integration tests.

**Spec:** This plan implements Part 2 of the ratified six-part typing-correctness architecture. It depends on Part 1 — Evidence Authority and Callable Contract Certification — specifically its structured normal-return facts, admissible publication rules, and evidence-status meet semantics. Repository source is authoritative for implementation details; the grounding revision below records the pre-Part-1 shape that this plan must be rebased onto after Part 1 lands.

## Repository grounding

This plan was freshly grounded against `aureat/phalcom-lang` `main` at:

```text
24fc9fd98f3c3c534c4d52b613962a39b9374185
feat(semantic): add rich type diagnostics tests and polish presentation
```

Part 1 intentionally changes several anchors named below. Before implementing this plan, re-resolve the exact post-Part-1 definitions rather than preserving stale field names mechanically.

Current field/lifecycle anchors at the grounding revision:

- `phalcom-semantic/src/checker/flow/state.rs`
  - `FieldState` stores `field`, `contract`, `current`, `initialization`, and `version`.
  - it does **not** store whether `current` satisfies the contract.
  - it does **not** store field-write causal invalidity.
  - field joins combine `current` and `FieldInitialization`, but have no contract-validity lattice.
- `phalcom-semantic/src/checker/expression.rs`
  - direct field assignment analyzes the RHS and performs `apply_assignability(...)`.
  - regardless of relation outcome, it then calls `ctx.write_current_field(field_id, field_k, val_typed.knowledge)`.
  - therefore a mismatching write still records the field as `DefinitelyInitialized` with no persistent record that the contract was refuted.
  - direct `Expr::Field` reads receive only the current `TypeKnowledge`; causal invalidity is not propagated from field state.
- `phalcom-semantic/src/checker/context.rs`
  - `write_current_field(...)` seeds/writes a field as `DefinitelyInitialized` and accepts no relation state or causal invalidity.
  - `resolve_current_field(...)` returns `(FieldId, TypeKnowledge)` only.
  - `get_field(...)` projects the declaration signature directly; ordinary receiver property access therefore has no access to finalized lifecycle authority.
- `phalcom-semantic/src/checker/field_lifecycle.rs`
  - `FieldLifecycleFact` stores `field`, `contract`, `read_knowledge`, and `initialization`.
  - `default_field_seeds(...)` converts any relation reported as `Proven` into `Established<contract>` even if the initializer itself was only assumed.
  - `finalize_instance_field_lifecycle(...)` treats `DefinitelyInitialized` as sufficient to publish `Established<contract>` and does not inspect whether writes were valid.
- `phalcom-semantic/src/checker/analysis.rs`
  - `FlowFieldSummary` stores contract/current/initialization only.
  - at the grounding revision, `BodyExitFacts.returns` does not contain actual return-site flow; Part 1 replaces this with structured normal-return facts carrying the real exit snapshot.
- `phalcom-semantic/src/session.rs`
  - field defaults are computed before callable bodies.
  - constructors are analyzed before ordinary methods.
  - `field_lifecycle` is mutated after each constructor.
  - the same evolving lifecycle table is then supplied as a seed to later constructor bodies, creating a source-order hazard: a later constructor may enter with a field treated as initialized merely because an earlier constructor established it.
- `phalcom-semantic/src/db/fingerprint.rs`
  - `FlowFieldSummary` is fingerprinted explicitly and must include any new semantic field dimensions.
- `phalcom-semantic/tests/semantic/capabilities/fields.rs`
  - already covers a valid default, constructor-only initialization, field branch joins, and ordinary read/write flow.
  - it does not cover mismatching constructor writes, assumed writes, multiple constructor independence, causal invalidity, or lifecycle authority.

The existing implementation is close to the desired architecture. This plan is a hardening/refinement of that model, not a replacement.

---

# 1. Dependency gate from Part 1

Part 2 must not begin until Part 1 is merged and green.

The implementation must consume these Part-1 interfaces or their approved final equivalents:

```rust
pub enum EvidenceStatus {
    Established,
    Assumed,
}

impl EvidenceStatus {
    pub const fn meet(self, other: Self) -> Self;
}

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

The exact method names may have changed during Part 1, but the semantic contract may not:

1. each normal callable exit carries the actual `FlowStateSummary` at that exit;
2. invalid/suppressed recovery knowledge is distinguishable from admissible proof;
3. evidence-strength meet is canonical and reusable;
4. a callable's public proof cannot be established from invalid recovery facts.

If Part 1 still exposes `BodyExitFacts.returns` as a synthetic entry-flow vector or stores return values separately from their status/flow, stop. Part 2 would otherwise rebuild lifecycle proof on a known-unsound foundation.

---

# 2. Problem statement

A field declaration contains a persistent contract:

```phalcom
_value: Int
```

A constructor write contains a runtime value fact:

```phalcom
_value = expression
```

Those facts answer different questions:

```text
Initialization question:
    Did execution write a value to `_value` on this path?

Contract question:
    What value knowledge was written, and does that knowledge satisfy Int?
```

The current implementation partially separates declaration contract and current knowledge, but collapses those two questions during lifecycle finalization.

Today the following erroneous program can conceptually flow like this:

```phalcom
class Cell {
  _value: Int

  @constructor
  new() {
    _value = "wrong"
  }
}
```

```text
write happened
    ↓
FieldInitialization::DefinitelyInitialized
    ↓
constructor normal exit says definitely initialized
    ↓
finalize_instance_field_lifecycle
    ↓
Established<Int>
```

The `FieldMismatch` diagnostic does not prevent the later lifecycle proof from laundering the declaration into established knowledge.

That violates the central rule established in Part 1:

> A relation proof and an initialization event do not independently establish the truth of a declaration contract.

Field lifecycle must instead prove both:

```text
1. every relevant normal construction exit contains an initialized field;
2. every such field value satisfies the declared contract with sufficient evidence authority.
```

---

# 3. Normative semantic laws

These laws are acceptance requirements. Tests must encode them directly.

## F1 — Initialization and validity are orthogonal

A write of the wrong type is still a write.

```text
_value: Int
_value = "wrong"

initialization = DefinitelyInitialized
current        = Established<String>
validity       = Refuted
```

Do not change the initializer to `MaybeInitialized` merely because the type is wrong. That would destroy useful execution facts and produce poorer diagnostics.

## F2 — Field writes preserve the actual value fact

After a write, `FieldState.current` is the actual RHS `TypeKnowledge` after ordinary proof-preserving transformation.

Never replace it with the declared contract simply because assignability was proven.

```text
contract = Number [Assumed]
actual   = Int [Established]
Int <: Number = Proven

current  = Int [Established]
validity = Validated
```

The narrower actual fact remains available to flow-sensitive code inside the current body.

## F3 — Relation success does not upgrade an assumed write

```text
actual = Assumed<Int>
Int <: Number = Proven
```

must produce:

```text
validity = Assumed
current  = Assumed<Int>
```

not `Validated` and not `Established<Number>`.

## F4 — Refutation is sticky across a reachable field join

If any reachable incoming path contains a refuted field contract, the joined field state cannot be certified as valid.

```text
Validated + Refuted -> Refuted
Assumed   + Refuted -> Refuted
```

The current knowledge can still be a union of both actual values for diagnostics/IDE recovery.

## F5 — Definite initialization requires every reachable predecessor

Existing initialization semantics remain:

```text
Definite + Definite -> Definite
Uninitialized + Uninitialized -> Uninitialized
anything else -> MaybeInitialized
```

Unreachable predecessors are excluded by `FlowState::join*` as they are today.

## F6 — Lifecycle certification considers normal constructor exits only

Throwing/aborting constructor paths do not produce an object and therefore do not require a valid constructed-instance field state.

Normal exits do.

```text
throw path     -> constructor throw exit, not lifecycle certification input
return/fallthrough normal path -> lifecycle certification input
```

## F7 — Constructors are independent alternatives

Each constructor begins from declaration/default field seeds, not from the finalized result of a previously analyzed constructor.

Source order must not affect lifecycle output.

For two constructors:

```phalcom
@constructor newA() { _value = 1 }
@constructor newB() { }
```

`_value` is not definitely initialized for the class merely because `newA` was analyzed before `newB`.

## F8 — Established lifecycle publication requires universal validated exits

A known field contract `T` becomes:

```text
Established<T, FieldLifecycle>
```

only when every relevant normal constructor exit is:

```text
DefinitelyInitialized
AND
FieldContractValidity::Validated
AND
clean enough to participate in formal publication
```

## F9 — Assumed lifecycle publication remains assumed

If all normal exits initialize the field but at least one relies on an assumed value premise:

```text
read_knowledge = Assumed<T, FieldLifecycle>
```

Do not silently strengthen to Established.

## F10 — Refuted or invalid lifecycle cannot publish the declaration as known

A refuted or causally invalid field write may remain queryable as recovery state, but finalized public field knowledge must not claim the contract as established.

Recommended public recovery:

```text
Refuted / invalid -> Unknown(SuppressedByInvalidCause)
Blocked           -> Unknown(InferenceBlocked)
Missing init      -> Unknown(MissingInitializer)
Dynamic boundary  -> Dynamic(the preserved dynamic reason when available)
```

The exact unknown reason may be adjusted to existing enum semantics, but it must be conservative and deterministic.

## F11 — Ordinary instance reads consume lifecycle publication

Inside a body where current flow has a precise field state, direct field syntax reads the current flow state.

Outside that current-object flow—for example `counter.value` through a declaration field projection—formal field reads should consume the finalized lifecycle fact when one exists.

A source annotation by itself remains an assumption, not proof that every object has a valid initialized value.

## F12 — Causal invalidity follows field state

If a field's current value depends on an invalid write, reading that field must propagate the corresponding `CausalInvalidity` into the resulting `TypedExpression`.

This prevents Part 1's callable-publication layer from treating recovery field knowledge as clean proof.

## F13 — Static/class-field lifecycle is out of scope

This plan hardens instance construction. Do not redesign class/static initialization order, module initialization, lazy fields, alias analysis, ownership, or concurrency.

---

# 4. Target data model

## 4.1 Field contract validity

Add a small field-specific relation state in `checker/flow/state.rs` (or a focused neighboring module if the final file becomes too large):

```rust
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FieldContractValidity {
    Unchecked,
    Validated,
    Assumed,
    Refuted,
    Blocked(crate::types::outcome::BlockReason),
    DynamicBoundary(crate::types::outcome::DynamicBoundaryObligation),
}
```

Do **not** add cancelled/budget/internal-failure variants to this enum. Those are query/analysis terminal states, not stable field semantic states. If a relation is cancelled, budget-exceeded, or internally failed, the containing callable analysis already records the terminal condition; field validity should conservatively become `Blocked(...)` only when a stable block reason exists, otherwise leave the field unusable for lifecycle certification through causal/analysis status.

Why not reuse `BindingConsistency` directly?

- `BindingConsistency` contains binding-specific assumption bases and mutation semantics.
- fields need a smaller state focused on object lifecycle reduction.
- forcing fields into the local-binding type would make the types less clear for little code reuse.

The reconciliation *rules* should mirror bindings; the enum need not.

## 4.2 Field state

Extend the existing `FieldState`:

```rust
pub struct FieldState {
    pub field: FieldId,
    pub contract: TypeKnowledge,
    pub current: TypeKnowledge,
    pub initialization: FieldInitialization,
    pub validity: FieldContractValidity,
    pub causal_invalidity: CausalInvalidity,
    pub version: u32,
}
```

`contract` remains persistent across writes and joins.

`current` remains the flow-sensitive actual value fact.

`initialization` says whether a value was written on all relevant paths.

`validity` says whether current execution evidence satisfies the persistent contract.

`causal_invalidity` says whether the current field fact is contaminated by a source error/recovery path.

## 4.3 Flow field summary

Extend `FlowFieldSummary` in `checker/analysis.rs`:

```rust
pub struct FlowFieldSummary {
    pub contract: TypeKnowledge,
    pub current: TypeKnowledge,
    pub initialization: FieldInitialization,
    pub validity: FieldContractValidity,
    pub causal_invalidity: CausalInvalidity,
}
```

This is necessary because Part 1's `NormalReturnFact.flow` is the lifecycle proof input. Dropping validity at the summary boundary would recreate the same laundering bug one layer later.

## 4.4 Field lifecycle fact

Extend `FieldLifecycleFact`:

```rust
pub struct FieldLifecycleFact {
    pub field: FieldId,
    pub contract: TypeKnowledge,
    pub read_knowledge: TypeKnowledge,
    pub initialization: FieldInitialization,
    pub validity: FieldContractValidity,
    pub causal_invalidity: CausalInvalidity,
}
```

The lifecycle fact is a class-level reduction over constructors/defaults; it is not a replacement for per-body `FieldState`.

## 4.5 Field write reconciliation

Introduce a pure helper, preferably in `checker/field_lifecycle.rs` if it remains focused, or a new small `checker/field.rs` only if both read/write and lifecycle logic would otherwise become tangled:

```rust
pub(crate) struct FieldWriteReconciliation {
    pub current: TypeKnowledge,
    pub validity: FieldContractValidity,
}

pub(crate) fn reconcile_field_write(
    contract: &TypeKnowledge,
    actual: &TypeKnowledge,
    relation: RelationOutcome,
) -> FieldWriteReconciliation;
```

This function must be pure: no diagnostics, no flow mutation, no context access.

Mapping:

```text
no known contract                         -> Unchecked
Proven + Established actual               -> Validated
Proven + Assumed actual                   -> Assumed
Refuted                                   -> Refuted
Blocked                                   -> Blocked(reason)
DynamicBoundary                           -> DynamicBoundary(obligation)
Cancelled/Budget/Internal                 -> do not certify; caller preserves terminal analysis state
```

The returned `current` is always `actual.clone()`.

## 4.6 Field validity join

Add one deterministic helper:

```rust
pub(crate) fn join_field_validity(
    inputs: impl IntoIterator<Item = FieldContractValidity>,
) -> FieldContractValidity;
```

Precedence for reachable inputs:

```text
Refuted > Blocked > DynamicBoundary > Unchecked > Assumed > Validated
```

with these important interpretations:

- any refuted reachable path means universal certification fails;
- any blocked path means proof is unavailable;
- a dynamic boundary cannot become a static proof;
- an unchecked path prevents certification;
- otherwise any assumed contributor weakens the result to assumed;
- validated is produced only if every input is validated.

If multiple blocked/dynamic reasons must be combined, use deterministic existing reason-join conventions or stable ordering. Do not make result depend on predecessor traversal order.

---

# 5. File and responsibility map

| File | Responsibility in this plan |
|---|---|
| `phalcom-semantic/src/checker/flow/state.rs` | Extend `FieldState`; define/join field validity; preserve validity/causal state through writes, joins, and loop widening. |
| `phalcom-semantic/src/checker/analysis.rs` | Extend `FlowFieldSummary`; ensure structured normal exits retain all field proof dimensions. |
| `phalcom-semantic/src/checker/context.rs` | Build field summaries; attach lifecycle table; expose current/public field read facts; accept complete field-write reconciliation state. |
| `phalcom-semantic/src/checker/expression.rs` | Reconcile direct field writes, preserve actual RHS knowledge, propagate field causal invalidity on reads, use lifecycle publication for receiver field reads where appropriate. |
| `phalcom-semantic/src/checker/field_lifecycle.rs` | Pure field-write reconciliation (unless extracted), default initializer checking, constructor lifecycle reduction, public read authority. |
| `phalcom-semantic/src/session.rs` | Seed every constructor from raw/default lifecycle facts; finalize across constructors only after/while accumulating independent analyses; supply finalized lifecycle to ordinary bodies. |
| `phalcom-semantic/src/db/fingerprint.rs` | Fingerprint validity/causal changes in field summaries and any lifecycle product if persisted. |
| `phalcom-semantic/tests/semantic/capabilities/fields.rs` | Main source-level lifecycle/field regression suite. |
| `phalcom-semantic/tests/semantic/foundations/flow_graph.rs` or a new focused `foundations/field_flow.rs` | Low-level validity/join laws. Prefer a new focused file if more than ~5 tests are added. |
| `phalcom-semantic/tests/semantic/capabilities/authority.rs` | Cross-check that assumed field evidence cannot become established through callable publication. |
| `phalcom-semantic/tests/semantic/incremental/*` | Only if field/lifecycle products participate in incremental fingerprints at the post-Part-1 revision. |
| `phalcom-semantic/tests/semantic/mod.rs` and capability/foundation module registries | Register new test module only if a new file is created. |

Avoid unrelated refactors in `expression.rs` or `session.rs`. Part 3 owns the broader control-outcome/executable-region cleanup.

---

# 6. Execution order

```text
Task 0  Rebase on Part 1 and freeze field baseline
   |
Task 1  Add field validity + causal state to flow model
   |
Task 2  Add pure field-write reconciliation and correct direct writes
   |
Task 3  Correct default initializer proof
   |
Task 4  Fix constructor seeding independence in session orchestration
   |
Task 5  Rebuild lifecycle finalization from actual normal exits
   |
Task 6  Publish lifecycle-aware field reads and propagate causal invalidity
   |
Task 7  Add multi-constructor and composed proof regressions
   |
Task 8  Update fingerprints and incremental stability coverage
   |
Task 9  Run proof audit and full closure gate
```

Tasks 1–3 establish local field correctness. Task 4 fixes constructor analysis independence. Task 5 is the class-level certification step. Task 6 makes consumers observe the corrected publication. Tasks 7–9 close composition and regression risk.

---

# Task 0: Rebase onto Part 1 and freeze the post-Part-1 baseline

**Files:**
- Read: `phalcom-semantic/src/checker/analysis.rs`
- Read: `phalcom-semantic/src/checker/body.rs`
- Read: `phalcom-semantic/src/checker/context.rs`
- Read: `phalcom-semantic/src/checker/field_lifecycle.rs`
- Read: `phalcom-semantic/src/checker/flow/state.rs`
- Read: `phalcom-semantic/src/session.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/fields.rs`
- Test: Part-1 callable publication tests.

**Interfaces:**
- Consumes: Part-1 `NormalReturnFact`, `BodyExitFacts.normal_returns`, evidence meet, return publication rules.
- Produces: an exact implementation baseline and failing field-correctness probes without changing semantic assertions to fit existing behavior.

- [ ] **Step 1: Verify the branch is clean and Part 1 is present.**

Run:

```sh
git status --short
git rev-parse HEAD
git log -5 --oneline
rg "struct NormalReturnFact|normal_returns" phalcom-semantic/src/checker
rg "fn meet" phalcom-semantic/src/types/evidence.rs
```

Expected:

- no unrelated working-tree changes;
- Part-1 commit(s) are visible;
- structured normal-return facts exist;
- evidence meet exists.

If any required Part-1 interface is absent, do not emulate it locally in Part 2.

- [ ] **Step 2: Run Part-1 closure tests before field work.**

Run the exact closure command named in Part 1, then at minimum:

```sh
cargo test -p phalcom-semantic --test semantic callable_publication -- --nocapture
cargo test -p phalcom-semantic --test semantic authority -- --nocapture
```

Expected: GREEN.

- [ ] **Step 3: Run the existing field capability suite unchanged.**

```sh
cargo test -p phalcom-semantic --test semantic fields -- --nocapture
```

Expected: existing tests are GREEN before new RED probes are added. Record any pre-existing failures separately.

- [ ] **Step 4: Add the minimum RED proof-laundering regression.**

Add to `phalcom-semantic/tests/semantic/capabilities/fields.rs`:

```rust
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

    let exit = constructor
        .exits
        .normal_returns
        .first()
        .expect("constructor normal exit");
    let field = exit.flow.fields.values().next().expect("field exit state");

    assert_eq!(field.initialization, phalcom_semantic::checker::flow::FieldInitialization::DefinitelyInitialized);
    assert_eq!(field.current.ty(), Some(string_ty));
    assert!(
        !f.expression(read, "_value").knowledge.is_established()
            || f.expression(read, "_value").knowledge.ty() != Some(int_ty),
        "a refuted constructor write must not establish the declared field contract"
    );
    assert!(f.diagnostics(phalcom_semantic::diagnostic::DiagnosticCode::FieldMismatch).next().is_some());
}
```

If `Fixture::diagnostics(...)` does not exist after Part 1, use the existing snapshot diagnostic iterator pattern from `authority.rs`; do not add a large fixture API solely for this assertion.

- [ ] **Step 5: Run only the new test and capture the failure.**

```sh
cargo test -p phalcom-semantic --test semantic wrong_constructor_write_is_initialized_but_never_certifies_field_contract -- --nocapture
```

Expected before the fix: failure because lifecycle publication still establishes `Int` or because exit field summaries lack validity/causal state.

- [ ] **Step 6: Commit the RED regression only.**

```sh
git add phalcom-semantic/tests/semantic/capabilities/fields.rs
git commit -m "test(semantic): expose unsound field lifecycle certification"
```

---

# Task 1: Add field validity and causal state to the flow model

**Files:**
- Modify: `phalcom-semantic/src/checker/flow/state.rs`
- Modify: `phalcom-semantic/src/checker/analysis.rs`
- Modify: `phalcom-semantic/src/checker/context.rs` where flow summaries are built.
- Modify: `phalcom-semantic/src/db/fingerprint.rs` only enough to compile; semantic fingerprint assertions land in Task 8.
- Test: create `phalcom-semantic/tests/semantic/foundations/field_flow.rs` if the current foundation registry supports a focused module; otherwise extend the smallest existing flow-state test file.
- Modify test module registries as required.

**Interfaces:**
- Consumes: `TypeKnowledge`, `FieldInitialization`, `CausalInvalidity`, `BlockReason`, `DynamicBoundaryObligation`.
- Produces:
  - `FieldContractValidity`;
  - extended `FieldState`;
  - extended `FlowFieldSummary`;
  - deterministic `join_field_validity(...)`;
  - field joins/widening that preserve all proof dimensions.

- [ ] **Step 1: Write unit tests for the validity lattice before changing structs.**

Add tests equivalent to:

```rust
#[test]
fn field_validity_join_never_strengthens_a_reachable_path() {
    use FieldContractValidity::*;

    assert_eq!(join_field_validity([Validated, Validated]), Validated);
    assert_eq!(join_field_validity([Validated, Assumed]), Assumed);
    assert_eq!(join_field_validity([Assumed, Assumed]), Assumed);
    assert_eq!(join_field_validity([Validated, Unchecked]), Unchecked);
    assert!(matches!(
        join_field_validity([Validated, Refuted]),
        Refuted
    ));
}
```

Add deterministic blocked/dynamic cases using concrete existing reason constructors.

- [ ] **Step 2: Run the focused tests to verify RED.**

```sh
cargo test -p phalcom-semantic --test semantic field_validity_join -- --nocapture
```

Expected: compile failure because `FieldContractValidity`/helper do not exist.

- [ ] **Step 3: Add `FieldContractValidity`.**

In `checker/flow/state.rs`, define:

```rust
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FieldContractValidity {
    Unchecked,
    Validated,
    Assumed,
    Refuted,
    Blocked(crate::types::outcome::BlockReason),
    DynamicBoundary(crate::types::outcome::DynamicBoundaryObligation),
}
```

Keep it near `FieldInitialization` and `FieldState` so the field flow model is readable as one unit.

- [ ] **Step 4: Implement deterministic `join_field_validity`.**

Use an explicit fold with semantic precedence. Do not rely on derived enum ordering.

A valid implementation shape is:

```rust
pub(crate) fn join_field_validity(
    inputs: impl IntoIterator<Item = FieldContractValidity>,
) -> FieldContractValidity {
    let mut result = FieldContractValidity::Validated;
    let mut saw_any = false;

    for input in inputs {
        saw_any = true;
        result = join_two_field_validities(result, input);
    }

    if saw_any {
        result
    } else {
        FieldContractValidity::Unchecked
    }
}
```

`join_two_field_validities` must choose blocked/dynamic reasons deterministically. Prefer reusing existing stable reason-combination helpers if Part 1 exposes them; otherwise compare a stable semantic key, not allocation order.

- [ ] **Step 5: Extend `FieldState`.**

Change:

```rust
pub struct FieldState {
    pub field: FieldId,
    pub contract: TypeKnowledge,
    pub current: TypeKnowledge,
    pub initialization: FieldInitialization,
    pub validity: FieldContractValidity,
    pub causal_invalidity: CausalInvalidity,
    pub version: u32,
}
```

Update all construction sites. Initial values:

- no initializer: `validity = Unchecked`, `causal_invalidity = Clean`;
- existing lifecycle seed: copy lifecycle validity/causal state;
- test-only seed helpers: set explicit values, never hide them behind `Default` unless semantically correct.

- [ ] **Step 6: Extend `FlowFieldSummary`.**

In `checker/analysis.rs`:

```rust
pub struct FlowFieldSummary {
    pub contract: TypeKnowledge,
    pub current: TypeKnowledge,
    pub initialization: FieldInitialization,
    pub validity: crate::checker::flow::FieldContractValidity,
    pub causal_invalidity: CausalInvalidity,
}
```

Then update the context method that converts `FlowState` into `FlowStateSummary` to copy both new fields exactly.

Find the current summary builder with:

```sh
rg "FlowFieldSummary" phalcom-semantic/src/checker phalcom-semantic/src/db
```

- [ ] **Step 7: Extend `FlowState::write_field`.**

Replace the narrow write API:

```rust
write_field(&mut self, field, current, initialization)
```

with either explicit parameters or a small write-state object. Prefer explicit parameters because this is a single internal call path:

```rust
pub fn write_field(
    &mut self,
    field: &FieldId,
    current: TypeKnowledge,
    initialization: FieldInitialization,
    validity: FieldContractValidity,
    causal_invalidity: CausalInvalidity,
)
```

The method must update all four current dimensions atomically before incrementing `version`.

- [ ] **Step 8: Extend field join semantics.**

In `FlowState::join_impl`, change field reduction to compute:

```rust
let current = join_type_knowledge(...);
let initialization = ...; // existing reachable rule
let validity = join_field_validity(incoming.iter().map(|state| state.validity.clone()));
let causal_invalidity = incoming
    .iter()
    .map(|state| state.causal_invalidity)
    .fold(CausalInvalidity::Clean, CausalInvalidity::join);
```

Preserve the existing contract invariant check.

- [ ] **Step 9: Ensure loop widening inherits the corrected field join.**

`widen_loop_state_with_hierarchy()` currently obtains widened fields from `join_impl`. Verify it still does after the change. Add a focused test if the post-Part-1 implementation changed that path.

- [ ] **Step 10: Add a field-flow test with incompatible reachable paths.**

Construct two `FlowState`s with the same field contract:

```text
left:
  current = Int
  initialization = Definite
  validity = Validated

right:
  current = String
  initialization = Definite
  validity = Refuted
```

Assert after join:

```text
initialization = Definite
validity = Refuted
current = Int | String (or equivalent canonical join)
causal_invalidity = joined causes
```

This test encodes F1/F4 and must remain independent of source syntax.

- [ ] **Step 11: Make `db/fingerprint.rs` compile with the new summary shape.**

Add hashing for `validity` and `causal_invalidity` in `hash_flow_summary`. Define a small `hash_field_validity(...)` helper mirroring `hash_binding_consistency(...)` rather than formatting debug strings.

Task 8 will add semantic fingerprint tests; this step only prevents an incomplete model from compiling.

- [ ] **Step 12: Run focused and crate tests.**

```sh
cargo test -p phalcom-semantic --test semantic field_flow -- --nocapture
cargo test -p phalcom-semantic --test semantic fields -- --nocapture
cargo test -p phalcom-semantic --lib checker::flow -- --nocapture
```

Expected: new low-level tests GREEN. The RED lifecycle test may still fail until Task 5.

- [ ] **Step 13: Commit.**

```sh
git add phalcom-semantic/src/checker/flow/state.rs \
        phalcom-semantic/src/checker/analysis.rs \
        phalcom-semantic/src/checker/context.rs \
        phalcom-semantic/src/db/fingerprint.rs \
        phalcom-semantic/tests/semantic
git commit -m "refactor(semantic): track field contract validity in flow"
```

---

# Task 2: Add pure field-write reconciliation and correct direct field writes

**Files:**
- Modify: `phalcom-semantic/src/checker/field_lifecycle.rs` or create focused `phalcom-semantic/src/checker/field.rs` only if code ownership is cleaner after Task 1.
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/fields.rs`
- Test: low-level reconciliation tests beside the pure helper.

**Interfaces:**
- Consumes: `RelationOutcome`, Part-1 `EvidenceStatus::meet`, extended `FieldState`.
- Produces:
  - `FieldWriteReconciliation`;
  - `reconcile_field_write(...)`;
  - `CheckingContext::write_current_field(...)` that accepts validity + causal state;
  - direct field assignment that records the actual value even on mismatch but never loses the refutation.

- [ ] **Step 1: Add pure reconciliation tests.**

Test these exact cases:

```text
Established<Int> + Proven     -> current Established<Int>, Validated
Assumed<Int>     + Proven     -> current Assumed<Int>, Assumed
Established<String> + Refuted -> current Established<String>, Refuted
Unknown + Blocked             -> current Unknown, Blocked
Dynamic + DynamicBoundary     -> current Dynamic, DynamicBoundary
```

Also assert `current == actual` in every case.

- [ ] **Step 2: Run to RED.**

```sh
cargo test -p phalcom-semantic --lib reconcile_field_write -- --nocapture
```

Expected: helper absent.

- [ ] **Step 3: Implement `FieldWriteReconciliation`.**

Recommended code shape:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FieldWriteReconciliation {
    pub current: TypeKnowledge,
    pub validity: FieldContractValidity,
}
```

Implementation must inspect the relation outcome and **actual evidence status**, not contract evidence status, when deciding `Validated` versus `Assumed`.

For `RelationOutcome::Proven { .. }`:

```rust
let validity = match actual.status() {
    Some(EvidenceStatus::Established) => FieldContractValidity::Validated,
    Some(EvidenceStatus::Assumed) => FieldContractValidity::Assumed,
    None => FieldContractValidity::Unchecked,
};
```

Do not call `TypeKnowledge::established(contract_ty, ...)` here.

- [ ] **Step 4: Make field assignment use one relation computation.**

In the direct `Expr::Field` assignment arm in `expression.rs`, keep the existing expected-type analysis, then compute relation once.

Current code calls `apply_assignability(...)` primarily for diagnostics and then discards the semantic outcome before writing.

Refactor to:

1. analyze RHS;
2. call `apply_assignability(...)`;
3. reconcile from `application.outcome.clone()` and RHS knowledge;
4. derive write causal invalidity from:
   - RHS causal invalidity;
   - any relation diagnostic cause/status applied to the assignment;
5. write current field with actual knowledge + validity + causal invalidity;
6. return Unit assignment expression with the same causal invalidity/status.

Do not perform a second independent relation query.

- [ ] **Step 5: Change `CheckingContext::write_current_field`.**

Recommended signature:

```rust
pub(crate) fn write_current_field(
    &mut self,
    field: FieldId,
    contract: TypeKnowledge,
    current: TypeKnowledge,
    validity: FieldContractValidity,
    causal_invalidity: CausalInvalidity,
)
```

If the field has not been seeded, seed it as:

```rust
FieldState {
    field: field.clone(),
    contract,
    current: TypeKnowledge::Unknown(UnknownReason::MissingInitializer),
    initialization: FieldInitialization::Uninitialized,
    validity: FieldContractValidity::Unchecked,
    causal_invalidity: CausalInvalidity::Clean,
    version: 0,
}
```

Then perform a definite write with the supplied actual/validity/causal fields.

- [ ] **Step 6: Add a RED/GREEN source test asserting the wrong actual survives.**

Extend the Task-0 regression so it asserts the constructor exit field has:

```text
current.ty() == String
initialization == DefinitelyInitialized
validity == Refuted
```

and not merely that the final read is non-established.

This should become GREEN at this task even though public lifecycle may still be wrong until Task 5.

- [ ] **Step 7: Add an assumed-write authority test.**

Use an annotated parameter, which enters a body as assumed callable-signature knowledge:

```phalcom
class Cell {
  _value: Number

  @constructor
  new(_ value: Int) {
    _value = value
  }
}
```

At the constructor field exit, assert:

```text
current = Assumed<Int>
validity = Assumed
initialization = DefinitelyInitialized
```

Do not assert final public field read yet; Task 5 owns lifecycle reduction.

- [ ] **Step 8: Run focused tests.**

```sh
cargo test -p phalcom-semantic --test semantic wrong_constructor_write_is_initialized_but_never_certifies_field_contract -- --nocapture
cargo test -p phalcom-semantic --test semantic assumed_constructor_write_remains_assumed_in_field_state -- --nocapture
cargo test -p phalcom-semantic --lib reconcile_field_write -- --nocapture
```

Expected: constructor exit state assertions GREEN. The public lifecycle assertion may remain RED until Task 5; if the test combines both, split the lifecycle expectation into a later test rather than weakening it.

- [ ] **Step 9: Commit.**

```sh
git add phalcom-semantic/src/checker/field_lifecycle.rs \
        phalcom-semantic/src/checker/context.rs \
        phalcom-semantic/src/checker/expression.rs \
        phalcom-semantic/tests/semantic/capabilities/fields.rs
git commit -m "fix(semantic): preserve field write validity and actual evidence"
```

---

# Task 3: Correct default field initializer proof

**Files:**
- Modify: `phalcom-semantic/src/checker/field_lifecycle.rs`
- Modify: `phalcom-semantic/src/checker/context.rs` only if a typed-expression helper is required.
- Test: `phalcom-semantic/tests/semantic/capabilities/fields.rs`

**Interfaces:**
- Consumes: `reconcile_field_write(...)`, `TypedExpression`, `FieldLifecycleFact`, `FieldContractValidity`.
- Produces: default field seeds that preserve actual initializer authority and diagnostics rather than converting relation success into established contract knowledge.

## Why this is a separate task

Current `default_field_seeds(...)` does:

```text
initializer knowledge
    ↓
relation Proven?
    ↓ yes
Established<contract, FieldLifecycle>
```

This is the same relation-proof laundering fixed in Part 1 at callable boundaries.

A default initializer is also executable initialization evidence, so it should use exactly the field-write reconciliation rules established in Task 2.

- [ ] **Step 1: Add a default-initializer mismatch regression.**

```rust
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
    assert!(!value.knowledge.is_established() || value.knowledge.ty() != Some(f.ty("Int")));
    assert!(f.has_diagnostic(DiagnosticCode::FieldMismatch));
}
```

Use existing fixture/snapshot diagnostic APIs if helper names differ.

- [ ] **Step 2: Add an assumed-default test if a stable assumed initializer source is expressible.**

Prefer a default that references declaration-owned assumed knowledge only if the language currently permits it at field initialization. If not, test the pure `reconcile_field_write` path instead and keep the source-level assumed case on constructor parameters. Do not invent unsupported syntax merely to create the test.

- [ ] **Step 3: Run mismatch test to RED.**

```sh
cargo test -p phalcom-semantic --test semantic wrong_default_initializer_never_establishes_declared_field_contract -- --nocapture
```

Expected: current implementation either lacks the correct diagnostic or publishes the contract too strongly.

- [ ] **Step 4: Change default initializer analysis to retain full typed state.**

At the grounding revision `default_field_seeds()` calls `synthesize_expr(...)`, which returns only `TypeKnowledge`.

Switch to a full typed-expression path:

```rust
let initializer = super::expression::synthesize_typed_expr(ctx, default);
```

or the post-Part-1 equivalent that records the ordinary expression product once.

Do not analyze the initializer twice.

- [ ] **Step 5: Apply the contract relation using the normal diagnostic path.**

For known field contracts, call the context relation application once with `DiagnosticCode::FieldMismatch` and a field-default-specific message such as:

```text
"default initializer does not match field `<name>` type"
```

Then pass that relation outcome and `initializer.knowledge` through `reconcile_field_write(...)`.

- [ ] **Step 6: Build the default lifecycle fact without replacing actual knowledge.**

For an initializer that normally produces a value:

```rust
FieldLifecycleFact {
    field,
    contract,
    read_knowledge: reconciliation.current,
    initialization: FieldInitialization::DefinitelyInitialized,
    validity: reconciliation.validity,
    causal_invalidity: initializer.causal_invalidity.join(relation_causal),
}
```

If the initializer is `Never`/does not normally complete, do not claim definite initialization. Use the most conservative existing representation (`MaybeInitialized` if the flow model cannot yet express “constructor entry unreachable”) and record non-publishable knowledge. Part 3/5 can refine control topology later.

- [ ] **Step 7: Preserve the valid established default behavior.**

The existing test:

```text
default_initializer_establishes_instance_field_lifecycle_read
```

must remain GREEN for literal `0 : Int` because the literal is established and the relation is proven.

- [ ] **Step 8: Run field tests.**

```sh
cargo test -p phalcom-semantic --test semantic default_initializer -- --nocapture
cargo test -p phalcom-semantic --test semantic fields -- --nocapture
```

Expected:

- valid literal default publishes an established lifecycle read;
- wrong default emits mismatch and never publishes Established declared type;
- actual wrong type remains visible in seed/recovery state where test access permits.

- [ ] **Step 9: Commit.**

```sh
git add phalcom-semantic/src/checker/field_lifecycle.rs \
        phalcom-semantic/tests/semantic/capabilities/fields.rs
git commit -m "fix(semantic): preserve default field initializer proof authority"
```

---

# Task 4: Make constructor entry seeds independent of constructor analysis order

**Files:**
- Modify: `phalcom-semantic/src/session.rs`
- Modify: `phalcom-semantic/src/checker/body.rs` or `CallableBodyRequest` only if clearer naming is needed.
- Test: `phalcom-semantic/tests/semantic/capabilities/fields.rs`
- Test: optionally a small incremental/session test if the bug is easier to observe through snapshot lifecycle facts.

**Interfaces:**
- Consumes:
  - `default_field_lifecycle`: field state produced only from declarations/default initializers;
  - `field_lifecycle`: finalized class-level publication accumulated from constructor analyses.
- Produces: every constructor body seeded from the same raw/default lifecycle baseline, regardless of which constructor was analyzed first.

## Current hazard

At the grounding revision, session logic begins with:

```text
default_field_lifecycle
field_lifecycle = default_field_lifecycle.clone()
```

After constructor A is analyzed, `field_lifecycle` is finalized and mutated. Constructor B then receives the current `field_lifecycle` through formal inputs/body analysis.

That creates this invalid temporal dependency:

```text
constructor A proof
      ↓
class lifecycle publication
      ↓
constructor B entry state
```

Constructors are alternatives, not sequential executions.

The correct dependency is:

```text
                defaults
                /      \
         constructor A constructor B
                \      /
            lifecycle reduction
```

- [ ] **Step 1: Add a source-order regression with two constructors.**

Use two classes or two fixture variants with constructors reversed:

```phalcom
class Cell {
  _value: Int

  @constructor
  initialized(_ value: Int) {
    _value = value
  }

  @constructor
  missing() {
  }

  read() { _value }
}
```

and the same class with `missing()` before `initialized(...)`.

Assert both snapshots have identical lifecycle outcome:

```text
initialization = MaybeInitialized
read_knowledge is not Established<Int>
```

If duplicate constructor selectors are disallowed, use distinct selectors/labels/names according to Phalcom constructor syntax; preserve both as `@constructor` callables.

- [ ] **Step 2: Run to RED or verify the current source-order hazard explicitly.**

```sh
cargo test -p phalcom-semantic --test semantic constructor_lifecycle_is_independent_of_constructor_source_order -- --nocapture
```

Expected: at least one ordering can incorrectly observe a previously finalized field state at constructor entry, or the new deeper exit assertions expose the pollution.

If the current parser/selector semantics happen to mask the final publication difference, assert constructor B's **entry/exit field state** directly: it must begin from the same default seed in both orderings.

- [ ] **Step 3: Separate constructor seed lifecycle from ordinary-body lifecycle in session orchestration.**

In `session.rs`, keep both variables with clear names:

```rust
let default_field_lifecycle = ...;
let mut finalized_field_lifecycle = default_field_lifecycle.clone();
```

When constructing `FormalQueryInputs`/`CallableBodyRequest`:

```rust
let lifecycle_for_body = if is_constructor {
    &default_field_lifecycle
} else {
    &finalized_field_lifecycle
};
```

Use the default lifecycle for **every constructor**.

Use the finalized lifecycle for ordinary methods after all relevant constructor analysis has been incorporated.

- [ ] **Step 4: Prefer class-level finalization after collecting all constructors.**

The cleanest implementation is:

1. analyze all constructors for a class, each from defaults;
2. collect their `CallableAnalysis` products;
3. call `finalize_instance_field_lifecycle(defaults, constructors)` once for the class;
4. merge those facts into `finalized_field_lifecycle`;
5. analyze ordinary members against that finalized table.

If the current session loop makes one-call finalization awkward, incremental recomputation after each constructor is acceptable **only if** no constructor consumes the evolving result. The final value must still be a deterministic reduction over all constructor analyses.

Do not redesign the whole session pass in this task.

- [ ] **Step 5: Preserve incremental dependencies.**

If body-query input fingerprints include field lifecycle products, make constructor body queries depend on default lifecycle inputs only. Ordinary body queries may depend on finalized lifecycle. Do not create a self-dependency where a constructor body invalidates because of a lifecycle result derived from itself.

Use:

```sh
rg "field_lifecycle" phalcom-semantic/src/db phalcom-semantic/src/session.rs
```

to inspect the post-Part-1 query graph before editing.

- [ ] **Step 6: Run order tests and existing lifecycle tests.**

```sh
cargo test -p phalcom-semantic --test semantic constructor_lifecycle_is_independent_of_constructor_source_order -- --nocapture
cargo test -p phalcom-semantic --test semantic constructor_only_field_requires_all_normal_paths_to_initialize -- --nocapture
cargo test -p phalcom-semantic --test semantic fields -- --nocapture
```

Expected: GREEN except any public validity assertions that still await Task 5.

- [ ] **Step 7: Commit.**

```sh
git add phalcom-semantic/src/session.rs \
        phalcom-semantic/tests/semantic/capabilities/fields.rs
git commit -m "fix(semantic): isolate constructor field entry seeds"
```

---

# Task 5: Rebuild lifecycle finalization from actual admissible normal constructor exits

**Files:**
- Modify: `phalcom-semantic/src/checker/field_lifecycle.rs`
- Modify: `phalcom-semantic/src/session.rs` only for finalization call shape.
- Modify: `phalcom-semantic/src/checker/analysis.rs` only if a small helper on `NormalReturnFact` is required.
- Test: `phalcom-semantic/tests/semantic/capabilities/fields.rs`

**Interfaces:**
- Consumes:
  - Part-1 `CallableAnalysis.exits.normal_returns`;
  - each `NormalReturnFact.flow.fields`;
  - `FieldState`/`FlowFieldSummary` initialization, validity, current, causal invalidity.
- Produces:
  - `FieldLifecycleFact` with initialization, validity, causal invalidity, and correctly-authorized public `read_knowledge`.

## Lifecycle reduction algorithm

For each instance field declared on class `C`:

1. collect all **admissible normal** exits of all constructors for `C`;
2. if there are no normal exits, preserve default lifecycle facts rather than manufacturing construction proof;
3. for each exit, obtain that field's `FlowFieldSummary`;
4. reduce initialization over all exits;
5. reduce validity over all exits;
6. join causal invalidity over all exits;
7. publish read knowledge according to the result matrix below.

### Required publication matrix

For a known contract type `T`:

| Initialization | Validity | Causal | Public `read_knowledge` |
|---|---|---|---|
| Definite | Validated | Clean | `Established<T, FieldLifecycle>` |
| Definite | Assumed | Clean | `Assumed<T, FieldLifecycle>` |
| Definite | Refuted | any | `Unknown(SuppressedByInvalidCause)` |
| Definite | Blocked | any | `Unknown(InferenceBlocked)` or exact stable block-derived unknown |
| Definite | DynamicBoundary | Clean | corresponding `Dynamic` if reason is representable |
| Definite | Unchecked | any | conservative `Unknown` |
| Maybe | any | any | `Unknown(MissingInitializer)` |
| Uninitialized | any | any | `Unknown(MissingInitializer)` |
| any | Validated/Assumed | non-clean | non-established recovery (`Unknown(SuppressedByInvalidCause)` is preferred) |

For an unknown/dynamic declaration contract, never invent a concrete contract type during lifecycle reduction.

- [ ] **Step 1: Add focused lifecycle reduction unit tests.**

Build synthetic `CallableAnalysis` or direct flow summaries only if test constructors are not excessively verbose. Otherwise expose a small pure reducer helper and test that.

Required cases:

```text
all definite + all validated -> Established contract
all definite + one assumed   -> Assumed contract
all definite + one refuted   -> non-known/refuted publication
one maybe/uninitialized      -> MissingInitializer
one causal-invalid exit      -> non-established publication
throw-only/no normal exits   -> defaults preserved
```

- [ ] **Step 2: Run lifecycle reducer tests to RED.**

```sh
cargo test -p phalcom-semantic --lib field_lifecycle -- --nocapture
```

Expected: current finalizer only understands initialization and therefore fails validity/authority cases.

- [ ] **Step 3: Extend `FieldLifecycleFact`.**

Add:

```rust
pub validity: FieldContractValidity,
pub causal_invalidity: CausalInvalidity,
```

Update `default_field_seeds()` constructions from Task 3 and `seed_flow_for_owner()`.

When seeding a body from a lifecycle fact, copy the validity and causal invalidity exactly into `FieldState`.

- [ ] **Step 4: Replace synthetic `constructor.exits.returns` consumption.**

After Part 1, lifecycle finalization must read:

```rust
constructor.exits.normal_returns.iter()
```

and use:

```rust
normal_return.flow.fields.get(&fact.field)
```

Do not reconstruct state from:

- `entry_flow`;
- final callable `ctx.flow`;
- a separate return-values vector;
- class defaults alone.

The normal-return flow snapshot is the proof boundary.

- [ ] **Step 5: Filter non-admissible constructor exits conservatively.**

Part 1's `NormalReturnFact` records `status` and `causal_invalidity`. Define an internal predicate with a narrow meaning such as:

```rust
fn normal_exit_can_certify_fields(exit: &NormalReturnFact) -> bool {
    exit.status.is_ready() && exit.causal_invalidity == CausalInvalidity::Clean
}
```

However, **do not drop invalid normal exits and then certify from the remaining subset**. An invalid normal path still means the constructor implementation cannot be universally certified.

Instead:

- include its field state in initialization reasoning where available;
- make resulting lifecycle causality non-clean / validity blocked for certification.

Only truly non-normal exits (throws, unreachable paths) are excluded.

- [ ] **Step 6: Implement initialization reduction.**

For every normal constructor exit:

- missing field summary counts as not definitely initialized;
- `DefinitelyInitialized` must appear on all exits for a definite result;
- if all are `Uninitialized`, result may stay `Uninitialized`;
- otherwise `MaybeInitialized`.

Do not rely on constructor count or default initialization alone.

- [ ] **Step 7: Implement validity reduction.**

Use `join_field_validity(...)` over every normal exit's field validity. If a field summary is absent, contribute `Unchecked`.

Then join causal invalidity over all corresponding exit field states plus exit-level causal invalidity.

- [ ] **Step 8: Add one helper that derives public read knowledge.**

Prefer a pure function:

```rust
fn lifecycle_read_knowledge(
    contract: &TypeKnowledge,
    initialization: FieldInitialization,
    validity: &FieldContractValidity,
    causal_invalidity: CausalInvalidity,
) -> TypeKnowledge
```

This concentrates the authority policy and makes it unit-testable.

For a known validated clean contract, construct established knowledge with `EvidenceOrigin::FieldLifecycle`.

For assumed, construct assumed knowledge with `EvidenceOrigin::FieldLifecycle`.

Never use `contract.clone()` directly if its source annotation status would obscure the lifecycle proof origin; preserve the type, select status from the reduction, and use `FieldLifecycle` as origin.

- [ ] **Step 9: Make the original wrong-write regression GREEN.**

Run:

```sh
cargo test -p phalcom-semantic --test semantic wrong_constructor_write_is_initialized_but_never_certifies_field_contract -- --nocapture
```

Expected:

- constructor exit field current is String;
- initialization is definite;
- validity is Refuted;
- final read is not Established<Int>;
- mismatch diagnostic remains.

- [ ] **Step 10: Add/green assumed lifecycle publication.**

For the constructor parameter case from Task 2, ordinary post-construction read must be no stronger than:

```text
Assumed<Number or declared field type, FieldLifecycle>
```

It must not become Established merely because all constructor paths assign something relation-compatible.

- [ ] **Step 11: Preserve established literal/default constructor behavior.**

Existing valid cases with compiler-established literals or otherwise established current facts must still establish the public field contract when all normal exits validate.

- [ ] **Step 12: Commit.**

```sh
git add phalcom-semantic/src/checker/field_lifecycle.rs \
        phalcom-semantic/src/session.rs \
        phalcom-semantic/tests/semantic/capabilities/fields.rs
git commit -m "fix(semantic): certify fields from valid constructor exits"
```

---

# Task 6: Publish lifecycle-aware field reads and propagate causal invalidity

**Files:**
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Modify: `phalcom-semantic/src/checker/body.rs` only if lifecycle attachment naming changes.
- Modify: `phalcom-semantic/src/session.rs` to attach finalized lifecycle to ordinary bodies.
- Test: `phalcom-semantic/tests/semantic/capabilities/fields.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/authority.rs`

**Interfaces:**
- Consumes: corrected `FieldLifecycleTable` and per-body `FieldState`.
- Produces:
  - current-object field reads with field causal invalidity;
  - ordinary receiver field reads that prefer finalized lifecycle publication when available;
  - no declaration-only upgrade when lifecycle cannot certify a field.

## 6.1 Current-object direct field reads

`Expr::Field { value: "_value" }` currently uses `resolve_current_field(...)` and returns a `TypedExpression` containing only knowledge.

Change the internal return value to a small fact:

```rust
pub(crate) struct FieldReadFact {
    pub field: FieldId,
    pub knowledge: TypeKnowledge,
    pub causal_invalidity: CausalInvalidity,
}
```

or return the relevant pieces as a tuple if a type would be used in only one place.

Prefer the struct if both direct and property reads share it.

- [ ] **Step 1: Add a field-read causality test.**

Construct a method/constructor where a mismatching assignment is followed by `_value` and inspect the expression analysis.

Assert:

```text
read knowledge retains recovery actual information if available
read.causal_invalidity != Clean
```

and, through Part 1, that this read cannot establish an unannotated caller's canonical return.

- [ ] **Step 2: Extend `resolve_current_field`.**

Read the `FieldState` from `ctx.flow` and return:

```text
knowledge = state.current
causal_invalidity = state.causal_invalidity
```

If no flow state exists, fall back to lifecycle publication before falling all the way back to declaration-only knowledge.

- [ ] **Step 3: Update `Expr::Field` synthesis.**

Construct:

```rust
let mut typed = TypedExpression::new(read.knowledge.with_range(*range));
typed.causal_invalidity = read.causal_invalidity;
typed
```

Do not convert invalid recovery knowledge into Unknown at expression synthesis time; Part 1's publication boundary handles admissibility. Keeping the recovery fact is useful for diagnostics and editor features.

## 6.2 Lifecycle attachment

At the grounding revision `CheckingContext` has `field_signatures` but not a stored lifecycle reference even though body requests receive one for seeding.

- [ ] **Step 4: Attach `FieldLifecycleTable` to checking context.**

Add:

```rust
field_lifecycle: Option<&'a FieldLifecycleTable>,
```

plus:

```rust
pub(crate) fn attach_field_lifecycle(&mut self, lifecycle: &'a FieldLifecycleTable)
```

or fold it into the existing body-context constructor if that is cleaner after Part 1.

Keep lifecycle immutable during one body analysis.

- [ ] **Step 5: Attach the correct table in `analyze_callable_body`.**

When a lifecycle table is supplied in `CallableBodyRequest`, both:

1. seed current flow appropriately;
2. attach it for non-current-object public field reads.

Constructor bodies receive the default table per Task 4; ordinary bodies receive finalized lifecycle.

## 6.3 Receiver property field reads

At the grounding revision `synthesize_get_property` asks `ctx.get_field(...)`, which returns declaration signature knowledge.

That means:

```phalcom
let x = counter._value
```

can observe the source annotation even when constructor lifecycle could not prove initialization.

- [ ] **Step 6: Add a lifecycle-aware external field read regression.**

Use a class with an uninitialized or refuted instance field and another class that reads it through an instance receiver if field visibility syntax permits it.

If `_value` is intentionally private and external access is invalid, use a public field declaration or whatever syntax the language currently supports. Do not alter visibility semantics in this task.

Assert the property field read is not Established declared type when lifecycle is uncertified.

- [ ] **Step 7: Add `resolve_field_read(...)`.**

Recommended policy:

```text
if matching current-flow field on current owner exists:
    use current field state
else if finalized lifecycle contains field:
    use lifecycle read_knowledge + lifecycle causal invalidity
else:
    use declaration signature projection
```

The final fallback remains necessary for:

- static/class fields (out of scope);
- declarations without lifecycle products;
- partial semantic states.

For an instance field whose lifecycle fact explicitly says unknown/missing initialization, **do not skip that fact and fall back to the source annotation**. Unknown lifecycle is meaningful negative proof availability.

- [ ] **Step 8: Update `synthesize_get_property`.**

Use `resolve_field_read(...)` instead of raw `get_field(...)` for field access.

Propagate causal invalidity from both receiver and field:

```rust
field_causal.join(recv_typed.causal_invalidity)
```

The receiver itself remains a required premise.

- [ ] **Step 9: Keep assignment expected-type lookup declaration-based.**

When choosing the expected type for assigning to a field, the persistent contract still comes from `FieldSemanticSignature.declared_type`, not from lifecycle read knowledge.

Do not accidentally use an unknown lifecycle read as “no field contract”.

This preserves the declaration/current split:

```text
write check -> declared contract
read fact   -> current/lifecycle knowledge
```

- [ ] **Step 10: Run focused tests.**

```sh
cargo test -p phalcom-semantic --test semantic fields -- --nocapture
cargo test -p phalcom-semantic --test semantic field_read -- --nocapture
cargo test -p phalcom-semantic --test semantic authority -- --nocapture
```

Expected: field reads now reflect construction proof instead of annotation existence, and invalid field causality reaches expression/callable publication.

- [ ] **Step 11: Commit.**

```sh
git add phalcom-semantic/src/checker/context.rs \
        phalcom-semantic/src/checker/expression.rs \
        phalcom-semantic/src/checker/body.rs \
        phalcom-semantic/src/session.rs \
        phalcom-semantic/tests/semantic/capabilities
git commit -m "fix(semantic): publish lifecycle-aware field reads"
```

---

# Task 7: Add comprehensive constructor/lifecycle composition regressions

**Files:**
- Modify: `phalcom-semantic/tests/semantic/capabilities/fields.rs`
- Modify: `phalcom-semantic/tests/semantic/capabilities/authority.rs`
- Optionally create: `phalcom-semantic/tests/semantic/integration/constructor_field_proofs.rs` if `fields.rs` becomes unwieldy.
- Modify test module registry if a new file is added.

**Interfaces:**
- Consumes: all Part-2 semantics so far.
- Produces: source-level regression matrix proving field initialization, validity, evidence authority, constructor exit handling, and callable publication compose correctly.

Do not substitute snapshots of rendered diagnostics for semantic assertions. The suite must inspect `TypeKnowledge`, field summaries, validity, and causal state directly.

- [ ] **Step 1: Add valid constructor certification.**

Source:

```phalcom
class Box {
  value: Int
  @constructor new() { value = 1 }
  read() { value }
}
```

Assert:

```text
constructor normal exit:
  value.current = Established<Int>
  initialization = DefinitelyInitialized
  validity = Validated

ordinary read:
  Established<Int, FieldLifecycle>
```

- [ ] **Step 2: Add assumed constructor certification.**

Source:

```phalcom
class Box {
  value: Number
  @constructor new(_ input: Int) { value = input }
  read() { value }
}
```

Because callable parameters are assumptions at body entry, assert:

```text
exit validity = Assumed
public lifecycle read status = Assumed
```

If later language semantics explicitly treat invocation parameter checks as runtime-established inside a body, update this case only when that semantic decision changes Part-1 authority rules. Do not locally strengthen it for convenience.

- [ ] **Step 3: Add wrong constructor write.**

Already introduced in Task 0/2. Final assertions:

```text
DefinitelyInitialized
Refuted
actual String preserved
FieldMismatch exists
public Int not established
```

- [ ] **Step 4: Add one-path-missing initialization.**

Use currently working branch syntax:

```phalcom
@constructor new(_ flag: Bool) {
  if flag { value = 1 }
}
```

Assert public read is Unknown(MissingInitializer) or the canonical equivalent.

Do not depend on `if let`; Part 3 owns its executable-region correction.

- [ ] **Step 5: Add both-branches-valid initialization.**

```phalcom
if flag {
  value = 1
} else {
  value = 2
}
```

Assert definite + validated if the branch condition/path evidence itself does not weaken the values.

- [ ] **Step 6: Add one-valid / one-refuted branch.**

```phalcom
if flag {
  value = 1
} else {
  value = "bad"
}
```

Assert:

```text
initialization = DefinitelyInitialized
validity = Refuted
public contract not established
```

This is the best regression for proving F1 and F4 compose.

- [ ] **Step 7: Add multiple constructor universal certification.**

Cases:

```text
constructor A valid + constructor B valid -> class field may certify
constructor A valid + constructor B missing -> MaybeInitialized / no certification
constructor A valid + constructor B refuted -> no certification
```

Run each with source order reversed or parameterize the fixture texts.

- [ ] **Step 8: Add throw-only path exclusion.**

Example:

```phalcom
@constructor new(_ fail: Bool) {
  if fail {
    throw Error.new()
  }
  value = 1
}
```

The throw branch should not be treated as a constructed-object normal exit. The remaining normal exit validates `value`.

If current branch/throw topology is not precise enough before Part 3 to prove this case, add the test as a guarded expected limitation only if the project convention supports capability gates. Do not distort field lifecycle to compensate for a control-flow bug owned by Part 3. Record the exact dependency in the plan execution log.

- [ ] **Step 9: Add return/field/call composition test.**

Use:

```phalcom
class Cell {
  value: Int
  @constructor new() { value = 1 }
  get() { value }
}

class Probe {
  @class
  run() {
    Cell.new().get()
  }
}
```

Assert the established field lifecycle can feed an established unannotated callable result through Part 1.

Then add the refuted variant and assert recovery field knowledge cannot establish the downstream callable result.

- [ ] **Step 10: Add a “declaration is not proof” test for uninitialized fields.**

```phalcom
class Cell {
  value: Int
  @constructor new() {}
  get() { value }
}
```

Assert `get()` does not publish Established<Int> merely because the field has an `Int` annotation.

- [ ] **Step 11: Run the matrix.**

```sh
cargo test -p phalcom-semantic --test semantic fields -- --nocapture
cargo test -p phalcom-semantic --test semantic constructor_field -- --nocapture
cargo test -p phalcom-semantic --test semantic authority -- --nocapture
```

Expected: all supported cases GREEN; any control-flow-gated case is explicitly attributed to Part 3 rather than silently weakened.

- [ ] **Step 12: Commit.**

```sh
git add phalcom-semantic/tests/semantic
git commit -m "test(semantic): cover constructor field proof composition"
```

---

# Task 8: Update deterministic fingerprints and incremental field-lifecycle stability

**Files:**
- Modify: `phalcom-semantic/src/db/fingerprint.rs`
- Modify: any lifecycle fingerprint/product module discovered after Part 1.
- Test: relevant `phalcom-semantic/tests/semantic/incremental/*`
- Test: add focused fingerprint unit tests if there is already a fingerprint test module.

**Interfaces:**
- Consumes: extended `FlowFieldSummary`, `FieldContractValidity`, `CausalInvalidity`, lifecycle publication.
- Produces: fingerprints where a validity/causal-only semantic change invalidates dependents, while identical lifecycle semantics remain reusable.

## Why this matters

A change can preserve field type text but alter its proof status:

```text
revision A: lifecycle Established<Int>
revision B: lifecycle Assumed<Int>
```

or:

```text
revision A: Validated
revision B: Refuted
```

Those are semantic changes even though `TypeId` remains `Int`.

- [ ] **Step 1: Audit current hashing.**

Run:

```sh
rg "FlowFieldSummary|FieldLifecycle|hash_flow_summary|hash_exit_facts" phalcom-semantic/src/db phalcom-semantic/src/session.rs
```

List every persisted/fingerprinted product carrying field proof state.

- [ ] **Step 2: Complete `hash_field_validity`.**

Use stable discriminants and hash contained reasons/obligations semantically:

```text
Unchecked        -> tag 0
Validated        -> tag 1
Assumed          -> tag 2
Refuted          -> tag 3
Blocked(reason)  -> tag 4 + hash_block_reason
DynamicBoundary  -> tag 5 + obligation.reason
```

Do not hash debug formatting.

- [ ] **Step 3: Hash field causal invalidity by shape, not local diagnostic ID.**

Use the existing `hash_causal_invalidity(...)` helper. This preserves semantic invalidity shape without making fingerprints depend on allocator identities.

- [ ] **Step 4: Add a fingerprint test for validity-only change.**

Construct two otherwise identical `FlowStateSummary`/`CallableAnalysis` products differing only in:

```text
Validated -> Assumed
```

Assert different product fingerprints.

Add a second case:

```text
same semantic validity/knowledge but different local cause ID
```

Assert fingerprints remain equal if causal *shape* is the same and cause identity is intentionally non-semantic.

- [ ] **Step 5: Add an incremental source regression.**

Use a session revision that changes a constructor from established literal initialization to an assumed source, or from valid to invalid while preserving the declared field type.

Assert dependent ordinary getter/caller products recompute when public field lifecycle authority changes.

Do not require unrelated callables to recompute.

- [ ] **Step 6: Add a no-op/trivia stability check if the existing incremental framework supports it.**

Changing formatting around a constructor without changing field lifecycle semantics should preserve product identity/fingerprint according to existing source-movement rules.

- [ ] **Step 7: Run incremental suite.**

```sh
cargo test -p phalcom-semantic --test semantic incremental -- --nocapture
cargo test -p phalcom-semantic db::fingerprint -- --nocapture
```

Expected: proof-status changes propagate; semantically identical lifecycle products remain stable.

- [ ] **Step 8: Commit.**

```sh
git add phalcom-semantic/src/db/fingerprint.rs \
        phalcom-semantic/tests/semantic/incremental \
        phalcom-semantic/src/session.rs
git commit -m "fix(semantic): fingerprint field proof state"
```

---

# Task 9: Audit every field proof creation site and close Part 2

**Files:**
- Audit: `phalcom-semantic/src/checker/field_lifecycle.rs`
- Audit: `phalcom-semantic/src/checker/flow/state.rs`
- Audit: `phalcom-semantic/src/checker/context.rs`
- Audit: `phalcom-semantic/src/checker/expression.rs`
- Audit: `phalcom-semantic/src/session.rs`
- Audit: `phalcom-semantic/src/db/fingerprint.rs`
- Test: entire `phalcom-semantic` crate and semantic suite.

**Interfaces:**
- Consumes: complete Part-2 implementation.
- Produces: no remaining field proof-strength laundering path in the instance-field lifecycle surface.

- [ ] **Step 1: Audit every `FieldState` construction site.**

```sh
rg "FieldState \{" phalcom-semantic/src phalcom-semantic/tests
```

For each site, verify all dimensions are intentional:

```text
contract
current
initialization
validity
causal_invalidity
version
```

No constructor should use `Validated` as a convenient default.

- [ ] **Step 2: Audit every established field-lifecycle construction.**

```sh
rg "FieldLifecycle|EvidenceOrigin::FieldLifecycle|TypeKnowledge::established" \
   phalcom-semantic/src/checker/field_lifecycle.rs \
   phalcom-semantic/src/checker/context.rs \
   phalcom-semantic/src/checker/expression.rs
```

For every `TypeKnowledge::established(..., EvidenceOrigin::FieldLifecycle)` answer:

> What universal constructor/default proof justifies established authority?

The only valid normal answer is the lifecycle reduction path with definite + validated + clean premises.

- [ ] **Step 3: Audit field relation consumers.**

```sh
rg "FieldMismatch|apply_assignability|check_knowledge_against_type" \
   phalcom-semantic/src/checker
```

Ensure no field path treats `RelationOutcome::Proven` as sufficient to upgrade actual assumed knowledge.

- [ ] **Step 4: Audit constructor lifecycle seed direction.**

```sh
rg "default_field_lifecycle|finalized_field_lifecycle|field_lifecycle" phalcom-semantic/src/session.rs
```

Manually verify:

```text
constructor input <- defaults only
ordinary body input <- finalized lifecycle
finalized lifecycle <- defaults + all independent constructor analyses
```

There must be no edge:

```text
finalized lifecycle -> constructor analysis
```

unless it refers exclusively to immutable declaration/default seed facts and cannot contain another constructor's result.

- [ ] **Step 5: Audit direct and receiver field reads.**

Confirm:

```text
assignment expected type -> declaration contract
current direct read       -> flow state
post-construction read    -> lifecycle publication
fallback declaration read -> only when no meaningful lifecycle product exists
```

An explicit lifecycle `Unknown(MissingInitializer)` is meaningful and must not be replaced with annotation-derived `Assumed<Int>` by fallback.

- [ ] **Step 6: Audit causal invalidity propagation.**

Trace a mismatching field write through:

```text
RHS TypedExpression
  -> relation diagnostic
  -> FieldState.causal_invalidity
  -> FlowFieldSummary.causal_invalidity
  -> FieldLifecycleFact.causal_invalidity
  -> field read TypedExpression
  -> Part-1 NormalReturnFact/publication
```

Add one test at any missing edge rather than patching without coverage.

- [ ] **Step 7: Format and lint.**

```sh
cargo fmt --all -- --check
cargo clippy -p phalcom-semantic --all-targets -- -D warnings
```

Expected: GREEN. If workspace policy does not enforce `-D warnings`, use the repository's canonical clippy command instead; do not suppress new warnings with broad allows.

- [ ] **Step 8: Run focused semantic suites.**

```sh
cargo test -p phalcom-semantic --test semantic fields -- --nocapture
cargo test -p phalcom-semantic --test semantic authority -- --nocapture
cargo test -p phalcom-semantic --test semantic callable_publication -- --nocapture
cargo test -p phalcom-semantic --test semantic incremental -- --nocapture
```

Expected: GREEN.

- [ ] **Step 9: Run the full crate tests.**

```sh
cargo test -p phalcom-semantic --all-targets
```

Expected: GREEN.

- [ ] **Step 10: Run the broader workspace gate used by the repository.**

Inspect CI/workspace scripts and run the current canonical workspace command. At minimum, if no narrower project command supersedes it:

```sh
cargo test --workspace --all-targets
```

Expected: no regression outside semantic checking.

- [ ] **Step 11: Record Part-3 dependencies rather than expanding scope.**

If remaining failures require any of the following, record them for Part 3/4/5 instead of implementing them here:

```text
if-let executable-region semantics
abrupt branch-value joins
predicate contradiction reachability
loop fixed points
continue/backedge topology
```

Part 2's responsibility is that whatever normal exit flow the checker supplies is reduced soundly; it does not own all control-flow precision.

- [ ] **Step 12: Final commit.**

```sh
git add phalcom-semantic
git commit -m "fix(semantic): complete sound instance field lifecycle proofs"
```

---

# 7. Detailed test matrix

The implementation is not complete until the following behaviors are represented in automated tests.

## 7.1 Single-path field writes

| Contract | Actual | Actual authority | Relation | Exit init | Exit validity | Public field |
|---|---|---|---|---|---|---|
| `Int` | `Int` | Established | Proven | Definite | Validated | Established `Int` |
| `Number` | `Int` | Established | Proven | Definite | Validated | Established `Number` |
| `Number` | `Int` | Assumed | Proven | Definite | Assumed | Assumed `Number` |
| `Int` | `String` | Established | Refuted | Definite | Refuted | not Established `Int` |
| `Int` | Unknown | — | Blocked | Definite if write occurs | Blocked | Unknown |
| `Int` | Dynamic | — | Dynamic | Definite if write occurs | DynamicBoundary | Dynamic/non-static |

## 7.2 Initialization topology

| Reachable normal exits | Result |
|---|---|
| all definite | Definite |
| definite + uninitialized | Maybe |
| definite + maybe | Maybe |
| all uninitialized | Uninitialized |
| valid normal + throw-only alternative | only normal construction exit contributes |
| no normal constructor exits | preserve defaults/no fabricated construction proof |

## 7.3 Validity topology

| Exit validities | Result |
|---|---|
| Validated + Validated | Validated |
| Validated + Assumed | Assumed |
| Assumed + Assumed | Assumed |
| any Refuted | Refuted |
| no refutation, any Blocked | Blocked |
| no refutation/block, any Dynamic | DynamicBoundary |
| any Unchecked with otherwise static proof | Unchecked |

## 7.4 Multiple constructors

Cover at least:

```text
A valid, B valid
A valid, B missing
A valid, B refuted
A assumed, B valid
A throw-only, B valid
```

and at least one source-order reversal.

## 7.5 Causal publication

Cover:

```text
invalid RHS
   ↓
field write recovery fact
   ↓
field read
   ↓
callable tail
```

Expected: recovery may remain queryable, but downstream callable public result cannot become Established through that chain.

---

# 8. Required explanation/diagnostic behavior

This plan is primarily semantic correctness work. Do not redesign rich diagnostic presentation, but preserve enough structured evidence for it.

## 8.1 Field mismatch

A wrong write must continue to emit one primary `FieldMismatch` diagnostic at the assignment/default initializer site.

Do not emit a second redundant “constructor failed field lifecycle” error solely because finalization observes the already-refuted field. Lifecycle finalization should consume the existing causal invalidity and produce semantic state; presentation may later use that evidence in richer diagnostics.

## 8.2 Missing initialization

If the existing system already emits a missing-field-initialization diagnostic, preserve it. If it does not, this plan may add a single specialized diagnostic only if lifecycle correctness otherwise has no user-facing explanation.

Do not broaden the task into a full constructor-definite-assignment diagnostic redesign.

## 8.3 Explanation graph

Where the explanation DAG already records field reads/writes, ensure the evidence status reflects the corrected field fact. Do not create an entire new explanation taxonomy in this plan.

A field read from an assumed lifecycle must not have an explanation node labeled `Established` merely because its type is known.

---

# 9. Non-goals and scope guards

Explicitly out of scope:

- class/static field initialization semantics;
- alias analysis (`other.field = ...` mutating the same object as `self`);
- ownership/borrowing/escape analysis;
- immutable-field write-once enforcement beyond existing behavior;
- inheritance initialization order and `super` constructor protocols unless an existing test proves this plan directly regresses them;
- lazy fields;
- thread/concurrency initialization;
- object-state typestates;
- dependent field invariants;
- general CFG/SSA rewrite;
- `if let`/loop correctness owned by later plans;
- presentation/LSP-specific field inference;
- advisory information as formal field evidence.

If a failing test requires one of these, record it as adjacent work rather than adding architecture to Part 2.

---

# 10. Acceptance criteria

Part 2 is complete only when all of the following are true.

## Data model

- [ ] `FieldState` separately stores persistent contract, actual current knowledge, initialization, contract validity, causal invalidity, and version.
- [ ] `FlowFieldSummary` preserves validity and causal invalidity at normal return boundaries.
- [ ] `FieldLifecycleFact` records lifecycle validity and causal invalidity in addition to public read knowledge.

## Field writes

- [ ] Every direct current-field write preserves RHS `TypeKnowledge` rather than replacing it with the declaration.
- [ ] A relation-proven assumed RHS yields assumed field validity.
- [ ] A refuted write is still definitely initialized on that executed path but is marked refuted.
- [ ] Field mismatch causal invalidity is retained in field state.

## Defaults

- [ ] Default initializers use the same proof law as ordinary field writes.
- [ ] Relation success alone never converts an assumed default into established contract knowledge.
- [ ] A wrong default does not establish its declared field type.

## Constructors

- [ ] Every constructor starts from declaration/default lifecycle seeds, independent of earlier constructors.
- [ ] Source order of constructors does not change lifecycle proof.
- [ ] Lifecycle finalization consumes actual Part-1 normal-exit flow snapshots.
- [ ] Throw/abrupt exits do not masquerade as normal constructed-object exits.

## Lifecycle publication

- [ ] Established public field knowledge requires definite initialization, validated contract validity, and clean causal state on every normal constructor exit.
- [ ] Assumed contributors cap lifecycle publication at Assumed.
- [ ] Refuted/blocked/missing/invalid exits cannot publish Established declared type.
- [ ] No normal constructors means defaults are preserved without invented proof.

## Reads

- [ ] Direct field reads use current flow facts and propagate field causal invalidity.
- [ ] Post-construction instance field reads use finalized lifecycle knowledge when available.
- [ ] An explicit lifecycle Unknown is not replaced by declaration-only assumed knowledge through fallback.
- [ ] Assignment checking still uses the persistent field declaration contract.

## Incremental semantics

- [ ] Validity/status/causal changes participate in semantic fingerprints.
- [ ] Local diagnostic cause IDs do not make semantically identical products unstable.
- [ ] Dependent bodies refresh when lifecycle authority changes.
- [ ] Constructor analyses do not self-depend on lifecycle products derived from themselves.

## Tests

- [ ] Low-level validity/reconciliation/join laws are covered.
- [ ] Wrong write, assumed write, default write, missing path, mixed branch, multiple constructor, and source-order cases are covered.
- [ ] At least one field→callable composition test proves Part 1 and Part 2 agree.
- [ ] Existing field, authority, callable publication, and incremental suites are GREEN.
- [ ] `cargo test -p phalcom-semantic --all-targets` is GREEN.

---

# 11. Handoff contract to Part 3

Part 3 — Canonical Control Outcomes and Executable Regions — may assume the following after this plan lands:

```text
FieldState is sound:
    initialization is not confused with validity
    actual current knowledge is preserved
    validity never strengthens its evidence premises
    causal invalidity is retained

Constructor lifecycle is sound relative to supplied normal exits:
    all constructors are independent
    actual return-site flow snapshots are consumed
    universal exit validity controls publication

Field reads are sound:
    current flow for current object
    lifecycle publication for constructed objects
    declaration contract remains separate
```

Part 3 can therefore change branch/executable-region topology without redesigning field proof semantics. Better control flow should merely produce more precise `FlowState`/`NormalReturnFact` inputs to the lifecycle reducer.

That separation is intentional:

```text
Part 2 answers:
    "Given these reachable constructor exits, what field fact is proven?"

Part 3 answers:
    "Which exits and branch states are actually reachable?"
```

The two concerns must not be collapsed again.

---

# 12. Final implementation principle

The field subsystem should end this plan following the same model as local bindings and callable returns:

```text
Declaration
    !=
Current value evidence
    !=
Relation success
    !=
Certified public fact
```

For fields specifically:

```text
field declaration contract
      +
actual write evidence
      +
relation result
      +
path initialization
      +
constructor normal-exit coverage
      +
causal validity
      ↓
field lifecycle publication
```

No one input is sufficient on its own.

The implementation should remain deliberately small: one validity dimension, one reconciliation function, one deterministic join, one lifecycle reduction, and corrected read/write plumbing through the structures that already exist. That is enough to make object construction and field facts trustworthy without turning Phalcom into a typestate or whole-program object-state analyzer.

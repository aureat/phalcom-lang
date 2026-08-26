# Phalcom Semantic Correctness Part 1 — RED Regression Test Handoff

## Purpose

This package contains an intentionally failing Rust regression suite for the remaining semantic-correctness defects identified in the Part 1 review. The tests are designed for **TDD integration**: add them to the canonical semantic test binary, prove that each one fails for the documented semantic reason, then repair production code without weakening the oracle.

The test file is grounded against `aureat/phalcom-lang` at:

```text
9b5873025b47dc7addb826f165530391fa93e171
Merge PR #4: fix semantic dispatch and test oracle corrections
```

The repository has reorganized its semantic tests. The canonical integration binary is now:

```text
phalcom-semantic/tests/semantic.rs
    -> semantic/mod.rs
       -> foundations/
       -> capabilities/
       -> incremental/
       -> ...
```

Therefore the supplied Rust file is intended to become:

```text
phalcom-semantic/tests/semantic/foundations/semantic_correctness_regressions.rs
```

and `phalcom-semantic/tests/semantic/foundations/mod.rs` should gain:

```rust
mod semantic_correctness_regressions;
```

Do **not** restore the old standalone test layout.

---

## Normative authority

Read these together before changing production code:

1. `docs/impl/semantic/phalcom_semantic_correctness_single_world_takeover_part1_formal_epistemic_foundation_spec.md`
2. `docs/impl/semantic/phalcom_semantic_correctness_part1_corrections_and_amendments.md`

The corrections/amendments override the WIP wherever they conflict.

The central laws exercised by this suite are:

```text
TypeKnowledge
AnalysisStatus
CausalInvalidity
```

are independent axes.

In particular, this is valid and required:

```text
Ready
+ Established(T)
+ CausalInvalidity::One(C)
```

A declaration is a persistent contract, not current value knowledge. A refuted contract does not overwrite the actual fact. Expected type is contextual judgment, not evidence. A relation's terminal states are not booleans. Flow joins and widening cannot choose arbitrary predecessor metadata. Generic failures must report the real failed judgment.

---

# Integration procedure

## 1. Add the test module only

Copy the supplied Rust file to:

```text
phalcom-semantic/tests/semantic/foundations/semantic_correctness_regressions.rs
```

Add to:

```text
phalcom-semantic/tests/semantic/foundations/mod.rs
```

the line:

```rust
mod semantic_correctness_regressions;
```

Do not change production code yet.

## 2. Run the focused RED suite

Use:

```bash
RUST_MIN_STACK=8388608 \
cargo test -p phalcom-semantic \
  --test semantic \
  semantic_correctness_regressions \
  -- --nocapture
```

Every failing test must be inspected. A useful RED failure is an assertion failure demonstrating the semantic defect described below.

A parser failure, missing test import, API typo, or unrelated fixture error is **not** a valid RED state. Repair test integration until the test reaches production semantics and fails on the intended oracle.

To isolate one case:

```bash
RUST_MIN_STACK=8388608 \
cargo test -p phalcom-semantic \
  --test semantic \
  semantic_correctness_regressions::causal_invalidity_does_not_suppress_analyzable_downstream_dispatch \
  -- --exact --nocapture
```

## 3. Fix one semantic law at a time

Recommended sequence:

```text
1. causal invalidity / expression status orthogonality
2. return-summary knowledge preservation
3. relation terminal-outcome propagation
4. generic actual-conflict evidence
5. flow widening reconciliation
6. callable parameter provenance
7. constructor result provenance
8. explicit-vs-inferred binding contract publication
9. deterministic unknown-reason joins
```

After each fix, rerun the individual test, then the entire new module.

## 4. Run the canonical semantic target

After the focused module is green:

```bash
RUST_MIN_STACK=8388608 \
cargo test -p phalcom-semantic --test semantic
```

Then:

```bash
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic
cargo fmt --all -- --check
```

Run the repository-standard LSP/workspace compatibility gates after the semantic changes are stable.

---

# Test-by-test contract and expected current failure

## 1. `causal_invalidity_does_not_suppress_analyzable_downstream_dispatch`

Source shape:

```phalcom
let x: Int = CellNum.new()
let y = x.cellOnly()
```

Required semantic product:

```text
x:
    contract     = Int / SourceAnnotation
    current      = Established(CellNum)
    consistency  = Refuted(CellNum <: Int)
    invalidity   = One(C1)

read/use of x:
    current      = Established(CellNum)
    causal       = One(C1)

x.cellOnly():
    knowledge    = Established(Int)
    status       = Ready
    causal       = One(C1)
```

There must be exactly one binding-initializer mismatch.

### Why current HEAD should fail

`checker/expression.rs::analyze_expression()` currently derives status with:

```rust
if owned_cause {
    Invalid(...)
} else if let Some(cause) = typed.causal_invalidity.suppression_cause() {
    Suppressed(cause)
} else {
    Ready
}
```

A variable read correctly inherits `BindingState.causal_invalidity`, but the wrapper then translates that non-clean causal state into `Suppressed`.

### Production seam

Primary:

```text
phalcom-semantic/src/checker/typed_expr.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/context.rs
```

The durable repair is to carry first-class analysis status through the expression transfer layer, not to infer status from causal invalidity after the fact.

Do not "fix" this by clearing causal invalidity. The downstream expression must remain:

```text
Ready + Established(Int) + One(C1)
```

---

## 2. `normal_return_summary_preserves_dynamic_reason`

Required:

```text
Dynamic(RuntimeReflection)
    -> Dynamic(RuntimeReflection)
```

### Why current HEAD should fail

`checker/analysis.rs::normal_return_summary()` currently performs a correct knowledge join, then replaces every joined result without a concrete `TypeId` with:

```text
Unknown(UncheckedExpression)
```

`Dynamic` has no `TypeId`, so a deliberate dynamic boundary is falsely reported as a checker coverage gap.

### Production seam

```text
phalcom-semantic/src/checker/analysis.rs
```

The likely repair is simply to preserve the joined `TypeKnowledge`; do not special-case `ty().is_none()` as coverage failure.

---

## 3. `normal_return_summary_preserves_existing_unknown_reason`

Required:

```text
Unknown(UnresolvedName("missing"))
    -> Unknown(UnresolvedName("missing"))
```

The same current code incorrectly converts it to:

```text
Unknown(UncheckedExpression)
```

This test exists separately from the Dynamic case because the two regressions protect different semantic laws:

```text
Dynamic != Unknown
actual Unknown reason != checker-coverage reason
```

---

## 4. `knowledge_join_unknown_reason_is_order_independent`

Required law:

```text
join(A, B) == join(B, A)
```

for reachable `Unknown` states.

### Why current HEAD should fail

`types/evidence.rs::join_type_knowledge()` currently uses `find_map()` and returns the first `UnknownReason` encountered. Swapping predecessor order therefore changes the published semantic reason.

### Production seam

```text
phalcom-semantic/src/types/evidence.rs
```

Choose a deterministic conservative merged reason. Do not preserve "first predecessor wins" under a different iterator.

A compact dedicated flow-merge reason is preferable to constructing unbounded recursive reason trees if no existing deterministic priority rule is semantically correct.

---

## 5. `generic_conflict_reports_actual_failed_upper_bound`

This is intentionally a solver-level adversarial test.

Constraints:

```text
Int <: T
T <: Number
T <: String
```

with hierarchy:

```text
Int <: Number
Int </: String
```

The actual conflict is therefore:

```text
lower = Int
upper = String
```

### Why current HEAD should fail

The solver checks all upper bounds and discovers the failed one, but final conflict construction currently records:

```rust
upper: uppers[0]
```

Thus a valid first upper bound (`Number`) is falsely published as the failed evidence.

### Production seam

```text
phalcom-semantic/src/checker/inference.rs
InferenceSession::solve()
```

Retain the actual `upper` at the point the subtype judgment fails. If origin metadata for final reconciled bounds is later made more precise, preserve that as well. Never synthesize a different bound merely because it is convenient to index.

This test complements the already-existing real-conflict/origin tests; it covers the specific "later upper fails" case they do not.

---

## 6. `callable_parameter_body_entry_uses_signature_assumption_provenance`

Required parameter body-entry state:

```text
contract:
    Int / BindingContractOrigin::CallableParameter

current:
    Int
    EvidenceStatus::Assumed
    EvidenceOrigin::CallableSignature

consistency:
    Assumed {
        basis: CallableParameterContract
    }
```

### Why current HEAD should fail

`body.rs` passes the signature's existing `param.ty.clone()` directly to:

```text
bind_callable_parameter(...)
```

and that helper retains the incoming current `TypeKnowledge` unchanged. The contract origin is corrected to `CallableParameter`, but the value evidence is not re-derived as a callable-entry assumption.

`binding.rs::assumption_basis()` only produces `CallableParameterContract` when the current origin is `CallableSignature`; otherwise it falls back to `DerivedEvidence(origin)`.

### Production seam

```text
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/binding.rs
```

The source annotation and the body-entry premise are different semantic derivations. Normalize at body entry.

---

## 7. `constructor_result_uses_constructor_semantics_origin`

Required:

```text
CellNum.new():
    Established(CellNum)
    EvidenceOrigin::ConstructorSemantics
```

and a binding initialized from it should preserve that origin unless a genuinely new derivation changes it.

### Why current HEAD should fail

`checker/call.rs::promote_exact_return()` currently promotes every exact concrete return as:

```rust
EvidenceOrigin::CallableSignature
```

regardless of callable category.

### Production seam

```text
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/expression.rs
resolved callable metadata / dispatch
```

The call-result promotion helper needs enough resolved callable information to select:

```text
constructor   -> ConstructorSemantics
ordinary      -> CallableSignature
trusted native-> NativeSignature
generic       -> GenericInference
```

Do not infer constructor-ness from spelling alone if canonical callable identity already knows it.

### Existing test oracle that must be updated

Current:

```text
phalcom-semantic/tests/semantic/capabilities/authority.rs
```

contains constructor assertions that expect `CallableSignature`. That oracle conflicts with the Part 1 normative origin table. When the production fix lands, update those existing assertions rather than treating their current green status as authority.

---

## 8. `inferred_initializer_contract_is_not_published_as_explicit_declaration`

Required for:

```phalcom
let x = 1
```

is:

```text
contract = Int / InferredInitializer
current  = Established(Int)
explicit declared type = None
```

### Why current HEAD should fail

`BindingState::from_seed()` currently computes:

```rust
let declared = seed.contract.as_ref().map(|contract| contract.ty);
```

This makes *every* contract—including `InferredInitializer`, callable parameter and contextual contracts—look like an explicit declaration through the legacy field.

### Production seam

```text
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/context.rs
publication/presentation compatibility consumers
```

Recommended direction:

```text
contract_type()
explicit_declared_type()
inferred_contract_type()
```

and retire the ambiguous mirror. If the field must survive temporarily, it must mean only source-authored local declaration, not "any persistent contract".

Do not remove the inferred contract itself. Monomorphic reassignment policy is intentionally preserved.

---

## 9. `loop_widening_reconciles_joined_current_against_persistent_contract`

The fixture creates:

```text
contract = Number

header:
    current = Int
    consistency = Validated

next:
    current = String
    consistency = Refuted(String <: Number)
```

Widened current is:

```text
Int | String
```

and its consistency must be the actual relation:

```text
Refuted((Int | String) <: Number)
```

### Why current HEAD should fail

`FlowState::widen_loop_state()` joins the knowledge but, when predecessor consistency labels differ, replaces the semantic relation with:

```text
Blocked(RecursiveFixpoint)
```

without re-running reconciliation against the persistent contract.

### Production seam

```text
phalcom-semantic/src/checker/flow/state.rs
```

The correct shape likely requires hierarchy/relation context at widening, just as `join_with_hierarchy()` already accepts it. Do not preserve the current no-hierarchy API merely to avoid changing callers if that API cannot compute the required semantic result.

---

## 10. `relation_policy_does_not_report_terminal_states_as_success`

This is a **transitional red test**, because the current API is itself the defect.

It checks:

```text
Blocked
Cancelled
BudgetExceeded
InternalFailure
```

and rejects the current behavior where all non-`Refuted` outcomes return boolean `true`.

### Required end-state

Do not stabilize the test around `false`. The normative production API should return/preserve the actual relation outcome. After the refactor, rewrite this test to assert exact variant preservation.

Target mapping:

```text
Proven           -> Ready / continue
Refuted          -> Invalid + owning diagnostic
Blocked          -> Blocked
DynamicBoundary  -> DynamicBoundary
Cancelled        -> Cancelled
BudgetExceeded   -> BudgetExceeded
InternalFailure  -> InternalFailure
```

### Production seam

```text
phalcom-semantic/src/checker/policy.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/body.rs
call/argument/return checking sites
```

Current functions such as:

```text
handle_relation_outcome(...) -> bool
enforce_assignability(...) -> bool
enforce_knowledge_against_type... -> bool
```

must not remain semantic state-decision APIs.

---

## 11. `expected_contract_blockage_reaches_expression_status`

Source:

```phalcom
let x: Int = missing
```

Required:

```text
missing.knowledge
    = Unknown(UnresolvedName(...))

missing.status
    = Blocked(UnknownType(UnresolvedName(...)))

x.current
    = same Unknown

x.consistency
    = Blocked(UnknownType(...))
```

The `Int` contract remains context. It does not become evidence and does not turn the unresolved expression into an assumption.

### Why current HEAD should fail

The let initializer passes expected context into `analyze_expression()`, but the expression wrapper currently produces `Ready` whenever there is no owned diagnostic, no causal invalidity and no dynamic knowledge. The blocked relation is not represented on `TypedExpression`, so it cannot survive transfer into the published `ExpressionAnalysis`.

### Production seam

```text
phalcom-semantic/src/checker/typed_expr.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/context.rs
```

This is another reason the durable fix is first-class `AnalysisStatus` on the typed transfer result rather than post-hoc status reconstruction.

---

# Existing tests with semantically wrong or incomplete oracles

Do not merely add the new file and leave contradictory tests untouched.

## Constructor provenance oracle

File:

```text
phalcom-semantic/tests/semantic/capabilities/authority.rs
```

The current helper/test expects constructor calls and constructor-initialized bindings to have:

```text
EvidenceOrigin::CallableSignature
```

Normative Part 1 requires:

```text
EvidenceOrigin::ConstructorSemantics
```

Update the old test when the implementation changes.

## Divergent same-BindingId contract oracle

File:

```text
phalcom-semantic/tests/semantic/foundations/flow_graph.rs
```

Current test:

```text
divergent_branch_contracts_fail_closed_without_first_branch_metadata
```

expects:

```text
contract = None
declared = None
consistency = Blocked(RecursiveFixpoint)
```

This is **not** the normative invariant.

Different persistent contracts for the same `BindingId` are an internal semantic invariant violation. The checker must fail closed without converting "contradictory impossible state" into "unconstrained binding".

Before changing this test, choose the production representation:

```text
Result<FlowState, FlowInvariantFailure>
```

or an equivalent explicit internal-failure carrier is preferable.

A debug assertion alone is insufficient if release behavior can silently erase the contract and continue.

Do not add an arbitrary "pick one contract" policy.

---

# Follow-on tests that require an API decision first

These were intentionally **not** forced into the supplied Rust file because writing them against today's representation would hard-code the wrong abstraction.

## A. First-class `TypedExpression::status`

Current `TypedExpression` has `knowledge` and `causal_invalidity` but no `AnalysisStatus`.

After adding status, add constructor/transfer-law tests for all legal combinations:

```text
Ready + Established + Clean
Ready + Established + One(C)
Invalid(C) + Established + One(C)
Suppressed(One(C)) + Unknown(SuppressedByInvalidCause) + One(C)
Blocked + Unknown + Clean
DynamicBoundary + Dynamic + Clean
Cancelled
BudgetExceeded
InternalFailure
```

Negative laws:

```text
causal invalidity does not imply Suppressed
Invalid does not imply Unknown
Known does not imply Ready
Dynamic does not imply Unknown
```

## B. Full relation-outcome propagation

Once the boolean adapters are removed, make the test table exhaustive over:

```text
Assignable / Proven
Refuted
DynamicBoundary
Blocked
Cancelled
BudgetExceeded
InternalFailure
Uncertain (if retained at Assignability level)
```

Assert exact `AnalysisStatus` mapping and unchanged actual `TypeKnowledge`.

## C. Binding reconciliation infrastructure terminals

Current `BindingConsistency` has no variants for `Cancelled`, `BudgetExceeded` or `InternalFailure`. Do not disguise these as unrelated `BlockReason`s.

Choose a richer reconciliation result, then directly test:

```text
Cancelled       remains Cancelled
BudgetExceeded  remains BudgetExceeded
InternalFailure remains InternalFailure
```

## D. Same-ID conflicting contracts

After choosing the explicit flow-invariant failure representation, add the regression that constructs two reachable states with the same `BindingId` but different contracts and asserts the explicit invariant failure.

The current `contract = None` behavior must disappear.

## E. Epistemic `FlowStateSummary`

Current summary is:

```rust
BindingId -> TypeId
fact_count
```

It cannot distinguish:

```text
Established(Int)
Assumed(Int)
Dynamic(...)
Unknown(...)
```

Change the summary representation or remove it from semantic identity. Then add a paired test proving:

```text
Established(Int) -> Assumed(Int)
```

changes the semantic product/fingerprint.

Keep the already-existing cause-ID renumbering control green:

```text
One(DiagnosticCauseId(17))
One(DiagnosticCauseId(93))
```

must have the same semantic product identity when all semantic shape is equal.

But:

```text
One -> Multiple
```

must differ.

## F. `TypeEvidence` construction privacy

This is an API invariant, not a runtime behavior test.

Current public fields allow arbitrary callers to manufacture:

```text
Established + any origin
```

Make fields private or `pub(crate)` and expose controlled constructors/getters.

If the repository already has compile-fail infrastructure, add a compile-fail API test. Do not add a new test framework solely for this one check.

## G. Generic missing-variable metadata

There are still internal `unwrap_or(KindId::TYPE)` fallbacks during variable-variable inference operations.

Because a missing solver variable record is an internal-corruption state rather than valid source input, test it at the module-private/unit seam after the representation for internal solver failure is chosen. It should never silently become `Type`.

## H. Generic `Cancelled` / `BudgetExceeded` through call semantics

`InferenceOutcome` contains these variants, but `CallCheckResult` currently has no `AnalysisStatus`.

Once call checking carries status, inject/produce terminal inference outcomes and assert:

```text
Cancelled      -> AnalysisStatus::Cancelled
BudgetExceeded -> AnalysisStatus::BudgetExceeded(...)
```

A fixed inference-independent return may retain its independently known `TypeKnowledge`; status remains orthogonal.

---

# Do not make these "fixes"

The following changes can make individual assertions green while violating the model:

```text
BAD: clear CausalInvalidity so downstream expression becomes Ready
BAD: turn every causally invalid expression into Unknown
BAD: make Blocked/Cancelled return false and keep bool as the semantic API
BAD: replace all Unknown joins with one arbitrary hard-coded reason without a stated law
BAD: delete inferred binding contracts to make `declared` disappear
BAD: keep loop widening without hierarchy and guess consistency
BAD: change constructor tests to keep CallableSignature because old tests already expect it
BAD: make a generic conflict always report the last upper rather than the actual failed judgment
BAD: hash raw DiagnosticCauseId to make fingerprint changes visible
BAD: use `Unit`, `Never`, `Object`, or `Type` as missing-information sentinels
```

---

# Suggested production repair slices

## Slice 1 — expression status becomes a transfer result

Expected files:

```text
phalcom-semantic/src/checker/typed_expr.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/call.rs
```

Goal:

```text
knowledge
status
causal invalidity
```

travel together but remain orthogonal.

This should resolve the causal suppression test and provide the carrier needed for terminal relation/call outcomes.

## Slice 2 — eliminate semantic bool adapters

Expected files:

```text
phalcom-semantic/src/checker/policy.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/body.rs
```

Return full `Assignability` / relation state after diagnostic policy.

Only `Refuted` owns mismatch diagnostics.

## Slice 3 — repair knowledge aggregators

Expected files:

```text
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/types/evidence.rs
```

Preserve Dynamic/Unknown reasons and deterministic flow semantics.

## Slice 4 — inference conflict fidelity

Expected file:

```text
phalcom-semantic/src/checker/inference.rs
```

Carry the actual failed upper through final reconciliation.

Do not redesign the solver.

## Slice 5 — flow invariants and widening

Expected file:

```text
phalcom-semantic/src/checker/flow/state.rs
```

Reconcile widened current facts with the persistent contract, and introduce an explicit same-ID contract-invariant failure rather than erasing the contract.

## Slice 6 — provenance and contract publication

Expected files:

```text
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/analysis.rs
```

Normalize callable body-entry assumptions; promote constructor/native/ordinary/generic results with the correct origin; retire the ambiguous `declared` mirror.

---

# Completion criteria for the agent

Do not report this repair complete merely because the new file is green.

Minimum:

```bash
RUST_MIN_STACK=8388608 \
cargo test -p phalcom-semantic \
  --test semantic \
  semantic_correctness_regressions \
  -- --nocapture

RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test semantic
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic
cargo fmt --all -- --check
```

Then re-run the Part 1 audit/release gate from the WIP plus amendment items 32–42.

Specifically verify that no old test oracle was made "green" by preserving one of the defects above.

---

# Package verification status

This handoff was written from the live GitHub source at commit:

```text
9b5873025b47dc7addb826f165530391fa93e171
```

The supplied Rust file was designed against the APIs present at that commit.

The generation environment does not contain a working local checkout of `aureat/phalcom-lang`, so the full Cargo RED run could not be executed here. The integrating agent must perform the required RED verification in the repository before touching production code. That is a deliberate TDD gate, not an optional cleanup step.

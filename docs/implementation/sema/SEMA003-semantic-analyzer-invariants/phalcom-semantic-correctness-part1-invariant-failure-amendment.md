# Phalcom Semantic Correctness Part 1
## Amendment — Internal Semantic Invariant Failure Mechanics

> **Status:** Normative implementation amendment to
> `phalcom-semantic-correctness-part1-bugfix-implementation-spec-REVISED.md`
>
> **Target repository:** `aureat/phalcom-lang`
>
> **Verified baseline inherited from parent plan:** `main` at
> `9b5873025b47dc7addb826f165530391fa93e171`
>
> **Scope:** This amendment ratifies and specifies the mechanics for impossible analyzer states. It does **not** reopen the other decisions in the revised bugfix plan. The relation-outcome, suppression, constructor-provenance, budget/cancellation, evidence, flow, generic-inference, and fingerprint semantics in the parent plan remain in force except where this amendment explicitly refines invariant-failure handling.

---

# 1. Amendment purpose

The revised Part 1 repair specification already identifies several states that must never be interpreted as ordinary user program facts. The clearest current example is a flow join in which the same `BindingId` arrives with incompatible declaration-stable metadata:

```text
predecessor A:
    BindingId(7)
    contract = Int

predecessor B:
    BindingId(7)
    contract = String
```

This state cannot be explained by normal flow-sensitive typing. A binding's **current value knowledge** may differ between predecessors, but its persistent declaration contract cannot spontaneously change while retaining the same semantic identity.

The parent repair plan correctly requires the flow join to stop erasing such disagreement into `contract = None`. This amendment completes that design by defining how Phalcom:

1. represents analyzer invariant violations structurally;
2. distinguishes them from user type errors and ordinary analysis incompleteness;
3. records enough forensic context for compiler developers;
4. contains the failure at the smallest safe semantic boundary for end users;
5. fails loudly in tests, CI, and developer builds;
6. guarantees that invariant failures cannot be silently converted into `Unknown`, `Blocked`, `TypeMismatch`, or weaker binding state;
7. preserves incremental and fingerprint correctness when an invariant failure occurs.

The governing rule is:

> **Impossible analyzer states are internal semantic invariant failures. They are never user type errors, never ordinary uncertainty, and never permission to weaken the user's semantic constraints.**

---

# 2. Ratified semantic policy

This amendment ratifies the following policy.

## 2.1 Three distinct failure domains

Phalcom semantic analysis distinguishes three fundamentally different domains.

### Domain A — user semantic invalidity

The analyzer successfully performs a judgment and proves the program violates a language rule.

Example:

```phalcom
let x: Int = 1

if condition {
    x = "hello"
}
```

The declaration-stable contract is still:

```text
x.contract = Int
```

The branch current facts are:

```text
entry:
    current = Int

then branch:
    current = String
```

The relevant semantic relation is:

```text
String <: Int
```

and the checker can prove it is false.

Result:

```text
RelationOutcome::Refuted(...)
    ↓
AssignmentMismatch diagnostic
    ↓
AnalysisStatus::Invalid(...)
```

This is a normal user-facing type error.

### Domain B — honest analysis incompleteness

The analyzer cannot complete a semantic judgment, but there is no evidence that its internal invariants are broken.

Representative outcomes:

```text
Unknown(...)
Blocked(...)
DynamicBoundary(...)
Cancelled
BudgetExceeded(...)
Suppressed(...)
```

These are legitimate analyzer states.

They must remain distinct and must not be converted into success or refutation merely because the checker wants to continue.

### Domain C — analyzer invariant failure

The analyzer reaches a state that should be impossible if its own semantic bookkeeping is correct.

Examples include:

```text
same BindingId + incompatible persistent contracts
same BindingId + incompatible mutability
same inference variable identity + contradictory intrinsic kind metadata
same expression identity + incompatible owning body
binding identity resolves to two incompatible declarations
semantic product structurally violates an invariant promised by its constructor
```

These are not properties of the Phalcom program.

They are bugs in the analyzer.

Result:

```text
InvariantFailure
    ↓
InternalSemanticIncident
    ↓
AnalysisStatus::InternalFailure(...)
    ↓
abort/contain the smallest unsafe semantic computation
```

No ordinary type mismatch diagnostic is emitted.

---

# 3. User error versus invariant failure: canonical examples

This distinction must be made explicit in code review and tests because the same surface program may contain multiple current types without violating analyzer invariants.

## 3.1 Valid analyzer state with incompatible current facts

Source:

```phalcom
let x: Int = 1

if condition {
    x = 2
} else {
    x = "hello"
}
```

Conceptual flow:

```text
binding identity:
    BindingId(7)

persistent contract:
    Int

then:
    current = Established(Int)
    contract = Int

else:
    current = Established(String)
    contract = Int
    consistency = Refuted(String <: Int)
```

At merge:

```text
contract = Int
current  = join(Int, String)
```

If the joined current proposition is `Int | String`, reconciliation against `Int` is refuted.

This is a user semantic error.

It must **not** produce an internal incident.

## 3.2 Impossible analyzer state

Suppose the analyzer somehow constructs:

```text
then:
    BindingId(7)
    contract = Int

else:
    BindingId(7)
    contract = String
```

There is no legal flow transfer that changes the persistent contract of `BindingId(7)` from `Int` to `String`.

The likely causes are implementation defects such as:

- incorrect `BindingId` reuse;
- shadowed declarations receiving the same identity;
- declaration metadata being mutated by flow transfer;
- incompatible flow states being merged;
- stale cached state being attached to the wrong binding identity.

The join must produce a structured internal failure.

It must not produce:

```text
contract = None
```

because `None` means "this binding has no persistent contract", which weakens the source program.

It must not produce:

```text
TypeMismatch(Int, String)
```

because the developer did not create a relation between those two declaration contracts.

It must not produce:

```text
Unknown(...)
```

because the problem is not missing knowledge. The analyzer knows that its own invariants have been violated.

---

# 4. Core design principle: semantic detection and operational policy are separate

The semantic subsystem should never embed the development-vs-release choice directly into the flow algorithm.

Incorrect design:

```rust
if contracts_disagree {
    if cfg!(debug_assertions) {
        panic!("...");
    } else {
        return FlowState::empty();
    }
}
```

This mixes:

```text
semantic detection
```

with:

```text
process failure policy
```

and usually causes the release path to invent a weaker semantic state.

Instead:

```text
semantic operation
    ↓
structured invariant failure
    ↓
incident recording
    ↓
semantic containment
    ↓
environment policy
        ├── developer/test/CI: fail loudly
        └── release/LSP: contain and remain operational
```

The semantic result is identical in every build:

```text
InternalFailure
```

Only the outer operational reaction differs.

---

# 5. New internal model

The implementation should introduce two related but distinct concepts:

1. a **local invariant failure value** returned by the semantic operation that detected the impossible state;
2. a **recorded internal semantic incident** attached to the containing semantic analysis product/session.

These should not be collapsed into one type because they serve different lifetimes.

---

# 6. Local invariant-failure types

## 6.1 Flow-specific failure type

Modify:

`phalcom-semantic/src/checker/flow/state.rs`

Introduce:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FlowInvariantFailure {
    DivergentBindingContract {
        binding: BindingId,
        left: BindingContract,
        right: BindingContract,
    },

    DivergentMutability {
        binding: BindingId,
        left: bool,
        right: bool,
    },
}
```

The exact field representation may be made more compact if cloning full contracts is expensive. The semantic requirements are:

- binding identity is retained;
- the disagreement category is retained;
- enough old/new/incoming metadata is retained to explain the bug;
- source-origin metadata is retained when already cheaply available.

Do not allocate large vectors on the normal path. Rich predecessor snapshots may be collected only on failure.

## 6.2 Future extension

Do not create one giant `SemanticInvariantFailure` enum immediately if the only current producer is flow.

Prefer subsystem-specific failure types:

```text
FlowInvariantFailure
InferenceInvariantFailure
IdentityInvariantFailure
DatabaseInvariantFailure
```

that can later be normalized into the shared incident representation.

This keeps detection close to the subsystem that knows what the violated invariant means.

---

# 7. `FlowState::join_with_hierarchy` becomes fallible

Current:

```rust
pub fn join_with_hierarchy(...) -> FlowState
```

Target:

```rust
pub fn join_with_hierarchy(...)
    -> Result<FlowState, FlowInvariantFailure>
```

## 7.1 Contract disagreement

For each common reachable `BindingId`:

```text
incoming contracts identical
    -> continue

incoming contracts differ
    -> Err(DivergentBindingContract)
```

Do not:

- erase to `None`;
- choose the first predecessor;
- choose the most general type;
- construct a union contract;
- mark `Unconstrained`;
- downgrade to `Blocked`;
- emit a user diagnostic.

A persistent binding contract is declaration-stable.

## 7.2 Mutability disagreement

For the same `BindingId`:

```text
incoming mutability identical
    -> continue

incoming mutability differs
    -> Err(DivergentMutability)
```

Current behavior that effectively collapses disagreement by taking an `all(...)` result is not semantically valid.

Mutability is declaration metadata, not flow-sensitive value knowledge.

## 7.3 Current value disagreement remains normal

This remains legal:

```text
same BindingId
same persistent contract
same mutability

predecessor A current = Int
predecessor B current = String
```

The normal type-knowledge join and reconciliation rules apply.

No internal incident is created merely because current flow facts differ.

---

# 8. Loop widening must obey the same invariant contract

The parent plan already requires widening to re-reconcile widened current knowledge.

This amendment adds:

> Widening is not allowed to repair declaration-stable metadata disagreement.

Target API:

```rust
pub fn widen_loop_state_with_hierarchy(...)
    -> Result<FlowState, FlowInvariantFailure>
```

or an equivalent fallible API.

Before/current during widening, validate:

```text
same BindingId
    contract equality
    mutability equality
```

If either invariant fails, return the appropriate `FlowInvariantFailure`.

Then, only for valid metadata:

1. join/widen current knowledge;
2. join denotation conservatively;
3. join causal invalidity;
4. reconcile widened current knowledge against the persistent contract;
5. store recomputed consistency.

---

# 9. Shared incident representation

Add a checker/session-level internal incident model in a location that is accessible to callable analysis and snapshot/session aggregation.

Recommended conceptual shape:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InternalSemanticIncident {
    pub id: InternalSemanticIncidentId,
    pub kind: InternalSemanticIncidentKind,
    pub module: ModuleId,
    pub callable: Option<CallableId>,
    pub expression: Option<ExpressionId>,
    pub range: Option<SourceRange>,
    pub details: InternalSemanticIncidentDetails,
}
```

Representative kinds:

```rust
pub enum InternalSemanticIncidentKind {
    FlowInvariantViolation,
    RelationInvariantViolation,
    InferenceInvariantViolation,
    IdentityInvariantViolation,
    DatabaseInvariantViolation,
}
```

For the immediate implementation:

```rust
pub enum InternalSemanticIncidentDetails {
    DivergentBindingContract {
        binding: BindingId,
        left: BindingContractSummary,
        right: BindingContractSummary,
    },

    DivergentMutability {
        binding: BindingId,
        left: bool,
        right: bool,
    },
}
```

Exact enum factoring is implementation detail.

The semantic requirements are:

- every incident has a stable identity within one semantic analysis generation;
- the incident identifies the affected semantic owner where known;
- the incident records the invariant category;
- it contains enough information to debug the analyzer;
- it is not represented as a `SemanticDiagnostic` blaming the source program.

---

# 10. Internal incident identity

Add:

```rust
InternalSemanticIncidentId
```

or use an existing incident-ID abstraction if one is already appropriate.

Like `DiagnosticCauseId`, this ID is local allocation identity.

It is useful for:

```text
AnalysisStatus::InternalFailure(id)
callable status -> incident linkage
debug reports
tests
```

Raw numeric incident IDs must not become semantic fingerprint identity across equivalent analyses.

Fingerprint rules should use incident **kind/shape/semantic details**, not allocator numbering.

---

# 11. Checker context owns incident recording during body analysis

Modify:

`phalcom-semantic/src/checker/context.rs`

Add storage equivalent to:

```rust
pub internal_incidents: Vec<InternalSemanticIncident>
pub next_internal_incident_id: u32
pub terminal_internal_failure: Option<InternalSemanticIncidentId>
```

The exact structure can differ.

Add one central operation:

```rust
pub fn record_internal_incident(
    &mut self,
    kind: InternalSemanticIncidentKind,
    details: InternalSemanticIncidentDetails,
    range: Option<SourceRange>,
) -> InternalSemanticIncidentId
```

and one flow-specific adapter:

```rust
pub fn publish_flow_invariant_failure(
    &mut self,
    failure: FlowInvariantFailure,
    range: SourceRange,
) -> InternalSemanticIncidentId
```

This helper should:

1. allocate incident identity;
2. capture current module/callable/expression owner when available;
3. store the structured incident;
4. mark the checker control as terminally internally failed;
5. return the incident ID.

Do not emit an ordinary user semantic diagnostic.

---

# 12. Current callable identity should be available to the incident recorder

`CheckingContext` currently knows current module/class/side and expression ownership, while `analyze_callable_body` owns the `CallableId`.

The amendment requires the context to be able to identify the callable in an internal incident.

Either:

```rust
CheckingContext {
    current_callable: Option<CallableId>,
    ...
}
```

or an equivalent explicit owner mechanism should be used.

Avoid reconstructing callable identity from class + selector after the incident occurs if the exact identity is already known at body-entry time.

---

# 13. Fail-closed flow after a flow invariant violation

Once a flow join has detected a declaration-stable invariant violation, the joined environment cannot be trusted.

Do not continue with:

```text
first predecessor
empty/default FlowState
contract-stripped FlowState
arbitrary intersection
```

The context should transition to an explicit fail-closed flow/control state.

Preferred design:

```rust
enum FlowValidity {
    Reachable,
    Unreachable,
    Poisoned(InternalSemanticIncidentId),
}
```

or an equivalent representation.

If introducing explicit poisoned flow is too invasive for the initial repair, the existing unreachable state may be used temporarily **only if** the terminal internal-failure state is separately preserved and later publication cannot confuse it with ordinary unreachable control flow.

The semantic rule is:

> No statement after an invariant-corrupted flow join may consume a fabricated binding state.

---

# 14. Callable/query containment boundary

For current body analysis, the natural containment boundary is the affected callable semantic computation.

On an internal flow incident:

```text
flow operation fails
    ↓
incident recorded
    ↓
current flow becomes poisoned/fail-closed
    ↓
current expression/statement reports InternalFailure
    ↓
remaining statements in the callable are not analyzed as ordinary flow
    ↓
CallableAnalysis publishes internal failure
```

Do **not** allow the checker to analyze later statements using an invented valid state.

Do **not** automatically abort the entire module or workspace.

Other callable analyses whose dependencies are unaffected remain usable.

This establishes the containment rule:

> **Abort the smallest semantic computation whose internal premises are no longer trustworthy.**

For the current architecture, that is normally the callable body query/product.

Future database-level invariant failures may require a broader containment boundary.

---

# 15. Callable status must represent internal failure explicitly

The parent plan already notes that current `CallableAnalysisStatus` lacks internal failure.

Amend it to include an equivalent of:

```rust
pub enum CallableAnalysisStatus {
    Complete,
    Partial,
    Blocked,
    Cancelled,
    BudgetExceeded,
    InternalFailure(InternalSemanticIncidentId),
}
```

If multiple internal incidents can occur before containment, a compact aggregate may be used, but the first incident should be sufficient to establish terminal internal failure.

Do not map internal failure to:

```text
Partial
Blocked
Cancelled
BudgetExceeded
```

Those have different meanings.

---

# 16. Expression-level publication

For the three current expression/control-flow joins in:

`phalcom-semantic/src/checker/expression.rs`

every `join_with_hierarchy` call must explicitly handle `Err`.

Conceptually:

```rust
match FlowState::join_with_hierarchy(...) {
    Ok(joined) => {
        ctx.flow = joined;
        // normal expression result
    }

    Err(failure) => {
        let incident =
            ctx.publish_flow_invariant_failure(failure, expression_range);

        ctx.poison_flow(incident);

        typed.status =
            AnalysisStatus::InternalFailure(incident);

        return typed;
    }
}
```

## 16.1 Preserve only independently safe value knowledge

Suppose both branches have already independently established the expression value type before the flow-state join fails.

Example conceptual state:

```text
then expression value = Established(Int)
else expression value = Established(Int)

flow metadata join = internal invariant failure
```

It is permissible to retain:

```text
knowledge = Established(Int)
status    = InternalFailure(I1)
```

because the type proposition was independently established before the corrupted flow environment was needed.

But the implementation must not continue deriving **new** value facts from the poisoned joined environment.

This is the same principle used elsewhere in the semantic analyzer:

```text
operation status
    ≠
independently known type knowledge
```

---

# 17. Statement-level publication

The current `Statement::For` path in:

`phalcom-semantic/src/checker/statement.rs`

also invokes `join_with_hierarchy`.

On `Err`:

1. record the incident through `CheckingContext`;
2. poison/fail-close flow;
3. stop ordinary transfer for this statement;
4. expose the terminal condition to `analyze_callable_body`;
5. do not synthesize a user diagnostic;
6. do not continue subsequent statements under an invented state.

Because `check_statement` currently returns `Option<TypeKnowledge>`, do not encode the incident in that return value.

Use context terminal state or evolve the statement result to a structured carrier if the broader status-propagation work in the parent plan already makes that natural.

---

# 18. `analyze_callable_body` must stop after internal failure

Modify:

`phalcom-semantic/src/checker/body.rs`

After each statement/expression transfer, check whether the context has entered terminal internal failure.

Conceptually:

```rust
check_statement(&mut ctx, stmt);

if let Some(incident) = ctx.terminal_internal_failure() {
    status = CallableAnalysisStatus::InternalFailure(incident);
    break;
}
```

Tail-expression handling must do the same.

Do not continue processing the callable's remaining statements.

This is required because later statements may depend on a flow environment that is no longer trustworthy.

---

# 19. Publication into `CallableAnalysis`

The published callable product should retain:

```text
status = InternalFailure(I1)
internal_incidents contains I1
```

alongside any already-safe partial semantic products produced before the failure.

Existing expressions/bindings analyzed before the incident remain queryable.

Products that require the poisoned post-join state must not be fabricated.

This yields a useful release/LSP behavior:

```text
hover/navigation before failure point:
    remains available

affected/later flow-dependent region:
    unavailable/internal-failed

unrelated callables:
    remain available
```

---

# 20. Module/workspace aggregation

`SemanticAnalysis`, snapshot, or the nearest module/session aggregate should expose internal incidents separately from ordinary diagnostics.

Conceptually:

```rust
pub struct SemanticSnapshot {
    ...
    pub internal_incidents: Vec<InternalSemanticIncident>,
}
```

or an indexed equivalent.

Do not merge them into:

```text
all_diagnostics()
```

as ordinary type errors.

A presentation layer may decide to surface a generic “internal analyzer error” notice in release/dev tooling, but the semantic diagnostic collection must retain the distinction.

---

# 21. Development/test/CI failure policy

The semantic analyzer must always record and publish an incident structurally first.

Then the outer execution environment applies policy.

Ratified policy:

```text
test / CI / developer validation
    -> internal incident is fatal

release compiler / release LSP
    -> incident is contained at smallest safe semantic boundary
```

The implementation should not rely solely on `cfg(debug_assertions)` because CI may execute optimized/release-like builds.

Prefer an explicit policy at the session/test harness boundary.

Conceptual type:

```rust
pub enum InternalFailurePolicy {
    Contain,
    FailFast,
}
```

This does **not** need to become a public end-user configuration flag.

It can be chosen internally by constructors used for:

```text
production workspace/session -> Contain
semantic tests               -> FailFast
CI validation entry points   -> FailFast
developer compiler harness   -> FailFast
```

---

# 22. Record first, then fail loudly

Fail-fast behavior should occur **after** the incident has been constructed and attached to its semantic context.

Desired sequence:

```text
detect invariant
    ↓
construct typed FlowInvariantFailure
    ↓
record InternalSemanticIncident
    ↓
mark callable/query InternalFailure
    ↓
development policy triggers failure
```

Do not use a raw assertion at the detection site that destroys context.

Bad:

```rust
assert_eq!(left.contract, right.contract);
```

Better:

```rust
return Err(FlowInvariantFailure::DivergentBindingContract { ... });
```

Then the higher layer can panic/test-fail with a rich report.

---

# 23. Developer failure report format

Fail-loud output should include semantic context useful for immediate debugging.

Representative report:

```text
INTERNAL SEMANTIC INVARIANT FAILURE

module:
  app/core

callable:
  Probe.run()

operation:
  flow join

incident:
  I3

binding:
  BindingId(7)

invariant:
  one BindingId must have one declaration-stable persistent contract

predecessor A:
  contract: Int
  origin: SourceAnnotation
  source: 142..145

predecessor B:
  contract: String
  origin: SourceAnnotation
  source: 201..207

This is an analyzer bug, not a user type error.
```

The exact rendering is non-normative.

The required information is:

- invariant category;
- semantic owner;
- binding/expression identity where applicable;
- conflicting semantic metadata;
- enough source context to reproduce;
- clear classification as analyzer failure.

---

# 24. Release/LSP behavior

Release behavior should not panic the long-lived language server solely because one callable produced an internal invariant failure.

Instead:

```text
affected callable
    -> CallableAnalysisStatus::InternalFailure

affected expression
    -> AnalysisStatus::InternalFailure

affected flow
    -> poisoned/fail-closed

unrelated queries
    -> continue normally
```

The LSP/presentation layer may surface a generic internal-analysis notification if desired, but it must not render the incident as:

```text
Type mismatch: Int is not assignable to String
```

because the source program did not cause the invariant violation.

---

# 25. Universal semantic-test incident gate

Add a shared test helper under the canonical semantic test support layer.

Conceptual API:

```rust
pub fn assert_no_internal_incidents(
    analysis: &SemanticAnalysis,
)
```

Implementation behavior:

```rust
assert!(
    analysis.snapshot.internal_incidents.is_empty(),
    "semantic analyzer produced internal incidents:\n{:#?}",
    analysis.snapshot.internal_incidents,
);
```

The exact storage path may differ.

## 25.1 Make it difficult to forget

The preferred test-harness design is for normal semantic analysis helpers to enforce this automatically.

For example:

```rust
fn analyze(source: &str) -> Analysis {
    let analysis = analyze_single_module(...);

    assert_no_internal_incidents(&analysis);

    analysis
}
```

Tests that intentionally exercise internal incidents should use an explicit alternative:

```rust
analyze_allowing_internal_incidents(...)
```

This makes an internal incident opt-in in the test suite rather than something every test author must remember to reject.

---

# 26. Intentional invariant-failure tests must bypass the universal gate explicitly

Tests such as:

```text
divergent_same_binding_contract_is_internal_failure
```

must deliberately use a low-level harness that allows incidents.

This prevents the normal helper from failing before the test can inspect the structured incident.

The naming should make intent obvious.

Examples:

```rust
analyze_unchecked_for_internal_incident(...)
join_flow_states_for_invariant_test(...)
```

Avoid broad global disabling.

---

# 27. Test-driven implementation sequence

This amendment follows the parent's TDD discipline.

Every production change begins with a failing regression.

---

# 28. RED 1 — direct flow contract invariant

Location:

- direct flow-state test module, likely under
  `phalcom-semantic/tests/semantic/foundations/`
  or the existing flow-state unit tests.

Construct two reachable states:

```text
BindingId(7)

state A:
    contract = Int
    mutable  = true

state B:
    contract = String
    mutable  = true
```

Assert:

```rust
matches!(
    FlowState::join_with_hierarchy(...),
    Err(FlowInvariantFailure::DivergentBindingContract { ... })
)
```

Also assert the implementation does not return:

```text
contract = None
consistency = Unconstrained
first predecessor contract
union contract
```

Current baseline should fail because `join_with_hierarchy` is infallible.

---

# 29. RED 2 — direct flow mutability invariant

Construct:

```text
same BindingId
same contract
different mutability
```

Assert:

```text
Err(FlowInvariantFailure::DivergentMutability { ... })
```

Current behavior should fail because mutability disagreement is silently collapsed.

---

# 30. RED 3 — ordinary current-type disagreement is not an incident

Construct:

```text
same BindingId
same contract = Number
same mutability

A.current = Int
B.current = Float
```

Assert normal success:

```text
Ok(joined)
```

and verify:

```text
joined.contract == Number
joined.current  == join(Int, Float)
joined has no invariant incident
```

This regression prevents overcorrecting normal flow variance into internal failure.

---

# 31. RED 4 — user mismatch remains a user mismatch

Source-level program:

```phalcom
let x: Int = 1

if condition {
    x = "hello"
}
```

Assert:

```text
ordinary AssignmentMismatch diagnostic exists
no InternalSemanticIncident exists
```

This protects the semantic distinction between:

```text
current value violates contract
```

and:

```text
contract metadata itself diverged
```

---

# 32. RED 5 — expression caller publishes `InternalFailure`

Use either:

1. a focused checker-context seam that drives one of the current expression branch joins with crafted predecessor states; or
2. a narrowly constructed internal AST/control-flow fixture.

Do not attempt to invent source syntax that naturally creates same-ID divergent contracts; valid source should not be able to do so.

Assert:

```text
ExpressionAnalysis.status == InternalFailure(I1)
internal incident I1 exists
incident kind == FlowInvariantViolation
flow is poisoned/fail-closed
no TypeMismatch diagnostic was emitted for the invariant
```

If branch result knowledge was independently established before the flow join, assert that knowledge is retained.

---

# 33. RED 6 — statement/`for` caller fails the callable internally

Exercise the `Statement::For` flow-join caller through the narrowest legitimate injected flow seam.

Assert:

```text
callable.status == InternalFailure(I1)
I1 is recorded
subsequent statements are not analyzed under fabricated normal flow
```

If the test suite can inspect later-expression absence/status, verify that later flow-dependent products were not falsely published.

---

# 34. RED 7 — development test harness rejects unexpected incidents

Create a semantic test fixture that deliberately causes an incident through a controlled test seam.

Run it through the normal shared `analyze(...)` helper.

Expected:

```text
test fails because assert_no_internal_incidents fires
```

Then run the same fixture through the special invariant-test helper and assert the incident structurally.

This verifies both:

```text
normal test suite catches analyzer bugs automatically
```

and:

```text
dedicated invariant tests can inspect them intentionally
```

---

# 35. RED 8 — release containment does not kill unrelated callable products

Create a test/session fixture with:

```text
Callable A -> injected flow invariant failure
Callable B -> ordinary valid analysis
```

Under `InternalFailurePolicy::Contain`, assert:

```text
A.status == InternalFailure
B.status == Complete
B's semantic expressions/bindings remain available
```

This is the key end-user containment regression.

---

# 36. RED 9 — fail-fast policy records before failing

Use a test policy hook or session-level fail-fast callback.

Assert that the incident has been stored before fail-fast behavior occurs.

The test may use a catch-unwind boundary if the implementation chooses panic for fail-fast mode, but structured result/test-harness failure is preferable where practical.

The important ordering is:

```text
record -> mark status -> fail
```

not:

```text
panic before record
```

---

# 37. RED 10 — widening rejects declaration-stable divergence

Craft loop header/next-header states with:

```text
same BindingId
different persistent contract
```

Assert:

```text
Err(FlowInvariantFailure::DivergentBindingContract)
```

Then separately verify normal widening:

```text
same contract
different current facts
```

recomputes consistency after joining current knowledge.

---

# 38. RED 11 — incident ID renumbering is not semantic identity

Construct two equivalent callable/module products differing only in local:

```text
InternalSemanticIncidentId(3)
InternalSemanticIncidentId(9)
```

while incident kind/details are equivalent.

Their semantic product fingerprint should be equal.

Then change:

```text
DivergentBindingContract -> DivergentMutability
```

or change relevant semantic details.

The fingerprint should differ.

---

# 39. Production implementation order

Implement in this order.

## Phase 1 — flow failure values

Modify:

`phalcom-semantic/src/checker/flow/state.rs`

1. add `FlowInvariantFailure`;
2. make `join_with_hierarchy` return `Result`;
3. reject divergent contracts;
4. reject divergent mutability;
5. preserve normal current-type joins;
6. make widening fallible under the same invariants.

Do not yet add release panic policy in this module.

## Phase 2 — internal incident model

Add the shared incident types and ID.

Recommended ownership:

```text
checker/analysis.rs
checker/incident.rs
identity.rs
```

depending on existing module organization.

Avoid coupling the incident model to flow-specific implementation types in public snapshot surfaces; use normalized detail objects where appropriate.

## Phase 3 — context recording

Modify:

`checker/context.rs`

Add:

```text
current callable identity
incident allocation
incident storage
terminal internal-failure state
publish_flow_invariant_failure
poison/fail-close flow helper
```

## Phase 4 — migrate all production join callers

Current baseline production callers:

`checker/expression.rs`

- if-let merge;
- if/else/control branch merge;
- control-loop method/body/continue/break merge.

`checker/statement.rs`

- `Statement::For` merge.

Every call must exhaustively handle `Result`.

Static audit after migration:

```bash
rg 'join_with_hierarchy\(' phalcom-semantic/src/checker
```

No production checker call may:

```text
.unwrap()
.unwrap_or_default()
.unwrap_or_else(|_| FlowState::new())
choose a predecessor on Err
erase contract metadata
```

## Phase 5 — callable containment

Modify:

`checker/body.rs`
`checker/analysis.rs`

1. add `CallableAnalysisStatus::InternalFailure`;
2. check context terminal incident after semantic transfers;
3. stop analyzing the remaining body;
4. publish already-safe partial products;
5. include incident collection in callable result/snapshot aggregation.

## Phase 6 — test-harness fail-loud gate

Modify shared semantic test support.

Add:

```text
assert_no_internal_incidents
normal analyze helper auto-check
special explicit allow-internal-incidents helper
```

Migrate existing semantic test helpers so the default path fails on incidents.

## Phase 7 — session/release policy

Introduce internal:

```text
Contain
FailFast
```

selection at the analysis/session boundary.

Production single-module/workspace/LSP constructors use `Contain`.

Tests and CI validation entry points use `FailFast` or the universal incident assertion.

Do not expose this as a casual language/user option.

## Phase 8 — fingerprint/incremental support

Hash:

```text
incident kind
semantic details relevant to downstream behavior
InternalFailure status shape
```

Do not hash raw local incident ID.

Ensure internal-failed callable products invalidate dependents appropriately when their semantic status changes.

---

# 40. Fingerprint semantics

Invariant incidents are semantically observable because:

```text
Complete
```

and:

```text
InternalFailure
```

are not equivalent callable products.

Therefore fingerprints must change when:

- callable status changes from complete/partial/etc. to internal failure;
- incident kind changes;
- semantically relevant incident details change.

Fingerprints must not change solely because:

```text
InternalSemanticIncidentId(2)
```

became:

```text
InternalSemanticIncidentId(5)
```

after allocation order changed.

The same semantic-identity rule used for diagnostic cause IDs applies.

---

# 41. Dependency and cache behavior

A cached internally failed callable product may be reused only while its semantic inputs and dependencies remain equivalent.

When an edit removes the condition that exposed the analyzer bug, recomputation must be able to replace:

```text
InternalFailure
```

with:

```text
Complete
```

or another legitimate status.

Do not permanently poison a workspace/session after one incident.

Containment is product-local/generation-local.

---

# 42. Diagnostics and presentation policy

Internal incidents are not ordinary `SemanticDiagnostic`s.

Do not allocate:

```text
DiagnosticCode::TypeMismatch
DiagnosticCode::AssignmentMismatch
DiagnosticCode::BindingInitializerMismatch
```

for an analyzer invariant.

If the LSP wants to notify the user, presentation may generate a separate generic internal-analysis message such as:

```text
Semantic analysis encountered an internal compiler error in this callable.
Some information may be unavailable.
```

That presentation behavior is outside this amendment's required Part 1 repair.

The formal semantic product must retain the incident classification regardless of whether UI displays it.

---

# 43. Telemetry/logging policy

Where the product/environment permits internal logging, log:

```text
incident kind
module
callable
expression/binding identity
semantic disagreement details
compiler build/revision
```

Do not treat telemetry as the only record of the failure.

The structured incident must remain available in semantic analysis so tests and deterministic diagnostics can inspect it without external logging infrastructure.

---

# 44. Panic/assert policy

Do not use raw `assert!` as the primary semantic implementation.

Allowed uses:

- test helper that asserts incident list is empty;
- fail-fast session policy after structured incident recording;
- debug-only secondary assertion after returning/recording enough data, if it materially improves detection.

Disallowed uses:

```text
assert contract equality inside join
panic before incident recording
release-only fallback that returns a default flow state
```

The semantic API must remain testable as a structured failure.

---

# 45. Broader invariant audit introduced by this amendment

The implementation work should perform a focused audit for places that currently convert impossible state into a plausible default.

Search for patterns including:

```bash
rg 'unwrap_or\(KindId::TYPE\)' phalcom-semantic/src
rg 'contract\s*=\s*None' phalcom-semantic/src/checker
rg 'unwrap_or_default\(\)' phalcom-semantic/src/checker
rg 'unreachable!\(' phalcom-semantic/src/checker phalcom-semantic/src/types
rg 'expect\(' phalcom-semantic/src/checker phalcom-semantic/src/types
```

This audit does **not** require replacing every `expect`/`unreachable!`.

Classify each hit:

```text
A. parser/language impossibility already structurally guaranteed
B. local programmer assertion safe to panic
C. recoverable user semantic invalidity
D. honest analyzer incompleteness
E. internal semantic invariant failure that should become structured
```

Only category E is mandatory follow-up for this amendment unless the parent bugfix plan already covers the hit.

---

# 46. Relationship to the generic kind bug

The parent plan identifies:

```rust
unwrap_or(KindId::TYPE)
```

for missing inference-variable metadata.

This amendment clarifies the category:

> If an already-created inference variable identity is missing required intrinsic kind metadata inside the solver, that is an analyzer invariant failure, not evidence that the variable has kind `Type`.

The generic-inference task may initially use a subsystem-specific structured failure that ultimately records an `InternalSemanticIncident`.

Do not silently replace it with `Type`.

Whether that conversion lands in the same commit as flow incidents or a later generic-inference commit may follow the parent's task order.

---

# 47. Relationship to relation-engine internal failure

`RelationOutcome::InternalFailure` already represents a relation operation that failed internally.

This amendment does not require every `InternalFailure(String)` in the relation layer to immediately become a process panic.

At the checker boundary:

```text
RelationOutcome::InternalFailure
    ↓
record/normalize InternalSemanticIncident
    ↓
AnalysisStatus::InternalFailure
```

under the same contain-vs-fail-loud policy.

This gives one coherent treatment of analyzer failures across flow and relation subsystems.

---

# 48. Relationship to suppression

Internal failure must not be converted into suppression.

Suppression means:

> a required semantic premise is unavailable because of an upstream **user-semantic invalidity** represented by a diagnostic cause.

Internal invariant failure means:

> analyzer state itself is not trustworthy.

Therefore:

```text
InternalFailure
    ≠ Suppressed
```

A caller that depends on an internally failed semantic product should itself become internal-failed/blocked according to dependency policy, not pretend it was suppressed by a user diagnostic cause.

---

# 49. Relationship to `Unknown`

Do not encode invariant failure as:

```text
TypeKnowledge::Unknown(UnknownReason::...)
```

unless the type knowledge is independently unknown for a legitimate reason and internal failure is carried separately.

The operation status must still say:

```text
InternalFailure
```

because `Unknown` alone would incorrectly communicate:

> no type fact is available

rather than:

> the analyzer violated its own invariant.

---

# 50. Relationship to `Blocked`

Do not use:

```text
Blocked(RecursiveFixpoint)
Blocked(OpaqueNative)
Blocked(SuppressedDependency)
```

as containers for invariant failure.

Those are semantically different states.

This amendment reinforces the parent's requirement that terminal outcomes remain distinct.

---

# 51. Relationship to user-visible acceptance

An internal incident must never make an invalid program appear valid by weakening constraints.

Examples of prohibited recovery:

```text
persistent contract disappears
mutability silently becomes immutable
first predecessor wins
unknown becomes assumed annotation
failed invariant becomes Dynamic
```

Containment means stopping unsafe analysis, not continuing with a made-up semantic state.

---

# 52. Recommended code organization

A reasonable non-normative organization is:

```text
phalcom-semantic/src/
├── checker/
│   ├── incident.rs
│   ├── flow/
│   │   └── state.rs
│   ├── context.rs
│   ├── expression.rs
│   ├── statement.rs
│   ├── body.rs
│   └── analysis.rs
└── identity.rs
```

Potential types:

```text
InternalSemanticIncidentId      identity.rs
InternalSemanticIncident        checker/incident.rs
InternalSemanticIncidentKind    checker/incident.rs
FlowInvariantFailure            checker/flow/state.rs
```

Exact placement is not normative.

---

# 53. Recommended commit boundaries

## Commit 1 — tests exposing invariant distinction

```text
test(semantic): specify flow invariant failures
```

Add RED tests for:

- divergent contract;
- divergent mutability;
- normal current-flow disagreement;
- ordinary user mismatch produces no incident.

## Commit 2 — fallible flow APIs

```text
fix(semantic): make flow joins fail closed on invariant violations
```

Implement:

- `FlowInvariantFailure`;
- fallible join;
- fallible widening;
- direct tests GREEN.

## Commit 3 — incident model and checker publication

```text
feat(semantic): record internal semantic incidents
```

Implement:

- incident ID/model;
- context recording;
- current callable ownership;
- flow poisoning;
- expression/statement caller migration.

## Commit 4 — callable containment

```text
fix(semantic): contain invariant failures at callable boundary
```

Implement:

- callable `InternalFailure`;
- stop body analysis;
- publish safe partial products;
- source/composition containment tests.

## Commit 5 — developer and CI fail-loud policy

```text
test(semantic): fail semantic suite on internal incidents
```

Implement:

- universal test helper;
- fail-fast policy;
- intentional-invariant test seam;
- CI/static gates.

## Commit 6 — fingerprint and incremental identity

```text
fix(semantic): fingerprint internal failures by semantic shape
```

Implement:

- status/incident semantic hashing;
- raw incident ID exclusion;
- incremental replacement/recovery tests.

Generic/relation subsystem incident normalization may land in their parent-plan commits if that keeps each change cohesive.

---

# 54. Verification commands

At minimum:

```bash
cargo fmt --check
cargo check -p phalcom-semantic
cargo clippy -p phalcom-semantic --all-targets -- -D warnings
cargo test -p phalcom-semantic
```

Then run the repository's canonical semantic suite command used by the current test organization.

Static audits:

```bash
rg 'join_with_hierarchy\(' phalcom-semantic/src/checker
rg 'widen_loop_state' phalcom-semantic/src phalcom-lsp/src
rg 'contract\s*=\s*None' phalcom-semantic/src/checker/flow
rg 'unwrap_or\(KindId::TYPE\)' phalcom-semantic/src
```

Manual review gate:

- every flow join caller handles `Result`;
- no invariant failure path emits ordinary mismatch diagnostics;
- no invariant failure path fabricates a normal `FlowState`;
- no normal semantic test helper permits unexpected incidents;
- release containment does not become development silence.

---

# 55. Acceptance criteria

This amendment is complete only when all of the following are true.

## Semantic classification

- [ ] User type contradictions remain ordinary diagnostics/`Invalid`.
- [ ] Honest incomplete analysis remains `Unknown`/`Blocked`/`Dynamic`/`Cancelled`/`BudgetExceeded`/`Suppressed` as appropriate.
- [ ] Impossible analyzer states become `InternalFailure`.

## Flow

- [ ] `join_with_hierarchy` is fallible.
- [ ] Same-`BindingId` contract divergence produces structured failure.
- [ ] Same-`BindingId` mutability divergence produces structured failure.
- [ ] Current value-type divergence remains a normal join.
- [ ] Widening enforces the same stable metadata invariants.
- [ ] No invariant path uses `contract = None` as recovery.

## Incidents

- [ ] Internal incidents have structured kind/details.
- [ ] Incidents identify module/callable and binding/expression where available.
- [ ] Incident IDs are local allocator identity only.
- [ ] Incidents are separate from user semantic diagnostics.

## Containment

- [ ] Affected flow becomes poisoned/fail-closed.
- [ ] Affected callable publishes `InternalFailure`.
- [ ] Later statements do not execute under fabricated flow state.
- [ ] Unrelated callable/module analysis remains operational in contain mode.

## Development behavior

- [ ] Shared semantic tests fail on any unexpected internal incident.
- [ ] Dedicated invariant tests can explicitly opt out and inspect incidents.
- [ ] CI/developer validation fails loudly on incidents.
- [ ] Incident is recorded before fail-fast behavior occurs.

## Incrementality

- [ ] `InternalFailure` changes semantic product identity.
- [ ] Incident semantic kind/details participate in fingerprinting as required.
- [ ] Raw local incident IDs do not.
- [ ] A later clean recomputation can replace an internally failed cached product.

---

# 56. Non-goals

This amendment does not require:

- a general end-user crash reporter;
- remote telemetry infrastructure;
- a UI design for internal compiler error notifications;
- making every Rust `assert!`, `expect`, or `unreachable!` structured;
- changing valid user type errors into incidents;
- making every semantic subsystem fallible in one patch;
- exposing `InternalFailurePolicy` as a public language/compiler option.

It establishes the invariant-failure architecture and applies it to the currently identified semantic correctness repairs.

---

# 57. Final normative statement

The parent bugfix specification should be read with the following strengthened rule:

> **Phalcom's semantic analyzer must never recover from an impossible internal semantic state by weakening, erasing, guessing, or fabricating user semantic facts. The detecting subsystem returns a structured invariant failure. The checker records an internal semantic incident, marks the affected semantic operation as `InternalFailure`, and contains the failure at the smallest boundary whose premises are no longer trustworthy. Production/LSP execution remains operational outside that boundary; tests, CI, and developer validation fail loudly after the incident has been recorded.**

This rule applies immediately to flow contract/mutability divergence and should serve as the standard pattern for future impossible semantic states in inference, relation processing, identity management, and incremental semantic products.

---
id: LANG005.C4.P3-COMPLETION
category: LANG
program: LANG005
checkpoint: LANG005.C4
kind: implementation-plan
status: PLANNED
completion: NOT_STARTED
verification: UNVERIFIED
prepared: 2026-09-15
repository: aureat/phalcom-lang
audited_branch: agent/c4-p3-complex-closure-7
audited_revision: f9c74963bb9936e734dafdc04c52d4bdae8f1d2f
audited_commit: "fix(semantic): preserve trait dispatch terminal proof states"
depends_on:
  - LANG005.C4.P1
  - LANG005.C4.P2
  - landed LANG005.C4.P3 T1-T17 architecture
closes:
  - LANG005.C4.P3
  - LANG005.C4
next_checkpoint: LANG005.C5
---

# LANG005.C4.P3 — Completion and Certification Plan

## Patch-Grade Plan for Full C4 Closure

## 0. Purpose

This plan drives `LANG005.C4` from the current late-P3 implementation state to **fully complete, certified, documented closure**.

It is deliberately **not** a replay of the original C4.P3 implementation plan. The audited repository already contains most of the originally planned architecture: trait-dispatch indexing, semantic selection products, detached conformance witnesses and trait defaults, executable conformance plans, runtime conformance environments, trait-selected invocation, abstract-requirement invocation, default-to-default execution, mixed witness origins, and bound trait references.

The remaining work is concentrated in four areas:

1. **Semantic correctness closure** for generic source conformances and terminal proof-state propagation.
2. **Integration closure** for ambiguity diagnostics, exact runtime environments, editor/source projection, and incremental behavior.
3. **Vertical acceptance closure** through realistic `Iterable` / `Iterator` programs and adversarial stress cases.
4. **Certification and lifecycle closure** through broad verification, removal of temporary agent scaffolding, and final checkpoint/walkthrough/handoff documentation.

The executor must treat the audited live implementation as the starting architecture and preserve it unless a task below explicitly requires changing a seam.

---

# 1. Audited Starting State

The plan was derived from the live repository state audited at:

```text
branch: agent/c4-p3-complex-closure-7
HEAD:   f9c74963bb9936e734dafdc04c52d4bdae8f1d2f
commit: fix(semantic): preserve trait dispatch terminal proof states
```

The implementation may advance before execution starts. Therefore **T0 must re-ground against the actual live HEAD** and adapt mechanical names while preserving the semantic contracts in this document.

The current architecture already demonstrates or substantially implements:

```text
explicit conformance heads
coherence and overlap rejection
conformance-local callable ownership
source-level witness plans
exact ConformanceEvidence
trait-dispatch source contributions and index
ordinary trait-evidenced member discovery
shared-witness convergence
semantic trait-dispatch selection attached to expressions
semantic-to-executable trait invocation projection
detached conformance witness compilation
detached trait-default compilation
runtime conformance plan/environment registry
trait-selected invocation
abstract trait requirement invocation
default -> requirement -> concrete witness
default -> default -> witness
inherent witness execution
inherited witness execution
C2 conditional witness execution
data-component witness execution
trait-default bound references
conformance-witness bound references
recursive trait defaults
GC survival during ordinary trait-default execution
specialized exact generic target conformances such as Value<Int> vs Value<String>
```

The final implementation must **extend and certify** these products rather than replace them with class-table mutation, runtime conformance search, or a second trait solver.

---

# 2. Final C4 Acceptance Contract

C4 is complete only when the implementation proves the following end-to-end authority chain:

```text
source `impl TraitRef for Target`
    -> stable source ImplId
    -> canonical conformance head
    -> exact receiver/target matching
    -> exact TraitRef specialization
    -> complete ConformanceEvidence
    -> ordinary trait-evidenced member selection
    -> deterministic convergence or ambiguity
    -> per-expression semantic selection / terminal status
    -> semantic lowering projection
    -> executable conformance plan/environment
    -> detached witness/default execution
    -> correct direct call / nested default / bound-reference execution
```

and simultaneously proves the negative architectural invariants:

```text
trait defaults are not injected into target DeclarationSurface or runtime class dictionaries
conformance-local witnesses are not made inherent
runtime does not discover or prove conformance
runtime does not scan traits/classes to select a witness
LSP/editor code does not independently solve conformance
one generic source conformance does not mint one ImplId per exact application
exact enum-case conformance does not leak to the enum root or sibling cases
Unknown/Blocked/Cancelled/BudgetExceeded/InternalFailure are not disguised as Dynamic
associated types and generic trait bounds do not leak forward from C5/C6
```

---

# 3. Executor Contract

The executor should behave as a patch implementer, not as a speculative language designer.

For every task:

1. Read the live implementation around every named symbol before editing.
2. Add the smallest failing regression that proves the gap.
3. Make the narrowest architectural change that repairs the canonical authority seam.
4. Run only the focused tests needed to prove the gate before expanding verification.
5. Preserve exact semantic identity; do not replace structural identity with strings or source order.
6. Do not resolve a semantic problem in the VM if the missing information belongs in semantic analysis.
7. Do not duplicate an existing solver/query in editor, compiler, or runtime code.
8. Record any deviation from this plan in the final walkthrough, with the reason and equivalent invariant.

If a task discovers that a stated requirement cannot be satisfied without crossing a STOP/CONSULT boundary in §24, stop that task and report the dependency rather than improvising a new language rule.

---

# 4. Non-Negotiable C4 Semantic Invariants

These invariants apply globally throughout the plan.

## 4.1 Identity invariants

1. `ImplId` is source/provenance identity, not exact applied conformance identity.
2. One generic source conformance retains one `ImplId` across every exact specialization.
3. Exact target identity preserves complete applied type arguments.
4. Exact trait identity preserves complete `TraitRef` arguments.
5. Exact enum-case identity preserves `VariantId` and must never normalize to only the enum declaration.
6. `TraitRequirementId` identifies the trait requirement, not its selected witness.
7. Conformance-local callable identity remains owned by `CallableOwnerId::Conformance(ImplId)`.
8. A conformance-local source callable is reused across exact applications of the same source impl.
9. A trait default remains trait-owned and is never cloned into conformance or target ownership.
10. A data-component witness remains a `DataComponentId`; no synthetic getter is required.

## 4.2 Authority invariants

11. P1 head matching does not imply complete conformance evidence.
12. P2 `ConformanceEvidence` is the semantic authority for witness/default selection.
13. P3 ordinary lookup consumes semantic evidence; it does not rerun witness selection.
14. Lowering consumes semantic trait-dispatch selection; it does not rerun trait discovery.
15. Runtime executes an already-proven selection; it does not infer conformance.
16. Editor/LSP projections consume semantic products; they do not independently solve conformance.
17. Runtime class dictionaries remain inherent behavior dictionaries, not conformance registries.
18. Trait default execution remains detached from target runtime class installation.
19. Conformance-local witness execution remains detached from target runtime class installation.

## 4.3 Generic exactness invariants

20. Matching `impl<T> Echo<T> for Value<T>` against `Value<Int>` must bind `T := Int` before exact evidence resolution.
21. The source `TraitRef` `Echo<T>` is not an exact trait reference for `Value<Int>`.
22. Exact conformance evidence for `Value<Int>` must contain `Echo<Int>`.
23. Exact conformance evidence for `Value<String>` must contain `Echo<String>`.
24. These exact evidences share the source `ImplId` but differ in exact target / exact `TraitRef` / impl environment.
25. No consumer may substitute the generic `TraitRef` independently from the canonical conformance-head matcher.

## 4.4 Proof-state invariants

26. `Proven` is the only state that authorizes successful conformance execution.
27. `Incomplete` is not `Dynamic`.
28. `Unknown` is not `Dynamic`.
29. `Blocked` is not `Dynamic`.
30. `Cancelled` is not `Dynamic`.
31. `BudgetExceeded` is not `Dynamic`.
32. `InternalFailure` is not `Dynamic`.
33. Terminal conformance/dispatch states must reach expression/callable analysis without being silently converted to runtime reflection.
34. Terminal states must not produce executable trait-invocation lowering.

## 4.5 Dispatch invariants

35. Canonical inherent dispatch has priority over trait-evidenced ordinary dispatch.
36. If multiple traits select the same concrete callable for the same ordinary member, they converge rather than conflict.
37. Competing independent defaults for the same ordinary member are ambiguous.
38. There is no conformance specialization and no "most specific conformance wins" rule.
39. Source order, import order, and hash-map iteration order cannot break ambiguity or coherence ties.
40. Bound callable references freeze the exact semantic trait selection available at reference creation.
41. Invoking a trait-bound reference must not perform semantic trait selection again.

## 4.6 Runtime environment invariants

42. Runtime type environment and runtime conformance environment are orthogonal products.
43. Installing a conformance environment cannot erase a generic type environment.
44. Installing a type environment cannot erase the active conformance environment.
45. Nested trait defaults reuse the same conformance environment unless an explicitly different evidenced call is entered.
46. Recursive trait defaults preserve the active conformance environment.
47. Bound references retain any environment needed by their frozen selection across GC.
48. Detached methods and conformance plans remain GC-root-safe.

## 4.7 Incremental invariants

49. Add/edit/delete of a conformance invalidates every semantic product that consumed that conformance and no unrelated product by aggregate fallback.
50. Generic source conformance edits invalidate affected exact applications even though they share one source `ImplId`.
51. Body-only witness edits preserve source identity and selection identity unless body inference legitimately changes a published signature.
52. Requirement signature edits revalidate conformances and their use sites.
53. Default edits revalidate selected-default use sites.
54. Ambiguity introduction/removal invalidates ordinary member resolution.
55. Cold and incremental end states are semantically equivalent.
56. Source/editor targets are replaced or removed when their owning conformance/default/witness disappears.

## 4.8 Scope invariants

57. C4 does not introduce associated type binding/projection semantics; C5 owns them.
58. C4 does not introduce general `T: Trait` constraint proof or conditional conformance on trait bounds; C6 owns them.
59. C4 does not introduce trait objects/public existential vtables.
60. C4 does not introduce public conformance reflection descriptors.
61. C4 does not add metatype/class-side conformance unless independently ratified.
62. C4 does not add implicit structural conformance.

---

# 5. Work Breakdown and Gate Sequence

Execute in this order:

```text
T0   live takeover and baseline lock

T1   canonical generic-source exact specialization
T2   terminal proof-state propagation into ordinary checker analysis
T3   strict compiler semantic-failure boundary
G1   semantic correctness authority closed

T4   first-class trait-dispatch ambiguity diagnostic
T5   convergence / ambiguity / terminal interaction matrix
G2   user-visible semantic dispatch closure

T6   generic source runtime integration
T7   runtime type-env + conformance-env composition
T8   exact generic trait-bound references + GC
T9   exact enum-case execution and non-leakage
G3   T18 complete

T10  semantic source/editor projection for trait-evidenced behavior
T11  LSP-facing integration without re-solving
T12  incremental dependency/fingerprint/cold-parity certification
G4   T19 complete

T13  Iterable<Item, Cursor> mixed-witness vertical
T14  stateful Iterator<Item> vertical
T15  realistic ambiguity/convergence verticals
T16  valid high-density C4 stress program
T17  invalid/adversarial diagnostic corpus
G5   T20 + semantic stress closure

T18  runtime anti-authority / anti-injection certification
T19  temporary probe/workflow cleanup
T20  broad workspace certification
T21  walkthrough, handoff, checkpoint, final scoped diff
G6   LANG005.C4 COMPLETE
```

The task numbers intentionally describe the **remaining completion sequence**, not the numbering of the original P3 plan.

---

# 6. T0 — Live Takeover and Baseline Lock

## Purpose

Re-ground this plan against the actual repository revision before editing. The branch may have advanced after the audit.

## Read

At minimum:

```text
docs/implementation/LANG005/LANG005.C4/LANG005.C4-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C4/LANG005.C4-GUIDANCE.md
docs/implementation/LANG005/LANG005.C4/LANG005.C4.P2-handoff.md
docs/implementation/LANG005/LANG005.C4/LANG005.C4.P3-trait-evidenced-dispatch-lowering-and-integration-plan.md

phalcom-semantic/src/impls.rs
phalcom-semantic/src/trait_dispatch.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/diagnostic.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/editor_query.rs
phalcom-semantic/src/source_index/

phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/
phalcom-core/src/frame.rs
phalcom-core/src/vm/
phalcom-core/src/heap/
phalcom-core/tests/core/language/traits.rs

phalcom-semantic/tests/semantic/impls/
phalcom-semantic/tests/semantic/incremental/
```

Also inventory:

```text
.github/scripts/c4_p3_*.py
.github/workflows/*c4*p3*.yml
```

## Steps

### T0.1 Record live state

Record:

```text
git rev-parse HEAD
git status --short
git log -1 --oneline
```

If the branch differs from the audited branch, record the new branch and why it is the execution branch.

### T0.2 Classify every original P3 task

Build a local working table:

```text
ORIGINAL P3 TASK | LIVE STATUS
T1               | complete / partial / replaced
...
T21              | complete / partial / open
```

Do not implement from this table; use it to prevent duplicated work.

### T0.3 Reproduce the known generic-source failure

Use or convert the existing T18 probe into a normal focused test without yet applying its patch. Confirm whether the current HEAD still fails.

Expected failing semantic shape:

```phalcom
trait Echo<T> { echo(_ value: T) -> T }
class Value<T> { @constructor new() {} }
impl<T> Echo<T> for Value<T> {
  echo(_ value: T) -> T { value }
}
```

with exact receiver use through `Value<Int>` and `Value<String>`.

### T0.4 Reconfirm terminal-state collapse

Inspect the live ordinary dispatch seam. If all non-proven `TraitDispatchResolution` states are still converted to `ResolvedDispatchResult::Dynamic`, keep T2 unchanged. If this has already been fixed, run/expand the tests in T2 rather than re-editing.

### T0.5 Reconfirm editor gap

Search `EditorSemanticQuery` and LSP semantic adapters for `TraitDispatchIndex`, `TraitDispatchSelection`, `TraitDispatchSite`, and `ConformanceEvidence` consumption. Classify the current T19 state.

### T0.6 Reconfirm temporary scaffolding

Identify which `.github/scripts/c4_p3_*` files and dedicated workflows are temporary agent probes. Do not delete them until their useful coverage is committed as normal tests.

## Gate G0

Proceed only when the executor has:

```text
live revision pinned
generic-source behavior reproduced or proven fixed
terminal-state behavior classified
editor/source integration classified
temporary probe inventory recorded
```

No production code change should be committed in T0.

---

# 7. T1 — Canonical Generic-Source Exact Specialization

## Purpose

Repair the highest-severity semantic defect: a generic source conformance must specialize its source `TraitRef` from the exact receiver before P2 evidence resolution.

## Expected live defect

A source conformance such as:

```phalcom
impl<T> Echo<T> for Value<T> { ... }
```

is indexed by a source `TraitRef` equivalent to `Echo<T>`. Ordinary trait dispatch matches the exact receiver `Value<Int>` but then attempts evidence lookup using the unspecialized source trait reference rather than `Echo<Int>`.

This causes semantic selection to disappear and runtime to fall through to ordinary missing-message behavior.

## Primary files

Likely:

```text
phalcom-semantic/src/impls.rs
phalcom-semantic/src/trait_dispatch.rs
phalcom-semantic/src/traits.rs                    if exact TraitRef helpers belong there
phalcom-semantic/tests/semantic/impls/queries.rs
phalcom-semantic/tests/semantic/capabilities/traits*.rs or equivalent
```

Do not begin in `phalcom-core` or VM code.

## Required canonical operation

Introduce or expose one conformance-index operation semantically equivalent to:

```rust
pub fn instantiate_source_for_target(
    &self,
    store: &mut TypeStore,
    impl_id: &ImplId,
    exact_target: TypeId,
) -> Option<ConformanceHeadMatch>
```

The exact signature may differ, but the product must contain:

```text
source ImplId
exact_target
exact_trait_ref
impl_bindings
```

## Algorithmic requirement

For one already-indexed source conformance:

```text
1. retrieve source contribution by ImplId
2. reject non-lookup-eligible contribution
3. match source target head against exact receiver
4. produce impl-owned type-parameter bindings
5. apply those bindings to source TraitRef arguments
6. construct exact TraitRef
7. return exact ConformanceHeadMatch
```

Do **not** scan unrelated conformances in this helper. Trait-dispatch family bucketing has already chosen candidate source impls.

Do **not** change `query_exact` semantics merely to make this path work. `query_exact(target, exact_trait_ref)` must remain an exact query.

## Required semantic tests

### GS-01 same generic source, multiple exact targets

```phalcom
trait Echo<T> { echo(_ value: T) -> T }
class Value<T> {}
class Marker {}
impl<T> Echo<T> for Value<T> {
  echo(_ value: T) -> T { value }
}
```

Construct exact `Value<Marker>` and query trait-evidenced `echo(_)`.

Assert:

```text
resolution = Found
selection.source_impl = source ImplId
selection.exact_target = Value<Marker>
selection.exact_trait_ref = Echo<Marker>
```

### GS-02 exact `Int` and `String` applications

Assert two exact selections:

```text
same ImplId
different exact_target
different exact_trait_ref
correct impl bindings
```

### GS-03 no per-application source identity

Assert the conformance source table / witness plan still contains only one source `ImplId`.

### GS-04 nested trait argument substitution

Use a source trait ref whose argument structurally contains the impl parameter, if supported by current C4 type syntax, e.g. a nested generic constructor. Ensure substitution traverses the complete type form rather than only direct parameters.

### GS-05 nonmatching exact target

A candidate bucket may contain the source impl family, but `Value<Int>` must not instantiate a source target pattern that cannot match it.

### GS-06 deterministic identity

Repeat with changed insertion/source order where feasible. Exact product must be identical.

## Implementation constraints

- Reuse canonical `TypeSubstitution`/type environment logic.
- Reuse P1 target-head matching; do not create a second generic matcher.
- Do not invent an exact `ConformanceId`.
- Do not mint new `CallableId`s for each exact application.
- Preserve coherence behavior; this is exact instantiation, not specialization precedence.

## Focused verification

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic impls::queries
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::traits
```

Use actual module filters if names differ.

## Commit boundary

Suggested:

```text
lang005: specialize generic source conformances for exact trait dispatch
```

## T1 acceptance

T1 is complete when semantic trait discovery for a generic source impl produces exact `TraitRef` evidence for multiple exact receivers without changing source identity.

---

# 8. T2 — Preserve Terminal Trait-Dispatch Proof States Through Ordinary Checking

## Purpose

Prevent rich trait-dispatch terminal outcomes from being collapsed back into ordinary runtime `Dynamic` at the checker boundary.

## Primary files

Likely:

```text
phalcom-semantic/src/trait_dispatch.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/types/outcome.rs             only if a shared projection is missing
semantic checker tests
```

## Required state distinction

Preserve at least:

```text
Incomplete
Unknown
Blocked
Dynamic
Cancelled
BudgetExceeded
InternalFailure
```

`Dynamic` remains a legitimate state only when the trait-dispatch proof actually crossed a dynamic boundary.

## Preferred architecture

Do not automatically explode every general dispatch enum unless necessary. Use the repository's existing `AnalysisStatus` / causal-invalidity architecture.

The required semantic projection is:

```text
TraitDispatchResolution
    Proven Found         -> normal resolved trait dispatch
    Missing              -> ordinary missing member path
    Incomplete           -> static invalid/incomplete path
    Unknown              -> AnalysisStatus corresponding to unknown/blocked proof as canonical architecture dictates
    Blocked              -> AnalysisStatus::Blocked
    Dynamic              -> AnalysisStatus::DynamicBoundary
    Cancelled            -> AnalysisStatus::Cancelled
    BudgetExceeded       -> AnalysisStatus::BudgetExceeded
    InternalFailure      -> AnalysisStatus::InternalFailure
```

The exact `AnalysisStatus` representation must follow existing repository conventions.

## Critical rule

For every non-proven terminal state:

```text
no TraitDispatchSelection is published as successful
no trait lowering site is emitted
no runtime trait invocation is generated
```

## Required tests

Add tests at the **actual expression-analysis boundary**, not only low-level resolver tests.

### PS-01 Dynamic stays Dynamic

Construct a legitimate dynamic-boundary trait-dispatch case and assert `AnalysisStatus::DynamicBoundary`.

### PS-02 Blocked stays Blocked

Force or fixture a blocked proof and assert it does not become Dynamic.

### PS-03 Cancelled stays Cancelled

Use the semantic control/cancellation test harness.

### PS-04 BudgetExceeded stays BudgetExceeded

Use a bounded query budget fixture rather than a huge uncontrolled program.

### PS-05 InternalFailure stays InternalFailure

Use existing internal-failure policy/testing seams; do not introduce production-only panics.

### PS-06 Incomplete is statically invalid/incomplete

An incomplete exact conformance must not become runtime-reflective dispatch.

### PS-07 no executable attachment

For all terminal cases, inspect expression/lowering products and assert no executable `TraitInvoke`/trait bound reference selection exists.

## Focused verification

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::traits
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic checker
```

Use actual filters.

## Commit boundary

```text
lang005: preserve terminal trait dispatch states through checking
```

---

# 9. T3 — Strict Compiler Semantic-Failure Boundary

## Purpose

Ensure compiler/certification paths cannot silently use a last-known-good or bootstrap fallback snapshot after a terminal semantic workspace failure.

The workspace session may legitimately retain a best-effort API for interactive clients, but compilation must use a strict resultful boundary.

## Primary files

Likely:

```text
phalcom-semantic/src/session.rs
phalcom-semantic/src/lib.rs or public analysis facade
phalcom-core/src/modules/compile.rs
semantic workspace tests
phalcom-core compiler tests
```

## Required architecture

Expose or reuse two explicit policies:

```text
interactive/best-effort update
    may return retained last-known-good snapshot under documented policy

strict compiler update
    returns terminal analysis failure to caller
```

Do not delete interactive fallback behavior merely to satisfy this task if LSP depends on it.

## Required compiler rule

`ProgramAnalyzer` / `ProgramCompiler` must not successfully compile a fallback snapshot when the requested source's semantic publication terminated in:

```text
InternalFailure
Cancelled
BudgetExceeded
unrecoverable blocked terminal result
```

The exact error wrapper should follow current `ProgramCompileError` architecture.

## Tests

### SF-01 internal semantic failure propagates

Use a deterministic injected/test-only semantic failure seam and assert compilation returns structured failure.

### SF-02 interactive API retains documented fallback behavior

If interactive fallback is intended, keep a regression showing it still works independently.

### SF-03 strict path never substitutes bootstrap-only snapshot

Assert source declarations expected from the program cannot silently disappear while compilation reports success.

## Commit boundary

```text
semantic: make compiler analysis fail closed on terminal workspace failure
```

## Gate G1 — Semantic authority closed

Before moving on, all must hold:

```text
generic source conformance exact specialization passes
terminal states survive checker integration
compiler path fails closed
focused semantic trait suites green
```

---

# 10. T4 — First-Class Trait-Dispatch Ambiguity Diagnostic

## Purpose

Turn retained trait-dispatch candidate provenance into a precise C4 diagnostic rather than a generic "dispatch ambiguous" fallback.

## Primary files

Likely:

```text
phalcom-semantic/src/diagnostic.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/trait_dispatch.rs
semantic diagnostics tests
```

## Add a canonical diagnostic code

Mechanical naming may follow repository convention, for example:

```rust
TraitDispatchAmbiguous
```

with a stable string such as:

```text
trait.dispatch.ambiguous
```

Use the repository's naming style if another canonical family already landed.

## Required diagnostic evidence

The diagnostic should retain enough information to present:

```text
receiver/exact target
selector
candidate exact TraitRefs
candidate TraitRequirementIds
source ImplIds
selected witness/default origin
candidate source locations
```

Do not flatten candidates to only callable names.

## Required deterministic ordering

Sort candidates by canonical semantic identity, not source insertion or map order.

A suitable order can be based on:

```text
exact TraitRef identity
requirement identity
source ImplId
selection origin identity
```

## Core negative case

```phalcom
trait Pretty {
  render -> String { "pretty" }
}
trait Debuggable {
  render -> String { "debug" }
}
class Item {}
impl Pretty for Item {}
impl Debuggable for Item {}

class Caller {
  run(_ item: Item) -> String { item.render }
}
```

Expected: dedicated trait ambiguity diagnostic.

## Companion convergence case

```phalcom
class Item {
  render -> String { "item" }
}
impl Pretty for Item {}
impl Debuggable for Item {}
```

Expected: no trait ambiguity; inherent callable resolves canonically.

## Tests

### AM-01 competing defaults -> trait ambiguity code
### AM-02 candidate count and identities retained
### AM-03 candidate ordering stable under source reorder
### AM-04 concrete inherent resolver removes ambiguity
### AM-05 same concrete selected callable from multiple requirements converges
### AM-06 ambiguity produces no lowering selection

## Commit boundary

```text
lang005: diagnose competing trait dispatch candidates
```

---

# 11. T5 — Dispatch Interaction Matrix

## Purpose

Lock the semantic behavior of ordinary lookup after T1–T4 before runtime expansion.

## Cases

Create one focused semantic matrix covering:

```text
inherent member only
trait member only
inherent + trait default
inherent + conformance witness
inherited member + trait
conditional inherent + trait default
same concrete callable satisfying two traits
competing trait defaults
generic source conformance
specialized exact conformances
exact enum-case conformance
terminal incomplete/blocked path
```

For each assert:

```text
ResolvedDispatchResult shape
TraitDispatchSite presence/absence
selected CallableId or witness origin
exact target
exact TraitRef
diagnostic family
```

This task should mostly add tests. If it exposes a semantic contradiction, repair the canonical dispatch layer before proceeding.

## Gate G2

Semantic C4 ordinary lookup is closed when:

```text
all candidate origins resolve through one authority
ambiguity is user-visible and deterministic
convergence is deterministic
terminal states do not lower
exact generic source evidence is retained
```

---

# 12. T6 — Generic Source Conformance Runtime Integration

## Purpose

Convert the existing failing T18 generic-source probe into a committed executable regression and prove the semantic repair reaches lowering/runtime unchanged.

## Primary files

Likely:

```text
phalcom-core/tests/core/language/traits.rs
phalcom-core/src/modules/semantic_lowering.rs only if a projection bug remains
compiler trait lowering files only if evidence is dropped
runtime files only if exact evidence already arrives correctly and execution still fails
```

## Required program

Use a construction form that isolates C4 from unrelated applied-metatype construction issues:

```phalcom
trait Echo<T> {
  echo(_ value: T) -> T
}

class Value<T> {
  @constructor new() {}
}

impl<T> Echo<T> for Value<T> {
  echo(_ value: T) -> T { value }
}

class Caller {
  intEcho(_ receiver: Value<Int>) -> Int { receiver.echo(7) }
  stringEcho(_ receiver: Value<String>) -> String { receiver.echo("seven") }
}

let caller = Caller.new()
let intValue: Value<Int> = Value.new()
let stringValue: Value<String> = Value.new()
let intResult = caller.intEcho(intValue)
let stringResult = caller.stringEcho(stringValue)
```

Assert deterministic results:

```text
intResult == 7
stringResult == "seven"
```

## Structural assertions

Before VM execution, inspect semantic/lowering products and assert:

```text
same source ImplId
exact Echo<Int> selection for int call
exact Echo<String> selection for string call
distinct runtime conformance environment inputs where exact type arguments differ
no target class method injection
```

## Important diagnosis rule

If execution still falls through to `doesNotUnderstand`, identify the earliest lost product:

```text
ExpressionAnalysis.trait_dispatch
ModuleLoweringSemantics.trait_invocations
ExecutableConformancePlan
runtime conformance environment
selected detached callable
```

Fix that seam only. Do not install `echo(_)` into `Value`'s class dictionary.

## Commit boundary

```text
test: execute generic source trait conformances for exact targets
```

If production code is required, use a semantic message instead.

---

# 13. T7 — Runtime Type Environment + Conformance Environment Composition

## Purpose

Prove that generic type specialization and trait conformance execution coexist rather than overwriting one another.

## Primary files

Inspect before changing:

```text
phalcom-core/src/frame.rs
phalcom-core/src/vm/
phalcom-core/src/compiler/
phalcom-core/src/modules/semantic_lowering.rs
runtime environment registry structures
```

## Required invariant

A frame may need both:

```text
RuntimeTypeEnvironmentId
RuntimeConformanceEnvironmentId
```

These must be independently preserved through:

```text
trait-selected call
default call
abstract requirement call
nested default call
recursive default call
bound-reference invocation
```

## Test design

Use a generic receiver and generic trait/default where the final value depends on both environments.

A good test should make an incorrect environment overwrite observable, not merely inspect non-null IDs.

Conceptual shape:

```phalcom
trait Mapper<T> {
  map(_ value: T) -> T
  twice(_ value: T) -> T { self.map(self.map(value)) }
}

class Box<U> {}
impl<U> Mapper<U> for Box<U> {
  map(_ value: U) -> U { value }
}
```

If the current runtime cannot construct exact `Box<Int>` directly, use annotation-based construction as in T6.

## Tests

### ENV-01 direct trait witness preserves exact type environment
### ENV-02 default -> requirement preserves both environments
### ENV-03 nested default -> default -> requirement preserves both
### ENV-04 recursive default preserves both
### ENV-05 two exact applications cannot contaminate each other

## Implementation rule

Do not merge the two environment IDs into one ambiguous untyped registry merely to pass tests unless that is already the established runtime architecture. Preserve their semantic distinction.

---

# 14. T8 — Exact Generic Trait-Bound References and GC

## Purpose

Complete T17/T18 certification for first-class trait-evidenced callable references.

## Existing baseline

Direct trait-default and conformance-witness bound references already execute. The remaining acceptance surface is exact generic capture and post-capture GC safety.

## Required tests

### BRX-01 generic exact capture

For two exact applications of one generic source conformance:

```text
Value<Int>    -> Echo<Int>
Value<String> -> Echo<String>
```

create a bound reference from the `Int` value and assert invoking it later uses only the frozen `Echo<Int>` evidence.

### BRX-02 trait default bound reference

Capture a default that itself invokes an abstract requirement; invoke it later and prove the captured conformance environment is used.

### BRX-03 GC after capture

```phalcom
let f = &value.member(...)
System.gc
f(...)
```

The receiver, detached method, executable plan, type environment and conformance environment must remain valid.

### BRX-04 independent captures

Capture references from two exact generic instances and invoke them interleaved. No environment contamination.

### BRX-05 ordinary bound-reference regression

Existing non-trait bound methods/families remain unchanged.

## Runtime rule

Reference creation freezes exact semantic selection. Invocation must not call trait semantic lookup or runtime trait scanning.

## Commit boundary

```text
lang005: certify exact trait-bound reference environments
```

---

# 15. T9 — Exact Enum-Case Conformance Execution

## Purpose

Prove exact `VariantId` conformance reaches execution without leaking to the enum root or sibling cases.

## Before editing

Confirm the live semantic session still publishes exact-case conditional/conformance products keyed by full `ExactEnumCase(VariantId)` identity.

Do not reimplement the old C2 exact-case publisher if it is already landed.

## Required vertical

Use a generic or non-generic enum fixture supported by current syntax. Conceptually:

```phalcom
enum Result<T> {
  Ok(T)
  Error(String)
}

trait HasValue<T> {
  value -> T
}

impl HasValue<Int> for Result<Int>::Ok(_) {
  value -> Int { ... }
}
```

Adapt exact-case syntax to the ratified language syntax.

## Required assertions

```text
Ok exact case resolves conformance
Ok exact case executes selected behavior
Result<Int> root does not resolve the case conformance
Error exact case does not resolve the case conformance
no root/sibling runtime method injection
selection/evidence retains VariantId
```

## If source refinement is the blocker

If the language currently has no way to retain the exact-case type at the call site, identify the smallest already-ratified refinement/narrowing path needed by C4. Do not weaken the acceptance test to a root enum value and do not invent new general pattern semantics.

## Gate G3 — T18 complete

T18 is complete only when all are green:

```text
generic source direct execution
runtime type+conformance environment composition
exact generic bound reference
bound reference GC
exact enum-case execution and non-leakage
```

---

# 16. T10 — Semantic Source and Editor Projection for Trait-Evidenced Behavior

## Purpose

Make trait-evidenced ordinary behavior visible to editor/compiler semantic queries without duplicating conformance resolution.

## Primary files

Likely:

```text
phalcom-semantic/src/editor_query.rs
phalcom-semantic/src/source_index/
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/trait_dispatch.rs
semantic editor-query tests
```

## Required model

Use two authoritative channels:

### A. Resolved use sites

For an analyzed expression/reference, consume:

```text
ExpressionAnalysis.trait_dispatch
ExpressionAnalysis.trait_dispatch_candidates
callable-reference semantic resolution
```

for exact definition/navigation/provenance.

### B. Receiver member enumeration

For completion/member listing where no expression selection yet exists, query the compiler-owned `TraitDispatchIndex` using the exact receiver and selector/member-family boundary.

The editor query may enumerate semantic candidates, but it must not rebuild P1/P2 conformance/witness proof independently.

## Required projection targets

For a resolved trait-evidenced member:

```text
ConformanceCallable      -> conformance source CallableId/site
TraitDefault             -> trait default CallableId/site
InherentCallable         -> selected inherent CallableId/site
InheritedCallable        -> inherited callable source
ConditionalInherent      -> actual inherent-impl callable source
DataComponent            -> DataComponentId/source component
```

## Ambiguity

For ambiguous trait dispatch:

```text
return/display all semantic candidates
never choose the first candidate
never fabricate one definition target
```

## Completion behavior

Trait-only members should appear in completion for an exact receiver when the conformance is proven.

Do not duplicate a selector already provided by higher-priority inherent behavior.

Convergent requirements that resolve to the same effective member should not appear as duplicate user-visible entries unless the editor API intentionally exposes provenance separately.

## Tests

### ED-01 definition -> conformance witness
### ED-02 definition -> trait default
### ED-03 definition -> inherited witness
### ED-04 definition -> conditional inherent witness
### ED-05 definition -> data component
### ED-06 completion includes trait-only member
### ED-07 completion suppresses duplicate inherent winner
### ED-08 exact generic TraitRef preserved in hover/presentation product
### ED-09 ambiguity exposes candidates and no arbitrary target
### ED-10 bound reference navigation uses frozen trait selection

## Important separation

Presentation strings are not semantic authority. Keep canonical identities in the compiler product and render them later.

---

# 17. T11 — LSP Adapter Integration Without Re-Solving

## Purpose

Wire the compiler-owned editor products through `phalcom-lsp` where required by existing completion/definition/hover architecture.

## Primary files

Only after T10 products exist:

```text
phalcom-lsp/src/... completion
phalcom-lsp/src/... definition/navigation
phalcom-lsp/src/... hover
existing semantic adapter layer
LSP integration tests
```

## Rules

The LSP may:

```text
format labels
map source sites to URIs/ranges
sort/present completion items
```

It may not:

```text
scan impl declarations
run trait matching
choose a conformance
choose a witness/default
resolve ambiguity
specialize TraitRef arguments
```

## Tests

Use end-to-end LSP tests only for adapter behavior that cannot be proven in semantic tests.

At minimum:

```text
definition of trait-default-backed member
completion of trait-only member
ambiguous trait member has no arbitrary definition jump
incremental edit updates definition/completion result
```

Keep LSP test count small; semantic behavior belongs in `phalcom-semantic`.

---

# 18. T12 — Incremental Dependency, Fingerprint, and Cold-Parity Certification

## Purpose

Certify C4 products under add/edit/delete and exact generic specialization.

## Primary files

Likely:

```text
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/tests/semantic/incremental/
```

Production edits are required only if tests expose stale products or over-broad invalidation.

## Required dependency identities

Ensure C4 consumers can depend on appropriate canonical products, conceptually including:

```text
conformance head/source contribution
conformance witness plan/evidence source product
trait surface / requirement/default
callable signature/body
declaration/inherent dispatch surface
conditional inherent domain
trait dispatch contribution/index family
```

Do not replace fine-grained edges with a workspace-global `TraitSystemChanged` sentinel unless no finer architecture exists and the performance tradeoff is explicitly justified.

## Required edit matrix

### INC-01 add conformance

Before: member missing. After: trait-evidenced member selected.

### INC-02 remove conformance

Selected member disappears; no stale lowering/source target.

### INC-03 witness signature edit

Compatibility/evidence/use site revalidated.

### INC-04 witness body-only edit

Selection identity stable; executable method body changes.

### INC-05 trait requirement signature edit

Conformance completeness/compatibility and use sites revalidate.

### INC-06 trait default add/remove/edit

Selection/default execution availability updates.

### INC-07 inherited witness override

Selected concrete witness changes.

### INC-08 C2 conditional-domain edit

Conditional witness/default decision updates.

### INC-09 ambiguity introduction/removal

Adding/removing a competing trait/default changes call between Found and Ambiguous.

### INC-10 generic source head/body edit

Affected exact `Value<Int>` and `Value<String>` applications revalidate despite sharing one source `ImplId`.

### INC-11 exact trait argument change

Ensure exact TraitRef products do not alias incorrectly.

### INC-12 module deletion

Conformance/trait contribution, source references, editor members and diagnostics disappear.

## Cold/incremental parity

For each representative edit sequence, compare the final incremental snapshot to a fresh cold analysis of identical source.

Compare at least:

```text
diagnostics
TraitDispatchSite / selection
exact TraitRef
selected witness origin
ConformanceEvidence product identity/content
source/editor definition targets
lowering selection existence where exposed by core integration
```

## Fingerprint tests

Add direct fingerprint regressions for trait-dispatch-relevant semantic products if the current fingerprint layer does not already prove them.

Fingerprint semantics must ignore ephemeral IDs but change when externally relevant selection/evidence changes.

## Gate G4 — T19 complete

Proceed only when:

```text
editor semantic products consume C4 authority
LSP does not solve traits
incremental add/edit/delete is correct
cold/incremental parity is proven
```

---

# 19. T13 — `Iterable<Item, Cursor>` Mixed-Witness Vertical

## Purpose

Create the strongest realistic C4 vertical without introducing C5 associated types.

## Fixture

Use the generic-parameter equivalent:

```phalcom
trait Iterable<Item, Cursor> {
  iterate(_ cursor: Option<Cursor>) -> Option<Cursor>
  iteratorValue(_ cursor: Cursor) -> Item

  each(_ f) { ... }
  count -> Int { ... }
  contains(_ expected: Item) -> Bool { ... }
  toList -> List<Item> { ... }
}
```

Adapt bodies to currently available core collection/closure syntax.

## Required conformer

`RangeView` or equivalent:

```text
iterate
    -> existing inherent callable

iteratorValue
    -> conformance-local witness

each/count/contains/toList
    -> trait defaults
```

## Required execution trace

At least one tested operation must force:

```text
trait default toList
    -> trait default each
    -> abstract iterate requirement
    -> inherent witness
    -> abstract iteratorValue requirement
    -> conformance-local witness
```

## Assertions

Do not assert only final output.

Also inspect semantic/lowering products and assert the requirement map contains the intended mixed origins.

## Required cases

```text
toList deterministic content
count deterministic value
contains hit/miss
bound reference to one default or selected requirement
multiple RangeView instances
```

## Anti-injection assertion

Target runtime class must not contain the trait defaults or conformance-local `iteratorValue` merely because the conformance exists.

## Test location

Prefer:

```text
phalcom-core/tests/core/language/traits.rs
```

or a dedicated trait vertical fixture module if the file is becoming too large.

---

# 20. T14 — Stateful `Iterator<Item>` Vertical

## Purpose

Catch receiver/environment/state contamination that pure functions do not expose.

## Fixture

```phalcom
trait Iterator<Item> {
  next -> Option<Item>

  nextOr(_ fallback: Item) -> Item { ... }
  countRemaining -> Int { ... }
  drain -> List<Item> { ... }
}
```

Implement a mutable `CountdownIterator` or equivalent using current field semantics.

## Required proof surface

```text
conformance-local receiver binding
field read/write through exact Self
repeated default -> requirement -> witness calls
state mutation across calls
two independent instances
nested defaults
no frame contamination
no conformance-environment contamination
```

## Required tests

### IT-01 repeated `next`
### IT-02 `nextOr`
### IT-03 `countRemaining`
### IT-04 `drain`
### IT-05 two iterators interleaved
### IT-06 bound reference to stateful trait member if syntax permits

---

# 21. T15 — Realistic Ambiguity and Convergence Verticals

## Purpose

Prove ambiguity semantics as language behavior, not only as a unit-level resolver property.

## Program A — competing defaults

Two traits provide the same ordinary selector through independent defaults. One target explicitly conforms to both. Ordinary call must fail at compile/semantic analysis with the dedicated ambiguity diagnostic.

## Program B — concrete convergence

The same target has an inherent member satisfying both trait requirements. Ordinary call must execute the concrete member and not report ambiguity.

## Program C — shared inherited convergence

If supported cleanly by the hierarchy, two trait requirements should converge on the same inherited concrete callable.

## Program D — source-order permutation

Reorder trait and conformance declarations; result/diagnostic must remain unchanged.

---

# 22. T16 — High-Density Valid C4 Stress Program

## Purpose

Exercise cross-layer interactions in one deterministic executable program.

## Target size

Aim for approximately:

```text
6-8 traits
8-12 target declarations
15-20 conformances
```

Exact count may vary if a smaller program covers all mechanisms densely.

## Must include

```text
conformance-local witnesses
inherent witness reuse
inherited witnesses
data-component witnesses
C2 conditional/specialized witnesses
trait defaults
default -> requirement
default -> default
same concrete callable satisfying multiple traits
distinct generic TraitRefs
one generic source conformance with multiple exact applications
separate specialized exact target conformances
exact enum-case conformance
bound references
mutable receiver state
multiple live conformance environments
```

## Deterministic assertions

Compute concrete values and assert them. "Program did not crash" is insufficient.

Prefer one result record/tuple or several named result bindings that collectively prove the paths.

## Architectural assertions

Inspect compiled/lowering/runtime artifacts where the test harness allows:

```text
trait invocation sites exist
selected requirement slots are stable
trait defaults remain detached
conformance witnesses remain detached
runtime class dictionaries contain only inherent members
```

---

# 23. T17 — Invalid / Adversarial Diagnostic Corpus

## Purpose

Complete C4's negative contract with precise diagnostic-family assertions.

## Required isolated cases

At minimum:

```text
third-party ownership violation
duplicate exact conformance
generic conformance overlap
generic + exact overlap
missing requirement witness
incompatible explicit witness
visibility/access mismatch
property mutability mismatch
extra conformance-local member
duplicate explicit witness
incompatible inherent candidate does not silently satisfy requirement
competing trait defaults
unsupported associated type/binding syntax remains C5-rejected
unsupported trait-bound conditional conformance remains C6-rejected
unsupported metatype conformance
```

Also add regressions for the newly repaired areas:

```text
generic source exact TraitRef specialization mismatch
terminal blocked/internal failure does not become runtime Dynamic
ambiguous trait call emits dedicated diagnostic
exact-case conformance does not leak
```

## Diagnostic assertion rule

Do not use only:

```rust
assert!(snapshot.has_errors())
```

Assert canonical `DiagnosticCode` / family and, where meaningful, candidate labels or source locations.

## Gate G5

T20/stress closure requires:

```text
Iterable vertical green
Iterator vertical green
ambiguity/convergence verticals green
valid stress program green
invalid corpus precise
```

---

# 24. T18 — Runtime Anti-Authority and Anti-Injection Certification

## Purpose

Prove the runtime architecture has not accidentally drifted toward trait installation or runtime semantic discovery.

## Required negative assertions

For representative conformances containing:

```text
trait default
conformance-local witness
data-component witness
conditional inherent witness
```

inspect the target runtime class dictionaries.

Assert:

```text
trait default selector absent unless independently inherent
conformance-local witness selector absent unless independently inherent
```

Then execute the trait-evidenced call successfully.

## Runtime-code audit

Search the runtime/VM for loops or maps that could semantically decide conformance by scanning:

```text
traits
impl declarations
conformance source records
class dictionaries
```

Runtime may index already-lowered plans by IDs. It may not decide whether a type conforms or which witness satisfies a requirement.

## Required assertion for bound references

Invoking a captured trait-bound reference must use its frozen executable selection/environment, not a fresh runtime trait search.

## Commit boundary

Usually test-only unless audit exposes a violation.

---

# 25. T19 — Remove Temporary Agent Probe Infrastructure

## Purpose

Convert all useful T18/T20 probes into durable repository tests and remove implementation-era patch scripts/workflows.

## Inventory candidates

Audit all files matching patterns such as:

```text
.github/scripts/c4_p3_*.py
.github/workflows/agent-c4-p3-*.yml
.github/workflows/*c4-p3-focused*.yml
```

## Rule

Delete a temporary file only after its useful regression coverage is represented by committed normal tests.

## Explicit prohibition

No required production semantic fix may remain encoded as a text-replacement Python script used only in CI.

## Verify repository CI policy

Do not remove a workflow if it has become an intentional permanent project workflow. Classify rather than blindly delete.

## Commit boundary

```text
chore: remove temporary C4 P3 probe infrastructure
```

---

# 26. T20 — Broad Certification

## Purpose

Run the complete C4 acceptance surface only after all focused gates are green.

## Required commands

Adapt module filters to actual test paths, but certify at least:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --test integration impl_syntax

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::traits
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic impls
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::inherent_impl
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::traits

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace

cargo fmt --all -- --check
git diff --check
```

If the project has canonical lint/clippy commands in AGENTS.md/CI, run those required by repository policy as well.

## Failure classification

Every failure must be classified before any patch:

```text
C4_ACTIVE          caused by unfinished task in this plan
C4_REGRESSION      introduced by completion patches
PREEXISTING        demonstrably present at T0 baseline
BASELINE_INFRA     toolchain/environment/infrastructure issue
SCOPE_CONFLICT     requires work outside C4 ownership
SPEC_CONFLICT      implementation reveals unresolved language-semantics conflict
```

Do not weaken assertions or skip tests to make broad certification pass.

## Performance sanity

This plan is correctness-first, but final certification should also verify no obvious accidental whole-workspace scan was introduced into high-frequency ordinary dispatch/editor queries.

If C4 completion turns every member lookup into a linear scan of all conformances, either add the already-planned family/bucket index or record a blocking performance defect before closure.

Do not perform speculative micro-optimization unrelated to measured/query-structure risk.

---

# 27. T21 — Walkthrough, Handoff, Checkpoint, and Final Scoped Diff

## Purpose

Leave C4 as an authoritative completed checkpoint that C5/C6 can safely extend.

## Create/update

```text
docs/implementation/LANG005/LANG005.C4/LANG005.C4.P3-walkthrough.md
docs/implementation/LANG005/LANG005.C4/LANG005.C4.P3-handoff.md
docs/implementation/LANG005/LANG005.C4/LANG005.C4-CHECKPOINT.md
```

Update guidance only if completion discovered a durable invariant not already documented.

## Walkthrough contents

The walkthrough must name the **landed symbols**, not only conceptual names, for:

```text
ConformanceIndex exact source instantiation
ConformanceHeadMatch exact TraitRef path
ConformanceEvidence
TraitDispatchContribution / index
TraitDispatchSelection / site / ambiguity products
terminal-state checker projection
semantic dependencies/fingerprints
editor/source projection
TraitInvocationSpec
ExecutableConformancePlan
runtime conformance environment
trait-selected invocation
trait requirement invocation
trait-bound method/reference runtime object
```

Document the final strongest execution trace, preferably the mixed `Iterable<Item, Cursor>` path:

```text
RangeView
  -> exact Iterable<Int, Int> conformance
  -> ConformanceEvidence
  -> trait-dispatch selection of toList
  -> executable conformance plan
  -> detached default toList
  -> detached default each
  -> abstract iterate slot
  -> inherent RangeView.iterate
  -> abstract iteratorValue slot
  -> detached conformance witness
  -> deterministic List<Int>
```

Also document generic-source exact specialization:

```text
one source ImplId
  -> Value<Int>    / Echo<Int>
  -> Value<String> / Echo<String>
```

## Handoff contents

The C5/C6 handoff must state exactly what is stable and what remains deferred.

### Stable C4 extension seams

```text
explicit conformance identity/head/index
source witness plan
exact ConformanceEvidence
TraitRequirementId keyed selection map
trait dispatch index/selection
runtime executable evidence transport
```

### C5 owns

```text
associated type declarations
associated bindings in conformances
projection normalization
associated-type-driven Iterable final form
```

### C6 owns

```text
generic T: Trait constraints
trait-bound proof machinery
conditional conformance on trait evidence
nested evidence environments required by those constraints
```

C5/C6 must extend evidence structures rather than replacing C4 conformance identity or runtime authority.

## Checkpoint update

Set the checkpoint to `COMPLETE` only after T20 certification.

Example lifecycle state:

```yaml
status: COMPLETE
completion: COMPLETE
verification: CERTIFIED
active_plan: none
next_plan: LANG005.C5
```

Record actual final revision and verification commands/results.

## Final scoped diff audit

Inspect:

```bash
git status --short
git diff --stat <T0-baseline>..HEAD
git log --oneline <T0-baseline>..HEAD
```

Ensure no temporary probes, debugging output, broad unrelated refactors, or uncommitted files remain.

## Final commit

Suggested documentation closure commit:

```text
docs: close LANG005 C4 trait conformance checkpoint
```

---

# 28. Detailed Test Inventory

The executor should ensure the final repository contains coverage equivalent to the following matrix. Existing tests can satisfy entries; do not duplicate them unnecessarily.

## A. Generic exact evidence

```text
GE-01 generic source impl -> exact Int TraitRef
GE-02 generic source impl -> exact String TraitRef
GE-03 same source ImplId across exact applications
GE-04 nested argument substitution
GE-05 nonmatching target rejected
GE-06 exact target identity retained through lowering
```

## B. Terminal proof-state propagation

```text
TP-01 incomplete
TP-02 unknown
TP-03 blocked
TP-04 dynamic
TP-05 cancelled
TP-06 budget exceeded
TP-07 internal failure
TP-08 no lowering for any non-proven terminal state
```

## C. Dispatch and ambiguity

```text
DA-01 trait-only ordinary member
DA-02 inherent beats trait
DA-03 inherited concrete witness
DA-04 conditional inherent witness
DA-05 shared concrete convergence
DA-06 competing defaults ambiguity
DA-07 source-order independence
DA-08 ambiguity no executable selection
```

## D. Runtime evidence transport

```text
RT-01 conformance witness direct execution
RT-02 trait default direct execution
RT-03 default -> requirement -> conformance witness
RT-04 default -> requirement -> inherent witness
RT-05 default -> requirement -> inherited witness
RT-06 default -> requirement -> conditional witness
RT-07 default -> requirement -> data component
RT-08 default -> default -> requirement
RT-09 recursive default
RT-10 generic source exact Int/String
RT-11 exact type + conformance environment
RT-12 exact enum case
RT-13 root/sibling case non-leakage
RT-14 forced GC
RT-15 no class-table injection
```

## E. Bound references

```text
BR-01 default reference
BR-02 conformance witness reference
BR-03 default reference calling requirement
BR-04 generic exact reference
BR-05 two exact references interleaved
BR-06 GC after capture
BR-07 ordinary bound-reference regression
```

## F. Source/editor/LSP

```text
ED-01 definition conformance witness
ED-02 definition trait default
ED-03 definition inherited witness
ED-04 definition conditional witness
ED-05 definition data component
ED-06 trait-only completion
ED-07 inherent winner de-duplicates completion
ED-08 exact TraitRef presentation
ED-09 ambiguity candidates
ED-10 incremental definition update
```

## G. Incremental

```text
IN-01 add conformance
IN-02 remove conformance
IN-03 witness signature edit
IN-04 witness body edit
IN-05 trait requirement edit
IN-06 trait default edit
IN-07 inherited override edit
IN-08 conditional domain edit
IN-09 ambiguity add/remove
IN-10 generic source edit affects exact uses
IN-11 module delete
IN-12 cold/incremental parity
```

## H. Verticals

```text
VA-01 Iterable mixed-witness toList
VA-02 Iterable count/contains
VA-03 Iterator repeated mutable state
VA-04 Iterator multiple instances
VA-05 ambiguity compile-fail
VA-06 concrete convergence executable
VA-07 valid high-density stress program
VA-08 invalid adversarial corpus
```

---

# 29. Likely Files to Change

This is a planning map, not permission to touch every file.

## Semantic core

```text
phalcom-semantic/src/impls.rs
    canonical source-conformance exact specialization

phalcom-semantic/src/trait_dispatch.rs
    consume exact specialized ConformanceHeadMatch
    preserve terminal evidence/candidate provenance

phalcom-semantic/src/checker/context.rs
    ordinary dispatch terminal propagation
    trait ambiguity publication
    semantic dependency recording if missing

phalcom-semantic/src/checker/analysis.rs
    expression/callable trait products and status/fingerprint representation

phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/call.rs
    ambiguity / terminal call behavior

phalcom-semantic/src/diagnostic.rs
    dedicated trait ambiguity diagnostic family

phalcom-semantic/src/session.rs
    strict analysis boundary and incremental invalidation if required

phalcom-semantic/src/editor_query.rs
phalcom-semantic/src/source_index/*
    C4 editor/source projection
```

## Compiler/runtime

Only as evidence requires:

```text
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/* trait invocation/reference paths
phalcom-core/src/frame.rs
phalcom-core/src/vm/*
phalcom-core/src/heap/* bound reference tracing
```

If T1/T2 requires changing VM files, STOP and re-check semantic ownership first.

## Tests

Likely:

```text
phalcom-semantic/tests/semantic/impls/queries.rs
phalcom-semantic/tests/semantic/capabilities/traits*.rs
phalcom-semantic/tests/semantic/incremental/*.rs
semantic editor-query tests
phalcom-lsp tests if adapter work changes
phalcom-core/tests/core/language/traits.rs
```

## Documentation

```text
docs/implementation/LANG005/LANG005.C4/LANG005.C4-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C4/LANG005.C4.P3-walkthrough.md
docs/implementation/LANG005/LANG005.C4/LANG005.C4.P3-handoff.md
```

---

# 30. Prohibited Shortcuts

The executor must not use any of the following to close a failing test:

```text
install trait defaults into target declaration surfaces
install trait defaults into runtime class method tables
install conformance witnesses into target class method tables
runtime-scan all conformances for a receiver
runtime-scan traits to choose a default
choose `.first()` from multiple conformance or trait candidates
specialize generic source conformances by minting new ImplIds
specialize generic witness bodies by cloning new source CallableIds per application
normalize exact enum cases to enum-root identity
convert every non-proven semantic state to Dynamic
make LSP/editor independently resolve conformances
hard-code generic Int/String cases in lowering/runtime
introduce associated types merely to make Iterable realistic
introduce C6 trait-bound proof machinery
silently retain required fixes only in `.github/scripts` probes
weaken negative tests to `has_errors()` when a precise diagnostic is required
skip workspace failures introduced by C4
```

---

# 31. STOP / CONSULT Triggers

Stop implementation and request architectural review if any task appears to require one of these changes:

1. `ImplId` must become exact applied conformance identity.
2. Generic exact applications require cloned source conformance declarations.
3. Exact `TraitRef` specialization cannot be expressed through the existing type substitution/environment system.
4. Correct generic trait dispatch appears to require runtime trait scanning.
5. Terminal proof states can only be preserved by redefining global `Dynamic` semantics.
6. Exact enum-case conformance requires erasing `VariantId` to the enum root.
7. Bound references require trait reselection at invocation time.
8. GC safety appears to require installing detached trait methods onto runtime classes.
9. Editor completion appears to require a second conformance/witness solver.
10. C4 verticals appear to require associated types rather than the generic-parameter fixture.
11. A fix introduces conformance specialization or "most specific impl wins".
12. A fix requires general `T: Trait` proof or conditional conformance on trait evidence.
13. The only way to pass strict compiler tests is to remove an intentionally documented interactive last-known-good API.
14. An unrelated predecessor issue would require a broad C1/C2 redesign rather than a narrow enabling repair.
15. Runtime/class representation changes become significantly broader than transporting already-proven evidence.

---

# 32. Verification Budget and Workflow

Do not run the workspace suite after every edit.

Use this escalation model:

## Level 1 — task-local

Run the smallest unit/integration test that proves the changed semantic seam.

Examples:

```text
impl query tests
trait dispatch tests
checker trait tests
one core trait runtime test
```

## Level 2 — gate-local

At G1/G2/G3/G4/G5 run the relevant crate-level focused suites.

## Level 3 — pre-certification

Run full trait/inherent/incremental semantic/core suites.

## Level 4 — T20 only

Run `cargo test --workspace`, `cargo check --workspace`, formatting, diff checks, and repository-required broad CI commands.

The priority is rapid correctness closure while retaining strong final certification.

---

# 33. Suggested Commit Sequence

Commit granularity should preserve reviewer-meaningful architecture. A reasonable sequence is:

```text
1. lang005: specialize generic source conformances for exact trait dispatch
2. lang005: preserve terminal trait dispatch states through checking
3. semantic: make compiler analysis fail closed on terminal workspace failure
4. lang005: diagnose competing trait dispatch candidates
5. test: execute generic source trait conformances for exact targets
6. lang005: preserve exact runtime environments for trait execution
7. lang005: certify exact trait-bound reference environments
8. test: execute exact enum-case trait conformances
9. lang005: project trait-evidenced behavior to editor queries
10. lsp: consume compiler-owned trait member projections
11. test: certify C4 trait incrementality and cold parity
12. test: add C4 iterable and iterator verticals
13. test: add C4 trait conformance stress corpus
14. chore: remove temporary C4 P3 probe infrastructure
15. docs: close LANG005 C4 trait conformance checkpoint
```

Mechanical grouping may differ if the live code makes two adjacent patches inseparable, but avoid one monolithic "finish C4" commit.

---

# 34. Final Completion Checklist

C4 may be marked complete only when every item below is checked.

## Semantic identity and evidence

- [ ] Generic source conformance specializes exact `TraitRef` from exact target.
- [ ] One source `ImplId` is reused across exact applications.
- [ ] Exact generic targets remain distinct.
- [ ] Exact TraitRefs remain distinct.
- [ ] Exact enum cases retain `VariantId`.
- [ ] `ConformanceEvidence` remains the witness/default authority.

## Ordinary dispatch

- [ ] Trait-only ordinary members resolve.
- [ ] Inherent behavior retains priority.
- [ ] Shared concrete witnesses converge.
- [ ] Competing defaults diagnose dedicated trait ambiguity.
- [ ] Ambiguity is deterministic.
- [ ] Bound references freeze exact selection.

## Proof states

- [ ] Incomplete is preserved.
- [ ] Unknown is preserved.
- [ ] Blocked is preserved.
- [ ] Dynamic is preserved only when genuinely dynamic.
- [ ] Cancelled is preserved.
- [ ] BudgetExceeded is preserved.
- [ ] InternalFailure is preserved.
- [ ] Non-proven states do not lower executable trait invocation.

## Compiler/runtime

- [ ] Generic source conformance executes for at least two exact applications.
- [ ] Type and conformance environments coexist correctly.
- [ ] Trait defaults execute detached.
- [ ] Conformance witnesses execute detached.
- [ ] Default -> requirement works for every supported witness origin.
- [ ] Default -> default -> witness works.
- [ ] Recursive default works.
- [ ] Exact enum-case conformance executes without leakage.
- [ ] Exact generic bound reference executes.
- [ ] Bound trait reference survives GC.
- [ ] Runtime class dictionaries are not used as conformance authority.
- [ ] Compiler strict analysis does not silently compile fallback snapshots.

## Editor/source/incremental

- [ ] Definition/navigation reaches selected trait/default/witness source.
- [ ] Trait-only members participate in completion.
- [ ] Ambiguity exposes candidates rather than choosing first.
- [ ] LSP does not solve traits independently.
- [ ] Add/edit/delete conformance invalidation works.
- [ ] Requirement/default/witness edits invalidate consumers correctly.
- [ ] Generic source edits invalidate exact applications.
- [ ] Cold and incremental final snapshots agree.

## Vertical acceptance

- [ ] Generic-parameter `Iterable<Item, Cursor>` mixed-witness program passes.
- [ ] Stateful `Iterator<Item>` program passes.
- [ ] Competing-default program fails with precise diagnostic.
- [ ] Concrete-convergence program executes.
- [ ] High-density valid C4 stress program executes deterministic results.
- [ ] Invalid/adversarial corpus asserts diagnostic families.

## Repository closure

- [ ] Temporary C4 agent patch scripts/workflows are removed or explicitly retained as permanent infrastructure.
- [ ] Focused tests are green.
- [ ] Workspace tests are green or every unrelated baseline failure is formally classified.
- [ ] Workspace check is green.
- [ ] `cargo fmt --all -- --check` is green.
- [ ] `git diff --check` is green.
- [ ] P3 walkthrough is written against landed symbols.
- [ ] P3 handoff clearly defines C5/C6 extension seams.
- [ ] C4 checkpoint is updated to COMPLETE/CERTIFIED with final revision and evidence.
- [ ] Working tree is clean.

---

# 35. Final Acceptance Statement

`LANG005.C4` is complete when Phalcom can take an explicit source conformance, preserve its exact generic or enum-case identity, prove a complete witness/default mapping, expose that behavior through ordinary semantic member lookup, diagnose genuine ambiguity without arbitrary precedence, transport the already-proven evidence through lowering, execute detached witnesses/defaults and first-class bound references under the correct runtime environments, project the same truth to editor/incremental consumers, and pass realistic `Iterable`, `Iterator`, generic-source, exact-case, ambiguity, GC, and stress programs — **without turning traits into classes, conformance into inherent behavior, or runtime lookup into semantic authority**.

At that point C4 should be frozen as the stable conformance/execution substrate for C5 associated types and C6 trait constraints rather than extended further inside this checkpoint.

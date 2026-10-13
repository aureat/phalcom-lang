---
id: TYPEAUD001.C1.P2
category: semantic-correctness
program: TYPEAUD001
checkpoint: TYPEAUD001.C1
kind: implementation-plan
title: "Canonical Relation Algebra, Ordered Tuple/Pack Subtyping, and Parameter-Aware Proof Completeness"
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
branch: main
prepared_against_revision: e2ae4a0fd360e4075ce8ad229a3e39e1531fb771
prepared: 2026-10-13
planner_grade: luna-patch-grade
depends_on:
  - TYPEAUD001.C1.G0-baseline-and-authority-gate
follows: null
supersedes: null
owner_scope: canonical relation algebra and ordered structural subtyping
incidents:
  - INC-002
  - INC-003
  - INC-008
  - INC-010
additional_confirmed_scope:
  - ordered Tuple/Pack source remainder must not satisfy a missing required fixed labeled entry
---

# TYPEAUD001.C1.P2 — Luna Patch-Grade Implementation Plan

## Canonical Relation Algebra, Ordered Tuple/Pack Subtyping, and Parameter-Aware Proof Completeness

## 0. Plan purpose

`TYPEAUD001.C1.P2` repairs the canonical subtype/relation layer for four open type-system audit incidents and one additional source-confirmed defect found while re-auditing the same code path. The implementation must make relation results follow the semantic proposition rather than traversal order, make Tuple and Pack labeled lanes obey their normative ordered-product semantics, and ensure knowledge-to-contract checking gives unconditional canonical subtype laws a chance to prove before unresolved generic parameters force an epistemic block.

This is a **semantic authority correction**, not a cosmetic cleanup. The affected relation results flow into expression validity, binding contracts, field lifecycle, generic evidence, coverage, static evaluation, diagnostics, and downstream compilation decisions. P2 therefore must be implemented as one coherent relation package with direct proof-state oracles and explicit acceptance gates.

The plan is written against `main` revision:

```text
 e2ae4a0fd360e4075ce8ad229a3e39e1531fb771
```

Before source modification begins, T0 must re-pin the actual implementation base and record any drift. Mechanical drift may be adapted. Any change in semantic authority, representation, or P2/P3 ownership is a STOP / CONSULT event.

---

# 1. Scope

## 1.1 Incidents owned by P2

| Incident | Defect | Required architectural correction |
|---|---|---|
| `INC-002` | Tuple labeled subtyping matches by label name and ignores labeled-lane order | Ordered fixed-prefix relation for labeled Tuple entries; same canonical rule for Pack |
| `INC-003` | Target-union existential relation can return an earlier `Blocked` before a later `Proven` | Canonical ANY/disjunction relation reducer |
| `INC-008` | Source-union universal relation can return an earlier `Blocked` before a later definitive `Refuted` | Canonical ALL/conjunction relation reducer |
| `INC-010` | `check_knowledge_against_type_bounded` blocks when a top-level parameter appears before unconditional canonical subtype laws can prove | Proof-first canonical relation plus unresolved-parameter refutation fence |

## 1.2 Additional source-confirmed defect included in P2

The current Tuple and Pack relation code permits a typed **source labeled remainder** to satisfy a missing fixed target labeled entry. That is invalid because an anonymous remainder admits **zero or more** entries and therefore cannot prove required presence.

Normative consequence:

```telos
(x: Int, **: String) </: (x: Int, y: String)
(**: String)         </: (y: String)
```

The same rule applies to first-class `Pack` forms.

This is not a separate architectural package. It is repaired by the same ordered fixed-prefix/remainder relation introduced for INC-002.

## 1.3 Primary implementation owner

```text
phalcom-semantic/src/types/relation.rs
```

Likely supporting owner if reducer logic belongs with the outcome type:

```text
phalcom-semantic/src/types/outcome.rs
```

Focused tests are expected primarily under:

```text
phalcom-semantic/tests/audit_probes/subtyping_bounds.rs
phalcom-semantic/tests/semantic/advanced/relation_matrix.rs
phalcom-semantic/tests/semantic/foundations/type_model.rs
phalcom-semantic/tests/semantic/foundations/pack_types.rs
```

A new focused file such as:

```text
phalcom-semantic/tests/semantic/foundations/relation_algebra.rs
```

is acceptable if it keeps the proof matrix clearer than extending heterogeneous existing files.

---

# 2. Explicit non-goals

P2 must not absorb adjacent checkpoint work.

## 2.1 P1 exclusions

Do not modify or redesign:

- semantic publication transactions;
- `SemanticDb` rollback/journaling;
- snapshot authority;
- pending semantic delta durability;
- compiler/LSP failure propagation.

P2 consumes P1's semantic-publication guarantees; it does not reproduce them.

## 2.2 P3 exclusions

Do not implement the full Dynamic generic-relation repair from `INC-009`.

Specifically, P2 must not broadly redefine:

- `StaticArgument::DynamicType` relation semantics;
- applied-generic variance across Dynamic slots;
- runtime boundary obligations for applied generics;
- downstream P3 Dynamic consumers.

P2 **must** preserve `DynamicBoundary` as a distinct outcome and design relation reducers so P3 can extend them without replacing them.

## 2.3 P4 exclusions

Do not repair in P2:

- Tuple/Pack specialization reconstruction;
- independent remainder loss during materialization;
- Record-row tail materialization;
- residual specialization environment reconstruction.

## 2.4 Generic inference exclusions

Do not add ambient generic evidence to the core relation API merely to fix INC-010. Existing generic proof/inference authorities remain in:

```text
generic_evidence.rs
checker/call.rs
checker/inference.rs
types/rigid.rs
```

P2 owns unconditional canonical subtype truth and parameter-sensitive uncertainty, not a second generic-constraint solver.

---

# 3. Normative semantic authority

Implementation must be derived from the accepted specifications, not current code behavior.

## 3.1 Subtype and solver laws

Relevant laws include:

- `T0100` — subtype reflexivity;
- `T0101` — subtype transitivity;
- `T1000` — solver positive conclusions are sound;
- `T1001` — solver negative conclusions are sound;
- `T1002` — unknown is not disproof;
- `T1003` — unknown is not proof;
- `T1024` — solver proves `Never <: A`;
- `T1026` — union least-upper-bound solver law;
- `T1030` — solver order independence;
- `T1033` — deterministic solver result;
- `T1036` — registry-entailable propositions must not be disproven.

## 3.2 Union laws

For proper types:

```text
(A | B | ...) <: T
```

is universal/conjunctive over members:

```text
(A <: T) AND (B <: T) AND ...
```

while:

```text
T <: (A | B | ...)
```

is existential/disjunctive over alternatives:

```text
(T <: A) OR (T <: B) OR ...
```

Union member storage/interner order is non-semantic.

## 3.3 Ordered product laws

Tuple and Pack each have two ordered structural lanes:

```text
positional fixed prefix + positional remainder
labeled fixed prefix    + labeled remainder
```

Labeled fixed entries:

- use `Symbol` labels;
- are ordered;
- are required;
- are compared at corresponding positions;
- are covariant in their element type.

Anonymous remainders admit zero or more entries and cannot establish the existence of a required fixed entry.

Record is different: it remains unordered/key-based and presence-qualified.

## 3.4 Parameter law relevant to INC-010

`Never` is the bottom proper type:

```text
Never <: A
```

for every well-formed proper type `A`, including a proper-type generic parameter.

Unresolved parameter relations that are not universally entailed must remain epistemically unresolved rather than being promoted to proof or hard refutation.

---

# 4. P2 invariants

The implementation and its tests must preserve the following durable contracts.

### P2-INV-01 — Union result is allocation-order independent

Under sufficient budget and absent cancellation/internal failure, changing union member construction or `TypeId` allocation order must not change the semantic `Proven` / `Refuted` / unresolved result.

### P2-INV-02 — Source union is ALL

For `U = A | B | ...`:

```text
U <: T
```

is proven iff every member relation is proven, and definitively refuted if any member relation is refuted.

### P2-INV-03 — Target union is ANY

For `U = A | B | ...`:

```text
T <: U
```

is proven if any alternative relation is proven, and definitively refuted only when every alternative is refuted.

### P2-INV-04 — Blocked is epistemic, not decisive

`Blocked` is neither proof nor refutation. It must not prevent discovery of a later decisive result for the relevant quantifier.

### P2-INV-05 — Operational terminals remain operational

`Cancelled`, `BudgetExceeded`, and `InternalFailure` must not be flattened into logical relation outcomes or silently ignored to continue traversal.

### P2-INV-06 — DynamicBoundary remains distinct

`DynamicBoundary` is not `Blocked`, `Refuted`, or `Proven`. P2 may isolate its mixed-reducer policy but must not erase the category.

### P2-INV-07 — Tuple labeled lane is ordered

Tuple labeled fixed entries are compared by corresponding position and label identity, never by unordered `.find(label)` lookup.

### P2-INV-08 — Pack labeled lane is ordered

First-class Pack obeys the same ordered fixed-prefix semantics as Tuple.

### P2-INV-09 — Remainders cannot prove fixed presence

A source anonymous positional/labeled remainder cannot satisfy a target fixed entry that is absent from the source fixed prefix.

### P2-INV-10 — Surplus entries are controlled by target remainder

Source fixed entries beyond the target fixed prefix are legal only when admitted by the target lane remainder and satisfy its type constraint when typed.

### P2-INV-11 — Source openness must be compatible with target openness

An open source lane cannot subtype a closed target lane. Typed/unconstrained remainder covariance follows the normative lane rules.

### P2-INV-12 — Record remains unordered

The Tuple/Pack ordered helper must not alter Record key-based semantics.

### P2-INV-13 — Canonical proof precedes parameter uncertainty

Knowledge-to-contract checking must attempt unconditional canonical subtype proof before a top-level parameter causes an unresolved result.

### P2-INV-14 — Parameters are not fabricated into proof

Deleting the old parameter guard must not cause unsupported `T <: Int` or `Int <: T` propositions to become statically proven.

### P2-INV-15 — Parameter uncertainty is not fabricated into hard falsity

Where the canonical engine's fallback `TypeMismatch` only reflects unsupported open-parameter reasoning, knowledge-to-contract checking must preserve unresolved state rather than report a contradiction.

### P2-INV-16 — Core identity is canonical, not spelling-based

Special behavior for core types must use their actual canonical Universe declaration identities. Synthetic root-level names are not equivalent merely because their spelling matches.

---

# 5. Architectural design

## 5.1 Canonical quantified relation reducers

P2 must introduce one reusable internal authority for combining branch outcomes.

Acceptable shapes include:

```rust
enum RelationConnective {
    All,
    Any,
}
```

with one reducer, or narrow helpers:

```rust
reduce_all_relations(...)
reduce_any_relations(...)
```

Do not keep two independent ad hoc union loops.

### 5.1.1 ALL / conjunction table

| Observed branch set | Aggregate |
|---|---|
| all `Proven` | `Proven` |
| any `Refuted` | `Refuted` |
| `Blocked` + later `Refuted` | `Refuted` |
| `Refuted` + later `Blocked` | `Refuted` |
| `Proven` + `Blocked`, no refutation | `Blocked` |
| multiple `Blocked`, no refutation | deterministic `Blocked` |

Short-circuit condition:

```text
Refuted
```

because no unseen branch can reverse conjunction falsity.

Do **not** short-circuit on `Blocked`.

### 5.1.2 ANY / disjunction table

| Observed branch set | Aggregate |
|---|---|
| any `Proven` | `Proven` |
| all `Refuted` | `Refuted` |
| `Blocked` + later `Proven` | `Proven` |
| `Proven` + later `Blocked` | `Proven` |
| `Refuted` + `Blocked`, no proof | `Blocked` |
| multiple `Blocked`, no proof | deterministic `Blocked` |

Short-circuit condition:

```text
Proven
```

Do **not** short-circuit on `Blocked`.

### 5.1.3 Operational terminal rule

Unless a consultation changes this ruling, preserve immediate termination for:

```text
Cancelled
BudgetExceeded
InternalFailure
```

These represent computation that cannot validly continue rather than incomplete logical evidence.

Allocation-order independence is required for **completed adequately-resourced relations**, not for computations deliberately terminated by resource/cancellation policy.

### 5.1.4 DynamicBoundary seam

The reducer must track `DynamicBoundary` independently from `Blocked`. If mixed `DynamicBoundary + Blocked` semantics are not already authoritative, isolate final precedence in one helper and do not invent a broad P3-level law in multiple branches.

If a concrete P2 test requires choosing an unestablished mixed rule, STOP / CONSULT before expanding scope.

## 5.2 Deterministic blocked aggregation

Continuing past `Blocked` means more than one `BlockReason` may be observed. Returning the first observed reason would reintroduce union member order dependence into published analysis state.

P2 must therefore establish a stable join for multiple blocked reasons.

Preferred approaches, in order:

1. Reuse an existing semantic precedence if one already exists for `BlockReason`.
2. Add a private deterministic ranking/join analogous to `join_unknown_reason` in `types/evidence.rs`.
3. If a new externally visible reason model is required, STOP / CONSULT.

The merge must be:

```text
stable
commutative
order independent
```

for semantically unordered branch sets.

## 5.3 Evidence/provenance preservation

The reducer must not gratuitously erase branch evidence.

For ANY:

- return the evidence from the branch that proves the proposition.

For ALL:

- preserve existing evidence behavior at minimum;
- do not redesign the global evidence representation merely for P2;
- if evidence composition is already supported, use the canonical mechanism rather than dropping it.

For source-union refutation, retain the public union-level `RelationFailure` shape unless changing it is necessary. If the implementation can preserve the offending member as explanation/provenance without public redesign, do so.

## 5.4 Shared ordered-lane checker

P2 must replace duplicated label-search logic with one semantic ordered-lane algorithm reused by Tuple and Pack.

A recommended decomposition is:

```rust
check_ordered_positional_lane(...)
check_ordered_labeled_lane(...)
check_ordered_remainder_relation(...)
```

or a typed generic internal adapter over common lane entries.

The helper must not merge Tuple and Pack identity. It shares only relation mechanics.

### 5.4.1 Fixed-prefix rule

For every lane:

```text
source.fixed.len >= target.fixed.len
```

must hold before comparing target fixed entries.

A source remainder may contain zero entries and therefore cannot satisfy a target fixed-entry deficit.

### 5.4.2 Positional fixed comparison

For each target fixed index `i`:

```text
source[i].type <: target[i].type
```

using the canonical recursive relation.

### 5.4.3 Labeled fixed comparison

For each target fixed index `i`:

```text
source[i].label == target[i].label
AND
source[i].type <: target[i].type
```

Do not search another index for the same label.

### 5.4.4 Surplus fixed source entries

For `source.fixed[target.fixed.len..]`:

- target `Closed` -> `Refuted`;
- target `Unconstrained` -> admitted;
- target `Typed(T)` -> each surplus element type must satisfy `<: T`.

### 5.4.5 Remainder-to-remainder rule

After fixed entries are validated:

| Source remainder | Target remainder | Result |
|---|---|---|
| Closed | Closed | allowed |
| Closed | Typed(T) | allowed |
| Closed | Unconstrained | allowed |
| Typed(S) | Closed | refuted |
| Typed(S) | Typed(T) | require `S <: T` |
| Typed(S) | Unconstrained | allowed |
| Unconstrained | Closed | refuted |
| Unconstrained | Typed(T) | refuted |
| Unconstrained | Unconstrained | allowed |

This rule is lane-local and applies independently to positional and labeled lanes.

## 5.5 Parameter-aware knowledge relation

The current behavior:

```text
Known equality
  -> Proven
else if actual or expected is TypeData::Parameter
  -> Blocked(RecursiveFixpoint)
else
  -> canonical subtype relation
```

must be replaced by proof-first logic.

Required shape:

```rust
handle cancellation
handle Dynamic actual knowledge
handle Unknown actual knowledge
extract Known evidence

if actual == expected {
    return Proven
}

let parameter_sensitive =
    matches!(actual, TypeData::Parameter(_))
    || matches!(expected, TypeData::Parameter(_));

let outcome = check_subtype_bounded(...);

match outcome {
    Proven { .. } => outcome,

    Cancelled
    | BudgetExceeded(_)
    | InternalFailure(_)
    | Blocked(_)
    | DynamicBoundary(_) => outcome,

    Refuted(_) if parameter_sensitive => {
        // preserve epistemic uncertainty
        Blocked(...)
    }

    Refuted(_) => outcome,
}
```

The exact code may differ, but the semantic order must not.

### 5.5.1 Required positive control

```text
Known(Never) against T -> Proven
```

because the canonical relation must establish `Never <: T` first.

### 5.5.2 Required negative controls

The repair must not turn these into unconditional proof:

```text
T <: Int
Int <: T
```

when no contextual generic evidence proves them.

### 5.5.3 BlockReason taxonomy

The current `RecursiveFixpoint` reason is semantically questionable for an ordinary unresolved open parameter.

T0/T6 must inspect current `BlockReason` variants.

- If an appropriate existing reason exists, use it.
- If not, prefer minimal scope and preserve `Blocked` correctness first.
- Adding a public/fingerprinted variant requires an explicit impact census and may trigger consultation if it broadens metadata/API compatibility.

## 5.6 Canonical Object identity requirement

The existing audit probe that constructs:

```text
DeclarationId(ModuleId::universe_root(), "Object")
```

is not the canonical Universe Object identity.

Canonical Object is obtained through:

```text
universe_declaration(UniverseKey::Object)
```

and resolves to the source-owned Universe declaration.

P2 must **not** change a synthetic-root fixture merely to force an Object rule. Instead:

- keep synthetic identity distinct;
- add a canonical Object regression separately if relevant;
- do not conflate `Object` with normative top type `Any`.

---

# 6. Task plan

## T0 — Pin implementation base, read authorities, and perform affected-surface census

### Objective

Establish a reproducible implementation baseline and verify that the architectural assumptions in this plan still match `main`.

### Required reads

At minimum:

```text
AGENTS.md or current root agent guidance
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/shared-consultation-escalation-protocol.md
docs/implementation/TYPEAUD001/TYPEAUD001-PROGRAM.md
docs/implementation/TYPEAUD001/TYPEAUD001-STATUS.md
docs/implementation/TYPEAUD001/C1/TYPEAUD001.C1-CHECKPOINT.md
docs/implementation/TYPEAUD001/C1/TYPEAUD001.C1-GUIDANCE.md
P1 completion handoff, if P1 has completed by execution time
all available INC-002/003/008/010 reports and relevant audit handoffs
```

Normative specs:

```text
docs/specs/typing/type/type.subtype.md
docs/specs/typing/type/type.inference-solver.md
docs/specs/typing/type/type.union.md
docs/specs/typing/type/type.never.md
docs/specs/typing/type/type.tuple.md
docs/specs/tuple-and-record-typing.md
docs/specs/structural-products.md
docs/specs/generic-constraints.md
```

### Source census

Inspect at minimum:

```text
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/outcome.rs
phalcom-semantic/src/types/evidence.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/binding.rs
phalcom-semantic/src/checker/field_lifecycle.rs
phalcom-semantic/src/generic_evidence.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/coverage/usefulness.rs
phalcom-semantic/src/static_eval/machine.rs
```

### Deliverable

Record:

- exact HEAD;
- clean/dirty worktree state;
- P1 lifecycle state;
- changed files since this plan's `analysis_base`;
- affected public/private relation callers;
- any drift to `RelationOutcome`, `BlockReason`, Tuple/Pack representations, or relation entrypoints.

### Gate

No P2 source patch before T0 confirms the canonical owner remains `types/relation.rs` and the checkpoint still assigns INC-002/003/008/010 to P2.

---

## T1 — Establish red proof oracles before semantic implementation

### Objective

Create focused tests that fail for the intended reasons on the pre-fix implementation.

### Required red oracles

#### R1 — Target union: earlier Blocked masks later Proven

Construct a target union whose canonical member order evaluates:

```text
Blocked
then
Proven
```

and assert final result must be `Proven`.

Also construct reversed allocation history and assert the same semantic result.

#### R2 — Source union: earlier Blocked masks later Refuted

Construct a source union with branch outcomes:

```text
Blocked
then
Refuted
```

and assert final result must be `Refuted`.

Also verify allocation-order independence.

#### R3 — Tuple labeled order

Assert:

```text
(x: Int, y: String) </: (y: String, x: Int)
```

while same-order covariance remains valid.

#### R4 — Tuple remainder cannot provide required fixed entry

Assert:

```text
(x: Int, **: String) </: (x: Int, y: String)
```

#### R5 — Pack labeled order

Create the first-class Pack analogue of R3.

#### R6 — Pack remainder cannot provide required fixed entry

Create the first-class Pack analogue of R4.

#### R7 — INC-010 bottom law through knowledge relation

Create a proper-type parameter `T`, then assert:

```text
check_subtype_bounded(Never, T) == Proven
check_knowledge_against_type_bounded(Known(Never), T) == Proven
```

The second assertion should fail before the fix if the old guard remains.

#### R8 — Parameter negative controls

Record pre/post required classification for:

```text
Known(T) against Int
Known(Int) against T
```

Neither may become fabricated `Proven`.

### Test placement

Prefer direct relation API tests. A source-level fixture is not required for the red phase if it would obscure the defect.

### Evidence requirement

Record exact command, selected count, pass/fail count, and failing assertion for every red oracle. Do not merely state “tests fail.”

---

## T2 — Implement canonical relation reduction primitives

### Objective

Introduce the reusable ALL/ANY relation combiner before changing the union branches.

### Requirements

The helper must:

- accept branch outcomes incrementally or from an iterator;
- distinguish logical outcomes from operational terminals;
- continue past `Blocked` when a decisive later result may exist;
- deterministically merge multiple blocked reasons;
- preserve `DynamicBoundary` distinctly;
- retain decisive branch proof/refutation information;
- not allocate/sort branch lists merely to choose an outcome;
- not reset relation budget/cancellation state.

### Unit matrix

Test reducer behavior independently if feasible:

#### ALL

```text
P + P -> P
P + B -> B
B + P -> B
B + R -> R
R + B -> R
R + P -> R
```

#### ANY

```text
R + R -> R
R + B -> B
B + R -> B
B + P -> P
P + B -> P
P + R -> P
```

where:

```text
P = Proven
R = Refuted
B = Blocked
```

Add operational terminal coverage.

### Gate

Do not migrate union recursion until reducer semantics have direct focused coverage.

---

## T3 — Migrate target union to canonical ANY semantics

### Objective

Repair INC-003.

### Current anti-pattern to remove

```text
for target member:
    Proven  -> return Proven
    Refuted -> continue
    anything else -> return immediately
```

### Required behavior

The target union branch must feed each `sub <: member` result into the canonical ANY reducer.

`Blocked` must not terminate the search while an unseen alternative may prove the relation.

### Required tests

- R1 green;
- `Refuted + Refuted -> Refuted`;
- `Refuted + Blocked -> Blocked`;
- `Blocked + Refuted -> Blocked`;
- 3+ member decisive-first/middle/last cases;
- allocation-order permutation cases;
- nested target union normalization case if union flattening already canonicalizes members.

### Performance requirement

Short-circuit immediately on `Proven`.

---

## T4 — Migrate source union to canonical ALL semantics

### Objective

Repair INC-008.

### Current anti-pattern to remove

```text
for source member:
    Proven  -> continue
    Refuted -> return Refuted
    anything else -> return immediately
```

### Required behavior

The source union branch must feed each `member <: sup` result into the canonical ALL reducer.

`Blocked` must not terminate the search while an unseen member may definitively refute the universal proposition.

### Required tests

- R2 green;
- `Proven + Proven -> Proven`;
- `Proven + Blocked -> Blocked`;
- `Blocked + Proven -> Blocked`;
- 3+ member decisive-first/middle/last cases;
- allocation-order permutation cases.

### Performance requirement

Short-circuit immediately on `Refuted`.

---

## T5 — Extract canonical ordered-lane relation helper

### Objective

Create one shared semantic algorithm before migrating Tuple and Pack.

### Required semantics

The helper must encode:

1. required fixed-prefix length;
2. pairwise element covariance;
3. label equality at corresponding position for labeled lane;
4. surplus source entry admission by target remainder;
5. remainder-to-remainder compatibility;
6. recursive relation outcome propagation without collapsing `Blocked`/operational states.

### Implementation constraints

- No hash map or label-search table is needed for ordered Tuple/Pack lanes.
- Do not reorder fixed entries.
- Do not materialize new Tuple/Pack types merely for relation checking.
- Do not change Record relation semantics.

### Gate

The helper must have enough direct tests or immediate Tuple migration coverage to prove fixed-prefix semantics before Pack migration.

---

## T6 — Migrate Tuple relation to the ordered helper

### Objective

Close the Tuple portion of INC-002 and the required-entry/remainder subfinding.

### Remove

Any labeled-lane logic equivalent to:

```rust
sub_lab.iter().find(|e| e.label == target_label)
```

and any path where source remainder is consulted because the source fixed prefix is shorter than the target fixed prefix.

### Required Tuple matrix

#### Fixed identity/order

```text
(x: Int, y: String) <: (x: Int, y: String)        Proven
(x: Int, y: String) <: (y: String, x: Int)        Refuted
```

#### Covariance

```text
(x: SubA, y: SubB) <: (x: A, y: B)                Proven when nested relations prove
```

#### Required prefix

```text
(x: Int) </: (x: Int, y: String)
(x: Int, **: String) </: (x: Int, y: String)
(**: String) </: (y: String)
```

#### Closed target surplus

```text
(x: Int, y: String) </: (x: Int)
```

#### Open target surplus

```text
(x: Int, y: String) <: (x: Int, **: ...)
```

and typed target remainder covariance.

#### Source openness

```text
(x: Int, **: String) </: (x: Int)
(x: Int, **: String) <: (x: Int, **: ObjectLikeTarget)
```

only when the typed remainder relation actually proves.

#### Lane independence

Test positional-open/labeled-closed, positional-closed/labeled-open, both-open, both-closed.

### Gate

All Tuple-specific red oracles from T1 must be green before Pack migration is considered complete.

---

## T7 — Migrate Pack relation to the same ordered helper

### Objective

Ensure first-class Pack and Tuple obey the same ordered lane law without sharing type identity.

### Required checks

Mirror the core Tuple cases for Pack:

- same labeled order succeeds;
- reversed labeled order fails;
- source remainder cannot satisfy target fixed entry;
- closed target rejects surplus fixed entries;
- open target admits surplus according to remainder;
- typed remainder covariance;
- independent positional/labeled lane openness.

### Negative identity control

Retain:

```text
Pack != Tuple
```

and ensure ordinary subtype relation between distinct Pack/Tuple constructors remains whatever the normative relation currently requires; do not create cross-kind structural equivalence merely because their lane algorithm is shared.

---

## T8 — Add Record negative-control regression

### Objective

Prove the ordered-product repair did not leak into unordered Record semantics.

### Required oracle

Construct semantically identical Records with fields presented in different order and verify canonical Record identity/relation remains order independent.

Also preserve required/optional/tail semantics already covered by existing Record tests.

### Gate

No shared helper may redefine Record field matching.

---

## T9 — Repair knowledge-to-contract parameter handling

### Objective

Close INC-010 without fabricating generic proofs or refutations.

### Implementation sequence

1. Preserve cancellation check.
2. Preserve `Dynamic` -> `DynamicBoundary` behavior.
3. Preserve `Unknown` -> `Blocked(UnknownType(...))` behavior.
4. Preserve exact same-type proof.
5. Record whether the relation is top-level parameter-sensitive.
6. Invoke canonical `check_subtype_bounded`.
7. Return canonical `Proven` unchanged.
8. Return canonical operational/unresolved outcomes unchanged.
9. Convert only parameter-sensitive unsupported hard refutation to unresolved `Blocked` according to the accepted P2 policy.
10. Leave closed concrete `Refuted` results unchanged.

### Required tests

```text
Never <: T                         Proven
Known(Never) against T             Proven
T <: T                             Proven
Known(T) against T                 Proven
Known(T) against unrelated Int     not Proven; unresolved according to policy
Known(Int) against T               not Proven; unresolved according to policy
Unknown against T                  Blocked
Dynamic against T                  DynamicBoundary
```

### Canonical Object control

If testing Object behavior, use:

```text
CoreDeclarationIds::default().object
```

or `universe_declaration(UniverseKey::Object)`.

Do not mutate the synthetic root-level Object test to stand in for canonical Object.

### STOP condition

If correctly preserving `T <: Int` uncertainty requires ambient generic evidence inside `check_knowledge_against_type_bounded`, stop and prepare consultation rather than widening the relation API speculatively.

---

## T10 — Audit direct relation consumers

### Objective

Verify that the corrected relation classes are consumed correctly and no caller relied on the old accidental early-return behavior.

### Mandatory consumers

Inspect and, where relevant, run focused tests for:

```text
checker/context.rs
checker/binding.rs
checker/field_lifecycle.rs
impls.rs
generic_evidence.rs
checker/coverage/usefulness.rs
checker/inference.rs
static_eval/machine.rs
```

Include focused consumer coverage for the implementation-witness checks in `impls.rs` and subtype-based candidate filtering and obligation selection in `checker/inference.rs`. These paths call `check_subtype_bounded` or `is_subtype` directly and must be reviewed for the corrected relation outcomes even when no consumer code change is needed.

### Required consumer properties

- `Refuted` still produces ordinary mismatch diagnostics where owned by checker policy.
- `Blocked` remains non-diagnostic unresolved state.
- `DynamicBoundary` remains distinct.
- `Cancelled`, `BudgetExceeded`, `InternalFailure` remain operational statuses.
- Generic subtype obligation checking automatically benefits from the canonical union reducer; no duplicate reducer is added.
- Binding reconciliation preserves the corrected INC-010 result classification.

### Change policy

Do not patch consumers merely to restore old test expectations. Change a consumer only when the corrected relation exposes an actual consumer bug.

Any newly discovered independent semantic defect gets its own incident record and does not silently expand P2 unless required for correctness of the P2 patch.

---

## T11 — Add source-level integration regressions

### Objective

Prove the canonical API repairs reach authored Telos semantics where a clean reproducer is available.

### Minimum desired integration set

1. Reversed labeled Tuple order is rejected in assignment/argument/contract context.
2. Same-order labeled Tuple covariance succeeds.
3. A Tuple source with only a typed labeled remainder cannot satisfy a required fixed labeled target entry.
4. A generic/proper-type symbolic contract accepts an expression whose established type is `Never`, when a source-level non-returning form provides a stable reproducer.

### Union source-level reproducer

Add one only if it naturally exercises the `Blocked + decisive` branch mix. Do not invent an artificial source program when the direct canonical API test is the authoritative oracle.

---

## T12 — Stabilization, performance, and acceptance evidence

### Objective

Demonstrate the patch is bounded, deterministic, and does not regress the relation engine's resource contracts.

### Required checks

- focused relation algebra tests;
- Tuple/Pack structural tests;
- parameter relation tests;
- relation budget tests;
- inheritance cycle/recursion tests;
- relevant generic evidence tests;
- narrow incremental/cold-equivalence test after P1 handoff if available;
- crate-level check/test target sufficient to prove compilation of affected consumers.

### Performance expectations

The patch must not:

- re-sort union alternatives per relation query;
- clone all union members unnecessarily;
- allocate hash maps for ordered labeled lanes;
- rebuild Tuple/Pack forms;
- reset relation budgets per branch.

Expected complexity:

```text
union reducer: O(number of visited alternatives)
Tuple/Pack fixed lanes: O(number of compared/surplus entries)
```

The ordered labeled repair should remove current repeated `.find()` matching and therefore can improve worst-case labeled-lane relation complexity.

---

## T13 — Incident closure evidence and lifecycle artifacts

### Objective

Close P2 only after implementation and verification evidence agree.

### Required incident disposition

For each:

```text
INC-002
INC-003
INC-008
INC-010
```

record:

- root cause;
- exact canonical owner fixed;
- discriminating regression IDs/tests;
- commands and results;
- source revision containing the fix;
- any remaining limitations;
- closure status.

Also record the required-entry/remainder subfinding explicitly, either as an amendment to INC-002 or as a separately registered incident if project convention requires independent incident IDs.

### Required P2 artifacts

Create:

```text
docs/implementation/TYPEAUD001/C1/TYPEAUD001.C1.P2-walkthrough.md
docs/implementation/TYPEAUD001/C1/TYPEAUD001.C1.P2-handoff.md
```

Update:

```text
TYPEAUD001.C1-CHECKPOINT.md
TYPEAUD001-STATUS.md
```

according to lifecycle convention.

Do not mark C1 complete from P2 completion alone.

---

# 7. Detailed conformance / verification matrix

The implementation plan should track the following CVs explicitly in its execution ledger.

## 7.1 Union reducer matrix

| ID | Proposition / branch pattern | Required outcome |
|---|---|---|
| CV-01 | ANY: `Proven` | `Proven` |
| CV-02 | ANY: `Blocked, Proven` | `Proven` |
| CV-03 | ANY: `Proven, Blocked` | `Proven` |
| CV-04 | ANY: `Refuted, Blocked` | `Blocked` |
| CV-05 | ANY: `Blocked, Refuted` | `Blocked` |
| CV-06 | ANY: all `Refuted` | `Refuted` |
| CV-07 | ALL: all `Proven` | `Proven` |
| CV-08 | ALL: `Blocked, Refuted` | `Refuted` |
| CV-09 | ALL: `Refuted, Blocked` | `Refuted` |
| CV-10 | ALL: `Proven, Blocked` | `Blocked` |
| CV-11 | ALL: `Blocked, Proven` | `Blocked` |
| CV-12 | multiple blocked reasons | deterministic/order-independent blocked result |
| CV-13 | operational `Cancelled` | preserved terminal |
| CV-14 | operational `BudgetExceeded` | preserved terminal |
| CV-15 | operational `InternalFailure` | preserved terminal |
| CV-16 | allocation-order permutation, target union | same semantic result |
| CV-17 | allocation-order permutation, source union | same semantic result |
| CV-18 | 3+ target alternatives, decisive last | correct ANY result |
| CV-19 | 3+ source members, decisive last | correct ALL result |

## 7.2 Tuple ordered-lane matrix

| ID | Case | Required outcome |
|---|---|---|
| CV-20 | same labeled order, equal element types | `Proven` |
| CV-21 | same labeled order, covariant element types | `Proven` |
| CV-22 | same labels reordered | `Refuted` |
| CV-23 | source fixed labeled prefix shorter than target | `Refuted` |
| CV-24 | source typed labeled remainder but missing target fixed entry | `Refuted` |
| CV-25 | source unconstrained labeled remainder but missing target fixed entry | `Refuted` |
| CV-26 | closed target with surplus source labeled fixed entry | `Refuted` |
| CV-27 | unconstrained open target admits surplus source labeled fixed entry | `Proven` |
| CV-28 | typed target remainder accepts compatible surplus | `Proven` |
| CV-29 | typed target remainder rejects incompatible surplus | `Refuted` or nested decisive non-proof according to relation |
| CV-30 | typed source remainder <: typed target remainder | follows nested `<:` |
| CV-31 | typed source remainder <: unconstrained target | `Proven` |
| CV-32 | unconstrained source remainder <: typed target | `Refuted` |
| CV-33 | unconstrained source remainder <: unconstrained target | `Proven` |
| CV-34 | open source labeled lane <: closed target labeled lane | `Refuted` |
| CV-35 | positional lane remains ordered and unaffected by labeled refactor | existing semantics preserved |
| CV-36 | independent positional/labeled openness | lane-local behavior preserved |

## 7.3 Pack parity matrix

| ID | Case | Required outcome |
|---|---|---|
| CV-37 | Pack same-order labeled prefix | `Proven` |
| CV-38 | Pack reordered labeled prefix | `Refuted` |
| CV-39 | Pack source remainder cannot satisfy fixed target entry | `Refuted` |
| CV-40 | Pack surplus/remainder matrix | same ordered-lane rules as Tuple |
| CV-41 | Pack remains distinct from Tuple | identity/relation distinction preserved |

## 7.4 Record negative controls

| ID | Case | Required outcome |
|---|---|---|
| CV-42 | same Record fields in different construction order | same canonical Record semantics |
| CV-43 | required/optional/tail semantics | no regression from ordered helper extraction |

## 7.5 Parameter/knowledge matrix

| ID | Case | Required outcome |
|---|---|---|
| CV-44 | canonical `Never <: T` | `Proven` |
| CV-45 | `Known(Never)` against `T` | `Proven` |
| CV-46 | `T <: T` | `Proven` |
| CV-47 | `Known(T)` against same `T` | `Proven` |
| CV-48 | `Known(T)` against unrelated `Int` | not fabricated `Proven`; unresolved according to accepted policy |
| CV-49 | `Known(Int)` against `T` | not fabricated `Proven`; unresolved according to accepted policy |
| CV-50 | `Unknown` against `T` | `Blocked(UnknownType(...))` |
| CV-51 | `Dynamic` against `T` | `DynamicBoundary` |
| CV-52 | canonical Universe Object identity | existing canonical behavior preserved |
| CV-53 | synthetic root `"Object"` | remains distinct from canonical Object |
| CV-54 | concrete incompatible non-parameter relation | remains `Refuted` |
| CV-55 | cancellation during knowledge relation | `Cancelled` preserved |
| CV-56 | budget exhaustion during knowledge relation | `BudgetExceeded` preserved |

## 7.6 Consumer/integration matrix

| ID | Case | Required outcome |
|---|---|---|
| CV-57 | checker relation application sees corrected `Proven` | no spurious Blocked status |
| CV-58 | checker sees corrected `Refuted` | mismatch diagnostic still emitted by owning policy |
| CV-59 | binding reconciliation | corrected relation class preserved |
| CV-60 | generic subtype proof uses canonical reducer | no parallel relation logic |
| CV-61 | coverage/static-eval operational states | category distinctions preserved |
| CV-62 | source-level reordered labeled Tuple | rejected |
| CV-63 | source-level same-order covariance | accepted |
| CV-64 | source-level missing fixed entry behind remainder | rejected |
| CV-65 | incremental/cold semantic parity | same relation-derived published semantics |

---

# 8. Verification gates

## G0 — Baseline and authority gate

**Pass criteria:**

- implementation HEAD pinned;
- P1 handoff/lifecycle state understood;
- relation owner/caller census recorded;
- no unreviewed semantic drift invalidates this plan;
- T1 red oracles established.

## G1 — Quantified reducer gate

**Pass criteria:**

- direct ALL/ANY matrix passes;
- Blocked does not prematurely decide quantified relations;
- operational terminals remain distinct;
- blocked reason aggregation is deterministic.

## G2 — Union relation gate

**Pass criteria:**

- INC-003 and INC-008 red oracles turn green;
- allocation/interner order does not alter completed semantic result;
- nested and 3+ member cases pass;
- budget/cancellation controls pass.

## G3 — Ordered Tuple gate

**Pass criteria:**

- labeled order significant;
- fixed target entries require fixed source presence;
- surplus/remainder rules match normative specification;
- lane independence preserved.

## G4 — Pack parity gate

**Pass criteria:**

- Pack uses the same ordered lane rules;
- required-entry/remainder bug fixed;
- Pack remains semantically distinct from Tuple.

## G5 — Record non-regression gate

**Pass criteria:**

- Record field order remains non-semantic;
- existing presence/tail behavior unaffected.

## G6 — Parameter proof-completeness gate

**Pass criteria:**

- `Never <: T` and `Known(Never)` against `T` both prove;
- same-parameter reflexivity proves;
- unrelated open-parameter relations are not fabricated into proof;
- concrete incompatible closed relations still refute;
- Unknown/Dynamic/operational states remain correctly classified.

## G7 — Consumer integration gate

**Pass criteria:**

- checker, binding, generic evidence, coverage/static evaluation consume corrected outcomes without category collapse;
- no duplicate relation algebra introduced outside canonical owner.

## G8 — Incremental/resource gate

**Pass criteria:**

- relation budget/depth tests pass;
- cycle/termination tests pass;
- narrow incremental/cold result parity passes or any unrelated baseline failure is explicitly classified;
- no evidence of material pathological relation-work growth beyond required exploration past Blocked branches.

## G9 — Lifecycle closure gate

**Pass criteria:**

- every mandatory CV has a disposition;
- all four P2 incidents have closure evidence;
- required-entry/remainder subfinding is durably recorded;
- walkthrough and handoff exist;
- checkpoint/status updated truthfully;
- no release-complete claim is made for the wider C1 program unless its own gate is satisfied.

---

# 9. Patch sequencing and file ownership

Recommended patch order:

```text
Patch A
  tests: red relation oracles

Patch B
  outcome/relation reducer helpers
  target/source union migration
  reducer and union tests green

Patch C
  ordered lane helper
  Tuple migration
  Tuple tests green

Patch D
  Pack migration
  Record negative control
  Pack/Record tests green

Patch E
  parameter proof-first knowledge relation
  INC-010 matrix green

Patch F
  consumer/source-level integration tests
  stabilization and lifecycle evidence
```

This segmentation keeps failures attributable and avoids mixing union algebra, structural products, and parameter proof-state logic in one undifferentiated diff.

If implementation is committed incrementally, each commit should compile and should not knowingly leave the canonical relation layer in an internally contradictory state.

---

# 10. Test execution strategy

Use the repository's pinned toolchain and current workflow guidance. Record exact commands rather than generic claims.

Preferred progression:

```text
1. single red oracle
2. smallest focused test module
3. related semantic test target
4. affected crate compilation/check
5. narrow integration set
6. checkpoint-required broader gate only after semantics stabilize
```

Do not run broad workspace suites after every edit.

When a test fails:

- classify whether it is the intended red oracle, a P2 regression, or pre-existing baseline;
- for semantic failures, record observation -> hypothesis -> discriminating test before speculative repair;
- after one serious correction for the same semantic failure, consult rather than repeatedly broadening the patch without evidence.

---

# 11. Performance and determinism requirements

## 11.1 Union

P2 may perform additional branch work only where the old implementation incorrectly stopped at `Blocked`.

Required properties:

- no re-sorting in relation hot path;
- no branch-vector cloning solely for reduction;
- same shared budget/cancellation object throughout recursion;
- ANY still stops on first `Proven`;
- ALL still stops on first `Refuted`.

## 11.2 Ordered Tuple/Pack

The new labeled fixed-prefix algorithm should be linear.

Do not retain repeated unordered label searches once order is normative.

Expected improvement:

```text
old labeled matching: potentially O(n^2)
new ordered matching: O(n)
```

for fixed labeled prefixes.

## 11.3 Determinism

A completed relation must not expose canonical `TypeId` allocation order through:

- `Proven` vs `Blocked`;
- `Refuted` vs `Blocked`;
- chosen blocked reason when several unresolved alternatives exist.

Operational termination due budget/cancellation remains intentionally resource/order sensitive only insofar as the computation does not complete.

---

# 12. STOP / CONSULT triggers

Stop source modification and prepare a consultation packet if any of the following occurs:

1. Correct union reduction requires a new public semantic meaning for `DynamicBoundary`.
2. There is no deterministic blocked-reason rule without changing public proof-state architecture.
3. Tuple or Pack representation has changed so the normative ordered-lane model no longer maps to `TypeData` as assumed.
4. Accepted tests/specification materially require unordered labeled Tuple or Pack semantics.
5. Fixing INC-010 requires passing `GenericEvidenceEnvironment` or inference state into the canonical knowledge relation API.
6. Correct parameter behavior cannot be expressed without redefining the semantics of all `TypeData::Parameter` relations.
7. Canonical Object behavior conflicts with an accepted specification rather than synthetic fixture identity.
8. P3 Dynamic work has landed and changes relation outcome precedence in a way that invalidates the P2 reducer contract.
9. P1's final handoff changes semantic publication/resource guarantees relied on by P2.
10. A broad consumer must be redesigned merely to accept the corrected canonical relation result.

The consultation packet must include:

- exact HEAD/worktree state;
- current task/gate;
- minimal failing proposition;
- relevant source with line numbers;
- normative authority;
- observed outcome and expected outcome;
- hypothesis;
- discriminating test;
- recommended architectural ruling;
- exact plan amendment required.

---

# 13. Incident-specific closure criteria

## INC-002

Close only when:

- Tuple labeled fixed entries compare by position + label;
- required target entries cannot be supplied by source anonymous remainder;
- Pack has the corresponding fix;
- valid same-order covariance and remainder rules pass;
- Record remains unordered;
- source-level or canonical integration evidence exists.

## INC-003

Close only when:

- target union ANY reduction continues past Blocked;
- later Proven dominates earlier epistemic blockage;
- allocation-order/permutation tests pass;
- operational terminals remain distinct.

## INC-008

Close only when:

- source union ALL reduction continues past Blocked;
- later Refuted dominates earlier epistemic blockage;
- allocation-order/permutation tests pass;
- operational terminals remain distinct.

## INC-010

Close only when:

- `Never <: T` is proven by the canonical relation;
- `Known(Never)` against `T` receives the same proof through knowledge-to-contract API;
- unrelated open-parameter relations are not fabricated into proof or ordinary closed-type contradiction;
- Unknown/Dynamic/operational categories remain distinct;
- test fixtures use canonical core identity where core behavior is asserted.

---

# 14. Completion deliverables

P2 implementation is complete only when all of the following exist and agree:

1. Canonical source correction.
2. Focused red-before/green-after incident tests.
3. Quantifier/order/property tests.
4. Tuple + Pack + Record control matrix.
5. Parameter proof-completeness matrix.
6. Consumer integration evidence.
7. Resource/termination evidence.
8. Per-incident closure records or checkpoint dispositions.
9. `TYPEAUD001.C1.P2-walkthrough.md`.
10. `TYPEAUD001.C1.P2-handoff.md`.
11. Updated C1 checkpoint.
12. Updated program status.

Do not use `COMPLETE` merely because source code compiles.

---

# 15. Definition of done

`TYPEAUD001.C1.P2` is done when the following statement is supported by executed evidence:

> The canonical Telos subtype engine evaluates source-union and target-union relations according to their universal and existential semantics rather than branch storage order; Tuple and Pack compare both fixed lanes as ordered required structural prefixes with correct typed/unconstrained remainder rules; anonymous remainders never fabricate required fixed entry presence; Record remains unordered; and knowledge-to-contract checking gives unconditional canonical subtype laws such as `Never <: T` a chance to prove before preserving unresolved generic-parameter uncertainty. Distinct proof, refutation, dynamic-boundary, blocked, cancellation, budget, and internal-failure outcomes remain semantically separate across canonical consumers.

Only after that statement is evidenced should INC-002, INC-003, INC-008, and INC-010 be marked closed and P2 be advanced to `COMPLETE`.

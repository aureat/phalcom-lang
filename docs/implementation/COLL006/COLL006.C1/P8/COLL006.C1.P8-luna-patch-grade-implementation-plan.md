---
id: COLL006.C1.P8
category: COLL
program: COLL006
checkpoint: COLL006.C1
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - COLL006.C1.P7
follows: COLL006.C1.P7
supersedes: null
repository: aureat/phalcom-lang
branch: main
prepared_against_revision: 73ea73f4622417510c39850e61664cf8c2c4a900
prepared_at: 2026-10-12
---

# COLL006.C1.P8 — Generic Intervals, Persistent Ordered Storage, and `RangeMap<K,V>`

## 0. Executor contract

This plan is a **Luna patch-grade implementation contract** for `COLL006.C1.P8`. It is intended for GPT-5.6 Luna High/Extra High execution under architectural supervision. It is not permission to redesign ordering, numeric `Range`, the collection hierarchy, the native floor, data representation, or the persistent-hash substrate.

The implementer:

- may adapt private helper names, file decomposition, imports, local Rust types, and equivalent mechanical APIs to the live repository;
- must preserve the semantic architecture, ownership boundaries, complexity contracts, GC model, callback/re-entry rules, and public invariants specified here;
- must not infer a public selector, error type, removal rule, normalization rule, or ordering contract that has not been made normative;
- must not invent or activate `Comparable`, `TotalOrder`, `Hashable`, or another collection-private ordering trait;
- must not generalize the existing numeric/progression `Range`;
- must not add any native binding unless an accepted amendment to TDR-0017 explicitly admits the exact binding;
- must prefer ordinary Telos source/data for `Bound<T>`, `Interval<T>`, interval algebra, and—if the T0 capability probes prove it viable—the persistent ordered tree itself;
- must use a P7-style GC-visible native substrate only as the explicitly gated fallback described in this plan;
- must never hide an unbounded `Value`-bearing persistent graph behind `Arc`, `Rc`, raw pointers, or an untraced host arena;
- must never hold a live `Heap`/object borrow across re-entrant Telos `<=>` or `==` callbacks;
- must preserve bilateral `<=>` semantics, including reflected comparison and `Ordering::Unordered`;
- must keep all persistent updates failure-atomic: user callback failure or validation failure may have external callback side effects, but must not publish a partially updated `RangeMap`;
- must follow the testing budget instead of running broad suites after every edit;
- must classify unrelated failures instead of opportunistically repairing them;
- must STOP AND CONSULT when a trigger in §13 or a task-local trigger fires;
- must keep `COLL006.C1-CHECKPOINT.md` current at durable task/gate boundaries;
- must produce `COLL006.C1.P8-walkthrough.md` and `COLL006.C1.P8-handoff.md` at completion.

Testing is evidence gathering. During BUILD mode, run the smallest discriminator capable of falsifying the current implementation hypothesis. Workspace/release certification remains P9-owned unless P8 causes a concrete cross-crate failure that cannot be classified more narrowly.

---

## 1. Goal

Implement a generic immutable interval model (`Bound<T>`, `Interval<T>`), one canonical interval-algebra authority, a structurally shared persistent balanced ordered tree, and an immutable `RangeMap<K,V>` that maps pairwise-disjoint intervals to values with **O(log n) point lookup**, **persistent old-root immutability**, **O(log n + k) range-splice/overlay architecture**, precise GC visibility, and callback-safe Telos ordering/equality semantics—without redesigning numeric `Range`.

---

## 2. Checkpoint acceptance objective

`COLL006.C1.P8` is one bounded plan inside `COLL006.C1`; it does **not** close the checkpoint.

P8 advances the C1 acceptance objective that:

```text
Bound<T> and Interval<T> model generic ordered-domain intervals
Interval semantics are independent of numeric progression/slicing Range
RangeMap<K,V> is a persistent mapping from non-overlapping Interval<K> regions to V
point lookup is containment-based and asymptotically O(log n)
persistent updates preserve old roots and structurally share unaffected nodes
persistent ordered nodes remain GC-visible
ordering/equality callbacks are re-entry safe
strict insertion cannot create overlap
overlay/split preserves exact endpoint semantics
randomized model agreement demonstrates semantic correctness
```

P9 remains responsible for whole-C1 convergence, including `Entry<K,V>` migration, broad conversion/API consistency, repository-wide certification, final residual classification, and release-level checkpoint closure.

P8 is therefore a **foundation + feature** plan:

```text
generic interval value semantics
        ↓
canonical endpoint/interval algebra
        ↓
persistent balanced ordered substrate
        ↓
RangeMap strict operations + range splicing
        ↓
normalization / logical equality / iteration
        ↓
focused GC, callback, randomized, and complexity evidence
```

---

## 3. Repository grounding

### 3.1 Prepared-against state

```text
repository:              aureat/phalcom-lang
branch:                  main
revision:                73ea73f4622417510c39850e61664cf8c2c4a900
revision message:        feat(LANG007): implement Product/Sum reflection and
                         compiler-derived structural capabilities (P5)
P7 completion revision:  c4b67217307a5eb588b8365819a7958e70dad75d
P7 status:               COMPLETE / IMPLEMENTED / FOCUSED_TESTED
P8 checkpoint state:     PROPOSED / NOT_STARTED / UNVERIFIED
native surface baseline: 488 generated declarations
```

The current `main` tip is one commit ahead of the P7 completion revision. The intervening merge adds LANG007.C4.P5 structural reflection/type-member capability work. It does **not** add `Bound`, `Interval`, `RangeMap`, persistent ordered storage, or new P8 native bindings. The generated native surface remains exactly `488` declarations at the prepared-against revision.

### 3.2 Current repository facts that constrain P8

1. `COLL006.C1-CHECKPOINT.md` names P8 as the active successor after completed P7 and still records P8 as `PROPOSED / NOT_STARTED / UNVERIFIED`.
2. `COLL006.C1-master-implementation-record.md` requires `Bound<T>`, `Interval<T>`, validity/empty semantics, containment/overlap/intersection/adjacency, persistent ordered storage, point lookup, strict insert, remove, overlay/split, a normalization/coalescing decision, unbounded endpoints, and custom ordered-domain tests.
3. The same master record fixes the principal gates: pairwise non-overlap, at most one point result, O(log n) point lookup, randomized reference-model agreement, and no forced redesign of numeric `Range`.
4. `COLL006.C1-detailed-implementation-plan.md` names a persistent AVL tree as the preferred C1 baseline and requires one interval kernel rather than duplicating endpoint semantics inside `RangeMap`.
5. `docs/specs/collection-families.md` currently specifies P6/P7 families through `Bijection<K,V>` and does **not** yet normatively define `Bound`, `Interval`, or `RangeMap`. Public P8 API/behavior therefore still needs an explicit normative update before production public selectors are frozen.
6. `phalcom-core/core/universe/src/collections/range.ph` remains the existing numeric/progression/slicing `Range`. It uses numeric lower/upper bounds, integer stepping, sequence slicing, `first`/`last`/`size`, and progression indexing. It is not the P8 generic interval abstraction.
7. `phalcom-core/core/universe/src/object/ordering.ph` defines the canonical four-state `Ordering` ADT: `Less`, `Equal`, `Greater`, `Unordered`.
8. Binary `<=>` is bilateral. Compiler lowering in `phalcom-core/src/compiler/lib/expr.rs::compile_bilateral_binary` evaluates operands once, honors direct/reflected candidates, validates an `Ordering` result, and reverses reflected ordering where required.
9. Semantic analysis in `phalcom-semantic/src/checker/expression.rs::apply_binary_operation_from_typed` resolves direct/reflected binary dispatch from available type evidence. Whether unconstrained generic `K` can directly express `a <=> b` is therefore a **T0 discriminator**, not an assumption.
10. P7 established `PersistentHashNode` as an immutable GC-visible heap node and a two-phase callback-safe persistent update architecture. That architecture is precedent for a native P8 fallback, but P8 must not modify or generalize the HAMT substrate.
11. P7 preserves `std::mem::size_of::<Object>() == 48` by boxing large persistent/public payloads. P8 must retain the live execution-start `Object` size ceiling.
12. `phalcom-native-surface/src/generated.rs` currently declares `GENERATED_PRIMITIVE_DECLARATION_COUNT = 488`.
13. TDR-0017 freezes the VM native primitive floor. P6 and P7 obtained exact amendments for their collection native bindings. P8 has no accepted native-floor amendment at the prepared-against revision.
14. TDR-0094 is already allocated to the `primitive` declaration-classification decision. P8 must allocate the next free TDR number from the live decision registry if a new amendment is required; do not assume a number from this plan.
15. `phalcom-native-meta/src/universe.rs`, `phalcom-semantic/src/core_surface/identity.rs`, `phalcom-core/src/universe/core_classes.rs`, and `phalcom-core/src/universe/invariants.rs` know about the current native collection families through Bag/Bijection but have no P8 RangeMap identity today.
16. `phalcom-core/core/universe/src/collections/package.ph` exposes current collection modules through `.bijection`, `.tuple`, `.pack`, `.record`, `.range`, and `.bytes`; no P8 module exists.
17. `phalcom-core/core/universe/src/errors/argument.ph` contains `ArgumentError`, `KeyError`, `IndexError`, `SliceError`, `DuplicateKeyError`, and `BijectionConflictError`; the proposed P8 error vocabulary is not yet present.
18. Ordinary `data`/sum-data support and the newly merged Product/Sum reflection work make a **source-first P8 representation materially more viable than it was in older planning snapshots**. P8 must prove this route before requesting native-floor expansion.

> **Execution rule:** re-read every named live path before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

### 3.3 Repository drift that must be rechecked at execution start

T0 must not assume that this prepared-against SHA is still HEAD. Record:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -1 --oneline
```

If `main` advanced:

- compare the new HEAD with `73ea73f4622417510c39850e61664cf8c2c4a900`;
- re-check P8 lifecycle state, collection spec, decision registry, comparison semantics, native-surface count, and relevant Universe/runtime files;
- adapt ordinary mechanical changes;
- STOP AND CONSULT only if the new changes materially alter P8 semantics, ownership, ordering authority, data/runtime representation, native-floor governance, or persistent-storage assumptions.

---

## 4. Required reads before implementation

### 4.1 Workflow and lifecycle

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/shared-consultation-escalation-protocol.md

docs/implementation/COLL006/PROGRAM.md
docs/implementation/COLL006/STATUS.md
docs/implementation/COLL006/COLL006.C1/COLL006.C1-CHECKPOINT.md
```

### 4.2 P8 architecture authorities

```text
docs/implementation/COLL006/COLL006.C1/COLL006.C1-master-implementation-record.md
docs/implementation/COLL006/COLL006.C1/COLL006.C1-detailed-implementation-plan.md

docs/implementation/COLL006/COLL006.C1/COLL006.C1.P7-walkthrough.md
docs/implementation/COLL006/COLL006.C1/COLL006.C1.P7-handoff.md
```

### 4.3 Normative specifications and decisions

```text
docs/specs/README.md
docs/specs/collection-families.md
docs/specs/objects/data-objects.md
docs/specs/source-declarations.md

docs/decisions/README.md
docs/decisions/accepted/0017-freeze-vm-blessed-primitive-floor.md
docs/decisions/accepted/0028-collections-representation-and-literals.md
docs/decisions/accepted/0033-amend-floor-admit-collection-container-primitives.md
docs/decisions/accepted/0047-non-moving-mark-sweep-collector.md
docs/decisions/accepted/0063-admit-bytes-native-octet-buffer.md
docs/decisions/accepted/0080-data-is-a-nominal-transparent-immutable-value-product.md
docs/decisions/accepted/0092-ordered-mutable-hash-collection-native-floor.md
docs/decisions/accepted/0093-persistent-hash-collection-native-floor.md
docs/decisions/accepted/0094-primitive-is-a-declaration-classification.md
```

If any decision has been superseded or renumbered in the live tree, follow the current decision registry.

### 4.4 Comparison and data semantics

```text
phalcom-core/core/universe/src/object/ordering.ph
phalcom-core/core/universe/src/object/object.ph
phalcom-core/src/compiler/lib/expr.rs
phalcom-semantic/src/checker/expression.rs

docs/internals/data-representation/03-data-classes-and-records.md
docs/internals/data-representation/04-adt-enum-variants.md
phalcom-core/src/vm/data.rs
```

### 4.5 Existing collection/runtime precedents

```text
phalcom-core/core/universe/src/collections/range.ph
phalcom-core/core/universe/src/collections/bag.ph
phalcom-core/core/universe/src/collections/bijection.ph
phalcom-core/core/universe/src/collections/package.ph

phalcom-core/src/heap/persistent_hash.rs
phalcom-core/src/primitive/persistent_hash.rs
phalcom-core/src/primitive/ordered_hash.rs
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/accessors.rs
phalcom-core/src/heap/trace.rs
phalcom-core/src/heap/mod.rs
phalcom-core/src/primitive/mod.rs

phalcom-native-meta/src/universe.rs
phalcom-semantic/src/core_surface/identity.rs
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/universe/invariants.rs
phalcom-modules/src/builtin.rs
phalcom-native-surface/src/generated.rs
```

### 4.6 Existing test patterns

```text
phalcom-core/tests/core/mod.rs
phalcom-core/tests/core/collections/persistent_hash.rs
phalcom-core/tests/core/collections/bag.rs
phalcom-core/tests/core/collections/bijection.rs
phalcom-core/tests/core/collections/contract.rs
```

Do not restart a repository-wide exploration if these owners and the current handoff agree. Expand inspection only when a concrete discriminator requires it.

---

## 5. Normative authority and decisions still required

### 5.1 Already fixed by C1

The following are already architectural requirements and are **not** open P8 design questions:

| Topic | Fixed requirement |
|---|---|
| `Range` | Existing numeric/progression/slicing `Range` remains distinct and is not redesigned. |
| `Interval` | Generic ordered-domain interval abstraction, not an iterable progression. |
| `RangeMap` | Persistent immutable point-to-value interval mapping. |
| Stored regions | Pairwise non-overlapping. |
| Point lookup | At most one result; architectural target O(log n). |
| Storage | Persistent balanced ordered structure; AVL is preferred C1 baseline. |
| Persistence | Old roots immutable; structural sharing required. |
| GC | Every persistent `Value`-bearing node reachable from a root must be visible to GC. |
| Comparison authority | Existing Telos comparison semantics; no new private ordering trait. |
| Re-entry | No heap/object borrow across Telos `<=>` or `==` callbacks. |
| Overlay complexity | Architecture must support O(log n + k), not repeated O(k log n) deletions. |
| P9 boundary | `Entry<K,V>` convergence remains P9-owned. |

### 5.2 Public API decisions that P8 must make normative before production API code

Before T2 freezes source declarations, update the owning normative specification and, where required, create a consultation record covering the following.

#### D-P8-01 — `Bound<T>` representation

Recommended:

```telos
data Bound<T> {
    Unbounded
    Included(_ value: T)
    Excluded(_ value: T)
}
```

The implementation may adapt exact current sum-data syntax, but the semantic states must remain exactly these three.

#### D-P8-02 — `Interval<T>` construction and invalid/empty semantics

Recommended model:

```telos
data Interval<T>(
    lower: Bound<T>,
    upper: Bound<T>,
)
```

Recommended distinction:

```text
valid nonempty     lower < upper, or lower == upper with both endpoints Included
valid empty        equal endpoint with at least one side Excluded
invalid            finite lower endpoint is ordered after finite upper endpoint
```

`Interval` itself may represent an empty interval as a value. `RangeMap` must never retain an empty interval. A descending interval should be rejected by map-changing operations rather than silently normalized or reversed.

If the public constructor itself is to reject descending intervals rather than allowing `isValid == false`, that is a public semantic choice and must be recorded explicitly.

#### D-P8-03 — `Ordering::Unordered`

The ordered-domain contract must specify that any comparison required to establish interval/tree order must yield `Less`, `Equal`, or `Greater`. Encountering `Unordered` must produce a deterministic failure; it must never be coerced into equality or an arbitrary branch.

The exact error class/message must be ratified. Do not invent it ad hoc in Rust.

#### D-P8-04 — ordering stability

A `K` retained as an endpoint must have stable ordering-relevant observable state for the lifetime of every `RangeMap` version that retains it. P8 cannot prove transitivity, antisymmetry, or mutation stability dynamically.

The spec must state this as an ordered-domain precondition. Do not attempt to solve it by introducing a language-wide ownership or immutability system.

#### D-P8-05 — strict insertion

Recommended semantic shape:

```text
inserting(interval, value)
    returns a new RangeMap
    rejects invalid/empty interval
    rejects any overlap with an existing interval
    leaves adjacency legal
```

The error for overlap should be a dedicated `RangeOverlapError` if that vocabulary is accepted. The exact selector spelling and error hierarchy must be fixed before implementation.

#### D-P8-06 — removal semantics

This is not yet sufficiently normative.

Recommended P8 behavior is **coverage removal**:

```text
removing(interval)
    subtracts the interval from mapped coverage
    drops fully-covered entries
    truncates partially-covered entries
    splits a containing entry into two remainders when necessary
```

This composes naturally with overlay and validates the required split machinery. If exact-entry removal is preferred instead, record that explicitly and provide another public operation for coverage subtraction if overlay still requires it internally.

Do not choose between these silently.

#### D-P8-07 — overlay

Recommended:

```text
overlaying(interval, value)
    replaces mapping over exactly interval
    preserves unaffected left/right remainders
    may remove k fully-covered entries
    returns a new RangeMap
```

Overlaying an invalid/empty interval must have an explicit rule. Recommended: invalid => error; empty => identity.

#### D-P8-08 — normalization/coalescing

Recommended mandatory normal form:

> Two neighboring entries with equal values must coalesce when their interval union is contiguous with no unmapped point.

Examples:

```text
[a,b) -> X   +   [b,c] -> X    => [a,c] -> X
[a,b] -> X   +   (b,c] -> X    => [a,c] -> X
[a,b) -> X   +   (b,c] -> X    => MUST NOT merge; point b is a gap
```

This rule makes a canonical logical partition possible. It invokes Telos `V == V`, so coalescing equality callbacks are part of the preflight/failure-atomicity contract.

If canonical coalescing is rejected/deferred, the spec must explicitly define equality/hash for multiple equivalent partitions before P8 can publish them.

#### D-P8-09 — equality/hash

Recommended if normalization is mandatory:

```text
RangeMap equality = same normalized in-order sequence of (Interval<K>, V)
RangeMap hash     = deterministic fold over the same logical sequence
```

Never hash tree shape, AVL heights, node handles, or allocation identity.

Any P8 hash implementation remains subject to the existing C1/P7 numeric hash-law residual; P8 does not own a general numeric-hash repair.

#### D-P8-10 — iteration/item shape

Before P9's `Entry<K,V>` convergence, recommended item shape:

```telos
(interval: Interval<K>, value: V)
```

Do not introduce a new P8-specific `Entry` migration.

### 5.3 Error vocabulary to ratify

Recommended additions under `ArgumentError` unless the owning spec chooses a more specific hierarchy:

```text
InvalidIntervalError
RangeOverlapError
```

The ordered-domain `Unordered` failure may use `InvalidIntervalError` only if the spec explicitly defines it as an interval-domain validation failure; otherwise use a separately ratified error or `ArgumentError`.

Callback errors thrown by user `<=>` or `==` must propagate unchanged.

---

## 6. Takeover state and stable interfaces

| Concept | Current owner/path | P8 use | Invariant |
|---|---|---|---|
| Canonical `Ordering` | `core/universe/src/object/ordering.ph` | endpoint/domain comparison | Four states including `Unordered`. |
| Bilateral compare lowering | `core/src/compiler/lib/expr.rs` | source `<=>` behavior | Direct/reflected semantics preserved; operands evaluated once. |
| Binary semantic selection | `phalcom-semantic/src/checker/expression.rs` | generic comparison capability | Do not reconstruct dispatch in P8 runtime code. |
| Numeric `Range` | `core/universe/src/collections/range.ph` | explicit non-owner/reference | Must remain progression/slicing abstraction. |
| Data runtime | `core/src/vm/data.rs` + data representation docs | source-first Bound/Interval/node route | Ordinary data is GC-managed and nominal. |
| Persistent HAMT precedent | `heap/persistent_hash.rs` | lifetime/path-copy precedent only | Do not merge ordered and hashed storage. |
| Persistent callback preflight | `primitive/persistent_hash.rs` | native fallback pattern | User callbacks before callback-free commit. |
| Mutable key callback engine | `primitive/ordered_hash.rs` | rooting/send precedent | No heap borrow across callback. |
| Heap object enum | `heap/object.rs` | native fallback only | Preserve execution-start slot-size ceiling. |
| GC tracing | `heap/trace.rs` | native fallback only | Every node edge exact and explicit. |
| Universe metadata | `phalcom-native-meta/src/universe.rs` | only where canonical/native identity requires it | Do not bless source data mechanically. |
| Core identity | `phalcom-semantic/src/core_surface/identity.rs` | only if semantic special handling is required | No gratuitous P8 special-casing. |
| Native floor | TDR-0017 + amendments | native fallback gate | No binding before accepted amendment. |
| P7 public collections | `bag.ph`, `bijection.ph` | source/native layering precedent | High-level behavior stays in source. |
| Native census | `phalcom-native-surface/src/generated.rs` | fallback verification baseline | Prepared baseline 488; execution baseline must be recounted. |

---

## 7. Architecture

### 7.1 Authority flow

P8 must keep semantic authority directional:

```text
canonical Telos ordering (<=> and Ordering)
        ↓
canonical source endpoint/interval algebra
        ↓
persistent ordered storage mechanics
        ↓
RangeMap operations
        ↓
source-visible collection behavior
```

A storage implementation may accelerate traversal/path copying, but it must not become a second interval-semantics authority.

### 7.2 Preferred architecture: Route A — source-first persistent AVL

**Route A is the first architecture to prove in T0.**

Use ordinary Telos source/data for:

```text
Bound<T>
Interval<T>
private persistent AVL node
RangeMap<K,V> wrapper/state
interval comparison helpers
lookup/insert/remove/overlay/split/join
normalization
iteration/equality/hash
```

Conceptually:

```telos
data Bound<T> {
    Unbounded
    Included(_ value: T)
    Excluded(_ value: T)
}

data Interval<T>(
    lower: Bound<T>,
    upper: Bound<T>,
)

data _RangeMapNode<K, V>(
    interval: Interval<K>,
    value: V,
    left: Option<_RangeMapNode<K, V>>,
    right: Option<_RangeMapNode<K, V>>,
    height: Int,
)
```

Exact syntax/recursion representation must follow live source capabilities. A node may need to hold child wrapper objects rather than directly recursive values if the current data runtime requires it. That is mechanically flexible if it remains GC-visible, immutable, structurally shared, and asymptotically equivalent.

Why Route A is preferred:

1. TDR-0017 explicitly favors library-level behavior above the native floor.
2. Current data support can represent immutable nominal products and recursive data.
3. Endpoint algebra can remain exclusively in source rather than being mirrored in Rust.
4. Ordinary source values are automatically part of the managed object graph.
5. `<=>` and `==` callbacks occur through normal language sends, avoiding native heap-borrow re-entry hazards.
6. A path-copy AVL does not intrinsically require VM primitives.
7. P8's required performance contracts are asymptotic, not a mandate for Rust-native storage.

Route A is accepted only if T0 proves:

```text
generic/source comparison can preserve bilateral <=> semantics
recursive generic immutable nodes are expressible and executable
RangeMap can expose the required Collection/Iterable surface without a new native identity requirement
path-copy construction does not require mutable aliasing or unsupported recursive construction
a source implementation can realize split/join/range-splice in O(log n + k)
GC retention/reclamation behaves correctly for shared roots
```

If those probes succeed, **do not add a P8 native-floor amendment merely for speed**.

### 7.3 Fallback architecture: Route B — GC-visible native ordered substrate

Route B is permitted only when T0 produces a concrete blocker for Route A and the resulting native boundary is reviewed/ratified.

Conceptual private node:

```rust
pub struct PersistentOrderedNode {
    // exact representation mechanically flexible
    interval: Value,           // preferred if canonical source Interval can be retained
    value: Value,
    left: Option<ObjRef>,
    right: Option<ObjRef>,
    height: u32,
}
```

or a boxed enum/struct with equivalent fields.

Required properties:

```text
Object::PersistentOrderedNode(Box<...>) or equivalent boxed arm
Object::RangeMap(Box<RangeMapObject>) only if a native public wrapper is necessary
children referenced by ObjRef
endpoints/interval/value referenced as traced Values
immutable nodes after publication
all old roots remain valid
all rotations/path copies allocate new nodes
```

Do not store a second native endpoint algebra. Prefer retaining canonical `Interval<K>` values and use precomputed traversal decisions/plans from callback phase A.

#### Route B two-phase rule

Any operation that can invoke user `<=>` or `==` must separate:

```text
Phase A — compare/validate/preflight
    may re-enter VM
    may allocate temporary values
    uses VM temp roots for handles/Values held across sends
    holds no Heap/object borrow across sends
    computes a complete mutation/splice plan

Phase B — commit
    &mut Heap only
    zero user callbacks
    path-copy / rotate / join / publish
```

For coalescing, **all required value-equality callbacks belong to Phase A**.

A callback failure ends Phase A and publishes no new `RangeMap` root.

### 7.4 Native-floor fallback gate

If Route B requires source-visible `native` methods, perform this sequence exactly:

```text
1. determine minimal mechanism-only selector inventory
2. write P8 API/native-floor consultation
3. allocate the next free TDR number from live docs/decisions/README.md
4. obtain accepted amendment to TDR-0017
5. only then author/register native declarations
6. regenerate native-surface catalog
7. verify generated/runtime/metadata parity
```

Do not predeclare an expected final count. Record the execution-start generated count, add the **accepted exact binding count**, and reconcile any unrelated concurrent catalog changes separately.

A likely consultation candidate, **not pre-approved by this plan**, is a narrow set analogous to P7:

```text
new
rawSize
rawGet
rawStrictInsert
rawRemove
rawOverlay
rawEntries
```

The final set may be smaller or mechanically different. It must not move interval algebra, high-level equality/hash, or iterator algorithms into Rust merely for convenience.

---

## 8. Ownership boundaries

### 8.1 P8 owns

- `Bound<T>` source-visible value semantics.
- `Interval<T>` source-visible value semantics.
- canonical endpoint and interval algebra.
- ordered-domain runtime contract required by `RangeMap`.
- persistent balanced ordered storage mechanism for RangeMap.
- `RangeMap<K,V>` source API and invariants.
- strict insertion.
- point lookup.
- ratified removal semantics.
- overlay/split/range-splice.
- normalization/coalescing if ratified.
- P8 collection errors.
- P8-focused GC/re-entry/property/performance evidence.
- P8 lifecycle records and focused native-floor decision if needed.

### 8.2 P8 does not own

- redesign of numeric/progression `Range`;
- new language-wide `Comparable`, `TotalOrder`, `Hashable`, or successor protocol;
- a new `Iterator`/`Iterable` hierarchy;
- `Entry<K,V>` convergence/migration;
- P7 HAMT redesign;
- P6 ordered-hash behavior changes;
- general numeric hash-law repair;
- a new ownership/borrow checker;
- arbitrary compiler optimizations for AVL/RangeMap;
- release-wide C1 certification;
- unrelated D1/D2/D4/P5 parser residual repair;
- opportunistic Product/Sum reflection redesign after the consolidated merge.

### 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| `<=>` dispatch semantics | language/compiler/semantic binary operator machinery | interval comparison | direct Rust `compare` send that bypasses reflected semantics |
| endpoint ordering rules | one P8 interval kernel | Interval + RangeMap | separate Rust and source implementations |
| interval emptiness/validity | one P8 interval kernel | insert/remove/overlay/tests | ad-hoc per-operation bound checks |
| tree order | lower-endpoint comparison using canonical kernel | AVL traversal | Rust `Ord`, pointer order, hash |
| map non-overlap | RangeMap mutation algorithms | lookup/iteration/equality | post-hoc cleanup |
| normalization | RangeMap mutation boundary | equality/hash/iteration | shape-dependent equality |
| persistent node lifetime | source data runtime or GC-visible native heap | all RangeMap versions | Arc/Rc/raw host graph |
| native selector inventory | accepted P8 TDR if Route B | source declarations/runtime metadata | unratified helper bindings |
| public errors/selectors | normative collection spec | source/runtime/tests | test-only invented behavior |

---

## 9. Global invariants

Reference these IDs in code comments/tests where useful.

```text
INV-01 — Existing numeric Range semantics remain unchanged.

INV-02 — Bound<T> has exactly Unbounded, Included(T), and Excluded(T)
         semantic states; endpoint role supplies -∞/+∞ interpretation.

INV-03 — One canonical interval kernel owns lower/upper ordering,
         emptiness/validity, containment, overlap, intersection,
         contiguity, and union/split endpoint rules.

INV-04 — P8 uses canonical bilateral Telos <=> semantics. It does not
         reconstruct ordering from <, >, ==, Rust Ord, hashes, or identity.

INV-05 — Any comparison required for tree/interval order must yield
         Less/Equal/Greater. Unordered is never silently coerced.

INV-06 — A stored RangeMap interval is valid and nonempty.

INV-07 — Stored RangeMap intervals are pairwise non-overlapping.

INV-08 — In-order traversal is strictly ordered by canonical lower-bound order.

INV-09 — Any point belongs to at most one stored interval.

INV-10 — lookup(point) returns Some(value) iff exactly one stored interval
         contains point; otherwise None.

INV-11 — Point lookup traverses a balanced tree and is O(log n); no linear scan.

INV-12 — Persistent updates never mutate an existing node reachable from an
         older RangeMap root.

INV-13 — Every successful update structurally shares unaffected subtrees.

INV-14 — AVL height metadata and balance factors are correct after every
         insert/remove/split/join/rotation.

INV-15 — Overlay/removal range splicing is architected as O(log n + k), not
         k independent O(log n) updates.

INV-16 — All persistent Value/ObjRef edges are GC-visible and precisely traced.

INV-17 — No Heap/object borrow survives across a re-entrant Telos <=> or ==
         callback.

INV-18 — Callback/validation failure publishes no partial new RangeMap root.

INV-19 — If coalescing is ratified, no two neighboring entries with equal
         values remain separately stored when their intervals have no gap.

INV-20 — Equality/hash, if published by P8, depend on normalized logical
         entries, never AVL topology, heights, ObjRefs, or allocation identity.

INV-21 — Ordered-domain endpoint behavior must remain stable while retained by
         any live RangeMap version.

INV-22 — P8 does not require successor/predecessor arithmetic. Generic
         adjacency means exact boundary contiguity, not x+1-style adjacency.

INV-23 — Object arena slot size does not grow beyond the execution-start
         accepted size; native fallback payloads are boxed as needed.

INV-24 — No P8 native declaration is installed before exact native-floor
         admission is accepted.

INV-25 — Entry<K,V> migration is not pulled forward from P9.
```

---

## 10. Canonical interval algebra

This section is implementation guidance that becomes binding once T1 records the corresponding public semantics.

### 10.1 Context-sensitive `Unbounded`

`Bound<T>::Unbounded` has no single global position independent of endpoint role:

```text
lower Unbounded = -∞
upper Unbounded = +∞
```

Therefore use distinct endpoint-role comparisons:

```text
compareLower(a, b)
compareUpper(a, b)
```

Do not define RangeMap ordering by a context-free `Bound<T>.compare` unless the normative API deliberately specifies such an order separately.

### 10.2 Same-value endpoint tie breaking

For equal underlying finite values `x`:

```text
lower-bound order:
    Included(x) < Excluded(x)

upper-bound order:
    Excluded(x) < Included(x)
```

Including infinities:

```text
lower:
    Unbounded < Included(x) < Excluded(x)

upper:
    Excluded(x) < Included(x) < Unbounded
```

For different finite endpoint values, compare their payloads using canonical `<=>`.

### 10.3 Containment

For point `p`:

```text
lower Unbounded       => true
lower Included(x)     => p >= x
lower Excluded(x)     => p >  x

upper Unbounded       => true
upper Included(x)     => p <= x
upper Excluded(x)     => p <  x
```

Implement these through one comparison result per boundary where possible; do not send `<`, `>`, and `==` independently.

### 10.4 Equal-bound emptiness matrix

At the same endpoint value `x`:

| Interval | Result |
|---|---|
| `[x,x]` | singleton, nonempty |
| `[x,x)` | empty |
| `(x,x]` | empty |
| `(x,x)` | empty |

A finite lower greater than finite upper is invalid under the recommended model.

### 10.5 Intersection

Compute:

```text
result.lower = later/max lower bound
result.upper = earlier/min upper bound
```

Then classify validity/emptiness through the same interval kernel.

Recommended public return:

```text
Option<Interval<T>>
```

where `None` means no points are shared. Do not use an arbitrary sentinel interval.

### 10.6 Generic contiguity / adjacency

Do **not** define generic adjacency through successor arithmetic.

Two intervals touch without an unmapped point only when their finite boundary payloads compare `Equal` and the inclusion combination covers that boundary point.

Examples:

```text
[a,b) + [b,c]    contiguous
[a,b] + (b,c]    contiguous
[a,b] + [b,c]    overlapping at b, not merely adjacent
[a,b) + (b,c]    gap at b
```

Keep `overlaps` and `touchesWithoutGap` distinct.

### 10.7 Split complement

When subtracting/overlaying a region at finite boundary `x`:

```text
overlay lower Included(x)  => left remainder upper Excluded(x)
overlay lower Excluded(x)  => left remainder upper Included(x)

overlay upper Included(x)  => right remainder lower Excluded(x)
overlay upper Excluded(x)  => right remainder lower Included(x)
```

An unbounded overlay side leaves no remainder on that side.

These complement rules must be centralized in the interval kernel and reused by remove/overlay.

---

## 11. Expected impact map

### 11.1 Expected source/API areas

Likely Route A files:

```text
phalcom-core/core/universe/src/collections/interval.ph          # create
phalcom-core/core/universe/src/collections/range_map.ph         # create
phalcom-core/core/universe/src/collections/package.ph           # expose
phalcom-core/core/universe/src/errors/argument.ph               # P8 errors if ratified
phalcom-core/core/universe/src/errors/package.ph                # re-export
phalcom-modules/src/builtin.rs                                  # builtin module registration if required
docs/specs/collection-families.md                               # normative P8 contract
```

Potential metadata/identity files **only if the live source-loading/type system requires them**:

```text
phalcom-native-meta/src/universe.rs
phalcom-semantic/src/core_surface/identity.rs
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/universe/invariants.rs
```

Do not add P8 types to these mechanically. Ordinary source `data` declarations should remain ordinary source declarations unless a real canonical/native consumer requires `UniverseKey`/core-class identity.

### 11.2 Route B-only runtime areas

If and only if Route B is accepted:

```text
phalcom-core/src/heap/persistent_ordered.rs                     # create
phalcom-core/src/heap/range_map.rs                              # create if native wrapper needed
phalcom-core/src/heap/mod.rs
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/accessors.rs
phalcom-core/src/heap/trace.rs

phalcom-core/src/primitive/persistent_ordered.rs                # create
phalcom-core/src/primitive/range_map.rs                         # create
phalcom-core/src/primitive/mod.rs

phalcom-native-meta/src/universe.rs
phalcom-semantic/src/core_surface/identity.rs                   # only if required
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/universe/invariants.rs
phalcom-core/src/value/mod.rs
phalcom-core/src/value/render.rs

phalcom-native-surface/src/generated.rs                         # generated, never hand-edited
docs/decisions/accepted/<next-free>-...md                       # after consultation
docs/decisions/README.md
```

### 11.3 Expected tests

```text
phalcom-core/tests/core/mod.rs                                  # register new modules
phalcom-core/tests/core/collections/interval.rs                 # create
phalcom-core/tests/core/collections/range_map.rs                # create
phalcom-core/tests/core/collections/persistent_ordered.rs       # Route B or substrate-specific tests
phalcom-core/tests/core/collections/contract.rs                 # only applicable cross-family contract additions
```

Potential semantic/compiler discriminator tests should live beside the existing binary/generic dispatch tests rather than creating a collection-private semantic framework.

### 11.4 Documentation/state

```text
docs/implementation/COLL006/COLL006.C1/COLL006.C1-CHECKPOINT.md
docs/implementation/COLL006/COLL006.C1/COLL006.C1.P8-api-and-floor-consultation.md   # if consultation required
docs/implementation/COLL006/COLL006.C1/COLL006.C1.P8-walkthrough.md
docs/implementation/COLL006/COLL006.C1/COLL006.C1.P8-handoff.md
```

### 11.5 Unexpected-touch rule

Touch adjacent helpers/tests when mechanically required by the chosen route. STOP AND CONSULT before entering a materially different compiler/type-system/GC/native subsystem to make P8 possible when that change alters an authority or architectural boundary.

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer may not change:

- separation of generic `Interval` from numeric `Range`;
- canonical bilateral `<=>` semantics;
- absence of a new `Comparable`/`TotalOrder` trait;
- non-overlap invariant;
- persistent old-root immutability;
- precise GC visibility;
- callback-safe update publication;
- O(log n) point lookup architecture;
- O(log n + k) overlay/range-splice architecture;
- AVL-class balancing unless an explicit consultation approves another persistent balanced tree;
- one interval-algebra authority;
- TDR gate before new native bindings;
- P9 ownership of `Entry<K,V>` convergence.

### 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- current sum-data syntax (`data` vs equivalent current declaration form);
- private helper/type names;
- whether `Bound` and `Interval` share one source module or two;
- exact AVL node representation;
- source recursive child representation;
- iterative vs recursive internal traversal;
- height integer host type in private native nodes;
- source iterator helper class names;
- exact test module names/filters after verifying live test discovery;
- boxing/decomposition that preserves the current Object slot ceiling;
- equivalent split/join algorithm formulation;
- native fallback selector spelling **only inside the consultation before ratification**.

### 12.3 VERIFY-FIRST

| Assumption | Verify in | If false |
|---|---|---|
| Generic `K` code can express `a <=> b` | T0 semantic/source probe | test Dynamic source bridge; if still impossible, STOP AND CONSULT |
| A Dynamic source helper preserves bilateral/reflected `<=>` | T0 reflected-comparison probe | STOP; do not replace with direct native `compare` send |
| Ordinary recursive generic data nodes work | T0 source AVL probe | evaluate Route B; do not invent host graph |
| Source RangeMap can participate in current Collection/Iterable model | T0 minimal source class probe | determine exact identity requirement; consult if new native/core identity is required |
| Source path-copy AVL can meet structural-sharing/complexity requirements | T0/T3 prototype | Route B only with concrete evidence + floor decision |
| `Bound`/`Interval` need no `UniverseKey` | T0 source-loading/typing probe | add only the minimal canonical metadata actually required |
| Current Object size is still 48 | T0 Rust size test/current P7 test | record new accepted live baseline; STOP on unexplained growth |
| Native surface remains 488 | T0 generator/current generated constant | record live value; do not force 488 if unrelated accepted changes landed |
| P8 error names remain unratified | T0 spec/decision census | consume newer authority if present |
| P8 TDR number remains unallocated | T1 decision registry | allocate next free ID live |
| Coalescing is still preferred, not normative | T1 spec census | consume newer ruling if one exists |
| Removal semantics remain unspecified | T1 spec census | consume newer ruling if one exists |
| Existing Product/Sum merge does not alter P8 data construction | T0 minimal data probes | classify whether P8 architecture must adapt; consult on material semantic conflict |

---

## 13. Global STOP / CONSULT triggers

STOP editing and prepare a consultation packet if any of the following occurs:

1. Generic ordered-domain comparison cannot be expressed while preserving canonical bilateral `<=>` without a new language-wide trait/type-system foundation.
2. The only workable runtime comparison path would bypass reflected comparison or reinterpret `Ordering::Unordered`.
3. P8 would require modifying numeric `Range` semantics or representation.
4. Route A fails for a reason that would require a material new native boundary; do not move to Route B without consultation/floor governance.
5. New native bindings are needed but no accepted TDR-0017 amendment exists.
6. Persistent Value-bearing nodes cannot be represented in the managed GC graph without a new memory subsystem.
7. A callback path would hold a live `Heap`/object borrow across `<=>` or `==`.
8. O(log n) point lookup cannot be preserved.
9. Overlay/removal remains O(k log n) because the implementation cannot provide split/join/range-splice.
10. Empty/invalid interval behavior, removal semantics, overlap error, `Unordered` behavior, or normalization cannot be made normative from current authorities.
11. Coalescing/equality semantics require a public contract materially different from §5 recommendations and no accepted ruling exists.
12. Endpoint mutability/order stability can only be made sound by adding a new ownership/immutability/type-system mechanism.
13. A new Object variant materially increases every heap slot despite reasonable boxing.
14. The same nontrivial semantic failure survives one well-grounded correction.
15. Tests can only pass by weakening non-overlap, persistence, comparison, GC, callback, or complexity invariants.
16. P8 begins absorbing `Entry<K,V>`, P9-wide migration, numeric hash-law repair, or unrelated C1 residual work.
17. The consolidated LANG007 structural-reflection changes expose a real semantic conflict that requires changing P8's public data model rather than adapting mechanics.
18. Two plausible fixes have materially different semantic authority or lifetime architecture.

A triggered consultation cannot be self-waived.

---

## 14. Debugging budget

### 14.1 Mechanical failures

Allow up to three coherent cycles while evidence shows progress:

```text
inspect
identify one concrete mechanical cause
make one coherent correction
rerun the smallest discriminator
```

Examples:

```text
module registration omission
source syntax drift
missing exhaustive Object match arm
generic type metadata wiring
test filter name mismatch
borrow checker refactor that preserves architecture
```

### 14.2 Semantic failures

Before editing, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious correction for the same underlying semantic failure. If it persists, STOP AND CONSULT.

Semantic examples:

```text
reflected comparison returns wrong orientation
boundary inclusion lost during split
overlap admitted at an included endpoint
old root changes after rotation
Unordered silently chooses branch
coalescing changes mapped domain
```

### 14.3 Architectural failures

Zero speculative architecture-fix attempts. Consult immediately.

Do not rerun an unchanged failing test unless relevant code/inputs changed.

---

## 15. Testing surface analysis

### 15.1 Coverage matrix

| ID | Dimension | Invariants | Required evidence |
|---|---|---|---|
| CV-01 | Bound construction | 02 | all three variants, generic payload |
| CV-02 | lower-bound ordering | 03–05 | finite, equal-value inclusion tie, unbounded |
| CV-03 | upper-bound ordering | 03–05 | finite, equal-value inclusion tie, unbounded |
| CV-04 | interval validity/emptiness | 03,06 | `[x,x]`, `[x,x)`, `(x,x]`, `(x,x)`, descending |
| CV-05 | contains | 03–05 | before/at/interior/upper/after for all inclusion forms |
| CV-06 | overlap/intersection | 03 | disjoint, touching, singleton, containment, unbounded |
| CV-07 | contiguity | 03,22 | no-successor boundary matrix |
| CV-08 | split complements | 03 | all four Included/Excluded complement cases |
| CV-09 | custom ordered domain | 04–05,21 | nonnumeric user type with `compare` |
| CV-10 | reflected comparison | 04 | source `<=>` honors reflected candidate |
| CV-11 | Unordered | 05,18 | deterministic rejected operation, old root intact |
| CV-12 | comparator callback error | 17–18 | error propagates; no partial root |
| CV-13 | comparator allocation/GC | 16–18 | callback may allocate/force GC safely |
| CV-14 | AVL rotations | 12–14 | LL, RR, LR, RL |
| CV-15 | AVL deletion/rebalance | 12–14 | leaf, one-child, two-child, root, cascading rebalance |
| CV-16 | structural sharing | 12–13 | unaffected subtree handles/identity shared where observable in substrate tests |
| CV-17 | point lookup | 07–11 | 0/1 result, boundary points, unbounded |
| CV-18 | strict insert | 06–10,18 | disjoint success, adjacency success, every overlap rejection |
| CV-19 | removal | 06–10,15 | exact, partial-left, partial-right, middle split, multi-entry, unbounded |
| CV-20 | overlay | 06–10,15 | no-overlap, partials, containment, multi-entry, unbounded |
| CV-21 | normalization | 19 | mergeable same-value neighbors and nonmergeable gap |
| CV-22 | value equality callback | 17–19 | mutation/throw/GC during coalescing equality |
| CV-23 | iteration/logical entries | 08,20,25 | sorted normalized labeled tuples |
| CV-24 | equality/hash | 19–20 | same logical map via different operation histories |
| CV-25 | multi-version GC | 12–16 | shared roots survive; unreachable unique nodes reclaim |
| CV-26 | randomized oracle | 06–15,19 | generated operation sequences vs sorted reference model |
| CV-27 | lookup complexity | 11,14 | comparator/visit counts grow logarithmically |
| CV-28 | overlay complexity | 15 | comparator/visit counts approximate log n + k, not k log n |
| CV-29 | Range firewall | 01 | existing numeric Range focused tests unchanged |
| CV-30 | native floor | 24 | zero additions Route A, exact approved delta Route B |
| CV-31 | Object size | 23 | execution baseline retained |
| CV-32 | Product/Sum merge regression | 02–04 | ordinary P8 data compiles/constructs/reflects without semantic regression |

### 15.2 Required custom ordered-domain fixture

Use a nonnumeric domain, conceptually:

```telos
data TestDate(_ ordinal: Int) {
    compare(_ other: TestDate) -> Ordering {
        ordinal <=> other.ordinal
    }
}
```

Do not rely only on `Int`, `Float`, or numeric `Range`; that would fail to prove generic ordered-domain behavior.

### 15.3 Comparison callback fixtures

At least one fixture must support:

```text
increment comparison counter
allocate during compare
optionally trigger/encourage GC
return Ordering::Unordered for selected pairs
throw a user Error for selected pairs
```

A separate fixture/probe must exercise reflected `<=>` selection. Do not approximate this by calling `compare` directly.

### 15.4 Randomized reference model

The oracle must be intentionally simple, not another tree:

```text
Vec<(ModelInterval, ModelValue)>
sorted by canonical lower-bound order
linear lookup
linear overlap/remove/overlay
explicit normalization
```

Generate operation sequences over a small finite model domain, including unbounded and all endpoint inclusion forms. For each successful step compare:

```text
logical ordered entries
size / isEmpty
sampled or exhaustive point lookup over model domain
non-overlap invariant
normalization invariant
operation error classification
selected retained old versions
```

Use deterministic seeds and print/retain the seed on failure.

---

## 16. Verification execution budget

### 16.1 Modes

#### BUILD

Run exact probes and focused tests only.

#### STABILIZE

Run all P8 focused suites plus directly affected collection/native/semantic suites.

#### CERTIFY

P8 final focused acceptance. Do not automatically run full workspace/release certification; P9 owns that.

### 16.2 Verification ladder

```text
T0  exact source/semantic capability discriminator
T1  interval focused tests
T2  persistent ordered substrate tests
T3  RangeMap focused tests
T4  affected collection + GC/native integration
T5  crate-level checks / generated-surface check
T6  workspace/release only if concrete P8 evidence demands it
```

### 16.3 Baseline commands

At T0, discover current filters first:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core -- --list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic -- --list
```

Then establish predecessor/current baseline with the smallest relevant lanes:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-native-surface-gen -- --root . --check

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_persistent_hash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_bag
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_bijection
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_collections
```

Adapt filter spellings to `-- --list`; do not treat historical filter names as sacred.

### 16.4 BUILD-mode prohibition

Do not repeatedly run:

```text
cargo test --workspace
cargo clippy --workspace
scripts/verify.sh --full
full semantic corpus
full compiler corpus
```

during local P8 construction. These do not discriminate endpoint/tree defects efficiently.

### 16.5 Stabilization commands

At final task gates, expected commands include:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_interval -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_range_map -- --nocapture

# Route B only, or if a dedicated substrate test module exists:
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_persistent_ordered -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_collections -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core memory_gc -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --all-targets
cargo fmt --all -- --check
git diff --check
```

If Route B lands native bindings:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-native-surface-gen -- --root . --check
```

If P8 does not modify `phalcom-semantic`, the semantic all-target check is optional at final stabilization after T0 generic comparison probes pass.

---

## 17. Baseline / unrelated failure policy

Classify failures:

```text
A — definitely P8-caused
B — probably P8-caused
C — ownership unclear
D — clearly unrelated/baseline
```

- A/B: repair under P8.
- C: perform one bounded comparative classification pass.
- D: record and continue.
- Never weaken assertions to make a broad gate green.

Known inherited residual categories such as D1/D2/D4, the P5 parser integration residual, and `R-P7-NUMERIC-HASH-LAW` remain outside P8 unless new comparative evidence establishes P8 causality.

Do not turn an inherited ignored numeric hash-law test into P8 work merely because `RangeMap.hash` exercises child hashes. Record the inherited limitation consistently and leave its general repair to the existing owner.

---

# 18. Tasks

## T0 — Rebaseline, capability probes, and architecture-route gate

### Purpose

Prove the live repository assumptions that determine whether P8 can remain source-first, and prevent implementation from beginning on a stale or impossible architecture.

### Preconditions

- P7 remains complete in the checkpoint.
- Work begins on `main` unless the user explicitly directs otherwise.
- No production P8 edits yet.

### Consumes

- current checkpoint/master/detailed plan;
- P7 walkthrough/handoff;
- current comparison/data/runtime source;
- current decision/native catalog.

### Produces

A recorded P8 execution baseline and a route decision:

```text
Route A — source-first persistent AVL
or
Route B candidate — requires consultation/native-floor approval before native implementation
```

### Read

All §4 required-read files, prioritizing:

```text
COLL006.C1-CHECKPOINT.md
COLL006.C1.P7-handoff.md
collection-families.md
ordering.ph
object.ph
compiler/lib/expr.rs
semantic/checker/expression.rs
vm/data.rs
persistent_hash.rs
primitive/persistent_hash.rs
0017-...
0093-...
docs/decisions/README.md
phalcom-native-surface/src/generated.rs
```

### Probes to add/run

#### T0-P1 — generic `<=>`

Compile/run a minimal generic probe:

```telos
class GenericComparisonProbe<K> {
    compare(_ a: K, _ b: K) -> Ordering {
        a <=> b
    }
}
```

Use a concrete custom ordered type to instantiate/call it.

#### T0-P2 — Dynamic bridge

If P1 is rejected semantically, test:

```telos
class DynamicComparisonProbe {
    @class
    compare(_ a: Dynamic, _ b: Dynamic) -> Ordering {
        a <=> b
    }
}
```

This is acceptable only if it executes the same bilateral operator machinery.

#### T0-P3 — reflected compare

Construct a focused case proving that the source `<=>` expression honors reflected selection/orientation. This must fail if someone replaces the helper with a one-sided `candidate.compare(query)` send.

#### T0-P4 — `Unordered` and throw

Prove that callbacks can return `Ordering::Unordered` and throw a user error, and record current behavior of the raw `<=>` expression. P8 later adds ordered-domain validation above this.

#### T0-P5 — ordinary sum/data declaration

Prototype `Bound<T>` with current source syntax and a minimal `Interval<T>` product. Compile, construct, pattern match, and access generics.

#### T0-P6 — recursive generic immutable node

Prototype a tiny private recursive generic data/node with left/right options and an integer height. Construct two path-copied versions and confirm the old version is unchanged.

#### T0-P7 — source collection wrapper

Prototype the minimal class/data arrangement required for a generic `RangeMap<K,V>` to be a `Collection`/`Iterable` while holding an immutable persistent root. Do not implement full API.

#### T0-P8 — baseline census

Record:

```text
current HEAD
native generated declaration count
Object size assertion/current result
P7 focused tests
numeric Range focused baseline
current P8 spec/decision gaps
```

### Acceptance

Route A is selected if all of these are true:

```text
bilateral <=> can be invoked correctly from source
recursive generic source node construction works
persistent root can be represented without mutable aliasing
RangeMap can expose collection identity/protocols
no native primitive is required for correctness
```

If Route A fails, record the exact failing capability and STOP for T1 consultation. Do not immediately implement Route B.

### Tests to run now

Only the new discriminators and P7/native baseline commands in §16.3.

### Checkpoint update

Mark P8 `IN_PROGRESS` only after baseline and route decision are recorded. Add any material blocker/consultation trigger.

---

## T1 — Ratify the P8 public contract and, only if necessary, native fallback floor

### Purpose

Make the missing public semantics normative before production code freezes them.

### Preconditions

- T0 route evidence exists.
- Current `collection-families.md` still lacks P8 semantics, or a newer spec owner has been identified.

### Produces

A normative P8 section covering all decisions in §5 and, for Route B only, an accepted exact native-floor amendment.

### Modify

Primary:

```text
docs/specs/collection-families.md
```

Potential consultation/state:

```text
docs/implementation/COLL006/COLL006.C1/COLL006.C1.P8-api-and-floor-consultation.md
docs/implementation/COLL006/COLL006.C1/COLL006.C1-CHECKPOINT.md
```

Route B only, after approval:

```text
docs/decisions/accepted/<next-free>-persistent-ordered-rangemap-native-floor.md
docs/decisions/README.md
```

### Required contract sections

Specify:

```text
Bound variants
Interval validity and emptiness
contains / overlaps / intersection / touchesWithoutGap / unionIfContiguous
Unbounded endpoint semantics
Unordered handling
ordered-domain stability precondition
RangeMap constructor/empty behavior
size/isEmpty
point get/lookup
strict insertion
overlap error
removal semantics
overlay semantics
normalization/coalescing
iterator/entries shape
equality/hash policy
callback error propagation
persistence/failure atomicity
complexity promises
numeric Range non-interference
```

### Route B floor consultation

The packet must contain:

```text
why Route A failed
why a native mechanism is necessary for correctness/performance contract
exact proposed selectors
which responsibilities stay in source
heap/GC representation
Phase A/Phase B callback safety
execution-start native count
expected delta from approved selectors
Object size strategy
test obligations
```

No code-level native declaration may land before acceptance.

### Forbidden approaches

- “temporarily” inventing selector/error behavior in tests;
- accepting `Unordered` by arbitrary ordering;
- assigning a stale TDR number;
- using the TDR as a substitute for normative collection semantics;
- putting high-level interval algebra into the native floor.

### Acceptance

Every P8 public behavior needed by T2–T6 has an unambiguous normative owner.

For Route B, exact native bindings are accepted before T3 native work.

### Tests to run now

No runtime suite is required merely for documentation changes. Run documentation/link checks only if repository workflow makes them cheap and directly applicable.

---

## T2 — Implement `Bound<T>`, `Interval<T>`, and the canonical interval kernel

### Purpose

Establish the only authority for endpoint and interval semantics before tree code exists.

### Preconditions

- T1 public semantics ratified.
- Generic/Dynamic comparison path from T0 selected.
- No RangeMap tree production logic yet.

### Create/modify

Recommended:

```text
phalcom-core/core/universe/src/collections/interval.ph
phalcom-core/core/universe/src/collections/package.ph
phalcom-modules/src/builtin.rs                         # if required by live module registry
phalcom-core/core/universe/src/errors/argument.ph      # ratified errors
phalcom-core/core/universe/src/errors/package.ph
phalcom-core/tests/core/collections/interval.rs
phalcom-core/tests/core/mod.rs
```

Touch native metadata/CoreDeclarationIds only if T0 proved a real source-loading/type-identity requirement.

### Required implementation shape

Implement source-visible values and one private kernel/helper authority for:

```text
compareDomain(a, b) -> Ordering              # using canonical source <=> path
requireOrdered(ordering)                     # rejects Unordered per ratified error
compareLower(a, b)
compareUpper(a, b)
isValid(interval)
isEmpty(interval)
contains(interval, point)
overlaps(a, b)
intersection(a, b)
touchesWithoutGap(a, b)
unionIfContiguous(a, b)
complementLowerForLeftRemainder(bound)
complementUpperForRightRemainder(bound)
```

Exact helper names may differ.

Comparison discipline:

```text
one <=> result
pattern-match Ordering
do not send <, >, == to reconstruct domain order
```

`Bound::Included(x)` and `Bound::Excluded(x)` payload comparison uses the selected canonical comparator from T0.

### Test changes

Implement CV-01 through CV-13 where they apply to interval behavior.

Minimum boundary matrix:

```text
(-∞,+∞)
(-∞,b)
(-∞,b]
(a,+∞)
[a,+∞)

[a,b]
[a,b)
(a,b]
(a,b)

[x,x]
[x,x)
(x,x]
(x,x)

descending finite bounds
```

For each meaningful shape test contains at:

```text
before lower
exact lower
strict interior
exact upper
after upper
```

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_interval -- --nocapture
```

Use actual discovered filter.

### Acceptance

- all interval boundary tests pass;
- custom nonnumeric domain passes;
- reflected `<=>` probe remains passing;
- `Unordered` is rejected according to spec;
- no RangeMap/tree code contains duplicated endpoint logic.

### Local STOP / CONSULT

Stop if generic interval algebra requires a new comparison trait or one-sided native dispatch.

### Checkpoint update

Record the finalized interval API/invariants and G1 evidence.

---

## T3 — Implement the persistent AVL substrate

### Purpose

Create structurally shared balanced ordered storage with no RangeMap semantic shortcuts.

### Preconditions

- T2 interval kernel stable.
- Route A selected, or Route B floor amendment accepted.

### Route A files

Likely inside:

```text
phalcom-core/core/universe/src/collections/range_map.ph
```

or a private adjacent source module if current package visibility supports it.

### Route B files

Create/modify the §11.2 runtime files and dedicated substrate tests.

### Required AVL mechanics

Every node stores conceptually:

```text
interval
value
left
right
height
```

Implement:

```text
height(node)
balanceFactor(node)
rebuild(node, left, right)
rotateLeft
rotateRight
rebalance
findPredecessorByLower
findSuccessorByLower
insertByLowerPathCopy
removeByLowerPathCopy
minNode / maxNode as needed
splitByLower
join
joinWithPivot
orderedEntries / callback-free snapshot as route permits
```

For overlay/removal complexity, the substrate must expose enough split/join/range extraction capability to avoid deleting each overlapped node independently.

### Tree ordering

Key the tree by interval lower endpoint using `compareLower`.

Because stored intervals are nonempty and pairwise non-overlapping, two stored intervals may not have the same canonical lower bound unless they are the same insertion target/conflict case. Handle this through RangeMap validation rather than silently overwriting a tree node.

### Persistence

A rotation must allocate/rebuild only the affected path:

```text
old A/B/C subtrees remain referenced
new nodes point to shared unaffected children
no mutation of old node height/child slots
```

### Route B callback rule

No rotation/path-copy code may invoke Telos.

Phase A computes:

```text
search path
comparison outcomes
overlap relationships
split/join plan
coalescing decisions when applicable
```

Phase B performs only callback-free heap mechanics.

### Test changes

Implement CV-14, CV-15, CV-16, CV-25, CV-27 structural portions.

Explicit rotation fixtures:

```text
LL
RR
LR
RL
```

Verify after every step:

```text
abs(balanceFactor) <= 1
height = 1 + max(child heights)
in-order lower bounds strictly increasing
old root traversal unchanged
```

### Complexity evidence during development

Use node-visit/comparison counters. A point search should be bounded by AVL height. A conservative automated contract may use a bound such as:

```text
comparisons <= 4 * ceil(log2(n + 2)) + 8
```

only after validating that the counter measures the intended comparisons. Prefer an invariant/height-derived bound over wall-clock timing.

### Tests to run now

Route A:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_range_map persistent_tree -- --nocapture
```

Route B:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib persistent_ordered
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_persistent_ordered -- --nocapture
```

Adapt to live filters.

### Acceptance

- all four rotations;
- delete/rebalance cases;
- old roots immutable;
- unaffected subtree sharing proven at substrate level;
- balanced height;
- no hidden/untraced node graph;
- substrate exposes split/join operations adequate for T5.

### Local STOP / CONSULT

Stop if the only overlay strategy available is repeated O(log n) per overlapped node.

---

## T4 — Implement `RangeMap<K,V>` lookup and strict insertion

### Purpose

Publish the first complete persistent RangeMap core while enforcing the non-overlap invariant at mutation time.

### Preconditions

- T1 public selectors/errors ratified.
- T2 interval kernel passes.
- T3 AVL substrate passes.

### Create/modify

Primary source:

```text
phalcom-core/core/universe/src/collections/range_map.ph
phalcom-core/core/universe/src/collections/package.ph
```

Plus Route B native/identity files if approved.

### Required public behavior

At minimum, as ratified:

```text
new / empty construction
size
isEmpty
get(point) -> Option<V>
contains(point) or equivalent if specified
strict insertion returning a new RangeMap
iterator/entries may remain T6 if not needed yet
```

### Point lookup algorithm

For point `p`:

```text
candidate = greatest stored interval whose lower endpoint can begin at/before p
if candidate exists and intervalKernel.contains(candidate.interval, p):
    Some(candidate.value)
else:
    None
```

Do not scan neighboring intervals after candidate selection.

A convenient search comparator may compare `p` against node lower bounds directly, but it must be semantically equivalent to canonical lower-bound ordering and use the same comparison authority.

### Strict insertion algorithm

For new interval `I`:

```text
validate I
reject empty I
search predecessor/successor around I.lower
test only the necessary neighbors for overlap
if overlap:
    return/raise ratified RangeOverlapError
else:
    path-copy AVL insertion
    perform ratified local normalization/coalescing if T6 semantics are already active
    return new map
```

Because the tree is already non-overlapping and ordered by lower bounds, predecessor/successor checks are sufficient for a strict single-interval insertion.

### Failure atomicity

The following must return/raise without changing the receiver root:

```text
invalid interval
empty interval where forbidden
Unordered endpoint comparison
user comparison error
overlap
value-equality error if immediate coalescing is active
```

### Test changes

Implement CV-17 and CV-18 completely, plus callback cases from CV-11–CV-13 against actual RangeMap operations.

Strict-insert overlap matrix must include:

```text
new inside existing
existing inside new
left partial overlap
right partial overlap
identical interval
single included shared boundary that overlaps
adjacent but non-overlapping Included/Excluded combinations
unbounded overlap
unbounded adjacency where meaningful
```

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_range_map strict_insert -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_range_map lookup -- --nocapture
```

### Acceptance

- invariant checks after every successful insertion;
- overlap never committed;
- lookup returns at most one value;
- old root unchanged;
- comparator count demonstrates logarithmic search.

### Checkpoint update

Record G3/G4 as appropriate once core RangeMap behavior is durable.

---

## T5 — Implement range removal, overlay, and O(log n + k) splice mechanics

### Purpose

Complete the range-editing semantics that distinguish RangeMap from a simple interval-keyed dictionary.

### Preconditions

- T1 removal/overlay semantics ratified.
- T4 strict map invariant stable.
- T3 split/join substrate available.

### Range removal

Under the recommended coverage-removal contract:

```text
remove region R
    split tree around R.lower / R.upper
    identify at most one left-boundary overlapping interval
    identify at most one right-boundary overlapping interval
    discard k fully covered middle entries
    construct exact left/right remainder intervals via canonical complement rules
    rejoin retained trees/remainders
```

Do not invoke `remove(key)` k times.

### Overlay

Conceptually:

```text
overlay(R, V):
    validate R
    if empty and spec says identity: return self

    (left, middle/right...) = split around R
    preserve left boundary remainder if existing interval crosses R.lower
    preserve right boundary remainder if existing interval crosses R.upper
    discard covered middle
    insert R -> V
    coalesce with immediate neighbors if ratified
    join in O(log n + k)
```

`k` is the number of stored intervals overlapped/removed by the operation.

### Boundary split examples

The tests must verify exact inclusion, not only sampled interior points:

```text
existing [a,d], remove/overlay [b,c]
    -> [a,b) and (c,d] remainders

existing [a,d], remove/overlay (b,c)
    -> [a,b] and [c,d] remainders

existing (a,d), remove/overlay [b,c)
    -> (a,b) and [c,d) remainders
```

Follow the centralized complement rules rather than hand-writing these per operation.

### Callback planning

If Route B:

- all endpoint comparisons to identify splice boundaries occur before commit;
- if normalization/coalescing requires `V == neighbor.value`, execute those callbacks before commit;
- temp-root every retained Value/ObjRef needed across callback sends;
- commit receives a complete callback-free splice plan.

### Test changes

Implement CV-19 and CV-20.

Required overlay cases:

```text
overlay into empty map
overlay entirely in gap
overlay exact existing interval
overlay inside one interval (two remainders)
overlay covers one interval exactly
overlay left partial
overlay right partial
overlay spans multiple complete entries
overlay spans partial-left + full-middle + partial-right
overlay unbounded lower
overlay unbounded upper
overlay fully unbounded
overlay adjacent without overlap
all Included/Excluded boundary permutations
```

### Complexity test

Construct maps where overlay intersects `k` consecutive nodes at sizes such as:

```text
n = 32
n = 1024
n = 32768
```

Use comparator/node-visit instrumentation to demonstrate:

```text
search/split overhead ~ logarithmic in n
middle traversal/reclamation ~ linear in k
no k * log(n) repeated-tree-search signature
```

Do not gate on wall-clock microbenchmarks.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_range_map remove -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_range_map overlay -- --nocapture
```

### Acceptance

- exact domain semantics on every boundary;
- no overlap after each operation;
- old roots unchanged;
- O(log n + k) architectural evidence;
- no duplicated interval algebra.

### Local STOP / CONSULT

Stop if the only working implementation is repeated strict removal/insertion per covered node and cannot be transformed into split/join/splice.

---

## T6 — Implement normalization, iteration, equality, and hash

### Purpose

Make `RangeMap` a coherent persistent value collection rather than exposing tree-shape accidents.

### Preconditions

- T1 normalization/equality/hash semantics ratified.
- T5 mutation/splice semantics stable.

### Normalization

If canonical coalescing is ratified, every successful mutating operation must leave the tree normalized.

Coalesce neighboring intervals only when:

```text
values are Telos-equal
AND
intervals do not overlap
AND
their union has no unmapped point
```

Do not merge across a one-point gap.

Never compare values with Rust equality or identity.

### Callback safety

`V == V` is user code.

Route A naturally executes it in source but still must preserve receiver persistence on failure.

Route B must execute required equality callbacks in Phase A before allocating/publishing the final root.

### Iteration

Iterate in canonical lower-bound order and yield the ratified pre-P9 item, recommended:

```telos
(interval: Interval<K>, value: V)
```

Iterator design may snapshot the persistent root/reference; because RangeMap itself is immutable, a root-retaining iterator has deterministic version semantics.

Do not migrate to `Entry<K,V>` in P8.

### Equality

With canonical normalization:

```text
different size/entry count -> false
in-order intervals compare according to interval value semantics
corresponding values use Telos ==
```

The tree shape may differ due to different operation histories; equality must ignore AVL topology.

If interval equality itself invokes endpoint `==` but ordered-domain equivalence should be based on `<=> == Equal`, make the normative distinction explicit. Do not accidentally use object identity for endpoints. Prefer the equality semantics ratified for `Bound`/`Interval`.

### Hash

If published:

```text
fold normalized logical entries deterministically
include interval endpoint state + child value hash
do not include height, tree shape, ObjRef, allocation order
```

The hash law must be consistent with the chosen equality relation. If existing numeric SameValueZero/hash residual prevents a complete law claim for numeric endpoints/values, retain the inherited residual honestly rather than patching Number under P8.

### Test changes

Implement CV-21 through CV-24.

Create at least two maps with identical logical coverage/value mapping built through materially different operation histories and AVL shapes; assert equality/hash agreement.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_range_map normalization -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_range_map equality -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_range_map iteration -- --nocapture
```

### Acceptance

- normalized form stable;
- gap-preserving non-merges correct;
- iteration sorted/logical;
- equality/hash independent of AVL topology;
- no P9 `Entry` work pulled forward.

---

## T7 — GC, callback re-entry, randomized oracle, and complexity hardening

### Purpose

Prove the cross-layer invariants most likely to fail despite correct happy-path behavior.

### Preconditions

- T2–T6 feature behavior complete.

### GC stress

Required scenarios:

#### GCR-1 — shared versions

```text
m0
m1 = m0.insert(A)
m2 = m1.insert(B)
m3 = m2.overlay(C)

retain m0 and m3
drop m1/m2
force GC
verify m0 and m3
```

#### GCR-2 — endpoint/value reachability

Use heap-backed endpoint and value objects that are otherwise reachable only from live RangeMap versions. Force GC and verify callbacks/lookup still see live objects.

#### GCR-3 — iterator retention

Create iterator from a map, drop external map reference where possible, force GC, verify iteration remains valid.

#### GCR-4 — reclamation

Where the test harness exposes heap counts/weak observability, verify nodes unique to dropped versions become reclaimable while shared nodes retained by live roots remain.

### Callback re-entry

Required comparator cases:

```text
compare reads the same map
compare allocates
compare triggers/permits GC
compare mutates another object
compare throws
compare returns Unordered
```

Because RangeMap is immutable, same-receiver structural mutation is not the P6 issue; the critical rule is that an outer operation never publishes a new root after a failed callback.

Value-equality/coalescing cases:

```text
== reads maps
== allocates
== throws
== mutates unrelated state
```

### Randomized property test

Run deterministic random sequences including:

```text
strict insert
lookup
remove
overlay
no-op empty operation if specified
unbounded intervals
all inclusion forms
intentional overlap errors
```

Keep old versions periodically and verify them after later operations.

Use at least:

```text
small exhaustive-ish domain run
1000+ operation deterministic random run
multiple fixed seeds
```

Scale according to test runtime; correctness and reproducibility matter more than raw count.

### Complexity hardening

Prove:

```text
AVL height logarithmic
lookup comparator/visit count logarithmic
strict insert logarithmic excluding callbacks intrinsic to constant neighbor validation
overlay/removal = logarithmic boundary search + linear affected-region work
persistent update allocation proportional to copied paths/rebuilt affected region, not total n
```

Do not add unstable wall-clock pass/fail thresholds.

### Native fallback audits

Route B only:

```text
Object size
trace exhaustiveness
Value::get_class surface receiver behavior
debug rendering
native metadata identity
generated descriptor parity
runtime installation parity
temp-root recovery after callback error
```

### Tests to run now

Focused GC/callback/property/perf filters only, then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_interval -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_range_map -- --nocapture
```

### Acceptance

CV-25 through CV-31 pass, and randomized oracle produces no mismatch.

### Checkpoint update

Record randomized seeds/configuration, complexity evidence, GC gate, and any retained residual.

---

## T8 — Cross-family stabilization and consolidated-main regression gate

### Purpose

Verify that P8 integrates with the current consolidated repository without regressing P6/P7, numeric Range, or the recently merged structural-data work.

### Preconditions

- T7 focused hardening passes.

### Required regression surfaces

1. **P6/P7 collection aggregate** — RangeMap addition must not alter existing mutable/persistent hash behavior.
2. **Numeric Range** — progression, slicing, and existing Range tests unchanged.
3. **Data/Product/Sum** — P8 ordinary data declarations do not regress structural reflection or generic data behavior merged at current main.
4. **Universe/native metadata** — only required identities added; no accidental core-surface special cases.
5. **Generated native surface**:
   - Route A: no P8 native binding increase.
   - Route B: exact accepted delta and clean generator.
6. **GC** — aggregate memory/GC tests still pass.
7. **Formatting/checks**.

### Commands

Discover exact live filters, then run the directly affected set:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_collections -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_bag -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_bijection -- --nocapture

# Current exact Range filter must be discovered from --list.
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core range -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core memory_gc -- --nocapture

# If P8 touched or depends on semantic/data behavior in a way needing direct regression:
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::structural -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --all-targets
cargo fmt --all -- --check
git diff --check
```

Route B additionally:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-native-surface-gen -- --root . --check
```

### Do not automatically run

```text
scripts/verify.sh --full
workspace nextest
workspace clippy
release CI matrix
Miri
VS Code E2E
```

Those remain P9/release responsibilities unless a P8 change directly touches their owner and focused classification is impossible.

### Acceptance

All directly affected lanes pass or failures are truthfully classified under §17.

---

## T9 — Lifecycle closeout, walkthrough, and handoff to P9

### Purpose

Make P8's final architecture and evidence durable so P9 does not rediscover or redesign it.

### Preconditions

- T8 focused stabilization complete.
- No unresolved P8-owned semantic or architectural blocker remains.

### Update checkpoint

Set P8 truthfully, expected normal state:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

Do not claim `RELEASE_COMPLETE`.

Record:

```text
execution-start SHA
final SHA if committed
chosen Route A or Route B
final public API semantics
normalization/removal ruling
native-floor decision and exact delta if Route B
interval invariants
AVL/persistence invariants
GC/callback evidence
randomized oracle configuration
complexity evidence
commands actually run
deferred/unrelated failures
P9 next action
```

### Create walkthrough

```text
docs/implementation/COLL006/COLL006.C1/COLL006.C1.P8-walkthrough.md
```

Must include:

- final implementation result;
- source-first/native route decision and evidence;
- canonical interval kernel;
- persistent AVL representation;
- RangeMap mutation algorithms;
- normalization behavior;
- error semantics;
- important files changed;
- consultations/TDRs;
- tests added;
- exact commands/results;
- randomized seeds/configuration;
- complexity/GC evidence;
- deviations from this plan;
- residual risks.

### Create handoff

```text
docs/implementation/COLL006/COLL006.C1/COLL006.C1.P8-handoff.md
```

Must tell P9:

```text
P8 public APIs to treat as stable
RangeMap logical-entry/iteration shape before Entry migration
persistent ordered substrate that must not be redesigned
normalization/equality/hash semantics
native selectors that are frozen, if any
GC/callback invariants
known residuals
first P9 commands
P9-owned Entry/conversion/certification boundaries
```

### Acceptance

A fresh P9 planner can continue without reopening P8 architecture.

---

# 19. Verification gates

## G0 — Baseline and architecture discriminator

Purpose: establish current repository truth and select Route A or trigger consultation.

Must prove:

```text
current main SHA recorded
P7 baseline coherent
native count recorded
generic/Dynamic bilateral comparison route known
reflected comparison proven
ordinary Bound/Interval data works
recursive generic source node works or concrete failure recorded
source RangeMap collection wrapper works or concrete failure recorded
```

No production P8 code before G0.

---

## G1 — Public contract gate

Purpose: prevent implementation-defined API semantics.

Must prove:

```text
collection spec contains P8
empty/invalid semantics fixed
Unordered behavior fixed
strict-insert overlap behavior fixed
removal semantics fixed
overlay semantics fixed
normalization fixed
iteration/equality/hash fixed
errors fixed
```

Route B additionally requires accepted native-floor amendment.

---

## G2 — Interval kernel gate

Run focused interval tests.

Pass criteria:

```text
CV-01..CV-13 relevant interval cases PASS
custom nonnumeric domain PASS
reflected comparison PASS
no numeric Range changes
one kernel authority confirmed
```

---

## G3 — Persistent AVL gate

Pass criteria:

```text
LL/RR/LR/RL rotations PASS
delete/rebalance PASS
height/balance invariant PASS
in-order lower-bound order PASS
old roots unchanged
structural sharing demonstrated
GC-visible representation confirmed
split/join available
```

---

## G4 — RangeMap core gate

Pass criteria:

```text
lookup O(log n)
strict insert preserves disjointness
all overlap forms rejected
adjacency accepted where non-overlapping
Unordered/callback error publishes nothing
old roots preserved
```

---

## G5 — Range splice gate

Pass criteria:

```text
remove semantics complete
overlay semantics complete
all endpoint complement matrices correct
unbounded cases correct
multi-entry splice correct
O(log n + k) architectural evidence
```

---

## G6 — Normalization/value semantics gate

Pass criteria:

```text
ratified coalescing invariant
gap not accidentally filled
iteration canonical
logical equality independent of tree shape
hash consistent with equality within known global hash-law limits
value-equality callback failure atomic
```

---

## G7 — GC/property/performance gate

Pass criteria:

```text
multi-version GC PASS
endpoint/value reachability PASS
callback allocation/GC PASS
randomized oracle PASS
lookup complexity PASS
overlay complexity PASS
Object size PASS
native floor/catalog PASS or unchanged Route A
```

---

## G8 — Focused integration gate

Pass criteria:

```text
P8 focused suites green
P6/P7 aggregate unaffected
numeric Range unaffected
relevant data/Product/Sum regression green
core all-target check green
format/diff hygiene green
```

After G8, perform T9 lifecycle closeout.

---

## 20. Final focused acceptance matrix

P8 may be marked `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` only when every row is either PASS or explicitly inapplicable due to the chosen route.

| Obligation | Required result |
|---|---|
| `Bound<T>` states | PASS |
| generic `Interval<T>` | PASS |
| valid/empty/invalid semantics | PASS |
| contains | PASS |
| overlaps | PASS |
| intersection | PASS |
| boundary contiguity | PASS |
| unbounded endpoints | PASS |
| custom nonnumeric domain | PASS |
| reflected `<=>` preservation | PASS |
| `Unordered` rejection | PASS |
| comparator error atomicity | PASS |
| persistent AVL rotations | PASS |
| persistent delete/rebalance | PASS |
| old-root immutability | PASS |
| structural sharing | PASS |
| precise GC | PASS |
| point lookup | PASS |
| O(log n) lookup evidence | PASS |
| strict insert | PASS |
| no-overlap invariant | PASS |
| removal | PASS |
| overlay/split | PASS |
| O(log n+k) splice evidence | PASS |
| normalization | PASS if ratified |
| iteration | PASS |
| equality/hash contract | PASS if published |
| randomized oracle | PASS |
| numeric Range firewall | PASS |
| native floor governance | PASS / no new floor in Route A |
| generated native catalog | PASS / unchanged in Route A |
| Object slot size | PASS |
| checkpoint/walkthrough/handoff | COMPLETE |

---

## 21. Performance and resource evidence

### 21.1 Lookup

Do not use wall-clock timing as the principal proof.

Use:

```text
AVL height
node visits
domain comparison callback count
```

at multiple `n`. The observed growth must remain logarithmic and consistent with AVL height.

### 21.2 Strict insertion

Allocation/copy evidence should show:

```text
one root-to-leaf path plus rotation rebuilds
unaffected subtrees shared
no full-tree clone
```

### 21.3 Removal/overlay

Instrumentation must distinguish:

```text
boundary search/split cost = O(log n)
affected middle traversal = O(k)
rejoin/rebalance = O(log n)
```

Reject an implementation whose traces show a fresh root search for each of k removed intervals.

### 21.4 GC/resource model

Route A:

- confirm ordinary data nodes contribute normal managed allocations;
- confirm old versions hold shared nodes alive;
- confirm dropped unique nodes become collectible.

Route B:

- every node allocated through the GC-visible heap;
- boxed node/public payloads preserve Object slot ceiling;
- temp roots are balanced/truncated on callback success and failure;
- no secondary Arc/Rc ownership graph.

---

## 22. Checkpoint bookkeeping

### At plan start

Update `COLL006.C1-CHECKPOINT.md` only after T0 records:

```text
branch/SHA
dirty state
native count
architecture route
baseline status
```

Then mark P8 `IN_PROGRESS`.

### During plan

Update the checkpoint at durable boundaries only:

```text
G0 architecture route
G1 public contract/native-floor decision
G2 interval semantics
G3 persistent AVL interface
G5 overlay/removal completion
G7 randomized/GC/performance evidence
any consultation or deferred residual
```

Do not turn the checkpoint into a command-by-command diary.

### At completion

Record:

```text
COMPLETE / IMPLEMENTED / FOCUSED_TESTED
final stable invariants
exact verification evidence
consultations/TDRs
residuals
P9 as next plan
```

---

## 23. Walkthrough deliverable

Create:

```text
docs/implementation/COLL006/COLL006.C1/COLL006.C1.P8-walkthrough.md
```

Required sections:

```text
1. Final result
2. Prepared/executed revisions
3. Route A/Route B decision
4. Normative API decisions
5. Bound/Interval implementation
6. Persistent AVL representation
7. RangeMap lookup/insert/remove/overlay
8. Normalization/equality/hash/iteration
9. GC/callback architecture
10. Native-floor/catalog changes, if any
11. Tests added
12. Exact commands/results
13. Randomized oracle evidence
14. Complexity/resource evidence
15. Deviations/incidents
16. Residual risks/classification
```

---

## 24. Handoff deliverable

Create:

```text
docs/implementation/COLL006/COLL006.C1/COLL006.C1.P8-handoff.md
```

Must contain:

```text
current revision/state
stable P8 API
canonical interval kernel ownership
ordered-domain contract
persistent AVL ownership
normalization rule
RangeMap iteration item before Entry convergence
GC/re-entry invariants
native bindings and TDR if Route B
focused verification commands
retained residuals
P9 objective
explicit "do not redesign" list
```

The P9 handoff must explicitly state:

> `Entry<K,V>` convergence may adapt RangeMap's association presentation according to P9's ratified migration, but must not alter P8's interval semantics, map domain, overlap invariant, persistence model, or storage complexity architecture.

---

## 25. Completion truth table

Do not conflate:

```text
Bound/Interval source written
Interval tests passing
AVL written
RangeMap API written
RangeMap focused tests passing
randomized oracle passing
GC/re-entry tests passing
affected collection suites passing
native catalog reconciled
P8 complete
C1 checkpoint complete
release certified
```

Normal P8 completion target:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

P9 owns final C1 convergence/release certification.

---

## 26. Plan self-review checklist

Before implementation begins, verify this plan still satisfies the live repository:

- [ ] current `main` revision recorded;
- [ ] P7 remains complete;
- [ ] P8 remains active/not already implemented elsewhere;
- [ ] collection spec still has a P8 gap or newer authority located;
- [ ] decision registry re-read;
- [ ] native surface count re-censused;
- [ ] generic/reflected comparison probes defined;
- [ ] source-first persistent-node probe defined;
- [ ] no assumption that P8 requires native storage;
- [ ] no native binding planned without an accepted amendment;
- [ ] one canonical interval kernel owns endpoint semantics;
- [ ] empty/invalid semantics explicitly ratified;
- [ ] removal semantics explicitly ratified;
- [ ] normalization explicitly ratified;
- [ ] `Unordered` behavior explicitly ratified;
- [ ] endpoint order-stability contract documented;
- [ ] split complement rules are tested;
- [ ] AVL split/join exists before overlay is implemented;
- [ ] old-root immutability has direct tests;
- [ ] callback/GC tests cover both comparator and value equality;
- [ ] randomized oracle is independent/simple;
- [ ] complexity gates use counts/structure, not wall-clock noise;
- [ ] numeric `Range` has a no-regression firewall;
- [ ] `Entry<K,V>` remains P9-owned;
- [ ] Product/Sum merge is treated as current baseline, not a P8 redesign target;
- [ ] walkthrough/handoff/checkpoint obligations are explicit;
- [ ] no placeholder public semantics remain before T2.

---

# Appendix A — Recommended public semantic sketch for T1 consultation

This appendix is a **recommended contract to ratify**, not an already accepted specification.

```telos
data Bound<T> {
    Unbounded
    Included(_ value: T)
    Excluded(_ value: T)
}

data Interval<T>(
    lower: Bound<T>,
    upper: Bound<T>,
)
```

Recommended interval surface, adapting names to current Telos conventions:

```text
isValid -> Bool
isEmpty -> Bool
contains(_ point: T) -> Bool
overlaps(_ other: Interval<T>) -> Bool
intersection(_ other: Interval<T>) -> Option<Interval<T>>
touchesWithoutGap(_ other: Interval<T>) -> Bool
unionIfContiguous(_ other: Interval<T>) -> Option<Interval<T>>
```

Recommended RangeMap surface:

```text
RangeMap<K,V>.new() -> RangeMap<K,V>
size -> Int
isEmpty -> Bool
get(_ point: K) -> Option<V>

inserting(_ interval: Interval<K>, _ value: V) -> RangeMap<K,V>
    invalid/empty => ratified interval error
    overlap       => RangeOverlapError

removing(_ interval: Interval<K>) -> RangeMap<K,V>
    subtract mapped coverage

overlaying(_ interval: Interval<K>, _ value: V) -> RangeMap<K,V>
    replace exactly that domain

iterator -> RangeMapIterator<K,V>
entries -> logical ordered snapshot/view as ratified

==(_ other) -> Bool
hash -> Int
```

Error names/hierarchy and exact selector spelling must be made normative in T1 before implementation.

---

# Appendix B — RangeMap invariant checker for tests

Tests should have a debug/reference invariant function that can inspect logical entries and assert:

```text
1. every interval valid
2. every interval nonempty
3. lower bounds strictly ordered
4. consecutive intervals do not overlap
5. if normalization active:
     no consecutive equal-valued intervals touch without a gap
6. logical entry count == public size if size counts intervals
7. sampled/exhaustive model points produce 0 or 1 containing interval
```

Route B substrate tests additionally assert:

```text
8. height metadata exact
9. balance factor in [-1, 1]
10. all child handles valid
```

Do not expose this checker as public API.

---

# Appendix C — Overlay boundary matrix

For a stored interval `E` and overlay `R`, tests must cover the qualitative relations:

```text
R strictly before E
R strictly after E
R adjacent-left to E
R adjacent-right to E
R == E
R strictly inside E
E strictly inside R
R overlaps E on left
R overlaps E on right
R spans multiple E entries
```

For each finite shared boundary, cover:

```text
Included / Included
Included / Excluded
Excluded / Included
Excluded / Excluded
```

The expected mapped points, not visual bracket similarity, are authoritative.

---

# Appendix D — Route B native fallback non-negotiables

If native fallback is selected:

```text
DO:
  keep interval semantics in source
  trace every Value/ObjRef edge
  box payloads to protect Object size
  use VM temp roots across callbacks
  finish all callbacks before commit
  make commit callback-free
  generate native surface from authored declarations
  test runtime/metadata/descriptor parity

DO NOT:
  call one-sided compare directly as a substitute for <=>
  implement Rust Ord for Telos K
  store endpoint payloads outside GC
  use Arc<Rc<Node>>
  mutate published nodes
  perform callback from rotation/join code
  add convenience native methods without floor approval
  encode equality/hash over tree topology
```

---

# Appendix E — P8 non-regression firewall

P8 completion must leave these predecessor truths intact:

```text
P6:
  MutableBag / IndexedDict / OrderedSet semantics unchanged
  ordered_hash hash/equality authority unchanged unless a separately justified
  mechanical helper extraction has zero behavior change

P7:
  PersistentHashNode unchanged in semantics
  Bag/Bijection 14 admitted bindings unchanged
  HAMT engine unchanged
  old-root persistence unchanged
  numeric hash-law residual remains correctly classified

LANG007.C4.P5 merged baseline:
  Product/Sum structural reflection remains green
  ordinary data nominal identity remains canonical
  no P8-only semantic special casing leaks into general structural reflection

Range:
  numeric/progression/slice behavior unchanged
```

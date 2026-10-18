---
id: COLL006.C1.P5
title: Slice<T> and Buffer<T> — Detailed Implementation Plan
program: COLL006
checkpoint: COLL006.C1
plan: P5
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
repository: aureat/phalcom-lang
branch: feat/telos-collection-upgrade
recorded_against_revision: 78cfd9202f5dea7dd2713bff71d3e475f20810d6
recorded_against_tree: f05c4e428cea8eecfb3f367ee407b009ce481be2
date: 2026-10-10
next_plan: COLL006.C1.P6
---

# COLL006.C1.P5 — `Slice<T>` and `Buffer<T>` Detailed Implementation Plan

## 0. Purpose

This plan defines the implementation of `COLL006.C1.P5`: the **contiguous view and bounded construction/storage layer** of the Telos collection model.

P5 introduces two canonical collection families:

```text
Slice<T>
    read-only
    zero-copy
    runtime length
    strong backing owner + offset + length
    initial zero-copy sources: Array, Vector, Buffer

Buffer<T>
    mutable
    owned
    bounded logical capacity
    runtime size
    initialized prefix only
    no implicit growth
```

P5 also owns the convergence of the earlier `LANG007.C4.P14` parser-snapshot requirement onto the ratified C1 collection architecture:

```text
mutable caller List<T>
        ↓ snapshot once
private owned Array<T>
        ↓ zero-copy
read-only Slice<T>
        ↓
parser source/indexed consumer
```

P5 **does not** reopen the P1–P4 sequence-literal, inference, static-value, or materialization architecture.

The plan is written for an advanced reasoning implementer. The implementer is expected to inspect and adapt to local symbol drift, but must preserve the semantic ownership and invariants specified here.

---

# 1. Repository grounding

## 1.1 Authoritative branch and revision

Implement against:

```text
repository: aureat/phalcom-lang
branch: feat/telos-collection-upgrade
HEAD: 78cfd9202f5dea7dd2713bff71d3e475f20810d6
tree: f05c4e428cea8eecfb3f367ee407b009ce481be2
```

HEAD message at plan creation:

```text
docs(COLL006): record residual ruling in P4 lifecycle
```

Do not implement P5 from an older local clone or from `main` without first reconciling the branch.

## 1.2 Current C1 lifecycle state

At this revision:

```text
P1  COMPLETE / IMPLEMENTED / FOCUSED_TESTED
P2  COMPLETE / IMPLEMENTED / FOCUSED_TESTED
P3  COMPLETE / IMPLEMENTED / FOCUSED_TESTED
P4  COMPLETE / IMPLEMENTED / BASELINE_BLOCKED

P5  PROPOSED / NOT_STARTED / UNVERIFIED
P6  PROPOSED
P7  PROPOSED
P8  PROPOSED
P9  PROPOSED
```

The P4 verification classification is not a P5 start blocker. The remaining D-class/static-constant baseline failures are inherited verification residuals, not missing P5 prerequisites.

## 1.3 Stable P1–P4 interfaces P5 inherits

P5 may rely on all of the following:

1. Bare runtime bracket literals default to `Array<T>`.
2. Bare static/`const` bracket literals default to `Vector<T,N>`.
3. `<T>[...]` explicitly selects `Array<T>`.
4. `<T,N>[...]` explicitly selects `Vector<T,N>`.
5. Explicit/contextual `List<T>` remains supported.
6. Neutral bracket-literal family selection is owned by semantic checking.
7. The compiler consumes checked `SequenceLiteralResolution`; it does not infer collection family.
8. `StaticValueData::Sequence` is the canonical static homogeneous sequence representation for Array/Vector constants.
9. Static materialization builds immutable `VectorObject` values and frozen ordinary `ArrayObject` values.
10. `const Array<T>` remains nominally `Array<T>` and is protected at runtime through alias-safe freezing.
11. Runtime ordinary `Array<T>` remains fixed-extent but element-mutable.
12. `Vector<T,N>` remains immutable and statically dimensioned.
13. P5 must not modify these contracts merely to simplify Slice/Buffer implementation.

---

# 2. P5 architectural objective

P5 completes this part of the collection taxonomy:

| Family | Owns elements | Runtime length | Static extent in type | Structural resize | Element mutation | View |
|---|---:|---:|---:|---:|---:|---:|
| `List<T>` | yes | yes | no | yes | yes | no |
| `Array<T>` | yes | yes | no | no | yes | source |
| `Vector<T,N>` | yes | reflected at runtime | yes | no | no | source |
| `Slice<T>` | no | yes | no | no | no | yes |
| `Buffer<T>` | yes | yes | no | append until capacity | yes | source |

The central semantic distinction is:

```text
Array   = owned fixed-extent runtime sequence
Vector  = owned immutable statically dimensioned sequence
Slice   = borrowed-by-ownership read-only projection
Buffer  = owned bounded construction/storage sequence
```

`Slice<T>` is not an immutable owned collection. Its visible content can change when its backing `Array<T>` or `Buffer<T>` mutates.

`Buffer<T>` is not a second `List<T>`. Its capacity is a semantic upper bound and it does not grow automatically.

---

# 3. Non-negotiable invariants

## 3.1 Slice invariants

P5 must establish and preserve all of the following:

```text
S1. Slice<T> is read-only in v1.

S2. Slice<T> stores a strong owner reference and logical range.
    No raw interior pointer is relied upon for lifetime correctness.

S3. Supported zero-copy v1 backing families are exactly:
        Array<T>
        Vector<T,N>
        Buffer<T>

S4. List<T> is not a live zero-copy Slice source in P5.

S5. Slice length is runtime state and is not part of nominal type identity.

S6. Slice of Slice normalizes directly to the original backing owner.

S7. Slice lookup remains O(1), independent of nesting depth.

S8. Existing Slice length/range does not change when a Buffer later appends.

S9. Mutating an initialized Array/Buffer backing position is observable
    through an existing Slice covering that position.

S10. A Vector-backed Slice observes stable content because Vector is immutable.

S11. Slice equality is current-content structural equality, not backing identity.

S12. Slice is not Map/Set key-safe in v1, regardless of backing kind.

S13. Slice GC tracing keeps the backing owner alive.

S14. Slice does not independently trace backing elements; the owner does.

S15. Slice is not a neutral bracket-literal family.

S16. Slice has no independent static-value/materialization representation.
```

## 3.2 Buffer invariants

```text
B1. Buffer<T> has runtime logical capacity.

B2. Buffer<T> has runtime semantic size.

B3. Always:
        0 <= size <= capacity

B4. Buffer stores exactly the initialized prefix.

B5. Rust Vec capacity is not Telos Buffer capacity.

B6. Spare logical capacity does not contain fake/sentinel Telos Values.

B7. Buffer is not an uninitialized-memory abstraction.

B8. push/append may increase size only when size < capacity.

B9. push at capacity fails atomically.

B10. indexed replacement is allowed only when index addresses an
     already initialized element.

B11. writing at index == size does not implicitly append.

B12. sparse initialization is forbidden.

B13. automatic capacity growth is forbidden.

B14. P5 exposes no size-decreasing mutation.
     No pop/remove/truncate/clear/shrink semantics are introduced.

B15. Buffer index/set/push target O(1).

B16. Buffer equality is structural equality of initialized contents.
     Logical capacity is not equality-significant.

B17. Buffer is not Map/Set key-safe.

B18. GC traces only initialized values.

B19. replacing an initialized element releases the old element from
     Buffer reachability.

B20. Buffer is not a bracket-literal family.

B21. Buffer has no compile-time static-value representation.
```

---

# 4. P5 scope

## 4.1 Required work

P5 owns:

1. canonical `Slice<T>` Universe identity;
2. canonical `Buffer<T>` Universe identity;
3. generic type-form metadata for both;
4. canonical semantic declaration IDs;
5. runtime class/metaclass integration;
6. native heap representations;
7. heap allocation/accessor integration;
8. `Value::class` mapping;
9. GC tracing;
10. minimal native primitive floor;
11. Telos source-level collection APIs;
12. Array → Slice;
13. Vector → Slice;
14. Buffer → Slice;
15. nested slicing;
16. range-bound reuse;
17. Buffer logical capacity;
18. Buffer initialized-prefix semantics;
19. Buffer push/set/index;
20. capacity exhaustion behavior;
21. structural equality;
22. canonical iterable integration;
23. Map/Set key-admission policy updates;
24. parser `ListSource<T>` migration to private Array + Slice;
25. LANG007.C4.P14 lifecycle convergence;
26. focused correctness, GC, representation, package and regression tests;
27. P5 walkthrough/handoff/checkpoint updates.

## 4.2 Explicit non-goals

P5 must not implement:

- `List<T>` live Slice views;
- mutable/write-through `Slice<T>`;
- Buffer automatic growth;
- Buffer shrink/remove/pop/clear;
- sparse Buffer writes;
- `Buffer<T,N>`;
- `Slice<T,N>`;
- `ImmutableSequence<T>`;
- `ImmutableArray<T>`;
- runtime existential/dynamic-`N` Vector;
- new bracket-literal family inference;
- `StaticValueData::Slice`;
- `StaticValueData::Buffer`;
- `MaterializationDescriptor::Slice`;
- `MaterializationDescriptor::Buffer`;
- new static-evaluator sequence semantics;
- new compiler sequence-literal lowering;
- new bytecodes merely for Slice/Buffer;
- generic variance redesign;
- P6 ordered-hash substrate work;
- full collection conversion convergence;
- global P9 certification.

---

# 5. Current repository architecture relevant to P5

## 5.1 Canonical collection identity

Relevant live file:

```text
phalcom-native-meta/src/universe.rs
```

Current canonical collection keys include:

```rust
UniverseKey::Collection
UniverseKey::List
UniverseKey::Array
UniverseKey::Vector
UniverseKey::Map
UniverseKey::Set
UniverseKey::Tuple
UniverseKey::Pack
UniverseKey::Record
UniverseKey::Range
UniverseKey::Bytes
```

P5 must add:

```rust
UniverseKey::Slice
UniverseKey::Buffer
```

and extend:

```rust
UniverseKey::source_path()
UniverseKey::name()
UniverseKey::from_name()
UNIVERSE_CLASS_RELATIONS
UNIVERSE_BINDINGS
UNIVERSE_TYPE_FORMS
```

Expected source ownership:

```text
Slice  -> universe.collections.slice
Buffer -> universe.collections.buffer
```

Both are ordinary public `Collection` subclasses.

## 5.2 Canonical semantic declaration IDs

Relevant live file:

```text
phalcom-semantic/src/core_surface/identity.rs
```

Current `CoreDeclarationIds` contains:

```rust
pub list: DeclarationId,
pub array: DeclarationId,
pub vector: DeclarationId,
...
```

P5 must add:

```rust
pub slice: DeclarationId,
pub buffer: DeclarationId,
```

constructed from exact `UniverseKey` identity.

Do not introduce spelling-based semantic recognition.

## 5.3 Runtime class integration

Relevant file:

```text
phalcom-core/src/universe/core_classes.rs
```

Current bootstrap creates:

```rust
collection_class
list_class
array_class
vector_class
map_class
set_class
...
```

P5 must add:

```rust
slice_class
buffer_class
```

with:

```text
superclass = Collection
native_repr = true
```

and propagate the fields through `CoreClasses` construction/resolution.

## 5.4 Heap representation

Relevant files:

```text
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/mod.rs
phalcom-core/src/heap/accessors.rs
phalcom-core/src/heap/trace.rs
```

Current relevant object arms:

```rust
Object::List(...)
Object::Array(...)
Object::Vector(...)
Object::Bytes(...)
Object::Pack(...)
```

P5 adds native runtime object arms for Slice and Buffer.

## 5.5 Value classification

Relevant file:

```text
phalcom-core/src/value/mod.rs
```

Every new native object arm must map to its canonical runtime class.

Search every exhaustive object-kind/classification match while implementing P5. Do not assume `Value::class` is the only match site.

## 5.6 Existing view precedent

Relevant files:

```text
phalcom-core/src/heap/pack.rs
phalcom-core/src/product/mod.rs
phalcom-core/src/primitive/pack.rs
phalcom-core/tests/core/memory/pack_gc.rs
```

`Pack` already implements the ownership pattern P5 needs:

```text
root owns storage
slice stores root ObjRef + ranges
slice-of-slice normalizes to root
GC traces root from slice
```

P5 should reuse the **lifetime/root-normalization design**, not Pack's product-specific layout machinery.

## 5.7 Existing index semantics

Relevant file:

```text
phalcom-core/src/primitive/index.rs
```

Reuse:

```rust
normalize_element_index(...)
```

for point indexing on Slice and Buffer.

Do not fork negative-index behavior.

## 5.8 Existing range slicing semantics

Relevant Universe source:

```text
phalcom-core/core/universe/src/collections/range.ph
```

The canonical authority for finite sequence slicing is:

```telos
Range.rawSliceBounds(size)
```

It already handles:

- omitted bounds;
- negative coordinates;
- clamping;
- inclusive upper ends;
- integer-coordinate validation.

P5 should reuse this path for Slice-producing range operations. Do not reproduce these semantics in Rust.

## 5.9 Existing key-admission policy

Relevant helper:

```text
phalcom-core/src/primitive/mod.rs::is_mutable_collection_key
```

Current branch rejects at least:

```text
List
Array
Map
Set
Bytes
```

P5 must extend key rejection to:

```text
Buffer
Slice
```

Because Slice can expose changing structural content, the existing helper name becomes conceptually narrower than the policy. Prefer either:

1. rename it to a key-stability-oriented name such as `is_unstable_collection_key`; or
2. retain the name but update its documentation to make the broader stability rule explicit.

Do not make Slice key legality conditional on whether the backing happens to be Vector.

## 5.10 Current parser snapshot implementation

Relevant files:

```text
packages/parse/src/source/list-source.ph
packages/parse/src/core/source.ph
packages/parse/tests/runtime/src/core/source-test.ph
phalcom-core/tests/core/modules/parse_package.rs
```

Current `ListSource<T>`:

```text
copies caller List<T> into a new private List<T>
```

This already provides snapshot isolation but does not use the ratified P5 collection architecture.

P5 will migrate this to:

```text
private Array<T> snapshot
read-only Slice<T> consumer-facing/internal storage
```

The parse package gate is currently documented as:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core modules_parse_package
```

---

# 6. Representation design

## 6.1 `SliceObject`

Preferred v1 representation:

```rust
pub struct SliceObject {
    owner: ObjRef,
    start: usize,
    len: usize,
}
```

Required invariants:

```text
owner points to Array, Vector, or Buffer
start <= owner_visible_extent
start + len <= owner_visible_extent
owner is the original root backing, never another Slice
```

No raw pointer.

No borrowed Rust slice.

No required cached backing-kind field unless implementation evidence demonstrates a concrete need.

### Why owner+offset+length

This gives:

```text
construction       O(1)
size               O(1)
index              O(1)
nested slice       O(1)
GC lifetime        explicit
backing mutation   observable
```

## 6.2 `BufferObject`

Preferred representation:

```rust
pub struct BufferObject {
    logical_capacity: usize,
    elements: Vec<Value>,
}
```

Semantic size is:

```rust
elements.len()
```

Invariant:

```rust
elements.len() <= logical_capacity
```

The Rust allocator's `Vec::capacity()` is private implementation detail.

### Allocation policy

At construction:

```rust
let mut elements = Vec::new();
elements.try_reserve_exact(logical_capacity)?;
```

This makes capacity commitment explicit and keeps subsequent allowed pushes within the pre-reserved logical bound.

Do not use `vec![sentinel; capacity]`.

Do not create uninitialized public Value slots.

## 6.3 Object arena size

Current runtime architecture deliberately preserves approximately:

```text
size_of::<Object>() == 40 bytes
```

after boxing fat variants.

Before adding Slice/Buffer:

1. measure current `Object` size;
2. implement the likely inline representations;
3. remeasure;
4. if either arm increases the common slot ceiling, box that payload.

P5 must not silently regress every heap allocation merely to store Buffer or Slice.

---

# 7. Public type semantics

## 7.1 `Slice<T>`

`Slice<T>` is a normal one-parameter nominal generic:

```text
Slice : Type -> Type
```

Its logical length is not part of type identity.

Examples:

```text
Slice<Int> length 2
Slice<Int> length 100
```

have the same nominal type.

## 7.2 `Buffer<T>`

`Buffer<T>` is also:

```text
Buffer : Type -> Type
```

Capacity is runtime state.

These are the same type:

```text
Buffer<Int>(capacity 8)
Buffer<Int>(capacity 4096)
```

## 7.3 Variance

Do not make P5 the owner of variance-system changes.

`Buffer<T>` is mutable and semantically invariant.

`Slice<T>` might support covariance in a future variance model, but P5 should use existing canonical generic semantics and avoid Slice-specific relation hacks.

---

# 8. Public API requirements

Exact selector spelling can adapt to live conventions, but behavior must satisfy the following surface.

## 8.1 `Slice<T>`

Required capabilities:

```text
size
at(index) / indexed lookup
get(index) where consistent with current collection vocabulary
iteration
range slicing -> Slice<T>
structural ==
toList where existing collection conversion conventions justify it
```

Not required:

```text
set
put
subscript assignment
append
push
remove
clear
```

## 8.2 `Buffer<T>`

Required capabilities:

```text
new(capacity)
size
capacity
at(index) / indexed lookup
get(index)
set/indexed replacement
push/append
iteration
range slicing -> Slice<T>
structural ==
```

Do not add shrink operations in P5.

## 8.3 Array/Vector Slice production

Add source-level Slice-producing surfaces to:

```text
Array<T>
Vector<T,N>
Buffer<T>
Slice<T>
```

Prefer typed methods rather than weakening integer index signatures with `Dynamic`.

Conceptual shape:

```telos
array.slice(range) -> Slice<T>
vector.slice(range) -> Slice<T>
buffer.slice(range) -> Slice<T>
slice.slice(range) -> Slice<T>
```

Exact selector names should follow live style and existing Range usage.

---

# 9. Range semantics

P5 must keep one range-normalization authority:

```text
Range.rawSliceBounds(size)
```

Recommended control flow:

```text
user range
    ↓
Range.rawSliceBounds(source.size)
    ↓
Result<(start,end), SliceError>
    ↓
native Slice creation from normalized coordinates
```

Native code should:

- validate backing kind;
- validate normalized bounds defensively;
- perform checked offset arithmetic;
- normalize Slice owner to root;
- allocate one Slice object.

Native code should **not** reinvent:

- negative-bound semantics;
- inclusive-end semantics;
- open-bound semantics;
- clamping policy.

---

# 10. Buffer capacity semantics

## 10.1 Constructor

Conceptually:

```telos
Buffer<T>.new(_ capacity: Int) -> Buffer<T>
```

Requirements:

```text
capacity >= 0
capacity representable by host usize
allocation reservation succeeds
initial size == 0
reported capacity == requested capacity
```

Reject:

- negative capacity;
- host-unrepresentable capacity;
- reservation overflow;
- allocation failure.

Use checked conversion. No unchecked `as usize`.

## 10.2 Push

Precondition for success:

```text
size < capacity
```

After success:

```text
size' = size + 1
capacity' = capacity
old prefix unchanged
new value at old size
```

At full capacity:

```text
push fails
state is unchanged
```

## 10.3 Replacement

Replacement may address only the initialized prefix.

Allowed:

```text
0 <= normalized_index < size
```

Forbidden as replacement:

```text
index == size
index > size
```

Replacement must not alter size/capacity.

## 10.4 Shrinking

P5 should expose no size-decreasing mutation.

This avoids invalidating extant owner+offset+length Slice values.

If future work needs shrink/remove/pop, it requires a separate lifetime/view decision.

---

# 11. Capacity failure contract

The lifecycle records require failure semantics but do not fix exact public error spelling.

Before freezing P5 public tests, make one explicit local ruling for:

```text
logical capacity exhaustion
```

The invariant is fixed:

```text
push at size == capacity must fail atomically
```

The following are mechanically open:

```text
throw versus Result
exact error class
```

Guidance:

```text
invalid constructor argument
    -> existing ArgumentError/resource conventions

host reservation/allocation failure
    -> existing runtime ResourceLimit/allocation path

logical bounded-capacity exhaustion
    -> one stable, explicitly documented public failure contract
```

Do not proliferate a one-off error type unless the repository's error vocabulary and likely reuse justify it.

---

# 12. Equality and hashing/key policy

## 12.1 Slice equality

Structural equality is over current visible contents.

Therefore:

```text
same owner + same visible contents       equal
different owner + same visible contents  equal
different visible content                unequal
different length                         unequal
```

Mutating a backing Array/Buffer may change later equality results.

## 12.2 Buffer equality

Compare initialized contents only.

Capacity is not equality-significant:

```text
Buffer(capacity 4, values [1,2])
Buffer(capacity 100, values [1,2])
```

compare equal.

## 12.3 Implement equality at high level

Prefer Telos loops analogous to Array/Vector.

Avoid native equality loops that hold heap borrows across user-overridable equality sends.

## 12.4 Map/Set key admission

Reject Buffer and Slice as keys.

Required negative cases:

```text
Buffer
Slice<Array>
Slice<Buffer>
Slice<Vector>
```

The Vector-backed Slice case is mandatory: key safety is a Slice-family rule, not dynamic backing-specific behavior.

Keep immutable positive controls such as Vector/Tuple/Record unchanged.

---

# 13. Iterable integration

Reuse:

```text
CanonicalIterator
CanonicalIterable
```

No new protocol hierarchy.

Implement source-order iterators for Slice and Buffer.

Requirements:

```text
Slice iterator traverses exactly Slice.len visible values.

Buffer iterator traverses elements[0..size].

No spare Buffer capacity is visible.

No temporary List/Array allocation is required for ordinary iteration.
```

If a reusable indexed-iterator abstraction already exists and can be used without distorting type semantics, reuse it; otherwise local `SliceIterator<T>` and `BufferIterator<T>` are acceptable.

---

# 14. GC and rooting requirements

## 14.1 Slice edge

`trace_object` must treat Slice as:

```text
SliceObject
    └── owner ObjRef
```

No raw pointer may be the sole owner link.

## 14.2 Buffer edges

Trace every `Value` in `elements`.

Because:

```text
elements.len() == semantic size
```

this automatically traces only initialized values.

Spare capacity has no semantic Value objects and therefore no GC edges.

## 14.3 Replacement reachability

After:

```text
buffer[i] = old
buffer[i] = new
```

Buffer must no longer retain `old`.

A forced collection must be able to reclaim it if no other root exists.

## 14.4 Deep Slice normalization

If only the deepest nested Slice is rooted:

```text
root backing must survive
deep Slice must survive
intermediate Slice objects may become collectible
```

This is an important proof that nested slices normalize to the backing root rather than chain through intermediate views.

---

# 15. Parser/P14 convergence

## 15.1 Historical state

The earlier LANG007.C4 P14 proposal described a dedicated immutable homogeneous indexed snapshot family.

The current repository does not contain a live `ImmutableSequence<T>` implementation.

The parse package instead currently snapshots a caller List by manually copying it into another private List.

P5 supersedes the **generic parser snapshot mechanism**, not the completed text/Substring work.

## 15.2 Ratified P5 direction

Parser token snapshots should use:

```text
caller List<T>
    ↓ O(n) snapshot
private Array<T>
    ↓ zero-copy view
Slice<T>
    ↓
ListSource<T>
```

Do not use dimensioned `Vector<T,N>` for runtime-sized token snapshots.

Do not add `ImmutableSequence<T>`.

## 15.3 `ListSource<T>` migration

Current conceptual structure:

```telos
class ListSource<T> {
  mut _items: List<T>
}
```

Target:

```telos
class ListSource<T> {
  _items: Slice<T>
}
```

Construction should create one independent Array snapshot from the caller List, then retain a Slice over that Array.

Preferred mechanism where current syntax/type inference supports it:

```telos
let backing: Array<T> = <T>[*items]
_items = backing.slice(..)
```

If a direct source helper is cleaner, it must preserve the same architecture:

```text
one owned Array snapshot
one read-only Slice
no copied Slice payload
```

## 15.4 Why this is the correct parser architecture

It proves all of P5's intended semantics simultaneously:

- caller List can mutate later;
- parser membership/order stay stable;
- parser consumer cannot mutate snapshot through Slice;
- traversal is O(1) indexed;
- no dimensioned Vector abuse;
- no duplicate immutable sequence family;
- the Array may be reachable only through Slice, giving a real GC ownership proof.

---

# 16. Expected write set

This is an expected rather than rigid file list. Verify current symbols before editing.

## 16.1 Native metadata / semantic identity

```text
phalcom-native-meta/src/universe.rs
phalcom-semantic/src/core_surface/identity.rs
```

## 16.2 Runtime heap

```text
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/mod.rs
phalcom-core/src/heap/accessors.rs
phalcom-core/src/heap/trace.rs
phalcom-core/src/heap/slice.rs         NEW
phalcom-core/src/heap/buffer.rs        NEW
```

## 16.3 Runtime primitives

```text
phalcom-core/src/primitive/mod.rs
phalcom-core/src/primitive/slice.rs    NEW
phalcom-core/src/primitive/buffer.rs   NEW
```

Potential adjacent helpers:

```text
phalcom-core/src/primitive/index.rs
```

but prefer reuse over modification.

## 16.4 Runtime classes/value mapping

```text
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/value/mod.rs
```

## 16.5 Universe source

```text
phalcom-core/core/universe/src/collections/slice.ph      NEW
phalcom-core/core/universe/src/collections/buffer.ph     NEW
phalcom-core/core/universe/src/collections/array.ph
phalcom-core/core/universe/src/collections/vector.ph
phalcom-core/core/universe/src/collections/package.ph
```

Possibly related exports in the Universe root if package exposure requires explicit additions.

## 16.6 Parser package

```text
packages/parse/src/source/list-source.ph
packages/parse/tests/runtime/src/core/source-test.ph
packages/parse/README.md
```

## 16.7 Tests

Likely new/extended suites under:

```text
phalcom-core/tests/core/collections/
phalcom-core/tests/core/memory/
phalcom-core/tests/core/modules/
phalcom-semantic/tests/semantic/
```

Use current test organization rather than forcing arbitrary new top-level harnesses.

## 16.8 Lifecycle records

Expected records:

```text
docs/implementation/COLL006/COLL006.C1/
    COLL006.C1.P5-...plan.md
    COLL006.C1.P5-walkthrough.md
    COLL006.C1.P5-handoff.md
    COLL006.C1-CHECKPOINT.md

docs/implementation/COLL006/
    PROGRAM.md
    STATUS.md
```

Also amend the exact LANG007.C4 P14 lifecycle/documentation location that currently owns the older immutable-snapshot direction.

---

# 17. Implementation task sequence

# T0 — Ground the implementation

## Goal

Establish exact live symbols and reproducible pre-P5 evidence before production changes.

## Required investigation

Record:

```text
git branch --show-current
git rev-parse HEAD
git status --short
```

Confirm:

```text
feat/telos-collection-upgrade
78cfd9202f5dea7dd2713bff71d3e475f20810d6
clean worktree
```

If HEAD has moved intentionally, update the P5 record and audit the delta before continuing.

## Required source census

Search all runtime match sites for at least:

```text
Object::List
Object::Array
Object::Vector
Object::Bytes
```

Classify which must gain Slice/Buffer behavior.

Particular targets:

```text
heap/object.rs
heap/trace.rs
heap/accessors.rs
heap/mod.rs
value/mod.rs
primitive/mod.rs
universe/core_classes.rs
typing/reflection helpers
debug/render/type-name surfaces
test-only kind inspection
```

## Baseline measurements

Record:

```text
size_of::<Value>()
size_of::<Object>()
native surface declaration count
```

P5 must keep Value representation unchanged and must not grow the Object arena slot ceiling.

## Baseline tests

Run the focused P4 controls used by the branch handoff, plus the parser source package gate.

At minimum establish that current:

```text
Array suite
Vector suite
COLL006 semantic focused suite
static_sequence focused suite
parse package source tests
native-surface --check
cargo check --workspace --all-targets
git diff --check
```

behave as recorded.

Do not spend P5 on inherited D-class failures if they reproduce exactly as classified.

### G0 exit criteria

- live revision recorded;
- clean baseline classified;
- Object match census complete;
- Object size recorded;
- parse snapshot baseline proven;
- no unexpected P5 prerequisite defect discovered.

---

# T1 — Canonical Slice/Buffer identity and class skeleton

## Goal

Land canonical language/runtime identities before storage behavior.

## Implement

### `phalcom-native-meta/src/universe.rs`

Add:

```rust
UniverseKey::Slice
UniverseKey::Buffer
```

Extend:

```rust
UNIVERSE_CLASS_RELATIONS
source_path()
name()
from_name()
UNIVERSE_BINDINGS
UNIVERSE_TYPE_FORMS
```

Generic forms:

```text
Slice<T>
Buffer<T>
```

with `T : Type`.

### `phalcom-semantic/src/core_surface/identity.rs`

Add:

```rust
pub slice: DeclarationId,
pub buffer: DeclarationId,
```

using canonical universe declaration identity.

### `phalcom-core/src/universe/core_classes.rs`

Add:

```rust
slice_class
buffer_class
```

as native-representation subclasses of `Collection`.

### Universe source modules

Add source-owned class shells with exact generic parameters and Collection inheritance:

```telos
native class Slice<T> is Collection { ... }
native class Buffer<T> is Collection { ... }
```

Expose both through the collections package.

## Tests

Prove:

- exact Universe paths;
- exact exported class names;
- generic arity 1;
- type parameter kind is `Type`;
- `Slice<Int>`/`Slice<String>` distinct;
- `Buffer<Int>`/`Buffer<String>` distinct;
- runtime class rows are canonical, not fresh shadow classes;
- superclass is canonical Collection;
- ordinary generic type reflection remains coherent.

### G1 exit criteria

Canonical identity exists end-to-end without storage implementation hacks.

---

# T2 — Buffer heap substrate

## Goal

Implement Buffer's storage law in isolation before introducing views.

## Add `BufferObject`

Preferred:

```rust
pub struct BufferObject {
    logical_capacity: usize,
    elements: Vec<Value>,
}
```

Provide narrowly scoped methods such as:

```rust
try_new(capacity) -> Result<Self, ...>
len()
capacity()
is_full()
get(index)
set(index, value)
push(value)
elements()
```

Keep invariants inside the object where practical.

## Add `Object::Buffer`

Update:

```text
heap/object.rs
heap/mod.rs allocation
heap/accessors.rs
value class mapping
kind/type-name/debug match sites
trace.rs
```

## Constructor semantics

Use checked conversion from Telos Int to `usize`.

Use fallible reservation.

Do not expose allocator capacity as semantic capacity.

## Primitive floor

Implement only what source-level methods need, conceptually:

```text
Buffer.class::new(capacity)
Buffer::rawLength
Buffer::rawCapacity
Buffer::rawAt(index)
Buffer::rawSet(index,value)
Buffer::rawPush(value)
```

Exact selector spelling can follow current native naming conventions.

Native primitives should implement representation safety, not high-level policy that belongs in `.ph`.

## Failure behavior

Before publishing API tests, record the exact capacity-exhaustion contract chosen under section 11.

## GC

`trace_object(Object::Buffer)` visits every initialized element.

## Key safety

Add Buffer to key rejection immediately when the heap arm lands. Do not leave a temporary window where a mutable content-sensitive Buffer can become a Map/Set key.

## T2 tests

### Constructor

- 0 capacity;
- 1 capacity;
- ordinary capacity;
- negative input;
- non-integral numeric if constructor parameter can reach such value dynamically;
- host-overflow input;
- allocation/reservation failure seam where deterministic test support exists.

### Push

- push into empty available Buffer;
- fill exactly to capacity;
- one push beyond capacity;
- repeated failure remains state-stable.

### Set

- replace first/middle/last initialized element;
- negative valid index;
- negative invalid index;
- `index == size` rejected;
- `index > size` rejected;
- size unchanged after set.

### GC

- initialized heap children survive while Buffer rooted;
- replaced child becomes collectible.

### G2 exit criteria

Buffer owns valid bounded storage independently of Slice.

---

# T3 — Slice heap substrate

## Goal

Implement owner-rooted views over stable contiguous backings.

## Add `SliceObject`

Preferred:

```rust
pub struct SliceObject {
    owner: ObjRef,
    start: usize,
    len: usize,
}
```

Methods:

```rust
owner()
start()
len()
is_empty()
```

Keep mutation impossible at this layer except construction.

## Root normalization helper

Create one canonical construction helper, conceptually:

```rust
fn make_slice(
    vm: &mut VM,
    owner_value: Value,
    start: usize,
    len: usize,
) -> PhResult<Value>
```

Behavior:

### Array owner

```text
root = Array ObjRef
base_start = 0
visible_len = array.len
```

### Vector owner

```text
root = Vector ObjRef
base_start = 0
visible_len = vector.len
```

### Buffer owner

```text
root = Buffer ObjRef
base_start = 0
visible_len = buffer.len
```

### Slice owner

Normalize:

```text
root = slice.owner
base_start = slice.start
visible_len = slice.len
```

Then:

```text
new_start = checked_add(base_start, requested_start)
new_len   = requested_len
```

Validate:

```text
requested_start <= visible_len
requested_start + requested_len <= visible_len
```

Create a Slice whose owner is never another Slice.

## Add `Object::Slice`

Update every exhaustive runtime match surface.

## Value access helper

One internal lookup path should:

1. validate Slice receiver;
2. normalize point index against Slice length;
3. compute checked backing index;
4. inspect owner object kind;
5. read Array/Vector/Buffer storage.

Do not allocate.

Do not copy.

## GC

Trace exactly `slice.owner`.

## Key safety

Reject Slice as Map/Set key from the moment the heap arm is usable.

## T3 tests

- full Array view;
- partial Array view;
- Vector view;
- Buffer view;
- empty Slice;
- one-element Slice;
- nested Slice;
- deep nested Slice;
- root owner identity;
- offset composition;
- checked overflow defense;
- unsupported owner rejection;
- point-index negative behavior consistent with Array/Vector;
- O(1) root-normalized representation.

### G3 exit criteria

A Slice is a correct native owner-rooted view before public range APIs are layered on top.

---

# T4 — Source-level Slice/Buffer collection APIs

## Goal

Expose typed Telos collection behavior while keeping the native floor minimal.

## `slice.ph`

Implement:

```text
size
first
last
at/get
throwing or optional accessor variants consistent with Array/Vector style
iterator
range slice
==
!= if current collection style defines it explicitly
toList if justified
```

Do not add mutation methods.

## `buffer.ph`

Implement:

```text
size
capacity
first
last
at/get
indexed replacement
push/append
iterator
range slice
==
!= if needed
toList if useful/current style requires
```

No shrink methods.

## Array integration

Add typed Slice-producing method using `Range.rawSliceBounds`.

Do not change ordinary Array integer indexing or P4 fixed/frozen semantics.

## Vector integration

Add typed Slice-producing method.

Do not change Vector immutability, hash law, or exact `N`.

## Buffer integration

Add typed Slice-producing method over initialized prefix only.

## Slice integration

Nested range slicing goes through the same normalized native construction helper.

## Iterators

Implement CanonicalIterator/CanonicalIterable conformance.

### G4a functional gate

- source methods compile;
- all three backing families create views;
- nested views work;
- iterators expose correct values;
- structural equality works.

---

# T5 — Cross-family mutation, key, GC and representation closure

## Goal

Prove the important interactions that unit tests of isolated types cannot establish.

## Array-backed live content

Test:

```text
Array = [a,b,c]
Slice = Array view
mutate Array slot
Slice observes new value
```

Also prove frozen const Array still rejects mutation through any alias. Slice does not provide a bypass.

## Buffer-backed live content

Test:

```text
Buffer values [a,b,c]
Slice captures visible range
replace Buffer slot
Slice observes replacement
```

Then:

```text
append to Buffer
existing Slice len unchanged
existing Slice range unchanged
```

## Vector-backed stability

Prove immutable content remains stable.

## Key rejection

Map and Set must reject:

```text
Buffer
Slice<Array>
Slice<Buffer>
Slice<Vector>
```

Preserve positive Vector immutable controls.

## Deep-GC test

Retain only deepest Slice over a backing containing heap children.

Force collection.

Prove:

```text
root backing survives
child values survive
deep Slice survives
intermediate Slice need not survive
```

## Object-size gate

Remeasure `size_of::<Object>()`.

If increased beyond baseline:

- inspect which new variant causes it;
- box the offending payload;
- repeat measurement.

### G4 exit criteria

The native object graph, key law and arena representation are coherent.

---

# T6 — Range and boundary certification

## Goal

Prove P5 did not fork sequence-range semantics.

Use `Range.rawSliceBounds` for all high-level view creation.

## Required cases for each relevant source

At least Array, Vector, Buffer and Slice should exercise:

```text
..
a..
..b
a..b
a..=b
negative lower
negative upper
negative both
lower beyond start
upper beyond end
clamping
zero-length result
empty source
one-element source
non-integral bound
reversed normalized interval according to current Range contract
```

Expected behavior must match existing Range semantics exactly.

No Rust-specific alternate bound model.

### G5 exit criteria

There is one semantic range normalizer in the language surface.

---

# T7 — Parser/P14 convergence

## Goal

Replace private List snapshot storage in `ListSource<T>` with private Array + Slice without changing parser behavior.

## Step T7.1 — preserve baseline contract

The following must remain true:

```text
caller List mutation after source construction
does not change ListSource membership/order
```

## Step T7.2 — migrate storage

Target conceptual shape:

```telos
class ListSource<T> {
    _items: Slice<T>

    new(_ items: List<T>) {
        let backing: Array<T> = <T>[*items]
        _items = backing.slice(..)
    }

    size -> Int { _items.size }
    at(_ index: Int) -> T { ... }
}
```

Adapt exact syntax to live generic/spread support.

The private Array must not need to be stored separately if Slice correctly owns it.

## Step T7.3 — strengthen parser-source tests

After constructing `ListSource<T>`:

- append to original List;
- replace original element where available;
- structurally modify original List where APIs permit;
- assert source size/content unchanged.

Also assert ordinary parser functions still behave:

```text
Cursor.current
Cursor.advance
ListSource.satisfy
ListSource.select
Parse.run
Parse.runAll
```

## Step T7.4 — run complete parse package fixture

Run:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core modules_parse_package
```

Do not treat a single `source-test.ph` pass as complete parser convergence.

## Step T7.5 — amend P14 lifecycle records

Record explicitly:

```text
historical proposal:
    dedicated ImmutableSequence<T>

actual earlier C4 implementation:
    text/Substring slice work + private List parser snapshot behavior

C1/P5 convergence:
    private Array<T> snapshot + read-only Slice<T>

result:
    no ImmutableSequence type
    no runtime-sized Vector snapshot
```

Do not rewrite history to claim the earlier proposal existed as a runtime type.

### G6 exit criteria

Parser snapshot use case works on the canonical P5 collection model.

### G6 acceptance amendment — inherited parser-package failures (2026-10-11)

The incident audit supplied by the user selects disposition 2: independently
prove P5-owned snapshot behavior while retaining the original complete-package
integration obligation. This amendment supersedes the indivisible G6 acceptance
interpretation; T7.4's command remains mandatory and its failures remain visible.

#### G6.A — P5 parser snapshot acceptance

G6.A requires executable evidence that:

1. `ListSource<T>` constructs a private Array snapshot retained by canonical Slice;
2. caller append, replacement, and middle removal independently preserve original
   membership, order, and retained references;
3. empty, singleton, and populated sources support Cursor, satisfy/select,
   `Parse.run`, and ordered `Parse.runAll`, including predicate/selection rejection,
   incomplete input, and trailing input;
4. after releasing the caller List, Slice retains the constructor-local Array
   and its heap-backed elements through a forced GC cycle;
5. the fixture uses the real parser and collection implementation, without mocks
   or replacement collection types.

The standalone fixture is
`packages/parse/tests/runtime/src/core/list-source-snapshot-test.ph`. It must not
import TextSource or `parse.text`. Require normal exit and the exact marker
`PASS core-list-source-snapshot` through both:

```bash
RUSTFLAGS='' cargo run -p phalcom-core --bin phalcom -- packages/parse/tests/runtime/src/core/list-source-snapshot-test.ph
RUSTFLAGS='' cargo test -p phalcom-core --test core parse_list_source_snapshot_fixture_passes -- --nocapture
```

The separately selectable native ownership test
`parse_list_source_snapshot_owner_survives_gc` must inspect the real source field,
prove its owner is Array rather than List/Vector, collect the unrooted caller,
and exercise Cursor/parser consumption after collection. Generic Slice tests
alone do not establish this complete object graph. Run the `parse_list_source_snapshot`
filter for both focused tests.

G6.A may pass and T7 may close only when all of this evidence passes.

#### G6.B — Complete parse-package certification

The original command remains mandatory:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core modules_parse_package
```

The initial package failures included String/Substring adapter mismatches and
optional bytes consumed as integers; the user-directed follow-up below records
their correction. G6.B remains `BASELINE_BLOCKED` until the complete package
gate passes. A P5-caused failure is active responsibility rather than a
baseline waiver.

**User-directed audit follow-up (2026-10-11):** The String/Substring and
optional-byte adapter mismatches were corrected at their parser/fixture
consumers. The complete package command now compiles the declared project and
passes 3 of 4 Rust tests; 11 of 12 runtime fixtures pass. The remaining
`source-test.ph` token-backed Cursor `current`/`advance` path reaches the
compiler's explicit associated-family invocation lowering error. Keep G6.B and
aggregate G6 `BASELINE_BLOCKED`; do not redesign compiler associated
resolution/lowering or weaken the fixture under P5. The ownership decision is
requested in `COLL006.C1.P5-G6B-associated-family-invocation-consultation.md`.
P9 retains the full package rerun after the owning-layer disposition.

#### Aggregate disposition and retained P9 obligation

Record G6.A (`PASS | FAIL | NOT_VERIFIED`) and G6.B
(`PASS | BASELINE_BLOCKED | FAIL`) independently. Aggregate G6 remains
`BASELINE_BLOCKED` while G6.B fails, even if T7 is `IMPLEMENTED / FOCUSED_TESTED`.
This does not block T9 or authorize parser/text repairs under P5.

No String/Substring contract, Option extraction behavior, comparison dispatch,
operator-fixture expectation, or Array/Slice architecture is changed by this
amendment. Snapshots provide structural independence, not recursive deep copying.

P9 must recheck both parser residuals, verify their owning corrective work, rerun
the complete package command, and compare any remaining failure against
contemporaneous main under the checkpoint residual policy. It must not claim
full parser integration certification without the clean complete-package result.

### G6 closure amendment — 2026-10-18

The current `main` tree completed the required package gate. The exact command
selected four Rust integration checks and passed all four; its runtime corpus
passed all 13 fixtures. This supersedes the earlier blocked disposition above.
G6.A remains PASS, G6.B is PASS, and aggregate G6 is PASS. P5 is therefore
`COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.

The remaining Cursor fixture issue was missing semantic context at an
unqualified `Some(...)` constructor inside an equality expression. With no
expected enum owner, `Option<Token>` was not available to resolve the nested
`Token` constructor. The fixture now spells the owner as
`Option<Token>::Some(...)` and checks variants/payloads structurally, avoiding
an unrelated value-equality requirement. This uses the existing canonical
associated resolution and static variant-construction lowering path. It does
not add generic or dynamic associated-family invocation, reconstruct AST
spelling in the compiler, erase generic arguments, or weaken lowering
invariants. Dynamic associated-family invocation remains separate follow-up
work and is not a blocker for G6.B.

The package gate and focused regressions are complete. Workspace tests,
Clippy, and release certification were not run, so P5 verification remains
`FOCUSED_TESTED`, not `RELEASE_COMPLETE`.

---

# T8 — Conversion/API convergence within P5 bounds

## Goal

Finish only the conversion edges required by P5.

Required zero-copy edges:

```text
Array<T>    -> Slice<T>
Vector<T,N> -> Slice<T>
Buffer<T>   -> Slice<T>
Slice<T>    -> Slice<T> subview
```

Allocating conversions such as:

```text
Array -> List
Vector -> Array
Vector -> List
```

already exist where implemented and must not regress.

Do not turn P5 into the final collection conversion plan.

Checked conversions to `Vector<T,N>` remain subject to the existing exact-size/static-extent semantics and can remain with their existing owner/P9 convergence unless P5 needs a narrowly scoped helper.

### G7 exit criteria

P5-required conversion graph is complete without introducing implicit collection conversions.

---

# T9 — Stabilization and lifecycle closeout

## Goal

Demonstrate P5 correctness at its owned boundary and prepare P6 takeover.

## Required focused suites

Run all new P5 tests plus:

```text
existing Array runtime tests
existing Vector runtime tests
existing relevant collection-contract tests
COLL006 semantic focused tests
static_sequence semantic focused tests
parse package runtime fixtures
GC Slice/Buffer tests
Map/Set key-admission regressions
native surface check
```

## Required build/hygiene

Run:

```text
cargo check --workspace --all-targets
git diff --check
```

Run scoped rustfmt for all changed Rust files.

If repository-wide `cargo fmt --all -- --check` still fails only due documented inherited drift, classify rather than silently repairing unrelated files.

Run owning-crate broad tests where practical under the lifecycle BUILD/CERTIFY rules.

## Native surface

Regenerate/review the native surface after adding Slice/Buffer native declarations.

Do not preserve the old P4 numeric declaration count as an invariant.

Record:

```text
old count
new count
exact intended delta
--check result
```

## Lifecycle documents

Create/update:

```text
P5 walkthrough
P5 handoff
C1 checkpoint
COLL006 program/status
LANG007 P14 convergence record
parse package docs
```

### P5 completion classification

If all P5-owned gates pass but inherited global baseline failures remain unchanged:

```text
status: COMPLETE
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
```

or the precise lifecycle classification dictated by the current convention.

Current final state (2026-10-18): all P5 gates, including the complete
parse-package gate, pass. Record P5 as
`COMPLETE / IMPLEMENTED / FOCUSED_TESTED`; workspace tests, Clippy, and release
certification remain unrun. The `BASELINE_BLOCKED` example above describes
the earlier conditional state and is not the current P5 classification.

Do not mark workspace/release certification that was not actually run.

---

# 18. Test plan

# 18.1 Canonical identity/type tests

| Test | Expected |
|---|---|
| `Slice<Int>` resolves | canonical application |
| `Slice<String>` resolves | distinct application |
| `Buffer<Int>` resolves | canonical application |
| wrong generic arity | diagnostic |
| Slice superclass | canonical Collection |
| Buffer superclass | canonical Collection |
| runtime class of Slice | canonical Slice |
| runtime class of Buffer | canonical Buffer |
| reflected generic arity | 1 |
| Slice lengths differ | same nominal type |
| Buffer capacities differ | same nominal type |
| Universe source path | canonical exact module |
| import/re-export | canonical declaration identity retained |

# 18.2 Buffer construction tests

Required:

```text
capacity 0
capacity 1
capacity N
negative
host-overflow
reservation overflow
allocation failure seam where deterministic
```

Postconditions:

```text
size == 0
capacity == requested
no initialized values
```

# 18.3 Buffer push tests

For capacity `N`:

```text
push values 0..N-1 -> success
push Nth extra      -> failure
```

Assert after failed push:

```text
size == N
capacity == N
all old values identical
extra value not retained by Buffer
```

# 18.4 Buffer replacement tests

Cover:

```text
index 0
middle
last initialized
negative valid
negative invalid
index == size
index > size
```

Assert:

```text
set never changes size
set never changes capacity
index == size never appends
```

# 18.5 Slice construction tests

For Array, Vector and Buffer:

```text
full
prefix
suffix
middle
one element
empty at start
empty in middle
empty at end
```

# 18.6 Nested Slice tests

At minimum:

```text
root -> slice
root -> slice -> slice
root -> slice -> slice -> slice
```

Assert representation:

```text
deep.owner == root backing
deep.owner != intermediate Slice
deep.start == composed start
deep.len == expected
```

# 18.7 Mutation visibility

Array:

```text
view
mutate backing slot
view sees new value
```

Buffer:

```text
view
replace initialized slot
view sees new value
append later
view length unchanged
```

Vector:

```text
view remains stable
```

# 18.8 Range tests

Exercise canonical `Range.rawSliceBounds` behavior for:

```text
open lower
open upper
both open
half-open
inclusive upper
negative bounds
out-of-range clamping
empty
single element
non-integral
reversed/effective empty according to existing policy
```

# 18.9 Iterable tests

Slice:

```text
empty
single
multi
partial
nested
source order
live Array/Buffer replacement visible before iteration
```

Buffer:

```text
empty
partially filled
full
replacement visible
spare capacity invisible
```

# 18.10 Equality tests

Slice:

```text
same owner same view          true
same values different owners  true
different values              false
different lengths             false
backing mutation changes later equality
```

Buffer:

```text
same values same capacity       true
same values different capacity  true
different value                 false
different size                  false
```

# 18.11 Key-admission tests

Map and Set must reject:

```text
Buffer
Slice over Array
Slice over Buffer
Slice over Vector
```

Retain controls:

```text
Array rejected
List rejected
Bytes rejected
Vector accepted under existing element hash law
Tuple/Record immutable controls unchanged
```

# 18.12 GC tests

## Slice root-only

```text
create backing with heap children
create Slice
retain only Slice
force GC
read elements
```

Repeat for Array, Vector, Buffer.

## Nested Slice

```text
retain only deepest Slice
force GC
```

Assert root backing survives while intermediate Slice may be reclaimed.

## Buffer child tracing

Heap children survive while Buffer rooted.

## Buffer replacement

Old replaced child becomes collectible after all other roots disappear.

# 18.13 Object representation tests

Record before/after:

```text
size_of::<Value>()
size_of::<Object>()
```

Requirements:

```text
Value remains unchanged
Object common slot ceiling does not increase
```

# 18.14 Parser/P14 tests

Retain current ListSource snapshot test.

Extend:

```text
mutate caller after ListSource construction
source membership/order unchanged
parser operations unchanged
```

Run all parse package fixtures.

# 18.15 Regression tests

Re-run at least:

```text
Array runtime suite
Vector runtime suite
COLL006 semantic filter
static_sequence filter
const Array frozen-mutation cases
Vector exact extent cases
native surface
module/package loading
```

P5 must not change bracket-family selection or static sequence behavior.

---

# 19. Failure/atomicity requirements

Every failing Buffer operation must leave the object internally valid.

## Constructor failure

No partially initialized published Buffer.

## Push at capacity

No mutation.

## Invalid set/index

No mutation.

## Slice construction failure

No partially published invalid Slice.

## Invalid nested bounds

No owner/range mutation; Slice objects are immutable after construction anyway.

## Parser snapshot construction failure

No partially initialized `ListSource` object should become observable.

---

# 20. Resource and arithmetic safety

All conversions from Telos numeric values to host indexes/capacities must be checked.

Avoid:

```rust
value as usize
len as u32
start + len
```

where overflow/truncation can be user-influenced.

Prefer:

```rust
usize::try_from(...)
checked_add(...)
checked_sub(...)
try_reserve_exact(...)
```

The exact helper types should follow current runtime conventions.

P5 should not add an arbitrary language-wide maximum Buffer capacity unless an existing VM resource policy provides one or a documented correctness/security requirement demands it.

---

# 21. Re-entry discipline

Slice/Buffer primitives should keep the native floor mechanical.

Operations that invoke user behavior such as:

```text
==
hash
toString
```

should be implemented at the Telos level where possible.

Do not hold a live mutable/immutable heap object borrow across a Telos send.

This is particularly important for structural equality.

---

# 22. Implementation boundaries with P4

P5 must not edit P4 machinery merely for convenience.

Protected semantic/compiler areas include:

```text
phalcom-semantic/src/checker/sequence_literal.rs
phalcom-semantic/src/static_value/
phalcom-semantic/src/static_eval/
phalcom-semantic/src/materialization.rs
sequence-literal compiler lowering
static-constant Array/Vector materialization
```

An edit in one of these areas is not automatically forbidden, but it is a **review trigger**: the implementer must demonstrate why P5 correctness genuinely requires it.

Expected P5 implementation should be achievable without changing collection-literal semantics.

---

# 23. Stop/consult conditions

Stop implementation and obtain architectural review if any of the following appears necessary:

1. storing a raw pointer into backing collection storage;
2. allowing Slice owner chains rather than root normalization;
3. adding live List zero-copy views;
4. adding Buffer shrink/remove semantics;
5. allowing sparse Buffer initialization;
6. introducing automatic Buffer growth;
7. changing Slice/Buffer nominal identity based on length/capacity;
8. adding Slice/Buffer to bracket literal family selection;
9. adding static-value or static-materialization variants for Slice/Buffer;
10. changing `SequenceLiteralResolution`;
11. adding a runtime existential Vector form;
12. adding new inference/cardinality solver machinery;
13. introducing a dedicated `ImmutableSequence<T>`;
14. making parser snapshot semantics depend on `Vector<T,N>`;
15. growing the common Object arena slot when boxing would avoid it;
16. crossing into P6 ordered-hash/Map/Set storage redesign;
17. requiring a generic variance redesign;
18. changing canonical `Range.rawSliceBounds` semantics;
19. requiring a new parser source abstraction incompatible with private Array + Slice;
20. any conflict between workflow documents and the live P1–P4 implemented authority model.

---

# 24. Code-quality requirements

P5 should keep the implementation intentionally small and explicit.

## Rust

Prefer:

- small representation structs;
- invariant-preserving constructors;
- narrow accessors;
- checked arithmetic;
- fallible allocation;
- exhaustive object matches;
- clear module-level representation comments;
- no `unsafe` for collection backing ownership;
- no duplicated range/index normalization.

## Telos

Prefer:

- strongly typed method signatures;
- reuse of `CanonicalIterable`;
- reuse of existing `Range` normalization;
- straightforward loops;
- no Dynamic-based overload shortcuts;
- no hidden collection conversions;
- public methods that reveal the semantic distinction between fixed views and bounded owners.

---

# 25. Suggested implementation checkpoints/commits

The implementer may use commits roughly along these boundaries:

```text
1. feat(COLL006): add Slice/Buffer canonical universe identities
2. feat(COLL006): add bounded Buffer runtime storage
3. feat(COLL006): add owner-rooted Slice runtime views
4. feat(COLL006): expose Slice/Buffer collection protocols
5. test(COLL006): certify Slice/Buffer GC and key invariants
6. feat(COLL006): converge parse snapshots on Array and Slice
7. docs(COLL006): complete P5 lifecycle records
```

Do not force this exact history if the branch workflow prefers fewer commits. The important property is that intermediate commits remain coherent enough to debug/bisect.

---

# 26. Completion checklist

## Canonical identity

- [x] `UniverseKey::Slice`
- [x] `UniverseKey::Buffer`
- [x] exact source paths
- [x] class relations
- [x] Universe bindings
- [x] generic type forms
- [x] `CoreDeclarationIds::slice`
- [x] `CoreDeclarationIds::buffer`
- [x] runtime classes
- [x] `native_repr`
- [x] canonical `.class`

## Buffer

- [x] owned runtime representation
- [x] logical capacity separate from `Vec::capacity`
- [x] initialized-prefix-only storage
- [x] checked construction
- [x] fallible reservation
- [x] size
- [x] capacity
- [x] index read
- [x] set existing initialized index
- [x] push
- [x] full-capacity failure
- [x] atomic failure
- [x] no growth
- [x] no shrink
- [x] iterable
- [x] structural equality
- [x] key rejection
- [x] GC tracing

## Slice

- [x] owner/start/len representation
- [x] no raw pointer
- [x] Array source
- [x] Vector source
- [x] Buffer source
- [x] nested root normalization
- [x] checked arithmetic
- [x] point index
- [x] range slicing
- [x] read-only API
- [x] iterable
- [x] structural equality
- [x] key rejection
- [x] GC owner tracing
- [x] live Array mutation visibility
- [x] live Buffer replacement visibility
- [x] fixed view length after Buffer append

## Parser/P14

- [x] no `ImmutableSequence`
- [x] no runtime-sized Vector snapshot
- [x] `ListSource` owns/retains Slice
- [x] backing is independent Array snapshot
- [x] caller mutation isolation retained (independent G6.A fixture)
- [x] G6.A parser snapshot/GC acceptance passes
- [x] G6.B complete parse package passes (4/4 Rust integration checks; 13/13 runtime fixtures)
- [x] P14 lifecycle wording amended

## Representation/performance

- [x] `Value` size unchanged
- [x] `Object` slot ceiling unchanged
- [x] Slice construction representation proves zero-copy
- [x] nested views remain direct-root
- [x] Buffer push does not allocate after successful initial reserve under ordinary allowed use

## Regression

- [x] Array tests
- [x] Vector tests
- [x] COLL006 semantic tests
- [x] static_sequence tests
- [x] frozen const Array controls
- [x] native surface
- [x] workspace all-target check
- [x] diff check
- [x] scoped formatting

---

# 27. Required closeout statement

P5 should not be marked complete until a statement materially equivalent to the following is true:

```yaml
slice:
  canonical_type: "Slice<T>"
  runtime_length_in_type: false
  read_only: true
  zero_copy: true
  owner_rooted: true
  raw_pointer_lifetime_dependency: false
  supported_backings:
    - Array
    - Vector
    - Buffer
  nested_views_normalize_to_root: true
  point_index_complexity: "O(1)"
  construction_complexity: "O(1)"
  array_mutation_visible: true
  buffer_replacement_visible: true
  buffer_append_changes_existing_view_length: false
  vector_backing_stable: true
  map_set_key_safe: false
  gc_owner_edge_verified: true

buffer:
  canonical_type: "Buffer<T>"
  capacity_runtime: true
  size_runtime: true
  invariant: "0 <= size <= capacity"
  initialized_prefix_only: true
  spare_capacity_contains_values: false
  automatic_growth: false
  shrink_v1: false
  sparse_initialization: false
  push_at_capacity_fails_atomically: true
  index_complexity: "O(1)"
  set_complexity: "O(1)"
  push_complexity: "O(1)"
  gc_traces_initialized_values_only: true
  map_set_key_safe: false

semantic_boundaries:
  bracket_literal_family_added: false
  static_value_kind_added: false
  static_materialization_kind_added: false
  sequence_inference_changed: false
  vector_identity_changed: false

parser_snapshot:
  dedicated_immutable_sequence_added: false
  runtime_dimensioned_vector_used: false
  backing: "private Array<T>"
  consumer_view: "Slice<T>"
  caller_list_mutation_isolated: true
  snapshot_acceptance_g6_a: pass
  parse_package_gate_g6_b: PASS # 4/4 Rust integration checks; 13/13 runtime fixtures (2026-10-18)
  aggregate_g6: PASS

runtime:
  value_size_regressed: false
  object_slot_ceiling_regressed: false
  forced_gc_view_owner_proof: pass
  deep_slice_root_normalization: pass
  native_surface_current: true

verification:
  p5_owned_gates: pass
  p4_adjacent_regressions: pass
  workspace_all_targets_check: pass
  inherited_baseline_failures_reclassified_as_p5: false
```

---

# 28. P6 handoff boundary

P5 hands P6 a collection runtime with the following additional stable facts:

```text
Slice<T>
    exists as canonical read-only zero-copy view
    not key-safe

Buffer<T>
    exists as canonical bounded mutable sequence
    not key-safe

Array / Vector / Buffer
    all provide Slice views

parser token snapshots
    no longer require a duplicate immutable sequence family

mutable-key/key-stability guard
    accounts for Buffer and Slice
```

P6 may then focus exclusively on:

```text
reusable ordered-hash backing
Map/Set preservation
MutableBag
IndexedDict
OrderedSet
re-entry-safe hash/equality behavior
```

P6 must not absorb unfinished P5 view/storage correctness.

---

# 29. Final implementation direction

The implementation order should remain:

```text
T0  ground branch + baselines + match census
 ↓
T1  canonical Slice/Buffer identity
 ↓
T2  Buffer bounded-storage substrate
 ↓
T3  Slice owner-rooted view substrate
 ↓
T4  typed Telos APIs + iterators + equality
 ↓
T5  cross-family mutation/GC/key/representation proof
 ↓
T6  range-bound certification
 ↓
T7  parser/P14 convergence
 ↓
T8  P5-bounded conversion convergence
 ↓
T9  regression/certification/lifecycle closeout
```

The architectural center of P5 is not syntax or type inference. It is the correctness of these two statements:

```text
A Slice is a constant-range read-only projection whose backing owner remains
alive and whose indexing cost does not grow with slicing depth.

A Buffer is an owned sequence whose initialized prefix is always exactly its
semantic size and whose size can never exceed its logical capacity.
```

If the implementation preserves those laws, reuses the established Array/Vector/Range/GC infrastructure, and converges the parser snapshot without introducing a duplicate immutable-sequence type, P5 will complete the contiguous collection layer cleanly and leave P6 with a stable foundation.

---
id: COLL006.C1
category: COLL
program: COLL006
checkpoint: COLL006.C1
kind: checkpoint-record
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
requires:
  - LANG007.C4.P3
---

# COLL006.C1 — Extended collections foundation and first-generation collection family

COLL006.C1 is the bounded acceptance objective for Telos's first complete extended-collections generation.

It combines the language-level homogeneous-sequence literal upgrade with the runtime/static sequence families and the standard-library/storage foundations needed for the new associative, ordered, persistent, and interval collections.

This record is the **single durable shared state authority** for all numbered plans belonging to `COLL006.C1`.

P1, P2, P3, and P4 are complete at their bounded implementation scope. P4 owns static sequence values/materialization and the bare-bracket default cutover; P5 is the next proposed plan for Slice/Buffer.

---

## 1. Acceptance objective and ownership boundary

C1 is accepted only when all of the following are true:

1. bracket syntax is represented neutrally rather than as an intrinsically `List` AST construct;
2. contextual collection-literal resolution is semantic-checker-owned and can realize bracket literals as `Array`, `List`, or `Vector`;
3. `_` generic/static inference holes required by collection literals and inference-producing annotations are integrated with the canonical inference solver and never escape unsolved;
4. runtime bare bracket literals default to `Array<T>`;
5. static/`const` bare bracket literals default to `Vector<T, N>`;
6. `<T>[...]` explicitly selects `Array<T>`;
7. `<T, N>[...]` explicitly selects `Vector<T, N>`;
8. ordinary expected types can select `List<T>`, `Array<T>`, or `Vector<T,N>` without intermediate collection allocation/conversion;
9. `Array<T>`, `List<T>`, and `Vector<T,N>` have coherent, tested runtime representations and mutation/extent semantics;
10. explicit constant Arrays are statically evaluable and cannot be mutated after runtime materialization;
11. `Vector<T,N>` is statically dimensioned and immutable in both runtime and const phases;
12. static homogeneous sequence values participate in canonical static identity, metadata, and materialization without creating a second compile-time execution model;
13. `Slice<T>` and `Buffer<T>` provide the first coherent contiguous-view/bounded-storage layer;
14. Map/Set ordered-hash machinery is factored safely enough to implement `MutableBag`, `IndexedDict`, and `OrderedSet` without duplicating re-entrant hash/equality logic;
15. a persistent hash substrate supports `Bag` and `Bijection` with structural sharing and old-root immutability;
16. `Bound<T>` / `Interval<T>` and persistent ordered storage support `RangeMap<K,V>`;
17. `Entry<K,V>` provides the canonical explicit mapping-association value for the new mapping constructors;
18. migration from legacy bare-bracket-`List` semantics is complete for owned repository source and tests;
19. GC/rooting, re-entrant user code, static materialization, equality/hash laws, collection invariants, and required performance contracts have focused acceptance evidence;
20. residual failures are classified and the checkpoint record truthfully reflects final completion and verification state.

### 1.1 C1 owns

C1 owns implementation and integration of:

```text
contextual homogeneous bracket literals
collection inference holes
Array<T>
List<T> literal/default migration
Vector<T,N>
static homogeneous collection values
Slice<T>
Buffer<T>
Entry<K,V>
Bag<T>
MutableBag<T>
Bijection<K,V>
IndexedDict<K,V>
OrderedSet<T>
Bound<T>
Interval<T>
RangeMap<K,V>
ordered mutable hash storage
persistent hash storage
persistent ordered storage
collection-specific migration, diagnostics, GC and performance proof
```

### 1.2 C1 does not own

C1 must not expand into:

- a new free-function declaration model;
- a second `Iterator` / `Iterable` hierarchy;
- a trait also named `Collection`;
- arbitrary declaration-signature inference holes unrelated to inference-producing initializer contexts;
- a new compile-time VM;
- a second static identity/materialization model competing with LANG007.C4;
- a general ownership/borrow checker solely to avoid the first const-Array runtime guard;
- mutable Slice capability design;
- persistent List/PersistentVector/Deque/LinkedList/Queue/Stack families;
- a wholesale redesign of existing numeric `Range` iteration semantics;
- collection-private `Hashable`, `Comparable`, or `TotalOrder` traits;
- v1 unboxed/SIMD numerical storage as a correctness prerequisite;
- opportunistic repair of unrelated repository failures.

---

## 2. Authority model

C1 preserves four separate authorities:

```text
Desired language behavior
    explicit user rulings + accepted normative decisions/specifications

Current repository reality
    live source + live tests + current repository revision

Implementation architecture
    numbered COLL006.C1 plans + approved amendments/consultations

Current work state
    this checkpoint record + scoped implementation evidence
```

Rules:

- accepted C1 language behavior may require superseding legacy collection-literal decisions;
- legacy tests encoding bare-bracket-`List` behavior are migration targets, not semantic authorities after the new rule is ratified;
- live source may mechanically differ from plan paths/symbols;
- material architectural drift requires consultation rather than silent adaptation;
- the compiler must consume semantic products rather than reconstructing collection-family decisions.

---

## 3. Ratified target semantics

The following are checkpoint requirements, not claims about the current implementation.

### 3.1 Sequence families

```text
List<T>
    homogeneous
    mutable
    dynamically resizable
    runtime extent
    explicitly requested after the literal cutover

Array<T>
    homogeneous
    owned
    contiguous Value-slot storage in v1
    runtime extent
    fixed extent after construction
    mutable elements for ordinary runtime instances
    explicitly constructible/evaluable as a const Array

Vector<T, N>
    homogeneous
    owned
    immutable
    contiguous Value-slot storage in v1
    exact static extent N
    valid as a runtime or const value
```

### 3.2 Literal defaults and explicit heads

```telos
let x = [1, 2, 3]
// Array<Int>

const y = [1, 2, 3]
// Vector<Int, 3>

let z: List<_> = [1, 2, 3]
// List<Int>

const a: Array<_> = [1, 2, 3]
// static Array<Int>

let b = <_>[1, 2, 3]
// Array<Int>

const c = <Int>[1, 2, 3]
// static Array<Int>

let d = <_, _>[1, 2, 3]
// Vector<Int, 3>

const e = <Int, _>[1, 2, 3]
// static Vector<Int, 3>
```

Interpretation:

```text
[...]       family selected by unique context, otherwise phase default
<T>[...]    explicit Array family
<T,N>[...]  explicit Vector family
```

`List` is explicitly requested through a contextual type or constructor rather than the anonymous literal head.

### 3.3 Context and phase precedence

Family selection must obey:

```text
1. explicit literal family
2. unique concrete expected collection family
3. evaluation-phase default
```

The phase default must not break overload ambiguity.

If more than one contextual collection family is viable, compilation reports ambiguity and the programmer may disambiguate with `<...>[...]` or a named collection constructor/context.

### 3.4 Inference holes

`_` means:

> create a query-local inference variable of the parameter's declared kind/domain and solve it from available constraints.

It does not mean:

```text
Dynamic
Any
existential wildcard
ignored argument
fallback type
```

No inference hole may survive into a durable binding type, exported metadata, reflection product, compiler product, or runtime type environment.

### 3.5 Binding and phase semantics

C1 preserves TDR-0090 binding semantics:

```text
let      runtime, non-rebindable binding
let mut  runtime, rebindable binding
const    static/compile-time binding
```

Collection interior mutability is separate from binding rebinding.

An ordinary runtime `Array<T>` remains element-mutable even when held by `let`.

A const Array remains `Array<T>` rather than becoming `ConstArray<T>`, but runtime materialization must preserve its immutability through aliases.

### 3.6 Additional collection families

```text
Slice<T>
    read-only zero-copy contiguous view
    runtime length
    owner + offset + length semantics
    v1 sources: Array, Vector, Buffer

Buffer<T>
    mutable owned bounded-capacity storage
    invariant 0 <= size <= capacity
    initialized prefix only
    no implicit capacity growth

Bag<T>
    persistent immutable multiset
    value -> positive multiplicity

MutableBag<T>
    mutable multiset

IndexedDict<K,V>
    mutable mapping
    semantic positional order
    key lookup + positional access/mutation
    order-sensitive equality

OrderedSet<T>
    mutable unique sequence/set hybrid
    semantic positional order
    order-sensitive equality

Bijection<K,V>
    persistent immutable one-to-one bidirectional mapping

RangeMap<K,V>
    persistent mapping from non-overlapping Interval<K> regions to V
    point lookup by containment
```

`SetList` is not part of this checkpoint vocabulary; the canonical name is `OrderedSet`.

---

## 4. Historical pre-P1 repository grounding

Baseline inspected while defining C1:

```text
branch: main
revision: 800474d8a9f2e1e12f422bf3132729c4e69012d4
```

The following facts describe the historical preparation revision. Current takeover interfaces are recorded below and in the latest plan handoff.

1. The AST currently has `Expr::ListLiteral` / `ListLiteralExpr`.
2. The semantic checker currently synthesizes bracket literals through `synthesize_list_literal` and canonical `List`.
3. Compiler lowering currently emits `BuildList` or dynamic List-literal builder bytecodes.
4. The VM currently materializes those bytecodes as `ListObject`.
5. Accepted legacy collection-literal decisions (`0025`, `0028`) encode bare `[...]` as List construction and therefore require explicit amendment/supersession at the semantic cutover.
6. `ListObject` is backed by `Vec<Value>` and already provides the dynamically-resizable sequence substrate.
7. `MapObject` is insertion ordered and uses a vector plus hash index with Telos-level hash/equality re-entry.
8. `Set<T>` reuses Map backing, preserves encounter order, exposes indexed `at`, but equality is order-insensitive.
9. Current `Range` is a numeric/progression abstraction, not a generic interval type.
10. Canonical `Iterator` and `Iterable` traits already exist with associated `Item` / `Iterator` types and substantial default algorithms.
11. `Collection` already exists as a nominal Universe superclass.
12. The inference solver already has query-local `InferenceTerm::Var` and applied inference terms with heterogeneous static arguments.
13. Generic/static type applications can carry proper Type arguments and static terms, making `Vector<T,N>` compatible with the existing general type representation.
14. `StaticValueStore` currently lacks a reusable homogeneous sequence node.
15. The static evaluator handles structural products such as Tuple/Record but not current List literals as canonical static homogeneous sequences.
16. Existing `map.ph` contains a nongeneric `Entry` class with mutable fields despite describing the value as immutable; Map iteration currently exposes labeled `(key: K, value: V)` tuples.
17. No reusable persistent hash-map substrate is currently established for Bag/Bijection.
18. `LANG007.C4.P14` currently describes an owned immutable homogeneous indexed snapshot abstraction; dimensioned `Vector<T,N>` changes the correct ownership of that parser-snapshot need and requires coordinated amendment rather than silent reuse.

These facts are repository grounding, not permanent architecture if later approved plans intentionally change them.

---

## 5. Numbered plans ledger

The ledger records current C1 execution state. Proposed successor plans retain the original decomposition.

| Plan | Scope | Status | Completion | Verification | Intended outcome |
|---|---|---|---|---|---|
| `COLL006.C1.P1` | Neutral bracket-literal AST/semantic architecture; `<...>[...]`; `_` inference holes; inference-bearing partial binding annotations | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Establish semantic authority and solver foundation without changing bare-literal defaults |
| `COLL006.C1.P2` | `Array<T>` runtime/native representation, explicit/contextual Array literals, direct lowering, iteration, GC | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Complete runtime fixed-after-construction homogeneous sequence |
| `COLL006.C1.P3` | `Vector<T,N>` runtime/native representation, static extent inference/cardinality proof, constructors, equality/hash, direct lowering | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Vector runtime/static formation, solved-only publication, negative extent rejection, Array homogeneous contracts, union/deferred publication, incremental parity, and focused runtime/compiler gates pass |
| `COLL006.C1.P4` | Static homogeneous sequence model; const Vector; const Array; materialization; repository-wide bracket-default cutover/migration | COMPLETE | IMPLEMENTED | BASELINE_BLOCKED | T10/T11 cutover and stabilization complete; G7 focused evidence passes except inherited D4 static-constant failures |
| `COLL006.C1.P5` | `Slice<T>`, `Buffer<T>`, contiguous storage/view integration, parser-snapshot/P14 convergence | PROPOSED | NOT_STARTED | UNVERIFIED | Complete contiguous view/bounded-construction family |
| `COLL006.C1.P6` | Reusable ordered-hash backing; preserve Map/Set; add `MutableBag`, `IndexedDict`, `OrderedSet` | PROPOSED | NOT_STARTED | UNVERIFIED | Complete mutable ordered/hash-indexed family without duplicating re-entry logic |
| `COLL006.C1.P7` | Persistent hash substrate; `Bag<T>`; `Bijection<K,V>` | PROPOSED | NOT_STARTED | UNVERIFIED | Complete persistent hash collections with structural sharing |
| `COLL006.C1.P8` | `Bound<T>`, `Interval<T>`, persistent ordered storage, `RangeMap<K,V>` | PROPOSED | NOT_STARTED | UNVERIFIED | Complete generic interval-domain mapping |
| `COLL006.C1.P9` | API/conversion convergence, `Entry<K,V>` migration boundary, diagnostics, property/GC/performance certification, docs/checkpoint closure | PROPOSED | NOT_STARTED | UNVERIFIED | Certify the complete C1 collection family |

Corrective plans required to make the C1 acceptance objective truthful remain inside C1 rather than creating a new checkpoint.

---

## 6. Target architecture and ownership

The following architecture is fixed for C1 planning unless explicitly amended.

### 6.1 Parser

Parser owns only source formation:

```text
[...]
<T>[...]
<T,N>[...]
_
```

It must not decide whether a bare literal is Array/List/Vector from evaluation phase or contextual type.

### 6.2 Semantic checker

Semantic analysis owns:

- collection-family selection;
- expected-type propagation;
- phase-sensitive defaulting;
- element-type inference;
- Vector extent inference;
- literal/head conflict diagnostics;
- inference-hole ownership and completion;
- compile-time cardinality proof where statically available.

The checker should publish a durable checked resolution keyed by expression identity, conceptually:

```text
SequenceLiteralResolution
    family
    result_type
    element_type
    extent where applicable
    checked expansion/construction plan
```

Exact private names may differ.

### 6.3 Compiler

Compiler lowering consumes the checked semantic product.

It must not:

- inspect phase and choose a collection itself;
- count elements and decide `N` independently;
- default missing semantic information to List/Array;
- perform an implicit intermediate collection conversion.

Known-size direct literals should lower directly to their target collection.

### 6.4 Static evaluator

The static evaluator consumes checked collection construction identity and produces canonical static homogeneous sequence values.

It must not execute arbitrary runtime constructor bytecode as a compile-time VM.

A shared static sequence representation should preserve exact `result_type`, allowing Array and Vector to share storage machinery without sharing type identity.

### 6.5 Runtime/VM

Runtime owns:

- native dense storage;
- bounds/mutation enforcement;
- const-Array runtime immutability defense;
- Buffer logical capacity;
- Slice backing references;
- ordered/persistent storage mechanisms;
- GC tracing;
- execution-only re-entry guards.

Runtime representation must not become language type identity.

### 6.6 Collection protocol surface

High-level algorithms should remain in Telos source where efficiently derivable.

Native primitives should be the minimal storage/mechanism floor.

Every iterable collection reuses canonical `Iterator` / `Iterable`.

---

## 7. Durable semantic invariants for C1

These are accepted checkpoint requirements to be established by implementation.

### 7.1 Literal and inference invariants

- Bare runtime `[...]` defaults to `Array<T>` only after the P4 atomic cutover.
- Bare static `[...]` defaults to `Vector<T,N>` only after the P4 atomic cutover.
- Before that cutover, infrastructure plans preserve current behavior except where explicit/contextual new-family construction is being tested.
- `<T>[...]` identifies Array family.
- `<T,N>[...]` identifies Vector family.
- A unique expected `List<T>`, `Array<T>`, or `Vector<T,N>` selects that family.
- Phase defaulting never resolves an overload ambiguity between multiple viable collection families.
- `_` is a solver variable with the target parameter's domain/kind.
- An underconstrained `_` produces a deterministic diagnostic.
- Unsolved inference variables never become canonical `TypeId`s or durable products.

### 7.2 Sequence invariants

- `List<T>` remains dynamically resizable.
- `Array<T>` has runtime extent fixed after construction.
- Ordinary runtime Array permits indexed element replacement but not structural resize.
- Array extent is not part of its type.
- `Vector<T,N>` has exactly `N` elements for every runtime/static value.
- Vector is immutable in every phase.
- `N` is a static `Int` argument and cannot be negative.
- A Vector literal with runtime-unknown spread cardinality cannot pretend to have inferred static `N`.
- Generic dense-sequence contiguity in v1 means contiguous Telos `Value` slots, not a promised C ABI layout of `T`.

### 7.3 Const/static invariants

- `const` is phase/static availability, not a separate collection classifier.
- `const Array<T>` is legal when explicitly requested.
- A materialized const Array cannot be mutated through direct or aliased runtime access.
- Const Array does not become value-hashable merely because one instance is frozen.
- `const Vector<T,N>` materializes as an intrinsically immutable Vector.
- Static collection initializers are not re-executed during runtime materialization.
- C1 extends C4 static-value/materialization authority; it does not create a competing system.

### 7.4 Slice/Buffer invariants

- Slice v1 is read-only.
- Slice stores a strong backing owner plus logical range; no raw interior pointer is source-visible or relied upon for lifetime correctness.
- Zero-copy v1 Slice sources are structurally stable collections (Array, Vector, Buffer).
- Buffer maintains `0 <= size <= capacity`.
- Buffer reads only the initialized prefix.
- Buffer does not implicitly grow beyond logical capacity.

### 7.5 Hash/ordered collection invariants

- New hash-based collections use Telos `hash` / `==` semantics, not Rust identity/equality over `Value`.
- No native collection holds a live Heap borrow across re-entrant Telos hash/equality/order sends.
- IndexedDict positional order is semantic and participates in equality.
- OrderedSet positional order is semantic and participates in equality.
- Current Map/Set semantics must not regress merely because their backing is factored.
- OrderedSet uniqueness and index maps never diverge.
- IndexedDict key and positional indexes never diverge.
- MutableBag stores no zero/negative multiplicity entries.

### 7.6 Persistent collection invariants

- Persistent updates never mutate a node reachable from an older public root.
- Bag equality is multiplicity-based and representation-order independent.
- Bijection forward and reverse roots are exact inverses.
- Failed Bijection updates publish no half-updated state.
- RangeMap stores no overlapping intervals.
- RangeMap point lookup returns at most one mapping.
- Persistent-node graphs remain fully visible to GC tracing despite structural sharing.

---

## 8. Representation direction

Exact private layouts remain mechanically flexible, but the following representation direction is the baseline for numbered plans.

| Type / substrate | Baseline representation direction |
|---|---|
| `List<T>` | existing `Vec<Value>` growable storage |
| `Array<T>` | fixed contiguous `Box<[Value]>`-like storage + runtime const/frozen state or equivalent |
| `Vector<T,N>` | fixed immutable contiguous `Box<[Value]>` / shareable equivalent |
| `Slice<T>` | backing owner + start + length + backing-kind/adaptor metadata as needed |
| `Buffer<T>` | logical capacity + initialized `Vec<Value>` prefix |
| mutable ordered hash | generalized current MapObject vector + hash-index machinery |
| persistent hash | structurally shared immutable hash trie / equivalent |
| persistent ordered | deterministic structurally shared balanced ordered tree / equivalent |

V1 must not expose native `Value` storage as a stable public ABI. Later numeric specialization of `Array<Float>` / `Vector<Float,N>` must remain possible.

---

## 9. Stable interface / takeover map

P1 and P2 established the interfaces described in their handoffs. The following table is the historical preparation map. P3 adds `checker/sequence_literal.rs`, shared `phalcom_type_meta::StaticInt`, `Object::Vector`, and checked extent/expansion lowering; see `COLL006.C1.P3-handoff.md` for the current takeover map and limitations.

The first plans inherit these current repository authorities:

| Concern | Current authority |
|---|---|
| Bracket AST | `phalcom-ast/src/ast.rs` `Expr::ListLiteral` / `ListLiteralExpr` |
| Bracket parsing | `phalcom-ast/src/parser.rs` list-literal parser |
| Literal checking | `phalcom-semantic/src/checker/expression.rs` `synthesize_list_literal` path |
| Binding annotation checking | `phalcom-semantic/src/checker/statement.rs` + `ExpectedType` |
| Generic inference | `phalcom-semantic/src/checker/inference.rs` |
| Type applications/static arguments | `phalcom-semantic/src/types/store.rs` |
| Static values | `phalcom-semantic/src/static_value/store.rs` |
| Static execution | `phalcom-semantic/src/static_eval/machine.rs` |
| Static materialization | `phalcom-semantic/src/materialization.rs` and C4-owned metadata/lowering seams |
| Literal lowering | `phalcom-core/src/compiler/lib/expr.rs` |
| List runtime | `phalcom-core/src/heap/list.rs` + list primitives |
| Map/Set runtime | `phalcom-core/src/heap/map.rs` + map/set primitives |
| Heap object variants | `phalcom-core/src/heap/object.rs` |
| Universe identities | `phalcom-native-meta/src/universe.rs` |
| Universe class bootstrap | `phalcom-core/src/universe/core_classes.rs` |
| Iterable laws | `phalcom-core/core/universe/src/collections/iterator_laws.ph` |
| List/Map/Set/Range/Bytes surfaces | `phalcom-core/core/universe/src/collections/*.ph` |

Every numbered plan must verify relevant symbols/paths against its starting revision before editing.

---

## 10. Normative and decision work required

C1 cannot honestly close while legacy accepted decisions still claim bare brackets are intrinsically List literals.

The semantic-cutover plan must amend or supersede, as applicable:

```text
docs/decisions/accepted/0025-list-literal-syntax.md
docs/decisions/accepted/0028-collections-representation-and-literals.md
```

It must preserve the current binding/static authority of TDR-0090.

C1 also requires coordinated treatment of the existing `LANG007.C4.P14` immutable-snapshot proposal because `Vector<T,N>` is statically dimensioned and therefore is not the natural type for an arbitrary runtime-length parser snapshot.

Recommended ownership direction for P14 convergence:

```text
runtime parser snapshot storage:
    privately owned Array<T>

read-only exposure where needed:
    Slice<T>
```

Any material departure from that direction must be recorded as a C1 consultation/amendment rather than silently changing either program.

---

## 11. Coverage obligation ledger

These are designed correctness dimensions. They do not imply that every test runs during every build task.

| Dimension | Required C1 coverage |
|---|---|
| Literal defaults | runtime bare -> Array; static bare -> Vector |
| Explicit literal heads | `<T>` Array; `<T,N>` Vector; explicit/context conflicts |
| Contextual inference | binding, argument, return, component/field, nested expected type, generic call |
| Inference holes | kind/domain correctness, solving, underconstraint, forbidden durable-signature positions |
| Empty/single/many | empty typed literals, N=0 Vector, one element, large literal |
| Spread | runtime-length Array/List spread; only statically exact Vector spread |
| Binding semantics | `let` vs `let mut` rebinding; interior Array mutation; Vector immutability |
| Const semantics | const Vector; explicit const Array; alias mutation defense; no initializer re-execution |
| Type identity | Array length not in type; Vector N in type; cross-module identity |
| Equality/hash | structural laws; immutable/persistent value hash laws; mutable-key restrictions |
| GC | dense storage, Slice owner, Buffer prefix, persistent shared nodes |
| Re-entry | user-defined hash/equality/order callbacks and failure recovery |
| Persistence | old-root immutability, structural sharing, collision paths |
| Ordered hybrids | index/key/member mirrors after insert/remove/move |
| RangeMap | endpoint semantics, non-overlap, overlay split, unbounded intervals, lookup |
| Diagnostics | ambiguity, size mismatch, const mutation, duplicate element, capacity, conflicts, overlap |
| Performance | direct literal allocation, O(1) indexing, hash lookup, persistent update, O(log n) RangeMap |
| Migration | owned source depending on legacy bare List semantics is explicit after cutover |

---

## 12. Verification execution doctrine

Completed focused evidence is recorded in §13. P3 runtime verification remains incomplete.

Numbered plans should use the repository verification ladder:

```text
T0 exact reproducer/new regression
T1 directly affected feature tests
T2 owning subsystem/module suite
T3 adjacent cross-layer integration
T4 broad crate/package verification
T5 workspace/release verification
```

Expected policy:

- BUILD mode: T0/T1 only as needed for discriminating feedback;
- STABILIZE mode: directly affected semantic/compiler/runtime suites;
- CERTIFY mode: broader gates at coherent plan/checkpoint boundaries;
- do not run workspace-wide rituals after every storage or parser edit;
- classify unrelated failures rather than repairing them opportunistically.

`RELEASE_COMPLETE` must not be claimed unless its actual broad evidence exists.

---

## 13. Verification ledger

| Gate | Scope | Evidence | Result | Proves |
|---|---|---|---|---|
| `COLL006.C1.P1.G7` | Syntax, inference holes, evaluation phase, sequence resolution, lowering | `cargo check --workspace --all-targets`, focused parser/semantic tests | `FOCUSED_TESTED` | Proves neutral syntax, inference holes, sequence literal publication and lowering fail-closed boundary |
| `COLL006.C1.P3.G1` | Static generic formation and Vector semantic identity | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic coll006` | 22 passed | Includes 7 new P3 tests; proves concrete/symbolic/large static arguments, partial holes, eight Vector literal/context combinations, and retained P1/P2 compatibility |
| `COLL006.C1.P3.G2` | Initial core compile gate | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --all-targets` | Passed before adding the final runtime test file | Compile evidence for initial runtime/lowering slice; does not prove execution |
| `COLL006.C1.P3.G3` | Vector runtime gate | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_vector` | INCOMPLETE: user-aborted during compilation | No runtime result or selected-test count was obtained |

### P4 T11/G7 stabilization evidence — 2026-10-09

| Gate | Scope | Evidence | Result | Proves |
|---|---|---|---|---|
| `COLL006.C1.P4.T11` | Incremental static sequence edits and publication | `cargo test -p phalcom-semantic --test semantic incremental::coll006_c1_p3_publication` | 3 passed | Cold/incremental parity, phase changes, content/extent changes, exact-spread loss, invalid/valid transitions, stale-value absence, descriptor/fingerprint changes, and unaffected product reuse |
| `COLL006.C1.P4.T11` | T11 source-order/extent regression | `cargo test -p phalcom-core --test core core_vector` | 9 passed | Explicit Vector child evaluation is exactly once/source ordered; zero extent is supported; existing immutability, GC, spread, and API contracts remain green |
| `COLL006.C1.P4.G7` | COLL006 semantic acceptance | `cargo test -p phalcom-semantic --test semantic coll006` | 51 passed | P1–P4 sequence semantics, static identity, cutover, and incremental regressions |
| `COLL006.C1.P4.G7` | Static sequence foundation | `cargo test -p phalcom-semantic --test semantic static_sequence` | 10 passed | Sequence storage/laws, evaluator, descriptors, underconstraint, boundaries, and T11 sequence cases |
| `COLL006.C1.P4.G7` | Incremental sequence cases | `cargo test -p phalcom-semantic --test semantic incremental::coll006_c1_p3_publication` | 3 passed | T11 incremental matrix |
| `COLL006.C1.P4.G7` | Static-constant semantic suite | `cargo test -p phalcom-semantic --test semantic static_constants` | 21 passed / 3 inherited D4 failures | P4 static cases pass; aggregate remains `BASELINE_BLOCKED` by computed Record-key projection |
| `COLL006.C1.P4.G7` | Array runtime | `cargo test -p phalcom-core --test core core_array` | 11 passed | Fixed Array storage, mutation/resize boundary, order, GC, and bare runtime default |
| `COLL006.C1.P4.G7` | Vector runtime | `cargo test -p phalcom-core --test core core_vector` | 9 passed | Immutable Vector storage, exact spreads, source order, zero extent, GC, conversion, and fail-closed checks |
| `COLL006.C1.P4.G7` | Static constants/runtime owner | `cargo test -p phalcom-core --test core static_const` | 23 passed | Static materialization, frozen Array alias defense, owner cache, no initializer re-execution, and GC |
| `COLL006.C1.P4.G7` | Module import/re-export owner identity | `cargo test -p phalcom-core --test core modules_parity` | 8 passed | Selective, renamed, qualified, transitive re-export, independent two-consumer, and GC owner identity |
| `COLL006.C1.P4.G7` | Native surface | `cargo run -p phalcom-native-surface-gen -- --root . --check` | 417 declarations current | Authored native surface remains synchronized |
| `COLL006.C1.P4.G7` | Workspace all-target check | `cargo check --workspace --all-targets` | Passed | Cross-crate compilation after T11 changes |
| `COLL006.C1.P4.G7` | Diff hygiene | `git diff --check` | Passed | No whitespace errors in the scoped worktree diff |

`cargo fmt --all -- --check` was also run. It reports formatting drift in
pre-existing P3/C1 files; no broad formatting rewrite was applied. The T11
Rust files pass a scoped `rustfmt --check --edition 2024` check. The global
formatting gate cannot be reported as passing.

### P4 T0 entry gate — 2026-10-09

- Starting branch revision: `bee1b2fbffa901cf006e2b4f5948ca478436ee10`, the merge of `origin/main` into `feat/telos-collection-upgrade`; merge base is current `origin/main` `c2dd635ee34f2cef3e64e86c87a5e77992f29fa7`.
- Post-convergence focused P3 gates passed against the live tree: semantic `coll006` **46 passed**, core `core_vector` **8 passed**, core `core_array` **10 passed**, and native-surface `--check` reported **417 declarations current**.
- The first semantic build exposed one missing sequence-family statement terminator and three checked-binding call sites missing the newly required phase argument; those local compile errors were fixed before the passing rerun. No broad workspace or release certification was run here.
- P3 was accepted for the bounded scope stated above. At this historical T0 gate P4 started `IN_PROGRESS`; T11/G7/T12 later completed it as `COMPLETE / IMPLEMENTED / BASELINE_BLOCKED` with D4 retained at G7.

---

## 14. Deferred / baseline issue ledger

### COLL006.C1.D1 — Broader metadata failure during exact-integer prerequisite

- Observed: `structural_member_presence_and_remainder_affect_fingerprints` returned `NonExportableRecordKeyDomain`; the metadata filter reported 32 passed and 1 failed.
- Classification: D suspected unrelated by scoped inspection; no clean-predecessor reproduction was performed.
- Active acceptance: the focused exact-integer tests passed; this remains a residual broad-suite failure rather than certification evidence.
- Reproduction: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::metadata`.
- Revisit: owning metadata work or P3/P9 broader certification if required.

### COLL006.C1.D2 — Broader reflection failures during exact-integer prerequisite

- Observed: reflection filter reported 25 passed and 4 failed: Universe math package exposure drift and module/package/project fixture `NotCallable` diagnostics.
- Classification: D suspected unrelated by scoped inspection; no clean-predecessor reproduction was performed.
- Active acceptance: the exact-integer reflection regression passed. These broad failures do not establish a Vector defect or a passing broad gate.
- Reproduction: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core reflection`.
- Revisit: owning reflection/module work or broader certification.

### COLL006.C1.D3 — Deferred broad certification and P4-owned work

- Vector and Array runtime suites pass after the final publication/formation changes: 8 and 10 tests respectively.
- Concrete negative Vector extents are rejected during canonical annotation formation, including concrete and partially inferred element arguments. Static value evaluation/materialization remains P4-owned.
- Bounded symbolic identity construction is implemented and runtime-tested. General symbolic arithmetic and extra zero-contribution operands are outside the adopted lane.
- Inference-owned publication is complete for the bounded P3 surface: late generic failure, deferred scalar/row readiness, exact-case conformance, union aggregation, independent survivors, and invalid/valid/invalid cold parity are covered.
- Array homogeneous public read/replacement contracts are typed in the Universe source; the pre-convergence P3 native-surface check passed at 406 declarations, and P4 T0 confirmed the merged tree at 417 declarations.
- Broader metadata/reflection suites and release certification remain deferred under their existing D1/D2/P9 policy. P4 does not absorb those unrelated broad failures.

### COLL006.C1.D4 — Computed Record-key failures in the static-constant filter

- Observed before P4: `foundations::static_constants` reported 15 passed and 3 failed, with `RecordKeyNotStatic` in computed Record-key projection cases.
- Rechecked during T3/G2: the same filter selected 23 tests, with 20 passed and the same 3 failures: `test_field_signature_key_edit_matches_cold_surface_and_invalidates_exactly`, `test_callable_signature_projects_all_parameter_and_return_record_keys`, and `test_field_signature_projects_a_cold_computed_record_key`.
- Classification: inherited D failure. The diagnostics and three failing cases match the pre-P4 result; these fixtures exercise computed Record-key projection, not the new empty-sequence diagnostic path. No clean-predecessor comparison was run in this continuation.
- P4 evidence unaffected: the six `static_sequence` tests, the exact empty-sequence regression, symbolic identity-spread test, and 46-test `coll006` filter all passed. G2 is `BASELINE_BLOCKED`, not `PASS`; the plan's §0.4 amendment permits T4 to proceed with these matched D failures recorded.

#### G7 recheck of D4

- Observed at G7: `foundations::static_constants` selected 24 tests, with 21 passing and the same three D4 `RecordKeyNotStatic` failures: `test_callable_signature_projects_all_parameter_and_return_record_keys`, `test_field_signature_projects_a_cold_computed_record_key`, and `test_field_signature_key_edit_matches_cold_surface_and_invalidates_exactly`.
- Classification: inherited D4 baseline; the failures are unchanged and exercise computed Record-key projection rather than P4 sequence behavior.
- Active acceptance: all P4 sequence, cutover, incremental, Array, Vector, static-constant, module/re-export, GC, and boundary discriminators pass. P4 is implementation-complete, with verification `BASELINE_BLOCKED` because the named aggregate filter cannot be reported as a clean pass.
- Reproduction: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic static_constants`.
- Revisit: owning static Record-key projection work or COLL006.C1.P9 broader certification.

---

## 15. Consultation / amendment ledger

### COLL006.C1.A1 — Collection ontology and literal defaults

**Incident:** pre-implementation architecture design  
**Plan/task:** pre-plan checkpoint formation  
**Trigger:** existing List-default collection model does not express fixed runtime arrays, statically dimensioned immutable vectors, contextual collection literals, or const Array requirements coherently.  
**Decision:**

```text
runtime bare [...]        -> Array<T>
static bare [...]         -> Vector<T,N>
<T>[...]                  -> explicit Array<T>
<T,N>[...]                -> explicit Vector<T,N>
expected List<T>          -> contextual List
expected Array<T>         -> contextual Array
expected Vector<T,N>      -> contextual Vector
```

`Array<T>` has runtime extent; `Vector<T,N>` has static dimension.

**Architectural consequence:** bare bracket literals must become neutral/contextual syntax; semantic analysis becomes family-selection authority; legacy bare-List decisions require migration.  
**Plan amendment:** establishes the initial P1–P4 decomposition.  
**Additional tests:** full literal/context/phase matrix and ambiguity coverage.

### COLL006.C1.A2 — Constant Array remains supported

**Incident:** const/static collection ontology  
**Plan/task:** pre-plan checkpoint formation  
**Trigger:** static bare literals naturally map to Vector, but callers still require explicitly typed/selected constant Arrays.  
**Decision:** `const Array<T>` is legal; it remains classifier `Array<T>` and must not be mutable through direct or aliased runtime materialization.  
**Architectural consequence:** static sequence representation must preserve exact result type; runtime Array materialization needs a frozen/const-safe state or semantically equivalent enforcement.  
**Plan amendment:** owned by P4.  
**Additional tests:** direct mutation rejection, alias mutation rejection, cross-module materialization.

### COLL006.C1.A3 — Vector is dimensioned; parser snapshots are not Vector by default

**Incident:** P14 overlap  
**Plan/task:** pre-plan checkpoint formation  
**Trigger:** `Vector<T,N>` requires static N, while parser snapshots commonly have runtime-selected size.  
**Decision:** do not reuse `Vector<T,N>` as an arbitrary runtime-length immutable snapshot abstraction. Prefer private `Array<T>` snapshot storage plus `Slice<T>` read-only exposure.  
**Architectural consequence:** C1.P5 must coordinate with/amend the P14 direction rather than creating duplicate immutable sequence types.  
**Plan amendment:** P5 dependency/ownership requirement.  
**Additional tests:** snapshot independence, no mutable backing escape, Slice lifetime/GC.

### COLL006.C1.A4 — OrderedSet terminology

**Incident:** collection vocabulary  
**Plan/task:** pre-plan checkpoint formation  
**Trigger:** earlier `SetList` terminology.  
**Decision:** canonical public name is `OrderedSet<T>`.  
**Architectural consequence:** all C1 records/plans/tests use `OrderedSet`; `SetList` is not introduced as an alias.  
**Plan amendment:** P6 vocabulary requirement.  
**Additional tests:** none beyond OrderedSet invariant/equality coverage.

### COLL006.C1.A5 — Exact static integer representation

Decision inherited from the user-authorized P3 requirements analysis: canonical static integer identity preserves exact Telos Int values. Implemented scalar owner is `phalcom-type-meta::StaticInt`, re-exported by semantics. `TypeData::ConstInt`, type-member arguments, metadata schema 12, and fingerprints use the shared scalar. Type identity remains `TypeId`.

Evidence: exact-integer foundation, metadata/schema, and reflection tests recorded in the P3 walkthrough. Resource/count conversion remains separate from semantic identity.

### COLL006.C1.A6 — Heterogeneous static generic formation

Decision inherited from the analysis: complete and partial applications elaborate values in the destination parameter domain, and inference holes use that domain. Intrinsic Int bootstrap and `KindSpec::StaticDomain` project source-owned declarations. Tests prove Int terms, symbolic binders, partial holes, and large signed integers. The current value elaborator supports integer literals and unary-negative integer literals; general checked static expression evaluation remains a follow-up.

### COLL006.C1.A8 — Exact spread and lowering product extension

Decision inherited from the analysis: checked sequence products retain canonical extent and exact tuple-positional/Vector-storage lanes. Runtime-sized iterable spreads cannot establish Vector cardinality. Initial projection and bytecodes are implemented with concrete checked `u32` counts and runtime cardinality assertions. The initial concrete-u32 limitation was subsequently extended by the adopted bounded symbolic identity-spread proof; earlier continuation runtime evidence is recorded below. Broader symbolic construction is not established.

These entries record adopted requirements and partial implementation, not a claim that the P3 acceptance objective is complete.

### COLL006.C1.A9 — Empty static sequence keeps element underconstraint across cutover

**Consultation:** P4 T3, 2026-10-09. Before T10, a bare static `[]` retains the
legacy List family; after T10 it selects `Vector<T,0>`. In both phases, an
empty literal with no element-type evidence emits the existing
`GenericInferenceUnderconstrained` diagnostic at the enclosing static
initializer boundary and publishes no value. A typed empty Vector remains
valid. Family selection does not move into `sequence_literal::synthesize`, and
the diagnostic is delayed until normal contextual/inference solving completes.
No new diagnostic code or ADR is needed. See the P4 plan's §0.3 amendment and
the T3 consultation record below.

### COLL006.C1.A10 — G2 inherited-failure handoff to T4

**Consultation:** P4 T3/G2, 2026-10-09. If all P4-active G2 discriminators pass
and all remaining failures are positively matched inherited D failures, record
G2 as `BASELINE_BLOCKED` and permit T4 to proceed. Do not report the full filter
as passed. For this handoff, the three unchanged computed-Record-key failures
in `foundations::static_constants` are tracked as D4. See the P4 plan's §0.4
amendment.

### COLL006.C1.A11 — Residual ownership and P9 classification

**Consultation:** user-provided residual scope/ownership ruling, 2026-10-10.
**Decision:** retain P4 as `COMPLETE / IMPLEMENTED / BASELINE_BLOCKED`; D4 blocks
only a clean aggregate `static_constants` result and does not block P5. P5 may
proceed with no execution blockers. D1 and D2 remain suspected unrelated
verification residuals because no clean-predecessor comparison has established
their provenance. Keep the P1–P9 ledger unchanged; do not add a P4 corrective
plan without evidence of a C1-owned defect.

At P9, rerun each surviving residual and compare its exact failing fixture(s)
against contemporaneous `origin/main` first. If main reproduces the same
failure, classify it as `EXTERNAL_BASELINE`; only investigate a historical C1
boundary when main passes and the C1 candidate fails. Split D2 by the actual
failure causes. Classify each residual as `RESOLVED`, `EXTERNAL_BASELINE`,
`C1_OWNED`, or `AMBIGUOUS`. C1 closure requires no `AMBIGUOUS` residuals. A
confirmed external failure may leave C1 `COMPLETE / IMPLEMENTED /
BASELINE_BLOCKED`; a C1-owned defect returns to its owning plan or a corrective
C1 plan.

P9 owns broad workspace/CI certification and the C1 behavior, property, GC,
re-entry, cold/incremental, performance, migration, documentation, and
lifecycle gates. P4's bounded acceptance is not retroactively expanded.

---

## 16. Fixed / flexible / verify-first classification

### FIXED

Implementers must not change without consultation:

- sequence-family semantics and literal defaults in §3;
- semantic analysis as collection-family authority;
- `_` not being `Dynamic`;
- Array runtime extent versus Vector static extent;
- Vector immutability;
- explicit const Array support;
- List remaining the resizable sequence;
- OrderedSet naming;
- no phase-default overload tie-breaker;
- no duplicate Iterator/Iterable/Collection protocol hierarchy;
- persistent old-root immutability;
- C4 static-evaluation/materialization authority.

### MECHANICALLY FLEXIBLE

Implementers may adapt without consultation:

- private Rust helper names;
- exact module/file decomposition;
- exact bytecode encoding if semantics and direct construction are preserved;
- whether fixed dense storage uses `Box<[Value]>`, an equivalent wrapper, or a shareable internal layer;
- private ordered-hash helper layout;
- private persistent-node branching factor/balancing implementation;
- local iterator/helper decomposition;
- equivalent current-repository APIs discovered during plan execution.

### VERIFY-FIRST

Each relevant plan must verify before editing:

- current AST/parser symbols;
- current inference APIs and kind/domain representation for static `Int`;
- current C4 static-value/materialization seams;
- current native primitive-floor governance;
- current Map/Set re-entry/locking behavior;
- current P14 status and parser snapshot consumers;
- current accepted decision/spec filenames and supersession mechanism;
- current collection corpus and migration surface;
- current repository revision/dirty state.

If verification changes only mechanics, adapt locally. If it invalidates ownership or semantics, STOP AND CONSULT.

---

## 17. Global STOP AND CONSULT triggers

Every C1 plan inherits the following triggers.

STOP AND CONSULT if:

1. partial annotations can only be implemented by interning unsolved variables into canonical `TypeStore`;
2. `_` would need to mean `Dynamic` or an existential wildcard to proceed;
3. parser or compiler must independently reconstruct collection-family semantics;
4. expected-type propagation cannot distinguish unique family context from overload ambiguity;
5. `Vector<T,N>` cannot represent `N` as an ordinary canonical static `Int` argument without special parallel type identity;
6. exact Vector cardinality requires an Array/Vector-specific second inference solver rather than reusable Pack/static reasoning;
7. const Array can be made safe only by changing its public classifier or silently copying at arbitrary alias boundaries;
8. C1 static collection work would duplicate or contradict C4 static identity/materialization ownership;
9. factoring MapObject changes observable Map/Set semantics;
10. a hash/persistent implementation would retain Heap borrows across re-entrant Telos callbacks;
11. persistent node storage cannot be traced correctly by the current GC without changing GC ownership architecture;
12. RangeMap requires changing current numeric Range semantics instead of using a separate interval value;
13. a planned performance contract cannot be met without materially changing public semantics;
14. tests can pass only by weakening one of the checkpoint invariants;
15. implementation must enter an unexpected subsystem with architectural consequences;
16. any plan materially expands beyond the C1 acceptance objective.

The implementer cannot self-waive a triggered consultation.

---

## 18. Active plan and next action

```text
active plan:           COLL006.C1.P5 (proposed)
latest completed plan: COLL006.C1.P4
execution blockers:    none for P5
verification residuals: D4 — computed Record-key failures block a clean aggregate static_constants PASS;
                         D1 — metadata failure, still suspected unrelated;
                         D2 — reflection/module fixture failures, still suspected unrelated
next action:           begin P5 Slice/Buffer implementation from the completed P4 handoff
```

Continue in the `telos-collections-upgrade` worktree from the completed P4 state.
T10/G6 and T11/G7 are complete; live defaults are runtime Array and static
Vector. Read the P4 walkthrough and handoff for the stable P5 takeover map.


### P3 continuation consultation — symbolic identity construction (2026-10-08)

Trigger: semantic analysis accepts canonical symbolic extent identity, while compiler projection requires concrete u32 counts (cross-layer contract clarification).
Adviser decision: adopt a semantic `VectorIdentitySpread` construction proof only for a literal containing exactly one native Vector-storage spread with the same canonical result extent. Compiler consumes that proof and emits a dedicated immutable-storage copy operation. No runtime interpretation of N, second solver, optional-count fallback, or symbolic arithmetic is authorized. All concrete checks remain intact. Zero-position additional operands remain outside this bounded path so effects cannot be dropped.
Plan amendment: T4/T5 include this bounded identity proof and execution lane. Focused evidence must cover generic execution, distinct storage, binder mismatch, runtime-sized rejection, and unchanged concrete checks.

Layout consultation: predecessor 5fee70c9 already contains inline `FamilyObject` with a 48-byte payload on the 64-bit host; Vector's boxed slice payload is 16 bytes. The newly added absolute `Object <= 40` test oracle is incompatible with the predecessor. Preserve representation and verify no Vector-driven enum growth using the existing largest payload rather than an unsupported absolute limit.


### P3 continuation publication diagnosis

Ordinary element mismatches previously passed because synthesis propagated expectations without checking them. Elements now use the existing checked-expression path. Explicit heads also no longer erase canonical surrounding contracts or fixed slots in partial surrounding contracts; these relations must be proved before sequence publication.

The adviser identified a remaining general lifecycle gap: a candidate depending on a subsequently failed/underconstrained/cancelled enclosing inference context can outlive that context. Full closure requires transactional ownership of dependent construction products at inference-context finalization. Do not prematurely solve an outer frame, add a second solver, or add statement-only rollback. Fixed-slot regressions are covered in this continuation; general context-owned cleanup remains an explicit P3 acceptance gap.

Implementation follow-up (2026-10-08): the [active P3 T4 publication supplement](COLL006.C1.P3-inference-publication-plan.md) is now authorized and `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`. Its discriminator reproduced the late generic failure; checker-local staging, receipts, provenance, deferred scalar/row readiness and extraction integration are present. Closure remains open; the current evidence and residual work are recorded below and in the [detailed continuation report](COLL006.C1.P3-inference-publication-report.md).


### P3 earlier continuation verification ledger (2026-10-08)

These completed results supersede the earlier aborted-runtime snapshot and predate the inference-publication supplement. They do not certify the newest checker/session edits. P3 remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`.

| Gate | Command/scope | Completed result | Evidence limit |
|---|---|---|---|
| G1 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic coll006` | 27 passed | Five new test functions include table-driven negative/nonpublication cases, family explanations, and symbolic identity proof/rejections; general inference-context transaction cleanup remains open |
| G2 Vector | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_vector` | 8 passed | Final combined run includes direct bytecode, concrete/symbolic construction, API/hash/conversions, immutable/nested GC/iterator ownership, Map/Set keys, defensive cardinality failure/nonpublication |
| G2 Array | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_array` | 10 passed | Shared checker/builder regression gate; later contextual validation additionally covered by the final semantic collection gate |
| Native artifact | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-native-surface-gen -- --root . --check` | Passed, 406 records current | No authored native signatures changed in this continuation |
| Integration build | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace --all-targets` | Passed | Compile evidence, not workspace behavioral certification |
| Formatting/diff | Scoped rustfmt check on eight formatted Rust files; `git diff --check` | Passed | Two unrelated inherited formatting sections in `checker/expression.rs` intentionally preserved |

Initial runtime failures were diagnosed and closed: unsupported pre-existing-layout oracle (adviser clarification), fixture recompilation after binding installation, missing generic constructor evidence, and an incorrect Map mutation selector. No production representation or accepted semantic oracle was weakened. Existing broader metadata/reflection baseline records remain inherited and un-recertified.

Deferred deliberately: full workspace tests/clippy/release certification (P9), broader metadata/reflection reruns without new coverage, and the remaining formation/Array API/inference-publication/incremental acceptance work listed in the current handoff. No commit or push performed in this continuation.


### P3 publication consultation — static annotation owner (2026-10-08)

Plan/task: publication supplement T5. Trigger: unexpected session caller inference identity/lifetime boundary. Observed: partial static annotations were elaborated in a temporary checker whose graph was dropped before a fresh checker analyzed the initializer; confirmed unchanged from e3dfe492. Decision: keep annotation elaboration and initializer checking in the same configured checker; missing supplied owners fail closed. Architecture impact: no graph transfer, new solver, durable identity, or P4 evaluator work. Plan amendment: Level 1 T5 caller clarification recorded in the supplement. Implemented continuation: same-checker elaboration is exercised by valid partial Vector/List, incompatible fixed extent, empty underconstraint, and annotation-diagnostic retention regressions; static acceptance errors and invalid-owner products are rejected before extraction. Missing-owner handling is fail-closed at expression/static entry. Focused evidence passes; direct injected missing-owner API coverage remains open.


### P3 inference-owned publication — completed state (2026-10-08)

State: `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`. Current inspected branch is `feat/telos-collection-upgrade`; source changes remain uncommitted in the requested worktree. No push or clean-worktree claim is made.

### P4 takeover and implementation state (2026-10-09)

P4 was implemented in `telos-collections-upgrade`. The feature branch was merged with
`origin/main` at `bee1b2fb`; source overlaps were additive or formatting-only.
P1/P2 lifecycle conflicts were resolved to retain the feature branch's completed
records. The generated native-surface count combines the 6 feature-branch and
11 main-branch additions over the shared 400-declaration base, for 417. The
pre-existing P3 worktree edits do not overlap the merged paths and remain
uncommitted.

P3 remains `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` based on its recorded
pre-convergence evidence; P4 T0 separately reran the focused P3 semantic,
Vector, Array, and native-surface gates after convergence and all passed. P4 is
now `COMPLETE / IMPLEMENTED / BASELINE_BLOCKED`; T11 and G7 are complete, with
the three inherited D4 failures retained. P5 is the next proposed plan.

P4 has completed the T1/T2 canonical static Sequence storage and laws, T3
checked bracket evaluation, T4 recursive descriptors, T5 native Vector/frozen
Array materialization, T6 same-module and linked canonical-owner reads, and T9
List migration/normative updates.
Temporary materialization Values remain rooted until the canonical owner cache
takes ownership. T1/T2 focused laws passed. T3's focused evaluator and
symbolic-spread cases passed, while CV-35 exposed a missing underconstraint
diagnostic for bare `const empty = []`. Its architecture ruling, implementation,
and focused test evidence are recorded below. T4–T7 results and the passing G3
and G4 gates are recorded after this summary. T8 census and classification are
complete; T9 migration is complete and G5 passed. T10/G6, T11/G7, and T12 are
complete; P4 is `COMPLETE / IMPLEMENTED / BASELINE_BLOCKED`.
The user selected
the closed-sequence boundary for symbolic `VectorIdentitySpread`:
static evaluation returns `Symbolic` for an open exact-static-Int extent before
evaluating the spread and publishes no sequence root. Evaluator-side extent
specialization and symbolic arithmetic remain unauthorized. Descriptor family
is retained as immutable checked `StaticSequenceFamily` metadata derived during
canonical interning; this avoids expanding the descriptor API with a TypeStore
argument.

T8's original census was refreshed during T9 after source migration. The final
T12 delta inventory contains 505 classified bare nodes with no ambiguous or
unclassified sites: `LIST_RESIZING` 8, `LIST_IDENTITY` 196,
`FIXED_RUNTIME` 36, `STATIC_IMMUTABLE` 12, `READ_ITERATE_ONLY` 243, and
`LEGACY_TEST` 10. See the P4 migration census for per-site rationale, owner
impact, parser diagnostics, discovery method, and the T12 addition inventory.

Existing C1 residual metadata/reflection evidence and release-wide
certification remain deferred under the baseline policy.

Implemented private takeover interface: `checker/sequence_publication.rs` owns staged canonical resolutions and conjunctive context/frame receipts; `CheckingContext` owns application readiness records and terminal decisions. The registry is outside shared solver handles. Probes copy owner metadata by value and start with fresh pending products. Existing committed map/schema and compiler/runtime products are unchanged by this supplement. Actual contextual provenance survives Proper expectations and propagates through required operands; independent declarations retain independence. Deferred children reuse existing frame/residual/row/materialization/conformance rules before any terminal receipt is issued.

| Evidence | Completed result | Scope and limit |
|---|---|---|
| `cargo test -p phalcom-semantic --lib sequence_publication` | 3 passed | Registry conjunction, ancestor rejection/independent survival, terminal/missing-owner behavior, probe isolation/disposal |
| `cargo test -p phalcom-semantic --test semantic coll006_c1_p3_publication` | 14 passed | Generic/Proper expectations, nested/deferred children, independent closure declaration, row cases, valid/invalid partial List/Vector static ownership, empty underconstraint and annotation diagnostics |
| `cargo test -p phalcom-semantic --test semantic foundations::record_row_inference` | 6 passed | Directly affected row inference regression suite |
| `cargo check -p phalcom-semantic -p phalcom-core --all-targets` | Passed | Current semantic/core source compiles with all targets |
| `cargo test -p phalcom-core --test core core_vector` | 8 passed | Vector consumer/runtime regression suite |
| `cargo test -p phalcom-core --test core core_array` | 10 passed | Array consumer/runtime regression suite |
| `cargo test -p phalcom-semantic --test semantic foundations::generic_inference_proof_integrity` | 7 passed, 1 failed | Existing argument-range oracle remains open; no assertion weakened |
| `cargo test -p phalcom-semantic --test semantic foundations::static_constants` | 15 passed, 3 failed | Computed-record-key diagnostics remain outside this publication slice; no repair attempted |
| `cargo fmt --all -- --check` | Incomplete | Reports inherited formatting drift across the dirty worktree; no broad formatting applied |

Commands used `RUSTFLAGS='' RUSTC_WRAPPER=''`. The List<_> static pass does not establish sound partial Vector owner handling: it can avoid the exact-extent operation that exposed the dangling session.

Adopted deferred-proof correction: preserve exact argument bindings used for conformance, and apply their refinement only to conformance discharge. Scalar equality/subtype authority remains in the existing registered residual constraints. The helper compiled in the 10-test run; case-only conformance regression evidence is still required.

Final focused evidence: publication unit laws, exact-case/deferred controls, union shared-argument ownership, invalid/valid/invalid incremental parity with cold agreement and unrelated callable reuse, diagnostic-range/origin regressions, negative concrete extent formation, Vector runtime (8), Array runtime (10), COLL006 semantic (46), workspace all-targets check, and native surface check (406). No broad release certification or commit/push is claimed.


### P4 T3 consultation — empty static bracket element evidence (2026-10-09)

Trigger: P4 T3/CV-35 and requirements §13.3 required an underconstraint
diagnostic for a bare empty static literal, while accepted collection decisions
only defined the empty List surface/value and live `sequence_literal::synthesize`
returned `Unknown(NoTypeEvidence)` without a diagnostic. The implementation
must preserve legacy family selection until the authorized T10 cutover.

Adviser ruling: before T10, `const empty = []` remains the legacy List family,
but missing element type evidence must emit existing
`GenericInferenceUnderconstrained` and publish no value. After T10, the same
source selects `Vector<T,0>` and must retain the same unresolved-T diagnostic
and no-value behavior. A typed `const empty: Vector<Int,0> = []` remains valid.
Diagnose only after the enclosing static initializer's normal contextual and
inference opportunity; do not add a diagnostic in `sequence_literal::synthesize`
or introduce early phase defaults. No new diagnostic code or ADR is needed.

Plan amendment: T3/CV-35 now records both pre-cutover and post-T10 outcomes; G6
owns the post-cutover case. Implementation adds the diagnostic at the static
initializer boundary for a bare empty bracket that remains
`Unknown(NoTypeEvidence)` with no expected type. The focused regression is
present and passes. Results: `static_sequence` **6 passed**; the exact empty
diagnostic test **1 passed**; symbolic identity spread **1 passed**; `coll006`
**46 passed**. `foundations::static_constants` selected 23 tests: **20 passed,
3 inherited D failures** recorded as D4. G2 is `BASELINE_BLOCKED`, not passed;
T4 may proceed under the §0.4 plan amendment because the P4-active cases pass
and the remaining failures match the pre-P4 baseline. P4 remains
`IN_PROGRESS / PARTIAL / UNVERIFIED`.


### P4 T4 descriptor slice — 2026-10-09

Implemented CV-11/CV-12 descriptor coverage for canonical Array/Vector family
classification, recursively materializable nested sequences, outer deferral
when a sequence contains an unsupported Tuple, accurate `Sequence` versus
`DeferredStructural` capabilities, and cloned descriptors that remain
self-contained after dropping the static-value store. `MaterializationDescriptor`
retains no static-value reference or TypeStore pointer.

Verification: the exact descriptor test
`static_sequence_materialization_is_recursive_and_truthfully_deferred` passed
(1 selected), followed by the owning `static_values_sequence` filter (3 passed).
T4 is `IMPLEMENTED / FOCUSED_TESTED`. Runtime object materialization is T5 and
remains pending; descriptors alone do not prove runtime reads, GC rooting, or
owner-cache behavior. G2 remains `BASELINE_BLOCKED` under D4, with the T4
handoff allowed by the explicit plan amendment.


### P4 T5/T6 runtime sequence slice and G3 — 2026-10-09

T5's recursive materializer constructs native Vector storage and frozen Array
storage directly from checked descriptors. Recursive heap children stay in
temporary roots until the owner module's static-constant cache has published the
outer value. A failed descriptor does not leave a partial owner value and
restores temporary-root depth. Direct VM coverage also checks nested values
survive collection and repeated ensure returns the canonical owner object.

T6 confirms same-module reads, selected and renamed imports, qualified module
reads, and two-hop re-exports all resolve to the same source-owned sequence
objects. Runtime initializer bytecode does not contain Array/Vector literal
construction for the constants. The source module owns each materialized value;
aliases do not create global slots or copies. Array constants retain the Array
runtime classifier and frozen state.

Focused evidence:

| Gate | Command/filter | Result |
|---|---|---|
| T5 direct materializer | `cargo test -p phalcom-core --lib modules::static_constant` | 3 passed |
| T6 sequence runtime reads | `cargo test -p phalcom-core --test core static_const_sequence` | 2 passed |
| T6 static-constant regression filter | `cargo test -p phalcom-core --test core static_const` | 23 passed |
| G3 static sequence semantics | `cargo test -p phalcom-semantic --test semantic static_sequence` | 7 passed |
| G3 Vector runtime | `cargo test -p phalcom-core --test core core_vector` | 8 passed |
| G3 Array runtime | `cargo test -p phalcom-core --test core core_array` | 10 passed |

G3 is `PASS`. T5 and T6 are `IMPLEMENTED / FOCUSED_TESTED`. G2 remains
`BASELINE_BLOCKED` under D4; G3 does not broaden or repair the inherited
computed-Record-key failures. No workspace/release certification was run.


### P4 T7 const-Array mutation safety and G4 — 2026-10-09

The existing checker-local binding-phase index supplies stable `BindingId`
provenance for module constants and imported constants. T7 uses that provenance
only when the direct receiver has canonical Array type. Direct subscript writes
and the canonical `Array.at(_, put:)` replacement method emit
`StaticObjectMutation`; ordinary `let` and `let mut` Array bindings remain
mutable. Aliases remain a runtime concern and are rejected by the frozen Array
guard. Static Vector has no setter or mutation API; dynamic selector misses
continue through the language's runtime missing-message behavior.

Focused evidence:

| Gate | Command/filter | Result |
|---|---|---|
| T7 semantic provenance | `cargo test -p phalcom-semantic --test semantic foundations::static_const_mutation` | 3 passed |
| G4 COLL006 semantic | `cargo test -p phalcom-semantic --test semantic coll006` | 46 passed |
| G4 static-constant runtime | `cargo test -p phalcom-core --test core static_const` | 23 passed |
| G4 Array runtime | `cargo test -p phalcom-core --test core core_array` | 10 passed |

The T7 cases prove direct and imported const mutation rejection, named and
subscript replacement coverage, ordinary Array mutation, runtime alias
freezing, Vector's absent mutator, Array's runtime class, and rejection as a
stable Map key. G4 is `PASS`; T7 is `IMPLEMENTED / FOCUSED_TESTED`. No semantic
binding redesign was needed.


### P4 T9 List migration and G5 — 2026-10-09

The G5 AST-aware census had 495 sites: 331 in `.ph` files and 164 in Rust
source/test strings. All sites were classified; no `AMBIGUOUS` or
unclassified entries remained. G5 totals were `LIST_RESIZING` 7,
`LIST_IDENTITY` 196, `FIXED_RUNTIME` 35, `STATIC_IMMUTABLE` 4,
`READ_ITERATE_ONLY` 243, and `LEGACY_TEST` 10. Every List-required
producer now has explicit List intent in its annotation, parameter/result
contract, or typed intermediate. See the P4 migration census for the complete
site list and evidence.

The accepted TDR-0091 and current sequence-literal specification define
explicit-head selection, unique contextual-family selection, then runtime
Array/static Vector defaults. TDR-0025/TDR-0028 and static-evaluation/bytecode
wording now agree. The closed-sequence T3 decision remains in force: an open
symbolic Vector identity spread is not reduced or published.

G3/G4 focused reruns after source migration remain passing:

| Gate | Command/filter | Result |
|---|---|---|
| Semantic literals | `cargo test -p phalcom-semantic --test semantic coll006` | 46 passed |
| Static sequence | `cargo test -p phalcom-semantic --test semantic static_sequence` | 7 passed |
| Static constants | `cargo test -p phalcom-core --test core static_const` | 23 passed |
| Array runtime | `cargo test -p phalcom-core --test core core_array` | 10 passed |
| Vector runtime | `cargo test -p phalcom-core --test core core_vector` | 8 passed |

Exploratory language-corpus filters were mixed: explicit List migration cases
exposed existing iterator/protocol and unrelated path/Bytes/negative-oracle
failures, while `option`, `system`, `values`, positive indexing, and reflection
filters passed. T10 later applied the runtime Array/static Vector defaults. The
iteration lowering failures also reproduce in untouched iteration fixtures.
These broad corpus outcomes are not reported as passes and do not alter G5's
listed acceptance criteria; they remain recorded for later classification if
the T10 focused gates implicate them.

G5 is `PASS`: census is fully classified, List-dependent sites are explicit,
normative updates are accepted and indexed, and G3/G4 remain valid after the
migration. G2/G7 remain `BASELINE_BLOCKED` under D4's three
computed-Record-key `RecordKeyNotStatic` failures; the amendment does not turn
that filter into a pass.


### P4 T10 atomic default cutover and G6 — 2026-10-09

T10 installs phase defaults only after explicit-head and unique-context
selection. `sequence_literal.rs` maps runtime evaluation to Array and static
evaluation to Vector; `check_statement` sets static phase while checking each
`const` initializer and restores the caller phase afterward. The latter closes
a phase-owner gap: without it the static-constant product selected Vector while
the ordinary module binding fact still exposed Array. The new regression checks
both module binding facts and confirms the static product agrees.

The context-free compiler path fails closed for headless and explicit Vector
literals without semantic lowering. Syntax-explicit `<T>[...]` Array remains
available through the compatibility path. The parameter-type overload matrix
was not added: current object/trait specs prohibit parameter/result-type
selector overloads, so CV-25 uses the valid ambiguous trait-dispatch case; the
runtime Array default does not select a candidate.

| G6 evidence | Completed result | Scope and limit |
|---|---|---|
| Semantic cutover | `cargo test -p phalcom-semantic --test semantic coll006` — **49 passed** | Includes runtime Array default, top-level const Vector versus runtime let Array, ambiguity, and existing explicit/contextual controls |
| Array runtime/compiler | `cargo test -p phalcom-core --test core core_array` — **11 passed** | Includes default Array source-order effects and explicit Array behavior |
| Vector runtime/compiler | `cargo test -p phalcom-core --test core core_vector` — **8 passed** | Includes headless/explicit-Vector fail-closed checks and Vector construction/runtime |
| Static constant runtime/module | `cargo test -p phalcom-core --test core static_const` — **23 passed** | Includes canonical owner, import/re-export/GC, frozen Array alias, and Vector mutation boundary |
| Semantic static constants | `cargo test -p phalcom-semantic --test semantic static_constants` — **21 passed, 3 failed** | The three D4 computed-Record-key `RecordKeyNotStatic` failures remain baseline; all static-sequence cases in this filter passed |

G6 is `PASS`; T10 is `IMPLEMENTED / FOCUSED_TESTED`. G2 remains
`BASELINE_BLOCKED` by D4. T11/G7/T12 subsequently completed; see the final G7
ledger above and the P4 walkthrough. P4 is `COMPLETE / IMPLEMENTED /
BASELINE_BLOCKED`; P5 is the next proposed plan. Workspace all-target check and
native-surface verification passed at G7; workspace tests, clippy, and release
certification remain outside P4.


### P4 bounded correctness follow-up and closure — 2026-10-10

P4 remains `COMPLETE / IMPLEMENTED / BASELINE_BLOCKED`; this is a correctness
follow-up to its existing acceptance objective, not a reopened feature plan.
Work was performed on `feat/telos-collection-upgrade` from starting revision
`30746708` in the `telos-collections-upgrade` worktree. No semantic authority,
ownership, materialization, or GC architecture changed, so no architecture
consultation or implementation-plan amendment was needed. The separate
residual-ownership ruling is recorded as A11 above.

The follow-up closed four bounded concerns from the P4 analysis:

- **Spread inference:** constraint handling now distinguishes concrete relation
  checking, inference-constraint registration, exact mismatch, and a missing
  query-local owner. A missing owner records an analysis incident and fails
  closed without publishing construction facts. Concrete spread compatibility
  remains checked by the canonical subtype relation; registered inference is
  not treated as proof before owner solving.
- **Publication and incremental closure:** partial `Vector<_, _>` spread
  inference is covered alongside contradictory spreads and late generic-owner
  rejection. Multi-hop static spreads now verify content propagation, extent
  invalidation, independent-product reuse, and cold/incremental parity.
- **Materialization safety:** nested static Arrays are checked for recursive
  freezing through aliases and survival after forced GC. Failed sequence
  materialization restores temporary-root depth and publishes no owner-cache
  entry; a subsequent valid materialization succeeds. Descriptor traversal is
  exercised at 128 nested sequences.
- **Checked counts and caller boundary:** fixed construction counts now retain
  host `usize` lengths instead of narrowing to `u32`; the compiler keeps its
  checked `u16` fast path and builder fallback. The production
  `intern_sequence` caller remains the static evaluator, consuming the
  committed `SequenceLiteralResolution`; no second subtype checker was added.

Follow-up verification, run with `RUSTFLAGS='' RUSTC_WRAPPER=''` except where
GC stress is shown:

| Gate | Result |
|---|---|
| Semantic sequence-literal unit tests | 3 passed |
| Semantic `coll006` | 54 passed |
| Semantic `static_sequence` | 10 passed |
| Multi-hop incremental spread test | 1 passed |
| 128-level descriptor test | 1 passed |
| Core `core_vector` / `core_array` | 9 / 11 passed |
| Core `static_const` / `modules_parity` | 24 / 8 passed |
| Failed-materialization cleanup and retry unit test | 1 passed |
| GC-stress canonical import/re-export owner test | 1 passed |
| GC-stress nested frozen-Array test | 1 passed |
| Native surface `--check` | 417 declarations current |
| `cargo check --workspace --all-targets` | Passed |
| `git diff --check` | Passed |
| Semantic `static_constants` | 21 passed; the same 3 D4 `RecordKeyNotStatic` failures |

The repeated D4 failures are `test_callable_signature_projects_all_parameter_and_return_record_keys`,
`test_field_signature_projects_a_cold_computed_record_key`, and
`test_field_signature_key_edit_matches_cold_surface_and_invalidates_exactly`.
They remain a verification residual that prevents a clean aggregate
`static_constants` PASS; they were not changed. Workspace tests, clippy, and
release certification remain outside this bounded follow-up. P5 remains the
next plan and has no execution blocker from D4.

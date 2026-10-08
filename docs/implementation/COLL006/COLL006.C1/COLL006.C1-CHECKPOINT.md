---
id: COLL006.C1
category: COLL
program: COLL006
checkpoint: COLL006.C1
kind: checkpoint-record
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
requires:
  - LANG007.C4.P3
---

# COLL006.C1 — Extended collections foundation and first-generation collection family

COLL006.C1 is the bounded acceptance objective for Telos's first complete extended-collections generation.

It combines the language-level homogeneous-sequence literal upgrade with the runtime/static sequence families and the standard-library/storage foundations needed for the new associative, ordered, persistent, and interval collections.

This record is the **single durable shared state authority** for all numbered plans belonging to `COLL006.C1`.

Publishing this checkpoint record claims **no implementation, no completed plan, and no verification evidence**.

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

## 4. Current repository grounding

Baseline inspected while defining C1:

```text
branch: main
revision: 800474d8a9f2e1e12f422bf3132729c4e69012d4
```

The following are established current-repository facts and must be verified again by each plan when mechanically relevant:

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

## 5. Proposed numbered plans ledger

No numbered plan has yet been published. The following ledger defines the intended C1 decomposition and may be mechanically adjusted when each plan is authored, but changes to architectural ownership or acceptance scope require checkpoint amendment.

| Plan | Scope | Status | Completion | Verification | Intended outcome |
|---|---|---|---|---|---|
| `COLL006.C1.P1` | Neutral bracket-literal AST/semantic architecture; `<...>[...]`; `_` inference holes; inference-bearing partial binding annotations | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Establish semantic authority and solver foundation without changing bare-literal defaults |
| `COLL006.C1.P2` | `Array<T>` runtime/native representation, explicit/contextual Array literals, direct lowering, iteration, GC | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Complete runtime fixed-after-construction homogeneous sequence |
| `COLL006.C1.P3` | `Vector<T,N>` runtime/native representation, static extent inference/cardinality proof, constructors, equality/hash, direct lowering | PROPOSED | NOT_STARTED | UNVERIFIED | Complete immutable dimensioned sequence |
| `COLL006.C1.P4` | Static homogeneous sequence model; const Vector; const Array; materialization; repository-wide bracket-default cutover/migration | PROPOSED | NOT_STARTED | UNVERIFIED | Make runtime bare brackets Array and const bare brackets Vector atomically |
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

No COLL006-specific implementation interface exists yet.

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

No verification has been run for COLL006.C1 yet.

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

---

## 14. Deferred / baseline issue ledger

No C1-specific baseline failure is yet recorded.

Future plans must add entries here rather than silently repairing unrelated failures.

Required format:

```text
ID:
Observed:
Classification: unrelated | baseline | unclear-but-nonblocking
Why it does not block active acceptance:
Reproduction:
When to revisit:
```

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
active plan:           none
latest completed plan: COLL006.C1.P2
blockers:              none recorded
next action:           author and execute COLL006.C1.P3 (Vector<T, N>)
```

`COLL006.C1.P1` should be a patch-grade plan for:

1. neutralizing the existing `ListLiteral` AST/checker/compiler naming/ownership while preserving current behavior;
2. parsing `<T>[...]` / `<T,N>[...]`;
3. adding `_` inference-hole syntax and solver integration;
4. supporting inference-bearing collection annotations such as `Array<_>` / `Vector<Int,_>` in inference-producing binding contexts;
5. publishing checked sequence-literal resolution architecture;
6. leaving the repository-wide bare-bracket default cutover for P4.

At P1 start, the implementer must update this checkpoint ledger before substantial implementation. At P1 completion, the implementer must update this record and produce the required walkthrough and handoff.

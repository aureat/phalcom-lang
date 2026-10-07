---
id: COLL006.C1
category: COLL
program: COLL006
checkpoint: COLL006.C1
kind: master-implementation-record
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
state_authority: COLL006.C1-CHECKPOINT.md
requires:
  - LANG007.C4.P3
---

# COLL006.C1 — Master Implementation Record

> **Purpose:** combined implementation requirements analysis, architecture specification, execution master plan, verification design, and implementer guidance for `COLL006.C1`.
>
> **State authority:** this file is **not** the checkpoint state authority. `COLL006.C1-CHECKPOINT.md` remains the single durable source of truth for plan state, implemented invariants, verification evidence, amendments, deferred failures, and next action.
>
> **Implementation status:** no COLL006.C1 implementation is claimed by this record.
>
> **Repository baseline inspected during architectural preparation:** `main` at `800474d8a9f2e1e12f422bf3132729c4e69012d4`.
>
> **Execution doctrine:** think broadly; make the implementer act narrowly. This record pre-decides architecture, ownership, invariants, failure modes, test surfaces, and escalation boundaries so that numbered plans can remain bounded and executable.

---

# 0. Executive summary

`COLL006.C1` upgrades Telos collections from the current primarily `List`-centric bracket-literal model into a coherent family of collection abstractions whose semantics are distinguished by **extent**, **mutability**, **ordering**, **ownership**, **persistence**, and **addressing mode**.

The central sequence model is:

```text
List<T>
    homogeneous
    mutable elements
    dynamically resizable
    runtime extent
    explicitly requested after the literal cutover

Array<T>
    homogeneous
    mutable elements for ordinary runtime instances
    fixed extent after construction
    runtime extent
    contiguous owned storage
    valid at runtime
    may also be explicitly requested as a const/static value

Vector<T, N>
    homogeneous
    immutable
    exact static extent N
    contiguous owned value storage
    valid both at runtime and as a const/static value
```

Bracket literals become **contextual homogeneous-sequence literals** rather than syntax intrinsically denoting `List`.

The final target semantics are:

```telos
let a = [1, 2, 3]
// Array<Int>

const b = [1, 2, 3]
// Vector<Int, 3>

let c: List<_> = [1, 2, 3]
// List<Int>

let d: Array<_> = [1, 2, 3]
// Array<Int>

const e: Array<_> = [1, 2, 3]
// static Array<Int>

let f: Vector<_, _> = [1, 2, 3]
// Vector<Int, 3>

let g = <_>[1, 2, 3]
// Array<Int>

let h = <_, _>[1, 2, 3]
// Vector<Int, 3>
```

The program additionally adds:

```text
Slice<T>
Buffer<T>

Bag<T>
MutableBag<T>

IndexedDict<K, V>
OrderedSet<T>

Bijection<K, V>

Entry<K, V>
Bound<T>
Interval<T>
RangeMap<K, V>
```

and the reusable storage foundations needed to implement those abstractions correctly:

```text
dense fixed storage
shared ordered-hash storage
persistent hash storage
persistent ordered storage
static homogeneous sequence values
```

The implementation is intentionally decomposed into nine numbered plans:

```text
P1  contextual literal + inference foundation
P2  Array<T>
P3  Vector<T,N>
P4  static sequences + repository-wide literal semantic cutover
P5  Slice<T> + Buffer<T>
P6  ordered-hash family: MutableBag / IndexedDict / OrderedSet
P7  persistent hash family: Bag / Bijection
P8  Interval + persistent ordered storage + RangeMap
P9  convergence, migration closure, GC/property/performance certification
```

The checkpoint is complete only when all nine plan outcomes, or explicitly approved corrective replacements, satisfy the acceptance objective recorded in `COLL006.C1-CHECKPOINT.md`.

---

# 1. Acceptance objective

`COLL006.C1` succeeds when Telos has a coherent first-generation extended collection system with all of the following properties.

## 1.1 Language and inference

1. Bracket literal syntax is represented neutrally rather than as an intrinsically `List` AST form.
2. Semantic analysis, not the parser or compiler, selects the concrete collection family.
3. Runtime bare `[...]` defaults to `Array<T>`.
4. Static/`const` bare `[...]` defaults to `Vector<T, N>`.
5. `<T>[...]` explicitly selects `Array<T>`.
6. `<T, N>[...]` explicitly selects `Vector<T, N>`.
7. A unique contextual expected type may select `List<T>`, `Array<T>`, or `Vector<T,N>`.
8. Ambiguous contextual family selection produces a diagnostic rather than using phase defaulting as an overload tie-breaker.
9. `_` creates a real inference variable in the parameter's declared kind/domain and never degrades to `Dynamic`.
10. Partial inference-bearing annotations such as `Array<_>` and `Vector<Int,_>` are solved from the initializer before a canonical binding type is published.

## 1.2 Sequence runtime

11. `List<T>` remains dynamically resizable.
12. `Array<T>` has runtime extent and cannot structurally resize after construction.
13. Ordinary runtime Array permits indexed element replacement.
14. `Vector<T,N>` has exactly `N` elements and is immutable in all phases.
15. Array and Vector use direct collection-specific literal construction, not `List` allocation plus conversion.
16. Dense sequence storage is contiguous at the Telos `Value` slot level in v1.
17. Public semantics do not expose the current `Value` representation as a stable ABI, preserving future unboxed numeric specialization.

## 1.3 Static/const semantics

18. `const x = [...]` defaults to `Vector<T,N>`.
19. Explicit constant `Array<T>` is supported.
20. Static Array and Vector share reusable homogeneous static-value machinery without sharing nominal type identity.
21. Static initializers are not re-executed during runtime materialization.
22. Materialized constant Arrays cannot be mutated through direct or aliased runtime access.
23. Constant Array remains classifier `Array<T>`; no `ConstArray<T>` family is introduced.

## 1.4 Views and bounded storage

24. `Slice<T>` is a read-only zero-copy contiguous view.
25. Slice retains the backing owner rather than relying on raw interior pointers.
26. `Buffer<T>` maintains `0 <= size <= capacity`, stores only initialized values, and never implicitly exceeds logical capacity.

## 1.5 Associative and ordered collections

27. Existing Map/Set behavior remains correct after any storage refactor.
28. `MutableBag<T>` maintains positive multiplicities and total-count invariants.
29. `IndexedDict<K,V>` makes positional order semantic and order-sensitive.
30. `OrderedSet<T>` makes positional order semantic while preserving uniqueness.
31. `Bag<T>` is persistent and structurally shared.
32. `Bijection<K,V>` is persistent and maintains exact forward/reverse inverse invariants.
33. `RangeMap<K,V>` uses generic intervals rather than overloading the current numeric progression-oriented `Range`.

## 1.6 Runtime correctness

34. New native collection objects are precisely traced by GC.
35. No collection holds a live heap borrow across re-entrant Telos `hash`, `==`, or comparison callbacks.
36. Persistent old roots remain valid after new versions are created.
37. Failure paths do not leave partially updated Bijections, ordered indexes, Bag counts, or RangeMap interval invariants.

## 1.7 Migration and certification

38. Repository source depending on legacy bare-bracket-List semantics is migrated explicitly.
39. Legacy accepted decisions describing `[...]` as intrinsically List are amended/superseded before final cutover acceptance.
40. Required focused type-system, semantic, compiler, runtime, GC, property, and performance evidence is recorded in the checkpoint.

---

# 2. Current repository reality

The following are current-repository facts established during preparation. Numbered plans must verify mechanically relevant facts against their own starting revision.

## 2.1 Literal pipeline is currently List-specific

Current architecture is effectively:

```text
[...]
  ↓
Expr::ListLiteral / ListLiteralExpr
  ↓
synthesize_list_literal
  ↓
List<T>
  ↓
BuildList / BeginListLiteral / ...
  ↓
ListObject
```

The relevant current authorities include:

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-semantic/src/checker/expression.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/vm/dispatch.rs
```

This must be generalized without letting the compiler become a second semantic authority.

## 2.2 List runtime is already the correct growable substrate

`ListObject` is backed by `Vec<Value>` and already provides the appropriate semantics for a dynamically resizable homogeneous sequence.

The implementation should preserve List's storage role and change only literal/default selection and any constructor ergonomics required by the new model.

## 2.3 Map/Set already use ordered storage

Current Map runtime uses an insertion-ordered entry vector plus a hash index.

Current Set reuses Map backing.

Consequences:

- `IndexedDict` is justified only because positional order is **semantic**, directly addressable/mutable, and equality-sensitive.
- `OrderedSet` is justified only because positional order is **semantic**, directly mutable, index-searchable, and equality-sensitive.
- no new type should be introduced merely to obtain deterministic encounter order.

## 2.4 Current Range is not the right RangeMap key abstraction

Current Range is numeric/progression-oriented and assumes Number-like stepping semantics.

`RangeMap<K,V>` therefore needs a generic interval abstraction, not a forced generalization of existing numeric Range within this checkpoint.

## 2.5 Canonical iteration already exists

`Iterator` and `Iterable` already exist with associated types and a substantial set of default combinators.

No parallel collection iteration protocol should be introduced.

## 2.6 Generic inference infrastructure already exists

The semantic inference subsystem already has query-local inference variables and applied inference terms.

This should be reused for `_`.

No collection-specific solver should be introduced.

## 2.7 Static-value support is incomplete for homogeneous sequences

The static-value store supports several structural/value categories but does not yet have a reusable homogeneous sequence node.

C1 must extend the C4 static-value/materialization architecture rather than inventing a separate const collection mechanism.

## 2.8 Persistent map storage does not yet exist as a reusable substrate

Bag and Bijection require a new persistent hash foundation.

A clone-on-write ordinary Map may be useful only as a temporary correctness oracle, not the final performance architecture if persistence is advertised as a core abstraction.

---

# 3. Normative collection ontology

## 3.1 `List<T>`

```text
element type         T
extent               runtime
extent mutable       yes
element mutable      yes
owned                yes
contiguous           implementation may be contiguous; not required as public semantic contract
default bare literal no
const default        no
```

`List<T>` is the explicit dynamically resizable collection.

Representative use:

```telos
let queue: List<Int> = [1, 2, 3]
queue.append(4)
```

After the literal cutover, programmers asking for resizeable collection semantics must make that request explicit through expected type or constructor.

## 3.2 `Array<T>`

```text
element type         T
extent               runtime value information
extent mutable       no
element mutable      yes for ordinary runtime Array
owned                yes
contiguous           yes at Value-slot level in v1
const-capable        yes when explicitly selected
```

Two Arrays with different lengths have the same type:

```telos
let a: Array<Int> = <Int>[1, 2]
let b: Array<Int> = <Int>[1, 2, 3, 4]
```

Array is the runtime fixed-after-construction sequence.

## 3.3 `Vector<T,N>`

```text
element type         T
extent               static type argument N
extent mutable       no
element mutable      no
owned                yes
contiguous           yes at Value-slot level in v1
const-capable        yes
```

`Vector<Int,3>` and `Vector<Int,4>` are distinct applied types.

Vector is both:

- an immutable general homogeneous value sequence;
- the natural basis for mathematical dimension-aware operations when scalar/algebraic constraints are later added.

## 3.4 `Slice<T>`

Read-only zero-copy projection over structurally stable contiguous backing.

Initial sources:

```text
Array
Vector
Buffer
```

List live views are deferred because List structural mutation changes positional meaning.

## 3.5 `Buffer<T>`

Owned bounded construction/storage type.

```text
logical capacity     runtime
size                 runtime
0 <= size <= capacity
automatic growth     no
initialized prefix   yes
```

Buffer is not a second List and not an uninitialized-memory abstraction.

## 3.6 `Bag<T>` / `MutableBag<T>`

Bag semantics are multiplicity semantics:

```text
m(x) >= 0
```

with only positive counts stored.

Immutable Bag uses persistent storage.

MutableBag uses mutable hash storage.

## 3.7 `IndexedDict<K,V>`

Mapping plus semantic positional order.

Key lookup remains key-based.

Position is exposed through named operations:

```text
entryAt
keyAt
valueAt
indexOfKey
move
insert(at:)
removeAt
```

`dict[3]` must never ambiguously mean slot 3 when `K == Int`.

## 3.8 `OrderedSet<T>`

Unique ordered collection where order is semantically observable and equality-sensitive.

Current Set remains membership-semantic/order-insensitive.

## 3.9 `Bijection<K,V>`

Persistent finite partial bijection:

```text
forward[k] = v  iff  reverse[v] = k
```

No duplicate values.

No silent displacement.

## 3.10 `Interval<T>` / `RangeMap<K,V>`

`Interval<T>` models ordered bounds independent of stepping/progression.

Recommended source model:

```telos
enum Bound<T> {
  Unbounded
  Included(_ value: T)
  Excluded(_ value: T)
}

data Interval<T>(
  lower: Bound<T>,
  upper: Bound<T>,
)
```

`RangeMap<K,V>` stores disjoint intervals and performs point lookup by containment.

---

# 4. Bracket literal semantics

## 4.1 Neutral syntax

The parser should produce a neutral bracket-literal AST form.

Recommended conceptual shape:

```rust
Expr::BracketLiteral(Box<BracketLiteralExpr>)

struct BracketLiteralExpr {
    head: Option<BracketLiteralHead>,
    elements: Vec<BracketLiteralElement>,
    range: SourceRange,
}
```

Exact private Rust names are mechanically flexible.

## 4.2 Explicit heads

Target grammar:

```text
[...]         contextual/default family
<T>[...]      explicit Array family
<T,N>[...]    explicit Vector family
```

Each argument may use `_`:

```telos
<_>[...]
<Int>[...]

<_, _>[...]
<Int, _>[...]
<_, 3>[...]
<Int, 3>[...]
```

## 4.3 Family precedence

Family selection order is fixed:

```text
1. explicit literal head
2. unique concrete expected collection family
3. evaluation-phase default
```

Phase default:

```text
runtime -> Array
static  -> Vector
```

Phase default must not break overload/family ambiguity.

## 4.4 Contextual examples

```telos
let a: Array<_> = [1, 2, 3]
// Array<Int>

let b: List<_> = [1, 2, 3]
// List<Int>

let c: Vector<_, _> = [1, 2, 3]
// Vector<Int, 3>

const d: Array<_> = [1, 2, 3]
// static Array<Int>

const e: Vector<_, _> = [1, 2, 3]
// static Vector<Int, 3>
```

No intermediate collection should be constructed and converted.

## 4.5 Ambiguity

If contextual overload resolution yields two viable collection families, e.g. one candidate expects Array and another expects Vector, bare brackets are ambiguous.

The compiler must not choose Array merely because evaluation is runtime.

The programmer may write:

```telos
<_>[1, 2, 3]
```

or:

```telos
<_, _>[1, 2, 3]
```

to fix the family.

## 4.6 Element-type inference

Unconstrained literal element type follows existing union/common-type evidence behavior.

Expected type should constrain element checking directly.

Example:

```telos
let values: Array<Number> = [1, 2.5]
```

must result in `Array<Number>`, not first construct `Array<Int | Float>` and then convert.

## 4.7 Empty literals

Bare or inferred empty literal has no evidence for `T`.

These should be underconstrained:

```telos
let a = []
let b = <_>[]
let c = <_, _>[]
```

These are valid:

```telos
let a: Array<Int> = []
let b: List<Int> = []
let c: Vector<Int, _> = []
// N = 0
```

Do not silently infer `Dynamic` merely to make an empty literal compile.

## 4.8 Spread cardinality

Array and List may accept runtime-length spread.

Vector may accept spread only if exact cardinality is statically provable.

```telos
let a: Vector<Int, 3> = ...
let b = <_, _>[0, *a]
// Vector<Int, 4>
```

A spread from Array/List does not provide static cardinality merely because that particular runtime value currently has a length.

---

# 5. `_` inference holes

## 5.1 Meaning

`_` is a scoped inference request.

It is:

```text
a fresh solver variable
with the target parameter's expected kind/domain
which must be fully solved before durable publication
```

It is not:

```text
Dynamic
Any
wildcard existential
ignored argument
anonymous type alias
```

## 5.2 Syntax integration

The AST generic-argument syntax should gain a dedicated inference-hole form.

Do not parse `_` as a fake identifier or `Dynamic`.

## 5.3 Binding inference

Current binding annotation flow resolves a complete type before checking the initializer.

Partial annotations require a new inference-bearing contract.

Conceptually:

```text
annotation syntax
   ↓
InferenceTerm
   ↓
initializer checking
   ↓
constraint solving
   ↓
fully solved canonical type
   ↓
binding publication
```

No unsolved variable may enter:

- exported metadata;
- reflection;
- persistent semantic snapshots;
- compiler products;
- runtime type environments.

## 5.4 Initially allowed positions

Recommended first slice:

```text
let/let mut/const binding annotations with initializers
literal heads
explicit constructor generic arguments when call inference can solve them
```

Defer arbitrary public API signature holes unless separately designed.

---

# 6. Evaluation phase

## 6.1 Phase is separate from expected type

The checker needs explicit static/runtime phase context.

Conceptual:

```rust
enum EvaluationPhase {
    Runtime,
    Static,
}
```

This can be represented differently internally, but phase must be explicit and scoped.

## 6.2 Defaulting

```text
runtime + no family context -> Array
static  + no family context -> Vector
```

## 6.3 Nested propagation

Static phase must propagate into nested static expressions unless a construct establishes a different semantic boundary.

Example:

```telos
const x = Some([1, 2, 3])
```

The nested literal remains static and defaults to Vector unless contextual Option payload type selects another family.

---

# 7. Type-system requirements

## 7.1 Array

`Array<T>` is an ordinary one-parameter nominal generic application.

Array is invariant in `T` in v1.

## 7.2 Vector

`Vector<T,N>` is an ordinary nominal application with heterogeneous static arguments:

```text
T : Type
N : Int
N >= 0
```

Do not add a special type-store variant for Vector.

## 7.3 Cardinality reasoning

Vector construction requires reusable reasoning over positional argument cardinality.

Desired relation:

```text
Count(positionals) == N
```

This should reuse Pack/static machinery.

Avoid an Array/Vector-specific cardinality solver.

## 7.4 Variance

All new collection generic parameters are invariant in v1 unless a later dedicated variance design proves soundness.

Mutability requires invariance for Array/List/Buffer/MutableBag/OrderedSet/IndexedDict.

Keeping immutable families invariant initially avoids half-designed widening/update semantics.

---

# 8. Semantic products and compiler authority

The semantic checker should publish a checked literal-resolution product keyed by expression identity.

Conceptually:

```rust
struct SequenceLiteralResolution {
    family: SequenceLiteralFamily,
    result_type: TypeId,
    element_type: TypeId,
    extent: Option<StaticTerm>,
    construction: SequenceConstructionPlan,
}
```

Compiler responsibilities:

```text
consume the resolution
compile child expressions in checked order
materialize the selected target family
```

Compiler forbidden behavior:

```text
re-evaluate phase defaults
re-solve generic arguments
re-count and independently infer Vector N
fallback to List/Array if resolution is missing
insert hidden List -> Array/Vector conversion
```

Missing semantic resolution should be an internal compiler error.

---

# 9. Runtime dense storage architecture

## 9.1 Array

Recommended baseline:

```rust
struct ArrayObject {
    elements: Box<[Value]>,
    frozen: bool,
}
```

`frozen` may be represented differently, but const materialized Array must be runtime-protected.

## 9.2 Vector

Recommended baseline:

```rust
struct VectorObject {
    elements: Box<[Value]>,
}
```

or an equivalent immutable/shareable fixed block.

The runtime object must defend:

```text
elements.len() == N
```

as an invariant, while semantic analysis remains the primary proof authority.

## 9.3 Contiguous semantics

Public v1 guarantee:

```text
contiguous Telos Value slots
```

Not guaranteed:

```text
raw inline C layout of arbitrary T
```

Future unboxed specialization remains allowed.

## 9.4 Direct literal bytecode

Known-size literals should directly construct target storage.

Possible design:

```text
BuildArray count
BuildList count
BuildVector count
```

or a generalized equivalent.

Dynamic builders may be shared internally for Array/List expansion paths.

Vector dynamic builder must only be emitted when static exact result cardinality has already been proven.

---

# 10. Const/static sequence architecture

## 10.1 Shared static representation

Add a reusable homogeneous static sequence node.

Conceptually:

```rust
StaticSequence {
    result_type: TypeId,
    elements: Box<[StaticValueRef]>,
}
```

This can represent:

```text
Array<Int> static value
Vector<Int,3> static value
```

without conflating their classifiers.

## 10.2 Static evaluator

The evaluator consumes checked collection resolution.

It must not run arbitrary runtime collection constructors.

## 10.3 Materialization

```text
Static Vector -> immutable VectorObject
Static Array  -> frozen ArrayObject
```

Materialization must preserve exact result type and must not re-run initializer source.

## 10.4 Const Array

Direct source mutation should be rejected semantically.

Runtime mutation primitives must also reject a frozen Array through aliases.

No separate `ConstArray<T>` public type is introduced.

---

# 11. Slice and Buffer

## 11.1 Slice

Recommended representation:

```rust
struct SliceObject {
    owner: Value,
    start: usize,
    len: usize,
    backing_kind: ...,
}
```

Do not store an unrooted raw pointer.

Slice is read-only in v1.

Array mutations may be observed through an existing Slice.

Vector backing is stable.

Buffer slices refer only to already initialized prefix indices.

## 11.2 Buffer

Recommended representation:

```rust
struct BufferObject {
    logical_capacity: usize,
    elements: Vec<Value>,
}
```

Important:

```text
elements.len() == semantic size
Vec::capacity() != semantic capacity
```

No uninitialized Telos `Value` slots are exposed.

## 11.3 Vector conversion

Because Vector N is static and Buffer/Array/List sizes are runtime, conversion to Vector should be explicitly checked:

```text
Vector<T,N>.from(iterable) -> Result<...>
```

No runtime existential/dynamic-N Vector is introduced merely for conversion ergonomics.

---

# 12. Ordered mutable hash substrate

Current Map runtime already contains valuable machinery:

```text
ordered entry vector
hash bucket index
cached hash values
Telos hash/== callbacks
re-entry guards
```

P6 should factor that machinery into a reusable internal substrate while preserving current Map/Set behavior.

Consumers:

```text
Map
Set
MutableBag
IndexedDict
OrderedSet
```

This is internal representation reuse, not public semantic inheritance.

## 12.1 IndexedDict

Required invariants:

```text
unique keys
key lookup agrees with positional entry
positional order participates in equality
replace existing value does not implicitly move key
remove + reinsert appends unless explicit position supplied
```

## 12.2 OrderedSet

Required invariants:

```text
all values unique
at(i) and indexOf(value) agree
order participates in equality
positional insertion/move is explicit
```

Constructor duplicate handling should be strict.

An explicit deduplication constructor/factory may be added separately.

## 12.3 MutableBag

Backing:

```text
ordered/mutable hash map T -> count
```

Order is not semantically meaningful for Bag equality.

Stored counts must remain strictly positive.

---

# 13. Persistent hash substrate

P7 requires a reusable structurally shared immutable hash map.

Preferred class of data structure:

```text
HAMT / CHAMP / equivalent path-copy persistent hash trie
```

Requirements:

- structural sharing;
- collision correctness;
- Telos `hash` and `==`;
- old-root immutability;
- precise GC traversal;
- no heap borrow across callbacks;
- update returns new root;
- failure does not publish partial root.

A full-copy map may exist only as a test/reference oracle during development.

## 13.1 Bag

Conceptual:

```text
Bag<T> = PersistentHashMap<T, positive multiplicity>
```

Multiplicity arithmetic should preserve arbitrary-precision Telos `Int` semantics where counts can exceed native machine integers.

## 13.2 Bijection

Conceptual:

```text
forward: PersistentHashMap<K,V>
reverse: PersistentHashMap<V,K>
```

`inverse` should be O(1) where representation permits root swapping.

Conflict behavior is explicit.

No silent eviction.

---

# 14. Interval and RangeMap

## 14.1 Bound/Interval

Generic ordered-domain intervals are distinct from numeric progressions.

Recommended immutable value model:

```telos
enum Bound<T> {
  Unbounded
  Included(_ value: T)
  Excluded(_ value: T)
}

data Interval<T>(
  lower: Bound<T>,
  upper: Bound<T>,
)
```

Required operations:

```text
contains
overlaps
intersection
adjacent
empty/valid checks
```

## 14.2 RangeMap storage

Because stored ranges must not overlap, a persistent balanced ordered map keyed by lower endpoint is sufficient for point lookup:

```text
find predecessor/lower-bound candidate
test candidate interval containment
```

Expected complexity:

```text
point lookup       O(log n)
overlap/overlay    O(log n + k)
```

## 14.3 Normalization

Preferred final semantics canonicalize adjacent equal-valued intervals when safe, so representation equality approaches extensional mapping equality.

If v1 cannot do that without materially complicating value equality callbacks, the plan may temporarily use exact normalized interval-entry equality only with explicit checkpoint amendment.

---

# 15. Entry<K,V>

Replace or modernize the existing inconsistent nongeneric Entry concept with:

```telos
data Entry<K, V>(
  _ key: K,
  _ value: V,
)
```

This becomes the canonical explicit association value for new mapping constructors.

Do not invent `key => value` constructor-argument syntax.

Migrating existing Map iteration from labeled tuples to Entry may be staged if compatibility cost is high.

---

# 16. GC, rooting, and re-entry requirements

## 16.1 Dense collections

Array and Vector trace every contained `Value`.

Array overwrite must permit the replaced value to become collectible.

## 16.2 Slice

Slice traces the backing owner.

No raw interior pointer may be the sole lifetime mechanism.

## 16.3 Buffer

Trace only initialized `elements`.

Logical spare capacity contains no fake `Value`.

## 16.4 Persistent nodes

Every Value-bearing node reachable from a public root must be visible to GC tracing.

Structural sharing means nodes may be reachable from multiple roots; marking must be idempotent.

## 16.5 User callback re-entry

No collection implementation may hold a live `Heap`/object borrow across a Telos call to:

```text
hash
==
<=>
comparison helper
```

This rule applies to:

```text
Map
Set
MutableBag
IndexedDict
OrderedSet
Bag
Bijection
RangeMap
```

---

# 17. Equality and hash policy

Recommended v1 matrix:

| Type | Equality | Value-hashable/key-safe |
|---|---|---:|
| `Array<T>` | ordered structural | no |
| `List<T>` | ordered structural | no |
| `Vector<T,N>` | ordered structural | yes when element hash law holds |
| `Slice<T>` | current-content structural | no |
| `Buffer<T>` | current-content structural | no |
| `Set<T>` | membership/order-insensitive | no |
| `OrderedSet<T>` | ordered structural | no |
| `Map<K,V>` | mapping/order-insensitive | no |
| `IndexedDict<K,V>` | ordered entry equality | no |
| `Bag<T>` | multiplicity equality | yes |
| `MutableBag<T>` | multiplicity equality | no |
| `Bijection<K,V>` | mapping equality | yes |
| `RangeMap<K,V>` | canonical/extensional mapping equality | yes |

Const Array does not become hashable merely because a particular instance is frozen.

---

# 18. Performance contracts

The implementation should target:

| Operation | Required target |
|---|---:|
| Array index read/write | O(1) |
| Array fixed literal build | O(n), one dense element allocation |
| List index | O(1) |
| List append | amortized O(1) |
| Vector index | O(1) |
| Slice index | O(1) |
| Buffer index/set/push | O(1) |
| MutableBag lookup/update | expected O(1) |
| IndexedDict key lookup | expected O(1) |
| IndexedDict positional access | O(1) |
| OrderedSet membership/index lookup | expected O(1) |
| persistent Bag lookup | persistent-hash depth |
| persistent Bag update | path-copy hash depth |
| Bijection lookup either direction | persistent-hash depth |
| RangeMap point lookup | O(log n) |
| RangeMap overlap/overlay | O(log n + k) |

Performance tests should detect gross contract violations without turning C1 into a microbenchmarking project.

---

# 19. Migration requirements

Current accepted semantics and repository code assume bare `[...]` means List.

The cutover must be atomic and deliberate.

Required migration steps:

1. neutral literal infrastructure lands while preserving current default;
2. Array and Vector explicit/contextual construction become available;
3. static sequence support becomes available;
4. repository source using bare brackets is classified;
5. call sites requiring dynamic List semantics become explicit;
6. legacy decisions are amended/superseded;
7. bare runtime default flips to Array;
8. bare static default flips to Vector;
9. corpus and docs are updated in the same semantic epoch.

Forbidden shortcut:

```text
infer List if later code happens to call append
```

Collection type identity must not become flow-sensitive based on downstream method use.

---

# 20. Implementation plan decomposition

## COLL006.C1.P1 — Contextual literal and inference foundation

### Goal

Create the language/compiler infrastructure required by Array/Vector/List contextual literals without changing final bare-literal defaults yet.

### Required work

1. Refactor `ListLiteral` AST naming/ownership into neutral bracket/sequence literal representation.
2. Preserve current behavior during the refactor.
3. Add parser support for:
   ```text
   <T>[...]
   <T,N>[...]
   ```
4. Add generic/static inference-hole syntax `_`.
5. Integrate inference holes with canonical inference sessions.
6. Add inference-bearing binding annotations.
7. Ensure all holes resolve before durable publication.
8. Add checked sequence-literal semantic resolution product.
9. Make compiler lowering consume semantic resolution instead of syntax identity.
10. Keep legacy bare default behavior until the cutover plan.

### Major risks

- interning unresolved variables into TypeStore;
- parser/compiler retaining List as hidden authority;
- ambiguity handling being conflated with phase defaulting;
- `_` accidentally becoming `Dynamic`.

### Gate requirements

- behavior-neutral legacy corpus passes;
- explicit head parsing is exact;
- inference-hole positive/negative tests pass;
- no unresolved variables escape;
- compiler uses checked resolution.

---

## COLL006.C1.P2 — `Array<T>`

### Goal

Implement the complete runtime fixed-after-construction homogeneous sequence.

### Required work

1. Universe identity and canonical class registration.
2. Native representation.
3. minimal primitive floor:
   ```text
   size
   at
   set
   construction
   ```
4. source-level protocol and Iterator/Iterable conformance.
5. direct literal lowering.
6. dynamic spread builder.
7. contextual `Array<_>` literal realization.
8. explicit `<T>[...]`.
9. Array equality.
10. no resize selectors.
11. GC tracing.
12. diagnostic coverage.

### Gate requirements

- O(1) index read/write;
- no structural resize path;
- contextual/explicit literals build Array directly;
- runtime spread works;
- GC tests pass;
- no List conversion is used internally.

---

## COLL006.C1.P3 — `Vector<T,N>`

### Goal

Implement immutable statically dimensioned homogeneous vectors at runtime.

### Required work

1. Universe identity.
2. nominal generic signature with `T: Type`, `N: Int`.
3. reject negative N.
4. runtime representation.
5. immutable protocol.
6. structural equality and value hash.
7. exact constructor cardinality inference.
8. reusable Pack positional-count reasoning.
9. contextual `Vector<_,_>` realization.
10. explicit `<T,N>[...]`.
11. direct lowering.
12. static-cardinality spread rules.
13. cross-module canonical type identity tests.

### Gate requirements

- `Vector<Int,3>` and `Vector<Int,4>` distinct;
- every runtime value length matches N;
- mutation unavailable;
- constructor arity mismatch diagnosed semantically;
- runtime-unknown spread cardinality rejected;
- no Vector-specific alternate type identity or solver.

---

## COLL006.C1.P4 — Static homogeneous sequences and semantic cutover

### Goal

Implement const Array/Vector and atomically activate new bracket defaults.

### Required work

1. reusable static homogeneous sequence node.
2. static evaluator construction for checked Array/Vector literals.
3. static explicit constructors where semantically supported.
4. materialization descriptors.
5. Vector materialization.
6. frozen Array materialization.
7. direct const mutation diagnostics.
8. runtime alias mutation defense.
9. repository literal-use census.
10. migrate List-dependent bare literals.
11. amend/supersede legacy List-literal decisions.
12. flip:
    ```text
    runtime [...] -> Array
    static  [...] -> Vector
    ```
13. update docs/corpus.

### Gate requirements

- static initializer executes once;
- exported const sequence survives import/materialization;
- const Array remains Array;
- frozen Array mutation blocked through alias;
- bare runtime and static defaults changed atomically;
- explicit List behavior preserved.

---

## COLL006.C1.P5 — `Slice<T>` and `Buffer<T>`

### Goal

Complete the contiguous view and bounded construction layer.

### Required work

1. Slice native representation.
2. read-only indexing/iteration.
3. Array/Vector/Buffer slice sources.
4. backing-owner tracing.
5. nested slicing.
6. Buffer runtime representation.
7. logical capacity.
8. initialized-prefix rules.
9. push/set/index.
10. capacity failure semantics.
11. Array/Vector conversion APIs where appropriate.
12. amend P14 parser-snapshot direction:
    ```text
    private Array snapshot + Slice view
    ```

### Gate requirements

- no raw-pointer lifetime dependency;
- Slice retains owner through GC;
- Buffer cannot read uninitialized slots;
- Buffer cannot exceed semantic capacity;
- parser snapshot use case works without dimensioned Vector abuse.

---

## COLL006.C1.P6 — Ordered mutable hash family

### Goal

Factor current Map/Set storage correctly and add MutableBag, IndexedDict, OrderedSet.

### Required work

1. verify Map/Set current invariants.
2. extract reusable ordered-hash backing.
3. preserve existing Map/Set observable behavior.
4. implement MutableBag.
5. implement IndexedDict.
6. implement OrderedSet.
7. positional-index repair on insert/remove/move.
8. duplicate/conflict diagnostics.
9. re-entry tests.
10. GC tests.

### Gate requirements

- Map/Set regression suite green;
- no duplicate hash/equality implementation;
- IndexedDict key/index views agree;
- OrderedSet uniqueness/index views agree;
- MutableBag totals agree with counts;
- re-entrant user hash/equality remains safe.

---

## COLL006.C1.P7 — Persistent hash family

### Goal

Build reusable persistent hashing and implement Bag/Bijection.

### Required work

1. persistent hash node model.
2. lookup/update/remove.
3. collision nodes.
4. path-copy structural sharing.
5. GC traversal of shared nodes.
6. Telos hash/equality callbacks.
7. Bag multiplicity semantics.
8. Bag algebra.
9. Bijection forward/reverse roots.
10. inverse.
11. explicit conflict behavior.
12. property tests.

### Gate requirements

- old roots remain unchanged;
- collisions correct;
- shared-node GC safe;
- Bag algebra laws hold;
- Bijection inverse invariant holds after every successful update;
- failure publishes no half state.

---

## COLL006.C1.P8 — Interval and RangeMap

### Goal

Add general ordered intervals and persistent interval-domain mappings.

### Required work

1. `Bound<T>`.
2. `Interval<T>`.
3. validity/empty semantics.
4. contains/overlaps/intersection/adjacency.
5. persistent ordered tree.
6. point lookup.
7. strict insert.
8. remove.
9. overlay/split.
10. normalization/coalescing decision.
11. unbounded endpoints.
12. custom ordered-domain tests.

### Gate requirements

- no overlap invariant;
- at most one point result;
- O(log n) lookup design;
- randomized reference-model agreement;
- no forced redesign of numeric Range.

---

## COLL006.C1.P9 — Convergence and certification

### Goal

Close C1 coherently rather than leaving nine locally-correct but inconsistent collection implementations.

### Required work

1. modernize `Entry<K,V>` boundary.
2. normalize constructor naming and `@constructor call(...)` use where appropriate.
3. conversion graph.
4. Iterable/Iterator consistency.
5. diagnostics audit.
6. public docs/examples.
7. collection taxonomy docs.
8. property-test suite consolidation.
9. GC stress.
10. performance contract checks.
11. broad migration audit.
12. checkpoint verification ledger closure.
13. residual failure classification.
14. final walkthrough/handoff/closure records.

### Gate requirements

- all required plan ledgers complete or superseded;
- no contradictory public collection APIs;
- no old `SetList` terminology;
- no stale bare-List documentation;
- focused cross-layer certification green;
- broader release gate status truthfully recorded.

---

# 21. Parallelization strategy

Safe dependency graph:

```text
P1
├── P2 Array
├── P3 Vector
└── P6 ordered-hash substrate

P2 + P3
└── P4 static sequences/cutover
    └── P5 Slice/Buffer

P6
└── P7 persistent hash

P8 can proceed after P1 once its ordering/Interval architecture is reverified.

P2–P8
└── P9 convergence/certification
```

Important:

- P4 must not begin the semantic cutover until P2 and P3 runtime families are stable.
- P5 should not define parser snapshots independently of the P14/C4 integration decision.
- P6 must preserve Map/Set before new ordered collections are considered complete.
- P7 must not be blocked on Array/Vector.
- P8 is largely independent of persistent hashing.

---

# 22. Fixed / flexible / verify-first classification

## FIXED

The implementer must not change without consultation:

- runtime bare default Array;
- static bare default Vector;
- `<T>` explicit Array;
- `<T,N>` explicit Vector;
- List explicit resizeable semantics;
- Array runtime extent;
- Vector static N;
- Vector immutability;
- const Array support;
- `_` as a solver variable, never Dynamic;
- semantic checker family-selection authority;
- no phase-default overload tie-breaker;
- OrderedSet name;
- existing Iterator/Iterable reuse;
- no duplicate Collection trait;
- persistent old-root immutability;
- C4 static-value/materialization ownership.

## MECHANICALLY FLEXIBLE

The implementer may adapt:

- private file/module decomposition;
- helper names;
- precise bytecode encoding;
- private dense storage wrapper;
- private persistent trie branching factor;
- balanced ordered tree implementation choice;
- local iterator implementation;
- current-repository helper APIs that drift mechanically from the plan.

## VERIFY-FIRST

Each plan must verify:

- live AST/parser symbols;
- inference APIs;
- static Int/domain representation;
- C4 materialization APIs;
- primitive-floor governance;
- Map/Set re-entry handling;
- P14 current status;
- current decision/spec filenames;
- current collection corpus/migration surface;
- repository revision and dirty state.

If verification changes mechanics only, adapt.

If it invalidates ownership or semantics, STOP AND CONSULT.

---

# 23. Global forbidden shortcuts

Do not:

1. treat `_` as `Dynamic`;
2. emit `Array<Dynamic>` for underconstrained empty literals;
3. make the compiler re-infer literal family;
4. use runtime phase default as an overload tie-breaker;
5. silently convert List to Array/Vector to satisfy literal context;
6. implement Vector N as hidden runtime metadata instead of a real static type argument;
7. create one runtime class per Vector dimension;
8. encode Array extent into its type;
9. add `ConstArray`;
10. allow const Array mutation through aliases;
11. expose raw dense storage pointers as stable source semantics;
12. create a second Iterable/Iterator hierarchy;
13. introduce `SetList`;
14. duplicate Map's re-entrant hash machinery in each new hash collection;
15. call Rust `Hash`/`Eq` on Telos Values as semantic key equality;
16. clone entire maps on every Bag update and call that the final persistent implementation;
17. implement RangeMap as a linear List in the final architecture;
18. make current numeric Range generic merely to avoid defining Interval;
19. hold heap borrows across user hash/equality/comparison callbacks;
20. repair unrelated repository failures opportunistically;
21. run broad workspace tests after every small edit;
22. claim verification not actually executed.

---

# 24. STOP AND CONSULT triggers

STOP AND CONSULT if any of the following occurs:

1. inference holes require canonical TypeStore pollution with unsolved variables;
2. partial annotations cannot be represented with existing inference sessions without redefining type identity;
3. literal family cannot be selected in semantic analysis without parser/compiler duplication;
4. overload applicability and collection family context cannot be separated cleanly;
5. Vector N cannot remain an ordinary static Int generic argument;
6. exact positional cardinality requires a new collection-specific solver;
7. const Array alias safety cannot be enforced without changing classifier semantics or introducing arbitrary copying;
8. C1 work conflicts with C4 static identity/materialization authority;
9. Map/Set backing extraction changes current observable behavior;
10. a hash operation requires heap borrows across user callbacks;
11. persistent-node values cannot be traced safely with current GC ownership;
12. RangeMap requires a global ordering/type-system redesign not anticipated by P8;
13. a planned asymptotic guarantee appears impossible without changing public semantics;
14. tests pass only after weakening an invariant;
15. the same nontrivial semantic failure survives one serious corrective attempt;
16. implementation must enter an unexpected subsystem with architectural consequences;
17. the repository's current normative authorities materially conflict with this architecture;
18. patch scope becomes materially larger than the plan's bounded objective.

The implementer cannot self-waive these triggers.

---

# 25. Failure classification

Use:

```text
A — definitely caused by current change
B — probably caused by current change
C — unclear
D — clearly pre-existing/unrelated
```

Policy:

```text
A/B -> active implementation responsibility
C   -> one bounded classification pass; defer if nonblocking and still unclear
D   -> record and defer immediately
```

Do not consume architectural consultation on unrelated baseline failures unless they block proof of active acceptance.

---

# 26. Build / stabilize / certify modes

## BUILD

Goal: implement architecture with minimal useful tests.

Run:

- exact new regressions;
- syntax/parser tests for parser edits;
- focused semantic tests for inference edits;
- focused runtime tests for representation edits.

Do not run:

- workspace-wide test;
- workspace-wide clippy;
- broad unrelated package suites.

## STABILIZE

Goal: prove a coherent plan outcome.

Run:

- directly affected semantic/compiler/runtime suites;
- relevant language-corpus lanes;
- collection contract tests;
- cross-layer integration tests where architecture spans layers.

Classify unrelated failures.

## CERTIFY

Goal: checkpoint confidence.

Run broader gates only where the plan/checkpoint requires them.

Promote `verification` only when evidence justifies it.

---

# 27. Verification ladder

Use:

```text
T0 — exact reproducer/new regression
T1 — directly affected feature tests
T2 — owning subsystem/module suite
T3 — adjacent cross-layer integration
T4 — broad crate/package verification
T5 — workspace/release verification
```

Typical use:

- P1: T0/T1 semantic/parser/compiler; T2 semantic near completion.
- P2/P3: T0/T1 runtime + language corpus; T2 core collection suites.
- P4: T3 static/compiler/runtime integration; broader migration corpus.
- P6/P7/P8: T0/T1 property/invariant tests; T2 collection runtime suites.
- P9: T3/T4 and any explicitly required T5 certification.

---

# 28. Coverage obligation matrix

| Dimension | P1 | P2 | P3 | P4 | P5 | P6 | P7 | P8 | P9 |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| parser syntax | ✓ |  |  | ✓ migration |  |  |  |  | ✓ |
| contextual inference | ✓ | ✓ | ✓ | ✓ |  |  |  |  | ✓ |
| `_` solving | ✓ | ✓ | ✓ | ✓ |  |  |  |  | ✓ |
| runtime representation |  | ✓ | ✓ |  | ✓ | ✓ | ✓ | ✓ | ✓ |
| static evaluation |  |  |  | ✓ |  |  | optional | optional | ✓ |
| GC/rooting |  | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| re-entry callbacks |  |  |  |  |  | ✓ | ✓ | ✓ | ✓ |
| persistence |  |  |  |  |  |  | ✓ | ✓ | ✓ |
| property laws |  |  | ✓ |  |  | ✓ | ✓ | ✓ | ✓ |
| performance contracts |  | ✓ | ✓ |  | ✓ | ✓ | ✓ | ✓ | ✓ |
| migration |  |  |  | ✓ |  |  |  |  | ✓ |
| docs/API convergence |  |  |  |  |  |  |  |  | ✓ |

---

# 29. Specific test requirements

## 29.1 Literal/inference

Positive:

```telos
let a = [1,2,3]                       // after cutover: Array<Int>
let b: Array<_> = [1,2,3]            // Array<Int>
let c: List<_> = [1,2,3]             // List<Int>
let d: Vector<_,_> = [1,2,3]         // Vector<Int,3>

let e = <_>[1,2,3]                   // Array<Int>
let f = <Int>[1,2,3]                 // Array<Int>
let g = <_,_>[1,2,3]                 // Vector<Int,3>
let h = <Int,_>[1,2,3]               // Vector<Int,3>
let i = <_,3>[1,2,3]                 // Vector<Int,3>
let j = <Int,3>[1,2,3]               // Vector<Int,3>

const k = [1,2,3]                    // Vector<Int,3>
const l: Array<_> = [1,2,3]          // static Array<Int>
const m = <_>[1,2,3]                 // static Array<Int>
const n = <_,_>[1,2,3]               // static Vector<Int,3>
```

Negative:

```telos
let x: Vector<Int,4> = [1,2,3]
let x: List<Int> = <Int>[1,2,3]
let x: Array<Int> = <Int,3>[1,2,3]

let x = <_>[]
let x = <_,_>[]
```

## 29.2 Binding behavior

```telos
let a: Array<Int> = [1,2,3]
a[0] = 9          // valid
a = other         // invalid rebinding

let mut b: Array<Int> = [1,2,3]
b[0] = 9          // valid
b = other         // valid rebinding

let v: Vector<Int,3> = [1,2,3]
v[0] = 9          // invalid
```

## 29.3 Const Array alias safety

Test direct and aliased mutation.

Conceptually:

```telos
const a: Array<Int> = [1,2,3]
```

Direct write is statically rejected.

Passing the materialized value through an ordinary runtime Array-typed or Dynamic alias must still reject mutation at runtime.

## 29.4 Vector cardinality

Test:

```text
N=0
N=1
large concrete N
constructor exact match
too few
too many
statically known Vector spread
runtime-length Array/List spread rejection
cross-module re-export
```

## 29.5 Ordered mutable structures

Randomized sequences of:

```text
insert
remove
move
replace
lookup
index lookup
```

must preserve mirror invariants.

## 29.6 Persistent structures

Use a slow reference oracle.

Verify:

```text
old root unchanged
new root correct
collision handling
repeated update
remove absent
shared branch behavior
GC after dropping one of several roots
```

## 29.7 RangeMap

Randomized interval operations versus a simple reference model.

Boundary coverage:

```text
included/excluded lower
included/excluded upper
unbounded lower/upper
empty interval
adjacent intervals
overlap rejection
overlay split
remove
coalescing
```

---

# 30. Error vocabulary

Prefer a bounded set of semantically meaningful errors/diagnostics:

```text
InferenceHoleUnderconstrained
InferenceHoleNotAllowedHere

SizeMismatchError

ConstantMutationError

CapacityError

DuplicateElementError

BijectionConflictError

InvalidIntervalError
RangeOverlapError
```

Reuse existing canonical errors such as `IndexError` and `KeyError` where they already represent the condition.

Do not create one error class per convenience method.

---

# 31. Conversion graph

Conversions should be explicit when they allocate or change collection semantics.

Recommended:

```text
List<T> -> Array<T>
Array<T> -> List<T>

Array<T> -> Slice<T>        zero-copy view
Vector<T,N> -> Slice<T>     zero-copy view
Buffer<T> -> Slice<T>       zero-copy view

Array<T> -> Vector<T,N>     checked exact size
List<T> -> Vector<T,N>      checked exact size
Buffer<T> -> Vector<T,N>    checked exact size

Bag<T> <-> MutableBag<T>
```

Literal contextualization is not an implicit conversion because the literal has not yet acquired collection identity.

Do not add general implicit List/Array/Vector conversions.

---

# 32. Constructor model

Use ordinary Telos constructors.

Where function-like construction is desirable, prefer:

```telos
@constructor
call(...)
```

Do not invent free functions.

Representative intent:

```telos
Array<Int>(1,2,3)
Vector<Int,3>(1,2,3)
Vector(1,2,3)
List<Int>(1,2,3)
```

Exact signatures must be grounded in the live constructor/call and rest-pack semantics during the owning plan.

Vector cardinality must be proved semantically, not merely checked in a runtime constructor.

---

# 33. P14 / parser snapshot interaction

Dimensioned `Vector<T,N>` is not appropriate as the general result of snapshotting a runtime-sized List.

Recommended parser/source architecture:

```text
source snapshot owns private Array<T>
consumer receives read-only Slice<T> or source-level indexed access
```

P5 must coordinate this with the live LANG007.C4.P14 state.

Do not introduce a duplicate `ImmutableSequence<T>` merely because parser snapshots need immutable observation.

---

# 34. Future-compatible numerical representation

V1 correctness uses contiguous `Value` slots.

The implementation must preserve the possibility of later specialized storage, e.g.:

```rust
enum DenseStorage {
    Generic(Box<[Value]>),
    Float64(Box<[f64]>),
    // later
}
```

Potential future beneficiaries:

```text
Vector<Float,2>
Vector<Float,3>
Vector<Float,4>
Array<Float>
```

Do not expose `Box<[Value]>` as a language ABI.

SIMD/unboxing is explicitly not a C1 acceptance requirement.

---

# 35. Lifecycle obligations

Each numbered plan must:

1. read `COLL006.C1-CHECKPOINT.md`;
2. verify the relevant current repository baseline;
3. add/activate itself in the checkpoint ledger at start;
4. update checkpoint state only at meaningful durable architecture/verification events;
5. record consultations/amendments;
6. record unrelated/baseline failures;
7. distinguish designed coverage from executed tests;
8. update final completion/verification state;
9. create `<PLAN>-walkthrough.md`;
10. create `<PLAN>-handoff.md` unless explicitly unnecessary;
11. set the next action.

The checkpoint remains the shared durable state authority.

This master record should be amended only when architecture, acceptance, or overall decomposition materially changes.

---

# 36. Implementer operating rules

For every plan:

```text
preserve unrelated dirty work
inspect git status first
do not reset/clean
stage only authorized work
adapt mechanical repository drift locally
escalate architectural drift
run the smallest informative test during build mode
do not repair unrelated failures
do not silently weaken invariants
```

When debugging a semantic failure, record:

```text
Observed
Hypothesis
Evidence
Prediction
Discriminating test
```

Allow one serious corrective attempt for the same nontrivial semantic failure.

If it persists, consult.

---

# 37. Master acceptance gates

These gates summarize the checkpoint rather than replacing plan-specific gates.

## G1 — Literal/inference foundation

Proves:

```text
neutral bracket AST
inference-hole semantics
partial initializer-driven annotations
semantic resolution ownership
```

## G2 — Runtime sequence family

Proves:

```text
Array
List
Vector
direct lowering
runtime invariants
```

## G3 — Static sequence family and cutover

Proves:

```text
const Vector
const Array
materialization
bare runtime -> Array
bare static -> Vector
repository migration
```

## G4 — Contiguous views/storage

Proves:

```text
Slice
Buffer
P14 snapshot convergence
```

## G5 — Ordered mutable hash family

Proves:

```text
Map/Set preserved
MutableBag
IndexedDict
OrderedSet
```

## G6 — Persistent hash family

Proves:

```text
persistent hash
Bag
Bijection
```

## G7 — Interval mapping family

Proves:

```text
Bound
Interval
persistent ordered storage
RangeMap
```

## G8 — Whole-C1 certification

Proves:

```text
API coherence
GC/rooting
property laws
performance contracts
migration closure
residual issue classification
checkpoint truth
```

---

# 38. Completion criteria

`COLL006.C1` may be marked `COMPLETE` only when:

1. every required plan is complete or explicitly superseded;
2. the final source semantics match this record or an approved amendment;
3. `COLL006.C1-CHECKPOINT.md` reflects the actual architecture;
4. legacy bare-List default semantics are fully migrated/amended;
5. residual failures are classified;
6. all required focused acceptance evidence exists;
7. verification state is truthful;
8. the next program action is explicit.

`RELEASE_COMPLETE` is stronger than checkpoint implementation completion and must only be used if the required broad release gates actually ran successfully.

---

# 39. Immediate next action

The next artifact to author is:

```text
COLL006.C1.P1 — Contextual Sequence Literal and Inference Foundation
```

That plan should be patch-grade and executable, but narrowly limited to:

```text
neutralizing ListLiteral ownership
adding explicit literal heads
adding inference holes
supporting initializer-driven partial annotations
publishing checked sequence-literal resolution
preserving current bare-literal behavior during P1
```

P1 must not prematurely implement the repository-wide default cutover.

The cutover belongs to P4 after Array, Vector, and static sequence support exist and have focused evidence.


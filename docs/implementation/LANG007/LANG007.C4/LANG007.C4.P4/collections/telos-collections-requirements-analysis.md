# Telos Collections Expansion — Repository-Grounded Requirements Analysis

**Repository:** `aureat/phalcom-lang`  
**Branch inspected:** `main`  
**Revision inspected:** `e0de68b2e6394cc28a21dc3ca6cf0dae7c7963f4`  
**Date:** 2026-10-07  
**Document status:** Requirements analysis / architecture investigation  
**Implementation status:** No implementation performed by this document

---

## 0. Executive conclusion

The proposed collection expansion is viable, but the repository shows that several earlier assumptions must be corrected before implementation.

The most important repository-grounded conclusions are:

1. **`List<T>` is definitively the mutable dynamic-array collection.** Its runtime representation is `ListObject { elements: Vec<Value> }`, with indexed mutation, append, and slice replacement. A general-purpose mutable `Vector<T>` would therefore be redundant. Reserving `Vector<T>` for an immutable value sequence is architecturally sound.

2. **The already-approved C4 P14 “immutable homogeneous indexed snapshot” work overlaps almost exactly with the proposed `Vector<T>`.** Telos should not implement both `ImmutableSequence<T>` and `Vector<T>`. P14 should be amended/converged so that `Vector<T>` becomes the canonical immutable homogeneous indexed collection and also satisfies the parser-snapshot use case.

3. **`[a, b, c]` currently means `List`, not `Array`.** This is not merely an implementation accident; accepted TDR-0025 and TDR-0028 explicitly define bracket literals as fresh mutable `List` construction, and the runtime now has a `BuildList` path. Changing unconstrained brackets to infer `Array<T,N>` is a separate language change requiring an explicit superseding/amending decision, parser/semantic/lowering migration, and compatibility work. The collection implementation tranche should not silently change this.

4. **Current binding semantics differ from the earlier brainstorming shorthand.** Under TDR-0090:
   - `let` is a runtime **immutable binding**;
   - `let mut` is a runtime **mutable/rebindable binding**;
   - `const` is a compile-time immutable binding.

   A `let`-bound mutable object can still expose mutating methods; `let` prevents rebinding of the variable, not necessarily mutation of the referenced/runtime value. Therefore the correct statement is: **runtime `Array<T,N>` is a mutable fixed-extent object; `let` does not itself create its mutability.**

5. **Canonical `Iterator` and `Iterable` traits already exist and are substantial.** They have associated `Item`/`Iterator` types and rich default combinators. The proposed collection work must reuse them. Adding another `Iterable`, `Iterator`, or generic “Collection trait” would be wrong. `Collection` is already a nominal Universe superclass.

6. **`Map<K,V>` is already insertion-ordered internally and externally.** `MapObject` stores entries in a `Vec`, exposes indexed entry/key access internally, and public key/value/entry views preserve encounter order. Consequently, the proposed `IndexedDict<K,V>` as merely “an insertion-ordered mapping with access by index” is almost redundant with existing `Map`. It is justified only if its **order is semantic** and it adds direct positional mutation/index APIs such as `entryAt`, `indexOfKey`, `insert(at:)`, `removeAt`, and `move`, with order-sensitive equality.

7. **`Set<T>` is already insertion-ordered and indexable.** It has `rawAt`/`at` by insertion position while equality remains set-like and order-insensitive. Therefore the proposed `SetList<T>` is justified only if it adds **sequence-semantic order**: positional insertion/removal/move, efficient `indexOf`, and order-sensitive equality. Without those stronger semantics, it duplicates `Set`.

8. **A mapping `Entry` concept already exists, but it is not yet the right canonical abstraction.** `map.ph` contains a nongeneric `Entry` class whose fields are declared `mut`, despite its documentation calling it immutable; map iteration currently yields labeled tuples `(key: K, value: V)` instead. The modern architecture should introduce/upgrade to a generic immutable value:
   ```telos
   data Entry<K, V>(
     _ key: K,
     _ value: V,
   )
   ```
   which gives positional construction such as `Entry(key, value)` without inventing inadmissible `key => value` argument syntax. Migrating all map iteration to `Entry` is **not required** for the first slice; labeled tuple entries are already established and can remain compatible.

9. **`Range` is currently numeric, nongeneric, and partly progression-oriented.** `RangeMap<K,V>` cannot honestly promise ranges over dates and arbitrary ordered domains using the current `Range`. The clean dependency is a new generic immutable **interval descriptor** (recommended name: `Interval<T>`) separated from numeric iteration/progression. `RangeMap` should index `Interval<K>` rather than force the existing numeric `Range` to become generic in the same tranche.

10. **No reusable persistent map implementation exists in the repository.** There is no existing HAMT/persistent hash-map substrate. `packages/immer` is a clone-and-mutate façade, not persistent structural sharing. A truly persistent `Bag<T>` and an immutable persistent `Bijection<K,V>` therefore require a new internal persistent associative storage substrate.

11. **Static `Array<T,N>` is a cross-layer feature.** `StaticValueStore` currently has nodes for primitives, Tuple, Record, Data, Variant, and Pack, but no generic homogeneous sequence/Array node. `materialization.rs` still marks structural materialization as deferred. Supporting `const Array<T,N>` requires semantic static-value representation **and** materialization policy; it is not accomplished by adding only a runtime `Array` class.

12. **`Array<T,N>` is syntactically aligned with Telos’s static-domain model but still needs an implementation proof.** `<N: Int>` is normative syntax for a typed static generic parameter, but there is little live runtime collection code using value-parameterized nominal classes. Before `Array<T,N>` becomes a kernel/native type, the implementation must verify exact applied-type formation, substitution, reflection, and lowering for a runtime nominal type with a static `Int` argument.

The recommended implementation architecture is therefore:

```text
Existing foundation
──────────────────────────────────────────────────────────
Collection superclass
Iterator / Iterable traits
List<T>                   mutable dynamic sequence
Map<K,V>                  mutable insertion-ordered mapping
Set<T>                    mutable insertion-ordered set
Tuple                     immutable structural product
Range                     numeric range/progression descriptor
Bytes                     fixed mutable octet buffer

New shared value/foundation layer
──────────────────────────────────────────────────────────
Entry<K,V>                immutable generic association value
Interval<T>               generic ordered interval descriptor
PersistentHashMapStorage  internal structural-sharing hash map
PersistentIntervalTree    internal persistent ordered interval index
StaticSequence            semantic homogeneous static-value substrate

New collection layer
──────────────────────────────────────────────────────────
Array<T,N>                fixed-extent mutable runtime sequence
Vector<T>                 immutable homogeneous indexed value sequence
Slice<T>                  read-only zero-copy indexed range view
Buffer<T>                 fixed-capacity mutable initialized-prefix builder
Bag<T>                    persistent immutable multiset
MutableBag<T>             mutable multiset
RangeMap<K,V>             persistent point-to-value interval mapping
Bijection<K,V>            persistent bidirectional one-to-one map

Conditional / must be strengthened before implementation
──────────────────────────────────────────────────────────
IndexedDict<K,V>          only if order is semantic + positional mutation
SetList<T>                only if order is semantic + list-like positional mutation
```

---

# 1. Scope of this analysis

This document analyzes requirements for the collection family discussed in the design conversation:

```text
Array<T,N>
Vector<T>
Slice<T>
Buffer<T>

Bag<T>
MutableBag<T>

RangeMap<K,V>
Bijection<K,V>
IndexedDict<K,V>
SetList<T>
```

It additionally analyzes the supporting abstractions required to implement them coherently:

```text
Entry<K,V>
Interval<T>
persistent hash-map storage
persistent ordered/interval-tree storage
static homogeneous sequence values/materialization
iteration/conformance contracts
hash/equality/order requirements
error types
construction/conversion conventions
native runtime integration
testing infrastructure
```

The report deliberately distinguishes:

- **existing repository facts**;
- **requirements forced by those facts**;
- **recommended architecture**;
- **design gates that still require an explicit decision**.

It does **not** treat earlier speculative examples as authoritative when they disagree with current Telos syntax, decisions, or runtime architecture.

---

# 2. Repository authorities inspected

The analysis is grounded primarily in the following live sources.

| Area | Repository source |
|---|---|
| Universe identities | `phalcom-native-meta/src/universe.rs` |
| Kernel class bootstrap | `phalcom-core/src/universe/core_classes.rs` |
| Heap object variants | `phalcom-core/src/heap/object.rs` |
| List representation | `phalcom-core/src/heap/list.rs` |
| Map/Set representation | `phalcom-core/src/heap/map.rs` |
| Range representation | `phalcom-core/src/heap/range.rs` |
| Tuple representation | `phalcom-core/src/heap/tuple.rs` |
| Collection superclass | `phalcom-core/core/universe/src/collections/collection.ph` |
| List surface | `phalcom-core/core/universe/src/collections/list.ph` |
| Map surface | `phalcom-core/core/universe/src/collections/map.ph` |
| Set surface | `phalcom-core/core/universe/src/collections/set.ph` |
| Range surface | `phalcom-core/core/universe/src/collections/range.ph` |
| Bytes surface | `phalcom-core/core/universe/src/collections/bytes.ph` |
| Iterator/Iterable laws | `phalcom-core/core/universe/src/collections/iterator_laws.ph` |
| Root equality/hash/comparison | `phalcom-core/core/universe/src/object/object.ph` |
| Collection contract tests | `phalcom-core/tests/core/collections/contract.rs` |
| Constructor decision | `docs/decisions/accepted/0051-constructors-are-ordinary-class-side-methods.md` |
| Current binding/static decision | `docs/decisions/accepted/0090-structural-static-evaluation-packs-products-sums-and-binding-semantics.md` |
| Data value semantics | `docs/decisions/accepted/0080-data-is-a-nominal-transparent-immutable-value-product.md` |
| Type/static domains | `docs/specs/type-level-domains.md` |
| Existing literal decisions | `docs/decisions/accepted/0025-list-literal-syntax.md`, `docs/decisions/accepted/0028-collections-representation-and-literals.md` |
| Static value arena | `phalcom-semantic/src/static_value/store.rs` |
| Static materialization | `phalcom-semantic/src/materialization.rs` |
| C4 lifecycle state | `docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md` |
| P14 immutable-sequence proposal | `docs/implementation/LANG007/LANG007.C4/LANG007.C4.P4/LANG007.C4.P4.R1/LANG007.C4.P4.R1-stdlib-parse-telos-follow-on-plan.md` |

Where older plans disagree with live implementation or accepted decisions, this report treats accepted decisions and current code as authoritative.

---

# 3. Verified current collection architecture

## 3.1 Canonical Universe collection identities

`UniverseKey` currently includes:

```text
Collection
List
Map
Set
Tuple
Pack
Record
Range
Bytes
```

The canonical class relation table places:

```text
Collection < Object

List   < Collection
Map    < Collection
Set    < Collection
Tuple  < Collection
Range  < Collection
Bytes  < Collection
```

`Pack` and `Record` currently sit directly under `Object`, reflecting their structural/product role rather than the ordinary runtime collection superclass.

None of the proposed new collection types currently exists in the Universe catalog.

### Requirement

Any new **kernel/native** collection identity must update the canonical Universe identity system; creating an ad-hoc class only in the VM is architecturally incorrect.

For a native collection this normally implies synchronized changes across:

```text
phalcom-native-meta/src/universe.rs
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/<type>.rs
phalcom-core/src/primitive/<type>.rs
phalcom-core/core/universe/src/collections/<type>.ph
native surface metadata / generated surface
VM bootstrap / tracing / rendering / exhaustive object matches
collection contract tests
```

Native representation classes must also be marked `native_repr` so generic `new_` cannot manufacture an invalid `InstanceObject` pretending to be that native type.

---

## 3.2 `List<T>` is already the resizable vector

`ListObject` is:

```rust
pub struct ListObject {
    elements: Vec<Value>,
}
```

and provides:

- `get`;
- indexed `set`;
- `push`;
- `replace_slice`;
- length;
- ordinary `Vec` growth.

The public `List<T>` surface adds:

- `append`;
- `prepend`;
- `insert`;
- `remove`;
- `popFirst`;
- `popLast`;
- `removeAll`;
- `swap`;
- indexed/range replacement;
- structural list equality;
- iteration.

### Conclusion

Do **not** introduce another mutable dynamic `Vector<T>`. The user’s decision to reserve `Vector<T>` for an immutable value sequence is consistent with the live implementation.

---

## 3.3 `Map<K,V>` is already insertion ordered

`MapObject` has two principal structures:

```text
entries: Vec<(key, value, cached_hash_bucket)>
index:   HashMap<hash_bucket, Vec<entry_slot>>
```

The entry vector is explicitly the source of truth for insertion/encounter order.

It already provides:

```text
entry_at(slot)
key_at(slot)
```

and the public `Map` surface exposes live `keys`, `values`, and `entries` views in encounter order.

Map equality, however, is **mapping equality**, not order-sensitive sequence equality:

```text
same key set + equal corresponding values
```

### Consequence for `IndexedDict`

The short definition:

> insertion-ordered mapping with access by index

does **not** establish enough distinction from current `Map`.

A separate `IndexedDict<K,V>` is justified only if position becomes a first-class semantic property with APIs and laws that ordinary `Map` intentionally does not have.

Required differentiators are detailed in §12.

---

## 3.4 `Set<T>` is also insertion ordered and indexable

`Set<T>` reuses `MapObject` and exposes:

```telos
at(_ i)
```

which returns the member at insertion-order position `i`.

Its equality intentionally ignores insertion order.

### Consequence for `SetList`

The short definition:

> ordered set / list of unique elements

also overlaps heavily with existing `Set`.

A separate `SetList<T>` should exist only if:

- positional order participates in equality;
- positional insertion/removal/move is part of the contract;
- efficient value-to-index lookup is part of the contract;
- list-like ordering is semantic rather than only deterministic encounter order.

Without those properties, `SetList` would be a synonym for current `Set`.

---

## 3.5 `Range` is not a generic ordered interval

Current `Range` is:

```telos
native class Range is Collection
```

with native bounds typed as:

```text
Option<Number>
```

and iteration that assumes integral numeric stepping by `+ 1`.

Its runtime representation records:

```text
lower
upper
upper_inclusive
```

but its public semantics combine two concepts:

1. **interval containment/bounds**;
2. **numeric progression/iteration**.

### Consequence for `RangeMap`

A `RangeMap` intended for:

```text
Int
Date
DateTime
version values
other ordered domains
```

cannot simply reuse current `Range` without either:

- breaking/generalizing `Range` substantially; or
- introducing a generic interval abstraction.

This report recommends the second approach.

---

## 3.6 `Bytes` already occupies one fixed mutable buffer niche

`Bytes` is a fixed-length mutable byte collection with:

- indexed reads/writes;
- fill;
- zeroize;
- copy;
- slice-copy;
- UTF-8 conversion;
- iteration.

### Consequence for generic `Buffer<T>`

`Buffer<T>` should **not** merely mean “mutable fixed-length storage,” because:

- `Bytes` already serves that role for octets;
- `Array<T,N>` will serve the generic fixed-extent role.

`Buffer<T>` needs a different contract: **runtime capacity plus an initialized prefix**, intended for controlled construction/storage.

---

# 4. Syntax and semantic verification

## 4.1 Constructor syntax

The current accepted constructor model allows ordinary named constructors:

```telos
@constructor
new(...)
```

and also:

```telos
@constructor
call(...)
```

`examples/symbolic/src/base.ph` contains a live example:

```telos
class Var is Expr {
  @constructor
  call(_ symbol) {
    ...
  }
}
```

Therefore the user’s preferred constructor style is valid:

```telos
@constructor
call(...)
```

However, `new` remains valid and is still common throughout Universe code. This collection design should use `call` where function-like construction is desirable, without falsely claiming that Telos requires it universally.

`@constructor` is already class-side by semantic elaboration. Combining `@class @constructor` is invalid per TDR-0051.

---

## 4.2 Mapping entry syntax

The previously shown:

```text
Bijection("TR" => "Türkiye")
```

is inadmissible and should not be used.

Current mapping-related source uses:

- map literal syntax;
- labeled method arguments;
- labeled tuple entries `(key: K, value: V)`;
- ordinary object/data construction.

For explicit association values, the modern recommended form is a generic immutable `data` value:

```telos
data Entry<K, V>(
  _ key: K,
  _ value: V,
)
```

The accepted `data` semantics make this:

- nominal;
- transparent;
- immutable;
- value-oriented;
- generic;
- structurally suitable for static values and compiler optimization.

Construction is then ordinary data construction:

```telos
Entry("TR", "Türkiye")
```

rather than new argument grammar.

### Existing `Entry` cleanup

`map.ph` already contains:

```telos
class Entry {
  mut _key
  mut _value
  ...
}
```

described as immutable.

That discrepancy should be resolved instead of introducing a second unrelated `Entry` symbol.

Recommended migration:

```text
legacy nongeneric class Entry
        ↓
generic immutable data Entry<K,V>
```

Map iteration itself does not need to migrate immediately: its existing labeled tuple `(key: K, value: V)` is already a valid, typed iteration shape.

---

## 4.3 Binding semantics

Current normative rules are:

```telos
let x = runtimeExpression
let mut y = runtimeExpression
const z = staticExpression
```

with:

```text
let      runtime binding, not rebindable
let mut  runtime binding, rebindable
const    static/compile-time immutable value
```

Therefore:

```telos
let xs = List<Int>.new()
xs.append(1)
```

can still be conceptually valid because mutation of the referenced collection is not the same as rebinding `xs`.

The same distinction must apply to runtime `Array<T,N>`.

The collection specification should say:

> `Array<T,N>` is a mutable runtime object with fixed extent.

It should **not** say:

> `let` means mutable Array.

---

## 4.4 Static generic parameter syntax

The current type-level-domain specification explicitly permits:

```telos
<T: Type>
<N: Int>
```

for static value parameters.

Therefore this intended type form is aligned with the normative model:

```telos
Array<T, N>
```

where `N` is an `Int` static argument.

### Important implementation gate

Repository searches found normative/parser support for `<N: Int>` but little live collection/runtime proof that a native generic class with a static-value generic parameter is fully operational through:

- type application;
- inference;
- exact type identity;
- substitution;
- reflection;
- imported metadata;
- runtime lowering.

Before implementing `Array<T,N>`, add a narrow proof fixture for a nominal runtime type parameterized by a static `Int`.

Do not discover this limitation halfway through native Array implementation.

---

# 5. Existing traits and protocols: what is already present

## 5.1 `Iterator` and `Iterable` already exist

The canonical file is:

```text
phalcom-core/core/universe/src/collections/iterator_laws.ph
```

`Iterator` has associated:

```text
Item
```

and default combinators including:

```text
map
filter
filterMap
flatMap
flatten
chain
skip
skipWhile
take
takeWhile
mapWhile
inspect
zip
```

`Iterable` has associated:

```text
Item
Iterator
```

and default operations including:

```text
iter
indexed
each
contains
count
find
index
any
all
none
nth
last
fold
reduce
join
group
partition
toSet
toMap
toList
```

### Requirement

Every new iterable collection should conform to these exact canonical traits.

No new `Iterator` or `Iterable` abstraction belongs in this slice.

---

## 5.2 `Collection` is a nominal superclass, not a trait

Current source:

```telos
native class Collection is Object {}
```

Do not introduce a trait also called `Collection`.

If a new behavioral trait is needed, it must use a distinct name and solve a real generic-programming need.

---

## 5.3 `Hashable`, `Comparable`, and related traits are not active canonical Universe protocols

A future placeholder file contains ideas such as:

```text
Equatable
Comparable
Hashable
From
TryFrom
```

but those are not the live generic contracts used by Map/Set today.

Current hash collections instead rely on ordinary Telos sends:

```text
hash
==
```

and the runtime re-enters the VM to execute them.

### Requirement

Do not write the new collection APIs as though this currently works:

```telos
<K: Hashable>
<K: TotalOrder>
```

unless the implementation tranche explicitly activates and integrates those traits first.

For the first implementation:

- hash-based structures should use the existing `hash` / `==` protocol;
- ordered structures should use the existing comparison protocol;
- static trait constraints can be introduced later as a separate type-system/stdlib improvement.

---

# 6. Recommended minimal new shared abstractions

The repository does reveal several genuine missing foundations.

## 6.1 `Entry<K,V>` — required

Recommended:

```telos
data Entry<K, V>(
  _ key: K,
  _ value: V,
)
```

Requirements:

- generic;
- immutable;
- value-oriented;
- positional constructor;
- `.key` and `.value` component access;
- usable by `Bijection`, `IndexedDict`, `RangeMap`, conversion utilities, and builders.

Do not require all existing `Map` iteration APIs to migrate in the same patch.

---

## 6.2 `Interval<T>` — required for general `RangeMap`

Recommended semantic role:

> An immutable generic interval descriptor over an ordered domain, with no implied iteration or successor operation.

This avoids making current numeric `Range` carry incompatible responsibilities.

A first version should support:

```text
bounded lower
bounded/unbounded upper
upper inclusive/exclusive
lower inclusive by default
containment
overlap
intersection
adjacency
```

A more general lower-inclusive/lower-exclusive model can be added if real consumers require it.

### Why not simply make `Range<T>` generic now?

Current `Range` is deeply integrated with:

- numeric `size`;
- integral iteration;
- `+ 1`;
- list slicing;
- range literal semantics.

Generic interval containment and progression are separate abstractions. Combining them now would enlarge the collection tranche into a Range redesign.

`Interval<T>` keeps `Range` stable and gives `RangeMap` the correct domain object.

---

## 6.3 Internal persistent hash-map substrate — required

A persistent immutable `Bag` described as “basically a PersistentMap” cannot be implemented efficiently with the current mutable `MapObject`.

The repository contains no established HAMT/persistent-map implementation.

Required internal substrate:

```text
PersistentHashMapStorage<K,V>
```

or an equivalent VM-internal structure supporting:

- immutable roots;
- structural sharing;
- lookup by Telos `hash` + `==`;
- insert/update returning a new root;
- remove returning a new root;
- traversal;
- precise GC tracing of keys/values;
- no Rust borrow held across re-entrant Telos hash/equality sends;
- deterministic-enough traversal for reproducible behavior where required.

The public API does **not** have to expose a `PersistentMap<K,V>` class immediately. The storage layer can remain internal and be reused by:

```text
Bag
Bijection
future persistent maps/sets
```

### Implementation choice

Two implementation tiers are possible:

**Tier A — semantic-first copy-on-write map**

- clone the affected map structure on update;
- easier to implement;
- persistent semantics correct;
- O(n) update cost.

**Tier B — real HAMT / structurally shared trie**

- expected O(log32 n)-style path-copy update;
- more implementation complexity;
- appropriate if persistence is intended as a first-class performance feature.

Because `Bag` is explicitly requested as persistent, this report recommends designing the representation boundary so Tier A can be replaced by Tier B without source-visible change, while preferring Tier B if implementation time allows.

---

## 6.4 Persistent ordered/interval-tree substrate — required for efficient `RangeMap`

`RangeMap` has a different indexing requirement from a hash map.

It needs ordered lookup by interval boundary and efficient point containment.

Recommended internal structure:

```text
PersistentIntervalTree<K,V>
```

implemented using a persistent balanced search tree keyed by lower boundary, optionally augmented with upper-bound metadata if range-overlap queries are included.

Required operations:

```text
point lookup
predecessor/lower-bound search
overlap search
strict insert
delete
overlay/split
ordered traversal
```

A linear `List<Entry<Interval<K>,V>>` can prove semantics, but should not be called the final `RangeMap` implementation if logarithmic lookup is part of the intended abstraction.

---

## 6.5 Static homogeneous sequence value — required for `const Array` and static `Vector`

`StaticValueData` currently has no homogeneous sequence node.

A reusable semantic node is preferable to one-off `StaticArray` machinery.

Conceptually:

```text
StaticSequence {
    result_type: TypeId
    elements: [StaticValueRef]
}
```

with an interned backing analogous to Pack/Tuple storage.

This can support closed static values of:

```text
Array<T,N>
Vector<T>
possibly future immutable sequence types
```

without conflating their runtime classifier semantics.

The exact node name/layout is an implementation detail, but the static-value model should be general enough that every new homogeneous collection does not add another bespoke arena.

---

# 7. `Array<T,N>` requirements

## 7.1 Semantic contract

`Array<T,N>` should mean:

> A homogeneous, contiguous, fixed-extent runtime collection containing exactly `N` elements of `T`.

Core invariants:

```text
size == N
N >= 0
element domain is T
element replacement is permitted at runtime
structural resize is impossible
```

No operation equivalent to:

```text
append
remove
insert
resize
```

belongs on `Array`.

Element replacement does:

```text
Array<T,N> -> Array<T,N>
```

without changing classifier identity.

---

## 7.2 Runtime representation

Recommended native representation:

```text
ArrayObject
    elements: Box<[Value]>
    frozen/static-materialized flag only if required by const materialization
```

The fixed boxed slice gives representation-level enforcement of non-resizability.

A `Vec<Value>` whose public API simply omits growth operations is weaker because later implementation mistakes can accidentally introduce resize semantics. The `TupleObject` precedent demonstrates the value of choosing a representation that enforces the invariant.

### Mutability difference from Tuple

`Tuple` exposes immutable product storage.

`Array` needs indexed write access:

```text
rawAt
rawSet
rawSize
```

but no growth primitive.

---

## 7.3 Type-level extent

Before native implementation, prove:

```telos
Array<T, N: Int>
```

through a focused semantic fixture covering:

1. formation of `Array<Int, 3>`;
2. rejection of non-`Int` `N`;
3. imported/re-exported type identity;
4. substitution through generic functions;
5. equality of canonical applications;
6. distinctness of `Array<Int,3>` and `Array<Int,4>`;
7. preservation through reflection/materialization metadata where applicable.

The type checker must reject negative extents at the earliest static point.

If the generic-domain system currently accepts all `Int` but cannot express `N >= 0` declaratively, enforce the invariant in Array type formation.

---

## 7.4 Construction

Do not invent `new`.

The user-preferred callable constructor can be used once exact arity validation is available:

```telos
@constructor
call(...)
```

However, variadic construction must establish:

```text
number of supplied elements == N
```

ideally statically.

This interacts with recent exact rest-shape/Pack work and should be deliberately wired to that machinery rather than reimplemented.

Alternative factories useful independently of variadic arity inference:

```text
filled(value)
generate(block)
from(iterable) -> Result<Array<T,N>, SizeMismatchError>
```

Exact spelling should follow actual selector conventions when implementation begins.

---

## 7.5 Literal semantics are a separate decision

Current:

```telos
[1, 2, 3]
```

is a `List`.

Therefore this implementation must **not** silently make it:

```text
Array<Int,3>
```

Changing that requires:

- superseding/amending TDR-0025;
- amending TDR-0028;
- parser/AST/compiler changes;
- inference/contextual-literal design;
- migration of all fixtures and packages expecting List;
- explicit empty-literal inference rules.

Recommended first implementation:

```text
Array exists independently of bracket literal inference.
```

A later dedicated language-design decision can choose whether brackets become contextually polymorphic or whether another syntax is introduced.

---

## 7.6 `const Array` is not solved by ordinary runtime Array

This is a major architecture gate.

A static constant is an immutable semantic value. A runtime `Array` is intentionally mutable.

If a static Array is materialized into an ordinary mutable runtime `ArrayObject` and aliased into user code, ordinary mutating methods could mutate the supposedly constant value.

The implementation must choose one of these policies:

### Policy A — frozen materialized Array

`ArrayObject` can carry a frozen bit for statically materialized constants.

Mutators reject writes to a frozen instance.

Pros:
- preserves `Array<T,N>` classifier;
- repeated uses can share one materialized object;
- matches the user’s “const Array versus runtime mutable Array” distinction.

Cons:
- mutability becomes per-instance state;
- Map/set key admissibility and hashing rules become more complex;
- mutation failures become partly runtime.

### Policy B — fresh mutable copy at runtime escape

Every runtime use that needs an Array gets a fresh mutable materialization.

Pros:
- ordinary Array stays uniformly mutable.

Cons:
- identity/repeated-use semantics become surprising;
- potentially expensive;
- “constant” no longer corresponds to one runtime object.

### Policy C — prohibit mutable runtime escape until capabilities exist

Static Array can be evaluated and inspected statically but cannot materialize into ordinary runtime `Array<T,N>` until the language has a read-only/frozen capability boundary.

Pros:
- semantically strongest.

Cons:
- delays a useful feature.

### Recommendation

For the first complete implementation, prefer **Policy A** if `const Array` must cross into runtime immediately, with explicit tests proving that mutators reject frozen materializations.

Do not pretend compile-time `const` provenance alone can enforce deep immutability after arbitrary parameter passing: the current type surface does not encode mutable versus read-only parameter capabilities.

---

# 8. `Vector<T>` requirements

## 8.1 Merge with C4 P14

The existing P14 proposal describes:

> an owned immutable homogeneous indexed snapshot with O(1) indexed reads and a dedicated heap sequence representation.

That is the same semantic niche as the chosen `Vector<T>`.

### Requirement

Amend P14 to implement:

```text
Vector<T>
```

instead of separately introducing:

```text
ImmutableSequence<T>
```

This avoids permanent duplication.

`Vector<T>` can satisfy both:

- general immutable sequence use;
- parser source snapshot requirements;
- later mathematical vector behavior.

---

## 8.2 Semantic contract

`Vector<T>` should be:

> An immutable, owned, ordered, homogeneous, finite indexed value sequence.

Unlike `Array<T,N>`:

- its size is runtime data, not a static type parameter;
- it cannot be mutated in place.

Unlike `List<T>`:

- no structural or element mutation occurs after construction.

Unlike `Tuple`:

- it is homogeneous and sequence-oriented rather than a heterogeneous structural product.

---

## 8.3 Representation

Recommended:

```text
VectorObject
    elements: Arc<[Value]> or Box<[Value]>
```

An `Arc` backing is attractive if:

- slicing;
- freeze-from-buffer;
- cheap cloned handles;
- later persistent-like transformations

can exploit sharing.

A boxed slice is simpler and remains valid if sharing is not initially required.

No persistent-vector complexity contract should be promised in v1.

Operations such as:

```text
with(index, value)
appended(value)
concat(other)
```

may allocate/copy.

---

## 8.4 Generic vector behavior versus mathematics

`Vector<T>` should support arbitrary `T`.

The base collection layer owns:

```text
size
at/get
iterator
slice
map/collect
concat
reversed
toList
```

Mathematical operations belong in later constrained extensions:

```text
vector addition
scalar multiplication
dot product
norm
distance
Hadamard operations
```

Do not require the collection implementation slice to solve `Real`, `Field`, numeric algebra, or dimension-indexed linear algebra.

The mathematical library can add behavior when `T` satisfies appropriate algebraic contracts.

---

## 8.5 Equality and hashing

Because Vector is a value collection, recommended equality is order-sensitive structural equality.

Hashing requires care because Telos currently lacks a live static `Hashable` capability and allows arbitrary `T`.

At minimum:

```text
equal Vectors must hash equally whenever their elements obey the ordinary Telos hash/equality law
```

The implementation must also account for the existing broader issue: immutable containers can contain references to mutable objects whose equality/hash behavior may not be stable.

This is already relevant to Tuple; Vector should not invent a separate rule.

Recommended v1 policy:

- structural equality;
- structural hash using element `hash`;
- Map/Set key admissibility follows the existing collection key policy;
- add a dedicated regression test for nested mutable collection elements and document the result;
- file a broader hashability-law issue if the existing Tuple semantics are already unsound for such nesting.

Do not introduce a half-integrated `Hashable` trait only for Vector.

---

# 9. `Slice<T>` requirements

## 9.1 Semantic contract

A v1 `Slice<T>` should be:

> A read-only zero-copy view over a contiguous indexed subrange of another collection.

Core properties:

```text
owned storage: no
size: runtime
resize: no
element mutation through Slice: no in v1
indexing: O(1)
view retains backing owner strongly
```

The initial read-only design avoids inventing `MutableSlice<T>` before Telos has a coherent general read/write borrowing/capability model.

---

## 9.2 Backing sources

Safe initial sources:

```text
Array<T,N>
Vector<T>
Buffer<T>   only under the monotonic-prefix Buffer contract in §10
```

Do **not** initially expose a live zero-copy `List.sliceView` because `List` supports arbitrary structural replacement/removal, making the meaning of an outstanding positional view ambiguous.

Current `List` range slicing already returns a fresh `List`, which can remain unchanged.

---

## 9.3 Representation

Avoid storing raw native pointers into backing buffers.

A robust representation stores:

```text
backing owner Value/ObjRef
start index
length
backing kind / adapter identity
```

Every access resolves the backing owner through the heap.

This avoids dangling raw memory even if an underlying Rust allocation moves.

It also makes GC tracing straightforward: the Slice strongly traces the backing collection.

---

## 9.4 Pure-Telos versus native implementation

A source class containing:

```text
_source
_start
_length
```

could provide correct high-level semantics by delegating `at`.

However, without a canonical typed random-access source protocol, preserving static `T` and guaranteed O(1) semantics across different source classes may be awkward.

Two viable architectures:

### A. Native Slice

A `SliceObject` stores backing + start + length and raw access knows the approved backing kinds.

Pros:
- precise O(1) contract;
- type-independent runtime representation;
- easy GC;
- no public helper trait required.

### B. Source-level Slice plus a new `IndexedSequence` trait

Pros:
- less native surface;
- dogfoods traits.

Cons:
- expands public protocol design;
- requires confirming generic associated-type typing for all sources;
- may turn a storage implementation task into a trait-system tranche.

### Recommendation

Use a **native backing descriptor with a thin `.ph` public protocol** for v1, matching the established native-collection pattern.

---

# 10. `Buffer<T>` requirements

## 10.1 Semantic contract

To avoid duplication with List, Array, and Bytes:

> `Buffer<T>` is owned mutable contiguous storage with a runtime **logical capacity** and a monotonically initialized prefix.

State invariant:

```text
0 <= size <= capacity
```

Recommended v1 structural rules:

- `push` initializes the next slot if capacity remains;
- `set` modifies an already initialized slot;
- no implicit capacity growth;
- no middle insertion/removal;
- no shrinking/uninitializing operations in v1;
- `freeze` produces a `Vector<T>`;
- capacity is runtime data, not part of the type.

The monotonic-prefix restriction is valuable because it gives outstanding Slice views stable bounds.

---

## 10.2 Representation

No unsafe uninitialized `Value` storage is required.

Use:

```text
logical_capacity: usize
elements: Vec<Value>
```

with:

```text
elements.len() <= logical_capacity
```

Reserve host capacity for efficiency, but **do not expose Rust Vec capacity as semantic capacity** because the allocator may reserve more than requested.

Safe reads only address:

```text
0 <= i < elements.len()
```

Never make capacity imply initialized readable values.

---

## 10.3 Buffer versus Bytes

`Bytes` remains the specialized octet type with:

- compact `u8` storage;
- encoding/decoding;
- zeroization;
- byte-oriented operations.

`Buffer<T>` stores general Telos `Value`s.

Do not replace Bytes with `Buffer<Int>`.

---

# 11. `Bag<T>` and `MutableBag<T>` requirements

## 11.1 Mathematical model

For a bag `B`, define multiplicity:

```text
m_B(x) ∈ Int
m_B(x) >= 0
```

Only positive multiplicities are stored.

Telos currently has no canonical `Nat` type, so v1 should use `Int` with the invariant:

```text
stored count > 0
```

rather than introducing `Nat` merely for Bag.

Expose two cardinalities:

```text
size / count          total multiplicity
distinctCount         number of distinct values
```

Exact selector naming should follow current stdlib conventions during implementation.

---

## 11.2 `Bag<T>` persistence

`Bag<T>` is explicitly persistent immutable.

Conceptual backing:

```text
PersistentHashMapStorage<T, Int>
cached_total_count: Int
```

Operations return new bags while the original remains unchanged:

```text
added
removed
union
sum
intersection
difference
```

Multiplicity algebra:

```text
sum          A(x) + B(x)
union        max(A(x), B(x))
intersection min(A(x), B(x))
difference   max(A(x) - B(x), 0)
```

These names must be documented because multiset “union” terminology varies across ecosystems.

---

## 11.3 Iteration

Recommended canonical `Iterable<Item = T>` behavior:

- iteration yields each occurrence according to multiplicity.

Also expose a distinct entry traversal yielding:

```text
Entry<T, Int>
```

for algorithms that should not repeat values `count` times.

Bag iteration order is **not semantic** and must not participate in equality.

---

## 11.4 `MutableBag<T>`

`MutableBag<T>` has the same multiset laws but mutates in place.

A practical first implementation can wrap/reuse the existing mutable Map semantics:

```text
Map<T, Int>
cached_total_count
```

provided all updates maintain:

```text
no stored zero/negative counts
cached total == sum of stored counts
```

`MutableBag` does not require a new persistent map.

Conversions:

```text
Bag -> MutableBag
MutableBag -> Bag
```

should be explicit.

If `freeze` can reuse storage safely, that is an optimization; semantics must not depend on it.

---

# 12. `IndexedDict<K,V>` requirements and redundancy gate

## 12.1 Current short definition is insufficient

Current `Map` already:

- preserves insertion order;
- has O(1)-style indexed backing access;
- exposes ordered keys/values/entries views.

Therefore:

> “mapping that keeps insertion order and allows access by index”

does not justify a separate class.

---

## 12.2 Required stronger contract

If `IndexedDict` is retained, define it as:

> A mapping whose entry order is **semantic** and directly mutable by position.

Required API capabilities:

```text
lookup by key
entryAt(index)
keyAt(index)
valueAt(index)
indexOfKey(key)

insert new entry at position
removeAt(index)
move key/entry to position
replace value without moving position
```

Required equality:

```text
same entries in same order
```

This intentionally differs from current `Map`, whose equality ignores insertion order.

For order-insensitive comparison, a separate operation may compare mapping equivalence.

---

## 12.3 Index syntax

Do not overload one `[_]` surface for both key and positional addressing.

For:

```text
IndexedDict<Int, V>
```

`dict[3]` must unambiguously remain key lookup if ordinary mapping subscript is supported.

Positional access should use named selectors such as:

```text
entryAt
keyAt
valueAt
```

matching established Telos selector style.

---

## 12.4 Representation

Two choices:

### A. New native class reusing `MapObject`

`Object::IndexedDict(Box<MapObject>)` can reuse:

- entry vector;
- hash-bucket index;
- re-entrant equality/hash machinery.

Additional primitives provide:

- locate slot;
- insert at slot;
- remove at slot;
- move slot.

This is the efficient recommendation if `indexOfKey` should be O(1) expected.

### B. Source wrapper over Map plus List

Simpler, but positional changes require coordinated mutation and potentially O(n) repair of indices.

### Recommendation

If the stronger IndexedDict contract is accepted, reuse/generalize `MapObject` at the native representation level rather than maintaining three independent source collections.

If the stronger contract is **not** accepted, do not implement IndexedDict; add public `Map.entryAt`/`keyAt`/`valueAt` instead.

---

# 13. `SetList<T>` requirements and redundancy gate

## 13.1 Existing Set overlap

Current `Set` already provides:

```text
uniqueness
insertion order
indexed `at`
fast membership
order-insensitive equality
```

Therefore SetList needs a stronger sequence contract.

---

## 13.2 Required semantic distinction

Recommended definition:

> A mutable ordered sequence of unique elements whose positional order is part of value equality.

Required operations:

```text
contains(value)
indexOf(value)

at(index)
insert(value, at: index)
remove(value)
removeAt(index)
move(value, to: index)
append/add if absent
```

Required invariant:

```text
i != j => values[i] != values[j]
```

Required equality:

```text
same elements in the same order
```

Set-style order-insensitive comparison can be an explicit secondary operation.

---

## 13.3 Construction and duplicate handling

Do not silently discard duplicate constructor inputs.

For a strict constructor:

```text
duplicate input => DuplicateElementError
```

Provide an explicit deduplicating factory if desired:

```text
unique(iterable)
```

preserving first occurrence.

That makes data loss intentional rather than implicit.

---

## 13.4 Representation

The existing `MapObject`/Set backing already has most of the required machinery.

A `SetList` native class can reuse/generalize the same ordered-hash representation and add:

```text
indexOf
insertAt
removeAt
move
```

This is preferable to duplicating List + Set synchronization logic in source.

If these operations are not desired, do not add SetList: current Set is already the ordered unique collection.

---

# 14. `Bijection<K,V>` requirements

## 14.1 Invariant

A bijection stores unique associations in both directions:

```text
forward[k] = v  iff  reverse[v] = k
```

No two distinct keys may map to equal values.

---

## 14.2 Recommended mutability model

The user explicitly required persistence for Bag, but did not restate mutability for Bijection.

Two models are possible:

### Mutable Bijection

- easier to build over two current `Map`s;
- updates must atomically preserve both maps;
- `.inverse` as a live mutable view introduces aliasing concerns.

### Persistent immutable Bijection

- naturally mathematical/value-oriented;
- `.inverse` can be O(1) by swapping two persistent roots;
- shares the persistent hash-map substrate already required for Bag;
- update failure cannot leave the object half-mutated.

### Recommendation

Implement **immutable persistent `Bijection<K,V>`** unless a concrete mutable use case requires `MutableBijection` later.

---

## 14.3 Key requirements

Because both sides act as keys:

```text
K must be admissible as a hash-map key
V must be admissible as a hash-map key
```

The current language does not express that with a live `Hashable` trait constraint, so v1 enforcement follows the existing Map key protocol and runtime key admissibility rules.

Do not claim compile-time `K: Hashable, V: Hashable` until that trait is actually active.

---

## 14.4 Conflict handling

Conflicts must be explicit.

Given:

```text
a -> 1
b -> 2
```

adding:

```text
c -> 2
```

must not silently remove `b`.

Define dedicated conflict information distinguishing:

```text
existing key conflict
existing value conflict
both/same association
```

A new source-level error can be introduced:

```text
BijectionConflictError
```

or an equivalent typed data error/result.

---

# 15. `RangeMap<K,V>` requirements

## 15.1 Semantic model

`RangeMap` is not:

```text
Map<Interval<K>,V>
```

with ordinary exact-key lookup.

Its primary lookup is:

```text
point K -> V whose stored interval contains point
```

Core invariant:

```text
stored intervals do not overlap
```

This guarantees at most one answer per point.

---

## 15.2 Interval dependency

Use `Interval<K>`, not current numeric `Range`.

This gives RangeMap a domain independent of stepping/iteration.

Current Range can later be refactored to reuse Interval internally if desirable, but that should not block RangeMap.

---

## 15.3 Operations

At minimum:

```text
get(point) -> Option<V>
contains(point)
entries / intervals
strict insertion
removal
```

Two insertion modes are useful and semantically distinct:

### Strict insertion

Reject any overlap.

### Overlay assignment

Assign a value to an interval while splitting/truncating prior intervals.

Example:

```text
existing: [0,100) -> A
overlay:  [30,50) -> B

result:
[0,30)   -> A
[30,50)  -> B
[50,100) -> A
```

Do not make those two behaviors one ambiguous `insert`.

---

## 15.4 Persistence

An immutable persistent RangeMap is recommended because:

- range configurations are naturally versioned;
- interval edits are structural transformations;
- it shares the persistent-value philosophy of Bag/Bijection.

It does **not** share a hash-map representation; use the ordered interval tree foundation.

---

## 15.5 Equality

Two options:

1. representation equality: exact same stored intervals/values;
2. extensional equality: same point-to-value function after normalization.

Extensional equality requires canonical coalescing or expensive comparison.

### Recommendation

For v1, use canonical interval-entry equality:

```text
same ordered interval/value entries
```

and do not automatically coalesce adjacent equal-valued intervals unless the operation explicitly normalizes.

This keeps equality predictable and implementation tractable.

---

# 16. Static evaluation and materialization requirements

## 16.1 Current state

`StaticValueData` supports:

```text
Unit
Bool
Char
Int
Float
String
Symbol
Type
StaticTerm
Tuple
Record
Data
Variant
Pack
```

No Array, Vector, Bag, Map, or generic sequence node exists.

`materialization.rs` currently marks:

```text
Tuple
Record
Data
Variant
Pack
```

as `DeferredStructural`.

This means even current structural static materialization is not fully closed.

---

## 16.2 Required extension

A static homogeneous sequence substrate should support:

```text
exact result TypeId
ordered element StaticValueRefs
canonical interning
type derivation/validation
metadata serialization
materialization descriptor generation
```

For `Array<T,N>`, validation must prove:

```text
number of elements == N
all elements assignable to T
```

For `Vector<T>`:

```text
all elements assignable to T
```

with runtime cardinality stored in the static value.

---

## 16.3 C4 scheduling implication

Do not couple the first runtime collection implementation to unfinished C4 P8 structural materialization.

Recommended staging:

```text
Stage 1:
    runtime collection type exists and is tested

Stage 2:
    static evaluator can form/intern closed static collection values

Stage 3:
    runtime materialization of those static values lands with the C4
    structural materialization work
```

This keeps collection runtime work independently testable.

---

# 17. Hash/equality/order architecture gaps exposed by the new collections

The new designs expose an existing architectural weakness: the Universe gives every object `hash` and `==`, but the type system has no active canonical trait expressing:

```text
stable value equality
stable hash consistency
total ordering
```

`Map`/`Set` therefore enforce admissibility partly at runtime and partly by convention.

This matters for:

```text
Bag keys
Bijection keys and reverse values
Vector hashing
RangeMap ordered keys
Interval comparisons
persistent map/tree nodes
```

## 17.1 Do not solve this incidentally

Activating `Hashable`/`Comparable` is a meaningful stdlib/type-system decision affecting existing Map, Set, Tuple, data values, user classes, and generic algorithms.

It should not be smuggled into a collection patch.

## 17.2 Requirements for the first slice

- use existing `hash`/`==` VM re-entry for hash structures;
- use existing comparison sends for ordered intervals;
- preserve MapObject’s no-borrow-across-reentrant-send discipline;
- add focused tests for mutable/reentrant keys;
- document key-admissibility behavior.

## 17.3 Follow-on recommendation

Create a dedicated foundational proposal for:

```text
Equatable
Hashable / StableHashable
Comparable / TotalOrder
```

and migrate Map/Set/new collections together once semantics are ratified.

---

# 18. Recommended public trait additions

The collection tranche itself needs fewer new traits than initially assumed.

## 18.1 Required now

None beyond existing:

```text
Iterator
Iterable
```

Every new sequence/container that iterates should conform to them.

## 18.2 Potentially useful later: indexed sequence capability

There is a real shared behavioral shape across:

```text
List
Array
Vector
Slice
Buffer
SetList
```

A future trait can describe:

```text
finite ordered indexed sequence
associated Item
size
at/get
iterator
```

Possible name:

```text
IndexedSequence
```

This is preferable to a vague `Sequence` if random/indexed access is specifically guaranteed.

However, its exact syntax should be compiled/proven before being made part of this collection implementation plan.

## 18.3 Defer `Contiguous`

A public `Contiguous<T>` trait is premature because Telos currently has no safe public pointer/span abstraction whose semantics such a trait would expose.

`Slice` construction can initially be implemented by explicit methods on approved source types.

## 18.4 Defer `Mapping`

`Map`, `Bijection`, `IndexedDict`, and `RangeMap` have different lookup semantics:

```text
Map           exact key
Bijection     exact key + reverse value
IndexedDict   key + position
RangeMap      point contained in interval
```

A common Mapping trait should emerge from proven common algorithms, not from taxonomy aesthetics.

---

# 19. Error/value requirements

Existing errors include:

```text
ArgumentError
KeyError
IndexError
SliceError
DuplicateKeyError
```

New semantic failures justify specific errors where generic `ArgumentError` would lose important information.

Recommended additions:

```text
SizeMismatchError          Array construction from wrong cardinality
CapacityError              Buffer capacity exhausted
DuplicateElementError      strict SetList construction/insertion
BijectionConflictError     key/value uniqueness violation
RangeOverlapError          strict RangeMap insertion overlap
InvalidIntervalError       malformed interval bounds
```

Use existing `IndexError` for ordinary invalid positional indexing.

Do not proliferate errors for conditions already represented canonically by `Option` or `Result`.

---

# 20. Native versus source implementation matrix

| Type/Foundation | Recommended first implementation | Reason |
|---|---|---|
| `Entry<K,V>` | source `data` | immutable transparent value; no native storage need |
| `Interval<T>` | source `data` + methods | immutable descriptor; behavior in Telos |
| `Array<T,N>` | native backing + `.ph` protocol | fixed mutable Value storage, static extent invariant |
| `Vector<T>` | native backing + `.ph` protocol | immutable homogeneous storage; replaces P14 snapshot type |
| `Slice<T>` | native view descriptor + `.ph` protocol | zero-copy multi-backing O(1) view |
| `Buffer<T>` | native backing + `.ph` protocol | capacity/init-prefix semantics |
| persistent hash map | VM-internal Rust structure | structural sharing + reentrant Telos hash/equality |
| `Bag<T>` | native/source façade over persistent hash storage | persistent multiset |
| `MutableBag<T>` | source or native façade over current Map | current Map already supplies mutable hashing |
| `Bijection<K,V>` | façade over two persistent hash roots | inverse sharing and atomic invariant |
| persistent interval tree | VM-internal Rust structure | ordered persistent point lookup |
| `RangeMap<K,V>` | façade over persistent interval tree | logarithmic point/range operations |
| `IndexedDict<K,V>` | native class reusing/generalizing MapObject **if retained** | order semantic + O(1) key/index coupling |
| `SetList<T>` | native class reusing/generalizing MapObject **if retained** | unique ordered positional mutation |

The general repository pattern should remain:

```text
native representation only for underivable storage/complexity guarantees
        +
public algorithms/protocol authored in Telos source
```

---

# 21. Frozen primitive-floor implications

Existing collection architecture follows a strict rule: new native storage operations require explicit raw native primitives only when their behavior is not derivable in Telos.

Therefore every native type needs a primitive-floor census and, where required by current governance, an accepted floor amendment.

Likely minimal raw surfaces:

## Array

```text
construct/factory bridge
rawSize
rawAt
rawSet
```

## Vector

```text
construct/freeze bridge
rawSize
rawAt
rawSlice/share primitive only if needed
```

## Slice

```text
rawSize
rawAt
backing-aware construction
```

## Buffer

```text
construct(capacity)
rawSize
rawCapacity
rawAt
rawSet
rawPush
freeze bridge
```

Persistent Bag/RangeMap/Bijection should expose only the raw operations needed to access/update their persistent representation; algebraic combinators stay in `.ph` when efficiently expressible.

Do not add one primitive for every convenience method.

---

# 22. GC and runtime safety requirements

Every new object containing Telos `Value`s must participate correctly in tracing.

Mandatory proofs/tests:

1. **Array**
   - traces every element;
   - mutation writes preserve liveness;
   - frozen/static materialization does not bypass tracing.

2. **Vector**
   - traces shared backing exactly once per live object graph semantics;
   - Arc/shared storage cannot retain untraceable raw ObjRefs outside GC ownership rules.

3. **Slice**
   - strongly traces backing owner;
   - does not trace raw interior pointers;
   - slice remains valid across GC relocation/arena activity.

4. **Buffer**
   - traces only initialized elements;
   - unused capacity contains no fake `Value`s requiring tracing.

5. **Persistent hash/tree nodes**
   - every node recursively traces stored keys/values;
   - shared nodes may be reached from multiple roots safely;
   - drop/reclamation must match the VM’s arena/ownership model;
   - no host pointer becomes semantic identity.

6. **Hash re-entry**
   - never retain `&Heap`/`&mut Heap` references across Telos `hash` or `==` sends;
   - preserve MapObject’s existing re-entrant mutation lock discipline.

---

# 23. Complexity contracts

A collection type should exist because its semantics or asymptotics differ, so complexity should be part of the design.

Recommended v1 targets:

| Operation | Target |
|---|---:|
| `Array.at/set` | O(1) |
| `Vector.at` | O(1) |
| `Vector.with/appended` | O(n) allowed in v1 unless shared optimization implemented |
| `Slice.at` | O(1) |
| `Buffer.at/set/push` | O(1) |
| `Bag.countOf` | O(1) expected |
| persistent Bag update | O(log n)-ish/HAMT path copy preferred; O(n) temporary bootstrap must be documented |
| `MutableBag.countOf/update` | O(1) expected |
| `Bijection` lookup either direction | O(1) expected |
| `RangeMap.get(point)` | O(log n) |
| `RangeMap` overlap query | O(log n + k) |
| `IndexedDict` key lookup | O(1) expected |
| `IndexedDict.entryAt` | O(1) |
| `IndexedDict.indexOfKey` | O(1) expected if new type retained |
| `SetList.contains/indexOf` | O(1) expected |
| `SetList.at` | O(1) |
| SetList/IndexedDict middle insert/move | O(n) acceptable |

These targets should be testable through representation-aware focused tests/benchmarks rather than only documentation.

---

# 24. Test requirements

## 24.1 Existing harness reuse

`phalcom-core/tests/core/collections/contract.rs` already checks important collection laws.

Extend or factor it rather than creating unrelated test styles.

However, one “sequence contract” is not sufficient for all new collection kinds.

---

## 24.2 Array tests

Required:

```text
formation Array<Int,0>, Array<Int,1>, Array<Int,3>
distinct type identities for different N
negative N rejected
exact size invariant
indexed read/write
out-of-range read/write
no append/remove/resize selectors
iteration
structural equality
copy/conversion behavior
GC retention after element mutation
generic substitution/import/re-export of N
```

Static Array:

```text
closed const formation
static indexing
static equality
materialization
mutation rejection on materialized const
no initializer re-execution
```

---

## 24.3 Vector tests

```text
construction arbitrary T
immutable surface: no set/append-in-place/remove
indexed reads
iterator conformance
structural equality
hash law for ordinary value elements
List.snapshot -> Vector independence
Vector.toList -> fresh mutable copy
slice/view retention
GC of shared backing
P14 parse-source snapshot integration
```

Math extensions later receive their own numerical law tests.

---

## 24.4 Slice tests

```text
zero-length slice
full slice
nested slice
start/end boundaries
invalid ranges
backing GC retention
Array element mutation observed through read-only Slice if live-view semantics chosen
Vector stability
Buffer prefix growth does not invalidate existing in-range slice
List live view unavailable in v1
no mutation selectors on Slice
```

---

## 24.5 Buffer tests

```text
capacity 0
push to exactly capacity
push beyond capacity -> CapacityError/Result
size <= capacity invariant
set only initialized index
read beyond size invalid even when below capacity
no implicit reallocation semantic growth
freeze to Vector preserves values
post-freeze behavior explicitly specified
GC traces initialized prefix
```

---

## 24.6 Bag law tests

For randomly generated small bags:

```text
count == sum multiplicities
distinctCount == number of positive entries
added increments one count
removed saturates or errors exactly as specified
sum commutative
union commutative
intersection commutative
difference semantics
original Bag unchanged after every persistent operation
structural sharing does not leak mutation
equality independent of insertion/build order
hash law where applicable
```

MutableBag additionally tests in-place updates and freeze independence.

---

## 24.7 Bijection law tests

```text
forward[k] = v iff inverse[v] = k
duplicate key conflict
duplicate value conflict
inverse.inverse == original
removal updates both directions
persistent updates leave original unchanged
hash/equality re-entry errors leave structure consistent
GC traces both maps without leaks/dangling refs
```

Property-based testing is particularly valuable here.

---

## 24.8 RangeMap law tests

```text
point outside all intervals -> None
point on every boundary
inclusive/exclusive endpoint semantics
strict overlap rejection
adjacent intervals allowed
overlay split correctness
unbounded endpoint behavior
original persistent map unchanged after update
ordered traversal
date-like/comparable custom object domain
comparison callback/re-entry failure leaves structure valid
```

Use a simple reference model in tests for randomized interval operations.

---

## 24.9 IndexedDict tests

If retained:

```text
key lookup
entryAt/keyAt/valueAt
indexOfKey
replace existing key does not move it
remove + reinsert moves to end
explicit move
middle insertion
order-sensitive equality
order-insensitive sameMapping helper if provided
integer-key ambiguity: dict[3] always means key 3, not positional slot 3
```

---

## 24.10 SetList tests

If retained:

```text
uniqueness
contains
indexOf
at
append absent
duplicate append policy
strict positional duplicate insertion error
removeAt
move
order-sensitive equality
set-equivalence helper
constructor duplicate rejection
explicit deduplication factory preserves first occurrence
```

---

# 25. Recommended implementation sequence

The dependencies argue against implementing all types in one patch.

## Phase 0 — ratify/record semantic deltas

Before code:

1. record the new collection taxonomy;
2. record `Vector` replacing P14 `ImmutableSequence`;
3. decide `const Array` materialization policy;
4. decide whether bracket literals remain List for this tranche;
5. strengthen or cancel IndexedDict/SetList;
6. decide immutable Bijection/RangeMap mutability policy;
7. obtain any native-floor amendments.

---

## Phase 1 — shared source-level values

Implement and test:

```text
Entry<K,V>
Interval<T>
new collection-specific errors
```

No native runtime representation changes required.

---

## Phase 2 — Vector/P14 convergence

Implement `Vector<T>` as the P14 immutable homogeneous snapshot type.

Then migrate:

```text
List.snapshot -> Vector<T>
parse snapshot consumers -> Vector where appropriate
```

This closes an already-identified stdlib deficiency and avoids a duplicate immutable-sequence type.

---

## Phase 3 — runtime Array foundation

First prove static `Int` generic argument formation for a runtime nominal type.

Then add:

```text
Array<T,N>
```

runtime backing and protocol.

Do **not** change `[ ... ]` literal semantics in this phase.

---

## Phase 4 — Buffer and Slice

Implement:

```text
Buffer<T>
Slice<T>
```

with the monotonic Buffer initialized-prefix model and read-only Slice model.

Wire:

```text
Array.slice
Vector.slice
Buffer.slice
Buffer.freeze -> Vector
```

---

## Phase 5 — persistent hash substrate

Build/test the internal persistent hash map.

Critical tests must cover:

```text
Telos hash/== re-entry
structural sharing
GC
collision handling
failed sends
mutation-lock/reentrancy
```

---

## Phase 6 — Bag and Bijection

On the persistent hash substrate:

```text
Bag<T>
Bijection<K,V>
```

Then implement `MutableBag<T>` using current mutable Map infrastructure.

---

## Phase 7 — persistent interval tree and RangeMap

Build:

```text
PersistentIntervalTree
RangeMap<K,V>
```

over `Interval<K>`.

---

## Phase 8 — IndexedDict / SetList conditional tranche

Only execute if the stronger contracts in §§12–13 are accepted.

Prefer sharing/generalizing current `MapObject` rather than creating unrelated ordered-index implementations.

---

## Phase 9 — static sequence evaluation/materialization

Extend:

```text
StaticValueStore
static evaluator
metadata
materialization descriptors
compiler lowering
runtime frozen Array behavior
```

for closed Array/Vector constants.

Coordinate with the active LANG007.C4 lifecycle rather than bypassing it.

---

## Phase 10 — mathematical Vector extensions

After numerical algebra abstractions are stable, add constrained implementations for:

```text
+ / -
scalar multiplication
dot
norm
distance
Hadamard operations
```

This should remain outside core collection storage implementation.

---

# 26. Expected repository touch points

The exact write set depends on which types receive native representation, but likely files include:

```text
phalcom-native-meta/src/universe.rs
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/mod.rs
phalcom-core/src/heap/<new collection>.rs
phalcom-core/src/primitive/<new collection>.rs
phalcom-core/src/primitive/mod.rs
phalcom-core/src/universe/primitives.rs
phalcom-core/src/native/install.rs
phalcom-core/core/universe/src/collections/package.ph
phalcom-core/core/universe/src/collections/<new collection>.ph
phalcom-core/core/universe/src/errors/argument.ph
phalcom-native-surface-gen/...
phalcom-native-surface/...
phalcom-core/tests/core/collections/contract.rs
phalcom-core/tests/fixtures/language/collections/...
```

Static support additionally touches likely authorities such as:

```text
phalcom-semantic/src/static_value/store.rs
phalcom-semantic/src/static_eval/...
phalcom-semantic/src/materialization.rs
phalcom-semantic/src/metadata/...
phalcom-core/src/modules/semantic_lowering.rs
compiler/runtime constant materialization paths
```

`Array<T,N>` may additionally require changes in the type application/static-argument pipeline if the proof fixture reveals gaps.

Do not start those cross-layer edits without first checking the active LANG007.C4/P4 ownership matrix, because C4 is currently `IN_PROGRESS / PARTIAL`.

---

# 27. Collection taxonomy by semantic axes

The following table arranges the resulting vocabulary according to the properties that actually distinguish types.

| Type | Homogeneous | Positional order semantic | Uniqueness | Cardinality | Runtime mutation | Structural resize | Persistence | Ownership | Primary addressing |
|---|---:|---:|---|---|---:|---:|---:|---|---|
| `Pack<*Ts>` | no | yes | no | static shape | no | no | value | owned/structural | structural position/label |
| `Tuple` | not necessarily | yes | no | fixed runtime product shape | no | no | value | owned | position/label |
| `Array<T,N>` | yes | yes | no | exact static `N` | **elements yes** | **no** | no | owned | position |
| `List<T>` | yes | yes | no | runtime | yes | **yes** | no | owned | position |
| `Vector<T>` | yes | yes | no | runtime | **no** | no in-place | not promised | owned | position |
| `Slice<T>` | yes | yes | no | runtime view length | no through v1 Slice | no | no | **view** | position |
| `Buffer<T>` | yes | yes | no | `size <= capacity` | yes | prefix growth only | no | owned | position |
| `Set<T>` | yes | encounter order only | unique | runtime | yes | yes | no | owned | value + encounter position |
| `SetList<T>` | yes | **yes** | unique | runtime | yes | yes | no | owned | value + position |
| `Map<K,V>` | n/a | encounter order only | keys unique | runtime | yes | yes | no | owned | key |
| `IndexedDict<K,V>` | n/a | **yes** | keys unique | runtime | yes | yes | no | owned | key + position |
| `Bag<T>` | yes | no | multiplicity | runtime | no | persistent transform | **yes** | owned/shared | value -> count |
| `MutableBag<T>` | yes | no | multiplicity | runtime | yes | yes | no | owned | value -> count |
| `Bijection<K,V>` | n/a | no | keys + values unique | runtime | no recommended | persistent transform | **yes** | owned/shared | key + reverse value |
| `RangeMap<K,V>` | n/a | domain order | nonoverlapping intervals | runtime | no recommended | persistent transform | **yes** | owned/shared | contained point |
| `Bytes` | octets | yes | no | fixed runtime | yes | no | no | owned | byte position |

---

# 28. Structural equivalences and near-equivalences

These equivalences are useful for implementation and reasoning, but they do **not** mean the public types should collapse when their laws differ.

## Bag

```text
Bag<T>
≈ PersistentMap<T, Int>
+ invariant count > 0
+ cached total multiplicity
```

## MutableBag

```text
MutableBag<T>
≈ Map<T, Int>
+ invariant count > 0
+ cached total multiplicity
```

## Set versus Bag

Mathematically:

```text
Set<T>
≈ Bag<T> with multiplicity restricted to {0,1}
```

Operationally they differ because current Set is mutable and insertion-ordered.

---

## Bijection

```text
Bijection<K,V>
≈ PersistentMap<K,V>
+ PersistentMap<V,K>
+ exact inverse invariant
```

---

## RangeMap

```text
RangeMap<K,V>
≈ persistent ordered map of Interval<K> -> V
+ non-overlap invariant
+ lookup by contained point instead of exact Interval key
```

---

## IndexedDict

```text
IndexedDict<K,V>
≈ Map<K,V>
+ semantic positional order
+ key -> index capability
+ positional mutation
```

Because current Map already has insertion order, only the second group of properties justifies a new type.

---

## SetList

```text
SetList<T>
≈ Set<T>
+ semantic positional order
+ value -> index capability
+ positional insertion/removal/move
```

Again, current Set already has insertion order and indexed read.

---

## Array

```text
Array<T,N>
≈ fixed-length mutable homogeneous sequence
```

It is **not** equivalent to:

```text
Tuple<T,T,...>
```

because Array extent is numeric and homogeneous rather than a heterogeneous positional product shape.

---

## Vector

```text
Vector<T>
≈ the planned P14 immutable homogeneous indexed snapshot
```

It is not a persistent vector unless a later performance/representation decision explicitly says so.

---

## Slice

```text
Slice<T>
≈ (backing owner, offset, length)
```

with read-only O(1) projection into approved backing sources.

---

## Buffer

```text
Buffer<T>
≈ (logical capacity, initialized prefix)
```

not a second List and not a fixed Array.

---

# 29. Explicitly rejected/redundant additions for this tranche

Do not add:

```text
DynamicArray<T>       -> List<T>
MutableVector<T>      -> List<T>
ImmutableSequence<T>  -> merge into Vector<T>
FixedArray<T,N>       -> Array<T,N>
StaticArray<T,N>      -> ambiguous; const is phase, not collection name
ReadonlyArray<T,N>    -> future capability/view problem
Span<T>               -> duplicates Slice<T>
Multiset<T>           -> synonym of Bag<T>
BiMap<K,V>            -> synonym of Bijection<K,V>
OrderedSet<T>         -> overlaps SetList/current Set
OrderedDict<K,V>      -> overlaps current Map/IndexedDict
```

A public `PersistentMap<K,V>` may eventually be useful, but should not be exposed merely because Bag needs an internal persistent hash map.

---

# 30. Architecture decisions still required before an implementation plan

The implementation plan should not begin until the following are explicitly settled.

| ID | Decision | Recommended ruling |
|---|---|---|
| **COLL-N1** | Does `[a,b,c]` remain List during this tranche? | **Yes.** Separate literal redesign later. |
| **COLL-N2** | What happens when `const Array<T,N>` materializes at runtime? | Frozen Array instance, or defer runtime escape. Do not leave unspecified. |
| **COLL-N3** | Does P14 `ImmutableSequence<T>` become `Vector<T>`? | **Yes.** |
| **COLL-N4** | Is `Vector` structurally equal/value-hashed? | **Yes**, subject to existing element hash/equality law; audit nested mutable elements. |
| **COLL-N5** | Is v1 Slice read-only? | **Yes.** |
| **COLL-N6** | Can v1 Slice view List? | **No.** Preserve List copy slicing initially. |
| **COLL-N7** | Is Buffer monotonic-prefix / no shrink? | **Yes** for v1. |
| **COLL-N8** | Is Bag truly structurally persistent? | **Yes**, as requested. |
| **COLL-N9** | Is Bijection immutable persistent? | **Recommended yes.** |
| **COLL-N10** | Is RangeMap immutable persistent? | **Recommended yes.** |
| **COLL-N11** | Does RangeMap use generic `Interval<K>` instead of current `Range`? | **Yes.** |
| **COLL-N12** | Does IndexedDict have semantic order + positional mutation? | Must be **yes**, otherwise omit it and extend Map. |
| **COLL-N13** | Does SetList have order-sensitive equality + positional mutation/index lookup? | Must be **yes**, otherwise current Set already occupies the niche. |
| **COLL-N14** | Do we activate Hashable/Comparable traits now? | **No.** Separate foundational tranche. |
| **COLL-N15** | Do we make mapping iteration globally return Entry<K,V>? | **No for first slice.** Keep existing labeled tuple compatibility. |

---

# 31. Final recommendation

Proceed, but do **not** treat this as “add ten new classes.”

The coherent implementation program is:

1. **Consolidate vocabulary first.**
   - `Vector<T>` replaces planned `ImmutableSequence<T>`.
   - `Entry<K,V>` becomes a real generic immutable `data` value.
   - add `Interval<T>` for generic range-domain semantics.

2. **Implement the storage primitives that genuinely add new semantics.**
   - `Array<T,N>`;
   - `Vector<T>`;
   - `Slice<T>`;
   - `Buffer<T>`.

3. **Build persistence once, then reuse it.**
   - persistent hash substrate;
   - `Bag<T>`;
   - `Bijection<K,V>`;
   - persistent interval tree;
   - `RangeMap<K,V>`.

4. **Keep mutable Bag cheap.**
   - `MutableBag<T>` can reuse current mutable Map architecture.

5. **Do not create redundant hybrid collections.**
   - `IndexedDict` must be stronger than Map;
   - `SetList` must be stronger than Set;
   - otherwise extend the existing types instead.

6. **Keep static evaluation as a deliberate second vertical.**
   - runtime Array/Vector first;
   - general homogeneous static-value node;
   - materialization/frozen semantics coordinated with LANG007.C4.

7. **Do not invent new generic traits unnecessarily.**
   - reuse canonical `Iterator` and `Iterable`;
   - defer Hashable/Comparable/Mapping/Contiguous unless a concrete implementation requirement forces them.

The resulting family has a defensible semantic basis:

```text
Array      fixed extent, mutable elements
List       dynamic extent, mutable structure
Vector     immutable owned sequence
Slice      read-only zero-copy view
Buffer     bounded mutable construction/storage

Set        unordered/set equality, deterministic encounter order
SetList    unique sequence where order itself is semantic

Map        mapping equality, deterministic encounter order
IndexedDict
           mapping + semantic positional order

Bag        persistent multiplicity mapping
MutableBag mutable multiplicity mapping

Bijection  persistent one-to-one bidirectional mapping
RangeMap   persistent interval-domain mapping
```

That structure fits the live Telos runtime far better than adding collection names based only on terminology from other ecosystems.

---

# Appendix A — high-risk implementation invariants

The implementation plan should carry these as non-negotiable invariants.

```text
A1  Array<T,N>.size == N for every runtime value.
A2  Array can replace elements but can never structurally resize.
A3  Static Array materialization cannot be mutated through an alias.
A4  Vector has no in-place mutation path.
A5  Slice stores no raw interior pointer whose validity depends on a Rust allocation address.
A6  Buffer reads only initialized prefix elements.
A7  Buffer never silently exceeds logical capacity.
A8  Bag stores no zero/negative multiplicity.
A9  Persistent updates never mutate a node reachable from an older public root.
A10 Bijection forward and reverse roots are exact inverses after every successful operation.
A11 Bijection failure leaves both roots unchanged.
A12 RangeMap contains no overlapping intervals.
A13 RangeMap point lookup has at most one result.
A14 IndexedDict key identity and positional identity never diverge.
A15 SetList sequence and membership indices never diverge.
A16 No collection holds a Heap borrow across re-entrant Telos hash/equality/comparison sends.
A17 Every Value-bearing native node participates in precise GC tracing.
A18 New collections reuse canonical Iterable/Iterator evidence.
A19 No new class silently changes accepted [ ... ] List-literal semantics.
A20 Native raw primitives remain minimal; convenience behavior stays in Telos source.
```

# Appendix B — repository-grounded corrections to earlier brainstorming

| Earlier brainstorming idea | Repository-grounded correction |
|---|---|
| `[1,2,3]` naturally becomes Array | Today it is normatively and operationally List. Changing it is separate language work. |
| `let` means mutable runtime value | `let` is immutable binding; object interior mutation is separate. |
| Need to add Iterable/Iterator | Already canonical and feature-rich. |
| Need generic Entry from scratch | An Entry class already exists; modernize/replace it with generic immutable data. |
| Map needs insertion ordering for IndexedDict | Map already has insertion order. IndexedDict needs stronger positional semantics. |
| SetList fills the ordered-set gap | Current Set already preserves insertion order and has indexed read. SetList needs stronger sequence semantics. |
| RangeMap can use Range<K> | Current Range is nongeneric Number-based progression. Introduce Interval<K> or redesign Range separately. |
| Bag can simply wrap PersistentMap | No PersistentMap implementation currently exists. It is a new foundation. |
| `const Array` only needs an Array class | StaticValueStore/materialization currently has no homogeneous sequence node and structural materialization remains incomplete. |
| Vector requires a brand-new independent type program | Existing P14 immutable snapshot proposal should become Vector rather than coexist with it. |


# Spec A.2 — Unit, Tuple, and Record Runtime Representation

Status: implementation specification. Requires Spec A.1 landed. This phase builds the canonical runtime substrate without yet replacing the compiler's temporary positional-Tuple bridge or exposing the full new product surface.

## 1. Mission

Introduce a canonical, allocation-free Unit value; migrate native Tuple storage from the obsolete positional-only `Box<[Value]>` model to the ratified two-lane product model; and add a positive-arity immutable native Record representation that preserves Symbol field identity and encounter order. Establish one shared runtime finalization boundary that guarantees no empty Tuple or closed empty Record object can ever be allocated.

This phase is primarily representation and invariants. It must leave the old positive positional Tuple source path operational through the A.1 compatibility bridge. Full literal lowering, duplicate/computed-label user diagnostics, Tuple lane projections, Record surface methods, structural equality/hash, and removal of `Tuple.fromList` are completed in A.3.

## 2. Authority and repository baseline

The normative sources are `collections-next/tuple-record-and-symbols-spec.md` §§8, 11, 19–21, 26–29, 34–36, and 40–41, plus `collections-next/product-normalization-and-unit-spec.md` §§3–14, 35–37. Typing/reflection sections are explicitly out of scope.

Verify HEAD before editing. At specification time:

- `phalcom-core/src/value/mod.rs` defines the `Copy` tagged enum `Value::{Nil, Bool, Number, Symbol, Obj}`. `Nil` is a private sentinel, not a user value. Unit does not exist.
- `phalcom-core/src/heap/tuple.rs` defines `TupleObject { elements: Box<[Value]> }` with `new`, `len`, `is_empty`, `get`, and `elements`.
- `phalcom-core/src/heap/object.rs` has `Object::Tuple(TupleObject)` but no Record arm.
- `phalcom-core/src/heap/mod.rs` exposes `alloc_tuple(Box<[Value]>)` and no Record allocator.
- `phalcom-core/src/heap/accessors.rs` contains immutable Tuple accessors and deliberately no `tuple_mut`.
- `phalcom-core/src/heap/trace.rs` exhaustively traces `Object` variants and currently walks `tuple.elements()`.
- `phalcom-core/src/universe/core_classes.rs` creates native `Tuple` under `Iterable`; no `Unit` or `Record` class exists.
- `phalcom-core/src/primitive/tuple.rs` provides the legacy `Tuple.fromList(_)`, `size_`, and `at_` floor bindings.
- `phalcom-core/core/core.ph` defines the current positional-only Tuple behavior over those raw primitives.

Run `./scripts/verify.sh` before edits and preserve a green gate after every representation slice.

## 3. Canonical Unit representation

### 3.1 Add `Value::Unit`

Use an immediate tagged value:

```rust
pub enum Value {
    Nil,
    Unit,
    Bool(bool),
    Number(f64),
    Symbol(Symbol),
    Obj(ObjRef),
}
```

This is the preferred representation for this repository because `Value` is already the zero-allocation home of Bool/Number/Symbol, is `Copy`, and has explicit class dispatch for immediates. A heap singleton would add allocation, GC rooting, and identity machinery without buying anything.

`Value::Unit` is public surface state. It is not related to the private `Value::Nil` sentinel and must never pass through absence surfacing (`sentinel_to_option`).

### 3.2 Exhaustive `Value` audit

Update every exhaustive `Value` match. At minimum audit:

```text
phalcom-core/src/value/mod.rs
phalcom-core/src/value/render.rs
phalcom-core/src/frame.rs / call-context handling as compilation requires
phalcom-core/src/primitive/**
phalcom-core/src/vm/**
```

Required behavior:

- `as_obj(Unit) -> None`;
- `type_name(Unit) -> "unit"`;
- `Unit.class -> unit_class`;
- `to_context(Unit)` uses `CallContext::Immediate`;
- `Unit.value_eq(Unit) == true`, and Unit is unequal to every non-Unit value;
- `Hash for Value` gives Unit one stable discriminated digest;
- debug/native rendering prints `()` where `Value` itself is rendered directly.

Do not make Unit compare equal to an arbitrary zero-field nominal object. Only product construction normalization yields Unit.

### 3.3 Core Unit class

Add `unit_class` to `CoreClasses` and create `Unit` as a core class under `Object`, not under Tuple or Record. The product capability hierarchy is explicitly deferred; do not encode it as inheritance here.

Mark `Unit` as `native_repr` so generic instance allocation cannot manufacture fake Unit instances. Normal core-class global installation should expose the class object as `Unit`; the value is produced by syntax/construction, not by allocating an instance.

Add a minimal `class Unit` reopen in `phalcom-core/core/core.ph` only where surface behavior is already required. Recommended safe definitions are:

```phalcom
class Unit {
  toString => "()"
  hash => 0
}
```

`==` can remain inherited from `Object` once `Value::value_eq` handles Unit. Do not add speculative `size`, iteration, indexing, or product-protocol methods merely because they may eventually be mathematically sensible; the common product capability surface is deferred.

No new native primitive is required for Unit.

## 4. Tuple representation migration

### 4.1 Required semantic layout

A positive Tuple has an ordered positional prefix and an ordered labeled suffix. Use a compact combined representation:

```rust
pub struct TupleObject {
    values: Box<[Value]>,
    labels: Box<[Symbol]>,
}
```

`labels.len()` is the labeled-lane length. The corresponding labeled values are the suffix of `values`; therefore:

```text
positional_len = values.len() - labels.len()
```

This representation is preferable to three independently allocated slices because it keeps the total iteration/index order contiguous and needs only one Value buffer. It still preserves the lane boundary and ordered Symbol identities exactly.

Construction invariants:

```text
values.len() > 0
labels.len() <= values.len()
labels are unique
positionals = values[..positional_len]
labeled values = values[positional_len..]
```

The heap representation must never admit `values.len() == 0`. Empty product construction normalizes before allocation.

### 4.2 TupleObject API

Replace positional-only helpers with names that expose the semantic layout internally without exposing mutation:

```rust
len() -> usize
positional_len() -> usize
labeled_len() -> usize
values() -> &[Value]
positionals() -> &[Value]
labeled_values() -> &[Value]
labels() -> &[Symbol]
get(index: usize) -> Option<Value>
get_label(label: Symbol) -> Option<Value>
labeled_entries() -> iterator/view over (Symbol, Value)
```

There must remain no `tuple_mut` accessor. Tuple immutability is a representation guarantee, not a convention enforced only by missing public selectors.

Linear Symbol lookup is acceptable for this foundation. Do not add a per-Tuple hash map unless profiling demonstrates a need; Tuple labels are ordered, usually small, and a lookup index would increase every Tuple's footprint.

### 4.3 Heap allocation

Change `Heap::alloc_tuple` so it cannot be used to create an empty product. Prefer a signature that makes the invariant explicit, for example:

```rust
pub(crate) fn alloc_tuple_nonempty(
    &mut self,
    values: Box<[Value]>,
    labels: Box<[Symbol]>,
) -> ObjRef
```

The method may `debug_assert!` structural preconditions but must not be the semantic finalization API. All language/runtime construction should route through the product finalizer described in §6, which performs zero normalization and duplicate checks before calling the allocator.

## 5. Record representation

### 5.1 Native positive-arity Record

Add `phalcom-core/src/heap/record.rs` with a fixed immutable representation preserving encounter order:

```rust
pub struct RecordObject {
    labels: Box<[Symbol]>,
    values: Box<[Value]>,
}
```

Required invariants:

```text
labels.len() == values.len()
labels.len() > 0
labels are unique
slot i's Symbol label corresponds to values[i]
array order is construction encounter order
```

Provide only immutable accessors:

```rust
len() -> usize
labels() -> &[Symbol]
values() -> &[Value]
get(label: Symbol) -> Option<Value>
entries() -> ordered iterator/view of (Symbol, Value)
```

Do not add a mutable accessor. Record field sets and values are fixed after finalization.

A linear lookup is the required first implementation. The authoritative design permits shared `RecordShape` objects and canonical unordered shape IDs as optimizations, but neither is needed to make semantics correct. Avoid shape interning in Spec A unless the repository already has an obvious reusable facility: it would add caching/identity complexity before any workload demonstrates value.

### 5.2 Object arm and slot-size discipline

Add `Object::Record(...)`. Start with an inline `RecordObject` if `size_of::<Object>()` remains within the current expected slot-size invariant; otherwise box the Record payload. Do not casually enlarge every SlotMap entry. The heap documentation notes that payload boxing is deliberately used to protect the common arena slot size.

Whichever representation is selected, add a regression test for `size_of::<Object>()` or update the existing memory-layout invariant consciously. A Record optimization is not a valid reason to silently increase all heap slots.

### 5.3 Heap plumbing

Update:

```text
phalcom-core/src/heap/mod.rs
phalcom-core/src/heap/accessors.rs
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/trace.rs
```

Export `RecordObject`, add `alloc_record_nonempty`, `record`, and `as_record`. There must be no `record_mut`.

`trace_object` must trace every Record value through `Value::as_obj`. Record labels are `Symbol`s and therefore are not heap edges. Keep the `Object` match exhaustive with no wildcard.

Update test-only object-kind helpers and any other exhaustive `Object` matches surfaced by the compiler.

## 6. Shared product finalization boundary

Add one internal runtime construction module, recommended as `phalcom-core/src/product.rs`, instead of letting the compiler/VM allocate Tuple and Record objects directly in several places.

Recommended APIs:

```rust
pub(crate) fn finish_tuple(
    vm: &mut VM,
    positionals: Vec<Value>,
    labeled: Vec<(Symbol, Value)>,
) -> Result<Value, ProductBuildError>

pub(crate) fn finish_record(
    vm: &mut VM,
    fields: Vec<(Symbol, Value)>,
) -> Result<Value, ProductBuildError>
```

The exact ownership shape may be optimized, but the semantic boundary is mandatory.

`finish_tuple` must:

1. validate duplicate labeled Symbols;
2. if both lanes are empty, return `Value::Unit` without allocating;
3. otherwise flatten to total Tuple order and allocate one positive Tuple object.

`finish_record` must:

1. validate duplicate field Symbols while retaining the first-seen encounter ordering only for successful unique inputs;
2. if the closed field set is empty, return `Value::Unit` without allocating;
3. otherwise allocate one positive Record object preserving encounter order.

Use a small internal error such as:

```rust
pub enum ProductBuildError {
    DuplicateLabel(Symbol),
}
```

A.3 maps these failures to user-facing compile/runtime diagnostics. Do not encode “first wins” or “last wins”.

These finalizers are the only language-facing gateway to product allocation. The hard invariant after A.2 is:

```text
there is no runtime path that constructs an empty Tuple heap object
there is no runtime path that constructs a closed empty Record heap object
```

## 7. Preserve the A.1 positional compatibility path

`Tuple.fromList(_)` remains temporarily installed because A.1 still compiles ordinary positive positional Tuple literals through it. Adapt `tuple_class_from_list` to the new finalizer:

- copy the List's elements into the positional lane;
- use an empty labeled lane;
- call `finish_tuple`;
- an empty List must now return `Value::Unit`, never an empty `TupleObject`.

This last rule matters even though normal old Tuple literals were positive: `Tuple.fromList(List.new())` is currently callable and would otherwise create an observable pre-normalized empty Tuple.

Keep `tuple_raw_size` and `tuple_raw_at` operational against the new total-order Value buffer so existing Tuple methods and tests remain green. Their eventual strict/negative-index semantics are not redesigned here; Spec C owns general indexing and slicing. A.2 only preserves behavior until later surface work.

Do not add a Record constructor primitive in A.2. Record is not source-executable until A.3, and creating a temporary public constructor would become another API to retire.

## 8. Core classes

In `phalcom-core/src/universe/core_classes.rs`:

- create `Unit` under `Object`;
- keep `Tuple` under its existing `Iterable` superclass because Tuple value iteration is ratified and already integrated with the repository's iterator substrate;
- create `Record` under `Object`, not `Iterable`, for now. The Record spec preserves encounter order for “iteration, where defined” but deliberately does not ratify what ordinary iteration yields. Do not choose values-versus-fields-versus-entries through inheritance accidentally.
- add `Unit` and `Record` to native-representation protection as appropriate; Unit is immediate and Record is a dedicated heap arm.

Add `Unit` and `Record` constants to `primitive::ClassName` if that list remains the repository's canonical native class-name table.

Do not introduce a Tuple/Record/Unit common superclass. The product capability hierarchy is deferred.

## 9. Primitive-floor governance

A.2 should add no new primitive binding. It changes the internals behind existing Tuple bindings and introduces runtime Record storage that is not yet surfaced.

This is deliberate. `docs/adr/accepted/0039-amend-floor-admit-collection-container-primitives.md` and `docs/spec/current/core/floor-census.md` govern the currently admitted Tuple floor. New lane/Record raw methods belong to A.3, where the exact minimal floor delta can be reviewed as one coherent surface change.

If implementation unexpectedly requires a new native method in A.2, stop and resolve the floor amendment rather than silently registering it.

## 10. Tests

### 10.1 Unit representation tests

Add Rust-level tests proving:

- `Value::Unit` is `Copy` and non-heap;
- `Value::Unit.class(vm)` resolves to `Unit`;
- Unit equals Unit and not other immediates;
- Unit hashes deterministically;
- native/debug rendering is `()`;
- generic native allocation cannot create an instance masquerading as Unit.

### 10.2 Tuple object tests

Test the raw object/finalizer with:

- pure positional values;
- pure labeled values;
- mixed lanes;
- total order equals positionals followed by labeled values;
- labels remain encounter ordered;
- Symbol lookup returns the correct suffix value;
- duplicate labels fail finalization;
- zero lanes return Unit and do not increment heap live-object count;
- no mutable Tuple accessor exists.

Keep all existing positional Tuple language goldens green through `Tuple.fromList`.

### 10.3 Record object tests

Test:

- positive Record allocation preserves field encounter order;
- field lookup by Symbol identity;
- duplicate labels fail finalization;
- zero fields return Unit and do not allocate;
- GC retains objects referenced only from Record values;
- Record labels themselves need no GC edges;
- no mutable accessor exists.

### 10.4 GC and layout tests

Add a GC reachability probe in which a Record is rooted and its child object survives collection, then becomes collectible when the Record root disappears. Update any exhaustive object-kind test. Pin or consciously verify `size_of::<Object>()` so Record does not accidentally inflate every heap slot.

## 11. Expected write set

Primary files:

```text
phalcom-core/src/value/mod.rs
phalcom-core/src/value/render.rs
phalcom-core/src/heap/tuple.rs
phalcom-core/src/heap/record.rs                  # new
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/mod.rs
phalcom-core/src/heap/accessors.rs
phalcom-core/src/heap/trace.rs
phalcom-core/src/product.rs                      # recommended new internal module
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/primitive/mod.rs
phalcom-core/src/primitive/tuple.rs              # adapt legacy bridge only
phalcom-core/core/core.ph                         # minimal Unit reopen + Tuple compatibility fixes
phalcom-core/tests/**
```

Compile errors may expose additional exhaustive `Value`/`Object` matches. Fix those mechanically and include them in the implementation report. Do not change parser semantics in this phase except for a bug proven to block A.2.

## 12. Completion gate

A.2 is complete only when:

1. Unit is a canonical immediate value with a real core class;
2. every `()`/empty-Record runtime finalization path available internally can return the same `Value::Unit` without heap allocation;
3. no empty Tuple heap object can be constructed, including through legacy `Tuple.fromList([])`;
4. Tuple storage preserves two lanes and total order while remaining immutable;
5. Record exists as a positive-arity immutable heap representation preserving Symbol fields and encounter order;
6. GC tracing is correct for Record and migrated Tuple;
7. the current positive positional Tuple language suite still passes through the compatibility bridge;
8. the primitive floor has not grown;
9. `./scripts/verify.sh --full` is green.

The implementation report must include the final `Value` and heap representations, measured `Object` size before/after, zero-allocation proof/tests for Unit normalization, GC tests, primitive-floor delta (expected `+0`), and the verification tail.

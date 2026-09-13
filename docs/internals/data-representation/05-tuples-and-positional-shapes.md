# 05 — Tuples and Positional Shapes

**Sources:**
- `phalcom-core/src/heap/tuple.rs` — `TupleObject`
- `phalcom-core/src/product/mod.rs` — `finish_tuple`, product construction boundary

A *tuple* in Phalcom is a fixed-arity, immutable product — a flat sequence of
values that may be all positional, all labeled, or mixed. Tuples are a first-class
value type distinct from both lists (mutable, variable-length) and data class
instances (nominally typed). This document covers the heap object structure,
the empty-tuple boundary, the labeled extension, and GC behavior.

---

## Source syntax

```phalcom
(1, 2, 3)              // positional tuple, arity 3
(x: 1, y: 2)           // labeled tuple (record-like surface)
(1, 2, label: 3)       // mixed: two positional, one labeled
()                     // empty tuple — normalized to Unit
```

Tuples are immutable once constructed. There is no `tuple.set(index, value)`
operation at the language surface.

---

## The empty-tuple boundary — `Value::unit()`

An empty tuple `()` is **never** allocated on the heap. The compiler normalizes
it to `Value::unit()` before any heap interaction. The product construction
function `finish_tuple` enforces this as a hard boundary:

```rust
// phalcom-core/src/product/mod.rs
pub(crate) fn finish_tuple(
    vm: &mut VM,
    mut positionals: Vec<Value>,
    labeled: Vec<(Symbol, Value)>,
) -> Result<Value, ProductBuildError> {
    unique(&labeled)?;                          // reject duplicate labels
    if positionals.is_empty() && labeled.is_empty() {
        return Ok(Value::unit());               // ← the boundary
    }
    // … build TupleObject and allocate
}
```

This invariant means no `TupleObject` with zero elements ever exists in the
arena. Code that receives a `Value` from a tuple expression and sees tag `Unit`
knows it was the empty tuple; code that sees tag `Obj` knows it is non-empty.

---

## `TupleObject` — the heap payload

```rust
// phalcom-core/src/heap/tuple.rs
pub struct TupleObject {
    values: Box<[Value]>,   // all values: positionals followed by labeled values
    labels: Box<[Symbol]>,  // labels for the labeled suffix only
}
```

The layout rule: **labels are a suffix of values**.

```
values = [pos0, pos1, …, posN, lv0, lv1, …, lvM]
labels =                       [l0,  l1,  …, lM ]
```

- `positionals()` returns `&values[..positional_len()]`
- `labeled_values()` returns `&values[positional_len()..]`
- `positional_len() = values.len() - labels.len()`
- `labeled_len() = labels.len()`

### Invariants enforced by `TupleObject::new`

1. `labels.len() <= values.len()` — labels can never outnumber values.
2. Labels are unique — checked with an O(n²) scan at construction.

### Allocation

`Heap::alloc_tuple_nonempty(values: Box<[Value]>, labels: Box<[Symbol]>)` is
the only path to a live `Object::Tuple`. The `(crate)` visibility and the
`_nonempty` name suffix communicate that callers must ensure non-emptiness and
that `finish_tuple` owns the zero check.

---

## Labeled field access

Given a label `Symbol`, `get_label` scans the labels slice linearly:

```rust
pub fn get_label(&self, label: Symbol) -> Option<Value> {
    self.labels
        .iter()
        .position(|candidate| *candidate == label)
        .map(|i| self.labeled_values()[i])
}
```

This is O(n) in the number of labels but tuples are expected to be small in
practice.

For positional access by index, `get(index)` returns `values.get(index).copied()`.
Out-of-bounds returns `None`, which surfaces as `Option.None` to the caller —
never a panic.

---

## Immutability as a representation guarantee

`TupleObject` has no `&mut self` accessor at all. The `values` and `labels`
fields are private and the struct exposes only read-only slices. This is a
deliberate representation choice documented in the source:

> *Immutability is a representation guarantee: the backing Box<[Value]> is a
> fixed-length slice, and TupleObject exposes no mutation accessor at all —
> a later diff cannot accidentally reintroduce mutation the way a missing
> selector could.*

Because tuples are immutable, they are value-hashable and valid as `Map`/`Set`
keys (collection-protocol law 4). Mutable objects like `List` and `Map` are not.

---

## Duplicate label rejection

`finish_tuple` calls `unique(&labeled)` before constructing the tuple, returning
`ProductBuildError::DuplicateLabel(label)` if the same symbol appears twice in
the labeled portion. This is a VM-level invariant boundary; semantic analysis
should catch duplicates earlier, but the runtime check ensures the heap always
holds well-formed tuples.

---

## GC tracing

The GC tracer visits `Object::Tuple(tuple)` by iterating the full `values` slice.
Every element is a `Value`; for each one where `tag == Obj`, the tracer follows
the `ObjRef` into the heap.

Unlike `ProductStorage`, tuples do not have a separate `value_slot_offsets` map
— all slots are full `Value`s, so the tracer visits every slot unconditionally.

---

## Relationship to `data` classes and records

| | Tuple | Data class | Record |
|---|---|---|---|
| Named fields | Optional (labeled) | Required | Required |
| Nominal type | No (structural) | Yes | No (structural) |
| Heap struct | `TupleObject` | `DataObject` | `RecordObject` |
| Storage engine | Direct `Box<[Value]>` | `ProductStorage` | `Box<[Value]>` |
| Immutable | Yes | Yes | Yes |
| Empty normalization | `Value::unit()` | DataSingleton | — |

Tuples and records use a direct `Box<[Value]>` layout, bypassing the
`ProductStorage` packing layer. Data classes go through `ProductStorage` because
they support typed (scalar-optimized) slots.

---

## See also

- [01-value-representation.md](01-value-representation.md) — `Unit` tag for empty tuples
- [02-product-layout-and-storage.md](02-product-layout-and-storage.md) — the product engine used by data classes
- [03-data-classes-and-records.md](03-data-classes-and-records.md) — how data classes differ
- [ADR-0032](../../adr/accepted/0032-collections-representation-and-literals.md) — Tuple native representation decision
- [ADR-0039](../../adr/accepted/0039-amend-floor-admit-collection-container-primitives.md) — floor primitives amendment

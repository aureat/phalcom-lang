# 02 — Product Layout and Storage

**Sources:**
- `phalcom-core/src/product/layout.rs`
- `phalcom-core/src/product/storage.rs`
- `phalcom-core/src/product/registry.rs`
- `phalcom-core/src/product/mod.rs`

Both `data` class instances and non-nullary ADT constructor payloads use the
same underlying storage subsystem: a word-packed flat buffer described by a
`ProductLayout`. This document explains the layout model, the packing rules,
and how values are encoded and decoded.

---

## Overview

When the VM materializes a `data Point(x: Int, y: Int)` or an ADT case
`Some(value: T)`, it needs somewhere to store the field values. The product
subsystem provides:

1. **`ProductLayout`** — the static descriptor that says how many words the
   buffer occupies and where each field sits inside it.
2. **`ProductStorage`** — the actual flat `Box<[u64]>` buffer for one instance,
   paired with the layout ID that describes it.
3. **`ProductLayoutRegistry`** — a VM-global interning registry that deduplicates
   layouts so identical shapes share a single descriptor.

---

## Component slot representations (`ProductSlotRepr`)

Each field in a product occupies one *slot*, which can use one of five
physical representations:

```rust
pub enum ProductSlotRepr {
    Int64,    // 1 word — raw i64 bits
    Float64,  // 1 word — raw f64 bits (IEEE 754)
    Bool,     // 1 word — 0 = false, any non-zero = true
    Symbol,   // 1 word — u32 interned symbol index in lower half
    Value,    // 2 words — full 16-byte Value (payload word, then meta word)
}
```

The `word_size()` method returns 1 for the scalar kinds and 2 for `Value`:

```
Int64, Float64, Bool, Symbol  →  1 word (8 bytes)
Value                         →  2 words (16 bytes)
```

Scalar slots trade GC transparency for density. When a field is typed as `Int`,
the compiler can elect `Int64` and store the raw i64 bits in a single word,
halving the storage cost relative to a `Value` slot. The GC does not need to
visit scalar slots; only `Value` slots require tracing.

Currently, the standalone/unlinked compiler synthesizes all slots as `Value`
(the safe default). The semantic lowering layer emits typed specs once type
inference resolves component types.

---

## `ProductLayout` — the static descriptor

```rust
pub struct ProductLayout {
    pub word_len: u32,                        // total buffer size in words
    pub components: Box<[ProductComponentLayout]>,  // ordered component metadata
    value_slot_offsets: Box<[u32]>,           // word offsets of all Value slots (for GC)
}

pub struct ProductComponentLayout {
    pub logical_index: u32,   // component position (0-based, dense)
    pub word_offset:   u32,   // starting word index in the buffer
    pub repr:          ProductSlotRepr,
}
```

### Layout construction

Layouts are built from `ProductLayoutSpec`, a list of `ProductComponentSpec`
(index + repr). The `build_layout()` method assigns word offsets in logical
index order, compacts them sequentially, and validates that:

- logical indices are dense (0, 1, 2, … N-1)
- no two components overlap in word space
- word offsets do not overflow

The resulting `ProductLayout` is immutable once built.

### Word offset example

Consider `data Point(x: Int, y: Float, label: Symbol)` with all slots as scalars:

```
component 0  "x"      Int64     word_offset = 0   (words 0)
component 1  "y"      Float64   word_offset = 1   (words 1)
component 2  "label"  Symbol    word_offset = 2   (words 2)
total word_len = 3
```

Now consider `data Wrapper(inner: Value, flag: Bool)`:

```
component 0  "inner"  Value     word_offset = 0   (words 0–1)
component 1  "flag"   Bool      word_offset = 2   (words 2)
total word_len = 3
value_slot_offsets = [0]  ← GC must visit word 0 (and implicitly word 1)
```

---

## `ProductStorage` — the instance buffer

```rust
pub struct ProductStorage {
    pub layout: ProductLayoutId,  // which layout describes this buffer
    pub words:  Box<[u64]>,       // flat word array
}
```

`ProductStorage` allocates a zeroed buffer of `layout.word_len` words and
provides two operations: `store_component` and `load_component`.

### Storing a component

`store_component(layout, logical_index, value: Value) -> Result<(), &'static str>`

The method dispatches on the slot's `repr`:

| `repr` | Encoding |
|---|---|
| `Int64` | `words[offset] = value.as_int()? as u64` |
| `Float64` | `words[offset] = value.as_float()?.to_bits()` |
| `Bool` | `words[offset] = if value.as_bool()? { 1 } else { 0 }` |
| `Symbol` | `words[offset] = symbol.0 as u64` |
| `Value` | `words[offset] = payload; words[offset+1] = meta` |

Each scalar store validates that the runtime value matches the expected type.
The `Value` store uses `raw_words()` to extract both words atomically.

### Loading a component

`load_component(layout, logical_index) -> Result<Value, &'static str>`

The inverse: reads one or two words from the buffer and reconstructs a `Value`.
Scalar slots wrap the raw bits in the appropriate `Value` constructor:

| `repr` | Decoding |
|---|---|
| `Int64` | `Value::int(words[offset] as i64)` |
| `Float64` | `Value::float(f64::from_bits(words[offset]))` |
| `Bool` | `Value::bool(words[offset] != 0)` |
| `Symbol` | `Value::symbol(Symbol(words[offset] as u32))` |
| `Value` | `Value::from_raw_words(words[offset], words[offset+1])` |

### Convenience constructor

`ProductStorage::from_values(layout_id, layout, values: &[Value])`

Accepts an ordered slice of `Value`s matching `layout.components.len()` and
stores each one in order. This is the main entry point when constructing an
instance from evaluated arguments.

---

## `ProductLayoutRegistry` — layout interning

The VM holds a `ProductLayoutRegistry` (accessed as `vm.heap.product_layouts`)
that deduplicates layouts:

```rust
// Intern or retrieve a layout
let layout_id: ProductLayoutId = registry.register(layout_spec);

// Retrieve a registered layout by ID
let layout: &ProductLayout = registry.layout(layout_id)?;
```

This means two `data` declarations with identical component signatures share a
single layout descriptor in memory. The u32 `ProductLayoutId` is the stable
identifier used in `ProductStorage` to avoid carrying the full layout inline.

---

## GC tracing with `value_slot_offsets`

The garbage collector needs to scan every live `Value` inside a `ProductStorage`
to trace heap references. Scalar slots (`Int64`, `Float64`, etc.) hold no
`ObjRef` and can be skipped.

`ProductLayout::value_slot_offsets()` returns the word offsets of all `Value`
slots. The collector iterates over this slice and visits two words at each offset:

```
for offset in layout.value_slot_offsets() {
    // words[*offset] and words[*offset + 1] form a Value
    // visit ObjRef if tag == Obj
}
```

This design keeps the tracing hot path lean — zero iterations when all slots
are scalars, cheap iteration otherwise.

---

## Building a layout spec (summary)

```rust
use crate::product::{ProductComponentSpec, ProductLayoutSpec, ProductSlotRepr};

let spec = ProductLayoutSpec::new(vec![
    ProductComponentSpec { logical_index: 0, repr: ProductSlotRepr::Int64 },
    ProductComponentSpec { logical_index: 1, repr: ProductSlotRepr::Value },
]);
let layout = spec.build_layout().expect("valid layout");
```

The returned `ProductLayout` is then registered in `ProductLayoutRegistry` to
obtain a `ProductLayoutId`, which is stored in `RuntimeDataDescriptor` (for
data classes) or `RuntimeVariantDescriptor` (for ADT cases).

---

## See also

- [03-data-classes-and-records.md](03-data-classes-and-records.md) — how data
  class instances are constructed using `ProductStorage`
- [04-adt-enum-variants.md](04-adt-enum-variants.md) — how ADT case payloads
  use the same storage
- [ADR-0010](../../adr/accepted/0010-tagged-value-enum.md) — the tagged Value design
- [ADR-0011](../../adr/accepted/0011-static-instance-slot-layout.md) — static slot layout policy

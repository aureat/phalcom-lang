# 01 — Value Representation

**Source:** `phalcom-core/src/value/repr.rs`

Every runtime value in Phalcom is a uniform `Value` — exactly 16 bytes, two
64-bit words. There is no pointer-tagging, no NaN-boxing, no variant-sized
payload. The representation is explicit, testable, and deliberate.

This document explains the layout, every tag, the `Option` depth trick, and the
equality and hashing rules that fall out of it.

---

## Physical layout

```
 ┌──────────────────────────────────────────────────────────────────────┐
 │  payload: u64                                                        │
 ├──────────────────────────────────────────────────────────────────────┤
 │  meta: u64                                                           │
 │    bits  0– 7 :  ValueTag (u8) — which kind of value this is        │
 │    bits  8–39 :  some_depth (u32) — Option<T> nesting depth         │
 │    bits 40–63 :  reserved, must always be zero                      │
 └──────────────────────────────────────────────────────────────────────┘
```

The `meta` word therefore carries three independent fields packed into one
64-bit integer. The constants are:

```rust
const TAG_MASK:      u64 = 0xff;
const DEPTH_SHIFT:   u32 = 8;
const DEPTH_MASK:    u64 = 0xffff_ffffu64 << DEPTH_SHIFT;
const RESERVED_MASK: u64 = !(TAG_MASK | DEPTH_MASK);
```

`Value` has `#[repr(C)]` to guarantee the `(payload, meta)` field order.
The test `value_is_exactly_sixteen_bytes` asserts `size_of::<Value>() == 16`
at compile time.

---

## The tag (`ValueTag`)

```rust
#[repr(u8)]
pub(crate) enum ValueTag {
    Nil           = 0,   // internal sentinel — no source form
    Unit          = 1,   // ()
    Bool          = 2,   // true / false
    Int           = 3,   // i64 integer
    Float         = 4,   // f64 floating-point
    Symbol        = 5,   // interned string identifier
    Obj           = 6,   // generational ObjRef into the heap arena
    None          = 7,   // the bare absence value (bottom of Option<T>)
    AdtSingleton  = 8,   // nullary enum variant stored without heap allocation
    DataSingleton = 9,   // nullary data class stored without heap allocation
}
```

### Tag–payload table

| Tag | Payload interpretation | Notes |
|---|---|---|
| `Nil` | 0 (unused) | VM-internal; never exposed to user code |
| `Unit` | 0 (unused) | The canonical result of `()` |
| `Bool` | `0` = false, `1` = true | Anything non-zero would also be `true` by check |
| `Int` | `payload as i64` — full signed 64-bit integer | Full i64 range without boxing |
| `Float` | `f64::from_bits(payload)` — raw IEEE 754 bits | Bit-exact NaN payloads are preserved |
| `Symbol` | `payload as u32` — interned symbol index | Upper 32 bits of payload are unused |
| `Obj` | `ObjRef::from_opaque_u64(payload)` — arena handle | The arena is a SlotMap (ADR-0009) |
| `None` | 0 (unused) | Represents `Option.None` (not `nil`) |
| `AdtSingleton` | `RuntimeVariantId` packed as u32 | Used for nullary enum variants |
| `DataSingleton` | `RuntimeDataDescriptorId` packed as u32 | Used for nullary data class values |

---

## The `some_depth` field — zero-cost `Option<T>` nesting

Phalcom represents `Option<T>` without heap allocation by storing a *nesting
depth* in the `some_depth` field of `meta`.

A plain value `v` has `some_depth == 0` and is treated as itself.

Wrapping it with `Some(…)` increments the depth:

```
Value::int(42)                        some_depth = 0  → bare Int
Value::int(42).with_some_depth(1)     some_depth = 1  → Some(42)
Value::int(42).with_some_depth(2)     some_depth = 2  → Some(Some(42))
```

The accessors enforce this contract:

- `is_int()` — returns `true` **only if** `tag == Int && some_depth == 0`.
- `is_some()` — returns `true` if `some_depth > 0` (regardless of inner tag).
- `as_int()` — returns `None` if the value is `Some(Int)`, not a bare `Int`.

This means **a wrapped value always rejects its inner-type accessors**, preventing
accidental extraction through Option layers without an explicit unwrap.

The depth field is 32 bits wide, supporting nesting up to `u32::MAX` deep —
far beyond any practical use but also free in terms of space.

```
with_some_depth(d):  meta = (meta & !DEPTH_MASK) | ((d as u64) << DEPTH_SHIFT)
some_depth_raw():    ((meta & DEPTH_MASK) >> DEPTH_SHIFT) as u32
without_some_wrappers(): meta & !DEPTH_MASK  ← strips depth, reveals inner kind
```

---

## `Value::none()` vs `Value::nil()`

These are two distinct tags and two distinct values.

| | Tag | Meaning |
|---|---|---|
| `Value::nil()` | `Nil` | VM-internal sentinel. Never exposed at the language surface. |
| `Value::none()` | `None` | The `Option.None` value. Exposed to user code as the absence of a value. |

User code cannot observe `Nil`. The `None` value is the correct absence
representation.

---

## Equality and hashing

`Value` implements `PartialEq`, `Eq`, and `Hash`. The rules:

1. Two values with different tags, or different `some_depth`, are never equal.
2. `Nil`, `Unit`, and `None` compare by tag + depth only (no payload).
3. `Bool` compares the boolean meaning of payload (non-zero = true).
4. `Int` compares as `i64`.
5. `Float` compares with IEEE 754 `==` — `0.0 == -0.0` (hash-consistent), `NaN != NaN`.
6. `Symbol` compares the u32 interned index.
7. `Obj` compares by reference identity (`payload` is the arena handle).
8. `AdtSingleton` and `DataSingleton` compare the u32 descriptor ID.

The `Float` hash is careful: `0.0` and `-0.0` hash identically (since they are
`==`), and all `NaN` payloads hash to the canonical `f64::NAN` bits to avoid
instability.

The `===` identity operator (same-bits) is implemented separately as
`same_bits(self, other)` — it compares both words exactly and does not apply
the semantic rules above.

---

## `AdtSingleton` and `DataSingleton` — no-allocation nullary values

When a `data` declaration or an `enum` variant has no fields, there is nothing
to store on the heap. Instead:

- A nullary `data` declaration with descriptor ID `d` is represented as
  `Value { payload: d as u64, meta: DataSingleton as u64 }`.
- A nullary `enum` variant with runtime ID `v` is represented as
  `Value { payload: v as u32 as u64, meta: AdtSingleton as u64 }`.

Both carry a u32 identity in `payload`. Because descriptor/variant IDs fit in
32 bits, the upper 32 bits of `payload` are always zero for these tags.

---

## Constructor and accessor API summary

```rust
// Constructors
Value::nil()                               // Nil sentinel
Value::unit()                              // Unit ()
Value::bool(b: bool)                       // Bool
Value::int(n: i64)                         // Int
Value::float(f: f64)                       // Float
Value::symbol(s: Symbol)                   // Symbol
Value::obj(r: ObjRef)                      // Obj (heap pointer)
Value::none()                              // None (Option::None)
Value::adt_singleton(v: RuntimeVariantId)  // AdtSingleton
Value::data_singleton(d: RuntimeDataDescriptorId) // DataSingleton

// Option wrapping
value.with_some_depth(depth: u32)   // wraps value in depth levels of Some
value.without_some_wrappers()       // strips depth, reveals inner representation
value.some_depth_raw() -> u32       // current nesting depth

// Type guards (all check depth == 0)
value.is_nil(), value.is_unit(), value.is_bool(), value.is_int()
value.is_float(), value.is_symbol(), value.is_obj(), value.is_some()
value.is_adt_singleton(), value.is_data_singleton()

// Extractors (return None if tag or depth mismatches)
value.as_bool() -> Option<bool>
value.as_int()  -> Option<i64>
value.as_float() -> Option<f64>
value.as_obj()  -> Option<ObjRef>
value.as_adt_singleton() -> Option<RuntimeVariantId>
value.as_data_singleton() -> Option<RuntimeDataDescriptorId>
```

---

## See also

- [02-product-layout-and-storage.md](02-product-layout-and-storage.md) — packed word storage for multi-field objects
- [ADR-0010](../../adr/accepted/0010-tagged-value-enum.md) — rationale for the explicit tag representation over NaN-boxing
- [ADR-0044](../../adr/accepted/0044-option-bootstrap-formalization-and-defer-niche-encoding.md) — why niche encoding is deferred

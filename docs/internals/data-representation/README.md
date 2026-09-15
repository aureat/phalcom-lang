# Data Types, Layouts, and Representation

This folder documents how Phalcom implements data types end-to-end — from source
syntax through static semantics to bytes on the heap.

## Why read this

The representation system is the heart of the runtime. Every value the VM
touches, every field a user accesses, every pattern the compiler matches against
— all pass through the structures described here. Understanding these five
layers unlocks the whole VM.

## Architecture in one diagram

```
Source (Phalcom)
  │
  ├── Primitive literals  Int, Float, Bool, Symbol, Unit
  ├── data Foo(x, y)      record-shaped data class
  ├── enum Option<T> { … }  algebraic data type
  └── (a, b: c)           tuple / labeled product
        │
        ▼  phalcom-ast
  AST nodes: DataDef, EnumDef, TupleExpr …
        │
        ▼  phalcom-semantic
  Static identities: DeclarationId, VariantId, DataComponentId
  Layout specs: ProductLayoutSpec
        │
        ▼  phalcom-core / compiler
  Bytecode: Data, FinalizeData, Tuple, AdtCase, MatchVariant …
        │
        ▼  phalcom-core / vm + heap
  Runtime registry, heap allocation, GC tracing
```

## The five representation layers

| Layer | Document |
|---|---|
| 1. Uniform `Value` (16 bytes) — primitives, singletons, heap refs | [01-value-representation.md](01-value-representation.md) |
| 2. Packed product storage — the word array shared by data & ADTs | [02-product-layout-and-storage.md](02-product-layout-and-storage.md) |
| 3. Data classes (`data Foo(…)`) and record-shaped types | [03-data-classes-and-records.md](03-data-classes-and-records.md) |
| 4. Enum variants (`enum E { A, B(x, y) }`) | [04-adt-enum-variants.md](04-adt-enum-variants.md) |
| 5. Tuples — positional and labeled products | [05-tuples-and-positional-shapes.md](05-tuples-and-positional-shapes.md) |
| End-to-end pipeline walkthrough | [06-code-walkthrough-and-pipeline.md](06-code-walkthrough-and-pipeline.md) |

## Core type taxonomy

### Immediate (unboxed) values

These live entirely inside a 16-byte `Value` — no heap allocation.

| Kind | Tag | What is stored in payload |
|---|---|---|
| `nil` | `Nil` | 0 |
| `()` (Unit) | `Unit` | 0 |
| `Bool` | `Bool` | 0 = false, 1 = true |
| `Int` | `Int` | i64 bits |
| `Float` | `Float` | f64 bits |
| `Symbol` | `Symbol` | Interned u32 index |
| `Option<T>` wrapping | any tag | depth field encodes nesting |
| Nullary ADT singleton | `AdtSingleton` | `RuntimeVariantId` as u32 |
| Nullary data singleton | `DataSingleton` | `RuntimeDataDescriptorId` as u32 |

### Heap-backed objects

These live in the arena; the `Value` holds an `ObjRef` with tag `Obj`.

| `Object` variant | Rust struct | What it represents |
|---|---|---|
| `Object::Tuple` | `TupleObject` | Fixed-arity immutable product, positional + labeled |
| `Object::Data` | `DataObject` | Positive-arity `data` class instance |
| `Object::AdtCase` | `AdtCaseObject` | Non-nullary enum constructor result |
| `Object::Record` | `RecordObject` | Dynamic labeled record literal |
| `Object::Instance` | `InstanceObject` | General class-based instance |

## Key system invariants

- **I-16B**: `Value` is always exactly 16 bytes (`size_of::<Value>() == 16`).
- **I-IMM**: `TupleObject` exposes no mutation accessor — immutability is a
  representation guarantee, not a protocol convention.
- **I-UNIT**: An empty tuple `()` is never allocated on the heap; the compiler
  normalizes it to `Value::unit()`.
- **I-PROD**: `ProductStorage` is shared between `DataObject` and `AdtCaseObject`
  — the layout engine is reused across both forms.
- **I-ID**: `VariantId ≠ RuntimeVariantId ≠ CaseDiscriminant` — static, VM, and
  physical identities are always distinct types.
- **I-REG**: Both `RuntimeDataRegistry` and `RuntimeAdtRegistry` are idempotent
  — re-registering the same owner is a no-op, not an error.

## Key source locations

| File | Purpose |
|---|---|
| `phalcom-core/src/value/repr.rs` | `Value`, `ValueTag`, bitfield layout |
| `phalcom-core/src/product/layout.rs` | `ProductLayout`, `ProductSlotRepr`, `ProductComponentSpec` |
| `phalcom-core/src/product/storage.rs` | `ProductStorage`: packing and unpacking |
| `phalcom-core/src/product/mod.rs` | Product construction boundary, `finish_tuple` |
| `phalcom-core/src/data.rs` | `RuntimeDataRegistry`, `RuntimeDataDescriptor` |
| `phalcom-core/src/heap/data.rs` | `DataObject` (heap payload) |
| `phalcom-core/src/vm/data.rs` | VM registration and getter synthesis |
| `phalcom-core/src/adt.rs` | `RuntimeAdtRegistry`, variant/enum descriptors |
| `phalcom-core/src/heap/adt.rs` | `AdtCaseObject` (heap payload) |
| `phalcom-core/src/heap/tuple.rs` | `TupleObject` |
| `phalcom-core/src/heap/object.rs` | `Object` enum — all heap variants |

## Relevant ADRs

| ADR | Topic |
|---|---|
| TDR-0008 | Handle-arena heap, `ObjRef` model |
| TDR-0009 | 16-byte explicit tagged `Value` |
| TDR-0010 | Static slot layout |
| TDR-0029 | Tuple, List, Map native representation |
| TDR-0034 | Raw-primitive floor amendment |
| TDR-0038 | Option bootstrap, deferred niche encoding |

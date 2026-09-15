# 04 — ADT Enum Variants

**Sources:**
- `phalcom-core/src/adt.rs` — identities, descriptors, `RuntimeAdtRegistry`
- `phalcom-core/src/heap/adt.rs` — `AdtCaseObject` heap payload
- `phalcom-core/src/vm/adt.rs` — VM registration and construction

Phalcom's algebraic data types (`enum`) provide sum types with named variants.
Each variant is either a *singleton* (no fields) or a *constructor* (one or
more fields). This document explains the identity model, the two representation
strategies, and how pattern matching navigates them.

---

## Source syntax

```phalcom
enum Option<T> {
  Some(value: T)
  None
}

enum Result<T, E> {
  Ok(value: T)
  Err(error: E)
}

enum Color {
  Red
  Green
  Blue
}
```

Each `enum` block introduces:
- An **enum root type** (e.g. `Option<T>`).
- One or more **variant constructors** — nullary (like `None`, `Red`) or
  positional/record-shaped (like `Some(value: T)`).

---

## The three identity levels

A fundamental design invariant is that static, VM, and physical identities are
three separate types that must never be confused:

```
Static (phalcom-semantic)      →  VariantId
VM-local runtime               →  RuntimeVariantId
Physical discriminant          →  CaseDiscriminant
```

| Identity | Type | Scope | What it identifies |
|---|---|---|---|
| `VariantId` | Semantic | Cross-process, stable | A variant in a module's semantic graph |
| `RuntimeVariantId` | VM | Per-VM instance | A variant registered in this VM session |
| `CaseDiscriminant` | Physical | Per-enum, dense | The index of the variant within its enum (0, 1, 2, …) |
| `RuntimeEnumId` | VM | Per-VM instance | An enum root in this VM session |

The separation means that the physical discriminant (used in bytecode jump tables)
can be compact and dense, while the VM identity (used in registry lookups) is
globally unique, and the semantic identity (used in incremental analysis) is
stable across recompilation.

---

## Enum root registration — `RuntimeEnumDescriptor`

Before variants can be registered, the enum root must be registered:

```rust
pub struct RuntimeEnumDescriptor {
    pub semantic_owner:  DeclarationId,
    pub runtime_id:      RuntimeEnumId,
    pub root_class:      ClassId,         // the abstract base class
    pub representation:  RuntimeAdtRepresentation,
    pub variants:        Vec<RuntimeVariantId>,
}
```

`RuntimeAdtRepresentation` is one of:
- `General` — standard representation (all enums except Option).
- `NativeOption` — core `Option<T>` uses the `Value` depth field for `Some`
  and the `None` tag for the empty case. No heap allocation at all.

Registration is idempotent. Calling `register_enum` twice with the same
`DeclarationId` is a no-op that returns the existing `RuntimeEnumId`.
Calling it with the same `DeclarationId` but a *different* root class or
representation is a `ConflictingEnumRegistration` error.

---

## Variant registration — `RuntimeVariantDescriptor`

```rust
pub struct RuntimeVariantDescriptor {
    pub semantic_id:   VariantId,            // static identity from phalcom-semantic
    pub runtime_id:    RuntimeVariantId,     // VM-local identity
    pub enum_id:       RuntimeEnumId,        // parent enum
    pub discriminant:  CaseDiscriminant,     // dense physical index within the enum
    pub shape:         RuntimeVariantShape,  // Singleton or Constructor
    pub payload_arity: u16,                  // number of fields (0 for singletons)
    pub layout:        Option<ProductLayoutId>, // None for singletons
    pub behavior_class: ClassId,             // variant's own class in the hierarchy
    pub singleton:     Option<Value>,        // pre-built Value for nullary variants
}
```

The `singleton` field is pre-computed at registration time for nullary variants,
so the VM can push `Value::adt_singleton(runtime_id)` without any computation.

---

## Variant shapes

### Singleton variants (no fields)

```
enum Color { Red }
```

A singleton variant carries no payload. Its runtime representation is:

```
Value { payload: RuntimeVariantId as u32 as u64, meta: AdtSingleton as u64 }
```

This is `Value::adt_singleton(variant_id)`. The `is_adt_singleton()` check
tests tag `== AdtSingleton && some_depth == 0`. Pattern matching on a singleton
variant is a single tag+payload comparison — no heap dereference.

### Constructor variants (one or more fields)

```
enum Option<T> { Some(value: T) }
```

A constructor variant allocates an `AdtCaseObject` on the heap:

```rust
// phalcom-core/src/heap/adt.rs
pub struct AdtCaseObject {
    pub variant: RuntimeVariantId,  // which variant this instance is
    pub storage: ProductStorage,    // packed field values
}
```

The calling `Value` is `Value::obj(obj_ref)` with tag `Obj`.

---

## The native `Option` special case

Core `Option<T>` uses neither `AdtSingleton` nor `AdtCaseObject`. Instead it
exploits the `some_depth` field in `Value`:

| Phalcom value | Representation |
|---|---|
| `Option.None` | `Value::none()` — tag `None`, depth 0 |
| `Option.Some(x)` | `x.with_some_depth(x.some_depth_raw() + 1)` |
| `Option.Some(Some(x))` | `x.with_some_depth(depth + 2)` |

This makes `Some(42)` an immediate value — no heap allocation at all — and
unwrapping is a bitfield operation on the `meta` word. The `NativeOption`
representation strategy in `RuntimeAdtRepresentation` marks which enum uses
this path.

The bound variant IDs for `Some` and `None` are stored in
`RuntimeAdtRegistry::native_option: Option<NativeOptionVariantIds>`:

```rust
pub struct NativeOptionVariantIds {
    pub some: RuntimeVariantId,
    pub none: RuntimeVariantId,
}
```

---

## `RuntimeAdtRegistry` — the central registry

```rust
pub struct RuntimeAdtRegistry {
    enums:                    Vec<RuntimeEnumDescriptor>,
    variants:                 Vec<RuntimeVariantDescriptor>,
    enum_by_declaration:      HashMap<DeclarationId, RuntimeEnumId>,
    variant_by_semantic_id:   HashMap<VariantId, RuntimeVariantId>,
    variant_by_behavior_class: HashMap<ClassId, RuntimeVariantId>,
    enum_by_root_class:       HashMap<ClassId, RuntimeEnumId>,
    native_option:            Option<NativeOptionVariantIds>,
}
```

Key lookup methods:

```rust
registry.enum_by_declaration(decl)   → Option<RuntimeEnumId>
registry.variant_by_semantic(vid)    → Option<RuntimeVariantId>
registry.variant_by_class(class)     → Option<RuntimeVariantId>
registry.enum_by_root_class(class)   → Option<RuntimeEnumId>
registry.enum_descriptor(id)         → Option<&RuntimeEnumDescriptor>
registry.variant_descriptor(id)      → Option<&RuntimeVariantDescriptor>
```

---

## Pattern matching

Pattern matching on an ADT value dispatches on:

1. **Tag check**: is the `Value` an `Obj` (heap case), an `AdtSingleton`
   (nullary case), or `None` (native Option)?
2. **Discriminant jump**: for heap cases, read the `variant` field of
   `AdtCaseObject` and look up the `CaseDiscriminant` in the variant descriptor.
3. **Payload destructuring**: for constructor cases, use `ProductStorage::load_component`
   to extract fields at their logical indices.

The compiler emits `Bytecode::MatchVariant(discriminant)` instructions that
implement step 2. Singleton matches reduce to step 1 only.

---

## GC tracing

For `Object::AdtCase(adt)`:
1. Look up `adt.storage.layout` in the layout registry.
2. Iterate `layout.value_slot_offsets()`.
3. Visit each two-word `Value` at the given offset.

For `AdtSingleton` values, no GC work is needed — the payload is a plain u32.

The registry itself holds `ClassId` handles for both enum root classes and
variant behavior classes. `enumerate_class_roots` provides a push-based
iterator for the GC roots scanner.

---

## See also

- [01-value-representation.md](01-value-representation.md) — `AdtSingleton` tag,
  `some_depth` for Option
- [02-product-layout-and-storage.md](02-product-layout-and-storage.md) — shared
  product storage engine
- [TDR-0009](../../decisions/accepted/0009-tagged-value-enum.md) — tagged Value design
- [TDR-0037](../../decisions/accepted/0037-option-bootstrap-formalization-and-defer-niche-encoding.md) — Option representation
- [spec: associated-lookup-surface.md](../../spec/associated-lookup-surface.md)
- [spec: nullary-adt-variant.md](../../spec/nullary-adt-variant.md)

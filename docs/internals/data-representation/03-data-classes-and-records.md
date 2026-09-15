# 03 — Data Classes and Record-Shaped Types

**Sources:**
- `phalcom-core/src/data.rs` — `RuntimeDataDescriptor`, `RuntimeDataRegistry`
- `phalcom-core/src/heap/data.rs` — `DataObject` (heap payload)
- `phalcom-core/src/vm/data.rs` — VM registration and getter synthesis
- `phalcom-core/src/compiler/lib/data_decl.rs` — compiler lowering
- `phalcom-ast/src/ast.rs` — `DataDef` AST node

A `data` declaration in Phalcom defines a record-shaped type: a named,
immutable product with named fields. This document covers the full lifecycle
from source syntax to runtime heap object.

---

## Source syntax

```phalcom
data Point(x: Int, y: Int)
data Color(red: Int, green: Int, blue: Int)
data Wrapper(inner)           // untyped field
data Singleton                // no fields — a nullary data type
```

Each `data` declaration introduces:
- A **type name** (e.g. `Point`).
- Zero or more **component fields** with optional type annotations.
- Auto-synthesized **getter methods** for each field.

---

## AST representation

The parser produces a `DataDef` node:

```
DataDef {
    name: "Point",
    shape: DataShape::Record([
        DataComponent { local_name: "x", type_annotation: Some(…) },
        DataComponent { local_name: "y", type_annotation: Some(…) },
    ]),
}
```

`DataShape` distinguishes tuple-position components from labeled (record)
components. Both forms ultimately produce the same field list for the product
engine.

---

## Semantic layer — `DataComponentId` and `DeclarationId`

The semantic analyzer assigns stable identities:

- `DeclarationId` — identifies the `data` declaration within a module
  (module path + declaration name).
- `DataComponentId` — identifies one field within a declaration
  (`DeclarationId` + logical index).

These are static, analysis-time identities that survive across incremental
re-analyses and are used as stable keys in the runtime registry.

---

## Compiler lowering

The compiler emits `Bytecode::Data` and `Bytecode::FinalizeData` instructions.
The source for this is `phalcom-core/src/compiler/lib/data_decl.rs`:

```
compile_data(data_def) →
  1. Look up DataDeclarationLoweringSpec from semantic lowering info
     (or synthesize a default spec for standalone compiles).
  2. Emit Bytecode::Data(spec_index)   ← registers the data type at load time
  3. Emit Bytecode::FinalizeData       ← publishes the global binding
```

The `DataDeclarationLoweringSpec` carries:
- `owner: DeclarationId`
- `layout: ProductLayoutSpec` — component count and slot representations
- `components: [DataComponentLoweringSpec]` — logical index + local name for
  each field

This spec is embedded in the chunk and re-used at execution time to register
the data class with the VM.

---

## VM registration — `RuntimeDataRegistry`

When the VM executes `Bytecode::Data`, it calls `VM::register_data_from_spec`:

```
register_data_from_spec(spec) →
  1. Check if already registered (idempotent).
  2. Allocate a fresh ClassObject (behavior class) for the data type.
  3. Build the ProductLayout from the spec's ProductLayoutSpec.
  4. Register the layout in ProductLayoutRegistry → get ProductLayoutId.
  5. Call RuntimeDataRegistry::register(owner, class_id, None, layout_id)
     → get RuntimeDataDescriptorId.
  6. Synthesize getter methods: for each component, emit a tiny bytecode
     chunk [GetLocal(0), GetDataComponent(logical_index), Return] and
     install it as a getter method on the behavior class.
```

The result is a fully-initialized entry in `RuntimeDataRegistry` and a live
behavior class with getter methods.

### `RuntimeDataDescriptor`

```rust
pub struct RuntimeDataDescriptor {
    pub semantic_owner:  DeclarationId,          // stable static identity
    pub runtime_id:      RuntimeDataDescriptorId, // VM-local integer ID
    pub behavior_class:  ClassId,                 // the synthesized class
    pub exact_type:      Option<RuntimeTypeRef>,  // None until type info is bound
    pub layout:          ProductLayoutId,          // which product layout to use
}
```

### `RuntimeDataRegistry`

The registry is a Vec of descriptors plus three lookup maps:

- By `(DeclarationId, Option<RuntimeTypeRef>)` — finds a descriptor by owner + type.
- By `ClassId` — finds a descriptor by its behavior class (used for pattern matching).
- Linear scan by `DeclarationId` — finds a descriptor by declaration alone.

Registration is idempotent: calling `register` twice with the same key returns
the existing `RuntimeDataDescriptorId`.

---

## Nullary data classes — `ValueTag::DataSingleton`

If a `data` declaration has no fields (zero arity), no heap allocation is ever
needed. The VM assigns a `RuntimeDataDescriptorId` and represents the singleton
value as:

```
Value { payload: descriptor_id as u64, meta: ValueTag::DataSingleton as u64 }
```

This is the `data_singleton(d: RuntimeDataDescriptorId)` constructor. The
`is_data_singleton()` and `as_data_singleton()` accessors check the tag and
zero depth.

---

## Positive-arity instances — `DataObject` on the heap

When a `data` class has one or more fields and is constructed with values, the
result is a heap-allocated `DataObject`:

```rust
// phalcom-core/src/heap/data.rs
pub struct DataObject {
    pub descriptor: RuntimeDataDescriptorId,  // which data type this is
    pub storage:    ProductStorage,            // packed field values
}
```

The `Heap::alloc_data(descriptor, storage)` method wraps this in
`Object::Data(Box::new(DataObject { … }))` and inserts it into the arena,
returning an `ObjRef`. The calling `Value` is then `Value::obj(obj_ref)` with
tag `Obj`.

### Construction flow

```
Source: Point.new(x: 1, y: 2)
  │
  ▼ Compiler
Bytecode: Push(1), Push(2), ConstructData(descriptor_index, component_count)
  │
  ▼ VM execution
1. Pop component values from stack in order: [1, 2]
2. Resolve RuntimeDataDescriptorId for this descriptor
3. Look up ProductLayout from registry
4. Build ProductStorage::from_values(layout_id, layout, &components)
5. Heap::alloc_data(descriptor_id, storage) → ObjRef
6. Push Value::obj(obj_ref)
```

---

## Field access — `GetDataComponent`

The VM instruction `Bytecode::GetDataComponent(logical_index)` pops a value,
asserts it is a `Data` heap object, and returns the field at `logical_index`:

```
GetDataComponent(idx):
  obj_ref = pop().as_obj()
  data_obj = heap.get(obj_ref) as Object::Data
  layout = product_layouts.layout(data_obj.storage.layout)
  value = data_obj.storage.load_component(layout, idx)
  push(value)
```

Getter methods synthesized at registration time wrap this single instruction.
A user-visible call `point.x` resolves to the getter method, which executes
`GetLocal(0); GetDataComponent(0); Return`.

---

## GC tracing

The GC tracer visits `Object::Data(data_obj)` by:
1. Looking up `data_obj.storage.layout` in the layout registry.
2. Calling `layout.value_slot_offsets()`.
3. For each offset, reading the two-word `Value` and visiting its `ObjRef` if
   the tag is `Obj`.

Scalar slots (Int64, Float64, Bool, Symbol) are not visited — they contain no
heap references.

---

## Equality and hashing

Data class instances do not automatically implement structural equality.
A `Point` instance uses identity equality (`===`) unless the user defines `==`
on the class. This mirrors the general object model where `==` is a method.

If the user defines `==` for a data class, the typical implementation compares
individual fields via their getters.

---

## See also

- [02-product-layout-and-storage.md](02-product-layout-and-storage.md) — the
  shared storage engine
- [04-adt-enum-variants.md](04-adt-enum-variants.md) — similar design for ADT
  constructor payloads
- [TDR-0010](../../decisions/accepted/0010-static-instance-slot-layout.md) — static
  slot layout decisions

# 06 — Code Walkthrough and Pipeline

This document traces three concrete examples end-to-end through the Phalcom
compilation and execution pipeline — one `data` class, one `enum` variant, and
one `tuple`. The goal is to show exactly which code is called at each stage and
what state changes result.

---

## Stage overview

```
┌─────────────────┐
│ Source (.ph)    │  Human-written Phalcom code
└────────┬────────┘
         │ phalcom-ast  (lexer + parser)
         ▼
┌─────────────────┐
│ AST             │  DataDef, EnumDef, TupleExpr, …
└────────┬────────┘
         │ phalcom-semantic  (name resolution, type inference)
         ▼
┌─────────────────┐
│ Semantic DB     │  DeclarationId, VariantId, ProductLayoutSpec
└────────┬────────┘
         │ phalcom-core / compiler  (bytecode emission)
         ▼
┌─────────────────┐
│ Chunk           │  Bytecode + embedded lowering specs
└────────┬────────┘
         │ phalcom-core / vm  (execution)
         ▼
┌─────────────────┐
│ VM State        │  Registry entries, Heap objects
└─────────────────┘
         │ phalcom-core / gc  (tracing collector)
         ▼
┌─────────────────┐
│ GC Trace        │  Live reference graph, object reclamation
└─────────────────┘
```

---

## Example 1: `data Point(x: Int, y: Int)`

### 1. Parse

The parser produces:

```
DataDef {
    name: "Point",
    shape: DataShape::Record([
        DataComponent { local_name: "x", type_annotation: Some(IntType) },
        DataComponent { local_name: "y", type_annotation: Some(IntType) },
    ]),
    range: …,
}
```

### 2. Semantic analysis

The semantic analyzer assigns:

```
DeclarationId { module: <module-id>, name: "Point" }
DataComponentId { owner: …, logical_index: 0 }   // "x"
DataComponentId { owner: …, logical_index: 1 }   // "y"
```

It produces a `DataDeclarationLoweringSpec`:

```
DataDeclarationLoweringSpec {
    owner: DeclarationId("Point"),
    layout: ProductLayoutSpec {
        components: [
            ProductComponentSpec { logical_index: 0, repr: Int64 },
            ProductComponentSpec { logical_index: 1, repr: Int64 },
        ]
    },
    components: [
        DataComponentLoweringSpec { id: …, local_name: "x", logical_index: 0 },
        DataComponentLoweringSpec { id: …, local_name: "y", logical_index: 1 },
    ],
}
```

### 3. Compile

`Compiler::compile_data` emits:

```
Bytecode::Data(spec_index)     // spec_index refers to the embedded spec
Bytecode::FinalizeData
Bytecode::StoreGlobal("Point") // publishes the global binding
```

### 4. VM execution — registration

When `Bytecode::Data(spec_index)` executes, `VM::register_data_from_spec` runs:

```
1. ClassObject::bare("Point") allocated → ClassId(N)
2. ProductLayoutSpec::build_layout() →
     ProductLayout {
         word_len: 2,
         components: [
             ComponentLayout { logical_index: 0, word_offset: 0, repr: Int64 },
             ComponentLayout { logical_index: 1, word_offset: 1, repr: Int64 },
         ],
         value_slot_offsets: []   // no Value slots — both are Int64
     }
3. ProductLayoutRegistry::register(layout) → ProductLayoutId(M)
4. RuntimeDataRegistry::register(
       semantic_owner: DeclarationId("Point"),
       behavior_class: ClassId(N),
       exact_type: None,
       layout: ProductLayoutId(M),
   ) → RuntimeDataDescriptorId(K)
5. Synthesize getter "x":
     Chunk: [GetLocal(0), GetDataComponent(0), Return]
     Install as getter method on ClassId(N)
6. Synthesize getter "y":
     Chunk: [GetLocal(0), GetDataComponent(1), Return]
     Install as getter method on ClassId(N)
```

### 5. VM execution — construction

When user code calls `Point.new(x: 1, y: 2)`:

```
Stack before: [recv=ClassObj(Point), 1, 2]

1. Pop component values: [Value::int(1), Value::int(2)]
2. Resolve RuntimeDataDescriptorId(K) for "Point"
3. Retrieve ProductLayout(M) from registry
4. ProductStorage::from_values(M, layout, &[int(1), int(2)])
     → words = [1u64, 2u64]
5. Heap::alloc_data(K, storage) → ObjRef(J)
6. Push Value::obj(ObjRef(J))
```

### 6. Field access

`point.x` sends the getter selector, which executes:

```
GetLocal(0)          // push the receiver (the Point object)
GetDataComponent(0)  // load component at logical_index 0
                     //   → ProductStorage::load_component(layout, 0)
                     //   → words[0] as i64 → Value::int(1)
Return               // return Value::int(1)
```

### 7. GC trace

When the GC traces `Object::Data(data_obj)` for this Point:

```
layout = product_layouts.layout(data_obj.storage.layout)
value_slot_offsets = layout.value_slot_offsets()  // → [] (empty — no Value slots)
// nothing to trace — both fields are Int64 scalars
```

---

## Example 2: `enum Option<T> { Some(value: T); None }`

### Registration (simplified — the VM bootstraps Option specially)

```
1. RuntimeAdtRegistry::register_enum_with_representation(
       semantic_owner: DeclarationId("Option"),
       root_class: ClassId(option_class),
       representation: NativeOption,
   ) → RuntimeEnumId(0)

2. register_variant(VariantId("Some"), …, shape: Constructor, payload_arity: 1)
     → RuntimeVariantId(0)
   singleton = None (constructor has no pre-built singleton)

3. register_variant(VariantId("None"), …, shape: Singleton, payload_arity: 0)
     → RuntimeVariantId(1)
   singleton = Some(Value::none())   // pre-built None value
```

### Construction of `Some(42)`

Because `Option` uses `NativeOption` representation:

```
Value::int(42).with_some_depth(1)
  → payload = 42u64 as i64 bits
  → meta = (Int tag) | (1u32 << 8)   // depth = 1
```

No heap allocation. The resulting value carries tag `Int`, depth 1.

### Construction of `None`

```
Value::none()
  → payload = 0
  → meta = None tag (7)
```

Also no heap allocation.

### Accessing `Some(42)`

The VM instruction `UnwrapSome` (or equivalent):

```
value = pop()           // Value with depth > 0
inner = value.without_some_wrappers()  // strips one depth level
push(inner)             // Value::int(42) with depth 0
```

### Pattern matching

```
match opt {
  Some(v) => …
  None    => …
}
```

Compiles to roughly:

```
// Is it None?
if value.tag() == None → jump to None branch
// Is it Some? (depth > 0)
if value.some_depth_raw() > 0 → jump to Some branch, bind value.without_some_wrappers()
```

---

## Example 3: `(1, label: "hello")`

### Parse

```
TupleExpr {
    positionals: [IntLiteral(1)],
    labeled: [(label: "label", value: StringLiteral("hello"))],
}
```

### Compile

```
Push(Value::int(1))
Push(Value::obj(…))    // heap-allocated StringObject for "hello"
Symbol("label")        // push the label symbol
MakeTuple { positional_count: 1, labeled_count: 1 }
```

### VM execution

`MakeTuple` (or equivalent):

```
1. Pop labeled entries in reverse: [(Symbol("label"), Value::obj(str_ref))]
2. Pop positionals: [Value::int(1)]
3. Call finish_tuple(vm, positionals=[int(1)], labeled=[(label, str_ref)])
4. unique(labeled) → ok (one entry)
5. Not empty → proceed
6. Build values: [int(1), str_ref]   // positionals first, then labeled values
   Build labels: [Symbol("label")]   // one label for the labeled suffix
7. Heap::alloc_tuple_nonempty(
       values: Box<[int(1), str_ref]>,
       labels: Box<[Symbol("label")]>
   ) → ObjRef(T)
8. Push Value::obj(ObjRef(T))
```

### GC trace

```
Object::Tuple(tuple)
  for value in tuple.values():
    if value.tag() == Obj:
      trace(value.as_obj())
```

The GC visits both `int(1)` (tag `Int` — skip) and `str_ref` (tag `Obj` — trace
the StringObject).

---

## Stage-by-stage summary

| Stage | Tool | Data produced | Key files |
|---|---|---|---|
| Parse | phalcom-ast | `DataDef`, `EnumDef`, `TupleExpr` | `phalcom-ast/src/ast.rs` |
| Semantic | phalcom-semantic | `DeclarationId`, `VariantId`, `ProductLayoutSpec` | `phalcom-semantic/src/` |
| Compile | phalcom-core compiler | `Bytecode::Data`, `Bytecode::FinalizeData`, `Bytecode::Tuple`, etc. | `phalcom-core/src/compiler/` |
| Register | phalcom-core vm | `RuntimeDataDescriptor`, `RuntimeVariantDescriptor`, behavior classes | `phalcom-core/src/vm/data.rs`, `phalcom-core/src/vm/adt.rs` |
| Construct | phalcom-core vm | `DataObject`, `AdtCaseObject`, `TupleObject` on the heap | `phalcom-core/src/heap/` |
| GC | phalcom-core gc | Live reference graph | `phalcom-core/src/heap/trace.rs` |

---

## See also

- [01-value-representation.md](01-value-representation.md)
- [02-product-layout-and-storage.md](02-product-layout-and-storage.md)
- [03-data-classes-and-records.md](03-data-classes-and-records.md)
- [04-adt-enum-variants.md](04-adt-enum-variants.md)
- [05-tuples-and-positional-shapes.md](05-tuples-and-positional-shapes.md)

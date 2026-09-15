# Product Representation Pipeline — Worked VM Examples

**Status:** Explanatory worked examples — non-normative  
**Specification domain:** `specs/vm/examples/`  
**Repository evidence baseline:** `aureat/phalcom-lang@1fc1e8449e6dd77fc533455bcb056fe9b00290b9`  
**Normative companions:** `../products.md`, `../values-and-objects.md`, `../bytecodes.md`, `../execution-model.md`

---

## 1. Purpose

This document is a code-oriented walkthrough of how representative Phalcom products travel from source syntax through semantic analysis, lowering, runtime metadata, physical layout, construction, observation, and garbage collection.

It replaces the older `docs/internals/data-representation/06-code-walkthrough-and-pipeline.md`.

The older walkthrough captured the right pedagogical idea but described an earlier product representation in which Tuple and Record instances carried direct per-instance arrays. The current runtime has converged materialized Tuple, Record, `data`, and enum payloads onto the shared `ProductLayout` / `ProductStorage` substrate, with separate shape and descriptor metadata.

This document is deliberately **non-normative**. Its examples illustrate the current reference implementation. The normative contracts are in `products.md` and the other VM specifications.

The examples cover:

1. a nominal `data` product;
2. a statically shaped Tuple;
3. a Record whose presentation order differs from canonical storage order;
4. an ordinary enum constructor;
5. a GADT constructor and branch refinement boundary;
6. native `Option`;
7. empty-product normalization;
8. product GC and scalar replacement.

---

# Part I — Pipeline overview

## 2. The complete path

A product-oriented source construct may pass through these layers:

```text
Phalcom source
     |
     v
phalcom-ast
     |
     | syntax nodes
     v
phalcom-semantic
     |
     | declaration identities
     | component identities
     | exact types
     | GADT evidence
     | product layout choices
     v
semantic lowering metadata
     |
     | ProductLayoutSpec
     | anonymous product construction specs
     | data/variant lowering specs
     | runtime type recipes
     v
phalcom-core compiler
     |
     | bytecode / embedded lowering specs
     | product optimization plan
     v
VM execution
     |
     +------------------------+
     |                        |
     v                        v
metadata registries      construction boundary
     |                        |
     |                        v
     |                  ProductLayout
     |                        |
     |                  ProductStorage
     |                        |
     +-----------+------------+
                 |
                 v
           runtime Value
                 |
       +---------+---------+
       |                   |
       v                   v
   observation             GC
```

Not every source product reaches every stage.

A scalar-replaced product may never become a heap object.

A nullary value may be immediate.

`Option.Some` normally bypasses `ProductStorage` entirely.

---

## 3. The five identities to keep separate

Before looking at concrete examples, it is useful to keep five axes distinct.

```text
semantic identity
    DeclarationId / VariantId / structural type

runtime metadata identity
    RuntimeDataDescriptorId / RuntimeVariantId /
    RuntimeAnonymousProductDescriptorId

shape identity
    ProductShapeId

physical layout identity
    ProductLayoutId

materialized backing identity
    ObjRef
```

Only some categories use all five.

For transparent products, `ObjRef` is never the language value identity.

---

# Part II — Example 1: nominal `data`

## 4. Source

Consider:

```phalcom
data Point(x: Int, y: Int)

const p = Point.new(x: 10, y: 20)
```

`Point` is a nominal transparent immutable product.

Its declaration identity and exact nominal type matter.

The physical storage of its two components is an implementation choice.

---

## 5. Semantic analysis

The semantic layer assigns stable declaration/component identities conceptually equivalent to:

```text
DeclarationId(Point)

DataComponentId(Point, logical_index=0)  # x
DataComponentId(Point, logical_index=1)  # y
```

Because both components are statically `Int`, lowering can choose:

```text
x -> Int64
y -> Int64
```

rather than two full `Value` slots.

The resulting lowering information contains a layout specification conceptually like:

```rust
ProductLayoutSpec {
    components: [
        ProductComponentSpec {
            logical_index: 0,
            repr: Int64,
        },
        ProductComponentSpec {
            logical_index: 1,
            repr: Int64,
        },
    ],
}
```

The semantic layer also retains the exact nominal type information needed for runtime typing/reification.

---

## 6. Registration

When the data declaration becomes live in a VM, the runtime establishes:

```text
semantic owner       = DeclarationId(Point)
behavior class       = ClassId(Point)
physical layout      = ProductLayoutId(...)
exact nominal type   = RuntimeTypeRef(Point)
runtime descriptor   = RuntimeDataDescriptorId(...)
```

Conceptually:

```rust
RuntimeDataDescriptor {
    semantic_owner: Point,
    runtime_id: ...,
    behavior_class: ...,
    exact_type: Some(PointRuntimeType),
    layout: point_layout,
}
```

The registry is keyed by the semantic owner and exact runtime type.

Repeated equivalent registration reuses the descriptor.

---

## 7. Physical layout

`ProductLayoutSpec::build_layout()` produces:

```text
logical component 0:
    repr        = Int64
    word_offset = 0

logical component 1:
    repr        = Int64
    word_offset = 1

word_len = 2
value_slot_offsets = []
```

The complete component payload therefore occupies:

```text
2 * 8 bytes = 16 bytes
```

instead of:

```text
2 * 16-byte Value = 32 bytes
```

for an all-`Value` representation.

No product component requires GC tracing.

---

## 8. Construction

When `Point.new(x: 10, y: 20)` is materialized, the construction path is conceptually:

```text
source arguments
    |
    v
[Value::int(10), Value::int(20)]
    |
    v
resolve RuntimeDataDescriptor
    |
    v
resolve ProductLayout
    |
    v
ProductStorage::from_values(...)
    |
    v
words = [10, 20]
    |
    v
DataObject {
    descriptor: point_descriptor,
    storage: ...
}
    |
    v
Value::obj(...)
```

The resulting `ObjRef` is backing storage, not the language value identity of `Point`.

---

## 9. Getter access

A generated getter for `x` can be understood as:

```text
receiver
   |
   v
resolve DataObject
   |
   v
descriptor -> layout
   |
   v
load logical component 0
   |
   v
decode Int64
   |
   v
Value::int(10)
```

The storage layer does not need to know the name `x`.

The mapping:

```text
x -> logical index 0
```

comes from semantic/data metadata.

---

## 10. GC

The Point layout has:

```text
value_slot_offsets = []
```

because both fields are raw scalar `Int64` slots.

Tracing the Point therefore finds no component heap edges.

The `DataObject` itself remains reachable through the ordinary object graph, but there are no product-value references to follow inside its storage.

---

## 11. Exact sameness

If another Point is independently materialized:

```phalcom
const q = Point.new(x: 10, y: 20)
```

`p` and `q` may have different backing `ObjRef`s.

For a transparent data product, that difference must not determine language-level `===`.

The exact nominal type and recursively exact component values determine product sameness.

---

# Part III — Example 2: a statically shaped Tuple

## 12. Source

Consider:

```phalcom
const t = (1, label: "hello")
```

This has:

```text
positional lane:
    [1]

labeled lane:
    [label: "hello"]
```

The Tuple shape is ordered.

---

## 13. Semantic/lowering shape

The static shape is conceptually:

```rust
TupleProductShape {
    positional_len: 1,
    labels: [Symbol("label")],
}
```

The source-order logical components are:

```text
0 -> 1
1 -> "hello"
```

The static product lowering can also carry:

- a `ProductLayoutSpec`;
- a runtime type recipe for the exact structural tuple type.

If the compiler proves the first component is a bare Int, it may choose:

```text
component 0 -> Int64
component 1 -> Value
```

because String is heap-backed and must travel through a full `Value` slot.

---

## 14. Building the Tuple shape

At runtime, static tuple finalization:

1. interns the label symbols;
2. registers the `TupleProductShape`;
3. receives a `ProductShapeId`.

Shape metadata is shared.

The Tuple instance does not carry its own `Box<[Symbol]>`.

---

## 15. Building the layout

For:

```text
[Int64, Value]
```

the layout is:

```text
component 0:
    word_offset = 0
    size = 1 word

component 1:
    word_offset = 1
    size = 2 words

word_len = 3
value_slot_offsets = [1]
```

The buffer occupies 24 bytes.

---

## 16. Building the anonymous descriptor

The VM then interns:

```rust
RuntimeAnonymousProductDescriptor {
    kind: Tuple,
    shape: tuple_shape_id,
    layout: tuple_layout_id,
    exact_type: Some(exact_tuple_type),
    ...
}
```

The descriptor is shared by all materialized tuples with the same runtime descriptor key.

---

## 17. Materialized object

A positive Tuple materializes as:

```rust
TupleObject {
    descriptor: tuple_descriptor_id,
    storage: ProductStorage {
        layout: tuple_layout_id,
        words: [...],
    },
}
```

The Tuple's labels and exact structural type are descriptor metadata.

The component buffer contains only encoded values.

---

## 18. Projection

Tuple index `0` maps directly to logical component `0`.

Loading it:

```text
storage word 0
    |
    v
Int64 decode
    |
    v
Value::int(1)
```

The labeled component at logical index `1` loads the two-word `Value` and reconstructs the string object's `Value`.

---

## 19. GC

The trace map is:

```text
[1]
```

so GC reconstructs only the `Value` beginning at word `1`.

The integer at word `0` is skipped.

If the string `Value` carries an `ObjRef`, that object is traced.

---

# Part IV — Example 3: Record presentation versus canonical storage

## 20. Source

Consider:

```phalcom
const r1 = #{ name: "A", age: 24 }
const r2 = #{ age: 24, name: "A" }
```

The two Records have different presentation order but the same structural key set.

---

## 21. Presentation metadata

For `r1`:

```text
presentation:
    [name, age]
```

For `r2`:

```text
presentation:
    [age, name]
```

This order remains relevant for rendering, iteration, reflection presentation, and expansion.

---

## 22. Canonical logical coordinates

Record storage uses canonical logical coordinates.

For illustration, suppose canonical ordering is:

```text
[age, name]
```

Then the first Record has a mapping like:

```text
presentation index 0: name -> logical 1
presentation index 1: age  -> logical 0
```

or:

```text
presentation_to_logical = [1, 0]
```

The source-order values:

```text
["A", 24]
```

are rearranged into logical storage order:

```text
[24, "A"]
```

The second Record may already arrive in that logical order.

The exact canonical ordering mechanism is metadata/lowering authority; presentation order must not be mistaken for storage order.

---

## 23. Shared physical layout

Both records can use the same layout:

```text
logical 0 (age):
    Int64

logical 1 (name):
    Value
```

and therefore the same `ProductLayoutId`.

They may have different `ProductShapeId`s because presentation metadata differs.

This is an important example of:

```text
same physical layout
same structural type
different presentation shape
```

---

## 24. Exact sameness

Despite different presentation order:

```phalcom
r1 === r2
```

is true if their corresponding values are recursively exactly same.

Record `===` resolves by key, not by presentation position.

---

## 25. Rendering

A renderer for `r1` should still be able to produce a presentation preserving:

```text
name, age
```

while a renderer for `r2` may preserve:

```text
age, name
```

It does this by walking presentation labels and translating each presentation coordinate through the shape mapping to logical storage.

Canonical physical storage MUST NOT accidentally reorder user-observable presentation.

---

# Part V — Example 4: ordinary enum constructor

## 26. Source

Consider:

```phalcom
enum Result<T, E> {
    Ok(_ value: T)
    Error(_ error: E)
}
```

and:

```phalcom
const r = Result::Ok(42)
```

`Result` is a nominal closed sum.

`Ok` is an exact constructor variant.

Its payload is a one-component product.

---

## 27. Semantic identities

The semantic layer assigns identities conceptually like:

```text
DeclarationId(Result)
VariantId(Result::Ok(_))
VariantId(Result::Error(_))
```

Those identities survive independently of physical runtime registration.

---

## 28. Runtime registration

The VM establishes:

```text
RuntimeEnumId(Result)
RuntimeVariantId(Ok)
RuntimeVariantId(Error)
```

and per-enum discriminants, for example:

```text
Ok    -> CaseDiscriminant(0)
Error -> CaseDiscriminant(1)
```

These are distinct identity levels.

The numeric values are VM/session implementation details.

---

## 29. Variant descriptor

The `Ok` variant descriptor contains conceptually:

```rust
RuntimeVariantDescriptor {
    semantic_id: VariantId(Result::Ok(_)),
    runtime_id: ...,
    enum_id: ...,
    discriminant: ...,
    shape: Constructor,
    payload_arity: 1,
    layout: Some(...),
    behavior_class: ...,
    singleton: None,
}
```

If `T` is statically represented as Int in the selected lowering, the payload layout may use:

```text
[Int64]
```

---

## 30. Construction

`Result::Ok(42)` conceptually becomes:

```text
variant = RuntimeVariantId(Ok)

payload values:
    [Value::int(42)]

layout:
    component 0 -> Int64

ProductStorage:
    words = [42]

AdtCaseObject:
    variant = RuntimeVariantId(Ok)
    storage = payload storage
```

The exact variant identity is **not** encoded by the integer payload or layout.

---

## 31. Matching

A match on `r` first establishes exact variant identity.

Only after the case is known does payload projection interpret the corresponding product storage.

Conceptually:

```text
is this exact Ok variant?
    |
    +-- no --> try another case
    |
    +-- yes
         |
         v
 project logical payload component 0
         |
         v
 Value::int(42)
```

The exact bytecode sequence belongs to `bytecodes.md`.

---

# Part VI — Example 5: GADT case

## 32. Source

Consider:

```phalcom
enum Expr<T> {
    IntLit(_ value: Int) -> Expr<Int>
    BoolLit(_ value: Bool) -> Expr<Bool>
}
```

and:

```phalcom
const e = Expr::IntLit(42)
```

---

## 33. Static result refinement

The declaration:

```phalcom
IntLit(_ value: Int) -> Expr<Int>
```

establishes the variant-local equality:

```text
T = Int
```

The `BoolLit` variant establishes:

```text
T = Bool
```

This evidence belongs to the semantic type system.

---

## 34. Runtime case representation

`IntLit(42)` still uses the ordinary enum constructor runtime model.

It has:

```text
VariantId(IntLit)
RuntimeVariantId(IntLit)
CaseDiscriminant(IntLit)
behavior class
payload layout [Int64]
payload storage [42]
```

There is no additional runtime object field:

```text
proof = "T = Int"
```

The equality proof is not payload data.

---

## 35. Branch refinement boundary

For a match:

```phalcom
match expr {
    IntLit(value) => ...
    BoolLit(value) => ...
}
```

execution establishes which exact runtime variant was selected.

The semantic checker uses that exact-case fact to introduce the corresponding static equality into the branch.

Thus the boundary is:

```text
runtime:
    exact variant recognized

semantic proof:
    exact variant => branch equality evidence
```

The runtime product representation and the static proof engine cooperate without duplicating proof data in each value.

---

# Part VII — Example 6: native `Option`

## 36. Source

Consider:

```phalcom
const a = Option::Some(42)
const b = Option::None
```

Semantically:

- `Some` is a constructor variant with one payload;
- `None` is a singleton variant.

Physically, core Option is specialized.

---

## 37. Runtime enum registration

The enum root is registered with:

```text
RuntimeAdtRepresentation::NativeOption
```

The runtime also records which `RuntimeVariantId`s correspond to:

```text
Some
None
```

This retains ordinary variant identity metadata even though value representation is specialized.

---

## 38. `Some(42)`

Instead of allocating:

```text
AdtCaseObject {
    variant: Some,
    storage: [42]
}
```

the runtime uses the uniform `Value` metadata:

```text
base tag   = Int
payload    = 42
some_depth = 1
```

The logical one-component product has been optimized away.

---

## 39. `None`

`Option::None` uses the immediate `None` tag.

No `AdtSingleton` heap/value representation is needed.

---

## 40. Nested Option

```phalcom
Some(Some(42))
```

increments the depth again:

```text
tag        = Int
payload    = 42
some_depth = 2
```

This demonstrates the general principle:

> A semantic product payload need not imply `ProductStorage` if the runtime has a sound specialized representation.

---

# Part VIII — Example 7: empty anonymous products

## 41. Empty Tuple

```phalcom
()
```

normalizes to:

```text
Value::unit()
```

No `TupleObject` exists.

---

## 42. Empty Record

The empty structural record product:

```phalcom
#{}
```

also reaches the canonical zero-product representation at the runtime construction boundary.

No positive `RecordObject` exists.

---

## 43. Runtime defensive boundary

Even when the compiler has already normalized an empty source product, runtime product finalizers retain the zero-product check.

This prevents dynamic or alternate construction paths from introducing an invalid empty heap Tuple/Record.

---

# Part IX — Example 8: scalar replacement

## 44. Source

Consider:

```phalcom
const point = Point.new(x: 10, y: 20)
const total = point.x + point.y
```

If `point` does not escape and the compiler proves its projections, a product optimization plan may keep the product virtual.

---

## 45. Materialized execution

Without scalar replacement:

```text
10
20
 |
 v
ProductStorage [10, 20]
 |
 v
DataObject(Point)
 |
 +-> getter x -> 10
 |
 +-> getter y -> 20
```

---

## 46. Virtual execution

With scalar replacement:

```text
virtual Point
    leaf 0 -> local X = 10
    leaf 1 -> local Y = 20

point.x -> local X
point.y -> local Y

X + Y -> 30
```

No data product allocation is required.

---

## 47. Why this is legal

The optimization is legal because `Point` is a transparent value product.

The program cannot observe:

- whether an `ObjRef` was allocated;
- the heap address;
- whether the product was later rematerialized.

If an operation requires a materialized `Value`, the compiler/runtime must construct an equivalent product descriptor/storage representation at that boundary.

---

# Part X — Product GC walkthrough

## 48. Mixed layout

Suppose a product has:

```text
component 0 -> Int64
component 1 -> Value
component 2 -> Bool
component 3 -> Value
```

The layout might be:

```text
component 0:
    word 0

component 1:
    words 1..2

component 2:
    word 3

component 3:
    words 4..5

word_len = 6
value_slot_offsets = [1, 4]
```

---

## 49. Trace

GC does not scan every word as though it were a `Value`.

It uses:

```text
[1, 4]
```

and reconstructs only the two full `Value` slots.

Conceptually:

```text
offset 1:
    Value(payload=words[1], meta=words[2])
    -> trace underlying object edge if present

offset 4:
    Value(payload=words[4], meta=words[5])
    -> trace underlying object edge if present
```

The scalar words:

```text
0
3
```

are skipped.

---

## 50. Option-wrapped object edge

A `Value` slot may contain:

```text
Some(heap_object)
```

whose surface accessor does not expose the base object directly.

GC still follows the embedded object reference using the GC-specific `Value` edge extraction rules from `values-and-objects.md`.

Product packing must not obscure that edge.

---

# Part XI — Descriptor and layout sharing

## 51. Shared layouts

Consider:

```phalcom
data Point(x: Int, y: Int)
data Size(width: Int, height: Int)
```

Both may use the same physical layout:

```text
[Int64, Int64]
```

Therefore they may share one `ProductLayoutId`.

They remain different nominal types because their data descriptors have different semantic owners and exact types.

---

## 52. Shared anonymous descriptor

Two static Tuple values with the same:

- Tuple shape;
- physical layout;
- exact structural runtime type;

may share one `RuntimeAnonymousProductDescriptorId`.

Their backing component buffers remain separate unless another optimization safely shares them.

---

## 53. Same exact Record type, different presentation

Two Records with the same key set may share:

- exact structural runtime type;
- physical layout;

while having different `ProductShapeId`s because presentation order differs.

This is expected, not an inconsistency.

---

# Part XII — What each layer is authoritative for

## 54. Authority table

| Layer | Authority |
|---|---|
| AST | written source shape |
| semantic DB | declaration/component/variant identities and static types |
| GADT proof engine | case equalities and branch refinement |
| lowering specs | runtime construction recipe and slot representation choice |
| `ProductShapeRegistry` | interned anonymous coordinate/presentation shapes |
| `ProductLayoutRegistry` | interned physical slot layouts |
| anonymous descriptor registry | Tuple/Record shape + layout + exact type join |
| data registry | nominal data descriptor identity |
| ADT registry | enum/variant runtime identity |
| `ProductStorage` | encoded component payload |
| heap `Object` | materialized backing |
| `Value` | universal VM carrier |

No lower layer should silently become authority for a higher-layer semantic fact merely because it has enough incidental information to guess.

---

# Part XIII — Common mistakes when reading the implementation

## 55. Mistake: “same layout means same type”

False.

```text
[Int64, Int64]
```

may represent many unrelated nominal/structural products.

---

## 56. Mistake: “Tuple labels live in each TupleObject”

No longer true.

Current Tuple objects carry an anonymous descriptor plus `ProductStorage`; shared shape metadata owns labels.

---

## 57. Mistake: “Record storage is source order”

Not necessarily.

Record presentation order and logical/storage order are separate.

---

## 58. Mistake: “a GADT needs a special runtime object”

False.

GADT specialization is static type evidence over the ordinary enum/variant runtime model.

---

## 59. Mistake: “an enum variant is identified by its payload layout”

False.

Variant identity comes from semantic/runtime variant descriptors.

---

## 60. Mistake: “a `data` value's ObjRef is its exact identity”

False for transparent data products.

Allocation is representation.

---

## 61. Mistake: “every product is heap allocated”

False.

Examples of non-materialized/specialized products include:

- Unit;
- `DataSingleton`;
- `AdtSingleton`;
- native `Option`;
- scalar-replaced virtual products.

---

# Part XIV — Source map for repository reading

## 62. Product substrate

Start here:

```text
phalcom-core/src/product/mod.rs
phalcom-core/src/product/layout.rs
phalcom-core/src/product/storage.rs
phalcom-core/src/product/registry.rs
phalcom-core/src/product/shape.rs
phalcom-core/src/product/anonymous.rs
phalcom-core/src/product/view.rs
```

---

## 63. Materialized product objects

```text
phalcom-core/src/heap/tuple.rs
phalcom-core/src/heap/record.rs
phalcom-core/src/heap/data.rs
phalcom-core/src/heap/adt.rs
```

---

## 64. Nominal metadata

```text
phalcom-core/src/data.rs
phalcom-core/src/adt.rs
```

---

## 65. VM construction and observation

```text
phalcom-core/src/vm/data.rs
phalcom-core/src/vm/adt.rs
phalcom-core/src/primitive/tuple.rs
phalcom-core/src/vm/gc.rs
```

---

## 66. Semantic/GADT layer

```text
phalcom-semantic/src/
phalcom-semantic/src/checker/gadt_proof.rs
docs/spec/adts.md
```

---

## 67. Optimization layer

```text
phalcom-core/src/compiler/lib/product_opt.rs
```

---

# Part XV — End-to-end summary

## 68. Data

```text
data declaration
 -> semantic DeclarationId/component identities
 -> ProductLayoutSpec
 -> ProductLayoutId
 -> RuntimeDataDescriptor
 -> ProductStorage
 -> DataObject or DataSingleton
 -> Value
```

---

## 69. Tuple

```text
tuple expression
 -> structural tuple type + shape
 -> ProductLayoutSpec
 -> ProductShapeId
 -> ProductLayoutId
 -> RuntimeAnonymousProductDescriptor
 -> ProductStorage
 -> TupleObject
 -> Value
```

Empty Tuple stops at Unit.

---

## 70. Record

```text
record expression
 -> presentation labels + structural key type
 -> canonical logical mapping
 -> ProductLayoutSpec
 -> ProductShapeId
 -> ProductLayoutId
 -> RuntimeAnonymousProductDescriptor
 -> ProductStorage in logical order
 -> RecordObject
 -> Value
```

Empty Record stops at the canonical zero product.

---

## 71. ADT

```text
enum declaration
 -> semantic enum/VariantId
 -> RuntimeEnumId/RuntimeVariantId
 -> CaseDiscriminant
 -> optional payload ProductLayout
 -> singleton immediate or AdtCaseObject
 -> Value
```

---

## 72. GADT

```text
GADT variant declaration
 -> ordinary variant identities
 -> ordinary payload layout/storage
 + static result-type equality evidence
 -> exact runtime case recognition
 -> semantic branch refinement
```

No per-value proof payload is required.

---

## 73. Option

```text
Option semantics
 -> RuntimeAdtRepresentation::NativeOption
 -> Some: Value.some_depth
 -> None: ValueTag::None
```

The general product payload is eliminated by specialization.

---

## 74. Final perspective

The current product architecture is built around deliberate separation:

```text
what the value means
what coordinates it has
what exact type it has
how it behaves
how it is laid out
whether it is materialized
where a materialization happens to live
```

The same `ProductStorage` machinery can therefore serve Tuple, Record, `data`, and enum payloads without turning them into the same language construct.

That separation is what makes packed layouts, precise GC, scalar replacement, GADT refinement, exact runtime typing, and future representation optimization composable rather than mutually constraining.

# Phalcom VM Products

**Status:** Draft 0.1 — authoritative implementation-internals specification  
**Specification domain:** `specs/vm/`  
**Implementation:** Phalcom reference runtime (`phalcom-core`)  
**Repository evidence baseline:** `aureat/phalcom-lang@1fc1e8449e6dd77fc533455bcb056fe9b00290b9`  
**Companion specifications:** `values-and-objects.md`, `bytecodes.md`, `execution-model.md`  
**Language-level companions:** ADT/GADT, tuple/record, generic-type, pattern-matching, and data-declaration specifications

---

## 1. Scope

This document specifies the **product model** of the Phalcom reference virtual machine.

In this specification, *product* is an implementation term for an immutable collection of logically indexed components whose values may be represented, stored, projected, traced, scalar-replaced, and reified through a common runtime substrate. The product machinery underlies multiple language categories without collapsing those categories into one semantic type.

This document is authoritative for:

- product categories and their runtime boundaries;
- product component coordinates;
- physical slot representation;
- `ProductLayoutSpec`, `ProductLayout`, and `ProductLayoutId`;
- `ProductStorage`;
- product layout interning;
- anonymous tuple/record shape metadata;
- tuple and record descriptors;
- nominal `data` descriptors and values;
- enum roots, variants, case identities, and payload representation;
- GADT interaction with runtime variant/product representation;
- singleton and zero-argument constructor distinctions;
- the native `Option` exception to general ADT payload representation;
- exact retained runtime types attached to product descriptors;
- transparent-product value identity;
- representation-independent exact sameness;
- garbage-collector tracing of packed products;
- scalar replacement, materialization, rematerialization, and optimization freedom;
- the boundaries between semantic identity, runtime metadata identity, shape identity, physical layout identity, and backing allocation.

This document does **not** redefine the uniform `Value` representation, `ObjRef`, object-arena semantics, or the complete `===` machinery. Those are specified by `values-and-objects.md`. It does, however, define the product-specific facts that those mechanisms consume.

This document also does not replace the source-language specifications for:

- `data` declaration syntax;
- enum/GADT syntax;
- associated variant lookup;
- generic inference;
- match syntax and exhaustiveness;
- record/tuple typing;
- visibility;
- constructor invocation.

Where those language facilities determine required runtime metadata or representation invariants, this document states the VM consequence.

The central rule is:

> **Product semantics are not physical layout. A product category, its semantic identity, its exact runtime type, its logical coordinates, its physical layout, and its materialized heap allocation are separate axes.**

A conforming reference implementation MUST preserve those distinctions.

---

## 2. Normative language

The terms **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, and **MAY** express requirements on the reference implementation.

This specification distinguishes four kinds of statement.

### 2.1 Semantic-runtime invariant

A semantic-runtime invariant is a property that must remain true regardless of storage strategy.

Examples:

- a `data` value is nominal while a Record is structural;
- Tuple label order is semantically significant;
- Record key order is not part of structural identity;
- a GADT result refinement does not create a new runtime product category;
- `ProductLayoutId` is never semantic type identity;
- a backing `ObjRef` is not the value identity of a transparent product.

### 2.2 Physical representation invariant

A physical representation invariant is a concrete property of the current reference runtime on which other VM subsystems rely.

Examples:

- product storage consists of 64-bit words;
- a `Value` slot occupies two words;
- `ProductStorage` carries a `ProductLayoutId`;
- precise tracing uses the layout's `value_slot_offsets`.

Changing such a property requires coordinated review of all dependent VM code and this specification.

### 2.3 Current lowering mechanism

A current lowering mechanism describes how the reference compiler or VM realizes the invariant today.

Examples include:

- semantic lowering producing `ProductLayoutSpec`;
- `finish_tuple_from_spec` registering a structural shape and layout;
- runtime registration synthesizing data getters;
- static product construction retaining an exact `RuntimeTypeRef`.

A future implementation MAY replace a lowering mechanism if the normative product contracts remain true.

### 2.4 Optimization

An optimization changes cost without changing observable product semantics.

Examples include:

- scalar replacement;
- virtual product leaf slots;
- packed scalar slots;
- nested-product flattening;
- rematerialization;
- sharing an immutable backing allocation;
- eliding materialization when `.class`, reflection, or `===` can be answered from retained metadata.

---

# Part I — Product taxonomy

## 3. Product and sum categories

Phalcom uses one product substrate across several language-level categories, but those categories remain semantically distinct.

The canonical taxonomy is:

| Category | Semantic kind | Identity model | Product role |
|---|---|---|---|
| `Unit` | zero product | canonical singleton value | canonical zero-component product |
| Tuple | structural ordered product | structural | anonymous product |
| Record | structural key product | structural | anonymous product |
| `data` | nominal transparent product | nominal + component value | named product |
| enum root | nominal closed sum | nominal sum identity | owns variants, not itself a payload product |
| enum constructor variant | exact nominal case | variant identity + payload values | payload is a product |
| enum singleton variant | exact nominal case | canonical variant value | no payload storage |
| GADT variant | exact nominal case with static result refinement | same runtime case model as ordinary variant | payload is ordinary variant product |
| `Option` | native-specialized closed sum | enum/variant semantics | bypasses general product storage for `Some`/`None` |
| class instance | nominal opaque object | allocation/object identity | **not** a transparent product |

The shared storage substrate MUST NOT be used to infer that two categories have the same language semantics.

In particular:

```text
ProductLayoutId
    != semantic type identity
    != runtime exact type identity
    != product-shape identity
    != enum variant identity
    != data declaration identity
    != backing allocation identity
```

---

## 4. Transparent products

`Unit`, Tuple, Record, and `data` are **transparent value products**.

Transparency means that their language-level value identity is defined independently of the identity of any materialized backing allocation.

Therefore the VM and compiler MAY:

- construct no heap object at all;
- scalar-replace all components;
- use packed slots;
- flatten nested products;
- reconstruct a materialized product later;
- share an immutable materialization;
- replace one materialization with another equivalent materialization.

Such transformations MUST NOT change any observable operation whose semantics depend on product value identity, including language-level exact sameness.

An implementation MUST NOT expose a transparent product's backing heap address, allocation count, or `ObjRef` as its semantic identity.

---

## 5. Sums and payload products

An enum is a closed nominal **sum**, not itself a product.

Each constructor variant may carry a payload product.

Conceptually:

```text
enum Result<T, E>
    = Ok(Product[T])
    | Error(Product[E])
```

This is a conceptual decomposition only. Runtime representation is described later.

The product machinery is responsible for payload component placement and storage. The ADT registry remains responsible for:

- enum-root identity;
- exact variant identity;
- discriminant;
- constructor/singleton distinction;
- behavior class;
- representation strategy.

Product storage MUST NOT become the source of truth for variant identity.

---

# Part II — Logical components and coordinates

## 6. Logical component index

Every materialized product layout describes an ordered set of **logical component coordinates**:

```text
0, 1, 2, ... N-1
```

The logical index is the stable coordinate used to connect:

- semantic component identity;
- lowering metadata;
- physical storage;
- generated projection operations;
- GC layout information.

Logical indexes MUST be dense within a concrete `ProductLayout`.

A physical word offset MUST NOT be treated as a logical component index.

---

## 7. Product coordinates are category-specific

The meaning of a logical coordinate depends on the product category.

### 7.1 Tuple

A Tuple has:

```text
ordered positional lane
followed by
ordered labeled lane
```

The tuple shape defines:

```rust
pub struct TupleProductShape {
    pub positional_len: u32,
    pub labels: Box<[Symbol]>,
}
```

The total logical arity is:

```text
positional_len + labels.len()
```

Tuple label order is semantic.

For example:

```phalcom
(x: 1, y: 2)
```

and:

```phalcom
(y: 2, x: 1)
```

have different tuple shapes.

### 7.2 Record

A Record is structurally identified by its key set, while preserving a separate observable presentation order.

The runtime therefore distinguishes:

```text
presentation coordinates
logical/storage coordinates
```

`RecordProductShape` stores:

```rust
pub struct RecordProductShape {
    pub presentation_labels: Box<[Symbol]>,
    pub logical_labels: Box<[Symbol]>,
    pub presentation_to_logical: Box<[u32]>,
}
```

`presentation_to_logical[p]` identifies the canonical logical coordinate corresponding to presentation position `p`.

The mapping MUST be a complete permutation.

This split allows record identity and canonical storage to be order-independent while preserving encounter order for operations where presentation is observable.

### 7.3 `data`

A `data` declaration owns named semantic components.

Each component has a semantic identity and a logical index.

The runtime product layout uses the logical index; the semantic layer remains the authority for the component's declaration identity and source name.

### 7.4 Enum constructor payload

A constructor variant owns zero or more payload fields.

The variant's payload fields have logical indexes local to that variant payload.

The variant descriptor remains the authority for the case identity; `ProductStorage` represents only the payload values.

---

# Part III — Product physical layout

## 8. `ProductSlotRepr`

The current physical slot kinds are:

```rust
pub enum ProductSlotRepr {
    Int64,
    Float64,
    Bool,
    Symbol,
    Value,
}
```

Their word sizes are:

| Representation | 64-bit words | Reconstructed VM value |
|---|---:|---|
| `Int64` | 1 | `Value::int(...)` |
| `Float64` | 1 | `Value::float(...)` |
| `Bool` | 1 | `Value::bool(...)` |
| `Symbol` | 1 | `Value::symbol(...)` |
| `Value` | 2 | exact `Value` raw words |

The scalar representations are an optimization over the universal 16-byte `Value` carrier.

A slot representation does not change the semantic value of the component.

For example, a field represented as `Int64` MUST reconstruct the same guest-visible integer that would be produced if the field had been stored in a full `Value` slot.

---

## 9. Scalar-slot eligibility

A scalar slot MAY be selected only when lowering information establishes that the component can be represented by that slot kind.

The storage layer validates scalar writes.

Therefore:

```text
Int64 slot   accepts a bare Int
Float64 slot accepts a bare Float
Bool slot    accepts a bare Bool
Symbol slot  accepts a bare Symbol
```

A wrapped `Some(Int)`, heap object, or other non-matching value MUST NOT be silently encoded into an incompatible scalar slot.

When a component cannot safely use a scalar slot, the lowering MUST use `ProductSlotRepr::Value`.

---

## 10. `ProductComponentLayout`

A concrete component placement is:

```rust
pub struct ProductComponentLayout {
    pub logical_index: u32,
    pub word_offset: u32,
    pub repr: ProductSlotRepr,
}
```

This structure separates:

```text
logical coordinate
physical word coordinate
physical representation kind
```

A conforming layout MUST preserve that separation.

---

## 11. `ProductLayout`

A concrete layout is:

```rust
pub struct ProductLayout {
    pub word_len: u32,
    pub components: Box<[ProductComponentLayout]>,
    value_slot_offsets: Box<[u32]>,
}
```

`word_len` is the total number of 64-bit words required by one materialized instance using that layout.

`components` is ordered by logical index.

`value_slot_offsets` records the starting word offset of each full two-word `Value` slot.

---

## 12. Layout validity

A valid `ProductLayout` MUST satisfy all of the following:

1. logical indexes are exactly dense `0..N`;
2. every component has a valid representation kind;
3. each component's word interval fits without integer overflow;
4. no two component word intervals overlap;
5. `word_len` covers every component interval;
6. every `Value` component contributes its starting offset to the precise-trace map;
7. scalar slots do not appear in the `Value` trace-offset list.

A nullary layout is valid and has:

```text
word_len = 0
components = []
value_slot_offsets = []
```

A nullary physical layout does not by itself determine the semantic category of the value.

---

## 13. `ProductLayoutSpec`

Before physical offsets are assigned, lowering carries:

```rust
pub struct ProductComponentSpec {
    pub logical_index: u32,
    pub repr: ProductSlotRepr,
}

pub struct ProductLayoutSpec {
    pub components: Box<[ProductComponentSpec]>,
}
```

`ProductLayoutSpec` describes desired logical slot representations.

`build_layout()` assigns dense physical word offsets and validates the resulting layout.

This split is important:

```text
semantic/lowering decision
        |
        v
ProductLayoutSpec
        |
        v
physical placement
        |
        v
ProductLayout
```

The semantic layer need not precompute byte/word offsets.

---

## 14. Layout interning

Concrete layouts are interned in `ProductLayoutRegistry`.

The returned:

```rust
ProductLayoutId(u32)
```

is a VM-local handle to an interned physical layout.

Equivalent physical layouts SHOULD share the same `ProductLayoutId`.

A `ProductLayoutId` MUST NOT be used as:

- semantic product type identity;
- exact generic specialization identity;
- Tuple structural shape identity;
- Record structural type identity;
- data declaration identity;
- enum variant identity.

Two semantically distinct product values may legally use the same physical layout.

---

# Part IV — `ProductStorage`

## 15. Storage structure

A materialized product buffer is:

```rust
pub struct ProductStorage {
    layout: ProductLayoutId,
    words: Box<[u64]>,
}
```

The layout ID records which physical layout describes the buffer.

The word array contains only storage payload.

Labels, nominal type identities, variant identities, and behavior classes are not duplicated into each component buffer unless explicitly required by another descriptor.

---

## 16. Construction from values

`ProductStorage::from_values(layout_id, layout, values)` constructs one complete storage buffer.

It requires:

```text
values.len() == layout.components.len()
```

and stores each value at its logical component index according to the layout.

A component-count mismatch is invalid.

The completed storage is suitable for publication only if every component was successfully encoded.

---

## 17. Encoding

The current encoding rules are:

| Slot kind | Encoding |
|---|---|
| `Int64` | integer bits in one word |
| `Float64` | `f64::to_bits` |
| `Bool` | `0` or `1` |
| `Symbol` | interned Symbol ID in one word |
| `Value` | `payload` word followed by `meta` word |

Encoding MUST reject a runtime value incompatible with the selected scalar representation.

---

## 18. Decoding

Every component load reconstructs a uniform `Value`.

Thus the product storage boundary is:

```text
Value
  -> packed representation
  -> Value
```

Guest-visible execution does not receive raw product words.

The reconstructed value MUST preserve the component's semantic value.

---

## 19. Storage/layout consistency

The current storage API checks that the supplied layout's word count matches the storage word count before reading or writing.

A runtime MUST fail cleanly rather than index a storage buffer according to a layout with an incompatible size.

A valid heap product object's descriptor and storage MUST agree on layout identity.

---

# Part V — Shape metadata

## 20. Shape and layout are independent

A product **shape** describes semantic/logical coordinates.

A product **layout** describes physical storage.

These are separate.

Examples:

```text
Tuple shape:
    positional lane length
    ordered labels

Record shape:
    presentation labels
    canonical logical labels
    presentation -> logical mapping

Layout:
    logical component 0 -> Int64 at word 0
    logical component 1 -> Value at words 1..2
```

A shape can potentially be paired with more than one layout over the lifetime of optimization strategy changes.

A layout can be shared by many shapes.

---

## 21. `ProductShapeRegistry`

Tuple and Record shapes are interned in `ProductShapeRegistry`.

The registry returns:

```rust
ProductShapeId(u32)
```

`ProductShapeId` is VM-local shape metadata identity.

For Records, the registry additionally maintains a label-to-logical-index lookup cache.

A `ProductShapeId` is not an exact runtime type.

---

## 22. Tuple shape invariants

`TupleProductShape` MUST satisfy:

- `positional_len` is the size of the positional lane;
- `labels` describes only the labeled suffix;
- labels are unique;
- label order is preserved and semantically significant;
- `total_len = positional_len + labels.len()`.

---

## 23. Record shape invariants

`RecordProductShape` MUST satisfy:

- presentation labels are unique;
- logical labels are unique;
- both label arrays have the same length;
- `presentation_to_logical` has the same length;
- `presentation_to_logical` is a complete permutation of logical coordinates;
- presentation order remains available for iteration/rendering/reflection/expansion;
- logical coordinates provide order-independent canonical storage and lookup.

Two Records with the same key set but different presentation order MAY have distinct `ProductShapeId`s while sharing the same logical layout and structural exact type.

---

# Part VI — Anonymous product descriptors

## 24. Runtime anonymous products

Materialized Tuple and Record values use:

```rust
pub struct RuntimeAnonymousProductDescriptor {
    pub runtime_id: RuntimeAnonymousProductDescriptorId,
    pub kind: AnonymousProductKind,
    pub shape: ProductShapeId,
    pub layout: ProductLayoutId,
    pub exact_type: Option<RuntimeTypeRef>,
}
```

The descriptor joins four orthogonal facts:

```text
product category
shape metadata
physical layout
optional exact retained structural type
```

This metadata is interned and shared across product instances.

A Tuple/Record instance MUST NOT need to duplicate its labels or complete type structure per value when the descriptor already owns that information.

---

## 25. Exact retained type

`exact_type` is present only when semantic/lowering metadata establishes an exact runtime type.

The runtime MUST NOT fabricate an exact structural type merely by inspecting payload values.

For example, runtime values alone are insufficient authority to infer all static generic, phantom, or structural typing facts.

Exact runtime type materialization is supplied through the runtime typing system and type environment.

This preserves the distinction:

```text
observed payload representation
    != authoritative exact type
```

---

# Part VII — Tuple products

## 26. Tuple semantics

A Tuple is a structural ordered immutable product.

Its semantic shape includes:

- positional count;
- ordered labeled suffix;
- total arity.

Tuple component order is semantically significant.

---

## 27. Empty Tuple

The empty Tuple:

```phalcom
()
```

normalizes to the canonical `Unit` value.

The VM MUST NOT materialize a positive `TupleObject` with zero components.

This invariant is enforced both by compiler normalization and by runtime construction boundaries.

---

## 28. Positive Tuple representation

A materialized positive Tuple is:

```rust
pub struct TupleObject {
    descriptor: RuntimeAnonymousProductDescriptorId,
    storage: ProductStorage,
}
```

The descriptor identifies:

- Tuple category;
- tuple shape;
- physical layout;
- optional exact structural runtime type.

The storage contains component values in logical tuple coordinate order.

Labels are descriptor/shape metadata, not per-instance payload arrays.

---

## 29. Dynamic versus statically shaped Tuple construction

The runtime exposes two conceptually different construction paths.

### 29.1 Dynamic/general construction

When exact lowering metadata is unavailable, the runtime may synthesize a layout using full `Value` slots for every component.

This is a safe general representation.

### 29.2 Statically shaped construction

When semantic lowering provides:

- exact tuple shape;
- `ProductLayoutSpec`;
- runtime type recipe;

the VM may:

1. register the shape;
2. build/intern the packed layout;
3. encode source-order component values;
4. materialize the exact runtime type;
5. intern the anonymous-product descriptor;
6. allocate or otherwise realize the Tuple representation.

This path allows native-width scalar packing.

---

## 30. Tuple exact sameness

For transparent Tuple values, language-level exact sameness is independent of backing allocation.

Two positive Tuples are `===` iff:

1. their tuple structural shapes are identical; and
2. corresponding components are recursively `===`.

For tuples, ordered label presentation is part of shape.

Therefore:

```phalcom
(x: 1, y: 2) === (y: 2, x: 1)
```

is false.

---

# Part VIII — Record products

## 31. Record semantics

A Record is a structural key-based immutable product.

Record structural identity is independent of encounter order.

Presentation order remains separately observable.

This duality is fundamental.

---

## 32. Empty Record

The empty Record product:

```phalcom
#{}
```

normalizes to the same canonical zero-product representation as Unit.

The VM MUST NOT allocate a positive `RecordObject` for an empty record.

This physical normalization does not erase the source-language distinction where source syntax or static typing retains one; the VM value representation uses the canonical zero product.

---

## 33. Positive Record representation

A materialized positive Record is:

```rust
pub struct RecordObject {
    descriptor: RuntimeAnonymousProductDescriptorId,
    storage: ProductStorage,
}
```

The descriptor identifies:

- Record category;
- record shape;
- canonical physical layout;
- optional exact structural runtime type.

The storage is arranged in logical coordinate order rather than necessarily in source encounter order.

---

## 34. Record presentation and storage

For a source Record such as:

```phalcom
#{ name: "A", age: 24 }
```

the shape retains:

```text
presentation_labels = [name, age]
```

while logical coordinates may use a canonical label order.

The mapping:

```text
presentation_to_logical
```

is used to rearrange source-order values into canonical storage coordinates.

Consequently, two records written in different presentation order may share:

- exact structural type;
- physical layout;
- component key/value meaning;

while preserving distinct presentation metadata.

---

## 35. Record exact sameness

Two positive Records are `===` iff:

1. they have the same key set; and
2. corresponding values by key are recursively `===`.

Presentation order does not affect Record exact sameness.

Thus:

```phalcom
#{ name: "A", age: 24 } === #{ age: 24, name: "A" }
```

is true if corresponding component values are exactly same.

This differs intentionally from Tuple labeled-lane semantics.

---

# Part IX — Nominal `data` products

## 36. `data` semantic category

A `data` declaration defines a nominal transparent immutable product.

It is nominal because declaration identity participates in its semantic/exact runtime type.

It is transparent because backing allocation identity is not language value identity.

It is a product because its component state is represented through product coordinates and storage.

---

## 37. Runtime data descriptor

The runtime descriptor is:

```rust
pub struct RuntimeDataDescriptor {
    pub semantic_owner: DeclarationId,
    pub runtime_id: RuntimeDataDescriptorId,
    pub behavior_class: ClassId,
    pub exact_type: Option<RuntimeTypeRef>,
    pub layout: ProductLayoutId,
}
```

The fields represent distinct kinds of identity:

| Field | Meaning |
|---|---|
| `semantic_owner` | stable declaration identity |
| `runtime_id` | VM-local descriptor handle |
| `behavior_class` | ordinary behavior dispatch class |
| `exact_type` | retained exact nominal runtime type, including specialization where available |
| `layout` | physical component storage arrangement |

None of these fields is interchangeable with another.

---

## 38. Data descriptor interning

`RuntimeDataRegistry` interns data descriptors by:

```text
(DeclarationId, optional exact RuntimeTypeRef)
```

A repeated registration of the same semantic owner/exact type resolves to the existing runtime descriptor.

Behavior-class lookup and declaration lookup are secondary indexes.

Runtime descriptor identity is VM-local.

---

## 39. Nullary `data`

A nullary data value has no component payload.

The current `Value` model can represent it immediately using:

```text
ValueTag::DataSingleton
payload = RuntimeDataDescriptorId
```

No `ProductStorage` allocation is required for the value.

The descriptor still supplies:

- nominal identity;
- behavior class;
- exact runtime type metadata;
- layout metadata.

---

## 40. Positive-arity `data`

A materialized positive-arity data value uses a heap `DataObject` whose conceptual state is:

```text
descriptor: RuntimeDataDescriptorId
storage:    ProductStorage
```

The descriptor identifies the nominal data type and physical layout.

The storage contains only component values.

A data object's backing `ObjRef` is not the semantic value identity of the data value.

---

## 41. Data getters and projection

A source-visible field getter is behavior on the data value.

The reference runtime may synthesize small getter methods that:

1. load the receiver;
2. project the component by logical index;
3. return the reconstructed `Value`.

The product storage layer remains name-agnostic: it projects by logical coordinate.

The semantic component identity and source field name belong to declaration/lowering metadata.

---

## 42. Data exact sameness

Two data values are `===` iff:

1. they have the identical exact reified nominal data type, including exact generic and phantom arguments represented by that type identity; and
2. corresponding components are recursively `===`.

Backing `ObjRef` identity does not participate.

A compiler MAY therefore scalar-replace or rematerialize data values without changing `===`.

---

# Part X — Enums and ADTs

## 43. Enum as a closed nominal sum

An enum root defines a closed nominal sum.

Runtime ADT metadata distinguishes:

```text
enum root identity
variant identity
case discriminant
variant behavior class
payload layout
representation strategy
```

The product subsystem supplies payload storage only.

---

## 44. Required identity separation

The runtime enforces the invariant:

```text
VariantId
    != RuntimeVariantId
    != CaseDiscriminant
```

and additionally:

```text
DeclarationId of enum
    != RuntimeEnumId
    != ClassId
```

### 44.1 `VariantId`

Static semantic identity of the exact variant declaration.

### 44.2 `RuntimeVariantId`

VM-local identity of the registered variant descriptor.

### 44.3 `CaseDiscriminant`

Dense physical case index within one enum, normally assigned in declaration order.

A discriminant MUST NOT be treated as globally unique across enums.

---

## 45. Runtime enum descriptor

An enum root is represented by:

```rust
pub struct RuntimeEnumDescriptor {
    pub semantic_owner: DeclarationId,
    pub runtime_id: RuntimeEnumId,
    pub root_class: ClassId,
    pub representation: RuntimeAdtRepresentation,
    pub variants: Vec<RuntimeVariantId>,
}
```

`representation` currently distinguishes:

```rust
General
NativeOption
```

The enum root descriptor owns the ordered registered variant set.

---

## 46. Runtime variant descriptor

A variant is represented by:

```rust
pub struct RuntimeVariantDescriptor {
    pub semantic_id: VariantId,
    pub runtime_id: RuntimeVariantId,
    pub enum_id: RuntimeEnumId,
    pub discriminant: CaseDiscriminant,
    pub shape: RuntimeVariantShape,
    pub payload_arity: u16,
    pub layout: Option<ProductLayoutId>,
    pub behavior_class: ClassId,
    pub singleton: Option<Value>,
}
```

The descriptor is the runtime authority for exact case identity.

The payload layout is subordinate metadata.

---

## 47. Variant shape

Runtime variant shape distinguishes:

```rust
Singleton
Constructor
```

This distinction is semantic and MUST be preserved.

A singleton variant:

```phalcom
None
```

is not equivalent to an explicit zero-argument constructor:

```phalcom
Empty()
```

even though neither carries positive payload fields.

A zero-argument constructor remains a constructor identity and construction operation.

---

## 48. Singleton variants

A singleton variant denotes one canonical exact-case value.

General ADTs may represent singleton variants immediately through:

```text
ValueTag::AdtSingleton
payload = RuntimeVariantId
```

The runtime variant descriptor supplies:

- parent enum;
- discriminant;
- behavior class;
- semantic variant identity.

No positive `ProductStorage` is needed.

---

## 49. Payload-bearing constructor variants

A positive-payload constructor variant uses product layout and storage for its payload.

Conceptually:

```text
AdtCaseObject {
    variant: RuntimeVariantId,
    storage: ProductStorage
}
```

The case object MUST retain exact variant identity separately from payload layout.

Two variants may legally use identical `ProductLayout`s.

Therefore:

```text
same layout
    does not imply
same variant
```

---

## 50. Variant registration

Enum registration is idempotent only when the repeated semantic owner agrees with the already-published:

- root behavior class;
- representation strategy.

Conflicting registration is an error.

Variant registration is keyed by semantic `VariantId`.

The runtime also indexes variants by behavior class for case/class mapping.

---

# Part XI — GADTs

## 51. GADT runtime principle

A GADT variant is **not a new physical runtime product category**.

A generalized algebraic data type extends ordinary enum semantics by attaching a more specific result type and type-equality evidence to a variant declaration.

For example:

```phalcom
enum Expr<T> {
    IntLit(_ value: Int) -> Expr<Int>
    BoolLit(_ value: Bool) -> Expr<Bool>
}
```

The static semantics establish:

```text
Expr::IntLit  => T = Int
Expr::BoolLit => T = Bool
```

The payloads remain ordinary variant products.

---

## 52. Equality evidence is static metadata

GADT equality evidence belongs to semantic typing/proof machinery.

It MUST NOT be stored redundantly in each runtime case value merely to preserve type-checker proofs.

Successful pattern elimination of an exact GADT case may introduce the corresponding equality evidence into a branch.

That branch refinement is a static semantic operation.

The runtime case test remains an exact variant identity test.

---

## 53. GADT exact case and runtime representation

A GADT variant has:

```text
semantic VariantId
runtime RuntimeVariantId
CaseDiscriminant
behavior ClassId
payload layout/storage
```

just like an ordinary ADT variant.

Its result specialization may affect:

- exact static type;
- runtime type metadata/reification where retained;
- reflection.

It does not require a distinct `Object` variant or a distinct product-storage format.

---

## 54. GADT payload layout

Payload slot layout is determined from payload component types, not from the fact that the variant is generalized.

For example:

```phalcom
IntLit(_ value: Int) -> Expr<Int>
```

may use an `Int64` payload slot.

The GADT equality `T = Int` is not encoded as another payload word.

---

# Part XII — Native `Option`

## 55. `Option` is semantically an enum and physically exceptional

Core `Option<T>` retains ordinary ADT semantics but uses:

```rust
RuntimeAdtRepresentation::NativeOption
```

rather than the general singleton/case-object representation.

The runtime registry binds the semantic/runtime identities of the `Some` and `None` variants.

---

## 56. `None`

`Option.None` uses the immediate `None` representation specified by `values-and-objects.md`.

It does not allocate an `AdtSingleton` or `AdtCaseObject`.

---

## 57. `Some`

`Option.Some(x)` is normally represented by incrementing the `some_depth` metadata of the existing `Value`.

No payload `ProductStorage` is allocated.

Nested `Some` layers increment the same nesting field.

The semantic fact that the value is the `Some` variant is recovered through the native Option representation and registry bindings.

---

## 58. Why `Option` does not invalidate the product model

`Some` is semantically a one-component constructor variant.

The native representation is an optimization that eliminates the general payload product because the universal `Value` carrier has a dedicated representation niche.

Therefore:

```text
semantic constructor payload
    does not imply
materialized ProductStorage
```

The same optimization principle applies more generally: a semantic product may remain virtual or be represented in a specialized form when observable behavior is preserved.

---

# Part XIII — Runtime type metadata and reification

## 59. Physical layout is not exact type

An exact `RuntimeTypeRef` and a `ProductLayoutId` answer different questions.

```text
RuntimeTypeRef:
    What exact runtime type is this value known to have?

ProductLayoutId:
    How are the materialized components physically stored?
```

The same exact type may potentially admit different physical layouts across optimization modes or runtime evolution.

Different exact types may share one physical layout.

---

## 60. Data exact types

`RuntimeDataDescriptor.exact_type` may retain a concrete nominal specialization.

This allows the runtime to preserve generic/phantom identity without storing a generic argument array in every data value.

---

## 61. Anonymous structural exact types

`RuntimeAnonymousProductDescriptor.exact_type` may retain exact Tuple/Record structural type metadata.

This type is supplied from semantic lowering/runtime type recipes.

It MUST NOT be inferred from payload values as a substitute for semantic authority.

---

## 62. No per-value specialization classes

Applied product types do not require one distinct `ClassObject` per exact specialization.

For example:

- all Tuples still have class `Tuple`;
- all Records still have class `Record`;
- a generic `data` declaration has a behavior class independent of each exact type specialization unless another specification explicitly requires otherwise.

Exact runtime types and behavior classes are separate axes.

---

# Part XIV — Construction boundaries

## 63. Construction pipeline

The canonical materialized construction pipeline is:

```text
semantic/lowering information
        |
        v
logical product shape / declaration identity
        |
        v
ProductLayoutSpec
        |
        v
ProductLayout
        |
        v
ProductLayoutRegistry -> ProductLayoutId
        |
        v
ProductStorage::from_values(...)
        |
        +------------------------------+
        |                              |
        v                              v
nominal descriptor                anonymous descriptor
(data / variant)                  (Tuple / Record)
        |                              |
        +---------------+--------------+
                        |
                        v
          materialized product value
```

Specialized representations such as Unit, nullary data/variant singletons, or native `Option` bypass positive storage.

---

## 64. Publication invariant

A materialized product MUST NOT become guest-observable in a partially encoded state.

The runtime SHOULD build complete immutable storage before publishing the final product value.

This supports:

- immutability;
- precise tracing;
- scalar layout validation;
- safe sharing;
- deterministic projection.

---

# Part XV — Projection and observation

## 65. Logical projection

Product projection is defined in terms of logical coordinates or semantic keys, not physical word offsets.

The storage subsystem exposes:

```text
logical index -> reconstructed Value
```

Higher layers map:

- Tuple position/label;
- Record key;
- data component;
- variant payload component;

to that logical index.

---

## 66. Tuple observation

Tuple runtime protocol operations may use `TupleView` or equivalent descriptor-backed access to:

- total size;
- positional lane;
- labels;
- indexed component values.

The backing storage remains encapsulated.

---

## 67. Record observation

Record observation must preserve both coordinate systems:

- key lookup resolves to canonical logical storage;
- iteration/rendering/reflection may use presentation order and map each presentation coordinate to the logical component.

A runtime MUST NOT accidentally expose canonical storage order as presentation order when the two differ.

---

## 68. Nominal product observation

For data and ADT cases:

- descriptor identity determines the semantic owner/case;
- layout determines physical component location;
- component metadata determines source/semantic field meaning.

Physical layout alone is insufficient for reflection.

---

# Part XVI — Exact sameness, equality, and hashing

## 69. Exact sameness of transparent products

Language-level `===` is representation-independent for transparent products.

The product-specific rules are:

| Category | Exact sameness |
|---|---|
| Unit | always same as Unit |
| Tuple | identical tuple shape + recursively same components |
| Record | same key set + recursively same values by key |
| `data` | same exact nominal type + recursively same components |

A heap allocation handle MUST NOT substitute for these rules.

---

## 70. Tuple order

Tuple exact sameness is order-sensitive.

Both positional order and labeled-lane order are semantic.

---

## 71. Record order

Record exact sameness is presentation-order-insensitive.

Lookup is by canonical key identity.

---

## 72. ADT exact cases

ADT/GADT exact-case identity is variant identity plus the relevant payload semantics.

A singleton variant is exactly its canonical case.

A constructor case is not identified solely by payload shape.

The full language-level equality contracts for enum values are defined by the language/ADT specifications and VM exactness implementation; product layout never substitutes for variant identity.

---

## 73. `==` and hash are separate contracts

`===`, `==`, and `hash` are separate operations.

This specification requires that product physical representation not leak into any of them.

Tuple and Record public equality/hash behavior may use their category-specific structural contracts.

A data type's `==` behavior MUST NOT be inferred merely from `ProductStorage`; source/library behavior remains authoritative.

---

# Part XVII — Garbage collection

## 74. Precise product tracing

`ProductLayout` precomputes:

```text
value_slot_offsets
```

containing the starting word offset of every `ProductSlotRepr::Value` component.

Scalar slots require no heap-edge tracing.

Therefore a product whose layout contains only:

```text
Int64
Float64
Bool
Symbol
```

has no component `Value` trace entries.

---

## 75. Tracing a `Value` slot

At each value-slot offset, two words reconstruct a `Value`.

GC must then use the value representation's GC-aware object-edge logic.

In particular, an Option-wrapped heap reference remains a live edge even when normal surface accessors do not expose the underlying base object.

That rule is specified in `values-and-objects.md`.

---

## 76. Descriptor registries and GC roots

Product shape/layout metadata contains Rust-side metadata and no guest heap object handles unless explicitly stated.

Data and ADT registries do hold behavior `ClassId`s.

Those behavior classes MUST participate in GC root enumeration.

Heap product values are traced through their `ProductStorage`.

---

# Part XVIII — Optimization and virtual products

## 77. Optimization freedom

Transparent product semantics permit the compiler to avoid materialization.

A product may remain:

```text
virtual
```

with individual leaf components stored directly in VM locals or compiler-managed slots.

This optimization is valid only when every possible observation can be implemented with equivalent semantics or forces correct materialization.

---

## 78. Scalar replacement

A local immutable product that does not escape may be represented as independent leaf values rather than as:

```text
descriptor + ProductStorage + heap object
```

For example:

```phalcom
const p = Point.new(x: 1, y: 2)
consume(p.x + p.y)
```

may be optimized so that `x` and `y` live directly in compiler-managed locals and no `DataObject` is allocated.

---

## 79. Materialization boundary

A virtual product must be materialized when an operation requires a concrete runtime representation that cannot be answered safely from the virtual plan.

Potential boundaries include:

- escaping into an opaque/dynamic context;
- storing into a heap location requiring a `Value`;
- reflection requiring runtime descriptor access;
- calling behavior whose implementation cannot consume the virtual form;
- returning across a boundary where the product cannot remain virtual.

An implementation MAY answer some observations directly from virtual metadata rather than materializing.

---

## 80. Semantic requirements on materialization

Materialization MUST preserve:

- product category;
- shape;
- exact nominal/structural type metadata where required;
- component values;
- presentation metadata;
- variant/data identity;
- behavior class;
- `===` result.

A program MUST NOT be able to determine whether a transparent product was continuously heap allocated, scalar replaced, or rematerialized.

---

# Part XIX — Data, Tuple, Record, and ADT comparison

## 81. Consolidated model

| Property | Tuple | Record | `data` | ADT/GADT constructor |
|---|---|---|---|---|
| Semantic kind | structural product | structural product | nominal product | exact case of nominal sum |
| Immutable | yes | yes | yes | payload immutable |
| Ordered positional coordinates | yes | no | declaration-defined | declaration-defined |
| Labels | optional ordered suffix | key set + presentation order | named components | constructor parameters/fields |
| Structural type identity | yes | yes | no | no |
| Nominal declaration identity | no | no | yes | yes |
| Uses `ProductLayout` | yes | yes | yes | payload yes |
| Uses `ProductStorage` when materialized positive | yes | yes | yes | yes |
| Descriptor kind | anonymous | anonymous | data descriptor | variant descriptor |
| Exact runtime type retention | optional `RuntimeTypeRef` | optional `RuntimeTypeRef` | optional `RuntimeTypeRef` | semantic/runtime type system as required |
| Nullary optimization | Unit | Unit | `DataSingleton` | `AdtSingleton` or constructor-specific representation |
| Backing allocation identity observable | no | no | no | case identity is variant-based, not layout-based |
| GADT proof evidence stored per value | n/a | n/a | n/a | no |

---

# Part XX — Invariants

## 82. Product invariants

The following invariants are normative.

### P-01 — Logical/physical separation

Logical component identity MUST NOT be inferred from physical word offset.

### P-02 — Layout/type separation

`ProductLayoutId` MUST NOT be semantic or exact runtime type identity.

### P-03 — Shape/layout separation

Tuple/Record structural shape metadata MUST remain independent of physical layout metadata.

### P-04 — Descriptor/storage consistency

A materialized object's descriptor and storage MUST agree on physical layout.

### P-05 — Dense logical coordinates

A concrete product layout uses dense logical component indexes `0..N`.

### P-06 — No overlap

Physical component intervals MUST NOT overlap.

### P-07 — Precise trace map

Every full `Value` slot MUST appear in the layout trace map; scalar slots MUST NOT.

### P-08 — Transparent product identity

Tuple, Record, and `data` backing allocation identity MUST NOT be guest-observable as product value identity.

### P-09 — Tuple order

Tuple positional and labeled-lane ordering is semantic.

### P-10 — Record key identity

Record structural identity is independent of presentation order.

### P-11 — Presentation preservation

Record encounter/presentation order MUST remain available where language semantics expose it.

### P-12 — Empty anonymous products

The runtime MUST NOT materialize positive zero-component Tuple/Record heap objects; they normalize to the canonical zero product.

### P-13 — Nominal data identity

A `data` product's semantic identity includes its declaration/exact nominal type and MUST NOT collapse to layout identity.

### P-14 — Variant identity

An enum payload's `ProductLayout` MUST NOT replace exact variant identity.

### P-15 — Identity-level separation

`VariantId`, `RuntimeVariantId`, and `CaseDiscriminant` remain distinct.

### P-16 — GADT evidence

GADT equality evidence is static proof information and MUST NOT be required as a per-value payload.

### P-17 — Native Option exception

`Option` MAY bypass general ADT product storage while preserving ordinary enum/variant semantics.

### P-18 — Scalar-slot soundness

A scalar slot MUST accept only compatible bare runtime values.

### P-19 — Exact-type authority

Exact retained runtime types come from semantic/runtime type metadata, not ad-hoc payload inspection.

### P-20 — Optimization transparency

Scalar replacement, flattening, sharing, and rematerialization MUST preserve all product-observable semantics.

---

# Part XXI — Reference implementation map

## 83. Core product subsystem

```text
phalcom-core/src/product/
    mod.rs
    layout.rs
    storage.rs
    registry.rs
    shape.rs
    anonymous.rs
    view.rs
```

Responsibilities:

| File | Responsibility |
|---|---|
| `layout.rs` | slot representations, component layouts, layout specs |
| `storage.rs` | packed word buffer encode/decode |
| `registry.rs` | physical layout interning |
| `shape.rs` | Tuple/Record semantic shape metadata |
| `anonymous.rs` | anonymous product runtime descriptors |
| `view.rs` | descriptor-backed Tuple/Record views |
| `mod.rs` | construction boundaries and static/dynamic finalizers |

---

## 84. Nominal data subsystem

```text
phalcom-core/src/data.rs
phalcom-core/src/heap/data.rs
phalcom-core/src/vm/data.rs
```

Responsibilities include:

- `RuntimeDataDescriptor`;
- registry interning;
- behavior class association;
- exact type retention;
- layout association;
- construction;
- component getter machinery.

---

## 85. ADT subsystem

```text
phalcom-core/src/adt.rs
phalcom-core/src/heap/adt.rs
phalcom-core/src/vm/adt.rs
phalcom-semantic/src/checker/gadt_proof.rs
```

Responsibilities include:

- enum and variant runtime identities;
- discriminants;
- representation strategy;
- variant behavior classes;
- payload layout;
- native `Option`;
- GADT branch proof/refinement at the semantic layer.

---

## 86. Compiler product optimization

```text
phalcom-core/src/compiler/lib/product_opt.rs
```

The compiler product plan records representation-aware optimization decisions, virtual product shapes, and ephemeral projections.

The optimizer is subordinate to the invariants in this document.

---

# Part XXII — Relationship to companion specifications

## 87. `values-and-objects.md`

`values-and-objects.md` owns:

- `Value`;
- value tags;
- `ObjRef`;
- heap/object categories;
- `Nil`;
- `None`/`Some` physical representation;
- GC extraction from a `Value`;
- object/value class mapping;
- global exact-sameness machinery.

This document owns the product-specific metadata and storage model consumed by those facilities.

---

## 88. `bytecodes.md`

`bytecodes.md` owns the semantics of instructions that:

- construct products;
- register data/enums;
- test variants;
- project components;
- materialize optimized values.

This document specifies what a valid product value/descriptor/storage state means before and after those instructions.

---

## 89. `execution-model.md`

`execution-model.md` owns:

- activation stack;
- frame/local rules;
- instruction dispatch;
- calls and returns;
- abrupt control flow.

Virtual or materialized product values follow that execution model.

---

# Part XXIII — Final model

## 90. Product architecture in one diagram

```text
                         semantic / lowering authority
                                   |
         +-------------------------+---------------------------+
         |                         |                           |
         v                         v                           v
   anonymous shape          nominal data identity       enum/variant identity
  Tuple / Record           DeclarationId + type       VariantId + enum root
         |                         |                           |
         |                         |                           |
         +------------+------------+---------------------------+
                      |
                      v
              ProductLayoutSpec
                      |
                      v
                ProductLayout
                      |
                 intern layout
                      |
                      v
               ProductLayoutId
                      |
                      v
               ProductStorage
                      |
       +--------------+---------------+
       |              |               |
       v              v               v
 Tuple/Record      DataObject     AdtCaseObject
 descriptor        descriptor       variant id
       |              |               |
       +--------------+---------------+
                      |
                      v
                    Value
```

Specialized cases bypass positive storage:

```text
Unit
DataSingleton
AdtSingleton
Native Option None/Some
virtual scalar-replaced products
```

The resulting system has one physical product substrate without conflating language categories.

That separation is the defining architectural property of Phalcom's product model.

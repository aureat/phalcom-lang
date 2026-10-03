# Structural Products

**Status:** Normative  
**Semantic ownership:** Pack, Tuple, Record, Product capabilities, ordered/unordered
payload shapes, structural tails, field keys and presence, structural lookup, structural
iteration, and structural product equality.

Runtime physical layout remains owned by [VM Products](vm/products.md). Argument
packing/rest/spread semantics are also constrained by
[Arguments and Parameters](callables/arguments.md). Static execution is owned by
[Static Evaluation](static-evaluation.md).

## 1. Product ontology

A **Product** has one structural payload.

Public structural capabilities are:

```telos
trait Product {
    type Kind: ProductKind
    type Shape: ProductShape
}

trait OrderedProduct {
    type OrderedShape: OrderedShape
    type Entries: CompleteTypePack
    type PositionalEntries: PositionalTypePack
    type LabeledEntries: LabeledTypePack
}

trait UnorderedProduct {
    type UnorderedShape: UnorderedShape
}
```

The compiler derives these sealed capabilities from canonical data-declaration shape.
Ordinary user `impl` declarations cannot forge them.

The traits are independent; inheritance among them is not required.

```text
ordered product data
    -> Product
    -> OrderedProduct

unordered product data
    -> Product
    -> UnorderedProduct
```

Declaration identity remains on the owning type. Equivalent reflected payload shapes do
not make distinct nominal data declarations equal.

## 2. Pack

```telos
intrinsic data Pack<***Es>(***Es)

type PositionalPack<T: Type> =
    Pack<*: T>

type LabeledPack<T: Type> =
    Pack<**: T>

type CompletePack<T: Type> =
    Pack<***: T>

type PositionalTypePack =
    PositionalPack<Type>

type LabeledTypePack =
    LabeledPack<Type>

type CompleteTypePack =
    CompletePack<Type>
```

Pack is a first-class immutable ordered product with two independent lanes:

```text
positional lane
labeled lane: ordered (Symbol, value) entries
```

Order within each lane participates in structural identity. Lane identity participates
in structural identity. Labeled entries use Symbol labels and duplicate labels are
invalid.

`Pack()` is the empty Pack. It is distinct from Unit/`()` and from the empty Record.

The public TypePack model is therefore ordinary Pack specialization. Compiler-internal
specialized TypePack terms may remain optimized canonical representations, but they do
not define an independent source-visible ontology.

### 2.1 Pack performance contract

Pack is specialized for variadic structural transport: construction, rest capture,
slicing, spreading, and static structural computation.

The implementation must preserve these semantic-performance properties:

- Pack may use flat contiguous storage, argument windows, slices, scalar replacement,
  compiler-virtual segments, or equivalent representations;
- lane lengths are O(1);
- a one-step rest/view can be represented in O(1) without copying the remaining Pack;
- Pack is not required to use recursively linked cons storage;
- materializing/finalizing n Pack entries is O(n);
- multi-spread lowering is O(total contributed entries), not repeated immutable
  concatenation;
- an ordinary fixed-shape call does not allocate a Pack solely because its argument
  shape is semantically Pack-shaped.

These representation freedoms are not observable through Telos value semantics.

### 2.2 Pack views

```telos
intrinsic data PositionalPackView<T> {
    Empty
    Entry(
        _ value: T,
        _ rest: PositionalPack<T>,
    )
}

intrinsic data LabeledPackView<T> {
    Empty
    Entry(
        _ label: Symbol,
        _ value: T,
        _ rest: LabeledPack<T>,
    )
}
```

A View is one-step semantic decomposition. `rest` may be an O(1) slice over the same
backing storage. Constructing or matching a View does not require constructing a
recursive chain of view nodes.

```telos
impl<T: Type> PositionalPack<T> {
    intrinsic type View: PositionalPackView<T>
}

impl<T: Type> LabeledPack<T> {
    intrinsic type View: LabeledPackView<T>
}

impl<T: Type> CompletePack<T> {
    intrinsic type PositionalEntries: PositionalPack<T>
    intrinsic type LabeledEntries: LabeledPack<T>
}
```

## 3. Tuple and Record

```telos
intrinsic data Tuple<***Es>(***Es)

intrinsic data Record<**Fs> {
    **Fs
}
```

Tuple and Pack are distinct named ordered products and may expose equivalent
`OrderedShape` values.

The zero-arity Tuple canonicalizes to Unit according to the ordinary Tuple/Unit rules.
The empty Pack does not.

Record is an intrinsic unordered product. Record is not the definition of unordered
reflection and is not interchangeable with LabeledPack merely because all of its keys
happen to be Symbols.

## 4. Ordered shape

Ordered-product constituents are **entries**. A labeled entry has a **label**.

```telos
intrinsic data OrderedTail<Rest> {
    Closed

    Typed(
        _ valueType: Type,
    )

    Unconstrained

    Captured(
        _ rest: Rest,
    )
}

type PositionalTail =
    OrderedTail<PositionalTypePack>

type LabeledTail =
    OrderedTail<LabeledTypePack>

intrinsic data OrderedShape(
    _ entries: CompleteTypePack,
    _ positionalTail: PositionalTail,
    _ labeledTail: LabeledTail,
)
```

`Closed` admits no additional entries. `Typed` admits zero or more additional
entries constrained by a common value Type. `Unconstrained` admits additional entries
without a common value-Type constraint. `Captured` preserves the exact correlated
remainder.

Anonymous open tails describe possible structure and are not finite iterable
collections. A captured concrete Pack may be iterated.

## 5. Unordered products

Unordered-product constituents are **fields**. A field has a **key**.

### 5.1 FieldKey

```telos
type FieldKey =
    Symbol | String | Int | Bool
```

Ordered-product labels remain Symbols.

FieldKey has a language-defined version-stable total order used for canonical
reflection, row normalization, fingerprints, and metadata. C4 orders key kinds:

```text
Symbol < String < Int < Bool
```

Within a kind:

- Symbol ordering uses stable semantic symbol spelling/identity, never process-local
  interner IDs;
- String ordering is by canonical string content;
- Int ordering is mathematical integer order;
- Bool ordering is `false < true`.

Source/insertion order does not participate in unordered structural identity.

### 5.2 Presence and fixed fields

```telos
intrinsic data Presence {
    Required
    Optional
}

intrinsic data FieldInfo(
    _ type: Type,
    _ presence: Presence,
)

intrinsic data ProductField(
    _ key: FieldKey,
    _ type: Type,
    _ presence: Presence,
)

type FixedFields =
    Record<**[FieldKey]: FieldInfo>
```

Optionality is field presence, not `Option<T>`.

A required target field requires a provably required source field. An optional source
field cannot satisfy a required target field merely because the value Types match.

### 5.3 Unordered tails

```telos
intrinsic data UnorderedTail {
    Closed

    Typed(
        _ keyType: Type,
        _ valueType: Type,
    )

    Unconstrained(
        _ keyType: Type,
    )

    Captured(
        _ rest: UnorderedShape,
    )
}

intrinsic data UnorderedShape(
    _ fields: FixedFields,
    _ tail: UnorderedTail,
)
```

`Closed` admits no unknown fields.

`Typed(K,V)` permits additional fields whose keys conform to K and whose values
conform to V.

`Unconstrained(K)` constrains possible key domain without a common value Type.

`Captured(rest)` preserves an exact correlated remainder.

Static computed FieldKeys that reduce to concrete keys become fixed fields. A collision
that appears after specialization is an error, never overwrite/last-write-wins.

### 5.4 Lookup

```telos
intrinsic data FieldLookup {
    Required(_ type: Type)
    Optional(_ type: Type)
    Absent
    Unknown
}
```

`UnorderedShape::Lookup<K>` returns the strongest result justified by fixed fields,
presence, key-domain information, and tail state.

```telos
impl UnorderedShape {
    intrinsic type Lookup<K: FieldKey>: FieldLookup
}
```

`Unknown` is a structural lookup result and is distinct from the internal proof-state
uncertainty described by [Static Evaluation](static-evaluation.md).

### 5.5 Unordered shape view

```telos
intrinsic data UnorderedShapeView {
    Empty(
        _ tail: UnorderedTail,
    )

    Field(
        _ field: ProductField,
        _ rest: UnorderedShape,
    )
}

impl UnorderedShape {
    intrinsic type View: UnorderedShapeView
    intrinsic type Tail: UnorderedTail
}
```

The view walks fixed fields in canonical FieldKey order and preserves the tail.
It is a one-step decomposition and does not require recursively linked storage.

## 6. General Product reflection

```telos
intrinsic data ProductKind {
    Ordered
    Unordered
}

intrinsic data ProductShape {
    Ordered(
        _ shape: OrderedShape,
    )

    Unordered(
        _ shape: UnorderedShape,
    )
}
```

Reflection describes payload structure only.

For a recursive product such as a node containing another node, a reflected entry stores
the contained `Type` value. Reflection does not recursively expand that Type's own
shape.

Creating a Pack, ProductShape, OrderedShape, UnorderedShape, Variant, or related
intrinsic static value never implicitly requests reflection of the value's own Type.

Capability derivation and shape reflection share lower-level canonical declaration
facts. Capability is not proved from reflection and reflection is not proved from
capability.

## 7. Structural static iteration

Static structural iteration is specified by [Static Evaluation](static-evaluation.md)
and uses these shapes directly.

Ordered value/type iteration:

```telos
for value in orderedProduct {
    // positional entries
}

for [label]: value in orderedProduct {
    // labeled entries
}
```

Unordered value/type iteration:

```telos
for [key]: value in unorderedProduct {
    ...
}

for [key]: FieldInfo(FieldType, presence) in UnorderedProductType {
    ...
}
```

Unordered static iteration follows canonical FieldKey order.

Only finite fixed entries/fields are iterated. Anonymous open tails are not expanded.

Heterogeneous static iteration preserves exact per-iteration entry refinement rather
than collapsing all elements into one imprecise common Type.

## 8. Structural equality and hashing

Pack uses structural semantic equality.

Two Packs are equal exactly when:

- they have the same lane structure;
- positional lengths and ordered values are equal;
- labeled lengths, labels, order, and corresponding values are equal.

Contained values use their ordinary Telos semantic equality. Hashing must be consistent
with equality, including the language's canonical numeric/Float equality policy.

Product shape equality is structural and does not erase nominal identity of the owning
data Type.

Type-containing structures compare/hash Type values through canonical semantic Type
identity/equality; they never recursively expand Type reflection to compare them.

## 9. Public-ontology cutover

C4 performs a hard public cutover to the Pack/Product model.

The following may remain internal optimized compiler terms but are not independent
public semantic abstractions:

```text
TypePack
LabeledTypePack
CompleteTypePack
RecordFields
RecordRow
```

Source-facing semantics use:

```text
PositionalPack<Type>
LabeledPack<Type>
CompletePack<Type>
UnorderedShape / FieldKey / FieldInfo / UnorderedTail
```

Specifications and new APIs must not reintroduce the retired public ontology.

## 10. Invariants

- **PRODUCT-DERIVED** — structural Product capabilities are compiler-derived and sealed.
- **PRODUCT-NOMINALITY** — payload-shape equality does not erase owning declaration
  identity.
- **PACK-DISTINCT** — Pack is distinct from Tuple, Unit, and Record.
- **PACK-FLAT-FREEDOM** — Pack semantics do not require recursive physical storage.
- **PACK-REST-O1** — semantic rest/view admits O(1) slice implementations.
- **PACK-CALL-FASTPATH** — fixed calls do not require Pack allocation.
- **FIELDKEY-STABLE** — canonical key ordering is stable and independent of runtime IDs.
- **TAIL-NOT-ITERABLE** — anonymous open tails are descriptions, not finite sequences.
- **REFLECTION-SHALLOW** — contained recursive Types remain Type references rather than
  recursively expanded shapes.

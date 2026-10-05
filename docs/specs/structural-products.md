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
    type Entries: CompletePack<Type>
    type PositionalEntries: PositionalPack<Type>
    type LabeledEntries: LabeledPack<Type>
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

`Pack` is a first-class immutable ordered structural transport product with two
independent lanes:

```text
positional lane: ordered sequence of positional values
labeled lane: ordered (Symbol, value) entries
```

Order within each lane participates in structural identity. Lane identity participates
in structural identity. Labeled entries use `Symbol` labels and duplicate labels are
invalid. Pack labels remain `Symbol`-only; `FieldKey` belongs exclusively to unordered
Record/product structure.

`Pack()` is the empty Pack. It is a proper first-class value with an immediate runtime
representation and is distinct from `Unit` / `()` and from the empty Record.

### 2.1 Pack types and remainders

The proper `Pack` type constructor has two independent ordered lanes and remainder
states:

```telos
intrinsic data Pack<***Es>(***Es)

type PositionalPack<T: Type> =
    Pack<*: T>

type LabeledPack<T: Type> =
    Pack<**: T>

type CompletePack<T: Type> =
    Pack<***: T>
```

Pack elements are required, positional elements precede labeled elements, labeled
entries have unique Symbol labels, and each lane preserves its own order.

The type constructor's anonymous remainder forms are:

```telos
Pack<*: T>       // typed positional remainder
Pack<**: T>      // typed labeled remainder
Pack<***: T>     // same typed remainder in both lanes
Pack<*: ...>     // unconstrained positional remainder
Pack<**: ...>    // unconstrained labeled remainder
Pack<***: ...>   // unconstrained remainder in both lanes
```

These are structural Pack type forms only when the origin resolves to the canonical
intrinsic Pack declaration.

There is a vital semantic distinction between expansion and remainder declaration:
- `Pack<*P>`, `Pack<**L>`, and `Pack<***E>` remain explicit expansions of existing
  type-level pack terms; they form the corresponding Pack from those lanes and do not
  declare an open remainder.
- `Pack<*: T>`, `Pack<**: T>`, and `Pack<***: T>` declare open typed lane remainders.
- `Pack<*: ...>`, `Pack<**: ...>`, and `Pack<***: ...>` declare unconstrained lane
  remainders.

Anonymous remainder syntax is not accepted by ordinary generic constructors.

Under `INC-P3-2`, C3 kinded domains (`TypePack`, `LabeledTypePack`, `CompleteTypePack`)
remain compiler/static kinded domains established by C3 and exposed through intrinsic
facades. Proper first-class Pack aliases (`PositionalPack<T>`, `LabeledPack<T>`,
`CompletePack<T>`) are proper Pack specializations and are not competing spellings for
the kinded domains.

### 2.2 Pack performance contract

Pack is specialized for variadic structural transport: construction, rest capture,
slicing, spreading, and static structural computation.

The implementation must preserve these semantic-performance properties:

- Pack may use flat contiguous storage, argument windows, slices, scalar replacement,
  compiler-virtual segments, or equivalent representations;
- lane lengths are O(1);
- a one-step rest/view can be represented in O(1) without copying the remaining Pack,
  sharing root backing storage;
- rest slicing in one lane preserves the opposite lane;
- Pack is not required to use recursively linked cons storage;
- materializing/finalizing n Pack entries is O(n);
- multi-spread lowering is O(total contributed entries), not repeated immutable
  concatenation (fused spread construction);
- an ordinary fixed-shape call does not allocate a Pack solely because its argument
  shape is semantically Pack-shaped.

These representation freedoms are not observable through Telos value semantics.

### 2.3 Pack views

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

### 3.1 Zero product and Unit

The syntax `()` denotes the zero-arity Tuple. Its canonical proper type is
`Unit = Tuple<>`, not a second empty Tuple type:

```text
Tuple([], []) ≡ Tuple<> ≡ Unit ≡ ()
```

The source alias `Unit` does not create a separate runtime owner or class;
runtime class mapping is specified by the VM and maps Unit to Tuple.
Positive-arity Tuple shapes remain structural Tuple types.

An empty Record remains the structural type `Record({})`, canonically aliased
as `EmptyRecord = Record<>`; it is not the same static type as `Unit`.

`Pack()` is the empty Pack and is distinct from both `Unit` / `()` and `EmptyRecord`.

### 3.2 Tuple formation, lane structure, and remainders

A Tuple type consists of two independent structural lanes:
1. an ordered **positional lane** containing positional elements;
2. an ordered **labeled lane** containing uniquely labeled elements with `Symbol` labels.

```text
Tuple(P, L, Rem_P, Rem_L)
```

where `P : TypePack`, `L : LabeledTypePack`, `Rem_P` is the positional remainder state,
and `Rem_L` is the labeled remainder state. Positional entries must precede labeled
entries.

The positional and labeled lanes have independent closed/open states:
- `Closed` — admits no elements beyond the declared fixed prefix;
- `Typed(T)` — admits zero or more additional elements in that lane constrained to `<: T`
  (`*: T` for positional, `**: T` for labeled);
- `Unconstrained` — admits zero or more additional elements in that lane without a common
  element type constraint (`*: ...` for positional, `**: ...` for labeled, or
  `...` / `***: ...` for both lanes).

Tuple complete remainder `(***: T)` is shorthand for `(*: T, **: T)`.
Tuple unconstrained remainder `(...)` is shorthand for `(*: ..., **: ...)`.

### 3.3 Tuple projections and operations

Tuple-family type members are domain-native operations:

```text
Positionals : TypePack
Labeled     : LabeledTypePack
At<I>       : Type
Map<F>      : Type
```

For `At<I>`, integer indexing uses 0-based indexing for `I >= 0` and end-relative
indexing `N + I` for `I < 0`. Type subscript is the same operation: `T[I] ≡ T::At<I>`.

### 3.4 Record formation, presence, and remainders

A Record type is formed from an unordered set of uniquely keyed fields and an optional
tail:

```text
Record(Fields, Tail)
```

Record keys belong to `FieldKey` (`Symbol | String | Int | Bool`).

Each field has an explicit presence qualifier:
- **Required** (`key: T`): the field must be present on instances of the record type.
- **Optional** (`key?: T`): the field may be absent; if present, its value must have type `T`.

Optionality is **member presence**, not `Option<T>` or `T | Unit`.

Record row tails have four categories:
1. `Closed` (`{ a: Int }`) — exactly the declared fields; admits no surplus fields.
2. `Captured(R)` (`{ a: Int, **R }`) — row polymorphism with correlated row variable `R`.
3. `Anonymous Typed(T)` (`{ a: Int, **: T }`) — admits zero or more additional fields, each constrained to type `T`.
4. `Anonymous Unconstrained` (`{ a: Int, ... }` or `{ a: Int, **: ... }`) — admits zero or more additional fields without a common value type constraint.

Captured row tails preserve correlated row identity. Anonymous remainders (`...`, `**: T`)
introduce no row variable or solver identity.

### 3.5 Record subtyping and compatibility

Ordinary `<:` applies strictly between proper `Type` terms:
1. **Target required field `k: T`**: source must provably contain `k` with Required presence (`k: S` where `S <: T`). An optional source field `k?: S` cannot satisfy a required target field (`{ k?: S } </: { k: T }`).
2. **Target optional field `k?: T`**:
   - If source provably contains `k: S` (Required) or `k?: S` (Optional), `S <: T` is required.
   - If source is closed and omits `k`, the target field is satisfied (absence).
   - If source has an anonymous remainder that could contain `k`, the source remainder must be typed `**: S` with `S <: T` to prove compatibility; an unconstrained source remainder `...` cannot prove compatibility for an optional target field.
3. **Extra source fields**:
   - Closed target rejects surplus source fields (`{ a: Int, b: String } </: { a: Int }`).
   - Open target with typed remainder `**: T` requires every surplus source field (and source remainder) to satisfy `<: T`.
   - Open target with unconstrained remainder `...` accepts surplus source fields.
4. **Source openness**: an open source cannot satisfy a closed target.

### 3.6 Record values and spread

Record value literals use bare braces, such as `{name: value}`. A `**source` entry
contributes a statically projectable record shape. A Record source contributes its closed
or symbolic row. A labeled Tuple source may contribute only its labeled lane, through
checked projection; its positional lane does not participate.

A Map is not a legal Record spread source: Map does not encode a statically exact key set.
Record spread does not create a mixed Tuple/Record destructuring rule or a generalized
product protocol.

## 4. Ordered shape

Ordered-product constituents are **entries**. A labeled entry has a `Symbol` **label**.

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
    OrderedTail<PositionalPack<Type>>

type LabeledTail =
    OrderedTail<LabeledPack<Type>>

intrinsic data OrderedShape(
    _ entries: CompletePack<Type>,
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

Ordered-product labels remain `Symbol`s. `FieldKey` belongs exclusively to unordered
Record/product structure.

FieldKey has a language-defined version-stable total order used for canonical
reflection, row normalization, fingerprints, and metadata:

```text
Symbol < String < Int < Bool
```

Within a kind:
- Symbol ordering uses stable semantic symbol spelling/identity, never process-local
  interner IDs;
- String ordering is by canonical UTF-8 string content;
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
- labeled lengths, Symbol labels, order, and corresponding values are equal.

Contained values use their ordinary Telos semantic equality. Hashing must be consistent
with equality, including the language's canonical numeric/Float equality policy.

Product shape equality is structural and does not erase nominal identity of the owning
data Type.

Type-containing structures compare/hash Type values through canonical semantic Type
identity/equality; they never recursively expand Type reflection to compare them.

## 9. Public-ontology cutover and `INC-P3-2` reconciliation

C4 establishes the Pack/Product model as the canonical public structural language.

Under `INC-P3-2`:
- `TypePack`, `LabeledTypePack`, and `CompleteTypePack` remain canonical compiler/static
  kinded domains established by C3. They are published as intrinsic Universe anchors and
  are not replaced or shadowed by proper Pack types.
- `Pack` is the first-class proper structural value and type introduced by C4.P3.
- `PositionalPack<T>`, `LabeledPack<T>`, and `CompletePack<T>` are proper first-class
  Pack specializations (`Pack<*: T>`, `Pack<**: T>`, `Pack<***: T>`).
- Internal `TypePack` machinery is not replaced or retired merely because first-class
  `Pack` now exists.
- For unordered products, the public ontology uses `UnorderedShape`, `FieldKey`,
  `FieldInfo`, and `UnorderedTail` instead of the internal `RecordFields` / `RecordRow`
  implementation terminology.

## 10. Invariants

- **PRODUCT-DERIVED** — structural Product capabilities are compiler-derived and sealed.
- **PRODUCT-NOMINALITY** — payload-shape equality does not erase owning declaration identity.
- **PACK-DISTINCT** — Pack is a proper first-class structural type/value, distinct from Tuple, Unit, and Record.
- **PACK-LABELS-SYMBOL** — Pack and Tuple labels remain `Symbol`-only; `FieldKey` belongs exclusively to unordered Record/product structure.
- **PACK-FLAT-FREEDOM** — Pack semantics do not require recursive physical storage.
- **PACK-REST-O1** — semantic rest/view admits O(1) slice implementations over shared root backing.
- **PACK-CALL-FASTPATH** — fixed calls do not require Pack allocation.
- **PACK-FUSED-SPREAD** — multi-spread construction may be fused into one operation.
- **FIELDKEY-STABLE** — canonical key ordering is stable and independent of runtime IDs.
- **TAIL-NOT-ITERABLE** — anonymous open tails are descriptions, not finite sequences.
- **REFLECTION-SHALLOW** — contained recursive Types remain Type references rather than recursively expanded shapes.

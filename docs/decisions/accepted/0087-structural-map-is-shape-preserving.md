# TDR-0087 — Structural `Map<F>` preserves shape and transforms payload types

- Ratified: 2026-09-21
- Status: Accepted
- Date: 2026-09-21
- Supersedes: none
- Amends: none
- Depends on: [TDR-0081](./0081-tuple-and-record-are-transparent-structural-value-products.md), [TDR-0083](./0083-type-level-domains-and-typed-compile-time-parameters.md), [TDR-0085](./0085-parameterized-type-members-and-symbolic-projections.md), [TDR-0086](./0086-type-level-subscript-is-tuple-at.md)
- Related: LANG007; Tuple/Record structural typing; higher-kinded type application
- Repository evidence revision: `aureat/phalcom-lang@5b63ade8073d13d6b528af4774181665c30cbd47`

## Context

LANG007 requires a structural type transformation that applies one unary type constructor to every payload type of a structural shape.

Examples:

```phalcom
(Int, String)::Map<Option>
```

should produce:

```phalcom
(Option<Int>, Option<String>)
```

and:

```phalcom
#{name: String, age: Int}::Map<Option>
```

should produce:

```phalcom
#{name: Option<String>, age: Option<Int>}
```

The same operation also needs to work over abstract structural domains:

```phalcom
P::Map<Option> // P: TypePack
R::Map<Option> // R: RecordRow
```

The design question is whether these are unrelated conveniences attached independently to Tuple, Record, packs, and rows, or one coherent structural algebra.

A second issue is open structure. If an open Record row maps only its currently visible fields but leaves the unknown tail unchanged, the resulting type is unsound: unknown fields would not have had the constructor applied.

A third issue is higher-kindedness. `Option` is a type-level form of kind `Type -> Type`, while `Int` is merely a proper type. `Map` must distinguish those cases through ordinary kind checking.

## Decision

### 1. `Map<F>` is a shape-preserving structural type operation

`Map<F>` transforms payload proper types while preserving the structural shape of its subject.

The parameter contract is:

```text
F : Type -> Type
```

`F` may be any well-formed type-level form of that kind.

No nominal special case for `Option` exists.

### 2. TypePack mapping preserves sequence order

For:

```text
P = [T₀, T₁, ..., Tₙ]
```

define:

```text
MapPack(P, F)
    =
[F<T₀>, F<T₁>, ..., F<Tₙ>]
```

with result kind:

```text
TypePack
```

Order and arity are preserved.

### 3. LabeledTypePack mapping preserves labels and label order

For:

```text
L = [(ℓ₀,T₀), ..., (ℓₙ,Tₙ)]
```

define:

```text
MapLabeledPack(L, F)
    =
[(ℓ₀,F<T₀>), ..., (ℓₙ,F<Tₙ>)]
```

with result kind:

```text
LabeledTypePack
```

Labels are unchanged.

Label order is unchanged.

### 4. Tuple mapping preserves Tuple shape

For a Tuple structural type:

```text
Tuple(P, L)
```

define:

```text
MapTuple(Tuple(P, L), F)
    =
Tuple(
    MapPack(P, F),
    MapLabeledPack(L, F)
)
```

Tuple mapping preserves:

```text
total arity
positional/labeled lane boundary
ordered labels
component order
```

Examples:

```phalcom
(Int, String)::Map<Option>
    ==
(Option<Int>, Option<String>)
```

```phalcom
(Int, name: String)::Map<Option>
    ==
(Option<Int>, name: Option<String>)
```

### 5. RecordRow mapping preserves keys and maps every payload

For a closed row:

```text
R = { ℓ₀:T₀, ..., ℓₙ:Tₙ }
```

define:

```text
MapRow(R, F)
    =
{ ℓ₀:F<T₀>, ..., ℓₙ:F<Tₙ> }
```

with result domain:

```text
RecordRow
```

Row key identity remains order-independent.

### 6. Open row mapping maps the symbolic tail

For an open row:

```text
{ fields | R }
```

define:

```text
MapRow({ fields | R }, F)
    =
{ map(fields, F) | MapRow(R, F) }
```

The following transformation is explicitly incorrect:

```text
{ fields | R }
    ->
{ map(fields, F) | R }
```

because it leaves unknown fields untransformed.

An abstract mapped tail therefore remains a stable symbolic row transformation until it can reduce.

### 7. Record mapping is row mapping lifted back into the Record type

For:

```text
Record(R)
```

define:

```text
MapRecord(Record(R), F)
    =
Record(MapRow(R, F))
```

Record key identity and openness are preserved.

### 8. Abstract mapping is a valid symbolic projection

The following are well-formed even when the subject is not concrete:

```phalcom
P::Map<Option>
L::Map<Option>
R::Map<Option>
```

provided the subject domain and `F`'s kind are established.

Such expressions remain symbolic until enough information exists to reduce them.

They must not be converted to `Dynamic` merely because their exact payload sequence is unknown.

### 9. Map uses ordinary type application

Every payload transformation:

```text
F<T>
```

uses Phalcom's ordinary kind-checked type application semantics.

`Map` does not own a second constructor-application engine.

Therefore:

```phalcom
(Int, String)::Map<Option>
```

is well formed because:

```text
Option : Type -> Type
```

while:

```phalcom
(Int, String)::Map<Int>
```

is invalid because:

```text
Int : Type
```

### 10. Map commutes with valid Tuple indexing

For every valid Tuple index `I`:

```text
(T::Map<F>)::At<I>
    ≡
F<T::At<I>>
```

This is a semantic law of the operation.

### 11. Empty shapes remain empty

Mapping a zero-component structure preserves zero shape under the accepted structural-product semantics.

No synthetic payload element is introduced merely to carry the constructor.

### 12. LANG007 Map is closed, not an open type-family facility

This decision defines one compiler-known structural operation.

It does not implicitly introduce:

```text
arbitrary user-defined type transforms
open rewrite families
type-level Fold
type-level Filter
type-level Zip
constructor composition
automatic Map fusion
```

Each additional operation requires its own accepted language rule.

### 13. Map does not implicitly distribute over unions

For the initial feature, a structural Map projection does not automatically branch over a union subject.

Union-distributive type computation, if desired later, requires a separate ruling defining per-branch invalidity, normalization, and complexity behavior.

## Consequences

Tuple, Record, TypePack, LabeledTypePack, and RecordRow mapping share one algebra while retaining their different identity laws.

The implementation can reuse one intrinsic member/projection architecture rather than adding unrelated Tuple-map and Record-map systems.

Open Record rows remain sound because unknown fields are transformed symbolically.

Generic code can preserve `P::Map<F>` and `R::Map<F>` without requiring concrete shapes.

The compiler must represent symbolic mapped rows/packs as stable semantic terms and preserve them through substitution, inference, and metadata.

## Rejected alternatives

### Tuple-only Map

Rejected because Record and pack mapping have the same payload-transform law and should share the same type-level concept.

### Leave open RecordRow tails unchanged

Rejected as semantically incorrect.

### Require a nominal constructor such as `Option` specifically

Rejected because the contract is kind-based: any `Type -> Type` form is valid.

### Distribute Map over unions automatically

Rejected for the initial feature because it introduces branchwise reduction and error semantics beyond this decision.

### Fuse nested maps automatically

Rejected. Constructor composition/fusion is an optimization or future algebraic rule, not required for correct Map semantics.

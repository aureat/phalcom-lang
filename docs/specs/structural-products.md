# Structural Products

**Status:** Normative
**Language area:** Static Tuple/Record typing and structural type operations
**Specification role:** Authoritative specification for structural Tuple/Record formation and closed type-level operations

This specification defines static structural Tuple and Record types and the
closed type-level operations provided for them and their shape domains. Runtime
storage, product layouts, garbage collection, and VM descriptors remain owned
by [VM Products](vm/products.md) and [VM Values and Objects](vm/values-and-objects.md).

## 1. Structural domains

The type-level domains are distinct:

```text
TypePack          ordered sequence of proper type forms
LabeledTypePack   ordered sequence of (label, proper type) pairs
RecordFields      unordered key/type associations, possibly with an open tail
```

`LabeledTypePack` is not `RecordFields`. A labeled pack preserves sequence order;
a record row preserves key association and is unordered for structural identity.

The structural mapping algebra is:

| Domain | Payload | Labels | Identity/order law | `Map<F>` result |
|---|---|---|---|---|
| `TypePack` | proper types | none | ordered | `TypePack` |
| `LabeledTypePack` | proper types | labels | ordered | `LabeledTypePack` |
| `RecordFields` | proper types | keys | unordered key set | `RecordFields` |
| Tuple proper type | pack plus labeled pack | ordered labels | ordered | Tuple proper type |
| Record proper type | row | record keys | unordered key set | Record proper type |

Universe publishes intrinsic `type data` anchors for `TypePack`,
`LabeledTypePack`, `CompleteTypePack`, and `RecordFields`. These names expose
the existing compiler-owned domains; they do not create replacement semantic
identities.

Expansion prefixes and semantic kinds are separate concepts. When the kind is
omitted, `*Ts` defaults to `TypePack`, `**Fs` to `RecordFields`, and `***Es` to
`CompleteTypePack`; there is no contextual kind inference. Explicit compatible
kind annotations remain available.

## 2. Tuple formation

A positive-arity Tuple type consists of an ordered positional prefix followed by
an ordered labeled suffix:

```text
Tuple(P, L)
```

where `P : TypePack` and `L : LabeledTypePack`.

For example:

```phalcom
(Int, String, name: Symbol)
```

has positional pack `[Int, String]` and labeled pack `[(name, Symbol)]`.
Positional entries must precede labeled entries. Tuple labels are unique, and
the ordered label sequence participates in Tuple identity.

## 3. Record formation

A Record type is formed from a row:

```text
Record(R)
```

where `R : RecordFields`. A closed row contains a finite key/type association;
an open row additionally contains an abstract remainder. Record structural
identity is determined by key/type association, not source or presentation
order. Canonical field sorting is an implementation normalization, not a
source-level ordering rule.

## 4. Zero product and `Unit`

The syntax `()` denotes the zero-arity structural product. Its canonical proper
type is `Unit`, not a second empty Tuple type:

```text
Tuple([], []) ≡ Unit ≡ ()
```

This semantic rule is independent of the runtime behavior class of `Unit`.
Positive-arity Tuple shapes remain structural Tuple types.

An empty Record remains the structural type `Record({})`; it is not the same
static type as `Unit`. The VM may realize its empty runtime value with the
canonical zero-product representation, as long as any retained static or
reflective Record identity is preserved according to the applicable boundary.

The empty product operations are:

```text
()::Positionals = []
()::Labeled     = []
()::Map<F>      ≡ ()
```

Indexing the empty product is invalid for every integer index.

## 5. Tuple projections

Tuple-family type members are domain-native operations:

```text
Positionals : TypePack
Labeled     : LabeledTypePack
At<I>       : Type
Map<F>      : Type
```

For a Tuple `Tuple(P, L)`, `Positionals` returns `P` and `Labeled` returns
`L`. They are type-level results, not runtime Tuple values.

`At<I>` accepts only a compile-time integer argument (`I :const Int`) in the
initial release. Symbol lookup and Range slicing remain runtime operations and
are not implicitly type-level `At` forms.

For a concrete Tuple with total arity `N`, integer indexing uses:

```text
j = I       when I >= 0
j = N + I   when I < 0
```

The projection is valid exactly when `0 <= j < N`; otherwise it is statically
invalid. The linear order is all positional entries followed by labeled entries
in declared order:

```phalcom
(Int, String, name: Symbol, flag: Bool)::At<0>  == Int
(Int, String, name: Symbol, flag: Bool)::At<2>  == Symbol
(Int, String, name: Symbol, flag: Bool)::At<-1> == Bool
```

Type subscript is the same operation:

```text
T[I] ≡ T::At<I>
```

The two spellings share one semantic type-member identity.

## 6. Shape-preserving mapping

`Map<F>` requires:

```text
F : Type -> Type
```

It transforms payload types while preserving the subject shape and its identity
laws. `Int` is therefore invalid as `F`, while `Option` is valid when
`Option : Type -> Type`.

### 6.1 Packs

```text
MapPack([T₀, ..., Tₙ], F)
    = [F<T₀>, ..., F<Tₙ>]
```

Order is preserved. For a labeled pack:

```text
MapLabeledPack([(l₀,T₀), ..., (lₙ,Tₙ)], F)
    = [(l₀,F<T₀>), ..., (lₙ,F<Tₙ>)]
```

Labels and label order are preserved.

### 6.2 Tuple

```text
Tuple(P, L)::Map<F>
    = Tuple(MapPack(P,F), MapLabeledPack(L,F))
```

The result is canonical structural Tuple type, or `()` when the subject is the
zero product.

### 6.3 Record rows and Records

For a closed row:

```text
MapRow({ l₀:T₀, ..., lₙ:Tₙ }, F)
    = { l₀:F<T₀>, ..., lₙ:F<Tₙ> }
```

For an open row:

```text
MapRow({ l₀:T₀, ..., lₙ:Tₙ, **R }, F)
    = { l₀:F<T₀>, ..., lₙ:F<Tₙ>, **MapRow(R,F) }
```

The unknown tail must be transformed; it must not remain unchanged. Record
mapping is:

```text
Record(R)::Map<F> = Record(MapRow(R,F))
```

The result preserves the Record key set and unordered identity law.

## 7. Mapping laws

For every valid operation:

```text
shape(S::Map<F>) = shape(S)
labels(T::Map<F>) = labels(T)
(T::Map<F>)::At<I> ≡ F<T::At<I>>
Lookup(Record(R)::Map<F>, K) ≡ F<Lookup(Record(R), K)>
```

The laws apply when the relevant index or key exists. This release does not
require map-fusion syntax or implicit union distribution. A structural
projection over a union reduces only when ordinary member applicability makes
the exact subject domain available; it does not automatically distribute over
union branches.

## 8. Symbolic operations and normalization

The following are valid semantic forms when their domains are established:

```text
P::Map<F>       when P : TypePack
L::Map<F>       when L : LabeledTypePack
R::Map<F>       when R : RecordFields
T::At<I>        when T and I satisfy member applicability/domain rules
T::Map<F>
```

They may remain symbolic when their subject or argument is abstract. A concrete
invalid index, wrong mapper kind, or non-constant index is a static error. A
valid but unreduced operation is stuck, not invalid.

For `T::At<I>`, validity requires that the available static information proves
Tuple-family member applicability and `I :const Int`. A parameter known only as
`T: Type` does not satisfy that obligation. No structural operation implicitly
distributes over a union in the initial surface.

Normalization is closed and compiler-known. It does not execute arbitrary
Phalcom code, inspect arbitrary runtime values, or turn LANG007 into an open
type-family language.

## 9. Runtime boundary

Static Tuple/Record types and type-level packs/rows are semantic products. They
do not imply runtime `TypePack` or `RecordFields` objects. Runtime Tuple and Record
values retain their existing behavior and representation contracts, including
zero-product normalization, Tuple label ordering, and Record key-set identity.

Tuple and Record structural types use their canonical Universe family
declarations as behavior owners for inherent implementations. The authorization
and overlap rules are specified by
[Implementation Declarations](objects/impl.md). Other structural type
expressions do not gain inherent behavior owners through this rule.

## 10. Invariants

| ID | Requirement |
|---|---|
| **STRUCT-TUPLE-LANE-ORDER** | A Tuple is an ordered positional prefix followed by an ordered labeled suffix. |
| **STRUCT-TUPLE-LABEL-ORDER** | Tuple label order participates in structural identity. |
| **STRUCT-RECORD-KEY-IDENTITY** | Record identity is determined by key/type association, not presentation order. |
| **STRUCT-PACK-ROW-DISTINCT** | `LabeledTypePack` and `RecordFields` are distinct domains and identities. |
| **STRUCT-ZERO-PRODUCT** | `()` is the zero product and canonicalizes to `Unit`. |
| **STRUCT-AT-IDENTITY** | `T[I]` and `T::At<I>` are one semantic operation. |
| **STRUCT-AT-BOUNDS** | A concrete out-of-range Tuple index is statically invalid. |
| **STRUCT-MAP-SHAPE** | `Map<F>` preserves structural shape and maps every payload type. |
| **STRUCT-MAP-OPEN** | Mapping an open row maps its unknown tail symbolically. |
| **STRUCT-MAP-KIND** | `Map<F>` requires `F : Type -> Type`. |
| **LANG007-CLOSED-CALCULUS** | Structural operations are a closed compiler-known set, not arbitrary compile-time execution. |

## 11. Examples

```phalcom
(Int, String)::Positionals
// [Int, String] : TypePack

(Int, name: String)::Labeled
// [(name, String)] : LabeledTypePack

(Int, String)[0]
// Int

(Int, String)::At<-1>
// String

(Int, String)::Map<Option>
// (Option<Int>, Option<String>)

(Int, name: String)::Map<Option>
// (Option<Int>, name: Option<String>)

{name: String, age: Int}::Map<Option>
// {name: Option<String>, age: Option<Int>}

P::Map<Option>
// valid symbolic result when P : TypePack
```

The following are invalid:

```phalcom
(Int, String)::Map<Int>       // mapper has kind Type, not Type -> Type
(Int, String)::At<2>          // concrete index is out of range
(Int, String)::At<runtimeIndex> // argument is not a compile-time Int term
T::Map<Option>                // unavailable when T is known only as T: Type
```

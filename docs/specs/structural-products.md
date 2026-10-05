# Structural Products

**Status:** Normative
**Language area:** Static Tuple/Record/Pack typing and structural type operations
**Specification role:** Authoritative specification for structural Tuple/Record/Pack formation and closed type-level operations

This specification defines static structural Tuple, Record, and proper Pack
types and the closed type-level operations provided for them and their shape
domains. Runtime storage, product layouts, garbage collection, and VM
descriptors remain owned by [VM Products](vm/products.md) and [VM Values and
Objects](vm/values-and-objects.md).

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

## 2. Tuple formation and lane structure

A Tuple type consists of two independent structural lanes:
1. an ordered **positional lane** containing positional elements;
2. an ordered **labeled lane** containing uniquely labeled elements.

```text
Tuple(P, L, Rem_P, Rem_L)
```

where `P : TypePack`, `L : LabeledTypePack`, `Rem_P` is the positional remainder state, and `Rem_L` is the labeled remainder state.

Positional entries must precede labeled entries. Tuple fixed elements (both positional and labeled) are always required (`name: T`). Tuple variability is expressed exclusively through positional and labeled lane remainders (`*: T`, `**: T`).

### 2.1 Tuple remainders
The positional and labeled lanes have independent closed/open states:
- `Closed` — admits no elements beyond the declared fixed prefix;
- `Typed(T)` — admits zero or more additional elements in that lane, each constrained to subtype of `T` (`*: T` for positional, `**: T` for labeled);
- `Unconstrained` — admits zero or more additional elements in that lane without a common element type constraint (`*: ...` for positional, `**: ...` for labeled, or `...` / `***: ...` for both lanes).

Tuple complete remainder `(***: T)` is shorthand for `(*: T, **: T)`.
Tuple unconstrained remainder `(...)` is shorthand for `(*: ..., **: ...)`.

### 2.2 Proper Pack types

`Pack` is a proper structural transport type with the same two ordered lanes
and remainder states as Tuple:

```text
Pack(P, L, Rem_P, Rem_L) : Type
```

Pack elements are required, positional elements precede labeled elements,
labeled entries have unique labels, and each lane preserves its own order. A
typed or unconstrained remainder admits zero or more additional entries in its
lane under the rules in §2.1. An empty Pack remains a Pack and is distinct from
Unit and every Tuple type, including an empty Tuple.

The type constructor's anonymous remainder forms are:

```phalcom
Pack<*: T>       // typed positional remainder
Pack<**: T>      // typed labeled remainder
Pack<***: T>     // same typed remainder in both lanes
Pack<*: ...>     // unconstrained positional remainder
Pack<**: ...>    // unconstrained labeled remainder
Pack<***: ...>   // unconstrained remainder in both lanes
```

These are structural Pack type forms only when the origin resolves to the
canonical intrinsic Pack declaration. `Pack<*P>`, `Pack<**L>`, and
`Pack<***E>` remain explicit expansions of existing type-level pack terms;
they form the corresponding Pack from those lanes and do not declare an open
remainder. Anonymous remainder syntax is not accepted by ordinary generic
constructors. The kinded `TypePack`, `LabeledTypePack`, and `CompleteTypePack`
domains remain distinct from the proper `Pack` type.

## 3. Record formation and presence

A Record type is formed from a set of uniquely labeled fields and an optional tail:

```text
Record(Fields, Tail)
```

where `Fields` is an unordered key/type/presence association, and `Tail` is the row tail state.

Each field has an explicit presence qualifier:
- **Required** (`name: T`): the field must be present on instances of the record type.
- **Optional** (`name?: T`): the field may be absent; if present, its value must have type `T`.

Optionality is **member presence**, not `Option<T>` or `T | Unit`. A field `{ name?: String }` is satisfied by an instance with no `name` field, or an instance with `name: String`.

### 3.1 Record remainders and row tails
A Record row tail has one of four categories:
1. `Closed` (`{ a: Int }`) — exactly the declared fields; admits no surplus fields.
2. `Captured(R)` (`{ a: Int, **R }`) — row polymorphism with correlated row variable `R`.
3. `Anonymous Typed(T)` (`{ a: Int, **: T }`) — admits zero or more additional fields, each constrained to type `T`.
4. `Anonymous Unconstrained` (`{ a: Int, ... }` or `{ a: Int, **: ... }`) — admits zero or more additional fields without a common value type constraint.

Captured row tails preserve correlated row identity. Anonymous remainders (`...`, `**: T`) introduce no row variable or solver identity.

### 3.2 Record subtyping and compatibility
Ordinary `<:` applies strictly between proper `Type` terms (`KindId::TYPE`).

For Record types:
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

### 3.3 Record values and spread

Record value literals use bare braces, such as `{name: value}`. A `**source`
entry contributes a statically projectable `RecordFields` shape. A Record
source contributes its closed or symbolic row. A labeled Tuple source may
contribute only its labeled lane, through the checked
`LabeledTypePack`-to-`RecordFields` projection; its positional lane does not
participate. Row composition preserves symbolic tails and enforces the
canonical duplicate/lacks obligations.

A Map is not a legal Record spread source, including an explicit Map literal:
Map type does not encode a statically exact key set. The source forms remain
distinct: `{...}` is Record syntax, `Map {...}` is Map syntax, and `#{...}` is
Set syntax. This identifies their heads only; it does not redefine broader
Map or Set entry semantics.

Record spread does not create a mixed Tuple/Record destructuring rule or a
generalized product protocol. The shared runtime product substrate does not
collapse Tuple, Record, Map, and Set into one source-level product category.

## 4. Zero product and `Unit`

The syntax `()` denotes the zero-arity Tuple. Its canonical proper type is
`Unit = Tuple<>`, not a second empty Tuple type:

```text
Tuple([], []) ≡ Tuple<> ≡ Unit ≡ ()
```

The source alias `Unit` does not create a separate runtime owner or class;
runtime class mapping is specified by the VM and maps Unit to Tuple.
Positive-arity Tuple shapes remain structural Tuple types.

An empty Record remains the structural type `Record({})`, canonically aliased
as `EmptyRecord = Record<>`; it is not the same static type as `Unit`. Runtime
representation and class mapping are specified by the VM value and product
specifications.

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
| **STRUCT-ZERO-PRODUCT** | `()` is the zero-arity Tuple and canonicalizes to `Unit = Tuple<>`; an empty Record canonicalizes to distinct `EmptyRecord = Record<>`. |
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

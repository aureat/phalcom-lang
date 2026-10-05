# Type-Level and Static Domains

**Status:** Normative  
**Semantic ownership:** proper Types, static value domains, kinded pack domains,
generic/static parameter domains, parameter-pack domain interpretation, static data
domains, and canonical type/static terms.

Structural Pack/Product formation and remainder details are owned by
[Structural Products](structural-products.md). Executable static computation is owned by
[Static Evaluation](static-evaluation.md).

## 1. Domain judgments

Telos distinguishes proper runtime Types, compile-time/static value domains, and
compiler kinded domains.

Conceptually:

```text
Type
StaticDomain(D)
ordinary static value Type/domain
Kinded pack domains: TypePack, LabeledTypePack, CompleteTypePack
```

Examples:

```text
Int                         : Type
Tuple<Int, String>          : Type
Pack<Int, String>           : PositionalPack<Type>
Shape::Scalar               : StaticDomain(Shape)
#name                       : Symbol
42                          : Int
*Ts                         : TypePack
**Ls                        : LabeledTypePack
***Es                       : CompleteTypePack
```

A static value's domain is not automatically a kind. Compile-time values remain typed
semantic values.

## 2. Proper Types

A **proper Type** is a runtime/value type accepted wherever the language requires a
`Type`.

Nominal classes/data types, structural Tuple/Record types, callable types, unions, and
other ordinary type forms remain governed by their owning specifications.

A reflected or static Type value is represented canonically by the semantic type
system; it is not a runtime metaobject merely because static Telos can manipulate it.

## 3. Static data domains

An intrinsic or source-defined static-data declaration owns a nominal static domain.
Its cases/constructors produce canonical semantic terms in that domain.

```telos
intrinsic type data Example {
    A
    B(_ value: Int)
}
```

Such terms may participate in static values, generic/static arguments, associated
static results, reflection, residual computation, and metadata when permitted by the
owning declaration.

Distinct static-data declarations have distinct semantic domains even when their source
payload shapes coincide.

## 4. Kinded domains and first-class Pack types

Under `INC-P3-2`, Telos maintains a strict distinction between C3 kinded domains and
C4 first-class Pack types. They are not competing spellings or aliases for the same
thing.

### 4.1 Kinded pack domains

The kinded/static pack domains established by C3 remain canonical:

```text
TypePack          ordered sequence of proper type forms
LabeledTypePack   ordered sequence of (Symbol, proper type) pairs
CompleteTypePack  ordered complete two-lane sequence of proper type forms
```

Universe publishes intrinsic `type data` anchors for `TypePack`, `LabeledTypePack`, and
`CompleteTypePack`. These expose the existing compiler-owned domains for kind
classification; they do not create runtime classes or forgeable user types.

Internal `TypePack` machinery, canonical compositions (`TypePackConcat`), and kinded
binders remain intact and are not replaced merely because first-class `Pack` exists.

### 4.2 Proper first-class Pack types

`Pack` is the first-class proper structural value and type introduced by `LANG007.C4.P3`.
Its specialized proper aliases are:

```telos
type PositionalPack<T: Type> =
    Pack<*: T>

type LabeledPack<T: Type> =
    Pack<**: T>

type CompletePack<T: Type> =
    Pack<***: T>
```

These are proper `Type` specializations parameterized by element type `T`. They are not
aliases for `TypePack`, `LabeledTypePack`, or `CompleteTypePack`.

Pack formation, remainder syntax (`Pack<*: T>`, `Pack<*: ...>`), O(1) rest slicing,
and one-step view decomposition are owned by [Structural Products](structural-products.md).

### 4.3 Unordered product domain

For unordered reflection, the public ontology uses `UnorderedShape`, `FieldKey`,
`FieldInfo`, and `UnorderedTail`. Internal `RecordRow` representations remain compiler
mechanisms and do not define a separate competing source-visible ontology.

## 5. Generic/static parameter domains

Generic parameters may bind proper Types, static values, or kinded pack domains.

Examples:

```telos
<T: Type>
<N: Int>
<K: FieldKey>
<*Ts: TypePack>
<**Ls: LabeledTypePack>
<***Es: CompleteTypePack>
```

Expansion prefixes and semantic kinds are separate concepts:
- `*Ts` defaults to `TypePack`;
- `**Fs` defaults to `RecordFields` in Record contexts or `LabeledTypePack` in Tuple/Pack contexts;
- `***Es` defaults to `CompleteTypePack`.

In ordered-Pack/static-computation parameter positions, the expansion binders preserve
Pack lane semantics:

```telos
<*Ts>      // positional lane capture
<**Ls>     // labeled lane capture
<***Es>    // complete Pack capture
```

In Record contexts, `Record<**Fs>` denotes unordered field structure, not an ordered
labeled Pack. Contextual structural meaning is owned by
[Structural Products](structural-products.md).

## 6. Generic/static arguments

A generic/static argument may be any semantically valid argument for its declared
parameter domain, including:

- a proper Type value;
- a primitive static value;
- a static-data term;
- a Pack static value;
- a structural reflection value;
- a normalized symbolic residual where the surrounding generic computation permits it.

Argument checking is domain-aware. A value belonging to one static domain is not
accepted merely because another domain has an isomorphic representation.

## 7. Canonical identity

Canonical semantic identities are stable within the compiler's semantic model.

Stable identity never relies on:

- AST pointer identity;
- runtime object identity;
- VM Symbol numeric IDs;
- arena allocation order in exported metadata;
- pretty-printed source text.

Metadata uses stable semantic references and content encodings appropriate to each domain.

## 8. Symbolic terms

A static expression may remain symbolic when generic/static parameters prevent
reduction.

Symbolic terms are represented by the normalized semantic residual model specified by
[Static Evaluation](static-evaluation.md).

A symbolic term retains its declared result domain and is not converted to an invented
fallback value.

## 9. Result domains

Parameterized `type` declarations and type/static members may return any supported
static result domain, not only `Type`.

Examples:

```telos
type IsSubtype<A: Type, B: Type>: Bool = ...
type Count<*P>: Int { ... }
type ShapeOf<T: Product>: ProductShape = ...
```

An omitted result annotation on an ordinary type definition defaults to `Type`.

Associated and static member specifications must preserve their general result
domain rather than collapsing every result to a kind-only form.

## 10. Invariants

| Invariant | Rule |
| --- | --- |
| **TL-PROPER-TYPE** | Proper runtime Types remain distinct from ordinary static values. |
| **TL-STATIC-TYPED** | Compile-time values have semantic Types/domains; they are not “kinds because they are compile time”. |
| **TL-DOMAIN-SEPARATION** | Kinded domains (`TypePack`, `LabeledTypePack`, `CompleteTypePack`) and proper Pack types (`Pack`, `PositionalPack<T>`, `LabeledPack<T>`, `CompletePack<T>`) are distinct; neither is an alias for the other. |
| **TL-ROW-PUBLIC** | Public unordered reflection uses the Product/UnorderedShape model, not RecordRow/RecordFields implementation names. |
| **TL-CANONICAL** | Canonical static/type identity is independent of source spelling and runtime object identity. |
| **TL-RESULT-DOMAIN** | Static/type definitions and members preserve explicit non-Type result domains. |
| **TL-SYMBOLIC-DOMAIN** | Residual symbolic terms retain their declared result domain. |

## 11. Boundaries

This specification does not prescribe TypeStore node layout, interning tables, solver
algorithms, metadata pool layout, or VM representation.

Those implementation choices may retain optimized historical TypePack and RecordRow
nodes provided all source-visible semantics conform to the ratified model.

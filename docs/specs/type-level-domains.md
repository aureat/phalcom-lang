# Type-Level and Static Domains

**Status:** Normative  
**Semantic ownership:** proper Types, static value domains, generic/static parameter
domains, parameter-pack domain interpretation, static data domains, and canonical
type/static terms.

Structural Pack/Product details are owned by
[Structural Products](structural-products.md). Executable static computation is owned by
[Static Evaluation](static-evaluation.md).

## 1. Domain judgments

Telos distinguishes proper runtime Types from compile-time/static value domains.

Conceptually:

```text
Type
StaticDomain(D)
ordinary static value Type/domain
```

Examples:

```text
Int                         : Type
Tuple<Int, String>          : Type
Pack<Int, String>           : PositionalPack<Type>
Shape::Scalar               : StaticDomain(Shape)
#name                       : Symbol
42                          : Int
```

A static value's domain is not automatically a kind. Compile-time values remain typed
semantic values.

## 2. Proper Types

A **proper Type** is a runtime/value type accepted wherever the language requires a
`Type`.

Nominal classes/data types, structural Tuple/Record types, callable types, unions and
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

## 4. Pack-based type collections

C4's public type-pack model is ordinary Pack specialization:

```telos
type PositionalTypePack =
    PositionalPack<Type>

type LabeledTypePack =
    LabeledPack<Type>

type CompleteTypePack =
    CompletePack<Type>
```

These are not independent public kind identities.

The compiler may retain specialized internal terms historically named
`TypePack`, `LabeledTypePack`, `CompleteTypePack`, concatenation nodes, or similar
representations. Such nodes are canonical implementation forms of the Pack-based public
model.

Likewise, unordered field structure is publicly represented through
`UnorderedShape`, `FieldKey`, `FieldInfo`, and `UnorderedTail`; an internal
`RecordRow` representation does not create a source-visible `RecordFields` domain.

## 5. Generic/static parameter domains

Generic parameters may bind proper Types or static values.

Examples:

```telos
<T: Type>
<N: Int>
<K: FieldKey>
<P: PositionalTypePack>
<LP: LabeledTypePack>
<CP: CompleteTypePack>
```

In ordered-Pack/static-computation parameter positions, the expansion binders preserve
Pack lane semantics:

```telos
<*Ts>      // positional Pack capture
<**Ls>     // labeled Pack capture
<***Es>    // complete Pack capture
```

The same punctuation also appears inside structural forms whose owner determines the
structural domain. In particular, `Record<**Fs>` and Record field/tail syntax denote
unordered field structure, not an ordered labeled Pack. This contextual structural
meaning is owned by [Structural Products](structural-products.md).

There is therefore no global rule that every source occurrence of `**` denotes one
compiler kind. The semantic owner/form determines whether the contribution is an
ordered labeled Pack lane or unordered field structure.

Where explicit syntax constrains each entry value, the constraint applies to entry
values, not automatically to the entire captured shape.

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

Metadata uses stable semantic references/content encodings appropriate to each domain.

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

Associated/static member specifications must therefore preserve their general result
domain rather than collapsing every result to a kind-only form.

## 10. Invariants

| Invariant | Rule |
| --- | --- |
| **TL-PROPER-TYPE** | Proper runtime Types remain distinct from ordinary static values. |
| **TL-STATIC-TYPED** | Compile-time values have semantic Types/domains; they are not “kinds because they are compile time”. |
| **TL-PACK-PUBLIC** | Public type packs are Pack specializations, not an independent TypePack ontology. |
| **TL-ROW-PUBLIC** | Public unordered reflection uses the Product/UnorderedShape model, not RecordRow/RecordFields implementation names. |
| **TL-CANONICAL** | Canonical static/type identity is independent of source spelling and runtime object identity. |
| **TL-RESULT-DOMAIN** | Static/type definitions and members preserve explicit non-Type result domains. |
| **TL-SYMBOLIC-DOMAIN** | Residual symbolic terms retain their declared result domain. |

## 11. Boundaries

This specification does not prescribe TypeStore node layout, interning tables, solver
algorithms, metadata pool layout, or VM representation.

Those implementation choices may retain optimized historical TypePack/RecordRow nodes
provided all source-visible semantics conform to the Pack/Product model.

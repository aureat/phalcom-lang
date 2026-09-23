# Phalcom Type Algebra Registry — `type.data`

**Status:** Normative specification  
**Registry families:** `type.data`, `type.type-data`  
**Identifier range used here:** `T0850–T0875`

## Purpose

This file defines the semantic laws of Phalcom `data` and `type data` declarations.

`data` introduces runtime value domains with nominal type identity and one or more constructors/variants. `type data` introduces proper types whose inhabitants exist in the static-value universe and therefore define user-authored kind domains. A `type data` case is a static value whose kind is its enclosing `type data` specialization; it is not itself automatically a proper type.

This specification focuses on constructor identity, variant membership, closed-domain decomposition, constructor disjointness, and generic specialization consistency.

## Domain and notation

Conceptually:

```phalcom
data Option<T> {
    None
    Some(T)
}

type data Direction {
    North
    East
    South
    West
}
```

Notation used below:

- `D::V` denotes a variant/constructor associated with data domain `D`.
- For runtime `data`, `caseType(D::V)` denotes the variant type corresponding to a runtime constructor case.
- For `type data`, an applied/nullary case is a static value of kind `D`; `exact(D::V)` denotes its singleton proper type when needed.
- `closed(D)` means the domain is statically closed.
- `variants(D)` denotes the declared case set.

## Laws

```phalcom
@id(T0850)
@name(type.data.kind)
DataTypeKind<D> {
    kind(D) == Type
}

@id(T0851)
@name(type.data.nominal.identity)
DataNominalIdentity<D1, D2> {
    if decl(D1) != decl(D2) {
        D1 != D2
    }
}

@id(T0852)
@name(type.data.variant.member)
DataVariantSubtype<D, V> {
    if V in variants(D) {
        caseType(D::V) <: D
    }
}

@id(T0853)
@name(type.data.variant.distinct)
DistinctDataVariants<D, V1, V2> {
    if V1 != V2 {
        caseType(D::V1) != caseType(D::V2)
    }
}

@id(T0854)
@name(type.data.variant.disjoint)
DistinctDataVariantsDisjoint<D, V1, V2> {
    if V1 != V2 {
        caseType(D::V1) & caseType(D::V2) == Never
    }
}

@id(T0855)
@name(type.data.closed)
DataDomainClosed<D> {
    if declarationClosed(D) {
        closed(D)
    }
}

@id(T0856)
@name(type.data.closed.decomposition)
ClosedDataDecomposition<D> {
    if closed(D) {
        D == union(caseTypes(variants(D)))
    }
}

@id(T0857)
@name(type.data.closed.exhaustive)
ClosedDataCasesExhaustive<D, v> {
    if closed(D) and v : D {
        exists V in variants(D) where v : caseType(D::V)
    }
}

@id(T0858)
@name(type.data.constructor.result)
DataConstructorResult<D, V, ***As> {
    if constructor(D::V, ***As) {
        D::V(***As) : D
    }
}

@id(T0859)
@name(type.data.constructor.case-membership)
DataConstructorCaseMembership<D, V, ***As> {
    if constructor(D::V, ***As) {
        D::V(***As) : caseType(D::V)
    }
}

@id(T0860)
@name(type.data.constructor.tag-stable)
DataConstructorTagStable<D, V, ***As> {
    tag(D::V(***As)) == V
}

@id(T0861)
@name(type.data.generic.specialization)
GenericDataSpecialization<D, A, B> {
    if not staticEquivalent(A, B) {
        D<A> != D<B>
    }
}

@id(T0862)
@name(type.data.generic.variant-specialization)
GenericVariantSpecialization<D, V, A, B> {
    if not staticEquivalent(A, B) {
        caseType(D<A>::V) != caseType(D<B>::V)
    }
}

@id(T0863)
@name(type.data.generic.constructor-result)
GenericDataConstructorResult<D, V, A, x> {
    if x : A and constructor(D<A>::V, x) {
        D<A>::V(x) : D<A>
    }
}

@id(T0864)
@name(type.type-data.kind)
TypeDataKind<D> {
    kind(D) == Type
    kindDomain(D)
}

@id(T0865)
@name(type.type-data.closed-constructors)
TypeDataConstructorFamilyClosed<D> {
    closedConstructorFamily(D)
}

@id(T0866)
@name(type.type-data.variant.kind)
TypeDataVariantKind<D, V> {
    if V in nullaryVariants(D) {
        staticValue(D::V)
        kind(D::V) == D
    }
}

@id(T0867)
@name(type.type-data.variant.exact-subtype)
TypeDataVariantExactSubtype<D, V> {
    if V in nullaryVariants(D) {
        exact(D::V) <: D
    }
}

@id(T0868)
@name(type.type-data.variant.distinct)
TypeDataVariantsDistinct<D, V1, V2> {
    if V1 != V2 {
        not staticEquivalent(D::V1, D::V2)
    }
}

@id(T0869)
@name(type.type-data.constructor.kind)
TypeDataConstructorKind<D, V, K> {
    if constructorInputKind(D::V) == K {
        kind(D::V) == K -> D
    }
}

@id(T0870)
@name(type.type-data.constructor.application-kind)
TypeDataConstructorApplicationKind<D, V, a, K> {
    if kind(D::V) == K -> D and kind(a) == K {
        staticValue(D::V<a>)
        kind(D::V<a>) == D
    }
}

@id(T0871)
@name(type.type-data.constructor.application-identity)
TypeDataConstructorApplicationIdentity<D, V, a, b> {
    if staticEquivalent(a, b) {
        staticEquivalent(D::V<a>, D::V<b>)
    }
}

@id(T0872)
@name(type.type-data.constructor.injective)
TypeDataConstructorInjective<D, V, a, b> {
    if injective(D::V)
        and staticEquivalent(D::V<a>, D::V<b>)
    {
        staticEquivalent(a, b)
    }
}

@id(T0873)
@name(type.type-data.domain.exhaustive)
TypeDataDomainExhaustive<D, x> {
    if kind(x) == D {
        exists V in constructors(D) where constructedBy(x, D::V)
    }
}

@id(T0874)
@name(type.type-data.finite-nullary.decomposition)
FiniteNullaryTypeDataDecomposition<D> {
    if finiteNullaryStaticDomain(D) {
        D == union(exactValues(nullaryVariants(D)))
    }
}

@id(T0875)
@name(type.data.runtime-type-data.static-boundary)
RuntimeDataTypeDataBoundary<D, TD> {
    if runtimeDomain(D) and staticOnlyDomain(TD) {
        runtimeRepresentable(D)
        not runtimeRepresentable(TD)
    }
}
```

## Law notes

### T0850 / T0851 — Data identity

A `data` declaration introduces a proper nominal type with declaration-based identity.

**Kind:** Semantic laws  
**Depends on:** T0750–T0754

### T0852–T0854 — Variant relations

Each variant belongs to its parent domain, distinct variants remain distinct, and distinct cases are disjoint.

**Kind:** Semantic data laws  
**Depends on:** T0100–T0104, T0515

### T0855–T0857 — Closed data domains

A closed data domain is exhausted by its declared cases.

**Kind:** Semantic finite-domain laws  
**Depends on:** T0450–T0465

### T0858–T0860 — Runtime constructors

Applying a runtime data constructor produces a value in both the constructor case and the enclosing data type, with a stable variant tag.

**Kind:** Semantic constructor laws  
**Depends on:** T0852

### T0861–T0863 — Generic data specialization

Generic data specializations preserve generic argument identity and constructor result specialization.

**Kind:** Cross-family semantic laws  
**Depends on:** T0800–T0824

### T0864–T0868 — `type data` kind domains

A `type data` declaration produces a proper type `D` that is also a kind domain for its static inhabitants. Nullary cases such as `Direction::North` are static values with `kind(Direction::North) == Direction`; they are not ordinary proper types merely because they occur at type level. Their singleton type is `exact(Direction::North)`.

This makes the model uniform with literals: a static `20` may have kind `Int`, while `Bound<Int>::Included<20>` may have kind `Bound<Int>`.

**Kind:** Semantic static-domain laws  
**Depends on:** `type.kind`, `type.static-value`, and `type.exact`

### T0869–T0872 — Parameterized static constructors

A parameterized `type data` constructor is a kind-level function. If `D::V` accepts an argument of kind `K`, then `kind(D::V) == K -> D`; applying it to a static argument of kind `K` yields a static value of kind `D`. Constructor injectivity is explicit rather than universal.

**Kind:** Static constructor laws  
**Depends on:** `type.kind`, `type.static-value`, and constructor identity laws

### T0873 — `TypeDataDomainExhaustive`

`type data` is closed by constructor family: every static inhabitant of `D` originates from one of its declared constructors. This does **not** imply that the domain is finite; constructors with payloads may induce infinitely many static values.

**Kind:** Static-domain closure law  
**Depends on:** T0865–T0872

### T0874 — `FiniteNullaryTypeDataDecomposition`

Only a finite nullary `type data` domain decomposes directly into a finite union of exact singleton types. Parameterized domains such as `Bound<Int>` are closed by constructors but are not assumed finite.

**Kind:** Cross-family finite-domain law  
**Depends on:** T0864–T0873 and `type.union`

### T0875 — `RuntimeDataTypeDataBoundary`

A static-only `type data` domain need not have runtime representation. The boundary is representation availability, not semantic kind validity.

**Kind:** Semantic/runtime boundary law  
**Depends on:** T0850, T0864

## Explicitly deferred laws

- GADT result refinement
- existential constructor arguments
- constructor pattern refinement
- recursive data domains
- representation/layout
- singleton-elision syntax
- whether an individual static case may be used directly as a singleton kind without `exact(...)`
- direct constructor-as-type shorthand
- reflection over constructor metadata
- runtime/static bridging rules

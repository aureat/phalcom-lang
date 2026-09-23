# Phalcom Type Algebra Registry — `type.data`

**Status:** Normative specification  
**Registry families:** `type.data`, `type.type-data`  
**Identifier range used here:** `T0280–T0304`

## Purpose

This file defines the semantic laws of Phalcom `data` and `type data` declarations.

`data` introduces runtime value domains with nominal type identity and one or more constructors/variants. `type data` introduces closed type-level/static domains. Both forms may define finite or parameterized constructor spaces.

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
- `caseType(D::V)` denotes the exact or variant type corresponding to a constructor case.
- `closed(D)` means the domain is statically closed.
- `variants(D)` denotes the declared case set.

## Laws

```phalcom
@id(T0280)
@name(type.data.kind)
DataTypeKind<D> {
    kind(D) == Type
}

@id(T0281)
@name(type.data.nominal.identity)
DataNominalIdentity<D1, D2> {
    if decl(D1) != decl(D2) {
        D1 != D2
    }
}

@id(T0282)
@name(type.data.variant.member)
DataVariantSubtype<D, V> {
    if V in variants(D) {
        caseType(D::V) <: D
    }
}

@id(T0283)
@name(type.data.variant.distinct)
DistinctDataVariants<D, V1, V2> {
    if V1 != V2 {
        caseType(D::V1) != caseType(D::V2)
    }
}

@id(T0284)
@name(type.data.variant.disjoint)
DistinctDataVariantsDisjoint<D, V1, V2> {
    if V1 != V2 {
        caseType(D::V1) & caseType(D::V2) == Never
    }
}

@id(T0285)
@name(type.data.closed)
DataDomainClosed<D> {
    if declarationClosed(D) {
        closed(D)
    }
}

@id(T0286)
@name(type.data.closed.decomposition)
ClosedDataDecomposition<D> {
    if closed(D) {
        D == union(caseTypes(variants(D)))
    }
}

@id(T0287)
@name(type.data.closed.exhaustive)
ClosedDataCasesExhaustive<D, v> {
    if closed(D) and v : D {
        exists V in variants(D) where v : caseType(D::V)
    }
}

@id(T0288)
@name(type.data.constructor.result)
DataConstructorResult<D, V, ***As> {
    if constructor(D::V, ***As) {
        D::V(***As) : D
    }
}

@id(T0289)
@name(type.data.constructor.case-membership)
DataConstructorCaseMembership<D, V, ***As> {
    if constructor(D::V, ***As) {
        D::V(***As) : caseType(D::V)
    }
}

@id(T0290)
@name(type.data.constructor.tag-stable)
DataConstructorTagStable<D, V, ***As> {
    tag(D::V(***As)) == V
}

@id(T0291)
@name(type.data.generic.specialization)
GenericDataSpecialization<D, A, B> {
    if A != B {
        D<A> != D<B>
    }
}

@id(T0292)
@name(type.data.generic.variant-specialization)
GenericVariantSpecialization<D, V, A, B> {
    if A != B {
        caseType(D<A>::V) != caseType(D<B>::V)
    }
}

@id(T0293)
@name(type.data.generic.constructor-result)
GenericDataConstructorResult<D, V, A, x> {
    if x : A and constructor(D<A>::V, x) {
        D<A>::V(x) : D<A>
    }
}

@id(T0294)
@name(type.type-data.kind)
TypeDataKind<D> {
    kind(D) == Type
}

@id(T0295)
@name(type.type-data.closed)
TypeDataClosed<D> {
    closed(D)
}

@id(T0296)
@name(type.type-data.variant.type)
TypeDataVariantIsType<D, V> {
    if V in variants(D) {
        kind(D::V) == Type
    }
}

@id(T0297)
@name(type.type-data.variant.subtype)
TypeDataVariantSubtype<D, V> {
    if V in variants(D) {
        D::V <: D
    }
}

@id(T0298)
@name(type.type-data.variant.disjoint)
TypeDataVariantsDisjoint<D, V1, V2> {
    if V1 != V2 {
        D::V1 & D::V2 == Never
    }
}

@id(T0299)
@name(type.type-data.decomposition)
TypeDataDecomposition<D> {
    D == union(variants(D))
}

@id(T0300)
@name(type.type-data.constructor.application-kind)
TypeDataConstructorApplicationKind<F, A> {
    if kind(F) == Type -> Type and kind(A) == Type {
        kind(F<A>) == Type
    }
}

@id(T0301)
@name(type.type-data.constructor.injective)
TypeDataConstructorInjective<F, A, B> {
    if injective(F) and F<A> == F<B> {
        A == B
    }
}

@id(T0302)
@name(type.type-data.exact-argument.identity)
TypeDataExactArgumentIdentity<F, a, b> {
    if a == b {
        F<exact(a)> == F<exact(b)>
    }
}

@id(T0303)
@name(type.type-data.closed.matchable)
TypeDataClosedMatchable<D> {
    if closed(D) {
        finiteCaseSpace(D) == variants(D)
    }
}

@id(T0304)
@name(type.data.runtime-type-data.static-boundary)
RuntimeDataTypeDataBoundary<D, TD> {
    if runtimeDomain(D) and staticDomain(TD) {
        D != TD
    }
}
```

## Law notes

### T0280 / T0281 — Data identity

A `data` declaration introduces a proper nominal type with declaration-based identity.

**Kind:** Semantic laws  
**Depends on:** T0230–T0234

### T0282–T0284 — Variant relations

Each variant belongs to its parent domain, distinct variants remain distinct, and distinct cases are disjoint.

**Kind:** Semantic data laws  
**Depends on:** T0020–T0024, T0135

### T0285–T0287 — Closed data domains

A closed data domain is exhausted by its declared cases.

**Kind:** Semantic finite-domain laws  
**Depends on:** T0100–T0115

### T0288–T0290 — Runtime constructors

Applying a runtime data constructor produces a value in both the constructor case and the enclosing data type, with a stable variant tag.

**Kind:** Semantic constructor laws  
**Depends on:** T0282

### T0291–T0293 — Generic data specialization

Generic data specializations preserve generic argument identity and constructor result specialization.

**Kind:** Cross-family semantic laws  
**Depends on:** T0250–T0274

### T0294–T0299 — `type data`

`type data` forms a closed static type domain whose variants are themselves proper types and whose parent is exactly the union of those variants.

**Kind:** Semantic static-domain laws  
**Depends on:** T0100–T0155

### T0300–T0302 — Parameterized static constructors

Type-level constructors obey ordinary kinding, declared injectivity, and exact-argument identity.

**Kind:** Cross-family semantic laws  
**Depends on:** T0012–T0015, T0262, T0065–T0067

### T0303 — `TypeDataClosedMatchable`

Closed static domains expose finite case spaces suitable for compile-time exhaustiveness analysis.

**Kind:** Cross-family semantic law  
**Depends on:** T0295–T0299

### T0304 — `RuntimeDataTypeDataBoundary`

Runtime `data` and static `type data` domains remain semantically distinct unless an explicit language rule unifies a specific declaration form.

**Kind:** Semantic boundary law  
**Depends on:** T0280, T0294

## Explicitly deferred laws

- GADT result refinement
- existential constructor arguments
- constructor pattern refinement
- recursive data domains
- representation/layout
- singleton-elision syntax
- direct constructor-as-type shorthand
- reflection over constructor metadata
- runtime/static bridging rules

# Phalcom Type Algebra Registry — `type.tuple`

**Status:** Normative specification
**Registry family:** `type.tuple`
**Identifier range used here:** `T0650–T0672`

## Purpose

This file defines the semantic laws of Tuple types. It is the formal-law
registry for the ordered structural-product semantics owned by
[`structural-products.md`](../../structural-products.md).

Tuples are ordered product types. Arity, positional order, `Symbol` labels where
present, and entry types are semantically significant. Fixed positional and
labeled entries are always required. The positional and labeled lanes are
independently closed or opened by their respective remainders.

This file does not define Record semantics. Record fields are unordered and use
`FieldKey`; Tuple entries are ordered and use positional indices or `Symbol`
labels.

## Domain and notation

Unless stated otherwise:

- `(A, B)` denotes a two-entry Tuple type.
- `(A,)` denotes a one-entry Tuple type.
- `Tuple<> ≡ Unit ≡ ()` canonicalizes the zero-entry Tuple to the singleton
  Unit type and value context.
- `tuple<***Es>` denotes a Tuple type conceptually.
- `T[I]` denotes compile-time Tuple type indexing.
- `T::At<I>` denotes the associated type-level projection at index `I`.
- labels such as `(x: A, y: B)` are ordered `Symbol` labels.
- `N` is the fixed positional arity when normalizing an index.

## Laws

```phalcom
@id(T0650)
@name(type.tuple.kind)
TupleKind<***Es> {
    kind(tuple<***Es>) == Type
}

@id(T0651)
@name(type.tuple.zero.unit)
ZeroTupleIsUnit {
    Tuple<> == Unit
    Tuple<> == ()
}

@id(T0652)
@name(type.tuple.arity.significant)
TupleAritySignificant<A, B> {
    (A,) != (A, B)
}

@id(T0653)
@name(type.tuple.singleton.distinct-from-entry)
SingletonTupleDistinctFromEntry<A> {
    (A,) != A
}

@id(T0654)
@name(type.tuple.position.order-significant)
TuplePositionOrderSignificant<A, B> {
    if A != B {
        (A, B) != (B, A)
    }
}

@id(T0655)
@name(type.tuple.entry.identity-substitution)
TupleEntryIdentitySubstitution<A, B, C> {
    if A == B {
        (A, C) == (B, C)
    }
}

@id(T0656)
@name(type.tuple.covariant)
TupleCovariant<A1, A2, B1, B2> {
    if A1 <: B1 and A2 <: B2 {
        (A1, A2) <: (B1, B2)
    }
}

@id(T0657)
@name(type.tuple.closed-fixed-subtype.requires-equal-arity)
ClosedFixedTupleSubtypeRequiresEqualArity<***As, ***Bs> {
    if closedFixedTuple(tuple<***As>)
        and closedFixedTuple(tuple<***Bs>)
        and tuple<***As> <: tuple<***Bs>
    {
        arity(***As) == arity(***Bs)
    }
}

@id(T0658)
@name(type.tuple.closed-fixed-subtype.pointwise)
ClosedFixedTupleSubtypePointwise<***As, ***Bs> {
    if closedFixedTuple(tuple<***As>)
        and closedFixedTuple(tuple<***Bs>)
        and arity(***As) == arity(***Bs)
        and pointwiseSubtype(***As, ***Bs)
    {
        tuple<***As> <: tuple<***Bs>
    }
}

@id(T0659)
@name(type.tuple.at.zero)
TupleAtZero<A, B> {
    (A, B)::At<0> == A
}

@id(T0660)
@name(type.tuple.at.one)
TupleAtOne<A, B> {
    (A, B)::At<1> == B
}

@id(T0661)
@name(type.tuple.index.projection-equivalence)
TupleIndexProjectionEquivalent<***Es, I> {
    tuple<***Es>[I] == tuple<***Es>::At<I>
}

@id(T0662)
@name(type.tuple.projection.kind)
TupleProjectionKind<***Es, I> {
    if validNormalizedIndex(I, arity(***Es)) {
        kind(tuple<***Es>::At<I>) == Type
    }
}

@id(T0663)
@name(type.tuple.projection.end-relative-indexing)
TupleProjectionUsesEndRelativeIndexing<***Es, I> {
    N == arity(***Es)

    normalizedIndex(I, N) ==
        if I >= 0 { I } else { N + I }

    if normalizedIndex(I, N) < 0
        or normalizedIndex(I, N) >= N
    {
        illFormed(tuple<***Es>::At<I>)
    }
}

@id(T0664)
@name(type.tuple.label.identity)
TupleLabelsSignificant<A, B> {
    (x: A, y: B) != (a: A, b: B)
}

@id(T0665)
@name(type.tuple.label.position-significant)
TupleLabelPositionSignificant<A, B> {
    (x: A, y: B) != (y: B, x: A)
}

@id(T0666)
@name(type.tuple.labeled-entry.covariant)
LabeledTupleCovariant<A1, A2, B1, B2> {
    if A1 <: B1 and A2 <: B2 {
        (x: A1, y: A2) <: (x: B1, y: B2)
    }
}

@id(T0667)
@name(type.tuple.union.pointwise-widening)
TuplePointwiseUnionWidening<A1, A2, B1, B2> {
    (A1, A2) <: (A1 | B1, A2 | B2)
}

@id(T0668)
@name(type.tuple.never.entry)
TupleWithNeverEntryIsNever<A> {
    (A, Never) == Never
}

@id(T0669)
@name(type.tuple.closed-target.rejects-surplus)
ClosedTupleTargetRejectsSurplusEntries<A, B> {
    (A, B) </: (A,)
    (x: A, y: B) </: (x: A)
}

@id(T0670)
@name(type.tuple.positional-remainder.subtype)
PositionalTupleRemainderSubtype<A, B, T> {
    (A, B) <: (A, *: ...)

    if B <: T {
        (A, B) <: (A, *: T)
    }

    (A, *: ...) </: (A, B, *: ...)
}

@id(T0671)
@name(type.tuple.labeled-remainder.subtype)
LabeledTupleRemainderSubtype<A, B, T> {
    (x: A, y: B) <: (x: A, **: ...)

    if B <: T {
        (x: A, y: B) <: (x: A, **: T)
    }

    (x: A, **: ...) </: (x: A, y: B, **: ...)
}

@id(T0672)
@name(type.tuple.complete-remainder.lane-wise)
CompleteTupleRemainderIsLaneWise<A, B, T> {
    (A, x: B) <: (***: ...)

    if A <: T and B <: T {
        (A, x: B) <: (***: T)
    }
}
```

## Law notes

### T0650 — `TupleKind`

Every well-formed Tuple type is a proper type.

**Kind:** Semantic law
**Depends on:** T0060

### T0651 — `ZeroTupleIsUnit`

The zero-entry Tuple is not a separate structural type. `Tuple<>`, `Unit`, and
`()` are one canonical singleton.

**Kind:** Semantic identity law
**Depends on:** T0001–T0003, T0562

### T0652–T0655 — Fixed-entry identity

Arity, positional order, and entry identity are part of Tuple identity. A
one-entry Tuple remains distinct from its entry type.

**Kind:** Semantic identity laws
**Depends on:** T0001–T0003

### T0656–T0658 — Closed fixed Tuple subtyping

Immutable fixed entries are covariant. Equal arity and pointwise subtyping are
the complete rule only when both Tuple operands are closed and have only fixed
entries. These laws do not apply to a target that carries `*:`, `**:`,
`***:`, or `...`.

**Kind:** Semantic subtype laws
**Depends on:** T0100–T0103, T0652

### T0659–T0663 — Projections and indices

Tuple indexing is zero-based for non-negative indices and end-relative for
negative indices:

```text
I >= 0 -> I
I < 0  -> N + I
```

Projection is rejected only when that normalized index lies outside `[0, N)`.
It must not produce `Never`, `Any`, `Unknown`, or an unresolved ordinary type
merely to keep solving.

**Kind:** Projection and well-formedness laws
**Depends on:** T0050, T0055, T0650–T0652

### T0664–T0666 — Labeled Tuple entries

Labeled Tuple entries retain ordered `Symbol` labels. The label and its
position participate in identity, and matching fixed labeled entries are
covariant.

**Kind:** Semantic identity and variance laws
**Depends on:** T0654, T0656

### T0667–T0668 — Derived entry laws

Pointwise union widening follows covariance. A required entry of type `Never`
makes the Tuple uninhabited.

**Kind:** Cross-family derived laws
**Depends on:** T0152–T0154, T0456, T0457, T0656

### T0669–T0672 — Open and remainder Tuple subtyping

Tuple subtyping is lane-wise. For each lane, fixed target entries must be
present as fixed source entries and must be compatible. A source remainder
cannot establish a required fixed target entry because it may be empty.

A closed target lane rejects surplus fixed source entries. A target `*: ...`
or `**: ...` admits them without a common entry-type constraint; a typed target
remainder admits them only when their types are subtypes of the target
constraint. An open source lane requires an open target lane. For source and
target typed remainders, `U <: T` is required; a typed source remainder is
accepted by an unconstrained target remainder; an unconstrained source
remainder is not accepted by a typed target remainder. `(***: T)` applies the
same rule independently to both lanes, and `(...)` is its unconstrained form.

**Kind:** Semantic subtype laws
**Depends on:** T0656–T0658

## Required implementation invariants

Tuple semantics must remain ordered even if Tuple entries reuse underlying row
machinery. Shared storage does not imply unordered Record behavior.

Tuple canonicalization must preserve:

- zero-entry canonicalization to `Unit`;
- fixed-entry arity and positional order;
- ordered `Symbol` labels;
- required fixed-entry presence;
- per-lane closedness and remainder constraints; and
- normalized static projection indices.

## Explicitly deferred laws

- Tuple concatenation/spread algebra
- Tuple pack mapping such as `Tuple::Map`
- labels that are diagnostic-only rather than semantic
- mutable Tuple slots, if ever introduced
- runtime indexing versus static type indexing

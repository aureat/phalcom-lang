# Phalcom Type Algebra Registry — `type.tuple`

**Status:** Normative specification  
**Registry family:** `type.tuple`  
**Identifier range used here:** `T0180–T0198`

## Purpose

This file defines the semantic laws of tuple types.

Tuples are ordered product types. Arity, positional order, labels where present, and element types are semantically significant. Tuple subtyping is covariant element-wise for immutable tuple structure. Projection laws connect tuple shape to compile-time indexing and associated-type access.

This file does not define record semantics. Record fields may be order-insensitive; tuple entries are not.

## Domain and notation

Unless stated otherwise:

- `(A, B)` denotes a two-element tuple type.
- `(A,)` denotes a one-element tuple type.
- `()` remains the unit value/type context and is not a zero-element tuple unless Phalcom explicitly unifies those concepts elsewhere.
- `tuple<***Es>` denotes a variadic tuple type conceptually.
- `T[I]` denotes compile-time tuple type indexing.
- `T::At<I>` denotes the associated type-level projection at index `I`.
- labels such as `(x: A, y: B)` are part of tuple entry structure when present.

## Laws

```phalcom
@id(T0180)
@name(type.tuple.kind)
TupleKind<***Es> {
    kind(tuple<***Es>) == Type
}

@id(T0181)
@name(type.tuple.identity)
TupleIdentity<***Es> {
    tuple<***Es> == tuple<***Es>
}

@id(T0182)
@name(type.tuple.arity.significant)
TupleAritySignificant<A, B> {
    (A,) != (A, B)
}

@id(T0183)
@name(type.tuple.singleton.distinct-from-element)
SingletonTupleDistinctFromElement<A> {
    (A,) != A
}

@id(T0184)
@name(type.tuple.position.order-significant)
TuplePositionOrderSignificant<A, B> {
    if A != B {
        (A, B) != (B, A)
    }
}

@id(T0185)
@name(type.tuple.element.identity-substitution)
TupleElementIdentitySubstitution<A, B, C> {
    if A == B {
        (A, C) == (B, C)
    }
}

@id(T0186)
@name(type.tuple.covariant)
TupleCovariant<A1, A2, B1, B2> {
    if A1 <: B1 and A2 <: B2 {
        (A1, A2) <: (B1, B2)
    }
}

@id(T0187)
@name(type.tuple.subtype.requires-equal-arity)
TupleSubtypeRequiresEqualArity<***As, ***Bs> {
    if tuple<***As> <: tuple<***Bs> {
        arity(***As) == arity(***Bs)
    }
}

@id(T0188)
@name(type.tuple.subtype.pointwise)
TupleSubtypePointwise<***As, ***Bs> {
    if arity(***As) == arity(***Bs)
        and pointwiseSubtype(***As, ***Bs)
    {
        tuple<***As> <: tuple<***Bs>
    }
}

@id(T0189)
@name(type.tuple.at.zero)
TupleAtZero<A, B> {
    (A, B)::At<0> == A
}

@id(T0190)
@name(type.tuple.at.one)
TupleAtOne<A, B> {
    (A, B)::At<1> == B
}

@id(T0191)
@name(type.tuple.index.projection-equivalence)
TupleIndexProjectionEquivalent<***Es, I> {
    tuple<***Es>[I] == tuple<***Es>::At<I>
}

@id(T0192)
@name(type.tuple.projection.kind)
TupleProjectionKind<***Es, I> {
    kind(tuple<***Es>::At<I>) == Type
}

@id(T0193)
@name(type.tuple.projection.bounds)
TupleProjectionRequiresValidIndex<***Es, I> {
    if I < 0 or I >= arity(***Es) {
        illFormed(tuple<***Es>::At<I>)
    }
}

@id(T0194)
@name(type.tuple.label.identity)
TupleLabelsSignificant<A, B> {
    (x: A, y: B) != (a: A, b: B)
}

@id(T0195)
@name(type.tuple.label.position-significant)
TupleLabelPositionSignificant<A, B> {
    (x: A, y: B) != (y: B, x: A)
}

@id(T0196)
@name(type.tuple.named-entry.covariant)
NamedTupleCovariant<A1, A2, B1, B2> {
    if A1 <: B1 and A2 <: B2 {
        (x: A1, y: A2) <: (x: B1, y: B2)
    }
}

@id(T0197)
@name(type.tuple.union.pointwise-widening)
TuplePointwiseUnionWidening<A1, A2, B1, B2> {
    (A1, A2) <: (A1 | B1, A2 | B2)
}

@id(T0198)
@name(type.tuple.never.element)
TupleWithNeverElementIsNever<A> {
    (A, Never) == Never
}
```

## Law notes

### T0180 — `TupleKind`

Every well-formed tuple type is a proper type.

**Kind:** Semantic law  
**Depends on:** T0018

### T0181 — `TupleIdentity`

Tuple types participate in ordinary semantic identity.

**Kind:** Semantic law  
**Depends on:** T0001

### T0182 — `TupleAritySignificant`

Tuple arity is part of semantic identity.

**Kind:** Semantic identity law  
**Depends on:** T0001–T0003

### T0183 — `SingletonTupleDistinctFromElement`

A one-element tuple is not semantically identical to its element type.

**Kind:** Semantic identity law  
**Depends on:** T0182

### T0184 — `TuplePositionOrderSignificant`

Tuple positions are ordered.

When the element types differ, swapping positions changes the tuple type.

**Kind:** Semantic identity law  
**Depends on:** T0182

### T0185 — `TupleElementIdentitySubstitution`

Replacing an element type with a semantically identical type preserves tuple identity.

**Kind:** Derived coherence law  
**Depends on:** T0001–T0003, T0181

### T0186 — `TupleCovariant`

Immutable tuple elements are covariant.

**Kind:** Semantic variance law  
**Depends on:** T0020–T0023

### T0187 — `TupleSubtypeRequiresEqualArity`

Tuple subtyping does not erase or invent positions. A tuple may only subtype another tuple of the same arity unless a future explicit variadic/open-tuple feature says otherwise.

**Kind:** Semantic subtype law  
**Depends on:** T0182

### T0188 — `TupleSubtypePointwise`

For equal-arity immutable tuples, pointwise element subtyping implies tuple subtyping.

**Kind:** Semantic subtype law  
**Depends on:** T0186, T0187

### T0189 / T0190 — Basic projections

Tuple projection by valid static index yields the element type at that position.

**Kind:** Semantic projection laws  
**Depends on:** T0180–T0182

### T0191 — `TupleIndexProjectionEquivalent`

Compile-time indexing and the associated projection `::At<I>` are two syntactic routes to the same semantic type projection.

**Kind:** Cross-syntax semantic law  
**Depends on:** T0189, T0190

### T0192 — `TupleProjectionKind`

A valid tuple element projection produces a proper type.

**Kind:** Kinding law  
**Depends on:** T0010, T0180

### T0193 — `TupleProjectionRequiresValidIndex`

Out-of-bounds tuple type projection is ill formed.

It must not produce `Never`, `Any`, `Unknown`, or an unresolved ordinary type merely to keep solving.

**Kind:** Well-formedness invariant  
**Depends on:** T0013, T0182

### T0194 — `TupleLabelsSignificant`

When tuple entries are labeled, labels participate in semantic identity.

This law assumes the ratified model where named tuple entries expose semantic labels rather than diagnostic-only source names.

**Kind:** Semantic identity law  
**Depends on:** T0181

### T0195 — `TupleLabelPositionSignificant`

Labels do not make tuples unordered. Position remains semantically significant.

**Kind:** Semantic identity law  
**Depends on:** T0184, T0194

### T0196 — `NamedTupleCovariant`

Named tuple elements are covariant when labels and arity agree.

**Kind:** Semantic variance law  
**Depends on:** T0186, T0194

### T0197 — `TuplePointwiseUnionWidening`

A tuple is a subtype of a tuple whose corresponding element types have been widened by union.

**Kind:** Cross-family derived law  
**Depends on:** T0106, T0107, T0186

### T0198 — `TupleWithNeverElementIsNever`

A strict product type with an uninhabited required element is itself uninhabited.

This law assumes every tuple position must be populated by a value. Because `Never` has no value, no value can inhabit `(A, Never)`.

**Kind:** Cross-family semantic law  
**Depends on:** T0032–T0034, T0180

## Required implementation invariants

Tuple semantics must remain ordered even if tuple entries reuse underlying row machinery.

Shared row storage does not imply record-style field-order irrelevance.

Tuple canonicalization must preserve:

- arity,
- positional order,
- entry labels where semantically present,
- element types,
- and valid static projection indices.

## Explicitly deferred laws

- open or variadic-tail tuple subtyping
- tuple concatenation/spread algebra
- tuple pack mapping such as `Tuple::Map`
- labels that are diagnostic-only rather than semantic
- mutable tuple slots, if ever introduced
- runtime indexing versus static type indexing

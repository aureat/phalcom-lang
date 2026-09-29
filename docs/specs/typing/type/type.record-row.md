# Phalcom Type Algebra Registry — `type.record-row`

**Status:** Normative specification  
**Registry families:** `type.record`, `type.row`  
**Identifier range used here:** `T0700–T0721`

## Purpose

This file defines the semantic laws of record types and their underlying row structure.

Records are unordered field products. Field names are semantic keys, field order is not semantic, and immutable record fields support width and depth subtyping. Mutable fields are invariant in their value type unless a more specific capability discipline later says otherwise.

Rows are the structural substrate shared by records and potentially other structured types, but sharing row machinery does not force identical surface semantics. In particular, tuple order remains significant under `type.tuple`.

## Domain and notation

Unless stated otherwise:

- `{ x: A, y: B }` denotes an immutable record type.
- `mutable { x: A }` denotes a record with a mutable field.
- `row { x: A, y: B }` denotes the corresponding structural row conceptually.
- `fields(R)` denotes the field-label set of a record or row.
- `R::At<"x">` denotes static field-type projection by label.
- `<:` means semantic subtype.
- `==` means semantic type identity.

## Laws

```phalcom
@id(T0700)
@name(type.record.kind)
RecordKind<***Fields> {
    kind(record<***Fields>) == Type
}

@id(T0701)
@name(type.row.field-names.unique)
RowFieldNamesUnique<R> {
    unique(fields(R))
}

@id(T0702)
@name(type.record.field-order.irrelevant)
RecordFieldOrderIrrelevant<A, B> {
    { x: A, y: B } == { y: B, x: A }
}

@id(T0703)
@name(type.record.field-name.significant)
RecordFieldNamesSignificant<A> {
    { x: A } != { y: A }
}

@id(T0704)
@name(type.record.identity-substitution)
RecordFieldIdentitySubstitution<A, B, C> {
    if A == B {
        { x: A, y: C } == { x: B, y: C }
    }
}

@id(T0705)
@name(type.record.width-subtype)
RecordWidthSubtype<A, B> {
    { x: A, y: B } <: { x: A }
}

@id(T0706)
@name(type.record.depth-subtype)
RecordDepthSubtype<A, B> {
    if A <: B {
        { x: A } <: { x: B }
    }
}

@id(T0707)
@name(type.record.width-depth-subtype)
RecordWidthDepthSubtype<A1, A2, B1> {
    if A1 <: B1 {
        { x: A1, y: A2 } <: { x: B1 }
    }
}

@id(T0708)
@name(type.record.required-fields)
RecordSubtypeRequiresFields<R1, R2> {
    if R1 <: R2 {
        fields(R2) <= fields(R1)
    }
}

@id(T0709)
@name(type.record.field-subtype)
RecordSubtypeRequiresFieldCompatibility<R1, R2, N> {
    if R1 <: R2 and N in fields(R2) {
        R1::At<N> <: R2::At<N>
    }
}

@id(T0710)
@name(type.row.projection)
RowFieldProjection<R, N> {
    if N in fields(R) {
        kind(R::At<N>) == Type
    }
}

@id(T0711)
@name(type.row.projection.missing)
MissingRowFieldProjectionIllFormed<R, N> {
    if N not in fields(R) {
        illFormed(R::At<N>)
    }
}

@id(T0712)
@name(type.record.projection)
RecordFieldProjection<A, B> {
    { x: A, y: B }::At<"x"> == A
    { x: A, y: B }::At<"y"> == B
}

@id(T0713)
@name(type.record.mutable.invariant)
MutableRecordFieldInvariant<A, B> {
    mutable { x: A } <: mutable { x: B }
        iff A == B
}

@id(T0714)
@name(type.record.mutable.width)
MutableRecordWidthSubtype<A, B> {
    mutable { x: A, y: B } <: mutable { x: A }
}

@id(T0715)
@name(type.row.permutation.identity)
RowPermutationIdentity<A, B, C> {
    row { x: A, y: B, z: C } ==
    row { z: C, x: A, y: B }
}

@id(T0716)
@name(type.row.extension)
RowExtension<R, N, A> {
    if N not in fields(R) {
        fields(extend(R, N: A)) == fields(R) + { N }
    }
}

@id(T0717)
@name(type.row.extension.projection)
RowExtensionProjection<R, N, A> {
    if N not in fields(R) {
        extend(R, N: A)::At<N> == A
    }
}

@id(T0718)
@name(type.row.extension.preservation)
RowExtensionPreservesExistingFields<R, N, A, M> {
    if N not in fields(R) and M in fields(R) {
        extend(R, N: A)::At<M> == R::At<M>
    }
}

@id(T0719)
@name(type.record.tuple.distinct)
RecordTupleDistinct<A, B> {
    { x: A, y: B } != (x: A, y: B)
}

@id(T0720)
@name(type.record.never-field)
RecordWithNeverRequiredFieldIsNever<A> {
    { x: A, y: Never } == Never
}

@id(T0721)
@name(type.record.union.field-widening)
RecordFieldUnionWidening<A, B> {
    { x: A } <: { x: A | B }
}
```

## Law notes

### T0700 — `RecordKind`

Every well-formed record type is a proper type.

**Kind:** Semantic law  
**Depends on:** T0060

### T0701 — `RowFieldNamesUnique`

A row may not contain two semantic fields with the same label.

Duplicate-field source syntax must be rejected or resolved before a valid semantic row exists.

**Kind:** Well-formedness invariant  
**Depends on:** T0700

### T0702 — `RecordFieldOrderIrrelevant`

Record field ordering is not semantic.

Reordering fields preserves record identity.

**Kind:** Semantic identity law  
**Depends on:** T0001–T0003

### T0703 — `RecordFieldNamesSignificant`

Field labels are semantic keys. Structurally identical value types under different labels do not produce identical record types.

**Kind:** Semantic identity law  
**Depends on:** T0001–T0003

### T0704 — `RecordFieldIdentitySubstitution`

Replacing a field type with a semantically identical type preserves record identity.

**Kind:** Derived coherence law  
**Depends on:** T0001–T0003

### T0705 — `RecordWidthSubtype`

An immutable record with additional fields may be used where a subset of those fields is required.

**Kind:** Semantic subtype law  
**Depends on:** T0100–T0103

### T0706 — `RecordDepthSubtype`

Immutable record fields are covariant in their value type.

**Kind:** Semantic variance law  
**Depends on:** T0100–T0103

### T0707 — `RecordWidthDepthSubtype`

Width and depth subtyping compose.

A record may provide additional fields and narrower types for the fields required by the target.

**Kind:** Derived semantic law  
**Depends on:** T0705, T0706, T0101

### T0708 — `RecordSubtypeRequiresFields`

If `R1 <: R2`, the source record must provide every field required by the target record.

**Kind:** Semantic subtype invariant  
**Depends on:** T0705

### T0709 — `RecordSubtypeRequiresFieldCompatibility`

For immutable fields required by the target record, the corresponding source field type must subtype the target field type.

**Kind:** Semantic subtype invariant  
**Depends on:** T0706

### T0710 — `RowFieldProjection`

Projecting a present row field yields a proper type.

**Kind:** Kinding law  
**Depends on:** T0050, T0701

### T0711 — `MissingRowFieldProjectionIllFormed`

Static projection of a missing field is ill formed.

The compiler must not manufacture `Never`, `Any`, or `Unknown` as though the field existed.

**Kind:** Well-formedness invariant  
**Depends on:** T0055, T0701

### T0712 — `RecordFieldProjection`

Record field projection returns the type associated with the selected semantic field label.

**Kind:** Semantic projection law  
**Depends on:** T0710

### T0713 — `MutableRecordFieldInvariant`

A writable field is invariant in its value type.

Covariance would be unsound: code expecting `mutable { x: B }` could write any `B` into storage that only safely accepts a narrower `A`.

**Kind:** Semantic variance law  
**Depends on:** T0100–T0104

### T0714 — `MutableRecordWidthSubtype`

Field mutability does not by itself invalidate width subtyping when the retained field contracts are identical.

This law concerns dropping visibility of extra fields, not changing the writable type of retained fields.

**Kind:** Semantic subtype law  
**Depends on:** T0705, T0713

### T0715 — `RowPermutationIdentity`

Rows are unordered maps from semantic labels to field contracts.

**Kind:** Semantic row law  
**Depends on:** T0701, T0702

### T0716 — `RowExtension`

Extending a row with a fresh field adds exactly that semantic field.

**Kind:** Semantic row law  
**Depends on:** T0701

### T0717 — `RowExtensionProjection`

The newly introduced field projects to its declared type.

**Kind:** Semantic row law  
**Depends on:** T0710, T0716

### T0718 — `RowExtensionPreservesExistingFields`

Extending a row with a fresh field preserves every existing field projection.

**Kind:** Semantic row law  
**Depends on:** T0716, T0717

### T0719 — `RecordTupleDistinct`

A record and a labeled tuple are not semantically identical merely because they expose similarly named entries.

Records are unordered structural field products; tuples remain ordered products.

**Kind:** Cross-family semantic law  
**Depends on:** T0654, T0664, T0702

### T0720 — `RecordWithNeverRequiredFieldIsNever`

A record with a required field of type `Never` is uninhabited and therefore semantically identical to `Never`.

**Kind:** Cross-family semantic law  
**Depends on:** T0152–T0154, T0700

### T0721 — `RecordFieldUnionWidening`

An immutable field may be widened to a union supertype.

**Kind:** Cross-family derived law  
**Depends on:** T0456, T0457, T0706

## Row/record separation

The row abstraction captures field membership, field labels, field projection, and row extension. Record semantics additionally determine:

- whether fields are readable or writable,
- whether width subtyping is permitted,
- how mutability affects variance,
- how record values are constructed,
- and which row contracts form proper record types.

This separation permits tuple entries and other structural features to reuse row machinery without inheriting record-specific order or subtyping semantics.

## Required implementation invariants

Record canonicalization must be independent of source field order.

Two records that differ only by source field permutation must converge on the same semantic record identity, while diagnostic/source information may retain original ordering separately.

Mutable field handling must not reuse immutable depth-subtyping rules without checking field capabilities.

## Explicitly deferred laws

- optional fields
- open rows and row variables
- polymorphic row tails
- read-only versus write-only capabilities
- getter/setter property decomposition
- method/interface members mixed with state rows
- object state/interface exposure
- nominal classes with structural record views
- row union/difference operations

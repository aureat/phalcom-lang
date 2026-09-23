# Phalcom Type Algebra Registry — `type.record-row`

**Status:** Normative specification  
**Registry families:** `type.record`, `type.row`  
**Identifier range used here:** `T0200–T0221`

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
@id(T0200)
@name(type.record.kind)
RecordKind<***Fields> {
    kind(record<***Fields>) == Type
}

@id(T0201)
@name(type.row.field-names.unique)
RowFieldNamesUnique<R> {
    unique(fields(R))
}

@id(T0202)
@name(type.record.field-order.irrelevant)
RecordFieldOrderIrrelevant<A, B> {
    { x: A, y: B } == { y: B, x: A }
}

@id(T0203)
@name(type.record.field-name.significant)
RecordFieldNamesSignificant<A> {
    { x: A } != { y: A }
}

@id(T0204)
@name(type.record.identity-substitution)
RecordFieldIdentitySubstitution<A, B, C> {
    if A == B {
        { x: A, y: C } == { x: B, y: C }
    }
}

@id(T0205)
@name(type.record.width-subtype)
RecordWidthSubtype<A, B> {
    { x: A, y: B } <: { x: A }
}

@id(T0206)
@name(type.record.depth-subtype)
RecordDepthSubtype<A, B> {
    if A <: B {
        { x: A } <: { x: B }
    }
}

@id(T0207)
@name(type.record.width-depth-subtype)
RecordWidthDepthSubtype<A1, A2, B1> {
    if A1 <: B1 {
        { x: A1, y: A2 } <: { x: B1 }
    }
}

@id(T0208)
@name(type.record.required-fields)
RecordSubtypeRequiresFields<R1, R2> {
    if R1 <: R2 {
        fields(R2) <= fields(R1)
    }
}

@id(T0209)
@name(type.record.field-subtype)
RecordSubtypeRequiresFieldCompatibility<R1, R2, N> {
    if R1 <: R2 and N in fields(R2) {
        R1::At<N> <: R2::At<N>
    }
}

@id(T0210)
@name(type.row.projection)
RowFieldProjection<R, N> {
    if N in fields(R) {
        kind(R::At<N>) == Type
    }
}

@id(T0211)
@name(type.row.projection.missing)
MissingRowFieldProjectionIllFormed<R, N> {
    if N not in fields(R) {
        illFormed(R::At<N>)
    }
}

@id(T0212)
@name(type.record.projection)
RecordFieldProjection<A, B> {
    { x: A, y: B }::At<"x"> == A
    { x: A, y: B }::At<"y"> == B
}

@id(T0213)
@name(type.record.mutable.invariant)
MutableRecordFieldInvariant<A, B> {
    mutable { x: A } <: mutable { x: B }
        iff A == B
}

@id(T0214)
@name(type.record.mutable.width)
MutableRecordWidthSubtype<A, B> {
    mutable { x: A, y: B } <: mutable { x: A }
}

@id(T0215)
@name(type.row.permutation.identity)
RowPermutationIdentity<A, B, C> {
    row { x: A, y: B, z: C } ==
    row { z: C, x: A, y: B }
}

@id(T0216)
@name(type.row.extension)
RowExtension<R, N, A> {
    if N not in fields(R) {
        fields(extend(R, N: A)) == fields(R) + { N }
    }
}

@id(T0217)
@name(type.row.extension.projection)
RowExtensionProjection<R, N, A> {
    if N not in fields(R) {
        extend(R, N: A)::At<N> == A
    }
}

@id(T0218)
@name(type.row.extension.preservation)
RowExtensionPreservesExistingFields<R, N, A, M> {
    if N not in fields(R) and M in fields(R) {
        extend(R, N: A)::At<M> == R::At<M>
    }
}

@id(T0219)
@name(type.record.tuple.distinct)
RecordTupleDistinct<A, B> {
    { x: A, y: B } != (x: A, y: B)
}

@id(T0220)
@name(type.record.never-field)
RecordWithNeverRequiredFieldIsNever<A> {
    { x: A, y: Never } == Never
}

@id(T0221)
@name(type.record.union.field-widening)
RecordFieldUnionWidening<A, B> {
    { x: A } <: { x: A | B }
}
```

## Law notes

### T0200 — `RecordKind`

Every well-formed record type is a proper type.

**Kind:** Semantic law  
**Depends on:** T0018

### T0201 — `RowFieldNamesUnique`

A row may not contain two semantic fields with the same label.

Duplicate-field source syntax must be rejected or resolved before a valid semantic row exists.

**Kind:** Well-formedness invariant  
**Depends on:** T0200

### T0202 — `RecordFieldOrderIrrelevant`

Record field ordering is not semantic.

Reordering fields preserves record identity.

**Kind:** Semantic identity law  
**Depends on:** T0001–T0003

### T0203 — `RecordFieldNamesSignificant`

Field labels are semantic keys. Structurally identical value types under different labels do not produce identical record types.

**Kind:** Semantic identity law  
**Depends on:** T0001–T0003

### T0204 — `RecordFieldIdentitySubstitution`

Replacing a field type with a semantically identical type preserves record identity.

**Kind:** Derived coherence law  
**Depends on:** T0001–T0003

### T0205 — `RecordWidthSubtype`

An immutable record with additional fields may be used where a subset of those fields is required.

**Kind:** Semantic subtype law  
**Depends on:** T0020–T0023

### T0206 — `RecordDepthSubtype`

Immutable record fields are covariant in their value type.

**Kind:** Semantic variance law  
**Depends on:** T0020–T0023

### T0207 — `RecordWidthDepthSubtype`

Width and depth subtyping compose.

A record may provide additional fields and narrower types for the fields required by the target.

**Kind:** Derived semantic law  
**Depends on:** T0205, T0206, T0021

### T0208 — `RecordSubtypeRequiresFields`

If `R1 <: R2`, the source record must provide every field required by the target record.

**Kind:** Semantic subtype invariant  
**Depends on:** T0205

### T0209 — `RecordSubtypeRequiresFieldCompatibility`

For immutable fields required by the target record, the corresponding source field type must subtype the target field type.

**Kind:** Semantic subtype invariant  
**Depends on:** T0206

### T0210 — `RowFieldProjection`

Projecting a present row field yields a proper type.

**Kind:** Kinding law  
**Depends on:** T0010, T0201

### T0211 — `MissingRowFieldProjectionIllFormed`

Static projection of a missing field is ill formed.

The compiler must not manufacture `Never`, `Any`, or `Unknown` as though the field existed.

**Kind:** Well-formedness invariant  
**Depends on:** T0013, T0201

### T0212 — `RecordFieldProjection`

Record field projection returns the type associated with the selected semantic field label.

**Kind:** Semantic projection law  
**Depends on:** T0210

### T0213 — `MutableRecordFieldInvariant`

A writable field is invariant in its value type.

Covariance would be unsound: code expecting `mutable { x: B }` could write any `B` into storage that only safely accepts a narrower `A`.

**Kind:** Semantic variance law  
**Depends on:** T0020–T0024

### T0214 — `MutableRecordWidthSubtype`

Field mutability does not by itself invalidate width subtyping when the retained field contracts are identical.

This law concerns dropping visibility of extra fields, not changing the writable type of retained fields.

**Kind:** Semantic subtype law  
**Depends on:** T0205, T0213

### T0215 — `RowPermutationIdentity`

Rows are unordered maps from semantic labels to field contracts.

**Kind:** Semantic row law  
**Depends on:** T0201, T0202

### T0216 — `RowExtension`

Extending a row with a fresh field adds exactly that semantic field.

**Kind:** Semantic row law  
**Depends on:** T0201

### T0217 — `RowExtensionProjection`

The newly introduced field projects to its declared type.

**Kind:** Semantic row law  
**Depends on:** T0210, T0216

### T0218 — `RowExtensionPreservesExistingFields`

Extending a row with a fresh field preserves every existing field projection.

**Kind:** Semantic row law  
**Depends on:** T0216, T0217

### T0219 — `RecordTupleDistinct`

A record and a labeled tuple are not semantically identical merely because they expose similarly named entries.

Records are unordered structural field products; tuples remain ordered products.

**Kind:** Cross-family semantic law  
**Depends on:** T0184, T0194, T0202

### T0220 — `RecordWithNeverRequiredFieldIsNever`

A record with a required field of type `Never` is uninhabited and therefore semantically identical to `Never`.

**Kind:** Cross-family semantic law  
**Depends on:** T0032–T0034, T0200

### T0221 — `RecordFieldUnionWidening`

An immutable field may be widened to a union supertype.

**Kind:** Cross-family derived law  
**Depends on:** T0106, T0107, T0206

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
- record spread syntax

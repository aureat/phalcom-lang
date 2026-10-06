# Phalcom Type Algebra Registry — `type.record-row`

**Status:** Normative specification
**Registry families:** `type.record`, `type.row`
**Identifier range used here:** `T0700–T0721`

## Purpose

This file defines the formal record and unordered-row laws required to agree
with [`structural-products.md`](../../structural-products.md). Records are
unordered field products. Their keys are `FieldKey`s; source ordering is not
semantic; and structural compatibility accounts for field presence and tail
state. Width compatibility is never implicit for a closed target.

Rows are the unordered structural substrate. Sharing row machinery does not
give Tuple entries Record behavior: Tuple order, `Symbol` labels, required
entries, and per-lane remainders remain governed by `type.tuple`.

## Domain and notation

Unless stated otherwise:

- `{ x: A, y: B }` denotes a closed Record type with two required fields.
- `{ x?: A }` denotes a closed Record with an optional `x` field.
- `{ x: A, ... }` denotes a Record with an unconstrained anonymous tail.
- `{ x: A, **: T }` denotes a Record with a typed anonymous tail.
- `fieldKeys(R)` denotes the set of declared fixed `FieldKey`s of `R`.
- `Lookup<K>` is the public unordered-reflection operation and returns a
  `FieldLookup` result: `Required(T)`, `Optional(T)`, `Absent`, or `Unknown`.
- `<:` means semantic subtype and `==` means semantic type identity.

Record keys are exactly:

```telos
type FieldKey =
    Symbol | String | Int | Bool
```

`Record::At<K>` is not part of the ratified public Record reflection model.
The laws below use `Lookup<K>` and `FieldLookup` instead.

## Laws

```phalcom
@id(T0700)
@name(type.record.kind)
RecordKind<***Fields> {
    kind(record<***Fields>) == Type
}

@id(T0701)
@name(type.row.field-key.domain)
FieldKeyDomain<K> {
    K : FieldKey iff K : Symbol | String | Int | Bool
}

@id(T0702)
@name(type.row.field-keys.unique)
RowFieldKeysUnique<R> {
    unique(fieldKeys(R))
}

@id(T0703)
@name(type.record.field-order.irrelevant)
RecordFieldOrderIrrelevant<A, B> {
    { x: A, y: B } == { y: B, x: A }
}

@id(T0704)
@name(type.record.field-key.significant)
RecordFieldKeysSignificant<A> {
    { x: A } != { y: A }
}

@id(T0705)
@name(type.record.field.identity-substitution)
RecordFieldIdentitySubstitution<A, B, C> {
    if A == B {
        { x: A, y: C } == { x: B, y: C }
    }
}

@id(T0706)
@name(type.record.closed-target.rejects-surplus)
ClosedRecordTargetRejectsSurplusFields<A, B> {
    { x: A, y: B } </: { x: A }
}

@id(T0707)
@name(type.record.open-target.width)
OpenRecordTargetAdmitsSurplusFields<A, B> {
    { x: A, y: B } <: { x: A, ... }
}

@id(T0708)
@name(type.record.typed-tail.width)
TypedRecordTailConstrainsSurplusFields<A, B, T> {
    if B <: T {
        { x: A, y: B } <: { x: A, **: T }
    }
}

@id(T0709)
@name(type.record.required.presence)
RequiredTargetFieldNeedsRequiredSource<A, B> {
    if A <: B {
        { x: A } <: { x: B }
    }

    { x?: A } </: { x: A }
}

@id(T0710)
@name(type.record.optional.presence)
OptionalTargetFieldCompatibility<A, B> {
    if A <: B {
        { x: A } <: { x?: B }
        { x?: A } <: { x?: B }
        {} <: { x?: B }
    }
}

@id(T0711)
@name(type.record.optional.open-source)
OptionalTargetFieldNeedsCompatibleOpenSource<A, B> {
    if A <: B {
        { **: A } <: { x?: B, ... }
    }

    { ... } </: { x?: B, ... }
}

@id(T0712)
@name(type.record.open-source.closed-target)
OpenSourceCannotSatisfyClosedTarget<S, T> {
    if hasOpenTail(S) and closedRecord(T) {
        S </: T
    }
}

@id(T0713)
@name(type.record.lookup.field-lookup)
RecordLookupProducesFieldLookup<A> {
    { x: A }.Lookup<x> == Required(A)
    { x?: A }.Lookup<x> == Optional(A)
    {}.Lookup<x> == Absent
    { ... }.Lookup<x> == Unknown
}

@id(T0714)
@name(type.row.captured-tail.lookup)
CapturedTailPreservesLookupCorrelation<R, K> {
    if capturedTail(R) == Q {
        R.Lookup<K> == Q.Lookup<K>
    }
}

@id(T0715)
@name(type.row.permutation.identity)
RowPermutationIdentity<A, B, C> {
    row { x: A, y: B, z: C } ==
    row { z: C, x: A, y: B }
}

@id(T0716)
@name(type.row.extension)
RowExtension<R, K, A> {
    if K : FieldKey and K not in fieldKeys(R) {
        fieldKeys(extend(R, K: Required(A))) == fieldKeys(R) + { K }
    }
}

@id(T0717)
@name(type.row.extension.lookup)
RowExtensionLookup<R, K, A> {
    if K : FieldKey and K not in fieldKeys(R) {
        extend(R, K: Required(A)).Lookup<K> == Required(A)
    }
}

@id(T0718)
@name(type.row.extension.preservation)
RowExtensionPreservesExistingLookup<R, K, A, M> {
    if K : FieldKey and K not in fieldKeys(R) and M in fieldKeys(R) {
        extend(R, K: Required(A)).Lookup<M> == R.Lookup<M>
    }
}

@id(T0719)
@name(type.record.tuple.distinct)
RecordTupleDistinct<A, B> {
    { x: A, y: B } != (x: A, y: B)
}

@id(T0720)
@name(type.record.never.required-field)
RecordWithNeverRequiredFieldIsNever<A> {
    { x: A, y: Never } == Never
}

@id(T0721)
@name(type.record.union.field-widening)
RecordFieldUnionWidening<A, B> {
    { x: A } <: { x: A | B }
}
```

## Record compatibility model

Record compatibility is determined as a whole relation, not by the old
field-set-subset shortcut. It checks declared-field presence, fixed field
compatibility, source and target tail state, and any correlation retained by a
captured tail.

### Declared target fields

- A target required field `K: T` needs a source result
  `Lookup<K> == Required(S)` with `S <: T`. A source `Optional(S)` result
  cannot satisfy it.
- A target optional field `K?: T` accepts `Required(S)` or `Optional(S)` only
  when `S <: T`. `Absent` satisfies it. `Unknown` does not establish
  compatibility.
- For an undeclared `K`, a closed source yields `Absent`. A typed anonymous
  source tail proves a compatible possible value for `K` only when its tail
  value constraint is a subtype of the target value type. An unconstrained
  anonymous source tail yields `Unknown`. A captured tail is checked through
  its correlated `Lookup<K>` result rather than treated as an anonymous tail.

### Tail and surplus-field rules

A Record tail is one of these states:

1. **Closed** — no surplus fields are admitted.
2. **Captured** — preserves an exact, correlated row identity.
3. **Typed** (`**: T`) — admits zero or more surplus fields whose values are
   subtypes of `T` (and any declared key-domain constraint).
4. **Unconstrained** (`...` or `**: ...`) — admits zero or more surplus fields
   without a common value-type constraint.

An open source cannot satisfy a closed target. A closed target rejects every
surplus source field. An unconstrained open target admits surplus fields. A
typed open target requires every surplus fixed source field and every possible
source tail contribution to satisfy its value constraint. Captured source tails
remain correlated and must be proven compatible through their retained shape,
not discarded into a fresh anonymous row.

Thus width compatibility is explicit:

```telos
{ x: A, y: B } <: { x: A, ... }
```

but not implicit:

```telos
{ x: A, y: B } </: { x: A }
```

## Law notes

### T0700–T0705 — Record identity and `FieldKey`

Every well-formed Record is a proper type. Fixed keys are unique and belong to
the `FieldKey` domain. Record source order is not semantic, but changing a key
changes the Record identity. Replacing a field value type with an identical type
preserves identity.

**Kind:** Semantic and well-formedness laws
**Depends on:** T0001–T0003, T0060

### T0706–T0708 — Explicit width compatibility

Closed targets do not perform width subtyping. A target admits surplus source
fields only through an explicit tail; a typed tail also constrains their value
types.

**Kind:** Semantic subtype laws
**Depends on:** T0100–T0103

### T0709–T0711 — Presence and open-source compatibility

Required and optional field presence are part of Record compatibility.
Optional presence is not `Option<T>` or `T | Unit`. A closed source may omit an
optional target key; an open source must prove the possible key value is
compatible. An unconstrained source tail cannot supply that proof.

**Kind:** Semantic subtype laws
**Depends on:** T0700, T0706

### T0712–T0714 — Tails and reflection

Closed/open status is part of compatibility. Public unordered reflection uses
`Lookup<K>` and `FieldLookup`; no independent `At` projection is introduced.
Captured tails retain their exact correlation for lookup and compatibility.

**Kind:** Semantic compatibility and reflection laws
**Depends on:** T0701, T0702, T0709–T0711

### T0715–T0718 — Unordered row operations

Row permutation preserves identity. Extending a row by a fresh `FieldKey`
introduces the corresponding required lookup result and preserves every other
fixed lookup.

**Kind:** Semantic row laws
**Depends on:** T0701–T0703, T0713

### T0719–T0721 — Cross-family derived laws

Records are distinct from labeled Tuples. A required `Never` field makes a
Record uninhabited, and required field values are covariant under union
widening.

**Kind:** Cross-family derived laws
**Depends on:** T0152–T0154, T0456, T0457, T0654, T0664

## Required implementation invariants

Record canonicalization must be independent of source order and use the
language-defined `FieldKey` ordering. It must preserve required/optional
presence, tail state, and captured-tail correlation. Consumers must use the
canonical `Lookup<K>` / `FieldLookup` product rather than reconstructing field
presence from a field set.

## Current boundary

Mutable structural Record types are not part of the current Record model.
Any future mutable structural Record rules require a separately ratified
specification; this registry defines no `mutable { ... }` laws.

## Explicitly deferred laws

- read-only versus write-only capabilities
- getter/setter property decomposition
- method/interface members mixed with state rows
- object state/interface exposure
- nominal classes with structural Record views
- row union/difference operations

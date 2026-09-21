# TDR-0083 — Type-level structure has distinct domains; compile-time values are typed values, not kinds

- Ratified: 2026-09-21
- Status: Accepted
- Date: 2026-09-21
- Supersedes: none
- Amends: none
- Depends on: [TDR-0081](./0081-tuple-and-record-are-transparent-structural-value-products.md)
- Related: LANG007; `docs/specs/generic-constraints.md`; `docs/specs/associated-types.md`; `docs/specs/structural-products.md`; `docs/specs/type-level-domains.md`
- Repository evidence revision: `aureat/phalcom-lang@5b63ade8073d13d6b528af4774181665c30cbd47`

## Context

Phalcom already distinguishes proper types from other type-level forms. The semantic kind system contains `Type`, `RecordRow`, and arrow kinds, and higher-kinded forms such as `Option : Type -> Type` already participate in type application without themselves being proper runtime value types.

LANG007 extends the language with variadic structural typing and compiler-known structural type operations. The intended surface includes ordered type packs, ordered labeled type packs, Tuple type indexing, and structural type transforms such as:

```phalcom
(Int, String)::Positionals
(Int, String, name: Symbol)::Labeled

(Int, String)::At<1>
(Int, String)::Map<Option>

P::Map<Option> // P: TypePack
R::Map<Option> // R: RecordRow
```

This feature introduces two distinct questions that must not be conflated:

1. what kinds of type-level structural values exist; and
2. how compile-time non-type arguments such as `1` in `At<1>` are classified.

Several superficially convenient encodings are semantically wrong:

- representing `TypePack` as a proper Tuple type;
- representing a labeled Tuple pack as `RecordRow`;
- treating `Int` as a kind merely because `I: Int` appears in a type-member parameter;
- treating every generic/member argument as a `TypeId` denoting a proper type;
- conflating ordered Tuple labels with order-independent Record fields.

These shortcuts would make later pack inference, projection normalization, metadata, and reflection depend on accidental implementation representations instead of the language model.

## Decision

### 1. Phalcom has distinct type-level structural domains

The type-level kind universe admits at least:

```text
κ ::= Type
    | RecordRow
    | TypePack
    | LabeledTypePack
    | (κ₁, ..., κₙ) -> κ
```

`Type`, `RecordRow`, `TypePack`, and `LabeledTypePack` are distinct semantic domains.

No implicit coercion exists merely because two domains can contain related information.

### 2. `TypePack` is an ordered sequence of proper types

A `TypePack` is an ordered finite sequence:

```text
[T₀, T₁, ..., Tₙ]
```

where every `Tᵢ` is a proper type of kind `Type`.

Therefore:

```text
[Int, String] : TypePack
```

but the pack itself is not a proper runtime value type.

Pack order is semantic.

The following are not equivalent:

```text
[Int, String]
[String, Int]
```

unless each corresponding element is canonically identical in the same position.

### 3. `LabeledTypePack` is an ordered sequence of unique labels and proper types

A labeled type pack is:

```text
[(ℓ₀, T₀), (ℓ₁, T₁), ..., (ℓₙ, Tₙ)]
```

where:

- every payload `Tᵢ` is a proper type;
- every structural label is unique;
- label order is semantic.

Thus:

```text
[(name, String), (age, Int)]
```

is not identical to:

```text
[(age, Int), (name, String)]
```

unless the order itself is also identical.

### 4. `LabeledTypePack` and `RecordRow` are not the same domain

A `RecordRow` is key-oriented structural type information whose field presentation order does not participate in canonical row identity.

A `LabeledTypePack` is sequence-oriented structural type information whose label order participates in canonical identity.

They may contain analogous label/type pairs, but they do not share equality, canonicalization, or inference semantics.

The implementation may share generic algorithms where sound; it must not share canonical identity merely for convenience.

### 5. Proper types and higher-kinded type forms remain distinct

A proper runtime value type has kind `Type`.

A type constructor can have a higher kind:

```text
Option : Type -> Type
```

and becomes a proper type only after a well-kinded application:

```text
Option<Int> : Type
```

This distinction is the basis for `Map<Option>`.

`Map<F>` accepts a type-level form whose kind is `Type -> Type`; it does not require `F` itself to be a proper type.

### 6. Generic/member parameters have semantic parameter domains

A parameter is classified by one of two broad domain forms:

```text
ParameterDomain
    = Kind(κ)
    | Const(τ)
```

Examples:

```phalcom
T: Type
P: TypePack
L: LabeledTypePack
F: Type -> Type
I: Int
```

mean conceptually:

```text
T  : Kind(Type)
P  : Kind(TypePack)
L  : Kind(LabeledTypePack)
F  : Kind(Type -> Type)
I  : Const(Int)
```

### 7. `I: Int` does not make `Int` a kind

A parameter such as:

```phalcom
I: Int
```

ranges over compile-time values whose ordinary value type is `Int`.

`Int` remains a proper type of kind `Type`.

The exact literal:

```text
0
```

used as a type-member argument in:

```phalcom
T::At<0>
```

is therefore a compile-time `Int` term, not a type and not a kind.

If Phalcom later gains singleton literal types, expected parameter domain still distinguishes a compile-time integer argument from a type expression.

### 8. Binder `:` classifies the parameter domain

Within generic/type-member binder syntax, `:` classifies the parameter into its semantic parameter domain.

A kind expression on the right establishes `Kind(κ)`.

A proper type on the right establishes `Const(τ)` where that declaration category permits compile-time value parameters.

This ruling does not introduce trait-bound shorthand.

Trait conformance remains governed by `|=` rather than being reinterpreted through `:`.

### 9. LANG007 does not thereby introduce general const generics

This decision defines the semantic model required by LANG007.

It does not by itself require every nominal class, enum, trait, alias, or callable generic parameter list to accept `Const(τ)` parameters immediately.

LANG007 first requires typed compile-time arguments for the type-member operations defined by later decisions.

Broader const-generic surface syntax remains a separate language feature unless explicitly ratified.

## Consequences

Phalcom gains a coherent domain model for variadic structural type computation without making packs into runtime container types.

Tuple labeled structure can preserve order without fighting Record row canonicalization.

Higher-kinded forms such as `Option` participate in structural transformations through the same kind system already used for type application.

Compile-time integer arguments can be represented and checked without inventing an `Int` kind or forcing integer literals into the proper-type arena.

The decision increases the number of semantic domains that generic substitution, inference, metadata, and tooling must understand. This cost is intentional: domain distinctions are part of the language meaning rather than optional implementation detail.

Future pack inference, variadic generics, and const generics can extend this model without redefining the meaning of existing terms.

## Rejected alternatives

### Encode `TypePack` as Tuple

Rejected because a pack is a type-level sequence of type terms, not a proper structural value type.

### Encode `LabeledTypePack` as `RecordRow`

Rejected because Tuple label ordering is semantic while Record field ordering is not.

### Treat every binder RHS as a kind

Rejected because `I: Int` classifies a compile-time value by its value type; `Int` is not a kind.

### Treat compile-time integer arguments as singleton proper types

Rejected for LANG007. Singleton literal types, if introduced later, are a distinct feature and must not be required merely to represent `At<1>`.

### Generalize all generic applications to const arguments immediately

Rejected as unnecessary scope expansion. This TDR defines the semantic foundation without forcing every declaration category to expose it at once.

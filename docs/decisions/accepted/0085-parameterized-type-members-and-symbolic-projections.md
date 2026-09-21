# TDR-0085 — Type members are parameterized, domain-aware projections with symbolic residuals

- Ratified: 2026-09-21
- Status: Accepted
- Date: 2026-09-21
- Supersedes: none
- Amends: none
- Depends on: [TDR-0083](./0083-type-level-domains-and-typed-compile-time-parameters.md), [TDR-0084](./0084-canonical-type-level-terms-and-inference-lifetimes.md)
- Related: LANG007; associated types; `docs/specs/type-members.md`; `docs/specs/associated-types.md`
- Repository evidence revision: `aureat/phalcom-lang@5b63ade8073d13d6b528af4774181665c30cbd47`

## Context

Phalcom already has trait-associated types.

An associated declaration such as:

```phalcom
trait Iterator {
    type Item
}
```

is trait-owned, participates in conformance evidence, and currently denotes a proper type.

LANG007 needs a broader type-level member concept.

Examples include:

```phalcom
(Int, String)::Positionals
(Int, String)::At<1>
(Int, String)::Map<Option>
P::Map<Option>
```

These operations differ from current associated types in several ways:

- they may be compiler/domain-owned rather than trait-owned;
- they may accept parameters;
- those parameters may be kinded forms or typed compile-time values;
- they may return `TypePack`, `LabeledTypePack`, `RecordRow`, or `Type`;
- they may remain symbolic even when no conformance lookup is involved.

It would be possible to implement each operation as a one-off compiler node or by overloading current associated-type projection storage, but that would duplicate identity, normalization, metadata, and diagnostics. It would also make later type-level operations harder to compose coherently.

## Decision

### 1. Phalcom has a general semantic category of type member

A `TypeMember` is a declaration/domain-owned type-level member whose application produces a type-level result.

Type-member ownership includes at least:

```text
trait-associated
domain-native/compiler-defined
```

Future language features may add other ownership forms such as ordinary inherent type members without changing the core projection model.

### 2. Type-member identity is declaration/domain identity, not spelling identity

Two type members with the same written name are not automatically the same member.

Canonical identity is owned by the semantic declaration or builtin structural domain.

For example:

```text
Tuple-domain Map
Record-domain Map
TypePack-domain Map
```

are distinct member identities even though they share the presentation name `Map`.

Likewise, a trait-associated member named `Map` does not become the intrinsic structural `Map`.

### 3. Type members declare parameter domains

A type member may declare parameters such as:

```phalcom
F: Type -> Type
I: Int
```

whose semantic domains are respectively:

```text
Kind(Type -> Type)
Const(Int)
```

The parser may preserve ambiguous source argument syntax, but semantic elaboration checks each argument against the member's expected parameter domain.

### 4. Type members declare result domains

A type member is not required to produce a proper type.

Examples:

```text
Positionals
    () -> TypePack

Labeled
    () -> LabeledTypePack

At<I: Int>
    -> Type

Tuple::Map<F: Type -> Type>
    -> Type

TypePack::Map<F: Type -> Type>
    -> TypePack

RecordRow::Map<F: Type -> Type>
    -> RecordRow
```

The declared result domain is part of the member's semantic contract and projection identity.

### 5. Type-member application creates a canonical projection

Conceptually, projection identity contains:

```text
subject
member identity
canonical arguments
qualification/evidence context where required
result domain
```

Source spelling and source span are not canonical identity.

A unified projection substrate may represent both trait-associated and compiler-intrinsic projections, provided their different normalization authorities remain explicit.

### 6. Trait-associated projection semantics remain authoritative

Generalizing projection representation does not weaken existing trait-associated semantics.

Trait-associated types continue to be resolved through:

```text
trait requirement identity
trait reference
conformance/generic evidence
associated binding
```

A compiler-intrinsic member does not bypass or replace that proof system.

### 7. A projection can be reduced, stuck, or invalid

The semantic normalizer distinguishes at least:

```text
Reduced
Stuck
Invalid
```

A `Stuck` projection is well formed but lacks enough concrete information to reduce.

Examples:

```phalcom
P::Map<Option>
T::At<I>
```

may remain symbolic when their subjects/arguments are abstract but valid.

An `Invalid` projection has enough information to prove the operation is not legal.

Examples include:

```phalcom
(Int, String)::Map<Int>
(Int, String)::At<100>
```

where the constructor kind or concrete index violates the member contract.

### 8. `Stuck` does not mean `Dynamic`

A well-formed symbolic projection must not be approximated as:

```text
Dynamic
Never
Unknown
```

merely because it cannot yet reduce.

Those categories have separate meanings.

### 9. Domain-native members do not make structural types ordinary `impl` targets

Tuple and Record structural types may expose compiler-defined type members without becoming open user-extensible nominal classes.

The following implication is invalid:

```text
Tuple has intrinsic type member Map
therefore
user source may add arbitrary inherent members to every structural Tuple shape
```

Structural `impl` remains governed by its own language rules.

### 10. Parameterized domain-native type members do not imply generic associated types

LANG007 requires parameterized domain-native members.

It does not by this decision legalize:

```phalcom
trait X {
    type Family<T>
}
```

or otherwise introduce generic associated types.

GAT syntax, coherence, conformance binding, defaults, and metadata remain a separate future decision.

### 11. Type-member lookup is domain-aware

A member is available because the subject's semantic domain establishes applicability.

A generic:

```phalcom
T: Type
```

does not automatically have every structural member.

A generic:

```phalcom
P: TypePack
```

may use TypePack-domain members because its domain is known.

Trait-associated ambiguity remains subject to explicit qualification.

No arbitrary "inherent wins" or name-only precedence rule is introduced.

## Consequences

Phalcom gains one reusable type-level member model for structural operations without misclassifying them as runtime methods or trait associated types.

Non-`Type` results can be represented without faking proper types.

Generic code can preserve symbolic projections soundly.

Existing associated-type implementations may be migrated onto a generalized projection substrate while preserving their proof/evidence semantics.

The compiler must maintain typed member signatures, canonical member identity, projection interning, result-domain checking, and source-navigation identity.

## Rejected alternatives

### Make every operation a dedicated compiler AST/type node

Rejected because identity, normalization, metadata, and diagnostics would duplicate per operation.

### Reuse member names as identity

Rejected because same-spelled members may belong to different domains or traits.

### Treat all type members as trait-associated types

Rejected because domain-native structural operations do not require conformance evidence.

### Introduce GATs as part of LANG007

Rejected as unnecessary expansion of trait semantics.

### Resolve abstract projections to `Dynamic`

Rejected because a symbolic type-level expression remains meaningful static information.

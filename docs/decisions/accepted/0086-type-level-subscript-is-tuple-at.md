# TDR-0086 — Tuple type subscript and `At<I>` are one compile-time member identity

- Ratified: 2026-09-21
- Status: Accepted
- Date: 2026-09-21
- Supersedes: none
- Amends: none
- Depends on: [TDR-0081](./0081-tuple-and-record-are-transparent-structural-value-products.md), [TDR-0083](./0083-type-level-domains-and-typed-compile-time-parameters.md), [TDR-0085](./0085-parameterized-type-members-and-symbolic-projections.md)
- Related: [TDR-0049](./0049-index-operator-as-real-selector.md); LANG007; Tuple structural typing
- Repository evidence revision: `aureat/phalcom-lang@5b63ade8073d13d6b528af4774181665c30cbd47`

## Context

TDR-0049 established that runtime bracket syntax is a real selector surface.

For runtime values:

```phalcom
value[index]
```

is not lowered to a runtime `at` method. Core classes opt into bracket behavior explicitly.

LANG007 introduces a distinct operation in the type-level member domain:

```phalcom
(Int, String)[1]
```

which denotes structural Tuple type indexing.

The language also wants a named type-level presentation:

```phalcom
(Int, String)::At<1>
```

If these two forms are modeled as separate type members that merely happen to normalize to equal results, several problems follow:

- metadata can report two identities for one conceptual operation;
- source navigation can disagree;
- diagnostics can drift;
- one spelling can gain behavior not present in the other;
- canonical equality depends on post-hoc normalization rather than member identity.

The type-level feature must also decide whether labels participate in integer indexing, whether negative indexes are supported, and how concrete bounds errors differ from symbolic projections.

## Decision

### 1. Type-level bracket indexing and `At<I>` denote one semantic member

For a subject supporting the Tuple structural subscript member:

```phalcom
T[I]
```

and:

```phalcom
T::At<I>
```

denote the same canonical type-member projection.

Formally:

```text
T[I] ≡ T::At<I>
```

at semantic member/projection identity, not merely after reduction.

### 2. The canonical semantic selector is the type-level subscript member

The canonical member identity is the subscript operation.

`At` is the standardized named presentation alias for that same member.

There is no independently declared canonical `At` member whose result is compared later to bracket indexing.

If source declarations use a bracket-shaped type-member syntax, that declaration directly owns the canonical identity.

If the parser/specification uses a named declaration form, bracket syntax still resolves to the same canonical member identity.

### 3. This does not amend runtime TDR-0049

TDR-0049 governs runtime message selector identity.

TDR-0086 governs type-level structural projection identity.

They intentionally share analogous surface syntax but occupy different semantic categories:

```text
runtime value[index]
    runtime bracket selector/send

type-form[index]
    type-member structural projection
```

No runtime `at` lowering is reintroduced.

No runtime selector is reused as the canonical type-member identity.

### 4. The initial type-level subscript argument domain is compile-time `Int`

The member parameter is:

```text
I : Const(Int)
```

or the equivalent source declaration:

```phalcom
type [I: Int]: Type
```

A runtime integer expression is not a valid type-member argument merely because its runtime value type is `Int`.

LANG007 does not initially add Symbol or Range type-level Tuple indexing.

### 5. Tuple integer indexing spans the full structural component order

For a Tuple with positional prefix `P` and labeled suffix `L`:

```text
linear(Tuple(P, L))
    =
P ++ eraseLabels(L)
```

Integer type indexing addresses that complete sequence.

For example:

```phalcom
(Int, String, name: Symbol, enabled: Bool)
```

has linear index order:

```text
0 -> Int
1 -> String
2 -> Symbol
3 -> Bool
```

Tuple labels remain available as structural metadata; integer indexing selects payload type.

### 6. Negative indexing is supported

For concrete total arity `N` and compile-time integer `i`:

```text
j =
    i       if i >= 0
    N + i   if i < 0
```

The index is valid iff:

```text
0 <= j < N
```

Examples:

```phalcom
(Int, String)::At<0>   == Int
(Int, String)::At<1>   == String
(Int, String)::At<-1>  == String
(Int, String)::At<-2>  == Int
```

### 7. Concrete out-of-bounds indexing is invalid

The following are static invalid projections:

```phalcom
(Int, String)::At<2>
(Int, String)::At<-3>
```

They are not:

```text
Dynamic
Never
Unknown
```

and they do not become runtime bounds checks.

### 8. Abstract indexing may remain symbolic

A well-formed projection such as:

```phalcom
T::At<I>
```

may remain symbolic when the Tuple shape or integer parameter is not yet concrete but the subject/member contract is established.

This is `Stuck`, not invalid.

### 9. Zero product has no valid concrete integer index

Under the accepted zero-product semantics inherited from TDR-0081:

```phalcom
()[0]
```

is statically invalid.

This decision does not create a synthetic element type for Unit.

## Consequences

The language gets one coherent type-level indexing operation with two useful presentations.

Tuple type indexing aligns with existing runtime Tuple integer behavior without coupling semantic type projection to runtime dispatch.

Negative indexes become usable in generic compile-time structural programming.

Metadata, source indexing, diagnostics, and normalization have one member identity to preserve.

The parser and semantic resolver must distinguish runtime expression indexing from type-level member indexing based on syntactic/semantic context.

## Rejected alternatives

### Make `At` and `[]` separate members

Rejected because it duplicates identity for one operation and makes equivalence dependent on normalization coincidence.

### Lower type-level brackets to runtime `[]`

Rejected because type checking must not execute runtime message dispatch.

### Index only positional Tuple entries

Rejected because runtime Tuple component order and structural shape include the labeled lane after the positional prefix.

### Reject negative type indexes

Rejected because negative integer indexing is already part of Tuple's runtime indexing semantics and has a deterministic compile-time normalization.

### Add Symbol and Range type indexing now

Rejected as unnecessary scope expansion. They remain separate future design questions.

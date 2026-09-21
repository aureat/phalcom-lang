# TDR-0084 — Canonical type-level terms are stable kinded identities; inference variables are ephemeral

- Ratified: 2026-09-21
- Status: Accepted
- Date: 2026-09-21
- Supersedes: none
- Amends: none
- Depends on: [TDR-0083](./0083-type-level-domains-and-typed-compile-time-parameters.md)
- Related: LANG007; semantic type interning; generic inference; metadata export; incremental compilation
- Repository evidence revision: `aureat/phalcom-lang@5b63ade8073d13d6b528af4774181665c30cbd47`

## Context

LANG007 introduces new type-level domains and structural operations whose values can remain abstract for significant portions of semantic analysis.

Examples include:

```phalcom
P::Map<Option> // P: TypePack
R::Map<Option> // R: RecordRow
T::At<I>
```

These expressions are well formed before all of their inputs are concrete.

The current compiler already has an important architectural distinction:

```text
canonical semantic identities
    versus
query-local inference variables
```

`TypeId`, `RecordRowId`, type parameter identities, and interned type forms are stable semantic identities within their owning store.

`InferVarId` and `RecordRowVarId` are temporary solver identities used only while answering one inference query.

LANG007 must preserve this distinction. If symbolic pack transformations or type-member projections are represented only as solver metavariables, then they cannot survive substitution, module boundaries, metadata export, incremental recomputation, or reflection. Conversely, if query-local inference IDs leak into durable semantic state, canonical equality and caching become dependent on one particular inference run.

A second issue is concrete canonicalization. Variadic structural types may be assembled through symbolic pack forms, but once all components become concrete they must not acquire a second identity parallel to Phalcom's existing structural Tuple or Record types.

## Decision

### 1. Canonical type-level terms and inference variables are different categories

A canonical type-level term represents stable language meaning.

An inference variable represents an unresolved query-local obligation.

No query-local inference identity may become canonical merely because a term remains abstract.

Conceptually:

```text
CanonicalTerm != InferenceVariable
```

### 2. Canonical kinded terms may inhabit any accepted kind

The canonical semantic store may identify forms of kinds such as:

```text
Type
Type -> Type
TypePack
LabeledTypePack
```

A canonical kinded-form identity is therefore broader than a proper runtime value type identity.

A separate proper-type proof/category must continue to identify the subset whose kind is exactly `Type`.

In the current implementation this maps naturally to the distinction:

```text
TypeId
    canonical identity of a kinded type-level form

ProperTypeId
    proof that a TypeId has kind Type
```

This Rust spelling is not itself the permanent language contract; the semantic distinction is.

### 3. `RecordRow` remains a distinct canonical domain

This decision does not force all domains into one arena.

`RecordRow` keeps its separate row representation because it has established open-tail semantics, row inference, row canonicalization, and order-independent field identity.

Uniform syntax does not require uniform storage identity.

### 4. Canonical symbolic terms are first-class semantic terms

A well-formed type-level expression may remain canonical and symbolic.

Stable symbolic terms include, where applicable:

```text
generic parameter references
type-member projections
Map projections
symbolic Tuple formation from packs
row projections
```

Their identity must be defined structurally from stable semantic inputs.

They must not depend on:

```text
source span
temporary solver slot
allocation address
diagnostic instance
query-local inference ID
```

### 5. Concrete symbolic structural forms collapse to existing canonical identities

A symbolic Tuple formation that later becomes fully concrete must normalize to the already-existing structural Tuple identity.

If it has zero components, it follows the accepted zero-product rule.

There must not be two canonical identities for one concrete structural type merely because one was written directly and one was produced by symbolic pack substitution.

Likewise, a symbolic Record or mapped row that becomes fully concrete must canonicalize into the existing Record/row identity model.

### 6. Query-local inference variables never enter durable semantic state

Temporary inference identities such as the current:

```text
InferVarId
RecordRowVarId
```

or future equivalents may exist only in query-local solver state.

They must not be serialized into:

```text
TypeStore canonical nodes
RecordRow canonical terms
module metadata
incremental fingerprints
cross-module public identities
reflection descriptors
source indexes
snapshots intended to outlive the inference query
```

### 7. Pack-kinded whole-value inference can reuse kind-aware inference

A parameter of kind:

```text
TypePack
```

or:

```text
LabeledTypePack
```

does not require a dedicated pack metavariable solely because its kind is new.

A kind-aware ordinary inference variable may represent an unknown whole pack as long as the solver enforces the declared kind.

A separate sequence/pack solver becomes justified only when later features introduce structural pack decomposition, expansion, concatenation, prefix/suffix matching, or split inference.

LANG007 C1 does not pre-commit to such a solver merely to represent an unknown pack.

### 8. Stable metadata serializes semantic structure, not local IDs

Durable metadata must encode new type-level forms through stable graph structure or stable declaration references.

It may serialize:

```text
pack node contents
type-member identity
projection subject/member/arguments/result domain
stable generic parameter references
stable row projection structure
```

It must not serialize raw canonical arena indexes or query-local inference IDs as language identity.

### 9. Canonical equality is semantic equality

Interning/hash-consing is an implementation strategy for canonical identity, not the definition of language equality.

Two terms that the language defines as identical must canonicalize identically even if they were constructed through different syntactic or inference paths.

## Consequences

The compiler can preserve abstract type-level structure across substitution, inference, metadata, and incremental analysis without manufacturing fake concrete types.

Pack-valued generic parameters can initially reuse existing kind-aware inference rather than forcing a new solver.

The metadata schema must grow to represent the new stable terms explicitly.

Some semantic APIs that historically assumed every `TypeId` denotes a proper runtime value type must instead check the form's kind or require a proper-type wrapper.

This is a desirable tightening: proper-type use becomes explicit instead of accidental.

## Rejected alternatives

### Store symbolic Map/At only as inference variables

Rejected because well-formed abstract projections are durable semantic expressions, not merely solver holes.

### Introduce a dedicated canonical PackId arena immediately

Rejected for kinded pack forms while the existing kinded-form identity can represent them correctly. A separate arena may be justified later only by a concrete architectural need, not by the fact that packs are not proper types.

### Serialize inference IDs into metadata

Rejected because their identity is query-local and unstable.

### Keep a separate concrete Tuple-from-packs identity

Rejected because one concrete structural Tuple shape must have one canonical identity.

### Convert unresolved projections to `Dynamic`

Rejected because abstraction is not loss of type evidence and `Dynamic` has different language semantics.

# Phalcom Semantic Analyzer Specification
## 09 — Semantic Products, Incrementality, and Fingerprints

**Status:** Normative semantic-analyzer specification.

**Purpose:** Specify semantic product identity, dependency ownership, fingerprint equivalence, invalidation, and the distinction between semantic changes and incidental source/allocator changes.

---

## 1. Incrementality is part of semantic correctness

An incremental analyzer can produce wrong answers even when every fresh computation is correct if it reuses a stale semantic product after a meaningful dependency changed.

Therefore semantic product identity is a correctness contract.

The fundamental rule is:

> Two products may share a semantic fingerprint only when all downstream-observable semantic behavior represented by that product is equivalent.

Conversely, incidental changes that do not alter semantic meaning should not force unnecessary downstream recomputation.

---

## 2. Three kinds of identity

The implementation must distinguish:

```text
semantic identity
input/source identity
ephemeral allocator identity
```

### 2.1 Semantic identity

Meaning that downstream semantic consumers may observe.

Examples:

```text
type
evidence status/origin where observable
binding contract
binding consistency
mutability
analysis status
causal shape
resolved callable identity
generic constraints
flow epistemic state
```

### 2.2 Input/source identity

Information that may require recomputing a product's payload or ranges even if semantic meaning is unchanged.

Examples:

```text
source text
source range shifts
presentation spans
comments/docs depending on product design
```

### 2.3 Ephemeral identity

Allocator-local values with no meaning across equivalent analyses.

Examples:

```text
DiagnosticCauseId allocation number
arena insertion index when order is incidental
temporary solver variable numbers not exposed semantically
```

Ephemeral identity must not accidentally become semantic cache identity.

---

## 3. Product boundaries

Representative semantic products include:

```text
declaration shell
declaration surface
callable signature
callable body / CallableAnalysis
flow summaries
source semantic indices
semantic snapshot projections
advisory products
```

Each product needs a documented semantic contract.

A consumer depends on the product whose semantics it actually read, not merely on a nearby broader source object.

---

## 4. Semantic product fingerprints

A product fingerprint summarizes semantic meaning for reuse/invalidation.

The implementation may use stable hashing, structured equality, or another mechanism. The hash algorithm is not normative.

What is normative is the equivalence relation.

For a callable-body product, semantically relevant dimensions include the parts of expression/binding/call/diagnostic state that downstream consumers are allowed to observe.

If a status or evidence-strength change can alter downstream behavior, the fingerprint must detect it either directly in that product or through a guaranteed dependent product.

### 4.1 Fingerprint domains

Implementations must distinguish the semantic questions represented by fingerprints:

```text
InputFingerprint
    whether the semantic inputs consumed by a product are equivalent

ProductFingerprint
    whether the published semantic meaning of the product is equivalent

PresentationFingerprint
    whether independently observable source-position or presentation data is equivalent
```

Names and physical representations may differ. One fingerprint may encode several domains only when its equivalence relation remains explicit and cannot cause semantic reuse from presentation-only equality.

### 4.2 Computation and publication effects

These terms are distinct:

| Term | Meaning |
|---|---|
| `Recomputed` | computation executed |
| `Changed` | resulting semantic product differs |
| `PresentationChanged` | positional/presentation product differs while semantic product may remain equal |
| `Invalidated` | prior product is no longer reusable |
| `Published` | product belongs to committed snapshot |

Therefore:

```text
source edited        != formal product changed
product recomputed   != product changed
range-only edit      may mean presentation changed without semantic changed
candidate computed   != candidate published
```

Counters and update reports must not use one of these terms as an undocumented approximation for another.

---

## 5. Epistemic changes are semantic changes

These states are not equivalent:

```text
Established(Int)
Assumed(Int)
```

even though the `TypeId` is identical.

Likewise:

```text
Ready
Suppressed
```

are not equivalent.

A flow-summary/product fingerprint that hashes only:

```text
BindingId -> TypeId
```

cannot distinguish those states and is insufficient if downstream logic consumes epistemic strength or status.

The same principle applies to:

```text
Unknown(reason A) vs Unknown(reason B)
Dynamic vs Unknown
Validated vs Refuted binding consistency
explicit vs inferred contract when presentation/semantics distinguish them
```

---

## 6. Causal shape versus cause allocator identity

Raw `DiagnosticCauseId` is not semantic identity.

For product fingerprints, causal state should be represented by semantic shape:

```text
Clean
One
Multiple
```

rather than hashing the raw local cause number.

Likewise analysis status:

```text
Invalid(_)
Suppressed(One(_))
```

hashes the relevant semantic status/cause shape, not the allocator integer.

This allows equivalent analyses with different local allocation order to reuse semantic products.

---

## 7. Diagnostic semantic identity

Diagnostics can contain both semantic content and local linkage.

Semantic content includes:

```text
diagnostic code
relevant type/relation operands
semantic severity/category
structural cause class
```

Local linkage includes:

```text
root cause allocator ID
presentation range
```

Whether a range participates in a *payload/input* fingerprint depends on the product boundary. It should not automatically participate in the *semantic product* fingerprint if a range-only edit should permit semantic reuse.

Similarly, raw root cause ID must not participate in semantic identity.

---

## 8. Range-only edits

A source edit that moves an expression without changing its semantics may require recomputing the source-rich payload so ranges are current.

But the resulting semantic product fingerprint may remain unchanged.

Conceptually:

```text
input fingerprint changes
        ↓
product recomputed with new ranges
        ↓
semantic fingerprint unchanged
        ↓
semantic dependents reuse
```

This distinction is essential for responsive editing.

---

## 9. Dependency ownership

Every semantic product consumed by another computation must be represented by a dependency edge or equivalent tracked relationship.

Examples:

```text
callable body
    -> callable signature
    -> declaration surface
    -> generic parameter/kind products
    -> resolved dispatch target
```

If a body uses generic constraints from the signature, its dependency graph must invalidate when those constraints change.

Recording a dependency on an unrelated broad product while reading a separately built semantic value is not sufficient.

---

## 10. Product reuse law

A cached product is reusable when:

1. its direct semantic inputs remain equivalent according to their input/product contracts;
2. every recorded semantic dependency still has an equivalent semantic product fingerprint;
3. the product's own generation/lifecycle context is valid.

Reuse is not permitted merely because a source file timestamp or query key is unchanged.

---

## 11. Flow product identity

Flow state is especially sensitive because type identity alone is not enough.

If downstream analysis may branch on:

```text
Established vs Assumed
Unknown vs Dynamic
contract type/origin
consistency
mutability
causal shape
denotation
```

then the flow semantic summary used for dependency comparison must preserve these distinctions.

It may do so with a dedicated compact summary rather than hashing the entire mutable flow object.

---

## 12. Explanation and provenance identity

Not every explanation detail must invalidate semantic dependents.

The product contract should distinguish:

```text
semantic evidence origin/status
```

from:

```text
presentation-only prose/range
```

A change from `ConstructorSemantics` to `CallableSignature` may be semantically observable if explanations or evidence-consuming compiler logic can distinguish them. The product boundary must explicitly decide and test this.

A source-span move alone should usually not alter semantic identity.

---

## 13. Formal versus advisory products

Formal and advisory products have separate semantic identities.

Advisory recomputation must not invalidate formal products merely because an advisory observation changed.

Formal semantic changes may invalidate or require reprojection of advisory products that consume formal context.

Direction:

```text
formal -> advisory
```

not:

```text
advisory -> formal
```

---

## 14. Snapshot publication

A semantic snapshot is a coherent read model over products from one validated semantic revision/generation.

Consumers must not observe an arbitrary mixture of:

```text
new binding flow
old callable signature
new diagnostic index
old source-site mapping
```

if those products are semantically incompatible.

Workspace transactions and publication mechanics must satisfy chapter 12. Consumer pinning and request consistency must satisfy chapter 13.

---

## 15. Structural sharing

Semantic snapshots and products should reuse immutable data when semantics are unchanged.

Correct structural sharing reduces memory and latency.

However, sharing is only valid after semantic equivalence has been established. It must not become a shortcut around dependency validation.

---

## 16. False cache hits versus false cache misses

A **false cache hit** occurs when the analyzer reuses a product whose semantics changed.

This is a correctness defect.

A **false cache miss** occurs when the analyzer recomputes despite semantic equivalence.

This is primarily a performance defect.

The fingerprint design should prioritize eliminating false hits while reducing false misses where possible.

---

## 17. Canonical equivalence matrix

The following should be treated as baseline semantic expectations:

| Change | Semantic product identity |
|---|---|
| `Int` -> `String` | changes |
| `Established(Int)` -> `Assumed(Int)` | changes |
| `Unknown(MissingInitializer)` -> `Unknown(InferenceConflict)` | changes when reason is observable/relevant |
| `Unknown` -> `Dynamic` | changes |
| `Ready` -> `Suppressed` | changes |
| `Ready` -> `Invalid` | changes |
| contract `Int` -> `Number` | changes |
| `Validated` -> `Refuted` | changes |
| mutable -> immutable | changes |
| resolved callable A -> B | changes |
| generic constraint set changes | changes |
| causal `Clean` -> `One(_)` | changes |
| `One(C17)` -> `One(C18)` with identical semantic cause | does not change solely for ID renumbering |
| source range shifts only | semantic identity normally unchanged |
| internal hash/map iteration order changes | unchanged |
| advisory observation changes | formal product unchanged |

Product-specific specifications may refine this table.

---

## 18. Determinism

Semantic fingerprints and product equality must not depend on nondeterministic map iteration or allocation order.

Collections included in semantic identity should be:

- canonicalized;
- sorted deterministically;
- stored in deterministic structures;
- or hashed in an order-independent but collision-safe semantic scheme.

This applies especially to:

```text
union members
dependency sets
constraint summaries
diagnostic aggregates
flow maps
```

---

## 19. Cold versus incremental equivalence

Given the same source/project state, a cold analysis and an incremental analysis must publish semantically equivalent products.

Differences in:

```text
allocator IDs
arena indices
cache hit counts
internal revision numbers
```

are permitted only when they are not part of external semantic behavior.

This should be tested by comparing normalized semantic products.

---

## 20. Removal and invalidation

When a declaration, callable, module, or source product disappears, dependents must not continue to observe the removed product through stale cache entries.

Ordinary recomputation should preserve incoming dependency relationships where the database architecture requires them; destructive invalidation should be reserved for actual disappearance or lifecycle reset as specified by the DB architecture.

---

## 21. Performance consequences

Precise semantic products enable selective recomputation:

```text
body-only edit
    -> body payload recomputes
    -> unchanged signature/declaration dependents reuse

range-only edit
    -> source-rich payload refreshes
    -> semantic dependents reuse

signature edit
    -> signature fingerprint changes
    -> callers/body dependencies recompute as required
```

Overly broad fingerprints cause expensive false misses.

Lossy fingerprints cause dangerous false hits.

The goal is semantic precision, not simply hashing less data.

---

## 22. External behavior guarantees

Consumers may rely on:

- cold and incremental semantic equivalence;
- evidence/status changes invalidating dependent semantic products;
- cause-number renumbering alone not invalidating semantics;
- range-only changes not unnecessarily invalidating semantic dependents;
- formal/advisory identity remaining separate;
- resolved call/contract/generic dependency changes invalidating affected products;
- snapshot publication being coherent.

---

## 23. Required regression families

### Fingerprint unit tests

- evidence status change;
- evidence origin change where semantically observable;
- contract type/origin change;
- binding consistency change;
- mutability change;
- causal shape change;
- raw cause renumbering;
- analysis status change;
- resolved callable change;
- flow epistemic-state change.

### Product stability

- range-only edit preserves semantic fingerprint;
- body implementation edit that does not change semantic product preserves dependent reuse;
- signature change invalidates callers;
- generic constraint/kind change invalidates affected inference/body products;
- unrelated declaration change does not invalidate independent callable bodies.

### Differential

- cold analysis versus incremental analysis after edit sequence produces normalized equivalent semantic products.

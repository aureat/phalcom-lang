---
plan: LANG005.C4.P1
checkpoint: LANG005.C4
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
next_plan: LANG005.C4.P2
---

# Walkthrough — LANG005.C4.P1 Explicit Conformance Declarations, Ownership, and Coherence

## Result

P1 is complete at `IMPLEMENTED` + `FOCUSED_TESTED`. The language now retains
explicit `impl TraitRef for Target` declarations as a distinct semantic source
product. Canonical target/TraitRef heads, trait-or-target ownership, workspace
publication, exact generic matching, global overlap rejection, incremental
replacement/removal, and source identity are established. P1 does not claim
that any target satisfies a trait.

## Source to query path

1. `ImplKind::{Inherent, Conformance}` in the shared AST preserves both heads,
   the `for` range, generic binders, where syntax, body, and source ranges.
2. `resolve_conformance_head` allocates impl-owned parameters under
   `TypeParameterOwner::Impl(ImplId)`, resolves the left head through
   `TraitHeaderTable`, resolves nominal or exact enum-case targets, and emits
   diagnostics for invalid or unsupported declarations.
3. `ConformanceTarget` preserves declaration ownership or exact `VariantId`
   identity. `conformance_is_authorized` accepts only the canonical trait-owner
   or target-owner module; linked re-exports do not transfer ownership.
4. `ConformanceContribution` retains source `ImplId`, exact source module,
   canonical TraitRef, target head, generic signature, eligibility, and
   diagnostics. Unauthorized or invalid products remain explainable but are
   excluded from lookup.
5. `ConformanceIndex` is snapshot-owned and globally composed. `query_exact`
   returns `ConformanceHeadMatch` with exact target/TraitRef and impl bindings;
   it does not choose among multiple candidates.
6. `overlap_conflicts` jointly unifies target and TraitRef heads. Generic and
   specialized overlaps are diagnosed with no specificity or source-order
   precedence.
7. Conformance members are excluded from inherent surfaces, callable
   definitions, semantic-shard callable fingerprints, and target-owned
   callable identities. Source indexing still exposes trait/target declaration
   references, including exact-case `VariantId` reverse references.

## P2 stable inputs

P2 should consume:

```text
SemanticSnapshot::conformance_index
ConformanceIndex::query_exact
ConformanceHeadMatch::{impl_id, exact_target, exact_trait_ref, impl_bindings}
ConformanceContribution source/provenance and generic metadata
TraitSurface and effective inherent behavior products
```

P2 must add witness/default/completeness products without moving conformance
members into inherent surfaces or renaming a head match as conformance
evidence.

## Focused evidence

| Surface | Result |
|---|---:|
| AST impl syntax and inherent regressions | 12 passed |
| semantic source-index integration | 22 passed |
| semantic impl/conformance lane | 55 passed |
| `cargo check -p phalcom-semantic` | passed |
| `cargo fmt --all -- --check` | passed |
| `git diff --check` | passed |

The semantic lane includes generic/specialized matching, exact TraitRef
domains, exact enum cases, linked-module ownership/re-export behavior,
add/edit/delete lifecycle, cold/incremental parity, coherence overlap, and
the P1 adversarial matrix.

## Boundary

No witness map, default selection, completeness proof, associated type,
conditional conformance, trait-object representation, runtime registry, or
trait-evidenced dispatch was implemented.

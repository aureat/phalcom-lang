---
program: LANG005
checkpoint: LANG005.C4
plan: LANG005.C4.P1
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
next_plan: LANG005.C4.P2
worktree: dirty; preserve unrelated working-tree changes
---

# Handoff — LANG005.C4.P1 to LANG005.C4.P2

## Entry condition

P1 is complete at `IMPLEMENTED` + `FOCUSED_TESTED`. C4 remains in progress.
P2 can begin witness resolution against the stable conformance identity,
ownership, publication, and coherence products below.

## Stable interfaces

| Contract | Canonical owner and boundary |
|---|---|
| Explicit source form | `ImplKind::Conformance { trait_ref, for_range }` plus `ImplDef.target` |
| Source identity | `ImplId { module, local }`; no `ConformanceId` was introduced |
| Trait head | `TraitRef { declaration, arguments }` resolved through C3 trait headers |
| Target identity | `ConformanceTarget::Declaration(DeclarationId)` or `ExactEnumCase(VariantId)` |
| Resolved head | `ResolvedConformanceHead` with impl generics, exact target head, source, eligibility, diagnostics |
| Ownership | `conformance_is_authorized`: canonical trait owner OR canonical target owner |
| Published source product | `ConformanceContribution`, retained in `SemanticSnapshot::conformance_index` |
| Exact query | `ConformanceIndex::query_exact(&mut TypeStore, target, &TraitRef)` |
| Query result | `ConformanceHeadMatch` with source `ImplId`, exact relation, and impl bindings |
| Coherence | `ConformanceIndex::overlap_conflicts`; overlap is joint target + TraitRef and has no precedence |
| Trait contract | C3 `SemanticSnapshot::trait_headers` / `trait_surfaces` |
| Inherent behavior | Existing declaration/effective surfaces; P1 never injects conformance members |
| Source tooling | `SourceIndexContext` canonical type targets and source occurrence reverse references |

## P2 constraints

- A head match is not requirement satisfaction and must not be exposed as
  `Conforms`, `Satisfied`, or `ConformanceEvidence`.
- Retrieve the source AST/body through the `ImplId` and module source; the
  P1 index intentionally stores source provenance and head identity, not
  witness selection.
- Conformance-local witness bodies are not target-owned inherent callables.
- Preserve `TraitSurface` as the sole requirement/default contract authority.
- Reuse effective inherent behavior and existing relation/compatibility
  authority; do not create a second type or trait solver.
- Keep unauthorized, unresolved, structurally illegal, and unsupported-
  condition contributions diagnostic-only for lookup.

## Inherited verification oracle

The focused semantic suite contains 55 passing `impls` tests. Particularly
relevant P2 handoff tests are:

```text
explicit_conformance_is_indexed_without_polluting_inherent_surface
generic_conformance_head_matches_exact_target_specialization
linked_modules_publish_canonical_ownership_through_reexports
exact_enum_case_conformance_retains_variant_identity
p1_generic_specialization_matrix_and_iterable_head_stay_at_head_level
incremental_and_cold_conformance_publication_have_the_same_head_identity
overlapping_generic_and_specialized_conformances_are_rejected
```

The full workspace/release gates were not run; they are not required to claim
P1 focused completion and remain separate certification work.

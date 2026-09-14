---
program: LANG005
checkpoint: LANG005.C4
plan: LANG005.C4.P2
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
next_plan: LANG005.C4.P3
worktree: dirty; preserve unrelated working-tree changes
---

# Handoff — LANG005.C4.P2

## Stable interfaces

| Contract | Canonical owner and boundary |
|---|---|
| Witness identity | `CallableOwnerId::Conformance(ImplId)` |
| Source plan | `ConformanceWitnessPlan`, keyed by source `ImplId` in `SemanticSnapshot` |
| Selection variants | `ConformanceCallable`, `InherentCallable` (with optional `conditional_impl` and canonical applicability evidence), `DataComponent`, `TraitDefault` |
| Completeness | `ConformanceCompleteness`; only `Complete` can produce evidence |
| Exact evidence | `SemanticSnapshot::resolve_conformance_evidence` / `conformance_evidence_for` |
| Exact matching | Existing `ConformanceIndex::query_exact`; multiple matches remain a coherence conflict |
| Receiver context | Witness body analysis uses `CallableBodyQuery::self_type_override` and does not grant target-private lexical access |
| Requirement view | `TraitSurface::instantiate` / `InstantiatedTraitRequirement`, specialized through `TypeEnvironment` and `TypeView` |
| Compatibility | `check_witness_compatibility_with_visibility` and `check_data_component_compatibility`, returning `WitnessCompatibility` |
| Effective inherent lookup | `resolve_effective_inherent_witness`, layered over ordinary dispatch and C2 conditional-member products |
| Evidence identity | `ConformanceWitnessPlan::fingerprint` and `ConformanceEvidence::fingerprint` |

## Invariants to preserve

- Conformance-local callables have no declaration owner and must not be passed
  to target-owned surface or dispatch mutation APIs.
- A bodyless, duplicate, or unmatched explicit member cannot fall through to
  an inherent member or trait default.
- A source generic conformance owns one symbolic plan; exact applications bind
  its `ImplId` environment and do not create new source witness identities.
- Trait defaults remain trait-owned and are checked once by C3.
- `DataComponentId` is retained directly; no synthetic getter callable is
  allowed.
- Trait parameter, target declaration parameter, and `Self` specialization
  use one environment/materialization path; callable-local generic identity
  is not rewritten.
- Compatibility uses the canonical bounded relation and never turns a
  blocked, dynamic, cancelled, budget, or internal result into a proof.
- Conditional applicability outcomes retain their terminal state; a proven
  data or default fallback may still satisfy the same requirement.
- Exact evidence specializes retained `InherentImplSpecialization` products
  through the exact conformance environment rather than re-solving C2.
- Generic callable constraints are checked in the obligation direction after
  alpha-renaming; candidate constraints must be implied by requirement
  constraints.
- P2 evidence is semantic proof only. P3 owns ordinary availability,
  callable-reference selection, lowering, and execution.

## Remaining implementation tasks

None for P2. P3 may now consume the completed semantic evidence products.

P3 may now begin dispatch, lowering, and runtime integration from the
completed P2 evidence products.

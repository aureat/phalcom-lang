---
id: LANG005.C5.P2
category: LANG
program: LANG005
checkpoint: LANG005.C5
kind: implementation-handoff
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
plan: LANG005.C5.P2
prepared: 2026-09-16
repository: aureat/phalcom-lang
starting_revision: 3a2dcbd49fa7548337b52e9562cab4f8582741e4
---

# LANG005.C5.P2 Handoff

## 1. State anchor

P2 is complete in the shared working tree on top of pushed baseline
`3a2dcbd49fa7548337b52e9562cab4f8582741e4`. The current tree intentionally
contains the P2 source, tests, and records; preserve unrelated work when
delivering it.

## 2. Completed prerequisites

P1 associated declaration/binding identity, exact evidence, unified
completeness, property requirements, and direct-field `via` products are
frozen. P2 T1–T10 are complete. The checkpoint records the focused gates and
the adviser decision requiring T5 to call the central normalizer.

## 3. Stable takeover map

- `TypeData::AssociatedProjection` / `TypeStore::associated_projection`:
  canonical type carrier.
- `TypeAnnotationExpr::AssociatedTypeProjection`: ratified initial syntax,
  currently contextual `Self::Item` only.
- `types::ProjectionNormalizationContext`, `ProjectionNormalizationMode`,
  `ProjectionNormalizationResult`, and `types::normalize_type`: sole
  normalization authority and terminal-state algebra.
- `ConformanceAssociatedTypePlan`: source binding-plan authority; T5 adapts
  it into the central normalizer and retains provenance/diagnostics.
- `ConformanceEvidence::associated_types` and `requirement_views`: exact
  normalized values/contracts.
- `SourceIndexContext::associated_projection_targets` plus the semantic-session
  occurrence table: source range to canonical associated requirement bridge.
- `SemanticTargetId::AssociatedType`: declaration, binding, and projection
  navigation target.

## 4. Invariants not to redesign

- Never identify a projection by `subject + string name` or source range.
- Never add generic `T::Item` assumption semantics or `<T as Trait>::Item`
  syntax in C5.P3.
- Never duplicate recursive normalization or cycle detection in T5, LSP,
  compiler, or runtime.
- Source mode must not request final evidence; exact mode must consume exact
  evidence; non-success states must remain honest.
- Structural type consumers rewrite projection constituents but do not solve
  conformance.
- Runtime and compiler remain consumers of semantic decisions.

## 5. Known drift and deferred failures

- The plan's private helper names drifted mechanically; the live implementation
  uses `types/projection.rs` and the existing session/source-index seams.
- `C5-BL-07`: two `capabilities::traits` Universe `Bool` dependency/capability
  failures remain, classification D.
- Workspace/release formatting, Clippy, all-targets tests, full Iterable
  migration/corpus, and broader C5 certification are not evidence from P2.

## 6. Must-read files

Read these before P3 changes:

```text
docs/implementation/LANG005/LANG005.C5/LANG005.C5-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5-GUIDANCE.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P2-associated-projection-formation-normalization-and-evidence-hardening-plan.md
docs/implementation/LANG005/LANG005.C5/LANG005.C5.P2-walkthrough.md
docs/specs/objects/traits.md
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/source_index/
```

## 7. Next plan and first checks

P3 owns Iterable migration, cross-stack compiler/runtime consequences of
already-normalized facts, editor/LSP polish, stress/negative coverage, and
broader certification. Begin with a fresh status/diff check, then the P3 plan's
T0 and the focused P2 gates only when needed for a changed seam.

## 8. Do not re-explore / do not redesign

Do not rediscover the P2 carrier, source plan, evidence map, or normalizer.
Do not repair the known Universe `Bool` baseline unless a new P3-specific
reproducer changes its classification. Do not move projection solving into a
consumer, introduce a runtime lookup protocol, or widen syntax without an
explicit language-design consultation.

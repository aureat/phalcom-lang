---
id: LANG005.C6.P1
category: LANG
program: LANG005
checkpoint: LANG005.C6
plan: LANG005.C6.P1
kind: handoff
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
prepared: 2026-09-16
repository: aureat/phalcom-lang
---

# LANG005.C6.P1 — Handoff to P2

## Authoritative documents

Read these before continuing:

- [C6 checkpoint](LANG005.C6-CHECKPOINT.md)
- [C6 guidance](LANG005.C6-GUIDANCE.md)
- [P1 plan](LANG005.C6.P1-canonical-generic-conformance-evidence-and-runtime-transport-plan.md)
- [P1 T5–T9 directions](LANG005.C6.P1.T5-T9-DIRECTIONS.md)
- [C6.P1 clarifications](LANG005.C6.P1-CLARIFICATIONS.md)
- [P1 walkthrough](LANG005.C6.P1-walkthrough.md)
- [Luna implementer prompt](../../../workflow/luna-implementer-prompt.md)
- [implementation-record lifecycle](../../../workflow/implementation-record-lifecycle-convention.md)
- [Luna and premium adviser prompts](../../../workflow/luna-and-premium-adviser-prompts.md)
- [shared consultation escalation protocol](../../../workflow/shared-consultation-escalation-protocol.md)
- [commit and push discipline](../../../workflow/commit-and-push-discipline.md)
- [final response format](../../../workflow/final-response-format.md)
- [C5.P3 predecessor handoff](../LANG005.C5/LANG005.C5.P3-handoff.md)

The referenced adviser task is [LANG005 Advisor](thread://01a0a645-6654-7b20-99dc-973a16164e05?hostId=local).

## Stable interfaces to reuse

- `GenericConstraint::Conforms` is the semantic direct-bound proposition.
- `SemanticDependency::TraitHeader` and its `QueryKey` mapping/currentness
  path are required for trait-reference formation and incremental replay.
- `TrackingTypeResolver::resolve_trait_name` records the trait-header lane;
  do not restore ordinary declaration-shell tracking for bound traits.
- `GenericEvidenceEnvironment` and semantic specialization evidence remain
  authoritative for proof/discharge.
- `StableTraitApplicationRef` and the exported `Conforms` metadata shape are
  the stable public representation.
- Core consumes semantic generic type/conformance recipes and the VM executes
  selected `InvokeTraitRequirement` targets through the runtime conformance
  environment. Do not add runtime conformance search.

Key source/test locations:

- `phalcom-semantic/src/generic_evidence.rs`
- `phalcom-semantic/src/checker/analysis.rs`
- `phalcom-semantic/src/checker/context.rs`
- `phalcom-semantic/src/db/query.rs`
- `phalcom-semantic/src/metadata/export.rs`
- `phalcom-semantic/src/source_index/builder.rs`
- `phalcom-type-meta/src/generic.rs`
- `phalcom-type-meta/src/validate.rs`
- `phalcom-core/src/modules/compile.rs`
- `phalcom-core/src/typing/inspect.rs`
- `phalcom-core/src/primitive/typing.rs`
- `phalcom-semantic/tests/semantic/capabilities/generic_conformance.rs`
- `phalcom-core/tests/core/language/generic_conformance.rs`
- `phalcom-core/tests/core/reflection/reflection.rs`

## Verification inherited

P1 focused acceptance is green:

- semantic generic-conformance 11/11;
- semantic constraints 3/3;
- semantic fingerprints 33/33;
- core generic-conformance 4/4;
- direct-conformance reflection 1/1;
- type-meta 5/5;
- associated-type LSP regression 1/1 and full LSP package green.

One broad recomputation test remains an inherited Universe baseline involving
retained surfaces/callable-signature presentation parity. It is recorded as
`BASELINE_BLOCKED`, not as a P1-owned defect. No P1 A/B blocker remains.

## P2 starting point

Begin `LANG005.C6.P2` from its own authoritative plan. Extend the P1 evidence
and normalization authority for generic associated projections and equality;
do not duplicate proof, projection normalization, identity, or runtime search
in a consumer layer. Keep the checkpoint lifecycle at
`IN_PROGRESS / PARTIAL`, with P1 recorded as
`COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.

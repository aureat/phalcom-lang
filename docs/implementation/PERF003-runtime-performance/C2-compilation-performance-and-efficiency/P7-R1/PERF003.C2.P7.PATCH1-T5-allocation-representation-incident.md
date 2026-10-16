# IMPLEMENTATION INCIDENT — T5 Conformance Evidence Allocation Representation

## 1. Identity

- Program: PERF003
- Checkpoint: PERF003.C2
- Plan: PERF003.C2.P7.PATCH1
- Task/Gate: T5 — generic call specialization evidence; preparation for T8 exact owner allocation
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `perf/universe-characterization`
- Starting revision: `5b4518073b46d7305ffa27e69e928a30ec7630e7`
- Current revision: `5b4518073b46d7305ffa27e69e928a30ec7630e7` (implementation remains uncommitted at packet creation)

## 2. Escalation trigger

PATCH1 plan §13 triggers 7, 16, and 18 apply:

7. A required map-backed field cannot be migrated to charged immutable storage without changing shared semantic representation.
16. Exact whole-callable allocation cannot be proven by guessing native map-node sizes; the plan explicitly forbids that.
18. The required migration crosses generic conformance, trait/witness products, and their consumers.

This also meets the shared consultation protocol's RED triggers for representation change and materially different architectural consequences.

## 3. Required invariant

- Preserve the architect-approved callable-local Exact/Assumed/Derived/CompilerDerived graph and schema 2.
- Do not introduce a global evidence owner or semantic re-query during import.
- Do not import into shadow `BTreeMap`/`HashMap` copies, estimate tree-node allocation sizes, or remove the global `UnaccountedBoundary` guards.
- PATCH1's callable completeness guards remain until GE-1..GE-12 and owner-parity pass.

## 4. Expected architecture

The approved 2026-10-16 PATCH1 decision requires complete Exact and Derived payloads to be reconstructed from the callable-local graph, with allocation-budgeted immutable owners. The active plan's T1 establishes charged frozen storage/read APIs; T5 preserves complete conformance selections and nested evidence; T8 requires exact cumulative accounting.

## 5. Observed behavior

Wire/export now represents all four evidence node variants and preserves ordered call conformance bindings. Import currently reconstructs Forwarded and CompilerDerived bindings, including relocated keys and classification. Exact and Derived import remain typed fail-closed errors.

The shared native types needed for complete Exact/Derived reconstruction own ordered maps as ordinary `BTreeMap`s. The current `AllocationBudget` supports exact contiguous buffers, `Arc`, and `BudgetedSortedMap`; it has no allocator-aware `BTreeMap` constructor. Reconstructing those native maps therefore creates allocations that cannot be charged exactly through the existing API.

## 6. Minimal reproduction

Relevant focused importer command:

```sh
RUSTUP_TOOLCHAIN=nightly-2026-07-10 RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib typed_call_specialization_arguments_survive_relocation -- --test-threads=1
```

Observed: 1 passed for Forwarded and CompilerDerived relocation, including a wrong-node-kind rejection. Exact and Derived remain rejected by design pending this decision.

Expected: complete Exact/Derived import without an unaccounted allocation or an estimated B-tree-node debit.

## 7. Relevant code path

```text
CallableAnalysisRecordV1.conformance_evidence
  -> import_callable_call_specializations
  -> import_callable_generic_call_conformance_binding
  -> import_callable_conformance_evidence_node
  -> native GenericConformanceEvidence
```

Relevant native fields:

- `impls.rs::ConformanceEvidence`: `requirement_views`, `requirements`, `associated_types`, and optional `trait_header_conformance_proofs` are `BTreeMap`-backed.
- `impls.rs::ConformanceAssociatedTypePlan::bindings` is a `BTreeMap`.
- `derived_conformance.rs::DerivedConformanceRule`: `requirement_views` and `requirements` are `BTreeMap`-backed.
- `phalcom-type-meta::allocation::AllocationBudget` exposes `BudgetedSortedMap`, but these native field types do not accept it.
- `SemanticWorldV2::validate_with_allocation_budget` and semantic import retain their fail-closed allocation guards.

## 8. Scoped implementation diff

- Added schema-2 callable-local conformance DTOs and bounded graph-reference validation.
- Added deterministic exporter traversal for ordered top-level bindings and Exact/Assumed/Derived/CompilerDerived payloads.
- Added importer support for Forwarded and CompilerDerived, with charged `Arc` construction, relocated type/trait keys, and source-kind checking.
- Kept Exact/Derived importer failure explicit; no guard was weakened.
- Updated the checkpoint and PATCH1 decision amendment for the approved owner graph.

## 9. Evidence

- `cargo check -p phalcom-semantic` completed successfully; it reported dead-code warnings because the callable-analysis importer is not yet connected to owner reconstruction.
- Semantic exporter discriminator: 1 passed.
- Semantic importer relocation discriminator: 1 passed.
- Compiler-artifact conformance-kind/cycle validator discriminator: 1 passed.
- `rustfmt --check` and `git diff --check` passed before this incident packet was prepared.
- Live type definitions directly show the map-backed fields listed above.
- `AllocationBudget` directly provides `sorted_map_builder`/`BudgetedSortedMap`, but no `BTreeMap` allocation path.
- No full owner reconstruction, exact allocation N/N−1/rollback, fresh-process/source-denied, determinism, or P5/P6 gate has run.

## 10. Implementer's causal hypothesis

The approved evidence graph is transportable, but its complete native Exact/Derived products cannot currently be reconstructed under PATCH1's exact allocation contract while retaining the existing shared `BTreeMap` field types. A source/frozen ordered-map abstraction could allow source analysis to keep its current construction behavior while artifact import constructs frozen owners from charged sorted buffers.

## 11. Alternative hypotheses

1. The shared semantic products should gain source/frozen map wrappers, with a common read API, and PATCH1 should migrate only the evidence-related maps required for callable reconstruction.
2. The owner graph and export path are in PATCH1 scope, but frozen storage for shared conformance products should be a separately planned prerequisite; Exact/Derived import remains fail-closed until that prerequisite lands.

Pre-charging estimated B-tree node layouts is excluded by the active plan. Removing or bypassing the global allocation guards is also excluded.

## 12. Attempts already made

- Reconstructed the direct Forwarded and CompilerDerived variants from portable keys; the focused fresh-relocation discriminator passed.
- Charged the CompilerDerived `Arc` through `AllocationBudget::arc` and added a wrong-node-kind discriminator; the focused test passed.
- Did not attempt Exact/Derived `BTreeMap` materialization because it would violate the allocation invariant rather than test a valid implementation hypothesis.

## 13. Why local authority is insufficient

Choosing source/frozen map wrappers changes shared native product field types and reader APIs used by trait dispatch, projection, associated-type normalization, fingerprints, and witness/conformance construction. Deciding which products and fields participate—and whether that migration belongs to PATCH1 or a separately numbered prerequisite—is a representation/scope decision, not a local codec detail.

## 14. Decision requested

Should PATCH1 introduce source/frozen budgeted ordered-map wrappers for the shared Exact/Derived evidence products and migrate the relevant readers, or should PATCH1 retain fail-closed Exact/Derived import and split that representation migration into a separately numbered prerequisite under PERF003.C2.P7? If the wrapper path is approved, identify its intended scope across `ConformanceEvidence`, `ConformanceAssociatedTypePlan`, `DerivedConformanceRule`, and any environment/proof maps, plus the required source/frozen equality and API invariants.

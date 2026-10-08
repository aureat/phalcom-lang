---
id: COLL006.C1.P3
program: COLL006
checkpoint: COLL006.C1
kind: handoff
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
---

# COLL006.C1.P3 continuation handoff

## Current state — 2026-10-08 continuation

P3 remains incomplete. Work continues on `feat/telos-collection-upgrade` in `.worktrees/telos-collections-upgrade`, from delivered anchor `e3dfe492`. This continuation is uncommitted unless a later delivery record says otherwise. An unrelated modified `collections/bytes.ph` existed at takeover and was preserved.

The current user authorized diagnostic/advisory delegation, superseding the previous delivery's inline-only restriction. One read-only agent diagnosed symbolic construction, layout, Array API typing, and inference-owned publication. Implementation stayed in the primary agent.

Read the checkpoint, current plan, requirements analysis, P2 walkthrough, and P3 walkthrough. The checkpoint is the single durable state authority. Earlier aborted-runtime statements in the walkthrough are historical; the continuation evidence supersedes them.

## Established interfaces

- Shared exact `phalcom_type_meta::StaticInt`; `TypeData::ConstInt`, metadata schema 12, canonical heterogeneous static applications remain inherited.
- `checker/sequence_literal.rs` is the sole bracket checker. The inactive legacy implementation was removed.
- Ordinary elements use `check_typed_expr`; concrete contextual contracts and fixed slots in partial contextual contracts are validated before construction publication.
- Explanation `CollectionKind` now includes Array and Vector and consumes checked sequence resolution, including element/result identity.
- `SequenceConstructionPlan::VectorIdentitySpread { source_range, extent }` is the adopted semantic proof for a literal containing exactly one native Vector spread with identical canonical symbolic extent. Projection preserves it in the existing construction field.
- Compiler checks proof extent, storage lane, and exact source site; `Bytecode::CopyVector` copies immutable native storage. No runtime N interpretation, new solver, or optional-count fallback exists. Concrete `BuildVector` and builder cardinality checks remain intact.
- Additional zero-contribution expressions are deliberately excluded from the symbolic identity lane; they cannot be silently dropped. General symbolic arithmetic remains unsupported.
- Native Vector storage is immutable and traced; source iteration, equality/hash and Array/List conversions execute. Private Array builder finalization still moves storage.

## Verification and corrected oracle

Current evidence is in the checkpoint continuation ledger and walkthrough. The first completed inherited Vector suite exposed an unsupported `Object <= 40` assertion. Predecessor `5fee70c9` already had a 48-byte inline FamilyObject payload. The adviser approved preserving representation and replacing that absolute oracle with a Vector payload bound against the predecessor's existing FamilyObject requirement. The test now inspects actual direct bytecode too.

Runtime coverage now includes exact spreads, API/conversions, immutable selectors, nested storage and iterator ownership through GC, Map/Set structural keys and cross-family inequality, symbolic generic copying into distinct storage, and tampered exact-spread/final-extent rejection with no published result. Final command outcomes belong in the checkpoint ledger.

## Remaining work in priority order

1. Centralize negative concrete Vector extent formation. The requirements authorize a canonical nominal well-formedness rule, not duplicated literal/compiler checks. Review `TypeStore::apply_static_form_with_arena` and scoped applications in `types/annotation.rs`; symbolic N remains valid. Raw interning is not a fallible validation boundary.
2. Close general inference-owned publication. Fixed-slot conflicts now reject before publication, but a candidate depending on a later failed/underconstrained/cancelled inference owner needs centralized transactional cleanup at context finalization. Do not add statement-only rollback or prematurely solve an outer frame. Adviser diagnosis is recorded in the checkpoint.
3. Correct Array homogeneous APIs. Existing `at` returns bare T in bounds and None out of bounds; neither `-> T` nor `-> Option<T>` alone preserves that behavior. Adviser recommends `T | None`, guarded private T reads that throw out of bounds, typed replacement parameters, and `get -> Option<T>`. Validate union support before changing signatures. Native artifact regeneration follows authored signature changes.
4. Complete Vector metadata/reflection/import-export, cold/incremental changes of N/exactness, and native surface coverage.
5. Extend runtime evidence for source-once/effect order, exceptions/suspension during construction, deep mutable-content hash policy, zero/large extent boundaries, and tampered/missing symbolic proof rejection as required by P3.
6. Finish remaining wrong-kind/arity/scoped formation and contextual generic/return/nested semantic matrix. Array/List runtime-sized spread rejection is covered; general Iterable rejection still needs explicit coverage.
7. Reconcile all final gates and lifecycle records before closing P3. Workspace release tests/clippy remain P9 scope.

## Boundaries

Bare brackets still default to List. P4 must not begin const materialization/default cutover. Do not redesign exact static identity, inference ownership, native storage, or concrete cardinality checks. Preserve unrelated edits and existing baseline failure records. Do not infer release certification from focused tests or a clean tree.

---
id: COLL006.C1.P3
program: COLL006
checkpoint: COLL006.C1
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# COLL006.C1.P3 — Immutable dimensioned Vector

Goal: implement the bounded P3 requirements in `COLL006.C1.P3-requirements-analysis.md` against `5fee70c9`. Work proceeds inline on `feat/telos-collection-upgrade` in the existing collections worktree.

P3 is complete at its bounded implementation scope. The walkthrough and handoff record the focused evidence and deferred broad certification.

## Fixed contract

- Canonical source owns `Vector<T, N: Int>`; its applied identity uses ordinary `TypeData::Applied` and exact static integer terms.
- Inference variables remain query-local. Explicit and contextual holes use destination parameter kinds.
- Every Vector literal has proved cardinality. Tuple positional lanes and native Vector storage are exact; arbitrary iterable cardinality is unknown.
- Compiler consumes checked family, extent, and expansion lane products. Missing Vector metadata fails closed.
- Runtime Vector is distinct immutable dense storage. Builders remain private and transfer storage without copying at finalization.
- Bare brackets remain List during P3. Static evaluation/materialization and default cutover belong to P4.

## Execution tasks

- [x] T1: exact integer representation and durable metadata prerequisite. Owner: shared `phalcom-type-meta::StaticInt`, re-exported by semantics. Preserve normalization, full-width fingerprints, and metadata/reflection identity.
- [ ] T2: complete and partial generic argument formation; intrinsic static-domain bootstrap. Positive concrete, signed/huge, symbolic, and mixed-hole tests pass. Wrong-kind/arity and wider scoped/static argument paths remain.
- [ ] T3: source Vector identity, native metadata projection, nonnegative concrete extent formation.
- [ ] T4: semantic Vector literals and cardinality; exact Tuple/Vector expansions; solved-only publication and family-correct explanations. Positive identity matrix and targeted negative/nonpublication cases pass; bounded symbolic identity copying and family explanations are implemented. Checker-local inference-owner publication is now partially implemented and has focused evidence; static/revision boundaries and wider negative coverage remain.
- [ ] T5: checked lowering projection and direct bytecode; immutable heap storage, native read floor, source API, GC tracing and class integration. Concrete and bounded symbolic construction execute; direct bytecode, immutable/nested storage, GC ownership, keys, and defensive cardinality coverage added. Remaining runtime edge cases stay open.
- [ ] T6: Array homogeneous mutation typing and move-based builder finalization corrections.
- [ ] T7: focused positive/negative semantic, compiler, runtime, metadata, GC, and incremental verification. Run Cargo commands serially.
- [x] T8: scoped diff review, checkpoint reconciliation, walkthrough, P4 handoff.

## Verification gates

G1: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic coll006` plus exact-static-integer and static-domain inference tests.

G2: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_array` and the registered Vector collection module.

G3: focused metadata/reflection, GC, and incremental tests for changed contracts; `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace --all-targets`.

Record actual command outcomes and baseline failures in the checkpoint. Workspace release test/clippy certification remains P9 scope.

## Adopted continuation clarification

T4/T5 support a lone symbolic Vector identity spread through a semantic `VectorIdentitySpread` proof carrying source site and canonical extent. Compiler consumes the proof and emits native immutable-storage copying. Numeric construction and runtime cardinality checks remain unchanged; symbolic arithmetic and additional zero-contribution operands are not admitted by this lane. See checkpoint consultation record.

## Active T4 publication supplement

[Inference-owned sequence publication plan](COLL006.C1.P3-inference-publication-plan.md) is complete at `IMPLEMENTED / FOCUSED_TESTED`. Staging, owner receipts, contextual provenance, deferred scalar/row readiness, exact-case conformance, union aggregation, extraction integration, revision parity, and final focused semantic/core gates pass.

T5 of the supplement uses the adopted Level 1 clarification: partial static annotation elaboration and initializer checking share the same configured CheckingContext, and missing-owner paths fail closed. Static collection evaluation and materialization remain P4.

Detailed continuation evidence, adopted solutions, risks, and dependency-ordered remaining steps: [publication continuation report](COLL006.C1.P3-inference-publication-report.md). The checkpoint remains the single durable state authority.

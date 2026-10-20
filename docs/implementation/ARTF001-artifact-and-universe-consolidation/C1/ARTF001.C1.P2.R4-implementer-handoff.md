# ARTF001.C1.P2.R4 continuation handoff

## Current takeover

- Worktree: `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact`.
- Branch: `feat/artifact-universe-consolidation`.
- Active plan: [`ARTF001.C1.P2.R4-luna-patch-grade-implementation-plan.md`](ARTF001.C1.P2.R4-luna-patch-grade-implementation-plan.md).
- Durable status: [`CHECKPOINT.md`](CHECKPOINT.md). It owns the shared R4 ledger and latest T3 evidence.
- Plan status remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`. R4.T3 is not complete because the prepublication failpoint gate could not compile. R4.T2 and subsequent R4 tasks remain open; P2 and P3 acceptance are not established.
- The user authorized committing and pushing this branch and requested a clean tree. Confirm both the remote branch update and clean worktree before reporting delivery.

## Implemented decisions

1. Runtime dependency obligation identity includes importer, dependency, source range, reason, read target, provenance target, product, and timing. Producer alone does not distinguish an obligation. Wire validation rejects exact duplicates and preserves distinct same-span obligations.
2. QueryImage V4 separates logical query kinds from physical product families. DeclarationShell roots may point to exact nominal-shell or type-alias rows; owner, family, index, and required-root checks are explicit. Package imports sibling `Module`.
3. Imported module graph authority uses a budgeted frozen base with mutable workspace overlays. Source-backed graphs remain mutable.
4. V4 semantic validation uses borrowed current-schema views shared with V3 common validation; production validation does not construct an owned V3 projection. Public budgeted paths remain fail-closed while allocations are unaccounted.
5. Fingerprint validation avoids temporary sorted buffers when imported surface maps are already frozen and ordered.

Read the consultation records linked from the latest `CHECKPOINT.md` sections before changing any of these architecture decisions.

## Latest focused evidence

The saved artifact `/tmp/artf001-universe.telosc` passes:

```sh
PHALCOM_UNIVERSE_TELOSC_FOR_TEST=/tmp/artf001-universe.telosc RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --features allocation-diagnostics --test artifact-allocation -- --nocapture --test-threads=1
```

Current whole-load N is **450,922,288 bytes**, matching independent allocator requests exactly. Named load stages, V4 validation and decode-plus-validation, semantic import, query reconstruction phases and products, publication phases, and QX all match their ledgers. QueryKind accounting covers all 46 kinds; fingerprint aggregation allocates zero bytes.

The same focused gate proves typed cumulative N−1 failure, clean N retry, module-stage child-ledger rollback and retry, plus direct semantic-import N−1 rollback. It also checks Contract B for the session boundary: the immutable image is complete and source-free, and a separately constructed `SemanticWorkspaceSession` retains the image snapshot by `Arc` identity. Session construction allocations are outside the artifact-load budget.

The gate passed after adding the session-boundary assertion (1 test, 15.74 seconds). It does not prove prepublication failpoint behavior or full source-denial/compiler parity.

## Remaining gates and known test blockers

- The four prepublication failpoint test was attempted with `cargo test -p phalcom-core --lib loaded_artifact_prepublication_failures_are_retryable -- --nocapture`. The lib-test target stops at 11 fixture/API compile errors in module materialization, registry, and semantic-lowering tests. T3.6 remains unverified; do not claim failpoint recovery.
- A semantic integration test target also stops during compilation with 451 current API-fixture errors. These fixture failures were not changed as part of the allocation work.
- R4.T2 authority closure, R4.T3 failpoints, T4 required-root census, T5 ordinary session edits/invalidation/rollback, T6 consumer consolidation, T7 fresh-process parity/speedup, and T8/T9 broad closure are still outstanding.
- Workspace/release tests and Clippy were deliberately not run. Do not infer release certification from a clean/pushed branch.

## Resume order

1. Continue from the latest T3 section in `CHECKPOINT.md`; do not repeat the semantic-import consultation or allocation investigation.
2. Resolve or isolate the failpoint test-target compile blocker without changing unrelated fixtures. Then run the four-boundary failpoint/retry gate.
3. Continue the ordered R4.T2/T3 gates and update checkpoint status only at meaningful verification points.
4. Proceed to T4 and later tasks only after their predecessors are accepted. Preserve the distinction between implementation status and verification strength.

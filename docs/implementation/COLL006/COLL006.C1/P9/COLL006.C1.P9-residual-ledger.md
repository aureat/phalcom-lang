---
id: COLL006.C1.P9
program: COLL006
checkpoint: COLL006.C1
kind: residual-ledger
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
---

# COLL006.C1.P9 — Residual Ledger

Statuses use the fixed P9 vocabulary: `RESOLVED`, `EXTERNAL_BASELINE`, `C1_OWNED`, or `AMBIGUOUS`.

| ID | Discriminator / observation | Owner and classification | Next action |
|---|---|---|---|
| `P9-T0-RANGE-NODE-NAME` | `modules_parse_package` first rejected `_RangeMapNode` at bytes 302..315. Renamed to `RangeMapNode`; the focused RangeMap construction/lookup test passes. | C1; **RESOLVED** | Retain the parser-supported private helper name. |
| `P9-T0-RANGE-DUPLICATE-NEW` | Universe bootstrap reported duplicate `RangeMap.new()` from `@constructor new()` and a redundant `@class new()`. Removed the redundant wrapper; the focused RangeMap construction/lookup test passes. | C1; **RESOLVED** | Keep `@constructor` as the canonical construction path. |
| `P9-T0-LISTSOURCE-IMMUTABLE-FIELD` | The corrected feature branch and current main both report `_items` already initialized at `packages/parse/src/source/list-source.ph:33`; the file is outside C1 and unchanged by this task. | Parse package; **EXTERNAL_BASELINE** | Preserve as the current parser-package baseline. |
| `R-PARSE-ASSOCIATED-INVOKE` | `source-test.ph` does not reach the token-backed Cursor `current` / `advance.current` assertion because compilation fails earlier on `ListSource`. Current main has additional metadata-export failures. | P5/compiler ownership remains unresolved; **AMBIGUOUS** | Consume `COLL006.C1.P5-G6B-associated-family-invocation-consultation.md` and isolate the exact expression before classification. |
| `P9-T0-P8-G7-EVIDENCE` | P8 T7/G7 evidence is now present: multi-version GC, endpoint/value reachability, callback allocation/GC and failure atomicity, randomized oracle, and lookup/overlay complexity pass in the RangeMap filter (15/15). | C1; **RESOLVED** | Preserve the exact evidence in the P8 checkpoint and handoff. |
| `P9-T0-P8-SPLICE-ARCHITECTURE` | The prescribed path-copy split/join/range-splice and normalized mutation path are implemented; RangeMap focused suite passes 15/15. | C1; **RESOLVED** | Preserve the architecture and focused evidence; do not redesign in P9. |
| `P9-T0-P8-RESIDUAL-HANDOFF` | P8 handoff and checkpoint now enumerate unresolved T8 integration results: two data-e2e failures, six memory_gc failures, one numeric Range generic-spread failure, and repository-wide formatting diffs. No same-state baseline comparison established ownership. | C1; **AMBIGUOUS** | Keep P8 active; classify or repair under P8 and satisfy G8 before P9 G0. |
| `P8-T8-DATA-E2E` | `language::data_e2e` selected 32 tests; 30 passed and 2 failed: same-spelled module-owner identity reports undefined alias `Factory`; constructor-call collision reports missing canonical runtime callable metadata. No data/semantic source changed in the continuation; branch-baseline ownership is not established. | C1/P8; **AMBIGUOUS** | Run only the precise discriminators after the responsible owner/baseline is identified; do not attribute to RangeMap without evidence. |
| `P8-T8-MEMORY-GC` | `memory_gc` selected 19 tests; 13 passed and 6 failed (five `AssignToImmutable("i")`, one `MissingSequenceLiteralLowering`). No GC/compiler source changed in the continuation; causal ownership is not established. | C1/P8; **AMBIGUOUS** | Preserve as a T8 blocker and classify against an equivalent P8 baseline before assigning an owner. |
| `P8-T8-NUMERIC-RANGE` | `generic_spread_accepts_range_user_iterable_and_bounded_pipeline` fails with `constructor.semantic_product_missing` for `CopyRange`; the same failure persists after T6. Numeric Range and outgoing-pack code are unchanged in the continuation. | C1/P8; **AMBIGUOUS** | Preserve the exact discriminator and classify against an equivalent baseline before assigning an owner. |
| `P8-T8-FORMAT` | `cargo fmt --all -- --check` reports 566 diffs across the repository, including untouched files. The task-owned `range_map.rs` passed scoped rustfmt after formatting; `git diff --check` passes. | Repository; **AMBIGUOUS** | Do not blanket-format the repository; identify the accepted formatting baseline or scoped G8 policy before claiming hygiene. |
| `R-P4-D1-D2-D4` | Required current discriminators were not rerun. Current main's parser-package comparison shows metadata-export drift, but it is not the exact P4 discriminator. | C1 ownership not re-established; **AMBIGUOUS** | Rerun the exact P4 discriminators on the feature branch and compare current main. |
| `R-P7-NUMERIC-HASH-LAW` | The known equal numeric representations, large integers, NaN payloads, and signed zero matrix was not rerun. | P7/global hash-law ownership not re-established; **AMBIGUOUS** | Rerun the exact law matrix and classify each result. |
| `P9-T0-PARSE-GATE-TIME` | The same `modules_parse_package` gate took 447.98s on the feature branch and 153.02s on current main. This is a single observed comparison; no causal attribution has been established. | **AMBIGUOUS** | Include in P9 performance analysis; compare repeatable focused inputs before assigning ownership. |

# SC-1 Correctness Remediation — Fresh-Session Handoff

Continue this task from the attached SC-1 amendment and continuation prompt. The amendment is authoritative over historical plans. Do not restart completed work.

## Mission and scope

- Deliver the retained SC-1 correctness amendments, checkpoint by checkpoint, with source declaration identity flowing through semantic lowering, runtime identity, reflection, and tooling.
- Current position is Slice 8 bounded evidence closed; Slice 9 baseline extraction remains gated by incomplete broad core-target evidence.
- Out of scope: toolchain/Cargo/CI policy changes, lightweight/native `Result` representation, unrelated parser or SC-2/SC-3 work, broad refactors, and historical tasks the amendment marks already resolved.
- User explicitly requested minimal reads/tests: read only the active slice and named symbols; do not use graphify; do not read historical plans or unrelated specs; do not run tests beyond the active slice’s listed commands without supervisor approval.
- Names are presentation. Never infer `DeclarationId`, `VariantId`, owner, module, or runtime identity from leaf spelling.

## Repository and branch authority

- Remote branch: `origin/fix/sc1-correctness-amendment`.
- Local branch/worktree: `fix/sc1-correctness-amendment` at `55bf6a5e7cda4d788a1fe3591caed7caa993e0e7` (`fix(core): require semantic identity for variant matches`).
- This branch worktree is `/Users/altunhasanli/dev/phalcom/phalcom-sc1-correctness-amendment`.
- Main checkout `/Users/altunhasanli/dev/phalcom/phalcom` has unrelated local dirty changes. Preserve them; do not clean, reset, overwrite, or stage them.
- Remote branch implementation is trusted and authoritative. Do not redo or replace its completed commits merely because they were authored by another agent.
- Current branch HEAD before this documentation checkpoint was `ee6d88a5` (`test(sc1): record post-repair gate evidence`); this handoff is task-owned and is being committed with the Slice 8 state evidence.

## Completed work to trust

- Slice 1/2 source/Universe authority and runtime ADT identity foundation landed in prior commits.
- Slice 3 prelude authority and `Option<T>` contracts landed; prelude is source-backed and shared, and callback contracts use positional `(T) -> U` forms.
- Qualified type paths now fail closed instead of dropping intermediate components (`fe9f13e0`). Do not reintroduce `members.last()` behavior.
- Match compiler fallback now compiles only canonical semantic `VariantId`; structural fallback remains allowed, but unlinked variant patterns return `CompilerError::MissingMatchLoweringSemantics` (`55bf6a5e`, `9b56c4e9`). Do not reconstruct owners/variants from `Some`, `None`, `Ok`, `Error`, `Err`, or ordering spellings.

## Current next checkpoint — Slice 9 prerequisite review

Slice 8 bounded evidence is recorded in `STATE.md`. Native/source reflection metadata, canonical type metadata, and fast census gates passed individually. The complete core target remains uncompleted because serial execution is impractical and the broad reflection filter repeatedly stalled at 0% CPU; these are explicitly classified, not normalized as passing. Do not rerun either broad target without a new bounded strategy.

Before Slice 9, confirm whether the amendment’s broad-correctness prerequisite has been satisfied by an approved alternative. Slice 9 is behavior-preserving baseline extraction and must not start while that prerequisite remains open.

## Historical checkpoint — Slice 4 semantic match resolution

Promoted ignored tests:

- `match_diag_02_ambiguous_variant_has_owner_candidates`
- `match_diag_03_inaccessible_variant_points_at_explicit_name`
- `match_res_08_ambiguous_contextual_owner_reports_no_arbitrary_candidate`

Fixtures are in:

- `/Users/altunhasanli/dev/phalcom/phalcom-sc1-correctness-amendment/phalcom-semantic/tests/semantic/adts/matching/diagnostics.rs`
- `/Users/altunhasanli/dev/phalcom/phalcom-sc1-correctness-amendment/phalcom-semantic/tests/semantic/adts/matching/resolution.rs`

Inspect only the direct candidate-resolution owners needed among:

- `phalcom-semantic/src/checker/pattern.rs`
- `phalcom-semantic/src/match_semantics.rs`
- `phalcom-semantic/src/enum_semantics.rs`
- `phalcom-semantic/src/associated.rs`

Required semantics:

- ambiguous contextual owner preserves candidate ambiguity and never picks declaration order or first spelling match;
- explicit inaccessible/wrong owner remains constrained to that owner and reports the explicit reference;
- promote tests only after fixtures are meaningful; do not weaken assertions or merely remove `#[ignore]`.

Allowed validation for this checkpoint: inspect current Cargo registration, run each promoted filter through the registered `semantic` target (red before fix and green after), and optionally one combined matching filter if registration supports it. No package/workspace suite without supervisor approval.

## CI evidence

CI run `33538642588` / #450 for `55bf6a5e` completed with:

- Miri (phalcom-ast): success;
- VS Code extension E2E: success;
- workspace Test (nightly-2026-07-10): failure;
- Clippy: failure;
- Rustfmt: failure.

Treat existing formatting/clippy and broad-test failures as baseline until reproduced against current changes. Repair only failures introduced by the active checkpoint; do not alter toolchain/Cargo/CI policy.

## Resume sequence

1. Read this handoff and `STATE.md`; inspect `git status`, branch, and current HEAD.
2. Preserve the explicit Slice 8 bounded-gate classification; do not treat interrupted broad targets as passes.
3. Do not begin Slice 9 unless complete broad correctness evidence is clean or the authoritative plan is amended.
4. If Slice 9 becomes eligible, record before/after structural semantic equivalence and rerun the same focused/package/full gates.

## Do not re-explore

- Do not repeat Slice 1/2/3 implementation or broad historical-plan reading.
- Do not use graphify.
- Do not touch main checkout dirty files.
- Do not introduce name-based registries, fabricated IDs, `ResolvedProjectId` durable metadata, zero fingerprints, Universe exposure bypasses, or lightweight `Result` storage.

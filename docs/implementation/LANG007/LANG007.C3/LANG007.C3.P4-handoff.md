Continue LANG007.C3.P4 from the checkpoint below. The active plan remains authoritative.

## Mission and scope
- Deliver: finish T13 stabilization/certification and leave P4/C3 verification truthful.
- In scope: review the incomplete G6 evidence and perform only further user/supervisor-authorized work.
- Out of scope: unrelated repairs to generic Option call-entry, captured setter validation, global formatting drift, or user-authored notes.
- Authority: `/Users/altunhasanli/dev/phalcom/phalcom/docs/implementation/LANG007/LANG007.C3/LANG007.C3.P4-intrinsic-structural-data-semantic-completion-plan.md` T13 / §16; checkpoint is the shared state.

## Current state
- Status: T1–T12 evidence and G1–G5 focused selector results/dispositions are recorded; the G5 collection aggregates remain unpassed/uncertified. G6 is incomplete and P4 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`.
- The one authorized workspace-test retry was run after correcting three stale AST test/snapshot oracles. Its process session disappeared after `phalcom-core --lib` reported 157 run, 155 passed, 2 failed. There is no captured final Cargo exit summary or evidence about later workspace targets. Do not describe this as a workspace-wide test total.
- After the scoped T13 delivery, only the two preserved user paths remain modified. Do not edit or stage `docs/.obsidian/workspace.json` or `docs/__unstructured__/Untitled 1.md`.
- No push was performed in this continuation.

## Evidence and decisions
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace --all-targets` — PASS, with one existing dead-code warning.
- First `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace` attempt — AST integration target stopped at 283 selected, 279 passed, 3 stale-oracle failures, 1 ignored. The AST oracle/snapshot fixes and exact focused passes are recorded in the checkpoint.
- The one authorized workspace retry — last captured target `phalcom-core --lib`: 157 run, 155 passed, 2 failed; command session was lost before final summary. Exact selectors independently reproduced both:
  - `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib kernel_option_match_override_reroutes_every_combinator -- --nocapture` — 1 selected, failed at the Some-through-none-arm assertion. This failure is already recorded as an internal generic call-entry failure in LANG005.C4.
  - `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib captured_setter_rejects_an_ordinary_positional_rhs_lane -- --nocapture` — 1 selected, failed its expected-Arity assertion. LANG004 R1 recorded a pre-c386 PASS; c386 changed `validate_captured_method_shape`. No P4/T13 commit edits this test or `phalcom-core/src/vm/send.rs`. Classify D/outside P4, but not a verified baseline.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings` — exited 101 on `match_like_matches_macro` for the pre-P4 `starts_expression` match (P4 added token alternatives only) and pre-P4 `phalcom-type-meta` lints (`result_large_err`, `manual_range_patterns`, `op_ref`). No unrelated lint fix was made.
- `cargo fmt --all -- --check` — exited 1 with broad workspace drift (5,263 output lines). Six identified P4-authored formatting blocks were corrected locally, without running a broad formatter. `rustfmt --edition 2024 --check phalcom-ast/src/parser.rs phalcom-core/src/compiler/lib/patterns.rs phalcom-core/src/compiler/lib/scope.rs phalcom-core/src/modules/compile.rs phalcom-core/src/modules/semantic_lowering.rs phalcom-core/tests/core/language/data_e2e.rs` — PASS. Other repository formatting drift remains.
- Global `git diff --check` — exit 2 only for six trailing-whitespace lines in the preserved `docs/__unstructured__/Untitled 1.md`; do not change that file.
- G5 collection aggregates remain encountered, not baseline-certified: `corpus::collections` stops at `TakeIterator.toList`; `corpus::collections_literals_negative` stops at the map-mutation fixture's `type.generic.underconstrained` before migrated fixtures. The checkpoint records full G1–G5 evidence and exact disposition.

## Code and artifact map
- `/Users/altunhasanli/dev/phalcom/phalcom/docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md` — authoritative statuses, gate evidence, and failure classifications.
- `/Users/altunhasanli/dev/phalcom/phalcom/docs/implementation/LANG007/LANG007.C3/LANG007.C3.P4-walkthrough.md` — implementation decisions and T13 certification outcome.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/universe/mod.rs::kernel_option_match_override_reroutes_every_combinator` — unrelated existing failure; do not edit for P4 without new causal evidence.
- `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-core/src/vm/send.rs::VM::validate_captured_method_shape` and `captured_setter_rejects_an_ordinary_positional_rhs_lane` — post-c386 discrepancy; no P4 path change found.

## Resume plan
1. Read the current checkpoint and this handoff; do not repeat the workspace test or rerun unchanged broad gates.
2. Obtain supervisor direction on whether any further G6 investigation is required. If asked to investigate either core failure, start with its recorded origin and a bounded causal check; do not weaken semantics or change expected results speculatively.
3. Preserve all user edits and keep P4 verification `UNVERIFIED` until evidence meets the plan's named certification criteria.

## Do not re-explore
- The T13.1 iterator census, G1–G5 focused selector results, T12.6 incremental matrix, and T12.7 editor matrix are already recorded in the checkpoint.
- Do not repeat the single authorized workspace-test retry merely to recover a final summary; its process is gone, and the last known scope is precisely recorded as incomplete/unknown after `phalcom-core --lib`.
- Do not broaden formatting changes or alter either preserved user note.

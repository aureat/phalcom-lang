# SC-1 Correctness Remediation Working State

## Current position

Active checkpoint: Slice 1 — canonical Universe source/interface authority
Completed checkpoints: S0
Current task: verify and complete the partially-landed Slice 1 implementation on `fix/sc1-correctness-amendment`
Next concrete action: run PR CI against the existing module-layer authority changes, then complete the semantic source-declaration/conformance half
Last verified evidence: S0 baseline recorded `builtin_catalog` 7/0/0 and unified `semantic` 905 passed, 2 known active failures, 51 ignored
Do not rerun unless changed: S0 preflight baseline at `4148de61f5415729fe5fe4ccfcef383292548ffe`
Active incident: none

## Working inputs

Plan copies for this work unit are in `state/plan-sources/`:

- `phalcom-sc1-correctness-amendment-plan.md` — authoritative amendment
- `phalcom-pre-sc1-stabilization-patch-grade-implementation-plan.md` — historical rationale only where retained
- `phalcom-post-universe-review-correctness-remediation-plan.md` — historical rationale only where retained

Repository-owned source plans remain unchanged at their original locations.

## Dirty-tree ownership

Pre-existing untracked paths at start:

- `.agents/skills/claude-d3js-skill/`
- `.agents/skills/karpathy-llm-wiki/`
- `docs/impl/semantic/semantic-completeness/sc-1/phalcom-sc1-correctness-amendment-plan.md`
- `docs/spec/universe.md`

Treat these as user-owned/non-SC-1. Preserve them. This working-state directory and implementation/test changes made for SC-1 are implementation-owned.

## Checkpoint S0 — Local evidence baseline

### Checkpoint contract

Tasks: amendment Slice 0
Semantic boundary: establish current target registration and classify known failures before edits.
Entry conditions: current HEAD and dirty state recorded; amendment copied into working-state folder; graph navigation complete.
Invariants established: no production behavior changes; scope excludes Cargo/toolchain/CI and lightweight Result representation.
Required evidence:

- registered `phalcom-modules` builtin-catalog target result;
- complete registered `phalcom-semantic` semantic-target result;
- exact active/ignored failure classification;
- source anchors for Slice 1 and Slice 2.

Deferred evidence: all slice-specific red/green regressions and package gates.

### Task S0 — Capture current evidence

Status: complete
Purpose: prevent stale historical failures from being misattributed to SC-1 changes.
Important files and symbols: amendment §6–7, `phalcom-modules/Cargo.toml`, `phalcom-semantic/Cargo.toml`.
Must remain true: no edits outside working state; preserve user-owned untracked files.
Important findings:

- Preflight: `HEAD` was `4148de61f5415729fe5fe4ccfcef383292548ffe`; pre-existing untracked amendment and `docs/spec/universe.md` remain, and the state directory is implementation-owned.
- `graphify-out/graph.json` existed. Required query returned the authority/identity graph slice containing `ModuleId`, `SemanticSnapshot`, `Universe`, and `semantic_lowering.rs`; direct source anchors remain required for edits.
- Target registration: `phalcom-modules` uses normal integration-test discovery, so `tests/builtin_catalog.rs` is the registered target; `phalcom-semantic/Cargo.toml:22-24` registers its sole unified `semantic` target.
- Slice 1 direct anchors: `phalcom-modules/src/interface.rs:114-296` builds declarations and exports; `phalcom-modules/src/builtin_interface.rs:84-146` adds synthetic root bindings and non-root export-all behavior; `phalcom-modules/src/linker.rs:602-675` resolves/links declared exports; `phalcom-semantic/src/declarations.rs:157-208` bootstraps native Universe declaration forms; `phalcom-semantic/src/session.rs:238-510` mixes bootstrap and source declaration construction.
- Slice 2 direct anchors: `phalcom-core/src/vm/adt.rs:106-117` returns a previously registered root by declaration; `phalcom-core/src/adt.rs:143-166` currently accepts repeated registration without checking root/representation agreement; `phalcom-core/src/vm/associated.rs:18-30` falls through to builtin leaf-name resolution after ADT lookup; `phalcom-core/src/vm/api.rs:14-18` is that name resolver.

Evidence:

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test builtin_catalog -- --nocapture` — passed: 7 passed, 0 failed, 0 ignored.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic -- --nocapture` — failed as baseline: 905 passed, 2 failed, 51 ignored.
- Active known baseline failures: `semantic::foundations::expression_engine::test_keyword_argument_mismatch_detected` (GATE-01 semantic bug) and `semantic::support::regressions::union_expectation_rejects_wrong_structural_members` (GATE-02 aborting panic-oracle harness bug).
- `phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md` records the same active-gate result and 51 ignored tests; its 100 ignore markers include ignored tests that are conditionally not part of this target run. No ignored test was forced or reclassified.

Deferred evidence: Slice 1/2 red-green fixtures and package gates; Slice 7 owns repair of the two active failures. No amendment regression existed at S0 because no production/test change had been made.

Resume pointer: Slice 1 is now active. The landed WIP baseline already contains the module-layer root-alias/non-root-export work and BCAT-08/09; verify that through PR CI, then complete source-driven semantic declaration shells and conformance before declaring Slice 1 complete.

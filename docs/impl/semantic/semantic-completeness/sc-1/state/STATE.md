# SC-1 Correctness Remediation Working State

## Current position

Active checkpoint: Slice 8 — broad amendment-owned validation
Completed checkpoints: S0, Slice 4, Slice 5, Slice 6, Slice 7; Slices 1–3 inherited and previously landed
Current task: finish bounded Slice 8 core/reflection/workspace evidence after repairing canonical Universe lowering
Next concrete action: complete bounded non-hanging core/reflection disposition, then decide whether Slice 9 is warranted; do not claim broad correctness while those gates remain incomplete
Last verified evidence: Slice 4 semantic matching filter `162 passed, 0 failed, 22 ignored`; Slice 5 semantic metadata `21 passed`, native conformance `3 passed`, module identity `4 passed`; Slice 6 focused Universe exposure/relative identity, package intrinsic, bootstrap measurement, standalone package/module, builtin-client, and dependency-sentinel gates pass; Slice 7 Option typing `4 passed`, GATE-01 `1 passed`, GATE-02 `1 passed`, and adjacent positional/keyword argument tests `1 passed` each; Slice 8 module target `107 passed, 0 failed`, semantic target `927 passed, 0 failed, 48 ignored` both before and after core repair, required core `native_adt_runtime` filter `6 passed, 0 failed`, canonical range regression `1 passed`, curated prelude regression `1 passed`, workspace check passed, and module unit tests `4 passed, 0 failed`
Do not rerun unless changed: S0 preflight baseline at `4148de61f5415729fe5fe4ccfcef383292548ffe`
Active incident: Slice 8 core full-target evidence remains incomplete only for the previously interrupted broad reflection run; the deterministic range-variant and `universe.None` exposure failures are repaired and focused-green

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

## Reconciliation — Slices 4–6

Slice 4 focused evidence: semantic `matching` filter passed with `162 passed, 0 failed, 22 ignored`, including promoted ambiguous contextual-owner, inaccessible explicit-owner, and no-arbitrary-candidate tests. Dirty changes cover declaration-backed contextual variant ownership, candidate sets, match-product fingerprinting, and corresponding test models. Slice 4 is checkpoint-ready.

Slice 5 focused evidence: semantic `metadata` filter passed with `21 passed`; semantic `native_conformance` passed with `3 passed`; module `identity_foundation` passed with `4 passed`. Dirty changes cover context-aware durable identity, revision fingerprints, source/native conformance, declaration-aware runtime class lookup, and Iterable source/native arity alignment. Slice 5 is checkpoint-ready.

Slice 6 reconciliation: inherited dirty changes cover Universe exposure/relative resolution, package intrinsics, canonical bootstrap dependency ordering, bootstrap measurement, and legacy identity cleanup. Stale `std.json` imports in standalone fixtures were removed or replaced with `universe.json`; unused canonical selector imports were removed after duplicate-source execution surfaced. VM bootstrap now marks successfully executed canonical modules `Initialized`, preventing imported Universe modules from being recompiled. Focused evidence: modules `integration` Universe exposure/relative identity `3 passed`; core `universe_package_intrinsics_match_provider_neutral_package_rules` `1 passed`; core `boot_01_bootstrap_measurement_separates_catalog_closure_and_execution` `1 passed`; core standalone package, standalone module, and builtin-client reflection gates each `1 passed`; semantic dependency-sentinel `1 passed`. Slice 6 is checkpoint-ready and committed as `18d17b33`.

Slice 7 reconciliation: active GATE-01 keyword mismatch now follows the shared argument assignability path; focused semantic gate passed. GATE-02 uses a result-returning type oracle instead of abort-sensitive unwind capture; focused regression passed. Generic callable inference now structurally relates canonical callable types, exact enum cases, contextual generic constructor results, and final expected-result constraints; Option `map`, `flatMap`, `unwrapOr`, and `okOr` focused tests passed `4/4`. Slice 7 adjacent positional/keyword binding tests passed `1/1` each. Slice 7 is checkpoint-ready and committed as `b3ff8116`.

## Reconciliation — Slice 8

Slice 8 semantic/module evidence is clean after a narrow regression correction in `phalcom-semantic/src/checker/call.rs`: retain the initial underconstrained outcome for diagnostics unless post-context solving has established value support. Focused regressions for expected-context underconstraint, both underconstrained diagnostic presentations, and `option_flat_map` passed `1/1` each; the Option group passed `4/4`; the complete semantic target passed `927/0/48`; the complete modules package passed `107/0/0`.

Earlier Slice 8 core evidence was partial and bounded. `native_adt_runtime` passed `6/6`. The complete core target was interrupted while progressing after deterministic `core_collections::range_literals_drive_collection_slices` (`StrError("unregistered variant")`) and `curated_prelude_exposes_public_names_and_hides_internal_classes` (`universe.None` exposure); both are dispositioned by the repair checkpoint below. The reflection filter was stopped after repeated long-running tests before its final summary; no failure was emitted before stopping.

### Slice 8 core repair checkpoint

The canonical lowering repair is now focused-green. `VM::universe_lowerings` performs one source-complete semantic analysis over the linked Universe corpus, so `Result::Ok`/`Result::Error` lowering retains canonical `Universe::errors.result::Result` variant owners instead of reaching compiler fallback identity. The semantic session permits only the exact bundled-source erased `Universe::Iterable` superclass form; ordinary/user source keeps the `KindExpectedType` rejection. Root prelude synchronization now exposes `UniverseKey::None` through the canonical `none_class` object even when the owner module binding is not materialized.

Focused evidence:

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly test -p phalcom-semantic --test semantic written_invalid_superclass_does_not_publish_ready_declaration -- --nocapture` — passed: `1 passed, 0 failed, 974 filtered out`.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly test -p phalcom-core --test core range_literals_drive_collection_slices -- --nocapture` — passed: `1 passed, 0 failed, 438 filtered out`, finished in `71.43s`.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly test -p phalcom-core --test core curated_prelude_exposes_public_names_and_hides_internal_classes -- --nocapture` — passed: `1 passed, 0 failed, 438 filtered out`, finished in `71.11s`.
- `rg` debug-probe deletion gate — passed; no `RANGE DEBUG`, `RESOLVE DEBUG`, `INFERENCE DEBUG`, `GEN DEBUG`, `QUERY DEBUG`, `SUPER DEBUG`, or `SESSION DEBUG` remains in core/semantic source.
- `git diff --check` — passed.

The broad reflection filter remains incomplete because repeated runs reached a 0%-CPU hang and were interrupted; do not loop that filter before bounded disposition. A serial complete-core probe was separately interrupted after about three minutes while CPU-bound and advancing at roughly one test per minute; it was projected to take several hours because each test reboots Universe. This is duration classification, not a semantic failure. Slice 9 remains last and is not started.

Search/deletion gates produced no forbidden semantic shortcut hits; remaining `UniverseKey::from_name` hits are pre-resolution source/catalog or presentation lookups. `git diff --check` is clean. Slice 8 now has focused semantic-call and canonical-Universe core-repair commits; the inherited seven-file Slice 5/6 dirty set and handoff remain unstaged. Broad acceptance remains open, and Slice 9 must not start until core/reflection evidence is clean or explicitly dispositioned by the authoritative plan.

## Reconciliation — inherited implementation residue

The seven tracked files left unstaged after the earlier checkpoints are amendment-owned residue, not unrelated work: `phalcom-core/src/native/verify.rs`, `phalcom-core/src/primitive/typing.rs`, `phalcom-core/src/typing/registry.rs`, and `phalcom-modules/tests/identity_foundation.rs` complete Slice 5 source/revision identity and reflection wiring; `phalcom-modules/src/linker.rs` provides Slice 6 whole-interface linking; `phalcom-core/src/compiler/lib/enum_decl.rs` and `phalcom-semantic/src/checker/declaration.rs` preserve canonical declaration-owner selection across the earlier identity slices. Their existing focused evidence is green: modules full target `107 passed`, semantic metadata `21 passed`, semantic native conformance `3 passed`, and core `native_adt_runtime` `6 passed`. They are reconciled in one focused commit without staging the untracked continuation handoff.

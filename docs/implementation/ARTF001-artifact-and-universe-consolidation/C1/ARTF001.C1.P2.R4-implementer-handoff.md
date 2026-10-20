# ARTF001.C1.P2.R4 Implementer Handoff

**Date:** 2026-10-20
**Worktree:** `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact`
**Branch:** `feat/artifact-universe-consolidation`
**Starting HEAD:** `961c128bbea1cf2ebf4096047a511658ecfe8840`

## Latest continuation finding — 2026-10-20

The explanation-only module-binding fingerprint issue and the reported test-harness repairs are complete, but the long vertical gate still does not reach artifact loading or allocation measurement. The exact command

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact-allocation -- --nocapture --test-threads=1
```

fails after 98.41 seconds while `build_universe_telosc` exports the semantic world:

```text
duplicate runtime dependency requirement
importer: universe.callable.function
dependency: universe.errors.error
source range: 831..836 in phalcom-core/core/universe/src/callable/function.ph
source: on(Error)
```

The strict duplicate check in `phalcom-compiler-artifact/src/semantic/world.rs` currently defines duplicate identity as `(importer, dependency, range)`. No allocation N, independent allocator equality, N−1 typed failure, rollback, or retry assertions were reached. The temporary diagnostic used to reveal the module/range has been removed; the validator remains unchanged.

**Root-cause lead, not yet proven:** `phalcom-semantic/src/phase_evidence.rs` projects unresolved source-visible native prelude reads in `add_native_prelude_read_requirements`, then also projects nominal declaration reads from semantic `TypeForm` expressions in the callable-analysis loop. `Error` in `on(Error)` appears to overlap these paths. Confirm the actual two serialized requirements, including `read_target` and provenance, before changing behavior. Do not weaken the artifact validator or deduplicate solely by `(importer, dependency, range)` until the coalescing contract is clear; same-span requirements can carry distinct targets or authority.

The immediate next action is to establish the requirements' full identities at the producer boundary, coalesce or suppress only a semantically redundant projection at its owning layer, and add a focused regression for the overlap. Also test a same-edge/same-range case with distinct targets to ensure required reads are not lost. Then rerun the exact long gate.

## Objective and current status

Continue `ARTF001.C1.P2.R4`, using the active plan and `C1/CHECKPOINT.md` as the authoritative scope and status record. R4.T0 is complete. R4.T1 remains in progress. R4.T2 and R4.T3 are not complete; some T2 implementation slices have begun to unblock the vertical artifact build. Do not mark a task complete from compile-only or partial test evidence.

**Earlier continuation update:** The module-binding mismatch is fixed at its source. `BindingState.explanation` was absent from the `ModuleStaticBindingIndex` query fingerprint, allowing the final session publication to reuse a preliminary index. The fingerprint now includes the explanation ID, and a focused query-reuse regression passes. The core integration target, semantic lib-test target, and artifact-allocation target now compile. The two `builtin_catalog` failures are resolved. The latest long allocation execution and its new blocker are recorded above.

The previously observed blocker was the canonical Universe artifact build failing semantic-world export because one module-static-binding owner row had a stale `explanation` field:

```text
module: ModuleId { project: Universe, path: errors.unsupported }
symbol: errors.unsupported::unsupported
owner body: errors.unsupported::<main>
source binding: BindingId(0)
```

That mismatch was corrected by including `BindingState.explanation` in module-static-binding query fingerprints; the focused regression passes. The exporter referential-integrity check remains strict. The current blocker is the duplicate runtime dependency requirement documented in the latest continuation finding above.

## Work completed in this continuation

- Added request-aware semantic publication so compiler-image requests can omit advisory and presentation products at their generation points while default/full sessions retain them. The request is configured through `SemanticWorkspaceSession` and consumed during publication.
- Added canonical operational-program publication for checked static bodies in canonical compiler-image sessions. Lowering and static-value interning occur before snapshot publication and root registration; lowering errors fail publication.
- Updated compiler specialization handling to validate static argument kinds, root only proper type terms, and preserve type, record-row, and dynamic arguments during generic evidence instantiation.
- Fixed the module-world invalid/duplicate import identity test failure.
- Ported the allocation test and current-schema ABI identities to the V2/V4/V3 artifact formats. The user-provided starting handoff reports the compiler-artifact suite passing; it was not rerun during this continuation.
- Added Universe `Behavior` exports needed by `Class` and `Metaclass` sources.
- Recorded prior architectural consultations and focused results in `C1/CHECKPOINT.md`. The latest consultation determined that absent canonical static programs are a session-publication implementation gap; no plan amendment was required.

The working tree contains the above changes plus earlier R4 work. Do not reset, clean, or overwrite the worktree. The unrelated dirty file in the main checkout, `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/session.rs`, must remain untouched; all artifact work belongs in the worktree named above.

## Verification evidence

| Command | Result |
|---|---|
| `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic-product-request` | PASS, 4/4. Covers full versus compiler-image request publication, static-program publication, and generic `List<Dynamic>` / `List<Int>` specialization. |
| `RUSTFLAGS='' cargo test -p phalcom-modules module_world_rejects_invalid_and_duplicate_module_import_identities -- --nocapture` | PASS. |
| `RUSTFLAGS='' cargo test -p phalcom-modules` | All 13 unit tests pass; integration suite has two unresolved `builtin_catalog` failures: expose-count expected 22 but got 23, and unresolved `universe.collections.iterator_laws` import. Classify these before treating them as baseline. |
| `RUSTFLAGS='' cargo test -p phalcom-core --test core modules::canonical_artifact` | BLOCKED before test execution by 10 compile errors in existing core test fixtures/API usage. No selected test ran. Errors include missing `RuntimeCallArgumentRecipe`, `LoweringBindingIdentity`, `MetadataExporter::build_complete_test_bundle`, and `SyntheticProjectId::from_raw`; a missing `CandidateBinding.identity`; and immutable linked-program mutation in `parity.rs`. |
| `RUSTFLAGS='' cargo test -p phalcom-core --test artifact-allocation -- --nocapture` | FAIL after 107.21s at `build_universe_telosc`: the strict module-static-binding owner-row check reports the `explanation` difference above. Allocation N, N−1, retry, and independent allocator comparison were not reached. |

The allocation test first failed because source-backed static definitions lacked canonical operational programs. That failure was addressed by the publication change above; the latest run advanced to the module binding mismatch. A temporary verbose exporter error was used to identify the differing field and has been removed. The original referential-integrity rejection remains intact.

## Immediate implementation sequence

1. Trace how `errors.unsupported::<main>`'s `BindingState.explanation` differs between the published `callable_analyses` row and the `ModuleStaticBindingFact`. Inspect the creation of the final `ModuleStaticBindingIndex`, snapshot assembly, explanation ownership/remapping, and any normalization between those steps. Determine whether the index was built from an earlier analysis or whether explanation IDs are intentionally removed from owner rows.
2. Preserve the exporter check. Do not simply remove `explanation` from equality or weaken the assertion. If the right fix depends on whether explanation IDs are canonical semantic state or optional owner-local evidence, request architectural guidance and record the decision in `CHECKPOINT.md` before editing.
3. Apply the correction at the owning semantic publication/projection layer so module facts and callable owner rows have a consistent canonical contract. Add focused coverage for both this mismatch and explanation-bearing module bindings, without duplicating authority in the exporter.
4. Rerun the exact gate:

   ```sh
   RUSTFLAGS='' cargo test -p phalcom-core --test artifact-allocation -- --nocapture
   ```

   It must reach and pass full-load exact N, independent allocator equality, typed N−1 rejection, and clean retry. Continue with any newly exposed failure rather than calling the gate complete early.
5. Repair or classify the core integration-target fixture/API drift so the requested `modules::canonical_artifact` test can actually run. Do not report a filtered test as passed when the target failed to compile.
6. Classify the two `phalcom-modules` `builtin_catalog` integration failures with a bounded comparison against the active plan and source changes. Preserve them as unresolved if they remain outside the active acceptance gate.
7. Continue the ordered R4 tasks in the active plan. In particular, T2 still requires the complete V2/V4/V3 build/load/source-free compiler path and native equivalence; T3 requires stage-level allocation census, rollback, failpoints, and retry. R4 also retains the later T4–T9 gates for query closure, imported-session lifecycle, consumer consolidation, source denial/parity/runtime, broad regression, and P2 closure.

## Continuation verification update

| Command | Result |
|---|---|
| `RUSTFLAGS='' cargo test -p phalcom-semantic --test module-static-binding-fingerprint -- --nocapture` | PASS, 1/1. Explanation-only changes invalidate module-binding query reuse. |
| `RUSTFLAGS='' cargo test -p phalcom-modules --test builtin_catalog -- --nocapture` | PASS, 9/9. |
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact` | PASS, 106/106. |
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib --no-run` | PASS. Current callable V2 fixtures compile. |
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core --no-run` | PASS. The prior 10 fixture/API errors are repaired. |
| `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact-allocation --no-run` | PASS. |
| `RUSTFLAGS='' cargo test -p phalcom-core --test core compiler_product_optimization::test_decision_engine_pure_cases -- --exact --nocapture` | PASS, 1/1. |
| `RUSTFLAGS='' cargo test -p phalcom-core --test core modules_parity::comparator_ -- --nocapture` | INCOMPLETE. Eight tests selected; bounded local run timed out after 60 seconds. |

Run the long active gate externally from this worktree:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact-allocation -- --nocapture --test-threads=1
```

## Topics requiring advanced semantic/artifact review

- **Runtime-requirement identity and projection overlap.** The current validator treats `(importer, dependency, source range)` as a unique row, but multiple semantic projections can observe one source read. Establish whether this is the intended canonical identity and how equal-edge observations combine `read_target`, producer, product, and timing. The `Error` in `on(Error)` is a likely overlap between native-prelude fallback and semantic `TypeForm` projection; inspect the actual rows. Preserve distinct binding targets and conflicting authority, and test both redundant same-target observations and distinct-target observations at one source span. Do not silence the validation failure by broad deduplication.
- **Explanation identity and cross-product consistency.** `ExplanationId` is callable-owner-local and is already validated for callable bindings and expressions by artifact export. Decide which module-binding fields are canonical state, when explanation references are retained or normalized, and which product is authoritative after snapshot publication. A local equality workaround could hide stale owner data or silently discard valid explanation authority.
- **Request-scoped semantic publication.** Compiler-image requests must suppress optional products where they are generated while retaining all compiler-required formal/source products. Default/full session behavior must remain unchanged, and publication rollback must remain atomic. Avoid generating presentation/advisory products and stripping them afterward.
- **Canonical static-program publication.** Checked static bodies must lower into canonical operational programs before snapshot/root closure; generated static values must be interned before export graph registration. Imported execution remains source-free and must match source evaluation for owner, dependencies, applications, result, and failure behavior.
- **Static-argument categories and runtime roots.** Specialization bindings can carry type, record-row, or dynamic arguments. Only actual type terms should contribute runtime type roots, while all categories must survive generic evidence instantiation with kind validation. Keep this aligned with semantic ownership and runtime metadata.
- **Exact allocation transactions.** The current allocation test is a long vertical discriminator. Preserve independent allocator measurement, the ledger's exact N, N−1 typed failure, child-stage rollback, failpoints, and clean retry. Do not substitute historical byte counts or treat the test's early build failure as allocation evidence.
- **Current-schema artifact authority.** P2 uses ModuleWorldV2, SemanticWorldV4, and QueryImageV3. Continue validating project-qualified stable identities, native authority projection families, dependency/fingerprint closure, query roots, source-free publication, and single-consumer staging against the active plan and checkpoint.

## Remaining gate sequence

1. Resolve and regression-test the runtime-requirement overlap described above at the owning projection layer.
2. Rerun the exact whole-load allocation test. It must first complete canonical artifact build and load, then prove exact current N, independent allocator equality, typed N−1 failure, child-stage rollback, and clean retries. The last run stopped before all of these checks.
3. Run the repaired `phalcom-core` canonical-artifact integration filter and any adjacent source/artifact parity tests required by R4.T2. The integration target now compiles, but this handoff has no evidence that those selected tests executed after fixture repair.
4. Complete T2's native-equivalence and semantic-authority gates (declarations, aliases, hierarchy, Pack/TypePack ownership, module execution/import ordinals, witnesses and proofs, source-free static execution, callable completeness, and manifest closure).
5. Complete T3 stage/query allocation census, `QX = 0`, failure/retry boundaries, and an explicit artifact-image versus session-construction transaction contract.
6. Continue T4–T9 in order: query closure; ordinary imported session and overlays; consumer consolidation; fresh-process source denial and parity; broad regression/schema freeze; final walkthrough, checkpoint closure, and P3 handoff.

P3 remains blocked until the checkpoint's full R4 acceptance gates pass. Do not promote T2 or T3 based on compile-only evidence or this build-stage failure.

## Guardrails for takeover

- Read the active R4 plan and `C1/CHECKPOINT.md` first; the checkpoint is the shared status authority.
- Preserve all existing dirty and untracked work. Do not reset, clean, broadly format, or stage unrelated files.
- Use focused verification during implementation. The N/N−1/retry run is an explicit active gate; broad workspace certification remains a later plan gate.
- Keep spec rules, implementation defects, and plan status distinct. Record new consultations, durable decisions, and meaningful verification in the checkpoint.

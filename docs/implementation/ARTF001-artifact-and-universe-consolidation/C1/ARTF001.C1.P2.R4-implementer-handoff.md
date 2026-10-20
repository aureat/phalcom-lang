# ARTF001.C1.P2.R4 continuation handoff

Continue implementation from this handoff. The original/current specification is attached and remains authoritative. Do not restart the investigation or reinterpret the goal as focused-test completion.

## Mission and scope

Prove successful compilation of a real `universe.telosc`, source-free loading and ordinary compiler/session use, and testing speedup attributable to eliminating Universe source bootstrap. Follow the original milestones and the active R4 plan through their required gates. None of those end-to-end outcomes has been established yet.

Work only in `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact`, branch `feat/artifact-universe-consolidation`, HEAD `4c3ec841272631c42524e00ed97fec417e33c36b`. All changes described below are uncommitted. No commit, push, or branch change was requested.

Preserve the unrelated modified `/Users/altunhasanli/dev/phalcom/phalcom/phalcom-semantic/src/session.rs` in the main checkout. The artifact worktree's session.rs is not modified. Do not reset, clean, broadly format, stage unrelated work, or copy the main edit into this worktree.

Read the supplied AGENTS instructions and relevant lifecycle/escalation workflow. User explicitly requested routine manual work be delegated to GPT-6 Luna with `xhigh` reasoning while the parent handles complex diagnosis. The prior worker finished; no build or child process remains live. Run Cargo checks serially.

## Current state

R4 remains **IN_PROGRESS / PARTIAL / FOCUSED_TESTED**. R4.T2/T3 are incomplete; later tasks are not certified. The user asked to stop and hand off. The goal is paused after saving these records.

Implemented and focused-tested:

1. The accepted runtime-obligation ruling: exact obligation identity, strict duplicate rejection excluding producer from identity, full-record native/wire ordering, owning-producer suppression of proven repeats, and native/wire round-trip coverage.
2. A mechanical V4 validation adapter correction: preserve V2 aliases in the temporary V3 hierarchy/alias view, retain superclass fields, handle unordered inputs, and preserve duplicate rows for validation.
3. Direct Universe source-read denial and fresh child negative controls. The previously unregistered canonical-artifact core test module is now registered and compiles.

Current blocking issue: query product mapping cannot reference alias DeclarationShell products. A separate nominal Package query is absent from the final declaration aggregate. An architectural consultation is pending; no query representation, root retention, schema/ABI, or semantic oracle correction has been attempted. Do not make one without the ruling required by AGENTS.

No `/tmp/artf001-universe.telosc` exists. No full load, exact allocation, artifact-backed positive source-denial, or speedup result exists. Broad workspace/release checks were deliberately not run.

## Start with these records

All paths in this list share `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact/docs/implementation/ARTF001-artifact-and-universe-consolidation/C1/`:

- `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact/docs/implementation/ARTF001-artifact-and-universe-consolidation/C1/CHECKPOINT.md` — shared durable authority and current task ledger; latest continuation sections contain accepted decisions and pending gates.
- `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact/docs/implementation/ARTF001-artifact-and-universe-consolidation/C1/ARTF001.C1.P2.R4-luna-patch-grade-implementation-plan.md` — active plan, including accepted T2.6a runtime identity amendment.
- `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact/docs/implementation/ARTF001-artifact-and-universe-consolidation/C1/ARTF001.C1.P2.R4-query-shell-consultation.md` — complete current incident, all unmappable owners, code path, alternatives, and precise decisions requested. Read before editing the blocker.

Original milestone handoff: `/Users/altunhasanli/.codex/attachments/e8470d3a-9250-4724-8ab8-fe569985556f/pasted-text-1.txt`.

Accepted runtime-obligation advisory ruling: `/Users/altunhasanli/.codex/attachments/a7bd0485-a696-4df0-a22f-244b33811871/Pasted text.txt` (same ruling was attached earlier under another attachment ID). It does not authorize a query-product representation change.

## Established decisions and evidence

### Runtime requirements — settled

Obligation identity is importer, dependency, range, reason, read target, provenance target, product, and timing. Consumer must equal importer. Producer does not create a new obligation. Distinct obligations at one span survive transport; duplicate obligations fail. No global producer precedence or final blanket deduplication is authorized. No schema bump was required for this accepted correction before R4 freeze.

The complete runtime collision census found 521 coarse-location groups: 374 exact-repeat groups, with 616 extra rows (`CallableOwner` 302, `DeclarationRead` 72, `PatternOwner` 242), and 147 distinct-obligation groups. No producer-only collision remained. Temporary instrumentation was removed. Log: `/tmp/artf001-r4-complete-census.log`.

### Query mapping — consultation pending

The complete census inspected 2,564 reusable candidates and found 16 DeclarationShell mapping failures:

- Fifteen native Alias products have V4 `type_aliases` rows and no nominal/static-domain/intrinsic-owner row: `collections.interval::{LowerBound,UpperBound}`; `collections.pack::{CompletePack,LabeledPack,LabeledTypePackView,PositionalPack,PositionalTypePackView}`; `collections.record::{EmptyRecord,FieldKey}`; `option.unit::Unit`; `reflection.structural::{FixedFields,LabeledTail,PositionalTail,VariantNames,Variants}`.
- Native Nominal `reflection.package_object::Package` is absent from all four checked row families.

`TypeDeclarationShell` is a native Nominal/Alias sum type. The session publishes both under QueryKind::DeclarationShell. Export and wire bounds/owner validation currently select only nominal `structural_contracts.declaration_shells`. Native query reconstruction already handles aliases from the canonical alias table. Do not manufacture nominal shells or silently discard alias roots. The adviser must select the product-family/index contract, owner/fingerprint rules, schema/ABI decision, and required-root behavior.

An adjacent importer check requires a nominal shell after finding a static-definition alias's V2 alias row. This is a prospective incompatibility, not an observed load failure; include it in the advisory decision.

Census log: `/tmp/artf001-r4-query-census.log`. It failed after 109.50s of test execution, 129.15s including compilation, returning the original first error. No artifact or stage ledger was produced. Census instrumentation is fully removed.

### Package — additional verified lead

A temporary resolver probe used the current Package source interface and production symbolic linker/resolver. It verified that Package has no explicit imports, `Module` is excluded from the actual canonical prelude, and unqualified `Module` resolves to None in Package's scope even when its declaration is known. Positive controls resolved Object from the prelude and Module in its own local scope. The source says `native primitive class Package is Module`.

Probe command: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib artifact_package_scope_probe -- --nocapture`; exactly 1 test passed in 0.07s. Log: `/tmp/artf001-r4-package-scope-probe.log`. The temporary probe was removed completely; resolver.rs has no diff.

This establishes the unresolved superclass lookup. The likely missing explicit import and subsequent blocked-declaration removal explain Package's absence, but the probe did not execute the entire semantic update or prove the exact removal event. No Package source or query-freeze correction was made. The consultation has this updated evidence.

## Code map

Use these exact artifact-worktree paths.

- `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact/phalcom-compiler-artifact/src/semantic/world.rs`: obligation key/comparator and validation; `SemanticWorldV4::to_v3_for_validation` now preserves aliases without an input-order assumption.
- `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact/phalcom-compiler-artifact/tests/round_trip_test.rs`: obligation hostile matrix and three alias projection regressions.
- `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact/phalcom-modules/src/linker.rs`: `RuntimeDependencyRequirement::canonical_cmp`, used by native finalization.
- `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact/phalcom-semantic/src/phase_evidence.rs`: exact repeat suppression in DeclarationRead, CallableOwner, and PatternOwner producers; four focused tests retain original prelude overlap coverage.
- `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact/phalcom-semantic/src/artifact/export.rs`: production `export_runtime_dependency_requirements` helper shared with regression; query blocker in `export_query_image_v3` / `portable_query_product_ref`.
- `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact/phalcom-semantic/src/artifact/import.rs`: `runtime_requirement_multiset_survives_native_export_v4_and_canonical_import` transports five same-span native records through the real exporter, V4 codec/validation, and canonical import.
- `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact/phalcom-modules/src/builtin.rs`: `UniverseSourceProvider::source_text` now actively denies reads under PHALCOM_DENY_SOURCE_BOOTSTRAP; new() stays inert.
- `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact/phalcom-core/tests/core/modules/canonical_artifact.rs`: four source-producing negative controls plus inert-provider positive control, each in its own child; same helper used by later artifact positive proof.
- `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact/phalcom-core/tests/core/mod.rs`: registration of `modules_canonical_artifact`; this roughly 1,300-line test file was previously not registered and zero-selected filters were not evidence. It now compiles.

Diagnosis-only paths: `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact/phalcom-semantic/src/declarations.rs::TypeDeclarationShell`; `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact/phalcom-semantic/src/resolver.rs::LinkedTypeResolver::resolve_type_name`; `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact/phalcom-semantic/src/session.rs` alias-shell publication and final blocked-declaration removal; `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact/phalcom-compiler-artifact/src/query.rs` nominal-only row count and product-owner validation; `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/artifact/phalcom-semantic/src/artifact/static_definition_import.rs` alias-shell requirement. These files are unchanged except the task-owned files listed above.

## Validation

Passed:

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact` — 114 tests, including 8 round-trip tests; three alias adapter regressions are covered.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib phase_evidence::native_prelude_tests -- --nocapture` — 4/4.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib runtime_requirement_multiset_survives_native_export_v4_and_canonical_import -- --nocapture` — 1/1.
- `env -u PHALCOM_UNIVERSE_TELOSC_FOR_TEST -u PHALCOM_SAVE_UNIVERSE_TELOSC_FOR_TEST RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_canonical_artifact::source_denial_guards_are_effective_without_artifact -- --nocapture --test-threads=1` — exactly 1 selected, 1 passed, 1028 filtered, 0.02s. No artifact was used or built; this proves guard efficacy only.
- Temporary Package probe described above — 1/1; removed after capture.
- Scoped diff review and `git diff --check` pass. Whole-file rustfmt checks expose pre-existing unrelated formatting; leave it untouched.

Failed vertical runs, all before artifact save/load:

- `/tmp/artf001-r4-vertical-gate.log`: runtime obligation correction advanced export to missing static-alias authority; 113.22s test / 145.91s total.
- `/tmp/artf001-r4-alias-vertical.log`: alias adapter correction advanced export to missing DeclarationShell query row; 112.88s test / 129.52s total.
- `/tmp/artf001-r4-query-census.log`: same query failure after complete diagnostic census; 109.50s test / 129.15s total.

The common command is `PHALCOM_SAVE_UNIVERSE_TELOSC_FOR_TEST=/tmp/artf001-universe.telosc RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact-allocation -- --nocapture --test-threads=1`. The census run additionally enabled temporary instrumentation, which no longer exists. These are current active-path failures, not classified unrelated baseline failures. Do not rerun an unchanged 110-second failure for reassurance.

Not run: full registered canonical-artifact suite, artifact-backed positive denial, whole-load allocation stages/N/N-1/retry, ordinary imported session edits/rollback, real package/runtime parity, timing comparison, broad workspace/release certification. Export currently prevents the prerequisite artifact.

## Resume plan

1. Read the attached specification, this handoff, checkpoint's latest sections, and the query-shell consultation. Confirm `git status --short` and preserve all current edits.
2. Obtain and record the advisory ruling for query transport and final-aggregate/query consistency. The prior runtime advisory does not answer this. Update checkpoint/plan as required before changing representation, retention, alias authority, or schema.
3. Apply the bounded ruling with nominal/alias positive and hostile owner/family/index cases, including static-definition alias authority. Use the Package scope evidence to avoid rediscovering the unresolved Module lookup; distinguish a source import repair from the separate stale-query/final-aggregate invariant.
4. Run focused discriminators first. Once they pass, run the exact vertical gate once with the save-file environment variable. If it succeeds, retain the file for focused loader tests; if a new issue appears, classify it before another semantic edit.
5. Complete ordered T2/T3 and later gates. T3 risk already identified: V4 validation and callable conversion deep-clone DTO collections without budgeting while the accounting guard returns Ok. This has not yet been measured. Preserve independent allocation accounting; do not debit observed bytes after the fact or weaken equality.
6. T4 needs an explicit required-root profile; current export still reverse-prunes candidates. The query allocation census lists 45 kinds versus QueryKind::ALL's 46 (AttributeArgument omitted). Resolve under the existing plan, without promoting open-world roots.
7. Reuse the registered fresh-process proof and tested denial controls after a valid artifact exists. Compare the same real compilation workload in fresh source and artifact child processes; include source construction versus artifact load in their respective totals, report artifact construction separately, and assert eliminated work. No speedup has been measured yet.
8. Complete remaining lifecycle records and broad gates at their named plan boundaries. Keep implementation completion separate from focused versus release verification.

## Do not re-explore

Do not repeat the runtime collision census, reconsider accepted obligation identity, or rerun source builds before a relevant correction. Do not treat registered-but-unrun tests or negative-control success as artifact-backed success. All temporary instrumentation and the Package probe are removed. Do not commit/push, widen the prelude, fabricate nominal shells, silently prune required roots, weaken assertions, or start destructive consumer consolidation before its required gates.

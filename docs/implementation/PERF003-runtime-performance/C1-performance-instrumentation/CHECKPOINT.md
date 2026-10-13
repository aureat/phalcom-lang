---
id: PERF003.C1
program: PERF003
kind: optimization
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
active_plan: PERF003.C1.P5
latest_completed_plan: null
next_plan: PERF003.C1.P5
prepared_against_revision: e2ae4a0fd360e4075ce8ad229a3e39e1531fb771
---

# PERF003.C1 — Test-Execution Performance

## 0. Checkpoint state

```yaml
program: PERF003
checkpoint: PERF003.C1
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
active_plan: PERF003.C1.P5
latest_completed_plan: null
next_plan: PERF003.C1.P5
next_action: >
  P5 T0 synchronization and source re-audit are complete. T1's run-scoped
  observability crate and T2 feature propagation are implemented and compile
  checked; T1's round-trip tests remain outstanding. T3 emits a versioned partial
  cold VM profile, with full VM startup still blocked at ProductKind source
  indexing. T4 baseline-phase spans and source counters are implemented and a
  direct cold semantic-baseline profile completed successfully. T5 semantic
  update spans and committed `SemanticUpdateStats` counters are implemented and
  build-checked; ordinary and canonical source-backed update profiles both
  completed. T6 adds exhaustive `QueryKind` classification and report-level
  query lifecycle metrics; default/feature builds and ordinary/canonical update
  profiles pass, while deterministic lifecycle tests remain outstanding. T7
  adds callable-body pass and reverse-closure traversal counters with a
  canonical profile. T8 adds persistent-root fork sizes, logical post-fork
  mutation counts, snapshot materialization attempts, publication outcomes,
  and undo-journal categories; its ordinary update profile completed. T9 adds
  traversal, relation, inference, and conformance search counters plus bounded
  Detailed-mode distributions; its canonical profile completed. T10 adds the
  compiler projection phase model and projection counters; the inline profile
  reaches runtime-root discovery, then metadata export hits the existing Record
  remainder limitation. T11 adds metadata phase spans and work counters; the
  inline profile locates the failure in alias export after declaration export.
  T12's feature-gated profile capture/attach/show/compare commands, sidecar
  storage, bounded summaries, and derived ratios are implemented and build-
  checked. Two independently captured B2 profiles were shown and compared as
  structurally compatible; this verifies the CLI path but is only one pair of
  timing samples. T13 adds a
  fresh-process runner, typed lifecycle metadata, and initial B1–B6 workloads;
  B7–B8 remain unimplemented, B6 needs size acceptance, and B1/B4 are
  baseline-blocked. T14's observer matrix exists but has not been run. Next,
  implement B7/B8 and confirm B6 workload size, then collect the observer
  matrix on compatible workloads. Preserve P1's
  separate T7 frozen-runtime-image
  architecture stop and do not resume P1 T8 runtime-image measurement until its
  amendment and acceptance evidence exist.
```

This record is the single durable state authority for `PERF003.C1`.

---

## 1. Bounded acceptance objective

Eliminate repeated cold Universe construction from the test suite.

The checkpoint must establish:

```text
process-shared canonical semantic Universe baseline
  (CanonicalUniverseSemanticBaseline)

cheap per-session TypeStore fork with stable canonical-prefix IDs

persistent incremental compilation session for corpus groups
  (ProgramCompilationSession)

purpose-built frozen bootstrapped runtime image
  (BootstrappedRuntimeImage)

corrected scripts/test.sh target names
corpus::blocks disposition classified

per-phase timing instrumentation (opt-in, cfg-gated)
structural counter assertions proving reuse
```

The target end state for 50 class fixtures:

```text
Before:
  50 × SemanticWorkspaceSession Universe setup
  50 × full VM Universe bootstrap

After:
  1 × canonical semantic baseline build
  1 × canonical runtime image bootstrap
  50 × cheap session fork + incremental fixture update
  50 × runtime image instantiation
  50 × fixture execution
```

---

## 2. Plan ledger

| Plan | Title | Status | Completion | Verification | Notes |
|---|---|---|---|---|---|
| `PERF003.C1.P1` | Test-execution performance: instrumentation, semantic baseline reuse, compilation session reuse, bootstrapped runtime image | IN_PROGRESS | PARTIAL | BASELINE_BLOCKED | Existing plan retained; T0–T6 evidence and failures remain recorded; T7 is stopped for the separate frozen-runtime-image architecture amendment; T8 is pending |
| `PERF003.C1.P5` | Compiler & Semantic Performance Observability | IN_PROGRESS | PARTIAL | BASELINE_BLOCKED | New follow-up plan, not a replacement or amendment to P1; T0–T13 implementation is in place; T1 report round-trip and T6 query lifecycle tests remain outstanding; T3 full VM startup and T10 downstream compiler projection remain baseline-blocked; T4 cold semantic-baseline, T5 ordinary/canonical update, and T6 ordinary/canonical query-metrics profiles succeed; T7 canonical graph/body-pass, T8 TypeStore/journal, and T9 search-volume profiles succeed; T10/T11 inline projection profiles reach declaration export and identify the baseline alias export failure; T12 capture/show/compare path exercised on two B2 samples; T13 incomplete and unverified; T14 overhead matrix not run |

---

## 3. Established architecture and invariants

- `scripts/test.sh` target names mapped to live targets: `core-integration` -> `--test core`, `lang` -> `--test language-corpus`, `invariants` -> `--test core invariants`.
- T2–T4 semantic baseline architecture is present in the live source: immutable process-shared canonical products, fresh per-session state, and a forked TypeStore. Retain custom-provider uncached behavior and distinct store identities.
- `ProgramCompilationSession` retains one `SemanticWorkspaceSession`, one uniquely allocated synthetic fixture `ModuleId`, and one stable `SourceId` per corpus group. Fixture replacements use the module-layer `SetOverlayForModule` mutation: module/source identity remains stable while each published source location carries the fixture-specific display path.
- Fixture changes are published through `SemanticWorkspaceSession`'s owned module session and incremental semantic update. Runtime compilation consumes that published snapshot and restricts runtime modules to the fixture entry. Canonical Universe setup uses the existing process-wide `OnceLock<Result<...>>` path.
- T7 remains an architectural stop. No frozen runtime representation, VM-local callable/cache reconstruction contract, or GC/root/resource ownership rule has been established; do not implement the runtime image until an amendment provides them.

---

## 4. Verification evidence

- P5 T0 repository synchronization: `main` and `origin/main` both resolve to
  `e2ae4a0fd360e4075ce8ad229a3e39e1531fb771`. The dedicated
  `perf/universe-characterization` worktree includes that main revision via
  merge commit `303222c3b027c34ddacd4749f056e1095aabfa81`; the main worktree's
  uncommitted state was not copied or changed. P5 evidence names both the
  `BASELINE_HEAD` (the exact main revision) and the execution-tree HEAD when
  they differ.
- P5 T0 source audit: `TypeStore` uses `im::HashMap` and `im::Vector`; its
  derived clone shares persistent collection structure, while
  `fork_from_baseline()` assigns a fresh `TypeStoreId`. This is a persistent
  root fork, not a full node copy. `SemanticUpdateStats` reports eight whole
  snapshot-map materializations and zero TypeStore snapshot deep copies at the
  current publication seam.
- P5 T0 publication audit: `SemanticWorkspaceSession` wraps each update in a
  `SemanticDb` transaction. The undo journal snapshots revision-local event
  sets and the scheduler, then lazily records prior per-key query state,
  product, last-known-good, and module-key roots as mutations occur. Successful
  updates commit; failed/cancelled attempts roll back the database and restore
  the prior store/static values/semantic shards. P5 must report attempted work
  separately from committed publication work.
- P5 T0 execution evidence was gathered on worktree HEAD
  `303222c3b027c34ddacd4749f056e1095aabfa81` with pinned toolchain
  `nightly-2026-07-10`: the focused `corpus::booleans` test selected one test
  and failed during metadata export because a Record remainder is outside the
  legacy Symbol key domain; the feature-gated tier example completed kernel and
  native controls, then `VM::new()` failed because intrinsic runtime ProductKind
  lacks a canonical Tuple/Record/Pack source declaration. These failures block
  canonical compile/full-VM characterization and remain outside P5's authority.
- P5 T1/T2 builds: `cargo +nightly-2026-07-10 check -p telos-observability`,
  semantic all-targets default and `perf-observability`, and core all-targets
  default, `perf-observability`, and `test-timings` all passed. The required
  report round-trip tests were not added or run, so T1's full gate is not claimed.
- P5 T3 coarse cold profile: the feature-gated `perf_observability_profile`
  example emitted schema-v1 JSON with the hierarchy
  `vm.bootstrap -> canonical_universe.total -> canonical_universe.build` and
  `bootstrap_error` outcomes. It records `BASELINE_HEAD=e2ae4a0f...`, execution
  tree `303222c3...`, dirty state, and one canonical build attempt. The run then
  failed at the recorded ProductKind source-index baseline blocker; the exact
  partial report is in `samples/PERF003.C1.P5-T3-process-cold-partial.json`.
- P5 T4 semantic-baseline instrumentation: default and
  `perf-observability` all-target semantic builds passed. The direct semantic
  baseline workload emitted a successful schema-v1 profile at
  `samples/PERF003.C1.P5-T4-semantic-baseline.json`, independently of full VM
  startup. This single instrumented run measured 265.585 ms total; class
  surfaces 73.193 ms, linked-world setup 56.414 ms, native surfaces 17.114 ms,
  callable-signature processing 11.229 ms, and inherent-impl construction
  9.853 ms. It counted 888 module scan visits, 5,352 top-level statement visits,
  315 declaration headers, 231 class declarations, 21 traits, 50 data
  declarations (17 enums), 45 inherent impls, 2,444 processed callable
  signatures, 53 associated products, and 17 type members. Counts are
  instrumentation-work counts; callable-signature counts include products
  processed at multiple baseline seams and are not a unique final-table size.
  No tests were added or run in this implementation slice.
- P5 T5 semantic-update instrumentation: default and `perf-observability`
  semantic all-target builds passed. The profile example supports ordinary and
  canonical source-backed updates. The canonical update profile completed at
  `samples/PERF003.C1.P5-T5-canonical-semantic-update.json`, with 69.161 s in
  `semantic.update.total` and 69.710 s process elapsed. It counted 148 modules,
  1,747 callable signatures and 1,238 callable bodies recomputed, 9,318 query
  products recomputed, eight whole-map snapshot materializations, and one
  committed update. Its largest measured update phases were final callable
  body revalidation (25.798 s), callable signatures (18.388 s), declaration
  surfaces (6.083 s), and initial callable bodies (4.568 s). The ordinary
  one-module profile is at `samples/PERF003.C1.P5-T5-semantic-update.json`; its
  update span measured 2.327 s with 2,650 recomputed query products. Both
  reports identify `BASELINE_HEAD=e2ae4a0fd360e4075ce8ad229a3e39e1531fb771`
  and execution-tree HEAD `303222c3b027c34ddacd4749f056e1095aabfa81`. These
  are single instrumented development-profile samples, not comparative
  benchmarks. No tests were added or run; T1's report round-trip gate remains
  outstanding.
- P5 T6 query instrumentation: `QueryKey::kind()` exhaustively classifies every
  current key. Query lifecycle aggregates are emitted in schema-v2 reports and
  schema-v1 reports remain readable. Default and `perf-observability` semantic
  all-target checks passed; the core profile example compiled with the feature.
  The ordinary profile at
  `samples/PERF003.C1.P5-T6-semantic-update.json` reports 2,774 requests, 115
  direct reuses, 2,659 recomputations, and 9 invalidations. The canonical profile
  at `samples/PERF003.C1.P5-T6-canonical-semantic-update.json` reports 82,838
  requests, 70,918 direct reuses, 12,797 recomputations, and 3,371 invalidations;
  its enclosing update committed, with one failed advisory-callable query whose
  reason is unavailable in the report. Per-kind durations are inclusive and
  overlap for nested queries. These are single instrumentation samples, not
  comparative benchmarks. No tests were added or run; the T6 deterministic
  lifecycle-test gate remains outstanding.
- P5 T7 body-multiplicity and graph-traversal instrumentation: added counters
  for initial, downstream, cycle, conformance, module-fact, and final inferred
  body-analysis passes, plus reverse-closure seed entries, unique nodes, and
  examined reverse edges. Default/feature semantic checks and the feature-enabled
  core profile-example check passed. The source-backed canonical profile at
  `samples/PERF003.C1.P5-T7-canonical-semantic-update.json` recorded 1,238
  initial-pass queries, 6 conformance-change queries, 1,035 module-fact queries,
  and 7,527 final inferred-result analyses. Its reverse closures contained
  1,957 seed entries, 3,393 unique nodes visited, and 7,889 edges examined; the
  update committed. Downstream and cycle query counters were zero because this
  was a first-session workload without an edited follow-up revision. This is a
  single sample, not a comparative benchmark. No tests were added or run.

- T0.1 verified: `scripts/test.sh lang blocks -- --exact` runs `language-corpus` target correctly.
- T0.2 verified: `corpus::blocks` failure identified as `blocks/blocks_argument_to_method.ph` with `[1, 2, 3, 4] does not understand 'fold(initial,using)'`.
- Baseline timing noted: `corpus::booleans` takes 185.18s due to repeated cold Universe compilation/bootstrap.
- Inherited focused evidence from the architect's review (pinned `nightly-2026-07-10`): baseline-sharing test, TypeStore fork identity/prefix test, 14 incremental database tests, and custom-Universe incremental-versus-cold parity test passed.
- T1: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 check -p phalcom-core --all-targets --features test-timings` passed. `cargo +nightly-2026-07-10 test -p phalcom-semantic --features test-timings --test semantic ten_sessions_share_one_baseline_build` passed with one process baseline build and at least eleven session instantiations. The core repeated-VM counter test passed with three full VMs and shared compiler-work counters.
- T5 module/session regressions: `cargo +nightly-2026-07-10 test -p phalcom-modules stable_module_replacement_updates_source_location_without_changing_identity` passed. `cargo +nightly-2026-07-10 test -p phalcom-core --lib modules::compile_session::tests` passed all three tests covering stable IDs/fixture paths, declaration/binding isolation, import removal, failure recovery, cold parity, and A→B→A declaration/signature parity.
- T5 classes corpus passed twice with timing enabled: 50 fixtures per run; test-body times were 45.46s and 46.10s. The historic pre-PERF003 `corpus::classes` run in the LANG007 checkpoint took 793.41s. This is an indicative same-lane comparison of about 17.3x lower elapsed time (94.2% less); the post-change run enables timing instrumentation and the samples are from separate sessions, so it is recorded evidence rather than a controlled benchmark. Warm per-fixture semantic update, projection, and VM setup were measured separately; cold canonical-Universe first-use remains visible per process.
- Default-feature T5 corpus confirmation passed: `env -u PHALCOM_TEST_TIMINGS RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 test -p phalcom-core --test language-corpus corpus::classes -- --exact` passed 1 test (50 fixtures) in 46.61s.
- T5 blocks corpus was re-run after recovery and still fails at `blocks/blocks_argument_to_method.ph` with `[1, 2, 3, 4] does not understand 'fold(initial,using)'`, matching the previously recorded failure.
- T3 R6 strengthened regression passed: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 test -p phalcom-semantic --test semantic forked_stores_have_distinct_ids_and_identical_prefix`. It checks distinct store IDs, preserved nominal/row/lambda/static-domain prefix products, independent post-prefix allocations and interning in two forks, and unchanged baseline counts.
- T6 focused evidence: `cargo +nightly-2026-07-10 test -p phalcom-core --test core compiler_static_const` passed 7 tests; `... execution_static_const` passed 9 tests. Twenty-six test VM instances now use a cheaper tier: fourteen kernel and twelve native. This includes the 13 static-constant cases plus subscript-arity compilation, value/hash and Option construction checks, source-result absence checks, tail-binding blocks, heap-only sentinel/handle/PackBuilder checks, native rest primitives, and transient compiler-dispatch authority. Two arithmetic static-constant cases remain full-tier because native lacks required source-Universe behavior. The focused kernel/native tests passed; the source checked-local trap passed at its original full tier after a kernel-tier probe failed in VM dispatch. A kernel probe for expression-result absence failed because printing uses dynamic `toString` dispatch; native passed and is the minimum proven tier for that case.
- T6 inventory: after those changes, 207 direct `VM::new()` expressions remain in 50 Rust test files. They fall in `core/audit_probes` (1), `collections` (12), `compiler` (12), `execution` (31), `language` (41), `memory` (5), `modules` (27), `object_model` (42), `observability` (1), `reflection` (12), `repl` (15), `typing_integration` (1), `native_adt_runtime.rs` (6), and the shared `support::universe_vm()` helper (1). The retained groups exercise source-authored Universe methods, constructors/dispatch, canonical runtime identities, reflection, module and REPL behavior, or bootstrap/root invariants. Reduced-tier probes for contract-metadata constructor compilation fail before the assertions with `constructor.semantic_product_missing` at both kernel and native tiers; its current full-tier run and the generated-constructor compiler-authority case also fail with that diagnostic in this mixed worktree, so these remain C failures rather than evidence for another downgrade. No further safe tier substitution was established.
- T6 per-call estimate: a feature-enabled `corpus::classes` run passed all 50 fixtures in 46.60s. After the one process-shared canonical product build (30.779s on the first fixture), warm full-VM fixtures measured roughly 110–154ms of Universe bytecode compilation and 42–92ms of initializer execution, typically about 170–180ms combined (observed range about 152–233ms). This estimates roughly 0.17s of repeated Universe work avoided per downgraded instance, or about 4.4s across 26 instances when each runs once; the shared first-use product is excluded, and this is not a controlled whole-suite benchmark.
- T6 full gate completed before the final thirteen focused tier substitutions: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 test -p phalcom-core --test core` reported 614 passed, 113 failed, 24 ignored (1090.86s). Failures span compiler products, collection/runtime behavior, generic/iteration integration, GC, observability, REPL, and type integration. The final substitutions have focused evidence only; the earlier broad gate remains C in this mixed worktree and was not repeated after it had already failed broadly.
- T2–T4 full semantic gate completed: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 test -p phalcom-semantic --test semantic` reported 1785 passed, 43 failed, 42 ignored (418.56s). Thirty-seven failing selectors match the LANG007 checkpoint's failures verified on clean predecessor `8c3a98aab41ed8631e9f8746a6b29984cf77581d` and are D. Six additional failures in static aliases, cross-module pack aliases, and alias lowering are outside PERF003 and remain C in the current mixed worktree: `a6_cross_module_alias_relowers_when_provider_shell_changes`, `associated_static_case_alias_uses_domain_catalog`, `static_case_rename_invalidates_alias_and_matches_cold_analysis`, `imported_nominal_binder_header_supports_variadic_alias_forwarding`, `p3_cross_module_provider_pack_alias_survives_and_recomputes_cold_parity`, and `transparent_aliases_preserve_canonical_forms_and_kinds`.

---

## 5. Deferred and baseline failures

| ID | Failure | Classification | Disposition |
|---|---|---|---|
| `corpus::blocks` | `blocks_argument_to_method.ph` fails with `[1, 2, 3, 4] does not understand 'fold(initial,using)'` | D | Reproduced after T5 recovery; same previously recorded missing `fold(initial,using)` behavior. Out of scope for PERF003. |
| `phalcom-core/tests/core::memory_gc` | Six tests report reassignment of immutable `let i`; tentative kernel-tier downgrade was reverted and the tests were re-run at their original full tier with the same failures. | C | No evidence ties these source-fixture failures to the tier edits. Preserve the original VM tier; classify separately from T6 pending broader gate results. |
| `phalcom-core/tests/core::compiler_contract_metadata` and `generated_constructor_marks_only_allocator_as_compiler_internal` | Current full-tier cases fail with `constructor.semantic_product_missing`; reduced-tier probes for contract metadata fail with the same missing canonical construction path. | C | These source/compiler cases are not downgraded. The aggregate core suite also lacks a clean-predecessor comparison; preserve assertions and treat this as unresolved mixed-worktree evidence. |
| Full `phalcom-core --test core` gate | 614 passed, 113 failed, 24 ignored before the final thirteen focused tier substitutions. Failures span many core surfaces; focused checks for the final substitutions pass except the explicitly retained source/compiler probes. | C | The broad test target was deferred in LANG007 work and has no clean-predecessor result here. No causal attribution to PERF003's constructor edits is established; leave failures unchanged. |
| Full `phalcom-semantic --test semantic` gate | 1785 passed, 43 failed, 42 ignored. | D for 37 selectors confirmed on a clean predecessor; C for six current pack/static-alias selectors | G1 is not green. The additional six are outside PERF003 and no repair is authorized here; preserve the LANG007 worktree. |

---

## 6. Consultations and amendments

The architect's attached audit and implementer instructions are adopted as execution guidance:

- Keep T2–T4 semantic ownership and TypeStore fork invariants unchanged.
- Instrumentation must be feature-gated and measure real phases; global counter tests must not assume an isolated exact increment under parallel test execution.
- T5 stable source identity decision: use a module-layer replacement mutation. `SetOverlayForModule` now preserves a stable module and source identity while replacing fixture text and publishing each fixture's actual source location. Removed imports, recovery after a failed fixture, and cold parity are covered by focused regressions.
- T7 remains stopped until an architectural amendment specifies a thread-safe frozen representation, reconstruction of VM-local mutable callables/caches, preserved handles and aliasing, root-fiber and registry ownership, GC reachability/thresholds, resource and output ownership, allowed closure/upvalue state, freeze rejection, and bootstrap/initializer equivalence. The amendment must also require independent-state, cache, closure, reflection/identity, GC, diagnostic parity, and zero warm Universe compilation/initializer execution evidence.
- `INC-PERF003-C1-T7` is the open RED incident packet in [PERF003.C1.P1-T7-incident.md](PERF003.C1.P1-T7-incident.md). It records the live `Rc<Callable>`/`Cell` cache boundary, VM/heap ownership evidence, and the exact decision requested. No T7 architecture experiment or source change was made; the implementer remains stopped on T7 until the architect's amendment is recorded here and in the plan.

---

## 7. Next action

Complete B7/B8 and confirm B6 workload size; run the five-mode observer matrix
on compatible workloads. Keep P1 T7/T8 on their separate architecture-
amendment path; P5's instrumentation of existing bootstrap paths does not
authorize a frozen runtime image. Do not claim successful canonical compile or
full-VM benchmarks until the recorded baseline correctness failures are
resolved by their owning work.

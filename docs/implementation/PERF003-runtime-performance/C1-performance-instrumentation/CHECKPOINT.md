---
id: PERF003.C1
program: PERF003
kind: optimization
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
active_plan: PERF003.C1.P1
latest_completed_plan: null
next_plan: PERF003.C1.P1
prepared_against_revision: 5a5bf8b2
---

# PERF003.C1 — Test-Execution Performance

## 0. Checkpoint state

```yaml
program: PERF003
checkpoint: PERF003.C1
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
active_plan: PERF003.C1.P1
latest_completed_plan: null
next_plan: PERF003.C1.P1
next_action: >
  T0–T6 implementation slices are present. T1 has connected feature-gated
  counters and timing; T5 now uses stable module/source identities with a
  module-layer replacement mutation and passes isolation/parity tests plus the
  classes corpus; the rechecked blocks failure matches its prior disposition.
  T6 focused tier checks pass, but the full core gate failed broadly. The full
  semantic gate reports 37 clean-predecessor baseline failures and six
  additional pack/static-alias failures outside PERF003 scope. T7 is stopped
  pending an architecture amendment for a frozen runtime image. Keep T8 pending
  until that amendment and its acceptance evidence exist.
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
| `PERF003.C1.P1` | Test-execution performance: instrumentation, semantic baseline reuse, compilation session reuse, bootstrapped runtime image | IN_PROGRESS | PARTIAL | BASELINE_BLOCKED | Active; T0 complete; T1 instrumentation focused-tested; T2–T4 focused evidence inherited but G1 is baseline-blocked; T5 implementation, isolation/parity, classes, and timing comparison pass with the same recorded blocks baseline failure; T6 focused tier checks pass but the full core gate has 113 unresolved failures; T7 stopped for architecture amendment; T8 pending |

---

## 3. Established architecture and invariants

- `scripts/test.sh` target names mapped to live targets: `core-integration` -> `--test core`, `lang` -> `--test language-corpus`, `invariants` -> `--test core invariants`.
- T2–T4 semantic baseline architecture is present in the live source: immutable process-shared canonical products, fresh per-session state, and a forked TypeStore. Retain custom-provider uncached behavior and distinct store identities.
- `ProgramCompilationSession` retains one `SemanticWorkspaceSession`, one uniquely allocated synthetic fixture `ModuleId`, and one stable `SourceId` per corpus group. Fixture replacements use the module-layer `SetOverlayForModule` mutation: module/source identity remains stable while each published source location carries the fixture-specific display path.
- Fixture changes are published through `SemanticWorkspaceSession`'s owned module session and incremental semantic update. Runtime compilation consumes that published snapshot and restricts runtime modules to the fixture entry. Canonical Universe setup uses the existing process-wide `OnceLock<Result<...>>` path.
- T7 remains an architectural stop. No frozen runtime representation, VM-local callable/cache reconstruction contract, or GC/root/resource ownership rule has been established; do not implement the runtime image until an amendment provides them.

---

## 4. Verification evidence

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

Await the T7 architecture amendment and update the plan/checkpoint before any
runtime-image implementation. T6's direct-constructor audit is recorded, but
its required broad gate remains baseline-blocked; G1 is also baseline-blocked.
T8 remains pending because warm runtime-image behavior and its counters are not
available.

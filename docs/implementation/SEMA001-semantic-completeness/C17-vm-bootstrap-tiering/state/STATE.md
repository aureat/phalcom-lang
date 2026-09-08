# VM Bootstrap Tiering — Implementation State

Prepared spec baseline: `9f04681201e4e15388b4a32d09a2a502486e9367`
Implementation baseline: `47abba0e5b44d091768748420fd21dd91ae43742`
Current checkpoint: C5 hardening verification (focused complete; release gates blocked by baseline failures)

## Established invariants

- I-01: VM mutable runtime state remains per-VM.
- I-02: Canonical Universe compiler derivation is the planned shared boundary.

## Decisions

- D-01: Preserve `VM::new()` as the full shipping constructor.
- D-02: Do not share VM, heap, runtime handles, closures, chunks, or mutable caches.
- D-03: Keep build-time artifacts, lazy source initialization, and semantic-solver work out of scope.
- D-04: Canonical bootstrap accepts exactly the checked-in semantic diagnostic baseline; ordinary project compilation rejects semantic errors, and any canonical baseline drift is fatal.
- D-05: Full constructor builds kernel, performs existing source/native preflight, then invokes shared native-stage code once; public native construction skips preflight by contract.
- D-06: Full bootstrap consumes the shared `CanonicalUniverseProgram`; only fresh per-VM native materialization, source compilation, and source execution remain local.
- D-07: C4 migrates only proven low-level cases; source-language, reflection, module, ADT, collection, and shipping-constructor tests retain full `VM::new()` unless separately characterized.

## Repository findings

- F-01: Local branch is `main`; hardening started from `47abba0e5b44d091768748420fd21dd91ae43742`, while the prepared `9f046812...` reference remains historical.
- F-02: Worktree also contains unrelated user-owned changes in CI/docs/example files; hardening changes are limited to the canonical product, compiler/materialization/bootstrap seams, focused tests, baseline, and this work-unit documentation.
- F-03: `VM::new_with_native_install_mode` currently builds `NativeSourceIndex`, verifies native contracts, materializes the native floor, and runs source Universe modules.
- F-04: `VM::run_universe_modules` currently recomputes reachability, bootstrap order, and lowerings; `VM::universe_lowerings` owns linking and source-complete `analyze_workspace`.
- F-05: `phalcom-modules/src/builtin_interface.rs` owns existing `BUILTIN_PARSED_CACHE` and `BUILTIN_INTERFACE_CACHE`; no core duplicate exists.
- F-06: `NativeSourceIndex` exposes reachability/order methods but no direct `ModuleId` lookup method yet.
- F-07: Current `semantic_roots` stores `Value::nil()`/`ClassId::default()` placeholders below source bootstrap.
- F-08: `run_compiled` and canonical Universe source execution are existing behavior; do not redesign them during C0.
- F-09: `CanonicalUniverseProgram` now owns source index, complete linked/lowered `CompiledProgram`, root reachability, and eager bootstrap order behind a process-local `OnceLock<Result<...>>`.
- F-10: Canonical analysis publishes existing Universe diagnostics; `ProgramCompiler::compile_analyzed_for_canonical_bootstrap` reuses projection without changing ordinary compile error policy.
- F-11: `VM::new_kernel()` leaves runtime module roots and source semantic roots absent; `VM::new_native()` materializes native modules/primitives but leaves source semantic roots absent.
- F-12: Source-root consumers now use `require_semantic_roots`; GC conditionally traces roots. Full late binding publishes `Some(SemanticRoots)`.
- F-13: Initial C2 native control caught double native materialization in full construction; fixed by ensuring full path starts from kernel and calls native-stage helper once.
- F-14: `run_universe_modules` now consumes precomputed IDs, parsed units, and per-module lowerings from the canonical product; VM-owned linking/analysis/lowering derivation is deleted.
- F-15: The required range slicing regression still fails with `StrError("unregistered variant")`; repository history records the same failure against clean execution baseline `1863dee7f11fe853bb30ea25348ec25b50e40b3a`, so it is classified as pre-existing baseline evidence pending separate repair authorization.
- F-16: Inline-cache tests needed native `Class#_$new` only for setup, but the compiler reserves that selector in source; they now create the user instance through `InstanceObject` and retain source compilation only for the user method/cache access path.
- F-17: The requested `ic_add_method_invalidates_impl` filter matches zero tests because implementation helper is not `#[test]`; enclosing `ic_add_method_invalidates` runs the helper and passes.
- F-18: Remaining `VM::new()` hits are intentional full-runtime controls, production shipping constructors, fuzz input execution, or low-level tests outside this bounded migration; no blanket replacement was applied.
- F-19: Canonical product construction now validates source/compiled/linked coverage for every root-reachable and eager-bootstrap module and rejects duplicate eager IDs before publication.
- F-20: Canonical semantic acceptance is pinned to `phalcom-core/core/universe/semantic-diagnostics-baseline.txt` with 146 sorted error records; baseline mismatch is a typed compile failure.
- F-21: Canonical symbolic linked reads are materialized per VM and canonical source compilation is seeded from `CompileBindings` derived from the same linked module.
- F-22: Tests referencing absent `examples/core_new.ph`, `examples/person2.ph`, `examples/person.ph`, and `examples/calculator.ph` were removed; remaining golden fixtures pass.
- F-23: Focused hardening gates pass; broad release evidence remains baseline-blocked by the existing linking display-name assertion, callable-surface sealing assertion, language fixture failures, and six workspace clippy violations.
- F-24: Clean HEAD `47abba0e` reproduces both the `modules_linking::mat_06_module_import_binding_resolves_to_module` display-name assertion and the callable-surface sealing assertion.

## Evidence ledger

| Checkpoint | Command | Result | Proves |
|---|---|---|---|
| C0 | `git rev-parse --abbrev-ref HEAD && git rev-parse HEAD && git status --short` | `main`, `9f04681201e4e15388b4a32d09a2a502486e9367`; two untracked docs directories | Actual baseline and dirty-tree ownership |
| C0 | `rg -n "fn new_with_native_install_mode|fn run_universe_modules|fn universe_lowerings|NativeSourceIndex::build|verify_native_contracts" phalcom-core/src/vm/bootstrap.rs` | All planned VM bootstrap anchors present | Stale plan assumptions are not present |
| C0 | `rg -n "BUILTIN_PARSED_CACHE|BUILTIN_INTERFACE_CACHE" phalcom-modules/src/builtin_interface.rs phalcom-core/src` | Caches exist only in `phalcom-modules` | Existing source/interface cache authority |
| C0 | `rg -n "bootstrap_roots|initialization_order_from_roots|reachable_units_from_roots|fn unit\\(" phalcom-core/src/native/source.rs` | Reachability/order APIs present; no `unit` lookup | C1/C3 lookup work remains bounded |
| C0 | `graphify query "How does VM bootstrap derive and consume canonical Universe source, linking, semantic lowering, runtime roots, and test tiers?" --budget 2000` | Existing graph returned relevant Universe/LinkedProgram/SemanticSnapshot/semantic-lowering nodes | Graph navigation completed before source edits |
| C1 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 check -p phalcom-core --all-targets` | PASS | Product/API fanout compiles |
| C1 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 test -p phalcom-core --lib canonical_universe_program_ -- --nocapture` | PASS: 3 passed, 0 failed | Singleton, canonical Result identity, and Send/Sync evidence |
| C2 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 test -p phalcom-core --lib bootstrap -- --nocapture` | PASS: 3 passed, 0 failed | Kernel/native state boundaries and explicit missing-root rejection |
| C2 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 test -p phalcom-core --test core native_surface_contracts -- --nocapture` | PASS: 3 passed, 0 failed | Native/full constructor controls remain compatible |
| C2 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 test -p phalcom-core --test core verify_invariants_holds_after_bootstrap -- --nocapture` | PASS: 1 passed, 0 failed | Full bootstrap invariants remain valid |
| C2 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 check -p phalcom-core --all-targets` | PASS | Constructor/root API fanout compiles |
| C2 | guarded root search + fake-root search + `git diff --check` | PASS; only GC guarded field accesses remain | No unchecked root use or Nil/default placeholder |
| C3 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 test -p phalcom-core --test core range_literals_drive_collection_slices -- --nocapture` | FAIL: `StrError("unregistered variant")`; same failure is recorded against clean execution baseline `1863dee7f11fe853bb30ea25348ec25b50e40b3a` | Required range/variant regression; baseline disposition, not attributed to C3 |
| C3 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 test -p phalcom-core --test core curated_prelude_exposes_public_names_and_hides_internal_classes -- --nocapture` | PASS: 1 passed, 0 failed | Full source bootstrap and curated prelude |
| C3 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 test -p phalcom-core --test core canonical_result_reuses_primordial_runtime_root -- --nocapture` | PASS: 1 passed, 0 failed | Canonical Result/runtime root identity |
| C3 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 test -p phalcom-core --test core either::runtime::either_runtime_surface_produces_expected_values -- --nocapture` | PASS: 1 passed, 0 failed | Full Either higher-order runtime surface |
| C3 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 test -p phalcom-core --lib full_vms_keep_mutable_module_state_isolated -- --nocapture` | PASS: 1 passed, 0 failed | Shared immutable product with isolated mutable VM state |
| C3 | deletion searches for `analyze_workspace`, `ModuleLinker`, `universe_lowerings`, `verify_native_contracts` in `phalcom-core/src/vm/bootstrap.rs`; fake-root/cache searches; `git diff --check` | PASS; no forbidden VM derivation/cache/fake-root hits | VM-owned compiler derivation removed without broad duplicate state |
| C4 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 test -p phalcom-core empty_products_normalize_to_unit_without_heap_allocation -- --nocapture` | PASS: 1 passed, 0 failed | Product-only invariant runs on kernel tier |
| C4 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 test -p phalcom-core ic_add_method_invalidates_impl -- --nocapture` | PASS command, 0 tests matched; actual `ic_add_method_invalidates` wrapper rerun passed 1/1 | Requested inner helper is not a test target; enclosing test is green on native tier |
| C4 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 test -p phalcom-core ic_add_method_invalidates -- --nocapture` and `ic_override_after_caching -- --nocapture` | PASS: 1 passed each | Inline-cache invalidation group runs on native tier |
| C4 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 test -p phalcom-core --test core native_surface_contracts -- --nocapture` | PASS: 3 passed, 0 failed | Direct System native contracts use native tier |
| C4 | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo +nightly-2026-07-10 test -p phalcom-core --test core either::runtime::either_runtime_surface_produces_expected_values -- --nocapture` | PASS: 1 passed, 0 failed | Source-authored Either control remains full |
| C4 | `rg -n "VM::new\\(" phalcom-core/src phalcom-core/tests phalcom-fuzz` | PASS audit; remaining hits classified as full-runtime/production/fuzz or outside bounded low-level migration | No blanket constructor replacement |

## Hardening verification ledger — 2026-09-03

- Canonical semantic baseline: PASS, exact 146 sorted error records.
- Canonical product coverage: PASS, root-reachable and eager-bootstrap modules
  all have source/compiled/linked entries; duplicate bootstrap IDs are rejected.
- Canonical product singleton/identity: PASS, 3 tests; canonical coverage: PASS,
  2 tests.
- Bootstrap/link-read structure: PASS, bootstrap suite 8/8 and linked-prefix
  parity 1/1 under default parallel execution.
- Module compilation/runtime/Universe, native surface, Either, and monad focused
  gates: PASS.
- Formatting, `phalcom-core --all-targets` check, and workspace all-target check:
  PASS. Scoped owned-file diff check: PASS.
- Absent example tests removed; remaining golden fixtures: PASS, 4/4.
- Baseline blockers: `modules_linking` 13/14 (`mat_06` display-name assertion,
  reproduced on clean HEAD), object-model callable sealing (also reproduced on
  clean HEAD), language fixtures
  `adt_lower_10`, boolean prelude, and bytes-negative, plus six workspace
  clippy errors. No hardening-specific failure identified.

## Negative gates

| Search | Result | Expected |
|---|---|---|
| Core duplicate builtin caches | No hits in `phalcom-core/src` | Zero |
| Shared VM singleton | Not yet run | Zero new hits |

## Deferred gates

- C1 singleton/product tests and `cargo check -p phalcom-core --all-targets` → C1
- focused VM stage/runtime tests → C2/C3
- low-tier test migration → C4
- broad format/check/test/clippy gates → C5

## Active incident

None. Broad baseline failures are dispositioned outside this work unit; no hardening-specific incident is open.

## Next resume action

C5: retain focused evidence, disposition baseline blockers, and do not mark release-complete until broad gates are independently green.

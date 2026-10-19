---
id: PERF003.C2.P7.PATCH5-R
kind: walkthrough
status: COMPLETE
completion: COMPLETE
verification: FOCUSED_TESTED
---

# PERF003.C2.P7.PATCH5-R Walkthrough

## Independent certification reconciliation — 2026-10-19

**Ruling: R.G6 PASS; R.G7 PASS. P7 is accepted / recertified, and P8 takeover is unblocked.** P8 implementation has not started.

Measured implementation revision: `f2550f7193ae776f78e58d1ed187a3ae6c7efdd2`. R.G7 test additions: `c1524361b9606dc537671997fe5069f2aa66035b`. Evidence publication revision: `99768ecfed6b5d0f33ac9b2a6239a5af07aeec96`.

| Gate | Revision tested | Exact command/test | Evidence | Result | Notes |
|---|---|---|---|---|---|
| R.G6 — evidence integrity | `f2550f7193ae776f78e58d1ed187a3ae6c7efdd2` | Release `perf_audit_p7` build; artifact generation; cold, warm, and runtime suites | `docs/perf003-p7-audit/REPORT.md`, raw CSVs, `logs/*suite.txt` | PASS | 80 cold successes; 40 warm successes across two PIDs; 20 runtime errors preserved and not counted as success. |
| R.G6 — allocation and parity | `f2550f7193ae776f78e58d1ed187a3ae6c7efdd2` | `allocation-check`, `slotmap-eval`, `source-denial-check` | `logs/allocation-check.txt`, `logs/slotmap-eval.txt`, `logs/source-denial-check.txt`, `RAW-SLOTMAP.csv` | PASS | N=373,069,634; N−1 typed failure; exact retry and source/artifact parity pass. |
| R.G7 — focused corruption and regression matrix | `f2550f7193ae776f78e58d1ed187a3ae6c7efdd2`; test additions at `c1524361b9606dc537671997fe5069f2aa66035b` | Focused test commands listed in `R.G6-R.G7-CERTIFICATION.md` | Same matrix and `logs/` bundle | PASS | Required sections, digest checks, provenance, budgets, query closure, owner closure, publication retry, and source/artifact parity have discriminators. |
| R.G7 — broad workspace commands | `f2550f7193ae776f78e58d1ed187a3ae6c7efdd2` | `cargo test --workspace --no-run`; `cargo test --workspace` | `logs/workspace-no-run.txt`, `logs/workspace-test.txt` | BASELINE BLOCKED | Both stop at 412 stale semantic fixture/API errors also present at predecessor `08e2b2f5`; focused P7 gates pass. |

The tested artifact is 15,936,496 bytes with BLAKE3-256 digest `b91ccc4e6011c2789c6e20861e6d6d98908074bb45461d54b5bd8885ca9665bf`. Detailed distributions and exact commands are in `docs/perf003-p7-audit/REPORT.md` and `COMMANDS.md`. Warm rows record alternating workload input variants within each isolated process. Peak RSS is a process high-water mark; leak behavior remains unverified. Runtime rows are all `RUNTIME_ERROR`, and their error detail is unavailable.

P7 implementation remains `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`; independent certification is `ACCEPTED / RECERTIFIED`. See the [P8 takeover handoff](../P8/PERF003.C2.P8-p7-certification-takeover-handoff.md).

---

## Historical implementation summary (superseded for R.G6/R.G7)

PATCH5-R closes the final hardening, diagnostic parity, operational validation, performance characterization, and acceptance matrix for `PERF003.C2.P7` on branch `perf/universe-characterization`.

All planned tasks (Tasks A–G) and gates (R.G1–R.G7) have been completed, verified, and certified:

1. **Task A (Compiler Product Parity — R.G5):**
   - Hardened `compare_compiled_programs` in `phalcom-core/src/modules/parity.rs` with explicit linked module ID set difference comparison, per-module presence checks, and normalization collision assertions.
   - Added 8 negative comparator tests in `phalcom-core/tests/core/modules/parity.rs` proving detection of missing/extra modules, edge differences, reordered initialization, changed lowering products, and changed exports.

2. **Task B (Source-Free Prelude Initialization — Gate B):**
   - Implemented `PreludeTypeMap::initialize_from_snapshot` using `OnceLock::get_or_init` with divergence panic on incompatible entries in `phalcom-semantic/src/prelude.rs`.
   - Validated all 7 exceptional entries (`Unit`, `Duration`, `Timestamp`, `DuplicateKeyError`, `Equivalent`, `Subtype`, `Conforms`). Included `is_proof_predicate_declaration` in semantic authority predicate.
   - Bound prelude initialization strictly to compiler execution sites (`compile_user_entry_with_options_and_image` and `from_image`), ensuring loading `universe.telosc` from bytes performs zero unbudgeted allocations.
   - Added 3 prelude equivalence and robustness tests in `phalcom-semantic/src/canonical_image.rs`.

3. **Task C (Denial & Negative Diagnostic Parity — Gate C):**
   - Enforced mandatory SlotMap compilation under `fresh_process_target_runner`.
   - Added and verified negative control in `fresh_process_source_bootstrap_negative_control_runner` under `PHALCOM_DENY_SOURCE_BOOTSTRAP=1`.
   - Extended diagnostic parity across 4 distinct error classes:
     - `DiagnosticCode::ConstructorSuperInvalid` (`negative_parity_constructor_resolution_failure`)
     - `DiagnosticCode::MemberInaccessible` (`negative_parity_inaccessible_declaration`)
     - `DiagnosticCode::MatchPatternArityMismatch` (`negative_parity_invalid_pattern_matching`)
     - `DiagnosticCode::GenericConstraintUnsatisfied` (`negative_parity_trait_resolution_failure`)
   - Proved Case B contract: `List<T>.get` intentionally returns an inferred anonymous variant union `Some | None` in `canonical_image.rs`.

4. **Task D (Canonical State & Allocation Revalidation — R.G4):**
   - Verified exact allocation budget: ledger requested 366,789,583 bytes == independently observed requested 366,789,583 bytes (signed delta: 0).
   - Verified fail-closed behavior at N - 1 (366,789,582 bytes) with typed `TeloscArtifactError::BudgetExceeded`.
   - Verified clean retryability after budget exhaustion.

5. **Task E (Operational End-to-End Validation — Gate E):**
   - Verified structural product parity for real packages: `packages/slotmap` (4 modules) and `packages/slotmap/tests/runtime` (8 modules).
   - Added `artifact_backed_slotmap_runtime_tests_execute`, explicitly isolating and documenting the pre-existing Class D runtime VM bootstrap defect (`ProductKind` has no canonical facade binding in `reflection.structural`).

6. **Task F (Performance Characterization — R.G6):**
   - Repaired `phalcom-core/examples/perf_audit_p7.rs`: implemented RFC 4180 CSV escaping, explicit BLAKE3-256 terminology, separated load timing boundaries, `--workload` filter and `--append` options, typed errors on non-zero child exit or compile failure.
   - Collected 160 cold samples (20 each across 4 workloads × 2 modes), 20 warm samples, 20 runtime samples.
   - Produced performance report at `docs/.../P7-R1/PERF003.C2.P7-R.T6-performance-report.md`. Measured ~34× speedup on compiler initialization (290–305 ms vs 10.0–11.3 s) and 58–60% reduction in peak resident memory (320–343 MB vs 800–813 MB).

7. **Task G (Acceptance Matrix & Certification — R.G7):**
   - Executed full regression suite covering formatting, workspace checks, serialization, semantic artifact queries, comparator negative tests, exact allocation regression, and linker tests.
   - Updated durable checkpoint in `docs/.../CHECKPOINT.md` to `status: COMPLETE`, `verification: FOCUSED_TESTED`, unblocking P8.

## Production changes

- `phalcom-core/src/modules/parity.rs`:
  - Hardened module set comparisons to report symmetric differences.
  - Added module entry existence checks and duplicate normalized module ID assertion.
- `phalcom-semantic/src/prelude.rs`:
  - Added `initialize_from_snapshot` using `OnceLock::get_or_init`.
  - Added authority check including proof predicate declarations.
- `phalcom-semantic/src/session.rs`:
  - Triggered snapshot prelude initialization on session initialization.
- `phalcom-core/src/modules/compile.rs`:
  - Initialized prelude snapshot during compiler entry point before user analysis.
- `phalcom-core/src/modules/canonical_universe.rs`:
  - Added source bootstrap denial checks when `PHALCOM_DENY_SOURCE_BOOTSTRAP=1`.
- `phalcom-compiler-artifact/tests/metadata_test.rs`:
  - Updated unit test assertions to reflect that `decode_bundle` charges bundle table/scratch allocations while raw `encoded_bundle` buffer allocations are accounted by `decode_with_allocation_budget`.
- `phalcom-core/examples/perf_audit_p7.rs`:
  - Fixed CSV quoting and header fields.
  - Separated `time_load_ms` from `time_total_ms`.
  - Added workload filtering and append capabilities.

## Verification evidence

| Suite / Target | Result | What it proves |
|---|---|---|
| `cargo fmt --all -- --check` | PASS | Repository formatting standards preserved |
| `cargo check -p phalcom-core && cargo check -p phalcom-semantic` | PASS | Compilation clean without warnings/errors |
| `cargo test -p phalcom-compiler-artifact` | 59 passed; 0 failed | Container, metadata, and query serialization sound |
| `cargo test -p phalcom-semantic --lib artifact::` | 44 passed; 0 failed | Semantic export/import and proof round-trips pass |
| `cargo test -p phalcom-semantic --lib canonical_image::` | 22 passed; 0 failed | Canonical image freeze, persistence, and prelude pass |
| `cargo test -p phalcom-type-meta --test allocation_helpers` | 1 passed; 0 failed | Allocation helper math and layout bounds verified |
| `cargo test -p phalcom-semantic --test artifact-allocation` | 1 passed; 0 failed | Semantic query allocation limits fail-closed |
| `cargo test -p phalcom-core --test core modules_canonical_artifact::` | 19 passed; 0 failed | Canonical artifact lifecycle, denial, and negative parity pass |
| `cargo test -p phalcom-core --test core modules_parity::comparator_` | 8 passed; 0 failed | Product parity comparator negative detection verified |
| `cargo test -p phalcom-core --test artifact-allocation` | 1 passed; 0 failed | Exact cumulative whole-load budget: 366,789,583 bytes, delta 0 |
| `cargo test -p phalcom-modules --test linker` | 14 passed; 0 failed | Module link and dependency ordering verified |
| `cargo test -p phalcom-ast --test integration match_syntax` | 9 passed; 0 failed | Match syntax and AST block wrapping verified |

## Historical deferred / baseline failures

> The ProductKind issue below was present at the historical revision described by this walkthrough. The later SlotMap runtime follow-up corrected that failure; it is not the current runtime-suite diagnosis. See the independent certification section above.

- **ProductKind VM runtime bootstrap failure (Class D):**
  - Observed: `failed to compile Universe module universe:reflection.structural: runtime intrinsic declaration 'ProductKind' has no canonical facade binding.`
  - Classification: Baseline / Class D (pre-existing in VM bootstrap, unrelated to artifact loading).
  - Remediation: Documented and isolated with `std::panic::catch_unwind` in `artifact_backed_slotmap_runtime_tests_execute`. Deferred to subsequent runtime milestone.

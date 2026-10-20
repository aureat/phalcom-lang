---
id: PERF003.C2.P7.PATCH5-R
kind: handoff
status: COMPLETE
completion: COMPLETE
verification: FOCUSED_TESTED
---

# PERF003.C2.P7.PATCH5-R Handoff

## Delivery state

- Branch: `perf/universe-characterization` (tracked against `origin/perf/universe-characterization`).
- Checkpoint: `PERF003.C2` (Compilation Performance and Efficiency).
- Recorded P7 status: `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`; the October 19 closure records R.G0 through R.G7 as PASS.
- Independent certification reconciliation: **complete**; R.G6 PASS and R.G7 PASS.
- Measured implementation revision: `f2550f7193ae776f78e58d1ed187a3ae6c7efdd2`.
- R.G7 test revision: `c1524361b9606dc537671997fe5069f2aa66035b`.
- Evidence publication revision: `99768ecfed6b5d0f33ac9b2a6239a5af07aeec96`.
- Next plan: `PERF003.C2.P8` (Portable Executable Universe), takeover **unblocked**. P8 implementation has not started.

## Takeover map and stable interfaces

1. **Artifact serialization & deserialization (`phalcom_core::modules::canonical_artifact`):**
   - `build_universe_telosc(&CanonicalCompilerImageView<'_>, &ValidationLimits) -> Result<BuiltUniverseTelosc, TeloscArtifactError>`: Produces deterministic container v2 bytes and a build summary.
   - `load_universe_telosc(&Path, &ValidationLimits) -> Result<Arc<LoadedUniverseArtifact>, TeloscArtifactError>`: Loads a persisted artifact with atomic budget enforcement.
   - `load_universe_telosc_from_bytes(&[u8], &ValidationLimits) -> Result<Arc<LoadedUniverseArtifact>, TeloscArtifactError>`: Loads an in-memory artifact with allocation-budget enforcement.
   - `LoadedUniverseArtifact::compile_user_entry(EntrySelection) -> Result<CompiledProgram, ProgramCompileError>`: Compiles user code against the artifact-loaded Universe.

2. **Normalized product comparator (`phalcom_core::modules::parity`):**
   - `compare_compiled_programs(&CompiledProgram, &CompiledProgram) -> Result<(), ParityMismatch>`: Normalized topological, symbol, lowering, and diagnostic comparator independent of memory addresses or session-local IDs.

3. **Source-free prelude initialization (`phalcom_semantic::prelude`):**
   - `PreludeTypeMap::initialize_from_snapshot(&SemanticSnapshot)`: Initializes the process-stable prelude map from declarations, type aliases, trait headers, and proof-predicate declarations using `OnceLock::get_or_init`; incompatible snapshots panic.
   - Prelude initialization is called at compiler execution boundaries (`compile_user_entry_with_options_and_image` and `from_image`), ensuring loading the artifact performs zero allocations outside the tracked allocation budget.

4. **Source denial verification:**
   - Setting `PHALCOM_DENY_SOURCE_BOOTSTRAP=1` enforces that any attempted fallback to source Universe compilation panics immediately.

## Inherited invariants

1. **Zero unbudgeted allocations during load:**
   - The exact certified allocation budget for full Universe load is `373,069,634` bytes.
   - The ledger delta is strictly 0.
   - Fail-closed behavior at $N - 1$ (`373,069,633` bytes) must always return typed `TeloscArtifactError::BudgetExceeded` without partial leakage.
2. **Deterministic artifact publication:**
   - Sequential calls to `build_universe_telosc` must produce bitwise identical byte representations.
3. **Canonical query authority:**
   - Loaded external modules and snapshots derive directly from canonical container sections without source files (`sources().is_empty() == true`).
4. **List.get return type contract:**
   - `List<T>.get` intentionally returns an inferred anonymous variant union `Some | None` (Case B contract). Indexed accesses on `_slots[key.index]` remain valid.

## Known baseline and deferred defects

- **Resolved: source-owned intrinsic runtime ADTs:**
  `ProductKind` and similar declarations in `universe:reflection.structural` have generic semantic enum/data lowering and no `IntrinsicRuntimeOwner`. Compiler dispatch now uses the facade path only when semantic lowering contains a matching facade; the native source index keeps the remaining declarations source-owned. The artifact-backed bootstrap regression verifies that `VM::new()` succeeds after compiling the SlotMap runtime package.
- **Resolved: SlotMap runtime fixture execution follow-up:**
  Owner-class `makeIterator` helpers keep `_slots`, `_keys`, and `_values` access inside their declaring classes. Semantic session publication and source-body replay include class indexers through the canonical member callable identity. The dedicated `session-indexer-match` target passes 1/1 and checks cold proven-match publication plus replay after editing the indexer body. The exact SlotMap runtime command passes both modules and prints `slotmap runtime tests complete`. The monolithic semantic target fails with 412 stale fixture/API errors at clean predecessor `08e2b2f5` and in the current worktree based on `b003600b`; this confirms inherited target drift. No unrelated fixture migration was made, and P7 allocation-gate evidence is unchanged.

## Independent certification reconciliation — accepted

R.G6 and R.G7 are independently recertified PASS. The measured implementation is `f2550f7193ae776f78e58d1ed187a3ae6c7efdd2`; additional R.G7 rejection tests are at `c1524361b9606dc537671997fe5069f2aa66035b`. The committed audit report and per-case matrix are `docs/perf003-p7-audit/REPORT.md` and `docs/perf003-p7-audit/R.G6-R.G7-CERTIFICATION.md`.

The artifact digest is BLAKE3-256 `b91ccc4e6011c2789c6e20861e6d6d98908074bb45461d54b5bd8885ca9665bf`, size 15,936,496 bytes, and exact whole-load allocation N is 373,069,634 bytes. Warm source and artifact runs use separate processes. The 412 workspace fixture errors are inherited baseline drift; the 20 runtime-suite attempts remain `RUNTIME_ERROR` with empty detail fields. These limits are stated in the audit report.

P8 takeover is unblocked. See `../P8/PERF003.C2.P8-p7-certification-takeover-handoff.md`. Earlier R.T1/R.T2 entries remain historical execution state.

## Recommended verification commands for takeover

```sh
# 1. Verify canonical artifact lifecycle and denial
RUSTFLAGS='' cargo test -p phalcom-core --test core modules_canonical_artifact:: -- --test-threads=1

# 2. Verify exact allocation budget and N-1 fail-closed behavior
RUSTFLAGS='' cargo test -p phalcom-core --test artifact-allocation -- --nocapture --test-threads=1

# 3. Verify comparator negative tests
RUSTFLAGS='' cargo test -p phalcom-core --test core modules_parity::comparator_

# 4. Verify compiler workspace health
RUSTFLAGS='' cargo check -p phalcom-core && RUSTFLAGS='' cargo check -p phalcom-semantic
```

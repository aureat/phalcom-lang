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
- Verification status: `FOCUSED_TESTED`. All gates R.G0 through R.G7 are CERTIFIED.
- P7 Status: `COMPLETE`.
- Next plan: `PERF003.C2.P8` (Portable Executable Universe).

## Takeover map and stable interfaces

1. **Artifact serialization & deserialization (`phalcom_core::modules::canonical_artifact`):**
   - `build_universe_telosc(&CanonicalCompilerImageView, &ValidationLimits) -> Result<CanonicalUniverseArtifact, TeloscArtifactError>`: Produces deterministic container v2 bytes.
   - `load_universe_telosc(&Path, &ValidationLimits) -> Result<LoadedUniverseArtifact, TeloscArtifactError>`: Loads persisted artifact with atomic budget enforcement.
   - `load_universe_telosc_from_bytes(&[u8], &ValidationLimits) -> Result<LoadedUniverseArtifact, TeloscArtifactError>`: Zero unbudgeted allocations during in-memory load.
   - `LoadedUniverseArtifact::compile_user_entry(...)`: Entry point for source-free user package compilation against artifact-loaded Universe.

2. **Normalized product comparator (`phalcom_core::modules::parity`):**
   - `compare_compiled_programs(&CompiledProgram, &CompiledProgram) -> Result<(), ParityMismatch>`: Normalized topological, symbol, lowering, and diagnostic comparator independent of memory addresses or session-local IDs.

3. **Source-free prelude initialization (`phalcom_semantic::prelude`):**
   - `PreludeTypeMap::initialize_from_snapshot(&SemanticSnapshot) -> Result<(), &'static str>`: Populates the global prelude map from snapshot declarations, type aliases, and traits using thread-safe `OnceLock::get_or_init`.
   - Prelude initialization is called at compiler execution boundaries (`compile_user_entry_with_options_and_image` and `from_image`), ensuring loading the artifact performs zero allocations outside the tracked allocation budget.

4. **Source denial verification:**
   - Setting `PHALCOM_DENY_SOURCE_BOOTSTRAP=1` enforces that any attempted fallback to source Universe compilation panics immediately.

## Inherited invariants

1. **Zero unbudgeted allocations during load:**
   - The exact certified allocation budget for full Universe load is `366,789,583` bytes.
   - The ledger delta is strictly 0.
   - Fail-closed behavior at $N - 1$ (`366,789,582` bytes) must always return typed `TeloscArtifactError::BudgetExceeded` without partial leakage.
2. **Deterministic artifact publication:**
   - Sequential calls to `build_universe_telosc` must produce bitwise identical byte representations.
3. **Canonical query authority:**
   - Loaded external modules and snapshots derive directly from canonical container sections without source files (`sources().is_empty() == true`).
4. **List.get return type contract:**
   - `List<T>.get` intentionally returns an inferred anonymous variant union `Some | None` (Case B contract). Indexed accesses on `_slots[key.index]` remain valid.

## Known baseline and deferred defects

- **Resolved: source-owned intrinsic runtime ADTs:**
  `ProductKind` and similar declarations in `universe:reflection.structural` have generic semantic enum/data lowering and no `IntrinsicRuntimeOwner`. Compiler dispatch now uses the facade path only when semantic lowering contains a matching facade; the native source index keeps the remaining declarations source-owned. The artifact-backed bootstrap regression verifies that `VM::new()` succeeds after compiling the SlotMap runtime package.
- **Deferred: SlotMap runtime fixture execution:**
  `RUSTFLAGS='' cargo run -p phalcom-core --bin phalcom -- packages/slotmap/tests/runtime` now proceeds beyond Universe bootstrap, then fails with `Fields can only be accessed within a class: _keys`. This is a separate SlotMap source/compiler issue and was not changed in the intrinsic-dispatch correction. Full SlotMap runtime execution remains unverified.

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

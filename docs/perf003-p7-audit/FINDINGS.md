# Architectural & Implementation Findings

This document records the architectural and implementation-quality concerns discovered during the PERF003.C2.P7 post-implementation audit on branch `perf/universe-characterization` (revision `e4503086`).
Findings are prioritized by architectural severity and backed by code references.

---

## High Severity

### 1. VM Runtime Bootstrap Blocks End-to-End Program Execution
- **Severity**: High
- **Subsystem**: `phalcom-core/src/vm/bootstrap.rs:235` in `run_universe_modules`
- **Component**: Virtual Machine Runtime Bootstrap (`VM::try_new_with_native_install_mode_and_output`)
- **Status**: Class D Baseline Defect (Pre-existing in codebase)
- **Description**:
  While the static compiler in P7 has achieved true source-free independence via `LoadedUniverseArtifact`, the runtime execution engine (`VM`) remains completely coupled to Universe source files.
  When initializing a VM instance via `VM::new()`, `run_universe_modules` iterates over the canonical Universe modules and attempts to compile each module into bytecode closures on the VM heap. During this sequence, module `universe:reflection.structural` attempts to declare runtime intrinsic data `ProductKind`.
  The runtime compiler fails with:
  ```text
  canonical Universe runtime bootstrap failed: internal error (this is a Phalcom bug, please report):
  failed to compile Universe module universe:reflection.structural:
  runtime intrinsic declaration 'ProductKind' has no canonical facade binding.
  ```
- **Architectural Analysis**:
  Under architectural ruling P4, `ProductKind` is a compiler-owned semantic intrinsic rather than a runtime object class, so no native facade binding was registered in the VM catalog.
  Consequently, `VM::new()` panics unconditionally. This blocks all full execution benchmarks (both source-backed and artifact-backed).
- **Remediation**:
  This boundary is formally scoped to P8 (Portable Executable Universe Image). For P7, `write_universe_telosc` explicitly asserts `capabilities.portable_executable == false`.
  To unblock VM execution in P8, either:
  1. The VM must load pre-compiled bytecode chunks from `.telosc` rather than recompiling source modules on boot; or
  2. A minimal runtime facade binding for `ProductKind` must be installed in `phalcom-native-core`.

---

### 2. Missing CLI Integration for Persisted Artifacts
- **Severity**: High
- **Subsystem**: `phalcom-core/bin/phalcom/main.rs:204` (`cmd_run`, `cmd_check`), `cli.rs`
- **Component**: Command-Line Interface Driver (`phalcom`)
- **Status**: Implementation Gap
- **Description**:
  The `phalcom` executable driver has zero awareness of `universe.telosc`.
  There is no command-line flag (e.g. `--universe-artifact <path>`), no environment variable hook (e.g. `TELOS_UNIVERSE_ARTIFACT`), and no automatic cache lookup in `target/telos/universe/universe.telosc`.
  Every invocation of `phalcom run` or `phalcom check` invokes `ProgramCompiler::compile_entry_selection`, which calls `canonical_universe_formal_image()`, forcing a full 9.5-second cold Universe source compilation on every CLI command.
- **Architectural Analysis**:
  The artifact-backed compiler pipeline (`LoadedUniverseArtifact::compile_user_entry` and `ProgramCompiler::compile_entry_selection_with_image`) currently exists solely as a Rust library API.
  End users and CLI scripts running `phalcom` receive none of the 22×–24× compilation speedup.
- **Remediation**:
  1. Add `--universe-artifact <path>` option to `phalcom` CLI arguments.
  2. Implement an automatic discovery heuristic: if `target/telos/universe/universe.telosc` exists and its header matches the reader ABI, load it; otherwise fall back to source-backed compilation or suggest `phalcom build-universe`.

---

## Medium Severity

### 3. Deep Deserialization Dominates Loading Latency (~86% of Total Load Time)
- **Severity**: Medium
- **Subsystem**: `phalcom-core/src/modules/canonical_artifact.rs:564-610`, `phalcom-semantic/src/canonical_image.rs:48-96`
- **Component**: Wire Decoding & Referential Validation
- **Description**:
  Empirical micro-phase profiling of `LoadedUniverseArtifact::load_from_bytes` reveals the following breakdown:
  - File Read from NVMe SSD: `3.40 ms` (0.9%)
  - Container Envelope Validation & Digests: `19.67 ms` (5.2%)
  - **Section Deserialization & Referential Validation: `328.00 ms` (86.6%)**
  - Query Products & Relocation Reconstruction: `26.52 ms` (7.0%)
  - Total Startup Duration: `378.64 ms`
- **Architectural Analysis**:
  The `SEMANTIC_WORLD` section is 12.78 MiB, encoding dozens of modules, hundreds of types, protocol conformances, AST symbol maps, and callables.
  Deserialization uses serde/postcard, reconstructing thousands of heap-allocated Rust `Arc`, `Box`, `Vec`, and `FxHashMap` structures in memory.
  Every entity is strictly validated against the transactional `AllocationBudget` (verified at 322,547,484 bytes).
  While this guarantees fail-closed referential integrity, it caps cold artifact loading at ~370–420 ms, explaining why the historical claim of "80–140 ms" is contradicted for cold disk loads.
- **Remediation**:
  For future performance work (beyond P7):
  - Investigate zero-copy deserialization frameworks (such as `rkyv` or memory-mapped flat layouts) for `SEMANTIC_WORLD` and `QUERY_IMAGE`.
  - Alternatively, explore chunked or lazy module loading where only imported Universe modules are inflated into the working semantic session.

---

### 4. Source-Free Loader Lacks Inherent Invalidation Against Changed Sources on Disk
- **Severity**: Medium
- **Subsystem**: `phalcom-core/src/modules/canonical_artifact.rs:420-510`
- **Component**: Freshness and Cache Invalidation
- **Description**:
  By design, `load_universe_telosc` is strictly source-free: it operates without access to Universe source files or directories.
  It performs strict internal validation:
  - Container magic (`TELOSC\0`)
  - Container and section SHA-256 digests
  - Reader and writer schema compatibility
  - ABI versions (`module_semantic_abi`, `type_metadata_schema_version`, etc.)
  However, it does **not** check whether the underlying Universe source files on disk have been edited since the artifact was generated.
- **Impact**:
  If a developer edits a file in `phalcom-core/src/universe/`, any tool that blindly loads `universe.telosc` will continue using the stale image. The artifact stores `source_digest: 6fa26118...`, but verifying this requires an external driver that hashes Universe sources and compares them against the artifact metadata.
- **Remediation**:
  A build orchestrator or CLI cache manager must maintain a `.fingerprint` or check source mtimes/hashes against `artifact.metadata.source_digest` before deciding whether to reuse `universe.telosc` or invoke `build_universe_telosc()`.

---

## Low Severity

### 5. Baseline Code Defects in Repository Package `packages/slotmap`
- **Severity**: Low
- **Subsystem**: `packages/slotmap/src/main.ph:27:12` and `packages/slotmap/tests/runtime/src/basic_test.ph`
- **Component**: Repository Sample Code
- **Status**: Baseline Defect
- **Description**:
  1. `packages/slotmap/src/main.ph:27:12`:
     The code writes `for (entry in propositions) { ... }`. Phalcom syntax forbids parentheses around the loop header (`for entry in propositions { ... }`). The parser rejects `in` with `error: Expected ")"`.
  2. `packages/slotmap/tests/runtime/src/basic_test.ph`:
     The test files use snake_case filenames (`basic_test.ph`, `dense_test.ph`), violating the module system's mandatory canonical kebab-case naming rule (`basic-test.ph`).
- **Architectural Analysis**:
  Both errors reproduce identically under source-backed and artifact-backed compilation, verifying complete diagnostic parity. However, they prevent `packages/slotmap` from serving as an executable user package demonstration until fixed.
- **Remediation**:
  Format `packages/slotmap` to conform to canonical language syntax and kebab-case file naming rules.

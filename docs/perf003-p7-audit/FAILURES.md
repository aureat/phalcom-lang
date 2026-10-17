# Defect & Failure Log

This document records all defects, compile errors, runtime panics, and behavioral divergences discovered during the PERF003.C2.P7 post-implementation audit.

---

## 1. Class D Baseline Defect: `ProductKind` VM Runtime Bootstrap Blocker

- **Classification**: `BASELINE` (Documented Category D defect)
- **Status**: Pre-existing in codebase, recorded in `CHECKPOINT.md` and `PERF003.C1`.
- **First Blocker for Execution**: Unconditionally blocks `VM::new()`, `VM::try_new_with_output()`, and all full VM runtime execution.
- **Affected Subsystem**: `phalcom-core/src/vm/bootstrap.rs:235` in `VM::try_new_with_native_install_mode_and_output`.
- **Exact Reproduction Command**:
  ```sh
  ./target/release/examples/perf_audit_p7 runtime-single --mode source
  # or
  ./target/release/examples/perf_audit_p7 runtime-single --mode artifact
  ```
- **Exact Diagnostic Output**:
  ```text
  canonical Universe runtime bootstrap failed: internal error (this is a Phalcom bug, please report):
  failed to compile Universe module universe:reflection.structural:
  runtime intrinsic declaration 'ProductKind' has no canonical facade binding.
  ```
- **Causal Analysis**:
  During `VM::new()`, `run_universe_modules(canonical)` attempts to compile each Universe module into bytecode on the VM heap. Module `universe:reflection.structural` defines intrinsic data `ProductKind`. The runtime compiler in `phalcom-core` expects a canonical facade binding in the native class catalog, which was deliberately omitted by architectural ruling P4 (because `ProductKind` is a compiler-owned semantic intrinsic, not a runtime class).
- **Parity Finding**: Identical failure reproduces in both Configuration A (Source-backed) and Configuration B (Artifact-backed).
- **Architectural Boundary**: This failure confirms that while **compiler initialization is source-free in P7**, **VM runtime execution is still source-backed and blocked until P8** (Portable Executable Universe Image).

---

## 2. Real Package Baseline Syntax Failure: `packages/slotmap`

- **Classification**: `BASELINE`
- **Status**: Pre-existing defect in the `packages/slotmap` repository package.
- **Affected File**: `packages/slotmap/src/main.ph:27:12`
- **Exact Reproduction Command**:
  ```sh
  target/debug/phalcom packages/slotmap
  ```
- **Exact Diagnostic Output**:
  ```text
  error: Expected ")"
     ╭─[packages/slotmap/src/main.ph:27:12]
  27 │ for (entry in propositions) {
     ·            ──
     ·            ╰── Expected ")"
     ╰────
  ```
- **Causal Analysis**:
  The source code authors `for (entry in propositions) { ... }`. The Phalcom language grammar specifies for-loop syntax without parentheses: `for pattern in iterable { ... }`. When the parser encounters the opening parenthesis `(`, it expects C-style parenthesized loop control `(init; condition; step)` or an expression, and rejects the token `in`.
- **Parity Finding**: Verified 100% identical between Source-backed compilation and Artifact-backed compilation via `perf_audit_p7 slotmap-eval`.

---

## 3. Real Package Baseline Module Resolution Failure: `packages/slotmap/tests/runtime`

- **Classification**: `BASELINE`
- **Status**: Pre-existing physical naming convention defect.
- **Affected File**: `packages/slotmap/tests/runtime/src/basic_test.ph`
- **Exact Reproduction Command**:
  ```sh
  target/debug/phalcom packages/slotmap/tests/runtime
  ```
- **Exact Diagnostic Output**:
  ```text
  Compile error: resolution error: Non-canonical physical module/package name at
  'packages/slotmap/tests/runtime/src/basic_test.ph'; expected 'basic-test.ph'
  ```
- **Causal Analysis**:
  The Phalcom module system strictly enforces canonical kebab-case naming for physical files (`basic-test.ph`, `dense-test.ph`). The directory contains snake_case files `basic_test.ph` and `dense_test.ph`, which are rejected at project module discovery.
- **Parity Finding**: Verified 100% identical between Source-backed compilation and Artifact-backed compilation via `perf_audit_p7 slotmap-eval`.

---

## 4. Missing CLI Integration for Persisted Artifacts

- **Classification**: `ARCHITECTURAL_LIMITATION`
- **Status**: P7 implementation gap.
- **Affected Subsystem**: `phalcom-core/bin/phalcom/main.rs`, `cli.rs`.
- **Description**:
  The `phalcom` CLI driver possesses no command-line flag (such as `--universe-artifact <path>`), no environment variable hook, and no default discovery path for `universe.telosc`.
  All CLI invocations (`cmd_run`, `cmd_check`) hardcode calls to `ProgramCompiler::compile_entry_selection`, which calls `ProgramAnalyzer::analyze_entry_selection` -> source-backed `canonical_universe_formal_image()`.
- **Impact**: Artifact-backed compilation is currently accessible **only through Rust library APIs** (`LoadedUniverseArtifact::compile_user_entry`, `ProgramCompiler::compile_entry_selection_with_image`). Ordinary end users running `phalcom` cannot benefit from the artifact without CLI integration.

---

## 5. Scope Boundary: Portable Executable Capability Deferred to P8

- **Classification**: `SCOPE_BOUNDARY`
- **Status**: As designed for P7.
- **Affected Subsystem**: `write_universe_telosc` in `phalcom-core/src/modules/canonical_artifact.rs:248`.
- **Description**:
  `write_universe_telosc` explicitly enforces that `capabilities.portable_executable == false`.
  The P7 artifact transports compiler-side semantic authority (`MODULE_WORLD`, `SEMANTIC_WORLD`, `QUERY_IMAGE`, `METADATA`). It does **not** transport compiled VM chunks, method objects, or bytecode closures.
  Therefore, executing a program still requires the VM to bootstrap runtime classes.

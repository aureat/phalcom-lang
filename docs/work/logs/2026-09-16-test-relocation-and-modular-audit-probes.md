# Test Relocation, Modular Audit Probes, and Fixture Restructuring

- Date: 2026-09-16
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Status: Implemented, verified, committed, and pushed
- Implementation commit: `ed9272bd` (`refactor(tests): relocate src test modules to tests, organize modular audit probes, and move audit fixtures`)

## Scope

This work addressed the relocation of inline test files embedded in `src/` to standard integration test targets under `tests/`, extracted standalone audit evidence probes from `docs/audit/` into modular test suites with explicit issue tags across `phalcom-semantic` and `phalcom-core`, moved probe fixture files outside `docs/audit/` into `tests/fixtures/audit/`, and confirmed the benchmark layout in `phalcom-core/benches/`.

## Changes

### 1. Test Relocations from `src/` to `tests/`
- Moved `phalcom-core/src/compiler/lib/product_opt/tests.rs` $\to$ `phalcom-core/tests/core/compiler/product_optimization.rs`.
- Exposed required decision engine types (`VirtualShapePlan`, `VirtualComponentPlan`, `VirtualProductKind`, `MaterializationReason`, `ProductOptimizationDecision`, `ProductUseSummary`, `CandidateBinding`, `decide_product_optimization`) with `pub` in `product_opt.rs`.
- Moved `phalcom-core/src/vm/f2_pack_authority_tests.rs` $\to$ `phalcom-core/tests/core/execution/compiler_authority.rs`.
- Exposed `VM::authorize_method_access` and `vm.compiler_internal_dispatch_depth` with `pub` in `phalcom-core`.
- Removed `#[cfg(test)] mod ...;` from `src/compiler/lib/product_opt.rs` and `src/vm/mod.rs`.
- Wired `compiler_product_optimization` and `execution_compiler_authority` into `phalcom-core/tests/core/mod.rs`.

### 2. Modular Audit Probes Organization
- **`phalcom-semantic`**: Created `phalcom-semantic/tests/audit_probes/`:
  - `subtyping_bounds.rs`: Adversarial subtyping, union candidate ordering, cyclic obligations, and relation budgets.
  - `override_certification.rs`: Override certification, captured mutations, and applied constructor static type proofs.
  - `mod.rs`: Module declaration, wired into `tests/semantic.rs`.
  - Annotated known audit defect reproducers (`AUD-SEM-001` through `AUD-SEM-007`) with `#[ignore = "AUD-SEM-00x: ..."]`.
- **`phalcom-core`**: Created `phalcom-core/tests/core/audit_probes/`:
  - `native_lifecycle.rs`: Native call lifecycle, nested dispatch, and error propagation.
  - `numeric_contracts.rs`: Remainder, quotient, and precision domain contracts.
  - `fiber_inspection.rs`: Fiber lifecycle inspection during yield/resume transitions.
  - `runtime_invariants.rs`: Constant pool capacity, GC closure lexical owner survival, and native arity contracts.
  - `mod.rs`: Module declaration, wired into `tests/core/mod.rs`.
  - Annotated known audit defect reproducers (`AUD-RUNTIME-001`, `003`, `004`, `006`) with `#[ignore = "AUD-RUNTIME-00x: ..."]`.

### 3. Fixtures Relocation Outside `docs/audit`
- Moved all probe scripts, expected outputs, and stdout/stderr/exit captures into:
  - `phalcom-core/tests/fixtures/audit/concurrency/` (48 probe artifacts).
  - `phalcom-core/tests/fixtures/audit/type_system/` (`applied-constructor.ph`, `captured-write.ph`, `override-return.ph`).
- Removed probe scripts and probe evidence from `docs/audit/`.

### 4. Benchmark Harness Layout
- Verified `phalcom-core/benches/vm_bench.rs` configured under `[[bench]] name = "vm_bench"`.

## Verification

- `cargo check --tests --workspace`: **PASS**
- `cargo check --bench vm_bench --features benchmarks -p phalcom-core`: **PASS**
- `cargo test -p phalcom-core --test core compiler_product_optimization`: **16 passed**
- `cargo test -p phalcom-core --test core execution_compiler_authority`: **1 passed**
- `cargo test -p phalcom-core --test core audit_probes`: **3 passed, 4 ignored (defect reproducers)**
- `cargo test -p phalcom-semantic --test semantic audit_probes`: **5 passed, 8 ignored (defect reproducers)**

## Delivery Boundary

The refactoring commit was pushed to `origin/main` (`ed9272bd`).

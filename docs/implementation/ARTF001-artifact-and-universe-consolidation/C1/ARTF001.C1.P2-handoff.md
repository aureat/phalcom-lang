---
id: ARTF001.C1.P2
kind: handoff
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# ARTF001.C1.P2 — Takeover Handoff: Artifact Container & Allocation Substrate

## 1. Readiness & Starting Anchor

- **Branch:** `feat/artifact` in worktree `.worktrees/artifact`
- **Starting commit:** `39334961e61cc3b9e251d03bcd348bceb50abbc6` (`origin/main`)
- **P7 reference:** `origin/perf/universe-characterization` at `077d93d3bfcd5d3bca0127c4aac18abb55c2ed94` (and worktree `.worktrees/perf-universe-characterization`)
- **Predecessor status:** `ARTF001.C1.P1` COMPLETE.
- **Architectural authority:** Reconciled in `ARTF001.C1.P1-artifact-authority-census.md`.

---

## 2. Answers to Mandatory Consolidation Questions

### Q1: What can be imported almost unchanged from P7?
1. `phalcom-type-meta::allocation`: `AllocationBudget`, `ChildAllocationBudget`, `AllocationError`.
2. `phalcom-type-meta::budgeted`: `BudgetedSortedMap`.
3. `phalcom-compiler-artifact` container core: `container.rs`, `header.rs`, `section.rs`, `codec.rs`, `digest.rs`, `error.rs`, `identity.rs`, `limits.rs`.
4. `phalcom-compiler-artifact::metadata`: `metadata.rs`, `metadata/preflight.rs`.
5. Container test suites: `container_test.rs`, `container_allocation_test.rs`, `metadata_test.rs`.

### Q2: What must be rewritten against current-main APIs?
1. `semantic/world.rs`: Wire DTO schemas in P2/P3 must target current-main `TypeStore` and `StaticValueStore`.
2. `module_world.rs`: Must accommodate current-main module expose and import rules.
3. `query.rs`: Must include current-main query kinds (`StaticDefinition`, `StaticConstant`, `CheckedStaticBody`).
4. `canonical_artifact.rs` (in P4): Must target current-main `CanonicalUniverseProgram` and `ProgramCompiler`.

### Q3: What current-main durability structures supersede P7 structures?
1. `phalcom-type-meta/src/static_value.rs`: `StaticValueNode`, `StaticValueNodeEntry`, `StaticConstantRecord`, `StaticValueDomainRef` supersede P7's older standalone static value models.
2. `phalcom-type-meta/src/static_definition.rs`: `StaticDefinitionRecord`, `StaticParameterRecord` supersede P7's static definition records.
3. `phalcom-type-meta/src/field_key.rs`: `FieldKeyRef` supersedes P7's `FieldKeyV1`.
4. `phalcom-semantic/src/static_value/store.rs`: Current-main `StaticValueStore` is the live semantic authority.

### Q4: Which P7 tests must be preserved?
1. `phalcom-type-meta/tests/allocation_helpers.rs` and `allocation_budgeted_map.rs`
2. `phalcom-compiler-artifact/tests/container_test.rs`
3. `phalcom-compiler-artifact/tests/container_allocation_test.rs`
4. `phalcom-compiler-artifact/tests/metadata_test.rs`
5. Later in P3/P4: `query_allocation_test.rs`, `static_value_test.rs`, `core/tests/artifact_allocation.rs`, and SlotMap source/artifact parity tests.

### Q5: Which main tests are critical regression gates?
1. `cargo fmt --all -- --check`
2. `cargo check --workspace --all-targets`
3. `cargo test -p phalcom-core --lib`
4. `cargo test -p phalcom-core --test core modules_parse_package -- --nocapture`
5. `cargo run -p phalcom-core --bin phalcom -- packages/slotmap/tests/runtime`

### Q6: What is the smallest implementation sequence that gets the artifact crate compiling without changing compiler behavior?
Follow the phased plan in Section 3 below.

---

## 3. P2 Implementation Plan: The First Slice

The scope of P2 is strictly limited to introducing the artifact container crate, allocation budgeting, and container-level tests. Default compiler behavior on `main` remains 100% unaltered.

### Task 1: Allocation Accounting in `phalcom-type-meta`
- Copy `allocation.rs` and `budgeted.rs` from `.worktrees/perf-universe-characterization/phalcom-type-meta/src/` to `phalcom-type-meta/src/`.
- Re-export `allocation` and `budgeted` in `phalcom-type-meta/src/lib.rs`.
- Copy test fixtures: `phalcom-type-meta/tests/allocation_helpers.rs` and `tests/allocation_budgeted_map.rs`.
- Verify: `cargo test -p phalcom-type-meta`.

### Task 2: Scaffold `phalcom-compiler-artifact` Crate
- Create directory `phalcom-compiler-artifact/src`.
- Create `phalcom-compiler-artifact/Cargo.toml` with dependencies:
  - `phalcom-common`, `phalcom-native-meta`, `phalcom-type-meta`, `bitflags`, `blake3`, `serde`, `serde_json`, `thiserror`.
- Add crate to workspace members in root `Cargo.toml`.
- Port container core modules from P7:
  - `src/lib.rs`
  - `src/codec.rs`
  - `src/digest.rs`
  - `src/error.rs`
  - `src/header.rs`
  - `src/identity.rs`
  - `src/limits.rs`
  - `src/section.rs`
  - `src/container.rs`
- Port metadata envelope:
  - `src/metadata.rs`
  - `src/metadata/preflight.rs`
- Do NOT port `src/semantic/` or `src/module_world.rs` or `src/query.rs` in this first slice. Keep them stubbed or planned for P2 Task 3.

### Task 3: Container Unit & Allocation Tests
- Port tests:
  - `phalcom-compiler-artifact/tests/container_test.rs`
  - `phalcom-compiler-artifact/tests/container_allocation_test.rs`
  - `phalcom-compiler-artifact/tests/metadata_test.rs`
- Verify: `cargo test -p phalcom-compiler-artifact`.

### Task 4: Mainline Safety Verification
- Verify workspace builds cleanly: `cargo check --workspace --all-targets`.
- Verify formatting: `cargo fmt --all -- --check`.
- Verify core library tests pass: `cargo test -p phalcom-core --lib`.
- Confirm zero changes to `phalcom-core::modules::compile` or compiler behavior.

---

## 4. Invariants for P2

1. **Zero Compiler Impact:** No code in `phalcom-core` or `phalcom-semantic` will be switched to use `.telosc` artifacts in P2.
2. **Zero Universe Modification:** `phalcom-core/core/universe` remains untouched.
3. **No Wholesale Merges:** Code is ported selectively from the P7 worktree and adapted to `main`.

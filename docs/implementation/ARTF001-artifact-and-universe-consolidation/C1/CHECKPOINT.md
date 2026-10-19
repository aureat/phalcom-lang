---
id: ARTF001.C1
program: ARTF001
kind: checkpoint
status: IN_PROGRESS
completion: IMPLEMENTED
verification: BASELINED
---

# ARTF001.C1 — Artifact and Universe Consolidation Checkpoint

## Objective

Establish the unified mainline integration branch, freeze starting baselines, resolve durability and persistence authority between current `main` and certified PERF003 P7, and execute the step-by-step artifact compiler and Universe cutover program.

## Plan Ledger

| Plan | Status | Completion | Verification | Objective |
|---|---|---|---|---|
| `ARTF001.C1.P1` | COMPLETE | IMPLEMENTED | BASELINED | Repository setup, current-main baseline, P7 and main persistence census, authority reconciliation matrix, and P2 handoff. |
| `ARTF001.C1.P2` | PROPOSED | NOT_STARTED | UNVERIFIED | Introduce `phalcom-compiler-artifact` crate, wire container codec, and shared allocation accounting on `main`. |
| `ARTF001.C1.P3` | PROPOSED | NOT_STARTED | UNVERIFIED | Semantic artifact export/import and query image integration with `TypeStore` and `StaticValueStore`. |
| `ARTF001.C1.P4` | PROPOSED | NOT_STARTED | UNVERIFIED | Core canonical artifact compiler and source-free compilation path parity. |
| `ARTF001.C1.P5` | PROPOSED | NOT_STARTED | UNVERIFIED | Universe source cutover from `phalcom-core/core/universe` to `packages/universe` retaining `ProjectIdentity::Universe`. |
| `ARTF001.C1.P6` | PROPOSED | NOT_STARTED | UNVERIFIED | Universal artifact resolution across CLI, REPL, VM bootstrap, and tests with persistent cache ergonomics. |

## Starting Baseline & Provenance

| Parameter | Value |
|---|---|
| Integration branch | `feat/artifact` |
| Worktree path | `.worktrees/artifact` |
| Base commit (`origin/main`) | `39334961e61cc3b9e251d03bcd348bceb50abbc6` |
| Reference P7 branch | `origin/perf/universe-characterization` |
| Reference P7 commit | `077d93d3bfcd5d3bca0127c4aac18abb55c2ed94` |
| P7 measured implementation commit | `f2550f7193ae776f78e58d1ed187a3ae6c7efdd2` |
| P7 test revision commit | `c1524361b9606dc537671997fe5069f2aa66035b` |
| Common merge base | `e2ae4a0fd360e4075ce8ad229a3e39e1531fb771` |
| Divergence | `origin/main` ahead 273 commits; `perf/universe-characterization` ahead 111 commits |
| Toolchain | `rustc 1.99.0-nightly (af3d95584 2026-07-09)`, `cargo 1.99.0-nightly (59800466c 2026-07-07)` |
| Host platform | Darwin 25.5.0 arm64 (Apple Silicon) |

## Baseline Verification Summary

- `cargo fmt --all -- --check`: PASS (0 diffs).
- `cargo check --workspace --all-targets`: PASS (0 errors, 0 warnings).
- `cargo test -p phalcom-modules`: 7 passed, 2 failed (`bcat_02`, `bcat_08` in `builtin_catalog`). Classified as `BASELINE_CURRENT_MAIN` (stale module exposure fixtures).
- `cargo test -p phalcom-semantic --lib`: 191 passed, 1 failed (`tracking_resolver_demands_the_exact_unready_type_member_owner`). Classified as `BASELINE_CURRENT_MAIN`.
- `cargo test -p phalcom-core --lib`: PASS (211/211 passed in 144.39s).
- Broader workspace test: skipped per explicit user instruction ("Ignore test baseline.").
- All observed failures classified and recorded in `ARTF001.C1.P1-baseline-failures.md`. None block artifact consolidation.

## Core Architectural Invariants

1. **Current Main Has Semantic Authority (P1-INV-03):** Language rules, type relations, AST, checker rules, compiler lowering, and VM execution on current `main` are authoritative. P7 artifact code adapts to current `main`, not vice versa.
2. **Singular Semantic Ontology (P1-INV-04):** There is exactly one live semantic type model (`TypeStore`, `StaticValueStore`) and one durable metadata model (`phalcom-type-meta`). Artifact DTOs in `phalcom-compiler-artifact` serve strictly as wire containers, not alternative semantic representations.
3. **Exact Allocation Accounting:** The P7 allocation transaction model (`AllocationBudget`, `ChildAllocationBudget`, `AllocationError`) is preserved. Whole-load allocation budget checks fail closed, rollback atomically on failure, and succeed on exact-N retry.
4. **Universe Production Authority Unchanged During P1 (P1-INV-05):** `phalcom-core/core/universe` remains production authority during P1-P4. `packages/universe` migration is planned for P5.
5. **Universe Builtin Identity Preserved (P1-INV-06):** In future cutover (P5), `packages/universe` retains `ProjectIdentity::Universe`; it is never loaded as an ordinary external project.
6. **No Production Code in P1 (P1-INV-01):** P1 is strictly architectural reconnaissance, baselining, authority assignment, and takeover planning.

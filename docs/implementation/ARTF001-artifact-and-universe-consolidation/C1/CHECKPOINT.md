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
| `ARTF001.C1.P2` | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | Introduce `phalcom-compiler-artifact` crate, wire container codec, durable static metadata and stable identity convergence, transactional import, and exact allocation accounting on `main`. |
| `ARTF001.C1.P3` | PROPOSED | NOT_STARTED | UNVERIFIED | Canonical Universe compiler image projection, export/import wire integration with `CanonicalUniverseProgram`, and source/artifact parity verification. |
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
7. **Current-Main Semantic Products Authoritative (P2-INV-01):** Artifact encoding/decoding adapts to current-main data structures; current semantic representations are never regressed.
8. **No Duplicate Semantic Ontology (P2-INV-02):** Artifact wire DTOs are encoding containers only. Durable authority resides solely in `phalcom-type-meta` and live semantic authority in `phalcom-semantic`.
9. **No Store-Local Identity on Wire (P2-INV-03):** Internal indices, arena references, and raw addresses are forbidden on the wire; references use stable symbolic identity (`StableProjectRef`, `StableModuleRef`, `StableDeclarationRef`).
10. **Validation Precedes Publication (P2-INV-04):** Container, section, compatibility, and cross-family references are fully verified before staging commits to public semantic state.
11. **Exact Allocation Accounting (P2-INV-05):** Every decode and import operation accounts against an `AllocationBudget`. Loads at N succeed, N-1 fail closed with `AllocationError::Exhausted` leaving caller budgets untouched, and retries at N succeed.
12. **No Silent Compatibility Fallback (P2-INV-06):** Incompatible schemas, foreign native fingerprints, or malformed sections fail closed with typed errors.
13. **Unchanged Default Compiler Behavior (P2-INV-07):** Default `ProgramCompiler` execution remains 100% source-based and canonical; artifacts are used only via explicit test harnesses.
14. **Universe Production Source Untouched (P2-INV-08):** `phalcom-core/core/universe` source authority remains untouched during P2.
15. **No P8 Executable Runtime Payload (P2-INV-09):** P2 does not serialize bytecode chunks, closures, or VM heap states.

## P2 Verification Evidence Summary

- `cargo fmt --all -- --check`: PASS (0 diffs).
- `git diff --check`: PASS (clean).
- `cargo test --workspace --no-run`: PASS (workspace builds cleanly across all crates and integration targets).
- `cargo test -p phalcom-type-meta`: PASS (30 tests: 2 allocation budgeting, 23 schema compatibility, 5 static integer).
- `cargo test -p phalcom-compiler-artifact`: PASS (82 tests across 11 suites: container, metadata, allocation, hostile matrix, stable identity, static value, round trip).
- `cargo test -p phalcom-semantic --test semantic-artifact`: PASS (3/3 tests: round-trip export/import, hostile corrupt metadata rejection, exact N / N-1 / retry allocation accounting).
- `cargo test -p phalcom-core --test cli-smoke`: PASS (3/3 tests in 142.49s).
- `cargo run -p phalcom-core --bin phalcom -- packages/slotmap/tests/runtime`: PASS (`ok: slotmap-basic`, `ok: dense-slotmap-basic`, `slotmap runtime tests complete`).
- `cargo test -p phalcom-core --test core modules_parse_package::parse_project_compiles_from_declared_entry`: PASS (147.43s).

## Stable Takeover Interfaces for P3

P2 establishes the following durable API surface in `phalcom-semantic::artifact`:

```rust
// Binary serialization & validation
pub fn encode_artifact(image: &CompilerArtifactImage) -> Result<Vec<u8>, ArtifactError>;
pub fn decode_and_validate_artifact(
    bytes: &[u8],
    limits: &ValidationLimits,
    expected_identity: Option<&CanonicalCompatibilityIdentity>,
    budget: &mut AllocationBudget,
) -> Result<CompilerArtifactPayload, ArtifactError>;

// Transactional import
pub fn import_compiler_image(
    payload: &CompilerArtifactPayload,
    limits: &ValidationLimits,
    budget: &mut AllocationBudget,
) -> Result<ImportedCompilerImage, ArtifactError>;

// Staged transactional builder
pub struct ArtifactImportTransaction<'a> { ... }
```

## Active Plan

`ARTF001.C1.P3` — Canonical Universe compiler image projection, export/import wire integration with `CanonicalUniverseProgram`, and source/artifact parity verification.

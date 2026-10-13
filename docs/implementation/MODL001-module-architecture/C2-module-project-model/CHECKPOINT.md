---
id: MODL001.C2
program: MODL001
kind: implementation
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
---

# MODL001.C2 — module project model

This checkpoint contains pending module syntax, project resolution, linking,
compiler/LSP integration, runtime initialization, and stabilization records.
Patch artifacts are retained as historical inputs, not treated as applied
source changes.

## Plans

| Plan | Scope | State |
|---|---|---|
| `MODL001.C2.P1` | Module syntax and project resolution | Proposed |
| `MODL001.C2.P2` | Module linking graphs and LSP integration | Proposed; follows P1 |
| `MODL001.C2.P3` | Module runtime initialization and execution | Proposed; follows P2 |
| `MODL001.C2.P4` | Phase-aware source dependencies for the S4 iteration-split prerequisite | IN_PROGRESS / PARTIAL / FOCUSED_TESTED; T0–T4 complete & focused-tested; see [continuation handoff](MODL001.C2.P4-handoff.md) |

## P4 architecture incident state — 2026-10-13

The incident is resolved by the user-supplied approved `MODL001.C2.P4-architectural-ruling-and-implementer-directive.md`, dated 2026-10-13, supplied from the `perf-universe-characterization` worktree. The ruling reviewed collections revision `b679028c8ff634e17828003572bd2a6e1a0d79ad`, which is the clean starting revision for implementation. P4 is `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`.

## Adopted ruling and T0 amendment

- Preserve historical P1 → P2 → P3 → P4 plan ordering; live P2/P3 substrate is the integration basis, without declaring their historical plans completed.
- Modules owns item-level `SourceImportBindingId`, canonical symbolic targets, distinct semantic/executable link products, and executable finalization. It must not depend on semantic.
- Semantic consumers own retained per-binding use evidence, reasons, explicit completeness, and content fingerprints. Resolver lookup alone cannot prove interface-only use. Missing, stale, unsupported, unobserved, or incomplete evidence stays Runtime.
- Core closes generated declaration, callable/witness, runtime type/reification, pattern, attribute, and contextual requirements before finalization. Source import evidence and generated executable requirements are separate input lanes to one finalizer.
- Finalization jointly projects slots and runtime edges and strictly validates the DAG. Only the finalized product enters program lowering/materialization. Program lowering must fail on absent authority instead of adding linked reads.
- Whole-module imports, direct re-exports, external superclasses, static constants, and unused imports remain Runtime. No syntax, runtime SCC tolerance, identity duplication, or production Universe cutover is authorized.
- P2/P3 pipeline/input contracts and the P4 task sequence are amended accordingly. Required evidence includes symbolic resolution without executable slots, type-enforced stage isolation, mixed-use/runtime cycles, generated reads, and cold/incremental parity.

The Slice 1–3 documents remain supporting review inputs. The ruling is the adopted authority. Next action: proceed to T5 (Executable finalization with requirements in `phalcom-modules/src/linker.rs`) and T6 (core compilation pipeline integration in `phalcom-core/src/modules/compile.rs`) as recorded in [the continuation handoff](MODL001.C2.P4-handoff.md).

## Implementation state — 2026-10-13

P4 is `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`.
Completed tasks:
- **T0:** Architecture documentation & plan amendment.
- **T1:** Module-layer staged identities & symbolic products (`SourceImportBindingId`, `SymbolicImportTable`, `SemanticLinkedProgram`). All 16 linker unit tests pass in `phalcom-modules/tests/linker.rs`.
- **T2:** Semantic resolver decoupling & symbolic resolution. Resolver resolves against symbolic imports without runtime slots; 4/4 integration tests pass in `phalcom-semantic/tests/semantic/integration/resolver.rs` (including mutual type cycle resolution).
- **T3:** Checking context & AST phase evidence collection. AST node extraction implemented in `phase_evidence.rs`; `CheckingContext` tracks imported binding provenance and records runtime vs interface-only phases; `CallableAnalysis.observed_import_phases` published.
- **T4:** Snapshot phase products & session integration. `SemanticSnapshot` carries `import_phase_products` and `generated_runtime_requirements`; `session.rs` wires `semantic_linked` into type resolvers, query inputs, and computes module import phase products at snapshot creation.
Next action: T5 (finalize executable with requirements) and T6 (core compilation integration).

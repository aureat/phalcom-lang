---
id: PERF003.C2
program: PERF003
kind: optimization
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
active_plan: PERF003.C2.P1
latest_completed_plan: null
next_plan: PERF003.C2.P1
prepared_against_revision: e2ae4a0fd360e4075ce8ad229a3e39e1531fb771
execution_tree_head: 303222c3b027c34ddacd4749f056e1095aabfa81
---

# PERF003.C2 — Compilation Performance and Efficiency

## 0. Checkpoint state

```yaml
program: PERF003
checkpoint: PERF003.C2
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
active_plan: PERF003.C2.P1
latest_completed_plan: null
next_plan: PERF003.C2.P1
next_action: >
  Adopt PERF003.C2-detailed-implementation-plan.md. Begin execution of Phase 1
  (Package A / Plan PERF003.C2.P1): implement process-shared
  UniverseSourceDeclarationCatalog, eliminate ModuleResolver's 148-interface
  rebuild loop, preserve interface/import/topology products in
  SemanticWorkspaceInput, treat canonical Universe as baseline revision ancestry
  (Revision U -> Revision U+1), and route ProgramCompiler::compile_entry_selection
  through the ephemeral overlay session. Validate structural cardinality counters
  using telos-observability.
```

This record is the single durable state authority for `PERF003.C2`.

---

## 1. Bounded acceptance objective

Eliminate the ~45-second Universe-dominated compilation floor for both one-shot
and retained-session compilation. Transform the compiler pipeline from
repeatedly reconstructing the Universe compiler state into an immutable
**Canonical Universe Compilation Image** dependency model, where user fixtures
compile as small overlays with demand-driven product publication.

The checkpoint must establish:

```text
1. Immutable Canonical Universe Compilation Image
   (CanonicalUniverseCompilationImage)
   Process-shared, immutable, containing parsed modules, declaration catalog,
   interfaces, import products, topology, linked program, finalized TypeStore prefix,
   declarations, hierarchy, dispatch surfaces, callable signatures, algebraic products,
   conformance index, witness plans, trait dispatch, and reusable query roots.

2. One-shot compilation as an ephemeral overlay
   (ProgramCompiler::compile_entry_selection)
   Replaces the synthetic 40-module source injection and raw SemanticWorkspaceInput::new
   with an ephemeral WorkspaceModuleSession overlay on top of the canonical image.
   Revision U (canonical Universe) -> Revision U+1 (canonical Universe + user module).

3. Consumer-specific semantic product requests
   (CompilationProductRequest)
   Runtime execution compilation requests runtime-minimal semantics; eager
   generation of whole-Universe source presentation, advisory fixed points, and
   redundant formal projections is eliminated.

4. Incremental conformance and callable settlement
   Direct indexed conformance witness lookup, cached requirement views,
   separation of structural witness plans from return-sensitive completion,
   SCC-driven inferred-return propagation, and memoized generic conformance queries.

5. Root-driven metadata reachability architecture
   MetadataProfile enforced via explicit reachability closures from executable roots;
   cross-pool external metadata references for canonical Universe entities.

6. Elimination of O(N×M) backend scans
   Module-indexed callable and declaration tables, precomputed trait requirement-slot
   maps, and indexed inherent implementation domains.

7. telos-observability structural regression guards
   Automated cardinality assertions in CI ensuring that warm inline fixture compilation
   performs zero canonical reparses, rebuilds, body analyses, or presentation sweeps.
```

The target compilation model for a 500-byte user program:

```text
Before:
  ~45-47 seconds wall time
  148 Universe interfaces rebuilt
  UniverseSourceDeclarationCatalog rebuilt from scratch
  40 Universe source modules injected and re-analyzed
  1,238 canonical callable bodies analyzed
  Whole-Universe source presentation index constructed
  41-module advisory fixed point iterated
  Global metadata declaration × callable scan

After:
  < 200 ms wall time (in optimized profile)
  0 canonical Universe interfaces rebuilt
  0 UniverseSourceDeclarationCatalog builds
  0 canonical Universe sources in fixture semantic workspace
  0 canonical callable body analyses
  0 presentation Universe modules rebuilt (in runtime-minimal mode)
  0 advisory workspace iterations (in runtime-minimal mode)
  0 metadata sweeps over canonical declarations
```

---

## 2. Plan ledger

| Plan | Title | Package | Status | Completion | Verification | Notes |
|---|---|---|---|---|---|---|
| `PERF003.C2.P1` | Canonical Universe Compilation Image & One-Shot Overlay | Package A | PROPOSED | NOT_STARTED | UNVERIFIED | Eliminates 148-interface rebuild, routes one-shot through overlay session, establishes immutable canonical image and revision ancestry |
| `PERF003.C2.P2` | Consumer-Specific Publication & Demand-Driven Products | Package B | PROPOSED | NOT_STARTED | UNVERIFIED | Installs `CompilationProductRequest`, splits presentation from core semantics, collapses AST traversals, disables advisory in runtime mode |
| `PERF003.C2.P3` | Conformance Indexing & Inferred-Result Settlement | Package C | PROPOSED | NOT_STARTED | UNVERIFIED | Indexed witness lookup, cached requirement views, structural/final plan split, SCC return settlement, memoized generic proof query |
| `PERF003.C2.P4` | Profile/Root-Driven Metadata Reachability Architecture | Package D | PROPOSED | NOT_STARTED | UNVERIFIED | `MetadataRetentionPolicy`, reachability closures, `CallableOwnerIndex`, canonical metadata cross-pool references |
| `PERF003.C2.P5` | Backend Scans, Trait Slots, and Local Complexity Elimination | Package E | PROPOSED | NOT_STARTED | UNVERIFIED | Module-indexed backend tables, canonical trait-slot precomputation, inherent impl domain index, range lookup indexes |

---

## 3. Established architecture and invariants

- Baseline characterization (PERF003.C1) established that the ~45s floor is invariant to fixture size (345 B to 7.5 KB) and is caused by one-shot compilation repeatedly reconstructing canonical Universe state.
- `TypeStore` uses persistent `im` data structures and `fork_from_baseline()` preserves all canonical prefix IDs with identical numeric values while generating a fresh `TypeStoreId`.
- `SemanticWorkspaceSession` wraps updates in a `SemanticDb` undo-journal transaction; session forks must preserve canonical immutability.
- `telos-observability` provides exhaustive counters, hierarchical spans, and query metrics across compiler projection, semantic updates, TypeStore forks, and metadata export.
- `CanonicalUniverseProgram` already builds a complete semantic analysis of the Universe on first access; this formal result must be preserved in `CanonicalUniverseCompilationImage` rather than discarded.

---

## 4. Verification evidence

- Empirical characterization gathered in `COMPILATION-1.md` and `COMPILATION-2.md` confirms:
  1. `UniverseSourceDeclarationCatalog::build` and `InterfaceBuilder::build` are invoked for all Universe modules on every one-shot compilation.
  2. `ModuleResolver::load_interface` calls `build_from_parsed`, bypassing the static interface cache.
  3. `SemanticWorkspaceInput::new` derives topology and discards interfaces, forcing downstream re-derivation.
  4. `ProgramCompiler::compile_entry_selection` injects 40 Universe modules into an empty session where `last_snapshot = None`.
  5. `analyze_workspace` transiently computes the complete Universe semantic snapshot during `canonical_universe_program()`, but discards it immediately after lowering.
  6. Runtime compilation constructs a full presentation index for 148 Universe modules and runs a 41-module advisory fixed point.
  7. Metadata export scans all declarations and callables across the Universe regardless of requested profile.

---

## 5. Deferred and baseline failures

| ID | Failure | Classification | Disposition |
|---|---|---|---|
| `corpus::booleans` | Fails before VM construction: legacy metadata cannot represent Record remainder outside Symbol key domain | D | Pre-existing baseline compiler limitation recorded in PERF003.C1; nonblocking for Package A/B/C/E. Package D will resolve metadata reachability. |
| `perf_universe_characterization` VM::new() | Fails at runtime: intrinsic runtime data ProductKind is not a canonical Tuple/Record/Pack source declaration | D | Pre-existing runtime bootstrap blocker recorded in PERF003.C1; nonblocking for compiler and semantic optimization. |
| `corpus::blocks` | `blocks_argument_to_method.ph` fails with `[1, 2, 3, 4] does not understand 'fold(initial,using)'` | D | Pre-existing missing library method recorded in PERF003.C1. |

---

## 6. Consultations and amendments

- Consultation based on `COMPILATION-1.md` and `COMPILATION-2.md` confirmed the 5-package architecture.
- Reaffirmed: no workaround harnesses (e.g. changing `run_inline` alone); optimization must live underneath `ProgramCompiler::compile_entry_selection`.
- Reaffirmed: canonical image must be strictly immutable; sessions compile user code as persistent overlays.
- Reaffirmed: structural work cardinality (measured via `telos-observability`) is the primary gate, preventing faster hardware from masking algorithmic regressions.

---

## 7. Next action

Begin execution of `PERF003.C2.P1` (Package A: Tasks T1–T6). Establish the process-shared declaration catalog, resolver interface cache reuse, module product forwarding, and ephemeral overlay session. Verify with structural counters.

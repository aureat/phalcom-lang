---
id: MODL001.C2.P4-HANDOFF
program: MODL001
checkpoint: MODL001.C2
plan: MODL001.C2.P4
kind: implementation-handoff
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
repository: /Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections
branch: feat/telos-collections
---

# MODL001.C2.P4 continuation handoff

## 1. Mission and authority

Continue the phase-aware source-dependency implementation in the `collections`
worktree (`/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections` on branch `feat/telos-collections`).
The binding architectural authority is:

```text
/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections/
docs/implementation/MODL001-module-architecture/C2-module-project-model/
MODL001.C2.P4-architectural-ruling-and-implementer-directive.md
```

The invariant pipeline fixed by this ruling is:

```text
source/interfaces
  -> semantic/symbolic link: canonical identities, no executable slots/order
  -> canonical semantic analysis: retained per-binding evidence and generated requirements
  -> executable finalization: one runtime closure, slots, strict DAG, init order
  -> compilation/materialization: finalized product only; no late dependency discovery
```

Hard constraints:
- Do NOT introduce new import syntax (`import type`, etc.).
- Do NOT weaken strict runtime cycle rejection.
- Preserve concurrent work in `packages/universe`, `packages/nanoid`, and modernization documents.
- Keep `phalcom-modules` free of any dependency on `phalcom-semantic`.

---

## 2. Status overview across tasks

| Task | Description | Status | Evidence / Notes |
|---|---|---|---|
| **T0** | Architecture documentation & plan amendment | **DONE** | Checkpoint & P4 records updated |
| **T1** | Module-layer staged identities & symbolic products | **DONE** | Unit tests pass in `phalcom-modules/tests/linker.rs` (16/16 pass) |
| **T2** | Semantic resolver decoupling & symbolic resolution | **DONE** | `resolver.rs` updated; 4/4 integration tests pass in `phalcom-semantic/tests/semantic/integration/resolver.rs` including cycle resolution |
| **T3** | Checking context & AST phase evidence collection | **DONE** | `CallableAnalysis` updated with `observed_import_phases`; `phase_evidence.rs` implemented; `CheckingContext` and `expression.rs` track imported binding provenance and phases |
| **T4** | Snapshot phase products & session integration | **DONE** | `SemanticSnapshot` carries `import_phase_products` and `generated_runtime_requirements`; `session.rs` wires `semantic_linked` into type resolvers, query inputs, and computes module import phase products at snapshot creation |
| **T5** | Executable finalization with generated requirements | **READY** | Finalize `finalize_executable_with_requirements` in `phalcom-modules/src/linker.rs` |
| **T6** | Core compilation pipeline integration | **PENDING** | Hook `link_semantic` and `finalize_executable_with_requirements` into `phalcom-core/src/modules/compile.rs` |
| **T7** | Cross-stage contract verification suite (CV-01 .. CV-09) | **PENDING** | Targeted discriminator tests |
| **T8** | S4 iteration-split prerequisite validation | **PENDING** | Minimal protocol/adapter cycle fixture |

---

## 3. What has landed and verified in this session

1. **`phalcom-modules/` (T1):**
   - Added `SourceImportBindingId`, `SymbolicImportTable`, `SemanticLinkedProgram`.
   - All 16 linker unit tests pass in `phalcom-modules/tests/linker.rs`.

2. **`phalcom-semantic/src/resolver.rs` (T2):**
   - Decoupled `LinkedTypeResolver` so that `linked: Option<Arc<LinkedProgram>>` is optional and `semantic_linked: Option<Arc<SemanticLinkedProgram>>` is supported.
   - Added `from_semantic`, `from_semantic_with_prelude`, and `with_semantic_linked(impl Into<Option<Arc<SemanticLinkedProgram>>>)`.
   - Updated `resolve_module_alias` and `resolve_type_name` to resolve against `SymbolicImportTable` and `LinkedModuleInterface` without touching executable slots or `LinkedReadSpec`.
   - Implemented `resolve_import_binding_id(&self, module: &ModuleId, name: &str) -> Option<SourceImportBindingId>`.
   - **Verification:** Ran `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic integration::resolver`: 4/4 pass.

3. **`phalcom-semantic/src/checker/` and `phase_evidence.rs` (T3):**
   - `phase_evidence.rs`: Created and fully mapped AST types to extract type annotation roots and declaration type roots.
   - `CallableAnalysis.observed_import_phases`: `Arc<BTreeMap<SourceImportBindingId, DependencyPhase>>`.
   - `CheckingContext`: Added `imported_source_bindings` and `observed_import_phases`. Added `record_import_phase` and `seed_symbolic_module_import_values`.
   - `Expr::Var` in `checker/expression.rs`: Records `DependencyPhase::Runtime` when reading imported local bindings or nominal types in expression position.
   - `resolve_type_annotation`: Records `DependencyPhase::InterfaceOnly`.
   - `BodyAnalysisContext`: Added `semantic_linked_module`.

4. **`phalcom-semantic/src/snapshot.rs` and `session.rs` (T4):**
   - `SemanticSnapshot`: Added `import_phase_products: Arc<BTreeMap<ModuleId, ModuleImportPhaseProduct>>` and `generated_runtime_requirements: Arc<[RuntimeDependencyEdge]>`.
   - `session.rs`: Wired `semantic_linked` into type resolvers, 5 seeding loops, and all 9 `FormalQueryInputs` sites.
   - At snapshot assembly in `session.rs`, calls `crate::phase_evidence::compute_module_import_phase_products` when `input.semantic_linked` is present and sets `snapshot_obj.import_phase_products` and `snapshot_obj.generated_runtime_requirements`.
   - Test mock fixtures updated in `effects_control.rs`, `integration_matrix.rs`, `termination.rs`, `source_index.rs`, `presentation.rs`, `fingerprints.rs`, `traits.rs`, and `query_ownership.rs`.
   - Both `cargo check -p phalcom-semantic` and `cargo test -p phalcom-semantic --test semantic integration::resolver` compile clean and pass.

---

## 4. Next steps for the continuing agent

### A. Executable Finalization with Requirements (T5)
In `phalcom-modules/src/linker.rs`:
1. Provide helper on `SemanticLinkedProgram`:
   ```rust
   pub fn to_unfinalized_program(&self) -> Arc<LinkedProgram>
   ```
2. Implement `finalize_executable_with_requirements`:
   ```rust
   pub fn finalize_executable_with_requirements(
       &self,
       semantic: SemanticLinkedProgram,
       phase_products: &BTreeMap<ModuleId, ModuleImportPhaseProduct>,
       requirements: &[RuntimeDependencyEdge],
   ) -> Result<Arc<LinkedProgram>, Vec<Diagnostic>>
   ```
   - Incorporate `requirements` into `graphs.runtime` before computing initialization order.
   - For each selective import, if `phase_products` records `DependencyPhase::InterfaceOnly`, omit the executable read slot and runtime edge. If `Runtime` (or missing/unobserved), project the runtime read slot and runtime edge.
   - Validate that `graphs.runtime` is an acyclic DAG. Reject cycles strictly.

### B. Core Compilation Pipeline Integration (T6)
In `phalcom-core/src/modules/compile.rs`:
1. Link semantically:
   ```rust
   let semantic = linker.link_semantic(&universe, &interfaces)?;
   ```
2. Feed `semantic` into semantic analysis:
   ```rust
   let workspace_input = SemanticWorkspaceInput::from_semantic(semantic.clone(), sources, generation);
   let outcome = session.update_with_budget_and_cancel(workspace_input, ...)?;
   ```
3. Finalize executable from semantic products:
   ```rust
   let linked = linker.finalize_executable_with_requirements(
       semantic,
       &snapshot.import_phase_products,
       &snapshot.generated_runtime_requirements,
   )?;
   ```
4. Hand `linked` to lowering and code generation.

### C. Contract Verification Suite (T7 & T8)
Run and verify:
- CV-01: Module-level item identities preserved through symbolic linking.
- CV-02: Pre-executable symbolic resolution succeeds without runtime slots.
- CV-03: Mutual type cycle resolves at interface phase without runtime slots.
- CV-04: Selective import used in value position becomes `DependencyPhase::Runtime`.
- CV-05: Non-source declaration requirement (e.g. superclass) becomes `RuntimeDependencyEdge`.
- CV-06: Strict runtime cycle rejection preserved when a cyclic runtime edge is present.
- CV-07: Phase evidence retained across incremental reuse.
- CV-08: Lowering/compilation consumes finalized `LinkedProgram` only.
- CV-09: S4 iteration-split prerequisite validation.

---

## 5. Verification commands

Run serially from `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections`:

```sh
# 1. Semantic compiler and tests
RUSTFLAGS='' cargo check -p phalcom-semantic
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic integration::resolver

# 2. Module linker unit tests
RUSTFLAGS='' cargo test -p phalcom-modules --test linker

# 3. Core module compilation check
RUSTFLAGS='' cargo check -p phalcom-core
```

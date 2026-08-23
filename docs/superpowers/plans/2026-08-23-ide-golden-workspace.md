# IDE Golden Workspace and Shared Integration Harness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build one canonical, clean-baseline, multi-project Phalcom workspace that is simultaneously a realistic VS Code project, a compiler/semantic integration fixture, an LSP regression workspace, and the seed for deterministic performance tests.

**Architecture:** `examples/ide-golden` is the single source of fixture truth. A test-only `phalcom-test-support` crate owns workspace copying, marker resolution, expectation loading, and guarded mutations; compiler/module/semantic/LSP tests consume it through dev-dependencies, while the VS Code Electron suite copies the same workspace and reads the same TOML contracts. Negative cases never live in git: tests apply a mutation to a temporary copy or open-buffer revision and require restoration to the zero-diagnostic `Ready` baseline.

**Tech Stack:** Rust 2024, `phalcom-modules`, `phalcom-semantic`, `phalcom-core`, `phalcom-lsp`, Tower LSP JSON-RPC test harness, serde/TOML expectation files, TypeScript VS Code Electron tests.

**Spec:** `examples/ide-golden/EXPECTATIONS.md` plus `examples/ide-golden/expectations/*.toml`.

## Global Constraints

- `examples/ide-golden` must always be clean at checkout: zero intentional parser, module, or semantic errors.
- The root VS Code folder is `examples/ide-golden`, with `project.toml` directly at the workspace root.
- The project graph is the intentional diamond `ide_golden -> geo -> units` and `ide_golden -> units`.
- There are exactly 26 committed `.ph` files in the baseline fixture; core/universe is excluded from that count.
- `units.internal` and `geo.internal` exist but are not exposed; compiler and completion must agree that they are unavailable externally.
- Marker syntax is `/*@stable.id*/`; tests resolve markers instead of hard-coding line/column positions.
- Machine-readable TOML expectations are authoritative; `EXPECTATIONS.md` is the human reproduction guide.
- Mutating tests operate on temporary workspace copies or unsaved LSP buffer revisions and restore the clean state.
- No production crate may depend on `phalcom-test-support`; it is dev/test-only.
- Do not modify Wave-4 flow implementation files until the F1–F5 work merges and its test suite is green.
- Do not introduce wall-clock CI thresholds until structural incremental assertions are stable and CI distributions have been measured.

---

## Repository State This Plan Builds On

The current repository already supplies the key seams this plan reuses:

- `phalcom-modules/src/manifest.rs`: strict `project.toml` parsing with path dependencies and project namespaces.
- `phalcom-modules/tests/integration.rs`: canonical examples for package exposure and `ModulePathNotExposed` behavior.
- `phalcom-lsp/tests/support/fixture.rs`: `/*@marker*/` stripping and UTF-16 position calculation.
- `phalcom-lsp/tests/support/workspace.rs`: temporary workspace copying.
- `phalcom-lsp/tests/support/lsp_client.rs`: in-process JSON-RPC client with semantic publication and perf-counter waits.
- `phalcom-lsp/src/backend.rs`: advertised definition, references, workspace-symbol, completion, hover, inlay, and semantic-token capabilities.
- `tools/vsphalcom/src/test/suite/lsp.e2e.test.ts`: real Electron/VS Code completion and live-edit E2E path.
- `.github/workflows/ci.yml`: blocking Rust workspace tests and a separate blocking VS Code Electron job.
- `phalcom-semantic/src/diagnostic.rs`: stable formal diagnostic codes used by mutation expectations.

The plan does not create parallel semantic infrastructure. It makes these existing layers consume one fixture contract.

---

### Task 1: Land the Golden Workspace Skeleton and Integrity Contract

**Files:**
- Modify: `Cargo.toml`
- Create: `phalcom-test-support/Cargo.toml`
- Create: `phalcom-test-support/src/lib.rs`
- Create: `phalcom-test-support/src/markers.rs`
- Create: `phalcom-test-support/src/golden.rs`
- Create: `phalcom-test-support/src/expectations.rs`
- Create: `phalcom-test-support/src/mutation.rs`
- Create: `examples/ide-golden/**`

**Interfaces:**
- Produces: `GoldenWorkspace::repository_fixture()`, `GoldenWorkspace::validate_integrity()`, `GoldenWorkspace::copy_to_temp()`, `MarkedSource::parse()`, `MarkedSource::position()`, `Mutation::apply()`.
- Consumes: only filesystem, serde/TOML, tempfile; no compiler/LSP production APIs.

- [ ] **Step 1: Write the failing integrity test before the fixture exists**

```rust
#[test]
fn repository_golden_workspace_satisfies_baseline_integrity() {
    GoldenWorkspace::repository_fixture()
        .validate_integrity()
        .unwrap();
}
```

- [ ] **Step 2: Run the RED test**

Run:

```bash
cargo test -p phalcom-test-support repository_golden_workspace_satisfies_baseline_integrity -- --exact
```

Expected: FAIL because `examples/ide-golden/expectations/baseline.toml` does not exist yet.

- [ ] **Step 3: Add the 3-project/26-source fixture and baseline TOML**

Use the exact directory graph in this plan. All package directories must contain `package.ph`. `units/internal.ph` and `geo/internal.ph` must remain unexposed.

- [ ] **Step 4: Add marker and guarded mutation unit tests**

The UTF-16 test must include a non-ASCII character:

```rust
let source = MarkedSource::parse("α./*@completion*/beta()\n");
assert_eq!(source.position("completion").utf16_character, 2);
```

The mutation test must prove `old` text is checked before replacement.

- [ ] **Step 5: Run GREEN**

```bash
cargo test -p phalcom-test-support
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml phalcom-test-support examples/ide-golden
git commit -m "test: add canonical IDE golden workspace"
```

---

### Task 2: Parse Every Golden Source with Current Parser

**Files:**
- Modify: `phalcom-test-support/Cargo.toml`
- Create: `phalcom-test-support/tests/golden_syntax.rs`

**Interfaces:**
- Consumes: `GoldenWorkspace`, `MarkedSource`, `phalcom_ast::parse`.
- Produces: a hard gate that all committed golden sources remain parse-clean after marker stripping.

- [ ] **Step 1: Add `phalcom-ast` as a dev-dependency of test support**

```toml
[dev-dependencies]
phalcom-ast = { path = "../phalcom-ast" }
```

- [ ] **Step 2: Write the parser test**

Collect every `.ph` beneath the golden root, strip markers, parse with `phalcom_ast::parse`, and assert `errors.is_empty()` with file path and full parser errors in the failure message.

- [ ] **Step 3: Run RED**

```bash
cargo test -p phalcom-test-support --test golden_syntax
```

Expected on the first execution: any syntax drift in the scaffold is exposed here. Fix fixture source only; do not weaken the parser assertion.

- [ ] **Step 4: Run GREEN**

Repeat until every one of the 26 committed sources parses.

- [ ] **Step 5: Commit**

```bash
git add phalcom-test-support examples/ide-golden
git commit -m "test: require golden workspace syntax to stay current"
```

---

### Task 3: Validate Project Discovery, Diamond Identity, and Exposure

**Files:**
- Modify: `phalcom-modules/Cargo.toml`
- Create: `phalcom-modules/tests/ide_golden.rs`

**Interfaces:**
- Consumes: `GoldenWorkspace`, `ProjectUniverse`, `FilesystemSourceProvider`, `ModuleResolver`.
- Produces: canonical project/module identity assertions used as prerequisites by compiler/LSP tests.

- [ ] **Step 1: Add test-only dependency**

```toml
[dev-dependencies]
phalcom-test-support = { path = "../phalcom-test-support" }
```

Preserve existing dev-dependencies.

- [ ] **Step 2: RED — assert the universe contains exactly three projects**

Load `examples/ide-golden/project.toml`; inspect the resolved universe. Assert project namespaces are exactly `ide_golden`, `geo`, `units`, and that the direct and transitive `units` dependency resolve to one `ResolvedProjectId`.

- [ ] **Step 3: RED — assert external public paths resolve**

From `ide_golden.lab.modules`, resolve `units.distance`, `units.weight`, `geo.point`, and `geo.route`.

- [ ] **Step 4: RED — assert hidden paths are rejected**

Resolve `units.internal` and `geo.internal`; assert `ModuleResolutionError::ModulePathNotExposed`.

- [ ] **Step 5: GREEN**

No production change should be necessary if module semantics are correct. If this fails, repair project-universe canonicalization/resolution rather than changing fixture expectations.

- [ ] **Step 6: Commit**

```bash
git add phalcom-modules
git commit -m "test: cover golden project graph and exposure"
```

---

### Task 4: Establish Compiler/Semantic Clean Baseline

**Files:**
- Modify: `phalcom-semantic/Cargo.toml`
- Create: `phalcom-semantic/tests/ide_golden.rs`
- Modify: `phalcom-core/Cargo.toml`
- Create: `phalcom-core/tests/ide_golden.rs`
- Modify only if defects are exposed: compiler analyzer/compile seams identified by the existing integration plan.

**Interfaces:**
- Consumes: linked golden workspace and formal semantic analysis.
- Produces: zero-diagnostic semantic snapshot, successful compiler check/compile, exact runtime transcript.

- [ ] **Step 1: RED — formal whole-workspace analysis has zero errors**

Build the same canonical linked program used by compiler/LSP integration. Assert all three projects are represented once and every diagnostic collection is empty.

- [ ] **Step 2: GREEN formal baseline**

Fix only genuine implementation defects. Do not remove valid annotations or cross-project relationships merely to make the fixture easier.

- [ ] **Step 3: RED — compiler check and compile accept the project**

Use the compiler's project entry-selection API against `ide_golden.main`.

- [ ] **Step 4: RED — runtime stdout is exact**

Load `expectations/runtime.toml` and compare exit code/stdout/stderr byte-for-byte.

- [ ] **Step 5: GREEN compiler/runtime baseline**

Repair source/semantic/compiler integration until the exact runtime contract passes.

- [ ] **Step 6: Commit**

```bash
git add phalcom-semantic phalcom-core examples/ide-golden
git commit -m "test: make IDE golden project compile and run cleanly"
```

---

### Task 5: Promote Golden Fixture Consumption into LSP Tests

**Files:**
- Modify: `phalcom-lsp/Cargo.toml`
- Modify: `phalcom-lsp/tests/support/fixture.rs`
- Modify: `phalcom-lsp/tests/support/workspace.rs`
- Modify: `phalcom-lsp/tests/support/lsp_client.rs`
- Create: `phalcom-lsp/tests/ide_golden.rs`
- Modify: `phalcom-lsp/tests/integration.rs`

**Interfaces:**
- Consumes: `phalcom_test_support::{GoldenWorkspace, MarkedSource}`.
- Produces: `TestWorkspace::from_golden()`, definition/references/workspace-symbol requests, diagnostics/status convergence helpers.

- [ ] **Step 1: RED — initialize the golden workspace and require zero diagnostics after convergence**

The test opens `src/main.ph`, waits for workspace/static convergence, then asserts no published diagnostics for open golden documents and final analysis status `Ready`.

- [ ] **Step 2: Refactor LSP-private marker/workspace helpers to delegate**

Keep LSP conversion from generic `MarkerPosition` to `tower_lsp::Position` inside LSP test support. Do not make `phalcom-test-support` depend on Tower LSP.

- [ ] **Step 3: Extend `TestLsp`**

Add exact helpers:

```rust
pub async fn definition(&mut self, uri: &str, position: Position) -> Value;
pub async fn references(&mut self, uri: &str, position: Position, include_declaration: bool) -> Value;
pub async fn workspace_symbols(&mut self, query: &str) -> Value;
```

Add a wait helper that observes the real analysis status/event path rather than sleeping.

- [ ] **Step 4: GREEN baseline**

Run:

```bash
RUST_MIN_STACK=8388608 cargo test -p phalcom-lsp --test integration ide_golden -- --test-threads=2
```

- [ ] **Step 5: Commit**

```bash
git add phalcom-lsp
git commit -m "test: drive LSP through canonical golden workspace"
```

---

### Task 6: Implement Module-Context Completion Against Canonical Module Interfaces

**Files:**
- Modify: `phalcom-lsp/src/completion.rs`
- Modify: `phalcom-lsp/src/backend.rs`
- Modify as needed: `phalcom-lsp/src/semantic/module_graph.rs`
- Modify as needed: canonical module query/publication layer from the compiler/LSP integration spec.
- Test: `phalcom-lsp/tests/ide_golden.rs`

**Interfaces:**
- Produces completion contexts for import roots, import path segments, selective imports, and module alias member access.
- Canonical authority: `phalcom-modules` resolution and linked exports/exposed children, never raw directory listing.

- [ ] **Step 1: RED — `completion.units.path`**

Apply an unsaved buffer edit equivalent to `import units.|`; assert `distance` and `weight` are returned and `internal` is absent.

- [ ] **Step 2: RED — `completion.geo.path`**

Assert `point`, `route`, and no `internal`.

- [ ] **Step 3: Implement syntactic completion-context dispatch before receiver completion**

Introduce a context enum whose module cases carry logical import prefix/module identity rather than source-text guesses.

- [ ] **Step 4: Query canonical exposed children/linked exports**

The invariant is: every import completion candidate must be accepted by `ModuleResolver` under the same importer identity.

- [ ] **Step 5: GREEN and parity assertion**

For every completion candidate in the strict module-child cases, construct the corresponding `ImportPath` and require compiler resolver success.

- [ ] **Step 6: Commit**

```bash
git add phalcom-lsp
git commit -m "feat: complete module paths from canonical interfaces"
```

---

### Task 7: Formal-First Hover and Inlay Hints

**Files:**
- Modify: `phalcom-lsp/src/inlay_hints.rs`
- Modify: hover/query code in `phalcom-lsp/src/backend.rs` and semantic query modules.
- Test: `phalcom-lsp/tests/ide_golden.rs`

**Wave-4 dependency:** Formal callable/body products must expose stable expression/binding type results before flow-refined hover is switched to formal-first.

- [ ] **Step 1: RED — annotated bindings suppress duplicate hints**

Use `inlay.value.explicit` and `inlay.parameter.explicit`; assert no `: Int` hint is emitted at those annotated positions.

- [ ] **Step 2: GREEN suppression using AST annotation presence**

Suppress inferred local, parameter, field, and return hints whenever an explicit annotation occupies that semantic position.

- [ ] **Step 3: RED — formal type hover**

`hover.int` requires `Int`; `hover.point` requires dependency-owned `Point`; neither may report `Unknown` when formal results exist.

- [ ] **Step 4: GREEN formal-first, advisory fallback**

Use current-revision/current-generation formal facts first. Use advisory `ValueShape` only when no valid formal fact exists.

- [ ] **Step 5: RED/GREEN — `hover.flow.refined` after Wave 4**

Inside the `is ExpressShipment` branch, require the formal narrowed receiver type.

- [ ] **Step 6: Commit**

```bash
git add phalcom-lsp
git commit -m "feat: align hover and inlays with formal typing"
```

---

### Task 8: Definition, References, Workspace Symbols, and Core Navigation

**Files:**
- Modify: `phalcom-lsp/src/semantic/occurrence.rs`
- Modify: definition/reference query code in `phalcom-lsp/src/backend.rs`
- Modify: core-source mapping layer used by `CORE_MODULE_URI`.
- Test: `phalcom-lsp/tests/ide_golden.rs`

- [ ] **Step 1: RED — cross-project definition targets**

Assert `navigation.parcel.use`, `navigation.point.cross_project`, and `navigation.distance.direct` land at their target marker/file pairs.

- [ ] **Step 2: GREEN module-aware occurrence/navigation identity**

Do not use string matching across files. Resolve declaration/module identities and map them to the document catalog.

- [ ] **Step 3: RED — core `Int` definition**

`navigation.core.int` must produce a physical core source location when a physical core source is selected.

- [ ] **Step 4: GREEN core logical-to-physical mapping**

Remove the current early-return behavior that discards definition locations owned by the logical core module; translate the logical owner/range through the selected `CoreSource.physical_uri`.

- [ ] **Step 5: RED/GREEN references and symbols**

Use `references.toml` and `symbols.toml`. References must be semantic and exclude comment/string occurrences.

- [ ] **Step 6: Commit**

```bash
git add phalcom-lsp
git commit -m "feat: navigate golden workspace and core semantically"
```

---

### Task 9: Diagnostic Mutation Contract and Status Lifecycle

**Files:**
- Modify: `phalcom-lsp/src/analysis_service.rs`
- Modify: `phalcom-lsp/src/backend.rs`
- Modify: `tools/vsphalcom/src/analysisStatus.ts`
- Test: `phalcom-lsp/tests/ide_golden.rs`
- Test: `tools/vsphalcom/src/test/suite/analysisStatus.test.ts`

- [ ] **Step 1: RED — binding mismatch mutation**

Apply `diagnostic.binding_mismatch` through `didChange`. Require exactly the canonical `type.binding.initializer_mismatch` diagnostic for the current document revision.

- [ ] **Step 2: RED — restore**

Restore baseline text. Require zero diagnostics and terminal `Ready`.

- [ ] **Step 3: RED — parser recovery supersedes stale formal diagnostics**

Apply `diagnostic.parser_recovery`; require parser diagnostics and no stale formal diagnostic from the old source revision.

- [ ] **Step 4: Fix terminal state transitions**

Every completed edit batch must end in one of `Ready`, `Error`, or a newer active `Analyzing` state. No edit-only batch may remain in `Publishing`/`Updating`.

- [ ] **Step 5: GREEN all mutation/status cases**

Run both Rust integration and extension status tests.

- [ ] **Step 6: Commit**

```bash
git add phalcom-lsp tools/vsphalcom
git commit -m "fix: make golden edit lifecycle diagnostic-safe"
```

---

### Task 10: Wave-4 Flow Integration Cases

**Files:**
- Modify after Wave 4 merges: `examples/ide-golden/src/lab/flow.ph`
- Modify: `examples/ide-golden/expectations/diagnostics.toml`
- Modify: `examples/ide-golden/expectations/mutations.toml`
- Test: `phalcom-semantic/tests/ide_golden.rs`
- Test: `phalcom-lsp/tests/ide_golden.rs`

- [ ] **Step 1: Re-ground current F1–F5 APIs**

Before editing the fixture, inspect the merged `checker/flow` APIs and `resolve_iteration_element`; do not code from the pre-merge implementation plan.

- [ ] **Step 2: RED — positive and negative narrowing**

Add mutation cases that use an `ExpressShipment`-only member outside its valid narrowed branch and require the checker to reject it.

- [ ] **Step 3: RED — mutation invalidation**

Narrow a binding, assign a value that invalidates the predicate, then consume an express-only member. Require a diagnostic.

- [ ] **Step 4: RED — conservative branch join and loop convergence**

Assert formal type knowledge after joins/widening through semantic test APIs rather than hover-string parsing.

- [ ] **Step 5: RED — custom iterable protocol**

Add a fixture-owned iterable whose element type is obtained only via `iterate(_)`/`iteratorValue(_)`; require correct `for` element typing and no nominal collection special case.

- [ ] **Step 6: GREEN and commit**

```bash
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test ide_golden
RUST_MIN_STACK=8388608 cargo test -p phalcom-lsp --test integration ide_golden -- --test-threads=2
git add examples/ide-golden phalcom-semantic phalcom-lsp
git commit -m "test: exercise formal flow through golden workspace"
```

---

### Task 11: Structural Incrementality and Diamond Invalidation

**Files:**
- Modify: `phalcom-semantic/src/db/**` as required by the formal incremental implementation plan.
- Modify: `phalcom-lsp/src/analysis_service.rs`
- Modify: `phalcom-lsp/src/perf.rs`
- Test: `phalcom-semantic/tests/ide_golden.rs`
- Test: `phalcom-lsp/tests/ide_golden.rs`
- Modify: `examples/ide-golden/expectations/invalidation.toml`

- [ ] **Step 1: RED — body-only edit does not rebuild module interfaces**

Capture semantic DB/perf counters, mutate only a callable body, converge, and assert the edited callable rechecks while unchanged module interfaces/project universe/resolver products are reused.

- [ ] **Step 2: RED — declaration edit reaches reverse dependents only**

Change a public callable type in `geo`; assert root consumers recheck and `units` products remain reused unless they actually depend on the changed query.

- [ ] **Step 3: RED — diamond units edit canonicalizes one invalidation source**

Change an exported `units` declaration; assert direct root consumers and `geo` consumers update from one canonical units project/module identity, not duplicate graphs.

- [ ] **Step 4: RED — query paths perform no disk IO/reanalysis**

After publication, hover/completion/definition/inlay requests must not increment filesystem-read or full-workspace-analysis counters.

- [ ] **Step 5: GREEN incremental semantic DB**

Move persistence/reuse into `phalcom-semantic`; do not implement an LSP-only cache of store-local `TypeId`s.

- [ ] **Step 6: Commit**

```bash
git add phalcom-semantic phalcom-lsp examples/ide-golden
git commit -m "perf: validate incremental semantics on golden workspace"
```

---

### Task 12: Structured Analysis Logging and Failure Preservation

**Files:**
- Modify: `phalcom-lsp/src/analysis_service.rs`
- Modify: `phalcom-lsp/src/backend.rs`
- Modify: `phalcom-lsp/src/perf.rs`
- Modify: `tools/vsphalcom/src/extension.ts`
- Test: Rust LSP integration and VS Code E2E.

- [ ] **Step 1: RED — failed static refresh preserves last valid snapshot**

Create a controlled project/link failure in a temp golden copy. Assert the worker emits an error event/log record and does not replace the last valid formal snapshot with `None`.

- [ ] **Step 2: GREEN error publication**

Emit session/sequence/phase/epoch/URI/revision/batch-size/duration information and the concrete project/resolution/link failure cause.

- [ ] **Step 3: RED/GREEN extension Output channel**

Electron test opens the Output channel or captures the extension-side sink and asserts a structured analysis failure is visible without requiring `PHALCOM_LSP_PERF=1`.

- [ ] **Step 4: Commit**

```bash
git add phalcom-lsp tools/vsphalcom
git commit -m "feat: expose LSP analysis lifecycle and failures"
```

---

### Task 13: VS Code Golden Workspace E2E

**Files:**
- Modify: `tools/vsphalcom/src/test/runTest.ts`
- Modify: `tools/vsphalcom/src/test/suite/lsp.e2e.test.ts`
- Create as useful: `tools/vsphalcom/src/test/support/golden.ts`

**Interfaces:**
- Consumes the same checked-in `examples/ide-golden` and TOML expectations.
- Produces real extension-host coverage for the bridge, not duplicate protocol exhaustiveness.

- [ ] **Step 1: RED — launch Electron with a copied golden workspace folder**

Use `workspacePath`/launch args so VS Code itself sees `project.toml` at the root. Configure the built `phalcom-lsp` binary before opening `.ph` documents.

- [ ] **Step 2: RED — Ready and zero Problems**

Wait on the extension's actual status API/observable state, not fixed sleep. Assert diagnostics collections are empty.

- [ ] **Step 3: RED/GREEN bridge cases**

Verify one representative case for completion, hover, definition, references, inlay hints, semantic diagnostics after mutation, restore-to-clean, and status lifecycle.

- [ ] **Step 4: Keep exhaustive candidate/type/token assertions in Rust**

Do not duplicate the entire TOML suite in Electron; E2E proves the language-client/extension bridge transports correct server results.

- [ ] **Step 5: Commit**

```bash
git add tools/vsphalcom
git commit -m "test: run VS Code E2E against IDE golden workspace"
```

---

### Task 14: Scale Workspace Builder and Performance Gates

**Files:**
- Create: `phalcom-test-support/src/scale.rs`
- Modify: `phalcom-test-support/src/lib.rs`
- Modify: `phalcom-lsp/tests/performance.rs`
- Optionally add ignored benchmark-style semantic test.

- [ ] **Step 1: RED — deterministic scale expansion**

```rust
let workspace = ScaleWorkspace::from_golden()
    .with_leaf_modules(2_000)
    .build()?;
assert_eq!(workspace.generated_modules(), 2_000);
```

Generated names must be deterministic (`module_0001`, etc.) and package exposure valid.

- [ ] **Step 2: GREEN structural performance assertions**

At 2,000 generated modules, assert progressive scanning yields to interactive work, hover remains query-only, and a body-only edit does not create a full formal workspace rebuild.

- [ ] **Step 3: Keep wall-clock tests ignored initially**

Reuse the existing performance harness to print cold convergence/edit latency distributions. Do not make milliseconds blocking until CI measurements are stable.

- [ ] **Step 4: Commit**

```bash
git add phalcom-test-support phalcom-lsp
git commit -m "perf: derive scale workloads from golden workspace"
```

---

### Task 15: Retire Duplicate VS Code Manual Fixtures

**Files:**
- Modify: `tools/vsphalcom/manual-test/CHECKLIST.md`
- Delete after parity is verified: duplicated `01-*.ph` through `05-*.ph` manual fixtures that the golden workspace supersedes.

- [ ] **Step 1: Map every old manual checklist item to a golden case ID**

Do not delete an old fixture until every behavior it uniquely covers is either intentionally kept elsewhere (for example the large syntax torture corpus) or represented by the golden workspace.

- [ ] **Step 2: Point manual testing at `examples/ide-golden/EXPECTATIONS.md`**

- [ ] **Step 3: Run full verification**

```bash
RUST_MIN_STACK=8388608 cargo test --workspace --all-targets
npm ci --prefix tools/vsphalcom
xvfb-run -a npm --prefix tools/vsphalcom test
graphify update .
```

- [ ] **Step 4: Commit**

```bash
git add tools/vsphalcom examples/ide-golden
git commit -m "test: consolidate manual IDE checks on golden workspace"
```

---

## Completion Gates

The program is complete only when all of these are true:

1. Opening `examples/ide-golden` converges to `Ready` with zero Problems.
2. `phalcom check`, compile, and runtime execution match the baseline/runtime TOML contracts.
3. The three-project diamond contains exactly one canonical `units` project identity.
4. Module completion and compiler resolution agree on every strict exposed-child case.
5. Hover/inlays prefer current formal facts; explicit annotations never receive duplicate inferred hints.
6. Same-project, cross-project, diamond, inherited, re-export, and core definition navigation work.
7. Parser and formal diagnostic mutations publish only current-revision diagnostics and restore to zero cleanly.
8. Wave-4 narrowing, joins, mutation invalidation, loop convergence, and protocol iteration are exercised through the fixture after F1–F5 lands.
9. Body-only formal updates reuse persistent semantic products and do not relink/rebuild the whole workspace.
10. Analysis failures are visible and preserve the last valid snapshot.
11. Real VS Code Electron E2E uses the golden workspace and verifies the extension bridge.
12. Scale tests derive large workspaces programmatically rather than bloating the manual fixture.

## Verification Commands

```bash
cargo test -p phalcom-test-support
cargo test -p phalcom-modules --test ide_golden
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test ide_golden
RUST_MIN_STACK=8388608 cargo test -p phalcom-core --test ide_golden
RUST_MIN_STACK=8388608 cargo test -p phalcom-lsp --test integration -- --test-threads=2
RUST_MIN_STACK=8388608 cargo test --workspace --all-targets
npm ci --prefix tools/vsphalcom
xvfb-run -a npm --prefix tools/vsphalcom test
graphify update .
```

## Self-Review

- Spec coverage: workspace shape, clean baseline, runtime, modules, type/flow semantics, LSP features, VS Code bridge, invalidation, core navigation, logging, and scaling all have explicit tasks.
- Placeholder scan: no task delegates an undefined implementation step; each behavioral change has a RED assertion, concrete seam, and GREEN command.
- Type/interface consistency: generic fixture support exposes byte/UTF-16 marker positions without depending on Tower LSP; LSP transport remains LSP-owned; persistent formal type identity remains `phalcom-semantic`-owned.
- Concurrency safety: Wave-4-owned flow implementation is not edited until the explicit re-grounding gate in Task 10.

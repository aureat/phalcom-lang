# Phalcom VM Bootstrap Tiering and Shared Canonical Universe Compilation
## Repository-Grounded, Checkpoint-Driven, Patch-Grade Implementation Plan

**Status:** Proposed implementation program  
**Companion specification:** `phalcom-vm-bootstrap-tiering-technical-spec.md`  
**Repository:** `aureat/phalcom-lang`  
**Prepared against remote branch:** `main`  
**Prepared against remote HEAD:** `9f04681201e4e15388b4a32d09a2a502486e9367` (`feat: extend semantic type-system closure`)  
**Parent revision:** `a37664e17e5e9f31378b7d497e51ad349d5ba905`  
**Local working tree:** unavailable to the planning environment; the implementing agent must record actual local branch/HEAD/status before editing.  
**Pinned toolchain:** `nightly-2026-07-10`

---

# 1. Implementation program

## 1.1 Dominant objective

Refactor VM bootstrap so that:

1. compiler-derived canonical Universe state is immutable and process-shared;
2. every VM retains fresh mutable runtime state;
3. `VM::new()` remains the existing full shipping bootstrap;
4. consumers/tests may request kernel-only or native-only VM construction;
5. kernel/native construction cannot accidentally trigger whole-Universe semantic analysis;
6. source-derived semantic roots are explicitly unavailable before full source bootstrap;
7. exact canonical Universe ADT/GADT/associated lowering semantics remain unchanged.

This program is intentionally narrower than the future persisted project-artifact architecture.

## 1.2 Current incident

Current `VM::new()` performs source/native verification and invokes a source-complete whole-Universe semantic analysis through `VM::universe_lowerings`. `phalcom_semantic::analyze_workspace` creates a fresh `SemanticWorkspaceSession`, so this work repeats for every VM.

Observed repository evidence records roughly 55–71 seconds of debug bootstrap cost in focused core tests. A recent generic-inference bug also manifested through unrelated VM construction because `VM::new()` happens to run the semantic compiler over the canonical Universe.

## 1.3 Architectural end state

```text
PROCESS-SHARED COMPILER PRODUCT
────────────────────────────────────────────────────────
UniverseSourceProvider
    ↓
BuiltinInterfaceBuilder caches          existing authority
    ↓
NativeSourceIndex
    ↓
verify_native_contracts
    ↓
ModuleResolver / ModuleLinker
    ↓
LinkedProgram
    ↓
one source-complete SemanticWorkspaceSession
    ↓
transient SemanticSnapshot
    ↓
CompiledModule / ModuleLoweringSemantics
    ↓
CanonicalUniverseProgram (OnceLock)
────────────────────────────────────────────────────────

FRESH VM STATE
────────────────────────────────────────────────────────
VM::new_kernel()
    Heap + kernel class tower + root fiber + empty registries

VM::new_native()
    kernel
      + canonical module/native floor
      + native primitive installation

VM::new()
    native
      + shared CanonicalUniverseProgram
      + fresh VM-local AST compilation
      + source module execution
      + semantic roots / post-bootstrap invariants
────────────────────────────────────────────────────────
```

No `Heap`, `ObjRef`, `ClassId`, runtime closure, populated `Chunk`, mutable inline cache, fiber, global slot contents, or resource object enters the shared compiler product.

---

# 2. Repository evidence ledger

| Claim | Repository evidence |
|---|---|
| `VM::new()` delegates to `new_with_native_install_mode` and the latter owns full kernel/native/source bootstrap | `phalcom-core/src/vm/bootstrap.rs` — `VM::new`, `VM::new_with_native_install_mode` |
| Full bootstrap builds `NativeSourceIndex`, verifies contracts, materializes Universe, installs natives, runs source modules, then resolves semantic roots | `phalcom-core/src/vm/bootstrap.rs` |
| `VM::run_universe_modules` derives lowerings before compiling/executing every selected source unit | `phalcom-core/src/vm/bootstrap.rs` — `run_universe_modules` |
| `VM::universe_lowerings` resolves/links the canonical Universe and calls one source-complete `analyze_workspace` | `phalcom-core/src/vm/bootstrap.rs` — `universe_lowerings` |
| `analyze_workspace` creates a new `SemanticWorkspaceSession` every invocation | `phalcom-semantic/src/workspace.rs` — `analyze_workspace` |
| Semantic session owns `SemanticDb`, `TypeStore`, base declarations/hierarchy/etc. | `phalcom-semantic/src/session.rs` — `SemanticWorkspaceSession` |
| Parsed builtin units and unlinked builtin interfaces are already process-cached | `phalcom-modules/src/builtin_interface.rs` — `BUILTIN_PARSED_CACHE`, `BUILTIN_INTERFACE_CACHE` |
| `NativeSourceIndex` is VM-free and retains canonical `Arc<ParsedModuleUnit>` units | `phalcom-core/src/native/source.rs` — `NativeSourceIndex` |
| Existing compiler model already has VM-independent `CompiledModule`/`CompiledProgram` | `phalcom-core/src/modules/compile.rs` |
| `ModuleLoweringSemantics` is compact immutable backend-facing semantics based on canonical IDs/source sites | `phalcom-core/src/modules/semantic_lowering.rs` |
| Canonical Universe materialization creates VM-local module `ObjRef`s and native globals | `phalcom-core/src/modules/builtin_materialize.rs` |
| General materialization is already separated from execution | `phalcom-core/src/modules/materialize.rs` |
| Current `run_compiled` already skips initialized/failed modules before compiling missing closures | `phalcom-core/src/interpret.rs` — `run_compiled` |
| `semantic_roots` currently uses Nil/default placeholders during bootstrap | `phalcom-core/src/vm/bootstrap.rs`, `phalcom-core/src/vm/mod.rs` |
| `semantic_roots` is consumed by numeric primitives, dispatch, and GC | `phalcom-core/src/primitive/{int,number}.rs`, `phalcom-core/src/vm/{dispatch,gc}.rs` |
| Core integration tests share one `core` test binary | `phalcom-core/tests/core/mod.rs`, `phalcom-core/tests/README.md` |
| Existing domain-neutral VM test helper seam exists | `phalcom-core/tests/support/vm.rs` |
| Canonical Result identity helper exists | `phalcom-semantic/src/core_surface/identity.rs` — `universe_declaration` |
| Existing runtime test checks canonical Result reuses primordial runtime root | `phalcom-core/tests/native_adt_runtime.rs` — `canonical_result_reuses_primordial_runtime_root` |
| Exact full-runtime regressions include range slicing, curated prelude, and Either runtime | `phalcom-core/tests/core/collections/contract.rs`, `modules/universe.rs`, `either/runtime.rs` |

---

# 3. Sources of truth and ownership rules

## 3.1 Canonical source

Source of truth:

```text
UniverseSourceProvider
+
BuiltinInterfaceBuilder
```

Consumers:

- `NativeSourceIndex`;
- `ModuleResolver`;
- `ModuleLinker`;
- semantic analysis;
- runtime source compilation.

Forbidden competing authority:

- a second core-owned parsed-source cache;
- handwritten VM-only interfaces derived separately from source.

## 3.2 Canonical module graph

Source of truth:

```text
LinkedProgram
```

Consumers:

- compiler binding projection;
- shared canonical compiler product;
- VM runtime compiler.

Forbidden competing authority:

- re-resolving canonical imports separately for each VM.

## 3.3 Formal semantics

Source of truth during canonical compilation:

```text
source-complete SemanticSnapshot
```

Derived shareable product:

```text
CompiledModule.lowering
    → ModuleLoweringSemantics
```

Forbidden competing authority:

- name-based runtime repair;
- source-incomplete per-unit analysis.

## 3.4 Runtime state

Source of truth:

```text
each VM's Heap + registries
```

Forbidden competing authority:

- process-global initialized VM;
- shared `ObjRef`/`ClassId`;
- mutable shared closure/chunk state.

## 3.5 Source-derived semantic runtime roots

Source of truth:

```text
actual full Universe source execution result
```

Forbidden competing authority:

- `Value::nil()` placeholder;
- `ClassId::default()` placeholder;
- string/boolean “initialized” flag divorced from values.

---

# 4. Checkpoint map

| Checkpoint | Tasks | Semantic boundary | Required evidence | Deferred evidence |
|---|---:|---|---|---|
| C0 | 1–2 | Baseline and exact ownership map are recorded without re-running known expensive broad suites | repository state, structural searches, one compile/no-run gate if needed | runtime timing, broad core/workspace |
| C1 | 3–5 | Canonical Universe compiler derivation is a single immutable modules-owned process product | singleton/product test, canonical Result lowering identity test, `cargo check -p phalcom-core` | VM behavior, broad core |
| C2 | 6–9 | Kernel/native/full VM stages are explicit and lower tiers cannot masquerade as full | stage-boundary tests, semantic-root hostile tests, focused native/kernel tests | full canonical runtime integration |
| C3 | 10–12 | Full `VM::new()` consumes the shared compiler product; VM bootstrap no longer owns linking/semantic analysis | range/prelude/Result/Either focused regressions, negative searches | test-suite migration, broad workspace |
| C4 | 13–15 | Tests use the minimum VM tier without changing the semantic meaning of full-runtime tests | migrated low-level test groups + retained full-runtime controls | workspace delivery gate |
| C5 | 16–17 | Performance architecture and broad compatibility are demonstrated | focused elapsed evidence, core target, final format/check/test/clippy | none |

Checkpoint dependency:

```text
C0 → C1 → C2 → C3 → C4 → C5
```

Do not begin C3 while C2 has an active incident: full runtime composition depends on honest capability state established in C2.

---

# 5. Checkpoint C0 — Baseline, drift lock, and bounded working state

Tasks:
- Task 1 — Record actual implementation baseline and state file.
- Task 2 — Reconfirm ownership/search anchors and avoid stale work.

Why this is a checkpoint:

The plan was prepared from remote `main`, not the implementer's local tree. Before editing cross-cutting VM/bootstrap code, the implementing agent must pin actual state and confirm that the primary symbols still have the responsibilities described here. This is a drift/diagnosis boundary, not a behavioral patch.

Entry conditions:

- repository clone/worktree is available;
- no implementation edits from this program have started.

Working set:

Primary:
- `phalcom-core/src/vm/bootstrap.rs`
- `phalcom-core/src/vm/mod.rs`
- `phalcom-core/src/native/source.rs`
- `phalcom-modules/src/builtin_interface.rs`
- `phalcom-core/src/modules/compile.rs`
- `phalcom-core/src/modules/semantic_lowering.rs`
- `phalcom-core/tests/support/vm.rs`

Secondary — inspect only if evidence requires it:
- `phalcom-semantic/src/workspace.rs`
- `phalcom-semantic/src/session.rs`
- `phalcom-core/src/modules/builtin_materialize.rs`
- `phalcom-core/src/modules/materialize.rs`

Out of scope for this checkpoint:
- source changes;
- semantic solver changes;
- bootstrap root semantics;
- test migration.

Semantic contract established by this checkpoint:

- actual local baseline is known;
- no stale task remains in the program;
- local changes that overlap the working set are identified before patching.

Semantic risks:
- planning against an outdated VM constructor;
- overwriting concurrent local work;
- accidentally re-planning an issue already repaired.

Hostile cases:
- local HEAD differs materially from `9f046812...`;
- `run_compiled` is already fixed (it is at planned remote HEAD) and must not be “fixed again”;
- another branch already introduced a canonical Universe cache with different ownership.

Required evidence:

1. ```bash
   git rev-parse --abbrev-ref HEAD
   git rev-parse HEAD
   git status --short
   ```
   Proves the actual implementation baseline and visible working-tree state.

2. ```bash
   rg -n "fn new_with_native_install_mode|fn run_universe_modules|fn universe_lowerings|NativeSourceIndex::build|verify_native_contracts" \
     phalcom-core/src/vm/bootstrap.rs
   ```
   Proves the expensive compiler path still belongs to VM bootstrap before migration.

3. ```bash
   rg -n "BUILTIN_PARSED_CACHE|BUILTIN_INTERFACE_CACHE" \
     phalcom-modules/src/builtin_interface.rs
   ```
   Proves existing parsed/interface cache ownership.

Do not run yet:
- `cargo test -p phalcom-core --test core`
- workspace tests
- the known ~55–71 second baseline merely for ritual repetition

The existing repository work log is sufficient historical evidence that the problem is real. If the implementing agent wants a local elapsed baseline, run one exact full-VM test only and record it as non-gating.

Escalate immediately if:
- `VM::universe_lowerings` no longer exists or is no longer the analysis owner;
- a process-shared canonical compiled product already exists;
- local uncommitted changes overlap the planned primary files in a semantically incompatible way.

Checkpoint completion:
- [ ] actual branch/HEAD/status recorded
- [ ] primary symbols confirmed
- [ ] stale work removed from local interpretation of the plan
- [ ] state file created/updated
- [ ] no active incident

Suggested commit:
- none

---

## Task 1 — Record actual implementation baseline and create state file

Purpose:

Create a concise resumable implementation-state record before cross-cutting edits.

Risk:
- Semantic: LOW
- Implementation fanout: local documentation

Owned files and symbols:
- `docs/impl/runtime/vm-bootstrap-tiering/state/STATE.md` — new program state record, if repository convention accepts this location

Inspect before editing:
- nearby `docs/impl/**/state/STATE.md` conventions
- current branch/HEAD/status

Do not inspect unless evidence forces expansion:
- parser
- LSP
- type solver internals

Dependencies:
- none

Source of truth:
- actual local Git state

Implementation boundary:

Changes:
- record plan/spec name;
- record prepared remote baseline and actual local baseline;
- record any overlapping dirty files;
- seed invariant/decision/evidence tables.

Must not:
- copy raw reasoning logs;
- claim tests passed before execution;
- overwrite unrelated existing state.

Edit operations:

1. RUN the C0 Git-state commands.
2. INSPECT one or two nearby state files for repository formatting.
3. CREATE the state file.
4. RECORD:
   - current checkpoint `C0`;
   - local HEAD;
   - dirty-file overlap;
   - long-term artifact work explicitly out of scope;
   - next action `C0 Task 2`.

Code instructions:

STRUCTURAL:

```md
# VM Bootstrap Tiering — Implementation State

Prepared spec baseline: 9f046812...
Implementation baseline: <actual>
Current checkpoint: C0

## Established invariants

None yet.

## Decisions

- D-01: Full `VM::new()` remains behavior-compatible.
- D-02: Mutable VM state is never process-shared.

## Evidence ledger

| Checkpoint | Command | Result | Proves |
|---|---|---|---|

## Deferred gates

- `cargo ... test --workspace --all-targets` → C5

## Active incident

None.

## Next resume action

C0 Task 2.
```

Testing classification:
- No behavioral test.

Checkpoint state update:
- local baseline and dirty set are authoritative for execution.

---

## Task 2 — Reconfirm current ownership and remove stale assumptions

Purpose:

Bound repository exploration and ensure the implementation starts from current code rather than older remediation notes.

Risk:
- Semantic: LOW
- Implementation fanout: inspection only

Owned files and symbols:
- no production edits

Inspect before editing:
- `VM::new_with_native_install_mode`
- `VM::run_universe_modules`
- `VM::universe_lowerings`
- `NativeSourceIndex::bootstrap_roots`
- `BuiltinInterfaceBuilder::{load_parsed,build}`
- `ProgramCompiler::compile_analyzed`
- `VM::run_compiled`

Do not inspect unless evidence forces expansion:
- parser grammar
- LSP adapters
- unrelated semantic features

Dependencies:
- Task 1 baseline

Source of truth:
- current repository code

Implementation boundary:

Changes:
- state-file findings only.

Must not:
- add `run_compiled` work if current code already skips initialized modules;
- plan a duplicate builtin parsed/interface cache.

Edit operations:

1. OPEN the listed symbols.
2. VERIFY `run_compiled` state check occurs before missing initializer compilation.
3. VERIFY builtin parsed/interface caches exist.
4. VERIFY current `bootstrap_roots` heuristic.
5. UPDATE state with any repository drift.
6. STOP if an assumption is contradicted.

Testing classification:
- Structural searches only.

Checkpoint state update:
- record exact ownership anchors used by C1.

---

# 6. Checkpoint C1 — Canonical Universe compiler derivation becomes a process-shared modules product

Tasks:
- Task 3 — Introduce `CanonicalUniverseProgram` ownership and one-time accessor.
- Task 4 — Move canonical verification/link/analysis/lowering derivation out of VM.
- Task 5 — Add compiler-product identity and singleton evidence.

Why this is a checkpoint:

Tasks 3–4 only become meaningful together. A new struct without moving the expensive derivation does not improve architecture; moving analysis without publishing one immutable product merely relocates the repeated work. C1 ends only when the modules/compiler layer owns one coherent canonical product that can later be consumed by VM bootstrap.

Entry conditions:
- C0 COMPLETE
- existing builtin parsed/interface caches confirmed
- `CompiledModule` / `CompiledProgram` confirmed as VM-independent

Working set:

Primary:
- `phalcom-core/src/modules/mod.rs`
- new `phalcom-core/src/modules/canonical_universe.rs`
- `phalcom-core/src/modules/compile.rs`
- `phalcom-core/src/modules/semantic_lowering.rs`
- `phalcom-core/src/native/source.rs`
- `phalcom-core/src/native/verify.rs`
- `phalcom-core/src/vm/bootstrap.rs` — source of code to move; do not change constructor behavior yet

Secondary — inspect only if evidence requires it:
- `phalcom-modules/src/builtin_interface.rs`
- `phalcom-modules/src/resolver.rs`
- `phalcom-modules/src/linker.rs`
- `phalcom-semantic/src/workspace.rs`
- `phalcom-semantic/src/core_surface/identity.rs`

Out of scope for this checkpoint:
- new VM constructors;
- semantic-root representation;
- test VM tier migration;
- changing bootstrap roots;
- bytecode caching;
- persisted artifacts.

Semantic contract established by this checkpoint:
- exactly one immutable canonical Universe compiler derivation exists per process;
- source-complete semantic analysis remains the formal authority;
- runtime-facing lowerings retain canonical declaration/variant identities;
- existing builtin parsed/interface caches remain the sole source cache authority;
- no VM handle enters the shared product.

Semantic risks:
- losing canonical `Result::Ok`/`Error` identity;
- accidentally retaining store-local semantic state;
- introducing a second source/interface cache;
- changing the current bootstrap diagnostic policy;
- `OnceLock` product failing `Sync` and being “fixed” unsafely.

Hostile cases:
- user/nonnative declaration named `Result` must remain distinct from canonical `UniverseKey::Result`;
- repeated accessor calls must not create distinct semantic products;
- a canonical build failure must not cause repeated expensive rebuild attempts;
- shared product must not store `ObjRef`, `ClassId`, runtime `Value`, or a mutable semantic session.

Required evidence:

1. ```bash
   cargo +nightly-2026-07-10 test -p phalcom-core \
     canonical_universe_program_is_process_singleton -- --nocapture
   ```
   Proves repeated access returns one immutable process product.

2. ```bash
   cargo +nightly-2026-07-10 test -p phalcom-core \
     canonical_universe_program_preserves_result_identity -- --nocapture
   ```
   Proves the compiler product owns the exact canonical `Result` declaration/variants, not a name-based approximation.

3. ```bash
   cargo +nightly-2026-07-10 check -p phalcom-core --all-targets
   ```
   Proves module/API fanout compiles.

4. ```bash
   rg -n "BUILTIN_PARSED_CACHE|BUILTIN_INTERFACE_CACHE" phalcom-core/src
   ```
   Expected: zero newly introduced core-owned duplicates.

Do not run yet:
- full `--test core`
- workspace tests
- Either full module

Escalate immediately if:
- `CompiledProgram` or any required retained field is not `Send + Sync`;
- satisfying `OnceLock` appears to require `unsafe impl Send/Sync`;
- canonical Universe analysis currently publishes errors and `ProgramCompiler::compile_analyzed` would newly reject bootstrap;
- the shared product appears to require keeping a raw `SemanticSnapshot` solely to make runtime compile work.

Checkpoint completion:
- [ ] all C1 tasks implemented
- [ ] singleton evidence passes
- [ ] canonical Result lowering evidence passes
- [ ] no duplicate parsed/interface cache
- [ ] no unsafe Send/Sync workaround
- [ ] state updated
- [ ] no active incident

Suggested commit grouping:
- `refactor(core): introduce shared canonical universe compiler program`
- `test(core): pin canonical universe program identity`

---

## Task 3 — Introduce `CanonicalUniverseProgram` and one-time accessor

Purpose:

Create the modules/compiler-owned immutable product that represents all expensive canonical compiler derivation reusable across VMs.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- new `phalcom-core/src/modules/canonical_universe.rs` — product + builder/accessor
- `phalcom-core/src/modules/mod.rs` — module/export surface
- `phalcom-core/src/modules/compile.rs` — existing `CompiledProgram` reuse/extraction if necessary

Inspect before editing:
- `CompiledProgram`
- `CompiledModule`
- `AnalyzedProgram`
- `ProgramCompiler::compile_analyzed`
- `NativeSourceIndex`
- `ModuleLoweringSemantics`

Do not inspect unless evidence forces expansion:
- VM dispatch
- heap implementation
- parser grammar
- LSP

Dependencies:
- C0 ownership map

Source of truth:
- `CompiledProgram` for VM-independent linked compiler representation
- `NativeSourceIndex` for current retained canonical AST corpus
- current `bootstrap_roots` semantics for execution-order selection

Implementation boundary:

Changes:
- add one immutable canonical wrapper;
- add one `OnceLock<Result<...>>` accessor;
- memoize construction failure.

Must not:
- store `VM`, `Heap`, `ObjRef`, `ClassId`, `Value`, closures, chunks with VM-local caches;
- add a second parsed/interface cache;
- retain a global semantic session.

Current implementation:

There is no high-level shared compiler product. `VM::new_with_native_install_mode` builds `NativeSourceIndex` and later `VM::universe_lowerings` derives linked/semantic/lowering state per VM.

Target implementation:

```text
modules::canonical_universe_program()
    ↓ first call
build CanonicalUniverseProgram
    ↓
publish immutable result

later calls
    ↓
borrow same product
```

Edit operations:

1. OPEN `phalcom-core/src/modules/mod.rs`.
2. ADD `pub mod canonical_universe;` or repository-consistent visibility.
3. CREATE `phalcom-core/src/modules/canonical_universe.rs`.
4. DEFINE a wrapper around existing `CompiledProgram` plus current source/index/order products.
5. DEFINE a thread-safe one-time result cell.
6. DEFINE a fallible accessor returning a shared reference.
7. SEARCH the new type for forbidden VM-local handle types.
8. CLEAN imports.

Code instructions:

STRUCTURAL:

```rust
use std::sync::{Arc, OnceLock};

pub struct CanonicalUniverseProgram {
    pub program: Arc<CompiledProgram>,
    pub source_index: Arc<crate::native::NativeSourceIndex>,
    pub root_reachable: Arc<[ModuleId]>,
    pub bootstrap_order: Arc<[ModuleId]>,
}

static CANONICAL_UNIVERSE_PROGRAM:
    OnceLock<Result<CanonicalUniverseProgram, CanonicalUniverseBuildError>> =
    OnceLock::new();

pub fn canonical_universe_program(
) -> Result<&'static CanonicalUniverseProgram, &'static CanonicalUniverseBuildError> {
    match CANONICAL_UNIVERSE_PROGRAM.get_or_init(build_canonical_universe_program) {
        Ok(program) => Ok(program),
        Err(error) => Err(error),
    }
}
```

This shape is not paste-mandatory. Reuse repository error types where that produces a cleaner API.

Important Rust mechanics:
- storing `Result` inside `OnceLock` memoizes failures;
- do not `panic!` inside `get_or_init`, because a panic leaves the cell uninitialized and can repeat expensive work later;
- no mutex is required for reads after `OnceLock` initialization.

Testing classification:
- tested at C1 boundary.

Optional compile checkpoint:
```bash
cargo +nightly-2026-07-10 check -p phalcom-core
```
Reason: catches `Send + Sync`, visibility, and error-type fanout before moving semantic derivation.

Checkpoint state update:
- record final product name/signature;
- record whether `CompiledProgram` was reused directly or a narrow extraction was required.

---

## Task 4 — Move canonical verification/link/analysis/lowering derivation out of VM

Purpose:

Make C1's product actually own the expensive process-invariant compiler work.

Risk:
- Semantic: HIGH
- Implementation fanout: cross-subsystem within `phalcom-core`

Owned files and symbols:
- `phalcom-core/src/vm/bootstrap.rs` — existing `universe_lowerings` body to migrate
- `phalcom-core/src/modules/canonical_universe.rs` — new owner
- `phalcom-core/src/modules/compile.rs` — reuse `AnalyzedProgram`/`CompiledProgram`
- `phalcom-core/src/native/verify.rs` — consume only, do not redesign

Inspect before editing:
- exact current `VM::universe_lowerings`
- `ProgramCompiler::compile_analyzed`
- `build_module_lowering_semantics`
- `NativeSourceIndex::bootstrap_roots`
- `NativeSourceIndex::initialization_order_from_roots`
- `NativeSourceIndex::reachable_units_from_roots`

Do not inspect unless evidence forces expansion:
- inference internals
- ADT runtime registry
- LSP

Dependencies:
- Task 3 product/accessor exists

Source of truth:
- source-complete linked semantic analysis
- `ModuleLoweringSemantics` for runtime compiler facts

Implementation boundary:

Changes:
- builder creates `NativeSourceIndex`;
- builder performs `verify_native_contracts`;
- builder loads all canonical interfaces, resolves imports, and links the source-complete Universe;
- builder runs exactly one `analyze_workspace`;
- builder projects modules into existing `CompiledModule` shape;
- builder computes root-reachable and current eager bootstrap order once.

Must not:
- preserve the old per-unit source-incomplete loop;
- change current eager-root selection;
- introduce name-based identity fallback;
- silently change whether semantic diagnostics abort bootstrap without first characterizing the current snapshot.

Current implementation:

`VM::universe_lowerings(&self, source_index)` creates a `ProjectUniverse`, resolves interfaces/imports, links all canonical source, calls `analyze_workspace`, and maps every source unit through `build_module_lowering_semantics`.

Target implementation:

The same semantic flow runs inside `build_canonical_universe_program()` once per process.

Edit operations:

1. COPY the logical flow of `VM::universe_lowerings` into the new modules-owned builder.
2. REMOVE any dependence on `&VM`; if one appears, classify it as an ownership leak and investigate.
3. BUILD `sources` from `source_index.units`.
4. RUN one source-complete `analyze_workspace`.
5. BEFORE using `ProgramCompiler::compile_analyzed`, CHECK `analysis.snapshot.has_errors()` behavior.
6. IF clean:
   - construct `AnalyzedProgram`;
   - reuse `ProgramCompiler::compile_analyzed`.
7. IF not clean:
   - do not broaden scope;
   - extract/reuse the module projection portion needed to produce `CompiledModule`s while preserving existing bootstrap acceptance behavior;
   - record the discrepancy as a repository finding.
8. COMPUTE:
   - root-only reachability;
   - `bootstrap_roots`;
   - dependency-first bootstrap order.
9. CONVERT order to stable `ModuleId` list.
10. RETURN immutable product.
11. LEAVE current VM caller behavior unchanged until C3.
12. DO NOT delete `VM::universe_lowerings` until C3 unless C1 temporarily delegates it to the shared product without behavior change.

Code instructions:

STRUCTURAL:

```rust
fn build_canonical_universe_program()
    -> Result<CanonicalUniverseProgram, CanonicalUniverseBuildError>
{
    let source_index = Arc::new(NativeSourceIndex::build()?);

    let descriptors = crate::native::PRIMITIVES.iter().collect::<Vec<_>>();
    crate::native::verify_native_contracts(&source_index, &descriptors)?;

    // Preserve current source-complete resolution/linking flow.
    let linked = ...;

    let sources = source_index
        .units
        .iter()
        .map(|unit| (unit.id.clone(), unit.clone()))
        .collect();

    let analysis = phalcom_semantic::analyze_workspace(
        phalcom_semantic::SemanticWorkspaceInput {
            linked: Arc::new(linked),
            sources: sources.clone(),
            generation: 0,
        },
    );

    // Reuse existing compile/projection representation without retaining
    // the SemanticSnapshot in the published shared runtime product.
    let program = ...;

    let root = ModuleId::universe_root();
    let root_reachable = source_index.reachable_units_from_roots(
        std::slice::from_ref(&root),
    )?;

    let roots = source_index.bootstrap_roots();
    let bootstrap_order = source_index
        .initialization_order_from_roots(&roots)?
        .into_iter()
        .map(|unit| unit.id.clone())
        .collect::<Vec<_>>();

    Ok(...)
}
```

Testing classification:
- focused semantic/compiler-product tests required at C1.

Checkpoint state update:
- record whether canonical analysis is diagnostic-clean;
- record exact error representation;
- record confirmed absence of VM-local handles.

---

## Task 5 — Prove singleton behavior and canonical Result identity at the product boundary

Purpose:

Defeat the two easiest incorrect implementations:
1. rebuilding the canonical product every time;
2. sharing a cheap but semantically incomplete lowering.

Risk:
- Semantic: HIGH
- Implementation fanout: local test

Owned files and symbols:
- preferably `phalcom-core/src/modules/canonical_universe.rs` test module
- or existing compiler semantic-boundary integration test if repository style clearly prefers it

Inspect before editing:
- `phalcom_semantic::core_surface::universe_declaration`
- `UniverseKey::Result`
- `ModuleLoweringSemantics::enums`

Do not inspect unless evidence forces expansion:
- runtime `ClassId`
- LSP

Dependencies:
- Tasks 3–4

Source of truth:
- canonical declaration helper and lowering product

Implementation boundary:

Changes:
- add singleton pointer identity test;
- add exact canonical Result owner assertion.

Must not:
- assert by bare string only;
- require a VM;
- compare runtime class IDs.

Edit operations:

1. ADD `canonical_universe_program_is_process_singleton`.
2. CALL accessor twice.
3. ASSERT `std::ptr::eq(first, second)` or `Arc::ptr_eq` on the retained `CompiledProgram`.
4. ADD `canonical_universe_program_preserves_result_identity`.
5. COMPUTE:
   ```rust
   let expected =
       phalcom_semantic::core_surface::universe_declaration(
           phalcom_native_meta::UniverseKey::Result
       );
   ```
6. LOCATE the canonical Result module by `UniverseKey::Result.source_path()`, not by scanning user-visible short names globally.
7. ASSERT the Result enum lowering owner equals `expected`.
8. If variant IDs are conveniently available, additionally assert every projected Result variant owner/family identity belongs to that exact declaration.

Testing classification:
- focused regression required.

Checkpoint state update:
- record exact test names and pass results.

---

# 7. Checkpoint C2 — VM bootstrap stages and semantic capability are explicit

Tasks:
- Task 6 — Extract kernel-only VM construction.
- Task 7 — Extract native-runtime construction.
- Task 8 — Make semantic roots explicitly optional and migrate consumers.
- Task 9 — Compose existing full constructor from the staged API without yet deleting the old lowering source.

Why this is a checkpoint:

Kernel/native constructors are unsafe to expose while `semantic_roots` can contain fake Nil/default identities. Conversely, optional semantic roots are not useful until constructors establish coherent states. These tasks must land together so each advertised VM tier is valid.

Entry conditions:
- C1 COMPLETE
- canonical compiler product exists independently of VM

Working set:

Primary:
- `phalcom-core/src/vm/bootstrap.rs`
- `phalcom-core/src/vm/mod.rs`
- `phalcom-core/src/universe/mod.rs`
- `phalcom-core/src/primitive/int.rs`
- `phalcom-core/src/primitive/number.rs`
- `phalcom-core/src/vm/dispatch.rs`
- `phalcom-core/src/vm/gc.rs`

Secondary:
- `phalcom-core/src/modules/builtin_materialize.rs`
- `phalcom-core/src/native/install.rs`
- exact additional `semantic_roots` search hits

Out of scope:
- changing source execution order;
- canonical cache consumption by `run_universe_modules` (C3);
- test-suite-wide constructor migration (C4).

Semantic contract established:
- `VM::new_kernel()` is a valid long-lived kernel VM;
- `VM::new_native()` is a valid native-floor VM without source semantic roots;
- `VM::new()` remains full;
- missing source-derived roots are represented by absence, never fake runtime identities.

Semantic risks:
- lower-tier primitive silently returning Nil;
- GC missing roots in full VMs;
- changing pristine-flag semantics;
- native constructor accidentally invoking canonical semantic compilation;
- `new_with_native_install_mode` behavior drift.

Hostile cases:
- invoke a source-root-dependent internal operation on native VM and verify explicit failure, not Nil;
- `VM::new_native()` must report zero source-executed units;
- kernel VM must not materialize canonical Universe module objects;
- full VM must still produce all three semantic roots.

Required evidence:

1. focused new bootstrap stage tests;
2. ```bash
   cargo +nightly-2026-07-10 test -p phalcom-core \
     system_print_runtime_returns_unit system_gc_runtime_returns_unit -- --nocapture
   ```
   If Cargo filtering cannot select both in one command, run the containing `native_surface_contracts` filter once.
3. ```bash
   cargo +nightly-2026-07-10 test -p phalcom-core \
     verify_invariants_holds_after_bootstrap -- --nocapture
   ```
   Proves full constructor remains valid.
4. ```bash
   rg -n "semantic_roots\.(unsupported|ellipsis|ordering_class)" phalcom-core/src
   ```
   Expected: only deliberately migrated guarded/accessor sites; no unchecked use that assumes full bootstrap.

Do not run yet:
- Either full module;
- full `core` target;
- workspace.

Escalate immediately if:
- `Universe::new` itself unexpectedly depends on materialized Universe modules;
- native primitive installation requires source-derived semantic roots during installation;
- converting semantic roots to `Option` appears to require changing user-language behavior for full VMs.

Checkpoint completion:
- [ ] kernel constructor valid
- [ ] native constructor valid
- [ ] semantic roots explicit
- [ ] full constructor still composes
- [ ] focused tests pass
- [ ] state updated
- [ ] no incident

Suggested commits:
- `refactor(vm): split kernel and native bootstrap stages`
- `refactor(vm): make source semantic roots explicit`

---

## Task 6 — Extract `VM::new_kernel()`

Purpose:

Create the minimum fresh VM state without any canonical source/native materialization.

Risk:
- Semantic: MEDIUM
- Implementation fanout: local VM constructor

Owned files and symbols:
- `phalcom-core/src/vm/bootstrap.rs` — constructor prefix
- `phalcom-core/src/vm/mod.rs` — docs if necessary
- `phalcom-core/src/universe/mod.rs` — comments naming `VM::new`

Inspect before editing:
- full field initialization in `new_with_native_install_mode`
- `Universe::new`
- root `FiberObject::root`
- GC exhaustive destructuring of VM fields

Do not inspect unless evidence forces expansion:
- semantic analyzer
- linker
- test fixtures

Dependencies:
- C1 only for eventual composition; `new_kernel` must not call C1 accessor

Source of truth:
- current `VM` field layout and `Universe::new`

Implementation boundary:

Changes:
- extract initial allocation/field initialization into `new_kernel`;
- leave runtime/source roots unavailable;
- retain default bootstrap measurement.

Must not:
- initialize canonical modules;
- install native descriptors;
- call canonical program accessor;
- add fake source roots.

Current implementation:
- all fields initialized directly in `new_with_native_install_mode`.

Target:
```rust
pub fn new_kernel() -> Self {
    // heap, Universe kernel, root fiber, empty maps/registries
}
```

Edit operations:

1. OPEN `new_with_native_install_mode`.
2. IDENTIFY the exact prefix ending before `NativeSourceIndex::build` / canonical materialization.
3. EXTRACT kernel allocation/field construction.
4. UPDATE comments that currently say certain state is finalized by `VM::new`.
5. ENSURE `runtime_roots` remains `None`.
6. ENSURE semantic roots use the new explicit absent state from Task 8; if sequencing within the checkpoint requires temporary compilation, do not commit an invalid intermediate state.
7. UPDATE exhaustive VM destructures as Rust compiler requires.

Code instructions:

STRUCTURAL only; field list is large and current repository must remain authoritative.

Testing classification:
- C2 stage tests.

Optional compile checkpoint:
```bash
cargo +nightly-2026-07-10 check -p phalcom-core
```
Reason: large struct/exhaustive-destructure fanout.

---

## Task 7 — Extract `VM::new_native()`

Purpose:

Expose a coherent native runtime floor that avoids source semantic compilation/execution.

Risk:
- Semantic: MEDIUM
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-core/src/vm/bootstrap.rs`
- existing `initialize_canonical_universe`
- current primitive installation calls

Inspect before editing:
- sequence between canonical module materialization and `run_universe_modules`
- `bind_primordial_universe`
- hard-coded field stamps
- `install_registered_primitives`
- typing primitive installation
- `finalize_all_primordial_base_names`
- `sync_universe_class_aliases`

Do not inspect unless evidence forces expansion:
- semantic solver
- runtime ADT generalization

Dependencies:
- Task 6
- Task 8 integrated in same checkpoint

Source of truth:
- current pre-source portion of `new_with_native_install_mode`

Implementation boundary:

Changes:
- compose from kernel;
- materialize canonical modules/native bindings;
- install native runtime;
- stop before source `.ph` execution.

Must not:
- call `canonical_universe_program`;
- call `NativeSourceIndex::build`;
- call `analyze_workspace`;
- populate source-derived semantic roots.

Target shape:

```rust
pub fn new_native() -> Self {
    Self::new_native_with_native_install_mode(NativeInstallMode::DescriptorOnly)
}

fn new_native_with_native_install_mode(mode: NativeInstallMode) -> Self {
    let mut vm = Self::new_kernel();
    // existing native/materialization stage
    vm
}
```

`new_with_native_install_mode(mode)` remains full and should later call the internal native constructor.

Testing classification:
- C2.

---

## Task 8 — Replace fake semantic roots with explicit availability

Purpose:

Make lower bootstrap tiers semantically honest and prevent Nil/default identities from becoming long-lived.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-core/src/vm/mod.rs` — `semantic_roots`
- `phalcom-core/src/vm/bootstrap.rs` — initialization/late binding
- `phalcom-core/src/primitive/int.rs`
- `phalcom-core/src/primitive/number.rs`
- `phalcom-core/src/vm/dispatch.rs`
- `phalcom-core/src/vm/gc.rs`

Inspect before editing:
- every `semantic_roots` use:
  ```bash
  rg -n "semantic_roots" phalcom-core/src
  ```
- existing `RuntimeError` internal/bootstrap variants

Do not inspect unless evidence forces expansion:
- language error design
- parser
- LSP

Dependencies:
- Tasks 6–7 coordinated

Source of truth:
- actual source-derived values produced after full Universe execution

Implementation boundary:

Changes:
- field becomes `Option<SemanticRoots>` or equivalent;
- full bootstrap assigns `Some(...)`;
- GC conditionally traces;
- semantic-dependent operations use a checked accessor.

Must not:
- retain Nil/default placeholders;
- auto-bootstrap Universe on missing roots;
- turn missing internal capability into user-visible `Dynamic`.

Current:
```rust
semantic_roots: SemanticRoots {
    unsupported: Value::nil(),
    ellipsis: Value::nil(),
    ordering_class: ClassId::default(),
}
```

Target:

```rust
semantic_roots: Option<SemanticRoots>
```

and a centralized accessor, where practical.

Edit operations:

1. CHANGE VM field type.
2. CHANGE kernel initialization to `None`.
3. CHANGE full late binding to `Some(SemanticRoots { ... })`.
4. UPDATE GC:
   - trace unsupported/ellipsis only when `Some`.
5. UPDATE int/number primitives:
   - obtain roots through checked accessor before returning unsupported.
6. UPDATE dispatch:
   - `GetEllipsis` and Ordering-dependent operations require roots explicitly.
7. SEARCH for any direct field use remaining.
8. ADD hostile lower-tier test verifying missing-root operation does not yield private Nil.

Code instructions:

STRUCTURAL:

```rust
pub(crate) fn require_semantic_roots(
    &self,
) -> Result<&SemanticRoots, RuntimeError> {
    self.semantic_roots.as_ref().ok_or_else(|| {
        RuntimeError::Internal(
            "operation requires source-authored Universe bootstrap".into()
        )
    })
}
```

Reuse a better existing error if repository inspection finds one.

Testing classification:
- focused hostile regression required at C2.

---

## Task 9 — Recompose full constructor from staged APIs

Purpose:

Preserve public behavior while eliminating duplicated constructor sequencing.

Risk:
- Semantic: HIGH
- Implementation fanout: local but central

Owned files and symbols:
- `phalcom-core/src/vm/bootstrap.rs` — `new`, `new_with_native_install_mode`

Inspect before editing:
- exact post-source steps:
  - `sync_universe_class_aliases`
  - semantic roots lookup
  - pristine flags
  - None assertion
  - `verify_invariants`

Dependencies:
- Tasks 6–8

Source of truth:
- existing `new_with_native_install_mode` ordering

Implementation boundary:

Changes:
- full constructor composes native stage then current source stage;
- preserve all post-source finalization exactly.

Must not:
- use shared canonical plan in the VM source loop yet if doing so would make C2 failure attribution harder; C3 owns that migration;
- change full runtime observable state.

Target concept:

```rust
pub fn new_with_native_install_mode(mode: NativeInstallMode) -> Self {
    let mut vm = Self::new_native_with_native_install_mode(mode);

    // Existing source bootstrap path for now.
    ...
    vm
}
```

Testing classification:
- C2 full-bootstrap control.

Checkpoint state update:
- record exact phase methods/order.

---

# 8. Checkpoint C3 — Full VM consumes shared canonical compiler state

Tasks:
- Task 10 — Change `run_universe_modules` to consume `CanonicalUniverseProgram`.
- Task 11 — Delete VM-owned linking/semantic/lowering derivation and per-VM conformance verification.
- Task 12 — Prove full runtime canonical behavior and isolation.

Why this is a checkpoint:

This is the semantic cutover. C1 built the replacement and C2 made constructor stages coherent, but test-time cost is not fixed until full VM bootstrap stops recomputing the compiler products. Required evidence must prove both the performance architecture and the canonical identity behaviors that the source-complete repair introduced.

Entry conditions:
- C1 COMPLETE
- C2 COMPLETE
- no semantic-root incident

Working set:

Primary:
- `phalcom-core/src/vm/bootstrap.rs`
- `phalcom-core/src/modules/canonical_universe.rs`
- `phalcom-core/src/native/source.rs`
- `phalcom-core/tests/core/modules/universe.rs`
- `phalcom-core/tests/native_adt_runtime.rs`
- `phalcom-core/tests/core/collections/contract.rs`
- `phalcom-core/tests/core/either/runtime.rs`

Secondary:
- `phalcom-core/src/modules/compile.rs`
- compiler binding helpers if shared `CompiledProgram.linked` needs adaptation

Out of scope:
- lowering source execution set;
- caching bytecode/closures;
- general user-project compilation.

Semantic contract:
- `VM::new()` performs no canonical source resolution/linking/semantic analysis/lowering projection per VM;
- every full VM consumes the same immutable canonical compiler product;
- every full VM still compiles/executes fresh source initialization state;
- canonical Result/variant semantics remain correct.

Semantic risks:
- wrong lowering attached to module;
- bootstrap order drift;
- source lookup by vector position instead of `ModuleId`;
- accidental omission of source/native verification;
- shared state leaking mutable VM data.

Hostile cases:
- user enum named `Result` stays distinct;
- two full VMs do not share mutable globals;
- full VM still exposes curated prelude;
- range/Result variant construction does not produce `unregistered variant`.

Required evidence:

1. ```bash
   RUSTFLAGS='' RUSTC_WRAPPER='' \
   cargo +nightly-2026-07-10 test -p phalcom-core --test core \
     range_literals_drive_collection_slices -- --nocapture
   ```
   Proves canonical range/variant lowering reaches runtime correctly.

2. ```bash
   RUSTFLAGS='' RUSTC_WRAPPER='' \
   cargo +nightly-2026-07-10 test -p phalcom-core --test core \
     curated_prelude_exposes_public_names_and_hides_internal_classes -- --nocapture
   ```
   Proves full source bootstrap/prelude state remains intact.

3. ```bash
   cargo +nightly-2026-07-10 test -p phalcom-core --test core \
     canonical_result_reuses_primordial_runtime_root -- --nocapture
   ```
   Proves canonical Result declaration/runtime root identity.

4. ```bash
   cargo +nightly-2026-07-10 test -p phalcom-core --test core \
     either::runtime::either_runtime_surface_produces_expected_values -- --nocapture
   ```
   Proves higher-order/generic runtime still operates through full bootstrap.

5. Negative migration gate:
   ```bash
   rg -n "analyze_workspace|ModuleLinker|universe_lowerings|verify_native_contracts" \
     phalcom-core/src/vm/bootstrap.rs
   ```
   Expected:
   - no `analyze_workspace`;
   - no `ModuleLinker`;
   - no `universe_lowerings`;
   - no `verify_native_contracts`.
   Any intentional textual occurrence must be a non-executing comment justified in state.

Do not run yet:
- full workspace
- reflection mega-filter unless one of the changed paths directly breaks its focused ownership test

Escalate immediately if:
- any canonical identity test passes only after restoring a fallback;
- shared plan order differs from current `NativeSourceIndex` order selection;
- per-VM compiler path still resolves imports because the shared product omitted required linked binding information.

Checkpoint completion:
- [ ] VM frontend/compiler derivation removed
- [ ] exact full runtime regressions pass
- [ ] negative gate clean
- [ ] isolation hostile test passes
- [ ] state updated
- [ ] no incident

Suggested commits:
- `refactor(vm): consume shared canonical universe program`
- `test(core): preserve full bootstrap canonical identities`

---

## Task 10 — Make `run_universe_modules` consume the shared plan

Purpose:

Turn the current VM source bootstrap loop into a pure VM-local consumer of precomputed compiler state.

Risk:
- Semantic: HIGH
- Implementation fanout: local core runtime/compiler boundary

Owned files and symbols:
- `phalcom-core/src/vm/bootstrap.rs` — `run_universe_modules`
- `phalcom-core/src/modules/canonical_universe.rs`

Inspect before editing:
- `CompiledProgram.modules`
- `CompiledModule.lowering`
- linked compile bindings
- source index lookup capabilities

Do not inspect unless evidence forces expansion:
- semantic checker internals

Dependencies:
- C1 shared plan
- C2 staged constructors

Source of truth:
- `CanonicalUniverseProgram`
- `CompiledModule.lowering`
- precomputed bootstrap order

Implementation boundary:

Changes:
- signature accepts shared canonical plan;
- no dependency/order/lowering recomputation;
- select parsed unit by canonical `ModuleId`;
- attach exact precomputed lowering;
- compile/run source in fresh VM as today.

Must not:
- index source by incidental vector position without identity validation;
- compile modules outside existing bootstrap order;
- share runtime closure objects.

Current:
```rust
fn run_universe_modules(
    &mut self,
    source_index: &NativeSourceIndex,
) -> PhResult<()> {
    let root_reachable = ...;
    let bootstrap_roots = ...;
    let units = ...;
    let lowerings = self.universe_lowerings(source_index)?;
    ...
}
```

Target:

```rust
fn run_universe_modules(
    &mut self,
    canonical: &CanonicalUniverseProgram,
) -> PhResult<()> {
    self.universe_bootstrap_measurement = ...;

    for id in canonical.bootstrap_order.iter() {
        let parsed = canonical.source_index
            .unit(id)
            .ok_or_else(...)?;
        let compiled = canonical.program.modules
            .get(id)
            .ok_or_else(...)?;

        let module = ...;
        self.heap.module_mut(module).lowering =
            Some(compiled.lowering.clone());

        // Current VM-local AST compile + execute.
        ...
    }
}
```

If `NativeSourceIndex` lacks an efficient `unit(&ModuleId)` lookup, prefer adding a small authoritative lookup method over constructing an unrelated map in VM bootstrap. Whether to add an internal `by_id` index is MEDIUM-risk structural work; preserve one owner.

Testing classification:
- C3.

---

## Task 11 — Delete VM-owned semantic/linking derivation

Purpose:

Complete the ownership migration so old and new authorities cannot silently coexist.

Risk:
- Semantic: HIGH
- Implementation fanout: local deletion with import cleanup

Owned files and symbols:
- `phalcom-core/src/vm/bootstrap.rs` — `universe_lowerings`, per-VM source-index/verification setup

Inspect before editing:
- current constructor after Task 9
- new canonical accessor

Dependencies:
- Task 10 compiled and focused checks viable

Source of truth:
- modules-owned canonical product

Implementation boundary:

Changes:
- full constructor obtains canonical plan;
- removes per-VM `NativeSourceIndex::build`;
- removes per-VM `verify_native_contracts`;
- deletes `VM::universe_lowerings`;
- removes linker/resolver/semantic imports no longer used by bootstrap.

Must not:
- leave dead fallback path “just in case”;
- retain a second source-complete analysis behind debug/test cfg;
- move the old method under another VM module.

Edit operations:

1. REPLACE constructor source-index setup with canonical accessor.
2. PASS canonical product to source bootstrap.
3. DELETE `VM::universe_lowerings`.
4. DELETE obsolete imports.
5. RUN negative search.
6. SEARCH repository for other production direct calls to the deleted method.
7. UPDATE comments/docs that still claim `VM::new` performs semantic analysis.

Testing classification:
- validated at C3 boundary.

---

## Task 12 — Add VM isolation and full-runtime cutover evidence

Purpose:

Prove sharing stopped at immutable compiler products.

Risk:
- Semantic: HIGH
- Implementation fanout: local test

Owned files and symbols:
- choose `phalcom-core/tests/core/modules/universe.rs` or a narrow bootstrap-specific test module

Inspect before editing:
- existing module/global helper methods

Dependencies:
- Tasks 10–11

Source of truth:
- each VM's own heap/module registry

Implementation boundary:

Changes:
- create two full VMs;
- mutate a VM-local binding/module state in one;
- prove second remains unchanged.

Must not:
- assert raw `ObjRef` or `ClassId` numbers differ across heaps;
- rely on test order.

Example hostile structure:

```rust
let mut first = VM::new();
let mut second = VM::new();

let first_mod = first.create_module("isolation_probe", "<test>");
let name = first.interner.intern("x");
first.define_global(first_mod, name, Value::int(42)).unwrap();

// Resolve/create an independent module in second and prove no binding leaked.
// Do not compare cross-heap handles numerically.
```

Prefer testing a mutable canonical/root global only if the test can restore no state and each VM is independent; a synthetic module is safer but proves general heap isolation, not canonical module isolation. A stronger test may mutate an existing canonical module binding in VM1 and show VM2 retains its original value, if namespace freeze rules permit it.

Testing classification:
- focused hostile regression required.

---

# 9. Checkpoint C4 — Minimum-runtime test migration

Tasks:
- Task 13 — Extend the existing test-support VM tier seam.
- Task 14 — Migrate proven kernel/native-only tests.
- Task 15 — Mark and preserve full-Universe controls.

Why this is a checkpoint:

The production architecture already improves repeated full VM cost at C3. C4 realizes the second performance axis: tests that do not semantically depend on source-authored Universe should not request it. This is test-architecture work, not a blanket textual replacement.

Entry conditions:
- C3 COMPLETE
- kernel/native/full constructors stable

Working set:

Primary:
- `phalcom-core/tests/support/vm.rs`
- `phalcom-core/tests/README.md`
- `phalcom-core/src/product.rs`
- `phalcom-core/src/chunk.rs`
- `phalcom-core/tests/core/native/contracts.rs`

Secondary — inspect only when classifying:
- `phalcom-core/tests/core/object_model/invariants.rs`
- other `VM::new()` hits from audit

Out of scope:
- source-language suites that intentionally need full Universe;
- global bulk replacement;
- fuzz target migration unless separately proven safe.

Semantic contract:
- test fixtures state their runtime dependency;
- low-level tests no longer implicitly compile the Universe;
- full-runtime tests remain full and continue to prove shipping behavior.

Semantic risks:
- downgrading a test whose behavior secretly depends on source-installed methods;
- changing test meaning merely to make it faster;
- helper proliferation.

Hostile cases:
- at least one explicit full-runtime test remains in every domain that needs source behavior;
- native-only direct primitive test works without semantic roots;
- a test that needs source `List`/`Result` is not migrated.

Required evidence:

1. ```bash
   cargo +nightly-2026-07-10 test -p phalcom-core \
     empty_products_normalize_to_unit_without_heap_allocation -- --nocapture
   ```
   Proves the known low-level reproducer no longer needs full Universe.

2. ```bash
   cargo +nightly-2026-07-10 test -p phalcom-core \
     ic_add_method_invalidates_impl -- --nocapture
   ```
   Proves a low-level cache/object-model test can use the minimal appropriate tier.

3. ```bash
   cargo +nightly-2026-07-10 test -p phalcom-core --test core \
     native_surface_contracts -- --nocapture
   ```
   Proves direct native-floor tests use the native tier correctly.

4. Full control:
   ```bash
   cargo +nightly-2026-07-10 test -p phalcom-core --test core \
     either::runtime::either_runtime_surface_produces_expected_values -- --nocapture
   ```
   Proves source-runtime helpers still request full semantics.

5. Audit:
   ```bash
   rg -n "VM::new\(" phalcom-core/src phalcom-core/tests phalcom-fuzz
   ```
   Every remaining hit is classified in state as:
   - intentionally full;
   - not yet migrated and justified;
   - out of scope.

Do not run yet:
- workspace test
- clippy
- all-targets test until C5

Escalate immediately if:
- a low-level test only passes under full VM because of an undocumented source dependency;
- moving it to native/kernel would require production semantics changes unrelated to its invariant.

Checkpoint completion:
- [ ] fixture API documented
- [ ] selected low-level tests migrated
- [ ] remaining full tests classified
- [ ] focused groups pass
- [ ] state updated
- [ ] no incident

Suggested commits:
- `test(core): expose kernel native and universe VM fixtures`
- `test(core): use minimum bootstrap tier for low-level runtime tests`

---

## Task 13 — Extend `tests/support/vm.rs` with explicit tiers

Purpose:

Reuse the existing domain-neutral fixture seam so tests do not scatter constructor policy.

Risk:
- Semantic: LOW
- Implementation fanout: local

Owned files and symbols:
- `phalcom-core/tests/support/vm.rs`
- `phalcom-core/tests/README.md`

Inspect before editing:
- `run_inline`
- `compile_inline`
- support module exports

Dependencies:
- C3 constructors

Source of truth:
- production constructors

Implementation boundary:

Changes:
- add tiny helpers `kernel_vm`, `native_vm`, `universe_vm`;
- document tier responsibilities;
- keep `run_inline` / `compile_inline` full unless independently characterized.

Must not:
- create cached/static VM;
- hide full bootstrap behind a helper named merely `vm()`.

Code instructions:

EXACT in spirit:

```rust
pub(crate) fn kernel_vm() -> VM {
    VM::new_kernel()
}

pub(crate) fn native_vm() -> VM {
    VM::new_native()
}

pub(crate) fn universe_vm() -> VM {
    VM::new()
}
```

Update test README with the tier table and rule:

> choose the lowest tier whose semantic contract contains the behavior under test; source-language helpers remain full by default.

Testing classification:
- no standalone behavior beyond C4.

---

## Task 14 — Migrate unambiguous low-level tests

Purpose:

Remove accidental full-Universe dependency from tests that exercise lower layers only.

Risk:
- Semantic: MEDIUM
- Implementation fanout: multi-file tests

Owned files and symbols:
- `phalcom-core/src/product.rs` — `empty_products_normalize_to_unit_without_heap_allocation`
- `phalcom-core/src/chunk.rs` — inline/global cache unit tests such as `ic_add_method_invalidates_impl`
- `phalcom-core/tests/core/native/contracts.rs` — direct System native runtime tests
- additional audit hits only after inspection

Inspect before editing for each test:
1. what function is called;
2. whether it performs source compilation;
3. whether it sends to source-installed methods;
4. whether it uses semantic roots;
5. whether native primitives are required.

Dependencies:
- Task 13

Source of truth:
- test's actual invariant, not desired speed

Implementation boundary:

Changes:
- product/heap-only → kernel where valid;
- direct native primitive/runtime-floor → native where valid;
- leave source-dependent cases full.

Must not:
- rewrite production code to make a test fit a lower tier;
- migrate by global search/replace.

Classification examples:

```text
empty product normalization
    likely KERNEL
    assertion is allocation/value behavior

direct System.print/System.gc Rust primitive
    NATIVE
    native class/floor behavior, no source body needed

inline cache method installation
    KERNEL or NATIVE
    inspect whether helper uses source compiler/native dispatch

Either runtime
    FULL
    source-authored generic/ADT behavior

curated prelude
    FULL
```

Testing classification:
- focused exact tests at C4.

---

## Task 15 — Preserve explicit full-runtime controls and audit remaining constructors

Purpose:

Prevent future contributors from interpreting `VM::new()` as a test smell in domains where full Universe is the subject.

Risk:
- Semantic: LOW
- Implementation fanout: test documentation/audit

Owned files and symbols:
- `phalcom-core/tests/README.md`
- state file
- optional clarifying comments only where ambiguity is genuine

Inspect before editing:
- `VM::new()` search output

Dependencies:
- Task 14

Source of truth:
- test ownership architecture

Implementation boundary:

Changes:
- classify remaining full uses;
- optionally replace with `universe_vm()` in shared-support consumers where explicit naming improves intent;
- retain direct `VM::new()` where production-constructor behavior itself is under test.

Must not:
- add noisy comments to every test;
- migrate full source behavior to native tier.

Testing classification:
- C4 focused full control.

---

# 10. Checkpoint C5 — Performance evidence and delivery gate

Tasks:
- Task 16 — Record before/after bootstrap evidence and architectural counts.
- Task 17 — Run broad delivery gates and close state.

Why this is a checkpoint:

C5 does not introduce new semantics. It proves that the architectural goals achieved in C1–C4 translate into the intended test-time behavior and that no broad compatibility regression remains.

Entry conditions:
- C4 COMPLETE
- no INCIDENT

Working set:

Primary:
- no new production files expected
- implementation state
- optional benchmark/timing logs

Secondary:
- only files implicated by failures

Out of scope:
- new optimizations discovered by measurement
- artifact/image work
- semantic solver tuning

Semantic contract:
- no new contract; delivery verification of prior checkpoints.

Semantic risks:
- misclassifying a broad baseline failure as this program;
- treating timing variance as correctness failure.

Hostile cases:
- second full VM still semantically analyzes Universe due an overlooked path;
- full core target remains effectively serialized by repeated cold compiler work;
- lower-tier tests unexpectedly touch full source bootstrap.

Required evidence:
- focused elapsed comparison;
- broad `phalcom-core` core target;
- final workspace format/check/test/clippy;
- final negative searches.

Do not run yet:
- nothing after final gate except diagnosis-driven reruns.

Escalate immediately if:
- broad failure traces into a removed/changed ownership seam;
- a workspace failure predates the implementation baseline and is reproducible there: classify BASELINE rather than expanding patch scope.

Checkpoint completion:
- [ ] performance evidence recorded
- [ ] broad gates pass or baseline blockers explicitly dispositioned
- [ ] negative gates clean
- [ ] deferred evidence audit empty
- [ ] state contains no incident

---

## Task 16 — Record non-flaky performance evidence

Purpose:

Demonstrate the practical reduction without encoding machine-specific wall-clock thresholds into correctness tests.

Risk:
- Semantic: LOW
- Implementation fanout: evidence only

Owned files:
- state/evidence record

Dependencies:
- C4

Source of truth:
- commands and measured output

Implementation boundary:

Measurements:

1. Fresh process full VM:
   ```bash
   /usr/bin/time -p env RUSTFLAGS='' RUSTC_WRAPPER='' \
     cargo +nightly-2026-07-10 test -p phalcom-core --test core \
     curated_prelude_exposes_public_names_and_hides_internal_classes \
     -- --nocapture
   ```

2. Multi-test/full-module evidence:
   ```bash
   /usr/bin/time -p env RUSTFLAGS='' RUSTC_WRAPPER='' \
     cargo +nightly-2026-07-10 test -p phalcom-core --test core \
     either:: -- --nocapture
   ```

Interpretation:
- first process may still pay one cold canonical semantic build;
- subsequent VMs inside the same process should no longer each pay that build;
- elapsed numbers are evidence, not hard pass/fail thresholds.

If optional timing instrumentation was added, record phase counts/times but do not make it mandatory to ship.

Testing classification:
- non-gating performance evidence.

---

## Task 17 — Final broad compatibility and deletion gates

Purpose:

Close the program after semantic checkpoints already established correctness locally.

Risk:
- Semantic: LOW
- Implementation fanout: workspace verification

Dependencies:
- all prior checkpoints COMPLETE

Required final broad gates:

```bash
cargo +nightly-2026-07-10 fmt --all -- --check
```

Proves:
- repository formatting consistency.

Does not prove:
- bootstrap semantics.

```bash
cargo +nightly-2026-07-10 check --workspace --all-targets
```

Proves:
- Rust API/caller migration is complete across workspace targets.

```bash
cargo +nightly-2026-07-10 test -p phalcom-core --test core
```

Proves:
- broad core integration behavior with the new shared/full bootstrap architecture.

```bash
cargo +nightly-2026-07-10 test --workspace --all-targets
```

Proves:
- broad workspace compatibility and catches consumers outside the focused core target.

```bash
cargo +nightly-2026-07-10 clippy --workspace --all-targets -- -D warnings
```

Proves:
- lint-clean delivery.

If the local `sccache`/Cargo environment itself exhibits the separately diagnosed build hang, rerun verification with:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' <command>
```

Record that as a build-environment workaround, not as runtime semantic evidence.

---

# 11. Final negative/deletion gates

Run after C3 and again at final delivery.

## 11.1 VM must not own canonical semantic derivation

```bash
rg -n "analyze_workspace|ModuleLinker|universe_lowerings|verify_native_contracts" \
  phalcom-core/src/vm/bootstrap.rs
```

Expected:
- zero executing production hits.

## 11.2 Fake semantic roots removed

```bash
rg -n "semantic_roots.*Value::nil|ordering_class:\s*ClassId::default" \
  phalcom-core/src
```

Expected:
- zero production hits.

## 11.3 No second builtin parsed/interface cache

```bash
rg -n "BUILTIN_PARSED_CACHE|BUILTIN_INTERFACE_CACHE" phalcom-core/src
```

Expected:
- zero hits.

The intended authoritative occurrences remain only in `phalcom-modules/src/builtin_interface.rs`.

## 11.4 No shared mutable VM

```bash
rg -n "OnceLock<.*VM|Lazy<.*VM|Mutex<.*VM|RwLock<.*VM" \
  phalcom-core phalcom-test-support
```

Expected:
- zero new fixture/bootstrap singleton VM state.
- pre-existing unrelated occurrences, if any, must be individually justified.

## 11.5 No source-incomplete fallback restored

Search the final diff and relevant bootstrap/compiler files for newly introduced name-based `Result`/variant special cases.

Example bounded searches:

```bash
rg -n 'owner\.name\s*==\s*"Result"|name\s*==\s*"Result"' \
  phalcom-core/src/vm phalcom-core/src/compiler
```

Expected:
- no new canonical-identity workaround introduced by this program.
- any pre-existing presentation-only match must be documented rather than automatically deleted.

---

# 12. Failure protocol for this implementation program

If required evidence fails, mark the checkpoint `INCIDENT` and stop forward expansion.

Record:

1. exact command;
2. exact failing test/check;
3. relevant error/output;
4. direct path from test to failure;
5. one passing comparator;
6. classification:
   - PRODUCT
   - FIXTURE
   - DEPENDENCY/PUBLICATION
   - BACKEND/HARNESS
   - BASELINE
   - PLAN DRIFT
7. allowed repair boundary;
8. rejected broad repairs.

## 12.1 Common predicted incidents

### Incident A — `OnceLock` rejects a nested type as non-`Sync`

Classification:
- PRODUCT/PLAN DRIFT depending exact field.

Allowed repair:
- identify the exact field;
- reduce the shared product to immutable/shareable projections.

Forbidden:
- `unsafe impl Send` / `unsafe impl Sync`.

### Incident B — `ProgramCompiler::compile_analyzed` rejects current Universe diagnostics

Classification:
- DEPENDENCY/PUBLICATION or PLAN DRIFT.

Allowed repair:
- preserve current bootstrap diagnostic acceptance;
- extract the reusable `CompiledModule` projection logic.

Forbidden:
- suppress diagnostics globally;
- weaken semantic errors to Dynamic;
- change unrelated Universe source.

### Incident C — Full runtime loses canonical Result variant identity

Classification:
- PRODUCT.

Allowed repair:
- shared linked/source/lowering product.

Forbidden:
- name-based Result special case;
- fallback VariantId fabrication;
- source-incomplete analysis.

### Incident D — Native-tier test fails due missing semantic root

Classification:
- FIXTURE unless the primitive's contract is genuinely native-only.

Allowed repair:
- keep that test full if its semantic contract requires source roots;
- choose a better native-tier candidate.

Forbidden:
- create fake semantic root;
- auto-upgrade native VM to full.

### Incident E — broad workspace test is a known pre-existing failure

Classification:
- BASELINE after reproducing on implementation baseline.

Allowed repair:
- record blocker separately.

Forbidden:
- expand this program into unrelated subsystem repair.

---

# 13. Repository drift protocol

Before each checkpoint:

1. verify primary files still exist;
2. verify symbols still own the responsibilities described;
3. inspect the effects of previous checkpoints;
4. search for new direct consumers if an API changed;
5. adapt mechanical details only.

The agent may adapt:
- helper names;
- exact error enum shape;
- internal visibility;
- file placement within the same ownership layer.

The agent may **not** silently change:
- process-shared compiler product ownership;
- fresh per-VM mutable state;
- source-complete canonical semantics;
- explicit lower-tier capability;
- full `VM::new()` compatibility.

If current repository state contradicts those semantic decisions, stop and escalate with code evidence.

---

# 14. Working-state protocol

After every checkpoint update:

```md
## Established invariants

- I-01: ...
- I-02: ...

## Decisions

- D-01: ...

## Repository findings

- F-01: ...

## Evidence ledger

| Checkpoint | Command | Result | Proves |
|---|---|---|---|

## Negative gates

| Search | Result | Expected |
|---|---|---|

## Deferred gates

- `<command>` → C<N>/Final

## Active incident

None.
```

Also record:

- important changed files/symbols;
- any structural deviation from this plan;
- exact reason for deviation;
- next resume action.

Do not record hidden reasoning or implementation diaries.

---

# 15. Supervisor checkpoint reports

## C1 example

```text
Checkpoint C1 COMPLETE

Established:
    Canonical Universe link/semantic/lowering derivation is one immutable
    process-shared modules product.

Changed:
    modules/canonical_universe.rs
    modules/mod.rs
    compile.rs (if projection extraction required)

Evidence:
    singleton test — PASS
    canonical Result product identity — PASS
    cargo check -p phalcom-core --all-targets — PASS

Hostile:
    no name-based Result identity — PASS

Deferred:
    VM source cutover → C3

Unexpected findings:
    <none / exact item>

Next:
    C2 — VM bootstrap stages.
```

## C2 example

```text
Checkpoint C2 COMPLETE

Established:
    Kernel, native, and full VM construction have explicit capabilities.
    Source-derived semantic roots are absent below full bootstrap.

Evidence:
    stage tests — PASS
    native primitive focused tests — PASS
    full invariant control — PASS

Next:
    C3 — full VM consumes shared canonical compiler program.
```

---

# 16. Checkpoint evidence summary template

Complete during implementation.

| Checkpoint | Semantic contract | Evidence | Status |
|---|---|---|---|
| C0 | Baseline and ownership map pinned | Git state + structural searches | PENDING |
| C1 | One immutable canonical compiler product | singleton + Result identity + check | PENDING |
| C2 | Honest kernel/native/full VM tiers | stage + semantic-root + native/full controls | PENDING |
| C3 | Full VM has no per-VM semantic/linking derivation | range/prelude/Result/Either + negative gate | PENDING |
| C4 | Tests use minimum required tier | low-level groups + full control + audit | PENDING |
| C5 | Performance architecture and workspace compatibility | elapsed evidence + broad gates | PENDING |

No row may be changed to COMPLETE before all checkpoint-required evidence succeeds.

---

# 17. Recommended commit groups

Prefer coherent commits rather than one commit per task.

### C1

```text
refactor(core): introduce shared canonical universe compiler program
test(core): pin canonical universe compiler identity
```

### C2

```text
refactor(vm): split kernel and native bootstrap stages
refactor(vm): make source semantic roots explicit
```

If the two are tightly interdependent for compilation, one combined C2 commit is preferable.

### C3

```text
refactor(vm): consume shared canonical universe program
test(core): preserve full bootstrap canonical semantics
```

### C4

```text
test(core): add explicit VM bootstrap tiers
test(core): use minimum runtime tier for low-level tests
```

### C5

No code commit should be necessary unless broad verification reveals a checkpoint-owned repair.

---

# 18. Known scope exclusions

Do not silently add any of the following:

- `.pui`/Universe image format;
- generated Rust Universe artifact;
- user-project compiled artifacts;
- build-time Universe compilation;
- disk cache;
- serialized semantic metadata redesign;
- process-global `SemanticSnapshot` baseline;
- LSP Universe baseline changes;
- bytecode relocation/template format;
- VM heap snapshots;
- lazy source module initialization;
- bootstrap root reduction;
- source storage `Arc<String>` migration;
- Cargo thread/sccache tuning;
- semantic solver performance optimization;
- generic inference feature work;
- unrelated reflection redesign.

If one becomes necessary to satisfy a checkpoint, that is a PLAN DRIFT escalation, not an automatic scope expansion.

---

# 19. Deferred-evidence audit

Before final completion, every deferred command must satisfy one of:

1. executed successfully;
2. explicitly removed from scope with a concrete reason approved by supervisor;
3. recorded as a known release blocker with ownership outside this program.

There must be no unlabeled “we did not get to it” gate.

---

# 20. Release-complete criteria

The implementation program is complete only when:

- [ ] C0–C5 are all COMPLETE;
- [ ] `VM::new()` is still the full shipping constructor;
- [ ] `VM::new_kernel()` and `VM::new_native()` have explicit tested contracts;
- [ ] canonical Universe semantic/linking/lowering derivation is process-shared;
- [ ] `phalcom-core/src/vm/bootstrap.rs` contains no direct whole-Universe semantic analysis/linking implementation;
- [ ] source/native conformance verification is not repeated per VM;
- [ ] no process-global mutable VM/heap exists;
- [ ] no fake Nil/default semantic roots remain;
- [ ] canonical Result/variant runtime identity tests pass;
- [ ] range/prelude/Either focused regressions pass;
- [ ] selected low-level tests use lower runtime tiers;
- [ ] remaining `VM::new()` test uses are audited/justified;
- [ ] final format/check/core/workspace/clippy gates pass or any baseline blockers are explicitly dispositioned;
- [ ] all negative/deletion gates have expected results;
- [ ] no unresolved state-file INCIDENT remains;
- [ ] no deferred gate is forgotten;
- [ ] documentation/comments that still claim every VM constructor performs full source bootstrap are updated.

---

# 21. Final implementation guidance

The most important execution discipline is to keep two dimensions separate:

```text
compiler-derived canonical facts
    can be shared

runtime identities/state
    must remain fresh
```

and:

```text
minimum runtime capability
    chosen explicitly by consumer/test

full shipping semantics
    remain the default VM::new()
```

If implementation pressure suggests solving the performance problem by sharing a VM, suppressing semantic analysis, restoring an identity fallback, caching store-local semantic IDs, or weakening full bootstrap semantics, stop. Those approaches optimize the symptom by violating the architecture.

The correct short-term result is not a temporary cache hack. It is the first clean separation between:

```text
canonical compilation
runtime materialization
runtime initialization
```

that the later compiled-project artifact architecture can build upon.

# 22. Hardening addendum — 2026-09-03

Implementation baseline: `47abba0e5b44d091768748420fd21dd91ae43742`.

Completed hardening work:

- canonical product fields are private, with crate-internal accessors and
  construction-time coverage validation for root-reachable/eager-bootstrap
  source, compiled, and linked modules;
- canonical semantic acceptance uses the checked-in exact 146-record error
  baseline at `phalcom-core/core/universe/semantic-diagnostics-baseline.txt`;
- canonical linked reads remain symbolic in the shared product and are
  materialized per VM before each source module compiles; `CompileBindings` comes
  from the same linked module used by formal analysis;
- process-global compiler work is counter-verified as one build/link/analyze/
  projection, while AST compilation and initializer execution remain per VM;
- absent example tests for non-existent `examples/core_new.ph`,
  `examples/person2.ph`, `examples/person.ph`, and `examples/calculator.ph` were
  removed; checked-in golden fixtures pass.

Evidence completed:

- formatting, `phalcom-core` checks, and workspace all-target compilation;
- canonical semantic/product/bootstrap/link-read focused tests;
- module compilation/runtime/Universe, native surface, Either, and monad gates;
- scoped diff checks and `graphify update .`.

C5/release status remains incomplete. `modules_linking` is 13/14 because the
pre-existing `mat_06_module_import_binding_resolves_to_module` assertion expects
a dotted runtime display name; clean HEAD reproduces that failure and the
callable-surface object-model failure. The broad language runner also exposed pre-existing
`adt_lower_10`, boolean-prelude, and bytes-negative fixture failures and was
stopped after classification because its 151 fixture processes exceeded the
useful verification window. Workspace clippy is blocked by six existing
`phalcom-semantic` lint errors. No production repair was added for these
unrelated baseline failures.

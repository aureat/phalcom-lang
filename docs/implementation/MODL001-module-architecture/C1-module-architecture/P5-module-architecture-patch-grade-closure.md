# Phalcom LSP Module Architecture — Patch-Grade Implementation Plan

**Plan type:** Repository-grounded, checkpoint-driven, patch-grade implementation program  
**Target architecture:** Authoritative module ownership, topology, linking, semantic projection, source provenance, incrementality, and LSP consumption  
**Repository:** `aureat/phalcom-lang`  
**Prepared against remote branch:** `main`  
**Exact remote HEAD:** `e932aac4e21a5b346e719ede5a24f94e7b924ab3`  
**HEAD commit:** `feat(semantic): complete SC-4.8 typing integration`  
**HEAD commit date:** 2026-09-04  
**Repository-state limitation:** The available repository connector exposes the remote repository state. Local working-tree status and local uncommitted changes are **not visible** and therefore are **unknown**. Before implementation, the executing agent MUST compare its local `HEAD` and working tree against the revision above and perform the drift protocol in §8.

---

# 1. Implementation Program

This program implements the authoritative Phalcom module architecture for compiler and LSP use.

The program is not an LSP-only repair. The repository currently has one compiler-owned `SemanticWorkspaceSession`, but module ownership, module-workspace rebuilding, semantic target projection, source provenance, and LSP presentation are not yet consistently aligned.

The target pipeline is:

```text
source candidates / overlays
        ↓
authoritative EntryOwnership
        ↓
canonical ModuleId + module topology
        ↓
canonical UnlinkedModuleInterface products
        ↓
canonical import resolution products
        ↓
canonical linked module products
        ↓
semantic target projection
        ↓
SourceSemanticIndex
        ↓
SemanticSnapshot
        ↓
EditorSemanticQuery
        ↓
LSP protocol adaptation
```

The implementation MUST preserve one source of truth at each layer. No checkpoint may repair a mismatch by introducing an LSP-specific resolver, a second linker, a name-based semantic identity fallback, or a cache owned by the wrong layer.

The core propagation rule is:

```text
input changes
    ↓
recompute immediate canonical product
    ↓
compare semantic product fingerprint
    ↓
propagate only if semantic meaning changed
```

---

# 2. Repository-Grounded Architecture Baseline

## 2.1 Relevant crates

The current workspace contains the following relevant ownership layers:

| Crate | Current responsibility relevant to this program |
|---|---|
| `phalcom-ast` | Import/export/expose/enum syntax and source ranges. No grammar change is planned. |
| `phalcom-modules` | Project/package identity, `ModuleId`, source providers, interface extraction, path resolution, exposure, linking, module graphs, module workspace lifecycle. |
| `phalcom-semantic` | Incremental semantic DB, declaration/type identity, linked type resolution, source semantic indexes, immutable snapshots, protocol-neutral editor queries. |
| `phalcom-lsp` | Workspace candidate discovery, source-update orchestration, protocol adaptation, diagnostics publication, navigation/completion/hover. |
| `phalcom-core` | Strict executable entry selection and closed-program compilation/runtime-facing program construction. |

### Out-of-scope ownership layers

The following are not expected to require semantic changes for this program:

- bytecode instruction formats;
- VM value representation;
- runtime ADT representation;
- generic constraint solving;
- exhaustiveness;
- parser grammar;
- native-surface metadata.

If implementation unexpectedly crosses into these subsystems, follow the escalation protocol instead of expanding scope automatically.

---

## 2.2 Primary existing identities and products

### `EntryOwnership`

Defined in:

```text
phalcom-modules/src/source.rs
```

Current variants:

```rust
pub enum EntryOwnership {
    ProjectOwned { project: ResolvedProjectId },
    StandalonePackageOwned { package_root: PathBuf },
    StandaloneModule { file: PathBuf },
    Inline { synthetic: SyntheticProjectId },
}
```

This type already expresses the target ownership categories. The defect is that production workspace ownership does not currently make it authoritative.

### `ModuleId`

Defined in `phalcom-modules/src/identity.rs`.

```text
ProjectIdentity
    Universe
    Resolved(ResolvedProjectId)
    Synthetic(SyntheticProjectId)

ModuleId
    project identity
    +
    logical ModulePath
```

`ModuleId` remains the canonical module identity. Do not create another LSP/document/module ID.

### `UnlinkedModuleInterface`

Defined in `phalcom-modules/src/interface.rs`.

It already owns:

- module-scope declarations;
- imports;
- exports;
- re-exports;
- `exposed_children`;
- module kind;
- metadata.

`InterfaceBuilder::build` already has a three-pass extraction model and unified module namespace checks.

### `SymbolId`

Defined in `phalcom-modules/src/linker.rs`.

```rust
pub struct SymbolId {
    pub module: ModuleId,
    pub name: Box<str>,
}
```

This is already the canonical linked identity for a module-owned global binding.

It MUST be reused for exported top-level `let`/`const` semantic targets rather than inventing another global ID.

### `LinkedExportTarget`

Defined in `phalcom-modules/src/interface.rs`.

```text
LinkedExportTarget::Binding(SymbolId)
LinkedExportTarget::Module(ModuleId)
```

This is the source of truth for cross-module symbol visibility.

### `SemanticTargetId`

Defined in `phalcom-semantic/src/identity.rs`.

Current variants include:

```text
Binding(SourceSiteId)
Declaration(DeclarationId)
Callable(CallableId)
Field(FieldId)
Module(ModuleId)
Variant(VariantId)
VariantFamily(VariantFamilyId)
VariantField(VariantFieldId)
```

It currently lacks a cross-module module-global value target. This program adds `ModuleBinding(SymbolId)` or an equivalent exact representation.

### `SemanticDb`

Defined in `phalcom-semantic/src/db`.

The semantic DB already owns:

- query input fingerprints;
- product fingerprints;
- dependency fingerprints;
- current-revision validation;
- reverse invalidation;
- lazy downstream reuse;
- cancellation/budgeting;
- `Arc`-backed products;
- last-known-good products.

This architecture MUST be preserved.

### `SourceSemanticIndex` / `EditorSemanticQuery`

Owned by `phalcom-semantic`.

These are the correct protocol-neutral editor-semantic authorities. LSP special cases must be retired once source provenance is complete.

---

# 3. Current Defects and Repository Evidence

The implementation agent should treat the following as established repository facts at the prepared revision.

| Finding | Current location / symbol | Consequence |
|---|---|---|
| Arbitrary parent folders become pseudo package/project roots | `phalcom-modules/src/session.rs` — `WorkspaceModuleSession::module_for_location` | Package-less siblings can acquire resolved package-like `ModuleId`s. |
| A second sibling resolver exists | `phalcom-modules/src/session.rs` — `resolve_standalone_import` | Workspace behavior diverges from canonical `ModuleResolver`. |
| `EntryOwnership` exists but is not the production authority | `phalcom-modules/src/source.rs` vs `session.rs` | Ownership rules are encoded implicitly in session control flow. |
| Persistent project loading enforces root `package.ph` | `phalcom-modules/src/project.rs` — `load_root_with_provider` | Correct package contract already exists for projects. |
| `load_synthetic_root` does not validate `package.ph` | `phalcom-modules/src/project.rs` | Compatibility helper can create package-like roots without package markers. |
| Strict standalone module execution rejects sibling imports | `phalcom-core/src/modules/compile.rs` — `analyze_standalone_module` | Compiler/runtime and LSP workspace disagree today. |
| `EntrySelection::Package` documents `package.ph` + `main.ph`, but only checks `main.ph` | `phalcom-core/src/modules/compile.rs` — `analyze_entry_selection` | Package identity is not enforced at this entry boundary. |
| Workspace rebuild rebuilds interfaces broadly | `phalcom-modules/src/session.rs` — `rebuild` | Body-only edits incur module work proportional to workspace/reachable graph. |
| Workspace rebuild re-resolves imports broadly | same | Existing resolver/provider caches provide little benefit. |
| Workspace rebuild links from every parsed source entry | same | Overlapping reachable components may be repeatedly linked. |
| `apply_batch` clones session maps and creates a new `FilesystemSourceProvider` | `phalcom-modules/src/session.rs` | O(workspace) staging and base-cache loss on edit. |
| `FilesystemSourceProvider::clear_cache` clears resolution, text, and reverse identity together | `phalcom-modules/src/source.rs` | Body edits can cause topology cache eviction. |
| `ModuleResolver` already records package interfaces consulted during path exposure checking | `phalcom-modules/src/resolver.rs` — `ImportResolutionTrace` | Existing abstraction should seed resolution-dependency caching. |
| Semantic DB already fingerprints unlinked/linked interfaces | `phalcom-semantic/src/db/fingerprint.rs` | Do not create competing interface fingerprint algorithms. Move/reuse ownership appropriately. |
| Module session builds an interface, then semantic DB can build it again | `phalcom-modules/src/session.rs`; `phalcom-semantic/src/session.rs` → `query_unlinked_interface` | Duplicate canonical product construction. |
| Qualified semantic lookup bypasses exports | `phalcom-semantic/src/resolver.rs` — `LinkedTypeResolver::resolve_type_name` | `module.PrivateType` can resolve by declaration existence alone. |
| Every linked binding is projected as `DeclarationId` for source indexing | `phalcom-semantic/src/session.rs` — source-index context target construction | Exported top-level values masquerade as nominal declarations. |
| Source index skips top-level enum declarations | `phalcom-semantic/src/source_index/builder.rs` — initial registration and `visit_statements` | Imported enum has no source declaration target. |
| Source index skips `Statement::Export` | same and `occurrence.rs` | Export syntax has no canonical source occurrence. |
| Selective import local site is assigned the upstream target directly | `SourceScopeBuilder::visit_imports` | Alias-local identity and remote-origin identity are collapsed. |
| Dependency preamble syntax is not fully represented as semantic occurrences | `source_index/occurrence.rs` | Import path / remote item / re-export / expose navigation is incomplete. |
| `EditorSemanticQuery` exposes definition sites but no module-source definition-location abstraction | `phalcom-semantic/src/editor.rs` | LSP still needs special handling for module targets. |
| LSP has AST/module-query fallback for import definition | `phalcom-lsp/src/backend.rs` — `compiler_import_definition_location` | LSP reconstructs semantic meaning outside source index. |
| Current positive LSP test blesses package-less sibling import | `phalcom-lsp/tests/module_navigation.rs` — `goto_definition_on_relative_import_path_and_selective_export` | Test suite currently locks in the wrong package semantics. |
| LSP worker treats any module publication `Err` as cancellation | `phalcom-lsp/src/analysis_service.rs` | Source-authored module errors can suppress current snapshot publication. |
| `SnapshotStatus::Partial` exists but is not the module-error publication mechanism | `phalcom-semantic/src/snapshot.rs` | Infrastructure exists for correct partial publication. |
| `ModuleQueryFacade` scans maps for child/reverse queries | `phalcom-modules/src/query.rs` | Topology indexes can improve repeated editor queries without adding LSP caches. |

---

# 4. Architectural Source-of-Truth Matrix

| Semantic fact | Authoritative owner | Derived consumers | Forbidden competing authority |
|---|---|---|---|
| Source ownership | `EntryOwnership` classification in `phalcom-modules` | workspace session, compiler entry selection, LSP ingestion | workspace-folder parent heuristic |
| Module identity | `ModuleId` | linker, semantic identities, snapshots, LSP | URI-derived module naming |
| Package identity | physical `package.ph` + ownership/topology classifier | resolver, compiler, LSP | directory existence or `main.ph` |
| Child path accessibility | topology + package `exposed_children` | resolver, completion | LSP filesystem scan |
| Module public symbol API | `LinkedModuleInterface.exports` | semantic resolver, source index, completion | declaration existence |
| Module-global binding identity | `SymbolId` | semantic `ModuleBinding`, references/navigation | fabricated `DeclarationId` |
| Nominal declaration identity | `DeclarationId` | type system, source index, editor | name-only lookup |
| Import path result | canonical module resolution product | linker, source index, module query facade | LSP import resolver |
| Interface fingerprint | module-owned semantic interface fingerprint | module session, semantic DB | duplicate hash algorithms |
| Linked-interface fingerprint | module-owned linked product fingerprint | module session, semantic DB | LSP cache |
| Semantic query cache | `SemanticDb` | snapshot publication | separate LSP/worker semantic cache |
| Source provenance | `SourceSemanticIndex` | `EditorSemanticQuery`, LSP | AST request-time navigation |
| Module source provenance | module query products / semantic snapshot | editor definition locations | filesystem lookup from LSP request |

---

# 5. Non-Negotiable Semantic Invariants

Use these as implementation-state invariant IDs.

### MOD-OWN-1 — One ownership

One source has exactly one canonical `EntryOwnership` for one committed module-workspace generation.

### MOD-PKG-1 — Explicit packages

A directory has package semantics only when the required `package.ph` marker exists.

### MOD-ID-1 — Canonical physical identity

One canonical physical source maps to at most one `ModuleId` per committed generation.

### MOD-RES-1 — One resolver

Only `phalcom-modules` interprets `ImportPath`.

### MOD-TOP-1 — Topology fingerprint meaning

`TopologyFingerprint` changes when namespace path-resolution semantics change and remains stable for body-only edits.

### MOD-IFACE-1 — Product-stability propagation

Body-only source edits that preserve `InterfaceFingerprint` do not trigger import re-resolution or relinking.

### MOD-EXP-1 — Linked export authority

Cross-module symbol lookup succeeds only through `LinkedModuleInterface.exports`.

### MOD-XPS-1 — Exposure is path visibility

`expose` controls public child traversal; it does not import/export a runtime binding.

### MOD-GLOBAL-1 — Global identity distinction

Top-level values use `SymbolId`-backed semantic identity and do not masquerade as `DeclarationId`.

### MOD-SRC-1 — Complete navigable provenance

Every navigable module/declaration/global/import/export/re-export/expose/enum identity has source provenance.

### MOD-WS-1 — User errors are publishable

Source-authored module/link errors yield current diagnostics and a partial current workspace publication; they do not masquerade as cancellation.

### MOD-LSP-1 — Read-only requests

LSP request handlers perform no filesystem I/O, parsing, interface construction, module resolution, linking, or semantic checking.

### MOD-CACHE-1 — Cache ownership

A cache lives in the layer that owns the cached canonical product.

### MOD-PARITY-1 — Compiler/LSP parity

Strict compiler and workspace/LSP use the same ownership, resolution, exposure, export, and linking semantics.

---

# 6. Tempting Wrong Fixes — Explicitly Forbidden

Do not implement any of the following shortcuts.

1. **Do not repair the current navigation bug in `phalcom-lsp` alone.**  
   Adding another fallback around `compiler_import_definition_location` would preserve split authority.

2. **Do not make every directory containing `.ph` files a package.**  
   The current package-less navigation test is wrong and must change.

3. **Do not treat `main.ph` as package identity.**  
   It is an executable entry convention only.

4. **Do not retain `resolve_standalone_import` as a hidden compatibility path.**  
   A standalone module does not get sibling resolution.

5. **Do not fix qualified private lookup by checking `known_declarations` more carefully.**  
   The correct authority is the target module's linked export table.

6. **Do not map every `SymbolId` to a `DeclarationId`.**  
   A user may export `const version`, `let state`, or any other module-global value.

7. **Do not give import aliases the upstream declaration as their only source identity.**  
   Alias rename/reference semantics require a local binding plus upstream origin.

8. **Do not add a navigation-result cache in LSP.**  
   `SemanticSnapshot` is already the immutable request-time cache.

9. **Do not solve incremental cost by parallelizing a full rebuild.**  
   First eliminate unnecessary work.

10. **Do not duplicate `unlinked_interface_product_fingerprint` and `linked_interface_product_fingerprint` in `phalcom-modules`.**  
    Move/generalize the canonical hashing so both module workspace and semantic DB use one algorithm.

11. **Do not use `ResolvedProject::revision_fingerprint` as `TopologyFingerprint`.**  
    The current project revision fingerprint recursively hashes source bytes and therefore changes on ordinary body edits.

12. **Do not restore stale snapshots as current truth after a source-authored module error.**  
    Last-known-good is an explicit fallback mechanism for infrastructure/cancellation, not a substitute for partial current publication.

13. **Do not change parser syntax.**  
    The AST already contains the required import/export/expose/enum source ranges.

14. **Do not change VM/runtime identities to solve editor/module identity problems.**

---

# 7. Checkpoint Map

| Checkpoint | Tasks | Semantic boundary | Required evidence | Deferred evidence |
|---|---:|---|---|---|
| **C0 — Authoritative ownership and package identity** | 1–4 | Every source is classified as Project / standalone package / standalone module / inline before module resolution; `package.ph` is enforced consistently. | Focused `phalcom-modules` ownership/session tests; focused `phalcom-core` entry tests; negative search for sibling fallback. | LSP navigation integration → C6; workspace-wide suite → Final Gate. |
| **C1 — Canonical topology and module-owned fingerprints** | 5–8 | Topology, interface, linked-interface, and resolution fingerprints become canonical module products; existing semantic DB reuses the same interface fingerprint algorithms. | Fingerprint unit tests; topology hostile tests; semantic fingerprint parity tests. | Transaction/performance evidence → C2. |
| **C2 — Incremental persistent module workspace** | 9–13 | Workspace updates stop cloning/rebuilding/resolving/linking broadly; product stability halts propagation; filesystem cache survives transactions. | Work-count tests in `phalcom-modules`; focused semantic cold/incremental equivalence; cache lifecycle tests. | Error-tolerant publication → C3; LSP end-to-end latency surface → C6/C7. |
| **C3 — Tolerant module diagnostics and current partial publication** | 14–18 | One resolver/linker algorithm supports strict compilation and tolerant workspace publication; user module errors become diagnostics, not cancellation. | Module/link diagnostics tests; semantic partial-snapshot tests; LSP worker publication test. | Full compiler/runtime strictness → C7/Final Gate. |
| **C4 — Canonical cross-module semantic identity** | 19–22 | Qualified type lookup obeys linked exports; nominal declarations and module globals have correct distinct semantic targets. | Semantic hostile visibility tests; global-value target test; compiler/semantic identity consistency. | Full source navigation → C5/C6. |
| **C5 — Complete source provenance and import-origin semantics** | 23–27 | Enums, variants, globals, import paths/items/aliases, exports, re-exports, and expose paths are all represented in compiler-owned source indexes. | Source-index tests; alias-local vs upstream-origin hostile tests; definition/reference consistency tests. | LSP protocol adapter behavior → C6. |
| **C6 — Editor query cutover and LSP fallback retirement** | 28–31 | `EditorSemanticQuery` is sufficient for module/declaration/global navigation; LSP special import resolver is deleted; corrected package fixtures drive LSP behavior. | LSP module-navigation tests; semantic-boundary negative searches; package-less negative fixture. | Topology live-edit and scale/perf gates → C7. |
| **C7 — Topology lifecycle, metrics, parity, and delivery closure** | 32–35 | `package.ph`, `project.toml`, source add/remove, export/expose changes invalidate the right products; metrics prove bounded recomputation; compiler/LSP parity is locked. | Lifecycle tests; work-count metrics; strict-vs-workspace parity tests; final negative gates. | Workspace-wide delivery commands → Final Delivery Gate. |

---

# 8. Repository Drift Protocol

Before **each checkpoint**, the implementing agent MUST do a bounded drift check:

1. Verify local repository:
   ```bash
   git rev-parse --abbrev-ref HEAD
   git rev-parse HEAD
   git status --short
   ```
2. Compare local `HEAD` with:
   ```text
   e932aac4e21a5b346e719ede5a24f94e7b924ab3
   ```
3. Inspect only the checkpoint's Primary files.
4. Confirm listed symbols still own the described responsibility.
5. Search for new callers only where the checkpoint changes a public API.
6. If local changes overlap the checkpoint, preserve them intentionally and record the conflict in the state file.

Mechanics may be adapted to repository drift.

The semantic design may **not** be silently changed.

If repository drift contradicts a semantic invariant in this plan, mark the checkpoint `INCIDENT` and escalate with exact evidence.

---

# 9. Working-State File Protocol

Create or update one concise implementation state file:

```text
docs/work/modules/lsp-module-architecture-implementation-state.md
```

If the repository's current documentation taxonomy has moved, use the nearest existing `docs/work/...` implementation-state location, but keep one file for this program.

After every checkpoint record:

```md
# Phalcom LSP Module Architecture — Implementation State

Prepared plan revision:
- remote baseline: e932aac4e21a5b346e719ede5a24f94e7b924ab3
- local implementation HEAD: <sha>

## Established invariants

- MOD-OWN-1: ...
- ...

## Decisions

- D-01: Reused `ResolverGeneration` as the coarse topology epoch; `TopologyFingerprint` is semantic.
- ...

## Evidence ledger

| Checkpoint | Command | Result | Proves |
|---|---|---|---|
| C0 | `cargo test ...` | PASS | ... |

## Negative/deletion gates

| Checkpoint | Search | Expected | Observed |
|---|---|---|---|

## Deferred gates

- `<command>` → C6
- `<command>` → Final Gate

## Active incident

None.

## Next resume action

Begin C<N> Task <M>.
```

Do not record chain-of-thought. Record facts, decisions, code anchors, evidence, and incidents only.

---

# 10. Checkpoint C0 — Authoritative Ownership and Package Identity

Tasks:
- Task 1 — Introduce one canonical ownership classifier.
- Task 2 — Route `WorkspaceModuleSession` through `EntryOwnership` and delete pseudo-project sibling fallback.
- Task 3 — Add validated standalone-package project loading and enforce package entry semantics in `phalcom-core`.
- Task 4 — Establish ownership/package hostile fixtures and migrate current wrong assumptions.

## Why this is a checkpoint

Tasks 1–4 collectively change the meaning of a filesystem source.

Testing after only Task 1 would not prove anything if `WorkspaceModuleSession` still creates synthetic parent-root projects. Testing only the workspace would still leave strict `phalcom-core` package selection inconsistent. The semantic boundary is established only when both compiler and workspace consume the same ownership classification.

## Entry conditions

- Remote baseline or drift-reviewed equivalent.
- Existing `EntryOwnership` variants remain available.
- Existing `ModuleId` / `ProjectIdentity` identity model remains authoritative.
- Existing `FilesystemSourceProvider` package-chain rules remain intact.

## Working set

### Primary

- `phalcom-modules/src/source.rs`
  - `EntryOwnership`
  - `resolve_source_path`
  - `FilesystemSourceProvider`
- `phalcom-modules/src/project.rs`
  - `ProjectUniverse`
  - `discover_owning_project`
  - `load_synthetic_root`
- `phalcom-modules/src/session.rs`
  - `WorkspaceModuleSession`
  - `module_for_location`
  - `resolve_standalone_import`
- `phalcom-core/src/modules/compile.rs`
  - `EntrySelection`
  - `ProgramAnalyzer::analyze_entry_selection`
  - `analyze_standalone_module`
- `phalcom-modules/tests/*` ownership/package tests
- `phalcom-core/tests/core/modules/*`

### Secondary — inspect only if evidence requires it

- `phalcom-modules/src/resolver.rs`
- `phalcom-modules/src/error.rs`

### Out of scope

- `phalcom-semantic`
- `phalcom-lsp`
- parser
- runtime/VM

## Semantic contract established by this checkpoint

- `package.ph` is the only filesystem package marker.
- A plain folder of `.ph` files does not create package/sibling import semantics.
- A directly opened source inside a valid standalone package retains standalone-package ownership.
- Strict compiler and workspace classify the same physical source consistently.
- `main.ph` affects executability only; it does not create package identity.
- `resolve_standalone_import` is no longer a semantic path.

## Semantic risks

- Assigning a different `ModuleId` to an existing persistent-project source.
- Incorrectly choosing a nested package as the standalone root.
- Letting an intermediate directory without `package.ph` become a package segment.
- Accidentally breaking Universe imports from standalone modules.
- Conflating `SyntheticProjectId` used for inline source with standalone package project identity.

## Hostile cases

1. `scratch/main.ph` + `scratch/helper.ph`, no `package.ph`: `.helper` MUST fail.
2. Same files plus `scratch/package.ph`: `.helper` MUST resolve.
3. `scratch/main.ph` alone MUST remain an executable standalone module.
4. `scratch/main.ph` without `package.ph` MUST NOT be accepted as `EntrySelection::Package`.
5. A source inside a persistent project MUST remain `ProjectOwned` even if an ancestor also resembles a standalone package.
6. Intermediate directory without `package.ph` MUST NOT silently become a package node.
7. Universe absolute imports from standalone modules continue to work.

## Required evidence

1. Focused module ownership/session tests:
   ```bash
   cargo test -p phalcom-modules workspace_session
   ```
   If the repository test filter does not match because tests remain inline in `session.rs`, run:
   ```bash
   cargo test -p phalcom-modules session::
   ```
   Proves canonical workspace ownership behavior.

2. Focused package/module integration tests:
   ```bash
   cargo test -p phalcom-modules --test integration
   cargo test -p phalcom-modules --test package_semantic_contract
   ```
   Proves existing project/package resolver contracts still hold.

3. Strict compiler module tests:
   ```bash
   cargo test -p phalcom-core core::modules
   ```
   If test filtering differs locally, use the specific discovered module test binary.
   Proves strict standalone/package entry behavior is aligned.

4. Negative deletion gate:
   ```bash
   rg 'resolve_standalone_import|load_synthetic_root' phalcom-modules/src/session.rs phalcom-core/src/modules/compile.rs
   ```
   Expected after C0:
   - `resolve_standalone_import`: **zero production occurrences**.
   - `load_synthetic_root`: no arbitrary-parent workspace use; any intentional standalone-package compatibility call must be documented and validate `package.ph`.

## Do not run yet

```bash
cargo test -p phalcom-lsp
```

Deferred to C6 because source provenance and LSP fallback have not been migrated yet.

```bash
cargo test --workspace --all-targets
```

Deferred to Final Gate.

## Escalate immediately if

- canonical package ownership cannot be represented with existing `ProjectIdentity`;
- fixing ownership appears to require parser changes;
- Universe import behavior relies on the arbitrary parent-root mechanism;
- persistent-project `resolve_source_path` produces identities incompatible with `FilesystemSourceProvider::locate`.

---

### Task 1 — Introduce one canonical ownership classifier

**Purpose:** Make `EntryOwnership` an executable production contract rather than an unused descriptive enum.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **multi-file**

**Owned files and symbols:**

- `phalcom-modules/src/source.rs` — `EntryOwnership` — canonical ownership result.
- `phalcom-modules/src/project.rs` — `discover_owning_project` — persistent project boundary discovery.
- Prefer a new `phalcom-modules/src/ownership.rs` only if keeping classification inside `source.rs` would mix filesystem provider and ownership policy excessively.
- `phalcom-modules/src/lib.rs` — re-export only after the abstraction is established.

**Inspect before editing:**

- `EntryOwnership`
- `discover_owning_project`
- `resolve_source_path`
- `ProjectUniverse::load_root`
- `ProjectUniverse::load_synthetic_root`

**Do not inspect unless evidence forces expansion:**

- semantic resolver;
- LSP backend;
- VM.

**Dependencies:** Existing `EntryOwnership`, `ModuleId`, and canonical path naming rules.

**Source of truth:** `EntryOwnership`.

**Implementation boundary:**

Create one classifier that answers:

```text
SourceLocation / canonical path
    → EntryOwnership
```

with the order:

```text
persistent owning project?
    → ProjectOwned

otherwise valid standalone package ancestry?
    → StandalonePackageOwned

otherwise source file?
    → StandaloneModule
```

`Inline` remains explicit and is not inferred from filesystem sources.

**Changes:**

- Add a canonical ownership-classification API.
- Canonicalize filesystem paths at the classification boundary.
- Cache directory ownership only later in C1/C2; C0 establishes semantics first.
- Determine standalone package ownership by `package.ph` ancestry, not directory existence.
- Ensure intermediate non-package directories cannot create package traversal accidentally.

**Must not:**

- manufacture a resolved project from `source.parent()`;
- scan sibling `.ph` files to infer package status;
- allocate a new identity merely because an editor opened a file.

**Current implementation:**

`WorkspaceModuleSession::module_for_location` performs ownership logic inline and creates a synthetic resolved root from the parent directory when no project is found.

**Target implementation:**

```text
classify ownership
    ↓
map ownership to canonical ModuleId
    ↓
store mapping
```

**Edit operations:**

1. OPEN `phalcom-modules/src/source.rs`.
2. FIND `EntryOwnership`.
3. ADD documentation that the enum is authoritative, not advisory.
4. ADD or route to a classifier API.
5. OPEN `phalcom-modules/src/project.rs`.
6. REUSE `discover_owning_project`.
7. ADD the minimum standalone-package marker traversal helper if no existing helper can express it.
8. SEARCH:
   ```bash
   rg 'EntryOwnership|discover_owning_project|load_synthetic_root' phalcom-modules phalcom-core phalcom-semantic phalcom-lsp
   ```
9. Record all production ownership callers for Task 2/3 migration.

**Code instructions:**

STRUCTURAL — exact ownership helper naming should follow nearby module conventions:

```rust
pub fn classify_entry_ownership(
    source: &SourceLocation,
    /* project/universe context as required */
) -> Result<EntryOwnership, OwnershipError>
```

The classifier may be a method on a session-owned object if project-ID allocation is required. Do not force this exact signature if it would duplicate `ProjectUniverse` ownership.

**Testing classification:** No standalone behavior test. Validated at C0 after consumers are migrated.

**Optional compile checkpoint:**

```bash
cargo check -p phalcom-modules
```

Run only if a new public ownership type/helper changes API fanout.

**Checkpoint state update:**

Record:
- exact classifier symbol;
- standalone package root-selection rule;
- whether existing `load_synthetic_root` remains and why.

---

### Task 2 — Route `WorkspaceModuleSession` through `EntryOwnership`

**Purpose:** Remove the current arbitrary-directory pseudo-project behavior and make workspace identity use the canonical classifier.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **local/multi-file**

**Owned files and symbols:**

- `phalcom-modules/src/session.rs`
  - `WorkspaceModuleSession::module_for_location`
  - `resolve_standalone_import`
  - `standalone_projects`
  - source/module maps
- ownership helper from Task 1.

**Inspect before editing:**

- full `module_for_location`;
- `kind_for_source`;
- `insert_state`;
- `rebuild`;
- all `standalone_projects` usages.

**Do not inspect unless evidence forces expansion:**

- `phalcom-lsp`;
- semantic source index.

**Dependencies:** Task 1 classifier.

**Source of truth:** `EntryOwnership`.

**Changes:**

- Replace the fallback branch that treats `path.parent()` as a resolved project root.
- `ProjectOwned`:
  - reuse/load the project;
  - map source through canonical project source-path logic.
- `StandalonePackageOwned`:
  - load/reuse a validated standalone-package project context;
  - map package/module paths using package topology.
- `StandaloneModule`:
  - assign a `ProjectIdentity::Synthetic` module identity at root;
  - do not expose siblings.
- Remove `resolve_standalone_import`.
- Remove the `ModuleResolver` error fallback that invokes it.
- Rename/refactor `standalone_projects` if it currently mixes standalone modules with standalone packages.

**Must not:**

- preserve package-less sibling success through another helper;
- call `load_synthetic_root` for every unowned parent directory.

**Current implementation:**

`module_for_location` creates a resolved synthetic root for a normal parent directory; `rebuild` catches resolver errors for synthetic projects and tries `resolve_standalone_import`.

**Target implementation:**

```text
StandaloneModule
    ModuleId::synthetic(..., ModulePath::root())
    imports: Universe only

StandalonePackageOwned
    validated package project identity
    imports: canonical relative/package resolution
```

**Edit operations:**

1. OPEN `phalcom-modules/src/session.rs`.
2. FIND `fn module_for_location`.
3. REPLACE the unowned-parent synthetic-root branch with Task 1 ownership dispatch.
4. FIND the `Err(error) if module.project.as_synthetic().is_some()` branch in `rebuild`.
5. REMOVE fallback to `resolve_standalone_import`.
6. DELETE `fn resolve_standalone_import`.
7. UPDATE fields used to retain standalone identity.
8. SEARCH:
   ```bash
   rg 'resolve_standalone_import|standalone_projects|load_synthetic_root' phalcom-modules/src/session.rs
   ```
9. Ensure only intentional ownership infrastructure remains.

**Testing classification:** Validated at C0.

**Optional compile checkpoint:**

```bash
cargo check -p phalcom-modules
```

Recommended here because `WorkspaceModuleSession` is a public API and borrow/ownership changes can create broad Rust compile errors.

---

### Task 3 — Validate standalone packages and strict entry parity

**Purpose:** Give standalone packages a validated module context and make `phalcom-core` use the same package marker contract.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **cross-crate**

**Owned files and symbols:**

- `phalcom-modules/src/project.rs`
  - `ProjectUniverse`
  - `load_synthetic_root`
- `phalcom-core/src/modules/compile.rs`
  - `EntrySelection::Package`
  - `ProgramAnalyzer::analyze_entry_selection`
  - `EntrySelection::Module`

**Inspect before editing:**

- `load_root_with_provider`;
- `load_synthetic_root`;
- package branch of `analyze_entry_selection`;
- project branch;
- standalone-module branch.

**Source of truth:** `EntryOwnership` + validated package marker.

**Changes:**

- Add a semantically named standalone-package loader, or strengthen/refactor `load_synthetic_root` so package consumers cannot bypass marker validation.
- The preferred API shape is:

STRUCTURAL:

```rust
pub fn load_standalone_package(
    &mut self,
    package_root: impl AsRef<Path>,
    entry: Option<&str>,
) -> Result<ResolvedProjectId, ProjectError>
```

- It MUST verify `package.ph`.
- `EntrySelection::Package` MUST first validate package identity, then separately validate `main.ph` executability.
- `EntrySelection::Module(file)` MUST discover standalone package ownership before falling back to `analyze_standalone_module`.
- Preserve plain-file standalone execution.

**Must not:**

- declare `main.ph` to be a package marker;
- duplicate project/package resolution inside `phalcom-core`.

**Testing classification:** Validated at C0.

---

### Task 4 — Ownership/package regression fixtures

**Purpose:** Replace wrong test assumptions and establish hostile ownership cases at the owning layer.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **tests**

**Owned tests:**

Prefer extending:

- `phalcom-modules/src/session.rs` inline tests for session identity/mutations;
- `phalcom-modules/tests/integration.rs`;
- `phalcom-modules/tests/package_semantic_contract.rs`;
- `phalcom-core/tests/core/modules/universe.rs` or the nearest module-entry test module.

**Required regressions:**

Add/rename tests equivalent to:

```text
standalone_sibling_files_do_not_form_package
standalone_module_cannot_relative_import_sibling
standalone_package_supports_relative_children
direct_file_inside_standalone_package_uses_package_identity
intermediate_directory_without_package_ph_is_not_package
package_entry_requires_package_ph
main_ph_does_not_create_package_identity
```

**Hostile requirement:**

The positive and negative sibling fixtures must be identical except for the presence of `package.ph` where possible.

**Testing classification:** Focused regressions required; run at checkpoint boundary.

---

## C0 checkpoint completion

- [ ] Tasks 1–4 implemented.
- [ ] Ownership tests pass.
- [ ] Strict core package/module tests pass.
- [ ] `resolve_standalone_import` has zero production occurrences.
- [ ] Arbitrary parent-root `load_synthetic_root` behavior is gone.
- [ ] Implementation state records MOD-OWN-1, MOD-PKG-1, MOD-ID-1.
- [ ] No active incident.

### Suggested commit grouping

```text
C0.1 refactor(modules): make EntryOwnership authoritative
C0.2 fix(modules): remove package-less sibling resolution
C0.3 fix(core): enforce standalone package ownership and package.ph
C0.4 test(modules): lock package ownership boundaries
```

---

# 11. Checkpoint C1 — Canonical Topology and Module-Owned Fingerprints

Tasks:
- Task 5 — Move canonical interface fingerprint ownership into `phalcom-modules`.
- Task 6 — Introduce `TopologyFingerprint` and canonical `ModuleTopology`.
- Task 7 — Extend import-resolution traces into reusable resolution products.
- Task 8 — Project topology indexes through module query products.

## Why this is a checkpoint

Incremental workspace work cannot be made correct by caching raw resolver answers alone. The system first needs explicit semantic products whose stability can be compared. This checkpoint defines those products and moves fingerprint authority to the module layer that owns the corresponding meaning.

## Entry conditions

- C0 COMPLETE.
- Canonical ownership exists.
- `ModuleResolver` remains the only path interpreter.
- `InterfaceBuilder` remains the interface authority.

## Working set

### Primary

- `phalcom-modules/src/interface.rs`
- `phalcom-modules/src/resolver.rs`
- `phalcom-modules/src/query.rs`
- `phalcom-modules/src/stabilization.rs`
- `phalcom-modules/src/lib.rs`
- new `phalcom-modules/src/fingerprint.rs`
- new `phalcom-modules/src/topology.rs` if chosen
- `phalcom-semantic/src/db/fingerprint.rs`
- `phalcom-semantic/tests/semantic/incremental/fingerprints.rs`

### Secondary

- `phalcom-modules/src/project.rs`
- `phalcom-modules/src/source.rs`

### Out of scope

- workspace transaction rewrite;
- tolerant linking;
- LSP.

## Semantic contract established

- `TopologyFingerprint` has a precise semantic meaning.
- Existing `ResolverGeneration` is reused as the coarse monotonic resolver/topology epoch; do not create a redundant `TopologyEpoch` unless repository evidence forces it.
- Interface and linked-interface fingerprint algorithms are owned by `phalcom-modules`.
- Semantic DB uses those canonical hashes instead of owning duplicate module-contract hashing.
- Resolver products retain the topology/exposure dependencies that justify reuse.
- Module child/reverse-import queries can use canonical indexes rather than scanning unrelated maps.

## Semantic risks

- Fingerprinting source positions, causing formatting edits to propagate.
- Omitting export/expose/import semantics from fingerprints.
- Treating `ProjectRevisionFingerprint` as topology meaning.
- Using one global topology fingerprint as the sole reuse condition and over-invalidating every import after any topology change.
- Moving fingerprint code and silently changing semantic hash behavior.

## Hostile cases

- Method body edit: interface and topology fingerprints unchanged.
- `export X` edit: interface fingerprint changes; topology fingerprint does not.
- `expose .child` edit: interface and topology fingerprints change.
- Adding/removing `package.ph`: topology changes.
- Comment/trivia movement: semantic interface fingerprint unchanged.
- Same resolver target after unrelated package addition remains reusable if dependency trace does not include the changed topology region.

## Required evidence

1. Fingerprint tests:
   ```bash
   cargo test -p phalcom-semantic --test semantic incremental::fingerprints
   ```
   Adapt exact filter to repository test harness.
   Proves moved/delegated fingerprint semantics are unchanged.

2. New module fingerprint/topology tests:
   ```bash
   cargo test -p phalcom-modules fingerprint
   cargo test -p phalcom-modules topology
   ```
   Proves topology-specific invariants.

3. Compile dependency direction:
   ```bash
   cargo check -p phalcom-modules
   cargo check -p phalcom-semantic
   ```
   Proves module crate did not acquire an illegal semantic dependency and semantic wrapper migration compiles.

## Do not run yet

- `phalcom-lsp` tests;
- whole workspace.

## Escalate immediately if

- semantic fingerprint helpers depend on semantic-only types for fields that belong to module interfaces;
- `TopologyFingerprint` cannot be defined without hashing source bodies;
- introducing topology requires changing `ModuleId` representation.

---

### Task 5 — Move canonical interface fingerprint ownership into `phalcom-modules`

**Purpose:** Prevent two independently evolving definitions of interface product equality.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **cross-crate**

**Owned files and symbols:**

- `phalcom-semantic/src/db/fingerprint.rs`
  - `hash_unlinked_interface`
  - `unlinked_interface_product_fingerprint`
  - `hash_linked_interface`
  - `linked_interface_product_fingerprint`
- `phalcom-modules/src/interface.rs`
- new `phalcom-modules/src/fingerprint.rs`
- `phalcom-modules/src/lib.rs`

**Source of truth:** Module-owned interface products.

**Changes:**

Introduce module-owned fingerprint wrappers:

STRUCTURAL:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InterfaceFingerprint(u64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LinkedInterfaceFingerprint(u64);
```

Expose canonical functions:

```rust
pub fn interface_fingerprint(
    interface: &UnlinkedModuleInterface,
) -> InterfaceFingerprint;

pub fn linked_interface_fingerprint(
    interface: &LinkedModuleInterface,
) -> LinkedInterfaceFingerprint;
```

Move the semantic-field hashing logic from `phalcom-semantic/src/db/fingerprint.rs` rather than reimplementing it from scratch.

Semantic DB wrappers remain:

```text
module InterfaceFingerprint.raw()
    → semantic ProductFingerprint
```

if `SemanticDb` still requires its generic fingerprint type.

**Must not:**

- hash source ranges into product fingerprint;
- change existing semantic equality behavior without a new explicit test;
- make `phalcom-modules` depend on `phalcom-semantic`.

**Edit operations:**

1. OPEN `phalcom-semantic/src/db/fingerprint.rs`.
2. FIND `hash_unlinked_interface`, `unlinked_interface_product_fingerprint`, linked equivalents.
3. IDENTIFY only module-owned hashing dependencies.
4. CREATE `phalcom-modules/src/fingerprint.rs`.
5. MOVE/generalize those hash routines.
6. EXPORT through `phalcom-modules/src/lib.rs`.
7. REPLACE semantic implementations with delegating wrappers.
8. UPDATE imports/tests.
9. SEARCH for all call sites:
   ```bash
   rg 'unlinked_interface_product_fingerprint|linked_interface_product_fingerprint' .
   ```
10. Ensure there is one canonical field-hashing algorithm.

**Testing classification:** Focused fingerprint tests at C1.

---

### Task 6 — Introduce canonical topology product and `TopologyFingerprint`

**Purpose:** Make namespace path semantics a first-class product that can be invalidated independently of source bodies.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **multi-file**

**Owned files and symbols:**

- new `phalcom-modules/src/topology.rs` preferred;
- `phalcom-modules/src/stabilization.rs` — reuse `ResolverGeneration`;
- `phalcom-modules/src/identity.rs`;
- `phalcom-modules/src/interface.rs` — `exposed_children`;
- ownership classifier from C0.

**Source of truth:** `ModuleTopology`.

**Required conceptual product:**

STRUCTURAL:

```rust
pub struct ModuleTopology {
    pub generation: ResolverGeneration,
    pub fingerprint: TopologyFingerprint,

    // Exact representation may use nested maps.
    pub nodes: BTreeMap<ModuleId, TopologyNode>,
    pub source_modules: BTreeMap<SourceId, ModuleId>,
    pub children: ...,
    pub exposed_children: ...,
}
```

`TopologyNode` must carry at least:

```text
ModuleId
ModuleKind
SourceLocation
ownership/project context
```

**Fingerprint inputs:**

Include semantic inputs capable of changing path resolution:

- ownership boundary;
- module/package existence;
- logical paths;
- module kind;
- import roots/project source roots/dependency roots;
- package exposure edges.

Exclude:

- source method bodies;
- local expressions;
- comments;
- symbol-only export changes.

**Important distinction:**

```text
ResolverGeneration
    monotonic coarse invalidation epoch

TopologyFingerprint
    deterministic semantic product identity
```

**Must not:**

- repurpose `ProjectRevisionFingerprint`;
- hash all source bytes;
- let workspace root membership itself create a topology node.

**Testing classification:** Focused topology unit/integration tests at C1.

---

### Task 7 — Turn `ImportResolutionTrace` into a reusable resolution product

**Purpose:** Reuse resolved import paths only when the topology facts that justified them are still valid.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **local/multi-file**

**Owned files and symbols:**

- `phalcom-modules/src/resolver.rs`
  - `ImportResolutionTrace`
  - `resolve_import_with_trace`
  - `validate_path_with_trace`

**Source of truth:** `ModuleResolver` resolution trace.

**Changes:**

Extend or wrap the existing trace with a retained product:

STRUCTURAL:

```rust
pub struct ImportResolutionProduct {
    pub importer: ModuleId,
    pub written_path: ImportPathIdentity,
    pub target: Result<ModuleId, ModuleResolutionDiagnostic>,
    pub dependencies: ResolutionTopologyDependencies,
    pub fingerprint: ResolutionFingerprint,
}
```

Do not store AST references in long-lived cache products. Store canonical path spelling/components and source range separately as provenance if needed.

Reuse existing:

```text
ImportResolutionTrace.package_interfaces
```

as the start of exposure dependency tracking.

Add structural dependencies needed for:

- source/module existence;
- package ancestry;
- external exposure edges;
- import-root mapping.

**Negative results:**

`ModuleNotFound` MAY be cached, but it MUST depend on topology such that creating the missing source invalidates it.

**Must not:**

- key only by workspace generation;
- key only by importer + string forever;
- let LSP own this product.

---

### Task 8 — Make module query facade consume topology indexes

**Purpose:** Make repeated editor module queries bounded and consistent with topology without creating request-time scans.

**Risk:**
- Semantic: **MEDIUM**
- Implementation fanout: **multi-file**

**Owned files and symbols:**

- `phalcom-modules/src/query.rs`
  - `module_children`
  - `external_import_children`
  - `reverse_importers`
- `phalcom-semantic/src/snapshot.rs`
  - `ModuleQueryProducts`

**Source of truth:** `ModuleTopology` + canonical resolution products.

**Changes:**

- Add topology/index references to `ModuleQueryFacade`.
- Replace full-map child scanning with topology child index.
- Replace reverse-import scan with retained reverse-resolution index where available.
- Keep the facade read-only and allocation-bounded.

**Must not:**

- move ownership/resolution policy into `ModuleQueryFacade`;
- query filesystem from facade.

**Testing classification:** Existing `phalcom-modules/tests/query.rs` should be extended rather than duplicated.

---

## C1 checkpoint completion

- [ ] Tasks 5–8 implemented.
- [ ] Module-owned fingerprint algorithms are canonical.
- [ ] Semantic DB delegates to canonical interface fingerprints.
- [ ] `TopologyFingerprint` tests distinguish body/export/expose/marker edits.
- [ ] Query facade uses topology indexes where planned.
- [ ] No new semantic dependency from `phalcom-modules`.
- [ ] State file updated.
- [ ] No active incident.

### Suggested commits

```text
C1.1 refactor(modules): own module interface fingerprints
C1.2 feat(modules): introduce canonical module topology product
C1.3 feat(modules): retain topology-aware import resolution products
C1.4 perf(modules): index module query topology
```

---

# 12. Checkpoint C2 — Incremental Persistent Module Workspace

Tasks:
- Task 9 — Separate committed module state from transaction deltas.
- Task 10 — Share base filesystem caches across transactions and split invalidation domains.
- Task 11 — Retain per-module interface products and stop duplicate semantic rebuilding.
- Task 12 — Reuse import resolution products and maintain reverse dependencies.
- Task 13 — Link affected components once and retain linked products.

## Why this is a checkpoint

All five tasks exist to establish one performance contract:

> A body-only edit recomputes the changed source/interface, observes an unchanged interface fingerprint, and stops module propagation.

Implementing only interface caching while still cloning workspace state and rebuilding linking broadly would not establish that contract.

## Entry conditions

- C1 COMPLETE.
- Canonical fingerprints/topology exist.
- Existing semantic DB product-stability behavior remains intact.

## Working set

### Primary

- `phalcom-modules/src/session.rs`
- `phalcom-modules/src/source.rs`
- `phalcom-modules/src/linker.rs`
- module topology/fingerprint files from C1
- `phalcom-semantic/src/workspace.rs`
- `phalcom-semantic/src/session.rs`
- `phalcom-semantic/src/db/query.rs`

### Secondary

- `phalcom-semantic/src/snapshot.rs`
- `phalcom-semantic/src/db/fingerprint.rs`

### Out of scope

- module-error tolerance;
- source index completeness;
- LSP.

## Semantic contract established

- Workspace transaction cost is proportional to changed entries rather than unconditional O(workspace) map cloning.
- Filesystem resolution caches survive safe transaction boundaries.
- Content invalidation is distinct from topology invalidation.
- Each module's canonical unlinked interface is built once per changed relevant source product.
- Semantic session consumes module-owned interface products instead of rebuilding them.
- Unchanged imports do not re-resolve after body edits.
- Overlapping reachable components are not relinked once per entry.
- Linked-interface fingerprint stability halts reverse propagation.

## Semantic risks

- Reusing stale negative resolution after source creation.
- Committing a partially mutated transaction after internal failure.
- Losing reverse-dependency edges needed for later invalidation.
- Letting semantic DB and module session use interfaces from different generations.
- Reusing a linked product whose import target changed while export spelling stayed the same.
- Purging last-known-good too aggressively on transient recomputation.

## Hostile cases

- Edit a method body in a leaf imported by 100 modules: zero import re-resolution and zero relink if interface unchanged.
- Add one export: path resolutions reused; affected linked reverse closure reconsidered.
- Create a previously missing module: cached negative resolution invalidates.
- Remove a source: old module product is hard-purged.
- Failed internal transaction does not mutate committed state.
- Cold and incremental snapshots agree on `ModuleId`, linked export targets, and semantic target identity.

## Required evidence

1. Module work-count tests:
   ```bash
   cargo test -p phalcom-modules incremental
   ```
   Add a dedicated test module/binary if no suitable home exists.

2. Semantic incremental equivalence:
   ```bash
   cargo test -p phalcom-semantic --test semantic incremental
   ```
   Use the repository's actual integration harness filter.

3. Focused crate checks:
   ```bash
   cargo check -p phalcom-modules
   cargo check -p phalcom-semantic
   ```

4. Cache negative-result hostile test:
   - missing import;
   - add source;
   - same session resolves on next update.

## Do not run yet

- LSP integration suite.
- full workspace.

## Escalate immediately if

- preserving transaction atomicity appears to require cloning all canonical products;
- semantic `query_unlinked_interface` cannot accept a module-owned precomputed product without violating DB dependency correctness;
- linker cannot expose a product boundary without changing runtime semantics.

---

### Task 9 — Replace full session staging clone with transaction deltas

**Purpose:** Preserve atomic updates without cloning every workspace map for every edit.

**Risk:**
- Semantic: **MEDIUM**
- Implementation fanout: **local/multi-file**

**Owned file:** `phalcom-modules/src/session.rs`.

**Source of truth:** committed `WorkspaceModuleSession` state.

**Current implementation:**

`apply_batch` constructs a full staged `WorkspaceModuleSession` by cloning universe/maps/linked/resolved state and creating a new provider.

**Target implementation:**

STRUCTURAL:

```rust
struct WorkspaceModuleTransaction<'a> {
    base: &'a WorkspaceModuleSession,
    source_updates: BTreeMap<...>,
    source_removals: BTreeSet<...>,
    identity_updates: ...,
    interface_updates: ...,
    resolution_updates: ...,
    link_updates: ...,
}
```

or an equivalent copy-on-write state object.

Lookups use:

```text
transaction delta
    ↓
committed state
```

Commit applies only validated changed entries.

**Edit operations:**

1. OPEN `session.rs`.
2. FIND `apply_batch`.
3. Identify every field cloned into `staged`.
4. Extract transaction-local mutations.
5. Preserve one commit point.
6. Ensure infrastructure errors return before commit.
7. Ensure source-authored diagnostic handling planned for C3 does not require rollback.
8. Remove full-map cloning.
9. Add an atomicity regression.

**Must not:** introduce a persistent collection dependency solely for this.

**Testing classification:** validated by C2 work-count/atomicity tests.

---

### Task 10 — Split and share filesystem cache state

**Purpose:** Make long-lived resolver caching survive source edits safely.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **multi-file**

**Owned files:**

- `phalcom-modules/src/source.rs`
  - `FilesystemSourceProvider`
  - `clear_cache`
  - caches
- `phalcom-modules/src/session.rs`

**Source of truth:** provider-owned canonical filesystem resolution/source identity.

**Changes:**

Refactor cache state so staged transactions can share generation-safe immutable cache entries.

Recommended shape:

STRUCTURAL:

```rust
struct FilesystemCacheState {
    resolution: Mutex<...>,
    source_text: Mutex<...>,
    source_identity: Mutex<...>,
}

pub struct FilesystemSourceProvider {
    cache: Arc<FilesystemCacheState>,
    topology_generation: AtomicU64 /* or shared generation owner */
}
```

Exact atomics/ownership must fit Rust mutability requirements.

Replace coarse `clear_cache` uses with semantic invalidations:

```text
invalidate_source_content(SourceId)
invalidate_topology(...)
purge_source_identity(SourceId)
```

Rules:

- ordinary `.ph` content change → content only;
- source add/remove/rename → topology + identity;
- package/project marker/config change → topology/ownership;
- interface `expose` change → topology product invalidation via C1, not filesystem cache blanket flush.

**Must not:** allow a transaction to observe an overlay from another uncommitted transaction.

---

### Task 11 — Retain one canonical unlinked interface product and pass it into semantic analysis

**Purpose:** Eliminate duplicate `InterfaceBuilder` work between module session and semantic DB.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **cross-crate**

**Owned files/symbols:**

- `phalcom-modules/src/session.rs` — `WorkspaceModuleUpdate`
- `phalcom-semantic/src/workspace.rs` — `SemanticWorkspaceInput`
- `phalcom-semantic/src/session.rs` — `update_module_workspace`, `update_with_budget_and_cancel`
- `phalcom-semantic/src/db/query.rs` — `query_unlinked_interface`

**Source of truth:** module-owned `UnlinkedModuleInterface` product.

**Changes:**

- Extend `WorkspaceModuleUpdate` with canonical unlinked interface products/fingerprints.
- Extend `SemanticWorkspaceInput` to receive the same products.
- Change semantic DB publication so it records/validates the supplied canonical product rather than invoking `InterfaceBuilder::build` again on the production workspace path.
- Keep a convenience single-module/test constructor if needed, but make any interface building there explicit and non-production.
- `ModuleQueryProducts` should be populated from the same canonical interface map.

**Must not:**

- keep both production builders “for safety”;
- allow semantic source index to see a different interface generation from linker.

**Current implementation evidence:**

`WorkspaceModuleSession::rebuild` builds interfaces, then `SemanticWorkspaceSession::update_with_budget_and_cancel` calls `query_unlinked_interface` over every source.

**Testing classification:** high-risk cross-consumer; cold/incremental equality at C2.

---

### Task 12 — Reuse import resolutions and build reverse resolution index

**Purpose:** Prevent path resolution from repeating when relevant topology is unchanged.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **multi-file**

**Owned files:**

- `phalcom-modules/src/session.rs`
- `phalcom-modules/src/resolver.rs`
- topology products from C1

**Source of truth:** `ImportResolutionProduct`.

**Changes:**

- Retain products keyed by stable importer/import-site identity.
- On changed interface:
  - preserve unchanged import declarations by semantic identity;
  - re-resolve only added/changed imports or imports whose recorded topology dependencies changed.
- Maintain:
  ```text
  target ModuleId → importer/import site(s)
  ```
- Use this reverse index for downstream invalidation and `ModuleQueryFacade::reverse_importers`.

**Must not:** key reuse only by textual path if importer ownership changed.

---

### Task 13 — Retain linked products and stop repeated component linking

**Purpose:** Make linked-interface stability the second module propagation barrier.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **multi-file**

**Owned files:**

- `phalcom-modules/src/linker.rs`
- `phalcom-modules/src/session.rs`

**Source of truth:** canonical linker + `LinkedInterfaceFingerprint`.

**Phase A — required immediately:**

- Stop calling `link_with_unresolved_imports` once per `parsed_sources` key for overlapping reachable components.
- Track visited/component membership and link each affected component once.

**Phase B — required for C2 contract:**

- Retain linked module products by `ModuleId`.
- Recompute affected module/component after interface/resolution changes.
- Compare `LinkedInterfaceFingerprint`.
- Propagate through reverse linked dependencies only when changed.
- Preserve runtime dependency graph correctness.

**Must not:**

- fork a “fast linker” separate from `ModuleLinker`;
- skip linker canonicalization for re-exports.

**Testing classification:** C2 work-count + existing linker tests.

---

## C2 checkpoint completion

- [ ] Transaction staging is delta/COW, not unconditional full-session clone.
- [ ] Filesystem cache state persists safely across edits.
- [ ] Body edit rebuilds one interface and stops propagation when fingerprint stable.
- [ ] Semantic DB consumes canonical module interface product.
- [ ] Import resolution reuse hostile tests pass.
- [ ] Link work is bounded and product-stable.
- [ ] Cold/incremental identities match.
- [ ] State file updated with measured work counts.
- [ ] No active incident.

### Suggested commits

```text
C2.1 refactor(modules): stage workspace updates as deltas
C2.2 perf(modules): split persistent filesystem cache invalidation
C2.3 refactor(semantic): consume canonical module interface products
C2.4 perf(modules): retain resolution and reverse dependency products
C2.5 perf(modules): incrementally reuse linked module products
```

---

# 13. Checkpoint C3 — Tolerant Module Diagnostics and Current Partial Publication

Tasks:
- Task 14 — Introduce module-workspace diagnostic/report products.
- Task 15 — Make resolver/interface failures recoverable per module in workspace mode.
- Task 16 — Make linker produce tolerant workspace results without creating a second linker.
- Task 17 — Convert module diagnostics into semantic snapshot diagnostics/status.
- Task 18 — Fix LSP worker cancellation classification.

## Why this is a checkpoint

The current worker can publish only if `WorkspaceModuleSession::apply_batch` returns `Ok`. Therefore fixing LSP cancellation handling alone would still leave no current semantic product to publish. The module pipeline must first represent user errors as data.

## Entry conditions

- C2 COMPLETE.
- Transaction rollback remains available for infrastructure failures.
- One canonical linker remains authoritative.

## Working set

### Primary

- `phalcom-modules/src/error.rs`
- `phalcom-modules/src/session.rs`
- `phalcom-modules/src/linker.rs`
- `phalcom-semantic/src/session.rs`
- `phalcom-semantic/src/snapshot.rs`
- `phalcom-semantic/src/diagnostic.rs`
- `phalcom-lsp/src/analysis_service.rs`

### Secondary

- `phalcom-modules/src/interface.rs`
- existing diagnostic adapters.

### Out of scope

- source-index completeness;
- LSP navigation fallback deletion.

## Semantic contract established

- Missing module, missing/non-exported name, invalid expose, and similar source-authored errors are current workspace diagnostics.
- Such errors do not discard all valid module products.
- `SnapshotStatus::Partial` reflects blocked module analysis.
- Strict compiler can reject the same report.
- Cancellation means cancellation/supersession, not `Result::Err` generically.
- Infrastructure failures remain distinct.

## Semantic risks

- Continuing after a link error with internally inconsistent binding indexes.
- Treating missing export as unresolved module.
- Losing exact importer item ranges.
- Publishing an invalid `LinkedProgram` as complete.
- Swallowing internal linker invariants as user diagnostics.
- LSP publishing an older snapshot under a newer source generation.

## Hostile cases

- `.module` exists but selected name is absent.
- Selected name exists but is private/not exported.
- One bad import in module A does not erase valid navigation in unrelated module Z.
- Remove an export while importer remains open: current partial snapshot + import diagnostic.
- Restore export: no restart required.
- Deliberate cancellation still discards stale work and does not publish.

## Required evidence

1. Module linker/resolution focused tests:
   ```bash
   cargo test -p phalcom-modules --test linker
   cargo test -p phalcom-modules --test integration
   ```

2. Semantic workspace partial publication:
   ```bash
   cargo test -p phalcom-semantic workspace_partial
   ```
   Add a focused test name/module if needed.

3. LSP worker publication/cancellation tests:
   ```bash
   cargo test -p phalcom-lsp analysis_service
   ```

4. Verify source errors are not converted to cancellation by search:
   ```bash
   rg 'solve_cancelled\s*=\s*publication_result\.is_err' phalcom-lsp/src/analysis_service.rs
   ```
   Expected: zero occurrences.

## Do not run yet

- module-navigation LSP suite → C6.
- workspace-wide → Final Gate.

---

### Task 14 — Add workspace module report and diagnostic model

**Purpose:** Represent module-source failures as publishable data.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **multi-file**

**Owned files:**

- `phalcom-modules/src/error.rs`
- `phalcom-modules/src/session.rs`
- optionally new `phalcom-modules/src/diagnostic.rs`

**Source of truth:** module layer's structured errors/ranges.

**Implementation boundary:**

Introduce a report that separates:

```text
successful module products
source-authored diagnostics
blocked modules
internal/infrastructure failure
```

STRUCTURAL:

```rust
pub struct WorkspaceModuleUpdate {
    pub linked: Arc<LinkedProgram>,
    pub interfaces: ...,
    pub sources: ...,
    pub diagnostics: BTreeMap<ModuleId, Vec<ModuleDiagnostic>>,
    pub blocked_modules: BTreeSet<ModuleId>,
    pub changed_modules: ...,
    pub removed_modules: ...,
    pub identity_changes: ...,
    // topology/resolution products as established in C1/C2
}
```

Exact names may differ.

A separate `Result<WorkspaceModuleUpdate, WorkspaceInfrastructureError>` is preferred over stuffing user diagnostics into `Err`.

**Diagnostic distinctions required:**

- relative import requires package context;
- module not found;
- import beyond package root;
- imported name absent;
- imported name exists but not exported;
- unknown local export;
- invalid expose outside package;
- invalid/missing expose child;
- module path not exposed;
- import binding collision;
- re-export cycle;
- runtime cycle where execution cannot be formed.

**Must not:** downgrade invariant/internal errors to user diagnostics.

---

### Task 15 — Recover per module from interface/resolution failures

**Purpose:** Preserve unaffected products when one module has invalid source-level module semantics.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **multi-file**

**Owned symbols:**

- `WorkspaceModuleSession::rebuild`
- `InterfaceBuilder::build`
- `ModuleResolver::resolve_import`

**Source of truth:** existing structured `InterfaceError` / `ModuleResolutionError`.

**Changes:**

- In workspace/tolerant mode, an invalid module interface may be marked blocked while other interfaces remain.
- `ModuleNotFound` should become an explicit diagnostic rather than silently `continue` without user-visible product.
- Validate `expose .child` existence after candidate topology is known, not only syntactic form.
- Record exact path/item ranges.

**Must not:** make `InterfaceBuilder` silently drop invalid exports/imports without a diagnostic.

---

### Task 16 — Add tolerant linker consumption without a second linker

**Purpose:** Produce usable linked products for valid modules while preserving exact linker semantics.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **multi-file**

**Owned file:** `phalcom-modules/src/linker.rs`.

**Source of truth:** `ModuleLinker` / `LinkContext`.

**Required design:**

Do not write a second workspace linker.

Refactor canonical linking so a shared internal operation can be consumed as:

```text
strict:
    diagnostics/failure → return LinkError

workspace:
    diagnostics/failure → mark affected module/component blocked,
                          retain unaffected canonical products
```

Where safe, a bad import can block only the importing module and dependents rather than the whole workspace.

If current `linked_reads` indexing makes per-import unresolved continuation unsafe, prefer **blocking that module** over adding a fake binding.

**Precise range repair:**

`resolve_export` currently can construct `MissingExport` with `SourceRange::default()`. Thread the importing/re-exporting item range into the diagnostic path so missing/private imported names point at the actual token.

**Private vs absent:**

Use the target `UnlinkedModuleInterface`:

```text
name in declarations && name not in exports
    → NonExportedImport

name absent from declarations/exports
    → UnknownImportName
```

Do not infer privacy by name patterns.

---

### Task 17 — Publish module diagnostics through semantic snapshot

**Purpose:** Make module errors part of the current semantic workspace view.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **cross-crate**

**Owned files:**

- `phalcom-semantic/src/session.rs`
- `phalcom-semantic/src/snapshot.rs`
- `phalcom-semantic/src/diagnostic.rs`

**Source of truth:** module report from Task 14.

**Changes:**

- Convert `ModuleDiagnostic` to `SemanticDiagnostic` without losing module/range.
- Merge module diagnostics with semantic diagnostics per module.
- Set:
  ```rust
  SnapshotStatus::Partial { blocked_modules: ... }
  ```
  when current source has blocked module products.
- Ensure current `sources`, `ModuleQueryProducts`, and source indexes correspond to the current generation.
- Use last-known-good only for explicit cancellation/budget/internal fallback according to existing semantic DB policy.

**Must not:** return previous snapshot as the current generation because one import is invalid.

---

### Task 18 — Separate LSP cancellation from module diagnostics/failure

**Purpose:** Stop treating all publication errors as stale/cancelled batches.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **local**

**Owned file:** `phalcom-lsp/src/analysis_service.rs`.

**Current code anchor:**

```rust
let publication_result =
    compiler_workspace_state.session.apply_module_mutations(mutations);
let solve_cancelled = publication_result.is_err();
```

**Target behavior:**

- publish `Ok(partial publication)` when source-authored module diagnostics exist;
- set cancellation only from worker epoch/shutdown or explicit semantic cancellation signal;
- emit `AnalysisEvent::Error` / internal log for infrastructure failures;
- never call a source error “cancelled”.

**Edit operations:**

1. OPEN worker batch code around `apply_module_mutations`.
2. REPLACE generic `is_err()` cancellation classification.
3. MATCH exact publication error categories established by Task 14/17.
4. Preserve epoch supersession logic.
5. Update worker counters/log event names if they currently conflate source failure/cancel.

**Testing classification:** focused analysis-service regression at C3.

---

## C3 checkpoint completion

- [ ] Tasks 14–18 implemented.
- [ ] User module errors are diagnostics in current partial snapshot.
- [ ] Invalid module does not erase unrelated module products.
- [ ] Missing vs private import is distinct.
- [ ] Exact source ranges are retained.
- [ ] LSP cancellation classification is explicit.
- [ ] `solve_cancelled = publication_result.is_err()` removed.
- [ ] State file updated.
- [ ] No active incident.

### Suggested commits

```text
C3.1 feat(modules): add tolerant workspace module reports
C3.2 fix(modules): retain structured resolution/interface diagnostics
C3.3 refactor(modules): share strict and tolerant linker execution
C3.4 feat(semantic): publish module diagnostics as partial snapshots
C3.5 fix(lsp): distinguish source errors from cancellation
```

---

# 14. Checkpoint C4 — Canonical Cross-Module Semantic Identity

Tasks:
- Task 19 — Fix qualified module type lookup to use linked public exports.
- Task 20 — Add `SemanticTargetId::ModuleBinding(SymbolId)`.
- Task 21 — Project linked bindings into nominal declarations vs module globals correctly.
- Task 22 — Give top-level source bindings canonical `SymbolId`-backed provenance.

## Why this is a checkpoint

The original navigation defect cannot be repaired robustly until semantic identity itself is correct. This checkpoint establishes the target algebra consumed by the source index in C5.

## Entry conditions

- C3 COMPLETE.
- Linked exports can be present or blocked explicitly.
- `SymbolId` remains canonical linker identity.

## Working set

### Primary

- `phalcom-semantic/src/identity.rs`
- `phalcom-semantic/src/resolver.rs`
- `phalcom-semantic/src/session.rs`
- `phalcom-semantic/src/source_index/builder.rs`
- `phalcom-semantic/src/editor.rs`
- `phalcom-semantic/tests/semantic/integration/imported_resolution.rs`
- relevant type-resolution tests

### Secondary

- `phalcom-modules/src/linker.rs`
- `phalcom-modules/src/interface.rs`

### Out of scope

- dependency-token occurrence completeness;
- LSP fallback deletion.

## Semantic contract established

- Qualified type lookup through a module alias sees only linked public exports.
- A private declaration cannot be reached by module alias spelling.
- `DeclarationId` continues to represent nominal class/enum/type-alias identity.
- exported top-level values use `ModuleBinding(SymbolId)`.
- imported nominal declarations retain their declaring-module identity.
- re-exported nominal declarations preserve canonical origin.

## Semantic risks

- Treating every `SymbolId` as value/global and breaking nominal imports.
- Using the importing module rather than the symbol's origin module for `DeclarationId`.
- Making module-valued exports appear as type declarations.
- Allowing same-name user declarations to collide through name-only logic.

## Hostile cases

- Target module exports `Public`, keeps `Hidden` private. `models.Public` resolves; `models.Hidden` does not.
- Target exports `const Point = ...`; importing `Point` must not fabricate `DeclarationId`.
- User has both `class Result` and Universe `Result`: identities remain distinct.
- Re-exported class resolves to original declaration module.
- Qualified multi-member path remains fail-closed until explicit namespace traversal exists.

## Required evidence

1. Exact semantic resolution tests:
   ```bash
   cargo test -p phalcom-semantic imported_resolution
   ```

2. Add focused private-qualified hostile test.

3. Add exported-global target test.

4. Compile:
   ```bash
   cargo check -p phalcom-semantic
   ```
   Proves exhaustive `SemanticTargetId` matches/callers migrated.

## Do not run yet

- LSP module navigation → C6.

## Escalate immediately if

- some production consumer assumes every module global has a `DeclarationId`;
- `DeclarationSurface` classification is insufficient and implementation tries to infer declaration kind by spelling;
- runtime code consumes `SemanticTargetId` directly and would require representation changes.

---

### Task 19 — Route qualified type lookup through linked exports

**Purpose:** Close the confirmed cross-module privacy bypass.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **local**

**Owned file:** `phalcom-semantic/src/resolver.rs`.

**Owned symbol:** `impl TypeResolver for LinkedTypeResolver::resolve_type_name`.

**Source of truth:** target `LinkedModuleInterface.exports`.

**Current implementation anchor:**

```rust
if let Some(LinkedReadSpec::Module(target_mod)) = ... {
    let leaf_name = &members[0];
    let decl =
        DeclarationId::new(target_mod.clone(), leaf_name.clone().into());
    if self.known_declarations.contains(&decl) {
        return Some(decl);
    }
}
```

**EXACT target logic, reconcile imports/types only:**

```rust
if let Some(linked_mod) = self.linked.modules.get(current_module) {
    if let Some(&import_id) = linked_mod.bindings.imports.get::<str>(root) {
        if let Some(LinkedReadSpec::Module(target_mod)) =
            linked_mod.linked_reads.get(import_id.0 as usize)
        {
            let target = self.linked.modules.get(target_mod)?;
            let export = target.interface.exports.get::<str>(&members[0])?;
            let phalcom_modules::interface::LinkedExportTarget::Binding(symbol) =
                &export.target
            else {
                return None;
            };

            let declaration =
                DeclarationId::new(symbol.module.clone(), symbol.name.clone());

            if self.known_declarations.contains(&declaration) {
                return Some(declaration);
            }
        }
    }
}
None
```

Keep the existing `members.len() != 1` fail-closed rule.

**Must not:** fall back to `known_declarations` in target module by spelling.

**Testing classification:** focused semantic hostile test required.

---

### Task 20 — Add module-global semantic target

**Purpose:** Represent linked module values without abusing nominal declaration identity.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **cross-crate within semantic consumers**

**Owned file:** `phalcom-semantic/src/identity.rs`.

**Source of truth:** `phalcom_modules::linker::SymbolId`.

**EXACT enum addition:**

```rust
ModuleBinding(phalcom_modules::linker::SymbolId),
```

Position it next to `Declaration`/`Module` according to local enum ordering conventions.

**Caller migration search:**

```bash
rg 'SemanticTargetId::' phalcom-semantic phalcom-lsp phalcom-core
```

Update exhaustive matches, especially:

- `EditorSemanticQuery::is_definition_site`;
- presentation/hover/completion target classification;
- source-index reverse maps;
- tests.

**Must not:** add a second `ModuleBindingId` if `SymbolId` already expresses canonical linked identity.

---

### Task 21 — Correct linked export → semantic target projection

**Purpose:** Project each linked binding to its actual semantic category.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **local/multi-file**

**Owned file:** `phalcom-semantic/src/session.rs`.

**Current anchor:**

```rust
LinkedExportTarget::Binding(symbol) => {
    context.targets.insert(
        (module.clone(), export.public_name.to_string()),
        SemanticTargetId::Declaration(
            DeclarationId::new(symbol.module.clone(), symbol.name.clone())
        ),
    );
}
```

**Source of truth:**

```text
known nominal declarations
+
linked SymbolId
```

**Target logic:**

Create one helper:

STRUCTURAL:

```rust
fn semantic_target_for_linked_symbol(
    symbol: &SymbolId,
    nominal_declarations: &HashSet<DeclarationId>,
) -> SemanticTargetId
```

Algorithm:

```text
candidate DeclarationId(symbol.module, symbol.name)

if candidate is a known class/enum/type alias declaration:
    Declaration(candidate)
else:
    ModuleBinding(symbol.clone())
```

The helper must use actual declaration tables/sets, not syntax names.

Use the same helper for:

- public export target projection;
- selective import remote target projection;
- re-export source target projection.

**Must not:** classify by capitalization or naming convention.

---

### Task 22 — Give top-level values module-binding source targets

**Purpose:** Make the definition site of `const version` match imported/re-exported `ModuleBinding(SymbolId)`.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **multi-file**

**Owned files:**

- `phalcom-semantic/src/source_index/builder.rs`
  - `visit_let`
- `source_index/scope.rs`
- editor definition classification.

**Source of truth:** `SymbolId(module, name)` for top-level bindings.

**Changes:**

- Local/parameter/destructure bindings remain `SemanticTargetId::Binding(SourceSiteId)`.
- Top-level `let`/`const` declaration sites receive:
  ```text
  SemanticTargetId::ModuleBinding(SymbolId)
  ```
- Add source metadata sufficient to mark that site as a definition.
- `EditorSemanticQuery::is_definition_site` handles `ModuleBinding`.

**Must not:** use source-site identity as the cross-module target for a top-level global.

---

## C4 checkpoint completion

- [ ] Tasks 19–22 implemented.
- [ ] Private qualified type hostile test passes.
- [ ] Exported global uses `ModuleBinding(SymbolId)`.
- [ ] Nominal imports/re-exports still use canonical `DeclarationId`.
- [ ] All exhaustive target matches compile.
- [ ] State file updated with target algebra.
- [ ] No active incident.

### Suggested commits

```text
C4.1 fix(semantic): enforce linked exports for qualified type lookup
C4.2 feat(semantic): add canonical module binding targets
C4.3 fix(semantic): project linked bindings by semantic category
C4.4 test(semantic): lock private visibility and global target identity
```

---

# 15. Checkpoint C5 — Complete Source Provenance and Import-Origin Semantics

Tasks:
- Task 23 — Add enum, variant, variant-family/field source declaration coverage.
- Task 24 — Separate import alias-local identity from upstream origin.
- Task 25 — Index import paths and remote selective items.
- Task 26 — Index exports and re-exports.
- Task 27 — Index expose child paths and package/module source sites.

## Why this is a checkpoint

This is the compiler-owned source-graph completion required before LSP can delete its special import fallback. The tasks are mutually meaningful only when `target_at` and reverse definition/reference queries can represent the complete dependency surface.

## Entry conditions

- C4 COMPLETE.
- Semantic target algebra is final for this program.
- Linked target projection is correct.

## Working set

### Primary

- `phalcom-semantic/src/source_index/builder.rs`
- `phalcom-semantic/src/source_index/occurrence.rs`
- `phalcom-semantic/src/source_index/scope.rs`
- `phalcom-semantic/src/source_index/site.rs`
- `phalcom-semantic/src/source_index/mod.rs`
- `phalcom-semantic/src/session.rs`
- `phalcom-semantic/tests/semantic/integration/source_index.rs`
- `phalcom-semantic/tests/semantic/integration/imported_resolution.rs`

### Secondary

- AST enum/dependency structures in `phalcom-ast` — inspect only; no grammar changes.
- enum semantic tables for canonical IDs.

### Out of scope

- LSP protocol conversion;
- rename protocol implementation beyond semantic identity support.

## Semantic contract established

- Enum declaration names have `DeclarationId` source sites.
- Variants and payload fields use existing `VariantId`, `VariantFamilyId`, and `VariantFieldId`.
- Module import path tokens can target `ModuleId`.
- Selective remote imported token targets upstream canonical semantic target.
- Local alias declaration/use has local binding identity and explicit upstream origin.
- Export token targets the exported local canonical target.
- Re-export path targets module and item targets canonical upstream origin.
- `expose` child targets `ModuleId`.
- Module definition targets have source provenance.

## Semantic risks

- Reconstructing variant IDs differently from enum semantic analysis.
- Making import alias declaration a second definition of the upstream declaration.
- Breaking current lexical usage resolution by changing import-target storage.
- Double-recording overlapping path/item occurrences with ambiguous shortest-range lookup.
- Attaching unresolved diagnostics as fake semantic targets.

## Hostile cases

- `from .shapes import Circle as C`: alias rename is local; go-to-definition follows upstream.
- Upstream declaration references include remote import token, not alias token as a declaration.
- `export version` where version is module-global targets `ModuleBinding`.
- `export Either from .either` targets original enum declaration.
- `expose .missing` yields diagnostic, no fabricated `ModuleId`.
- Enum declaration and imported enum target exactly the same `DeclarationId`.

## Required evidence

1. Source index suite:
   ```bash
   cargo test -p phalcom-semantic source_index
   ```

2. Imported-resolution suite:
   ```bash
   cargo test -p phalcom-semantic imported_resolution
   ```

3. Cross-consumer editor definition/reference test:
   - target at remote import token;
   - target at alias declaration/use;
   - definition locations;
   - reference sets.

4. `cargo check -p phalcom-semantic`.

## Do not run yet

- LSP integration → C6.

## Escalate immediately if

- enum source indexing appears to require new enum identity construction distinct from existing semantic tables;
- alias-local identity cannot be preserved without changing type inference;
- AST lacks a required source range already used by interface extraction.

---

### Task 23 — Index enums, variants, and variant fields

**Purpose:** Fix the immediate enum definition gap and complete already-declared source-site kinds.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **multi-file**

**Owned symbols:**

- `build_source_scope_index`
- `SourceScopeBuilder::visit_statements`
- new `visit_enum` / enum behavior helper(s)
- `SourceSiteKind::{Variant,VariantFamily,VariantField}`

**Current implementation anchors:**

```rust
if let Statement::Class(...)
else if let Statement::TypeAlias(...)
```

and:

```rust
Statement::Enum(_) => {}
```

**Changes:**

- Pre-register enum root `DeclarationId`.
- Create `DeclarationSourceInfo` for enum name/range.
- Traverse enum variants using the exact selector/`VariantId` construction already used by semantic enum analysis.
- Register variant payload fields with `VariantFieldId`.
- Register enum/root and variant behavior callables using existing `CallableId` rules.
- Use `VariantFamilyId` only where the current semantic model exposes a source-addressable family entity; do not synthesize one merely because `SourceSiteKind` exists.

**Code classification:** STRUCTURAL — exact enum AST/member helper shape must be reconciled with current SC-4.8 enum implementation before editing.

**Required pre-edit inspection:**

```bash
rg 'VariantFamilyId::|VariantFieldId::|VariantId::new' phalcom-semantic/src
```

Use those constructors exactly.

---

### Task 24 — Add import-origin relation without collapsing alias identity

**Purpose:** Support correct alias rename/reference semantics while retaining upstream navigation.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **multi-file**

**Owned files:**

- `source_index/scope.rs`
- `source_index/builder.rs`
- `source_index/occurrence.rs`
- `editor.rs`

**Source of truth:**

```text
local alias binding = SourceSiteId
upstream origin = SemanticTargetId
```

**Target data model:**

STRUCTURAL:

```rust
pub struct ImportBindingOrigin {
    pub local_binding: SourceSiteId,
    pub remote_target: SemanticTargetId,
}
```

Store by local import binding site in `SourceScopeIndex` or an adjacent canonical source product.

**Migration of current behavior:**

Current test `imported_alias_keeps_local_declaration_metadata_and_external_read_identity` expects the alias site/use to have the upstream `Declaration` target directly. That expectation must be revised.

Target:

```text
alias declaration site target
    Binding(alias_site)

alias usage target
    Binding(alias_site)

import origin(alias_site)
    Declaration(upstream) or ModuleBinding(upstream)

remote `Circle` token
    upstream target
```

`EditorSemanticQuery` follows the origin for navigation when requested.

**Important:** Type inference does not use this source-index alias identity for linked type resolution; verify this assumption before edit. If a production semantic checker consumes `SourceScopeIndex::resolve_name` for type truth, stop and trace before changing it.

---

### Task 25 — Index import path and selective remote occurrences

**Purpose:** Make import navigation compiler-owned.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **local/multi-file**

**Owned file:** `source_index/occurrence.rs`.

**Changes:**

Add dependency-preamble traversal before/alongside body traversal.

For:

```phalcom
from .either import Either as E
```

record:

```text
.either/path range
    OccurrenceKind::Module
    target Module(either ModuleId)

Either name_range
    Reference
    target upstream semantic target

E alias range
    declaration/local Binding target
```

For module import:

```phalcom
import .shapes as shapes
```

record path target separately from alias declaration.

If the AST exposes segment ranges, prefer final-segment exact range for `target_at`. Whole-path occurrence may additionally exist if deterministic shortest-range lookup remains unambiguous.

**Must not:** derive module target from spelling in occurrence builder; use `SourceIndexContext.resolved_imports`.

---

### Task 26 — Index export and re-export occurrences

**Purpose:** Make public interface syntax part of the canonical reference graph.

**Risk:**
- Semantic: **MEDIUM/HIGH**
- Implementation fanout: **local**

**Owned files:**

- `source_index/builder.rs`
- `source_index/occurrence.rs`

**Changes:**

For:

```phalcom
export Either
```

record `Either` as a reference to the actual local semantic target.

For:

```phalcom
export Either from .either
```

record:

```text
.either → Module(target)
Either  → canonical upstream target
```

No new declaration is created.

Use exact ranges from AST/interface surfaces.

---

### Task 27 — Index expose and module definition sources

**Purpose:** Complete package path provenance and make module targets navigable.

**Risk:**
- Semantic: **MEDIUM**
- Implementation fanout: **multi-file**

**Owned files:**

- source index files;
- semantic session context;
- snapshot module products.

**Changes:**

For:

```phalcom
expose .parser
```

record child path occurrence → `SemanticTargetId::Module(parser)`.

Ensure each physical module/package has a canonical source definition location available through snapshot module products. A synthetic source site MAY represent the module boundary if useful, but module navigation need not invent a zero-range semantic occurrence if module provenance is already represented protocol-neutrally in C6.

**Must not:** make `expose` a binding/reference to a value.

---

## C5 checkpoint completion

- [ ] Enum root source site exists.
- [ ] Variant/field/callable source coverage is consistent with existing IDs.
- [ ] Import path and remote item occurrences exist.
- [ ] Alias local identity and upstream origin are distinct.
- [ ] Export/re-export/expose occurrences exist.
- [ ] Imported enum exact identity matches declaration site.
- [ ] State file updated.
- [ ] No active incident.

### Suggested commits

```text
C5.1 feat(semantic): index enum and variant source identities
C5.2 refactor(semantic): model import aliases with upstream origins
C5.3 feat(semantic): index import and module path occurrences
C5.4 feat(semantic): index export reexport and expose references
```

---

# 16. Checkpoint C6 — Editor Query Cutover and LSP Fallback Retirement

Tasks:
- Task 28 — Add protocol-neutral semantic definition locations.
- Task 29 — Route navigation/reference presentation through `EditorSemanticQuery`.
- Task 30 — Delete import-specific LSP semantic fallback.
- Task 31 — Correct and extend LSP module navigation fixtures.

## Why this is a checkpoint

C5 produces the source graph; C6 proves that the LSP no longer reconstructs module meaning. Deletion evidence is mandatory because the migration is incomplete if the old fallback can still silently win.

## Entry conditions

- C5 COMPLETE.
- `target_at` can resolve module paths and import remote items.
- Module provenance exists in snapshot.
- Alias origin relation exists.

## Working set

### Primary

- `phalcom-semantic/src/editor.rs`
- `phalcom-semantic/src/snapshot.rs`
- `phalcom-lsp/src/backend.rs`
- `phalcom-lsp/src/request_context.rs`
- `phalcom-lsp/src/import_completion.rs`
- `phalcom-lsp/tests/module_navigation.rs`
- `phalcom-lsp/tests/semantic_boundary.rs` if present locally

### Secondary

- hover/completion adapter helpers only where target matching requires new `ModuleBinding`.

### Out of scope

- protocol rename feature if not already supported;
- runtime;
- parser.

## Semantic contract established

- Go-to-definition for import path and selective imported item comes through source semantic targets.
- Module target definitions are resolved protocol-neutrally by semantic editor queries.
- LSP performs no request-time import resolution.
- LSP package-less sibling import is rejected just like strict compiler.
- Valid package fixture navigates path, imported enum/class/global, and usage consistently.

## Semantic risks

- LSP still calling `module_queries().resolved_import_target` from request handlers.
- Definition locations for alias jumping to alias itself instead of origin.
- Module target location returning stale filesystem path.
- Completion exposing private module declarations.

## Hostile cases

- Valid package with exported enum `Either`: path + imported token + use all navigate.
- Same package but no `export Either`: path navigates to module, item does not pretend to resolve and diagnostic exists.
- No `package.ph`: `.either` path does not resolve.
- Import alias: go-to-definition follows upstream; reference query on alias remains local.
- Module-global exported value navigates to top-level binding.

## Required evidence

1. LSP navigation:
   ```bash
   cargo test -p phalcom-lsp --test module_navigation
   ```

2. Semantic boundary tests:
   ```bash
   cargo test -p phalcom-lsp --test semantic_boundary
   ```
   Only if this test binary exists at local drift check; otherwise use the repository's equivalent anti-fallback suite.

3. Negative searches:
   ```bash
   rg 'compiler_import_definition_location|import_path_range_at_offset' phalcom-lsp/src
   ```
   Expected: zero production occurrences.

   ```bash
   rg 'resolved_import_target' phalcom-lsp/src/backend.rs
   ```
   Expected: zero request-time navigation uses. Intentional import-completion use through snapshot query facade may remain outside `backend.rs`.

4. LSP crate:
   ```bash
   cargo test -p phalcom-lsp
   ```
   Run now because this checkpoint materially changes the shared protocol adapter semantics.

## Do not run yet

- full workspace → Final Gate.

## Escalate immediately if

- LSP needs filesystem I/O to convert a compiler definition to `Location`;
- `EditorSemanticQuery` lacks source provenance that exists only in an LSP cache;
- removing fallback makes valid package navigation impossible because C5 target is absent.

---

### Task 28 — Add `SemanticDefinitionLocation`

**Purpose:** Give editor consumers one compiler-owned API for ranged declarations and module source definitions.

**Risk:**
- Semantic: **MEDIUM**
- Implementation fanout: **multi-file**

**Owned files:**

- `phalcom-semantic/src/editor.rs`
- `phalcom-semantic/src/snapshot.rs`

**Source of truth:** `SourceSemanticIndex` + module query provenance.

**Target API:**

STRUCTURAL:

```rust
pub enum SemanticDefinitionLocation {
    Site(SourceSiteId),
    Module {
        module: ModuleId,
        source: SourceLocation,
    },
}
```

or:

```rust
Source { module, range }
Module { module, source }
```

Choose the shape that avoids LSP-specific URI/range concepts.

Add:

```rust
pub fn definition_locations(
    &self,
    target: &SemanticTargetId,
) -> Vec<SemanticDefinitionLocation>
```

Rules:

- local import alias binding navigation can follow `import_origin`;
- nominal/global/callable/etc use definition sites;
- module target uses `ModuleQueryFacade::definition_source`.

Keep lower-level `definition_sites` if references/tests need it.

---

### Task 29 — Route LSP target locations through editor definitions

**Purpose:** Make `Backend` a protocol adapter.

**Risk:**
- Semantic: **MEDIUM**
- Implementation fanout: **local**

**Owned file:** `phalcom-lsp/src/backend.rs`.

**Current anchor:**

```rust
fn compiler_target_locations(...) -> Vec<Location> {
    self.compiler_sites_locations(
        compiler,
        compiler.editor().definition_sites(target)
    )
}
```

**Changes:**

- Consume `editor().definition_locations(target)`.
- Convert module source location to URI at adapter boundary.
- Convert ranged sites using existing line indexes/source products.
- Update `compiler_reference_locations` only where `ModuleBinding`/alias semantics require it.

**Must not:** call `ModuleResolver`.

---

### Task 30 — Delete import-specific fallback

**Purpose:** Complete authority migration.

**Risk:**
- Semantic: **HIGH** because deletion proves C5/C6 completeness.
- Implementation fanout: **local**

**Owned file:** `phalcom-lsp/src/backend.rs`.

**Delete:**

- `import_path_range_at_offset`
- `compiler_import_definition_location`
- fallback call in `goto_definition`

**Final `goto_definition` shape:**

```text
request context
→ exact snapshot target_at
→ editor definition_locations
→ LSP conversion
→ None
```

No AST import special case.

**Negative gate:** zero production matches for both deleted helper names.

---

### Task 31 — Migrate LSP module navigation tests to true package semantics

**Purpose:** Stop the test suite from reintroducing the original bug.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **tests**

**Owned test:** `phalcom-lsp/tests/module_navigation.rs`.

**Current hostile baseline:**

`goto_definition_on_relative_import_path_and_selective_export` creates:

```text
shapes.ph
main.ph
```

without `package.ph` and expects `.shapes` navigation to work.

**Required migration:**

Positive fixture:

```text
workspace/
├── package.ph
├── main.ph
└── shapes.ph
```

`shapes.ph` exports `Circle`/`Either`.

Add negative companion:

```text
workspace/
├── main.ph
└── shapes.ph
```

Same relative import.

Expected:
- no module definition;
- relative-import package-context diagnostic after publication;
- no imported-member semantic target.

Add private export hostile test:

```text
package.ph present
shapes.ph defines Circle but does not export it
```

Expected:
- `.shapes` path definition succeeds;
- `Circle` imported token/use does not resolve as public target;
- non-exported diagnostic.

---

## C6 checkpoint completion

- [ ] Tasks 28–31 implemented.
- [ ] Positive package navigation passes.
- [ ] Package-less sibling hostile test passes.
- [ ] Private member path/member distinction passes.
- [ ] `compiler_import_definition_location` deleted.
- [ ] no request-time module resolver exists in LSP navigation.
- [ ] full `phalcom-lsp` tests pass.
- [ ] State file updated.
- [ ] No active incident.

### Suggested commits

```text
C6.1 feat(semantic): expose protocol-neutral definition locations
C6.2 refactor(lsp): consume semantic definition locations
C6.3 refactor(lsp): retire import navigation fallback
C6.4 test(lsp): enforce explicit package navigation semantics
```

---

# 17. Checkpoint C7 — Topology Lifecycle, Metrics, Cross-Layer Parity, and Closure

Tasks:
- Task 32 — Make package/project/source topology mutations reclassify identities incrementally.
- Task 33 — Add module-layer work metrics and hard-purge lifecycle.
- Task 34 — Add strict compiler vs workspace identity parity tests.
- Task 35 — Run migration negative gates and documentation/state closure.

## Why this is a checkpoint

Correct steady-state semantics are insufficient for a long-lived LSP. The original architecture requires topology edits to transition without restart, while preserving incremental bounds and compiler parity.

## Entry conditions

- C6 COMPLETE.
- LSP relies only on current semantic snapshot.
- Module fingerprints and cache dependencies exist.

## Working set

### Primary

- `phalcom-modules/src/session.rs`
- topology/ownership/fingerprint modules
- `phalcom-modules/src/source.rs`
- `phalcom-semantic/src/session.rs`
- `phalcom-semantic/src/db/mod.rs`
- `phalcom-lsp/src/analysis_service.rs`
- `phalcom-lsp/src/workspace_scan.rs`
- cross-layer test modules

### Secondary

- `phalcom-core/src/modules/compile.rs`
- `phalcom-modules/src/query.rs`

### Out of scope

- parallel execution;
- incremental parser;
- alternative collections.

## Semantic contract established

- `package.ph` add/remove reclassifies current sources without restart.
- source add/remove invalidates negative import caches and obsolete identities.
- `project.toml` changes invalidate project ownership/root/dependency topology.
- `export` edit does not unnecessarily invalidate path topology.
- `expose` edit invalidates external path consumers.
- obsolete identity products are hard-purged.
- work-count metrics demonstrate product-stability propagation.
- strict compiler and workspace produce equal canonical identity for same source universe.

## Semantic risks

- retaining stale last-known-good products for a permanently deleted `ModuleId`;
- global topology reset on every body edit;
- scanner treating workspace root as ownership root;
- source move temporarily giving two identities;
- compiler and workspace choosing different standalone package root.

## Hostile cases

- Add `package.ph` to two open sibling files: relative import becomes valid without restart.
- Remove `package.ph`: same import becomes invalid without restart.
- Add previously missing `foo.ph`: missing import resolves without restart.
- Delete imported module: current partial diagnostic; old target no longer navigable.
- Add/remove `export`: path identity unchanged; linked export identity changes.
- Add/remove `expose`: external import validity changes; internal same-package import remains valid.
- A body-only edit in a widely imported module triggers one interface build, zero path resolutions, zero relinks when interface fingerprint is stable.

## Required evidence

1. Module lifecycle:
   ```bash
   cargo test -p phalcom-modules topology
   cargo test -p phalcom-modules incremental
   ```

2. Semantic cold/incremental:
   ```bash
   cargo test -p phalcom-semantic incremental
   cargo test -p phalcom-semantic module_query_provenance
   ```

3. LSP live topology:
   ```bash
   cargo test -p phalcom-lsp module_navigation
   cargo test -p phalcom-lsp analysis_service
   ```

4. Core parity:
   ```bash
   cargo test -p phalcom-core core::modules
   ```

5. Cross-layer parity fixture added in Task 34.

## Escalate immediately if

- file watching cannot observe `package.ph` because scanner/watcher filtering excludes it;
- topological reclassification requires a full process restart;
- performance counters show body-only edit traversing all modules.

---

### Task 32 — Topology-aware source lifecycle invalidation

**Purpose:** Make ownership/topology changes first-class incremental events.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **cross-crate**

**Owned files:**

- `phalcom-modules/src/session.rs`
- topology/ownership files
- `phalcom-lsp/src/analysis_service.rs`
- `phalcom-lsp/src/workspace_scan.rs`

**Source of truth:** candidate discovery produces source events; module ownership/topology classifies them.

**Changes:**

- Treat scanner results strictly as source candidates.
- On add/remove:
  - reclassify affected ownership domain;
  - update topology;
  - retire old `ModuleId`s;
  - allocate/use canonical new identities;
  - invalidate resolution products whose dependencies intersect change.
- On `package.ph` add/remove:
  - invalidate affected directory ownership cache and descendants bounded by next ownership boundary;
  - refresh package/module identities.
- On project config/watch event:
  - invalidate affected persistent project topology.
- Active overlay `package.ph` SHOULD participate in topology if source-provider architecture permits it. If implementation cannot support unsaved marker topology in this checkpoint without a second filesystem authority, document the limitation as a release blocker rather than silently treating disk state as current.

**Must not:** make `WorkspaceScanState::roots` semantic package roots.

---

### Task 33 — Add work metrics and hard purge

**Purpose:** Verify optimization and prevent cache growth in long-lived IDE sessions.

**Risk:**
- Semantic: **MEDIUM**
- Implementation fanout: **multi-file**

**Owned files:**

- module session/topology/provider;
- `phalcom-semantic/src/db/mod.rs`;
- existing metrics structs.

**Required module counters:**

Equivalent counters:

```text
interfaces_built
interfaces_reused

imports_resolved
import_resolutions_reused

linked_modules_recomputed
linked_modules_reused
linked_components

ownership_lookups
ownership_cache_hits

filesystem_resolution_hits
filesystem_resolution_misses

topology_invalidations

changed_sources
affected_modules
identity_changes

purged_products
```

Do not force exact public names if existing `SemanticUpdateStats` should absorb some of them; keep module-owned work measurable at the module layer.

**Hard purge:**

Add explicit semantic DB/module cache lifecycle for permanently obsolete identities.

Current `SemanticDb` preserves `last_known_good` during invalidation; add a separate purge path that removes obsolete module query keys/products and dependency edges.

**Must not:** use hard purge for ordinary recomputation/cancellation.

---

### Task 34 — Add compiler/workspace canonical identity parity fixture

**Purpose:** Detect future divergence even if each layer passes its own tests.

**Risk:**
- Semantic: **HIGH**
- Implementation fanout: **tests/cross-crate**

**Preferred fixture:**

```text
package/
├── package.ph
├── main.ph
├── either.ph
└── constants.ph
```

`either.ph`:

```phalcom
enum Either<L, R> { ... }
export Either
```

`constants.ph`:

```phalcom
const version = "1"
export version
```

Assert between `ProgramAnalyzer` and `SemanticWorkspaceSession`:

```text
same ModuleId(main)
same ModuleId(either)
same resolved import target
same linked export SymbolId
same DeclarationId(Either)
same ModuleBinding(SymbolId(constants, version))
same re-export origin if façade included
```

Add a package-less negative comparator.

**Testing location:**

Prefer a cross-crate integration test in the highest existing layer that can invoke both APIs without creating dependency cycles. `phalcom-core` already depends on semantic/modules and is a likely owner. If LSP APIs are needed solely for display, keep LSP parity separate.

---

### Task 35 — Migration closure and repository documentation/state update

**Purpose:** Prove old authorities cannot silently remain.

**Risk:**
- Semantic: **MEDIUM**
- Implementation fanout: **repository-wide search/docs**

**Required negative searches:**

```bash
rg 'resolve_standalone_import' .
```

Expected: zero production occurrences; historical docs may remain.

```bash
rg 'compiler_import_definition_location|import_path_range_at_offset' phalcom-lsp/src
```

Expected: zero occurrences.

```bash
rg 'DeclarationId::new\(target_mod' phalcom-semantic/src/resolver.rs
```

Expected: zero qualified cross-module fallback occurrences.

```bash
rg 'SemanticTargetId::Declaration\(DeclarationId::new\(symbol\.module' phalcom-semantic/src
```

Expected: zero unconditional linked-binding projections.

```bash
rg 'load_synthetic_root' phalcom-modules/src/session.rs phalcom-core/src/modules/compile.rs
```

Expected:
- zero arbitrary-directory pseudo-package uses;
- if the function remains for compatibility/tests, every production occurrence is justified in the state file.

```bash
rg 'publication_result\.is_err\(\)' phalcom-lsp/src/analysis_service.rs
```

Expected: no cancellation classification based solely on generic error.

Review comments/docs near deleted behavior and update stale claims.

---

## C7 checkpoint completion

- [ ] Tasks 32–35 implemented.
- [ ] Live package-marker transitions pass.
- [ ] Missing-module negative cache recovery passes.
- [ ] hard purge proven for deleted/reidentified modules.
- [ ] body-edit work counts meet expected boundary.
- [ ] compiler/workspace identity parity passes.
- [ ] all negative/deletion gates reviewed.
- [ ] implementation state has no incident/deferred checkpoint evidence.
- [ ] ready for Final Delivery Gate.

### Suggested commits

```text
C7.1 feat(modules): invalidate topology and ownership incrementally
C7.2 perf(modules): add work metrics and cache purge lifecycle
C7.3 test(core): enforce compiler workspace module identity parity
C7.4 chore(modules): remove obsolete module compatibility paths
```

---

# 18. Cross-Checkpoint API Migration Ledger

The implementing agent must update these call sites when the corresponding product changes.

## `WorkspaceModuleUpdate`

Current consumers:

- `phalcom-semantic/src/session.rs`
  - `apply_module_mutation`
  - `apply_module_mutations`
  - `update_module_workspace`
- direct module-session tests.

Planned additions may include:

```text
interfaces
topology
resolution products
module diagnostics
blocked modules
module stats
```

Do not add fields without migrating semantic publication in the same checkpoint.

## `SemanticWorkspaceInput`

Current fields:

```rust
pub linked: Arc<LinkedProgram>,
pub sources: BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
pub generation: u64,
```

Current construction sites include:

- `SemanticWorkspaceSession::update_module_workspace`;
- `phalcom-semantic/src/workspace.rs::analyze_single_module`;
- semantic tests constructing synthetic linked worlds;
- `phalcom-core` analysis helpers that call `analyze_workspace`.

When canonical module interfaces are added:

- production workspace path MUST pass module-owned interfaces;
- direct tests may use a helper constructor that explicitly builds canonical interface products;
- do not silently reintroduce production duplicate `InterfaceBuilder`.

## `SemanticTargetId`

Adding `ModuleBinding(SymbolId)` requires exhaustive review:

```bash
rg 'SemanticTargetId::' phalcom-semantic phalcom-lsp phalcom-core
```

Classify each match:

- identity logic;
- definition/reference classification;
- presentation;
- completion;
- test expectation.

Do not default `_ =>` merely to make compilation pass where behavior is semantically meaningful.

## `ModuleQueryProducts` / `ModuleQueryFacade`

Topology/index additions must preserve:

- immutable snapshot-only reads;
- no filesystem;
- no lifecycle mutation.

---

# 19. Diagnostic Mapping Requirements

The exact final diagnostic code names should follow the existing catalog, but the following semantic distinctions must survive.

| Module condition | Required semantic distinction | Preferred source range |
|---|---|---|
| standalone relative import | requires package context | import path |
| missing module | module not found | import path/final segment |
| import beyond root | beyond package root | relative root/path |
| target name absent | imported name not found | selected item |
| target name private | imported name not exported | selected item |
| bad local `export` | unknown export | exported item |
| expose outside package | expose invalid here | child token |
| exposed child absent | exposed child missing | child token |
| external traversal blocked | module path not exposed | blocked segment |
| binding collision | import/declaration collision | conflicting binding |
| re-export cycle | cycle | participating re-export |
| runtime cycle | runtime init cycle | best retained dependency edge range |

`SourceRange::default()` is unacceptable when the AST/interface already retained the actual token range.

---

# 20. Performance Acceptance Criteria

This program is not complete merely because tests pass functionally.

Add deterministic work-count assertions.

## 20.1 Body-only edit

Fixture:

```text
A imports B
B imports C
100 unrelated modules
```

Change only a callable body in `C`, preserving interface.

Expected module-layer work:

```text
changed source parses/recovery: 1
interfaces built: 1
interfaces reused: all unchanged
import resolutions recomputed: 0
linked interfaces recomputed: 0
topology fingerprint: unchanged
```

Semantic callable work may occur in C and semantic dependents according to existing DB dependencies.

## 20.2 Export-only edit

Change `export X`.

Expected:

```text
C interface rebuilt: yes
TopologyFingerprint: unchanged
path resolution recomputation: none solely because of export
linked C: recomputed
reverse symbol importers/re-exporters: reconsidered
unrelated modules: reused
```

## 20.3 Exposure edit

Change `expose .child`.

Expected:

```text
package interface rebuilt
TopologyFingerprint changes
external import paths depending on that package edge reconsidered
same-owner internal imports not dependent on exposure remain reusable
```

## 20.4 Package marker edit

Add/remove `package.ph`.

Expected:

```text
ownership reclassification in affected domain
ModuleId identity changes where required
topology invalidation
dependent resolutions re-run
obsolete identities hard-purged
no process restart
```

Do not impose a wall-clock threshold as the primary test. Work-count assertions are deterministic and diagnose regressions better.

---

# 21. Failure Protocol

If required checkpoint evidence fails unexpectedly, mark that checkpoint:

```text
C<N> — INCIDENT
```

Do not continue into dependent checkpoints.

For the incident record, capture:

## 21.1 Exact reproduction

```text
command:
failing test/check:
important output:
```

## 21.2 Direct path

Example:

```text
LSP fixture
→ analysis worker
→ SemanticWorkspaceSession
→ WorkspaceModuleSession
→ ModuleResolver
→ failed import product
```

## 21.3 Passing comparator

Find one nearby success:

```text
persistent project works; standalone package fails
```

or:

```text
cold analysis resolves target; incremental update does not
```

## 21.4 Classification

Use exactly one primary classification:

```text
PRODUCT
FIXTURE
DEPENDENCY/PUBLICATION
BACKEND/HARNESS
BASELINE
PLAN DRIFT
```

## 21.5 Narrow repair boundary

State exactly which symbol/file may change.

## 21.6 Rejected broad fixes

Default forbidden responses:

- do not restore package-less sibling fallback;
- do not special-case LSP;
- do not weaken export visibility;
- do not fabricate `DeclarationId`;
- do not convert missing semantic target to `Dynamic`;
- do not disable topology cache invalidation;
- do not modify parser without evidence.

Only resume after the incident evidence is recorded.

---

# 22. Checkpoint Supervisor Report Template

At checkpoint completion:

```text
Checkpoint C<N> COMPLETE

Established:
    <dominant semantic claim>

Changed:
    path — symbol/responsibility
    ...

Evidence:
    command — PASS — proves ...
    ...

Hostile cases:
    case — PASS
    ...

Negative gates:
    search — expected/observed

Performance:
    interfaces built/reused: ...
    resolutions recomputed/reused: ...
    linked modules recomputed/reused: ...

Deferred:
    command → C<M>/Final Gate

Unexpected findings:
    none | concise facts

Next:
    C<N+1> — <name>
```

---

# 23. Checkpoint Evidence Summary

This table begins as `PLANNED`. The executing agent updates status only after running evidence.

| Checkpoint | Semantic contract | Evidence | Status |
|---|---|---|---|
| C0 | Explicit ownership and package identity shared by module workspace/core | modules ownership/session + core module tests + fallback negative search | PLANNED |
| C1 | Canonical topology/fingerprint products owned by modules | fingerprint/topology tests + semantic delegation parity | PLANNED |
| C2 | Persistent incremental module workspace | work-count tests + cold/incremental semantic equivalence | PLANNED |
| C3 | User module errors publish current partial snapshots | linker/resolution + semantic partial + analysis-service tests | PLANNED |
| C4 | Cross-module semantic identity obeys exports and global/nominal distinction | imported-resolution hostile tests + target exhaustive check | PLANNED |
| C5 | Complete source provenance including imports/exports/enums | semantic source-index + imported-resolution tests | PLANNED |
| C6 | LSP consumes editor semantic queries only | module-navigation + semantic-boundary + deletion searches | PLANNED |
| C7 | Lifecycle, metrics, parity, migration closure | topology/incremental/LSP/core parity + negative gates | PLANNED |

No row may be changed to COMPLETE without all required checkpoint evidence.

---

# 24. Final Broad Delivery Gates

Run only after C7 is COMPLETE.

## 24.1 Formatting

```bash
cargo +stable fmt --all -- --check
```

Proves:
- repository Rust formatting is clean.

Does not prove:
- semantic module invariants.

## 24.2 Workspace compilation

```bash
cargo +stable check --workspace --all-targets
```

Proves:
- cross-crate API migrations compile;
- test/example targets compile;
- exhaustive Rust match/caller updates are complete.

Does not prove:
- ownership or visibility semantics.

## 24.3 Full tests

```bash
cargo +stable test --workspace --all-targets
```

Proves:
- broad regression compatibility across the workspace.

Does not replace:
- focused checkpoint hostile-case evidence.

## 24.4 Clippy

```bash
cargo +stable clippy --workspace --all-targets -- -D warnings
```

Proves:
- delivery-quality Rust lint gate.

## 24.5 Optional project-specific targeted rerun

After broad gates, rerun only if a broad failure implicated a checkpoint:

```text
smallest exact regression
    ↓
checkpoint suite
    ↓
affected crate
    ↓
workspace
```

Do not rerun all gates repeatedly after unrelated small repairs.

---

# 25. Final Negative / Deletion Gates

Execute at final delivery even if previously run.

```bash
rg 'resolve_standalone_import' phalcom-modules phalcom-core phalcom-semantic phalcom-lsp
```

Expected:
- zero production code.
- historical docs/tests may mention the old symbol only if clearly archival.

```bash
rg 'compiler_import_definition_location|import_path_range_at_offset' phalcom-lsp/src
```

Expected:
- zero.

```bash
rg 'solve_cancelled\s*=\s*publication_result\.is_err' phalcom-lsp/src
```

Expected:
- zero.

```bash
rg 'DeclarationId::new\(target_mod' phalcom-semantic/src/resolver.rs
```

Expected:
- zero cross-module qualified fallback.

```bash
rg 'SemanticTargetId::Declaration\(DeclarationId::new\(symbol\.module' phalcom-semantic/src
```

Expected:
- zero unconditional linked-binding coercion.

```bash
rg 'load_synthetic_root' phalcom-modules/src/session.rs phalcom-core/src/modules/compile.rs
```

Expected:
- no arbitrary-parent package creation.
- any intentional standalone-package loader compatibility call explicitly validates `package.ph` and is recorded.

Review:

```bash
rg 'package-less|sibling import|synthetic root|special import definition|fallback' \
  phalcom-modules/src phalcom-semantic/src phalcom-lsp/src phalcom-core/src
```

Inspect each remaining match manually for stale comments or obsolete compatibility behavior.

---

# 26. Deferred-Evidence Audit

Before release completion, the state file MUST contain no deferred command without one of:

```text
PASS
explicitly removed from scope with justification
known release blocker
```

In particular verify that:

- LSP tests deferred from C0–C5 ran in C6/C7;
- strict compiler parity ran in C7;
- workspace-wide format/check/test/clippy ran at Final Gate.

Never convert “deferred” into “assumed pass”.

---

# 27. Staged Commit Groups

Recommended integration order:

```text
C0
  ownership classifier
  workspace/core ownership cutover
  ownership tests

C1
  module fingerprint ownership
  topology product
  resolution dependencies/query indexes

C2
  transaction/cache persistence
  canonical interface handoff
  resolution/link reuse

C3
  module report
  tolerant linker/diagnostics
  semantic partial publication
  LSP cancellation correction

C4
  qualified visibility
  ModuleBinding target
  semantic projection

C5
  enum/source-index completeness
  import origin
  import/export/reexport/expose occurrences

C6
  editor definition locations
  LSP cutover
  fallback deletion
  corrected navigation fixtures

C7
  topology live invalidation
  metrics/purge
  cross-layer parity
  migration closure
```

Do not squash all semantic migrations into one opaque commit if checkpoint-level review is available.

---

# 28. Known Scope Exclusions

The following work is intentionally excluded:

1. Incremental parser implementation.
2. Parallel interface building/linking.
3. Replacing `BTreeMap`/`BTreeSet` globally.
4. Interning all `ModuleId`/`ModulePath` values.
5. Lock-free caches.
6. Runtime/VM value representation changes.
7. General-purpose rename protocol design beyond source identity/origin support.
8. New language syntax.
9. Deep multi-component qualified type traversal beyond current supported module-alias member form.
10. Package-manager feature expansion.
11. Metadata serialization redesign except if exact module identity parity requires adapting an already-persisted field.
12. General LSP performance rewrites unrelated to module semantics.

If profiling after this program identifies `OccurrenceIndex::occurrence_for_site` or similar local scans as hot, treat that as a separate measured optimization.

---

# 29. State-File Completion Requirements

At the end of this implementation program, `lsp-module-architecture-implementation-state.md` MUST contain:

- final local implementation HEAD;
- all established MOD-* invariants;
- final decisions;
- all checkpoint evidence;
- all negative/deletion evidence;
- module work-count results;
- compiler/LSP parity result;
- no unresolved `INCIDENT`;
- no forgotten deferred gates;
- any intentional compatibility remnants with justification;
- next roadmap action, if any.

---

# 30. Release-Complete Criteria

The implementation is complete only when:

- [ ] C0 through C7 are all `COMPLETE`;
- [ ] all checkpoint semantic evidence passes;
- [ ] all high-risk hostile cases pass;
- [ ] package-less sibling resolution is impossible in compiler workspace and LSP;
- [ ] `package.ph` is consistently enforced as package identity;
- [ ] cross-module qualified lookup obeys linked exports;
- [ ] exported globals have `SymbolId`-backed semantic identity;
- [ ] enum/import/export/re-export/expose provenance is complete;
- [ ] LSP import navigation fallback is deleted;
- [ ] source-authored module failures publish current partial snapshots;
- [ ] body-only edits demonstrate bounded module recomputation;
- [ ] topology edits recover without restart;
- [ ] obsolete identities are hard-purged;
- [ ] strict compiler/workspace canonical identity parity passes;
- [ ] final negative/deletion gates pass;
- [ ] `cargo +stable fmt --all -- --check` passes;
- [ ] `cargo +stable check --workspace --all-targets` passes;
- [ ] `cargo +stable test --workspace --all-targets` passes;
- [ ] `cargo +stable clippy --workspace --all-targets -- -D warnings` passes;
- [ ] no unresolved state-file incident exists;
- [ ] stale architecture comments/docs changed by this migration are updated.

---

# 31. Resume Point

The implementing agent should begin with:

```text
Checkpoint C0
Task 1 — Introduce one canonical ownership classifier
```

Before editing, perform the repository drift protocol and record:

```text
local branch
local HEAD
working-tree changes
whether the C0 Primary symbols still match this plan
```

Do not begin from the LSP fallback. The original symptom is presented in the LSP, but the first authority violation is module ownership.

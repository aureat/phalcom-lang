# Phalcom LSP Module Architecture — Implementation State

Prepared plan revision:
- remote baseline: e932aac4e21a5b346e719ede5a24f94e7b924ab3
- local implementation HEAD: d77960c15c9cf3090152f53c0f348c69fa588573

## Established invariants

- MOD-OWN-1: `classify_entry_ownership` is the single filesystem ownership classifier.
- MOD-PKG-1: only `package.ph` establishes standalone package ownership; plain sibling files remain standalone modules.
- MOD-ID-1: classified filesystem paths are canonicalized before project/module identity mapping.

## Decisions

- D-01: Baseline drift check confirmed: remote e932aac4 vs local d77960c1 has zero committed diff on C0 primary files (`phalcom-modules/src/source.rs`, `project.rs`, `session.rs`, `phalcom-core/src/modules/compile.rs`).
- D-02: Working tree changes outside modules crates are preserved untouched.
- D-03: `classify_entry_ownership(&Path, &mut ProjectUniverse)` applies persistent-project precedence, then contiguous standalone-package marker ancestry, then standalone-module fallback. `Inline` remains explicit.
- D-04: `ProjectUniverse::load_standalone_package` is validated by `package.ph`. `load_synthetic_root` remains only as a legacy compatibility/test helper; no C0 workspace or strict-entry consumer uses it.
- D-05: The strict standalone-package test hang reproduces at pre-C0 f9e07721, so it is tracked as an inherited semantic-source-index incident rather than a C0 module diagnosis.
- D-06: Interface fingerprinting (`InterfaceFingerprint`, `LinkedInterfaceFingerprint`) canonicalized in `phalcom-modules/src/fingerprint.rs`. `phalcom-semantic/src/db/fingerprint.rs` delegates directly, eliminating duplicate hashing logic.
- D-07: `ModuleTopology` and `TopologyFingerprint` introduced in `phalcom-modules/src/topology.rs`. Topology fingerprint tracks project boundaries, module existence, module kind, and package exposure, while ignoring method bodies, comments, and local declaration changes.
- D-08: `ImportResolutionProduct` in `phalcom-modules/src/resolver.rs` retains target, written path, consulted packages, and resolution fingerprint for topology-aware cache reuse.
- D-09: `ModuleQueryFacade` consumes `ModuleTopology` and precomputed reverse import indexes for O(1) child and reverse lookup without request-time map scans.
- D-10: Task 9 transaction staging: delta-based `WorkspaceModuleTransaction` updates stage deltas and invalidations with zero full-session map clones; failed transactions return `Err` leaving committed state untouched.
- D-11: Task 10 cache sharing: `FilesystemCacheState` in `Arc` separates `invalidate_source_content` (single-file text edit) from `invalidate_topology` (file addition/removal) and `purge_source_identity`.
- D-12: Task 11 canonical interface sharing: `WorkspaceModuleSession` retains `UnlinkedModuleInterface` and `InterfaceFingerprint`. `WorkspaceModuleUpdate` and `SemanticWorkspaceInput` pass precomputed interfaces to `SemanticWorkspaceSession` and `query_unlinked_interface`, eliminating duplicate `InterfaceBuilder::build` passes.
- D-13: Task 12 import resolution reuse: `WorkspaceModuleSession` retains `ImportResolutionProduct` and reverse dependency index. Stable interfaces reuse resolved targets directly without re-resolving paths.
- D-14: Task 13 component-bounded linking: linking groups reachable interfaces by connected component and links each component once instead of once per source; body-only edits observe unchanged interface fingerprints and stop linking propagation entirely.
- D-15: C4 adds `SemanticTargetId::ModuleBinding(SymbolId)` for canonical exported module globals; nominal `DeclarationId` remains limited to known class, enum, and type-alias declarations.
- D-16: C4 centralizes linked-symbol projection in `semantic_target_for_linked_symbol`; public exports, selective imports, and re-exports consume the same nominal-vs-global decision without spelling inference.
- D-17: Qualified module type lookup follows `LinkedReadSpec::Module` into target public exports and preserves exported symbol origin; private names and unsupported deep qualification fail closed.
- D-18: Top-level `let`/`const` declaration sites use `ModuleBinding(SymbolId)`; imports, locals, parameters, and destructured bindings retain lexical `Binding(SourceSiteId)` identity. Editor definitions classify only top-level global sites as definitions.

## Evidence ledger

| Checkpoint | Command | Result | Proves |
|---|---|---|---|
| C0 | `git diff --quiet e932aac4..HEAD -- <C0 primary files>` | PASS (zero committed drift) | C0 primary files had no committed post-baseline changes before this worktree slice. |
| C0 | `RUSTFLAGS='' cargo check -p phalcom-modules` | PASS | Modules crate compiles with ownership changes. |
| C0 | `RUSTFLAGS='' cargo test -p phalcom-modules --test workspace_session -- --nocapture` | PASS: 14 passed | Ownership, package marker, nested package, project precedence, and session lifecycle fixtures. |
| C0 | `RUSTFLAGS='' cargo test -p phalcom-modules --test standalone_incremental_imports -- --nocapture` | PASS: 2 passed | Package-relative import recovery and package-less sibling rejection. |
| C0 | `RUSTFLAGS='' cargo test -p phalcom-modules --test integration` | PASS: 6 passed | Existing project/package resolver contracts. |
| C0 | `RUSTFLAGS='' cargo test -p phalcom-modules --test package_semantic_contract` | PASS: 4 passed | Package/module kind contracts. |
| C0 | `RUSTFLAGS='' cargo test -p phalcom-core --test core modules_universe::package_entry_requires_package_ph -- --nocapture` | PASS: 1 passed | `EntrySelection::Package` requires `package.ph`. |
| C0 | `RUSTFLAGS='' cargo test -p phalcom-core --test core modules_universe::main_ph_does_not_create_package_identity -- --nocapture` | PASS: 1 passed | `main.ph` alone remains standalone; marker enables package behavior. |
| C0 | `rg 'resolve_standalone_import|load_synthetic_root' phalcom-modules/src/session.rs phalcom-core/src/modules/compile.rs` | PASS: zero | Deleted workspace sibling fallback and arbitrary-parent strict entry use. |
| C0 | `git diff --check` | PASS | No whitespace errors in current worktree diff. |
| C1 | `RUSTFLAGS='' cargo test -p phalcom-modules --test topology` | PASS: 6 passed | Topology fingerprint stability, exposure/kind sensitivity, cycle detection, descendants. |
| C1 | `RUSTFLAGS='' cargo test -p phalcom-modules --test query` | PASS: 3 passed | Query facade routes through topology and reverse index when provided. |
| C1 | `RUSTFLAGS='' cargo test -p phalcom-modules` | PASS: 81 passed | All module tests pass with canonical fingerprints and topology. |
| C1 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic incremental::fingerprints` | PASS: 32 passed | Interface and semantic product fingerprints remain identical when delegating to modules. |
| C2 | `RUSTFLAGS='' cargo test -p phalcom-modules --test workspace_session` | PASS: 18 passed | Body-only edit stops propagation, atomicity on parse error, negative cache invalidation, product hard-purge on removal. |
| C2 | `RUSTFLAGS='' cargo test -p phalcom-modules` | PASS: 85 passed | Full modules test suite passes cleanly with retained products and delta transactions. |
| C2 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic incremental` | PASS: 127 passed | Semantic incremental equivalence intact with canonical interface products. |
| C2 | `RUSTFLAGS='' cargo test -p phalcom-core modules_universe` | PASS: 16 passed | Core universe suites pass cleanly. |
| C3 | `cargo test -p phalcom-semantic --test semantic module_query_provenance::semantic_snapshot_publishes_relative_import_alias_path_and_provenance -- --nocapture` | PASS: 1 passed after fixture marker repair | Canonical relative module alias/path query and source provenance under package ownership. |
| C3 | `cargo test -p phalcom-semantic --test semantic semantic::integration::imported_resolution::editor_definition_sites_exclude_local_import_declaration_for_external_target -- --nocapture` | PASS: 1 passed after fixture marker repair | External declaration target excludes local selective-import binding site. |
| C3 | `RUSTFLAGS='' cargo test -p phalcom-modules --test linker` | PASS: 11 passed | Strict linker and tolerant linker diagnostics/valid-module retention. |
| C3 | `RUSTFLAGS='' cargo test -p phalcom-modules --test workspace_session` | PASS: 18 passed | Workspace ownership, tolerant update products, and package-less sibling rejection. |
| C3 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic module_query_provenance` | PASS: 1 passed | Semantic module query publication. |
| C3 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic imported_resolution` | PASS: 5 passed | Canonical imported declaration/module targets and identity preservation. |
| C3 | `cargo test -p phalcom-lsp --test imported_binding_resolution` | PASS: 2 passed after fixture/synchronization repair | Cross-module definition and unresolved local-binding references through LSP. |
| C3 | `RUSTFLAGS='' cargo test -p phalcom-semantic workspace_partial` | PASS: 1 passed, 1125 filtered | Real partial snapshot regression publishes current module diagnostic, blocked-module count, and unrelated valid module product. |
| C3 | `cargo test -p phalcom-lsp analysis_service` | PASS: 0 tests matched | Plan filter executes successfully but is vacuous: no `analysis_service`-named test exists in registered LSP targets. |
| C3 | `cargo test -p phalcom-lsp` | BLOCKED: 50 passed, 8 failed, 2 ignored | Unit/imported-binding lanes green; remaining failures are older cross-file workspace fixture/baseline lanes outside C3. |
| C3 | `if rg 'solve_cancelled\\s*=\\s*publication_result\\.is_err' phalcom-lsp/src/analysis_service.rs; then unexpected hit; else zero matches; fi` | PASS: zero matches | Source-authored publication errors are not classified by generic `Result::Err` cancellation assignment. |
| C4 | `cargo test -p phalcom-semantic imported_resolution` | PASS: 8 passed | Qualified public/private/deep lookup, canonical nominal imports, exported global projection, top-level global provenance, and partial snapshots. |
| C4 | `cargo check -p phalcom-semantic` | PASS | Exhaustive `SemanticTargetId` consumers compile with `ModuleBinding(SymbolId)`. |
| C4 | `git diff --check` | PASS | C4 changes introduce no whitespace errors. |

## Negative/deletion gates

| Checkpoint | Search | Expected | Observed |
|---|---|---|---|
| C0 | `rg 'resolve_standalone_import|load_synthetic_root' phalcom-modules/src/session.rs phalcom-core/src/modules/compile.rs` | zero | zero |
| C0 | `rg 'load_synthetic_root' phalcom-modules/src phalcom-core/src` | no workspace/core consumer | declaration only at `phalcom-modules/src/project.rs:284` |
| C1 | `rg 'fn hash_import_path|fn hash_metadata' phalcom-semantic/src/db/fingerprint.rs` | zero | zero (delegated to phalcom-modules) |
| C2 | Full session clone in `apply_batch` | zero | zero (replaced by transaction deltas and write-ahead commit) |
| C2 | Duplicate `InterfaceBuilder::build` on production workspace semantic update | zero | zero (uses `input.interfaces` from module session) |

## Deferred gates

- `cargo test -p phalcom-lsp` → C6
- `cargo test --workspace --all-targets` → Final Gate

## Resolved incident (Pre-C0 inherited hang)

- Fix: eliminated duplicate cumulative occurrences in `attach_formal_analysis` via `baseline_occurrences` and deferred `rebuild_target_occurrences` to single batch call after callable analysis.
- Evidence: `RUSTFLAGS='' cargo test -p phalcom-core --test core modules_universe::standalone_package_has_no_project_binding -- --nocapture` PASS in ~15s. Full `modules_universe` suite PASS (16 passed).

## Next resume action

C4 focused evidence is green for Tasks 19–22. C3-I3 remains deferred to separate LSP fixture/baseline ownership; supervisor review is next before C5.

## Incident C3-I1 — Canonical import publication loses alias and external target

Observed: two C3-adjacent semantic publication regressions reproduce in the current working tree. A whole-module alias has no query target under its local alias spelling, and a selective import occurrence resolves to its local `Binding` instead of the exported declaration target.

Reproduction:

```text
cargo test -p phalcom-semantic --test semantic module_query_provenance::semantic_snapshot_publishes_relative_import_alias_path_and_provenance -- --nocapture
→ FAIL at phalcom-semantic/tests/module_query_provenance.rs:50: resolved_import_target(main, "shapes") = None; expected shapes ModuleId

cargo test -p phalcom-semantic --test semantic semantic::integration::imported_resolution::editor_definition_sites_exclude_local_import_declaration_for_external_target -- --nocapture
→ FAIL at phalcom-semantic/tests/semantic/integration/imported_resolution.rs:250: target_at(import declaration) = Binding(main); expected Declaration(shapes::Circle)
```

Direct path: `WorkspaceModuleSession::rebuild` resolves written import paths and builds `LinkedProgram`; `SemanticWorkspaceSession::update` derives `ModuleQueryProducts` and `SourceIndexContext`; `build_source_scope_index`/`OccurrenceIndex` attach import targets; `EditorSemanticQuery` reads published target occurrences.

Passing comparator: direct `SourceIndexContext` regressions `imported_binding_use_resolves_to_exported_declaration_not_local_import_site`, `imported_alias_keeps_local_declaration_metadata_and_external_read_identity`, and `module_import_read_resolves_to_canonical_module_target` pass when canonical resolved target maps are supplied explicitly. Module facade fixture `facade_exposes_canonical_roots_children_exports_and_provenance` also passes with explicit alias/path products.

Classification: fixture/setup regression inherited from pre-MOD-PKG-1 tests; not current C3 product behavior, backend harness, or cancellation. Both tests create sibling source files without the required `package.ph`, so canonical relative resolution correctly blocks them before alias/export projection.

Root cause: test fixtures from commits `4ed41be78` and `80e0b56b4` assumed the retired package-less sibling fallback. Current ownership contract requires `package.ph` for relative sibling imports.

Fix boundary: add empty `package.ph` to each temp workspace fixture so relative imports are valid under MOD-PKG-1, then rerun unchanged assertions. Do not weaken assertions, fabricate targets, restore package-less sibling fallback, or add feature-local inference.

Do not change: unrelated dirty Rust, AST, typing, editor-state, or documentation work outside this incident.

Regression: adding empty `package.ph` to both temp workspaces made both unchanged assertions pass. Production module, semantic, and LSP code untouched for this incident.

## Incident C3-I2 — Unresolved import references not ready in raw LSP harness

Observed: after C3-I1 fixture correction, `imported_binding_definition_crosses_module_boundary_at_declaration_and_use` passes, but `unresolved_selective_import_uses_compiler_owned_local_binding_identity` still receives no LSP result array for the local unresolved binding.

Reproduction:

```text
cargo test -p phalcom-lsp --test imported_binding_resolution
→ 1 passed; 1 failed: unresolved_selective_import_uses_compiler_owned_local_binding_identity
→ panic at phalcom-lsp/tests/imported_binding_resolution.rs:252:51: compiler-owned unresolved binding references
```

Direct path: raw `didOpen` harness → analysis worker publication → `RequestContext` exact-source pin → `SemanticSnapshot::editor().target_at` → `reference_sites` → LSP locations. The test currently sleeps 200 ms instead of waiting for exact semantic publication.

Passing comparator: the same target/index path passes in semantic `imported_resolution` after package correction; LSP imported-definition test in the same target passes after package correction. Existing `TestLsp::open_and_wait` synchronizes exact source publication.

Classification: backend test harness timing/publication synchronization, pending response inspection. No production cancellation or semantic fallback repair authorized.

Fix boundary: synchronize raw harness with current exact semantic publication before querying references; preserve local binding behavior and protocol assertions. Do not add request-time inference or weaken result assertions.

Resolution: replacing fixed 200 ms sleep with bounded reference polling made both imported-binding LSP tests pass. The polling waits for compiler-owned result availability and does not change semantic behavior.

## Incident C3-I3 — Full LSP package gate retains unrelated cross-file fixture failures

Observed: `cargo test -p phalcom-lsp` completed unit and imported-binding targets, then failed 8 integration tests: `stage2_index::goto_definition_and_workspace_symbol_resolve_across_files`, `stage4_hover::cross_file_hover_resolves_the_doc_from_the_declaring_file`, `stage3_completion::import_completion_uses_published_module_queries`, and five `workspace_semantics::*` tests.

Classification: fixture/baseline outside C3. These tests use package-less relative-import fixture roots (`.a`, `.b`, `.provider`, `.mover`) while current MOD-PKG-1 requires `package.ph`; unrelated workspace semantic propagation failures remain outside tolerant diagnostics/cancellation ownership.

Do not change: C3 production code, package ownership semantics, unrelated C4–C6 tests, or parallel dirty files. C3 checkpoint remains focused-green but package-gate blocked until supervisor authorizes separate fixture migration/baseline investigation.

Resume pointer: run targeted C3 evidence from this ledger. Do not rerun full `cargo test -p phalcom-lsp` unless cross-file fixture ownership is explicitly repaired or reclassified.

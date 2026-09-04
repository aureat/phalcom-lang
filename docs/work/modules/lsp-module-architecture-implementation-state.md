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

## Negative/deletion gates

| Checkpoint | Search | Expected | Observed |
|---|---|---|---|
| C0 | `rg 'resolve_standalone_import|load_synthetic_root' phalcom-modules/src/session.rs phalcom-core/src/modules/compile.rs` | zero | zero |
| C0 | `rg 'load_synthetic_root' phalcom-modules/src phalcom-core/src` | no workspace/core consumer | declaration only at `phalcom-modules/src/project.rs:284` |

## Deferred gates

- `cargo test -p phalcom-lsp` → C6
- `cargo test --workspace --all-targets` → Final Gate

## Active incident

### Exact reproduction

```text
command:
RUSTFLAGS='' cargo test -p phalcom-core --test core modules_universe::standalone_package_has_no_project_binding -- --nocapture
failing test/check:
modules_universe::standalone_package_has_no_project_binding
important output:
running 1 test; no `STEP 2: running` output after more than 60 seconds
```

Process evidence was approximately 97.5% CPU and 301 MB RSS. A stack sample repeatedly pointed to `phalcom_semantic::source_index::SourceSemanticIndex::rebuild_target_occurrences` and ordered target comparison.

### Direct path

`EntrySelection::Package` → `ProgramAnalyzer::analyze_entry_selection` → `discover_and_analyze` → `phalcom_semantic::analyze_workspace` → `build_source_semantic_index` → `SourceSemanticIndex::rebuild_target_occurrences`.

### Passing comparator

`modules_universe::package_entry_requires_package_ph`, `modules_universe::main_ph_does_not_create_package_identity`, and `modules_linking::comp_08_standalone_module_succeeds` pass. The same standalone-package test hangs on clean pre-C0 f9e07721.

### Classification

`BASELINE`

### Narrow repair boundary

No C0 file changes. If separately authorized, investigate `phalcom-semantic/src/source_index/mod.rs`, specifically `SourceSemanticIndex::rebuild_target_occurrences`, with a dedicated semantic incident plan.

### Rejected broad fixes

- Do not restore package-less sibling fallback.
- Do not special-case LSP.
- Do not weaken export visibility.
- Do not fabricate semantic declarations or targets.
- Do not disable topology invalidation.
- Do not modify parser without evidence.

Do not enter C1 until strict core module evidence is resolved or explicitly reclassified.

## Next resume action

Resolve or explicitly reclassify inherited semantic-source-index hang, then rerun strict core module evidence and complete C0 checklist. C0 implementation tasks 1–4 are otherwise present and focused module gates are green.

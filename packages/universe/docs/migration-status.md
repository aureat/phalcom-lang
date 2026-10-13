# Universe shadow migration status

## Current increment

- **Increment:** S0/S1 baseline, faithful shadow duplication, and initial migration metadata only
- **Branch:** `feat/telos-collections`
- **Commit:** `9be1ed1eba3f94ba05e8e4aca1ee55f5ccf13131` (starting revision; checkout was clean before this increment; this increment is uncommitted)
- **Scope:** Copy the complete authored legacy Universe package into `packages/universe/`; add initial provenance, identity, module, dependency, and status ledgers.
- **Files added:** Complete authored shadow package, excluding only ignored, untracked `src/.DS_Store`, plus the five docs in `packages/universe/docs/`.
- **Files changed:** None in the legacy tree.
- **Legacy files modified:** None.
- **Declarations migrated:** None; ownership and source layout match the legacy tree. The conservative scan indexes 364 top-level type-like declaration lines and 77 top-level value declarations; it does not claim parser/compiler identity validation.
- **Interfaces normalized:** None.
- **Identity changes:** None. Both manifests retain source package name and namespace `universe`; these manifest fields do not make standalone filesystem loading equivalent to the production builtin identity.

## Validation

- The full authored directory was copied, including `project.toml`, authored source, and existing ancillary files; the only exclusion is ignored, untracked Finder metadata `src/.DS_Store`. The baseline contains 173 on-disk regular files: 168 `.ph` files and five other files; 172 regular files were copied (the four non-`.ph` files listed in `legacy-sync.md` plus the 168 `.ph` source files).
- SHA-256 hashes for all copied legacy files are recorded in `legacy-sync.md` and compared against the copied files.
- Source-level discoverability inspection found production `UniverseSourceProvider::source_text` is an explicit `include_str!` table targeting only `phalcom-core/core/universe/src/**`; Cargo workspace members are explicit and omit `packages/universe`; `ProjectUniverse::load_root` requires an explicit manifest path and follows declared dependencies. No production registration/build behavior was changed. This is source inspection, not a runtime test.
- No semantic compiler validation exists for the shadow tree in this increment. No tests or compiler checks were run.

## Future integration prerequisite

A standalone `ProjectUniverse` load of the copied manifest would be filesystem-backed and have `Resolved` identity. Production builtin authority remains `ProjectIdentity::Universe` through the explicit legacy `UniverseSourceProvider`; this shadow package is not production builtin authority. This distinction must be preserved during later isolated validation and integration planning.

## Risks and prerequisites

- The copied source still has the original organization, including `collections/iterator_laws.ph`; controlled reorganization has not begun.
- Declaration and explicit-import inventories are source scans, not semantic validation. The 364-row type-like declaration index includes intrinsic aliases/data forms, while 77 top-level values are separately listed in `module-map.md`; neither inventory establishes parser/compiler identity. Implicit prelude bindings, native associations, bootstrap roots, and runtime initialization order remain unverified.
- Isolated semantic analysis for the shadow package is not available/established by this increment. Do not register both source trees in one production module graph or VM.
- Reconcile subsequent legacy changes through the provenance ledger before later moves.

## Next increment

Begin the controlled scalar/foundational-type migration only after reviewing this baseline and establishing intended target ownership for `Number`, `Int`, `Float`, `Option`, `Result`, and `Unit`. Preserve production authority and record every canonical owner move.

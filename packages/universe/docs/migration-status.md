# Universe shadow migration status

## Current increment

- **Increment:** S0/S1 baseline, faithful shadow duplication, and initial migration metadata
- **Branch:** `feat/telos-collections`
- **Commit:** `4a219874bc6e9d6538c229c2e2da712e9a57eb5e`
- **Starting revision:** `9be1ed1eba3f94ba05e8e4aca1ee55f5ccf13131`; checkout was clean before this increment.
- **Scope:** Copy the complete authored legacy Universe package into `packages/universe/`; add provenance, identity, module, dependency, and status ledgers.
- **Files added:** 172 authored package files (168 `.ph` files and four other files) plus five migration documents.
- **Files changed:** None in the legacy tree or production compiler/runtime/native/prelude code.
- **Legacy files modified:** None.
- **Declarations migrated:** None; the source layout and owners match the legacy tree. The inventories recognize 364 top-level type-like declaration lines and 77 top-level value declarations; they do not claim parser/compiler identity validation.
- **Interfaces normalized:** None.
- **Identity changes:** None. Both manifests retain source package name and namespace `universe`; these fields do not make standalone filesystem loading equivalent to production builtin identity.

## Validation

- The authored directory copy excludes only ignored, untracked Finder metadata at `src/.DS_Store`. The legacy directory has 173 regular files on disk; 172 authored files were copied.
- The SHA-256 manifest covers all 172 copied legacy files; every copied file matches its recorded hash.
- Source inventories match internally: 364 declaration rows match across the identity ledger and module map, and 77 top-level value declarations are listed. The native association ledger covers all 78 `UniverseKey` variants (72 binding rows and 6 without a binding row). The prelude inventory lists 37 metadata flags, 9 source-authored additions, and 3 compiler-provided `core.proof` entries.
- Source-level discoverability inspection found that production `UniverseSourceProvider::source_text` explicitly includes only `phalcom-core/core/universe/src/**`; Cargo workspace members omit `packages/universe`; and `ProjectUniverse::load_root` requires an explicit manifest path. No production registration or build behavior changed. These are source inspections, not runtime tests.
- No shadow compiler validation is available in this increment. No tests or compiler checks were run. `git diff --check` reports one trailing blank line in copied `src/text/patterns.ph`; that byte is present in the legacy source and retained to preserve the exact copy. The migration-document diff passes the whitespace check.

## Future integration prerequisite

A standalone `ProjectUniverse` load of the copied manifest would be filesystem-backed and have `Resolved` identity. Production builtin authority remains `ProjectIdentity::Universe` through the explicit legacy `UniverseSourceProvider`; this shadow package is not production builtin authority. Preserve this distinction during later isolated validation and integration planning.

## Risks and prerequisites

- The copied source retains the original organization, including `collections/iterator_laws.ph`; controlled reorganization has not begun.
- Declaration and import inventories are source scans, not semantic validation. Implicit prelude bindings, native associations, bootstrap roots, and runtime initialization order have not been validated against the shadow package.
- Isolated semantic analysis for the shadow package is not established. Do not register both source trees in one production module graph or VM.
- Reconcile later legacy changes through the provenance ledger before migrating affected declarations.

## Next increment

Begin the controlled scalar/foundational-type migration. Preserve production authority, record each canonical owner move, and mark semantic/native/prelude integration gates according to the evidence available.

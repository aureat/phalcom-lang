# Universe shadow migration status

## Current increment

- **Increment:** S3 scalar split — `Number`, `Int`, and `Float`
- **Branch:** `feat/telos-collections`
- **Commit:** Pending final review; based on `4d98eb53` (`docs(universe): record baseline delivery status`).
- **Scope:** Keep `Number` in `scalar/number.ph`; move `Int` to `scalar/int.ph` and `Float` to `scalar/float.ph`; author explicit module exports and re-export all three through the scalar package; update shadow migration inventories.
- **Files added:** `src/scalar/int.ph`, `src/scalar/float.ph`.
- **Files changed:** Shadow scalar source, identity ledger, current module map, dependency graph, legacy synchronization ledger, and migration status.
- **Legacy files modified:** None.
- **Declarations migrated:** `Int` and `Float` moved from shadow owner `universe.scalar.number` to `universe.scalar.int` and `universe.scalar.float`. `Number` remains at `universe.scalar.number`.
- **Interfaces normalized:** Authored `export Number`, `export Int`, and `export Float` in declaration modules; scalar facade exposes `.int` and `.float` and re-exports all three declarations.
- **Identity changes:** Shadow source-owner paths changed for `Int` and `Float`. This does not alter production or establish canonical semantic identity. `docs/specs/universe.md`, production native mappings, and the active prelude still refer to the legacy owner.

## Validation

- The `Number`, `Int`, and `Float` declaration bodies match the pre-migration shadow source byte-for-byte. A source scan finds exactly one declaration for each name; the current module map and identity ledger agree on all 364 declaration rows.
- The explicit import graph records 214 current edges, compared with 208 at the S0/S1 baseline: one edge was removed and seven added by the scalar split.
- Native/prelude source inspection confirms `UniverseKey::Int` and `UniverseKey::Float` still map to `scalar.number`, and both remain metadata-exported and prelude-enabled. The class relation metadata remains `Int <: Number` and `Float <: Number`.
- `git diff --check` passes for the scalar source and migration documents.
- **Validation passed:** Source ownership, declaration-body preservation, module-map consistency, and dependency-graph accounting.
- **Validation failed:** None observed.
- **Validation blocked:** Shadow semantic, native-anchor, prelude-binding, and runtime initialization validation is unavailable because no isolated shadow compiler/provider is established. No tests or compiler checks were run.

## Known compatibility and integration risks

- The normative spec and production metadata continue to assign `Int` and `Float` to `universe.scalar.number`; the shadow path is a migration target, not a verified canonical compiler identity.
- Source uses of the names remain mostly unqualified prelude lookups. A future isolated shadow catalog/prelude must bind those names to the shadow declarations before semantic validation.
- The scalar facade now imports the three declarations. The source graph records these dependencies, but shadow linker/runtime initialization behavior has not been verified.
- Production authority, native anchors, runtime class slots, bootstrap, and prelude were not changed.

## Next increment

Migrate the foundational `Option`, `Result`, and `Unit` declarations into their intended focused shadow modules, preserving one source owner per declaration and recording the still-legacy production/native/prelude mappings.

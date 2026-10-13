# Universe shadow migration status

## Current increment

- **Increment:** Phase S4 iteration source relocation and isolated source-interface validation.
- **Branch:** `feat/telos-collections`.
- **S4 starting revision:** `a1621a7cec577952612685508a4c87c90aea86cd`.
- **Commits so far:** `74b84dda` (active/target identity clarification), `1404e08f` (iteration source move and validator), `b8083fb0` (validator scope note), and `a4a3bab9` (unique declaration-owner assertions); the S4 status and incident records are in the current documentation change.
- **Legacy baseline:** `9be1ed1eba3f94ba05e8e4aca1ee55f5ccf13131`; the S4-start shadow body matches that baseline digest.
- **Perf branch observed for merge constraints:** `perf/universe-characterization` at `c0155584972ec8c9f0960e411a9501a9c003de92`. No integration or cherry-pick is proposed in S4.
- **Scope:** Move all iteration protocols and adapters together into the accurately named shadow module `collections/iteration/package.ph`, retarget every shadow import, expose `.iteration`, and add a test-only parse/interface validator. Record the proposed fine-grained split as blocked by current source-import runtime semantics.
- **Source files moved:** `src/collections/iterator_laws.ph` → `src/collections/iteration/package.ph`.
- **Source files removed:** `src/collections/iterator_laws.ph` in the shadow tree only.
- **Source imports retargeted:** 23 existing dependencies from `universe.collections.iterator_laws` to `universe.collections.iteration`, spanning collection, scalar/string, text, functional, and math modules.
- **Facade update:** `collections/package.ph` exposes `.iteration`. It does not re-export protocol or adapter names at the parent facade yet: the direct module exports all sixteen original names, while parent-level re-exports remain deferred until their initialization effect is validated. No `iterator_laws` name remains in shadow `.ph` source.
- **Test infrastructure:** `phalcom-modules/tests/universe_shadow_sources.rs` parses every shadow `.ph` source, constructs unlinked interfaces using `ModuleId::universe`, checks path collisions, Universe import targets and selected exports, exposed child sources, and hierarchical visibility of the migrated iteration path. It is test-only and does not register the shadow package in production.
- **Legacy and production changes:** None. No legacy `.ph` source, Universe source provider, native metadata, prelude construction, compiler bootstrap, VM, or runtime registration was changed.
- **Identity status:** The target identities for `Option`, `Result`, and `Unit` were amended in `docs/specs/universe.md` to match the approved shadow paths. For iteration, the current cohesive shadow source path is `universe.collections.iteration`; the proposed canonical split owner `universe.collections.iteration.protocols` remains deferred and unactivated. `functional.chain::ChainIterator<T>` remains a separate declaration from the generic iteration adapter `ChainIterator<I, J>`.

## Declaration and dependency inventory

- The original source contains two protocols, one blanket `Iterable` implementation, and fourteen adapter classes, all retained in the moved module and explicitly exported.
- The moved file is 909 lines. Its SHA-256 is `579442979fd801ae1c6f012532dfa5fb7cc794657f0b833bc382c5345df2d11e`, equal to `git show a1621a7:packages/universe/src/collections/iterator_laws.ph` and the initial-copy manifest digest. This verifies exact byte preservation of all declaration bodies and method implementations.
- The current source scan still recognizes 364 top-level type-like declarations and 77 top-level values. The authored import graph remains at 216 distinct edges; S4 retargets 23 existing edges without changing the count.
- Method selectors, signatures, generic bounds, adapter ownership, and related Bytes/String iterator responsibilities are recorded in [`iteration-inventory.md`](iteration-inventory.md).
- The dependency decision and its linker evidence are recorded in [`iteration-split-incident.md`](iteration-split-incident.md). The proposed child split would create a runtime SCC: protocol defaults construct adapters, while adapters conform to the protocols. This is a predicted SCC from source references and linker rules, not a linked shadow graph result.

## Validation

- **PASS:** `RUSTFLAGS='' cargo test -p phalcom-modules --test universe_shadow_sources -- --nocapture` — 1 test passed, including unique-owner assertions for all moved declarations. `ChainIterator` is intentionally expected in both `collections.iteration` and `functional.chain`, because those are distinct declarations. The validator’s visibility check covers the migrated iteration path. A broader exploratory visibility check exposed existing inaccessible-import paths in unrelated `universe.numbers.capabilities`, `universe.math.float_kernel`, and `universe.random.generator`; those baseline findings were not changed.
- **PASS:** Moved source hash equals the S4-start and initial-copy digest.
- **PASS:** `git diff --check` on the final scoped source and documentation changes.
- **BLOCKED:** Canonical semantic identity and generic-conformance validation for the shadow package; no isolated canonical shadow compiler/provider is connected. Building ordinary filesystem `ProjectUniverse` would not prove builtin `ProjectIdentity::Universe` identity.
- **NOT RUN:** Shadow linked runtime graph for the hypothetical child split, native association checks, VM initialization order, runtime parity, broad workspace tests, or production cutover. These are outside the safe S4 source slice.

## Compatibility and integration status

- The sixteen authored declarations remain exported from `universe.collections.iteration`; source import sites now name that module. The old shadow path is removed, while the legacy path remains production-authoritative and intact.
- `collections/package.ph` exposes `.iteration`, allowing the new module path through the existing collection package hierarchy. Parent-level binding re-exports are deferred to avoid creating an unvalidated eager initialization dependency through the package facade.
- `docs/specs/universe.md` separates active production owners from target shadow owners. The production compiler and VM still use the legacy source tree, native ownership, and prelude mappings; the normative target updates do not imply production implementation alignment.
- The shadow `Bytes`, `String`, and functional `ChainIterator` declarations remain in their current modules and import/use the moved protocols as consumers. They are not part of the generic-adapter relocation.
- No production source provider, native binding, prelude, runtime, compiler, or bootstrap behavior was changed or validated. **Production cutover was not performed.**

## Previous increment: S3 foundational types

Phase S3 moved `Option` to `option.ph`, `Result` to `result.ph`, and `Unit` to `scalar/unit.ph`, following the earlier `Int` and `Float` split into `scalar/int.ph` and `scalar/float.ph`; `Number` remains in `scalar/number.ph`. Their moved bodies were checked byte-for-byte, and the normative target owners were updated. Production source/native/prelude/runtime mappings remain legacy. The S3 checks were focused source checks; no compiler/runtime identity validation was performed.

## Next action

Keep the cohesive `iteration` module as the S4 source-delivery result. A future compiler/module-system plan should establish and test interface-only source dependencies before reopening the six-child split. Separately plan focused semantic identity and consumer checks for `Bytes`, `String`, and the functional chain abstraction; do not move or redesign those domain iterators as part of this increment.

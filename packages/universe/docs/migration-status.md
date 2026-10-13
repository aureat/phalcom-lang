# Universe shadow migration status

## Current increment

- **Increment:** Phase S5 system/concurrency source ownership separation (independent declarations slice).
- **Branch:** `feat/telos-collections`.
- **Starting revision:** `da436a6c6b02c98f237a810b2e39f8d4fdc4cf58` (`origin/feat/telos-collections` matched at start).
- **Delivery commits:** `44518300` (shadow source moves) and `08f771c4` (normative target and migration inventories).
- **Scope:** Move `Backoff` to `concurrency/backoff.ph`, `CompletionSource` to `concurrency/completion_source.ph`, `Tracer` to `diagnostics/tracer.ph`, and `OffBehavior` to `features/off_behavior.ph`; expose the new child packages, remove the empty redundant shadow `concurrent` package, and update ownership inventories. Preserve all declaration bodies and production/legacy ownership.
- **Ownership review:** `Tracer` remains the `@traced` sink/observability helper; `OffBehavior` remains the disabled-policy value for `@featureFlag`. The shadow package has no use sites requiring either declaration to stay in `concurrency.fiber`. The active production fixture `phalcom-core/tests/fixtures/language/decorators/decorators_stdlib_helper_classes.ph` exercises bare `Tracer`, `OffBehavior`, and `Backoff` names; it remains untouched with the legacy production source. Any later cutover must review those imports/prelude bindings explicitly.
- **Compatibility:** No shadow source imports `universe.concurrent`; the removed module contained only package documentation. The shadow root no longer exposes that path. The active production root continues exposing the legacy `.concurrent` package; external use of the unregistered shadow alias has not been evaluated.
- **Completion:** `PARTIAL` for S5. The source-only relocations for Backoff, CompletionSource, Tracer, and OffBehavior are implemented. System, Future, and FutureSubscription remain co-located with the already correctly owned Fiber in `concurrency/fiber.ph` pending the phase-aware import decision.
- **Verification:** `UNVERIFIED`; all four moved class blocks were independently compared byte-for-byte with their original `fiber.ph` blocks: `Backoff` (2,323 bytes; SHA-256 `ea87840e2dfbf98af2eb7cf4994413f5f79ba38798b81fa87c669de293b3fa28`), `CompletionSource` (925 bytes; `f4b3b2bdbefefa6965d7e3db42b02c731c8bc2aabcba1f554bd1b7ffff9f4069`), `Tracer` (661 bytes; `78ab812b8a802585507958564af63e3f6fea761b8f9840a46780743b5ed08fb5`), and `OffBehavior` (797 bytes; `0e4abe2094a2d767de0872de4d636e081bca43a941f56c5477634c187850f9e9`). Declaration uniqueness, package exposure/import/export wiring, absence of shadow `.concurrent` references, and `git diff --check` were inspected. No parser/compiler tests were run.
- **Architecture hold:** The remaining `System`/`Future`/`FutureSubscription` split is a real source/runtime cycle: `FutureSubscription.deliver` calls `System.wake`; `Future` calls `System.sleep`, `schedule`, and `runScheduled`; and native `system_sleep` calls `VM::create_pending_future`, which resolves the prelude `Future` class and constructs it. Evidence is in `concurrency/fiber.ph`, `phalcom-core/src/primitive/system.rs::system_sleep`, and `phalcom-core/src/vm/mod.rs::VM::create_pending_future`. `phalcom-native-meta/src/universe.rs` still maps `UniverseKey::System` and `UniverseKey::Fiber` to `concurrency/fiber` and marks both as prelude bindings. The phase-aware import incident must be resolved before changing those module boundaries or introducing a compiler/module-system integration; production mappings remain untouched.
- **Next action:** Obtain the P4 architecture decision, then amend the split plan and its source-to-runtime gates before separating System, Future, or FutureSubscription. CompletionSource has a one-way explicit import of Future, so that extraction does not introduce a source-level dependency cycle. Production source authority and strict runtime-cycle behavior remain unchanged.

## Completed increment: S4 iteration source relocation

- **Increment:** Phase S4 iteration source relocation and isolated source-interface validation.
- **Branch:** `feat/telos-collections`.
- **S4 starting revision:** `a1621a7cec577952612685508a4c87c90aea86cd`.
- **Commits so far:** `74b84dda` (active/target identity clarification), `1404e08f` (iteration source move and validator), `b8083fb0` (validator scope note), `a4a3bab9` (unique declaration-owner assertions), and `ae357861` (inventory, incident, and migration records); this update adds the final lifecycle classification.
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
- **Completion:** `IMPLEMENTED` for the bounded cohesive source-delivery slice; the proposed six-child protocol/adapter decomposition remains `PARTIAL/BLOCKED`. S4 is not semantically or runtime integrated.
- **Verification:** `FOCUSED_TESTED` for the unlinked source-interface validator, source-body digest, ownership inventory, and diff hygiene recorded below. Production cutover and semantic/runtime acceptance remain unverified.

## Declaration and dependency inventory

- The original source contains two protocols, one blanket `Iterable` implementation, and fourteen adapter classes, all retained in the moved module and explicitly exported.
- The moved file is 909 lines. Its SHA-256 is `579442979fd801ae1c6f012532dfa5fb7cc794657f0b833bc382c5345df2d11e`, equal to `git show a1621a7:packages/universe/src/collections/iterator_laws.ph` and the initial-copy manifest digest. This verifies exact byte preservation of all declaration bodies and method implementations.
- At S4 completion, the source scan recognized 364 top-level type-like declarations and 77 top-level values, and the authored import graph had 216 distinct edges; S4 retargeted 23 existing edges without changing that count. The current S5 graph count is recorded in `dependency-graph.md`.
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

Keep the cohesive `iteration` module as the S4 source-delivery result. The proposed compiler/module-system prerequisite is recorded as `MODL001.C2.P4`, but it is blocked before implementation pending the decisions in the [architecture incident](../../../docs/implementation/MODL001-module-architecture/C2-module-project-model/MODL001.C2.P4-architecture-incident.md). Do not reopen the six-child split until that design is resolved and its focused source-to-runtime path is tested. Separately plan focused semantic identity and consumer checks for `Bytes`, `String`, and the functional chain abstraction; do not move or redesign those domain iterators as part of this increment.

# Canonical Universe cycle — requirement provenance audit (2026-10-14)

Status: BLOCKED / SOURCE-INSPECTED / NOT RUNTIME-VERIFIED. Branch `feat/consolidation-universe`.

## Verified source contracts

1. `phalcom-semantic/src/phase_evidence.rs` emits `RuntimeDependencyRequirement` with either `read_target: Some(LinkedReadSpec)` or `read_target: None`. For example `push_declaration_read_requirement` emits a binding read; `push_callable_owner_requirement` emits an owner-only requirement. Neither carries a proof of native-floor availability.
2. `phalcom-core/src/modules/compile.rs::runtime_metadata_type_requirements` traverses metadata type roots and emits owner-only requirements with `read_target: None`. This is a *graph-only* projection by design.
3. `phalcom-modules/src/linker.rs::finalize_executable_with_requirements` adds a runtime dependency edge for **every** requirement, regardless of whether a read target exists, then performs strict SCC/DAG validation. The absence of a read target is not currently a reason to remove an initialization edge.
4. `phalcom-core/src/modules/canonical_universe.rs` supplies both semantic-generated requirements and metadata owner requirements into that same finalizer.
5. `phalcom-core/src/vm/bootstrap.rs` installs the native floor before it executes canonical Universe source initializers. Source-only classes are reserved ahead of source execution, but reserved slots do **not** prove that the corresponding value has been initialized.

## Important distinction

The current cycle report lists modules in an SCC. It does **not** identify a particular erroneous edge. The type-owner and generated-read projections are strong places to investigate, but no evidence yet proves that any specific edge is false. In particular, `read_target: None` does **not** imply a dependency is safe to discard: it may represent an actual runtime declaration or callable-owner requirement.

Avoid these invalid shortcuts:

- Do not drop every owner-only requirement or every edge to Universe.
- Do not admit SCCs in the executable runtime graph.
- Do not assume a preallocated module slot is an initialized declaration value.
- Do not add compiler-side late linked reads.
- Do not change the native/source ownership of a declaration by its name or capitalization.

## Required next implementation slice

1. Introduce diagnostic-only per-edge provenance for the *specific rejected SCC*, retaining `importer`, `dependency`, `range`, `reason`, and `read_target` (if present). Distinguish authored import, generated callable/pattern read, and metadata owner projection **at the producer**, not by guessing from `read_target`.
2. Run the exact reproducer in the consolidation worktree:
   `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_linking::mat_01_export_slots_populated -- --exact --nocapture`.
   Capture a minimal actual cycle with the originating source spans and requirements.
3. For each edge in that cycle, prove whether the target is (a) an initialized native-floor value, (b) a source-initializer-owned value, or (c) interface/metadata identity requiring no initialized runtime value. A preallocated uninitialized slot is not (a).
4. Only after a proof exists, encode the classification in a typed, modules-owned contract. Keep read allocation orthogonal to initialization ordering. Do not use a blanket project-based exemption.
5. Preserve and test the additional fail-closed canonical order check introduced by `5aeeffa8`/`e369d7dc`, including a DAG where a generated edge reverses the source-index order.
6. Re-run linkage, semantic phase evidence, canonical singleton/bootstrap, and runtime integration tests, then collections regression suites.

## Acceptance gate

No claim of full consolidation or fixed bootstrap until the strict canonical executable graph finalizes, the runtime discriminator passes, all emitted linked reads resolve to finalized slots, and canonical execution order respects every source-initialization dependency. Enum inference and PERF003 caching remain excluded.

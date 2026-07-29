# Phase 01 Compatibility Boundary

> **Release note:** This document records the Phase 01 transition boundary. Phase 12 removed `src/_internal/legacy_adapter.ph`, `src/_internal/phase01_surface.ph`, the historical monolith, and the migration generator. The 0.1.0 façade now delegates directly to authoritative modules; supported broad-v1 names are direct aliases only.

`src/_internal/legacy_adapter.ph` is a temporary implementation boundary, not the target architecture. It exists to keep the inherited broad-v1 behavior available while the package is decomposed phase by phase.

The adapter has been mechanically and manually migrated to current syntax:

- constructor methods use `@constructor`;
- control-flow conditions omit mandatory outer parentheses;
- mechanical counters use compound updates;
- public compatibility declarations carry reflective type annotations;
- implementation-only classes use the `_Legacy` prefix.

The root façade is the compatibility contract. Later phases may replace adapter internals without changing imports introduced by this checkpoint.

`WithSettings` delegates to the working compatibility `Check` attribute, and `StateInvariant` delegates to the working compatibility invariant attribute. Later-phase names that cannot work yet—such as `DirectoryDatabase`, `JsonReporter`, `Bundle`, `When`, and `Teardown`—fail explicitly when constructed instead of behaving as silent placeholders.

The compatibility engine's observation methods (`note`, `event`, `classify`, and `target`) also fail explicitly until the reporting and targeting phases can make their effects observable. Property-context cleanup uses `ensure`, so exceptions cannot leak the active context stack.

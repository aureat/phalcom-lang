---
id: LANG005.C5.P1
category: LANG
program: LANG005
checkpoint: LANG005.C5
kind: implementation-handoff
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
repository: aureat/phalcom-lang
repository_head: e5152196d716ed70fc47d104a0fd789e602285db
next_plan: LANG005.C5.P2
---

# LANG005.C5.P1 — Handoff to P2

## Current state

P1 is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`. Work remains in the shared
dirty checkout on `main`; no commit or push was requested or performed. The
C5 checkpoint remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`.

## Completed prerequisites

P1 provides associated declaration/binding foundations, trait-property
elaboration, direct-field `via` behavior, exact conformance evidence,
incremental lifecycle products, source-index identity, and ordinary executable
accessors. The affected crates compile. Focused semantic, AST, incremental, and
core tests are green for the new P1 behavior.

## Frozen takeover interfaces

P2 must inherit these exact products:

1. `AssociatedTypeRequirementId { owner: DeclarationId, index: u32 }` is the
   canonical trait-owned associated identity. Do not key projections by name or
   snapshot-local `TypeId`.
2. `TraitSurface.associated_types` is the separate associated requirement table;
   associated types are not callable `TraitRequirementId`s.
3. `AssociatedTypeBindingTemplate` is owned by one `ImplId` and stores the exact
   requirement, unspecialized impl-scope value template, source implementation,
   and source span.
4. `ConformanceAssociatedTypePlan` owns binding resolution, failures,
   diagnostics, and fingerprint for that implementation.
5. `ConformanceEvidence.associated_types` stores exact specialized bindings
   keyed by `AssociatedTypeRequirementId`.
6. `ConformanceFailure::AssociatedType` participates in the single existing
   `ConformanceCompleteness` proof state alongside behavioral failures.
7. Source-index associated declaration/binding targets use
   `SemanticTargetId::AssociatedType(AssociatedTypeRequirementId)`.

## Invariants not to redesign

- Trait properties are ordinary getter/setter behavior, never storage.
- `via` is direct-field accessor elaboration, not a runtime delegation protocol.
- Field type, mutability, private ownership, and non-inheritance remain
  authoritative; P1 does not search superclass fields.
- Generated accessors are ordinary callables and obey ordinary selector
  conflicts.
- Conformance-local `via` remains rejected; conformance scope does not gain
  class-private field authority.
- Compiler/runtime consume accepted semantic lowering; they do not solve
  conformance or associated types at runtime.
- Binding resolution is trait-surface-owned and conformance-plan-owned; P2 must
  not add a second syntax resolver or identity registry.

## Known drift and deferred failures

The plan's `phalcom-ast --test impl_syntax` target is absent in the live
manifest; use the existing integration binary/module selector. The planned
`traits_c5_p1` core module is also represented by focused tests in
`traits_p3_closure.rs`.

`C5-BL-07` remains an inherited baseline D: two full `capabilities::traits`
tests fail on Universe `Bool` declaration dependencies/capabilities. Do not
repair those as part of P2 unless a new C5-specific reproducer proves causal
coupling.

## Must-read files

- `docs/implementation/LANG005/LANG005.C5/LANG005.C5-CHECKPOINT.md`
- `docs/implementation/LANG005/LANG005.C5/LANG005.C5-GUIDANCE.md`
- `docs/specs/objects/traits.md`
- `phalcom-semantic/src/traits.rs`
- `phalcom-semantic/src/impls.rs`
- `phalcom-semantic/src/identity.rs`
- `phalcom-semantic/src/source_index/`
- `phalcom-core/src/modules/semantic_lowering.rs`
- `phalcom-core/src/compiler/lib/class_decl.rs`
- `phalcom-core/src/compiler/lib/impl_decl.rs`

## P2 objective and first checks

P2 owns associated projection formation and normalization, beginning with
`Self::Item` in the ratified contextual surfaces. It must use the frozen
trait-owned requirement and canonical conformance evidence products, preserve
ambiguous/cyclic/blocked terminal states, and stop before C6 generic trait-bound
proof machinery.

Recommended first checks:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic -p phalcom-core
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::traits
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::associated_types
```

## Explicit do-not-redesign guidance

Do not reopen P1 identity, completeness, property/via elaboration, field
ownership, or runtime boundaries while adding projection. Do not implement
generic `T: Trait`, conditional conformance, GATs, associated defaults, trait
objects, or runtime associated-type lookup in P2.

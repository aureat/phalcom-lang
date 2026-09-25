---
id: LANG007.C3.P3.R1
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: corrective-implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - LANG007.C3.P3
follows: LANG007.C3.P3
repository: aureat/phalcom-lang
prepared: 2026-09-25
title: "Intrinsic Semantic Application and Runtime Owner Corrective Closure"
---

# LANG007.C3.P3.R1 — Intrinsic Semantic Application and Runtime Owner

## Objective

Implement the user-approved corrective ruling in the attached task brief:
represent intrinsic structural type application independently from the
existing VM/native owner on the canonical declaration binding. Preserve the
existing source syntax, semantic owners, VM classes, RecordRow identity,
TupleFromPacks/CompleteTypePack authority, and AssociatedSurface model.

This plan adds no source syntax, first-class TypeFormer, generalized
higher-kinded machinery, or new type/row/member registry. It does not change
ordinary type-lambda behavior.

## Fixed architecture

- A declaration retains its ordinary/canonical type identity, explicit
  intrinsic application behavior, and optional runtime/native owner as sibling
  projections. The runtime-owner registration augments the binding; it does
  not replace the semantic application behavior.
- Ordinary declarations continue through the existing `TypeLambdaArena`
  `TypeId` path.
- Tuple application consumes a `CompleteTypePack`, projects the existing
  positional and labeled lanes, and uses `TypeStore::tuple_from_packs`.
- Record application consumes `RecordRowId` directly and uses the existing
  `TypeData::Record` authority. No row is encoded as `TypeId` or passed through
  `TypeLambdaArena`.
- Unit remains the canonical `store.unit()` semantic type and its VM class
  remains the existing runtime owner.
- Native method import asks for the runtime projection; semantic type
  formation asks for the declaration's semantic application behavior.
- `AssociatedSurface` and `TypeMemberDeclarationId` remain unchanged.

## Tasks and gates

### T1 — Make the declaration binding carry both projections

Extend the existing declaration type info/binding with an explicit semantic
application discriminator for Unit, Tuple, and Record, while retaining the
existing TypeId/source identity and optional runtime owner. Prefer the current
`DeclarationTypeInfo` owner so incremental fingerprints cover this fact; do
not introduce a second owner registry. `register_intrinsic_runtime_owner`
must only add/update the runtime projection.

**G1:** declaration-table tests prove canonical Unit/Tuple/Record bindings
retain both semantic application and runtime-owner projections.

### T2 — Split application and native-import use sites

Semantic annotation formation dispatches from the explicit semantic
application form before ordinary beta reduction. Tuple and Record consume the
existing pack/row identities. Native surface import continues to request the
runtime owner/class projection. Do not add RecordRow handling to
`TypeLambdaArena`.

**G2:** tests prove Tuple canonicalizes to `TupleFromPacks` using the original
`CompleteTypePack` lanes; Record yields `TypeData::Record` with its original
row identity; ordinary lambda application remains unchanged; runtime import
resolves native Tuple members to the existing VM class.

### T3 — Prove member and runtime identity preservation

Retain the existing `AssociatedSurface`, source-index, and Unit reuse paths.
Add or strengthen focused tests for the one associated namespace, Tuple
`TupleEntries` specialization identity, native Tuple runtime owner reuse, and
Unit runtime class identity.

**G3:** focused semantic and core tests pass with no extra registry, runtime
class, TypeId-for-row, or generic DataObject fallback.

### T4 — Corrective closure

Update the checkpoint, P3 handoff/walkthrough linkage, program status, and this
plan's terminal records truthfully. Preserve previously classified unrelated
baselines. Do not claim release certification.

**G4:** focused required tests pass, code and lifecycle diffs are reviewed,
and C3 returns to `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` only after both
projections and all fixed invariants are evidenced.

## Verification

Baseline before edits:

- `cargo test -p phalcom-semantic --test semantic semantic::foundations::declarations`
  — 7 passed.
- `cargo test -p phalcom-semantic --test semantic semantic::foundations::type_annotations`
  — 51 passed.

Focused verification after implementation:

- the same semantic declaration and type-annotation filters;
- semantic associated/type-member closure tests covering `TupleEntries`;
- the core declaration dispatch / native Tuple owner tests;
- exact new regressions for both declaration projections and ordinary lambda
  preservation.

Run Rust validation serially with `RUSTFLAGS='' RUSTC_WRAPPER=''`. Broader P3
baseline suites and release gates are not part of this corrective plan.

## Completion evidence required

Walkthrough and handoff must identify changed files, the canonical binding
model, each projection's consumers, tests actually run, previous baseline
classifications, and the final C3/P4 lifecycle state. No P5 is implied.

## Execution record

Status: `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.

The declaration binding now stores `DeclarationSemanticForm` and optional
`IntrinsicRuntimeOwner` together on `DeclarationTypeInfo`. Ordinary shell
refresh preserves registered projections. Semantic application reads the
semantic discriminator; runtime data lowering, native type import, and
dispatch read the runtime owner projection. Unit, Tuple, and Record preserve
their existing canonical representations and identities.

Focused gates passed:

- `cargo test -p phalcom-semantic --test semantic semantic::foundations::declarations` — 8 passed.
- `cargo test -p phalcom-semantic --test semantic semantic::foundations::type_annotations` — 51 passed.
- `cargo test -p phalcom-semantic --lib intrinsic_tuple_native_import_uses_runtime_owner_projection` — 1 passed.
- `cargo test -p phalcom-semantic --test semantic semantic::incremental::fingerprints` — 34 passed.
- `cargo test -p phalcom-semantic --test semantic semantic::integration::type_member_closure` — 10 passed.
- `cargo test -p phalcom-semantic --test semantic semantic::incremental::type_members` — 24 passed.
- `cargo test -p phalcom-core --test core intrinsic_unit_facade_reuses_the_preinstalled_runtime_class` — 1 passed.

The core Unit test initially exposed projection loss when an ordinary
declaration shell refreshed an existing intrinsic entry. `DeclarationTypeTable`
now preserves registered sibling projections through that refresh, with a
focused regression. A failed intermediate type-annotation run showed that its
runtime-owner-isolation fixtures must remove the old binding before inserting
a projection-isolated copy; after that fixture correction, all 51 tests pass.
No broad workspace or release gate was run. `cargo fmt --all -- --check`
remains non-green with the repository formatter drift already classified on
clean predecessors in C3; no broad reformat was applied. Existing unrelated
baselines remain as recorded in C3.

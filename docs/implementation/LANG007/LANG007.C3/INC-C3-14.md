# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: `LANG007`
- Checkpoint: `LANG007.C3`
- Plan: `LANG007.C3.P3`
- Task/Gate: T4 / G4
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Current implementation base: `0f16a1fa` plus the current T4 work unit

## 2. Escalation trigger

Mandatory triggers 1, 3, 5, 7, 11, and 12: the fixed P3 contract requires
structural applications for Tuple and Record while native semantic import also
needs the existing runtime class type. Record's row argument must retain its
separate `RecordRowId` authority across application.

## 3. Required invariants

- `intrinsic data Unit()`, `intrinsic data Tuple<***Es>`, and
  `intrinsic data Record<R: RecordRow>` bind to existing platform/runtime owners.
- Tuple generic application normalizes to existing `TupleFromPacks` and retains
  `CompleteTypePack` identity for `TupleEntries`.
- Record generic application normalizes to `TypeData::Record(RecordRowId)`;
  rows remain outside `TypeId` and the row solver remains authoritative.
- Native instance/class method signatures continue to use the existing runtime
  class identity and native descriptor owner.
- Source type members use owner-plus-selector `TypeMemberDeclarationId` and the
  one `AssociatedSurface` collision/lookup authority.

## 4. Expected architecture

P3 §7.2–7.8 requires the intrinsic source declaration, structural type form,
and runtime dispatch class to remain related projections of the existing
platform concept. The runtime binding and inherent-impl type-member bridge are
implemented. The missing piece is how declaration type application selects the
structural form while native type import selects the runtime class form.

## 5. Observed behavior

- Canonical `Unit` was migrated to `intrinsic data Unit()` with its Phalcom
  methods in `impl Unit`. Canonical Universe bootstrap succeeds and the compiler
  rebind test confirms it reuses the exact existing VM Unit class.
- Tuple and Record intrinsic type members were moved from class bodies into
  inherent impls. Their canonical owner-plus-selector products publish through
  the existing associated surface; parser and semantic focused tests pass.
- A bounded Tuple migration experiment changed the declaration to
  `intrinsic data Tuple<***Es>` and moved the native/Phalcom members into
  `impl<***Es> Tuple<***Es>`. Canonical bootstrap then failed while importing
  the native surface with `OwnerMissing { owner: Tuple, selector: rawAt(_) }`.
- `register_intrinsic_runtime_owner` currently removes the declaration type
  entry, while native method import requires `DeclarationTypeTable::form(Tuple)`
  to be a proper runtime class type. The attempted Tuple source change was
  reverted; canonical source currently retains `native class Tuple is
  Collection` and the new inherent-impl type-member block.
- The row-owned Record argument cannot be passed through the existing type-lambda
  beta path as a `TypeId`: `TypeLambdaArena::beta_reduce` accepts `TypeId`
  arguments and bound record tails currently produce
  `UnsupportedRecordRowArgument`.

## 6. Minimal reproduction

```phalcom
intrinsic data Tuple<***Es>
impl<***Es> Tuple<***Es> {
  internal native rawAt(_ index: Int) -> Dynamic
}
```

This parses and the source-index fixture can associate the impl with
`UniverseKey::Tuple`. Replacing the canonical Tuple source declaration with
this facade fails semantic bootstrap before compiler certification because the
runtime class form is absent from the declaration table.

## 7. Relevant code path

```text
phalcom-semantic/src/declarations.rs::bootstrap_universe_declarations
phalcom-semantic/src/declarations.rs::DeclarationTypeTable::register_intrinsic_runtime_owner
phalcom-semantic/src/types/native.rs::register_native_surfaces
phalcom-semantic/src/types/type_lambda.rs::TypeLambdaArena::beta_reduce
phalcom-semantic/src/types/store.rs::TypeStore::apply_static_form_with_arena
```

## 8. Implementer hypothesis

The runtime facade binding cannot be the only projection consumed by semantic
type formation. Tuple needs a structural generic application form and a runtime
class form for native method signatures. Record also needs row-owned application
to reach `TypeData::Record(RecordRowId)` without representing the row as a
`TypeId`. A specialized intrinsic-form application path may preserve both
existing authorities without adding a second type or row registry; its exact
owner and API need to be settled before coding.

## 9. Decision requested

Should T4 add a canonical intrinsic-form application path keyed by the existing
runtime-owner binding, mapping Tuple's complete-pack argument to the existing
`TupleFromPacks` constructor and Record's row argument directly through the
existing row-owned `RecordRowId` path, while native signature import continues
to use the existing runtime class form?

This is the recommended direction because it preserves the P3 source endpoint
and the TypeStore/row/runtime ownership boundaries without creating a new static
domain or runtime class.

## 10. Evidence

- `phalcom-ast` intrinsic impl type-member parser test: 1 passed.
- `phalcom-semantic` canonical Tuple intrinsic impl member publication: 1 passed.
- `phalcom-semantic` runtime structural declaration owner binding and Unit form:
  1 passed.
- `phalcom-core` canonical Universe bootstrap plus Unit existing-class reuse:
  1 passed after the Tuple migration experiment was reverted.
- Tuple data-facade migration experiment: canonical bootstrap failed with the
  `Tuple#rawAt(_)` owner-missing error above.

## 11. Adopted decision and status

ADOPTED by the user on 2026-09-24. T4 will add the intrinsic structural
application path using the existing `IntrinsicRuntimeOwner` association:

- Tuple application consumes the existing canonical `CompleteTypePack`,
  projects its two existing lanes, and returns `TupleFromPacks`.
- Record application consumes an existing `RecordRowId` directly and returns
  `TypeData::Record(row)`. It must not convert a row into a `TypeId` argument or
  extend `TypeLambdaArena`'s TypeId-only beta contract.
- Native signature import uses the runtime class projection of the same
  declaration, while source type formation uses its structural application.
- Unit remains canonical `store.unit()` and keeps its existing runtime class.
- Tuple/Record intrinsic type members are declared in inherent `impl` blocks
  and continue through owner-plus-selector `TypeMemberDeclarationId` and the
  existing `AssociatedSurface` authority. No parallel member registry is added.

The source syntax mismatch is resolved by accepting intrinsic type-member
declarations in inherent impls. T4/G4 remains in progress until Tuple and Record
source migration and the focused runtime/native/structural regressions pass.
This decision does not certify P3 or C3.

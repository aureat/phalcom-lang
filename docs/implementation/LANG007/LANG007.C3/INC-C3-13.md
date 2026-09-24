# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: `LANG007`
- Checkpoint: `LANG007.C3`
- Plan: `LANG007.C3.P3`
- Task/Gate: T4 / G4
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Starting revision: `518f66d19941eb24e379e60f64aa89df66643a8a`
- Current revision: `0f16a1fa` plus uncommitted T4 binding/reuse and census work

## 2. Escalation trigger

Mandatory triggers 1, 2, 8, 12, and 18: the planned migration moves Tuple's
intrinsic type members into an inherent `impl`, but the live AST and semantic
surface do not support type-member declarations in `ImplMember`.

## 3. Required invariant

Preserve `LANG007.C3.P3` invariants: `Tuple::Entries` and existing Tuple type
members retain canonical `TypeMemberDeclarationId` identity and `AssociatedSurface`
collision authority; the source/runtime facade remains associated with the
existing structural Tuple type and VM class; no parallel type-member namespace
or nominal Tuple type is introduced.

## 4. Expected architecture

T4's required endpoint places Tuple's intrinsic type members in
`impl Tuple<***Es> { ... }` while preserving member identity. T4 also requires
native behavior anchors to remain associated with the canonical Universe owner.

## 5. Observed behavior

- `phalcom_ast::ast::ImplMember` has `Behavior`, `Delegation`, and
  `AssociatedTypeBinding` variants; it has no type-member declaration variant.
- `TypeMemberDef` is currently held by `ClassMember::TypeMember`.
- `NativeSourceIndex::index_impl` indexes behavior members only. Its current
  behavior is sufficient to retain native method anchors when those methods
  move to `impl`; it cannot transfer type-member declarations.
- Therefore changing Tuple/Record from `native class` to `intrinsic data` removes
  the only current syntax location for their intrinsic type members unless the
  language syntax and semantic associated-surface pipeline are extended.

## 6. Minimal reproduction

AST definitions:

```text
phalcom-ast/src/ast.rs::ImplMember
    Behavior | Delegation | AssociatedTypeBinding
phalcom-ast/src/ast.rs::ClassMember
    includes TypeMember(TypeMemberDef)
```

The proposed T4 form `impl Tuple<***Es> { intrinsic type Entries: TupleEntries }`
cannot currently be represented as an `ImplMember`.

## 7. Relevant code path

```text
phalcom-ast/src/ast.rs::ImplMember
phalcom-ast/src/ast.rs::ClassMember::TypeMember
phalcom-semantic/src/session.rs::query_associated_surface publication
phalcom-core/src/native/source.rs::NativeSourceIndex::index_impl
```

## 8. Scoped implementation diff

The runtime facade binding/reuse path and canonical source-index support have
been implemented locally and focused-tested. No Unit/Tuple/Record source file
has been migrated. The proposed change to `ImplMember` or associated-surface
ownership has not been attempted.

## 9. Evidence

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic runtime_structural_declarations_bind_canonical_platform_owners` — 1 passed.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core intrinsic_unit_facade_reuses_the_preinstalled_runtime_class` — 1 passed. The test confirms no `Bytecode::Data` / `FinalizeData` and that the Unit global retains the exact preinstalled class value.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib intrinsic_runtime_facade_anchors_existing_native_class_and_impl_members` — 1 passed. The test confirms a runtime intrinsic Tuple declaration and an impl-native member map to the canonical Universe owner/relation.

## 10. Implementer's causal hypothesis

The T4 source endpoint assumes an impl-owned intrinsic type-member surface that
was not added by the preceding C1/C2/C3 semantic work. Reusing
`AssociatedTypeBinding` would alter its meaning and lose the declared generic
type-member contract, so this is not a mechanical representation change.

## 11. Adopted decision

The user chose to extend `ImplMember` and the associated-surface/type-member
pipeline for intrinsic type-member declarations in inherent impls, preserving
the existing owner-plus-selector `TypeMemberDeclarationId` and the single
`AssociatedSurface` authority. Keep this implementation within T4/G4; no
corrective plan is needed. The separate class-source-anchor alternative is
rejected.

Plan amendment: T4 now includes parser/AST representation and semantic/source
index publication for impl-owned intrinsic type members before Tuple/Record
source migration.

Status: ADOPTED — T4/G4 implementation resumed.

Parser-backed inherent-impl syntax and canonical Tuple associated-surface
publication are now focused-tested; see INC-C3-14 for the separate intrinsic
Tuple/Record data-application boundary discovered during source migration.

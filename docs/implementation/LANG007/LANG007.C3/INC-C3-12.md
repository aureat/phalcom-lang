# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: `LANG007`
- Checkpoint: `LANG007.C3`
- Plan: `LANG007.C3.P3`
- Task/Gate: T4 / G4
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Starting revision: `518f66d19941eb24e379e60f64aa89df66643a8a`
- Current revision: `518f66d19941eb24e379e60f64aa89df66643a8a` (working tree contains P3 T1–T3 work)

## 2. Escalation trigger

Mandatory triggers 1, 2, 7, and 12: T4's required runtime-facade binding premise is not established by T2/G2 and T3/G3, and implementing it affects semantic authority, structural identity, and compiler/runtime dispatch.

## 3. Required invariant

Preserve P3 `INV-12`–`INV-16` and `INV-30`: Unit has one canonical identity; Tuple and Record remain structural and retain their specialized runtime representations; intrinsic source declarations do not route through generic `DataObject` storage; moved members retain identity; native/runtime census remains exact.

## 4. Expected architecture

The P3 opening and §5.4 require a declaration-identity binding from intrinsic runtime source facades to existing compiler/runtime authorities before source migration. T4 additionally requires a compiler dispatch path that reuses those authorities and attaches behavior without ordinary generic data lowering.

## 5. Observed behavior

- `DeclarationTypeTable` currently has intrinsic static-kind bindings and an `intrinsic_type_owners` map. The latter is populated for existing `Class` declarations named Tuple/Record and for static facade owners; there is no registration of runtime intrinsic `Data` facades for Unit/Tuple/Record, nor a canonical Unit binding.
- `Compiler::compile_statement` dispatches every non-TypeOnly, non-sum `Statement::Data` to `compile_data`.
- `compile_data` requires a semantic `DataDeclarationLoweringSpec`, emits `Bytecode::Data` and `Bytecode::FinalizeData`, and defines the resulting data class global. This is the ordinary generic data path.
- `NativeSourceIndex` recognizes runtime class presentations from `Statement::Class` and sum presentations from data declarations. It does not recognize intrinsic runtime product/facade declarations as existing `UniverseKey` runtime class presentations.
- Existing TypeStore/runtime structural authorities are already present: Unit is canonicalized with the empty tuple; Tuple and Record have specialized runtime classes and explicit `UNIVERSE_CLASS_RELATIONS`.

## 6. Minimal reproduction

No failing test was run: no T4 source migration was attempted. The blocking path is directly visible in the live compiler dispatch:

```text
Compiler::compile_statement(Statement::Data)
  -> compile_data
  -> DataDeclarationLoweringSpec lookup
  -> Bytecode::Data / Bytecode::FinalizeData
```

Expected for intrinsic Unit/Tuple/Record: bind and reuse their existing compiler/runtime identities, with no generic data class/storage creation.

## 7. Relevant code path

```text
phalcom-semantic/src/declarations.rs::bootstrap_universe_declarations
  -> registers static intrinsic facades; runtime Unit/Tuple/Record remain Classes
phalcom-semantic/src/session.rs
  -> skips only registered bodiless intrinsic static facades
phalcom-core/src/compiler/lib/mod.rs::compile_statement
  -> Statement::Data runtime product -> compile_data
phalcom-core/src/compiler/lib/data_decl.rs::compile_data
  -> generic Data lowering and class global definition
phalcom-core/src/native/source.rs::NativeSourceIndex::index_program
  -> Class / sum-only owner presentation indexing
```

## 8. Scoped implementation diff

No T4 source changes have been made. Existing P3 changes implement T1/G1, T2/G2, and T3/G3 as recorded in the checkpoint.

## 9. Evidence

Direct source inspection of the paths listed in §7 confirms the current dispatch and registration behavior. `docs/implementation/LANG007/LANG007.C3/LANG007.C3.P3-intrinsic-structural-surface-universe-migration-tooling-certification-and-c3-closure.md` says at §0.2 not to migrate Tuple/Record/Unit before runtime facade binding exists, at §5.4 that the binding maps to existing owners, and in T4's preconditions that the runtime registration path exists and is proven not to use ordinary generic data storage. T2/G2's recorded scope only established static facade registration.

## 10. Implementer's causal hypothesis

T4's binding-path prerequisite was not represented as a completed T2/G2 acceptance item. The currently registered Tuple/Record owners support existing type-member dispatch for nominal class declarations; they do not bind `intrinsic data` source declarations to structural type formation and existing runtime classes. Unit has no corresponding intrinsic owner registration.

## 11. Alternative hypotheses

- The intended meaning of “runtime facade registration path exists” may be satisfied by the existing class-owner map plus the runtime `UniverseKey` table, with T4 expected to add only the source declaration and compiler projection.
- The prerequisite may have been omitted from T2/G2 and should become an explicit first T4 implementation/gate before the three source migrations.

These alternatives differ in whether the T4 plan needs a Level 1 clarification or a material task/dependency amendment.

## 12. Attempts already made

None. No speculative compiler, semantic, or runtime architecture change was attempted.

## 13. Why local authority is insufficient

Choosing how a runtime intrinsic declaration attaches to an existing structural `TypeData` authority and pre-existing runtime class affects canonical declaration/type identity and compiler/runtime dispatch. The plan prohibits implementing such a choice as a local workaround, and the current plan's prerequisite says the binding should already exist.

## 14. Adopted decision

The user ratified the recommended option on 2026-09-24: amend T4 to establish
and test the declaration-identity binding and existing-owner compiler/runtime
reuse path first, then migrate the Unit/Tuple/Record source presentations.
Keep this work within T4/G4 as a Level 1 sequencing clarification; do not add
a corrective plan. Existing TypeStore and VM runtime authorities remain the
owners, while the intrinsic binding remains a source declaration association.

## 15. Required focused verification

- canonical declaration identity resolves to the existing Unit/Tuple/Record authorities;
- intrinsic runtime data dispatch never emits generic `Bytecode::Data` or creates a `DataObject`;
- moved native/member anchors preserve canonical owner and selector identity;
- runtime class relations and source/native census remain exact;
- existing Unit/Tuple/Record semantic and specialized-storage behavior remains unchanged.

Status: ADOPTED; implementer may resume T4/G4.

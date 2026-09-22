---
id: LANG007.C2.P1
kind: implementation-plan-handoff
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
base_revision: 1a1e5ab022c84ad979a28994a5a135121baaaeaa
implementation_revision: 1a1e5ab022c84ad979a28994a5a135121baaaeaa
checkpoint: LANG007.C2
program: LANG007
prepared: 2026-09-22
---

# LANG007.C2.P1 - P2 Handoff

## State

P1 is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`. The working tree contains P1 source, test, lifecycle, and supplied planning-artifact changes. C2 remains open; P2 is the next plan.

## Stable Takeover Map

- `KindId::COMPLETE_TYPE_PACK`, `KindData::CompleteTypePack`, and `TypeData::CompleteTypePack { positionals, labeled }` are the exact complete two-lane type domain.
- `TypeStore::{complete_type_pack, project_positionals, project_labeled, project_complete}` is the only lane projection/construction authority.
- `TypeData::{TypePackConcat, LabeledTypePackConcat}` and `TypeStore::{type_pack_concat, labeled_type_pack_concat}` are the only canonical composition authority.
- `GenericParameterMode` lives in `types::parameter`, is retained on `TypeParameterData`, and is slot-aligned in `GenericSignature::parameter_modes`.
- `TypeSubstitution::try_apply` is the checked substitution seam. `materialize_type` propagates pack formation invalidity through `TypeMaterializationError::Pack`.
- Metadata schema is v3. Stable forms are `KindNode::CompleteTypePack`, `TypeNode::{CompleteTypePack, TypePackConcat, LabeledTypePackConcat}`, and `GenericParameterModeRef` / `TypeParameterRecord.mode`.

## Invariants

- `CompleteTypePack(P, L)` is neither a Tuple nor a proper Type. Tuple formation still goes through `tuple_from_packs`.
- `*`, `**`, and `***` mean positional, labeled, and complete lanes. `...` is not pack syntax.
- Generic mode is not inferred from kind. `Single(TypePack)` and `PositionalPack(TypePack)` are distinct public interfaces.
- Concat is ordered and flat. Valid concrete results collapse into C1 `TypePack` / `LabeledTypePack`; concrete duplicate labels fail; symbolic overlap remains residual.
- No `PackId`, `PackVarId`, second generic binding store, RecordRow conversion, runtime pack object, VM opcode, or ABI work exists.

## P2 Mandatory Rules

1. Elaborate source `*` / `**` / `***` directly into P1 canonical terms. Do not create source-only pack identities.
2. Before source syntax can reach symbolic labeled composition, route every relevant source-reachable substitution/materialization path through `try_apply` or the fallible materialization path. Do not expose C2 terms to an unchecked path that can hide duplication.
3. Do not infer binder mode from kind, use `...` as generic-pack syntax, add decomposition inference, or add runtime ABI work.

## Verification And First Commands

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::type_model -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::generic_application -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::metadata -- --nocapture --test-threads=1
```

Read the P1 walkthrough, the C2 checkpoint record, the accepted requirements analysis, and the ratified argument-pack/callable specifications before P2 edits. Do not re-explore or redesign the P1 ownership model.

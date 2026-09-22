---
id: LANG007.C1.P3
kind: implementation-plan-handoff
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
base_revision: 6551576035a6e55718497c457fac6073b322d599
implementation_revision: 6551576035a6e55718497c457fac6073b322d599
checkpoint: LANG007.C1
program: LANG007
prepared: 2026-09-22
---

# LANG007.C1.P3 — Handoff

## 1. Checkpoint Closure State

`LANG007.C1` is **`COMPLETE / IMPLEMENTED / FOCUSED_TESTED`**.

All three checkpoint plans are fully completed:
- `LANG007.C1.P1`: Semantic domains, type-member identity/projection, structural operations, inference/persistence, Universe/runtime boundary.
- `LANG007.C1.P2`: Authoritative specification and `intrinsic` keyword convergence (TDR-0089).
- `LANG007.C1.P3`: Architecture baseline, cross-layer certification, acceptance matrix, roadmap re-baselining, and C1 closure.

## 2. Canonical Architecture and Decisions Inherited

- **TDR-0081:** Tuple and Record are transparent structural value products.
- **TDR-0082:** Declaration modifiers and access domains.
- **TDR-0083:** Type-level structural domains (`TypePack`, `LabeledTypePack`, `RecordRow`, `Const(Int)`).
- **TDR-0084:** Canonical type-level terms are stable kinded identities (`TypeId` with `KindId`).
- **TDR-0085:** Type members are parameterized, domain-aware projections with symbolic residuals.
- **TDR-0086:** Tuple subscript and `At<I>` are one canonical compile-time Subscript identity.
- **TDR-0087:** Structural `Map<F>` preserves shape and maps payload types with `F : Type -> Type`.
- **TDR-0088:** Native/intrinsic type operations use closed compiler normalization, not VM execution.
- **TDR-0089:** `intrinsic` is a first-class implementation-kind keyword; `@intrinsic` is retired.

## 3. As-Built Capabilities Owned by C1 (DO NOT REIMPLEMENT)

Future plans and checkpoints must reuse the following established C1 substrate:

1. **Domain System:** `TypePack`, `LabeledTypePack`, and `RecordRow` are distinct domains. `TypePack` and `LabeledTypePack` are kinded terms stored in `TypeStore` via `TypeData::TypePack` and `TypeData::LabeledTypePack`.
2. **Type-Member Registry:** `TypeMemberSignature`, `TypeMemberSelector`, and `lookup_intrinsic_type_operation` in `phalcom-semantic/src/types/type_member.rs`.
3. **Subscript Alias Convergence:** `T[I]` and `T::At<I>` parse/lower to `TypeMemberSelector::Subscript` and normalize to `TupleAt`.
4. **Symbolic Projections:** `TypeLevelProjection` in `TypeData::TypeMemberProjection` preserves unreduced abstract operations.
5. **Structural Operations:** `Positionals`, `Labeled`, integer `At` (including negative indexing), and `Map<F>` over `Tuple`, `Record`, `TypePack`, `LabeledTypePack`, and `RecordRow` (with open-row mapped tails).
6. **Zero-Product Law:** `Tuple([], []) ≡ Unit`; `Record({})` is a distinct static Record type.
7. **Metadata Persistence:** `KindNode::TypePack`, `KindNode::LabeledTypePack`, `TypeNode::TypePack`, `TypeNode::LabeledTypePack`, `TypeNode::TypeMemberProjection` in `phalcom-type-meta`.
8. **Inference / Solver Separation:** Ephemeral solver variables (`InferVarId`, `RecordRowVarId`) never enter canonical type store, metadata, or snapshots.
9. **Zero-VM Runtime Contract:** Type members normalize strictly at compile time. No type-member VM bytecodes, no runtime projection solver, and no runtime pack/row objects exist.

## 4. Remaining Capability Gaps for Future Checkpoints

- Variadic pack expansion syntax (`...P`, `T...`) and variadic generic parameters (`<...T>`).
- Bounded pack inference (solving pack components from call sites, pack splitting, prefix/suffix matching).
- Variadic callable lowering and calling convention.
- Broader const generics beyond type-member arguments.
- Additional closed structural operators if accepted.
- Tooling/LSP rich inspection for symbolic projections.
- Release certification.

## 5. Known Residual Baseline Failures

- `BASELINE-001`: `semantic::capabilities::traits::trait_default_replays_external_nominal_callable_dependencies`
- `BASELINE-002`: `semantic::capabilities::traits::trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities`

Both are pre-existing trait dependency baselines outside LANG007.

## 6. Recommended Starting Commands for Successor Sessions

```sh
git status --short
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --test integration intrinsic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::type_member_closure
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::type_members
```

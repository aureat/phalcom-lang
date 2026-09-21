---
id: LANG007.C1.P1
kind: implementation-plan-handoff
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
base_revision: e99ad77923893afa00b46141c8cf88932cad8f78
implementation_revision: 479f8fb82fa58730fb2c40c2652b5e07016549ec
working_tree: uncommitted scoped P1 semantic/test changes plus lifecycle records and the prepared P2 plan
---

# LANG007.C1.P1 - Handoff

## Current state

P1 is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`. The repository `HEAD` is
`e99ad77923893afa00b46141c8cf88932cad8f78`; the P1 implementation is present
through `479f8fb82fa58730fb2c40c2652b5e07016549ec`, with scoped uncommitted
changes in:

- `phalcom-semantic/src/types/annotation.rs`
- `phalcom-semantic/tests/semantic/foundations/type_members.rs`

Do not discard or reset these changes. The untracked P2 plan and lifecycle
records are intentional continuation artifacts.

## Completed prerequisites

- `TypePack` and `LabeledTypePack` have distinct canonical kinds.
- `TypeMemberSelector::Subscript` is the shared identity for `T[I]` and
  `T::At<I>`.
- Typed type-member arguments include `Const(Int)` and are validated at the
  source formation boundary.
- Tuple `Positionals`, `Labeled`, `At`, and `Map` plus Record/pack mapping
  normalization are compiler-owned semantic operations.
- Substitution, row/inference preservation, stable metadata, source identity,
  and canonical Universe anchors have been wired for the new forms.
- No type-member VM dispatch or runtime pack/row object model was added.

## Stable takeover map

- Canonical member identity and intrinsic registry:
  `phalcom-semantic/src/types/type_member.rs`.
- Projection evaluator and symbolic fallback:
  `phalcom-semantic/src/types/projection.rs`.
- Source formation and argument validation:
  `phalcom-semantic/src/types/annotation.rs`.
- Canonical forms and kinds: `phalcom-semantic/src/types/{kind,id,store}.rs`.
- Substitution/inference preservation:
  `phalcom-semantic/src/{checker/gadt_proof.rs,checker/inference.rs,checker/row_inference.rs,types/substitution.rs}`.
- Stable metadata: `phalcom-semantic/src/metadata/export.rs` and
  `phalcom-type-meta/src/{kind,type_node,validate}.rs`.
- Universe anchors:
  `phalcom-core/core/universe/src/collections/{tuple,record}.ph`.

## Invariants not to redesign

- Keep `TypePack`, `LabeledTypePack`, and `RecordRow` distinct.
- Keep ordered Tuple labeled lanes out of sorted row canonicalization.
- Keep compile-time integer arguments as typed constants, not an `Int` kind or
  fake proper types.
- Keep symbolic projections distinct from invalid and dynamic outcomes.
- Keep existing trait-associated type identity/evidence authoritative.
- Keep structural normalization out of VM/runtime dispatch.
- Keep query-local inference and row variables out of canonical metadata.

## Known drift and residuals

The AST integration test target is `integration`, not `parser`; use the live
target name. Two trait dependency assertions fail both at the current tree and
at the clean predecessor baseline. They are nonblocking baseline work:

- `trait_default_replays_external_nominal_callable_dependencies`
- `trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities`

Do not repair these in P2 unless its narrow source/specification changes provide
new evidence that directly causes them.

## P2 objective

Execute `LANG007.C1.P2` T0 onward. P2 must consume this handoff and the
checkpoint's P1 gate evidence, then converge the accepted decision registry,
normative specifications, source/presentation surface, and lifecycle records on
the completed P1 model. The fixed ruling is that `intrinsic` is the source
keyword and `@intrinsic` is retired as a source/builtin attribute; internal
`NativeIntrinsicId` metadata remains distinct and may survive.

## Recommended first commands

```sh
git status --short --branch
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::type_members -- --nocapture
```

Then perform only P2's named `@intrinsic`/`intrinsic` authority census and
verify the next accepted TDR number before substantive P2 edits.

## Do not re-explore or redesign

Do not rerun the full P1 matrix merely to activate P2, reconstruct P1 from
history, reopen type-member identity, change the projection/result-domain
model, introduce VM type-member execution, or promote general const generics,
GATs, arbitrary type families, union distribution, or runtime pack objects.

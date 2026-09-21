---
id: LANG007.C1.P1
kind: implementation-plan-walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
base_revision: e99ad77923893afa00b46141c8cf88932cad8f78
implementation_revision: 479f8fb82fa58730fb2c40c2652b5e07016549ec
working_tree: uncommitted scoped P1 semantic/test changes plus the prepared P2 plan
---

# LANG007.C1.P1 - Implementation Walkthrough

## Result

P1 is complete as an implementation plan: `COMPLETE / IMPLEMENTED /
FOCUSED_TESTED`. The implementation landed in `479f8fb8` and the current
worktree contains the final focused argument-kind validation and hostile-case
regressions in `annotation.rs` and `foundations/type_members.rs`.

P1 establishes the semantic/compiler substrate for the first structural
type-operation vertical. P2 is a specification and source-surface convergence
plan, not a reopened P1 implementation plan.

## Implemented architecture

- `TypePack` and `LabeledTypePack` are distinct canonical kinds alongside
  `Type` and `RecordRow`.
- Ordered pack terms, labeled pack terms, and `TupleFromPacks` preserve domain
  and ordering distinctions; concrete tuple formation canonicalizes to the
  existing tuple representation.
- `TypeMemberSelector` gives bracket subscripting and named `At` one canonical
  subscript identity.
- `TypeMemberArgument` distinguishes type arguments, `Const(Int)` arguments,
  and row arguments. Invalid intrinsic argument shapes are rejected rather
  than becoming symbolic or dynamic fallbacks.
- `IntrinsicTypeOperation` and the projection normalizer own closed semantic
  operations for Tuple, Record, TypePack, LabeledTypePack, and RecordRow
  domains. Tuple indexing handles negative indexes and preserves positional
  then labeled ordering; mapping uses ordinary type-form application.
- Projection substitution, inference/materialization, relation handling, and
  stable metadata carry the new forms without publishing query-local
  inference or row-variable identities.
- Canonical Universe Tuple and Record declarations anchor the structural
  members as `intrinsic type` declarations. No type-member VM bytecode or
  runtime type-member dispatch was introduced.

## Important files

- `phalcom-semantic/src/types/type_member.rs` - canonical member selectors,
  domains, arguments, and intrinsic operation registry.
- `phalcom-semantic/src/types/projection.rs` - symbolic projection and
  intrinsic structural normalization.
- `phalcom-semantic/src/types/annotation.rs` - source projection formation and
  intrinsic argument validation.
- `phalcom-semantic/src/types/store.rs` - canonical pack, tuple-from-pack, and
  projection storage.
- `phalcom-semantic/src/checker/{gadt_proof,inference,row_inference}.rs` -
  substitution and inference preservation.
- `phalcom-semantic/src/metadata/export.rs` and
  `phalcom-type-meta/src/{kind,type_node,validate}.rs` - stable metadata.
- `phalcom-semantic/tests/semantic/foundations/type_members.rs` and
  `type_model.rs` - focused semantic regressions.
- `phalcom-ast/tests/parser.rs` - kind, projection, and intrinsic declaration
  parser coverage.
- `phalcom-core/core/universe/src/collections/{tuple,record}.ph` - canonical
  source anchors.

## Durable interfaces and invariants

- `TypeId` remains the canonical handle for kinded forms; `ProperTypeId`
  remains the proper-`Type` boundary.
- Tuple labeled-lane order is not represented through sorted record-row
  identity.
- A valid abstract projection remains symbolic; concrete invalidity remains an
  invalid formation result.
- Type-member computation remains compiler/semantic normalization and never
  becomes runtime dispatch.
- Existing trait-associated projection identity and evidence remain separate
  from intrinsic domain-member identity.

## Deviations and consultations

No architectural consultation was required. The live AST test target is named
`integration`, not `parser` as written in the planning examples; the parser
commands were adapted mechanically to the live target. The final focused
worktree changes add intrinsic argument-kind validation and two hostile-case
tests without changing the ownership or representation decisions.

## Verification executed

The following focused evidence passed:

| Scope | Result |
| --- | --- |
| `phalcom-ast` intrinsic parser tests | 2 passed |
| `phalcom-ast` type-pack-kind parser test | 1 passed |
| `phalcom-ast` type-member projection parser test | 1 passed |
| `phalcom-semantic` `foundations::type_members` | 6 passed |
| Type-pack/labeled-pack formation and tuple-from-packs canonicalization | 2 passed |
| `foundations::type_annotations` | 40 passed |
| `foundations::record_row_inference` | 3 passed |
| `foundations::generic_application` | 9 passed |
| `integration::metadata` | 14 passed |
| intrinsic presentation regression | 1 passed |
| affected crate checks (`phalcom-ast`, `phalcom-type-meta`, `phalcom-semantic`, `phalcom-core`) | passed |
| `cargo fmt --all -- --check` and `git diff --check` | passed |

The focused trait suite ran 38 tests: 36 passed and 2 reproduced baseline
failures listed below.

## Tests deliberately deferred

P1 did not run workspace-wide tests, workspace clippy, the full language
corpus, unrelated VM/runtime suites, all LSP/extension suites, broad
performance suites, or release certification. These remain outside the P1
focused acceptance gate.

## Residual baseline failures

Both failures reproduce at the clean predecessor revision
`5b63ade8073d13d6b528af4774181665c30cbd47`, so they are recorded as baseline
and are not P1 implementation blockers:

- `semantic::capabilities::traits::trait_default_replays_external_nominal_callable_dependencies`
- `semantic::capabilities::traits::trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities`

Reproduction:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  capabilities::traits::trait_default_replays_external_nominal_callable_dependencies -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  capabilities::traits::trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities -- --nocapture
```

The failures belong to the existing semantic dependency/evidence baseline and
should be revisited by their owning semantic plans, not by P2.

## Follow-up

P2 may start at T0 after consuming this walkthrough and the handoff. P2 must
preserve the completed P1 semantic architecture while converging the accepted
decision/specification corpus and retiring the obsolete `@intrinsic` source
attribute. P1 does not close the whole C1 checkpoint.

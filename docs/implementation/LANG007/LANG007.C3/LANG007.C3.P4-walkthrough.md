---
id: LANG007.C3.P4
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
repository: aureat/phalcom-lang
prepared: 2026-09-25
---

# LANG007.C3.P4 walkthrough

## Outcome

Implemented the adopted correction separating intrinsic semantic type
application from the runtime/native owner projection on one canonical
`DeclarationTypeInfo` binding. C3 is complete at `IMPLEMENTED /
FOCUSED_TESTED`; no release certification is claimed.

## Implementation

- Added `DeclarationSemanticForm`, `IntrinsicTypeApplication`, and
  `DeclarationTypeBinding` with an optional `IntrinsicRuntimeOwner`.
- Preserved both projections when an ordinary header/query shell refreshes a
  canonical intrinsic declaration; declaration fingerprints include the
  binding.
- Changed annotation formation to select Unit/Tuple/Record behavior from the
  semantic projection. Tuple still uses `CompleteTypePack` and
  `TupleFromPacks`; Record still consumes its `RecordRowId`; Unit still maps to
  `TypeStore::unit()`.
- Kept native import and runtime facade projection on the runtime owner. The
  focused native test proves Tuple methods resolve to the existing Tuple
  declaration and its runtime form, not the semantic test form.
- Kept `AssociatedSurface`, `TypeMemberDeclarationId`, runtime VM class
  identity, and TypeLambdaArena behavior unchanged.

## Verification

- Semantic declarations: 8 passed.
- Semantic type annotations: 51 passed, including Tuple/Record formation
  without a runtime owner and ordinary type-lambda behavior.
- Native Tuple runtime-owner projection: 1 passed.
- Incremental declaration fingerprints: 34 passed.
- Associated/type-member closure: 10 passed, including concrete and symbolic
  `TupleEntries` projection.
- Incremental type-member suite: 24 passed.
- Core Unit preinstalled-class reuse: 1 passed.

All focused C3/P4 gates passed. Workspace and release gates were not run.
`cargo fmt --all -- --check` remains non-green with repository-wide formatter
drift already classified on clean predecessors; no broad reformat was applied.
Previously classified AST/impl-query/runtime and incomplete full-binary
baselines remain in the C3 checkpoint.

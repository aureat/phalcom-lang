---
id: LANG006.C2.P1
kind: implementation-plan-walkthrough
status: COMPLETE
completion: COMPLETE
verification: FOCUSED_TESTED
base_revision: c3a53c3e364e588d46bd56ade246976e1996d744
working_tree: uncommitted scoped C2 implementation changes
---

# LANG006.C2.P1 — Implementation Walkthrough

## Result

Validated data construction is implemented and focused-tested. Data primary
constructor visibility is parsed after the name/generic list, published in the
canonical semantic constructor product, authorized through the C1 identity and
provenance relation, and enforced before lowering. Both existing `where`
placements remain accepted; new fixtures use the trailing form canonically.

## Implemented architecture

- `DataDef.constructor_visibility` and its source range carry normalized source
  visibility. Omitted visibility is Public; `public` is not introduced;
  `protected` and duplicate constructor modifiers are rejected.
- `DataConstructorSignature.visibility` is the sole semantic constructor
  visibility product. `DataConstructorId`, nominal identity, components, and
  runtime representation are unchanged.
- `CheckingContext::declaration_member_is_accessible` delegates to
  `is_member_accessible` with canonical module/package/project provenance.
  Direct tuple, record, and associated invocation targets are authorized before
  `apply_resolved_callable`.
- Exact and family associated constructor references authorize at acquisition.
  Captured authorized capabilities are not re-authorized when later applied.
- Visibility is included in data input/product fingerprints, so a visibility
  edit invalidates dependent construction while retaining stable identity.
- No core runtime, data descriptor, VM, or optimizer visibility authority was
  added.

## Focused acceptance

| Coverage | Evidence | Result |
| --- | --- | --- |
| CV-01 | AST default and restrictive tuple syntax | PASS |
| CV-02 | AST and semantic private record construction | PASS |
| CV-03 | protected, duplicate, public, and top-level-invalid forms | PASS |
| CV-04 | canonical product publication and fingerprints | PASS |
| CV-05 | private inherent factory | PASS |
| CV-06 | same-module non-owner private tuple/record denial | PASS |
| CV-07 | exact module boundary matrix | PASS |
| CV-08 | same, nested, and sibling package matrix | PASS |
| CV-09 | same-project and cross-project internal matrix | PASS |
| CV-10 | tuple/record authorization parity | PASS |
| CV-11 | inaccessible exact associated reference | PASS |
| CV-12 | inaccessible associated family capture | PASS |
| CV-13 | authorized constructor capability escape | PASS |
| CV-14 | generic private constructor specialization | PASS |
| CV-15 | nullary private construction denial before singleton lowering | PASS |
| CV-16 | re-exported name does not widen construction access | PASS |
| CV-17 | Public→Private incremental invalidation and cold parity | PASS |
| CV-18 | unrelated data product reuse | PASS |
| CV-19 | enabled/disabled product optimization differential | PASS |
| CV-20 | existing representation/equality/product suite plus no runtime edits | PASS |
| CV-21 | MemberInaccessible without unresolved/missing classification | PASS |
| CV-22 | production search found no independent runtime/reflection authority | PASS |

## Verification evidence

- AST visibility filter: 3 passed.
- Semantic data declarations: 19 passed.
- Topology-backed constructor visibility integration: 5 passed.
- Incremental data filter: 4 passed.
- Associated data-constructor denial, invocation, and capability-escape tests:
  passed; existing data-associated scenarios also passed.
- Core product optimization module: 17 passed, including the new private
  factory differential.
- `cargo check -p phalcom-semantic`, `cargo check -p phalcom-core`, and
  `git diff --check`: passed.

## Baseline boundaries

The full AST integration target previously completed with 249 passed, 3
inherited C1 snapshot failures, and 1 ignored test. The failures are the
existing `invocation_authority: Ordinary` and dunder field-kind snapshot
drifts; generated `.snap.new` artifacts were removed and snapshots were not
updated. Repository-wide formatting remains dirty from unrelated pre-existing
drift; the unrelated untracked `docs/implementation/TEST002/` plan artifact
was preserved, so no broad formatting rewrite was made.

No workspace/release certification was run. No consultation or plan amendment
was required.

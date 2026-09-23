# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: LANG007
- Checkpoint: LANG007.C3
- Plan: LANG007.C3.P2
- Task/Gate: T7 — source type-member projections in all declaration annotations
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Starting revision: `7f976f57dc317e5cceccb00e15529eca684288cb`
- Current revision: `7f976f57dc317e5cceccb00e15529eca684288cb` (before this report)

## 2. Escalation trigger

No new mandatory consultation trigger is established by this inspection. This is
an implementation handoff for a known T7 phase-ordering gap, not a reproduced
semantic failure or proof that the accepted architecture is impossible. The
existing T7 amendment already prohibits a second surface/name index and directs
the session phase reorganization.

If implementing that reorganization requires changing alias/generic-header
dependency ownership, publishing a competing surface, or introducing a second
canonical selector, stop and consult under triggers 1, 7, 14, or 16 as applicable.

## 3. Required invariant

- `AssociatedSurface` remains the sole authority for associated-name admission,
  collisions, and selection.
- A selected source member is retrieved by stable
  `TypeMemberDeclarationId`; its checked declaration product owns the signature.
- Every consuming annotation records dependencies on the selected surface and
  signature product.
- Alias and generic-header dependency contracts remain valid; no provisional
  second surface or parallel name index is published.

These are the T7/INC-C3-3 invariants; no new semantic invariant is proposed here.

## 4. Expected architecture

The session must make the canonical owner/header and associated-surface selection
products available before every annotation consumer, including aliases, generic
headers/constraints, type-member signatures, callable signatures, fields, and
later body analysis. Type formation uses the session/query-provided resolver
context already adopted in INC-C3-3. Intrinsic member identity remains governed by
its existing rules.

## 5. Observed behavior

Direct source inspection confirms the phase-order mismatch described in the T7
plan amendment:

- transparent aliases are lowered at `session.rs:2366`, before the generic
  signature/supertype-template phase beginning at `session.rs:2522`;
- callable and field signature materialization begins before the data/enum and
  associated-surface publication phase at `session.rs:3321`;
- the session updates the resolver's associated type-member context only after
  that publication loop, at `session.rs:4460`.

Therefore annotations consumed in earlier phases cannot select source type-member
projections through that published surface/context. This is a code-order finding;
no new test failure was reproduced in this report.

## 6. Minimal reproduction

No executable reproducer was run. The required discriminating coverage is to place
source type-member projections in each annotation phase that currently precedes
surface publication, then verify successful selection, wrong-domain rejection,
canonical projection identity, and incremental invalidation. Exact fixtures should
follow the accepted grammar and existing test helpers rather than be invented here.

Expected behavior: each supported declaration annotation resolves through the
same published `AssociatedSurface` decision and records the corresponding surface
and member-product dependencies.

## 7. Relevant code path

```text
session::analyze
  -> alias lowering (session.rs:2366)
  -> generic headers/signatures (session.rs:2522)
  -> callable/field signature consumers (session.rs:3119 onward)
  -> data/enum + AssociatedSurface + type-member publication (session.rs:3321 onward)
  -> resolver.update_associated_type_members (session.rs:4460)
```

Type formation is implemented by `resolve_type_member_projection` in
`phalcom-semantic/src/types/annotation.rs`; source selection and signature
retrieval are described by the T7 amendment in the P2 plan and INC-C3-3.

## 8. Scoped implementation diff

None. This report records the next implementation boundary; it does not change
semantic code or claim the phase reorganization is implemented.

## 9. Evidence

- The live session source has the ordering listed in section 5.
- The P2 T7 plan explicitly requires projections in all declaration annotations
  and says not to publish a provisional second surface or parallel name index.
- The checkpoint's INC-C3-3 records the user's accepted resolver seam and says
  the all-annotation phase reorganization remains unfinished.
- No tests or formatters were run for this report.

## 10. Implementer's causal hypothesis

The current stage graph publishes associated surfaces only after several formal
annotation consumers have already run. Extending only the resolver context cannot
make those earlier consumers see a canonical selection product; the stage order
must change while preserving the existing alias and generic-header dependency
tracking.

## 11. Alternative hypotheses

- Some annotation classes may not require source-member projection support under
  the accepted grammar; the plan/user requirement currently says all declaration
  annotations, so this must be resolved by concrete coverage rather than assumed.
- A staged query dependency may allow early construction of the final canonical
  surface without violating declaration prerequisites; this has not yet been
  demonstrated against the live phase graph.

Neither alternative authorizes a second selection authority.

## 12. Attempts already made

No phase-reordering implementation attempt is recorded here. The previously
implemented T7 resolver seam is recorded in INC-C3-3 and remains in place; this
report does not repeat or reclassify that work.

## 13. Why local authority is insufficient

At present, local authority is sufficient to map dependencies and attempt the
user-directed reorganization. It is not sufficient to change the accepted
ownership/dependency contract if the stage graph proves incompatible with it.
This report does not request an architectural override in advance.

## 14. Decision requested

Proceed with a dependency map and a concrete phase-reordering implementation that
makes one canonical `AssociatedSurface` available to all annotation consumers.
If the map shows that this cannot preserve the alias/generic-header contract, return
with the exact conflicting dependency cycle and proposed narrow amendment before
changing authority or publishing an alternate surface.

## 15. Resume decision

IMPLEMENTER MAY RESUME on the planned phase reorganization and discriminating
tests. Do not treat this report as evidence that T7 or P2 is complete.

## 16. Durable record

The checkpoint remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`; T7 phase
reorganization is open. No focused verification was performed for this report.

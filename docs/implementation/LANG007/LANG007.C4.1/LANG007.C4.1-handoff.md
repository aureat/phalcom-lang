---
id: LANG007.C4.1
kind: handoff
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# LANG007.C4.1 corrective closure handoff

The covering inherent-impl body-coordinate closure is implemented and focused-tested.

Stable invariants:

- `EffectiveCallableDefinition.signature` remains declaration-canonical.
- `InherentBodyContext` supplies the inverse covering view for body checking.
- Body-local `Self` is specialized to the source target head.
- Runtime receiver `self` remains distinct from type-level `Self`.
- Callable-local binders are not rewritten by owner-relative `Self` specialization.
- RecordRow carriers are recursively specialized through canonical TypeStore formation APIs.
- Callable body invalidation continues through `CallableBody -> CallableDefinition -> InherentBodyContext`.
- Source body identity remains `(ImplId, source_member_index)`.

Focused verification and the incremental reuse corrective follow-up are
recorded in `LANG007.C4.1-walkthrough.md` and `corrective-2.md`.

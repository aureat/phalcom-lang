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
- Runtime receiver `self` remains distinct from type-level `Self`; class-side bodies retain class-object receiver semantics while type-level `Self` uses the source target instance.
- Callable-local binders are not rewritten by owner-relative `Self` specialization.
- RecordRow carriers are recursively specialized through canonical TypeStore formation APIs.
- Callable body invalidation continues through `CallableBody -> CallableDefinition -> InherentBodyContext`.
- Source body identity remains `(ImplId, source_member_index)`.

Focused verification and the incremental reuse corrective follow-up are
recorded in `LANG007.C4.P4.R1-historical-covering-impl-closure-walkthrough.md`, `LANG007.C4.P4.R1-callable-body-incremental-reuse-follow-up.md`, and the
receiver-separation audit follow-up in `LANG007.C4.P4.R1-covering-inherent-impl-coordinate-audit.md`.

## Audit hardening and Telogue integration follow-up

The six three-slot covering permutations and representative nested shapes
(applied types, tuples and Packs, callable parameters/results, record rows,
and projections) are covered by `impls::bodies`. Its test-only type-graph
walk checks public signatures and constraints for impl-owned parameters,
including nested type arguments and row structure. The focused selector now
passes 21 tests, and `cargo check -p phalcom-semantic` passes after the body
override API rename.

Callable-body query/request and checker fields now distinguish the runtime
`receiver_value_override` from the type-level `instance_self_override`; the
type-level name is also used by body fingerprint inputs, and both names are
used by in-repository callers. The type-formation `self_type_override` field
and resolver method are unchanged.

Telogue's covering-impl returns use bare `Self`, and `query.ph` expresses its
variadic parameter as `Pack<*Goals>`. A core integration regression loads the
Telogue project manifest and compiles its declared `prolog.main` entry. That
gate currently reaches semantic analysis but fails on generic-pack
contribution diagnostics in `main.ph`, positional-lane projection diagnostics
for `Ts::Map<Term>` in `predicate.ph`, and a generic-inference conflict at the
`goals` value in `query.ph`. The query conflict also occurs with an explicit
`Query<*Goals>` return annotation, so the current evidence does not implicate
bare `Self`; full Telogue project compilation remains unverified. Re-run this
integration gate after those package-level issues are resolved, and do not
report project compilation as passing based on the focused C4.1 semantic
tests.

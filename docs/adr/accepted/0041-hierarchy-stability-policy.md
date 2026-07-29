# 41. Hierarchy-stability policy: sealed reparenting + single inheritance

- Status: Accepted
- Date: 2026-07-12
- Related: [ADR-0026](0026-class-hierarchy-mutability.md) (prior art — already
  ships the sealed-reparent / open-methods split this ADR ratifies as unit
  policy), [ADR-0011](0011-static-instance-slot-layout.md) (static per-class
  slot layout — the invariant sealing protects), [ADR-0017](0017-class-side-stored-static-fields.md)
  (class-side field offsets up the tower), [ADR-0018](0018-sacred-selector-inliner-and-override-guard.md)
  (override-epoch deopt guard — reused by method reopening; the escape hatch a
  future `reshape` would reuse), [ADR-0009](0009-handle-arena-heap.md) (handle
  heap — keeps a future reparent-with-migration implementable, so sealing now
  costs nothing later), [ADR-0012](0012-selector-signature-encoding-and-dispatch.md)
  (one-hashmap-probe dispatch — the invariant single inheritance protects),
  `docs/spec/current/object-model.md` §1.5/§5, `docs/spec/current/open-questions.md`
  Q4/Q10, `docs/forge/units/U13/plan.md`

## Context

[U13](../forge/units/U13/plan.md) grounds in two open questions that both
decide what the class graph's *shape* may assume, and both bound what the
inline-cache design (ADR-0012), the fixed instance slot layout (ADR-0011), and
the metaclass tower (ADR-0002) can rely on instead of leaving unstated:

- **Q4 — hierarchy mutability.** May a class's `superclass` be reassigned at
  runtime (Smalltalk: `Circle.superclass = Shape` is legal), or is it fixed
  after definition (Wren: no)?
- **Q10 — traits / mixins / multiple inheritance.** Does Phalcom stay single
  inheritance, or gain some form of composition beyond one `superclass`?

Both questions were **BLOCKED-ON-DECISION** in the U13 work order
(DEC-U13a, DEC-U13b). **Resolved by orchestrator autonomous authority,
2026-07-12** (DEC-U13a = A, DEC-U13b = A — both the architect-recommended
conservative option; reversible pre-release, per the standing decision
protocol for decisions the user has delegated). This ADR records the ruling as
unit policy and cites the enforcement + test coverage that lands with it.

Q4's decision (DEC-U13a) was **already shipped** as [ADR-0026](0026-class-hierarchy-mutability.md)
— that ADR's "Split the axes: methods are open, the superclass link is
sealed" is exactly DEC-U13a=A. This ADR does not re-litigate that design; it
ratifies it as the U13 unit's answer to Q4 and adds the U13 test coverage
(`sealed_hierarchy_rejects_runtime_reparent_and_keeps_invariants` in
[`tests/invariants.rs`](../../phalcom-core/tests/invariants.rs) and the
`runtime-errors/runtime_error_superclass_reparent_rejected.ph` golden). Q10
(DEC-U13b) had no dedicated ADR before this one — only the inline ruling in
`open-questions.md` — so this ADR is Q10's primary citation.

## Decision

**DEC-U13a — sealed after definition (Wren-style), methods stay open.**
A class's `superclass` is fixed at class creation
([`ClassObject::superclass`](../../phalcom-core/src/class.rs)). Any attempt to
reassign it at runtime — the `Behavior::superclass=(_)` primitive,
[`class_set_superclass`](../../phalcom-core/src/primitive/class.rs) — is
rejected with [`RuntimeError::InvalidSetSuper`](../../phalcom-core/src/error.rs)
("Can't set superclass of a class"), a clean, catchable, typed error, never a
panic and never a partial mutation. **Method reopening is unaffected**: a
class may still have methods added or replaced after its first definition
(the override-epoch mechanism, [ADR-0018](0018-sacred-selector-inliner-and-override-guard.md)),
because that axis costs nothing and does not touch slot layout.

```phalcom
class Circle is Shape { }
Circle.superclass = Rectangle   // ERROR: Can't set superclass of a class
```

**DEC-U13b — single inheritance only, traits/mixins deferred.** Phalcom keeps
exactly one `superclass` per class with `Object` as the tower root
(`ClassObject::superclass: Option<ClassId>` — the type itself forecloses
multiple superclasses; there is no surface grammar for `class C with T1, T2`
and none is added by this ADR). Traits/mixins/full multiple inheritance are
**out of scope for this unit and this ADR** — deferred, not rejected.

## Forward path for the deferred options (pre-approved, not built here)

- **A future reparent-with-migration primitive** (Q4's non-sealed option) is
  *not foreclosed* — [ADR-0009](0009-handle-arena-heap.md)'s handle heap keeps
  it implementable. It would need to recompute slot layouts of every live
  instance in the reparented subtree, invalidate every dependent inline cache
  (reusing the ADR-0018 override epoch), and re-run `verify_invariants()`
  post-mutation. Sealing now is forward-compatible: adding mutability later
  breaks nothing that depends on the seal.
- **Stateless traits, flattened at class-finalization** (Q10's admissible
  extension) are the *only* pre-approved composition mechanism, should the
  user want one later: a trait is a named bag of methods with **no fields**;
  `class C is S with T1, T2` would copy trait methods into `C`'s own
  method dictionary at finalization time, with an explicit conflict
  surfacing as a compile error. This preserves both invariants below —
  dispatch never grows a second lookup path, and no trait ever contributes
  instance state, so the fixed slot layout is never in question. **Full
  multiple inheritance with a runtime MRO walk (C3 linearization or
  otherwise) is rejected outright** as inadmissible under the committed
  dispatch/layout design, not merely deferred — it would require a per-send
  ancestor walk (breaking [ADR-0012](0012-selector-signature-encoding-and-dispatch.md)'s
  one-hashmap-probe dispatch) and per-instance variable state to come from
  more than one ancestor (breaking [ADR-0011](0011-static-instance-slot-layout.md)'s
  fixed offsets). Adopting it would be a redesign, not a feature.

## Consequences

- **Slot offsets and `ClassId`-keyed dispatch are provably stable.** Because
  the superclass link never moves after a class is created, ADR-0011's
  offset-stability proof holds unconditionally for every user class, not just
  the kernel tower — a future inline-cache population (ADR-0012, deferred)
  can key on `ClassId` alone with no invalidation-on-reparent case to handle.
- **Dispatch stays exactly one hashmap probe.** No MRO walk exists anywhere in
  the VM; nothing this ADR ships adds a second method-lookup path.
- **`verify_invariants()` cannot be silently defeated by a reparent attempt.**
  A rejected `superclass=` leaves the class graph byte-for-byte unchanged —
  tested directly (see Test coverage below) — so there is no path by which an
  attempted mutation corrupts the tower without a loud, typed error.
- **Concurrency (`docs/spec/current/concurrency.md`) is simplified.** Class
  objects are shared across fibers via the handle heap; a sealed hierarchy
  means a `superclass=` mid-computation can never surprise a suspended fiber
  — the mutable-hierarchy hazard Q4's option B would have introduced does not
  exist.
- **U16 (`Family`/`::` base-name index)** and any future trait-flatten step
  both build at the same class-finalization seam; sealing does not move or
  fork that seam.

## Test coverage

- `phalcom-core/tests/invariants.rs::sealed_hierarchy_rejects_runtime_reparent_and_keeps_invariants` —
  a direct Rust-level call to `class_set_superclass` on a user class asserts
  (a) the typed `RuntimeError::InvalidSetSuper`, (b) the class graph is
  byte-for-byte unchanged (`Dog.superclass` still `Animal`), and (c)
  `verify_invariants()` still passes *after* the rejected mutation.
- `phalcom-core/tests/lang/runtime-errors/runtime_error_superclass_reparent_rejected.ph` —
  an end-to-end CLI golden: a surface `B.superclass = Object` send exits
  non-zero with `Can't set superclass of a class`, never a panic.
- Existing tower regression coverage (`metaclass_superclass_parallels_instance_superclass`,
  `user_subclass_metaclass_parallels_superclass`, and siblings in the same
  file) is unaffected — this ADR adds no new mutation path for them to
  regress against.

## Alternatives considered

See [ADR-0026](0026-class-hierarchy-mutability.md) §Alternatives for Q4 (fully
sealed vs. fully mutable) — unchanged by this ADR. For Q10: full multiple
inheritance with C3 linearization was considered and rejected outright (see
Decision above) as incompatible with the committed one-probe dispatch and
fixed-offset slot layout; adopting it would be a different VM, not an
extension.

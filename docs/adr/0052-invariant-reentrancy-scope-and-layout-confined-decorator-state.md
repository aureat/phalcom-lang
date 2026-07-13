# 52. Invariant re-entrancy is receiver-scoped; per-receiver decorator state is Layout-confined

- Status: Proposed
- Date: 2026-07-13
- Related: [ADR-0008](0008-layered-exceptions-and-result.md) (unwind primitive —
  `ensure` fires on any unwind), [ADR-0009](0009-handle-arena-heap.md)
  (handle/arena heap, `Copy` `ObjRef`), [ADR-0011](0011-static-instance-slot-layout.md)
  (fixed per-class slot layout), [ADR-0013](0013-closure-upvalues-and-frame-token-return.md)
  (non-local return / unwind), [ADR-0050](0050-non-moving-mark-sweep-collector.md)
  (mark-sweep collector), `docs/spec/v0.2/experimental/annotations-contract-semantics.md`
  (invariant re-entrancy — the bug this amends), `docs/spec/v0.2/next/decorators.md`
  + `docs/spec/v0.2/next/attribute-classes.md` (tier model; attribute-classes.md
  already states the rule this ADR enforces), `docs/spec/v0.2/next/decorators-stdlib.md`
  (`@computed` — the violation this amends), `docs/forge/units/U-ANNOT-CONTRACTS/plan.md`
  (the two erratum fixes below — ownership-tracking pseudocode, fiber-switch
  `checking` state — were surfaced during that unit's build-order grounding,
  before any implementation, and folded back into this ADR the same day)

## Context

Two unratified annotation/decorator drafts each specify a runtime mechanism that
holds a reference or a counter at the wrong scope. Both were found during review
of the annotation/decorator spec set; both are correctness or memory bugs, not
open design questions, so they are fixed together here rather than left as
competing proposals.

### Bug 1 — invariant re-entrancy counter is fiber-scoped, not receiver-scoped

`annotations-contract-semantics.md`'s `@invariant` re-entrancy guard uses one
`in_public_call` counter **per fiber**, incremented on every public-method entry
and decremented on exit; the invariant check fires only on the 0→1 / 1→0
transitions. The stated intent is Eiffel's rule: don't re-check an object's
invariant on a call nested inside its own public call, since the object may be
transiently inconsistent mid-mutation.

The counter doesn't implement that rule — it implements "don't check any
invariant while any public call anywhere on this fiber is in flight." Because
the counter is shared across every receiver on the fiber, an object `B` whose
public method is called from *inside* object `A`'s public method sees the
counter already at 1 (from `A`) and skips its own check entirely, on entry and
exit — even though this is `B`'s own outermost call. `B`'s invariant is never
verified. This is the ordinary shape of object collaboration (one object
calling another's public methods), not an edge case, so it fires constantly,
not rarely.

A second, related gap: the original design never specifies what happens to the
counter across a thrown exception. If a woven precondition, a postcondition, or
an ordinary `throw` unwinds out of a public method without running the
epilogue that decrements the counter, the counter never returns to 0 —
invariant checking silently and permanently disables itself for the rest of
the fiber's life.

### Bug 2 — `@computed`'s per-receiver cache violates the tier line attribute-classes.md itself draws

`attribute-classes.md` states the governing rule directly: state that must live
*on the object being decorated* belongs in the Layout tier (a reserved slot),
not the Install tier (a decorator-owned side table) — because an Install-tier
decorator instance is created once at class-definition time and retained for
the life of the program, so anything it holds is retained for the life of the
program too. `@lazy` is given as the correct worked example: a reserved slot,
builtin-owned, populated lazily on the receiver.

`decorators-stdlib.md`'s `@computed` breaks that rule: it keeps a `Map` from
receiver → cached `Computed`, owned by the single class-level attribute
instance. Every receiver that ever reads the computed property becomes a
permanent key in that map — a strong reference held for the program's
lifetime. No instance of a class using `@computed` can ever be collected. This
works directly against the mark-sweep collector ([ADR-0050](0050-non-moving-mark-sweep-collector.md)):
the collector is correct, but this decorator pattern hands it objects it can
never prove unreachable.

## Decision

### Fix 1 — receiver-scoped invariant guard, unwind-safe

Replace the per-fiber integer counter with a **per-fiber identity set of
receivers currently under invariant-checking**: `checking: Set<ObjRef>`. This
is cheap — `ObjRef` is already `Copy`/hashable ([ADR-0009](0009-handle-arena-heap.md)),
and the set is empty outside of any invariant-checked call, so the common case
(no contracts in flight) costs nothing.

Woven prologue for a public method on a class carrying `@invariant`.
**Ownership must be captured as a local, before the insert** — gating the
epilogue on a re-check of `checking.contains(self)` is wrong, because once any
nested call exists, `self` is still in `checking` at exit time regardless of
which call inserted it, so a naive re-check would run the exit check (and the
removal) from every nested frame, not just the owning one:

```
let __invariant_owner = checking.contains(self).not   // captured BEFORE the insert below
if __invariant_owner {
    checking.insert(self)
    __check_invariant()          // entry check
}
// nested call on the same receiver (owner == false): no entry check, no insert
```

Woven epilogue, wired through the existing unwind primitive
([ADR-0008](0008-layered-exceptions-and-result.md) /
[ADR-0013](0013-closure-upvalues-and-frame-token-return.md) — `return`/`throw`/
fiber `abort` are one stack-unwind and `ensure` fires on any of them) rather
than only the normal-return path:

```
ensure {
    if __invariant_owner {        // the captured local, never a re-check of `checking`
        __check_invariant()      // exit check
        checking.remove(self)
    }
}
```

This fixes both parts of Bug 1. A different receiver `B` called from inside
`A`'s public method is not in `checking` (`A` only inserted itself), so `B`'s
own entry is correctly checked. And because removal is expressed through
`ensure` — the primitive the language already guarantees fires on every
unwind path — a thrown exception can no longer leave the guard permanently
inflated.

`checking` is per-fiber. Phalcom's cooperative single-threaded concurrency
model (concurrency §1) still makes this race-free — no lock is needed — but it
is now keyed by receiver identity instead of being a bare depth counter.

**`checking` must be fiber-switch state, not VM-global** (finding surfaced
during `U-ANNOT-CONTRACTS` planning, not present in the original decision
above). An `@invariant`-guarded call can `yield` mid-body — nothing in the
woven prologue/epilogue suppresses yielding — so if `checking` were kept as a
single VM-global set, a suspended fiber's in-flight invariant bookkeeping
would leak into whatever fiber resumes next, or a second fiber running
concurrently could see the first fiber's `checking` entries and wrongly skip
its own entry check. `checking: HashSet<ObjRef>` is per-`FiberObject`
(mirrored by a `VM::checking` pointer for the active fiber, same shape as the
existing `stack`/`frames`/`open_upvalues` fields), and it must be saved and
restored by the same code path that already swaps those three fields on fiber
resume/park — not a second, independent swap site.

### Fix 2 — per-receiver decorator state is Layout-only, enforced by tier assignment

`@computed` is reclassified from Install to **Layout tier**, using the same
reserved-slot mechanism `@lazy` already uses correctly: the memoized value and
its dirty/recompute bookkeeping live in a slot on the receiver itself,
allocated at `finalizeLayout` time, not in a side table owned by the attribute
instance. `decorators-stdlib.md`'s `@computed` example is amended to match
`@lazy`'s shape (`finalizeLayout` reserves a slot; the getter reads and
populates that slot directly on `self`; no `Map`).

This is generalized into a standing rule on the tier-assignment machinery
(`attribute-classes.md`'s tier inference), not just a one-off fix to one
decorator:

> **A decorator hook may not close over a collection keyed by the receiver it
> decorates.** Any decorator needing state that outlives a single call and is
> specific to one receiver must request a reserved slot via the Layout tier.
> This is not enforceable by static analysis — Phalcom is dynamically typed
> with no flow analysis, the same floor-not-proof limitation already accepted
> for the truthiness ban ([ADR-0021](0021-no-truthiness-enforcement.md),
> DEC-C) — so it is enforced as a **written contract on the builtin decorator
> library**, checked in code review and by the golden-test corpus (a snapshot
> test asserting `@computed`/`@lazy`-shaped builtins never retain a
> receiver-keyed side table).

`@memoize`, as given in `decorators-stdlib.md`, does not currently key its
cache by receiver at all — its cache is shared class-wide, which is a
*different* correctness bug (wrong result for a stateful method sharing one
cache across receivers, not a leak) and is tracked separately, not fixed by
this ADR. If a future per-receiver `@memoize` variant is added, it must follow
the same Layout rule as `@computed`.

## Consequences

- **Positive.** `@invariant` re-entrancy now matches its stated intent
  (Eiffel's own-object nesting rule) instead of silently disabling invariant
  checking for any object reached transitively during another object's public
  call — closes a real soundness hole before the feature ships.
- **Positive.** The exit-guard removal is now unwind-safe; a thrown
  precondition/postcondition error, or any other exception, can no longer
  permanently disable invariant checking for the rest of the fiber.
- **Positive.** `@computed`, and any future per-receiver builtin decorator,
  can no longer leak every receiver it has ever been called on — the
  mark-sweep collector ([ADR-0050](0050-non-moving-mark-sweep-collector.md))
  can reclaim these objects normally, same as any other object.
- **Positive.** A fiber that yields from inside an `@invariant`-guarded call no
  longer leaks its in-flight bookkeeping into whatever fiber resumes next, and
  two fibers with concurrently in-flight invariant checks no longer see each
  other's `checking` entries — closed by making `checking` fiber-switch state
  rather than VM-global.
- **Negative / accepted.** The per-fiber `checking` set costs a hash-set
  insert/remove per invariant-checked outermost call, versus a single integer
  increment/decrement in the original (rejected) design, plus one field's
  worth of fiber-switch save/restore alongside the existing `stack`/`frames`/
  `open_upvalues` swap. This sits entirely on the already-non-free `@invariant`
  path — the guard code only exists on
  classes that declare `@invariant` — so it does not tax code that doesn't use
  contracts.
- **Negative / accepted.** Reclassifying `@computed` as Layout-tier means it is
  no longer user-authorable in Phalcom source the way `@memoize`/
  `@synchronized`/etc. are (Layout hooks are compiler/builtin-owned per
  `decorators.md`'s existing user/compiler tier line — this ADR does not
  change that line, it puts `@computed` on the correct side of it, where it
  always belonged once it needs per-receiver storage).
- **Revisit trigger.** If Phalcom later gains weak references or a
  moving/generational collector with finalizers (beyond ADR-0050's
  non-moving mark-sweep), a receiver-keyed side table with *weak* keys
  becomes safe again, and Install-tier per-receiver caching could be
  revisited in a superseding ADR — but only with weak keys, never strong ones
  as given in the original `@computed` draft.

## What this precludes

Nothing new. The receiver-scoped invariant guard is strictly more correct than
the counter it replaces, with no user-visible surface change — the woven code
shape at the call site is unaffected. Confining per-receiver decorator state
to the Layout tier does not remove any capability: the two builtin examples
(`@lazy`, `@computed`) already demonstrate the pattern works for exactly this
use case. It only forecloses a user-authored Install-tier decorator from
holding a receiver-keyed cache — which was already precluded by the leak this
ADR exists to fix, not something newly given up.

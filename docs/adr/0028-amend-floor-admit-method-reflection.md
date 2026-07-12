# 28. Amend the frozen floor — admit the `Method` reflection surface

- Status: Proposed
- Date: 2026-07-12
- Related: [ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) (frozen floor,
  amended here); [ADR-0023](0023-amend-floor-admit-hash-and-kernel-reflection.md)
  (sibling omnibus amendment — pre-cleared the `Method`-reflection capability
  "in principle" for U-CORE-1/3/4/6, `hash` slice already landed at 73 → 80);
  [ADR-0006](0006-function-as-abstract-callable-root.md) (`Function` root,
  `Method`/`Block` siblings); [ADR-0013](0013-closure-upvalues-and-frame-token-return.md)
  (frame-token non-local return, re-entered by `invoke_method_object`);
  [ADR-0010](0010-tagged-value-enum.md) (closed `Value` enum, untouched here);
  `docs/forge/units/U-CORE-3/as-built.md` §2.3, §2.6 (drafted amendment text
  and `BoundMethod` representation decision); `docs/spec/v0.2/core/floor-census.md`
  §1.1, §2.9–§2.10 (re-baselined in the same implementation change as this ADR)

## Context

[ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) froze a permanent
VM-blessed primitive floor. [ADR-0023](0023-amend-floor-admit-hash-and-kernel-reflection.md)
then admitted four capabilities to that floor **in principle** as a single
omnibus ratification — `hash` (U-CORE-1), `Method` reflection (U-CORE-3),
`Number#toString` (U-CORE-4), and `Error#message`/`raise` (U-CORE-6) — so that
none of the four owning units would individually block on its own ADR round.
ADR-0023 was explicit that clearing the gate "in principle" is not the same as
landing: "Each named primitive is still *installed* only when its owning unit
actually lands and bumps the census," and each unit's own floor-census update
"applies its own delta *in lockstep with its own primitive installs*... so the
bump is deliberate and auditable one unit at a time."

U-CORE-1 has since landed, installing `hash` (+ `Behavior#name`/`methods`) and
moving the census 73 → 80. U-CORE-3 now lands the second slice: the `Method`
reflection surface. Per its as-built specification
(`docs/forge/units/U-CORE-3/as-built.md` §2.1–§2.6), every capability in this
surface reads representation **below** the `.ph` boundary — the ADR-0019 §1
derivability test — so none of it can be expressed as ordinary `core.ph`:

| Capability | Why not `.ph` |
|---|---|
| `Object#methodFor(_)` | Needs the resolved `MethodObject` handle via `Value::lookup_method` — no `.ph` primitive exposes it (`respondsTo(_)` returns only a `Bool`). |
| `Method#invokeOn(_,_)` | Needs to run a **specific, already-resolved** `MethodObject` against an explicit receiver through the VM's closure/dispatch machinery (`call_method`/`run_until`) — no `.ph` handle reaches that machinery. |
| `Method#bind(_)` | The `.ph`-expressible form (`{ *args => self.invokeOn(receiver, args) }`) requires variadic block literals, which do not exist yet (`BlockExpr.params` carries no `is_rest`); no fixed-arity `.ph` form is general over an arbitrary method's arity. |
| `Method#selector`, `Method#holder` | Read `MethodObject.signature.selector` / `.holder` directly — fields not exposed to `.ph`. |

As with ADR-0023's other three slices, this ADR is the **per-unit landing
record** for the `Method`-reflection slice: it performs the actual amendment
(adds the bindings to the written floor list) and fixes the concrete count
this unit moves, rather than resting on ADR-0023's in-principle clearance
alone.

**Base-count correction.** The U-CORE-3 as-built draft (§2.6) states the floor
moves "73 → 78," carried over from when it was drafted against ADR-0019's
original baseline. That count is stale: U-CORE-1 landed first and already
moved the base to 80 (73 + 7, ADR-0023 item 1). This ADR's actual delta is
+5 bindings applied to the **80** baseline, i.e. **80 → 85**, not 73 → 78.

## Decision

Amend [ADR-0019](0019-freeze-vm-blessed-primitive-floor.md)'s floor list to
add the **`Method` reflection surface** — 5 new floor bindings:

1. **`Object#methodFor(_)`** — reifies the `MethodObject` that method lookup
   resolves for a given selector `Symbol` on the receiver, returning it as a
   bare `Method` value (or the shared `None` singleton on a miss, ADR-0007).
   Justified because it reads the resolved handle `Value::lookup_method`
   produces, which no floor primitive currently surfaces to `.ph`.
2. **`Method#invokeOn(_,_)`** — applies the reified method to an explicit
   receiver and argument list by driving the VM's closure/dispatch machinery
   directly (`call_method` + a re-entrant `run_until`), without re-dispatching
   by selector. Justified because running **a specific, already-resolved**
   method against an arbitrary receiver is not expressible without that
   machinery.
3. **`Method#bind(_)`** — closes a reified method over a receiver, returning a
   value that `isA(Function)`, reads as a `Block` (ADR-0006), and responds to
   `call` by delegating to the same `invokeOn` engine. Justified because the
   ADR-0006 `.ph` form for this is blocked on variadic block literals (§0.2 of
   the as-built spec); no fixed-arity `.ph` form generalizes over arbitrary
   method arity.
4. **`Method#selector`** — the interned selector `Symbol`, read directly off
   `MethodObject.signature.selector`.
5. **`Method#holder`** — the defining `Class` (or metaclass, for a class-side
   method), read directly off `MethodObject.holder`; the `None` singleton if
   unbound.

**New heap representation (not a new `Value` arm).** `bind(_)` returns a new
`Object` enum variant, `Object::BoundMethod` — a method closed over a
receiver (`{ method: ObjRef, receiver: Value }`) whose surface class is
`Block`. It lives under the existing `Value::Obj` handle, so the closed
`Value` enum (ADR-0010) is untouched; the `Fiber`-arm forward-compat hazard is
not tripped.

**Behavior completions — no new binding.** These extend existing floor
primitives to recognize the two new receiver shapes; they add zero bindings
to the census:

- `block_arity` / `block_name` learn `Object::Method` (reads `signature`) and
  `Object::BoundMethod` (delegates to the wrapped method).
- `resolve_callable` / `block_call` learn `Object::BoundMethod`, dispatching
  it through the same `invoke_method_object` engine as `invokeOn` — before
  `resolve_callable`'s ordinary closure path, since a bound primitive method
  has no `ClosureObject` to resolve.

**Constraints (binding on the implementation, asserted as invariants):**

- `invokeOn(recv, args)` runs the **exact reified method**, with no
  re-dispatch by selector — the caller, not the VM, is responsible for
  receiver compatibility.
- `bound.call(args) ≡ method.invokeOn(recv, args)` for the same
  `(method, recv, args)` (R-INV-3.3) — holds by construction, because both
  paths funnel through the same `invoke_method_object` workhorse.
- An arity mismatch on either `invokeOn` or `bound.call` raises the native
  `RuntimeError::Arity` (R-INV-3.4), checked once, in one place, before the
  call touches the stack — not a truncation or a silently wrong value. (The
  surface `ArgumentError` class this maps to is U-CORE-6's concern; today it
  is the native error.)

**Floor count.** This unit moves the census **80 → 85** (+5), continuing from
U-CORE-1's landed slice of the ADR-0023 ceiling (73 → 80 → … → 88 across all
four units). `floor-census.md` §2.9/§2.10 and the §1.1 running count are
re-baselined to 85 in the same implementation change that lands this surface
(R-INV-0.1) — not by this document alone.

**No other floor move is authorized by this amendment.** The remaining
ADR-0023 slices (`Number#toString`, `Error#message`/`raise`) still land, and
still get re-baselined, under their own units' landing.

## Consequences

- Clears the concrete, auditable amendment for U-CORE-3's slice of the
  ADR-0023 ceiling: the floor list in ADR-0019 now names these 5 bindings
  explicitly, rather than resting on ADR-0023's in-principle admission alone.
- `Method < Function < Object` (per the §4.1 re-parent this unit also
  performs) now answers the full reflective surface — `arity`, `name`,
  `selector`, `holder`, `bind`, `invokeOn` — while deliberately **not**
  answering raw `call` while unbound; that split is a documented, permanent
  semantic, not a gap.
- `Object::BoundMethod` is a new heap variant with surface class `Block`; it
  does not touch the `Value` enum and keeps the callable tower open for a
  future `Fiber` (forward-compat §1) — a bound method carries no frame token
  and is not itself a lexical block.
- `invoke_method_object` re-enters `run_until` through the same frame-token
  infrastructure (ADR-0013) as the existing `send_dynamic`/`block_call`
  re-entrancy, so non-local `return` and `DeadFrameError` fencing continue to
  hold across an `invokeOn`/`bound.call` boundary (R-INV-3.2) without any
  change to that machinery.
- The permanent native surface grows by 5 bindings (80 → 85), continuing the
  trajectory ADR-0023 already authorized as an "at most 88" ceiling; nothing
  here exceeds that ceiling.

## Alternatives considered

- **Treat ADR-0023's in-principle clearance as sufficient; land no further
  document.** Rejected: ADR-0023 itself frames its four items as a ceiling,
  not an install, and calls for each unit's own census bump "in lockstep with
  its own primitive installs." A per-unit landing record keeps the floor list
  in ADR-0019 an accurate, citable statement of what is *actually* native
  today, not merely what has been pre-authorized.
- **Defer `bind` to a later unit, once variadic block literals land, and
  express it as `.ph`** (`{ *args => self.invokeOn(receiver, args) }`), landing
  only `methodFor`/`invokeOn`/`selector`/`holder` here. Considered and
  rejected in the as-built spec (§2.3, SD-3.1): the native `BoundMethod` arm is
  small (~40 lines across three exhaustive-match files), keeps the reflection
  surface coherent in one unit, and makes R-INV-3.3 testable now instead of as
  a forward invariant.
- **Reuse `BlockObject` for `bind`'s return value instead of a new heap arm.**
  Rejected: `BlockObject` wraps a closure and a home-frame token
  (`block.rs`), which a bound **primitive** method (e.g. `3.methodFor(#+).bind(3)`)
  does not have. A dedicated `BoundMethodObject` (method handle + receiver, no
  frame token) is required to cover primitive methods, not just closures.
- **Correct the as-built draft's "73 → 78" count in place rather than noting
  the discrepancy here.** Rejected: the as-built document is a frozen
  specification artifact; this ADR is the authoritative record of the actual
  count applied, and calls out the correction explicitly so the discrepancy
  cannot silently propagate into `floor-census.md`.

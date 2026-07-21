# 36. Amend the frozen floor — admit `Number#toString`

- Status: Accepted (code-confirmed 2026-07-14 — `number_to_string` implemented
  at `primitive/number.rs:88`; the "NN" placeholder in this header was never
  substituted with the correct file number until this pass)
- Date: 2026-07-12
- Related: [ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) (frozen
  floor, amended here); [ADR-0023](0023-amend-floor-admit-hash-and-kernel-reflection.md)
  (sibling omnibus amendment — pre-cleared `Number#toString` "in principle"
  for U-CORE-4 alongside `hash` (U-CORE-1), `Method` reflection (U-CORE-3),
  and `Error#message`/`raise` (U-CORE-6)); [ADR-0028](0028-amend-floor-admit-method-reflection.md)
  (sibling per-unit landing record — same amendment pattern, `Method`
  reflection slice, 80 → 85); [ADR-0015](0015-object-default-tostring.md)
  (`Object#toString` default — re-homed, not re-bound, by this unit);
  [ADR-0005](../retired/0005-number-as-flat-f64.md) (`Number` as a flat `f64`, the
  representation this binding renders); `docs/forge/units/U-CORE-4/as-built.md`
  §2, §6.1 (drafted amendment text and the native/`.ph` split);
  `docs/spec/current/core/floor-census.md` §1.1, §2.1, §2.4 (re-baselined in the
  same implementation change as this ADR)

## Context

[ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) froze a permanent
VM-blessed primitive floor. [ADR-0023](0023-amend-floor-admit-hash-and-kernel-reflection.md)
admitted four capabilities to that floor **in principle** as a single omnibus
ratification — `hash` (U-CORE-1), `Method` reflection (U-CORE-3),
`Number#toString` (U-CORE-4), and `Error#message`/`raise` (U-CORE-6) — so none
of the four owning units would individually block on its own ADR round.
ADR-0023 was explicit that clearing the gate "in principle" is not the same as
landing: each unit's own floor-census update "applies its own delta *in
lockstep with its own primitive installs*."

U-CORE-1 and U-CORE-3 have since landed (73 → 80 → 85). U-CORE-4 now lands the
third slice: per-type `toString`. Per its as-built specification
(`docs/forge/units/U-CORE-4/as-built.md` §2), only **one** capability in this
unit's scope fails the ADR-0019 §1 derivability test — rendering an `f64` as
decimal text:

| Capability | Why not `.ph` |
|---|---|
| `Number#toString` | Reads the receiver's raw `f64` bit representation to render it as text; no `.ph`-visible primitive exposes a number's digits (same derivability failure as `hash`, decisions.md Q1; DEFERRED #19). |

Every other `toString` this unit adds is **derivable** and stays in `core.ph`,
not a floor amendment:

- **`String#toString`** (`=> self`) — a string's display *is* itself, no
  representation read.
- **`Bool#toString`** (over the sacred `ifTrue(_, ifFalse)` selector) —
  derivable over an existing floor selector; non-sacred itself, so it does not
  extend the sacred set (floor-census §5).
- **`Option#toString`** (over the `match(some:none:)` eliminator) — derivable;
  `Some`/`None` inherit it from `Option`.

**`Object#toString` is re-homed, not re-bound.** The existing
`(Object, toString)` binding today aliases `object_name`, returning the
*metaclass* name for a class receiver (DEFERRED F4). This unit points that
same binding at a new, distinct native fn `object_to_string`
([ADR-0015](0015-object-default-tostring.md)'s `"<ClassName>"` default plus
the class-own-name fix) — the binding *set* is unchanged (`(Object,
toString)` already existed), so this is a fn substitution behind an existing
binding, not a new floor amendment. `object_name` itself is untouched and
stays bound to `Object#name`.

## Decision

Amend [ADR-0019](0019-freeze-vm-blessed-primitive-floor.md)'s floor list to
add **one** new floor binding:

1. **`Number#toString`** — renders the receiver's `f64` value as a decimal
   string, delegating to the shared native renderer (`Value::to_string`) so
   `n.toString` is byte-identical to `System.print(n)` (R-INV-4.1) by
   construction. Bound on `Number` (the abstract numeric root), not a
   concrete f64-only path, so a future `Integer`/`Float` split
   (forward-compat §4, [ADR-0024](0024-numeric-surface-split-int-float-and-division.md))
   can refine this per-subclass without breaking dispatch identity.

**Re-home, not a new binding.** `Object#toString` moves from `object_name` to
a new fn `object_to_string`; the `(Object, toString)` binding itself already
existed (ADR-0019's original floor), so this contributes to the
*distinct-native-fn* count but not the *installed-binding* count.

**No other floor move is authorized by this amendment.** The remaining
ADR-0023 slice (`Error#message`/`raise`) still lands, and still gets
re-baselined, under U-CORE-6's own landing.

**Floor count.** This unit moves the census **85 → 86** (+1 binding),
continuing from U-CORE-3's landed slice (80 → 85 → … → 88 across all four
ADR-0023 units — `hash` at 80, `Method` reflection at 85, `Number#toString`
here at 86, `Error#message`/`raise` still pending under U-CORE-6). Distinct
native Rust functions move **69 → 71** (+2: `number_to_string`, plus
`object_to_string` from the `Object#toString` re-home — `object_name` itself
is unchanged and still counted for `Object#name`). `floor-census.md` §1.1,
§2.1, and §2.4 are re-baselined to 86 in the same implementation change that
lands this surface (R-INV-0.1) — not by this document alone.

## Consequences

- Clears the concrete, auditable amendment for U-CORE-4's slice of the
  ADR-0023 ceiling: the floor list in ADR-0019 now names `Number#toString`
  explicitly, rather than resting on ADR-0023's in-principle admission alone.
- Resolves DEFERRED F4: a class receiver's `toString` now returns its own name
  (`Number.toString == "Number"`), not the metaclass's.
- Unblocks DEFERRED #30 (the string-interpolation desugar's `String.new(_)`
  stand-in, ADR-0022): with a real content `toString` on every value type,
  `desugar_string_interp` (`phalcom-ast/src/parser.rs`) *can* switch its
  target to `expr.toString` — a separate, later, `phalcom-ast` follow-up, not
  performed by this ADR or this unit.
- `Value::to_string` (the native print path) is extended to agree with the
  new `None`/`Some`/`List` message renderings (R-INV-4.1); this is a renderer
  change, not a new floor binding — `Value::to_string` is not itself a
  `(class, selector)` dispatch target.
- The permanent native surface grows by 1 binding (85 → 86), continuing the
  trajectory ADR-0023 already authorized as an "at most 88" ceiling; nothing
  here exceeds that ceiling.

## Alternatives considered

- **Treat ADR-0023's in-principle clearance as sufficient; land no further
  document.** Rejected, for the same reason ADR-0028 rejected it: a per-unit
  landing record keeps the floor list in ADR-0019 an accurate, citable
  statement of what is *actually* native today.
- **Give `String`/`Bool`/`Option` native `toString` bindings too, for
  uniformity with `Number`.** Rejected: none of the three fail the ADR-0019
  §1 derivability test (none reads representation below the `.ph` boundary),
  so a native binding would violate the floor's "default answer is no" rule
  for no gain — each is a two-to-four-line `core.ph` body over an existing
  floor selector.
- **Give `Number` a concrete-`f64`-only `toString`, hard-coding float
  formatting.** Rejected per forward-compat §4: binding on the abstract
  `Number` root and delegating to the value-generic `Value::to_string`
  renderer keeps a future `Integer`/`Float` split additive (each subclass
  overrides/inherits without a dispatch-identity break); a concrete-only
  binding would need to move at split time.

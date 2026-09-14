# 23. Amend the frozen floor — admit `hash`, kernel reflection, `Number#toString`, and `Error#message`/`raise`

- Status: Accepted
- Date: 2026-07-12
- Related: [ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) (frozen floor,
  amended here); [ADR-0006](0006-function-as-abstract-callable-root.md)
  (`Function` root); [ADR-0008](0008-layered-exceptions-and-result.md) (layered
  exceptions/`Result`); `docs/forge/units/U-CORE-0/decision-register.md` Q1 (hash), §4.1 (`Method`
  superclass); `docs/spec/current/core/U-CORE-1-implementation-spec.md` §2.2–2.3;
  `docs/spec/current/core/U-CORE-3-implementation-spec.md` §2.6;
  `docs/spec/current/core/U-CORE-4-implementation-spec.md` §6.1;
  `docs/spec/current/core/U-CORE-6-implementation-spec.md` (ADR-0019 amendment section)

## Context

[ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) fixed a permanent
VM-blessed primitive floor at 73 bindings: everything at or below it is native
Rust forever; everything above it is `.ph`. A method may move *up* (native →
`.ph`) freely, but moving *down* (adding a new native binding) requires "a new
superseding ADR that amends this list" — a deliberate one-way door, not an
ordinary commit.

The `docs/spec/current/core/` U-CORE-0 requirements pass (floor census, catalog delta,
gating decisions) independently identified **four** capabilities that fail
ADR-0019 §1's derivability test — each reads representation or identity below
the `.ph` boundary that no existing floor primitive exposes:

1. **`hash`** (U-CORE-1) — `Map`/`Set`'s key precondition; needs the heap
   handle, an `f64`'s bit pattern, a `String`'s bytes, or a `Symbol`'s interned
   id, none of which `.ph` can read.
2. **`Method` reflection** (U-CORE-3) — `Object#methodFor(_)`, `Method#bind(_)`,
   `Method#invokeOn(_,_)`, `Method#selector`, `Method#holder`; needs the
   resolved `MethodObject` handle and the closure/dispatch machinery.
3. **`Number#toString`** (U-CORE-4) — the value-content stringify path; today
   `toString` is the inherited `Object` default (class name), and the native
   digit-formatting logic lives below `.ph`.
4. **`Error#message`/`raise`** (U-CORE-6) — `raise` initiates a stack unwind
   (produces a `PhError` unwind payload no `.ph` construct can yield);
   `message` is a native accessor mirroring `Message`'s existing accessor
   family.

Each was independently drafted as its own "amend ADR-0019" note inside its
owning unit's implementation spec. Opening four separate superseding ADRs
against the same frozen list is unnecessary churn and risks the amendments
drifting out of sync (e.g. disagreeing on the resulting floor count). The
`docs/spec/current/core/README.md` cross-spec integration notes flagged this and
called for **one omnibus amendment** instead. (Numbering note: the individual
specs were drafted before U-LEX claimed ADR-0022 for string interpolation, so
some cite "ADR-0022" for this amendment — 0023 is the correct, current number.)

## Decision

Adopt this **single omnibus amendment** to [ADR-0019](0019-freeze-vm-blessed-primitive-floor.md),
admitting all four capabilities to the floor **in principle**. Each named
primitive is still *installed* only when its owning unit actually lands and
bumps the census — this ADR clears the ADR-0019 gate for all four so no unit
blocks on a separate ratification round.

**Additions to the floor:**

1. **`hash`** (owner: U-CORE-1) — `Object#hash` (identity digest of the heap
   handle) plus per-immediate overrides on `Number`, `String`, `Symbol`,
   `Bool` (value digests). Constraint: `a == b ⇒ a.hash == b.hash`
   (R-INV-1.3); `Number#hash` digests the *mathematical value*, class-agnostic,
   so a future Int/Float split keeps `2` and `2.0` hashing equal
   (forward-compat §4). Also admits `Behavior#name` (a class's own name,
   shadowing `Object#name`'s metaclass-name result for class receivers) and
   `Behavior#methods` (own method-dictionary selectors, as `Symbol`s) — both
   underivable for the same reason (they read the `ClassObject` name/method-map
   fields). **+7 bindings.**
2. **`Method` reflection** (owner: U-CORE-3) — `Object#methodFor(_)`,
   `Method#invokeOn(_,_)`, `Method#bind(_)`, `Method#selector`,
   `Method#holder`. Also introduces one new heap representation,
   `Object::BoundMethod` (surface class `Block`), as `bind(_)`'s return value —
   not a new `Value` arm. Constraint: `invokeOn(recv, args)` runs the exact
   reified method with no re-dispatch, and `bound.call(args) ≡
   method.invokeOn(recv, args)` (R-INV-3.3); an arity mismatch raises
   `RuntimeError::Arity` (R-INV-3.4). **+5 bindings.**
3. **`Number#toString`** (owner: U-CORE-4) — a native content-stringify getter
   on `Number`, distinct from and consistent with the existing native
   `Value::to_string` print-path (they must agree; the invariant is agreement,
   not merging — decisions.md §4.4). **+1 binding.**
4. **`Error#message`/`raise`** (owner: U-CORE-6) — `Error#message` (getter,
   native slot-0 accessor) and `Error#raise` (initiates the unified unwind,
   producing a `RuntimeError::Raise` payload — plumbing, not itself a bound
   selector). **+2 bindings.**

**Floor count.** Cumulative, if all four units land as specified: **73 → 88**
(+7 +5 +1 +2). Each unit's own floor-census update (`floor-census.md` §1.1/§2)
applies its own delta *in lockstep with its own primitive installs* — this ADR
authorizes the ceiling, it does not itself move the census; an implementer
must still land the R-INV-0.1 audit bump alongside the code, per each unit's
spec §5.5, so the bump is deliberate and auditable one unit at a time.

**No other floor move is authorized by this amendment.** A capability not
listed above still needs its own superseding ADR.

## Consequences

- Clears the ADR-0019 gate for **U-CORE-1, U-CORE-3, U-CORE-4, and U-CORE-6**
  in one ratification instead of four, so none of them individually blocks on
  a separate ADR round when the orchestrator dispatches them.
- The floor grows from 73 to (at most) 88 bindings across the four units — a
  ~20% increase to the permanent native surface. This is deliberate: each
  addition passed the ADR-0019 §1 derivability test on its own merits (see the
  per-unit justifications above and in each implementation spec), and the
  floor still excludes everything ADR-0019 named as "above the floor" —
  collections, higher-order string manipulation, Option/Result combinators,
  `Message.args`, rest-parameter collection beyond what's already landed.
- Because this is a ceiling, not a simultaneous install, `floor-census.md`
  will show partial progress toward 88 as units land one at a time (e.g. 73 →
  80 after U-CORE-1 alone). The R-INV-0.1 census audit (kernel reflection's
  own invariant-harness unit, U-CORE-1) is the backstop that catches any
  accidental extra primitive slipping in under this ceiling.
- `Method#bind(_)`'s `Object::BoundMethod` representation is a new heap
  variant, not a new `Value` arm — it does not touch the `Value` enum's
  forward-compat openness constraint (forward-compat §1).

## Alternatives considered

- **Four separate superseding ADRs**, one per unit. Rejected: each targets the
  same frozen list in the same document; splitting them risks the floor-count
  arithmetic drifting out of sync across four independently-reviewed documents,
  and gives no benefit a single document with four numbered items doesn't
  already provide.
- **Defer ratification until each unit is about to land**, ratifying
  piecemeal. Rejected: the U-CORE-0 requirements pass already did the
  derivability analysis for all four; re-litigating each at dispatch time adds
  a needless per-unit gate with no new information expected between now and
  then. This ADR ratifies the analysis once; per-unit landing still requires
  the unit's own green-gate implementation and census bump.

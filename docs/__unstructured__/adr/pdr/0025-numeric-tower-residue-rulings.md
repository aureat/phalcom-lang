# PDR-0025 — Numeric tower residue: `~/` is total over the tower and returns `Int`; construction never narrows; the `Bool` arm dies

- Status: **Accepted** (ratified 2026-07-21)
- Discharges: [PDR-0012](0012-numeric-tower-implementation-and-floor-amendment.md) open
  questions **Q-1** and **Q-2** (both flagged there as "genuinely unruled by ADR-0024" and
  blocking the primitives phase)
- Related: [ADR-0024](../adr/accepted/0024-numeric-surface-split-int-float-and-division.md)
  §5/§6 (whose contradiction Q-1 names), PDR-0012 rulings 3 (`normalize`), 11 (zero divisor),
  18 (`new` re-homing), 20 (the floor figure this record leaves unchanged),
  [ADR-0021](../adr/accepted/0021-no-truthiness-enforcement.md) (the posture ruling 4's
  discovery offends), [PDR-0020](0020-bitwise-operations-on-int.md) (`>> ≡ ~/ 2ⁿ` inherits
  ruling 1's semantics on the `Int` side; unaffected otherwise)
- No floor change; no new dependencies; no new tokens. Lands **inside** the tower unit —
  no separate unit.

## Context

PDR-0012 ratified the tower but left two questions it refused to guess at, both of which the
implementing agent hits in the primitives phase: whether `7.5 ~/ 2` is legal (ADR-0024 §5
says `~/` "returns an exact `Int`" unconditionally; §6 says any `Float` operand contaminates
to `Float` — both cannot hold), and what `Int.new(2.7)` does (the one place a user can
request a lossy narrowing). Reading the current constructor to answer the second surfaced a
third, unrecorded fact: `number_class_new` (`primitive/number.rs:22-44`) accepts a **`Bool`**
argument — `Number.new(true) == 1` — an arm no ADR mentions and the re-homing must decide
about rather than copy blind.

## Rulings

1. **`~/` is defined on both `Int` and `Float`, over all four operand combinations, and
   always returns `Int` with floor semantics.** It is the *stated exception* to ADR-0024
   §6's contamination rule, and the exception is the operator's purpose: `~/` means "the
   exact integer of this division"; a `Float`-returning `~/` would be `/`-plus-`floor` with
   extra steps. Precedent with cost: Dart's `~/` on `num` returns `int` (adopted); Python's
   `//` on floats returns a *float* — rejected, because the result then isn't exact and
   Python's own documentation routes users to `math.floor` to get the integer, which is the
   admission.

2. **`~/` raises wherever an exact `Int` does not exist.** Uniformly, on every combination:
   zero divisor (extends PDR-0012 ruling 11 from the `Int` path); non-finite operand
   (`nan`, `±inf`); **non-finite quotient** (e.g. `1.0e308 ~/ 1.0e-308` overflows the IEEE
   division before flooring). One rule, three triggers, one error shape.

3. **The `Float`-side result converts exactly, through `normalize`.** The floor of a finite
   `f64` is an integer exactly representable as a `BigInt`; conversion must not round-trip
   through `i64` (`1.0e300 ~/ 1.0` is a legal ~997-bit result). PDR-0012 ruling 3's demotion
   invariant applies to the result like any other. The single IEEE rounding inside the
   division itself is inherent to `Float` arithmetic and accepted — the spec text should say
   so, so nobody "fixes" it with a second rounding.

4. **Construction: `Int.new` and `Float.new` re-derive their acceptance sets; they do not
   inherit `number_class_new`'s.** Per kind:
   - `Int.new(2.7)` **raises** — Q-2's recommendation confirmed. *Integral* `Float`s raise
     too (`Int.new(2.0)` is an error): value-dependent acceptance would make `Int.new(x)` a
     runtime coin-flip, which is "silently wrong" wearing a validity check. Narrowing is
     spelled on the `Float` side (`truncated`/`rounded`/`floor`), owed by the Float-protocol
     record (spec queue item 4) — this ruling *creates* that demand and names it.
   - `Float.new(anyInt)` **is legal, IEEE round-to-nearest** — widening loses precision
     above 2⁵³ and that is accepted: an explicit constructor *is* the explicit conversion
     door, and this is the exact mirror of `/`'s ratified promotion. Every precedent
     language (Python `float(10**100)`, Ruby, Dart) agrees; none warns.
   - `Int.new(string)` parses an integer (i64 fast path, `BigInt` beyond — `Int` is
     unbounded in its parser too); `Float.new(string)` parses an `f64`; malformed raises,
     keeping `number_class_new`'s error shape but with ruling 18's `arg.type_name()` debt
     fixed in the same pass.
   - `Int.new(intVal)` / `Float.new(floatVal)` are identity; cross-class numeric arguments
     follow the two rules above (`Int.new(floatVal)` raises; `Float.new(intVal)` rounds).
   - **The `Bool` arm is dropped, not re-homed.** `Number.new(true) == 1` exists in code
     and in no record; it is a truthiness-adjacent coercion in a language that ratified
     truthiness *out* (ADR-0021 spent two enforcement layers keeping `Bool` and non-`Bool`
     apart; a blessed `Bool`→number door undercuts the posture from the constructor side).
     If demand ever materializes it belongs as an explicit `Bool`-side selector, its own
     record. **This is a behavior change** and gets its own negative-lane golden pinning
     the removal — it must not vanish silently inside the re-homing.

5. **Floor arithmetic: unchanged.** PDR-0012 ruling 20's **153** already priced Q-1 = yes
   (`Float#~/` included; its stated alternative was "152 if `Int`-only"), and ruling 18's
   +4 `new` bindings are untouched — ruling 4 changes acceptance *predicates*, not the
   binding count. Nothing for the census beyond what PDR-0012 already carries.

## Consequences

- The tower unit's primitives phase is unblocked: both blocking residue questions have
  answers, and the third (undocumented `Bool` arm) is decided instead of discovered
  mid-implementation.
- `7.5 ~/ 2 == 3` (an `Int`); `-7.5 ~/ 2 == -4` (floor, consistent with `Int` path and `%`).
- `Number.new(true)` stops working the day the split lands — release-notes-worthy, pinned
  by golden.
- The Float-protocol record inherits a concrete obligation: `truncated`/`rounded`/`floor`
  are no longer nice-to-haves but the sanctioned spelling of a door this record closed.

## Alternatives rejected

- **`~/` as `Int`-only** (PDR-0012's 152 branch) — leaves ADR-0024 §5's unconditional
  wording false and gives `Float` users no exact-division spelling at all.
- **`~/` on `Float` returning `Float`** (Python `//`) — ruling 1's rejection; not exact.
- **`Int.new(2.0)` accepted (integral-only narrowing)** — ruling 4's coin-flip argument.
- **Keeping the `Bool` arm for compatibility** — compatibility with behavior no record
  ever promised is not compatibility; it is accretion.

## Verified vs assumed

**Verified this session (HEAD `999004a`):** `number_class_new` full body
(`primitive/number.rs:22-44` — the `Bool` arm, the f64 string-parse, the `found: "value"`
debt); `new/0` and `new/1` both registered to it (`universe/primitives.rs`, static, on
`Number.class`); PDR-0012's Q-1/Q-2 text and ruling 20's 152/153 branch; clean tree at
write time (U-TRACE T1/T2 chain committed through `999004a`).

**Assumed:** `num-bigint`'s exact `f64 → BigInt` conversion for finite values (ruling 3's
mechanism; standard, but the implementing agent should use the crate's checked conversion,
not a cast). Everything else in this record is a decision, not a claim.

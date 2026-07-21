# PDR-0027 — Numeric semantics completion

- Status: **Accepted** (ratified 2026-07-21)
- Depends on: [PDR-0012](0012-numeric-tower-implementation-and-floor-amendment.md),
  [PDR-0025](0025-numeric-tower-residue-rulings.md), and
  [PDR-0026](0026-numeric-literals.md)
- Amends: [ADR-0019](../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md):
  `NEW_FLOAT_PROTOCOL = 10`, `NEW_NUMERIC_POWER = 2`; recompute the live census, never add
  prose totals
- Supersedes: PDR-0012 ruling 14's temporary acceptance of an integral `Float` from `hash`
- Specs: [tower](../spec/current/numbers/numeric-tower.md),
  [Float protocol](../spec/current/numbers/float-protocol.md), and
  [numeric text and errors](../spec/current/numbers/text-and-errors.md)

## Decision

1. **`Number` is allocator-abstract.** Every class allocation path, including `new` sent through
   reflection, rejects `Number` with `Error.kind == #abstractClass`. This is a class metadata /
   allocator check, not an overridable `Number.new` method. Lookup remains ordinary:
   `Number.respondsTo(#new)` is true if inherited lookup finds it, but invocation raises.
   `Int` and `Float` are concrete; both are `isA(Number)`.

2. **Float follows IEEE-754 binary64.** Arithmetic uses host binary64 operations without an
   implicit fused operation. Overflow, underflow, subnormals, signed zero, and infinities follow
   IEEE behavior. Public `==`, `!=`, `<`, `<=`, `>`, and `>=` use IEEE comparison: every ordered
   comparison involving `NaN` is false, `NaN != NaN` is true, and `+0.0 == -0.0` is true.
   `NaN` has no public ordering. Payload bits are not surface state.

3. **Map/Set key equivalence is deliberately narrower than `==`.** Numeric keys use internal
   SameValueZero-style equivalence: all NaNs are one key class; signed zeroes are one key class;
   equal integral `Int`/`Float` values are one key class. Their hashes match. This relation is
   internal to keyed collections; it does not change ordinary equality or expose a total order.
   `hash` must return `Int`; any other return, including integral `Float`, raises `#invalidHash`.

4. **Ratify Float classification and explicit narrowing.** `abs`, `sign`, `floor`, `ceil`,
   `truncated`, `rounded`, `isInteger`, `isNaN`, `isFinite`, and `isInfinite` are the protocol.
   Narrowing returns exact `Int` through `BigInt` normalization; non-finite inputs raise.
   `rounded` ties away from zero. The ten native bindings are Float-only; Int behavior is
   derivable in `core.ph`.

5. **Add `**` as right-associative power.** It is a normal overridable selector `**(_)`.
   Its grammar gives Python-compatible unary binding: `-2 ** 2` is `-(2 ** 2)` and
   `2 ** -2` is `2 ** (-2)`. `Int ** nonnegative Int` is exact and may produce `LargeInt`;
   a negative integral exponent returns `Float`; any Float operand uses binary64 power.
   `0 ** negative` raises `#divideByZero`. No `**=` is added.

6. **String construction and rendering are strict and canonical.** Constructors reject leading
   or trailing whitespace and radix prefixes; optional leading sign applies to finite decimal
   forms. `Int.new` accepts decimal integer text only. `Float.new` accepts decimal numeric text
   plus exactly `NaN`, `Infinity`, and `-Infinity`. Output is locale-independent and round trips
   as specified by the numeric text spec.

7. **Numeric failures are structured rich diagnostics.** They are `Error` values with stable
   `kind` Symbols and stable message templates. When the running instruction has a module source
   span, the existing `RuntimeError::Raise` traceback path must carry it to the innermost caret:
   primary label on the failing operator or constructor argument; a secondary label only where it
   materially identifies another operand. No source is invented for native, REPL, or generated
   frames without a span.

8. **Bound resources explicitly.** Exact integer allocation and exact-power work consume a VM
   numeric-resource budget; exhaustion raises `#numericLimit` rather than aborting or exhausting
   memory. The initial defaults and tuning policy are deferred, not the guard.

## Rationale and rejected precedents

- IEEE makes NaN unordered because it denotes an invalid/indeterminate numeric result, not one
  particular number. Making `NaN == NaN` true would violate IEEE comparisons and makes generic
  arithmetic less predictable. JavaScript's collection behavior remains useful only internally:
  `Map` can retrieve a NaN key without redefining language equality.
- Python's power precedence and negative-exponent result rule make algebraic expressions readable;
  Python's permissive whitespace parsing does not. Phalcom chooses strict constructors so input
  validation has one grammar and one failure point.
- Ruby's prefix-friendly `Integer` parsing is rejected: source literals already own radix
  prefixes; accepting them in strings would create a second, less visible literal grammar.
- Dart's `~/` established a recognizable integer-division spelling. Phalcom retains it because
  `//` is a line comment and because PDR-0025 already assigns it floor-division semantics.

## Consequences and preclusions

- `Number.new` cannot be bypassed through selector replacement or reflection.
- There is no public `totalOrder`, NaN payload API, numeric serialization format, or extended
  numerical-method family in this record.
- `toString` is display text, not an interchange format.
- Collection/index APIs keep their transitional integral-Float acceptance until their dedicated
  follow-on; the numeric tower itself does not depend on it.
- Large compile-time constants remain gated on GC-root verification before implementation ships.

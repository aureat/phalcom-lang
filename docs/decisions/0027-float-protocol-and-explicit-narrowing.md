# PDR-0027 — Float protocol: classification and explicit narrowing

- Status: **Proposed** (2026-07-21)
- Depends on: [PDR-0012](0012-numeric-tower-implementation-and-floor-amendment.md) and
  [PDR-0025](0025-numeric-tower-residue-rulings.md)
- Amends: [ADR-0019](../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md):
  `NEW_FLOAT_PROTOCOL = 10` native `Float` bindings
- Spec: [`docs/spec/v0.2/core/float-protocol.md`](../spec/v0.2/core/float-protocol.md)

## Proposed decision

1. **`Int` and `Float` expose one Number-facing protocol.** Its selectors are `abs`, `sign`,
   `floor`, `ceil`, `truncated`, `rounded`, `isInteger`, `isNaN`, `isFinite`, and
   `isInfinite`. `Int` implementations are exact identities, simple predicates, or
   `.ph`-derivable arithmetic; the ten `Float` implementations are native. `Number` remains
   abstract and keeps zero bindings, as PDR-0012 requires.

2. **All four narrowing selectors return exact `Int`.** On a finite `Float`, `floor`, `ceil`,
   `truncated` (toward zero), and `rounded` convert through the tower's `normalize(BigInt)`
   path. They raise on `NaN` and infinities. This is the explicit conversion door PDR-0025
   requires after making `Int.new(Float)` always raise.

3. **`rounded` uses nearest value, ties away from zero.** Thus `1.5.rounded == 2` and
   `(-1.5).rounded == -2`. This preserves the intended behavior in the existing pending
   Number fixtures while stating the tie rule they omit.

4. **Classification is IEEE-aware and total.** `isNaN`, `isFinite`, and `isInfinite` return
   `Bool`; all return fixed answers on `Int`. `isInteger` is true exactly for finite `Float`
   values with no fractional part, including both signed zeroes. `abs` preserves the concrete
   numeric class (`Int -> Int`, `Float -> Float`); `sign` returns `Int` `-1`, `0`, or `1`, and
   raises for `NaN` because no ordered sign exists.

5. **Names are intentional.** Use `isNaN`, not legacy Wren-port `isNan`; use result names
   `truncated` and `rounded`, not `truncate` and `round`. `floor` and `ceil` stay conventional.
   The unimplemented pending fixtures are changed with the implementation; they have not
   established compatibility.

6. **The native floor delta is +10, all on `Float`.** The `Int` half is defined in `core.ph`;
   no selector lives on abstract `Number`. Recompute the live census when PDR-0012 lands,
   then add `NEW_FLOAT_PROTOCOL = 10`; do not add guessed totals to prose.

## Rationale

PDR-0025 deliberately closes the implicit narrowing door in `Int.new`. A Float value still
needs a clear, exact route to `Int`; four named choices make loss policy visible at each call
site. Keeping the same vocabulary on `Int` avoids numeric type tests in generic code while
keeping the VM floor at the Float-only semantic boundary. The acronym spelling matches the
IEEE term and avoids treating `NaN` as ordinary camel-case prose.

## Consequences

- `Int.new(2.0)` remains an error; `2.0.truncated` is the explicit conversion.
- `Float#floor` and siblings never return a rounded `f64`; large finite results may become
  `LargeInt` through `normalize`.
- Non-finite classification remains observable, while non-finite-to-`Int` conversion is not.
- The existing `number_is_nan`, `number_is_infinity`, `number_round`, and `number_truncate`
  pending Wren ports require migration rather than promotion unchanged.

## Alternatives rejected

- **Keep the Wren spellings.** Rejected: they conflict with PDR-0025's already named
  `truncated`/`rounded` conversion family and obscure the established `NaN` acronym.
- **Make `Int.new(Float)` the conversion API.** Rejected by PDR-0025: it makes value loss look
  like construction and creates value-dependent acceptance.
- **Return `Float` from rounding selectors.** Rejected: the selectors are an explicit exact
  narrowing API, not a formatting operation.
- **Bind the entire protocol twice in Rust.** Rejected: `Int` behavior is derivable and does
  not justify an additional ten frozen primitive slots.

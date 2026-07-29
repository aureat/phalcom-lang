# PDR-0020 — Bitwise operations on `Int`: infinite two's complement, operator selectors, no wrapping

- Status: **Accepted** (ratified 2026-07-21)
- Amends: [ADR-0019](../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md) (floor +10 on `Int`)
- Depends on: [PDR-0012](0012-numeric-tower-implementation-and-floor-amendment.md) and [PDR-0025](0025-numeric-tower-residue-rulings.md); every selector here lives on unimplemented `Int`
- Spec: [`docs/spec/current/bitwise.md`](bitwise.md)

## Decision

1. **Bitwise uses infinite two's complement over abstract, unbounded `Int`.** `x << n` is exact multiplication by `2^n`; `x >> n` is floor division by `2^n`, hence arithmetic right shift; `~x == -x - 1`. AND, OR, and XOR use the corresponding infinite-two's-complement functions. No width, wraparound, unsigned right shift, or `leadingZeros` exists on `Int`.

2. **Six operations are ordinary operator selectors, not aliases for named selectors.** The installed selectors are `&(_)`, `|(_)`, `^(_)`, `~()`, `<<(_)`, and `>>(_)`. Surface syntax sends those exact selectors; `a & b` dispatches `&(_)`, including user overrides. `bitAnd(_)`, `bitOr(_)`, `bitXor(_)`, `bitNot`, `shl(_)`, and `shr(_)` do not exist.

3. **Four named queries complete the family:** `bitAt(_)`, `bitCount`, `bitLength`, and `trailingZeros`. All ten selectors are native `Int` floor bindings. `bitAt(_)` is sign-aware; `bitCount` and `bitLength` use `|x|`; `0.trailingZeros` raises.

4. **Operator tokens land with this record.** Maximal munch prefers `~/`, `<<`, and `>>` over their prefixes. `~` is unary only; `&`, `|`, and `^` are binary only. The precedence ladder is multiplicative → additive → shifts → `&` → `^` → `|` → comparison/equality → `and`/`or`, so `flags & mask == 0` groups as `(flags & mask) == 0`. No compound assignments (`&=`, `|=`, `^=`, `<<=`, `>>=`) land here.

5. **Errors are catchable `ArgumentError`s.** Negative shift counts, non-`Int` binary operands, non-`Int` bit indexes, and `0.trailingZeros` raise. Allocation failure from a pathological left shift becomes a defined Phalcom error, never a process abort; no proactive magic shift ceiling.

## Rationale

`Int` is one public exact type with hidden small and large representations. Python, Ruby, Haskell, and Smalltalk use infinite two's complement for this model; a fixed-width operation would invent a width Phalcom does not have. Floored `%` from PDR-0012 is load-bearing: it makes the mathematical digit recursion correct for negative operands. Bitwise joins the floor under ADR-0019's arithmetic-family standing: digit-recursion emulation does not make a kernel arithmetic operation suitable for `.ph`.

Symbolic selectors match existing arithmetic (`+(_)`, `*(_)`) and preserve the one-selector/one-dispatch-key invariant. Named aliases would double public surface and make reflection and overrides ambiguous. The cost is reserving six punctuation spellings before future patterns or block syntax can use them; arithmetic has record-backed demand now, so it wins that reservation.

## Consequences

- `Float` has no bitwise selectors; no implicit int32 coercion.
- Fixed-width/wrapping operations, `ushr`, `leadingZeros`, and bulk `Bytes` bitwise remain deferred. A future fixed-width design must bring measured demand.
- PDR-0012's tower lands first. This record adds `NEW_BITWISE = 10`; recompute the census at implementation rather than trusting prose totals.
- Required conformance covers Python-oracle rows for signs, zero, `i64` seams, and deep `LargeInt`; identities; promotion/demotion; error lanes; symbolic selector literals (`#&`, `#~`); and precedence/maximal-munch goldens.

## Alternatives rejected

- **Named `bitAnd`/`shl` selectors plus infix sugar.** Rejected: two public spellings for one operation weaken selector identity and reflection.
- **Bare `and`/`or`/`not`.** Rejected: clashes with Bool vocabulary and sacred control-flow selectors.
- **Fixed-width or wrapping semantics.** Rejected: abstract `Int` has no width; no demand survives measurement.
- **Deferred operator tokens.** Rejected: user-ratified operator selectors must be directly definable, reflected on, and sent now.

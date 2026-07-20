# Bitwise operations on `Int` — surface specification

**Status: DRAFT — normative upon ratification of
[PDR-0020](../../../decisions/0020-bitwise-operations-on-int.md).** Written 2026-07-20, grounded
at HEAD `617021a`. Do not build from this document until the PDR is Accepted.

This spec depends on the numeric tower
([PDR-0012](../../../decisions/0012-numeric-tower-implementation-and-floor-amendment.md),
Accepted, unimplemented; implementation spec [`numeric-tower.md`](numeric-tower.md)): every
selector here lives on `Int`, which does not exist in the tree yet. The owning unit
(**U-BITWISE**) is therefore gated on the tower's unit landing first.

Related: [ADR-0024](../../../adr/accepted/0024-numeric-surface-split-int-float-and-division.md)
(the split), [ADR-0019](../../../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md) (the
floor this amends), [`drafts/stdlib-catalog.md`](../drafts/stdlib-catalog.md) §0.2 (the sketch
this supersedes and partly corrects),
[`../../../analyses/hashbrown-in-phalcom.md`](../../../analyses/hashbrown-in-phalcom.md) §5
(the wrapping question this deliberately does *not* answer).

---

## 0. Scope and non-goals

In scope: bitwise selectors on `Int`, defined over the mathematical integer — no width, no
wrapping, no representation exposure.

Out of scope, each deliberately (§4): operator tokens; fixed-width/wrapping operations;
width-relative queries (`ushr`, `leadingZeros`); bitwise on `Float`; bulk bitwise on `Bytes`.

## 1. Semantic model: infinite two's complement

`Int` is exact and unbounded (ADR-0024 §2). The only coherent bitwise semantics over unbounded
integers is **infinite two's complement**: a non-negative integer has infinitely many leading
`0` bits, a negative integer infinitely many leading `1` bits. Precedent: Python `int`, Ruby
`Integer`, Haskell `Data.Bits Integer`, Smalltalk `LargeInteger` — every language with an
unbounded integer and bitwise operations chose this model; none invented a second one.

The operations are *defined by ratified arithmetic*, not alongside it:

- `x.shl(n)` ≡ `x * 2.pow(n)` — exact, never overflows (promotes).
- `x.shr(n)` ≡ `x ~/ 2.pow(n)` — **floor** division, so the shift is arithmetic
  (sign-preserving): `(-1).shr(k) == -1` for all `k ≥ 0`. This inherits PDR-0012's `~/`
  semantics rather than restating them.
- `x.bitNot` ≡ `-x - 1` — the involution `x.bitNot.bitNot == x` follows.
- `bitAnd`/`bitOr`/`bitXor` are the unique functions satisfying the digit recursion
  `f(a, b) = 2 * f(a ~/ 2, b ~/ 2) + g(a % 2, b % 2)` with `f(0,0)=0`, `f(-1,-1)` per op,
  where `g` is the boolean table on the low bits.

**A load-bearing interaction, worth naming:** the digit recursion is well-defined for negative
operands *only because* PDR-0012 ruling 10 made `Int#%` **floored** — `a % 2 ∈ {0, 1}` for all
`a`. Under truncating `%` (Rust's native `i64` behavior) the recursion produces garbage on
negatives. The floored-`%` ruling, made for the `~/`/`%` identity, is what makes bitwise
*arithmetically definable* here. This is also the spec's `.ph`-expressibility proof (§6).

### 1.1 Laws (conformance identities)

For all `Int` x, y and `n, m ≥ 0`:

1. `x.bitAnd(y).bitOr(x.bitXor(y)) == x.bitOr(y)`
2. De Morgan: `x.bitAnd(y).bitNot == x.bitNot.bitOr(y.bitNot)`
3. `x.bitAnd(y) + x.bitOr(y) == x + y`
4. `x.bitXor(y) == x.bitOr(y) - x.bitAnd(y)`
5. `x.shl(n).shl(m) == x.shl(n + m)`; `x.shl(n).shr(n) == x`
6. `x.bitNot == 0 - x - 1`; `x.bitNot.bitNot == x`
7. Demotion invariant (PDR-0012 ruling 3) holds on every result: no operation may return a
   `LargeInt`-backed value in `i64` range.

## 2. Selector surface

Ten selectors, all instance methods on `Int`:

| Selector | Result | Definition | Errors |
|---|---|---|---|
| `bitAnd(_)` | `Int` | §1 recursion, AND | non-`Int` argument raises |
| `bitOr(_)` | `Int` | §1 recursion, OR | non-`Int` argument raises |
| `bitXor(_)` | `Int` | §1 recursion, XOR | non-`Int` argument raises |
| `bitNot` | `Int` | `-x - 1` | — |
| `shl(_)` | `Int` | `x * 2^n`, exact | `n < 0` or non-`Int` raises |
| `shr(_)` | `Int` | `x ~/ 2^n`, floor/arithmetic | `n < 0` or non-`Int` raises |
| `bitAt(_)` | `Bool` | bit `i` of the infinite two's-complement form; `x.shr(i) % 2 == 1` | `i < 0` or non-`Int` raises |
| `bitCount` | `Int` | population count of `|x|` (Python `int.bit_count` semantics) | — |
| `bitLength` | `Int` | bits needed for `|x|`; `0.bitLength == 0` (Python `int.bit_length`) | — |
| `trailingZeros` | `Int` | largest `k` with `x % 2.pow(k) == 0`; identical for `x` and `-x` | `x == 0` raises |

Notes:

- `bitCount`/`bitLength` are **magnitude** queries. Under infinite two's complement a negative
  integer has infinitely many `1` bits, so a sign-aware popcount does not exist; Python's
  magnitude convention is adopted verbatim so Python is usable as a conformance oracle (§7).
- `bitAt(_)` **is** sign-aware: `(-1).bitAt(k) == true` for every `k ≥ 0`. The table's identity
  makes it derivable, so its presence is ergonomic, not semantic.
- `trailingZeros` raises on `0` rather than returning a width (Rust returns 64 — a
  width-relative answer unavailable here) or `None` (an `Option` in an arithmetic chain
  poisons every downstream send for a case that is almost always a logic error upstream).
- No argument coercion anywhere: `x.bitAnd(3.0)` raises even though `3.0` is integral —
  "integers are never silently wrong" (ADR-0024) applied on the way *in*, consistent with
  PDR-0012 Q-2's recommended `raise`.

## 3. Errors and resource posture

- All argument errors are catchable `ArgumentError`s naming selector, expected, and found —
  the `Bytes` kernel-class house pattern (`core.ph:1206ff`).
- `shl` with a pathological count (`1.shl(10.pow(12))`) is a **memory-exhaustion surface**:
  the result genuinely has that many bits. The implementation must turn allocation failure
  into a defined Phalcom error, never an abort — the security posture's "every malformed input
  becomes a diagnostic" rule. Whether a proactive count ceiling is wanted is Q-B3.

## 4. Explicitly out of scope, and why

1. **Operator tokens** (`&`, `|`, `^`, `~`, `<<`, `>>`). Verified absent from the lexer
   (grep over `phalcom-ast`, 0 hits, 2026-07-20 — twice now: the hashbrown analysis and this
   spec's session). Adding them is *pure parser sugar over these selectors* — exactly
   ADR-0055's `xs[i]` → `at(_)` pattern, zero floor cost — and it can land later without
   touching this spec. Deferring it also defers three decisions that deserve their own record:
   precedence (C's `&`-below-`==` scar is the canonical cautionary tale), `~` maximal-munch
   against PDR-0012's `~/`, and `|`/`&` against any future block-parameter or pattern syntax.
   Selectors-first is also the Smalltalk position (`bitAnd:`/`bitOr:`/`bitShift:`), which is
   Phalcom's home idiom.
2. **`ushr`** (from catalog §0.2's sketch). Unsigned/logical shift zero-fills *from a width*.
   Unbounded `Int` has no width; the operation is incoherent here. If fixed-width types ever
   arrive, `ushr` belongs to them. The catalog sketch is corrected by this spec.
3. **`leadingZeros`** (same sketch). Width-relative for the same reason — every non-negative
   `Int` has infinitely many leading zeros. `bitLength` is the coherent replacement (Rust's
   `leading_zeros` = `WIDTH - bit_length` precisely when a width exists).
4. **Wrapping / fixed-width operations.** The only demand source on record is the SwissTable
   SWAR analysis, which itself concludes the technique measures *worse* than the existing
   `Map` even when unblocked (`hashbrown-in-phalcom.md` §7). Speccing wrapping semantics would
   also reopen the unbounded-`Int` tension documented there (§5) — a fixed-width type or
   width-carrying selectors, either being a real design decision. Deferred until a demand that
   *survives measurement* exists (binary codecs are the plausible candidate; they should file
   their own record).
5. **Bitwise on `Float`** → `doesNotUnderstand`, by omission. JavaScript's int32-coercing
   operators inside an f64 number type are the anti-precedent (catalog §0.2 already cites the
   cost: a permanent seam plus the `>>> 0` folk idiom).
6. **Bulk bitwise over `Bytes`** (`xorWith(_)` etc.) — separate demand, separate record;
   PDR-0011's bulk-op admission posture governs it.

## 5. Implementation shape (informative until U-BITWISE)

Rides PDR-0012's machinery; nothing new below the surface:

- **Small path:** `Value::Int(i64)` uses native `& | ^ !` and shifts; `shl` computes through
  `BigInt` when the count exceeds the headroom (`64 - leading-bit position`), then demotes via
  the single `normalize` constructor (PDR-0012 ruling 3). Everything else on the small path is
  closed over `i64` except nothing — AND/OR/XOR/NOT of two in-range values are always in range.
- **Large path:** `num-bigint` — already the tower's pinned dependency (PDR-0012 ruling 4);
  this spec adds **zero** dependencies. *Assumed, must be verified at implementation:* that
  `num-bigint`'s signed `BigInt` bitwise impls use infinite-two's-complement semantics for
  negative operands. If they do not, negatives route through the `-x-1` identities; a
  conformance table (§7) is what catches a mismatch either way.
- **No new opcodes.** These are ordinary primitives behind ordinary sends; PDR-0012 ruling 24's
  "no arithmetic fast path yet" window is unaffected.

## 6. Floor amendment: +10 on `Int`

Ten new floor bindings (§2's table), all on `Int`. Live census is **148** (measured, U-BYTES);
PDR-0012 adds 16 when the tower lands. Per PDR-0012 ruling 21's discipline: this record's
constant is `NEW_BITWISE = 10`, composed against whatever the census measures at landing time —
**do not trust prose arithmetic; run `floor_census_matches_installed_bindings`.**

**The admission argument, stated honestly.** ADR-0019 admits a primitive only if the capability
"cannot be expressed in `.ph` at all," and says speed is never sufficient. Bitwise *is*
formally expressible in `.ph` — §1's digit recursion over `~/` and floored `%` is a working
implementation (the UTF-8 decoder at `core.ph:107` already does shift-emulation by division).
This is the first floor candidate in the record whose exclusion clause fails *formally* but
where admission is still right, and the PDR must therefore own an interpretation rather than
hand-wave: the floor **already** contains the practical-arithmetic family (`*`, `/`, `%`, `~/`
are each formally derivable from `+`/`-` and a loop; nobody proposes deriving them). The
admission rule's "cannot be expressed" has always meant *cannot be expressed except by
emulating the arithmetic itself*. Bitwise operations are members of that same family — O(1)
per machine word, definable only by digit-emulation in `.ph` — and are admitted under the
same standing as `%`, not as a speed exception. The tripwire against erosion: this
interpretation covers **numeric kernel operations with per-word native cost** and nothing
else; "it's slow in `.ph`" remains insufficient for everything outside that family.
This is PDR-0020 ruling 1; argue there.

## 7. Conformance

- **Oracle table:** a golden fixture of `(x, y, op, expected)` rows generated from Python
  (same model by construction), mandatory coverage: both signs × both operands, `0`, `±1`,
  `±(2^62)`, `±(2^63)` (the `i64` seam), `±2^100` (deep `LargeInt`).
- **Identity goldens:** §1.1's seven laws over the same value set.
- **Seam-crossing goldens:** `1.shl(62).shl(1)` (promotes), `1.shl(100).shr(100)` (demotes to
  small `1`), `(-1).shr(1000) == -1`.
- **Error goldens (negative lane):** negative shift count, `Float` argument, `0.trailingZeros`.
- **Magnitude-convention goldens:** `(-5).bitCount == 2`, `(-1).bitAt(1000) == true` — pinning
  that `bitCount` is magnitude-based while `bitAt` is sign-aware, the one asymmetry in §2.

## 8. Open questions

| # | Question | Recommendation |
|---|---|---|
| Q-B1 | Selector naming: `bitAnd(_)` (this spec, Smalltalk-derived) vs the catalog sketch's bare `and(_)`/`or(_)`/`not`? Bare names dispatch fine (per-class) but collide *as vocabulary* with `Bool`'s sacred `and(_)`/`or(_)`/`not` — same selector, unrelated semantics, and a reader's (or a future tool's) inference over selector names degrades | Keep `bit`-prefixed + `shl`/`shr`. Reject bare names |
| Q-B2 | Should `bitAt(_)` exist at all, given it is one identity away from `shr`+`%`? | Keep — flag-testing is the family's most common single use (catalog: open-flags, permissions, socket options) |
| Q-B3 | Proactive `shl` count ceiling vs allocation-failure-as-error only? | Failure-as-error only; a ceiling invents a magic number the allocator already owns |
| Q-B4 | Are `bitCount`/`bitLength`/`trailingZeros` in this record, or split to a follow-on? They are queries, not operators, and the floor delta drops to +7 without them | Keep — they are the same native-per-word family, the catalog demand list needs them (hex/base64 sizing), and one record beats two |
| Q-B5 | Operator sugar timing: reserve tokens now (lexer-only) or nothing until the sugar record? | Nothing now. Reserving unlexed tokens has no mechanism in this grammar; ADR-0055 shows sugar retrofits cleanly |

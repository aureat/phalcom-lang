# PDR-0020 — Bitwise operations on `Int`: infinite two's complement, selectors not operators, no wrapping

- Status: **Proposed** (2026-07-20)
- Amends: [ADR-0019](../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md) (floor +10 on
  `Int`, ruling 6 — and carries the rule-interpretation ruling 1, which is the record's real
  content)
- Depends on: [PDR-0012](0012-numeric-tower-implementation-and-floor-amendment.md) (Accepted,
  unimplemented — every selector here lives on `Int`; U-BITWISE is gated on the tower's unit)
- Related: [ADR-0024](../adr/accepted/0024-numeric-surface-split-int-float-and-division.md)
  (whose floored `%` — via PDR-0012 ruling 10 — is what makes ruling 2's semantics
  arithmetically definable), [PDR-0011](0011-admit-bytes-native-octet-buffer.md) (composition
  discipline, and the `Bytes` bulk-op posture ruling 5 defers to),
  [`drafts/stdlib-catalog.md`](../spec/v0.2/drafts/stdlib-catalog.md) §0.2 (the sketch this
  supersedes and corrects),
  [`docs/analyses/hashbrown-in-phalcom.md`](../analyses/hashbrown-in-phalcom.md) §5 (the
  wrapping tension this record deliberately declines to resolve)
- Spec: [`docs/spec/v0.2/core/bitwise.md`](../spec/v0.2/core/bitwise.md) — full surface,
  laws, conformance matrix. **Normative upon ratification.**

## Context

The stdlib catalog names bitwise operations as tier-0 groundwork ("§0.2: blocked on 0.1;
required by open-flags, permission modes, fixed-width codecs, hex/base64, every hash
function, socket options") but its sketch predates the tower's ratification and contains two
operations that are incoherent on an unbounded `Int`. Separately, the hashbrown analysis
(2026-07-20) established that **no record rules bitwise at all** — PDR-0012 contains zero
bitwise rulings — and flagged the unbounded-`Int`-vs-wrapping tension as open. The library
specs being written now (filesystem, network, process) keep generating demand for flags and
masks. This record closes the gap the analysis found, on the demand the catalog documents,
without answering the wrapping question nobody has demonstrated a paying use for.

## Rulings

1. **Floor admission is under the arithmetic-family standing, not a speed exception.**
   Bitwise *is* formally expressible in `.ph` (digit recursion over `~/` and floored `%`;
   `core.ph:107`'s UTF-8 decoder already emulates shifts by division), so ADR-0019's
   "cannot be expressed at all" clause formally fails. It fails for `*`, `/`, `%`, `~/` too —
   each derivable from `+`/`-` and a loop — and the floor never derived them. The admission
   rule's exclusion clause is hereby *interpreted*, not amended: "cannot be expressed" has
   always meant *except by emulating the arithmetic itself*. Bitwise operations are numeric
   kernel operations with per-machine-word native cost, admitted under the same standing as
   `%`. **Tripwire:** this interpretation covers that family only; "slow in `.ph`" remains
   insufficient for anything outside it, and the counter-move for dispatch-shaped cost is
   still "fund an IC/JIT above the floor."

2. **Semantics are infinite two's complement, defined by ratified arithmetic.**
   `shl(n) ≡ x·2ⁿ` (exact, promotes); `shr(n) ≡ x ~/ 2ⁿ` (floor ⇒ arithmetic shift);
   `bitNot ≡ -x-1`; AND/OR/XOR by the digit recursion, well-defined on negatives *because*
   `Int#%` is floored (PDR-0012 ruling 10 — a cross-record interaction now load-bearing).
   Precedent: Python/Ruby/Haskell/Smalltalk — every unbounded-integer language; there is no
   second design. No trap, no width, no wrap.

3. **Surface is ten `bit`-prefixed/named selectors; no operator tokens in this record.**
   `bitAnd(_) bitOr(_) bitXor(_) bitNot shl(_) shr(_) bitAt(_) bitCount bitLength
   trailingZeros` (spec §2). Operator sugar (`&`,`|`,`^`,`~`,`<<`,`>>`) is deferred as a
   future ADR-0055-style parser-sugar record — it is pure spelling over these selectors, and
   it bundles three decisions (precedence vs C's `&`-below-`==` scar, `~` munch against `~/`,
   `|`/`&` against future block/pattern syntax) that must not ride a floor amendment.
   Rejected: the catalog's bare `and(_)/or(_)/not` — same selectors as `Bool`'s sacred trio
   with unrelated semantics; dispatch tolerates it, readers and tools should not have to.

4. **`ushr` and `leadingZeros` are struck from the catalog sketch.** Both are width-relative;
   an unbounded `Int` has no width. `bitLength` is the coherent replacement for the latter;
   the former belongs to a fixed-width type if one ever exists. The catalog's §0.2 gains a
   pointer to this record.

5. **No wrapping, no fixed-width type, and this is a deliberate non-answer.** The only demand
   on record (SwissTable SWAR) measured *negative* even when fully unblocked
   (hashbrown analysis §7). A future record answering the analysis's open question
   (fixed-width type vs width-carrying selectors) must bring a demand that survives
   measurement — binary codecs are the plausible candidate. Likewise deferred: bitwise on
   `Float` (dNU by omission; the JS int32-coercion scar), bulk bitwise over `Bytes`
   (PDR-0011's bulk-op posture governs).

6. **Floor amendment: +10 on `Int`, constant `NEW_BITWISE`.** Live census 148 (measured,
   U-BYTES); the tower adds its 16 first (U-BITWISE is behind it by ruling of dependency), so
   the expected composed figure is 174 — **recompute against
   `floor_census_matches_installed_bindings` at landing; do not trust this sentence**
   (PDR-0012 ruling 21's discipline, adopted verbatim).

7. **Errors are catchable `ArgumentError`s; allocation failure is a defined error.**
   Negative shift counts, non-`Int` arguments (including integral `Float`s — no coercion),
   `0.trailingZeros` all raise. `shl` with a pathological count is a memory-exhaustion
   surface and must surface as a Phalcom error, never an abort (spec §3; ceiling question is
   spec Q-B3, recommended *no*).

## Consequences

- The stdlib catalog's tier-0 §0.2 blocker gains a ruled design; flags/masks/codecs specs can
  cite selectors instead of a sketch.
- The floor grows by 10 under an explicitly-argued interpretation of its own admission rule —
  the precedent-setting part; ruling 1's tripwire is what keeps it narrow.
- SWAR-class algorithms remain inexpressible (no wrapping) — intentionally, per ruling 5.
- Python becomes a usable conformance oracle for the whole family (spec §7), including the
  magnitude conventions for `bitCount`/`bitLength`.
- Zero new dependencies (`num-bigint` is already the tower's), zero new opcodes, zero grammar
  changes.

## Alternatives rejected

- **Author the family in `core.ph` over `~/`/`%`** — honors ADR-0019's letter, ~O(bits) sends
  per operation; unusable for the catalog's stated demand (codecs, hashing). Kept as the
  spec's *definitional* device and expressibility proof, not the implementation.
- **Bare `and(_)/or(_)/xor(_)/not` names** — ruling 3.
- **`Token::Int`-era operator tokens now** — ruling 3's deferral.
- **A `bitShift(_)` with sign-directed shifts (Smalltalk `bitShift:`)** — one selector, but
  every call site re-encodes direction in a sign convention; `shl`/`shr` read.
- **`trailingZeros` returning `None` or a width on `0`** — an `Option` poisons arithmetic
  chains; a width does not exist. Raise.

## Open questions

Spec §8 (Q-B1…Q-B5) travels with this record; Q-B1 (naming) and Q-B4 (query-selector scope,
+10 vs +7) are the two that change the floor arithmetic and should be ruled at ratification.

## Verified vs assumed

**Verified this session (2026-07-20, HEAD `617021a`):** zero bitwise/logical operator tokens
in `phalcom-ast` (`token.rs` grep — corroborating the hashbrown analysis's independent sweep);
zero `Value::Int`/`LargeInt` in `phalcom-core/src` (tower unimplemented); catalog §0.2's
sketch text; PDR-0012's full text (rulings 3, 10, 21, 24 relied on above); live census 148
via STATUS.md's U-BYTES row; `Bytes` kernel-class error idiom (`core.ph:1206ff`).

**Assumed, must be verified at implementation:** `num-bigint`'s signed bitwise impls use
infinite-two's-complement semantics on negatives (spec §5 names the fallback via `-x-1`
identities and the oracle table that catches a mismatch); the exact `i64` fast-path headroom
check for `shl` promotion.

# Specification — `Bytes` (the native octet buffer)

> **Status:** **Normative.** Encodes
> [PDR-0011](../../../pdr/0011-admit-bytes-native-octet-buffer.md) (**Accepted**,
> ratified 2026-07-20); the exploration and precedent survey behind it is
> [`drafts/bytes.md`](../drafts/bytes.md); the implementation spec is
> [`../../forge/units/U-BYTES/implementation-spec.md`](../../forge/units/U-BYTES/implementation-spec.md).
> [PDR-0013](../../../pdr/0013-path-is-bytes-backed-filesystem-surface.md) ruling 4
> (also Accepted) adds an eleventh primitive on this class, `utf8Lossy_`, censused with
> that record.
> **Floor delta: +10 primitives (audited floor 137 → 147; +1 more via PDR-0013)** — this is
> *not* a zero-floor spec;
> the amendment to [ADR-0019](../../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md),
> and the amended admission posture for container bulk operations (§3.1), are carried by
> PDR-0011. The 137 baseline is the tree's, not a record's: the source of record is
> `floor_census_matches_installed_bindings` (`phalcom-core/tests/invariants.rs:605`; last
> delta `Fiber#isRoot`, 136 → 137, 2026-07-19). Never quote a floor number from a document.
> Selector spellings follow
> [ADR-0012](../../adr/accepted/0012-selector-signature-encoding-and-dispatch.md) and
> [ADR-0043](../../adr/accepted/0043-no-default-arguments-keep-selector-identity-pristine.md);
> native primitives carry the trailing `_`
> ([ADR-0049](../../adr/accepted/0049-amend-floor-admit-string-byte-and-raw-write-primitives.md)).
>
> **Owner:** unassigned. Hard prerequisite for any
> [`stream-protocol.md`](stream-protocol.md) implementation (its §9).

## 1. What `Bytes` is

A fixed-length, mutable buffer of octets — the ADR-0020 kernel pattern: storage is a native
heap arm (`Object::Bytes`, backed by `Box<[u8]>`; PDR-0011 ruling 1), the protocol above the
floor primitives is authored in `.ph`. Length is fixed at construction and contents are
mutable — `Tuple`'s backing shape with `List`'s mutability corner. Fixed length is a security
property, not a convenience: it is what makes `zeroize` (§7) complete.

`Bytes` sits **under `Iterable`**, wired natively in the bootstrap exactly as
`List`/`Map`/`Set`/`Tuple`/`Range` are (`universe/core_classes.rs:105-125` is the pattern —
no `.ph` `extends`, no field layout). It supplies `size => self.size_` and
`iteratorValue(cursor) => self.at_(cursor)` (`Tuple`'s exact shape, `core.ph:1011`) and
inherits `Iterable#iterate` (`core.ph:645-651`) and the whole combinator suite unchanged.
The cursor is a `Number` index in `0..size`, so ADR-0048's "a cursor is never `None`"
constraint holds vacuously.

`Bytes` is **not** a `String` variant and never converts to one for free: `StringObject`
enforces UTF-8 and caches a content hash (`heap/string.rs:11-16`), so decode is fallible
(§4) and a `String` holding arbitrary octets is permanently foreclosed (PDR-0011
consequences).

## 2. The element type

**An element is a `Number` that is an integer in 0–255.** There is no `Byte` value type
(PDR-0011 ruling 2). At ruling time ADR-0024 is verified unbuilt — no `Int` heap arm,
`class Number {}` flat at `core.ph:82` — so `Number` is IEEE f64, and every integer in 0–255
is **exactly** representable; reads and writes lose nothing. The contract is worded
representation-independently, so ADR-0024 landing later changes nothing at this surface.

Writes enforce the range: a `set`/`fill` argument that is not an integer in 0–255 **raises**
(precondition violation, stream-protocol law 5's category — a programmer error must not
travel the same channel as data).

## 3. Floor primitives (+10)

Admitted by PDR-0011 ruling 3. Return conventions mirror `List`'s floor exactly
(`primitive/list.rs:72-103`): a fallible *read* returns the bare value or `None` (no `Some`
wrapping — and unlike `List`, the union is unambiguous, because an octet is never `None`);
a bad *write* is a native type error, not a `None`.

| Selector | Side | Returns | Meaning |
|---|---|---|---|
| `Bytes.new(_)` | static | `Bytes` | allocate `n` octets, **zero-filled**; type error unless `n` is a non-negative integer |
| `Bytes.fromString_(_)` | static | `Bytes` | the UTF-8 bytes of a `String`, one native copy |
| `size_` | instance | `Number` | raw length |
| `at_(_)` | instance | `Number` \| `None` | octet at offset; **total** — out-of-bounds is `None`, mirroring `list_raw_at` (`list.rs:72-79`) |
| `set_(_,_)` | instance | `None` | raw octet write; type error on out-of-range index or non-octet value, mirroring `list_raw_set` (`list.rs:91-103`) |
| `fill_(_)` | instance | `None` | overwrite every octet with the given value (one memset); type error on non-octet |
| `slice_(_,_)` | instance | `Bytes` | **copy** of `[start, end)` into a fresh buffer; type error on a bad range |
| `copyInto_(_,_)` | instance | `None` | copy the whole receiver into the given `Bytes` at the given offset (one memmove); type error if it does not fit |
| `utf8_` | instance | `String` \| `None` | fallible UTF-8 decode of the whole buffer; invalid → `None` |
| `utf8Lossy_` | instance | `String` | total lossy decode (invalid sequences → U+FFFD, Rust `from_utf8_lossy`); admitted by PDR-0013 ruling 4 for `Path#toString`, censused with that record |
| `equalsConstantTime_(_)` | instance | `Bool` | §8; the one selector whose *timing* is part of its contract |

Natives never build a `Result` — `Result`/`Ok`/`Err` are pure `.ph`; the `.ph` layer lifts
where it wants to.

### 3.1 The native/`.ph` boundary — where each operation lives, and why

PDR-0011 ruling 3 amends the admission posture **for kernel container arms** beyond
ADR-0019's inexpressibility-only rule, with a bright line:

- **A bulk operation with no user code inside its loop is native.** The arm exists to
  eliminate per-element representation and dispatch cost; a `.ph` per-byte loop over
  `at_`/`set_` reintroduces exactly the cost the arm was admitted to remove (two sends per
  byte, each a method-table probe — no inline cache exists). `fill_`, `slice_`,
  `copyInto_`, `fromString_`, `utf8_` are memset/memmove/validate over contiguous memory;
  making them `.ph` buys no expressiveness and costs O(n) dispatch.
- **A selector that runs a user block per element stays `.ph`, unconditionally.** This is a
  functionality line, not an economy: `Fiber#yield` is only legal at
  `native_reentry_depth == 0` (the restricted-yield guard, `vm/dispatch.rs:259`,
  ADR-0030 §4). A *native* `each` would put a native frame between the caller and the
  block, so any `yield` inside the block becomes a runtime error — Lua's
  "attempt to yield across a C-call boundary", the C-extension/generator wall Python hit,
  the lesson every coroutine language converged on the hard way. Keeping
  `each`/`map`/`filter`/`reduce` in `.ph` (inherited from `Iterable`) means iteration is
  ordinary `.ph` frames all the way down and fibers yield freely mid-iteration. It also
  keeps the literal-block call sites visible to the sacred inliner, which recognizes only
  literal `Expr::Block` arguments (`compiler/inliner.rs:9`, `:128`).

Consequences of the line: `concat` is `.ph` — but over `new` + two `copyInto_` calls
(**three** native sends total, no per-byte loop), not over a byte loop. Derivability with
teeth. `fromList` stays `.ph`: it is cold-path construction whose loop must run Phalcom
`==`/range checks per element anyway.

## 4. The `.ph` protocol

All `.ph` over §3 plus what `Iterable` provides. Zero additional primitives.

| Selector | Returns | Meaning |
|---|---|---|
| `size` | `Number` | `self.size_` |
| `at(_)` | `Number` \| `None` | passthrough of `at_` (`List#at`'s exact shape, `core.ph:781-783`); total — out-of-bounds is `None`, never a raise (collection-protocol law 1) |
| `set(_,_)` | receiver | write octet; **raises** on out-of-bounds index or non-octet value (§2) |
| `fill(_)` | receiver | `fill_` with the raise-lifting of `set` |
| `zeroize` | receiver | `self.fill_(0)` — one native call; the name carries the §7 contract |
| `isEmpty`, `each(_)`, `map(_)`, `filter(_)`, `reduce(_,_)`, `includes(_)`, … | | inherited from `Iterable` (ADR-0048), deliberately `.ph` — §3.1 |
| `==(_)` / `!=(_)` | `Bool` | structural equality, `List#==`'s exact shape (collection-protocol §4): `isA` guard, size check, pairwise loop. **Short-circuits — never use for secrets**; that is what §8 exists for |
| `hash` | `Number` | **identity** (inherited `Object#hash`) — mutable ⇒ not value-hashable, not a valid `Map`/`Set` key (law 4) |
| `toString` | `String` | total debug form (e.g. `Bytes(16)`); **not** the decoder |
| `utf8` | `String` \| `None` | decode via `utf8_`; the fallibility is the point (PDR-0011 consequences) |
| `utf8Lossy` | `String` | total display decode via `utf8Lossy_` — for humans; never round-trip it into data |
| `slice(_,_)` | `Bytes` | `slice_` — a copy, never a view (ruling 5; a view retains its parent under ADR-0050, Erlang's binary leak) |
| `concat(_)` | `Bytes` | `.ph` over `new` + `copyInto_` ×2 — §3.1 |
| `copyInto(_,_)` | receiver | `copyInto_` with raise-lifting |
| `Bytes.fromString(_)` | `Bytes` | `fromString_` |
| `Bytes.fromList(_)` | `Bytes` | from a `List` of octets; raises on any non-octet element. The builder story: build in a `List`, freeze into `Bytes` (`Tuple.fromList`'s shape) |
| `toList` | `List` | octets as a `List` of `Number`s |
| `toTuple` | `Tuple` | immutable, value-hashable snapshot — the `Map`-key escape hatch (ruling 4) |

## 5. Laws

1. **Totality.** `size` is defined on every receiver; `at(_)` is total — in-range octet,
   out-of-bounds `None`, never a raise (collection-protocol law 1, `List#at`'s convention).
   Precondition violations — bad `set`/`fill`/`copyInto` arguments, negative `new` length,
   non-octet `fromList` element — **raise**; they never return `None`
   (stream-protocol law 5's category split).
2. **Octet closure.** Every value read out of a `Bytes` is an integer `Number` in 0–255;
   every write outside that range raises. There is no path by which a non-octet enters a
   buffer.
3. **Fixed length.** `size` never changes after construction. No grow/shrink selector exists
   in this protocol, and none may be added without superseding PDR-0011 ruling 1 — length
   mutability would strand secret copies on realloc and void §7.
4. **Zero-filled birth.** `Bytes.new(n)` reads as `n` zeros; no constructor exposes
   uninitialized memory.
5. **Structural `==`, identity `hash`** (collection-protocol laws 3/4, `List`'s corner):
   `a == b` iff same size and pairwise-equal octets; `a == b` may hold while
   `a.hash != b.hash` — intended.
6. **`slice`/`concat` alias nothing.** Mutating a result never observes in the source, and
   vice versa. `copyInto` writes into its target and reads only its receiver; overlapping
   self-copy (`b.copyInto(b, k)`) behaves as memmove — as if through an intermediate copy.
7. **Decode is fallible and total.** `utf8` returns the decoded `String` iff the buffer is
   valid UTF-8, `None` otherwise; it never raises and never truncates.
8. **Fibers yield through iteration.** `Fiber.yield` inside a block passed to any `Bytes`
   combinator is legal — guaranteed by §3.1's boundary, checkable because a native
   combinator would trip the restricted-yield guard.

## 6. Iteration

Inherited `Iterable#iterate` (`core.ph:645-651`) with `size => self.size_` and
`iteratorValue(cursor) => self.at_(cursor)` (`core.ph:1011`'s shape). Empty buffers
terminate correctly (`0 < 0` → `None`). Iteration visits octets in index order
(collection-protocol law 2).

## 7. Zeroization — an obligation, not a mechanism

Phalcom **cannot guarantee secret erasure**: ADR-0050's tracing collector means no
deterministic destruction and no drop hook, and that is not retrofittable. What this
protocol offers, honestly (PDR-0011 ruling 7):

- `zeroize` is `self.fill_(0)` — **one native memset**, and, because of law 3, a
  **complete** one: a fixed-length `Box<[u8]>` never reallocates, so no octet that ever
  held the secret is outside the buffer being zeroed.
- The scoping idiom is `ensure`, which fires on **any** unwind (ADR-0008 §4 — `throw`,
  non-local `return`, fiber `abort`):

  ```phalcom
  let key = Bytes.new(32)
  try { useKey(key) } ensure { key.zeroize }
  ```

- **Residue, unhedged:** interpreter copies (value stack, fiber stacks, allocation
  intermediates, OS swap/core dumps) are outside any `.ph` obligation. `zeroize` is a real
  and *partial* mitigation; anyone shipping crypto on Phalcom must know it is partial.
  Claiming more is how .NET `SecureString` earned its deprecation.
- **Coupling, named:** this contract makes ADR-0050's *non-moving* choice security-relevant.
  A moving collector copies live objects and defeats `zeroize` silently. Any future record
  reopening moving GC must address this (PDR-0011 ruling 7).

Enforcement posture is ADR-0052's: written contract, code review, golden tests. No static
analysis exists to lean on.

Related, binding on future numeric work: **secret material must never route through
ADR-0024's auto-promoting `Int`** — promotion is value-dependent heap allocation, a timing
channel below the arithmetic (draft §9.1). Fixed-width `Bytes` is the secret carrier.

## 8. `equalsConstantTime(_)`

The `.ph` surface over `equalsConstantTime_(_)`. Native because the property — execution
time independent of buffer contents — is not expressible above the floor: any `.ph` loop
short-circuits, and so does `==` (§4). Node ships `crypto.timingSafeEqual` natively for
exactly this reason.

Contract (PDR-0011 ruling 6):

1. Compares full contents; time is constant with respect to **contents**.
2. **Length mismatch returns `false`** — still without inspecting contents. Lengths are
   **not concealed**; callers for whom length is secret must pad first. This is Go
   `crypto/subtle.ConstantTimeCompare`'s shape. Node's throw-on-mismatch is rejected: it
   leaks length through the exception path and forces a pre-check at every HMAC comparison.
3. A non-`Bytes` argument **raises** (precondition violation — silently returning `false`
   would hide a type bug in the security-critical path).

## 9. Conformance harness

`Bytes` is conformant **iff it passes the harness** (collection-protocol §1 rule):

| Check | Asserts |
|---|---|
| zero-filled birth | law 4; every octet of `Bytes.new(n)` is `0` |
| `at` totality | law 1; in-range → the octet, out-of-bounds → `None`, never raises |
| `set` roundtrip | `set(i, v)` then `at(i)` is `v` for all octet `v` |
| `set` preconditions | `256`, `-1`, `1.5`, out-of-bounds index each raise |
| `fill` | every octet reads the fill value; non-octet raises |
| octet closure | law 2; no sequence of protocol sends yields a non-octet element |
| fixed length | law 3; `size` unchanged by every mutating selector |
| structural `==` | law 5; equal contents ⇒ `==`, one differing octet ⇒ `!=`; non-`Bytes` argument is `!=`, not an error |
| identity `hash` | law 5; two `==` buffers may report different `hash` |
| iteration order | law 2 / §6; `each` visits `at(0)..at(size-1)` in order; empty buffer visits nothing |
| yield mid-iteration | law 8; a fiber may `Fiber.yield` inside an `each` block and resume correctly |
| `slice` copies | law 6; mutating a slice leaves the source unchanged, and vice versa |
| `copyInto` | law 6; contents land at the offset; overflow raises; overlapping self-copy is memmove-correct |
| `concat` | size is the sum; contents in order; aliases neither source |
| decode | law 7; valid UTF-8 → the right `String`; an invalid byte → `None`; empty buffer → `""` |
| `fromString` roundtrip | `Bytes.fromString(s).utf8 == s` for every `String` `s` |
| constant-time equal | §8; equal → `true`, unequal → `false`, length mismatch → `false`, non-`Bytes` raises |
| `zeroize` | §7; after `zeroize`, every octet reads `0` |
| `fromList` / `toList` | inverses on octet lists; `fromList` raises on a non-octet element |
| `toTuple` keys a `Map` | ruling 4's escape hatch actually works |

## 10. Open questions

| # | Question | Notes |
|---|---|---|
| BY-1 | Literal syntax | PDR-0011 Q-1. Lexer question, no owner; `fromList`/`fromString` suffice meanwhile |

Closed by PDR-0011 and recorded there: one-class (ruling 4, was draft B-1), slice copies
(ruling 5, B-2), fixed length (ruling 1, B-3), length-mismatch behavior (ruling 6, B-4),
native bulk ops incl. `fill_` — which dissolves B-6's native-`zeroize` question (ruling 3),
moving-GC coupling (ruling 7, B-7), identity hash (ruling 4, B-8).

## 11. What this document does not cover

- **`BytesReader` / `BytesWriter`.** They are stream-protocol.md §8's reference
  implementations, specified there; this document only supplies their element type.
- **Encodings beyond UTF-8.** `utf8` is the one decoder; no encoding registry is implied.
- **Bit-level access, endian views, packed structs.** `.ph` library territory over
  `at`/`set` if ever wanted; no floor claim.
- **The reactor and the filesystem.** [`stream-protocol.md`](stream-protocol.md) §9's
  boundaries apply unchanged.

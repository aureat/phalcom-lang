# Specification — `Bytes` (the native octet buffer)

> **Status:** **Proposed — normative upon ratification of
> [PDR-0011](../../../decisions/0011-admit-bytes-native-octet-buffer.md)** (Proposed
> 2026-07-20; per [`decisions/README.md`](../../../decisions/README.md) rule 5, nothing may be
> built against this document until that record is Accepted). Encodes PDR-0011's seven rulings;
> the exploration and precedent survey behind them is
> [`drafts/bytes.md`](../drafts/bytes.md).
> **Floor delta: +6 primitives (audited floor 137 → 143)** — this is *not* a zero-floor spec;
> the amendment to [ADR-0019](../../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md) is
> carried by PDR-0011. The 137 baseline is the tree's, not a record's: the source of record is
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
six floor primitives is authored in `.ph`. Length is fixed at construction and contents are
mutable — `Tuple`'s backing shape with `List`'s mutability corner. Fixed length is a security
property, not a convenience: it is what makes `zeroize` (§7) complete.

`Bytes` sits **under `Iterable`**, wired natively in the bootstrap exactly as
`List`/`Map`/`Set`/`Tuple`/`Range` are (`universe/core_classes.rs:105-124` is the pattern —
no `.ph` `extends`, no field layout). It supplies `size` and `iteratorValue(cursor)` the way
`List` does, over `size_`/`at_`, and inherits `Iterable#iterate` and the whole combinator
suite unchanged. The cursor is a `Number` index in `0..size`, so ADR-0048's
"a cursor is never `None`" constraint holds vacuously.

`Bytes` is **not** a `String` variant and never converts to one for free: `StringObject`
enforces UTF-8 and caches a content hash (`heap/string.rs:11-16`), so decode is fallible
(§5) and a `String` holding arbitrary octets is permanently foreclosed (PDR-0011
consequence 3).

## 2. The element type

**An element is a `Number` that is an integer in 0–255.** There is no `Byte` value type
(PDR-0011 ruling 2). At ruling time ADR-0024 is verified unbuilt — no `Int` heap arm,
`class Number {}` flat at `core.ph:82` — so `Number` is IEEE f64, and every integer in 0–255
is **exactly** representable; reads and writes lose nothing. The contract is worded
representation-independently, so ADR-0024 landing later changes nothing at this surface.

Writes enforce the range: a `set` argument that is not an integer in 0–255 **raises**
(precondition violation, stream-protocol law 5's category — a programmer error must not
travel the same channel as data).

## 3. Floor primitives (+6)

Admitted by PDR-0011 ruling 3 under ADR-0019's inexpressibility rule:

| Selector | Side | Returns | Meaning |
|---|---|---|---|
| `Bytes.new(_)` | static | `Bytes` | allocate `n` octets, **zero-filled**; raises unless `n` is a non-negative integer |
| `size_` | instance | `Number` | raw length |
| `at_(_)` | instance | `Option` | raw octet at offset; **total** — out-of-bounds is `None`, mirroring `list_raw_at` |
| `set_(_,_)` | instance | `Option` | raw octet write; total, `None` out-of-bounds, mirroring `list_raw_set` |
| `utf8_` | instance | `Option` | fallible UTF-8 decode of the whole buffer to a `String`; invalid → `None` |
| `equalsConstantTime_(_)` | instance | `Bool` | §8; the one selector whose *timing* is part of its contract |

Rejected as derivable (ruling 3): `slice_`, `concat_`, `fromString_`, `zeroize`. Natives
return `Option`, never `Result` — `Result`/`Ok`/`Err` are pure `.ph` and a primitive cannot
cheaply build one; the `.ph` layer lifts.

## 4. The `.ph` protocol

All `.ph` over §3 plus the ADR-0049 `String` byte floor. Zero additional primitives.

| Selector | Returns | Meaning |
|---|---|---|
| `size` | `Number` | `size_` |
| `at(_)` | `Option` | octet at index; total via `Option` (collection-protocol law 1: out-of-bounds → `None`, never a raise or `nil`) |
| `set(_,_)` | — | write octet; **raises** on out-of-bounds index or non-octet value (§2); returns the receiver |
| `isEmpty`, `each(_)`, `map(_)`, `filter(_)`, `reduce(_,_)`, … | | inherited from `Iterable` (ADR-0048) |
| `==(_)` / `!=(_)` | `Bool` | structural equality, `List#==`'s exact shape (collection-protocol §4): `isA` guard, size check, pairwise loop. **Short-circuits — never use for secrets**; that is what §8 exists for |
| `hash` | `Number` | **identity** (inherited `Object#hash`) — mutable ⇒ not value-hashable, not a valid `Map`/`Set` key (law 4) |
| `toString` | `String` | total debug form (e.g. `Bytes(16)`); **not** the decoder |
| `utf8` | `Option` | decode: `Some(string)` or `None`. The fallibility is the point (PDR-0011 consequence 3) |
| `slice(_,_)` | `Bytes` | **copy**, O(n) (ruling 5 — no views; a view would retain its parent under ADR-0050, Erlang's binary leak) |
| `concat(_)` | `Bytes` | new buffer, contents copied |
| `fill(_)` | — | overwrite every octet with the given value; raises on non-octet |
| `zeroize` | — | `fill(0)` with a name and a contract — §7 |
| `Bytes.fromList(_)` | `Bytes` | from a `List` of octets; raises on any non-octet element. The builder story: build in a `List`, freeze into `Bytes` (`Tuple.fromList`'s shape) |
| `Bytes.fromString(_)` | `Bytes` | the UTF-8 bytes of a `String`, via ADR-0049's `byteCount_`/`byteAt_(_)` |
| `toList` | `List` | octets as a `List` of `Number`s |
| `toTuple` | `Tuple` | immutable, value-hashable snapshot — the `Map`-key escape hatch (ruling 4) |

## 5. Laws

1. **Totality.** `size` is defined on every receiver; `at(_)` is total via `Option`
   (collection-protocol law 1). Precondition violations — bad `set` index, non-octet value,
   negative `new` length — **raise**; they never return `None`/`Err`
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
   vice versa.
7. **Decode is fallible and total.** `utf8` returns `Some` iff the buffer is valid UTF-8,
   `None` otherwise; it never raises and never truncates.

## 6. Iteration

Inherited `Iterable#iterate` (`core.ph:649-652`) with `size => size_` and `List`-shaped
`iteratorValue(cursor)`. Empty buffers terminate correctly (`0 < 0` → `None`). Iteration
visits octets in index order (collection-protocol law 2).

## 7. Zeroization — an obligation, not a mechanism

Phalcom **cannot guarantee secret erasure**: ADR-0050's tracing collector means no
deterministic destruction and no drop hook, and that is not retrofittable. What this
protocol offers, honestly (PDR-0011 ruling 7):

- `zeroize` is a `.ph` `set_(i, 0)` loop, and — because of law 3 — a **complete** one: a
  fixed-length `Box<[u8]>` never reallocates, so the loop provably touches every octet that
  ever held the secret.
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
| `at` totality | law 1; in-range → `Some`, out-of-bounds → `None`, never raises |
| `set` roundtrip | `set(i, v)` then `at(i)` is `Some(v)` for all octet `v` |
| `set` preconditions | `256`, `-1`, `1.5`, out-of-bounds index each raise |
| octet closure | law 2; no sequence of protocol sends yields a non-octet element |
| fixed length | law 3; `size` unchanged by every mutating selector |
| structural `==` | law 5; equal contents ⇒ `==`, one differing octet ⇒ `!=`; non-`Bytes` argument is `!=`, not an error |
| identity `hash` | law 5; two `==` buffers may report different `hash` |
| iteration order | law 2 / §6; `each` visits `at(0)..at(size-1)` in order; empty buffer visits nothing |
| `slice` copies | law 6; mutating a slice leaves the source unchanged, and vice versa |
| decode | law 7; valid UTF-8 → `Some` with the right `String`; an invalid byte → `None`; empty buffer → `Some("")` |
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
native `zeroize` rejected (ruling 3, B-6), moving-GC coupling (ruling 7, B-7), identity hash
(ruling 4, B-8).

## 11. What this document does not cover

- **`BytesReader` / `BytesWriter`.** They are stream-protocol.md §8's reference
  implementations, specified there; this document only supplies their element type.
- **Encodings beyond UTF-8.** `utf8` is the one decoder; no encoding registry is implied.
- **Bit-level access, endian views, packed structs.** `.ph` library territory over
  `at`/`set` if ever wanted; no floor claim.
- **The reactor and the filesystem.** [`stream-protocol.md`](stream-protocol.md) §9's
  boundaries apply unchanged.

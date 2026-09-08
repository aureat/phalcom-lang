# Raw source snapshot: standard-library

> Captured: 2026-09-08
> Source area: numeric/collection library specifications and library-program boundaries
> This immutable snapshot preserves the source excerpts used by the compiled concept articles. Program metadata is lifecycle evidence, not proof that proposed work is implemented.

--- docs/spec/conformance/numbers.md ---
# Numeric Conformance Specification

This document defines the independent models, fixtures, properties, differential tests, and ship gates required for the numeric implementation.

## 1. Reference-model architecture

The primary oracle is an implementation-independent mathematical model.

Use:

- arbitrary-precision integers for Int;
- exact dyadic decomposition for finite Float;
- exact rational comparison for mixed values;
- mathematical floor quotient and derived remainder;
- explicit binary64 rounding for Float results;
- canonical numeric-key and hash-input modeling;
- higher-precision real arithmetic for Float power accuracy.

External languages may be differential oracles only where Phalcom intentionally shares semantics:

- Python is suitable for infinite-two's-complement bitwise behavior and many floor-division cases.
- Host Rust operations are not authoritative where Rust truncates remainder, limits integer width, or delegates `pow` to platform libraries.

## 2. Required test layers

1. Lexer unit tests.
2. Parser/precedence unit tests.
3. Compiler constant tests.
4. Runtime semantic unit tests.
5. GC/rooting stress tests.
6. Public-language golden tests.
7. Negative/error golden tests.
8. Property-based tests.
9. Differential tests against selected oracles.
10. Primitive, intrinsic, and specialized-path equivalence tests.
11. Primitive-floor and class-reflection invariant tests.
12. Performance measurements recorded separately from semantic acceptance.

## 3. Canonical value corpus

Every relevant operation must sample:

### 3.1 Int

```text
0
±1
±2
±(2^52 - 1), ±2^52, ±(2^52 + 1)
±(2^53 - 1), ±2^53, ±(2^53 + 1)
±(2^62 - 1), ±2^62
±(2^63 - 1), ±2^63, ±(2^63 + 1)
±2^100
values immediately inside/outside the private immediate range
configured resource-limit boundaries
```

### 3.2 Float

```text
+0.0, -0.0
smallest positive/negative subnormal
largest subnormal
smallest normal
1.0 and adjacent representable values
2^52, 2^53 and adjacent representable values
largest finite positive/negative values
+Infinity, -Infinity
multiple NaN bit patterns and signs where constructible
values immediately around half-integers
values around rendering exponent thresholds -6 and 20
```

## 4. Representation invariants

### REP-1 — Class identity

```phalcom
1.class == Int
1.0.class == Float
(2 ** 200).class == Int
```

### REP-2 — Canonical demotion

A computation that crosses into LargeInt and returns into the private immediate range must produce the canonical immediate representation. This is asserted at Rust/runtime level.

### REP-3 — No surface tier leak

Equality, hash, rendering, reflection, pattern matching, method dispatch, and errors must not expose LargeInt as a separate type.

### REP-4 — Constant rooting

Load a module containing many oversized Int constants under GC stress. Every constant remains exact and live after repeated collections and module/function lifetime transitions.


--- docs/spec/current/stdlib/map-and-set.md ---
# Specification — `Map` and `Set` (hash collections)

> **Status:** **Accepted** (representation + `Map` literal ratified by the collections
> umbrella [ADR-0032](../../../adr/0032-collections-representation-and-literals.md);
> `Set` literal `#{…}` reserved-inactive). Absent classes (names reserved in `ClassName`), now
> **unblocked** — their precondition `Object#hash` landed with
> [U-CORE-1](../../../forge/units/U-CORE-1/ucore1.md)
> ([`catalog-delta.md`](./catalog-delta.md) §2.4/§4.5). Each is its own unit per
> [ADR-0020](../../../adr/0020-kernel-list-native-array-protocol.md); both must
> satisfy the [collection protocol](./collection-protocol.md). Inherits the
> baseline pin from [`README.md`](./README.md).
>
> **Owner:** U-STD.

## 1. Preconditions (all met)

- `Object#hash` + per-immediate value hashes (`Number`/`String`/`Bool`/`Symbol`) —
  **landed U-CORE-1** ([ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md)).
- `==` on keys — floor/`.ph`. Hash-consistency law (`a == b ⇒ a.hash == b.hash`,
  R-INV-1.3) holds, so hash lookup is correct.
- `Option` for total lookup — landed (U-CORE-2/U-STD).

## 2. `Map` — hash map

Keys use `hash` + `==`; values are arbitrary. Insertion-order iteration (stable
within a run) satisfies the protocol's deterministic-iteration law.

| Selector | Meaning |
|---|---|
| `Map.new()` | empty map (static) |
| `at(_)` | value for a key → `Some(v)` / `None` (total, no `nil`) |
| `at(_, put:)` | insert/overwrite; returns `self` |
| `size` | entry count |
| `includes(_)` | is a key present → `Bool` |
| `remove(_)` | delete a key; returns `self` |
| `keys` / `values` | live `Iterable` projections of keys / values, in iteration order |
| `entries` | live `Iterable` projection of `Entry` values, in iteration order |
| `each(_)` | apply a 1-arg block to each ordinary Map iteration value (the key) |

`Map` inherits ordinary one-value `Iterable#each(_)` traversal; it does not
change callback arity for map receivers. Pair traversal is explicit:
`map.entries.each { entry => ... }`, using `entry.key` and `entry.value`.

`==` is structural (same key set, pairwise-`==` values). `Map` is **mutable** ⇒
identity hash (collection-protocol law 4).

## 3. `Set` — hash set

Membership by `hash` + `==`; no duplicates.

| Selector | Meaning |
|---|---|
| `Set.new()` | empty set (static) |
| `add(_)` | insert (idempotent); returns `self` |
| `includes(_)` | membership → `Bool` |
| `size` | cardinality |
| `remove(_)` | delete; returns `self` |
| `each(_)` | apply a 1-arg block per element |

`==` is structural (same members, order-independent). `Set` is **mutable** ⇒
identity hash.

## 4. Representation (sub-decision)

| Option | Mechanism | Recommendation |
|---|---|---|
| **Native heap arm** (`Object::Map`/`Object::Set` over a Rust `HashMap`/`HashSet` keyed by the value's `hash`+`==`) | mirrors `List`'s `ListObject` (ADR-0020) | **Recommended** — O(1) ops, matches the "native container, `.ph` protocol" pattern; needs a small floor for `get_`/`put_`/`has_`/iteration |
| **`.ph` over `List`** of buckets | pure `.ph`, zero floor | rejected — O(n) lookup defeats the point; hashing in `.ph` is awkward |

The native arm implies a small **ADR-0019 amendment** (the raw hash-table
primitives) — scoped and justified when the unit lands, analogous to `List`'s five
raw primitives. Combinators (`map`/`filter`/…) stay `.ph`.

## 5. Non-goals

- **Literal syntax.** The **`Map` literal `{ a: 1 }` is ratified** and ships
  ([ADR-0032](../../../adr/0032-collections-representation-and-literals.md) §3.1:
  bare-identifier keys are symbols; `{}` stays a block; empty map is `Map.new()`).
  The **`Set` literal `#{…}` is reserved-inactive** — construct via `Set.new()` /
  `Set(1, 2)` (open-Q6). Parser/compiler work is U-LEX.
- **Ordering guarantees** beyond "stable within a run" — no sorted variant here.

## 6. Test strategy

Instantiate the [collection-protocol](./collection-protocol.md) conformance harness
for each; plus: hash-collision correctness, key-overwrite, `remove` idempotence,
`None` on missing key, structural `==`.

--- docs/spec/current/stdlib/bytes.md ---
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

--- docs/implementation/STDL001-numeric-library/PROGRAM.md ---
---
id: STDL001
category: STDL
kind: implementation
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
---

# STDL001 — numeric library

This program owns the numeric contract, value model, literal syntax, integer
and Float protocols, diagnostics, and integration verification.

--- docs/implementation/STDL002-standard-library-buildout/PROGRAM.md ---
---
id: STDL002
category: STDL
kind: standard-library-buildout
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# STDL002 — standard library buildout

- Status: **Program index.** Not ratified, not an ADR, not a normative spec.
- Date: 2026-07-20
- Scope ratified by the user on 2026-07-20; recorded in
  [`stdlib-catalog.md` §Amendment](../../spec/design/drafts/stdlib-catalog.md#amendment-2026-07-20--the-ratified-build-program).

## What this folder is

The reasoning for *why* Phalcom needs a standard library, what a modern one contains, and
what depends on what, lives in the exploration draft
[`stdlib-catalog.md`](../../spec/design/drafts/stdlib-catalog.md) (Tier 0–6,
open questions S-1…S-13). **That document is the argument. This folder is the program:**
the eighteen selected items, the order they must be built in, what each will cost, and one
spec file per item.

Know which tier you are reading — the four in
[`drafts/README.md`](../../spec/design/drafts/README.md) plus this one:

| Tier | Where | Means |
|---|---|---|
| Ratified decision | `docs/adr/accepted/` | Committed. Needs a superseding ADR to change. |
| Normative spec | `docs/spec/` | The designed surface. Cites its decision record. |
| As-built | `docs/forge/units/*/as-built.md` | What shipped, with `file:line`. |
| Draft | `docs/spec/design/drafts/` | Exploration. No authority. |
| **Program** ← *you are here* | `docs/implementation/stdl/STDL002-standard-library-buildout/` | **A selected, ordered work list.** Each item's spec file, once written, is a design document — it becomes normative only by growing an ADR. |

**Nothing here is built.** No item below has an owning unit. The per-item spec files are
listed but **not yet written**.

---

## Ranking scales

Two independent axes. Conflating them is how "just add files" becomes a six-week unit.

**Complexity (1–5)** — design risk. How much can go *irreversibly* wrong: how many
ratified decisions it touches, whether a wrong choice is a breaking change, how much of it
is a ruling rather than an implementation.

| | |
|---|---|
| **1** | Mechanical. Bind an existing Rust API; the design is not in question. |
| **2** | One or two contained design choices with obvious defaults. |
| **3** | A new representation or a new heap arm; localized but novel. |
| **4** | Touches a ratified ADR, or a wrong choice breaks a public surface later. |
| **5** | Touches the VM's core invariants, or is a genuinely hard domain in its own right. |

**Work (S/M/L/XL)** — implementation volume, roughly: **S** ≤ ~300 lines and a day;
**M** a few days; **L** a week-plus; **XL** multi-week, and the item is really a track.

High complexity with low work is the dangerous combination — it *looks* cheap on a
sprint board and is where a wrong ruling gets made in an afternoon. Items **17** and **3**
are that shape.

---

## The program

Ordering is dependency-forced first, then cheapest-useful-first among items that are free
to move. Blockers listed are the ones that must be *resolved*, not merely noted.

| # | Item | Cx | Work | Blocked on | Spec file |
|---|---|:--:|:--:|---|---|
| **1** | **Numeric tower** — `Number` abstract, `Int` (exact, unbounded) / `Float`, bitwise ops, `/` vs `~/` | **5** | **XL** | ADR-0024 (ratified, unbuilt). Nothing else. | `01-numeric-tower.md` |
| **2** | **`BigInt` surface + `Decimal`** — radix conversion, `modPow`/`gcd`; `Decimal` scale + rounding modes | 4 | L | 1 | `02-bigint-decimal.md` |
| **3** | **Sealed types / enums** — exhaustiveness checking over `match` | **4** | **M** | PDR-0001 (classes are closed) | `03-sealed-enums.md` |
| **4** | **`Comparable` / `Hashable` / `sort`** — ordering contract, total-order law, stable sort | 2 | M | 1 (`Float` NaN ordering); protocols prototype-only (S-12); **ffi.md F-12** | `04-ordering-hashing.md` |
| **5** | **`Bytes`** — mutable octet buffer, new heap arm | 3 | M | 1, 4 | `05-bytes.md` |
| **6** | **`Path`** — opaque, not `String` | 3 | M | 5; **S-4** ruling | `06-path.md` |
| **7** | **`Reader` / `Writer` / `Seekable`** — the stream protocol | **4** | **M** | 5; **S-2** ruling; S-12; F-12 | `07-streams.md` |
| **8** | **`File` / `Fs`** — open, read, write, metadata, permissions, directory walk | 4 | **XL** | 6, 7; **S-1** ruling (resource lifetime) | `08-file-fs.md` |
| **9** | **`stdio` / `env`** — real writers behind `System.print`; process environment | 2 | M | 7; **S-5**, **S-13** | `09-stdio-env.md` |
| **10** | **Text** — `Encoding` (UTF-8/16/Latin-1), `StringBuilder`, `Char` | 2 | M | 5; **S-9** (what does `StringCodePointSequence` yield today?) | `10-text.md` |
| **11** | **`math`** | **1** | **S** | 1 | `11-math.md` |
| **12** | **`random`** — seeded PRNG + OS CSPRNG, kept distinct | 2 | S | 1, 5 | `12-random.md` |
| **13** | **`Instant` / `Duration`** — monotonic time only | **1** | **S** | 1 | `13-instant-duration.md` |
| **14** | **`os`** | **1** | **S** | 1; **S-13** | `14-os.md` |
| **15** | **`Uuid`** — v4, v7, parse | **1** | **S** | 5, 12, 13 | `15-uuid.md` |
| **16** | **Backtraces** — reified `Backtrace`, `Error#backtrace`, `Error#cause` | 2 | M | — | `16-backtraces.md` |
| **17** | **`WeakRef` / `WeakMap`** | **4** | **M** | ADR-0050 amendment; **S-8** (cost unscoped) | `17-weakref.md` |
| **18** | **`DateTime` / `TimeZone` / calendar** | **5** | **XL** | 13, 10 | `18-datetime.md` |
| **19** | **Timers** — `sleep`, `Timer.after/every`, and the reactor behind them | **5** | **L** | 13; **S-2** and **S-3** rulings; `open-questions.md` §15 fairness | `19-timers.md` |

Nineteen rows for eighteen items: **Time is split**. See "Deviations" below.

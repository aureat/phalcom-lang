# `Bytes` — a native octet buffer under `Iterable`

- Status: **Draft — PROMOTED 2026-07-20.** The normative surface is
  [`../core/bytes.md`](../core/bytes.md), ruled by
  [PDR-0011](../../../decisions/0011-admit-bytes-native-octet-buffer.md) (Accepted). This
  file stays as the exploration/precedent record; where they disagree, the spec wins. Known
  staleness at promotion: the floor baseline below (§4's 125) predates the Fiber census
  admission — the audited floor is **137** (`invariants.rs:605`), so the delta is 137 → 147
  (PDR-0011 ruling 3 admits ten, not this draft's six — the container bulk-op posture).
- Date: 2026-07-15
- Depends on:
  [ADR-0019](../../../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md) (the floor + its admission rule) ·
  [ADR-0020](../../../adr/accepted/0020-kernel-list-native-array-protocol.md) (**the governing precedent**) ·
  [ADR-0032](../../../adr/accepted/0032-collections-representation-and-literals.md) / [ADR-0039](../../../adr/accepted/0039-amend-floor-admit-collection-container-primitives.md) (collections as native arms + per-arm floor accounting) ·
  [ADR-0048](../../../adr/accepted/0048-amend-iteration-bare-cursor-sentinel-and-iterable-root.md) (bare-cursor protocol, `Iterable` root) ·
  [ADR-0049](../../../adr/accepted/0049-amend-floor-admit-string-byte-and-raw-write-primitives.md) (String byte/slice floor) ·
  [ADR-0050](../../../adr/accepted/0050-non-moving-mark-sweep-collector.md) (non-moving mark-sweep — **load-bearing here**) ·
  [ADR-0008](../../../adr/accepted/0008-layered-exceptions-and-result.md) §4 (unified unwind / `ensure`) ·
  [ADR-0024](../../../adr/accepted/0024-numeric-surface-split-int-float-and-division.md) (Accepted, **not built**)
- Floor counts in this doc cite **the tree**, not an ADR — see §4. Per the overlay's
  *Known documentation defects* #4: never quote a floor number from an ADR.

> **This is a growing exploration doc.** Sections are append-friendly; the open-questions
> table (§10) is the intended landing spot for future insight. Nothing here is committed.

## 1. Thesis

The question that commissioned this doc was *"native or `.ph`? Is `Bytes` even a class?
Maybe it won't be as fast if we do bytes."* ADR-0020 already answers all three, and the
answer is not a compromise — it is the pattern every container in this language follows:

**`Bytes` is an ordinary Phalcom class whose storage is a native heap arm
(`Object::Bytes`) and whose protocol is authored in `.ph` on top.** Native storage,
dogfooded behavior. This is the "hybrid: native primitives, self-defined control" row of
the kernel matrix (ADR-0020 Decision), and it is exactly how `List`, `Map`, `Set`,
`Tuple`, and `Range` already work (`Object::List` … `Object::Range`,
`phalcom-core/src/heap/object.rs:60-103`).

"Won't be as fast" is the right instinct pointed at the wrong target. The slow design is
not "`Bytes` in `.ph`" versus "`Bytes` in Rust" — it is **`Bytes` over a `List` of
`Number`s**, and its cost is measurable rather than rhetorical. §2 measures it.

## 2. Representation — the memory math, measured

The `.ph`-over-`List` design stores each octet as a `Value::Number(f64)` inside
`ListObject`'s `Vec<Value>` (`phalcom-core/src/heap/list.rs:22-25`). Measured on this tree
(`size_of`, aarch64, debug):

| Type | Measured size |
|---|---|
| `Value` | **16 B** |
| `Object` | 40 B |
| `ListObject` (`Vec<Value>`) | 24 B |
| `StringObject` | 32 B |
| `Vec<u8>` | 24 B |

> **The commissioning thesis said "8+ bytes for one byte of payload." The tree is worse:
> `Value` is 16 bytes.** A `List`-backed `Bytes` costs **16 bytes of buffer per 1 byte of
> payload — a 16× blowup**, before any dispatch. A 1 MiB payload becomes a 16 MiB
> `Vec<Value>`. `Value::Number` is an immediate (ADR-0010), so there is no *extra* heap
> object per element — the 16× is entirely the enum's width. That is the honest answer to
> "won't be as fast": it is a 16× memory answer first and a dispatch answer second.

On top of the 16×, every read is `at(_)` → `at_(_)` — two full sends, each a hashmap probe
(no inline cache exists in the tree; overlay §Performance). Native `Object::Bytes` reads
one `u8` at an offset.

**The proposed arm costs nothing structurally.** `Vec<u8>` is 24 B — identical to
`ListObject`'s `Vec<Value>`, and well under the 40 B `Object` slot the `SlotMap` already
sizes to its fattest variant. Adding `Bytes(BytesObject)` widens no slot and boxes nothing
(`object.rs:24-115`; the boxing note at `:29-33`).

**Length: fixed at construction.** Proposed backing is `Box<[u8]>` (`Tuple`'s shape,
`Object::Tuple` at `object.rs:95`), not `Vec<u8>` (`List`'s shape) — **contents mutable,
length immutable**. This is not an ergonomic preference; §7 shows it is what makes
zeroization sound at all. A growable `Vec<u8>` reallocs, and a realloc leaves a full copy
of the old secret in the arena that no `zeroize` can ever reach.

## 3. Is it a class? Where in the hierarchy?

Yes — as ordinary as `List`. `List`/`Map`/`Set`/`Tuple`/`Range` are **not** declared
`extends Iterable` in `.ph`; their superclass is wired natively in the bootstrap
(`phalcom-core/src/universe/core_classes.rs:105-124`, e.g. `list_class` ←
`iterable_class` at `:106`), because a native heap arm has no field layout and needs no
`construct` lowering. `Bytes` follows that line verbatim. `.ph`-side view classes
(`MapView extends Iterable`, `core.ph:1146`) are the other, non-kernel pattern.

**`Bytes` sits under `Iterable`** (ADR-0048 §3). A subclass supplies `size` +
`iteratorValue`; it inherits the generic index cursor and the whole combinator suite
(`each`/`map`/`filter`/`reduce`/`includes`/`isEmpty`/`all`/…). The inherited
`Iterable#iterate` (`core.ph:649-652`) is:

```phalcom
iterate(cursor) {
  let next = (cursor == None).ifTrue({ 0 }, ifFalse: { cursor + 1 })
  return (next < self.size).ifTrue({ next }, ifFalse: { None })
}
```

**Does a byte-index cursor satisfy ADR-0048's new constraint — *a cursor value may never
itself be `None`*?** Yes, vacuously. The cursor is a `Number` index in `0..size`, never
`None`; `None` is produced only at exhaustion, as the sentinel. `Bytes` overrides nothing
— it inherits `iterate` unchanged and defines `size => self.size_` and
`iteratorValue(cursor) => self.at_(cursor)`. Empty `Bytes` terminates correctly
(`0 < 0` is false → `None`).

**Element type: `Number` in 0–255.** There is no `Byte` value type and this doc does not
propose one — it would be a new `Value` arm (ADR-0010 keeps `Value` minimal; even `Fiber`
and `Family` declined an arm, `object.rs:64-67`, `:119-121`), a new dispatch axis, and a
new numeric tower row, to buy a range check that `at_`/`set_` already enforce.

## 4. Floor delta

**Authoritative baseline: the tree, not an ADR.** `floor_census_matches_installed_bindings`
(`phalcom-core/tests/invariants.rs:616`) reconstructs the floor from a live `VM::new()`;
its constants sum to **125** bindings post-U-STRING (`invariants.rs:632-690`). Run green
this session.

> **Tree contradiction — REPORTED HERE, FIXED 2026-07-15 (DEFERRED CB-2).** This draft
> flagged that `docs/spec/v0.2/core/floor-census.md` was stale in two places at once
> (§1.1's table said **113** bindings / 98 fns, §7's audit-hook prose said **count = 117**,
> machine-checked was **125**) and that §8 pointed at `universe.rs`, now a directory
> (`phalcom-core/src/universe/`). All of it is now reconciled: §1.1 reads 125/110, §7 no
> longer hardcodes a number, §8 is rewritten to lead with symbols, and a new §1.3 names
> `invariants.rs::floor_census_matches_installed_bindings` as the source of record. The
> draft's 125 above was right all along. **New, from that pass:** the 125 is the *audited*
> floor — `VM::new()` installs 136, and `Fiber`'s 11 are outside the census entirely
> (§1.4, DEFERRED CB-5). Budget any bytes amendment against 125, but know the freeze has a
> hole in it.

Admission is ADR-0019's rule: proof the capability **cannot be expressed in `.ph` at all**.
Speed is explicitly never sufficient. Applying it honestly kills four of the seven
primitives the commissioning thesis floated:

| Selector | Side | Irreducible? | Verdict |
|---|---|---|---|
| `Bytes.new(_)` | static | allocates a native arm; `.ph` cannot | **admit** |
| `size_` | instance | raw length of a native buffer | **admit** |
| `at_(_)` | instance | raw octet at offset; total (`None` OOB), mirrors `list_raw_at` | **admit** |
| `set_(_,_)` | instance | raw octet write; mirrors `list_raw_set` | **admit** |
| `toString_` | instance | **fallible** UTF-8 decode → `Option`. No String primitive builds a `String` from arbitrary bytes; `String.new(_)` coerces a value, it does not decode a buffer | **admit** |
| `equalsConstantTime_(_)` | instance | see §8 — presupposes representation + timing control `.ph` cannot express | **admit** |
| `slice_(_,_)` | instance | **derivable** — `new(len)` + a `set_`/`at_` loop | reject |
| `concat_(_)` | instance | **derivable** — same | reject |
| `fromString_(_)` | static | **derivable** — ADR-0049 already gave `.ph` `byteCount_`/`byteAt_(_)` | reject |
| `zeroize` | instance | **derivable** — a `.ph` `set_(i, 0)` loop; and see §7 on why it is *complete* here | reject |

**Delta: +6 (125 → 131).** Floor-carrying classes +1 (`Bytes` is a new row). Mirrors
ADR-0039's per-arm style; comparable to `List`'s five and `Map`'s eight.

> **A real asymmetry worth recording.** ADR-0049 admitted `slice_(_,_)` on **String** and
> called it "the one Wren-cited irreducible case." It is irreducible *there* because
> `String` is immutable — `.ph` has no way to build a `String` octet by octet. It is
> **reducible on `Bytes`** precisely because `Bytes` is mutable and has `new` + `set_`.
> Same selector name, opposite admission verdict, and the mutability axis is the reason.
> A `.ph` `slice` is O(n) sends; ADR-0019's named counter-move is to fund an inline cache
> above the floor, not to nativize.

## 5. `Bytes` vs `String` — a separate arm

Phalcom already has byte-level access — but only *through* `String`
(`byteCount_`/`byteAt_(_)`/`slice_(_,_)`, ADR-0049, in the tree at census §2.5). So: is
`Bytes` a separate arm, or is it "`String` without the UTF-8 invariant"?

**Position: a separate arm, and the split is the point.** Rust's `String`/`str` vs
`Vec<u8>`/`[u8]` are distinct types *because the UTF-8 invariant is load-bearing* —
`String::from_utf8` is fallible, and that fallibility is the type system refusing to
launder arbitrary octets into text. Phalcom's tree already encodes the same invariant
natively: `StringObject { value: String, hash: u32 }` caches a **djb2 content hash** at
construction (`heap/string.rs:11-16`, `:35-40`), and `slice_` "**must** validate
`str::is_char_boundary` on both ends and **never panic**" (ADR-0049 Decision).
`String` cannot host arbitrary octets without breaking both.

Consistent with this, `String` is **not** under `Iterable` (`core.ph:84`, `class String {`)
— it exposes `StringByteSequence` (`core.ph:363`) and `StringCodePointSequence` (`:388`)
as sub-iterable views instead, because "iterate a string" is ambiguous. `Bytes` has no
such ambiguity: one element type, one cursor. That `String` needed the view pattern and
`Bytes` does not is itself evidence they are different kinds.

**What this precludes:** `Bytes` and `String` are not interconvertible for free.
`bytes.toString_` returns an `Option` and callers must handle `None` — a real ergonomic
tax on every decode site, permanently. This is the cost, taken deliberately: the
alternative (a `String` that may hold invalid UTF-8) makes `byteAt_`/`slice_`'s
char-boundary contract unstatable and silently poisons the cached content hash.

## 6. Mutability

**Verified in the tree: `String` is immutable.** `heap/string.rs:1-7` states it
("Its content is immutable"), and the struct exposes no mutable accessor. The cached hash
is the structural enforcement — mutation would silently invalidate it, and `Map`/`Set`
key on Phalcom `hash` (ADR-0039 Consequences).

`List`/`Map`/`Set` are mutable and therefore **inherit identity `Object#hash`** and are
**not valid `Map`/`Set` keys** (Q5, collection-protocol law 4; `object.rs:74-77`).
`Tuple`/`Range` are immutable and value-hash.

`Bytes` must pick a corner, and the two use cases pull opposite ways: crypto wants mutable
+ zeroizable; a hash digest wants immutable + value-hashable (so it can key a `Map`).

**Position (weakly held — see B-1): one class, mutable contents, fixed length.** It
inherits identity `hash`, is not a valid `Map` key, and sits in `List`'s corner. Rust's
answer is the two-type split (`Bytes`/`BytesMut`), and the cost is visible in the crate:
every API duplicates, every boundary needs `freeze()`/`into()`, and callers thread the
distinction through signatures that do not care. Phalcom would pay that twice over,
because it has no types to make the split cheap — it would be two classes, two floors, two
`.ph` protocols, and a conversion selector, to serve a `Map`-key use case that
`Tuple.fromList` already covers today.

## 7. Zeroization — the part with no clean answer

**State it plainly: Phalcom cannot guarantee secret erasure, and this is not fixable in
general.** ADR-0050 (Accepted, ratified 2026-07-14 per `docs/adr/STATUS.md:80`) selects a
non-moving precise mark-sweep collector. GC means **no deterministic destruction and no
`Drop`**. A secret in a `Bytes` lingers in the arena until collected — possibly forever,
and with no hook that fires when it is. Rust's `zeroize` crate depends *entirely* on
`Drop`; that mechanism does not exist here and cannot be retrofitted onto a tracing
collector.

The precedents are a graveyard, and both cost their ecosystems real credibility:

- **Java**: the standing advice is `char[]` over `String` for passwords, *specifically* so
  it can be overwritten — an admission that the managed heap will not do it for you. Cost:
  the advice is folklore, unenforced, and `String.intern`/JIT copies defeat it anyway.
- **.NET `SecureString`**: shipped, then **deprecated as unfixable** — Microsoft's own
  guidance is now "don't use it," because it cannot protect a secret already in managed
  memory, and every use site had to be unwound. Cost: an entire API's worth of false
  assurance, then a migration.

**What Phalcom can offer, honestly:** an explicit `bytes.zeroize` as a **documented
obligation**, scoped by `ensure`. The scoping *is* sound — ADR-0008 §4's unified unwind
means `ensure` fires on **any** unwind through it, not just `throw` (`0008:47`): non-local
`return` and fiber `abort` included. So this is airtight against control flow:

```phalcom
let key = Bytes.new(32)
try { useKey(key) } ensure { key.zeroize }
```

**Two findings that cut in Phalcom's favor, both structural:**

1. **Fixed length (§2) makes `.ph` `zeroize` complete.** A growable `Vec<u8>` reallocs and
   strands a copy of the old secret in the arena, unreachable by any selector. `Box<[u8]>`
   never reallocs, so a `set_(i, 0)` loop provably touches every octet that ever held the
   secret. This is the strongest argument for fixed length.
2. **ADR-0050's non-moving choice is load-bearing for zeroization, and nobody has noticed.**
   A moving/compacting collector *copies live objects*, scattering stale secret images
   across the arena and defeating `zeroize` completely. ADR-0050 chose non-moving and
   demoted moving to "kept reversibly open, not taken now" (overlay §GC). **Admitting
   `Bytes` with a zeroization contract converts that reversible door into a
   security-relevant one** — a future moving GC would silently break the contract with no
   test to catch it. This deserves to be written down before, not after.

**Enforcement posture: written contract + golden test.** Not static analysis — Phalcom has
none (ADR-0021's known gap; DEC-C). This is precisely ADR-0052's precedent, which found
its own invariant "not enforceable by static analysis — Phalcom is dynamically typed" and
settled on "a **written contract** … checked in code review and by the golden-test corpus"
(`0052:154-158`). `Bytes` inherits that posture unchanged.

**The residue, unhedged:** interpreter-level copies are unreachable. `at_` pushes a byte
onto the value stack; a fiber's stack buffers, the `Vec` behind `Bytes.new`'s allocation,
and OS-level swap/core-dumps are all outside any `.ph` obligation. `zeroize` is a real
mitigation and a partial one. Anyone shipping crypto on Phalcom should know it is partial.

## 8. Constant-time comparison

`equalsConstantTime_(_)` passes ADR-0019's rule **on its own merits**, independent of
crypto: the security property *is* a statement about representation and execution timing,
which `.ph` cannot express — a `.ph` loop over `at_` short-circuits, and `==` on `Bytes`
would too. The capability presupposes control below the `.ph` boundary. That is the
admission test, met squarely.

**Precedent with consequence:** Node ships `crypto.timingSafeEqual` as a **native
builtin** for exactly this reason — it cannot be written in JavaScript. The cost Node pays
is instructive and should be copied deliberately or not at all: it **throws on length
mismatch**, which leaks length through the exception path, and it lives in `crypto`, so
users who reach for `===` get a vulnerable comparison with no diagnostic. Phalcom's
version should decide the length-mismatch behavior explicitly (B-4), not inherit Node's by
accident.

It is also useful long before any crypto suite exists — HMAC/digest/token comparison is
the common case, and it is one selector.

## 9. Interaction hazards

- **primitive/library boundary ⊗ bootstrap order** (ADR-0020's named hazard). `toString_`
  returns a `String`, so `string_class` must exist before `Bytes` primitives install.
  `Bytes` slots alongside `List` in `core_classes.rs`'s ordering (after `Option`/`Bool`/
  `Number`/`Symbol`/`String`, `core_classes.rs:100-124`) and needs `Iterable`, which is
  created immediately above `List` (`:105`). An unwritten edge here is a hard boot failure
  with no user frame to blame (ADR-0019 Context) — `verify_invariants` should carry the
  `Bytes` row (`universe/invariants.rs:96` is the pattern).
- **cleanup ordering ⊗ unwinding.** `ensure`-scoped `zeroize` is sound *because* ADR-0008
  §4 unified `return`/`throw`/`abort` into one unwind. But `ensure` blocks nest, and a
  `zeroize` in an outer `ensure` runs *after* an inner one may have already leaked the
  value onward. Scoping is a contract about the *whole* lifetime, not one frame.
- **zeroization ⊗ moving GC** (§7.2). New, and the reason this doc exists at all.
- **value-dependent allocation ⊗ constant time** (§9.1 below).

### 9.1 `Int`/bignum — do not route secrets through `Number`

ADR-0024 (Accepted; **❌ not built** per `STATUS.md:54` — "code is still flat …
committed design, zero implementation"; the flat `class Number {}` is at `core.ph:82`
today, not the `:75` STATUS.md cites) makes `Int` an **auto-promoting
bignum**: a `LargeInt` heap kind with overflow promotion. Auto-promotion is
**value-dependent heap allocation** — whether an operation allocates depends on the
magnitude of the operand. That is a timing side-channel, structurally identical to V8's
SMI→HeapNumber boxing.

**Consequence: crypto must use fixed-width `Bytes` (or fixed-width limbs) and never route
secret material through `Int`.** This holds regardless of how carefully the arithmetic is
written; the channel is in the representation, below the arithmetic. The same structural
argument applies to JS `BigInt` — its operations are value-dependent in both time and
allocation, which is why serious JS crypto libraries build over `Uint8Array` rather than
`BigInt` for secret-dependent paths. *(Unsure of the exact wording of noble's published
rationale; the structural claim stands on its own and does not depend on the attribution.)*

Because ADR-0024 is unbuilt, this is cheap to honor **now** and expensive later. It is a
constraint on the `Bytes` protocol, not on `Int`: `Bytes` must never lower to `Int`
internally.

## 10. Open questions

| # | Question | Notes |
|---|---|---|
| B-1 | One class or two (`Bytes`/`BytesMut`)? | §6 takes "one, mutable, fixed-length" **weakly**. Rust's split costs API duplication + `freeze()` churn; the immutable/value-hashable use case may be served by `Tuple` already. Revisit if a `Bytes` `Map` key is ever actually wanted |
| B-2 | Does `slice` share or copy? | §4 rejects a native `slice_`, so `.ph` `slice` copies, O(n). Erlang's sub-binaries are O(1) views — and the cost is the notorious **binary leak**: one small sub-binary retains a whole large refc binary, with `binary:copy/1` as the standard production fix. Under ADR-0050 mark-sweep a shared slice would retain its parent identically. Copy looks right; O(n) is the price |
| B-3 | Fixed length (`Box<[u8]>`) or growable (`Vec<u8>`)? | §2/§7 argue fixed — realloc strands unzeroizable secret copies. But it forecloses a `Bytes` builder; does the builder use `List` then freeze (`Tuple.fromList`'s shape)? |
| B-4 | `equalsConstantTime_` on length mismatch | Node throws (leaking length via the exception path). Return `false` in constant time? Raise? §8 |
| B-5 | Is a literal syntax wanted? | ADR-0029/0032 reserved `#{…}`/`..` without activating. A `Bytes` literal (hex? `0x` blob?) is a lexer question with no current owner |
| B-6 | Should `zeroize` be native after all? | §4 says derivable ⇒ reject. But a native version could also poison the arena slot on free. Does that survive ADR-0019's rule, or is it "speed/paranoia," which the rule explicitly refuses? |
| B-7 | Does admitting `Bytes` freeze ADR-0050's non-moving choice? | §7.2. If yes, that is a real cost to name in the eventual ADR — moving GC is currently "reversibly open" |
| B-8 | `Bytes#hash` — identity or content? | Mutable ⇒ identity per Q5/law 4. But a content hash is the obvious want for digests. §6/B-1 |

## 11. Precedent — each with its cost

| Language | Model | What it cost them |
|---|---|---|
| **Rust** | `Vec<u8>`/`[u8]` distinct from `String`/`str`; `bytes::Bytes`/`BytesMut` for shared/mutable | `String::from_utf8` is fallible ⇒ every decode is a `Result` the caller threads. `Vec<u8>` cannot share cheaply, so `bytes` exists as a **separate crate** with refcounted slices — and its `Bytes`/`BytesMut` split duplicates the API surface and forces `freeze()` at every boundary |
| **Python** | `bytes` (immut) / `bytearray` (mut) / `memoryview` (view) | **Three** types users must choose between correctly. `memoryview`'s export-count rules leak into user code (`BufferError: Existing exports of data: object cannot be re-sized`) — a lifetime discipline in a language with no lifetimes. The `bytes`/`str` split was the single most painful axis of the Py2→3 migration, costing a decade of ecosystem churn |
| **JS** | `ArrayBuffer` (storage) / `Uint8Array` (view) / `SharedArrayBuffer` | The buffer/view split means every API accepts three shapes and none is canonical; **detached** buffers turn into runtime errors at a distance. `SharedArrayBuffer` was **globally disabled post-Spectre** and re-enabling it requires COOP/COEP headers — a hardware side-channel forced an ecosystem-wide deployment tax on a data type |
| **Java** | `byte[]` + `ByteBuffer` | `byte` is **signed** (−128..127) — a permanent papercut forcing `& 0xFF` at every use site, in a language that cannot fix it without breaking the world. `ByteBuffer`'s stateful position/limit/mark API is famously error-prone (`flip()`), and the heap/direct split doubles the mental model |
| **Erlang** | binaries + refc binaries + **O(1) sub-binaries** | The genuinely relevant model for slices — and the cost is the well-known **binary leak**: a small sub-binary retains its large parent indefinitely, with `binary:copy/1` as the standard production remedy. Sharing buys O(1) slicing and pays in retention. Directly informs B-2 |
| **Node** | `crypto.timingSafeEqual` as a native builtin | §8 — it **cannot** be written in JS, which is the precedent. Costs: throws on length mismatch (leaking length), and lives in `crypto`, so the default `===` stays silently wrong |
| **.NET** | `SecureString` | **Deprecated as unfixable** (§7). Shipped assurance it could not deliver, then made every call site a migration |

## 12. What this precludes

- **A `String` that holds arbitrary octets** (§5). Permanent; `toString_` is fallible
  forever, and every decode site pays.
- **Deterministic secret erasure** (§7). Not precluded by this design — precluded by
  having a GC at all. `Bytes` does not make it worse; it makes it *visible*, which is why
  the obligation must be written rather than implied.
- **A cheap moving GC, possibly** (B-7). If a zeroization contract ships, moving stops
  being a free representation choice.
- **`Int` as a secret carrier** (§9.1) — provided this is written down before ADR-0024 is
  built.
- **A `Byte` value type** (§3). Declining the arm is what keeps `Value` at 16 B; taking it
  later means a new dispatch axis, not an additive change.

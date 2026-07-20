# Hashbrown in Phalcom, the next hour — scalar expressibility and the honest deliverables (2026-07-20)

**Status: ANALYSIS.** Companion to [`hashbrown-in-phalcom.md`](hashbrown-in-phalcom.md) (same date,
same HEAD). That document answers "can the SwissTable algorithm be expressed, and would it be worth
it?" (no, and no). This one answers the follow-on question actually asked of it: *what could be done
in roughly one hour to have a full working implementation in idiomatic Phalcom?* It is a plan-shaped
analysis, not a plan — nothing here authorizes a unit.

Inherits the citation discipline of
[`../theory/00-provenance-and-citation-discipline.md`](../theory/00-provenance-and-citation-discipline.md).
Claims tagged **`[V]`** were verified this session by opening the named artifact; **`[P]`** are
inherited from the parent analysis (verified there, not re-opened here); **`[D]`** are derived
arguments — checkable, not yet demonstrated by running code.

Related: [`../spec/v0.2/core/map-and-set.md`](../spec/v0.2/core/map-and-set.md) §4,
[`../deferred/hashbrown-analysis-followups.md`](../deferred/hashbrown-analysis-followups.md),
[`../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md`](../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md),
[`../adr/accepted/0020-kernel-list-native-array-protocol.md`](../adr/accepted/0020-kernel-list-native-array-protocol.md).

---

## 1. The reframe: "idiomatic Phalcom" is not an open question

The phrase "whatever that turns out to be" presumes the idiom is undiscovered. It is not — the tree
ruled it:

- **`[P]`** ADR-0020's kernel pattern — **native storage, `.ph` protocol** — is the committed shape
  for collections, and `map-and-set.md` §4 *explicitly rejected* `.ph`-authored hash tables
  ("O(n) lookup defeats the point; hashing in `.ph` is awkward").
- **`[P]`** `MapObject`'s bucket index is `std::collections::HashMap` — which **is** hashbrown.

So by the tree's own definition, *hashbrown in idiomatic Phalcom already ships*: a SwissTable
underneath, Phalcom `hash`/`==` protocol on top. Nothing is missing, and no hour needs spending.
Every deliverable below is therefore either (a) a teaching artifact that does not claim the Map
seat, or (b) a performance lever on the thing that already holds it.

## 2. The observation the parent analysis leaves implicit

The parent's four blockers (R1 bitwise, R1/R2 integer width, R3 multi-byte load, R5 presized array)
are all blockers for the **word-parallel** form of SwissTable — the SWAR expressions, the group
load, the bitmask iteration. **None of them blocks a scalar SwissTable**, where control bytes are
examined one at a time:

| SwissTable piece | Word-parallel need (blocked) | Scalar substitute (available today) |
|---|---|---|
| ctrl storage | — | `Bytes` — **`[V]`** `.ph`-reachable kernel class, [`core.ph:1206`](../../phalcom-core/core/core.ph), `Bytes.new(n)` used in-kernel |
| tag encoding queries (§1.3 of parent) | 1-instruction bit tests | comparisons: EMPTY = 255, DELETED = 128, full < 128. Three `<`/`==` sends. The one-instruction *property* is lost; the *semantics* are kept |
| `h1` bucket select | `hash & mask` | `hash % capacity` — `%` is among Number's thirteen floor bindings; Phalcom hashes are already 53-bit-masked Numbers |
| `h2` 7-bit tag | `hash >> 57 & 0x7f` | `[D]` high bits via the floor idiom: `q = hash / 2^46; h2 = (q - q % 1) % 128` — no shift needed |
| group scan | one unaligned 8-byte load + SWAR match | per-byte `at(_)` over a group of 8 |
| probe sequence | `(pos + stride) & mask`, power-of-two table | `(pos + stride) % capacity`, `stride += 8` — `%` removes the power-of-two requirement entirely |
| `trailingZeros` | bitmask iteration | unnecessary — the scalar scan visits bytes in order |
| tombstone decision (§1.4 of parent) | `leading_zeros + trailing_zeros >= WIDTH` | count EMPTY slots in the two neighbouring groups by per-byte scan; same predicate, scalar spelling |
| in-place rehash (§1.5 of parent) | ctrl retagging + swap dance | rehash-by-copy. The in-place trick exists to avoid a second allocation in Rust; the arena heap makes the copy unremarkable |
| bucket array | fixed-size `Value` array | `List` grown by `push_` at construction — costs n sends once, works |

**`[D]`** Estimated size: ~200 lines of `.ph` plus golden tests. The semantic content of the
algorithm — probe chain, tombstone discipline, 7/8 load factor, EMPTY-vs-DELETED erase — survives
intact. What is deleted is exactly the word-parallelism the language cannot buy and (per the
parent's §7) would not profit from anyway.

Precedent for the comparison class: **Python's dict** — scalar open addressing in a dynamic
language, no SIMD, entirely respectable. That, not hashbrown, is the fair reference point for what
a `.ph` table can be.

## 3. The three honest hour-sized deliverables

### Option A — scalar SwissTable teaching artifact (recommended if the goal is "hashbrown written *in* Phalcom")

Build §2's table as a `.ph` class in `examples/` (or as a docs/learn fixture), keyed on user
`hash`/`==` — the polymorphism is real and present (parent §8). Positive-lane golden tests.

- **Preclusion check (mandatory):** zero. No floor delta, no new tokens, no spec change; pure `.ph`
  over shipped primitives. It must *not* be proposed as a Map replacement — that would collide with
  `map-and-set.md` §4 from one side and ADR-0019's "speed is never sufficient" admission rule from
  the other.
- **Hazard to document in-file:** no reentrancy guard. Native `Map` carries `#concurrentMutation`
  locking at the mutation site (the ruling recorded in the traceback spec cycle; see the
  `map_key_hash_mutation_*` test fixtures). A `.ph` table inherits none of it — a key whose `hash`
  mutates the table mid-probe corrupts it silently. State this as a known limit; it is consistent
  with the teaching-artifact framing, and it *demonstrates* why the native arm owns the Map seat.

### Option B — the `map_numeric` lever (recommended if the goal is "make the idiomatic thing faster")

The parent's §11 hypothesis: the dominant cost in `Map` lookup is not table layout but the
re-entrant `send_dynamic("hash")` per probe in `locate`
([`primitive/map.rs:55`](../../phalcom-core/src/primitive/map.rs)). **`[P]`** The `InlineCache`
machinery exists ([`chunk.rs:10-18`](../../phalcom-core/src/chunk.rs)) and is probed/refilled on
the send path (`vm/dispatch.rs:445`/`:461`). The hour: cache `hash`-method resolution per
key-class, then measure `map_numeric` before/after under ADR-0051 discipline (in-repo benchmark,
named mechanism, recorded number).

- **Hazard:** methods are open. A per-class `hash` cache must respect redefinition — ADR-0018's
  pristine-flag shape, or an epoch. This is the *inline cache ⊗ mutable hierarchy* hazard on the
  method axis; the hierarchy axis is sealed (ADR-0026/0041) and needs nothing.
- This targets the term that actually dominates. A SwissTable never would (parent §7.3).

### Not doable in an hour — and should not be smuggled in as if it were

Parent §9 steps 2–4 (fixed-width bitwise ruling, multi-byte `Bytes` load, presized `Value` array)
are each **decision records**, one of which (bitwise) carries the unresolved unbounded-`Int`-vs-
wrapping tension reaching back into just-ratified PDR-0012. Governance actions, not hour work —
and per the parent's standing caution, none of them should be picked up *because hashbrown wants
them*. They are filed, unowned, in
[`hashbrown-analysis-followups.md`](../deferred/hashbrown-analysis-followups.md).

## 4. Compatibility

A and B touch disjoint surfaces — A is pure `.ph` in `examples/` + test fixtures; B is native code
plus a benchmark run. No file overlap; they can proceed in parallel. Neither reopens a ruled
question; neither depends on an open one.

## 5. Provenance

**Opened this session:** the parent analysis in full;
[`phalcom/overlay.md`](../../.claude/skills/language-design/phalcom/overlay.md) (language-design
skill); `core.ph` grep for `Bytes` (kernel class at `:1206`, `Bytes.new(_)` exercised in-kernel at
`:1289`, `:1351`).

**Inherited, not re-opened:** every hashbrown `file:line`, every SCOREBOARD number, the four-gap
inventory, and the §7 inversion arithmetic — all carried from the parent with its own warrant tags,
including its caveat that §1.5/§1.6/§2 there are `[R]`-via-delegation.

**Not verified:** no `.ph` code was written or run. §2's scalar-substitution table is `[D]` —
derived, checkable line by line, and falsifiable by the one-hour build it describes. The nearest
negative control would be the build itself.

# U-COLLTYPES — Work order: the native collection runtime classes `Map` / `Set` / `Tuple` / `Range`

_Self-contained implementation plan for **one** implementer (dispatched as four **serialized** phases —
they share the spine files `heap.rs`/`universe.rs`/`core.ph`, so they cannot fan out in parallel; see §4.1).
**Reviewer ON** (deep VM change — new arena arms + a scoped floor amendment) — hand every phase diff to
`phalcom-reviewer`; never self-approve. **Worktree isolation** (mutates `heap.rs`/`universe.rs`/`core.ph`
while U-ITER, U-FIBER, and the U-CORE `core.ph` track are live). Green gate: `./scripts/verify.sh` exits 0 +
`cargo doc --workspace --no-deps` clean, **per phase**. Grounded in
**[ADR-0032](../../../adr/0032-collections-representation-and-literals.md)** (native heap arms, hashing
contract, §1 representation table), **[map-and-set.md](../../../spec/v0.2/core/map-and-set.md)**,
**[tuple-and-range.md](../../../spec/v0.2/core/tuple-and-range.md)**,
**[collection-protocol.md](../../../spec/v0.2/core/collection-protocol.md)** (the binding laws + the
U-CORE-5 conformance harness), **[decisions.md Q5](../../../spec/v0.2/core/decisions.md)** (mutability ⇒
identity hash / immutability ⇒ value hash), and the native-arm precedent
**[ADR-0020](../../../adr/0020-kernel-list-native-array-protocol.md)**. Floor extension governed by
**[ADR-0019](../../../adr/0019-freeze-vm-blessed-primitive-floor.md)** + its amendment convention
([ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md)) — **needs a NEW ADR** (§0,
the load-bearing gate)._

> **Unit-name note.** This unit builds the **runtime classes and their native arms**, NOT their literal
> syntax. The **literals** (`{k:v}` map / `(a,b)` tuple; `#{…}`/`..` reserved) are the separate, existing
> **U-COLL** unit (surface, owned by U-LEX per ADR-0032 §3) — **do not conflate**. `U-COLLTYPES` verified
> free at plan time (`ls docs/forge/units/` — `U-COLL` present, `U-COLLTYPES` absent). This plan edits
> **only** `docs/forge/units/U-COLLTYPES/plan.md`; it does **not** touch `PHASE2-INDEX.md`,
> `units/README.md`, or any spec/src file (concurrent editors).

---

## 0. Phase 0 — the ADR-0019 floor-amendment gate (do this FIRST; it BLOCKS all four phases)

Each native arm needs a **small, scoped set of raw floor primitives** (hash-table get/put/has/remove +
ordered indexed access for `Map`/`Set`; slice access for `Tuple`; bound-field access for `Range`).
[ADR-0032 §1](../../../adr/0032-collections-representation-and-literals.md) *authorizes the pattern* — "each
native arm brings a small, scoped ADR-0019 amendment for its raw primitives … justified per-unit exactly as
`List`'s five raw primitives were" — but ADR-0032 is an **umbrella that changes no runtime**; it does **not**
itself amend the frozen floor. Per [ADR-0019](../../../adr/0019-freeze-vm-blessed-primitive-floor.md), a new
native binding requires "a new superseding ADR that amends this list." The `hash`/reflection floor moves went
through the omnibus [ADR-0023](../../../adr/0023-amend-floor-admit-hash-and-kernel-reflection.md); the
`Method` moves through [ADR-0028](../../../adr/0028-amend-floor-admit-method-reflection.md). **This unit's
container primitives need the same.**

**Gate (BLOCKED-ON-DECISION — see DEC-CT-A):** before Phase 1 emits a single `primitive!`, one of the
following must land:

- **(recommended)** a new **ADR-00NN "amend the floor — admit native collection-container primitives"**
  (drafted by the `documentation-and-adrs` skill), enumerating the raw primitives per class below and
  clearing the ADR-0019 gate for all four in one ratification (as ADR-0023 did for four units at once); **or**
- an **omnibus note** appended to the ADR-0023-style amendment register covering the same set.

The raw-primitive census (the amendment's payload), justified per ADR-0019 §1's **derivability test** — each
reads heap/hash-table representation no existing `.ph`-visible primitive exposes:

| Class | Raw floor primitives (native, internal `raw*`) | Why underivable (ADR-0019 §1) |
|---|---|---|
| `Map` | `Map.class::new()`; `rawSize`; `rawGet(_)`; `rawPut(_,_)`; `rawHas(_)`; `rawRemove(_)`; `rawKeyAt(_)`; `rawValueAt(_)` (**8**) | a hash table keyed by the key's **Phalcom** `hash`+`==`; `.ph` cannot build a hash index (no bucket storage, no handle access), and O(n) `.ph`-over-`List` is the rejected alternative (ADR-0032 §Alternatives). |
| `Set` | `Set.class::new()`; `rawSize`; `rawAdd(_)`; `rawHas(_)`; `rawRemove(_)`; `rawAt(_)` (**6**) | same hash-set representation; may **share the Map backing helper** (a set is a keys-only ordered hash map). |
| `Tuple` | `Tuple.class::fromList(_)`; `rawSize`; `rawAt(_)` (**3**) | a fixed-length immutable `Value` slice — allocation + length + indexed slice read below the `.ph` boundary. `hash`/`==`/`each` stay `.ph` (DEC-CT-D). |
| `Range` | `Range.class::new(_,_,_)`; `rawStart`; `rawEnd`; `rawInclusive` (**4**) | three native fields; everything else (`each`/`size`/`includes`/`first`/`last`/`toList`/`iterate`) is `.ph` over these getters + `Number` arithmetic (laziness ⇒ no element storage). |

**Net floor delta:** **+21 installed bindings** (8+6+3+4), all **new distinct fns** except where `Set` reuses
`Map`'s backing helper (the *Rust helper* is shared; the *bindings* are distinct). This is **larger than
List's five** — precisely why it needs its own ratified ADR rather than riding ADR-0032's umbrella. **Read
`floor-census.md` §1.1 live at dispatch** (it reads **88 installed / 73 distinct fns** as of 2026-07-12) and
land the R-INV-0.1 census bump **in lockstep with each phase's installs** — never hardcode the literal; do
the arithmetic on whatever count HEAD carries.

**Phase 0 return:** the new ADR number (or omnibus note) + a one-line justification per raw primitive, before
any Phase 1 edit. If DEC-CT-A is unresolved at dispatch, **stop** — this is the one hard blocker.

---

## 1. Mission (one sentence)
Build the four native collection runtime classes as arena arms —
`Object::Map`/`Object::Set`/`Object::Tuple`/`Object::Range` over the handle heap
([ADR-0009](../../../adr/0009-handle-arena-heap.md)), mirroring `List`'s `ListObject`
([ADR-0020](../../../adr/0020-kernel-list-native-array-protocol.md)) — each with a **thin native raw-primitive
floor** and its **public protocol authored in `.ph`**, such that all four pass the **U-CORE-5 conformance
harness** and the Q5 mutability/hashability contract holds: `Map`/`Set` are **mutable ⇒ identity `hash`, not
valid keys**; `Tuple`/`Range` are **immutable ⇒ value `hash`, valid keys**.

## 2. Preconditions (verify on actual HEAD — do not assume)

- **U-CORE-1 landed** — `Object#hash` (identity digest of the handle, `object_hash`, `primitive/object.rs`
  L74) + per-immediate value overrides (`number_hash`/`string_hash`/`symbol_hash`/`bool_hash`,
  `universe.rs` L327/339/370/375). This is the **key-hashing precondition**: `Map`/`Set` compute a key's
  hash by **sending `hash`** (§3.1). Confirmed landed (`03764e3`, ADR-0023).
- **U-CORE-5 landed** — the collection-protocol **contract + reusable harness**. `tests/collections_contract.rs`
  is **confirmed present on disk** (the `ContractSpec { class_name, mutable, hashable, ordered }` +
  `assert_sequence_contract(vm, spec, build)` gate); U-CORE-5 wrote `build_list` + `list_satisfies_sequence_contract`.
  **This unit adds one `build_*` closure + one `#[test]` per class** — the harness *is* the definition of
  "conformant" (U-CORE-5 §3.3(a), R-INV-5.4). Re-confirm the `ContractSpec` field names on HEAD before wiring.
- **U-LIST landed** — the native-arm precedent this unit copies verbatim: `ListObject` (`src/list.rs`), five
  raw primitives + `toString` (`src/primitive/list.rs`), `Object::List` (`heap.rs` L93), `alloc_list`/`list`/
  `list_mut`/`as_list` (`heap.rs` L152/323/335/343), `expect_list` (`primitive/mod.rs` L224), the
  `list_class` field + `make_core_class`/`primitive!` registration (`universe.rs` L185/453). `List` is also
  the **`toList` materialization target** for `Range`/`Map#keys`/`Map#values` and the `keys`/`values` return
  type. Landed.
- **U-CORE-2 landed** — `Option`/`Some`/`None` for **total `at(_)`** (`Some(v)`/`None`, never `nil`, never a
  raise — [ADR-0021](../../../adr/0021-no-truthiness-enforcement.md)). `Map#at(_)`, `Tuple#at(_)`, and
  `list_raw_at`'s `None`-on-out-of-range precedent (`primitive/list.rs` L72–79) all depend on it. Landed
  (U-CORE-2 as-built).
- **U-CORE-6 landed (soft, for enforcement only)** — `throw`/`raise` + the unified unwind (ADR-0008,
  ADR-0023 amendment). Needed **only** for the H3 "mutable-key rejected" enforcement (DEC-CT-C). If confirmed
  landed (U-FIBER Phase 0 cites "the just-landed U-CORE-6 unwind"), enforce; the core classes ship regardless.
- **Iteration contract (ADR-0035 / iteration.md)** — each class implements `iterate(_)`/`iteratorValue(_)`
  (§3.5) to be `for`-/`each`-iterable. The **U-ITER** surface (`for`) may not have landed yet; this unit
  ships the two selectors regardless (they are ordinary `.ph` methods over the raw primitives, independent
  of the `for` lowering).
- Baseline `./scripts/verify.sh` green before the first edit. Re-run `graphify affected "heap.rs"`,
  `graphify affected "universe.rs"`, `graphify affected "core.ph"` and **check the concurrent U-ITER /
  U-FIBER / U-CORE `core.ph` editors** (§4.1) — worktree isolation is mandatory.

## 3. Design (ordered phases — realise the specs; do not re-litigate the model)

> Per [ADR-0020](../../../adr/0020-kernel-list-native-array-protocol.md) each collection "its own unit,"
> these **could** dispatch as four sub-units — but they share `heap.rs`/`universe.rs`/`core.ph`, so they
> **serialize** as ordered phases behind one worktree. All four instantiate the **same U-CORE-5 harness**.
> The native-vs-`.ph` split is fixed by ADR-0032 §1: **native arm + raw primitives, `.ph` protocol +
> combinators**. Follow the `List` shape exactly (`list.rs` + `primitive/list.rs` + `core.ph` L163).

### Phase 1 — `Map` + `Set` (the hash collections; the deepest diff — land first)

**Arms.** New `src/map.rs` `MapObject` and `src/set.rs` `SetObject` (or one shared `src/hashcoll.rs`), added as
`Object::Map(MapObject)` / `Object::Set(SetObject)` in `heap.rs` (mirroring `Object::List`). Backing store:
an **insertion-ordered** entry vector `Vec<(Value, Value)>` (Set: `Vec<Value>`) **plus** a
`HashMap<i64, SmallVec<usize>>` index from the key's **Phalcom hash** (a `Number`, truncated to `i64`) to
entry indices. Insertion order gives the protocol's **deterministic iteration** law (map-and-set.md §2:
"stable within a run"); the index gives O(1) lookup. `alloc_map`/`map`/`map_mut`/`as_map` + `expect_map`
follow the `List` accessor set (`heap.rs` L152/323/335/343; `primitive/mod.rs` L224).

**Raw primitives (§0 gate).** `Map`: `new`, `rawSize`, `rawGet(_)`, `rawPut(_,_)`, `rawHas(_)`,
`rawRemove(_)`, `rawKeyAt(_)`, `rawValueAt(_)`. `Set`: `new`, `rawSize`, `rawAdd(_)`, `rawHas(_)`,
`rawRemove(_)`, `rawAt(_)`. Internal (`raw*`), wrapped by `.ph`.

**The key-hashing crux (§Rubric — the load-bearing subtlety).** "Keyed by `hash`+`==`"
([ADR-0032 §1](../../../adr/0032-collections-representation-and-literals.md)) means **Phalcom** `hash`/`==`,
not Rust's `Value: Hash`/`value_eq` (`value.rs` L245/314 — those give **identity** for `Value::Obj`, which is
**wrong** for a value-keyed `Tuple`). So `rawGet`/`rawPut`/`rawHas`/`rawRemove` must **re-enter the VM**:
send `hash` to the key to get its bucket, and send `==` to disambiguate collisions. Two hard constraints:
- **Borrow model (the standing risk).** Never hold a `&Heap`/`&mut Heap` borrow of the map across a
  re-entrant send. Follow the arena discipline `list_raw_at` uses: extract `ObjRef` first, do the
  `send(hash)`/`send(==)` (which needs `&mut VM`), then re-resolve `vm.heap.map_mut(id)` to mutate. A borrow
  held across the send is a compile error at best, a stale-handle miscompile at worst. This is the crown
  fragility — pin it in review.
- **Mutable key rejection (H1/H3, DEC-CT-C).** `rawPut`/`rawAdd` reject a key that `isA` a **mutable**
  collection (`List`/`Map`/`Set`) — their identity `hash` is inconsistent with structural `==`
  (collection-protocol L48–52). Enforced by a raised `Error` (needs U-CORE-6).

**`.ph` protocol (`core.ph`, new `class Map` / `class Set`).** Per map-and-set.md §2/§3:
`Map`: `at(_)`→`rawGet` (returns `Some`/`None`), `at(_,put:)`→`rawPut` (returns `self`), `size`→`rawSize`,
`includes(_)`→`rawHas`, `remove(_)`→`rawRemove` (returns `self`), `keys`/`values` (a `List` in iteration
order, over `rawKeyAt`/`rawValueAt`+`rawSize`), `each(_)` (2-arg `{ k, v => }` over `rawKeyAt`/`rawValueAt`),
`==(_)` (structural: same key set, pairwise-`==` values — `.ph`, guarded by `isA(Map)`), `!=(_)` (routes
through `==`, the §1.3-U-CORE-5 `==`⊗`!=` decoupling hazard). `Set`: `add(_)`, `includes(_)`, `size`,
`remove(_)`, `each(_)` (1-arg), `==(_)` (same members, order-independent), `!=(_)`.

**Hashing.** `Map`/`Set` install **no** `hash` override — they inherit `Object#hash` (identity,
`object_hash`). Mutable ⇒ identity hash ⇒ **not valid keys** (Q5, collection-protocol law 4). Verified by the
harness `hashable=false` branch (H2 skipped).

### Phase 2 — `Tuple` (immutable product; value-hashable, no mutation selectors)

**Arm.** `src/tuple.rs` `TupleObject { elements: Box<[Value]> }` (a **fixed** immutable slice — not a
growable `Vec`), `Object::Tuple(TupleObject)` in `heap.rs`. No `set`/`push` accessors exist (immutability is
structural, not merely convention — tuple-and-range.md §1: "mutation selectors are absent by design").
`alloc_tuple(Box<[Value]>)`/`tuple`/`as_tuple` + `expect_tuple`.

**Raw primitives (§0 gate).** `Tuple.class::fromList(_)` (freeze a `List`'s elements into the slice — the
construction path until the `(a,b)` literal lands in U-COLL), `rawSize`, `rawAt(_)` (`Some`/`None`, total,
mirroring `list_raw_at`). **No mutation primitive.**

**`.ph` protocol (`core.ph`, new `class Tuple`).** `at(_)`→`rawAt`, `size`→`rawSize`, `each(_)` (1-arg over
`rawAt`+`rawSize`), `==(_)` (structural, `isA(Tuple)`-guarded, index-wise), `!=(_)`, `hash` (**value hash**,
DEC-CT-D — recommended `.ph` fold over `rawAt`+element `.hash`, order-sensitive combine). `iterate`/
`iteratorValue` as an integer cursor (identical shape to `List`'s, §3.5).

**Hashing (DEC-CT-D).** Value hash = order-sensitive combine of `element.hash`
(collection-protocol H2). Recommended: a **`.ph` fold** (`hash => …reduce over rawAt with acc = acc*31 +
element.hash…`) — **zero new floor**, and it inherits element hashes so it is automatically Int/Float-split
safe (§5, forward-compat §4). Fallback if `.ph` `Number` arithmetic proves too weak for a good combine: a
native `tuple_hash` primitive (a scoped **+1** on the §0 amendment). Consistency law: `A == B ⇒ A.hash ==
B.hash` (R-INV-1.3) — the fold satisfies it because it is a deterministic function of element hashes, and
`Tuple#==` compares those same elements.

### Phase 3 — `Range` (lazy numeric interval; value-hashable, no element storage)

**Arm.** `src/range.rs` `RangeObject { start: Value, end: Value, inclusive: bool }` (three fields, **no
element buffer** — laziness, tuple-and-range.md §2 RG-2), `Object::Range(RangeObject)`.
`alloc_range`/`range`/`as_range` + `expect_range`.

**Raw primitives (§0 gate).** `Range.class::new(start, end, inclusive)` (until the `..`/`...` sigils
activate in U-COLL — reserved-inactive per ADR-0032 §3.3, **committed convention `a..b` inclusive / `a...b`
exclusive**), `rawStart`, `rawEnd`, `rawInclusive`. **That is the whole floor** — everything else is `.ph`.

**`.ph` protocol (`core.ph`, new `class Range`).** All lazy, over the three getters + `Number` arithmetic:
`size` (`inclusive ? end-start+1 : end-start`, clamped ≥ 0), `includes(_)` (`start <= n && (inclusive ? n <=
end : n < end)`), `first`→`rawStart`, `last` (`inclusive ? end : end-1`), `each(_)` (**generates** `a, a+1,
…`; no allocation), `toList` (materialize into a `List` via `each`+`add` — the explicit escape hatch),
`==(_)` (structural over the generated sequence, or normalized `start`/`end`/`inclusive`; `isA(Range)`-guarded),
`!=(_)`, `hash` (**value hash**, `.ph` fold over `rawStart`/`rawEnd`/`rawInclusive` hashes — immutable ⇒
hashable). `iterate(cursor)`/`iteratorValue(cursor)`: cursor is an integer offset; `iteratorValue` = `start +
cursor`; `iterate` advances while `cursor < size`. Laziness must hold for `Range.new(1, 1000000, true)`
(§Test).

### 3.5 Iteration protocol (all four phases — ADR-0035 / iteration.md §1)

Every class implements `iterate(_)`/`iteratorValue(_)` as **ordinary `.ph` sends** (not sacred, not inlined
— ADR-0035 §4), so `for`/`each`/`map`/`filter`/`reduce` fall out for free once U-ITER lands. The cursor is a
**local** value (integer for `List`/`Tuple`/`Range`/ordered `Map`/`Set` via `rawKeyAt`/`rawAt`) — **never** a
collection-global position (L7 stateless-iteration law; forward-compat §1 fiber-safety). For `Map`/`Set` the
cursor value yielded by `iteratorValue` is a design point: **DEC-CT-E** (recommended: keys for both; `Map`'s
2-arg `each` remains the entry form).

### Rubric — hazards & preclusion (mandatory)

- **`hash` ⊗ mutable key (CROWN JEWEL, Q5 / collection-protocol law 4).** A `List`/`Map`/`Set` used as a
  `Map`/`Set` key has identity `hash` inconsistent with structural `==` → a silent key-lookup miscompile if
  admitted. Handled by **rejection** at `rawPut`/`rawAdd` (DEC-CT-C), not by a value-hash override on mutable
  classes. Pin a negative test (mutable key → raised `Error`).
- **native-frame ⊗ re-entrant key dispatch ⊗ Fiber (map-and-set.md / ADR-0030 §4).** `rawGet`/`rawPut`
  re-enter Phalcom to send `hash`/`==` on keys. A key whose `hash`/`==` tries to `Fiber.yield` sits under a
  native `block_call`-equivalent frame → **`CannotYieldAcrossNativeFrame`** (correct, expected — document
  it; do not engineer around it). No native fiber stacks are introduced (ADR-0009/GC preserved).
- **Borrow-model fragility (STANDING RISK).** The re-entrant `hash`/`==` sends inside the hash primitives
  must re-resolve the `ObjRef` after each send (arena discipline, `list_raw_at` precedent) — **no borrow held
  across a send**. This is the single most likely place to introduce a stale-handle panic or a `RefCell`-free
  aliasing bug. Reviewer must trace every `map_mut`/`send` interleaving.
- **immutability enforced structurally (Tuple).** `TupleObject` uses `Box<[Value]>` and exposes **no**
  mutation accessor — immutability is a representation guarantee, not a missing selector that a later diff
  could re-add by accident.
- **Value repr stays open (forward-compat §1).** Adding `Object::Map/Set/Tuple/Range` heap variants is the
  `List` precedent — **no new `Value` arm** (they are reached through `Value::Obj(ObjRef)`). Any exhaustive
  `Value` `match` this unit touches must keep a `_ =>` arm so a future `Fiber`/numeric arm still compiles
  (`object_hash`'s defensive catch-all, `primitive/object.rs` L82, is the model).
- **inline-cache ⊗ mutable-hierarchy:** N/A — no inline cache here; dispatch is ordinary method lookup.
- **Precedent:** `List`'s native-arm/`.ph`-protocol split (ADR-0020) — the exact template. Rejected
  alternative (`.ph`-over-`List` buckets: O(n) lookup, awkward `.ph` hashing) is in ADR-0032 §Alternatives —
  do not reopen.

## 4. Confirmed write-set (tight; **serialized** across phases — shared spine files)

| File | Why | Phase |
|---|---|---|
| `phalcom-core/src/map.rs` + `src/set.rs` (**new**; or one `src/hashcoll.rs`) | `MapObject`/`SetObject` backing structs (ordered entries + hash index) | 1 |
| `phalcom-core/src/tuple.rs` (**new**) | `TupleObject { Box<[Value]> }` | 2 |
| `phalcom-core/src/range.rs` (**new**) | `RangeObject { start, end, inclusive }` | 3 |
| `phalcom-core/src/heap.rs` **(SPINE)** | `Object::Map/Set/Tuple/Range` variants + `alloc_*`/`*`/`*_mut`/`as_*` accessors | 1,2,3 |
| `phalcom-core/src/primitive/map.rs` + `set.rs` + `tuple.rs` + `range.rs` (**new**) | the raw floor primitives per class | 1,2,3 |
| `phalcom-core/src/primitive/mod.rs` **(SPINE)** | `expect_map`/`expect_set`/`expect_tuple`/`expect_range` + `mod` decls | 1,2,3 |
| `phalcom-core/src/lib.rs` (or `mod.rs`) **(SPINE)** | `mod map/set/tuple/range;` declarations | 1,2,3 |
| `phalcom-core/src/universe.rs` **(SPINE)** | `make_core_class` for each + `Classes` fields + `install_primitives` + **floor-census bump** (§0) | 1,2,3 |
| `phalcom-core/core/core.ph` **(SPINE — never two editors)** | `class Map`/`Set`/`Tuple`/`Range` `.ph` protocol | 1,2,3 |
| `phalcom-core/tests/collections_contract.rs` | `build_map`/`build_set`/`build_tuple`/`build_range` + one `#[test]` each (extend U-CORE-5 harness) | 1,2,3 |
| `phalcom-core/tests/lang/collections/` + `tests/lang.rs` | `.ph` golden corpus + per-class extras | 1,2,3 |
| `phalcom-core/tests/invariants.rs` | R-INV-0.1 census bump (+21) + arm-registration invariant | 1,2,3 |
| [`0039-amend-floor-admit-collection-container-primitives.md`](../../../adr/0039-amend-floor-admit-collection-container-primitives.md) (**Proposed, Phase 0**) | the §0 ADR-0019 amendment | 0 |

**Deliberately NOT in scope:** any **literal** parser/lexer/compiler surface (`{k:v}`/`(a,b)`/`#{…}`/`..` —
that is **U-COLL** / U-LEX, ADR-0032 §3); the `for`/`break`/`continue` lowering (**U-ITER**); `Fiber`
machinery (**U-FIBER**); the combinator library beyond what falls out of the protocol (`map`/`filter`/`reduce`
are the existing `.ph` U-STD defaults — they inherit onto each new class via the protocol, no rewrite here);
`value.rs` `value_eq`/`Value: Hash` (**no change** — structural `==` is `.ph`, key hashing sends Phalcom
`hash`; listed to make the non-edit explicit).

### 4.1 Write-set collision risk (flag, don't resolve)
- **`heap.rs` / `universe.rs` — spine files.** Confirm no concurrent unit holds them. **U-FIBER** also
  mutates `heap.rs` (`Object::Fiber`) and `universe.rs` — **serialize** against it (worktree isolation
  mandatory for both; whichever lands second rebases its arm additively — both are pure new `Object`
  variants, so the merge is additive, not conflicting, but must not be co-scheduled).
- **`core.ph` — never two editors.** **U-ITER** (`List#iterate`), **U-FIBER** (`class Fiber`), and the
  **U-CORE `core.ph` track** all edit `core.ph`. U-COLLTYPES's four new `class` blocks must **serialize**
  against every one of them. The `src/*.rs` + `primitive/*.rs` new files are collision-free and can be
  written first.
- **The four phases share every SPINE file** → they **cannot** parallelize with each other. Ordered:
  Map+Set → Tuple → Range.

## 5. Build order (small, independently-green diffs)

0. **Phase 0 gate** — ratify [ADR-0039](../../../adr/0039-amend-floor-admit-collection-container-primitives.md) (drafted, Proposed; DEC-CT-A). No code until it is Accepted.
1. **Phase 1a — `Map`/`Set` arms + raw primitives (no `.ph`).** `MapObject`/`SetObject`, `Object::Map/Set`,
   `alloc_*`/accessors, `expect_*`, the raw primitives, class registration + census bump. Rust-level unit
   tests on the raw primitives (put/get/has/remove/keyAt) directly. Green (existing suite untouched — pure
   additive arms).
2. **Phase 1b — `Map`/`Set` `.ph` protocol + harness + goldens.** `class Map`/`Set` in `core.ph`;
   `build_map`/`build_set` + `#[test]`s in `collections_contract.rs`; `collections/` goldens; the
   mutable-key-rejection negative (DEC-CT-C). Green.
3. **Phase 2 — `Tuple`.** Arm + 3 raw primitives + `class Tuple` `.ph` (incl. value-hash fold, DEC-CT-D) +
   `build_tuple` `#[test]` + value-hash-equality + immutability goldens. Green.
4. **Phase 3 — `Range`.** Arm + 4 raw primitives + `class Range` `.ph` (lazy `each`/`size`/`includes`/
   `first`/`last`/`toList`) + `build_range` `#[test]` + inclusive/exclusive parity + large-range-laziness +
   `toList` round-trip goldens. Green.

Each phase is a self-verifiable commit; commit per green checkpoint (never a non-compiling tree).

## 6. Mandatory rules
- **Docs:** `///` on every new `*Object` struct + field, every `Object::` variant, every raw primitive
  (`//!` on each new module), citing ADR-0032 §1 / the §0 amendment ADR / the owning spec §.
  `cargo doc --workspace --no-deps` adds no warnings.
- **Green gate (per phase):** `./scripts/verify.sh` exits 0; no new clippy; **no `Rc`/`RefCell`** (mutation
  goes through `&mut Heap` like every heap object — ADR-0020 "no borrow-panic surface"); **no `unsafe`**.
  Follow `rust-best-practices`; run `rust-sanitizers-miri` on the re-entrant hash primitives if the miri lane
  is wired.
- **Reviewer ON**; **worktree isolation**. Every phase diff goes to `phalcom-reviewer`; the writer never
  self-approves.

## 7. Test strategy (the green gate must assert) — `collections/` label + `collections_contract.rs` gate

- **Conformance (all four, the gate):** each class instantiates the **U-CORE-5 harness** —
  `assert_sequence_contract(vm, &spec, build_*)` asserts L1–L7 (size total + = count, `at` total in range,
  `at(n)`→`None`, `add` grows for mutable, `each` visits `size` once in order, stateless/reentrant cursor),
  E1–E6 (structural `==`/`!=` algebra), and H1/H2 (hashable-iff-immutable: `Map`/`Set` `hashable=false` →
  H2 skipped; `Tuple`/`Range` `hashable=true` → `A==B ⇒ A.hash==B.hash` asserted).
- **`Map` extras:** hash-collision correctness (two keys with equal `hash` but `!=` coexist), key overwrite
  (`at(k,put:v1).at(k,put:v2)` → `size` unchanged, `at(k)`→`Some(v2)`), `remove` idempotence
  (`remove(absent)` is a no-op returning self), `None` on missing key, structural `==` (same entries, order-
  independent), `keys`/`values` in iteration order.
- **`Set` extras:** `add` idempotence (duplicate → `size` unchanged), membership, `remove` idempotence,
  structural order-independent `==`.
- **Mutable-key rejection (NEGATIVE, DEC-CT-C):** `aMap.at(aList, put: 1)` (and `aSet.add(aMap)`) raises a
  catchable `Error` — never silently identity-keys a mutable collection.
- **`Tuple` extras:** value-hash equality (`Tuple.fromList([1,2]).hash == Tuple.fromList([1,2]).hash`);
  structural `==` (`(1,2) == (1,2)`, `(1,2) != (1,3)`, cross-kind `(1,2) != [1,2]`); immutability (no
  `add`/`at(_,put:)` selector — a `doesNotUnderstand`, asserted); `Tuple` as a **valid `Map` key**
  (`aMap.at(Tuple.fromList([1,2]), put: 9)`, then `at(Tuple.fromList([1,2]))` → `Some(9)` — proves value-key
  lookup through the re-entrant `hash`+`==` path).
- **`Range` extras:** inclusive/exclusive parity (`Range.new(1,5,true).toList == [1,2,3,4,5]`,
  `Range.new(1,5,false).toList == [1,2,3,4]`); `size`/`first`/`last`/`includes` parity; **laziness** —
  `Range.new(1, 1000000, true)` builds + reports `size`/`includes(500000)` **without** materializing (assert
  it returns promptly and allocates no million-element buffer — a timing/allocation golden or a bounded
  `each` that `break`s early once U-ITER lands); `toList` round-trip on a small range.
- **Invariant (`invariants.rs`):** R-INV-0.1 census reads the bumped count (**+21**); a new invariant asserts
  each class's arm is registered and answers `size`/`at`/`each`/`==`/`hash` from a live `VM::new()`.
- **Do NOT duplicate** `List`'s own `tests/lang/list/` goldens — the `collections/` corpus guards the
  **shared contract** (U-CORE-5 §3.3(b) rule).

## 8. Decisions flagged (flag, don't pick)

| ID | Decision | Options | Architect recommendation |
|---|---|---|---|
| **DEC-CT-A** ⚠️ **ADR DRAFTED (Proposed) — the §0 gate** | The **ADR-0019 floor amendment** admitting the ~21 native container primitives. ADR-0032 authorizes the *pattern* but does not itself amend the floor. | Recommendation was (A) one standalone ADR over all four classes. | **Drafted as [ADR-0039](../../../adr/0039-amend-floor-admit-collection-container-primitives.md) (Proposed, +21, census 80→101 per-phase)**, enumerating all four classes' raw primitives. **Awaiting user ratification** — no Phase-1 code until Accepted. |
| **DEC-CT-B** | `Map`/`Set` **shared backing** vs separate structs. | **(A)** `Set` reuses the `Map` ordered-hash helper (keys-only); **(B)** independent `SetObject`. | **(A)** — a set *is* a keys-only ordered hash map; sharing the helper halves the re-entrant-hash surface (one place for the borrow-model review). Distinct `Object` variants + distinct bindings, shared Rust helper. |
| **DEC-CT-C** | **Enforce** mutable-key rejection now (needs U-CORE-6), or specify-only (U-CORE-5's deferral)? | **(A)** enforce: `rawPut`/`rawAdd` raise on a mutable-collection key; **(B)** leave identity-keyed (silently wrong per Q5). | **(A)** if U-CORE-6 is confirmed landed on HEAD — this unit is the *consumer* U-CORE-5 §2.4 deferred enforcement to. Ship the rejection + negative test. If U-CORE-6 is somehow not landed, degrade to (B) with a `DEFERRED.md` pointer and a `pending/` negative. |
| **DEC-CT-D** | `Tuple`/`Range` value **hash**: `.ph` fold vs native primitive. | **(A)** `.ph` fold over element/field `.hash`; **(B)** native `tuple_hash`/`range_hash`. | **(A)** — zero extra floor, and it inherits element hashes so it survives the Int/Float split automatically (forward-compat §4). Fall back to **(B)** (a scoped +1/+1 on the §0 amendment) *only* if `.ph` `Number` arithmetic can't express a serviceable combine — verify on HEAD what bitwise/wrapping ops `Number` exposes before choosing. |
| **DEC-CT-E** | What `Map`/`Set` `iterate`/`iteratorValue` **yields** per step (`for (x in m)` binds `x` to…). | **(A)** keys (both); **(B)** `(k,v)` `Tuple` entries for `Map`; **(C)** values. | **(A)** — keys for both (Python-precedented for `Map`; the only sensible choice for `Set`); `Map#each(_)` stays the 2-arg `{ k, v => }` entry form (map-and-set.md §2). `(k,v)`-`Tuple` iteration can be added later additively via a `Map#entries` view — not precluded. |

## 9. Must-not-preclude check (forward-compat.md / ADR-0021 / ADR-0035)

- **No global-stack assumption in iteration (forward-compat §1, L7).** Every `iterate`/`each` holds its
  cursor in a **local** value and never stores a position on the collection — so iteration stays reentrant
  and **fiber-local** when `Fiber` relocates the stack behind `current` (U-FIBER). The re-entrant key-hash
  sends touch no global-stack assumption; they route through ordinary dispatch. ✅ (proved by a reentrant-
  `each` golden per class).
- **Hash-by-value survives the Int/Float split (forward-compat §4).** `Tuple`/`Range` value hash is a combine
  of `element.hash`/`field.hash`, **never** of representation bits — so when `Number` splits into bignum
  `Int` + `Float` and `2`/`2.0` hash equal (the ADR-0024 constraint), `Tuple(2)` and `Tuple(2.0)` keep
  hashing equal with **no** change here. `Map`/`Set` key hashing likewise **sends** `key.hash` (never reads
  bits), inheriting whatever the numeric hash decides. ✅ (This is the "must survive the future Int/Float
  split — hash by mathematical value" gate.)
- **`at(_)` → `Option`, never `nil` (ADR-0021 / Invariant 4).** `Map#at(_)` and `Tuple#at(_)` return
  `Some`/`None`; out-of-range/missing is the `None` singleton (`list_raw_at` precedent), never `nil`, never a
  raise. ✅
- **`Value` stays open (forward-compat §1 / ADR-0010).** Four new `Object` arms, **zero** new `Value` arms;
  any `Value` `match` touched keeps a `_ =>` default (a future `Fiber` arm still compiles). ✅
- **Literals not precluded (ADR-0032 §3, U-COLL).** The classes are built with class-side constructors
  (`Map.new`, `Set.new`, `Tuple.fromList`, `Range.new`); the reserved `{…}`/`(…)`/`#{…}`/`..` sigils later
  desugar to exactly these construction sends (ADR-0032 §3.1/§3.2/§3.3) — U-COLL adds surface, not floor. The
  `Range` inclusive/exclusive convention is honoured now (RG-1) so the `..`/`...` activation is not a fresh
  decision. ✅
- **Combinators not precluded (ADR-0035 §5).** By implementing `iterate`/`iteratorValue` + `each`, every
  class inherits `map`/`filter`/`reduce` from the existing U-STD `.ph` defaults the day U-ITER migrates them
  onto the protocol — no per-class combinator code here. ✅
- **Moving/tracing GC (ADR-0009).** Backing stores live **inside** the arena `Object` (no native-memory side
  tables) → a future collector reaches every element/entry as an ordinary arena root. ✅

## 10. Return contract (report to `phalcom-reviewer`)

The §0 amendment ADR number + the ratified raw-primitive census per class · the four `Object::` arms + backing
structs (confirmed **no new `Value` arm**) · the `Map`/`Set` re-entrant key-hash design + the borrow-model
discipline proof (no borrow held across a send) · the mutable-key-rejection enforcement (DEC-CT-C) + its
negative golden · `Tuple` immutability (`Box<[Value]>`, no mutation accessor) + the value-hash choice
(DEC-CT-D: `.ph` fold or native, and why) + the `Tuple`-as-valid-`Map`-key round-trip · `Range` laziness proof
on `Range.new(1, 1000000, true)` + inclusive/exclusive parity + `toList` round-trip · all four `build_*`
closures + `#[test]`s green against the **U-CORE-5 harness** · how DEC-CT-B/C/D/E resolved · the exact floor
delta (**+21**, or +22/+23 if DEC-CT-D fell back to native) with the R-INV-0.1 census bump · confirmation
**zero `unsafe` / zero `Rc`/`RefCell`** · worktree/serialization notes vs U-ITER, U-FIBER, and the U-CORE
`core.ph` track · `verify.sh` + `cargo doc` (+ miri) tails per phase.

## 11. As-built correction (post-review, 2026-07-12) — history-honesty amendment

`phalcom-reviewer` returned **BLOCK** on the *commit record*, not the tree. The final HEAD
`10e1715` is **functionally correct, green, and sound** — every load-bearing check passed
(re-entrant hash borrow model, Q5 hash contract, exhaustive `Object::` match arms, floor
census 88→102→105→109 = +21 verified by the live `floor_census_matches_installed_bindings`
test, write-set disjointness vs U-ITER-FIX, and the legitimate hand-rolled `Range` harness).

The defect is bisectability + a false commit message, isolated to the phase boundary:

- **`2d140f0` ("Phase 2 — native Tuple")** added only `src/tuple.rs` + `src/primitive/tuple.rs`
  as **orphaned, unreferenced files** — no `mod tuple`, no `Object::Tuple` arm, no `universe.rs`
  registration, no `core.ph` `class Tuple`, no census bump, no `build_tuple` test. Its commit
  message claims a `.ph` class, an `invariants.rs` bump to 105, and passing `build_tuple` /
  `tuple_is_a_valid_map_key` tests — **none of which exist in that diff** (`git show
  2d140f0:phalcom-core/tests/invariants.rs` still reads 102). Tuple is not a runtime class here.
- **`f934cf1` ("Phase 3 — native Range")** silently bundles **all** of Phase 2's missing spine
  wiring (both `Object::Tuple` and `Object::Range` in `heap.rs`; both classes in `universe.rs`
  and `core.ph`; the `build_tuple`/`build_range`/tuple-hash/tuple-key tests; the whole census
  jump to 105→109). One vm.rs hunk even labels a line "Phase 2" inside the Phase-3 commit.

**Authoritative record:** the true "Phase 2 (Tuple) landed" state does **not** exist at
`2d140f0`; it exists only at `f934cf1`. Do **not** `git bisect` against `2d140f0` expecting a
working `Tuple`, and do not trust that commit's message as a description of its own diff.

**Why not squashed:** the commits are on shared `main` with live concurrent worktrees; rewriting
pushed `main` history is forbidden (concurrent-session stash/rebase hazard). Remedy is this
append-only correction note (reviewer's option (b)), not a history rewrite.

**Disposition: unit ACCEPTED on functional grounds at `10e1715`.** Block cleared by this note.
Reviewer's non-blocking observations (swap_remove doc-precision; `at(_)`→raw-value vs spec-prose
`Some(v)`, pre-existing and mirroring `List#at`; `Range#==` bound-equality by design;
`Symbol#==` defect already filed) → tracked in `DEFERRED.md`, none blocking.

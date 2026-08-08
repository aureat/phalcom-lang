# Could hashbrown be written in Phalcom? — analysis (2026-07-20)

**Status: ANALYSIS.** Not a proposal, not a plan, and it does not authorize a unit. Grounded in
Phalcom HEAD `8ed448c` and in `hashbrown` **0.17.1** read at
`/Users/altunhasanli/dev/phalcom/repos/hashbrown` (out-of-tree; no vendored copy exists here,
so its citations are `file:line` without links).

Inherits the citation discipline of [`../theory/00-provenance-and-citation-discipline.md`](../theory/00-provenance-and-citation-discipline.md).
Every claim carries a warrant tag — **`[V]`** verified by opening the named artifact,
**`[M]`** measured (with the instrument named), **`[R]`** recalled, **`[X]`** refuted,
**`[O]`** open. §10 is the provenance ledger: what was actually opened, and by whom.

Related: [`../spec/current/stdlib/map-and-set.md`](../spec/current/stdlib/map-and-set.md) §4 (which already
ruled the adjacent question), [`../spec/current/stdlib/bytes.md`](../spec/current/stdlib/bytes.md),
[`../../spec/library/numbers/numeric-tower.md`](../../spec/library/numbers/numeric-tower.md),
[`../pdr/0012-numeric-tower-implementation-and-floor-amendment.md`](../pdr/0012-numeric-tower-implementation-and-floor-amendment.md),
[`../spec/current/memory-management.md`](../spec/current/memory-management.md),
[`../forge/perf-log/SCOREBOARD.md`](../forge/perf-log/SCOREBOARD.md).

---

## 0. The question, and two different answers

"Can hashbrown be implemented in Phalcom?" hides two questions that come apart hard:

1. **Is the SwissTable algorithm expressible in the Phalcom language today?** — No. Four
   missing capabilities, each individually fatal (§4).
2. **If they were supplied, would porting it be worth doing?** — Also no, and this is the more
   interesting answer, because it stays true after every gap in (1) is closed (§7).

The two answers have different shelf lives. (1) is a snapshot that ratified-but-unbuilt work
will partly invalidate. (2) is a statement about the *shape* of a message-send VM and does not
expire.

There is also a third thing worth stating up front, because it is easy to miss and it reframes
the whole exercise: **hashbrown already runs underneath every Phalcom `Map`.** `MapObject`'s
bucket index is a `std::collections::HashMap` (§6), and std's `HashMap` *is* hashbrown. The
question is not whether Phalcom can reach SwissTable — it is already standing on one — but
whether the *language* can express it. Those are different claims and the distinction is the
substance of this document.

---

## 1. What hashbrown actually requires

Read as an inventory of language capability, not as a description of the algorithm.

### 1.1 The probe, reduced

Five operations per lookup:

| Step | hashbrown 0.17.1 |
|---|---|
| `h1 = hash as usize` (bucket select) | `src/raw.rs:61-64` |
| `h2 = (hash >> (MIN_HASH_LEN*8 - 7)) & 0x7f` (7-bit tag) | `src/control/tag.rs:35-49` |
| load 8/16 control bytes as one word | `src/control/group/generic.rs:73` (`ptr::read_unaligned`) |
| byte-parallel match against `h2` | `generic.rs:105-110` |
| `trailing_zeros() / BITMASK_STRIDE` | `src/control/bitmask.rs:39-70` |

Plus triangular probing, `pos = (pos + stride) & bucket_mask` with `stride += Group::WIDTH`
(`raw.rs:83-92`), which is total over a power-of-two table.

**`[V]`** `MIN_HASH_LEN = min(size_of::<usize>(), size_of::<u64>())` (`tag.rs:37-41`) — the h2
shift is width-adaptive, not a fixed `>> 57`. This exists to handle `usize`-width hashers such
as FxHash on 32-bit targets. Worth noting because a naive port hardcodes 57 and silently
degrades tag quality on any platform where the assumption breaks.

### 1.2 The SWAR fallback is the real target

`src/control/group/mod.rs:8-46` dispatches SSE2 → NEON → LSX → generic. **`[V]`** Miri always
forces the generic path — the intrinsics are opaque to it. So `generic.rs` is not a legacy
fallback kept for embarrassment; it is a first-class, continuously-exercised implementation.
A port to a language with no SIMD is therefore a port of `generic.rs`, and that is the fair
test. The intrinsic backends are an optimization on top, not the thing being ported.

`generic.rs` in full is 153 lines. Its three load-bearing expressions:

```rust
// match_tag — Stanford "value in word" bithack, :105-110
let cmp = self.0 ^ repeat(tag);
BitMask((cmp.wrapping_sub(repeat(Tag(0x01))) & !cmp & repeat(Tag::DELETED)).to_le())

// match_empty — top TWO bits, distinguishing EMPTY(0xFF) from DELETED(0x80), :119
BitMask((self.0 & (self.0 << 1) & repeat(Tag::DELETED)).to_le())

// convert_special_to_empty_and_full_to_deleted, :149-150
let full = !self.0 & repeat(Tag::DELETED);
Group(!full + (full >> 7))
```

**`[V]`** These two adjacent functions depend on *opposite* properties of fixed-width integer
overflow. `match_tag` works **because** borrow propagates across byte lanes under
`wrapping_sub`. `convert_special_…` works **because** carry does **not** propagate — the
non-wrapping `+` at `:150` is safe only under the case analysis spelled out in the comment at
`:142-148`. Flag this now; §5 is about it.

**`[V]`** `match_tag` is documented to return **false positives** when a tag differs from the
search value only in its lowest bit (`generic.rs:96-103`). Tolerated because the subsequent
`==` catches it, it never happens for EMPTY/DELETED, and it only occurs when there is at least
one true match. A correct port must preserve the *tolerance*, not just the expression — an
implementer who "fixes" the false positive has misunderstood the contract.

### 1.3 Tag encoding is chosen for the SWAR, not for readability

**`[V]`** `src/control/tag.rs:9-29`: `EMPTY = 0b1111_1111`, `DELETED = 0b1000_0000`, full = top
bit clear. Three properties are being bought at once:

- `is_full` is one bit test (`& 0x80 == 0`, `:17`)
- `special_is_empty` is a *different* single bit test (`& 0x01`, `:29`)
- `match_empty` can separate EMPTY from DELETED with `x & (x << 1)` — no comparison

This is a good example of a representation chosen so that three different queries each collapse
to one instruction. Any port that treats the tag values as arbitrary sentinels loses all three
and does not notice, because the code still works.

### 1.4 Why tombstones exist at all

**`[V]`** `raw.rs:3243-3284`. `find_inner` terminates on EMPTY, so erasing to EMPTY could
truncate a probe chain and make a live entry unreachable. `erase` decides EMPTY vs DELETED by
checking whether the surrounding two groups contain any EMPTY slot:

```rust
empty_before.leading_zeros() + empty_after.trailing_zeros() >= Group::WIDTH  // → must use DELETED
```

Only the EMPTY branch reclaims `growth_left` (`:3282`). A consequence noted in-tree at
`:3273-3275`: **tables smaller than `Group::WIDTH` can never contain DELETED.**

### 1.5 Resize and in-place rehash

**`[V]`** `raw.rs:2740-2794` — policy: if `new_items <= full_capacity / 2`, rehash in place to
reclaim tombstones; else grow, floored at `full_capacity + 1` to prevent delete-churn thrash.

**`[V]`** `raw.rs:2985-3078` — `rehash_in_place` is the cleverest thing in the file:

1. Bulk `convert_special_to_empty_and_full_to_deleted` over all groups, so DELETED now means
   "not yet rehashed" and EMPTY means "free". One pass, no extra state.
2. Install a `ScopeGuard`; on panic, any still-DELETED entry is dropped, ctrl set EMPTY,
   counters recomputed.
3. For each DELETED slot: if the new index is in the same group, just retag (the `likely` case,
   `:3044`). If the target is EMPTY, `copy_nonoverlapping`. If the target is DELETED,
   `swap_nonoverlapping` and **reprocess the displaced element in the same slot** — the `loop`
   at `:3029` whose only exit is `continue 'outer`.

**`[V]`** `raw.rs:2991-2994` documents that a panicking user `Hash` mid-rehash causes
unrehashed elements to be *dropped*, because their hash is unrecoverable. Panic safety here is
structural, not decorative.

### 1.6 Capacity policy

**`[V]`** `raw.rs:104-164`. Above 15, `(cap * 8 / 7).next_power_of_two()` — 87.5% max load.
Below 15, a lookup table on `(Group::WIDTH, table_layout.size)` picking 3/7/14, existing purely
to avoid wasting bytes on ctrl-alignment padding for tiny tables. The in-tree comment at
`:130-131` flags this as brittle if 32-byte groups are ever added. **`[V]`** `bucket_mask_to_capacity`
(`:182-191`) special-cases `< 8`.

---

## 2. The half that dissolves

`src/raw.rs` is 4627 lines. **`[V]`** The overwhelming majority of it is not SwissTable. It is
manual-memory engineering, and every item below is an artifact of *how Rust owns memory*, not
of *how the algorithm works*:

| Mechanism | Where | What it is for |
|---|---|---|
| one allocation split into a downward data region and upward ctrl region, sharing one pointer (`data_end() == self.ctrl.cast()`) | `raw.rs:2417-2441` | avoid two allocations and a second indirection |
| manual `Layout` arithmetic, alignment round-up by mask, `isize::MAX - (ctrl_align-1)` overflow guard | `raw.rs:216-235` | `Layout` combinators not stable; cites rust-lang/rust#95295 |
| `set_ctrl` writes **every** ctrl byte twice, branchlessly, at `index` and `index2` | `raw.rs:2565-2596` | so unaligned `Group::load` is legal at any position with no wraparound branch |
| `fix_insert_index` — a whole repair path with a rescan | `raw.rs:1668-1746` | trailing EMPTY ctrl bytes falsely match when `num_buckets < Group::WIDTH` |
| ZST escape hatch via `ptr::without_provenance_mut` | `raw.rs:416` | `Bucket<T>` for zero-sized `T` has no real address |
| `NonNull` chosen for variance **and the null niche** | `raw.rs:243-253` | makes `Option<Bucket<T>>` free |
| `#[may_dangle]` dropck eyepatch (nightly) vs plain `Drop` (stable) | `raw.rs:3485-3502` | a real semantic difference, not perf |
| `TrivialClone` specialization that memcpy's the whole allocation | `raw.rs:3394-3421` | clone fast path |
| `SizedTypeProperties::{IS_ZERO_SIZED, NEEDS_DROP}` skip-loops | `raw.rs:51-56`, used `:2213`, `:3821` | don't run drop glue that does nothing |
| `&mut dyn FnMut` / `&dyn Fn` used *deliberately to shrink codegen* | `raw.rs:1799`, `:2890` | monomorphization bloat; comments note LLVM devirtualizes after inlining |
| over-sized-allocation exploitation (`block.len() != layout.size()`) | `raw.rs:1581-1594` | use allocator slack |
| the empty singleton: `#[repr(C)]` struct with a `[Group; 0]` alignment field | `generic.rs:57-67`, `raw.rs:1495-1505` | an unallocated table still needs a loadable ctrl pointer |

**None of it survives translation into Phalcom, and not because Phalcom is too weak — because
Phalcom's heap makes the problems it solves not exist.**

**`[V]`** [`../spec/current/memory-management.md`](../spec/current/memory-management.md) §1: all heap
objects live in one `Heap`, a generational arena (`SlotMap<ObjRef, Object>`); every object is
named by a `Copy` handle (index-plus-generation), **not a pointer**. Sweeping bumps the
generation, so a stale handle resolves to a defined diagnostic, "never to undefined behaviour."

Consequences, one per row above: no `Layout`, no alignment arithmetic, no provenance, no ZSTs,
no dropck, no drop glue to skip, no monomorphization to fight, no allocator slack to reclaim,
no null niche to exploit — and, decisively, **no reason to mirror ctrl bytes**, because there
is no unaligned load whose legality needs buying. The single most intricate mechanism in
`raw.rs` (`set_ctrl`, plus the entire `fix_insert_index` repair path it necessitates) exists to
make a raw-pointer trick sound. A language without raw pointers pays nothing and gets nothing.

**`[V]`** `memory-management.md` §4 / Invariant M4: "No finalization. Object destruction runs no
user code. There is no `Drop` protocol, no `finalize`, no resurrection." So element destruction
— a recurring complication throughout `raw.rs` — is free.

This is the real headline. **A SwissTable in Phalcom would be a fraction of hashbrown's size,
and the deleted part is the part everyone finds impressive.** What is left is small, and §3
states it.

---

## 3. The irreducible requirement set

After §2, what a Phalcom SwissTable would genuinely need:

| # | Requirement | Why irreducible |
|---|---|---|
| R1 | fixed-width integers with `^ & \| ~ << >>` | the three SWAR expressions in §1.2 |
| R2 | **wrapping** subtraction, and non-wrapping addition, both with defined lane behavior | `match_tag` needs borrow propagation; `convert_special_…` needs its absence |
| R3 | load N contiguous octets as one integer | `Group::load`; without it the group scan is per-byte and the whole idea is gone |
| R4 | `trailingZeros` / `leadingZeros` | `BitMask::lowest_set_bit`, and the `erase` EMPTY-vs-DELETED decision (`raw.rs:3279`) |
| R5 | fixed-size array of values, addressable by index | the bucket array |
| R6 | unwind-safe cleanup around a user-supplied `hash` | `ScopeGuard` during rehash (`raw.rs:2999-3015`) |
| R7 | caller-supplied hash/equality as first-class functions | `RawTable` is hash-agnostic: takes `u64` + `impl Fn(&T) -> u64` (`raw.rs:909`, `:1024`) |

Two of the seven are already satisfied, and both are worth noticing:

**`[V]` R7 is satisfied.** `RawTable` never bounds on `Hash`/`Eq` at all — the trait machinery
lives entirely in `map.rs` (`make_hash` at `src/map.rs:236-239`, `make_hasher` at `:206-211`),
and `Borrow`/`Eq` are reachable only through the `Equivalent<K>` blanket impl at
`src/lib.rs:138-159`. The raw layer takes `u64`s and closures. Phalcom's block/closure surface
covers that shape directly, and Phalcom's user-overridable `hash`/`==` covers the layer above
it (§6). **Generics are not the blocker.** This is the opposite of what one expects when asking
whether a Rust container ports to a dynamic language.

**`[V]` R6 is satisfied, despite Phalcom having no destructors.** `ScopeGuard`
(`src/scopeguard.rs`, used at `raw.rs:2999-3015`, `:3470`, `:1468`) is cleanup driven by
*unwinding*. Phalcom's `ensure` is exactly that mechanism, and `memory-management.md` §4 says so
explicitly: cleanup "is driven by unwinding (ADR-0008), never by collection." A GC'd language
with no `Drop` nevertheless has the precise primitive this algorithm needs, because the need was
never about destruction — it was about unwinding.

R1–R5 are all absent. §4.

---

## 4. The gap, item by item

All claims below **`[V]`**, verified this session against HEAD `8ed448c` by opening the named
file.

### 4.1 R1 — bitwise operators are not lexable

Not "unimplemented primitives." **Unlexable tokens.** A grep for `BitAnd|BitOr|BitXor|Shl|Shr|Tilde|Caret|Ampersand|Pipe` across all of `phalcom-ast/src/` returns **0 hits**. There is no
token, no AST node, no parse rule.

Corroborated by the kernel working around its own absence:

```
phalcom-core/core/core.ph:107
  // or mid-sequence. UTF-8 decode via division/modulo (no bitwise ops).
```

A UTF-8 decoder written with `/` and `%` because the shifts do not exist. That is the most
direct possible evidence, and it is in the core library.

Also stated in the record: `docs/adr/accepted/0049-…md:17` ("no bitwise ops and no code-unit
accessor"), `docs/adr/retired/0062-…md:75` ("no bitwise operators, per ADR-0024's deferral").

### 4.2 R1/R2 — there is no integer type

`Value::Number(f64)` is the entire numeric surface. The complete set of installed `Number`
bindings, [`phalcom-core/src/universe/primitives.rs:109-127`](../../phalcom-core/src/universe/primitives.rs:109) — thirteen:

```
+  -  *  /  %  <  <=  >  >=  negated  hash  toString      Number.class::new/0, /1
```

And `core.ph:82` is literally `class Number {}` — empty. There is no `.ph`-side numeric surface
at all.

Effective integer width is **53 bits**, and the VM concedes it in a comment:

```rust
// phalcom-core/src/primitive/mod.rs:154-157
// Mask to 53 bits so the cast is lossless and the value round-trips as f64.
pub(crate) fn hash_code(bits: u64) -> Value {
    Value::Number((bits & 0x1F_FFFF_FFFF_FFFF) as f64)
}
```

A 64-bit group word cannot be represented. Neither can `h1` for a 64-bit hash.

### 4.3 R3 — `Bytes` is octet-at-a-time

`Bytes` landed 2026-07-20 and is the closest thing to raw memory in the language: eleven
primitives at [`universe/primitives.rs:312-322`](../../phalcom-core/src/universe/primitives.rs:312) —
`new/1`, `fromString_`, `size_`, `at_`, `set_`, `fill_`, `slice_`, `copyInto_`, `utf8_`,
`utf8Lossy_`, `equalsConstantTime_`. Backing is `Box<[u8]>`, fixed length
([`../spec/current/stdlib/bytes.md`](../spec/current/stdlib/bytes.md) §1).

**There is no multi-byte accessor**, and none can be synthesized. `*256 +` accumulation
overflows 2⁵³ at the seventh byte, and even below that you cannot mask or shift the result.
`Bytes` gives you the *storage* for a ctrl array and none of the *access* the algorithm needs.

`slice_` is always a copy, never a view — PDR-0011 ruling 5, to avoid retaining the parent under
a non-moving collector (Erlang's binary leak). So a group cannot be obtained as a borrowed
window either.

### 4.4 R5 — no fixed-size value array

`Bytes` is fixed-size but `u8`-only: it cannot hold `Value`s. `List` is `Vec<Value>`, growable,
and — **`[V]`** [`universe/primitives.rs:297`](../../phalcom-core/src/universe/primitives.rs:297) —
`List.new` is registered as `SignatureKind::Method(0)`. **There is no presized constructor.**
Reaching a 1024-slot table costs 1024 `push_` sends before the first insert.

There is also no uninitialized allocation anywhere (`bytes.md` law 4: "no constructor exposes
uninitialized memory"), and absence is `Option`/`None`, a heap singleton — so an empty slot is a
value, not a niche.

### 4.5 No escape hatch

**`[V]`** Zero `extern "C"` / `libloading` / `dlopen` anywhere in `phalcom-core`, `phalcom-ast`,
`phalcom-common`. No FFI.

**`[V]`** `@native` does not help, and it is worth being precise because the name invites the
opposite conclusion. Its expander is registered as a no-op (`compiler/attributes.rs:736`), and
the member is *deleted* before anything else sees it:

```rust
// phalcom-core/src/compiler/attributes.rs:1793
class.members.retain(|m| !member_has_attr(m, "native") && !member_has_attr(m, "ignore"));
```

Its own spec (`docs/spec/current/decorators/native.md`, "What it is not") says: "It is not a binding
directive. `@native` does not tell the compiler which Rust function to install, and does not
cause anything to be installed." It is an LSP anchor.

Adding a primitive means editing `universe/primitives.rs` and rebuilding the host — an ADR-0019
floor amendment, i.e. a governance action, not a language feature.

---

## 5. The wrapping trap — ratified work does not close R2

This is the finding most likely to be missed by anyone reading the roadmap and concluding "the
numeric tower unblocks this."

**`[V]`** [PDR-0012](../pdr/0012-numeric-tower-implementation-and-floor-amendment.md) is
**Accepted, ratified 2026-07-20, unimplemented** ([`../pdr/STATUS.md:30`](../pdr/STATUS.md)).
It rules the `Int`/`Float` split, floor 137 → 153.

**`[V]`** It contains **zero** occurrences of "bitwise", "shift", `bitAnd`, or `<<`. Verified by
grep over the record. Bitwise operations are not in it.

**`[V]`** Ruling A2 (`:63`): "`Int` is exact and unbounded: a tagged `i64` small path, a heap
`LargeInt` large path" — `Object::LargeInt(BigInt)`, with an invariant that a `LargeInt` value is
never in `i64` range (`:93-94`).

So even after PDR-0012 ships in full:

- R1 is still unmet — bitwise ops are ruled by nothing. They appear in
  `docs/spec/current/experimental/numeric-and-string-indexing.md` (status **Proposed**) and are
  sketched in `docs/spec/current/drafts/stdlib-catalog.md:109-113` as
  `and/or/xor/not/shl/shr/ushr/bitAt/bitCount/leadingZeros/trailingZeros`. A draft is not a record.
- **R2 is not merely unmet; it is in tension with what was ratified.** `Int` is *unbounded*.
  Unbounded integers have no wrapping semantics — there is no width for a borrow to propagate
  across. `cmp.wrapping_sub(repeat(0x01))` is not expressible over a BigInt-backed `Int` at any
  width, and `!full + (full >> 7)` needs the complementary guarantee that carry stops at the
  word boundary.

Supplying R2 therefore requires either a distinct fixed-width type (`U64`, `Word`, or similar) or
explicit width-carrying selectors (`wrappingSub64(_)`), and **that is a design decision, not a
missing primitive.** It touches the numeric tower's shape, which was just ratified. Anyone
proposing bitwise-on-`Int` should be made to answer this first.

Worth recording the general form: *ratified-and-unbuilt work is routinely read as "this gap is
already handled."* Here it is handled for indices and byte values and file offsets — the uses
`drafts/stdlib-catalog.md:98-104` actually enumerates — and specifically **not** for the one
thing this analysis needs.

---

## 6. What Phalcom's `Map` is today

**`[V]`** [`phalcom-core/src/heap/map.rs:53-62`](../../phalcom-core/src/heap/map.rs:53):

```rust
pub struct MapObject {
    entries: Vec<(Value, Value, i64)>,   // insertion-ordered; i64 = cached Phalcom hash bucket
    index: HashMap<i64, Vec<usize>>,     // bucket -> candidate slots
}
```

An **IndexMap-shaped design over separate chaining**, where the bucket math is delegated to Rust's
`std::collections::HashMap`. `Set` shares `MapObject` with the value slot left `Value::Nil`
(`:56-57`).

**No open addressing, no probe sequence, no control bytes, no SIMD** — in Phalcom's own map. The
SwissTable it sits on is std's, one level down, and the chained index discards its properties.

**`[V]`** Lookup, [`primitive/map.rs:54-66`](../../phalcom-core/src/primitive/map.rs:54):

```rust
fn locate(vm: &mut VM, id: ObjRef, key: Value) -> PhResult<(i64, Option<usize>)> {
    let bucket = send_hash(vm, key)?;
    let candidates: Vec<usize> = vm.heap.map(id).bucket(bucket).to_vec();
    for slot in candidates {
        ...
        if send_eq(vm, candidate_key, key)? { return Ok((bucket, Some(slot))); }
    }
    Ok((bucket, None))
}
```

**`[V]`** `send_hash` ([`primitive/mod.rs:359-369`](../../phalcom-core/src/primitive/mod.rs:359))
is a full re-entrant `vm.send_dynamic(value, "hash", &[])`. So every lookup re-enters the
interpreter. This is deliberate and correct — `Map` must key on *Phalcom* `hash`/`==`, not Rust's
`Value: Hash`, or a value-hashable `Tuple` key would misbehave (the rationale is in `heap/map.rs`'s
module doc, `:19-35`). It is also the dominant cost, and §7 is about that.

The `.ph` side is a thin wrapper over `size_`/`get_`/`has_`/`put_`/`keyAt_`/`valueAt_`. **None of
the hashing lives in `.ph`.**

Note also that `Bytes` is in the mutable-key rejection set (`primitive/mod.rs:400-408`) alongside
`List`/`Map`/`Set` — collection-protocol law 4. So the one buffer type cannot key the one map type.

---

## 7. The second answer: the optimization inverts

Grant R1–R5. Assume `Int`, bitwise, `loadU64_`, presized arrays all shipped. **A SwissTable would
still measure worse than what is there**, and the reason is structural.

### 7.1 Measured inputs

All **`[M]`**, from [`../forge/perf-log/SCOREBOARD.md`](../forge/perf-log/SCOREBOARD.md). Note the
file's own staleness warnings at `:29-56` — §1/§3a are stale at `5254586`; the rows below are §3bb
and §2, re-derived at `1d2baea` (2026-07-14), and the instrument is named per row.

| Quantity | Value | Where |
|---|---|---|
| bytecode instruction, hot loops | **~9.1–13.4 ns** | `SCOREBOARD.md:291-298` |
| send, full frame push (`bare_send`, criterion) | **~144 ns** | `:195` |
| primitive send, no frame push (`arith_send`) | **~113 ns** | `:196` |
| `method_call`, whole-process | **~218 ns/send** | `:255` |
| `map_numeric` | **~1.70 µs/op**, 32.4 ns/instr | `:258`, `:301` |
| `for` | **~576 ns/iteration**, 59 instr/iteration | `:256`, `:291` |

**`[M]`** `map_numeric` is the worst per-instruction row in the entire suite — 32.4 ns/instr
against `method_call`'s 9.1, a 3.6× spread (`:303`). The re-entrant `hash`/`==` sends per lookup
are why.

### 7.2 The derivation

**Derived from `[M]` above, not measured.** Stated as arithmetic so it can be checked or refuted.

One group probe in `.ph`, counting sends:

| Piece | Sends | At ~144 ns |
|---|---|---|
| `Group::load` **with** a hypothetical native `loadU64_` | 1 | 144 ns |
| `Group::load` **without** one (8× `at_`, 8× shift, 8× or) | ~24 | ~3.5 µs |
| `match_tag` (`^`, `wrappingSub`, `~`, `&`, `&`) | ~6 | ~860 ns |
| `trailingZeros` (de Bruijn: multiply, shift, table index) | ~4 | ~580 ns |
| **total, with native load** | **~11** | **~1.6 µs** |
| **total, without** | **~34** | **~4.9 µs** |

Now the thing being replaced. A linear `==` scan of 8 candidate slots — the operation SWAR
exists to avoid — is 8 × (`at_` + `==`) ≈ 16 sends ≈ **~2.3 µs**.

**So the SWAR group scan, given a native multi-byte load, buys roughly 2.3 µs → 1.6 µs. Without
that primitive it is 2× worse than the naive scan.** Either way it is inside the noise of
`map_numeric`'s existing ~1.70 µs/op, and it spends the entire operation budget on the metadata
scan.

### 7.3 Why this is not a tuning problem

hashbrown's premise is that **the probe is the cost**. SWAR/SIMD scans 8–16 slots per instruction
so the metadata scan hides inside the cache-miss shadow of the eventual key compare. The whole
design is a bet that a few nanoseconds of scan matter.

Phalcom's per-probe floor is a **message send** — ~144 ns, sixteen times the cost of a whole
bytecode instruction, and three orders of magnitude above the machine ops SWAR replaces. A
technique whose payoff is measured in single-digit nanoseconds cannot be expressed in a currency
whose smallest denomination is 144 of them.

This generalizes past hashtables, and it is the transferable content of this document:

> **A constant-factor optimization ports only if the target's minimum unit of work is smaller
> than the constant being optimized.**

Which is the same shape as a finding already recorded in
[`../design-notes/bytecode-representation-and-borrowed-techniques.md`](../design-notes/bytecode-representation-and-borrowed-techniques.md)
— a borrowed technique's *precondition* has to be checked before its *benefit* is estimated. There,
operand-free superinstructions were dropped because `Vec<Bytecode>` is not a bytestream. Here,
SwissTable fails because a send is not an instruction. Same class of error, different substrate.

---

## 8. What is writable in `.ph` today

For completeness, since the answer is not "nothing."

**A chaining hash table over `List`, bucketed with `%`, keyed on user `hash`/`==`.** The
polymorphism it needs is real and present: `Object#hash` is a floor getter
(`universe/primitives.rs:50` → `primitive/object.rs:74`, identity digest of the heap handle),
`Number#hash` overrides it value-wise (`:121` → `number.rs:60-73`), and any `.ph` class may
override `hash` and `==(_)` as ordinary methods — `core.ph:828` (`List#==`), `:953` (`Map#==`),
`:850` (`List#!=`, routing through `==`), with the universal `Object#!=` on the floor at
`universe/primitives.rs:54`. The invariant `a == b ⇒ a.hash == b.hash` is stated as R-INV-1.3.

It would be correct and slow: every bucket step is a send, and construction costs one `push_` per
slot for want of a presized `List`. Useful as a teaching artifact — it makes the hash/eq protocol
visible — not as a collection.

**`[V]`** And the tree already ruled the adjacent question.
[`../spec/current/stdlib/map-and-set.md`](../spec/current/stdlib/map-and-set.md) §4 evaluated exactly this
and rejected it: "`.ph` over `List` of buckets — rejected: O(n) lookup defeats the point; hashing
in `.ph` is awkward," recommending the native arm as "matches the 'native container, `.ph`
protocol' pattern."

That ruling generalizes to this whole analysis. **The hashtable is not a `.ph`-writable object in
this design, and that is a position, not a gap.** ADR-0020's kernel pattern — native storage,
`.ph` protocol — is the answer Phalcom already gave.

---

## 9. If someone wants it anyway

Dependency-ordered. Stated so the size is visible, not to endorse it.

1. **Implement PDR-0012.** Ratified 2026-07-20, unimplemented. Floor 137 → 153 (163 composed with
   PDR-0011). *Buys nothing on its own for this purpose* — see §5.
2. **Rule and implement fixed-width bitwise operations.** No record exists. Must resolve §5's
   tension between unbounded `Int` and wrapping semantics. Needs lexer + parser + AST + compiler
   work, since the tokens do not exist. **This is the expensive step and the one with an open
   design question in it.**
3. **Admit multi-byte `Bytes` load/store primitives.** Not among PDR-0011's ten (nor PDR-0013's
   eleventh). Floor amendment. Without it, §7.2's second row applies and the exercise is pointless.
4. **A presized/fixed `Value` array.** Exists in no spec found. Either `List.new(n)` or a new
   kernel arm.

Steps 2–4 are **three unwritten decision records**, one of which (2) carries an unresolved design
question that reaches back into just-ratified work. Step 1 alone buys nothing here.

**`[O]`** Whether R2 is better served by a distinct fixed-width type or by width-carrying
selectors on `Int` is genuinely open and is not answered by this document.

> **Filed.** Steps 2–4, plus §11's `map_numeric` hypothesis, are now recorded as unowned items in
> [`../deferred/hashbrown-analysis-followups.md`](../deferred/hashbrown-analysis-followups.md).
> That file restates the standing caution: none of them should be picked up *because hashbrown
> needs them* — this analysis concludes a Phalcom SwissTable would measure worse than the current
> native `Map` even with all four in hand. Each stands or falls on unrelated demand (binary
> codecs, hex/base64, hash-table dispatch cost).

---

## 10. Provenance ledger

Per R2 of the citation discipline: the verb needs its object.

**Opened first-hand in this session (main thread):**
`hashbrown/src/control/group/generic.rs` (in full), `control/bitmask.rs` (grep),
`control/tag.rs` (grep), `src/raw.rs:55-94` + targeted greps, `src/table.rs` (grep),
`phalcom-core/src/heap/map.rs` (grep), `phalcom-core/src/primitive/map.rs:50-70`,
`phalcom-core/src/primitive/mod.rs:150-160`, `:356-370`,
`phalcom-core/src/universe/primitives.rs:106-127`, `:297`, `:305-325`,
`phalcom-core/src/compiler/attributes.rs:1790-1796`, `phalcom-core/core/core.ph:107`,
`docs/spec/current/core/{bytes,map-and-set,numeric-tower}.md`,
`docs/spec/current/drafts/stdlib-catalog.md:95-135`, `docs/spec/current/memory-management.md:1-60`,
`docs/pdr/STATUS.md:30`, `docs/pdr/0012-…md` (grep),
`docs/forge/perf-log/SCOREBOARD.md:253-303`.

**Delegated to subagents, then spot-checked against the tree before use:** the `phalcom-ast`
bitwise-token sweep (re-run in the main thread, 0 hits), `attributes.rs:1793` (re-read),
`primitive/mod.rs:154-157` (re-read), `core.ph:107` (re-read), `List.new` arity
(`primitives.rs:297`, re-read), the `Bytes` primitive list (`primitives.rs:312-322`, re-read),
the SCOREBOARD rows (re-read at `:255-258`, `:291-303`), PDR-0012's bitwise absence and A2
(re-grepped). The `raw.rs` line citations in §1.5, §1.6 and §2 beyond the ranges above are
**subagent-reported and not independently re-opened** — they are the least-load-bearing claims
here (they describe hashbrown's internals, which §2 concludes are not portable anyway), but they
are tagged honestly: treat §1.5, §1.6 and the §2 table as **`[R]`-via-delegation** rather than
`[V]`, and re-open `raw.rs` before quoting any of them into a decision record.

**`[X]` Two delegated citations were wrong and were corrected before publication.** Recorded per
R6 rather than silently fixed, because the failure mode is the point:

- `core.ph:849` was reported as "the `!=`-routes-through-`==` rule." **Refuted** by `sed -n '849p'`
  — line 849 is a comment fragment ("decoupling hazard"). The rule is at `:850`, and the universal
  binding is at `universe/primitives.rs:54`. Off-by-one, and the surrounding claim was true, which
  is exactly what makes this class of error survive review.
- `vm/dispatch.rs:398-427` was carried in from a memory note as the inline-cache probe site.
  **Refuted** by grep — the probe is at `:445`, the refill at `:461`. The note was written against
  an older tree and the line numbers moved.

Both were `[R]` claims wearing `[V]` clothing. The negative control that caught them (R4) was
cheap: open the cited line and check that it says what the citation says it says. It was not run
by the process that produced them.

**Not verified:** nothing in this document rests on an external source, a paper, or a benchmark
run in this session. No new measurement was taken; every number in §7.1 is quoted from
SCOREBOARD.md with its staleness warning attached, and every number in §7.2 is arithmetic over
those, labelled as derived.

---

## 11. Summary

- **`[V]`** The SwissTable *algorithm* is not expressible in Phalcom today. Four independent
  blockers: no bitwise operators (unlexable), no integer type (53-bit f64), no multi-byte load,
  no fixed-size value array.
- **`[V]`** Roughly half of hashbrown's complexity is raw-memory engineering that a handle-arena
  GC'd heap makes unnecessary. The port would be *smaller* than the original, and R6/R7 —
  unwind-safe cleanup and caller-supplied hash/eq — are **already satisfied** by `ensure` and by
  blocks. Generics are not the blocker.
- **`[V]`** PDR-0012, though ratified, does not close the gap: it contains no bitwise rulings, and
  its unbounded `Int` is in tension with the wrapping semantics `match_tag` requires. That tension
  is an open design question (§5).
- **Derived from `[M]`** Even fully unblocked, the SWAR group scan costs ~1.6 µs of sends to
  replace a ~2.3 µs linear scan — inside the noise of `map_numeric`'s measured ~1.70 µs/op. The
  optimization inverts, because a send (~144 ns) is three orders of magnitude above the machine
  ops SWAR eliminates.
- **`[V]`** `map-and-set.md` §4 already ruled `.ph`-authored hash tables out on the same grounds.
  Native storage + `.ph` protocol (ADR-0020) is the standing answer.
- **`[O]`** The actionable lever on `map_numeric` is not table layout. It is the re-entrant
  `send_dynamic("hash")` per lookup in `locate` (`primitive/map.rs:55`). Caching `hash` method
  resolution per key-class — the `InlineCache` machinery already exists at `chunk.rs:10-18` and is
  probed and refilled on the send path at `vm/dispatch.rs:445` / `:461` — targets the term that
  actually dominates.
  **This is a hypothesis, not a plan: unmeasured, unowned, and not costed.**

# Deferred: follow-ups from the hashbrown feasibility analysis (unowned)

Split out of [`docs/analyses/hashbrown-in-phalcom.md`](../analyses/hashbrown-in-phalcom.md)
(2026-07-20). That document asks whether a SwissTable hash map could be written in Phalcom and
answers no, twice. Its §9 and §11 leave four things genuinely open. They are collected here
because an analysis **never authorizes implementation** — the next artifact for items 1–3 is a
PDR, not a branch.

**None of these blocks anything, and none should be picked up because "hashbrown needs it."** The
analysis concludes a Phalcom SwissTable would measure *worse* than the current native `Map` even
if all four landed. They are recorded because each was surfaced by that work and each stands on
its own merits for unrelated reasons — bit manipulation, binary codecs, and hash-table dispatch
cost are all real needs with or without SwissTable.

All references verified 2026-07-20 against `main` (`e33e8e5`).

---

## 1. Bitwise operations on `Int` have no decision record — and unbounded `Int` fights them

**Status:** open design question, not a missing primitive. **This is the item with actual
substance in it.**

Bitwise operators do not exist in the language at any level. A grep across `phalcom-ast/src/` for
`BitAnd|BitOr|BitXor|Shl|Shr|Tilde|Caret|Ampersand|Pipe` returns **zero hits** — there is no
token, no AST node, no parse rule. The kernel works around the absence in its own source:

```
phalcom-core/core/core.ph — grep "no bitwise ops"
  // or mid-sequence. UTF-8 decode via division/modulo (no bitwise ops).
```

They appear in `docs/spec/v0.2/drafts/stdlib-catalog.md` §0.2 as
`and/or/xor/not/shl/shr/ushr/bitAt/bitCount/leadingZeros/trailingZeros`, and in
`docs/spec/v0.2/experimental/numeric-and-string-indexing.md` at status **Proposed**. Neither is a
record. `stdlib-catalog.md` §0.2 lists the real demand: open-flags, permission modes, `Bytes`
fixed-width codecs, hex/base64, "every hash function", socket options.

**The trap.** [PDR-0012](../decisions/0012-numeric-tower-implementation-and-floor-amendment.md)
(Accepted 2026-07-20, unimplemented) is easy to read as unblocking this. It does not:

- It contains **zero** occurrences of "bitwise", "shift", `bitAnd`, or `<<`. Verified by grep over
  the record.
- Its ruling A2 makes `Int` **exact and unbounded** — a tagged `i64` small path plus a heap
  `LargeInt(BigInt)`. **Unbounded integers have no wrapping semantics**, because there is no width
  for a borrow to propagate across.

That second point is the design question. Word-parallel bit tricks need *both* halves of
fixed-width overflow behavior, and they need them adjacently. hashbrown's SWAR fallback is the
clean demonstration — two functions, twenty lines apart:

```rust
// match_tag: correct BECAUSE borrow propagates across byte lanes
cmp.wrapping_sub(repeat(0x01)) & !cmp & repeat(0x80)

// convert_special_to_empty_and_full_to_deleted: correct BECAUSE carry does NOT propagate
!full + (full >> 7)
```

Supplying this therefore requires either a **distinct fixed-width type** (`U64`, `Word`) or
**width-carrying selectors** (`wrappingSub64(_)`) — a choice that reaches back into a numeric
tower ratified the same week. **Whoever proposes bitwise-on-`Int` must answer this first**, and a
proposal that treats it as "add ten primitives to `Int`" has not.

**Also unpriced:** the lexer/parser/AST/compiler work. Every other item in the numeric tower is a
runtime change; this one adds surface syntax.

## 2. `Bytes` has no multi-byte load/store

`Bytes` is octet-at-a-time. Its eleven primitives (`phalcom-core/src/universe/primitives.rs`, grep
`bytes_cls`) are `new/1`, `fromString_`, `size_`, `at_`, `set_`, `fill_`, `slice_`, `copyInto_`,
`utf8_`, `utf8Lossy_`, `equalsConstantTime_`. There is no `loadU32_`/`loadU64_`/`storeU32_`, and
none is among [PDR-0011](../decisions/0011-admit-bytes-native-octet-buffer.md)'s ten or
PDR-0013's eleventh.

It cannot be synthesized in `.ph`: `*256 +` accumulation exceeds 2⁵³ at the seventh byte, and
without item 1 there is no way to mask or shift the result anyway. `slice_` is always a copy,
never a view (PDR-0011 ruling 5, to avoid retaining the parent under a non-moving collector), so
a borrowed window is not an alternative.

**Blocked on item 1** for the same reason: a `loadU64_` returning a 53-bit-truncated `Number` is
not useful. Real demand is binary codecs and the `stream-protocol.md` work, not SwissTable.
Would be an ADR-0019 floor amendment.

## 3. There is no fixed-size array of values

`Bytes` is fixed-size but `u8`-only — it cannot hold `Value`s. `List` is `Vec<Value>`, growable,
and has **no presized constructor**: `List.new` is registered as `SignatureKind::Method(0)`
(`phalcom-core/src/universe/primitives.rs`, grep `list_class_new`). Reaching a 1024-slot table
costs 1024 `push_` sends before the first useful write.

There is also no uninitialized allocation anywhere — `bytes.md` law 4 states "no constructor
exposes uninitialized memory" — and absence is `Option`/`None`, a heap singleton, so an empty slot
is a value rather than a niche.

Not specced anywhere found. Either `List.new(n)` or a new kernel arm. **Lower confidence than
items 1–2 that this is wanted at all** — presizing is a performance affordance, and this repo's
standing rule is that `SCOREBOARD.md` is the only source for a perf claim. Measure the `push_`
cost on a real workload before designing it.

## 4. `Map` lookup re-enters the VM per probe — the actual `map_numeric` lever

**Status:** hypothesis. Unmeasured, unowned, not costed. **Do not fold into a correctness unit**
(same rule as [`class-sealing-followups.md`](class-sealing-followups.md) items 1–2: a dispatch
change inside a correctness gate makes a red tree ambiguous).

`map_numeric` is the worst row in the suite on both axes — **~1.70 µs/op** and **32.4 ns/instr**
against `method_call`'s 9.1, a 3.6× spread (`docs/forge/perf-log/SCOREBOARD.md` §2/§3bb, measured
at `1d2baea`; note that file's own staleness warnings before quoting anything else from it).

The reason is visible in one line. `locate` (`phalcom-core/src/primitive/map.rs`, grep
`fn locate`) opens with `send_hash(vm, key)?`, and `send_hash`
(`phalcom-core/src/primitive/mod.rs`, grep `fn send_hash`) is a full re-entrant
`vm.send_dynamic(value, "hash", &[])`. **Every map lookup re-enters the interpreter**, then does
it again per `==` candidate.

This is correct and deliberate — `Map` must key on *Phalcom* `hash`/`==`, not Rust's
`Value: Hash`, or a value-hashable `Tuple` key would misbehave (the rationale is in
`phalcom-core/src/heap/map.rs`'s module doc). The cost is the point, not the correctness.

**Direction (not a ruling):** the send is monomorphic per key class in almost every real map, and
the machinery to exploit that already exists — `InlineCache { class, method, version }`
(`phalcom-core/src/chunk.rs`, grep `pub struct InlineCache`), probed and refilled on the send path
in `invoke_at` (`phalcom-core/src/vm/dispatch.rs`, grep `fn invoke_at`). Caching `hash` method
resolution per key-class targets the term that dominates.

**Two hazards before anyone tries it.** (a) The existing cache is keyed by *call site* (`cache_ip`
into a `Chunk`); a native primitive has no `cache_ip`, so this needs a different cache identity —
that is design work, not a port. (b) Invalidation is a **global** `VM::world_version` counter, not
a per-class epoch (`ClassObject` has no epoch field), so any method install anywhere invalidates
every entry. That is U-IC's still-unbuilt fourth piece; this item probably wants it first.

Do not start by writing the cache. Start by measuring how much of `map_numeric`'s 1.70 µs is
actually the two sends — the analysis asserts it is dominant but never instrumented it.

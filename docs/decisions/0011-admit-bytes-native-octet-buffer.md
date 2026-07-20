# PDR-0011 — Admit `Bytes`: a native octet buffer arm, ten floor primitives, and the container bulk-op posture

- Status: **Accepted** (ratified 2026-07-20, same day as proposed)
- Date: 2026-07-20
- Related: [ADR-0019](../adr/accepted/0019-freeze-vm-blessed-primitive-floor.md) (the floor
  freeze **this record amends** — `docs/adr/` is frozen, so the amendment is carried here, the
  way ADR-0039/ADR-0049 carried theirs when that folder was live),
  [ADR-0020](../adr/accepted/0020-kernel-list-native-array-protocol.md) (the governing
  precedent: native storage arm, `.ph` protocol above),
  [ADR-0048](../adr/accepted/0048-amend-iteration-bare-cursor-sentinel-and-iterable-root.md)
  (`Iterable` root, bare-cursor iteration),
  [ADR-0049](../adr/accepted/0049-amend-floor-admit-string-byte-and-raw-write-primitives.md)
  (String byte floor — the asymmetry in ruling 3 below),
  [ADR-0050](../adr/accepted/0050-non-moving-mark-sweep-collector.md) (non-moving GC —
  **made security-relevant by ruling 7**),
  [ADR-0024](../adr/accepted/0024-numeric-surface-split-int-float-and-division.md)
  (Accepted, **verified unbuilt 2026-07-20** — bears on ruling 2),
  [PDR-0005](0005-resources-are-disposable-handles-not-finalized.md) §7a /
  [`stream-protocol.md`](../spec/v0.2/core/stream-protocol.md) §9 (the consumer that makes
  this a blocker: every stream selector takes or fills a `Bytes`)
- Spec: [`docs/spec/v0.2/core/bytes.md`](../spec/v0.2/core/bytes.md) holds the protocol,
  laws, and conformance harness. Exploration and precedent survey:
  [`docs/spec/v0.2/drafts/bytes.md`](../spec/v0.2/drafts/bytes.md).

## Context

`stream-protocol.md` is normative and every selector in it (`read(_)`, `write(_)`, the
`BytesReader`/`BytesWriter` reference implementations) takes or fills a `Bytes` that does not
exist. The draft measured the alternative — `Bytes` over a `List` of `Number`s — at **16 bytes
of buffer per byte of payload** (`Value` is 16 B; draft §2), before dispatch cost. That is not
a viable octet type; the only real design is the ADR-0020 pattern.

## Rulings

1. **A new heap arm, `Object::Bytes(BytesObject)`, backed by `Box<[u8]>`** — contents mutable,
   **length fixed at construction** (`Tuple`'s backing shape, not `List`'s). Unboxed: 24 B is
   under the 40 B `Object` slot (`heap/object.rs:24`, boxing note at `:29-33`), so the arm
   widens nothing. Fixed length is load-bearing, not ergonomic: a growable `Vec<u8>` reallocs,
   and a realloc strands a copy of any secret in the arena beyond the reach of `zeroize`
   (draft §7 finding 1).
2. **The element type is `Number`, an integer in 0–255. No `Byte` value arm.** Verified at
   ruling time: ADR-0024 is still paper-only — no `Int`/`LargeInt` heap arm exists and
   `class Number {}` is flat (`core.ph:82`), so `Number` is IEEE f64, which represents every
   integer in 0–255 **exactly**. The contract is stated representation-independently
   ("an integer in 0–255"), so if ADR-0024 lands later, these values become small `Int`s with
   no surface change. A `Byte` value type is refused for ADR-0010's reason: even `Fiber` and
   `Family` declined a `Value` arm; a range check that `at_`/`set_` already enforce does not
   buy a new dispatch axis.
3. **Ten floor primitives; the audited floor goes 137 → 147 — and an amended admission
   posture for kernel container arms.** ADR-0019's inexpressibility-only rule ("speed never
   sufficient") is amended, for native container arms, with a bright functionality line:

   - **Bulk operations with no user code inside the loop admit natively.** The arm exists to
     eliminate per-element representation and dispatch cost; a `.ph` per-byte loop over
     `at_`/`set_` reintroduces exactly the cost the arm was admitted to remove (two sends
     per byte, each a method-table probe; no inline cache exists). Refusing native memset/
     memmove on a buffer type while admitting the buffer is a false economy.
   - **Any selector that runs a user block per element stays `.ph`, unconditionally.** Not
     economy — functionality: `Fiber#yield` is legal only at `native_reentry_depth == 0`
     (the restricted-yield guard, `vm/dispatch.rs:259`, ADR-0030 §4). A native `each` puts a
     native frame between caller and block, turning any `yield` inside the block into a
     runtime error — Lua's "attempt to yield across a C-call boundary", the C-extension wall
     Python hit, the lesson coroutine languages converged on the hard way. `.ph` combinators
     also keep literal-block call sites visible to the sacred inliner
     (`compiler/inliner.rs:128`).

   Admitted: `Bytes.new(_)`, `Bytes.fromString_(_)` (statics), `size_`, `at_(_)`, `set_(_,_)`
   (raw access mirroring `list_raw_*` — bare-value-or-`None` reads, type-error writes,
   `primitive/list.rs:72-103`), `fill_(_)` (memset; `fill_(0)` **is** `zeroize`, dissolving
   draft B-6), `slice_(_,_)` (copying extraction), `copyInto_(_,_)` (memmove into another
   `Bytes` — the primitive that lets `.ph` `concat` and stream buffering run with zero
   per-byte loops), `utf8_` (fallible decode — no existing primitive builds a `String` from
   arbitrary octets), `equalsConstantTime_(_)` (a timing property `.ph` cannot express).
   **Stays `.ph`:** `each`/`map`/`filter`/`reduce` (block-taking — the functionality line),
   `concat` (`new` + `copyInto_` ×2, three native sends), `fromList` (cold-path construction
   needing per-element Phalcom checks), `==` (short-circuiting is *correct* there; §8's
   selector exists precisely because `==` must not be constant-time's spelling).
4. **One class.** No `Bytes`/`BytesMut` split; Rust's version duplicates every API and forces
   `freeze()` at each boundary, and Phalcom has no static types to make the split pay. `Bytes`
   sits in `List`'s corner of collection-protocol laws 3/4: **structural `==`, identity
   `hash`, not a valid `Map`/`Set` key.** A digest that must key a `Map` converts via
   `toTuple`.
5. **`slice` copies.** No O(1) sub-buffer views: under ADR-0050's non-moving mark-sweep a
   shared slice retains its whole parent — Erlang's notorious binary leak, with
   `binary:copy/1` as that ecosystem's standing production fix. Copy is O(n) and honest.
6. **`equalsConstantTime` on length mismatch returns `false`**, in time constant with respect
   to buffer **contents**; lengths are not concealed and the spec says so. This is Go
   `crypto/subtle.ConstantTimeCompare`'s shape. Node's `timingSafeEqual` **throws** on
   mismatch, which leaks length through the exception path and makes every HMAC comparison
   need a pre-check — rejected.
7. **`zeroize` is the `.ph` name for `fill_(0)` — one native memset — and its guarantee is a
   documented obligation, not a mechanism.** Complete because of ruling 1's fixed length (no
   realloc ever stranded a copy).
   Posture is ADR-0052's: written contract + golden test, no static analysis. Consequence
   named now, before it can be forgotten: **this contract makes ADR-0050's non-moving choice
   security-relevant.** A moving collector copies live objects and scatters stale secret
   images no `zeroize` can reach; any future record reopening the moving-GC door must
   address this coupling explicitly. Interpreter-level residue (value stack, fiber stacks,
   swap) stays outside the contract and the spec says so, unhedged — the .NET `SecureString`
   deprecation is what shipping a stronger claim costs.

## Composition with PDR-0012

[PDR-0012](0012-numeric-tower-implementation-and-floor-amendment.md) (numeric tower, Proposed
the same day) amends ADR-0019 off the **same measured 137** over disjoint classes. Whichever
record ratifies second rebases its floor arithmetic (PDR-0012 ruling 21; both ⇒ **163**).
Ruling 2 here is already worded for that landing: elements are "integers in 0–255"
representation-independently, so under the tower they become small `Int`s with no surface
change — and the §9.1-draft constraint stands: secret material never routes through the
auto-promoting `Int`.

## Open questions

| # | Question | Notes |
|---|---|---|
| Q-1 | Literal syntax (hex blob?) | Lexer question, no owner. Draft B-5. Not blocking: `Bytes.fromList`/`fromString` cover construction |

## Consequences

- `stream-protocol.md` §9's hard dependency is dischargeable; `BytesReader`/`BytesWriter` and
  the filesystem spec can proceed once this is Accepted.
- The floor freeze gains a +10 amendment; `floor_census_matches_installed_bindings`
  (`phalcom-core/tests/invariants.rs:605`) must gain a `NEW_BYTES: usize = 10` constant and a
  `Bytes` class row when the arm ships.
- Ruling 3's posture is scoped to **kernel container arms** and is two-sided: it admits bulk
  no-user-code operations *and* it hard-forbids nativizing block-taking selectors anywhere in
  the kernel. The second half is checkable: the conformance harness requires `Fiber.yield`
  inside an `each` block to work (spec §9), which a native combinator cannot pass.
- `bytes.toString` is **not** the decoder: decode is fallible (`utf8` → `Option`), and
  `Object#toString` stays total. A `String` can never hold arbitrary octets; every decode
  site pays the `Option`, permanently (draft §5).

# PDR-0011 — Admit `Bytes`: a native octet buffer arm and six floor primitives

- Status: **Proposed**
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
3. **Six floor primitives; the audited floor goes 137 → 143.** Admitted under ADR-0019's rule
   (capability inexpressible in `.ph`; speed never sufficient): `Bytes.new(_)` (allocates a
   native arm), `size_`, `at_(_)`, `set_(_,_)` (raw buffer access, mirroring `list_raw_*`),
   `utf8_` (fallible UTF-8 decode — no existing primitive builds a `String` from arbitrary
   octets), `equalsConstantTime_(_)` (a timing property `.ph` cannot express — a `.ph` loop
   short-circuits). **Rejected as derivable:** `slice_`, `concat_`, `fromString_` (ADR-0049
   already gave `.ph` `byteCount_`/`byteAt_(_)`), and `zeroize` (a `set_` loop — and *complete*
   as one, because ruling 1's fixed length means no realloc ever stranded a copy). Note the
   deliberate asymmetry: `slice_` is irreducible on immutable `String` (ADR-0049's Wren-cited
   case) and reducible on mutable `Bytes`; mutability is the reason.
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
7. **`zeroize` is `.ph`, and its guarantee is a documented obligation, not a mechanism.**
   Posture is ADR-0052's: written contract + golden test, no static analysis. Consequence
   named now, before it can be forgotten: **this contract makes ADR-0050's non-moving choice
   security-relevant.** A moving collector copies live objects and scatters stale secret
   images no `zeroize` can reach; any future record reopening the moving-GC door must
   address this coupling explicitly. Interpreter-level residue (value stack, fiber stacks,
   swap) stays outside the contract and the spec says so, unhedged — the .NET `SecureString`
   deprecation is what shipping a stronger claim costs.

## Open questions

| # | Question | Notes |
|---|---|---|
| Q-1 | Literal syntax (hex blob?) | Lexer question, no owner. Draft B-5. Not blocking: `Bytes.fromList`/`fromString` cover construction |

## Consequences

- `stream-protocol.md` §9's hard dependency is dischargeable; `BytesReader`/`BytesWriter` and
  the filesystem spec can proceed once this is Accepted.
- The floor freeze gains a +6 amendment; `floor_census_matches_installed_bindings`
  (`phalcom-core/tests/invariants.rs:605`) must gain a `NEW_BYTES: usize = 6` constant and a
  `Bytes` class row when the arm ships.
- `bytes.toString` is **not** the decoder: decode is fallible (`utf8` → `Option`), and
  `Object#toString` stays total. A `String` can never hold arbitrary octets; every decode
  site pays the `Option`, permanently (draft §5).

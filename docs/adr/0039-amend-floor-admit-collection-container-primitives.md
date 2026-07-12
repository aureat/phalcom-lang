# 39. Amend the frozen floor — admit collection-container primitives (`Map`/`Set`/`Tuple`/`Range`)

- Status: Proposed
- Date: 2026-07-12
- Related: [ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) (the frozen floor);
  [ADR-0032](0032-collections-representation-and-literals.md) (native-arm representation);
  [ADR-0020](0020-kernel-list-native-array-protocol.md) (the `List` native-array precedent);
  [ADR-0023](0023-amend-floor-admit-hash-and-kernel-reflection.md) (amendment precedent + `hash`);
  [`collection-protocol.md`](../spec/v0.2/core/collection-protocol.md);
  [`docs/forge/units/U-COLLTYPES/plan.md`](../forge/units/U-COLLTYPES/plan.md) (DEC-CT-A)

## Context

[ADR-0032](0032-collections-representation-and-literals.md) ratified **native heap-arm**
representation for `Map`/`Set`/`Tuple`/`Range` (`Object::Map`/`Set`/`Tuple`/`Range`,
mirroring `List`'s `ListObject`) but deliberately scoped itself to representation +
literals + the hashing contract, leaving the floor to a dedicated amendment (its
Consequences: "each native arm's own raw primitives, admitted per-unit under the
ADR-0019 amendment convention"). Each arm needs **raw storage primitives below the
`.ph` boundary** — a `.ph` body cannot allocate or index a native hash table, a fixed
slice, or read raw bound fields — exactly as `List` needed five raw primitives
([ADR-0020](0020-kernel-list-native-array-protocol.md)). `U-COLLTYPES/plan.md` DEC-CT-A
enumerates the set; this ADR ratifies it.

## Decision

**Amend [ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) to admit the raw
container primitives** (combinators stay `.ph`). Per class:

| Class | Raw primitives | Count |
|---|---|---|
| `Map` | `new` `rawSize` `rawGet` `rawPut` `rawHas` `rawRemove` `rawKeyAt` `rawValueAt` | 8 |
| `Set` | `new` `rawSize` `rawAdd` `rawHas` `rawRemove` `rawAt` | 6 |
| `Tuple` | `fromList` `rawSize` `rawAt` | 3 |
| `Range` | `new` `rawStart` `rawEnd` `rawInclusive` | 4 |

Total **+21**, installed **per-phase** as each class lands (`floor-census.md` count
**80 → 101**; Map+Set +14, Tuple +3, Range +4). Everything above these — `at`/`each`/
`map`/`filter`/`reduce`/`==`/value-`hash` fold/`toList`/`keys`/`values`/`includes`/
`first`/`last` and the `iterate(_)`/`iteratorValue(_)` protocol — is **`.ph`** over the
raw primitives + the U-CORE-5 contract.

## Consequences

- **Unblocks U-COLLTYPES.** With the floor ratified, the four classes' public protocol
  is pure `.ph` certified against the U-CORE-5 conformance harness.
- **Largest single amendment** (cf. `List`'s 5, `hash`'s 5) — justified: four distinct
  native containers, each with irreducible raw storage access.
- **Key-hashing re-enters dispatch (load-bearing).** `Map`/`Set` must key by **Phalcom**
  `hash`+`==` (not Rust's identity `Value: Hash`, which would wrongly identity-key
  value-hashable `Tuple`s), so `rawGet`/`rawPut`/`rawHas` re-enter VM dispatch on keys —
  the primitive must **re-resolve the `ObjRef` after each such send** (arena borrow
  discipline, per `list_raw_at`), and is subject to the
  [ADR-0030](0030-fibers-and-futures-cooperative-concurrency.md) restricted-yield model
  (a key whose `hash`/`==` yields raises `CannotYieldAcrossNativeFrame`).
- **Hashing contract preserved.** Mutable `Map`/`Set` inherit identity `Object#hash`
  (not valid keys); immutable `Tuple`/`Range` value-hash; key hashing digests the
  *mathematical value* so the future `Int`/`Float` split stays consistent (forward-compat §4).
- `floor-census.md` must be updated per phase (R-INV-0.1); this is a hard Phase-0 gate
  for U-COLLTYPES.

## Alternatives considered

- **`.ph`-over-`List`.** Rejected by [ADR-0032](0032-collections-representation-and-literals.md)
  — O(n) `Map`/`Set` lookup defeats the purpose; hashing in `.ph` is awkward.
- **Reuse `List`'s raw ops for all four.** Rejected — hash-table, fixed-slice, and
  bound-field storage are genuinely different shapes; forcing them through the array
  primitives would reintroduce O(n) or misrepresent the container.
- **One generic container primitive.** Rejected — `Map`/`Set`/`Tuple`/`Range` have
  distinct storage and mutability; a single primitive would leak all four shapes.
- **Defer `Range`'s raw fields (compute lazily in `.ph`).** Rejected — `Range` holds no
  element store, but `start`/`end`/`inclusive` are raw reads a `.ph` body cannot
  fabricate from nothing; 4 trivial accessors are cheaper than a workaround.

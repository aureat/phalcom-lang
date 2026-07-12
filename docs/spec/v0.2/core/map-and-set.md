# Specification — `Map` and `Set` (hash collections)

> **Status:** **Proposal.** Absent classes (names reserved in `ClassName`), now
> **unblocked** — their precondition `Object#hash` landed with
> [U-CORE-1](../../../forge/units/U-CORE-1/as-built.md)
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
| `keys` / `values` | a `List` of keys / values, in iteration order |
| `each(_)` | apply a 2-arg block `{ k, v => … }` per entry |

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
| **Native heap arm** (`Object::Map`/`Object::Set` over a Rust `HashMap`/`HashSet` keyed by the value's `hash`+`==`) | mirrors `List`'s `ListObject` (ADR-0020) | **Recommended** — O(1) ops, matches the "native container, `.ph` protocol" pattern; needs a small floor for `rawGet`/`rawPut`/`rawHas`/iteration |
| **`.ph` over `List`** of buckets | pure `.ph`, zero floor | rejected — O(n) lookup defeats the point; hashing in `.ph` is awkward |

The native arm implies a small **ADR-0019 amendment** (the raw hash-table
primitives) — scoped and justified when the unit lands, analogous to `List`'s five
raw primitives. Combinators (`map`/`filter`/…) stay `.ph`.

## 5. Non-goals

- **Literal syntax** `{ a: 1 }` (Map) and `Set(1, 2)` — deferred to a literal-syntax
  proposal (sibling of [`list-literal-syntax.md`](./list-literal-syntax.md)).
- **Ordering guarantees** beyond "stable within a run" — no sorted variant here.

## 6. Test strategy

Instantiate the [collection-protocol](./collection-protocol.md) conformance harness
for each; plus: hash-collision correctness, key-overwrite, `remove` idempotence,
`None` on missing key, structural `==`.

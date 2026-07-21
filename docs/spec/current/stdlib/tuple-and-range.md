# Specification — `Tuple` and `Range`

> **Status:** **Accepted** (representation + `Tuple` literal + `Range` bound convention
> ratified by the collections umbrella
> [ADR-0032](../../../adr/0032-collections-representation-and-literals.md); `Range`
> literal `a..b`/`a...b` reserved-inactive). Absent classes (names reserved); each its own unit per
> [ADR-0020](../../../adr/0020-kernel-list-native-array-protocol.md), both
> satisfying the [collection protocol](./collection-protocol.md). Inherits the
> baseline pin from [`README.md`](./README.md).
>
> **Owner:** U-STD / deferred.

## 1. `Tuple` — fixed-arity immutable product

`(3, 4)` — an ordered, **immutable**, fixed-arity group. Immutability makes it the
collection-protocol's **value-hashable** case (law 4), unlike `List`.

| Selector | Meaning |
|---|---|
| `at(_)` | element at an index → `Some(v)` / `None` (total) |
| `size` | arity (fixed at construction) |
| `each(_)` | apply a 1-arg block per element, in order |
| `hash` | **by value** — consistent with structural `==` (immutable ⇒ hashable) |
| `==(_)` | structural (same arity, pairwise-`==`) |

- **No `add`/`at(_, put:)`** — immutable; mutation selectors are absent by design.
- **Destructuring** (`let (a, b) = t`) is surface syntax owned by U-LEX, not this
  spec; the value contract here is what destructuring reads.

## 2. `Range` — numeric interval

`a..b` / `a...b` — a lazy numeric sequence, not a materialized list.

| Selector | Meaning |
|---|---|
| `each(_)` | iterate `a, a+1, …` up to the bound |
| `includes(_)` | is `n` within the range → `Bool` |
| `size` | element count (derived from bounds) |
| `toList` | materialize into a `List` |
| `first` / `last` | endpoints |

**Bound convention (RG-1 — ratified, [ADR-0032](../../../adr/0032-collections-representation-and-literals.md)):**
`a..b` **inclusive** of `b`, `a...b` **exclusive** of `b` (the two-dot / three-dot
split in `object-model.md` §3). The `..`/`...` operator literal is
**reserved-inactive** with this committed meaning; construct a `Range` via its
constructor until it activates.

**Laziness (RG-2):** `Range` does **not** allocate its elements; `each` generates
them. `toList` is the explicit materialization escape hatch. This keeps `1..1000000`
cheap and is the reason `Range` is a distinct class rather than a `List` factory.

## 3. Representation

Both are small native heap arms (like `List`/`Tuple` over the arena):

- `Tuple` — a fixed-length immutable slice of `Value`s; hashes by folding element
  hashes (value-hashable).
- `Range` — three fields (`start`, `end`, `inclusive`); no element storage.

Any raw floor primitives they need are a scoped
[ADR-0019](../../../adr/0019-freeze-vm-blessed-primitive-floor.md) amendment,
justified when the unit lands; combinators stay `.ph`.

## 4. Non-goals

- **Literal syntax.** The **`Tuple` literal `(a, b)` is ratified** and ships
  ([ADR-0032](../../../adr/0032-collections-representation-and-literals.md) §3.2:
  disambiguated from grouping `(a)` by the comma; one-element `(a,)`; `()` empty).
  The **`Range` literal `1..5` / `1...5` is reserved-inactive** with the committed
  inclusive/exclusive convention (§3.3 / RG-1). Parser/compiler work is U-LEX.
- **Non-numeric / stepped ranges** (`Range` over chars, custom step) — deferred.

## 5. Test strategy

Instantiate the [collection-protocol](./collection-protocol.md) harness for each;
plus: `Tuple` value-hash equality (`(1,2).hash == (1,2).hash`), immutability (no
mutation selectors); `Range` inclusive/exclusive bound parity, laziness (large range
`each` without materialization), `toList` round-trip.

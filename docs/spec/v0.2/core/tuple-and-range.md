# Specification — `Tuple` and `Range`

> **Status:** **Proposal.** Absent classes (names reserved); each its own unit per
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

**Bound convention (sub-decision RG-1):** `a..b` **inclusive** of `b`, `a...b`
**exclusive** of `b` (the two-dot / three-dot split in `object-model.md` §3). Ratify
in the owning ADR before implementation.

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

- **Literal syntax** `(a, b)` (Tuple) and `1..5` / `1...5` (Range) — surface syntax,
  U-LEX; a sibling of [`list-literal-syntax.md`](./list-literal-syntax.md). Note the
  `(a, b)` tuple literal must be disambiguated from a parenthesized expression `(a)`
  by the comma — flag for the literal-syntax ADR.
- **Non-numeric / stepped ranges** (`Range` over chars, custom step) — deferred.

## 5. Test strategy

Instantiate the [collection-protocol](./collection-protocol.md) harness for each;
plus: `Tuple` value-hash equality (`(1,2).hash == (1,2).hash`), immutability (no
mutation selectors); `Range` inclusive/exclusive bound parity, laziness (large range
`each` without materialization), `toList` round-trip.

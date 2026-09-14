# 32. Collections: native representation, shared protocol, and literal surface

> **Supersession note (C.2 Range syntax and representation):** the historical
> Range rows below are superseded for Range only. Range now records optional
> lower/upper bounds, `..` is upper-exclusive, `..=` is upper-inclusive, and
> direct bytecode replaces the public three-argument constructor. `...` is not
> Range syntax.

- Status: Accepted
- Date: 2026-07-12
- Related: [ADR-0020](0020-kernel-list-native-array-protocol.md) (native `List`);
  [ADR-0029](0029-list-literal-syntax.md) (list literal — **ratified to Accepted here**);
  [ADR-0023](0023-amend-floor-admit-hash-and-kernel-reflection.md) (`hash`);
  [ADR-0021](0021-no-truthiness-enforcement.md) (no truthiness);
  [ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) (floor);
  [ADR-0016](0016-hand-written-lexer-and-recursive-descent-parser.md) (lexer/parser);
  [`decisions.md`](../../forge/units/U-CORE-0/decision-register.md) Q5;
  [`collection-protocol.md`](../../spec/current/core/collection-protocol.md),
  [`map-and-set.md`](../../spec/current/stdlib/map-and-set.md),
  [`tuple-and-range.md`](../../spec/current/stdlib/tuple-and-range.md),
  [`list-literal-syntax.md`](../../spec/current/syntax/list-literals.md);
  [open-question 6](../../spec/current/open-questions.md) (set literal)

## Context

The `Map`/`Set`/`Tuple`/`Range` family had four separate design specs (all
**Proposal**) and an in-flight list-literal ADR ([ADR-0029](0029-list-literal-syntax.md),
**Proposed**), but no ratified decision on the two axes that gate the collections
unit ([deferred-work §2](../../spec/current/deferred-work.md)): **per-class storage** and
**which literal sigils ship**. This ADR is the umbrella that ratifies the family —
flipping the four specs and ADR-0029 to Accepted — so each class's unit can be
scheduled. It changes no runtime today; it fixes the representation, the shared
contract, and the surface.

## Decision

### 1. Native heap-arm representation for every collection

Each collection is a **native heap arm** over the arena
([ADR-0009](0009-handle-arena-heap.md)), mirroring `List`'s `ListObject`
([ADR-0020](0020-kernel-list-native-array-protocol.md)):

| Class | Arm | Backing | Mutability |
|---|---|---|---|
| `Map` | `Object::Map` | Rust hash table keyed by `hash`+`==` | mutable |
| `Set` | `Object::Set` | Rust hash set keyed by `hash`+`==` | mutable |
| `Tuple` | `Object::Tuple` | fixed-length immutable `Value` slice | immutable |
| `Range` | `Object::Range` | three fields (`start`, `end`, `inclusive`); no element storage (lazy) | immutable |

The `.ph`-over-`List` alternative is **rejected** — O(n) lookup defeats `Map`/`Set`,
and hashing in `.ph` is awkward. Each native arm brings a **small, scoped
[ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) amendment** for its raw
primitives (raw hash-table get/put/has/iterate for `Map`/`Set`; slice access for
`Tuple`; bound fields for `Range`), justified per-unit exactly as `List`'s five raw
primitives were. **Combinators (`map`/`filter`/`reduce`/…) stay `.ph`.**

### 2. Shared protocol + laws are binding

All four satisfy the [collection-protocol](../../spec/current/core/collection-protocol.md)
(already Normative): totality, deterministic iteration, structural `==`, and
**hashability-iff-immutable**. Concretely (decisions.md Q5): `List`/`Map`/`Set` are
mutable ⇒ **identity `hash`, not valid `Map`/`Set` keys**; `Tuple`/`Range` are
immutable ⇒ **value `hash`, valid keys**. Structural `==` holds for all.

### 3. Literal surface — what ships now

| Literal | Ships? | Lowering / status |
|---|:--:|---|
| **List** `[a, b, c]` | ✅ | ADR-0029 (ratified here): desugar to `List.new().add(a)…`; trailing comma; construction-only (no subscript sugar). |
| **Map** `{k: v, …}` | ✅ | desugar to `Map.new().at(k, put: v)…` (§3.1). |
| **Tuple** `(a, b)` | ✅ | desugar to `Tuple` construction; disambiguated from grouping by the comma (§3.2). |
| **Set** `#{…}` | ⛔ reserved | sigil reserved, **inactive**; construct via `Set.new()` / `Set(…)` (open-Q6). |
| **Range** `a..b` / `a...b` | ⛔ reserved | operator sigils reserved, **inactive**, with committed convention (§3.3); construct via the `Range` constructor. |

All shipping literals are **pure parser desugaring to construction sends** — they
add **no floor primitive** beyond the target class's own arm (the ADR-0019 default,
as ADR-0029 established for `List`).

#### 3.1 Map literal `{k: v}`

- **Disambiguation from a block.** `{}` and `{ stmts }` / `{ p => body }` remain
  **blocks**. A brace group is a **map literal** iff its contents are one or more
  comma-separated `key: value` pairs. The empty map is `Map.new()` — **not** `{}`
  (which stays the empty block).
- **Key rule.** A bare identifier before `:` is a **symbol** key (`{a: 1}` ⇒ key
  `#a`), consistent with the language's keyword-label convention
  ([ADR-0025](0025-external-internal-parameter-names.md)). A **string/number literal
  or parenthesized expression** key is taken as an expression:
  `{"name": v}` ⇒ string key, `{(k): v}` ⇒ computed key. (A future extension may
  broaden computed keys; symbol + literal + parenthesized covers the common cases.)
- **Desugar.** `{a: 1, b: 2}` ≡ `Map.new().at(#a, put: 1).at(#b, put: 2)`; keys and
  values evaluate left-to-right; the value is a fresh mutable `Map`.

#### 3.2 Tuple literal `(a, b)`

- **Disambiguation from grouping.** `(a)` is a **parenthesized expression**;
  `(a, b)`, `(a, b, c)`, and the one-element `(a,)` (trailing comma required) are
  **tuples** — the comma is the distinguisher (Python precedent). `()` is the empty
  tuple.
- **Desugar.** `(a, b)` ≡ a `Tuple` construction send over the immutable arm;
  elements evaluate left-to-right; the value is value-hashable (§2).

#### 3.3 Range sigils `a..b` / `a...b` — reserved, committed

The operator literals are **reserved and inactive**, but their meaning is
**committed** so activation later is not a fresh decision (RG-1/RG-2 of
[tuple-and-range.md](../../spec/current/stdlib/tuple-and-range.md)):

- `a..b` — **inclusive** of `b`; `a...b` — **exclusive** of `b`.
- `Range` is **lazy** — `each` generates `a, a+1, …`; no element allocation;
  `toList` is the explicit materialization escape hatch.

Until the sigils activate, a `Range` is built through its class-side constructor
(owned by the `Range` unit).

## Consequences

- **Three genuinely-open collection decisions closed.** Representation (native
  arms), shared contract (binding), and literal surface (list/map/tuple ship;
  set/range reserved) are fixed; the four specs + ADR-0029 flip to Accepted, and
  deferred-work §2's collections row is struck.
- **Zero new floor for the literals.** Every shipping literal desugars to
  construction sends on its class; the only floor cost is each native arm's own raw
  primitives, admitted per-unit under the ADR-0019 amendment convention.
- **`#` and `..`/`...` stay free with committed semantics.** Reserving the set and
  range sigils (rather than shipping or discarding them) means a later unit can
  activate them additively without re-deciding meaning — and keeps the first
  literal batch small.
- **Owners.** List/Map/Tuple literals: **U-LEX** (surface) over the class arms;
  `Map`/`Set`/`Tuple`/`Range` runtime + arms: **U-STD**, each its own unit
  ([ADR-0020](0020-kernel-list-native-array-protocol.md)), each conforming to the
  protocol harness.

## Alternatives considered

- **`.ph`-over-`List` representation.** Rejected — O(n) `Map`/`Set` lookup and
  awkward `.ph` hashing; the native-arm/`.ph`-protocol split (ADR-0020) is the
  established pattern.
- **Ship set + range literals now.** Rejected for scope: `#{…}` and `..`/`...` add
  lexer/parser surface and (for `..`) an operator-precedence decision the first
  batch doesn't need; reserving them with committed semantics keeps the door open at
  no cost.
- **Map literal `{}` for empty map.** Rejected — collides with the empty block;
  requiring ≥1 `key: value` pair (empty map = `Map.new()`) keeps blocks unambiguous.
- **Expression keys for bare identifiers (`{a: 1}` ⇒ key = value of `a`).**
  Rejected — surprising in a Symbol-having, keyword-label language; bare identifier
  = symbol key matches the rest of the surface, with parenthesized keys for computed
  cases.
- **`BuildMap`/`BuildTuple` opcodes now.** Deferred, not foreclosed — same posture
  as ADR-0029's `BuildList(n)`: desugar to sends until a profile demands otherwise.

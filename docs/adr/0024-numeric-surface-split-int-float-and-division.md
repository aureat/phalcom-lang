# 24. Split `Number` into exact `Int` (auto-promoting bignum) and `Float`; `/` is true division, `~/` is integer division

- Status: Accepted
- Date: 2026-07-12
- Supersedes (in part): [ADR-0005](0005-number-as-flat-f64.md) — its single-`f64`
  representation now applies only to `Float`; `Int` gets its own exact representation.
- Related: [ADR-0009](0009-handle-arena-heap.md) (heap home for `LargeInt`),
  [ADR-0010](0010-tagged-value-enum.md) (`Value` gains `Int`/`Float` variants),
  [ADR-0023](0023-amend-floor-admit-hash-and-kernel-reflection.md) (value-based `Number#hash`),
  [ADR-0018](0018-sacred-selector-inliner-and-override-guard.md) (arithmetic inliner + deopt),
  [ADR-0012](0012-selector-signature-encoding-and-dispatch.md) (selector dispatch),
  `docs/spec/v0.2/open-questions.md` Q2, `docs/spec/v0.2/core/forward-compat.md` §4,
  `phalcom-core/src/value.rs`, `primitive/number.rs`

## Context

[ADR-0005](0005-number-as-flat-f64.md) modeled every number as one unboxed `f64`
and deferred an `Int`/`Float` split. Open-question Q2 asked whether to (a) expose
distinct surface types and (b) give integers *exact* semantics. Both are now
answered **yes**: users want `1.class != 1.0.class`, whole-number-only index/count
sites, and — decisively — **integer arithmetic that is never silently wrong**.

Exactness cannot be delivered by an `f64`-backed tag: `f64` is exact only to 2^53,
so a tag-only split would still lose precision on large integers. Delivering "never
wrong" integers means a real representation split, which is why this ADR *supersedes*
ADR-0005's single-representation clause rather than merely amending it.

The decision is gated **before the arithmetic inliner
([ADR-0018](0018-sacred-selector-inliner-and-override-guard.md)) hardens**, because
both the two-representation arithmetic and the division result-type rule must be in
place before fast paths are burned into bytecode.

## Decision

### 1. Surface tower
`Number` is **abstract** with two concrete subclasses:
- `Int  < Number` — exact, **unbounded** integers,
- `Float < Number` — IEEE-754 `f64` (ADR-0005's representation, retained here).

`Number` has no instances. Literals decide the class: `1` is an `Int`, `1.0` is a
`Float`.

### 2. `Int` representation — auto-promoting bignum (Smalltalk lineage)
`Int` is exact and never overflows, via a two-tier representation that is **invisible
at the surface** (`5.class` and `(100.factorial).class` both report `Int`):
- **Small path:** a tagged immediate `Value::Int(i64)` — the common case, no heap.
- **Large path:** a heap `LargeInt` (arbitrary-precision) on the ADR-0009 handle heap.
- **Promotion:** `+`/`-`/`*`/`~/` use `checked_*` on the `i64` fast path and **box to
  `LargeInt` on overflow**; operations that fit back into `i64` demote. There is **no
  trap and no wraparound** — `Int` arithmetic is total and exact.

Unlike Smalltalk, the small/large tiers are **not** distinct surface classes
(`SmallInteger`/`LargeInteger`); they are one `Int` class over two representations.

### 3. Cross-representation equality and hashing
`Int` and `Float` now have different bit patterns, so:
- **`==` is a value comparison.** `1 == 1.0` is `true`; comparison is by mathematical
  value across the two representations.
- **`hash` canonicalizes** (Python-style): an integral `Float` hashes as the equal
  `Int`, so `2.hash == 2.0.hash`. This satisfies
  [ADR-0023](0023-amend-floor-admit-hash-and-kernel-reflection.md) / forward-compat §4
  (`a == b ⇒ a.hash == b.hash`), which pre-authorized exactly this scheme.

### 4. `/` is always true division
`Int / Int` **promotes to `Float`**; truncation is never silent.
```phalcom
7 / 2        "=> 3.5   (Float)"
6 / 2        "=> 3.0   (Float) — even division still yields Float"
6 / 3 == 2   "=> true, but (6 / 3).class => Float"
7.0 / 2      "=> 3.5   (Float) — Float contaminates"
```

### 5. `~/` is integer division
Spelled `~/` (Dart precedent; `//` is unavailable — it is the line-comment token).
Two operands, returns an exact `Int`, **floor semantics** (rounds toward −∞, so its
sign agrees with `%`).
```phalcom
7 ~/ 2       "=> 3    (Int)"
-7 ~/ 2      "=> -4   (Int, floor — not -3)"
7 % 2        "=> 1"
100.factorial ~/ 2   "=> exact LargeInt, no precision loss"
```

### 6. Promotion rule for the other operators
`Int ⊕ Int → Int` (exact, auto-promoting); `_ ⊕ Float → Float` (the `Int` operand
converts to `f64`, which may lose precision — that is the user opting into floats).

## Consequences

- **Integers are never wrong.** `factorial(100)`, big accumulators, and hashing all
  stay exact; no overflow footgun. This is the whole point of the split.
- **Call sites can demand wholeness.** `list.at(i)`, `size`, arity, and loop counters
  take `Int`; a `Float` index is a type error at the boundary, not a deep runtime check.
- **Dispatch splits.** `1.class` is `Int`, `1.0.class` is `Float`; `isA(Int)` is
  meaningful; value types override per class. The tower gains `Int`/`Float` under
  `Number`.
- **`Value` and the heap grow.** `Value` gains `Int(i64)`/`Float(f64)` variants
  (ADR-0010) and a `LargeInt` heap kind (ADR-0009). Large integers are heap objects
  and participate in the GC/heap story like any other object.
- **The inliner keeps an `i64` fast path with a deopt edge.** Arithmetic inlines on
  the `Int(i64)` immediate; the overflow check *is* the guard — on overflow it falls
  off to the boxing slow path, exactly the ADR-0018 deopt shape. `/` always yields
  `Float`, so it needs no result-type guard.
- **Even division surprises.** `6 / 2` is `Float 3.0`, not `Int 3` — the deliberate
  cost of "no silent truncation"; `~/` is the tool when an `Int` is wanted.
- **`==`/`hash` are no longer bit-compares.** They canonicalize across
  representations. Slightly more work, but pre-authorized by ADR-0023.
- **ADR-0005 is partially superseded.** Its `f64` representation survives as `Float`'s
  representation; its "single surface `Number`, integers-are-really-floats" clauses are
  replaced. ADR-0005 gets a superseded-in-part pointer to this ADR.

## Alternatives considered

- **`f64`-backed tag (surface split, shared representation).** Cheapest — no new
  representation — but `Int` would inherit `f64`'s 2^53 ceiling and *not* be exact.
  Rejected: the user requires exact integers.
- **Fixed-width `i64` with trap-on-overflow.** Exact within ±2^63 and cheap (no heap),
  but has a hard ceiling that errors on large results (`factorial(21)` fails).
  Rejected in favor of unbounded exactness; kept in reserve as the fallback if bignum
  cost proves unjustified.
- **`i64` with wraparound.** Fast but silently wrong — the exact footgun the split is
  meant to remove. Rejected.
- **Expose `SmallInteger`/`LargeInteger` as surface classes (literal Smalltalk).**
  Leaks a representation detail into the object model; users would branch on which one
  they got. Rejected: one `Int` surface class over two hidden representations.
- **`/` floors on two `Int`s (C/Rust/Go/Python-2).** `Int`-in → `Int`-out, but
  `(a + b) / 2` silently truncates and `/`'s result type becomes operand-dependent.
  Rejected in favor of true division + explicit `~/`.
- **`~/` truncates toward zero (Dart's actual semantics).** Sign disagrees with `%`.
  Rejected in favor of floor.

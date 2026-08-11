# 48. Amend iteration: bare-cursor end-sentinel + kernel `Iterable` root

- Status: Accepted
- Date: 2026-07-13
- Amends: [ADR-0035](0035-iteration-protocol-cursor.md) §1 (protocol return shape) and
  §4 (loop-scaffold test). Everything ADR-0035 decided that is not restated here still
  holds — this is an amendment, not a supersession.
- Related: [`docs/spec/current/iteration.md`](../../spec/current/iteration.md) (normative spec);
  [ADR-0007](0007-option-as-abstract-with-some-none.md) / [ADR-0044](0044-option-bootstrap-formalization-and-defer-niche-encoding.md)
  (`Option`, the deferred niche); [ADR-0010](0010-tagged-value-enum.md) (`Value`, the
  private `Nil` sentinel); [ADR-0018](0018-sacred-selector-inliner-and-override-guard.md)
  (inliner); [ADR-0020](0020-kernel-list-native-array-protocol.md) (`List`).
- Realized by: **U-ITERABLE** ([`docs/forge/units/U-ITERABLE/plan.md`](../forge/units/U-ITERABLE/plan.md)).

## Context

ADR-0035 §1 defined `iterate(cursor)` to return `Some(nextCursor)` while iterating and
`None` at end. `Some` is an ordinary heap instance (`Object::Instance`, class
`some_class`, `value.rs`) — so **every continuing step of every loop allocates a heap
object**. `None`-at-end is free (the shared singleton), so the cost is entirely the
`Some(index)` wrapper on the hot path. A `for` over `n` elements creates `n` immediate
`Some`s, each born in `iterate` and immediately unwrapped by the loop desugar
(`_c.unwrap`). This is a per-element allocation on the single most common control-flow
construct in the language.

The `Some` wrapper existed to disambiguate a **`None`-valued cursor** (`Some(None)`) from
**end** (`None`). No kernel collection has a `None` cursor (all are integer indices), and
no realistic user cursor is `None`. The disambiguation buys nothing and costs an
allocation per step.

Separately, ADR-0035 left the combinator layer (`each`/`map`/`filter`/`reduce`) as
per-type `.ph` defaults. As built, `List`/`Map`/`Set`/`Tuple`/`Range` each **re-implement**
the identical cursor `iterate` shape plus the combinator set — five verbatim copies of
the same protocol body (`core.ph`). There is no shared iterable root to hang the one
implementation on.

## Decision

### 1. `iterate(_)` returns the **bare cursor**, or the `None` singleton at end

Amends ADR-0035 §1. The protocol becomes:

| Selector | Given | Returns |
|---|---|---|
| `iterate(_)` | the previous cursor, or `None` to start | the **next cursor directly**, or the **`None` singleton** when exhausted |
| `iteratorValue(_)` | a cursor | the element at that cursor |

No `Some` wrapper. The continuing value is the cursor itself (for `List`, a bare
`Number` index) — **zero allocation**; the end value is immediate `None` —
**zero allocation**. This satisfies the rule *no `Option` is reified unless the context
actually needs a reified `Option`* — the loop does not, so none is built.

**New constraint (mirrors Wren's null-cursor rule):** a **cursor value may never itself be
`None`**. `None` is reserved as the end sentinel. (The element `iteratorValue` yields may
be any value including `None`; only the *cursor* is constrained.) Kernel cursors are
integer indices, so this is vacuous for them; a user iterable with an exotic cursor must
pick a non-`None` cursor domain.

The `.ph` realization uses the **two-armed** `ifTrue(_, ifFalse:)` (which returns the
selected arm's value directly) rather than the **one-armed** `ifTrue { }` (which
Option-lifts its arm — the U-CORE-2 `WrapSome`, i.e. the wrapper creation being removed):

```phalcom
iterate(cursor) {
  let next = (cursor == None).ifTrue({ 0 }, ifFalse: { cursor + 1 })
  return (next < self.size).ifTrue({ next }, ifFalse: { None })
}
```

### 2. The loop scaffold tests the end sentinel by **identity**

Amends ADR-0035 §4. `for (x in coll)` lowers to:

```phalcom
var _c = coll.iterate(None)
while (_c != None) { let x = coll.iteratorValue(_c); body; _c = coll.iterate(_c) }
```

The `_c != None` test is a **direct identity comparison against the `None` singleton**,
emitted by the compiler — not an `isSome`/`unwrap` pair of sends, and not a truthiness
branch. `iterate`/`iteratorValue` remain **ordinary, non-inlined sends** (ADR-0035 §4
unchanged); only the sentinel test and the `while` skeleton are compiler-controlled.
`iteratorValue` receives the bare cursor (no `unwrap`). No existing opcode expressed the
`None`-identity branch (the bytecode set branches only via `Jump`/`JumpIfFalse`/`Loop`/
`GuardBool`/`GuardBlock`), so U-ITERABLE adds **one** opcode, `JumpIfNone(i32)` — same shape
as `Jump`/`Loop`, popping TOS and checking the immediate `None` variant. Net
floor-primitive delta (ADR-0019 sense) stays **0**; this is a bytecode addition, tracked
separately.

### 3. Kernel `Iterable` root; rehome the shared layer

A kernel `class Iterable` is introduced. `List`/`Map`/`Set`/`Tuple`/`Range` have their
**superclass wired to `Iterable`** in the Rust bootstrap (`universe.rs`). Defined **once**
on `Iterable`, over the protocol:

- the generic index-cursor `iterate(cursor)` (the §1 shape, over `self.size`),
- `each` / `map` / `filter` / `reduce` / `includes` / `isEmpty`.

A subclass supplies `size` + `iteratorValue` (plus its raw native accessors and mutators).
A collection whose cursor is **not** a `0..size` index overrides `iterate` itself. Kept
**per class** (not hoisted): the raw native accessors, `size`, `at`, `iteratorValue`,
mutators, and the `isA(Kind)`-guarded structural `==`/`!=` — the cross-kind guard (e.g.
`List != Tuple` with identical elements) is per-kind and cannot be generic. Existing
subclass overrides also stand: **`Map#each` is the 2-arity `{ k, v => … }` entry form**, and
`Map`/`Set`/`Range` provide **O(1) `includes`** — the hoist must not delete them. The
generic combinators are therefore written over `iterate`/`iteratorValue` **directly, never
`self.each`** (a `self.each` route would hand `Map`'s 2-arity `each` a 1-arity block).

## Consequences

- **Zero per-step allocation** in `for` and every combinator built on the protocol. The
  hottest construct in the language stops allocating.
- **No-nil Invariant 4 preserved.** `None` is a real surface value tested by **identity**,
  never truthiness; the private `Value::Nil` sentinel does not surface, and no new sentinel
  machinery is introduced. This is *not* a reintroduction of nil — it is `None` used as an
  ordinary sentinel value, exactly as `iterate(None)` already used it to mean "start".
- **5× duplication collapses to one.** The protocol + combinator body lives on `Iterable`;
  per-class code shrinks to raw accessors + `size`/`at`/`iteratorValue` + `==`.
- **Unlocks lazy views** (U-SEQ): view classes `extends Iterable` inherit the whole
  combinator suite by implementing two selectors — the Wren `MapSequence`/`WhereSequence`
  model becomes available.
- **The deferred `Some` niche-encoding** ([ADR-0044](0044-option-bootstrap-formalization-and-defer-niche-encoding.md);
  deferred-work.md §1) is **no longer on the iteration hot path** — iteration never
  allocates a `Some` at all, so the niche is now a pure `Option`-ergonomics optimization,
  not an iteration-performance one.
- **Cost:** a `None`-valued cursor can no longer be distinguished from end (accepted — no
  such cursor exists); and the loop-scaffold gains a compiler-emitted `None`-identity test
  (one small, well-contained lowering change on the spine `compile_for`).

## Alternatives considered

- **Keep `Some(cursor)` and niche-encode `Option` into `Value`.** Rejected for now — the
  niche is a larger `Value`-representation change gated on a GC + benchmarks
  ([ADR-0044](0044-option-bootstrap-formalization-and-defer-niche-encoding.md)), and it
  still pays a tag on every step. The bare-cursor protocol removes the allocation with a
  one-line `.ph` change and no `Value` change.
- **A private-`Nil` fast-path floor protocol (`next_` returning `Value::Nil`) distinct
  from the `Option` surface protocol.** Rejected — forks iteration into two protocols
  (kernel-fast vs user-slow), and `.ph` cannot even produce `Value::Nil` (it has no
  surface constructor; `sentinel_to_option` is one-way). The bare-cursor form needs no
  fork and no new sentinel: `None` is already writable in `.ph`.
- **Compile `for` directly to an index walk** (`i = 0; i < size; at(i)`) bypassing the
  protocol. Rejected — bakes `List`'s index cursor into the compiler and breaks generic
  and user iterables, the exact thing ADR-0035 avoided.

# 35. Iteration protocol: a Wren-style two-selector cursor

- Status: Accepted
- Date: 2026-07-12
- Related: [`docs/spec/current/iteration.md`](../../spec/current/iteration.md) (the promoted
  spec); [`docs/spec/current/control-flow.md`](../../spec/current/control-flow.md) (`for` sugar,
  the inliner); [`docs/spec/current/core/collection-protocol.md`](../../spec/current/core/collection-protocol.md);
  [ADR-0007](0007-option-as-abstract-with-some-none.md) (`Option` cursor);
  [ADR-0018](0018-sacred-selector-inliner-and-override-guard.md) (inliner);
  [ADR-0020](0020-kernel-list-native-array-protocol.md) (`List`);
  [ADR-0030](0030-fibers-and-futures-cooperative-concurrency.md) (generators on `Fiber`);
  promotes `docs/spec/current/experimental/iteration-protocol.md`

## Context

`for`, `.each`, and `.map` all desugar to block sends, but **no selector contract
defines what makes a value iterable** — so `for` cannot be implemented, and a user
type cannot opt into iteration. This was an untracked gap (no open-Q), staged in
`experimental/iteration-protocol.md`. Promoting it requires pinning three things
the draft left implicit: (a) the protocol selectors (the "magic methods"), (b) how
`for` **control flow** — including `break`/`continue` — lowers onto them, and (c)
how those sends interact with the sacred-selector **inliner** and with `Fiber`.

## Decision

**A two-selector internal cursor protocol (Wren-style) — no allocated iterator
object.**

### 1. The protocol (the "magic methods")

A value is **iterable** iff it answers two selectors:

- `iterate(_)` — given the previous cursor, or `None` to start, return the next
  cursor as `Some(cursor)`, or `None` when exhausted.
- `iteratorValue(_)` — given a cursor, return the element at it.

The cursor is an ordinary value (for `List`, an integer index) — **no iterator
object is allocated**. `Option` carries the "more?" signal, so there is no surface
`nil` (Invariant 4). These two are the only "magic methods"; everything else is
built on them.

### 2. `for` lowers to the cursor loop (not to `.each`)

`for (x in coll) { body }` lowers to the cursor **`while` loop**
([iteration.md](../../spec/current/iteration.md) §2), **superseding** control-flow.md's
earlier `for ≡ coll.each{…}` sketch. `for` lowers to the cursor loop *rather than*
`.each` precisely so that **`break` and `continue` work**: they are loop-control
jumps in the desugared `while`, which a block passed to `.each` could not express
(a non-local `return` is not `break`). `.each`/`.map`/`filter`/`reduce` remain
`.ph` combinators over the same protocol, for the full-traversal case.

### 3. `break` / `continue` are loop-control jumps (direct lowering)

`break` and `continue` are ratified as loop-control keywords for `for`/`while` (no
floor primitive, no block send). A loop that **contains** `break`/`continue`
compiles to a **dedicated jump-based loop** — condition, body, `break` → exit label,
`continue` → step label — **bypassing the overridable `whileTrue(_)` send**, so the
jump targets are always valid and there is **no inliner-deopt path** to define
([ADR-0018](0018-sacred-selector-inliner-and-override-guard.md); methods are open,
[ADR-0026](0026-class-hierarchy-mutability.md)). Loops without them keep the plain
`whileTrue`/cursor desugar (§2). Semantics are identical; the direct lowering is
only how loop-control stays sound. Owned by U-LEX alongside `for`.

### 4. Magic-method dispatch vs. the inliner

`iterate(_)`/`iteratorValue(_)` are **ordinary message sends**, dispatched
normally — they are **not** sacred selectors and are **not** inlined
([ADR-0018](0018-sacred-selector-inliner-and-override-guard.md)). Only the loop
*scaffold* (`whileTrue(_)`, the `Option` `isSome` test) is inlined. So a `for` loop
is: an inlined `while` skeleton driving two regular protocol sends per step. A user
type opts into `for`/`each`/`map` by implementing the two selectors and inherits
every combinator for free.

### 5. Interaction with `Fiber` (ADR-0030)

The cursor protocol needs **no** `Fiber`. Lazy/infinite sequences use a
`Fiber`-backed producer, subject to the restricted-yield model
([ADR-0030](0030-fibers-and-futures-cooperative-concurrency.md) §4): a generator
that `yield`s under a native `.each` callback hits `CannotYieldAcrossNativeFrame`,
whereas a cursor-based or inlined-`while` producer suspends freely.

## Consequences

- **`for` is unblocked** — it has a contract to compile against.
- **One iterable contract.** `List` is the reference iterable; `Map`/`Set`/`Tuple`/`Range`
  and user types conform by implementing `iterate`/`iteratorValue`; all combinators
  fall out.
- **Zero allocation, zero new floor.** The cursor is a value; the protocol is
  `.ph`/sends over each type's existing surface.
- **`break`/`continue` compose** because `for` lowers to `while`.
- **Owners.** U-LEX (`for`/`break`/`continue` surface); U-STD (`each`/`map`/`filter`/`reduce`
  defaults over the protocol). `List` provides the reference implementation.

## Alternatives considered

- **External pull-iterator object (Python `__iter__`/`__next__`).** Rejected —
  allocates an iterator per traversal; the cursor protocol is allocation-free. A
  `Stream`/generator layer, if wanted, builds on `Fiber` (§5), not on this.
- **`do:`-only (Smalltalk).** Rejected — a single internal-iteration selector can't
  express `break`/`continue` without non-local return gymnastics; the cursor form
  gives real loop control.
- **Make `iterate`/`iteratorValue` sacred/inlined.** Rejected — they are
  type-specific and open to override; inlining is reserved for the fixed `Bool`/`Block`
  control selectors (ADR-0018). The loop scaffold around them is already inlined.

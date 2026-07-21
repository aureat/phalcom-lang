# Control Flow

There is no control-flow grammar in Phalcom, only sugar over message sends —
which means `Bool` and `Block` are ordinary, overridable classes, and the
compiler earns the genericity back with a dedicated inliner.

For the exhaustive desugarings and the loop-control lowering, see
[Control Flow](../spec/current/control-flow.md) and [Iteration](../spec/current/iteration.md)
in the spec. This page is the tour.

## The central idea: it's all sends

`if`/`else` and `while` are keyword spellings of two message sends. Both
spellings compile to **identical opcodes** — the keyword form exists because
it's what a newcomer expects, not because the VM treats it specially:

```phalcom
if (c) { "yes" } else { "no" }
// ===
c.ifTrue { "yes" }.ifNone { "no" }
```

```phalcom
while (i < 10) { i = i + 1 }
// ===
{ i < 10 }.whileTrue { i = i + 1 }
```

`ifTrue`, `ifNone`, `whileTrue` are methods defined on `Bool`/`Block` in
`core.ph` — the same way `+` is a method on `Int`. Nothing about `if`/`while`
is a compiler primitive; see [Values](values.md) for the `Bool` class itself.

## `for`, and why it isn't `.each`

```phalcom
for (x in xs) {
  if (x < 0) { continue }
  if (x > 100) { break }
  process(x)
}
```

`for (x in xs) { body }` lowers to a `while` loop over the **cursor iteration
protocol** — two selectors, `iterate(_)`/`iteratorValue(_)`, that every
collection implements — **not** to `xs.each { body }`. That distinction is
why `break`/`continue` exist at all: they're jumps in the desugared `while`,
and a block handed to `.each` has no loop to jump out of. The protocol itself,
and why `.each`/`.map`/`.filter` are just `core.ph` defaults over the same two
selectors, is [Collections](collections.md)'s job to teach — full story in
[Iteration](../spec/current/iteration.md) and
[ADR-0035](../adr/0035-iteration-protocol-cursor.md).

## `and` / `or` / `not` — laziness from the object model

A short-circuiting operator can't be an eager send: send arguments evaluate
before the send happens. Phalcom's answer is Smalltalk's: the right-hand side
is a **block**, so the callee decides whether to run it at all.

```phalcom
isValid(x) and { isInRange(x) }   // isInRange only runs if isValid(x) is true
hasCache()  or  { rebuild() }     // rebuild only runs if hasCache() is false
```

`a and b` is really `a.and { b }`; `a or b` is `a.or { b }`. `and`, `or`, and
`not` are ordinary, overridable methods on `Bool` — laziness isn't a special
case, it falls out of "the argument is a block" the same way it would for any
method you wrote yourself.

## `??` and `?.` — the same trick for `Option`

Phalcom has no `nil`, so these two operators short-circuit over `Option`
instead of over truthiness. Full story, with the `Option` combinator table, in
[Values](values.md):

```phalcom
a ?? b        // a.orElse { b }        — b only evaluates if a is None
opt?.foo      // opt.map { x => x.foo } — None short-circuits the chain
```

## The inliner — why generic control flow is free

Here's the part that makes all of the above safe to build a language on
instead of a curiosity: every `ifTrue`/`whileTrue`/`and`/`or` send above is,
in the common case, **not actually a send**.

When the compiler sees a call to a **sacred selector** where the block
arguments are literal blocks written right at the call site, it emits jump
opcodes directly instead of allocating a closure and pushing a call frame:

```phalcom
(x > 0).ifTrue { "positive" }.ifNone { "non-positive" }
// compiles to a conditional jump over inline bytecode —
// no Block allocated, no call frame pushed
```

| Sacred selector | Inlines to |
|---|---|
| `ifTrue(_)`, `ifFalse(_)`, `ifTrue(_)ifFalse(_)` | conditional jump |
| `and(_)`, `or(_)` | conditional jump |
| `whileTrue(_)`, `repeat(_)` | loop jump |

The inline is guarded by a runtime receiver-type check: if the receiver isn't
the `Bool`/`Block` the inliner assumed, it **deoptimizes to a real send** —
so a redefined `Bool>>ifTrue` or a receiver of some other type still gets
correct, fully generic dispatch. You only pay for a send when you've actually
asked for the generic path.

This is [Invariant 5](../spec/current/control-flow.md), and it's load-bearing on
purpose: if blocks were slow, everyone would learn to avoid them, and "control
flow is just message sends" stops being true in practice even though it's true
on paper. See [ADR-0018](../adr/0018-sacred-selector-inliner-and-override-guard.md)
for the override-guard mechanics, including why a loop containing `break`/
`continue` bypasses the overridable `whileTrue` send entirely rather than
relying on inliner deopt.

---

Next: [Collections](collections.md) — the cursor protocol `for` is secretly
built on, and the `List`/`Map`/`Set`/`Tuple`/`Range` types that implement it.

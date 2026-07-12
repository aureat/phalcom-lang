# Blocks

A block is a closure literal — the value of `{ ... }` or a bare arrow. They are
how Phalcom gets `if`, `while`, `each`, and `on`/`ensure` without baking any of
them into the grammar.

## Literals

```phalcom
n => n * 2                 // unbraced, single parameter, expression body
{ x => x * 2 }              // braced, single parameter — same thing
{ acc, n => acc + n }       // braced, two parameters
{ System.print("hi") }      // braced, zero parameters
```

The unbraced arrow is expression-only — no statements, no `return`. `x => { ... }`
doesn't give you a JS-style arrow with a block body; it's an arrow that *returns
a block*, exactly as it reads. And the unbraced form only ever takes one
parameter: `n, x => n * 2` is illegal, because the comma is already spoken for
by argument lists and tuples — `f(n, x => n * 2)` would be genuinely ambiguous
otherwise. Once you need more than one parameter, or more than one expression,
reach for braces.

## Closures, not just literals

A block captures its lexical environment — locals, `self`, everything in
scope at the point it's written — the same way a closure does in any language
with them. It's a first-class value: bind it, store it, pass it around.

```phalcom
let factor = 3
let scale = { n => n * factor }   // captures `factor`
scale.call(5)                     // 15
scale(5)                          // 15 — call() sugar
```

Because blocks are values, passing behavior as an argument is just passing an
argument:

```phalcom
numbers.map({ n => n * 2 })
```

## Trailing-block sugar

A block literal that follows a call's argument list is passed as the call's
final argument — this is why `each`, `map`, `times`, and friends read like
built-in control flow instead of higher-order calls:

```phalcom
numbers.each { n => System.print(n) }
numbers.reduce(0) { acc, n => acc + n }
5.times { System.print("hi") }
```

Selector identity doesn't change: `numbers.map { n => n * 2 }` and
`numbers.map({ n => n * 2 })` are both sends of `map(_)`. The brace is purely
where the last argument lives, syntactically.

## Non-local return

`return` inside a block doesn't return from the block — it returns from the
**enclosing method**, unwinding through however many frames sit between the
`return` and that method's own frame:

```phalcom
findNegative(numbers) {
  numbers.each { n =>
    (n < 0).ifTrue { return Some(n) }   // exits findNegative, not the blocks
  }
  None
}
```

Contrast with the block's own value, which is just its last expression:

```phalcom
let firstNegative = { numbers.each { n => (n < 0).ifTrue { n } } }
// firstNegative.call() runs the whole `each` and yields its result —
// the inner block's value on the last iteration, not an early exit.
```

If you want "stop as soon as we find it," you need `return`, not a block's
trailing value — the loop itself doesn't stop just because one iteration
produced something. This is also why there's no `break`/`continue` inside a
block: early exit *is* `return`. `break`/`continue` exist only inside
`while`/`for` sugar, where they compile straight to jumps.

`return` is safe to use this way because a block can outlive the frame it
closed over — pass one into a callback that runs later, and the frame it would
return to may already be gone. Phalcom detects this at the return site rather
than corrupting the stack; see
[Blocks §5](../spec/v0.2/blocks.md#5-non-local-return) for the frame-token
mechanism and the exact error.

## The callable tower

A block is a real object — `blk.call()`, `blk.arity`, `blk.on(TypeError) { ... }`
are all ordinary message sends. Under the hood, a block, a method, and a
getter body all share **one closure representation**; `Block` and `Method`
are siblings under an abstract `Function` root
([ADR-0006](../adr/0006-function-as-abstract-callable-root.md)). A `Method`
is not a `Block` — it additionally carries a selector, a holder class, and a
receiver — but both answer the same call protocol, which is what makes
`m.bind(receiver)` able to hand you back something block-shaped. The full
tower, including `methodFor`/`invokeOn` reflection, is in
[Functions, Blocks & Methods](../spec/v0.2/functions.md).

Blocks are also cheap to reach for. The compiler inlines control-flow sends
over block literals whenever it can prove the shape, so `ifTrue`, `each`,
`while` — all ordinary sends over blocks — cost nothing extra on the common
path. More on that in [Control Flow](control-flow.md).

---

Next: [Control Flow](control-flow.md) — how `if`/`while`/`for` desugar to
block sends, and the inliner that makes them free.

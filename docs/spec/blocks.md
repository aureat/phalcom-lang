# Blocks

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

Blocks are the keystone construct. **A block, a lambda, a method body, and a getter
body are all the same thing**, spelled at different levels of ceremony. `Block` is a
real class ([Object Model](object-model.md)); a method is a `Block` bound to a class
under a selector.

## 1. Forms

```phalcom
n => n * 2                      // unbraced, single parameter, expression body
{ acc, n => acc + n }           // braced, any number of parameters
{ System.print("hi") }          // braced, zero parameters
{ x => ... }                    // canonical form
```

`=>` has exactly one meaning throughout the language: **"yields."** It is the same
token in a block header and in a method expression body ([Classes §Methods](classes.md)).

## 2. The unbraced form is expression-only

An unbraced arrow's body is a **single expression** — no statements, no `return`,
no brace-delimited body.

- `x => { ... }` is an arrow that **returns a block**, exactly as it reads — not
  "an arrow with a block body."
- There is no way to write a JS-looking arrow containing `return`. This makes
  non-local return (§5) safe *by construction*.

## 3. Unbraced arrows are single-parameter only

`n, x => n * 2` is **illegal**. The comma already separates call arguments and
tuple elements; giving it a third job creates a true ambiguity:

```phalcom
f(n, x => n * 2)   // two args, or one two-param lambda? unresolvable
```

The brace is what makes the multi-parameter comma safe — inside `{ }` the comma has
no other job.

## 4. Trailing block sugar

A block literal following a call's argument list is passed as the **final
argument**, filling the last declared parameter.

```phalcom
numbers.map { n => n * 2 }
numbers.reduce(0) { acc, n => acc + n }
5.times { System.print("hi") }
file.open("data.txt") { f => f.readAll() }
```

Selector identity is unaffected: `cond.ifTrue { ... }` and `cond.ifTrue({ ... })`
are both sends of `ifTrue(_:)`.

## 5. Non-local return

Every block captures the identity of the method frame in which it was created.
`return` inside a block unwinds to **that** frame and returns from the enclosing
*method*, not from the block.

```phalcom
findNegative(numbers) {
  numbers.each { n =>
    (n < 0).ifTrue { return Some(n) }   // exits findNegative
  }
  None
}
```

The last expression of a block is its value ([Classes §Implicit return](classes.md)),
so `return` is only ever needed for *early* exit.

**Escaping blocks.** A block may outlive its home frame. Give each block a **frame
token** — a frame pointer plus a generation counter. On non-local return, compare
the token against the live frame; if the generation does not match, raise
`DeadFrameError`. A cheap integer comparison converts a memory-safety hazard into a
clean runtime error. (Smalltalk raises `BlockCannotReturn` here.)

## 6. No `break` / `continue`

There is **no** `break` or `continue` inside a block. Early exit from a block is
`return`. `break`/`continue` exist only inside `while`/`for` sugar
([Control Flow](control-flow.md)), where they compile directly to jumps.

## 7. Blocks are objects

```phalcom
blk.call()      blk.call(1, 2)
blk(1, 2)                          // sugar for blk.call(1, 2)
blk.arity
```

Methods and blocks share one closure representation in the VM.
</content>

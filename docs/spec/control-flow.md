# Control Flow

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

## 1. Sugar

`if`, `else`, `while`, `for` are **keyword sugar** over message sends. They exist
because they are what a newcomer expects (Invariant 6), and because both spellings
compile to identical opcodes — nothing is lost.

```phalcom
if (c) { ... } else { ... }          // === c.ifTrue { ... }.ifNone { ... }
while (c) { ... }                    // === { c }.whileTrue { ... }
for (x in xs) { ... }                // === xs.each { x => ... }
```

## 2. `and` / `or` / `??` short-circuit

A short-circuiting operator **cannot** be an eager send — message arguments
evaluate before the send. Smalltalk's answer, which Phalcom adopts: the right-hand
side is a **block**.

```phalcom
a and b     // a.and { b }      -> selector and(_:)
a or  b     // a.or  { b }      -> selector or(_:)
a ?? b      // a.orElse { b }   -> Option
```

Laziness falls out of the object model for free. `and` and `or` are ordinary
methods on `Bool` and can be overridden.

## 3. The inliner — load-bearing

When the compiler sees a send of a **sacred selector** whose block arguments are
**literal blocks at the call site**, it emits jump opcodes instead of a send.

Sacred selectors: `ifTrue(_:)`, `ifFalse(_:)`, `ifTrue(_:)ifFalse(_:)`, `and(_:)`,
`or(_:)`, `whileTrue(_:)`, `repeat(_:)`.

The inlined code is guarded by a receiver type check that **deoptimizes to a real
send** if the receiver is not the expected `Bool` / `Block`. Result: zero closure
allocation and zero call frames on the common path, full genericity when someone
actually needs it.

This must land **early** (Invariant 5). If blocks are slow, users learn to avoid
them and every other decision in the spec unravels.
</content>

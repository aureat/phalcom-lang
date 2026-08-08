# Phalcom Language Specification — Current

**Status:** Draft 0.1 — design baseline for implementation.

Phalcom is a class-based, object-first language with JavaScript's surface
ergonomics and Smalltalk's semantics. This directory is the living specification,
split into parts. The [Open Questions](open-questions.md) (15) are now all resolved;
anything deliberately postponed — deferred decisions, still-open decisions, unbuilt
units — is tracked in [Deferred & Future Work](deferred-work.md) rather than being
silently invented.

## Reading order

| Part | Covers |
|------|--------|
| [Syntax](syntax/README.md) | Consolidated normative grammar — lexical + expression + declaration productions, precedence ladder, and a single-block grammar appendix |
| [Lexical Structure](lexical-structure.md) | Tokens, newline handling, literals, string interpolation, brace disambiguation |
| [Values & Absence](values-and-absence.md) | The value types, private `nil`, `Option` |
| [Object Model](object-model.md) | Kernel classes, the class/metaclass tower, core catalog |
| [Blocks](blocks.md) | Blocks/lambdas, non-local return, `Block` as a class |
| [Functions, Blocks & Methods](functions.md) | The callable tower: abstract `Function`, `Block`, `Method`, one closure representation |
| [Messages & Selectors](messages-and-selectors.md) | Selector identity, labels, spread & rest |
| [Selectors, Symbols & References](selectors.md) | Selector identity, # symbols, :: method references, @ attributes, field visibility |
| [Classes](classes.md) | `@constructor`, `@class`, fields, methods, accessors, operators |
| [Method Lookup](method-lookup.md) | Resolution order, `doesNotUnderstand`, `Message` |
| [Control Flow](control-flow.md) | `if`/`while`/`for` sugar, `and`/`or`, the inliner |
| [Iteration](iteration.md) | The cursor protocol (`iterate`/`iteratorValue`), `for` desugar, `break`/`continue` |
| [Error Handling](error-handling.md) | `throw`, `try`/`catch`/`on`/`ensure`, unwinding as one primitive |
| [Result](result.md) | `Result`/`Ok`/`Err` — the value channel for expected failure; bridges to exceptions |
| [Fibers & Futures](concurrency.md) | Cooperative concurrency: the `Fiber` primitive, `Future`, the scheduler |
| [System](system.md) | The runtime service surface: console, clock, process, scheduler |
| [Numbers](../library/numbers/) | Numeric tower and floating-point protocol |
| [Bitwise operators](../library/numbers/bitwise.md) | Integer bitwise semantics |
| [Standard library](stdlib/README.md) | Public library surfaces extending the core |
| [Traceback](traceback/README.md) | Diagnostic rendering contract |
| [Modules & Imports](modules.md) | `import "./path" as Name`, the `Module` namespace object, canonical-path memoization, cyclic imports |
| [Implementation Status](../../forge/spec-status.md) | Divergence between this spec and the current tree |
| [Open Questions](open-questions.md) | The 15 design questions — **all resolved**; decision record |
| [Deferred & Future Work](deferred-work.md) | Master index of everything postponed: deferred decisions, open decisions, unbuilt units, the experimental corpus |

## Invariants

These are the load-bearing rules. Any feature that violates one is rejected or
forces an explicit amendment.

1. **Everything is a message.** Operators, control flow, field access, and
   iteration are message sends underneath. Sugar is encouraged, but must desugar
   to sends.
2. **Named argument labels are part of selector identity.** `move(to,duration)`
   and `move(_,_)` are different selectors — a compiler fact, not sugar.
3. **Method lookup is one hashmap hit** on an interned selector symbol, warm.
4. **`nil` is a private VM primitive.** Never user-visible. Absence is `Option`
   (`Some(v)` / `None`).
5. **Blocks are first-class and cheap.** The inliner
   ([control flow](control-flow.md)) is load-bearing, not an optimization.
6. **Nothing surprises a JavaScript programmer** unless the surprise is
   signposted by unfamiliar syntax.

## Example

```phalcom
class Person {
  @constructor
  new(name:, age:) {
    _name = name
    _age = age
  }

  @constructor
  anonymous() { _name = "Anonymous" }

  name  => _name
  age   => _age
  isAdult => _age.map { a => a >= 18 }.unwrapOr(false)

  ==(other) => self.name == other.name and self.age == other.age

  describe() {
    _age.ifSome { a => return "\(_name), \(a)" }
    "\(_name), age unknown"
  }
}

let people = [
  Person.new(name: "Bob", age: 30),
  Person.new(name: "Alice"),
  Person.anonymous()
]

people.filter(p => p.isAdult)
      .map(p => p.name)
      .each { n => System.print(n) }
```
</content>

# The Phalcom Guide

A developer reference for people who already write code. It assumes you know what
a closure, a vtable, and a call frame are, and spends its words on what makes
Phalcom *specific* rather than re-teaching programming.

Phalcom is a **class-based, object-first language with JavaScript's surface and
Smalltalk's semantics**. If that phrase means something to you, you already have
the shape of the language:

- **Everything is a message.** `1 + 2` is `1.+(2)`. `if`/`while`/`for` are sugar
  over method sends. There is no privileged operator set the way there is in C —
  operators are just methods with punctuation for names.
- **Objects all the way up.** Classes are objects. Their classes (metaclasses) are
  objects too. `Int` is an instance of `Int class`, which is an instance of
  `Metaclass`. This is not trivia — it is how `static` methods and per-class state
  work without a bolted-on mechanism.
- **The surface looks like JavaScript.** Curly braces, `let`/`var`, `class Foo
  extends Bar`, `x => x + 1`. A JavaScript programmer can read Phalcom on day one.
  Where the semantics differ, the syntax warns you (Invariant 6: nothing surprises
  a JS programmer unless the surprise is signposted).

```phalcom
class Greeter {
  construct new(name:) { _name = name }

  greet() => "Hello, \(_name)"
}

Greeter.new(name: "Ada").greet()   // "Hello, Ada"
```

## How this guide relates to the spec

This is a **guide**: it teaches, in prose and examples, and it links out. The
normative rules — every combinator, every grammar production, every edge case —
live in the [language specification](../spec/v0.2/README.md), which is the source
of truth. When the guide says "see the spec," it means *that document owns the
exhaustive answer*; the guide owns the intuition and the shortest path to it.

The spec is versioned and frozen per release. This guide tracks the current
language.

## Reading order

Each page is self-contained enough to skim, but they build on each other in this
order:

| Page | What it covers |
|------|----------------|
| [Values](values.md) | `Int`/`Float`, `String` + interpolation, `Bool`, and the big one — there is no `nil`, absence is `Option` |
| [Variables](variables.md) | `let` vs `var`, block scope, `_fields`, and why bindings are not the same as slots |
| [Classes](classes.md) | `construct`, fields, methods, getters/setters, operators, `static`, `extends` |
| [Messages & Dispatch](messages.md) | Selector identity, labelled arguments, spread/rest, method lookup, `doesNotUnderstand` |
| [Blocks](blocks.md) | Block literals, closures, non-local return, and blocks as the substrate of control flow |
| [Control Flow](control-flow.md) | `if`/`while`/`for`, `and`/`or`, `??`/`?.`, and the inliner that makes it all cheap |
| [Collections](collections.md) | `List`, `Map`, `Set`, `Tuple`, `Range`, and the one cursor protocol behind every loop |
| [Errors](errors.md) | `throw`/`try`/`on`/`catch`/`ensure` for the exceptional, `Result`/`Ok`/`Err` for the expected |
| [Modules](modules.md) | Files as modules, `import`, and what a name actually resolves to |
| [Concurrency](concurrency.md) | `Fiber` as the one cooperative primitive, `Future`, the scheduler |
| [The Object Model](object-model.md) | The class/metaclass tower, method lookup, and why "classes are objects" pays off |

If you read only one page beyond this one, read [Messages & Dispatch](messages.md).
Selector identity is the single idea the rest of the language is built on.

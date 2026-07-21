# Values

Values are the atomic objects everything else is built from. They are immutable:
`3` is always `3`, and a `String`'s bytes never change in place. Each is a real
object with a real class, so `42.toString()` and `"hi".size` are ordinary message
sends, not compiler magic.

For the exhaustive protocol on each type, see
[Values & Absence](../spec/current/values-and-absence.md). This page is the tour.

## Numbers: `Int` and `Float` are different types

Most scripting languages have one number type. Phalcom has two, and the split is
deliberate — it is the difference between "I am counting" and "I am measuring."

- **`Int`** is an exact, arbitrary-precision integer. `2 ** 200` is not an
  approximation and never silently loses a bit.
- **`Float`** is an IEEE-754 `f64`. `0.1 + 0.2` does what f64 always does.

```phalcom
42            1_000_000        // Int (underscores are digit separators)
3.14          6.022e23         // Float
```

The two do not implicitly convert. `1 + 2.0` is a type decision, not a coercion
you get for free — mixing them is a send whose behaviour is defined by the numeric
protocol, not by C-style promotion rules. See
[ADR-0024](../adr/0024-numeric-surface-split-int-float-and-division.md) for the
full rationale.

### Two division operators

Because the types are distinct, division is too:

```phalcom
7 / 2         // 3.5   — true division, yields a Float
7 ~/ 2        // 3     — floor division, stays in Int
```

`~/` is integer division. Reach for it when you mean "how many whole times" and
`/` when you mean "what fraction." No guessing based on operand types.

## Strings

A `String` is an immutable sequence of characters, written in double quotes:

```phalcom
"hello"
```

### Interpolation

Phalcom interpolates with `\(...)` — a backslash and a parenthesized expression.
The expression is evaluated, `toString`'d, and spliced in:

```phalcom
let name = "Ada"
let age  = 36
"\(name) is \(age)"                    // "Ada is 36"
"next year: \(age + 1)"                // "next year: 37"
```

Any expression fits inside the parentheses, including message sends. A literal
`\(` is written `\\(`. The choice of `\(` over `${...}` or `%(...)` is
[ADR-0022](../adr/0022-string-interpolation-backslash-paren-sigil.md); the short
version is that a backslash already means "escape," so `\(` reads as "escape into
code" with no new sigil to learn.

Each `\(expr)` desugars to a `toString` send and concatenation — so anything with
a `toString` interpolates, and you control how your own types render by defining
one.

## Booleans

`true` and `false` are the two instances of `Bool`. What is unusual is that `Bool`
is an *ordinary class*, not a VM builtin: `ifTrue`, `ifFalse`, `and`, `or` are
methods you could have written yourself.

```phalcom
(x > 0).ifTrue { "positive" }
```

This falls out of "everything is a message," and it is why control flow is
extensible — see [Control Flow](control-flow.md). The compiler *inlines* these
sends when it can prove the receiver is a `Bool`, so the genericity costs nothing
on the common path.

## There is no `nil`

This is the value story you most need to internalize: **Phalcom has no `nil`,
`null`, or `undefined`.** None. There is no literal for "no value," and user code
cannot produce one. (`nil` exists inside the VM for uninitialized slots, but it has
no surface syntax and can never leak to you — Invariant 4.)

Absence is a real value with a real type: `Option`.

### `Option` — `Some(v)` or `None`

```phalcom
Some(42)      // there is a value, and it is 42
None          // there is no value
```

A `var` with no initializer is `None`. A declared-but-unassigned field reads as
`None`. Anything that "might not be there" hands you an `Option`, and the type
system makes you deal with the empty case instead of discovering it at 3am as a
null-pointer crash.

You rarely branch on an `Option` by hand. You transform it:

```phalcom
findUser(id)                           // Option<User>
  .map { u => u.name }                 // Option<String>
  .unwrapOr("anonymous")               // String — supply the default
```

The protocol splits cleanly by what a method *returns*:

| Group | Methods | Stays an `Option`? |
|-------|---------|--------------------|
| Transform | `map`, `flatMap`, `filter`, `orElse`, `zip` | yes |
| Extract | `unwrapOr`, `unwrapOrElse`, `unwrap`, `match` | no — you leave `Option`-world |
| Effect | `ifSome`, `ifNone` | yes (returns `self`, so calls chain) |
| Query | `isSome`, `isNone`, `contains` | returns a `Bool` |

`ifSome`/`ifNone` run a block for its side effect and hand `self` back — they never
extract, which is what keeps them chainable:

```phalcom
opt.ifSome { v => log(v) }.ifNone { warn("missing") }   // -> opt, unchanged
```

The one primitive that *leaves* `Option` with a value is `match`:

```phalcom
opt.match(
  some: { v => "got \(v)" },
  none: { "empty" }
)
```

Under the hood `Some` and `None` are two subclasses of an abstract `Option`,
mirroring `Bool`/`True`/`False`. `None` is a single shared instance — zero
allocation, identity-comparable. Every combinator is two method definitions
(`Some>>map`, `None>>map`), so *dispatch* does the branching; there is no tag to
test. That is the object model earning its keep, and the same trick shows up again
in [Errors](errors.md).

### No truthiness

`if (opt) { ... }` is a **compile error**. An `Option` is not a `Bool`, and
Phalcom will not quietly treat "present" as "true." Go through `.isSome`/`.isNone`
or `ifSome`/`ifNone`. This is a signposted departure from JavaScript — with no
`nil`, truthiness has nothing left to mean.

### `??` and `?.`

Two operators are sugar over `Option` sends, and they short-circuit:

```phalcom
a ?? b            // a.orElse { b }        — b only runs if a is None
opt?.foo          // opt.map { x => x.foo } — None short-circuits the chain
opt?.bar(baz)     // opt.map { x => x.bar(baz) }
```

Chained, each `?.` hop stays inside `Option`, and the first `None` skips the rest —
JavaScript's optional chaining, but honest about the type it produces.

## `Result` — absence with a reason

`Option` says "nothing here." When you need "nothing here, *and here's why*," reach
for `Result`: `Ok(v)` or `Err(e)`. It mirrors `Option` method-for-method (`map`,
`flatMap`, `unwrapOr`, `match`) and the two convert freely (`opt.okOr(err)`,
`result.ok()`). Use `Result` for expected, local failure — parsing, validation,
lookups — and save `throw` for the genuinely exceptional. The full story, including
how `Result` bridges to exceptions, is in [Errors](errors.md).

---

Next: [Variables](variables.md) — how `let`, `var`, and `_fields` bind these values
to names.

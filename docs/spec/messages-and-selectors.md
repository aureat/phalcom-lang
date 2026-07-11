# Messages & Selectors

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

**Governing ADRs:**
[ADR-0012](../adr/0012-selector-signature-encoding-and-dispatch.md) (label-encoded selectors and inline-cache-ready dispatch)

## 1. Send syntax

Dot notation is the primary send syntax.

```phalcom
receiver.name                      // unary send   -> selector `name`
receiver.add(1, 2)                 // positional    -> selector `add(_:_:)`
receiver.move(to: p, duration: 2)  //               -> selector `move(to:duration:)`
a + b                              // binary        -> selector `+(_:)`
a.name = v                         // assignment    -> selector `name=(_:)`
```

## 2. Selector identity

A selector is an interned symbol encoding **name + argument labels**, Smalltalk
style (Invariant 2):

| Call | Selector symbol |
|------|-----------------|
| `p.name` | `name` |
| `p.add(1, 2)` | `add(_:_:)` |
| `p.move(to: a, duration: b)` | `move(to:duration:)` |
| `p.move(a, b)` | `move(_:_:)` |
| `p.name = v` | `name=(_:)` |
| `a + b` | `+(_:)` |

`move(to:duration:)` and `move(_:_:)` are **distinct methods**. Argument order is
significant; labels are not reorderable. Because labels are baked into the interned
symbol, lookup stays a single hashmap probe.

## 3. Declaration

Labels are declared with a trailing colon on the parameter:

```phalcom
move(to:, duration:) { ... }       // selector: move(to:duration:)
move(x, y) { ... }                 // selector: move(_:_:)
```

A parameter declared `to:` is passed as `to: value` and bound in the body as `to`.
(See [open question](open-questions.md) on separate external/internal names.)

## 4. Rest parameters

```phalcom
sum(*numbers) {
  numbers.reduce(0) { acc, n => acc + n }
}
```

- A rest parameter collects trailing **positional** arguments into a `List`.
- It must be the **last** parameter.
- It is **positional-only** — a labelled parameter cannot be variadic.

**No `**kwargs`.** Labels *are* selector identity (Invariant 2); a method accepting
arbitrary labels has, by definition, an unknown selector — which is what
[`doesNotUnderstand`](method-lookup.md) is for. For open-ended keyed config, take a
`Map`:

```phalcom
configure(options: { host: "localhost", port: 8080 })
```

### Selector encoding for variadics

A variadic method interns as `sum(_...)`. A call `sum(1, 2, 3)` produces selector
`sum(_:_:_:)`, which will not match. Lookup therefore:

1. Exact selector probe (fast path, inline-cached).
2. On miss, probe the **variadic table** — keyed by `(name, min_positional_arity)`
   — before falling through to `doesNotUnderstand`.
3. Cache the resolution in the call site's inline cache.

The variadic table is built once at class-definition time; step 2 never runs on a
warm call site.

## 5. Spread at call sites

```phalcom
f(*args)      f(1, *rest)      [1, 2, *others]      { a: 1, *defaults }      Set(1, *xs)
```

With a spread argument the argument *count* — and therefore the selector — is not
known at compile time, so the compiler cannot emit a static send.

**Resolution.** Spread call sites emit a **`SEND_DYNAMIC`** opcode: it builds the
selector symbol at runtime from the materialized argument count and labels, then
performs a normal lookup. Slower than a static send, and that is correct —
spread is rare and the cost is visible at the call site.

`SEND_DYNAMIC` is the same primitive needed for `Object.perform(selector, args)`,
for reflective dispatch, and for forwarding out of `doesNotUnderstand`. Build it
once, use it three ways.
</content>

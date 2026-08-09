# Messages & Selectors

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

**Governing ADRs:**
[ADR-0012](../../adr/0012-selector-signature-encoding-and-dispatch.md) (label-encoded selectors and inline-cache-ready dispatch)

## 1. Send syntax

Dot notation is the primary send syntax.

```phalcom
receiver.name                      // unary send   -> selector `name`
receiver.add(1, 2)                 // positional    -> selector `add(_,_)`
receiver.move(to: p, duration: 2)  //               -> selector `move(to,duration)`
a + b                              // binary        -> selector `+(_)`
a & b                              // binary        -> selector `&(_)`
~a                                 // unary         -> selector `~()`
a.name = v                         // assignment    -> selector `name=(put)`
```

## 2. Selector identity

A selector is an interned symbol encoding **name + argument labels**, Smalltalk
style (Invariant 2). See [Selectors, Symbols & References §1](selectors.md#1-selector-identity)
for the full canonical-form grammar and rules R1–R5.

| Call | Selector symbol |
|------|-----------------|
| `p.name` | `name` |
| `p.add(1, 2)` | `add(_,_)` |
| `p.move(to: a, duration: b)` | `move(to,duration)` |
| `p.move(a, b)` | `move(_,_)` |
| `p.name = v` | `name=(put)` |
| `a + b` | `+(_)` |
| `a & b` | `&(_)` |
| `~a` | `~()` |

`move(to,duration)` and `move(_,_)` are **distinct methods**. Argument order is
significant; labels are not reorderable. Because labels are baked into the interned
symbol, lookup stays a single hashmap probe.

## 3. Declaration

Declarations separate selector labels from body-local bindings:

```phalcom
move(to, duration) { ... }              // labels and locals share names
move(to target, duration seconds) { ... } // separate external/local names
move(_ x, _ y) { ... }                  // selector: move(_,_)
```

A labeled parameter may declare a **separate internal binding** — this is
**decided** ([ADR-0025](../../adr/0025-external-internal-parameter-names.md)).
`move(to target)` has external label `to` and internal binding `target`: callers
pass `to: value`, and the body refers to it as `target`. The single-word form
(`to`) is sugar for the label==binding case. Selector identity is unchanged — the
**label**, not the internal binding, is what is encoded into the selector symbol
(`move(to,duration)`), so the internal name is a purely local concern.

## 4. Rest parameters

```phalcom
sum(*numbers) {
  numbers.reduce(0) { acc, n => acc + n }
}
```

- Before F.3, a rest parameter collects trailing **positional** arguments into
  a `List`. This U9 behavior is transitional; F.3 will replace it with
  lane-aware Unit/Tuple capture.
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

A variadic method interns as `sum(*)`. A call `sum(1, 2, 3)` produces selector
`sum(_,_,_)`, which will not match. Lookup therefore:

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

**`perform` accepts only selector symbols.** `Object.perform(selector, args)`
requires `selector` to be a *selector symbol* (`#move(_,to,duration)`) — a
complete method identity. Passing a *name symbol* (`#move`, bare) is a type
error: a name symbol identifies a family, not a single method, and `perform`
has no call-site label information to disambiguate with. See
[Selectors, Symbols & References §2](selectors.md#2-symbol-literals-) for the
name-symbol vs. selector-symbol distinction.

---

See [Selectors, Symbols & References](selectors.md) for the full treatment of
selector identity (§1), `#` symbol literals (§2), `::` method references (§3),
`@` attributes (§4), and field visibility (§5) — this part covers send syntax
and declaration; selectors.md covers symbol/reference machinery built on top.
</content>

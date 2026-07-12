# Messages & Dispatch

There are no special forms in Phalcom. `1 + 2`, `p.move(to: q)`, `if (x) { ... }` —
every one of these is a **message send**, and the identity of a message is
sharper than "the method name." Get this page and the rest of the object model
falls out for free.

## Everything is a message

`1 + 2` is sugar for `1.+(2)`. Operators are not a privileged syntax category —
they are methods with punctuation for names, defined on `Int` like any other
method. There is no arithmetic the compiler treats specially; `Int>>+` is a real
method you could, in principle, override.

```phalcom
1 + 2                              // -> 1.+(2)          unary-looking, binary op
receiver.name                      // unary send
receiver.add(1, 2)                 // positional send
receiver.move(to: p, duration: 2)  // labelled send
a.name = v                         // assignment send    -> name=(_)
```

Every one of these compiles to the same instruction: push a receiver and
arguments, send a selector. `if`/`while`/`for` are sugar over sends too (see
[Control Flow](control-flow.md)) — there is no second evaluation path hiding
underneath the syntax.

## Selector identity: name + labels (Invariant 2)

The part that is easy to get wrong coming from another language: a Phalcom
**selector** is not just a name. It is the name plus the argument *labels*, in
order, interned as a single symbol. This is Invariant 2, and it is a compiler
fact, not a stylistic convention.

```phalcom
class Sprite {
  move(x, y)              { ... }   // selector: move(_,_)
  move(to:, duration:)    { ... }   // selector: move(to,duration)
}
```

`move(_,_)` and `move(to,duration)` are **two different methods** that happen
to share a base name. Both can be defined on the same class simultaneously —
there is no arity clash, no overload-resolution step, because they were never
the same selector to begin with:

| Call | Selector symbol |
|------|-----------------|
| `sprite.move(3, 4)` | `move(_,_)` |
| `sprite.move(to: p, duration: 2)` | `move(to,duration)` |
| `sprite.name` | `name` |
| `sprite.name = v` | `name=(_)` |
| `a + b` | `+(_)` |

Labels are part of identity, so label **order** is part of identity too:
`move(to,duration)` and `move(duration,to)` are distinct selectors, and a
caller cannot reorder labels to match whichever one it means. There is no
keyword-argument sorting step — sorting would require knowing the callee's
declared order before dispatch, which is exactly what dispatch is for. See
[Messages & Selectors](../spec/v0.2/messages-and-selectors.md) and
[Selectors, Symbols & References §1](../spec/v0.2/selectors.md#1-selector-identity)
for the full grammar and rules R1–R5; the governing decision is
[ADR-0012](../adr/0012-selector-signature-encoding-and-dispatch.md).

This is also why dispatch is cheap: the selector is baked in at compile time as
an interned symbol, so a call site is one hashmap probe on a known key — no
scanning candidate overloads, no runtime arity check.

## Labelled arguments: the label is not the binding

A trailing colon on a parameter declares a label:

```phalcom
move(to:, duration:) {
  // body sees `to` and `duration` as local names
}
```

The single-word form is sugar for the common case where the word callers use
and the word the body uses are the same. But they don't have to be — a labelled
parameter can declare a **separate internal binding**
([ADR-0025](../adr/0025-external-internal-parameter-names.md)):

```phalcom
move(to target:, duration span:) {
  _position = target      // internal name, reads well in the body
  _duration = span
}

sprite.move(to: p, duration: 2)   // call site is unchanged
```

`to` and `duration` are the labels — they're what's encoded into the selector
(`move(to,duration)`) and what callers write. `target` and `span` are frame-local
slot names — purely a body concern, invisible to dispatch and to callers. Reach
for the split when the word that reads well in a message (`to`, `by`, `with`)
would make an awkward variable name in the body.

## Rest and spread

A trailing `*param` collects extra positional arguments into a `List`. It must
be last, and it's positional-only — a labelled parameter can't be variadic:

```phalcom
sum(*numbers) {
  numbers.reduce(0) { acc, n => acc + n }
}

sum(1, 2, 3)      // numbers = [1, 2, 3]
```

There is no `**kwargs` equivalent. Labels *are* selector identity, so a method
that accepted arbitrary labels would have, by definition, an unknown selector —
that's what `doesNotUnderstand` is for, below. For open-ended keyed config,
take a `Map` instead:

```phalcom
configure(options: { host: "localhost", port: 8080 })
```

Spread at a call site (`f(*args)`, `[1, *rest]`) means the argument count isn't
known until runtime, so the selector can't be resolved statically either — the
compiler emits a dynamic send that builds the selector at the call and looks it
up like any other. It's the same mechanism `Object.perform` and
`doesNotUnderstand` forwarding use underneath. Details in
[Messages & Selectors §4–5](../spec/v0.2/messages-and-selectors.md#4-rest-parameters).

## Method lookup: one hashmap hit, then a walk

Dispatch is deliberately boring:

1. Check the call site's **inline cache** — monomorphic, then polymorphic, then
   megamorphic.
2. On a cache miss, probe the receiver's class for the exact selector, walking
   up the superclass chain (a class-side send starts the walk at the
   metaclass).
3. If nothing matches an exact selector, probe the variadic table.
4. If that also misses, send `doesNotUnderstand(_)`.

That's the whole algorithm — one interned symbol, one hashmap key, a chain
walk on a miss. The full class/metaclass tower this walk runs over belongs to
[The Object Model](object-model.md); this page only needs you to know that
"lookup" means "walk superclasses for this exact selector," full stop. See
[Method Lookup](../spec/v0.2/method-lookup.md) for the normative resolution
order, including how `super.sel` restarts the walk at the *defining* class of
the current method rather than the receiver's class.

## `doesNotUnderstand`: the hook for everything reflective

When lookup falls all the way through, the failed send doesn't just error —
it's **reified** as a `Message` object and handed to `doesNotUnderstand`. This
one hook is what makes proxies, delegation, and DSLs fall out of the object
model for free, instead of needing separate machinery:

```phalcom
class Proxy {
  construct new(target:) { _target = target }

  doesNotUnderstand(msg) {
    // msg.selector -> Symbol      full selector, e.g. #move(_,to,duration)
    // msg.name     -> String      base name, e.g. "move"
    // msg.labels   -> List<String>
    // msg.args     -> List
    _target.perform(msg.selector, msg.args)
  }
}

let p = Proxy.new(target: sprite)
p.move(to: q, duration: 1)   // sprite never defines this on Proxy —
                              // lookup misses, doesNotUnderstand forwards it
```

`Object.doesNotUnderstand(_)` is defined to raise `MessageNotUnderstood` by
default — a plain object that doesn't override the hook fails loudly, as
expected. `perform` is the reflective send this proxy relies on: it takes a
*selector* symbol (`#move(_,to,duration)`), never a bare name symbol
(`#move`) — a name identifies a family of overloads, not one method, and
`perform` has no call-site labels to disambiguate with. See
[Method Lookup §2–3](../spec/v0.2/method-lookup.md#2-doesnotunderstand) and
[Selectors, Symbols & References §2](../spec/v0.2/selectors.md#2-symbol-literals-)
for the name-symbol vs. selector-symbol split.

---

Next: [Blocks](blocks.md) — closures, non-local return, and how blocks become
the substrate every control-flow construct sends messages to.

# Variables

Two binding forms, and the difference is exactly the one word: whether the name
can be reassigned. Everything else — scope, shadowing, fields — follows from
keeping that one distinction sharp.

For the normative rules, see [ADR-0014](../adr/0014-let-and-var-bindings.md) and
[Values & Absence §3](../spec/current/values-and-absence.md).

## `let` vs `var`

```phalcom
let name = "Ada"     // immutable — cannot be reassigned
var count = 0         // mutable — can be reassigned

count = count + 1     // ok
name = "Grace"         // compile error: `name` is a let binding
```

Reach for `let` by default. `var` is the exception you reach for when a name's
value genuinely needs to change under it — a loop counter, an accumulator, a
field-like local. The declaration site tells the reader which one they're
looking at; there's no need to scan for reassignments to find out.

### An uninitialized `var` is `None`, never `nil`

```phalcom
var result
result                 // None
result = 42
result                 // 42
```

There is no `nil` to fall back on for "declared but not yet given a value"
([Values](values.md#there-is-no-nil)) — so `var` with no initializer reads as
`None`, the same absence value you'd get anywhere else. `let` has no
initializer-less form: an immutable binding that can never be assigned isn't
meaningful, so `let x` alone is rejected at compile time.

## Compound assignment, and assignment as a statement

```phalcom
var total = 0
total += 10           // 10
total *= 2             // 20
total -= 5              // 15
```

`+=`, `-=`, `*=`, and friends read the current value, apply the operator, and
write the result back — sugar over `total = total + 10`, not a separate
primitive. They only work on `var` bindings for the obvious reason: they
reassign.

Assignment is a **statement**, not an expression that hands you back a value to
chain off of. `if (x = 5) { ... }` — the classic C typo — has nothing to parse
in Phalcom; there's no assignment-as-expression to accidentally write.

## Block scope and shadowing

A binding is visible from its declaration to the end of the innermost `{ }`
block that contains it — the same rule as JavaScript's `let`/`const`, not
`var`'s function-wide hoisting:

```phalcom
let x = 1
{
  let x = 2
  x                    // 2 — the inner binding
}
x                       // 1 — outer binding, untouched
```

The inner `let x` **shadows** the outer one for the rest of its block; it
doesn't mutate it, and once the block ends the outer name is back. Shadowing
works the same whether the outer name was `let` or `var` — a new binding is a
new name, not an assignment to the old one.

## `_fields`: private instance state

A leading underscore marks a **field** — state that belongs to an instance,
not a binding in some enclosing scope. Fields are a different token class
from ordinary identifiers, and a field reference is legal only inside a class
body ([Lexical Structure §3](../spec/current/lexical-structure.md)):

```phalcom
class Counter {
  @constructor
  new() { _count = 0 }

  increment() { _count = _count + 1 }
  value => _count
}
```

Fields aren't declared up front the way `let`/`var` locals are — assigning to
`_count` anywhere in the class body is what declares it, and the compiler
collects the full set of assigned fields to fix the instance's slot layout.
A field read before it's assigned anywhere in the class is a compile error (it
catches the `_naem`-typo class of bug); a field that's declared but not yet
assigned *on a given instance* reads as `None`, exactly like an uninitialized
`var`. The full model — visibility, inheritance, accessors — is
[Classes §2](../spec/current/classes.md); this page just introduces the binding
form.

## Bindings name values; they aren't slots on the object

It's tempting to picture `let`/`var` the way you'd picture a mutable cell or a
struct field in a lower-level language — a box you can peek at and overwrite in
place. That's not quite the model here. A binding is a **name resolved at
compile time to a stack or closure slot**; what it names is always a value (or,
for `var`, a value that can be replaced by a new one). Reassigning `var count`
doesn't mutate `0` into `1` — `Int` is immutable — it points the name at a
different value. The object underneath never changes; the binding just tracks
which object the name currently means. That's also why capturing a `var` in a
closure captures the *binding*, not a snapshot: the block sees whatever the
name currently points to, the same way it would in JavaScript.

---

Next: [Classes](classes.md) — `@construct`, `@constructor`, the full field-declaration and
visibility model, and how methods turn `_fields` into an object's behavior.

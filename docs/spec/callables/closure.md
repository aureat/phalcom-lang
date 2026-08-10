# Closure

[Callables](README.md) · [Arguments and rest](arguments.md) · [Execution contexts](execution.md) · [Runtime and activation](runtime.md) · [Function](function.md) · [Method](method.md) · [Dispatch and lowering](dispatch.md)

A `Closure` is a sealed/final `Function` value carrying compiled code and
lexical captures. It is a complete callable: no caller-supplied receiver is
needed to enter it. A Closure's captured environment may include local values,
captured mutable cells, module context, lexical authority, and the current
`self` when one exists.

## 1. Literal forms

Canonical Closure literals are:

```phalcom
|| { body }
|x| { body }
|x, y| { body }
|x| expression
|head, *tail| { body }
```

The expression form returns the expression's value. Braced Closure bodies use
the same final-expression rules as Method bodies. The retired expression-body
member syntax is not Closure syntax.

## 2. Closure is a value; a block is syntax

A brace-delimited block establishes lexical scope but does not automatically
allocate a Closure. `|| { body }` and `|parameters| { body }` explicitly create
Closure values. This distinction keeps ordinary lexical grouping and first-class
deferred execution separate.

The parser also recognizes contextual zero-argument trailing Closure sugar
after an eligible method send:

```phalcom
resource.withLock {
    work()
}
```

means:

```phalcom
resource.withLock || {
    work()
}
```

It is postfix-send syntax, not a restoration of braces as general Closure
expressions. To pass a Map or Set where that position is ambiguous, use
parentheses:

```phalcom
resource.withLock({ key: value })
```

Explicit parameterized trailing Closures remain available:

```phalcom
users.any where: |user| { user.active }
```

## 3. Parameters and rest

Closure parameters currently support only fixed positional parameters and at
most one terminal positional rest parameter. They reject labeled parameters,
`**rest`, `***rest`, multiple positional-rest parameters, and fixed parameters
after `*rest`.

```phalcom
|head, *tail| { tail }     // accepted
|**labels| { body }        // rejected
|***arguments| { body }    // rejected
|x, *tail, y| { body }     // rejected
```

Phalcom uses only:

```text
*      positional rest/spread
**     labeled rest/spread
***    complete rest/spread
```

```phalcom
f(*values)
f(**labels)
f(***arguments)
|head, *tail| { body }
```

The spelling `args...` is never rest/spread syntax in Phalcom. `...` is not a
spread operator.

Calling a Closure validates the resulting shape, not the spelling of the
call-site expression:

```text
labeled lane must be empty
positional count must equal fixed count without *rest
positional count must be at least fixed count with *rest
```

For `|head, *tail|`, residual positionals bind as:

```text
zero residual values        → ()
one or more residual values → Tuple
```

Closure rest never produces a List. Outgoing `**` and `***` are syntactically
valid, but a resulting non-empty labeled lane is rejected by the positional-only
Closure parameter model.

## 4. Capture and lexical context

A Closure captures the lexical bindings it references. Mutable captured
bindings retain shared lexical-cell semantics; an escaping Closure therefore
continues to observe the captured state according to the language's binding
rules.

When created while a Method or Closure has a current `self`, the Closure
captures that value:

```phalcom
class Box {
    callback {
        || { self }
    }
}
```

The resulting Closure returns the Box receiver even after `callback` has
returned. Capturing `self` does not change `super`: `super` remains anchored to
the lexically defining Method holder. See
[Execution contexts](execution.md#2-self).

## 5. Return and result

A Closure returns its final expression. Empty bodies return `()`. Bare
`return` means `return ()`, and `return value` returns from the current Closure
activation only.

```phalcom
const f = || {
    return 10
}
```

Calling `f()` returns `10`; it does not return from the Method that created
`f`. There is no implicit non-local return. `None` remains absence, distinct
from Unit `()`.

## 6. Implementation note

Compiled Closure code is represented by `ClosureObject`: shared compiled
callable metadata, module handle, captured upvalue handles, and lexical class.
The template's bytecode is shared; evaluating a literal creates the capture
instance appropriate to that lexical activation.

At call time, the Function router validates positional-only acceptance,
normalizes a rest suffix through `finish_tuple`, rewrites the stack window into
local argument slots, and pushes a Closure frame. See
[`heap/closure.rs`](../../../phalcom-core/src/heap/closure.rs),
[`vm/dispatch.rs`](../../../phalcom-core/src/vm/dispatch.rs), and
[`vm/send.rs`](../../../phalcom-core/src/vm/send.rs).

The current tree has transitional internal closure-carrier machinery. It is
classified at the language surface as `Closure` and must not be documented as
an additional public callable class.

## 7. Related chapters

- [Function](function.md) — common complete-call gateway
- [Arguments and rest](arguments.md) — shape and spread semantics
- [Execution contexts](execution.md) — lexical `self`, `super`, and return
- [Method](method.md) — Method lexical owner and exact activation
- [Runtime and activation](runtime.md) — Closure frame entry

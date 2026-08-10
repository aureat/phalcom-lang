# Closure

[`Closure`](README.md) is a sealed/final [`Function`](function.md) value that
carries compiled code and lexical captures. It is executable without a
receiver supplied at call time. A `Closure` created inside a Method or Closure
captures the current `self` value when one exists; that capture does not move
or alter the defining Method's lexical `super` anchor.

## Literals

Canonical Closure literals include:

```phalcom
|| { body }
|x| { body }
|x, y| { body }
|x| expression
|head, *tail| { body }
```

Closure parameters support fixed positional parameters and at most one
optional terminal positional rest parameter. They reject:

```phalcom
|x, *tail, y| { body }
|*a, *b| { body }
|**labels| { body }
|***arguments| { body }
```

Closure labels are not part of this specification. Fixed parameters after
`*rest` and multiple positional-rest parameters are invalid.

## Rest and spread

Phalcom uses one rest/spread notation everywhere:

```text
*      positional rest/spread
**     labeled rest/spread
***    complete rest/spread
```

Examples:

```phalcom
foo(*values)
foo(**labels)
foo(***arguments)
|head, *tail| { body }
```

The spelling `args...` is never rest/spread syntax in Phalcom. `...` is not a
spread operator.

For:

```phalcom
|head, *tail| { tail }
```

zero residual positional arguments produce `()`. One or more residual
arguments produce a `Tuple`. A Closure never captures rest into a List. A
Closure with no rest parameter does not materialize a rest value.

Outgoing calls may use `*`, `**`, or `***`; the Closure validates the resulting
argument shape. A complete pack with a non-empty labeled lane is rejected by a
positional-only Closure. A complete pack with no labels may succeed.

## Lexical scope and return

Closures preserve lexical captures and establish their own executable
activation. Their final expression is their result. Empty bodies produce `()`.
Bare `return` means `return ()`; `return value` returns from the current
Closure activation only. There is no implicit non-local return to an enclosing
Method or Closure. `None` remains absence, distinct from Unit `()`.

## Trailing Closure sugar

A bare brace following an eligible method-send expression is contextual postfix
sugar for a zero-argument Closure:

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

Likewise:

```phalcom
resource.withLock do: {
    work()
}
```

means:

```phalcom
resource.withLock do: || {
    work()
}
```

Braces are not general Closure primary expressions. General brace meaning is
preserved outside this contextual trailing-send position. Parentheses
disambiguate a Map or Set argument:

```phalcom
resource.withLock({ key: value })
```

Explicit parameterized trailing Closure syntax remains available, for example
`users.any where: |user| { user.active }`.

## Related callable types

- [`Callable model`](README.md) — shared `self`, return, and hierarchy rules
- [`Function`](function.md) — common `call(***arguments)` gateway
- [`Method`](method.md) — lexical owner and `super` anchor

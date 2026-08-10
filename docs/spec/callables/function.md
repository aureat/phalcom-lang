# Function

[`Function`](README.md) is the abstract, sealed, VM-backed root of complete
callables. Its concrete descendants are [`Closure`](closure.md),
[`BoundMethod`](bound-method.md), and [`Family`](family.md). A Function has no
missing receiver or other implicit runtime input: its remaining inputs are
explicit call arguments.

[`Method`](method.md) is not a Function. It is holder-owned behavior that still
requires a compatible receiver.

## Common gateway

The canonical Function gateway is:

```phalcom
call(***arguments)
```

Application remains ordinary message syntax:

```phalcom
f(args)
```

is observationally equivalent to:

```phalcom
f.call(args)
```

The `call` base-selector family is final for Function descendants. The runtime
routes this gateway to the concrete sealed representation instead of creating
finite `call`, `call(_)`, `call(_,_)`, and similar overload families. User
objects that define `call` remain callable through ordinary message dispatch,
but they are not thereby Functions.

`callWith(arguments)` is exactly:

```phalcom
self(***arguments)
```

It is not a second binder. The gateway can transport a complete argument shape
without promising that every concrete Function accepts every lane.

The public Function protocol does not prescribe scalar `arity` or generic
`name` fields. Native implementation may use allocation-free argument views,
stack windows, or rooted dynamic-pack storage even though the public gateway is
spelled `***arguments`.

## Argument notation

Phalcom uses one rest/spread notation everywhere:

```text
*      positional rest/spread
**     labeled rest/spread
***    complete rest/spread
```

Examples:

```phalcom
f(*values)
f(**labels)
f(***arguments)
```

The spelling `args...` is never rest/spread syntax in Phalcom. `...` is not a
spread operator.

## Concrete activation

The common gateway transports an argument shape; each concrete Function
performs its own parameter acceptance:

- A Closure validates fixed positional parameters, optional positional rest,
  and its prohibition on labels.
- A BoundMethod activates its stored exact Method with its stored receiver.
- A Family performs its bound method-family routing and then sends normally.

These activations preserve the same `self` and `super` rules described in the
[`callable model`](README.md).

## Related callable types

- [`Callable model`](README.md) — hierarchy and shared semantics
- [`Closure`](closure.md) — lexical Function
- [`BoundMethod`](bound-method.md) — receiver-complete Method
- [`Family`](family.md) — lookup-performing Function

# Method

[`Method`](README.md) is reified exact behavior owned by a holder. It is not a
`Function`: it still lacks a receiver. A Method becomes executable for a
receiver through binding or exact invocation after compatibility validation.

## Semantic identity

A Method carries these fields as one semantic unit:

```text
holder
selector
parameter shape
implementation
lexical super anchor
access authority
```

The holder identifies the class or metaclass that owns the behavior. The
selector identifies the method family member. Parameter shape determines which
argument lanes the exact implementation accepts. The implementation is the
compiled or native behavior. The lexical super anchor is the defining holder,
not the receiver supplied later. Access authority travels with the reified
method and is checked before execution.

## Receiver validation and execution

The conceptual public operations are:

```phalcom
method.bind(receiver)
method.invokeOn(receiver, ***arguments)
```

Both validate receiver compatibility before execution. A receiver must belong
to the holder or to an allowed subclass. A class-side Method uses analogous
metaclass ancestry. A holderless public Method cannot be bound or invoked;
that category remains rejected unless a later specification creates a safe
holderless category.

`bind` produces a [`BoundMethod`](bound-method.md), pairing this exact Method
with the validated receiver. `invokeOn` executes this exact reified Method and
does not redispatch its selector. Ordinary sends inside the Method still
dynamically dispatch on the supplied receiver. `super` remains anchored to the
Method's defining holder.

## Arguments

Phalcom uses one rest/spread notation everywhere:

```text
*      positional rest/spread
**     labeled rest/spread
***    complete rest/spread
```

Examples:

```phalcom
method(*rest) { body }
method(**rest) { body }
method(***rest) { body }
method.invokeOn(receiver, ***arguments)
```

The spelling `args...` is never rest/spread syntax in Phalcom. `...` is not a
spread operator.

Method parameter shapes may support fixed lanes and the ratified positional,
labeled, or complete rest modes. Matching is shape-based. Exact selector
resolution precedes rest-family resolution; wildcard selector-string parsing
is not the semantic mechanism.

## Returns and constructors

A normal Method returns its final expression. An empty body and bare `return`
produce `()`. `return value` returns from this Method activation only. `None`
continues to mean absence, not no-result.

An `@constructor` declaration generates a class-side factory that allocates an
instance, calls a generated or hidden initializer as an ordinary Method,
ignores the initializer result, and returns the allocated instance. The
initializer remains an ordinary Method at runtime. The compiler nevertheless
rejects `return value` in that constructor initializer; bare `return` is
allowed and returns Unit from the initializer. No special constructor return
opcode exists.

## Related callable types

- [`Callable model`](README.md) — hierarchy, `self`, returns, and constructors
- [`Function`](function.md) — complete callable protocol
- [`Closure`](closure.md) — lexical executable value
- [`BoundMethod`](bound-method.md) — result of validated binding

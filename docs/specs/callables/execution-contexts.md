# Execution Contexts

> **Status:** Draft normative specification  
> **Semantic ownership:** callable execution context: lexical environment, dynamic `self`, lexical `super` anchor, callee access authority, local `return`, and the relation between Method and Closure activations.  
> **Related specifications:** [Dispatch](dispatch.md), [Methods](method.md), [Bound Methods](bound-method.md), [Closures](closure.md), [Functions](functions.md).

An activation executes inside a context containing the information required to interpret receiver-relative and lexical operations.

Conceptually, a callable execution context may contain:

```text
current callable
dynamic self, where applicable
lexical environment
lexical Method holder / super anchor, where applicable
callee access authority
local return destination
```

These properties must not be collapsed into one dynamic receiver concept. In particular, `self` is dynamic while `super` and lexical access authority originate from the defining callable context.

## Method activation context

A Method activation receives `self` from the receiver on which that Method is activated.

For ordinary dispatch:

```phalcom
receiver.operation(...)
```

the selected Method executes with:

```text
self = receiver
```

For exact Method invocation:

```phalcom
method.invokeOn(receiver, ***arguments)
```

the exact Method still executes with:

```text
self = receiver
```

For a BoundMethod:

```phalcom
bound(...)
```

the stored receiver becomes `self`.

The entry route may be ordinary dispatch, exact invocation, or a stored BoundMethod, but ordinary sends performed inside the Method dispatch dynamically on the current `self`.

## `self` is not an explicit call argument

`self` does not occupy an ordinary positional or labeled argument slot.

It is supplied by Method activation.

Consequently, Method parameter shape describes only explicit call arguments.

A Closure may capture an existing `self`, but that capture is lexical environment state rather than an explicit Closure parameter.

## Lexical `super`

`super` is anchored to the lexically defining Method holder.

It does not mean:

```text
parent of self's runtime class
```

and it does not manufacture a parent instance.

A `super` send keeps the current dynamic `self` and changes only the lookup origin, as defined by `DSP-SUPER-RECEIVER`.

This remains true under exact invocation and binding.

If a Method defined by `Base` is invoked exactly on a `Derived` receiver:

```text
entry Method
    Base's exact Method

self
    supplied Derived receiver

ordinary sends
    dynamic on Derived

super sends
    begin above lexical Base
```

Changing the receiver therefore does not change the Method's lexical super anchor.

## Lexical access authority

Method execution carries the lexical access authority associated with that Method's definition.

Two access contexts must be kept distinct:

```text
caller authority
    may the initiating caller enter this Method?

callee authority
    what private/protected/internal behavior may this Method body access?
```

Dispatch and reflective gateways use the caller authority for entry checks.

After successful activation, sends performed by the Method body use the Method's lexical callee authority.

Forwarding primitives and callable gateways must not accidentally substitute their own implementation authority for either context.

## Closure activation context

A Closure carries the lexical environment captured when the Closure value was created.

That environment may include:

```text
referenced lexical bindings
shared mutable binding cells
module/lexical context
captured self
the lexical super context inherited from the defining Method scope
```

A Closure is therefore complete without a caller-supplied receiver.

Its explicit invocation inputs are only its declared arguments.

When a Closure captures `self`, that value remains available after the activation that created the Closure has returned.

## Closure `super`

A Closure does not establish an independent Method holder merely because it is callable.

A `super` send lexically nested inside a Method remains anchored to the relevant defining Method context even when executed later through a captured Closure.

Capturing `self` does not move that anchor.

A Closure created outside any Method context has no Method super anchor unless another language construct explicitly establishes one.

## Local `return`

`return` is local to the currently executing Method or Closure activation.

```phalcom
return
```

is equivalent to:

```phalcom
return ()
```

and:

```phalcom
return value
```

exits the current callable activation with `value`.

A Closure `return` never implicitly returns from the Method or Closure that created it.

For example:

```phalcom
make() {
  const f = || {
    return 10
  }

  f()
  20
}
```

the Closure call returns `10`, while `make()` continues and returns `20`.

**Invariant — `EXEC-RETURN-LOCAL`**

> `return` exits only the currently executing Method or Closure activation; Phalcom has no implicit non-local return across callable boundaries.

This rule keeps escaping Closures valid regardless of whether their creating activation still exists.

## Normal body completion

The exact rules for expression results belong to the specifications that own those expression forms.

At the callable boundary:

- a body that completes normally produces its semantic final value;
- an empty callable body produces `Unit`;
- an explicit bare `return` produces `Unit`;
- a raised error or another abnormal control transfer does not produce a normal return value.

Setter assignment syntax may discard the callable result and produce `Unit` at the source-expression level under `DSP-ASSIGNMENT-UNIT`.

## Closure boundaries and lexical control flow

A real Closure creates a new callable activation.

Lexical control-flow constructs inside that Closure cannot target loops or other lexical control scopes outside the Closure activation.

For example, a `break` inside a nested Closure cannot break a loop executing in the creating Method.

The detailed semantics of loops and `break` belong to the control-flow specification; this chapter owns only the callable-boundary rule.

## Constructors and execution contexts

Constructor initializer bodies execute using ordinary Method execution semantics.

They therefore receive normal Method `self`, lexical `super`, access authority, and local return semantics.

Object-construction rules may impose additional source restrictions or may discard an initializer's normal Method result when producing the constructed instance.

Those rules belong to the class/constructor specification rather than redefining Method activation here.

## Invariants defined by this specification

**`EXEC-RETURN-LOCAL`**

> `return` exits only the currently executing Method or Closure activation; Phalcom has no implicit non-local return across callable boundaries.

The dynamic-receiver/lexical-super distinction remains owned jointly with `DSP-SUPER-RECEIVER`.

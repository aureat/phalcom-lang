# Bound Methods

> **Status:** Draft normative specification  
> **Semantic ownership:** reusable exact Method binding: construction, stored receiver, exact activation, mutation behavior, access, and relation to Method and Family.  
> **Related specifications:** [Methods](method.md), [Functions](functions.md), [Dispatch](dispatch.md), [Execution Contexts](execution-contexts.md), [References and Families](references-and-families.md).

A `BoundMethod` is a complete Function consisting of:

```text
one exact Method
+
one stored receiver
```

It is the reusable form of exact Method activation.

It is not a live selector reference and does not perform receiver-side entry lookup when called.

## Construction

```phalcom
const bound = method.bind(receiver)
```

creates a BoundMethod retaining:

```text
exact Method identity
receiver value
```

The receiver is captured as the value supplied to `bind`.

Binding does not invoke the Method.

Binding also does not perform ordinary selector lookup on the receiver.

As specified by [Methods](method.md), the receiver may be any runtime value; holder-layout compatibility is enforced only by body operations that actually require holder-specific representation.

## Application

A BoundMethod is a `Function`, so ordinary application uses the Function call protocol:

```phalcom
bound(arguments)
```

The incoming complete shape is transported through the common Function gateway.

The BoundMethod then:

- validates the shape against its stored exact Method;
- checks entry access using the activation caller's authority;
- installs the stored receiver as `self`;
- executes the stored exact Method.

The entry selector is not redispatched on the stored receiver.

## Stable exact Method identity

If behavior associated with the same selector is later replaced on the receiver's class, an existing BoundMethod still refers to the exact Method captured at binding time.

**Invariant — `BND-EXACT-STABLE`**

> A BoundMethod retains the exact Method captured by `bind`; later method-table mutation does not replace its entry Method.

This is the principal semantic distinction from a bound `Family`.

A Family stores selector capability and performs live lookup when activated.

A BoundMethod stores an already selected Method.

## Dynamic sends inside the Method

Although entry behavior is frozen, ordinary sends performed by the stored Method body remain dynamically dispatched on the stored receiver.

Thus:

```text
entry Method
    stable exact Method

self
    stable captured receiver

ordinary sends in body
    dynamic on that receiver's current behavior

super
    lexical to stored Method holder
```

The BoundMethod does not freeze the receiver's entire behavioral universe.

## Access authority

Binding does not capture elevated caller privilege for later invocation.

When the BoundMethod is called, the caller of that activation must be authorized to enter the stored Method.

After entry, the Method executes with its own lexical callee authority.

This prevents a BoundMethod from becoming an unintended visibility bypass merely because it was created in a privileged scope.

## Equivalence with exact invocation

For the same Method, receiver, argument shape, and caller authority:

```phalcom
method.invokeOn(receiver, ***arguments)
```

and:

```phalcom
method.bind(receiver)(***arguments)
```

perform the same exact Method entry semantics.

The difference is lifetime:

```text
invokeOn
    supplies receiver for one activation

bind
    produces a reusable Function retaining receiver
```

## No implicit rebinding

Calling a BoundMethod with another value does not replace its stored receiver.

To create the same Method bound to another receiver, bind the Method again:

```phalcom
const other = method.bind(otherReceiver)
```

Any convenience API for explicit rebinding would need separate specification; ordinary Function application does not imply it.

## BoundMethod is not Family

These values can sometimes produce the same immediate result but have different mutation semantics:

```phalcom
const exact = method.bind(object)
const live = &object.render(_)
```

`exact` stores a concrete Method.

`live` stores selector capability and performs current lookup.

After replacing `render(_)`:

```text
exact(...)
    old captured Method

live(...)
    current selected Method
```

subject to access and shape acceptance.

## Invariants defined by this specification

**`BND-EXACT-STABLE`**

> A BoundMethod retains the exact Method captured by `bind`; later method-table mutation does not replace its entry Method.

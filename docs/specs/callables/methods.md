# Methods

> **Status:** Draft normative specification  
> **Semantic ownership:** reified exact behavior, Method identity, exact invocation, binding, receiver semantics, lexical ownership, access, and Method-specific reflection.  
> **Related specifications:** [Selectors](selectors.md), [Arguments and Parameters](arguments.md), [Dispatch](dispatch.md), [Execution Contexts](execution-contexts.md), [Bound Methods](bound-method.md), [Reflection](reflection.md), [Functions](functions.md).

A `Method` is one reified exact behavior owned by a behavioral definition context.

A Method is not a `Function`.

It contains executable behavior, but it is incomplete until a receiver is supplied.

Conceptually:

```text
Method
    exact behavior
    + lexical identity
    + parameter shape
    - receiver
```

A `BoundMethod` completes that missing receiver.

## Method semantic identity

A Method has observable semantic properties including:

```text
holder
exact selector
parameter shape
implementation
visibility
lexical access authority
lexical super anchor
```

The representation of those properties is implementation-private, but their semantic effects are not.

### Holder

The holder is the class, metaclass, or other behavior that lexically owns the Method.

It anchors:

```text
super lookup
lexical access authority
holder-specific representation assumptions, when present
```

### Selector

A Method has one selector identity as defined by [Selectors](selectors.md).

For a fixed Method, that selector identifies the exact behavior stored in the holder's behavior table.

Rest-capable Method declarations additionally have parameter-shape metadata used only after exact lookup has missed and rest-family resolution begins.

A rest-capable Method is not an infinite collection of exact selector objects.

### Parameter shape

The Method's parameter shape defines which incoming argument shapes it accepts and how they bind.

Parameter shape is distinct from selector identity and from runtime value types.

See [Arguments and Parameters](arguments.md).

## Method is incomplete by design

An unbound Method cannot be invoked through ordinary Function application because it lacks a receiver.

The supported receiver-supplying operations are:

```phalcom
method.invokeOn(receiver, ***arguments)
method.bind(receiver)
```

The first is one-shot exact invocation.

The second produces a reusable [BoundMethod](bound-method.md).

Neither operation redispatches the Method selector to choose a different entry Method.

## Exact invocation

```phalcom
method.invokeOn(receiver, ***arguments)
```

executes `method` itself.

It does not mean:

```phalcom
receiver.perform(method.selector, ***arguments)
```

and does not restart ordinary selector lookup on `receiver`.

Exact invocation:

- uses the supplied `receiver` as `self`;
- checks access using the initiating caller's authority;
- validates the supplied complete argument shape against the exact Method;
- enters the exact Method implementation;
- preserves the Method's lexical `super` anchor and lexical access authority.

**Invariant — `MTH-EXACT-ENTRY`**

> Exact Method invocation executes the reified Method itself and never reselects the entry behavior by sending its selector to the supplied receiver.

## Arbitrary receivers

Exact Method invocation does not require the supplied receiver to be nominally an instance of the Method holder or its subclass before activation begins.

Likewise, binding does not impose an eager holder/subclass restriction.

This allows reflective Method reuse with values whose behavior is compatible with the Method body.

However, a bytecode Method may perform operations whose representation assumes the holder's instance or class layout.

When such an operation is reached with an incompatible receiver representation, execution fails with the language's representation/layout incompatibility error before the invalid slot operation occurs.

Primitive/native Methods do not acquire an artificial holder-layout restriction merely because their holder is known.

This separates:

```text
may this exact Method be activated with this receiver?
```

from:

```text
does this particular body operation require a holder-compatible representation?
```

## Exact entry does not freeze internal sends

Fixing the entry Method does not make every send inside that Method statically bound.

For an exact Method defined by `Base` and invoked on a `Derived` receiver:

```text
entry Method
    exact Base Method

self
    supplied Derived receiver

ordinary sends inside body
    dynamic dispatch from Derived runtime behavior

super sends
    lookup above lexical Base
```

The receiver remains dynamic even though entry behavior is exact.

## Lexical identity is stable

Binding or exact invocation changes the dynamic receiver but does not rewrite:

```text
holder
lexical super anchor
callee access authority
```

**Invariant — `MTH-LEXICAL-IDENTITY`**

> Supplying a receiver to a Method changes dynamic `self` but does not change the Method's lexical holder, super anchor, or callee access authority.

This rule is required for predictable `super` and visibility semantics under reflection.

## Binding

```phalcom
const bound = method.bind(receiver)
```

creates a `BoundMethod` containing the exact Method and the supplied receiver.

Binding does not:

- clone the Method;
- synthesize a Closure;
- perform selector lookup on the receiver;
- capture a replacement Method;
- grant the receiver or later caller additional access authority.

The semantics of later application belong to [Bound Methods](bound-method.md).

## Binding and access authority

Binding itself is not a privilege-capture operation.

The ability to create a BoundMethod does not permanently store the binder's caller authority for later target entry.

When the BoundMethod is eventually activated, access to the exact stored Method is checked according to the activation caller's authority.

After entry succeeds, execution uses the Method's lexical callee authority.

This is the same caller/callee split used by ordinary dispatch.

## Native and source Methods

A Method may be implemented by source/bytecode behavior, a native primitive, generated behavior, or another implementation category defined by the language/runtime.

Implementation kind must not alter the core Method laws:

```text
exact selector identity
parameter-shape acceptance
entry access checking
dynamic self
lexical super
lexical callee authority
```

Native implementation is therefore a property of the Method implementation, not a different callable class.

Where reflection exposes implementation kind, it must not change invocation semantics.

## Method reflection surface

A Method may expose reflective properties such as:

```phalcom
method.selector
method.holder
method.isNative
method.implementationKind
```

The exact public reflection API is specified in [Reflection](reflection.md).

A scalar legacy notion such as `arity` must not be treated as the complete Method shape model when labels or rest parameters are present.

## Construction

Ordinary user code cannot manufacture arbitrary Method instances through generic construction.

Methods arise from behavior declarations, generated language behavior, native registration, or reflection over existing behavior.

If the runtime contains holderless Method representations for bootstrap or internal purposes, those representations do not automatically form a public source-level Method category. Public holderless Method semantics require an explicit specification.

## Constructors

A source constructor initializer may compile to or be represented by an ordinary Method.

That Method follows ordinary Method execution rules.

Any generated factory that allocates an instance, invokes the initializer, discards the initializer's own result, or restricts constructor-source `return` forms belongs to the constructor/class specification.

## Invariants defined by this specification

**`MTH-EXACT-ENTRY`**

> Exact Method invocation executes the reified Method itself and never reselects the entry behavior by sending its selector to the supplied receiver.

**`MTH-LEXICAL-IDENTITY`**

> Supplying a receiver to a Method changes dynamic `self` but does not change the Method's lexical holder, super anchor, or callee access authority.

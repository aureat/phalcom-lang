# Functions

> **Status:** Draft normative specification  
> **Semantic ownership:** value application, the open `call` protocol, the abstract `Function` runtime hierarchy, the complete Function call gateway, concrete Function categories, `callWith`, and the distinction between complete callable values and reified behavior that still requires a receiver.  
> **Related specifications:** [Arguments and Parameters](arguments.md), [Dispatch](dispatch.md), [Methods](methods.md), [References](references.md), [Families](families.md).

Phalcom distinguishes **callability** from the runtime class `Function`.

Any value may participate in application syntax by responding to the appropriate `call` operation. This callable protocol is open.

`Function`, by contrast, is the abstract core class of first-class callable values whose execution context is already complete except for the arguments supplied by the caller. Its concrete runtime hierarchy is closed.

This distinction allows ordinary user objects to be callable without pretending that every object with a `call` Method is one of Phalcom's core Function representations.

## Value application

Once an expression has resolved to a value, applying that value:

```phalcom
f(argument, label: value)
```

performs the corresponding `call` operation:

```phalcom
f.call(argument, label: value)
```

with the same argument shape.

The exact Method selector in this example is:

```text
call(_,label)
```

as defined by [Selectors](selectors.md).

**Invariant — `FUN-APPLICATION-CALL`**

> Applying a resolved value with `(...)` performs the corresponding `call` operation with the same argument shape.

Application is therefore part of ordinary message semantics, not a second independent invocation language.

The compiler may optimize value application, but an optimization must remain observationally equivalent to the `call` operation with respect to:

- receiver evaluation;
- argument evaluation;
- selector shape;
- access;
- lookup;
- errors;
- result.

### Unqualified names are resolved before value application

The source form:

```phalcom
helper(value)
```

does not by spelling alone prove that `helper` is a value.

Name resolution may resolve `helper` to a lexical or module value, in which case value application applies.

Where no value binding resolves and the surrounding context provides an implicit receiver, the expression may instead denote an implicit receiver Method send.

That distinction belongs to [Dispatch](dispatch.md).

`FUN-APPLICATION-CALL` applies after the callee expression has resolved as a value.

## The open `call` protocol

An object does not need to inherit `Function` in order to be callable through ordinary application syntax.

For example:

```phalcom
class Counter {
  call(_ amount) {
    _value = _value + amount
    _value
  }
}

const counter = Counter.new()
const result = counter(2)
```

is ordinary dispatch to:

```text
call(_)
```

on `counter`.

`Counter` remains an ordinary user-defined class.

Defining `call`:

- does not make `Counter` a subclass of `Function`;
- does not turn its instances into core Function representations;
- does not grant Function-specific routing semantics;
- does not prevent the class from defining whatever exact or rest-capable `call` Methods ordinary Method rules permit.

The callable protocol is therefore open even though the `Function` hierarchy is closed.

### Callability is shape-specific

A value is not simply “callable” in the sense of accepting every possible invocation.

An object may answer:

```text
call(_)
```

but not:

```text
call()
call(_,debug)
```

Whether an application succeeds depends on ordinary selector dispatch and, where applicable, rest-capable behavior.

Likewise, a core Function may transport a shape through its common gateway yet reject that shape according to its concrete callable semantics.

## Function as a complete callable

A `Function` is a first-class callable value that already contains the execution context required to run its callable behavior apart from the explicit arguments supplied by the caller.

The concrete Function kinds are:

```text
Closure
BoundMethod
Family
BoundMethodFamily
```

Each is complete for a different reason:

```text
Closure
    executable code + lexical environment

BoundMethod
    exact Method + stored receiver

Family
    selector capability + bound/associated target context

BoundMethodFamily
    immutable MethodFamily snapshot + stored receiver
```

`Function` itself is abstract and cannot be directly instantiated.

The concrete kinds are core runtime classes rather than an open user-extensible representation family.

## Function hierarchy

The language-level callable object hierarchy is:

```text
Object
├── Method
├── MethodFamily
└── Function
    ├── Closure
    ├── BoundMethod
    ├── Family
    └── BoundMethodFamily
```

`Function` is abstract.

`Closure`, `BoundMethod`, `Family`, and `BoundMethodFamily` are the concrete core Function categories.

`Method` and `MethodFamily` deliberately remain outside `Function`.

The hierarchy expresses completeness, not merely “contains executable code.”

## Method is not a Function

A `Method` represents one concrete piece of behavior associated with a selector and lexical holder.

It is not a complete callable value because it does not by itself supply the receiver that will become `self`.

Conceptually:

```text
Method
    exact behavior
    + no bound receiver
```

A Method can become executable in a complete receiver context through APIs such as:

```phalcom
method.bind(receiver)
```

which produces a `BoundMethod`, or:

```phalcom
method.invokeOn(receiver, ***arguments)
```

which performs exact Method invocation with an explicitly supplied receiver.

The detailed semantics of Method binding and exact invocation belong to the Method specification.

Calling an unbound Method as though it were an ordinary complete Function is invalid.

## MethodFamily is not a Function

A `MethodFamily` is an immutable reflective snapshot of concrete Methods matching a reflective selector query.

It lacks a receiver and is therefore not a complete Function.

Binding it:

```phalcom
snapshot.bind(receiver)
```

produces a `BoundMethodFamily`.

This gives the conceptual symmetry:

```text
Method
    + receiver
    → BoundMethod

MethodFamily
    + receiver
    → BoundMethodFamily
```

A `Family` is different. It is produced by callable-reference semantics and represents a live selector capability rather than an immutable reflected Method snapshot. See [Families](families.md).

## Closure

A `Closure` is a complete Function because it contains executable code together with the lexical context captured when the Closure value was created.

Its explicit call arguments are the only invocation inputs not already retained by the Closure value.

Closure-specific semantics include:

- lexical capture;
- Closure parameter restrictions;
- positional rest;
- `self` capture;
- return behavior.

Those belong to the Closure specification.

At the Function level, the important fact is that a Closure is a complete callable representation and therefore participates in the common Function application protocol.

## BoundMethod

A `BoundMethod` combines:

```text
one exact Method
one receiver
```

The Method has already been selected.

Application of a BoundMethod validates and binds the incoming argument shape against that exact Method and executes it with the stored receiver.

It does not redispatch the Method selector merely because another implementation of that selector may now exist on the receiver.

Ordinary sends made *inside* the Method remain dynamically dispatched in the usual way.

This contrasts with `Family`, whose live selector capability performs lookup when activated.

## Family

A `Family` is a Function produced by callable-reference semantics.

It stores an exact selector or selector pattern together with its bound receiver or associated context and routes the requested operation when activated.

For bound Families, lookup remains live under `FAM-BOUND-LOOKUP-LIVE`.

Family can therefore be complete as a callable value without containing one frozen Method.

Getter, Setter, Method, and subscript Family activation are specified in [Families](families.md).

## BoundMethodFamily

A `BoundMethodFamily` combines:

```text
an immutable MethodFamily snapshot
a receiver
```

and is therefore a complete Function.

Unlike a live `Family`, it routes only among the concrete Methods represented by its captured MethodFamily snapshot. Later receiver method-table changes do not turn the snapshot into a different reflected family.

The MethodFamily/BoundMethodFamily reflection surface belongs to the Method/reflection specifications; this document establishes why the bound form belongs to `Function`.

## The common Function call gateway

All concrete Function kinds share one complete-shape call gateway:

```phalcom
call(***arguments)
```

This is a rest-capable Method on the abstract `Function` root.

Ordinary value application still produces its natural exact `call` selector.

For example:

```phalcom
f()
f(value)
f(to: destination)
f(value, to: destination)
```

request respectively:

```text
call()
call(_)
call(to)
call(_,to)
```

Ordinary dispatch first performs exact lookup across the eligible hierarchy.

When no exact `call` implementation handles that shape, the Function root's complete-rest:

```text
call(***)
```

accepts the complete shape under `DSP-EXACT-BEFORE-REST` and enters the common Function gateway.

The gateway then delegates to the concrete Function semantics.

This design means Function descendants do not require a synthesized finite set of:

```text
call()
call(_)
call(_,_)
call(_,label)
...
```

Methods merely to receive arbitrary call shapes.

### The Function call family is final

The common `call` routing family is part of the sealed Function abstraction.

Concrete Function descendants do not replace it with their own competing `call` overload family.

Their callable differences are expressed by the semantics of their concrete Function representation after the common gateway has transported the call shape.

This finality applies to the core Function hierarchy.

It does not restrict an ordinary user class outside `Function` from defining its own `call` Methods as part of the open call protocol.

## Transport is not acceptance

The Function gateway can receive a complete argument shape without promising that the concrete Function accepts that shape.

**Invariant — `FUN-TRANSPORT-NOT-ACCEPTANCE`**

> Reaching the common `Function#call(***arguments)` gateway transports the complete argument shape; it does not imply that the concrete Function accepts that shape.

For example, a Closure with one positional parameter:

```phalcom
const f = |value| { value }
```

may receive a labeled call shape through the common Function gateway, but the Closure parameter model rejects a non-empty labeled lane.

Similarly:

- a BoundMethod accepts only shapes permitted by its exact Method;
- an exact Family accepts only the exact selector capability it retains;
- a pattern Family accepts only candidate selectors matching its pattern;
- a BoundMethodFamily routes only through its captured snapshot.

Transport and acceptance are therefore separate semantic stages.

The argument-shape model and callable-specific parameter acceptance are defined in [Arguments and Parameters](arguments.md).

## Complete forwarding with `callWith`

`Function` provides complete-product forwarding:

```phalcom
function.callWith(arguments)
```

with the semantic equivalence:

```phalcom
function.callWith(arguments)
```

≡

```phalcom
function(***arguments)
```

`arguments` must be a complete argument product as defined by [Arguments and Parameters](arguments.md):

```text
Unit
    empty complete shape

Tuple
    complete positional + labeled shape
```

`callWith` is not a List-based positional calling convention.

It does not:

- flatten labels;
- reorder labels;
- invent a second parameter binder;
- bypass concrete Function acceptance;
- bypass ordinary Function result/error semantics.

It is a convenience surface for feeding an already-materialized complete product through the same Function application machinery.

### `callWith` belongs to Function

The canonical `callWith` operation is part of the Function abstraction.

An ordinary user object that merely defines `call` does not automatically acquire `Function#callWith`.

Such a class may define its own operation named `callWith`, but that is ordinary user behavior rather than inheritance of the core Function gateway.

## Argument shape through Function application

Function application preserves the complete argument shape defined in `arguments.md`.

For:

```phalcom
f(1, to: point, debug: true)
```

the Function receives structurally:

```text
positionals:
    1

labeled:
    to: point
    debug: true
```

A Function gateway must not flatten that into:

```text
[1, point, true]
```

or erase:

```text
to
debug
```

because concrete Function acceptance may depend on those labels.

`ARG-COMPLETE-SHAPE-PRESERVED` therefore applies to Function transport.

## Concrete acceptance

After common Function transport, each concrete Function kind applies the acceptance rules appropriate to what it represents.

### Closure

Closure acceptance is defined by its Closure parameter shape.

Current Closure parameters are positional-only, with optional terminal positional rest.

A non-empty labeled lane is rejected.

### BoundMethod

BoundMethod acceptance is the underlying exact Method's parameter acceptance.

The Method identity is already selected, so acceptance does not choose another Method by runtime value type.

### Family

Family first applies the exact selector or selector-pattern capability retained by the Family.

If admitted, it routes the resulting operation according to [Families](families.md).

### BoundMethodFamily

BoundMethodFamily selects only among the concrete reflected Methods in its snapshot according to the snapshot's routing semantics.

It does not replace that snapshot with a live capture of the receiver's current family.

## Function results

A successful ordinary Function application evaluates to the result produced by the concrete callable activation.

For:

```phalcom
const result = f(arguments)
```

`result` is the value returned by `f`'s activation if execution completes normally.

If the callable raises or otherwise transfers control abnormally, that transfer propagates instead of producing a normal result.

Source constructs with explicitly different result semantics remain governed by their own specifications. In particular, assignment syntax produces `Unit` under `DSP-ASSIGNMENT-UNIT` even when its underlying Setter Method has another callable return contract.

## Function is abstract and its representation hierarchy is closed

`Function` cannot be directly instantiated.

User code does not extend the set of core Function representation classes by subclassing `Function`.

The concrete core categories:

```text
Closure
BoundMethod
Family
BoundMethodFamily
```

are the language-defined complete Function representations.

This closure of the representation hierarchy does not make callability closed.

User code remains free to implement the open `call` protocol on ordinary classes.

The distinction is:

```text
Function representation hierarchy
    closed

call message protocol
    open
```

This allows the language to give the core Function categories precise semantics without reserving application syntax exclusively for them.

## Function and static callable types are different concepts

The runtime class `Function` does not imply one universal static callable signature.

Different Function values may accept different:

```text
positional counts
label sequences
rest shapes
selector capabilities
return types
generic environments
```

Likewise, an ordinary object may be statically known to answer a particular `call` selector without being an instance of runtime class `Function`.

Callable type formation, variance, generic inference, higher-rank behavior, and Family callable typing belong to the type-system specification.

This document defines runtime/value application semantics, not a universal function type.

## Reflection boundary

The fundamental Function contract specified here is:

```text
value application through call
complete-shape Function gateway
complete forwarding through callWith
concrete Function acceptance
```

A single scalar `arity` is not sufficient to describe every Function capability:

- labeled shapes make total argument count insufficient;
- rest-capable callables accept multiple shapes;
- pattern Families can route multiple selectors;
- BoundMethodFamily can represent multiple captured Methods.

Likewise, a generic scalar `name` does not define callable identity for every Function category.

Any legacy or convenience reflective accessors named `arity` or `name` must therefore not be used as the semantic definition of Function callability or shape acceptance.

Structured callable reflection belongs to the reflection and Method specifications.

## Function access and authority

The common Function gateway is routing machinery, not an authority boundary that grants new privileges.

Where a concrete Function ultimately enters a Method or target send, the access rules of [Dispatch](dispatch.md) apply.

In particular:

- BoundMethod activation must respect the access semantics of its exact Method;
- Family activation preserves caller authority as specified by [Families](families.md);
- BoundMethodFamily activation must not gain authority merely because the snapshot was captured elsewhere.

The gateway itself must not convert an access violation into a message miss.

## Calling non-Function values

Because application uses the open `call` protocol, a non-Function value can still be successfully applied when ordinary dispatch resolves the requested `call` operation.

Conversely, a value that is neither a Function nor an object whose behavior accepts the requested `call` selector cannot be applied successfully.

Application failure follows ordinary dispatch semantics, including exact/rest lookup and terminal message-miss behavior where applicable.

The error is not “not a Function” merely because the receiver's runtime class is outside the Function hierarchy; what matters for open protocol application is whether the requested `call` operation can be performed.

## Invariants defined by this specification

The following coded invariants are defined here because they establish the two central laws that other callable specifications rely upon.

**`FUN-APPLICATION-CALL`**

> Applying a resolved value with `(...)` performs the corresponding `call` operation with the same argument shape.

**`FUN-TRANSPORT-NOT-ACCEPTANCE`**

> Reaching the common `Function#call(***arguments)` gateway transports the complete argument shape; it does not imply that the concrete Function accepts that shape.

Argument transport remains governed by `ARG-COMPLETE-SHAPE-PRESERVED`; ordinary exact/rest lookup remains governed by `DSP-EXACT-BEFORE-REST`; concrete Family routing remains governed by the Family specification.

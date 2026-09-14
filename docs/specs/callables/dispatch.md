# Dispatch

> **Status:** Draft normative specification  
> **Semantic ownership:** resolution and performance of selector-identified operations: name-resolution boundary, receiver and operand evaluation, exact and rest lookup, access, misses, `doesNotUnderstand`, `super`, and ordinary dispatch on class objects.  
> **Related specifications:** [Selectors](selectors.md), [Arguments and Parameters](arguments.md), [Functions](functions.md), [Methods](method.md), [References and Families](references-and-families.md), [Execution Contexts](execution-contexts.md).

Dispatch answers one question:

> Given a receiver, an exact selector, and the values required by that operation, which behavior handles the operation and how is that behavior entered?

Selector identity is defined by [Selectors](selectors.md). Argument shape and expansion are defined by [Arguments and Parameters](arguments.md). Dispatch begins after those structures are known.

## Source operation and target resolution

An explicit receiver operation:

```phalcom
receiver.move(value, to: destination)
```

always dispatches on the value produced by `receiver`.

An unqualified call-like form:

```phalcom
helper(value)
```

is resolved before dispatch decides whether value application or an implicit receiver send is intended.

Resolution prefers a value binding in the applicable lexical/module environment. If a value binding resolves, the expression is value application and therefore uses the `call` protocol specified by [Functions](functions.md).

If no value binding resolves and the surrounding context admits an implicit receiver, the expression is an ordinary Method send to that receiver.

Thus:

```phalcom
class Example {
  helper() { 10 }

  useMethod() {
    helper()          // implicit self.helper()
  }

  useValue() {
    const helper = || { 20 }
    helper()          // value application
  }
}
```

A lexical value binding therefore shadows an implicit receiver Method of the same source name.

**Invariant — `DSP-NAME-RESOLUTION-BEFORE-APPLICATION`**

> Whether `name(...)` is value application or an implicit receiver send is determined by name resolution before callable dispatch begins.

## Evaluation order

For an explicit receiver operation, the receiver expression is evaluated first. Operand expressions are then evaluated exactly once in source order.

For:

```phalcom
receiver().send(first(), second(), debug: flag())
```

the observable order is:

```text
receiver()
first()
second()
flag()
dispatch and activation
```

Labeled-lane organization does not reorder expression evaluation.

For named assignment:

```phalcom
receiver().name = rhs()
```

the order is:

```text
receiver()
rhs()
Setter activation
```

For subscript assignment:

```phalcom
receiver()[row(), column(), debug: mode()] = rhs()
```

the order is:

```text
receiver()
row()
column()
mode()
rhs()
SubscriptSet activation
```

If evaluation of the receiver or an earlier operand does not complete normally, later operand expressions are not evaluated and target behavior is not activated.

**Invariant — `DSP-EVALUATION-ORDER`**

> An explicit receiver operation evaluates the receiver first and then each operand expression exactly once in source order before behavioral activation.

## Exact selector lookup

Ordinary dispatch starts with one exact selector.

Lookup begins at the runtime class of the receiver and proceeds through its superclass chain.

At each behavior, dispatch asks for the exact selector identity requested by the operation.

For example:

```phalcom
class Parent {
  render(_ value) { ... }
}

class Child is Parent {
  render(_ value, debug enabled) { ... }
}
```

a `Child` receiver still inherits:

```text
render(_)
```

because defining:

```text
render(_,debug)
```

does not replace another selector merely sharing the base name `render`.

Overriding is therefore per exact selector identity.

Likewise, these remain independent:

```text
value
value()
value=(_)
value(_)
```

Getter, nullary Method, Setter, and unary Method are not substitutes for one another.

## Exact lookup precedes rest lookup

Rest-capable Methods participate only after exact selector lookup has failed across the entire eligible hierarchy.

The lookup phases are:

```text
exact selector through receiver hierarchy
    ↓ complete miss
compatible rest-capable Method through the same hierarchy
    ↓ complete miss
terminal message miss
```

A compatible inherited exact Method therefore takes precedence over a rest-capable Method declared on a more-derived class.

**Invariant — `DSP-EXACT-BEFORE-REST`**

> Rest-capable lookup begins only after exact-selector lookup has failed across the complete eligible inheritance chain.

Once rest lookup begins, the nearest compatible rest-capable Method is selected according to the structural acceptance rules in [Arguments and Parameters](arguments.md).

Rest selection does not inspect runtime argument value types.

## Access is distinct from lookup failure

A Method may exist and be selected yet be inaccessible to the initiating caller.

That situation is an access violation, not a lookup miss.

Dispatch must not continue searching merely to evade the selected Method's visibility, and it must not invoke `doesNotUnderstand` as if the Method were absent.

**Invariant — `DSP-ACCESS-NOT-MISS`**

> Selecting inaccessible behavior produces an access violation; it must not be treated as an absent selector or forwarded to `doesNotUnderstand`.

Caller authority governs whether the selected Method may be entered. Once entered, the Method executes with its own lexical authority as described in [Execution Contexts](execution-contexts.md).

Forwarding gateways must preserve the original caller authority for the target access check.

## Named accessors and Methods remain distinct

A named read:

```phalcom
object.value
```

requests Getter selector:

```text
value
```

It does not fall back to:

```text
value()
```

A named assignment:

```phalcom
object.value = next
```

requests Setter selector:

```text
value=(_)
```

It does not fall back to unary Method:

```text
value(_)
```

The same distinction is preserved by callable references and Family activation.

The assigned value of Setter and SubscriptSet operations remains outside structural selector slots under `SEL-SETTER-VALUE-EXTERNAL`.

## Subscript dispatch

Subscript syntax is ordinary selector-based behavior, not a separate built-in collection lookup system.

```phalcom
object[key, default: fallback]
```

requests SubscriptGet:

```text
[_,default]
```

and:

```phalcom
object[key, default: fallback] = replacement
```

requests SubscriptSet:

```text
[_,default]=(_)
```

The receiver's hierarchy is resolved using the same exact-before-rest and access rules applicable to other selector kinds, subject to the declaration forms valid for subscript behavior.

Collection types and user-defined indexable objects participate by defining the corresponding selectors.

## Assignment result

The underlying Setter Method may have its own callable return contract.

Assignment syntax does not expose that Method return as the assignment expression's result.

After successful named or subscript Setter activation:

```phalcom
object.name = value
object[index] = value
```

the assignment expression evaluates to `Unit`.

If Setter execution does not complete normally, no `Unit` value is produced; the abnormal transfer propagates.

**Invariant — `DSP-ASSIGNMENT-UNIT`**

> After successful Setter or SubscriptSet activation, assignment syntax evaluates to `Unit` independently of the selected Setter Method's own return value.

## Lexical `super`

A `super` send keeps the current dynamic receiver but changes where lookup begins.

Inside a Method defined by `Child`:

```phalcom
super.render(value)
```

lookup begins above the Method's lexically defining holder rather than at the runtime class of `self`.

The selected ancestor Method still receives the original runtime receiver as `self`.

**Invariant — `DSP-SUPER-RECEIVER`**

> A lexical `super` send changes the lookup origin without replacing the runtime receiver.

The ordinary phase ordering still applies from the `super` origin:

```text
exact lookup
then rest lookup
then terminal miss
```

The lexical anchor belongs to the Method definition. Binding or exact-invoking that Method on another receiver does not move the anchor.

## Class objects use ordinary dispatch

A class is itself a runtime object and may receive ordinary messages.

```phalcom
Fiber.new()
```

is ordinary dot dispatch on the class object `Fiber`.

It is not Java-like static dispatch.

Associated lookup:

```phalcom
Option::Some(42)
```

is a different operation. `::` identifies behavior associated with a namespace-bearing owner and is not ordinary receiver-dot dispatch.

## Message misses

A terminal message miss occurs only after all applicable ordinary resolution phases have failed.

For ordinary Method dispatch:

```text
exact lookup misses
and
compatible rest lookup misses
```

An access violation is not a miss.

A Method that executes and raises is not a miss.

A Method that returns an ordinary sentinel value is not a miss unless another protocol explicitly defines that value as cooperative decline.

## `doesNotUnderstand`

A terminal ordinary message miss is forwarded to the receiver's `doesNotUnderstand(_)` behavior.

The forwarded message preserves the attempted operation:

- the exact selector;
- positional values;
- ordered labels and their values;
- for Setter/SubscriptSet, the assigned value as the assigned value of that operation.

The miss must not rewrite selector identity or invent a synthetic label for the Setter RHS.

`doesNotUnderstand` is a miss hook. It is not the implementation mechanism for:

```text
Family construction
Method reflection
MethodFamily capture
respondsTo / understands queries
exact Method invocation
```

**Invariant — `DSP-DNU-ONLY-ON-MISS`**

> `doesNotUnderstand` may be entered only for a genuine terminal message miss, never as an intentional reflection or callable-routing mechanism.

## Static and dynamically assembled sends

A call whose final argument shape is known statically and a call whose shape is assembled through `*`, `**`, or `***` enter the same dispatch model after shape construction succeeds.

If two operations have the same:

```text
receiver
exact selector
argument values
assigned value, where applicable
```

they follow the same lookup, access, rest, activation, and miss semantics regardless of how the call shape was constructed.

The implementation may use different internal machinery without changing dispatch behavior.

## Behavioral mutation

Ordinary dispatch observes the receiver's current behavior at the time the operation is performed.

Replacing, adding, or removing Methods may therefore affect later ordinary sends and live Family activation.

An implementation cache must be invalidated or versioned so that caching cannot make a previous Method or previous miss permanently observable after behavior changes.

How caches are represented belongs to runtime architecture.

## Invariants defined by this specification

**`DSP-NAME-RESOLUTION-BEFORE-APPLICATION`**

> Whether `name(...)` is value application or an implicit receiver send is determined by name resolution before callable dispatch begins.

**`DSP-EVALUATION-ORDER`**

> An explicit receiver operation evaluates the receiver first and then each operand expression exactly once in source order before behavioral activation.

**`DSP-EXACT-BEFORE-REST`**

> Rest-capable lookup begins only after exact-selector lookup has failed across the complete eligible inheritance chain.

**`DSP-ACCESS-NOT-MISS`**

> Selecting inaccessible behavior produces an access violation; it must not be treated as an absent selector or forwarded to `doesNotUnderstand`.

**`DSP-ASSIGNMENT-UNIT`**

> After successful Setter or SubscriptSet activation, assignment syntax evaluates to `Unit` independently of the selected Setter Method's own return value.

**`DSP-SUPER-RECEIVER`**

> A lexical `super` send changes the lookup origin without replacing the runtime receiver.

**`DSP-DNU-ONLY-ON-MISS`**

> `doesNotUnderstand` may be entered only for a genuine terminal message miss, never as an intentional reflection or callable-routing mechanism.

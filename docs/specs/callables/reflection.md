# Callable Reflection

> **Status:** Draft normative specification  
> **Semantic ownership:** structural callable reflection: exact Method probes, capability probes, Behavior extraction, MethodFamily snapshots, BoundMethodFamily, reflective dynamic sends, and the distinction between reflection and execution.  
> **Related specifications:** [Selectors](selectors.md), [Dispatch](dispatch.md), [Methods](method.md), [Bound Methods](bound-method.md), [References and Families](references-and-families.md), [Functions](functions.md).

Callable reflection observes or reifies existing behavior without changing the fundamental distinction among:

```text
exact Method
live Family
immutable MethodFamily snapshot
ordinary dynamic send
exact Method invocation
```

Reflection must not use `doesNotUnderstand` merely to answer structural questions.

## Exact reflection and dispatch capability are different questions

An exact selector can be used to ask:

> Is there an exact accessible Method with this selector?

A concrete incoming shape can also be used to ask:

> Would ordinary dispatch understand this operation, including rest fallback?

These questions are intentionally distinct.

An exact Method identity should not be silently replaced by a rest-capable Method merely because that rest Method could accept the same call shape.

## `methodFor`

`methodFor` is an exact reflection query.

Conceptually:

```phalcom
const method = object.methodFor(#render(_))
```

returns the accessible exact Method selected by ordinary exact lookup for that selector, or absence.

It does not invoke the Method.

It does not invoke `doesNotUnderstand`.

It does not substitute a compatible rest-capable Method when no exact Method with that selector exists.

This preserves the meaning of “Method for this exact selector.”

**Invariant — `REFL-METHODFOR-EXACT`**

> `methodFor(exactSelector)` reifies only an accessible exact Method for that selector; rest compatibility does not manufacture an exact Method result.

## `respondsTo`

`respondsTo(exactSelector)` asks whether the receiver has an accessible exact response for that selector.

It is an exact structural probe.

It does not execute the Method and does not invoke `doesNotUnderstand`.

Its visibility answer is relative to the initiating caller's authority.

## `understands`

`understands(concreteSelector)` asks the broader dispatch-capability question:

> Would ordinary selector resolution accept this concrete operation shape?

It therefore considers:

```text
exact selector lookup
then compatible rest-capable lookup
```

without executing the selected Method.

`understands` does not call `doesNotUnderstand` on a miss.

This gives Phalcom two deliberate capabilities:

```text
respondsTo
    exact accessible Method identity

understands
    ordinary structural dispatch capability
```

## Behavior extraction with `>>`

Behavior reflection may use:

```phalcom
Behavior >> selectorSpec
```

as ordinary operator dispatch on a Behavior receiver.

The right-hand selector specification determines the reflective result.

### Exact selector extraction

For an exact Selector or canonical exact selector Symbol:

```phalcom
Behavior >> #render(_)
```

returns the effective accessible exact Method selected for that Behavior, or absence.

The operation is reflection, not invocation.

### SelectorPattern extraction

For a SelectorPattern:

```phalcom
Behavior >> pattern
```

the result is an immutable `MethodFamily` snapshot containing the accessible effective routes matching that pattern at capture time.

A pattern-shaped Symbol may be decoded to the same selector-pattern meaning where the language accepts that representation.

`>>` on unrelated receiver classes remains ordinary operator behavior. The reflection meaning exists because Behavior defines the corresponding selector; operator punctuation does not create a global reflection intrinsic.

## MethodFamily

A `MethodFamily` is an immutable snapshot of Methods captured from one Behavior by one SelectorPattern at one moment.

It is not a `Function`.

It does not perform live lookup on a receiver.

Conceptually it contains:

```text
captured exact routes
captured rest-capable routes
selector-pattern context
capture order
```

subject to visibility filtering performed during capture.

After creation, later method-table mutation does not alter the snapshot.

**Invariant — `REFL-METHODFAMILY-SNAPSHOT`**

> A MethodFamily is immutable with respect to the Behavior state from which it was captured; later behavior mutation does not add, remove, or replace its captured routes.

## Visibility during snapshot capture

MethodFamily capture uses the initiating caller's authority.

Inaccessible routes are not made available merely because the capture is implemented by a privileged reflection primitive.

Reflection therefore preserves the ordinary visibility boundary.

The resulting snapshot contains only the routes visible under the capture operation's semantics.

## `MethodFamily.selectors`

`selectors` returns a new collection describing the exact selector identities of captured routes.

Mutating the returned collection must not mutate the MethodFamily snapshot.

The selector identities themselves are canonical selector values/symbols as defined by the selector reflection model.

The ordering of this collection follows the snapshot's defined capture order.

The capture order must be stable for a given snapshot; implementations must not return an arbitrary hash-table order if order is observable.

## `MethodFamily.size`

`size` reports the number of captured routes represented by the snapshot, including captured exact and rest-capable routes.

It does not recapture the source Behavior.

## `MethodFamily.methodFor`

```phalcom
snapshot.methodFor(selector)
```

returns the captured Method whose own exact selector identity equals `selector`, or absence.

It does not consult the bound receiver behavior and does not perform live recapture.

Because it returns a Method value capable of later exact activation, access to that captured Method is checked using the caller's current authority.

A snapshot does not permanently grant future callers the authority of the original capturing caller.

## `MethodFamily.bind`

```phalcom
const bound = snapshot.bind(receiver)
```

produces a `BoundMethodFamily`.

Binding:

- stores the snapshot;
- stores the receiver;
- does not inspect receiver behavior;
- does not recapture methods;
- does not replace snapshot routes with current receiver routes.

The supplied receiver may be arbitrary; representation-sensitive Method body operations remain subject to the same rules as exact Method activation.

## BoundMethodFamily

A `BoundMethodFamily` is:

```text
immutable MethodFamily snapshot
+
stored receiver
```

and is a `Function`.

When applied, it routes only among Methods captured by the snapshot.

It does not consult the receiver's current behavior table to discover new or replaced Methods.

This contrasts with `Family`, whose bound selector capability performs live lookup.

For accepted routes:

```text
select from snapshot
→ authorize selected captured Method for current caller
→ exact Method activation on stored receiver
```

Ordinary sends inside the selected Method remain dynamically dispatched on the receiver.

## Dynamic reflective send with `perform`

```phalcom
receiver.perform(selector, ***arguments)
```

performs an ordinary dynamic send identified by `selector`.

The leading selector value controls the operation; the remaining complete argument shape is preserved under `ARG-COMPLETE-SHAPE-PRESERVED`.

`perform` performs selector lookup on `receiver`.

It is therefore different from:

```phalcom
method.invokeOn(receiver, ***arguments)
```

which executes an already selected Method exactly.

It is also different from:

```phalcom
&receiver.selector...
```

which constructs a reusable Family capability.

A terminal `perform` miss is an ordinary message miss and may reach `doesNotUnderstand`.

## Exact Method invocation belongs to Method

Reflection may be the mechanism by which a Method value is obtained, but:

```phalcom
method.invokeOn(receiver, ***arguments)
method.bind(receiver)
```

are Method semantics and are defined by [Methods](method.md).

Reflection must not redefine those operations with different access, receiver, or shape rules.

## Method, BoundMethod, Family, MethodFamily, and BoundMethodFamily

The callable reflection tower is:

```text
Method
    one exact reified behavior; receiver missing

BoundMethod
    one exact Method + receiver; entry Method frozen

Family
    selector capability + target context; live lookup on activation

MethodFamily
    immutable reflected snapshot; receiver missing

BoundMethodFamily
    immutable MethodFamily snapshot + receiver; routes snapshot only
```

These are distinct observable values.

No one type should be described as an optimization of another.

## Construction policy

Ordinary generic construction must not fabricate reflection objects whose semantic identity requires language/runtime provenance.

In particular, direct generic construction of objects such as:

```text
Method
MethodFamily
BoundMethodFamily
SelectorPattern
```

is invalid unless another specification explicitly defines a safe public constructor.

Such values are normally obtained from declarations, reflection, binding, parsing/compiling selector specifications, or other designated language operations.

## Reflection identity and equality

Exact selector equality belongs to [Selectors](selectors.md).

Method identity represents one reified concrete behavior.

MethodFamily identity represents one snapshot object, not merely one selector pattern.

Two snapshots captured from the same Behavior with the same pattern at different times are not required to be the same object and may contain different Methods.

BoundMethodFamily identity additionally includes its stored receiver context.

Unless a general equality specification defines structural equality for these reflection objects, reflection APIs must not infer equality merely from matching selector text or matching route counts.

## Invariants defined by this specification

**`REFL-METHODFOR-EXACT`**

> `methodFor(exactSelector)` reifies only an accessible exact Method for that selector; rest compatibility does not manufacture an exact Method result.

**`REFL-METHODFAMILY-SNAPSHOT`**

> A MethodFamily is immutable with respect to the Behavior state from which it was captured; later behavior mutation does not add, remove, or replace its captured routes.

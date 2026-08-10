# Reflection and Universal Object Protocol

[← Calls and rest](03-calls-rest-and-spread.md) · [Overview](README.md) · [Runtime conformance →](05-runtime-conformance.md)

---

## 1. Reflection boundary

Structural method reflection belongs to `Behavior`, not to every object instance.

Dynamic execution against a particular receiver remains on `Object`.

This establishes the distinction:

```text
Behavior reflection
    asks about method structure / resolution for instances governed by a Behavior

Object execution
    sends or handles a message on one concrete receiver
```

---

## 2. Behavior protocol

The canonical reflective Behavior surface includes:

```text
name
superclass
methods
methodFor(_)
respondsTo(_)
```

`Class` and `Metaclass` participate through the existing Behavior hierarchy.

---

## 3. `Behavior#name`

`Behavior#name` returns the Behavior/class object's own name.

Object instances do not inherit a universal reflective `name`.

For an instance:

```phalcom
person.class.name
```

obtains the class name.

This leaves:

```phalcom
person.name
```

free for ordinary domain semantics.

---

## 4. `Behavior#methods`

`methods` enumerates Methods defined directly on that Behavior's own method dictionary.

It does not include inherited Methods.

Conceptually:

```phalcom
Dog.methods
```

lists direct instance-side methods installed on `Dog`.

```phalcom
Dog.class.methods
```

lists direct class-side/metaclass methods.

An inherited/all-methods view may be derived or specified separately; `methods` itself remains direct-only.

---

## 5. `Behavior#methodFor`

`methodFor(selector)` performs normal inherited method resolution for instances governed by that Behavior.

If:

```text
Dog < Animal
Animal defines speak
Dog does not
```

then:

```phalcom
Dog.methodFor(#speak())
```

resolves the inherited `Animal` Method that a Dog instance would execute.

`methodFor` therefore differs from `methods`:

```text
methods
    direct dictionary

methodFor
    resolved behavior
```

### 5.1 Rest resolution

Where the supplied reflective selector/shape participates in rest-family resolution, `methodFor` follows the same normal resolver:

```text
exact
then compatible rest
then absence
```

The exact details of future richer selector/family reflection may be extended separately, but `methodFor` must not use a reflection-only dispatch rule.

### 5.2 No dNU

`methodFor` is a lookup operation.

It does not invoke `doesNotUnderstand`.

---

## 6. `Behavior#respondsTo`

`respondsTo(selector)` answers whether normal Method resolution finds an accessible Method for instances governed by that Behavior.

It uses the same exact/rest resolution rules as ordinary Method lookup.

`doesNotUnderstand` does not make a Behavior respond to arbitrary selectors.

Thus proxying through dNU does not convert every miss into `respondsTo == true`.

---

## 7. Instance-side and class-side reflection

For a class object `Person`:

```phalcom
Person.name
Person.superclass
Person.methods
Person.methodFor(selector)
Person.respondsTo(selector)
```

describe the **instance-side** behavior of `Person`.

For the class side:

```phalcom
Person.class.name
Person.class.superclass
Person.class.methods
Person.class.methodFor(selector)
Person.class.respondsTo(selector)
```

operate on the metaclass/class-side behavior.

For an ordinary instance:

```phalcom
person.class.methodFor(selector)
```

is the bridge from the instance to its governing Behavior.

---

## 8. Object protocol retained

The universal Object surface retains:

```text
class
is(_)
isExactly(_)
perform(...)
doesNotUnderstand(_)
==
!=
hash
toString
```

This list describes the callable/reflection decisions in scope and does not forbid other separately specified universal Object behavior.

---

## 9. `Object#class`

Every surface value has a class.

```phalcom
value.class
```

returns that class/Behavior.

The setter family:

```text
class=(put)
```

is not part of the public Object protocol.

Read-only class identity is expressed by absence of a public setter, not by installing a setter whose only behavior is to throw.

---

## 10. `Object#is` and `Object#isExactly`

`is` and `isExactly` remain receiver-specific type/introspection operations.

`isA` is not retained as a compatibility alias.

---

## 11. `Object#perform`

`perform` remains on Object because it dynamically executes a message against a concrete receiver.

Conceptually:

```text
receiver.perform(...)
```

means dynamic dispatch on `receiver`.

Its argument transport must preserve complete argument shape where the API supplies such a pack.

`perform` is not structural method reflection and therefore does not move to Behavior.

---

## 12. `Object#doesNotUnderstand`

`doesNotUnderstand` remains the ordinary overridable message-miss hook on Object.

It is receiver-specific and may support proxies or other dynamic behavior.

It is not used as the normal implementation mechanism for:

- Function calls;
- BoundMethod calls;
- Family routing;
- `respondsTo`;
- `methodFor`.

A true Method-resolution miss may eventually forward to dNU. Reflection probes do not.

---

## 13. Retired Object reflection

The following universal Object surface is retired:

```text
Object#name
Object#methodFor
Object#respondsTo
Object#class=(put)
Object#isA
```

The corresponding intended forms are:

```phalcom
obj.class.name
obj.class.methodFor(selector)
obj.class.respondsTo(selector)
```

where structural reflection is desired.

---

## 14. Method reflection and invocation

A reified Method remains a distinct object from a Family reference.

Core Method operations in this specification include:

```phalcom
method.bind(receiver)
method.invokeOn(receiver, ***arguments)
```

A Method is exact behavior.

A Family is a callable reference that may perform lookup at call time.

The following conceptual distinction is normative:

```phalcom
const method = Person.methodFor(selector)
method.invokeOn(person, ***arguments)
```

executes the exact resolved Method.

By contrast:

```phalcom
const family = person::operation
family(...)
```

performs Family routing and an ordinary target send.

---

## 15. Callable base reflection

`Function` does not define universal scalar:

```text
arity
name
```

as part of this specification.

Rest parameters, labeled Method shapes, and Families make a single scalar `arity` an incomplete description.

A future reflection specification may define richer parameter-shape introspection without changing call semantics.

---

## 16. Access control and reflection

Visibility authorization remains part of Method invocation.

A reflective operation that returns or probes a Method must not silently redefine access semantics.

`methodFor`, `respondsTo`, exact invocation, normal dispatch, and BoundMethod invocation must use a consistent access model as specified by the language's visibility rules.

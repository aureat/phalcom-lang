# References

> **Status:** Draft normative specification  
> **Semantic ownership:** callable-reference expressions introduced by `&`: target selection, bound versus associated references, receiver evaluation, exact and pattern reference forms, and the relationship between selector specifications and the resulting callable capability.  
> **Related specifications:** [Selectors](selectors.md), [Dispatch](dispatch.md), [Families](families.md).

A **callable reference** names behavior without performing that behavior.

Phalcom writes a callable reference by prefixing a selector-bearing member form with `&`:

```phalcom
&object.render(_)
&object.render(...)
&object.render...

&object.name
&object.name=(_)
&object.name=

&object[_, debug]
&object[...]=(_)

&Option::Some(_)
&Option::Some...
&Option::None
```

The selector form written after the final member operator determines the exact selector or selector pattern being referenced according to [Selectors](selectors.md). The `&` changes the operation from *perform this selector now* to *produce a first-class capability representing this selector specification*.

The resulting first-class value is a `Family`. Its activation behavior is specified in [Families](families.md).

This specification defines reference construction. It does not define ordinary message dispatch and does not define the complete runtime API of the resulting `Family`.

## Referencing rather than performing

The ordinary expression:

```phalcom
object.render(value)
```

performs the Method selector:

```text
render(_)
```

on `object`.

The reference expression:

```phalcom
&object.render(_)
```

does not perform `render(_)`. It produces a capability that retains the receiver and exact selector so that the operation can be performed later.

Likewise:

```phalcom
object.name
```

performs the Getter:

```text
name
```

whereas:

```phalcom
&object.name
```

references that Getter.

Similarly:

```phalcom
object.name = value
```

performs:

```text
name=(_)
```

while:

```phalcom
&object.name=(_)
```

references the exact Setter.

The same relationship applies to subscript selectors:

```phalcom
object[key, debug: true]
&object[_, debug]

object[key, debug: true] = value
&object[_, debug]=(_)
```

The selector specification in a reference contains shape, not operand expressions. `_`, labels, `...`, and accessor-pattern punctuation describe the selector capability being referenced; they are not evaluated as ordinary arguments.

## Selector shape is preserved

A callable reference does not erase the selector distinctions established by [Selectors](selectors.md).

These references are therefore distinct:

```phalcom
&object.value
&object.value()
&object.value=(_)
&object.value(_)
```

They refer respectively to:

```text
value       Getter
value()     Method
value=(_)   Setter
value(_)    Method
```

The `&` operator does not infer a callable kind from arity and does not collapse Getter/Method or Setter/Method identities.

Likewise:

```phalcom
&object.value(...)
```

is Method-pattern-shaped, while:

```phalcom
&object.value...
```

references the complete named selector family for base `value`.

The difference originates in selector semantics and is preserved by reference construction.

## Bound references

A **bound reference** uses ordinary dot or subscript member syntax on a receiver expression:

```phalcom
&object.render(_)
&object.render...
&object.name
&object.name=
&object[...]
&object[...]=(_)
```

The receiver expression is evaluated when the reference is constructed and the resulting receiver is retained by the Family.

For example:

```phalcom
const renderer = &makeRenderer().render(_)
```

evaluates:

```phalcom
makeRenderer()
```

at reference construction time, exactly once.

It does not defer `makeRenderer()` until the Family is later called.

**Invariant — `REF-RECEIVER-ONCE`**

> A bound callable reference evaluates its receiver expression exactly once when the reference is constructed and retains that resulting receiver for later Family activation.

This applies equally when the receiver expression itself contains calls or other ordinary evaluation:

```phalcom
&Router.new().route(_)
```

means conceptually:

```phalcom
const receiver = Router.new()
const reference = &receiver.route(_)
```

It does not mean:

```text
take a reference to Router.new()
then send route(_) to that reference
```

## The final member is the reference target

A reference expression may contain an arbitrarily rich receiver expression before its final member selector.

In:

```phalcom
&service.registry.current().route(_, debug)
```

the receiver expression is:

```phalcom
service.registry.current()
```

and the reference target is the final selector:

```text
route(_,debug)
```

Everything before that final selector component is evaluated according to ordinary Phalcom expression semantics.

This rule allows references to compose naturally with ordinary member access without requiring parentheses around every receiver expression.

## Construction does not perform receiver behavior

Constructing a bound reference does not send the referenced selector to the receiver.

For example:

```phalcom
const missing = &object.notDefined(_)
```

does not, merely by constructing the reference:

- invoke `notDefined(_)`;
- invoke `doesNotUnderstand`;
- execute user code to probe whether `notDefined(_)` exists;
- resolve and capture a concrete Method implementation.

Whether later activation succeeds is a separate question governed by [Families](families.md) and [Dispatch](dispatch.md).

**Invariant — `REF-CONSTRUCTION-NONPERFORMING`**

> Constructing a callable reference must not perform the referenced operation.

For bound references, this also means reference construction does not dynamically probe receiver behavior merely to decide whether the capability may exist.

This invariant does not mean every reference expression is guaranteed to be semantically valid. Syntax, selector-shape validity, associated-name resolution, generic constraints, and other static rules may reject a reference before runtime. The prohibition is against performing the referenced behavior as part of reference construction.

## Exact references

An **exact reference** names one exact selector.

Examples:

```phalcom
&object.name
&object.name=(_)

&object.refresh()
&object.send(_)
&object.send(_, to)

&object[_]
&object[_, debug]
&object[_, debug]=(_)

&object.+(_)
&object.==(_)
```

Each selector is interpreted according to canonical selector identity.

For example:

```phalcom
&object.send(_, to)
```

references:

```text
send(_,to)
```

It does not match every Method named `send`.

Likewise:

```phalcom
&object[_, debug]=(_)
```

references exactly:

```text
[_,debug]=(_)
```

and not another SubscriptSet shape such as:

```text
[_]=(_)
[_,mode]=(_)
```

Exact reference construction preserves the exact selector. The resulting Family's later activation behavior is defined in [Families](families.md).

## Pattern references

A reference may name a selector pattern rather than one exact selector.

Method-pattern examples:

```phalcom
&object.render(...)
&object.render(_, ...)
&object.render(..., debug)
&object.render(_, ..., debug)
```

Whole named-family references:

```phalcom
&object.render...
&object.value...
&object.+...
```

Named accessor references:

```phalcom
&object.value=
```

Subscript-pattern references:

```phalcom
&object[...]
&object[_, ...]
&object[..., debug]

&object[...]=(_)
&object[_, ..., debug]=(_)

&object[...]=
&object[_, ..., debug]=
```

The structural meaning of each pattern is defined in [Selectors](selectors.md). Reference construction retains that selector predicate as the capability specification.

A pattern reference does not enumerate the receiver's current Methods at construction time.

For example:

```phalcom
const routes = &router.route(...)
```

means:

```text
retain router
retain selector pattern route(...)
```

rather than:

```text
snapshot every currently installed route Method
```

How that predicate is applied later belongs to [Families](families.md).

## Whole-family references

A trailing `...` after a named base references the complete named selector family for that base:

```phalcom
&object.value...
```

The referenced selector predicate includes the named kinds defined by the selector specification:

```text
Getter
Setter
Method
```

It does not include subscript selectors merely because the same receiver also defines subscript behavior.

Whole-family syntax is different from a Method wildcard:

```phalcom
&object.value...       // complete named base family
&object.value(...)     // Method selectors only
```

That distinction remains visible in the resulting Family.

## Accessor references

Accessor-pattern syntax references accessor kinds without including Methods.

For a named base:

```phalcom
&object.value=
```

references the Getter and Setter selectors of base `value`.

It can therefore cover:

```text
value
value=(_)
```

but not:

```text
value()
value(_)
```

For the subscript base:

```phalcom
&object[...]=
```

references both:

```text
SubscriptGet
SubscriptSet
```

subject to the written structural pattern.

The Family specification defines how getter-like and setter-like activation is requested from such a capability.

## Operator references

Operator Methods use the same `&` mechanism as other named Method selectors.

Examples:

```phalcom
&object.+(_)
&object.+(from)
&object.+(...)
&object.+...
```

An exact operator reference preserves Method identity.

A binary operator expression such as:

```phalcom
a + b
```

may participate in bilateral operator semantics defined by [Dispatch](dispatch.md), but:

```phalcom
&a.+(_)
```

is simply an exact reference to Method selector `+(_)`.

## Associated references

A callable reference may use `::` to reference behavior associated with an owner:

```phalcom
&Option::Some(_)
&Option::Some(...)
&Option::Some...

&Parser::parse(_)
&Parser::parse(...)

&Option::None
```

The portion before `::` identifies the associated lookup owner. The selector specification after the associated name determines the callable capability being requested.

Associated reference syntax does not turn `::` into class-side message dispatch.

For example:

```phalcom
&Fiber.new()
```

is a bound reference to ordinary Method `new()` on the class object `Fiber`.

By contrast:

```phalcom
&Owner::member(_)
```

requests an associated callable reference from `Owner`.

### Associated lookup surfaces

An associated name may expose different semantic surfaces, such as a type, a value, and callable behavior.

These forms are therefore not interchangeable:

```phalcom
Owner::member
&Owner::member
Owner::member(...)
&Owner::member...
```

Bare:

```phalcom
Owner::member
```

requests the associated value surface.

A callable reference requests the callable capability selected by the written selector shape.

A direct invocation:

```phalcom
Owner::member(...)
```

requests associated callable selection and invocation.

A whole-family reference:

```phalcom
&Owner::member...
```

requests the complete associated callable family for that base.

Failure to resolve one surface must not be silently repaired by substituting another surface.

In particular, the existence of a payload constructor:

```phalcom
Option::Some(value)
```

does not imply that bare:

```phalcom
Option::Some
```

is a value.

Likewise, bare associated value lookup does not implicitly produce:

```phalcom
&Option::Some...
```

## Associated exact callable references

Where an associated declaration exposes a callable matching an exact selector, `&` may reference that exact member:

```phalcom
&Option::Some(_)
&Parser::parse(_, mode)
```

The capability retains the associated callable identity under the owner rather than creating a bound receiver send.

For a variant payload constructor:

```phalcom
enum Option<T> {
  @variant Some(_ value: T)
}
```

the expression:

```phalcom
&Option::Some(_)
```

references the exact constructor callable shape.

It does not construct a `Some` value until the resulting capability is activated.

A whole-family constructor reference remains available:

```phalcom
&Option::Some...
```

when the complete constructor family is desired.

## Associated singleton values and Getter references

A declaration may expose a canonical associated singleton value.

For example, a singleton variant may support:

```phalcom
Option::None
```

as the canonical singleton value.

The callable reference:

```phalcom
&Option::None
```

does not evaluate to that value directly.

It references the exact Getter-shaped associated capability whose activation yields the canonical value.

Thus:

```phalcom
const noneValue = Option::None
const noneCapability = &Option::None
```

produce semantically different values.

The first is the singleton.

The second is a Family capability.

The corresponding Family may later be activated through its Getter API:

```phalcom
noneCapability.get()
noneCapability.value
```

as specified in [Families](families.md).

The capability is not silently converted into a nullary Method call:

```phalcom
noneCapability()
```

remains Method-kind activation and must not fall back to the Getter merely because both require no structural selector slots.

## Nullary constructors are not singleton Getter references

A nullary constructor and a singleton associated Getter remain different semantics.

If a declaration exposes a nullary constructor:

```phalcom
Variants::Nullary()
```

the exact constructor reference is Method-shaped:

```phalcom
&Variants::Nullary()
```

or the whole constructor family may be referenced:

```phalcom
&Variants::Nullary...
```

This does not by itself imply that:

```phalcom
Variants::Nullary
```

is an associated singleton value, nor that:

```phalcom
&Variants::Nullary
```

is a valid Getter reference.

Conversely, an associated singleton Getter such as:

```phalcom
&Option::None
```

does not automatically become an explicit nullary constructor.

The associated declaration determines which surfaces exist.

## Applied and generic owners

An associated reference may begin from a generic declaration or an applied owner where the type system permits associated lookup:

```phalcom
&Option::Some...
&Option<Int>::Some(_)
```

Specializing the owner constrains the environment in which the associated callable is interpreted. It does not create a new selector identity.

The selector:

```text
Some(_)
```

remains structurally the same selector under `SEL-STRUCTURAL-IDENTITY`; the associated owner contributes type and declaration context outside the selector itself.

Generic inference, constructor-local generics, GADT result refinement, and callable typing belong to the type-system specifications. This document requires that reference construction preserve the associated owner context rather than erasing it into the selector.

## Reference construction and access

A bound reference does not authorize or select a concrete target Method at construction time merely because the referenced selector exists.

Access control is enforced when behavior is selected or activated according to [Dispatch](dispatch.md) and the specific Family route.

Associated references may be subject to declaration-level visibility or semantic accessibility during associated resolution. Such static resolution does not violate `REF-CONSTRUCTION-NONPERFORMING`: checking whether an associated declaration is available is not the same as invoking the referenced callable.

The important separation is:

```text
reference construction
    establish receiver/owner + selector specification

Family activation
    route the capability and perform behavior
```

## References are not reflected Methods

A callable reference does not mean:

```text
look up a Method object now
```

For a bound exact reference:

```phalcom
const render = &object.render(_)
```

the resulting value is a Family capability, not a `Method` reflection object and not a `BoundMethod`.

This distinction matters under behavior replacement:

```phalcom
const render = &object.render(_)

// render(_) implementation changes

render(value)
```

The Family rules determine which implementation is selected on activation. Reference construction itself did not freeze a Method.

Likewise, a pattern reference:

```phalcom
&object.render(...)
```

is not an immutable `MethodFamily` snapshot.

## Invalid references

A callable reference is invalid when its target does not form a valid reference expression or its selector specification is invalid.

Examples include invalid selector shape:

```phalcom
&object.route(debug, _)
```

or invalid setter form:

```phalcom
&object.value=(_, _)
```

An associated reference may also be invalid because the requested associated callable surface does not exist or is not accessible.

Reference syntax must not recover from such failures by changing selector kind.

For example:

```phalcom
&object.value
```

must not be reinterpreted as:

```phalcom
&object.value()
```

when no Getter exists.

Likewise:

```phalcom
&Owner::member
```

must not silently become:

```phalcom
&Owner::member...
```

when only a constructor or Method family exists.

## Invariants defined by this specification

The following coded invariants are defined here because they constrain reference construction across independent selector shapes and are directly observable.

**`REF-RECEIVER-ONCE`**

> A bound callable reference evaluates its receiver expression exactly once when the reference is constructed and retains that resulting receiver for later Family activation.

**`REF-CONSTRUCTION-NONPERFORMING`**

> Constructing a callable reference must not perform the referenced operation.

Rules concerning exact selectors, selector patterns, setter shape, and kind distinctions remain owned by [Selectors](selectors.md) and are not duplicated under new reference-specific invariant names.

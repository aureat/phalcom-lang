# `@constructor` — mark a constructor method

- Status: **Canonical design; implementation pending**
- Governing decision: [PDR-0028](../../../pdr/0028-class-and-constructor-decorator-canon.md)
- Related: [Classes](../classes.md) · [`@construct`](construct.md) · [`@class`](../classes.md#21-class-side-fields--class)

## What it does

`@constructor` is legal on methods only. It marks the method as a constructor for
the receiving class.

```phalcom
class Person {
  @constructor
  new(name:) {
    _name = name
  }

  @constructor
  anonymous() {
    _name = "Anonymous"
  }
}
```

Constructor identity is the ordinary selector. Labels and arity are part of that
selector; constructors do not form a separate overload namespace.

## Semantics

Calling a constructor:

1. allocates a fresh instance;
2. runs the constructor body with `self` bound to that instance;
3. returns the instance.

Constructors are class-side behavior for lookup and inheritance, but their bodies
initialize instance state. `@constructor` therefore combines placement and
constructor meaning; stacking `@class @constructor` on one method is invalid.

Named constructors are ordinary:

```phalcom
Future.value(_)
Future.error(_)
Ref.at(_, _)
```

`new` has no constructor-only naming rule. It remains the conventional selector
with the ordinary inherited class-side default defined by the Classes spec.

## Legal targets and migration

Only method declarations may carry `@constructor`. A class header uses
`@construct` when it wants field-derived construction.

The retired form remains migration-compatible and receives a hint:

```text
@constructor
new(...) { ... }
hint: did you mean @constructor?
```

The hint is non-fatal and does not use deprecation-error wording.

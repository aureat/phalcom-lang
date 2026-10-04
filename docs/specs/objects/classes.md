# Phalcom Language Specification — Class Instance Fields and Construction

## Scope

This specification defines declared instance fields and their initialization
lifecycle. It does not define the class/metaclass tower, method lookup, static
class fields, or physical object layout.

## Declared instance fields

A class's declared instance fields define its instance-field names. Assigning
to an undeclared name does not add a field.

A bare field declaration is immutable. A field prefixed by contextual `mut` is
mutable:

```phalcom
class Point {
  _x: Int
  mut _label: String
}
```

Either form may have a default expression. A default expression is evaluated
once for each normally constructed object, in declaration order, by the
initializer of the class that declares the field. The initializer completes
those defaults before its user-authored constructor body proceeds. A mutable
field remains assignable after its default is established.

A mutable field without a default may remain unset until a later write. An
ordinary read of that field surfaces `None`; this field rule does not make an
uninitialized local or global binding read as `None`.

## Immutable-field initialization

An immutable field without a default must be assigned exactly once during its
declaring class's canonical instance initializer. A default establishes that
field before the user-authored body and counts as its initialization; the body
cannot assign it again. Writes to immutable fields outside their canonical
initializer are invalid.

Every normally completed constructor path must initialize every required
immutable field. Throwing or non-returning paths do not produce a successfully
constructed object. Inherited fields are initialized by their declaring
classes, not replayed by a subclass.

Reading an immutable field before its initialization is invalid. A definite
read may be diagnosed statically; a read that reaches execution while the
field is still uninitialized must fail with an initialization error. It must
not produce `None` or expose an internal sentinel.

## Inheritance and constructor order

When a parent class has a canonical initialization lifecycle, a subclass
constructor must begin with exactly one explicit `super.new(...)` call to that
parent initializer. Omitting the call is an invalid constructor path; P1 does
not add implicit superclass chaining.

The parent initializer completes before the subclass's own defaults and
remaining body statements. This orders initialization from base to derived,
with each field default run only by its declaring class's initializer.

The source `@constructor` form provides the ordinary class-side factory and
instance initializer used for normal construction. A class with defaults and
no user-authored constructor has a zero-argument normal construction path when
that path does not omit a required parent initializer or leave a required
immutable field uninitialized. Generated `@construct` and `@data` constructors
follow the same once-only initialization rules.

Raw allocation is distinct from normal construction. It allocates storage but
does not evaluate defaults or establish immutable fields. Such an instance is
incomplete; reads of its uninitialized immutable fields fail as described
above.

# Classes

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

## 1. Constructors

```phalcom
class Person {
  construct new(name:, age:) {
    _name = name
    _age = age
  }

  construct new(name:) { _name = name }

  construct anonymous() { _name = "Anonymous" }
}
```

`construct` is a keyword. The compiler:

1. Emits allocation of a fresh instance.
2. Runs the body with `self` bound to it.
3. Returns `self` implicitly.

Users **never** write `let i = self.new(); i.init(...)`. There is no user-visible
allocator, no implicit zero-arg `new`, and no arity-shadowing magic. A `construct`
declares a method on the **metaclass** ([Object Model §5](object-model.md)).

`new` is not special — `construct anonymous()` is equally legitimate. Multiple
constructors are distinguished by selector, not arity hacks: `new(name:age:)` and
`new(name:)` are simply two different selectors.

## 2. Fields

Fields are `_`-prefixed and **implicitly declared by assignment**. The compiler
collects the set of fields assigned anywhere in the class body and fixes the slot
layout at class-definition time.

- **Read-before-write is a compile error.** Reading a field never assigned in *any*
  method of the class is rejected at compile time — catching the typo class
  (`_naem = name`) that a dynamic field model lets through silently.
- A field *declared* (assigned somewhere) but not yet assigned on a given instance
  reads as **`None`** ([Values & Absence](values-and-absence.md)).
- **Fields are private to the declaring class and not inherited-visible.** A
  subclass that writes `_name` gets its own new slot; it does not touch the
  superclass's. Cross-hierarchy access goes through accessors. This keeps slot
  offsets static and eliminates the fragile-base-class problem.

## 3. Methods, accessors, operators

```phalcom
class Person {
  name  => _name                                 // getter, expression body
  name=(value) { _name = value }                 // setter

  isAdult => _age >= 18

  greet(other) => "Hello {other.name}, I'm {_name}"

  describe() {                                    // block body
    _age.ifSome { a => return "{_name}, {a}" }
    "{_name}, age unknown"
  }

  ==(other) => self.name == other.name and self.age == other.age

  static species => "Homo sapiens"
}
```

- `=>` is **general expression-body sugar**, not getter-only. It works on any
  method.
- A getter is a method with no parameter list. `name` and `name()` are different
  selectors.
- Operators are ordinary methods.
- `static` declares on the metaclass.

## 4. Implicit return

The value of a method (and of a block) is its **last expression**. `return` exists
and means **early exit** ([Blocks §5](blocks.md)).
</content>

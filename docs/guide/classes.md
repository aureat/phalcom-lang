# Classes

A class is an object with a name, a superclass, and a method dictionary — nothing
more exotic than that. This page is about the surface: how you declare one, where
state lives, and how methods, accessors, and operators are all just entries in
the same table.

## Declaring a class

```phalcom
class Person {
  @constructor
  new(name:, age:) {
    _name = name
    _age = age
  }

  name => _name
}

class Employee is Person {
  @constructor
  new(name:, age:, title:) {
    super.new(name: name, age: age)
    _title = title
  }

  title => _title
}
```

`extends` is optional — a class with no `extends` clause is a direct subclass of
`Object`. Inheritance is **single**: one `superclass`, no mixins, no interfaces
to satisfy. The full class/metaclass story — why this is enough, and what it
buys you — is [The Object Model](../spec/current/object-model.md).

## Constructors

There is no implicit `new` and no user-visible allocator. A class gets an
initializer only if you write one, with `@constructor`:

```phalcom
class Person {
  @constructor
  new(name:, age:) {
    _name = name
    _age = age
  }

  @constructor
  new(name:) { _name = name }     // a different selector, not an overload
  @constructor
  anonymous()  { _name = "Anonymous" }
}

Person.new(name: "Ada", age: 36)
Person.new(name: "Grace")
Person.anonymous()
```

`@constructor` allocates the instance, runs the body with `self` bound to it, and
returns `self` — you never see or write the allocation step. `new` isn't a
keyword; `anonymous` is exactly as legitimate a constructor name. Because
selectors are labels-and-all, `new(name:, age:)` and `new(name:)` coexist as
two distinct constructors rather than one arity-juggling `new`. See
[Classes §1](../spec/current/classes.md#1-constructors) for the rest, and
[Messages & Dispatch](messages.md) for why labels are identity everywhere, not
just here.

## Fields

Fields are `_`-prefixed and declared **by assignment**, not by a separate
`field` form — the compiler scans a class body for every `_name` that gets
assigned and fixes that as the instance's slot layout:

```phalcom
class Counter {
  @constructor
  new() { _count = 0 }

  increment() { _count = _count + 1 }
  value => _count
}
```

Two consequences worth internalizing:

- Reading a field that is never assigned *anywhere* in the class is a **compile
  error** — the typo (`_naem`) is caught before you run anything.
- A field that's declared but not yet assigned on a given instance reads as
  **`None`**, the same absence value as everywhere else in the language (see
  [Values](values.md)).

Fields are private to the class that declares them — a subclass writing `_name`
gets its own slot, not the superclass's. There's no field-visibility syntax to
learn because there's only one visibility: external code reaches state through
a method, always. Details and rationale in
[Classes §2](../spec/current/classes.md#2-fields) and
[Selectors §5](../spec/current/selectors.md#5-field-visibility).

## Methods, accessors, and operators

A method body is either a block (`{ ... }`, using `return` for early exit) or,
for a single expression, `=>`:

```phalcom
class Person {
  name  => _name                  // getter — no parameter list
  name=(value) { _name = value }  // setter

  isAdult => _age >= 18           // => is general expression-body sugar

  greet(other) => "Hello \(other.name), I'm \(_name)"

  ==(other) => self.name == other.name and self.age == other.age
}
```

- `=>` isn't getter-only sugar — it works on any method, `greet` included.
- A getter is a method with **no** parameter list; `name` and `name()` are
  distinct selectors, so you can define both if you genuinely want two
  behaviors.
- Operators (`==`, `+`, `[]`, ...) are ordinary methods with punctuation for a
  name — nothing about dispatch treats them specially. See
  [Selectors §1](../spec/current/selectors.md#1-selector-identity) for how a
  selector's identity is computed.

## Implicit return

A method's value is its **last expression**, same as a block. `return` exists,
but it means *early* exit, not *the normal* way to produce a value:

```phalcom
describe() {
  _age.ifSome { a => return "\(_name), \(a)" }   // early exit
  "\(_name), age unknown"                         // implicit return
}
```

## `@class` members

`@class` declares a field or method on the class side rather than on instances:

```phalcom
class Person {
  @class
  species => "Homo sapiens"
}

Person.species   // "Homo sapiens"
```

Under the hood there's no separate class-side mechanism — `@class` methods live
on the **metaclass**, `Person`'s own class, and are looked up by walking the
metaclass's superclass chain exactly like an instance send walks `Person`'s.
That's why class-side methods and `@constructor` inherit correctly along a class
hierarchy for free. The full tower — metaclasses, `X class`, why classes being
objects pays off — is [The Object Model](../spec/current/object-model.md); the
slot-layout mechanics are [ADR-0011](../adr/0011-static-instance-slot-layout.md).

## `super` and method resolution

`super.someMethod(...)` sends to the superclass of the method's **defining**
class, not the receiver's runtime class — so a chain of overrides can each call
up one level without knowing how deep the hierarchy actually goes:

```phalcom
class Employee is Person {
  @constructor
  new(name:, age:, title:) {
    super.new(name: name, age: age)   // Person's constructor
    _title = title
  }
}
```

An ordinary (non-`super`) send starts the walk at the receiver's actual class
and climbs `superclass` until a matching selector turns up, falling through to
`doesNotUnderstand` if it never does. That walk, plus why it's cheap on the hot
path, is the subject of [Messages & Dispatch](messages.md).

---

Next: [Messages & Dispatch](messages.md) — selector identity, labelled
arguments, and the method-lookup walk that `super` and ordinary sends both ride on.

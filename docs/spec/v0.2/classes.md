# Classes

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

**Governing ADRs:**
[ADR-0063](../../adr/accepted/0063-constructors-are-ordinary-class-side-methods.md) (**Accepted** — constructors as ordinary class-side methods; `@constructor`/`@static`; `new_`) ·
[ADR-0011](../../adr/accepted/0011-static-instance-slot-layout.md) (static per-class slot layout) ·
[ADR-0014](../../adr/accepted/0014-let-and-var-bindings.md) (let and var bindings) ·
[ADR-0002](../../adr/accepted/0002-metaclass-tower-parallel-rule.md) (the parallel tower that resolves them)

## 1. Constructors

```phalcom
class Person {
  @constructor
  new(name:, age:) {
    _name = name
    _age = age
  }

  @constructor
  new(name:) { _name = name }

  @constructor
  anonymous() { _name = "Anonymous" }
}
```

**A constructor is an ordinary class-side method.** `@constructor` is a decorator,
not a keyword — it declares a method on the **metaclass** ([Object Model
§5](object-model.md)), which is why constructors inherit and why they resolve from
*any* receiver expression:

```phalcom
Person.new(name: "Ada", age: 36)      // literal class
var C = Person   C.new(name: "Ada")   // variable
M.Person.new(name: "Ada")             // module member
```

All three encode the same selector `new(name)` and go through the same ordinary
lookup. There is no separate constructor namespace and no call-site rewriting.

What the decorator buys over a hand-written class-side method is sugar. It expands to
two ordinary methods: a class-side one that allocates and returns, and an instance-side
initializer holding the body.

1. Allocates a fresh instance via `new_` (§1.2).
2. Runs the body with `self` bound to it.
3. Returns the instance implicitly.

Multiple constructors are distinguished by **selector**, not arity hacks:
`new(name:, age:)` and `new(name:)` are simply two different selectors.

### 1.1 `new` is a convention, not a rule

`@constructor anonymous()` is equally legitimate, and named constructors are the
ordinary case — `Future.value(_)`, `Future.error(_)`, `Ref.at(_,_)`, `Cell.of(_)` all
ship today. They carry no special machinery: nothing named `at()` sits at the tower
root, so `Ref.at(1, 2)` either finds the constructor or raises `doesNotUnderstand`.

`new` is the **one** name the language treats specially, and only because
`Class >> new()` occupies it as a default (§1.2). See ADR-0063 §7 for the tombstone
rule that keeps a wrong-arity `new` from silently reaching that default.

### 1.2 The allocator: `new_`

`Class >> new_()` is the sole primitive allocator — arity 0, uninitialized instance,
reserved (declaring `new_` in a user class is an error). Every class object reaches it
through the tower.

`Class >> new()` is **ordinary Phalcom**, a default at the tower root:

```phalcom
class Class {
  new() => self.new_()
}
```

It is shadowed by ordinary lookup like any other method — which is all a class
declaring its own `new` does. A class with no constructor inherits it, which is why
`Point.new()` works on a constructor-less class.

Because `new_` is public, a constructor can be written by hand, with no decorator, as
an ordinary pair of methods — this is exactly what `@constructor` desugars to:

```phalcom
class Point {
  @static
  new(x, y) {
    let instance = self.new_()
    instance.init(x, y)
    return instance
  }

  init(x, y) {
    _x = x
    _y = y
    return self
  }
}
```

`init` here is an ordinary instance method with no special status; the name is the
author's choice. This is the Smalltalk pattern (`Point class >> new` calling
`basicNew`), and `@constructor` exists to make the common case a one-liner — not
because the long form is forbidden.

**Relationship to `@constructor` on a class header.** [Selectors, Symbols &
References §4](selectors.md#4-attributes-) specifies `@constructor` as a class-header
attribute that *derives* a constructor from the declared fields. Same name, same
mechanism: the header form emits a `@constructor` method member, which then expands
exactly as a hand-written one does.

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

  This is the same privacy rule as [Selectors, Symbols & References §5](selectors.md#5-field-visibility):
  fields are always private, with no visibility syntax; every external access
  is a message send through a derived or hand-written accessor.

### 2.1 Class-side fields — `@classField`

`@classField` declares storage on the **class object** rather than on instances
([ADR-0017](../../adr/accepted/0017-class-side-stored-static-fields.md)):

```phalcom
class Counter {
  @classField var _count = 0

  @constructor
  new() { _count = _count + 1 }

  @static
  count => _count
}

Counter.new()  Counter.new()  Counter.new()
Counter.count                 // 3 — one slot, shared by every instance
```

**Storage is per declaring class.** A subclass gets its **own fresh slot**, reading
`None` until written — it does not share the superclass's:

```phalcom
class Base { @classField var _count = 0
             @static bump() { _count = _count + 1 }
             @static count => _count }
class Derived extends Base {}

Base.bump()  Base.bump()
Base.count      // 2
Derived.count   // None — its own slot
```

This is §2's field rule exactly, one tower level up: ADR-0017 is
[ADR-0011](../../adr/accepted/0011-static-instance-slot-layout.md)'s slot vector
shifted onto the metaclass. In Smalltalk terms `@classField` is a **class-instance
variable**, *not* a class variable — nothing is shared across a hierarchy, which is
why it is not spelled `@static` (dispatch placement is a different concept from
storage) and not `@shared`.

**Consequence, by design:** an inherited `@static` method touching an unset subclass
`@classField` reads `None`.

```phalcom
Derived.bump()   // None does not understand '+(_)'
```

The subclass genuinely has its own slot; the declaration's initializer is **not**
re-run per subclass, matching instance fields, which likewise read `None` until
written. A subclass that wants its own counter declares its own `@classField`.

## 3. Methods, accessors, operators

```phalcom
class Person {
  name  => _name                                 // getter, expression body
  name=(value) { _name = value }                 // setter

  isAdult => _age >= 18

  greet(other) => "Hello \(other.name), I'm \(_name)"

  describe() {                                    // block body
    _age.ifSome { a => return "\(_name), \(a)" }
    "\(_name), age unknown"
  }

  ==(other) => self.name == other.name and self.age == other.age

  @static
  species => "Homo sapiens"
}
```

- `=>` is **general expression-body sugar**, not getter-only. It works on any
  method.
- A getter is a method with no parameter list. `name` and `name()` are different
  selectors.
- Operators are ordinary methods.
- `@static` declares on the metaclass. It is a decorator, not a keyword — a pure
  modifier that sets one bit on the member, adding no machinery of its own. Legal on
  methods, getters, and setters.

`@static` and `@constructor` are both decorators but are different *kinds* of
attribute, and the difference is visible in what they produce:

| | `@static` | `@constructor` |
|---|---|---|
| Kind | modifier | derive |
| Effect | sets one bit in place | rewrites one member into two |
| Members in → out | 1 → 1 | 1 → 2 |

`@static @constructor` on one member is an error: a constructor is already class-side.

**Relationship to `@get`/`@set`.** The getter/setter pair above (`name =>
_name` / `name=(value) { ... }`) is hand-written. [Selectors, Symbols &
References §4](selectors.md#4-attributes-) proposes `@get`/`@set` field
attributes that derive the same accessor methods automatically. Planned — not
yet specified whether hand-written accessors and derived ones can coexist on
the same field or are mutually exclusive.

## 3.1 Duplicate selectors

Two members of one class body may not install the same selector on the same side
(instance or class-side), regardless of decorators:

```phalcom
class Foo {
  @constructor
  new(x) { _x = x }

  @static
  new(x) { return 42 }        // error: both define class-side `new(_)`
}
```

This is a **duplicate definition**, the same species as declaring `foo()` twice — not
a rule about constructors. Within one body there is no lookup order to appeal to.

Across a hierarchy there is, so a **subclass** `@static new()` shadowing a parent's
`@constructor new()` is legal and silent. That is an override, and overriding is what
a class hierarchy is for.

## 4. Implicit return

The value of a method (and of a block) is its **last expression**. `return` exists
and means **early exit** ([Blocks §5](blocks.md)).
</content>

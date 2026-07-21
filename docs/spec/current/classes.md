# Classes

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

**Governing ADRs:**
[PDR-0028](../../pdr/0028-class-and-constructor-decorator-canon.md) (**Accepted** — `@construct` classes, `@constructor` methods, target-polymorphic `@class`) ·
[ADR-0011](../../adr/accepted/0011-static-instance-slot-layout.md) (static per-class slot layout) ·
[ADR-0064](../../adr/accepted/0064-let-const-bindings-and-field-mutability.md) (**Accepted** — `let`/`const`; unkeyworded mutable fields; supersedes ADR-0014) ·
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
let C = Person   C.new(name: "Ada")   // variable
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
`Class >> new()` occupies it as a default (§1.2) — and even that is ordinary
inheritance, not constructor-specific dispatch (PDR-0028).

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
  @class
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

**Relationship to `@construct` on a class header.** [Selectors, Symbols &
References §4](selectors.md#4-attributes-) specifies `@construct` as the class-header
attribute that derives a constructor from declared fields. The derived member has
the method semantics of [`@constructor`](decorators/constructor.md), but the two
decorators have different legal targets.

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

**Mutability** ([ADR-0064](../../adr/accepted/0064-let-const-bindings-and-field-mutability.md)):

| Form | Meaning |
|---|---|
| `_x` / `_x = e` | mutable — **no keyword**; `= e` supplies a declaration default |
| `const _x = e` | immutable, defined at the declaration |
| `const _id` | immutable, assignable **only inside a `@constructor`** |

Mutable is the unkeyworded case because a field is already `_`-prefixed, already
private, and already declarable by assignment — a keyword would add nothing. `let _x`
at field position is rejected.

`const` enforcement is **syntactic**: a write from any member other than a constructor
is a compile error. It is deliberately not flow-sensitive (Phalcom has no flow
analysis), so two writes *within* one constructor are not caught, and a `const` field
that no constructor assigns reads `None` forever — reachable via `Point.new()` (§1.2),
and specified rather than repaired.

### 2.1 Class-side fields — `@class`

`@class` on a field declares storage on the **class object** rather than on instances
([ADR-0017](../../adr/accepted/0017-class-side-stored-static-fields.md)):

```phalcom
class Counter {
  @class _count = 0

  @constructor
  new() { _count = _count + 1 }

  @class
  count => _count
}

Counter.new()  Counter.new()  Counter.new()
Counter.count                 // 3 — one slot, shared by every instance
```

**Storage is per declaring class.** A subclass gets its **own fresh slot**, reading
`None` until written — it does not share the superclass's:

```phalcom
class Base { @class _count = 0
             @class bump() { _count = _count + 1 }
             @class count => _count }
class Derived extends Base {}

Base.bump()  Base.bump()
Base.count      // 2
Derived.count   // None — its own slot
```

This is §2's field rule exactly, one tower level up: ADR-0017 is
[ADR-0011](../../adr/accepted/0011-static-instance-slot-layout.md)'s slot vector
shifted onto the metaclass. In Smalltalk terms a `@class` field is a **class-instance
variable**, *not* a class variable — nothing is shared across a hierarchy, which is
why it is not spelled `@shared` or `@classvar` — each would assert a sharing that does
not exist. `@class` names **placement**, which is exactly what a class-side method and a
class-side field have in common, so one decorator covers both.

**Consequence, by design:** an inherited `@class` method touching an unset subclass
`@class` field reads `None`.

```phalcom
Derived.bump()   // None does not understand '+(_)'
```

The subclass genuinely has its own slot; the declaration's initializer is **not**
re-run per subclass, matching instance fields, which likewise read `None` until
written. A subclass that wants its own counter declares its own `@class` field.

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

  @class
  species => "Homo sapiens"
}
```

- `=>` is **general expression-body sugar**, not getter-only. It works on any
  method.
- A getter is a method with no parameter list. `name` and `name()` are different
  selectors.
- Operators are ordinary methods.
- `@class` declares on the metaclass. It is a decorator, not a keyword — a pure
  modifier that sets one bit on the member, adding no machinery of its own. Legal on
  methods, getters, and setters.

`@class` and `@constructor` are both decorators but are different *kinds* of
attribute, and the difference is visible in what they produce:

| | `@class` | `@constructor` |
|---|---|---|
| Kind | placement | constructor marker |
| Effect | places field or member on class side | marks one method as constructor |
| Members in → out | 1 → 1 | 1 → 1 |

`@class @constructor` on one member is an error: a constructor is already class-side.

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

  @class
  new(x) { return 42 }        // error: both define class-side `new(_)`
}
```

This is a **duplicate definition**, the same species as declaring `foo()` twice — not
a rule about constructors. Within one body there is no lookup order to appeal to.

Across a hierarchy there is, so a **subclass** `@class new()` shadowing a parent's
`@constructor new()` is legal and silent. That is an override, and overriding is what
a class hierarchy is for.

## 4. Implicit return

The value of a method (and of a block) is its **last expression**. `return` exists
and means **early exit** ([Blocks §5](blocks.md)).
</content>

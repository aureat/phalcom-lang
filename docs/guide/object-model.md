# The Object Model

You've been told twice already that classes are objects — once when `static`
inherited without ceremony, once when `doesNotUnderstand` fell out of an
ordinary method lookup. This page is where that stops being a slogan and
becomes a tower you can draw.

## Classes are objects, so they have a class too

Every value has a class. That's true of your instances, and it's true of the
classes themselves — a class is an object, so it's an instance of *something*.
That something is its **metaclass**:

```phalcom
class Sprite {
  move(dx, dy) { ... }
  static spawn() => Sprite.new()
}

let s = Sprite.new()

s.class              // Sprite          — an instance's class holds its methods
Sprite.class          // Sprite class    — the metaclass, holds Sprite's static methods
Sprite.class.class    // Metaclass       — every metaclass is itself an instance of Metaclass
```

Four rungs, one relationship (`instance-of`) climbed four times:

```
s ────────► Sprite ────────► Sprite class ────────► Metaclass
    .class       .class            .class
  (an instance) (a class,     (a metaclass,       (the class every
                 holds s's     holds Sprite's       metaclass is an
                 methods)      static methods)      instance of)
```

`Sprite class` is not a naming convention, it's a real, distinct object —
Sprite's own, private metaclass, created alongside `Sprite` and holding exactly
the methods you marked `static`. Nothing is special-cased to make `static`
work: it's an ordinary instance send where the "instance" happens to be a
class and the "class" happens to be a metaclass.

## The parallel rule: why inherited `static` works

Declaring `static species` on `Person` and reading it through `Employee` only
works because the metaclass hierarchy isn't flat — it *mirrors* the class
hierarchy, one level up:

```phalcom
class Person {
  static species => "Homo sapiens"
}

class Employee extends Person {}

Employee.species   // "Homo sapiens" — inherited, not redefined
```

`Employee.species` resolves because `Employee class` (the metaclass doing the
lookup) has `Person class` as its superclass — the same shape as the instance
side, one tier up. That's [ADR-0002](../adr/0002-metaclass-tower-parallel-rule.md),
the **parallel-hierarchy rule**:

```
(X class).superclass  ==  (X.superclass) class
```

Side by side:

```
instance side              metaclass side
──────────────             ───────────────────
Object                     Class
  └─ Behavior                └─ Object class
       ├─ Class                   └─ Behavior class
       └─ Metaclass                    ├─ Class class
                                        └─ Metaclass class

Person                     Person class
  └─ Employee                 └─ Employee class
```

The two trees have the same shape, offset by one `.class`. `Behavior` is the
shared abstract superclass of `Class` and `Metaclass` — the common home for
"things that have instances" ([ADR-0003](../adr/0003-introduce-behavior-kernel-class.md)).
The tower closes at the top: `Metaclass` is an instance of `Metaclass class`,
which is in turn an instance of `Metaclass` — the one deliberate cycle in an
otherwise acyclic model.

Without the parallel rule, every metaclass's superclass would just be `Class`,
flat, and `Employee.species` would fail to resolve — which is exactly the bug
ADR-0002 was written to fix. You don't need the rest of the tower memorized;
you need this one fact: **a subclass's metaclass inherits from its
superclass's metaclass**, so anything you put on the class side inherits
exactly like anything you put on the instance side.

## One lookup algorithm, two starting points

[Messages & Dispatch](messages.md) already gave you the algorithm: inline
cache, then a probe on the receiver's class walking `superclass`, then the
variadic table, then `doesNotUnderstand`. Nothing changes for class-side
sends — the walk is identical, it just starts one rung higher:

| Send | Walk starts at | Climbs |
|------|----------------|--------|
| `s.move(1, 2)` | `s.class` → `Sprite` | `Sprite`'s superclass chain |
| `Sprite.spawn()` | `Sprite.class` → `Sprite class` | `Sprite class`'s superclass chain |

One hashmap hit per class on the interned selector, same as any instance send.
`super` inside a static method restarts the walk at the superclass of the
*defining* metaclass, for the same reason `super` does anywhere else. See
[Method Lookup](../spec/v0.2/method-lookup.md) for the normative resolution
order — this page only needed you to see *what* the walk climbs over.

## Why this pays off

None of the following needed a bolted-on feature. They're all the same
mechanism, "classes are objects," pointed at a different question:

- **`static` methods** — a class-side send is an ordinary send to the
  metaclass. You just saw it inherit for free.
- **Per-class state** — `Class` is an ordinary heap-object kind (`U` in the
  catalog, not a VM special case), so a class carries slots the same way any
  instance does. There's no separate "static storage" mechanism to design.
- **Reflection** — `respondsTo`, `perform`, and `doesNotUnderstand` are just
  `Object` methods, inherited by every object including classes and
  metaclasses:

  ```phalcom
  Sprite.respondsTo(#move(_,_))    // true  — reflection is a message like any other
  s.perform(#move(_,_), [1, 2])    // same effect as s.move(1, 2)
  ```

  A failed send is reified as a `Message` and re-dispatched to
  `doesNotUnderstand(_)` — proxies and delegation are downstream of the same
  lookup walk, not a separate hook grafted on top. Full story in
  [Messages & Dispatch](messages.md#doesnotunderstand-the-hook-for-everything-reflective).

## The kernel: `Object` at the root, dispatch instead of branching

Every class's superclass chain terminates at `Object`, which defines the
universal protocol — `class`, `==`, `hash`, `toString`, `respondsTo`,
`perform`, `doesNotUnderstand` — overridable everywhere. `Behavior`, `Class`,
and `Metaclass` sit just below it as the kernel that makes the tower work;
everything else you use day to day sits below *them*.

A pattern repeats across the kernel classes you already know: instead of one
concrete class branching internally on a hidden tag, there's one **abstract**
superclass and two (or more) **concrete** subclasses, and the "branch" is just
which subclass got instantiated:

| Abstract | Concrete subclasses | Governing ADR |
|----------|---------------------|---------------|
| `Bool` | `True`, `False` | [ADR-0004](../adr/0004-boolean-as-abstract-bool-with-true-false.md) |
| `Option` | `Some`, `None` | [ADR-0007](../adr/0007-option-as-abstract-with-some-none.md) |
| `Result` | `Ok`, `Err` | [ADR-0008](../adr/0008-layered-exceptions-and-result.md) |

`ifTrue`/`ifFalse` are two method definitions — `True>>ifTrue`, `False>>ifTrue`
— not one method with an `if` inside it. `Option>>map` is `Some>>map` and
`None>>map`. There's no tag to test because there's nothing to test: method
lookup already knows which subclass it's holding, so dispatch *is* the
branch. It's the same trick you saw for `static`, run on values instead of
classes — "classes are objects" and "dispatch replaces branching" are two
views of the one idea that this whole page has been building toward.

---

For the exhaustive class catalog, the full bootstrap-construction order, and
every invariant the tower must satisfy, see
[the spec](../spec/v0.2/object-model.md).

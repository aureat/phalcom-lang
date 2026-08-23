I reviewed the mixin spec against our prior design discussions. It is mostly complete, but there are several things we **explicitly discussed and effectively ratified that are missing or under-specified**. Some are important enough that they should be added before considering the mixin specification final.

I would categorize them as: **missing semantic guarantees**, **missing interaction rules**, and **places where the spec accidentally drifted from previous decisions**.

---

# 1. Missing: mixin methods are not copied textually — method identity and origin matter

The spec says:

> "Mixins are flattened into the host’s effective method table."

That is correct semantically, but it risks implying literal copying.

We previously discussed that Phalcom reflection should preserve:

- where the method originated;
- what mixin application produced it;
- what host installed it.

The spec mentions reflection, but it should explicitly state the invariant:

```
A mixed method behaves as if declared by the host for dispatch,
but remains identifiable as originating from its mixin source.
```

Example:

```
@mixin
class DebugPrintable {
  debugString -> String {
    ...
  }
}

@compose(DebugPrintable)
class User {}
```

Reflection:

```
User.method(#debugString).owner
// User

User.method(#debugString).origin
// DebugPrintable

User.method(#debugString).application
// DebugPrintable<>
```

This distinction matters for:

- debugging;
- stack traces;
- IDE navigation;
- documentation generation;
- profiling.

I would add a formal section:

> Mixed methods have two identities:
>
> - dispatch identity: the consuming class;
> - origin identity: the contributing mixin application.

---

# 2. Missing: mixin application is compile-time, not runtime inheritance

The current wording:

> "A consuming class receives effective mixin methods."

is correct but could be interpreted as runtime delegation.

We previously converged on:

- no mixin object;
- no runtime forwarding;
- no mixin lookup chain.

The spec should explicitly prohibit:

```
class -> mixin -> mixin parent
```

runtime lookup.

The runtime model should be:

```
Object
  |
User
  |
effective method table
  |
  +-- User methods
  +-- DebugPrintable methods
  +-- Enumerable methods
```

not:

```
Object
 |
User
 |
MixinProxy
 |
DebugPrintable
```

This is important because it affects:

- performance;
- `super`;
- identity;
- dispatch caches.

---

# 3. Missing: mixin method overriding rules need one more case

The spec says:

> Class methods override mixin methods.

Good.

But it does not explicitly cover:

## A superclass mixin versus subclass mixin

Example:

```
@compose(DebugPrintable)
class Base {}

class Child is Base {}
```

Child inherits the mixed method.

Now:

```
@compose(AlternativePrintable)
class Child is Base {}
```

What happens?

The intended rule should be:

```
Direct mixins of Child override inherited methods,
including methods originally supplied by a superclass mixin.
```

Therefore:

```
Base
 └─ DebugPrintable.toString

Child
 └─ AlternativePrintable.toString
```

Child wins.

This follows normal inheritance intuition.

Add:

> A subclass's direct mixin contributions participate at the same precedence level as subclass-declared methods over inherited methods.

---

# 4. Missing: whether mixins can call methods supplied by another mixin

This was discussed indirectly but should be explicit.

Example:

```
@mixin
class A {
  foo() {
    self.bar()
  }
}

@mixin
class B {
  bar() {}
}
```

```
@compose(A, B)
class C {}
```

Should this work?

The answer should be:

**Yes, dynamically.**

Because Phalcom is message-oriented.

However:

```
@mixin
class A {
  foo() {
    self.bar()
  }
}
```

should not statically assume `bar` exists unless:

```
@mixin(BarProtocol)
class A {
}
```

or:

```
@mixin(
  with: [B]
)
```

is present.

So:

Runtime:

```
self.bar()
```

may dispatch to any available implementation.

Static checking:
requires declared capability.

This distinction is very Phalcom-like and should be captured.

---

# 5. Missing: mixin private methods and collision semantics

The spec mentions private helper methods but does not define collisions.

Example:

```
@mixin
class A {
  private helper() {}
}

@mixin
class B {
  private helper() {}
}
```

Does this conflict?

Recommendation:

Private selectors declared by mixins should still participate in conflict analysis.

Why?

Because after flattening they inhabit the same host namespace.

Otherwise:

```
A.helper()
B.helper()
```

becomes ambiguous.

Possible rule:

```
Mixin private methods are private to the host, not private to the mixin.
Therefore they obey normal selector collision rules.
```

This is important.

---

# 6. Missing: mixin generic specialization identity

We discussed first-class specialized types.

The spec says:

```
Enumerable<Int>
```

but does not specify identity.

It should state:

A specialized mixin application is a distinct descriptor:

```
Enumerable<Int>
Enumerable<String>
```

are different mixin applications.

Reflection:

```
Enumerable<Int>.origin
// Enumerable

Enumerable<Int>.arguments
// [Int]
```

This should align with the broader Phalcom type system decision:

> Types remember specialization.

Mixins should follow the same rule.

---

# 7. Missing: whether mixins can define class-side methods

The spec says:

> class-side constants belonging to the descriptor

but this is a dangerous area.

We discussed mixins primarily as instance behavior.

The question:

```
@mixin
class FactoryHelpers {
  class newDefault() {}
}
```

Should this exist?

My recommendation:

**No in v1.**

Reason:

Class-side behavior introduces another composition problem:

```
User.newDefault()
```

Which mixin owns the class-side selector?

Instance composition is already complicated enough.

Therefore:

Add:

```
Mixin composition applies only to instance-side behavior in version one.

Class-side methods are not contributed by mixins.
```

---

# 8. Missing: mixins and class variables

Related issue.

A mixin should not contribute:

```
class var cache
```

because that raises:

- ownership questions;
- initialization order;
- inheritance behavior.

Recommendation:

Explicitly prohibit:

```
Mixin declarations cannot declare class variables.
```

---

# 9. Missing: mixins and operator selectors

The spec says operators are allowed indirectly, but this deserves explicit mention.

Given Phalcom selectors:

```
+(other)
==(other)
[](index)
```

mixins absolutely should support them.

Example:

```
@mixin(Additive<T>)
class ArithmeticHelpers<T> {
  +(other: T) {
    ...
  }
}
```

This is an important use case.

Add:

> Operator selectors are ordinary selectors and may be contributed by mixins.

---

# 10. Missing: mixins and contracts/invariants

You included method contracts, but not class invariants.

Question:

Can:

```
@mixin
@invariant(...)
class M {}
```

exist?

Recommendation:

No.

Reason:

The invariant belongs to the host object, not reusable behavior.

A mixin can contribute methods with preconditions/postconditions:

```
@requires(...)
@ensures(...)
method()
```

but cannot impose object-wide invariants.

Add:

```
Mixins cannot declare class invariants.
```

---

# 11. Missing: mixin requirements should support generic self relationships

This was discussed in the type system.

Example:

```
@mixin(Comparable<Self>)
class OrderedOperations<Self> {
}
```

The requirement is not merely:

```
Comparable<T>
```

but:

```
host must conform to Comparable<Self>
```

The spec should mention that requirements may contain:

- declared type parameters;
- `Self`;
- generic applications;
- variance-aware relationships.

Example:

```
@mixin(Comparable<Self>)
class OrderingHelpers<Self> {}
```

This is likely necessary for fluent generic APIs.

---

# 12. Missing: mixins and sealed classes

The spec says:

> Mixins do not affect sealed membership.

Correct.

But one subtle point:

A sealed class can consume a mixin.

A mixin cannot extend a sealed class.

Add:

Valid:

```
@sealed
class Expr {}

@compose(DebugPrintable)
class Literal is Expr {}
```

Invalid:

```
@mixin
class ExprHelpers is Expr {}
```

---

# 13. Missing: mixin application to variants

The spec says variants may consume mixins, but this is slightly too permissive.

Given:

```
@sealed @data
class Result<T,E> {}

@variant
class Ok<T> is Result<T,Nothing> {}
```

Can:

```
@variant
@compose(DebugPrintable)
class Ok<T> is Result<T,Nothing>
```

work?

Probably yes.

But:

Can the parent sealed type consume a mixin and have all variants inherit it?

Example:

```
@sealed
@compose(DebugPrintable)
class Result<T,E> {}
```

This is likely also valid.

The spec should define:

```
Mixin composition follows ordinary class inheritance.
If applied to a sealed parent, subclasses inherit the composed methods.
```

---

# 14. Missing: interaction with method interception

The spec says mixins cannot declare interception.

Correct.

But there is a subtle issue:

Suppose:

```
@compose(DebugPrintable)
class Proxy {
  intercept(message, proceed) {}
}
```

A mixed method call:

```
proxy.debugString()
```

Does interception happen before method lookup?

Earlier we discussed Phalcom's future `intercept` as wrapping dispatch.

The likely rule:

```
Mixin methods are indistinguishable from ordinary host methods after installation.

Therefore interception sees them exactly like normal methods.
```

This should be explicit.

---

# 15. Missing: dynamic class construction

Phalcom is reflective.

Can users dynamically create:

```
Class.new(
  name: "DynamicUser",
  mixins: [...]
)
```

?

The spec currently assumes compile-time composition.

Recommendation:

State:

```
Version one requires mixin composition to occur during declaration construction.

Dynamic mixin application is not supported.
```

Otherwise runtime method table mutation becomes a huge VM concern.

---

# 16. Missing: serialization/debug representation

Because mixins are first-class reflective objects:

What does:

```
String(DebugPrintable)
```

produce?

Recommendation:

```
Mixin(DebugPrintable)
Mixin(Enumerable<Int>)
```

not:

```
class DebugPrintable
```

because it is not a class.

---

# 17. Missing: import/export/module behavior

A mixin is a named declaration.

Questions:

Can you export:

```
export mixin DebugPrintable
```

?

Can another module import it?

Recommendation:

Yes.

The module system should treat mixins similarly to classes and protocols:

```
exported declaration kinds:
- class
- protocol
- mixin
- function
- value
```

---

# 18. Missing: inheritance of mixin metadata

Example:

```
@compose(DebugPrintable)
class Base {}

class Child is Base {}
```

Reflection:

Should:

```
Child.directMixins
```

be:

```
[]
```

and:

```
Child.effectiveMixins
```

be:

```
[DebugPrintable]
```

Yes.

This distinction exists in the spec but should be formally tested.

---

# 19. One potential inconsistency: `@compose` on ordinary classes vs the earlier "protect limited syntax" goal

We discussed:

> protect the appearance of limited class syntax and everything else gets derived from classes.

The current:

```
@compose(...)
class C {}
```

is good.

However, there is one remaining question:

Should a class without composition metadata even have a declaration attribute object?

Probably yes internally.

Reflection:

```
C.composition
```

could return:

```
Composition.none
```

rather than requiring every class to have a special case.

---

# Missing acceptance tests

The spec has many tests, but I would add these specifically:

## Mixin calls another mixin

```
@mixin
class A {
  foo() {
    self.bar()
  }
}

@mixin
class B {
  bar() {}
}

@compose(A, B)
class C {}
```

Expected:

pass.

---

## Mixin private conflict

```
@mixin
class A {
  private helper(){}
}

@mixin
class B {
  private helper(){}
}

@compose(A,B)
class C {}
```

Expected:

compile error.

---

## Superclass mixin override

```
@compose(A)
class Parent {}

@compose(B)
class Child is Parent {}
```

Expected:

B wins.

---

## Class-side mixin rejection

```
@mixin
class Factory {
  class make(){}
}
```

Expected:

compile error.

---

## Generic specialization reflection

```
@compose(Enumerable<Int>)
class Numbers {}
```

Expected:

```
Numbers.mixins[0].arguments == [Int]
```

---

# Overall assessment

The spec is ~90% complete. The architecture is consistent with the decisions we made.

The biggest missing ratifications are:

1. **No runtime mixin chain — flattening is semantic, not delegation.**
2. **Mixed methods have dual identity: dispatch owner vs origin.**
3. **Mixin-to-mixin calls through `self` work dynamically, but require declared capabilities for static guarantees.**
4. **Private mixin methods participate in collision rules.**
5. **Mixin composition is instance-side only in v1.**
6. **Class variables, class methods, constructors, fields, invariants are explicitly forbidden.**
7. **Mixin applications are specialized first-class descriptors (`Enumerable<Int>`).**
8. **Superclass-inherited mixins and subclass mixins follow ordinary inheritance precedence.**
9. **Operator selectors are normal mixin methods.**
10. **Dynamic runtime mixin application is out of scope.**

With those additions, I would consider the mixin specification internally closed and aligned with the rest of Phalcom's object model.
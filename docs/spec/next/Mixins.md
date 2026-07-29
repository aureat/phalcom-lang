# Phalcom Mixin Feature Specification

## Status

This document specifies the proposed mixin feature for Phalcom.

The feature preserves Phalcom’s single-inheritance object model:

```
class Child is Parent {
  ...
}
```

A class has at most one nominal superclass. Mixins provide horizontal implementation reuse without introducing multiple class inheritance, constructor linearization, duplicated base-class state, or a Python-style method-resolution order.

The normative keywords **must**, **must not**, **should**, and **may** are used in their conventional specification sense.

---

# 1. Goals

The mixin feature provides:

1. reusable method implementations across unrelated classes;
    
2. explicit structural requirements on consuming classes;
    
3. generic reusable implementations;
    
4. composition of mixins into classes and other mixins;
    
5. deterministic method-conflict detection;
    
6. complete reflective metadata;
    
7. no change to nominal ancestry;
    
8. no runtime method-resolution order among mixins;
    
9. no implicit instance storage or constructor participation;
    
10. compatibility with protocols, abstract classes, generics, decorators, and Phalcom’s selector model.
    

A mixin is intended for behavior that can be implemented in terms of the consuming object’s messages.

---

# 2. Non-goals

The initial mixin feature does not provide:

- multiple class inheritance;
    
- mixin-owned instance fields;
    
- mixin constructors;
    
- cooperative constructor chaining;
    
- mixin finalizers;
    
- mixin participation in `super`;
    
- source-order method precedence;
    
- runtime insertion or removal of mixins;
    
- per-instance mixin composition;
    
- automatic delegation to helper objects;
    
- nominal subtyping through mixin application;
    
- automatic protocol conformance merely because a mixin is applied;
    
- mixin-defined dispatch interception;
    
- mixin-defined `doesNotUnderstand`.
    

Stateful reusable behavior should use ordinary object composition or require state-access selectors from its host.

---

# 3. Terminology

## 3.1 Mixin declaration

A class-shaped declaration marked with `@mixin`:

```
@mixin
class DebugPrintable {
  ...
}
```

The declaration creates a first-class `Mixin` descriptor rather than an instantiable class.

## 3.2 Mixin application

A reference to a mixin with any required generic arguments:

```
DebugPrintable
Enumerable<Int>
MappingOperations<String, User>
```

## 3.3 Host

The class or mixin into which a mixin application is composed.

## 3.4 Direct mixin

A mixin listed directly in the host declaration.

## 3.5 Transitive mixin

A mixin reached through another mixin’s `with:` composition list.

## 3.6 Effective mixin set

The flattened, deduplicated set of all direct and transitive mixin applications on a host.

## 3.7 Host requirement

A protocol type listed positionally in `@mixin(...)`.

```
@mixin(Iterable<T>, Sized)
```

The eventual host must structurally satisfy these protocols.

## 3.8 Contributed method

A method declared by a mixin and installed into a host’s effective method table.

---

# 4. Declaration syntax

## 4.1 Requirement-free mixin

```
@mixin
class DebugPrintable {
  debugString -> String {
    return self.class.name
  }
}
```

The parameterless spelling is canonical.

This spelling is invalid:

```
@mixin()
class DebugPrintable {
  ...
}
```

An empty argument list must be omitted.

## 4.2 Mixin with host requirements

```
@mixin(Iterable<T>, Sized)
class Enumerable<T> {
  ...
}
```

Positional arguments to `@mixin(...)` are protocol types required from the eventual host.

## 4.3 Mixin composing other mixins

```
@mixin(
  Comparable<T>,
  with: [
    EqualityFromComparison<T>,
    MinMaxOperations<T>
  ]
)
class OrderingOperations<T> {
  ...
}
```

The optional `with:` label accepts a non-empty list of mixin applications.

## 4.4 Requirement-free composed mixin

```
@mixin(
  with: [
    DebugPrintable,
    JsonPrintable
  ]
)
class StandardPrintable {
  ...
}
```

A mixin may compose other mixins without declaring direct protocol requirements.

## 4.5 Cardinality

For a mixin declaration:

```
@mixin occurrences:                exactly 1
positional protocol requirements:  0..unbounded
with: labels:                      0..1
with: list items when present:     1..unbounded
```

The language imposes no small fixed maximum on requirements or composed mixins.

An implementation may reject declarations that exceed a general implementation resource limit, but it must not define a special semantic limit such as eight requirements or sixteen included mixins.

---

# 5. Mixin descriptor semantics

A declaration such as:

```
@mixin(Iterable<T>)
class Enumerable<T> {
  ...
}
```

creates a first-class reflective mixin descriptor bound to `Enumerable`.

It does not create:

- an ordinary class object;
    
- an instantiable type;
    
- an instance layout;
    
- a constructor table;
    
- a superclass;
    
- a metaclass instance family;
    
- a nominal subtype relationship.
    

The following operation is invalid:

```
Enumerable.new()
```

A conforming implementation must report that mixins cannot be instantiated.

Conceptually, the descriptor contains:

```
MixinDescriptor(
  name: #Enumerable,
  typeParameters: const [T],
  directRequirements: const [Iterable<T>],
  directMixins: const [],
  directMethods: const [...],
  attributes: const [...],
  source: ...
)
```

This conceptual structure does not require a literal compiler lowering to ordinary Phalcom source.

---

# 6. Generic mixins

Mixins may declare generic parameters:

```
@mixin(Iterable<T>)
class Enumerable<T> {
  toList -> List<T> {
    ...
  }
}
```

A consuming declaration must apply the mixin with a valid number of type arguments:

```
@compose(Enumerable<Int>)
class IntegerBag {
  ...
}
```

This is invalid:

```
@compose(Enumerable)
class IntegerBag {
  ...
}
```

when `Enumerable` requires one type argument and no generic inference rule applies.

Generic application substitutes type parameters into:

- host requirements;
    
- composed mixin applications;
    
- method parameter annotations;
    
- method return annotations;
    
- method-level generic constraints;
    
- reflection metadata.
    

For:

```
@mixin(Iterable<T>)
class Enumerable<T> {
  firstOr(default: T) -> T {
    ...
  }
}
```

the application:

```
Enumerable<Int>
```

contributes the effective signature:

```
firstOr(default: Int) -> Int
```

Generic application does not create a new nominal class.

Equivalent applied mixin descriptors should be canonicalized:

```
Enumerable<Int> == Enumerable<Int>
// true
```

Canonical identity is recommended:

```
Enumerable<Int> === Enumerable<Int>
// preferably true
```

---

# 7. Host requirements

## 7.1 Requirement declaration

Positional arguments to `@mixin(...)` are structural requirements:

```
@mixin(Iterable<T>, Sized)
class Enumerable<T> {
  ...
}
```

Each positional argument must resolve to a protocol type.

Valid:

```
@mixin(Iterable<T>)
class Enumerable<T> {
  ...
}
```

Invalid:

```
@mixin(BaseCollection<T>)
class Enumerable<T> {
  ...
}
```

when `BaseCollection<T>` is a class.

Invalid:

```
@mixin(DebugPrintable)
class Enumerable<T> {
  ...
}
```

when `DebugPrintable` is a mixin.

A compiler should diagnose the expected descriptor category:

```
@mixin positional arguments must be protocol requirements.

BaseCollection<T> is a class.
Describe the required host behavior through a protocol.
```

## 7.2 Requirement meaning

A host requirement means:

> The final effective host must structurally satisfy the specialized protocol after inheritance, declaration derivation, and mixin composition have been resolved.

Requirements do not mean:

- the mixin inherits from the protocol;
    
- the mixin nominally conforms to the protocol;
    
- the mixin supplies the protocol’s requirements;
    
- runtime dispatch must pass through the protocol descriptor.
    

## 7.3 Requirement validation point

Requirements are validated after the compiler has collected:

1. methods inherited through `is`;
    
2. methods explicitly declared by the host;
    
3. methods generated by declaration attributes;
    
4. methods contributed by direct mixins;
    
5. methods contributed by transitive mixins;
    
6. compatible native or primitive methods.
    

This allows one mixin to help satisfy another mixin’s requirements.

## 7.4 Transitive requirements

A mixin inherits the requirements of every composed mixin.

```
@mixin(Comparable<T>)
class EqualityFromComparison<T> {
  ...
}

@mixin(
  Iterable<T>,
  with: [
    EqualityFromComparison<T>
  ]
)
class EnumerableOrdering<T> {
  ...
}
```

The effective requirements of `EnumerableOrdering<T>` are:

```
Iterable<T>
Comparable<T>
```

Equivalent requirements are deduplicated.

## 7.5 Requirement conflicts

Two requirements for the same protocol origin with incompatible applications may make a mixin application unsatisfiable.

Example:

```
@mixin(
  Iterable<Int>,
  Iterable<String>
)
class InvalidEnumerable {
  ...
}
```

This declaration is not automatically invalid merely because the protocol origin is repeated. It is invalid when the type system determines that no host can simultaneously satisfy both applications under the protocol’s variance and selector requirements.

The compiler should diagnose incompatible effective requirements during mixin validation.

## 7.6 Use of required selectors

A mixin may send messages guaranteed by its requirements:

```
@protocol
class Named {
  name -> String
}

@mixin(Named)
class NamedDescription {
  description -> String {
    return "\(self.class.name)(name: \(self.name))"
  }
}
```

The static checker may type-check `self.name` against `Named`.

A mixin may also send selectors not declared by a requirement, because Phalcom remains dynamic. Such sends should receive a checker warning or error according to the active checking mode.

The runtime must not insert protocol checks on every send.

---

# 8. Allowed mixin members

A mixin may declare:

- concrete instance methods;
    
- getters;
    
- ordinary operator methods;
    
- generic methods;
    
- private helper methods;
    
- method-level contracts;
    
- method-level metadata attributes;
    
- class-side constants belonging to the descriptor;
    
- documentation;
    
- generic type parameters.
    

Example:

```
@mixin(Comparable<T>)
class EqualityFromComparison<T> {
  ==(other: T) -> Bool {
    return self.compare(other) == Ordering.equal
  }

  !=(other: T) -> Bool {
    return not (self == other)
  }
}
```

A mixin method may use:

- `self`;
    
- ordinary dynamic message sends;
    
- lexical constants;
    
- generic parameters;
    
- blocks;
    
- local variables;
    
- contracts;
    
- other ordinary method behavior.
    

---

# 9. Forbidden mixin members

A version-one mixin must not declare:

- instance fields;
    
- mutable or constant instance slots;
    
- constructors;
    
- abstract methods;
    
- a superclass through `is`;
    
- variants;
    
- instance initialization hooks;
    
- instance destruction or finalization hooks;
    
- allocation hooks;
    
- `intercept`;
    
- `doesNotUnderstand`;
    
- methods designated as class constructors;
    
- host-layout directives.
    

Invalid field declaration:

```
@mixin
class Cached {
  var _cache: Map
}
```

Invalid constructor:

```
@mixin
class Cached {
  @constructor
  new() {
    ...
  }
}
```

Invalid nominal superclass:

```
@mixin
class Cached is BaseCache {
  ...
}
```

Invalid abstract member:

```
@mixin
class Enumerable<T> {
  @abstract
  each(block: [T] -> Any) -> None
}
```

The required selector must instead be represented by a protocol:

```
@protocol
class Iterable<out T> {
  each(block: [T] -> Any) -> None
}

@mixin(Iterable<T>)
class Enumerable<T> {
  ...
}
```

---

# 10. Stateful reusable behavior

Mixins have no instance storage.

Reusable behavior requiring host state must define that state through protocol-accessible operations.

```
@protocol
class CloseState {
  closed -> Bool
  markClosed() -> None
  closeUnderlying() -> None
}
```

```
@mixin(CloseState)
class CloseOnce {
  close() {
    if self.closed {
      return
    }

    self.markClosed()
    self.closeUnderlying()
  }
}
```

The host owns the representation:

```
@abstract(
  with: [CloseOnce]
)
class BaseResource {
  var _closed: Bool

  @constructor
  new() {
    _closed = false
  }

  closed -> Bool {
    return _closed
  }

  markClosed() {
    _closed = true
  }

  @abstract
  closeUnderlying() -> None
}
```

A behavior with an independent lifecycle, replaceable implementation, or complex mutable state should use ordinary composition:

```
class CachedRepository {
  const _cache: Cache
  const _source: Repository
}
```

Mixins must not be used to conceal collaborator objects or lifecycle ownership.

---

# 11. Composing mixins into concrete classes

Concrete classes consume mixins through `@compose(...)`.

```
@compose(
  Enumerable<Int>,
  DebugPrintable,
  conforms: [
    Iterable<Int>,
    Sized
  ]
)
class IntegerBag {
  ...
}
```

Positional arguments to `@compose(...)` are mixin applications.

The optional `conforms:` list contains explicit protocol claims.

## 11.1 Mixin-only composition

```
@compose(DebugPrintable)
class User {
  ...
}
```

## 11.2 Conformance-only composition metadata

```
@compose(
  conforms: [
    Hashable,
    Equatable<User>
  ]
)
class User {
  ...
}
```

## 11.3 Combined composition

```
@compose(
  EqualityFromComparison<Version>,
  DebugPrintable,
  conforms: [
    Comparable<Version>,
    Equatable<Version>
  ]
)
class Version {
  ...
}
```

## 11.4 Cardinality

For `@compose(...)`:

```
decorator occurrences:        0..1
positional mixin applications: 0..unbounded
conforms: labels:              0..1
conforms: items when present:  1..unbounded
minimum combined item count:   1
```

These forms are invalid:

```
@compose
class User {}
```

```
@compose()
class User {}
```

```
@compose(conforms: [])
class User {}
```

A class with no mixins and no explicit conformance simply omits `@compose`.

---

# 12. Composing mixins into abstract classes

Abstract classes consume mixins through the labeled `with:` parameter of `@abstract(...)`.

```
@abstract(
  with: [
    CloseOnce,
    DebugPrintable
  ],
  conforms: [
    Stream<T>
  ]
)
class BaseStream<T> is Resource {
  ...
}
```

The `with:` list accepts mixin applications.

The `conforms:` list accepts protocol types.

Mixin requirements may be satisfied by:

- concrete methods declared by the abstract class;
    
- abstract method declarations;
    
- methods inherited through `is`;
    
- methods supplied by other mixins;
    
- generated methods.
    

An abstract declaration may provisionally satisfy a protocol or mixin requirement through compatible abstract methods. Every concrete subclass must ultimately resolve the corresponding abstract selectors.

---

# 13. Composing mixins into mixins

A mixin composes other mixins through its `with:` list.

```
@mixin(
  Comparable<T>,
  with: [
    EqualityFromComparison<T>
  ]
)
class OrderingOperations<T> {
  <(other: T) -> Bool {
    return self.compare(other) == Ordering.less
  }
}
```

Mixin composition is transitive.

A host applying `OrderingOperations<Int>` receives:

- methods declared directly by `OrderingOperations<Int>`;
    
- methods contributed by `EqualityFromComparison<Int>`;
    
- methods from any further transitive mixins.
    

The host also inherits all effective requirements.

---

# 14. Mixin composition graph

The compiler must construct a directed graph of mixin applications.

An edge:

```
A → B
```

means mixin `A` directly composes mixin `B`.

The graph must be acyclic.

Invalid:

```
@mixin(with: [B])
class A {}

@mixin(with: [C])
class B {}

@mixin(with: [A])
class C {}
```

Diagnostic:

```
cyclic mixin composition

A → B → C → A
```

The diagnostic should include the source location of every edge in the cycle.

Generic applications participate in cycle detection by origin and substitution. A cycle cannot be hidden by repeatedly applying the same mixin with changing type arguments.

---

# 15. Flattening semantics

Mixins are flattened into the host’s effective method table during declaration construction.

They are not searched through a separate runtime mixin chain.

For:

```
@compose(Enumerable<Int>)
class IntegerBag {
  ...
}
```

the runtime class `IntegerBag` receives effective method entries corresponding to the methods contributed by `Enumerable<Int>`.

Flattening must preserve method-origin metadata.

A flattened method behaves as an ordinary instance method of the host for dispatch purposes.

This means:

- normal method lookup finds it;
    
- interception observes it as an ordinary method;
    
- reflection can enumerate it;
    
- inline caches may cache it normally;
    
- `doesNotUnderstand` is not invoked for its selector;
    
- overriding it in a subclass uses ordinary class inheritance semantics.
    

---

# 16. Method lookup precedence

The effective lookup order is:

```
1. Methods explicitly declared by the receiver’s class
2. Methods contributed to that class by mixins
3. Methods inherited from the superclass through is
4. doesNotUnderstand
```

Class-declared methods always have final authority over mixin contributions on the same class declaration.

Mixin methods override inherited superclass methods when the class directly composes the mixin.

Example:

```
class Base {
  describe -> String {
    return "base"
  }
}

@mixin
class Description {
  describe -> String {
    return "mixed"
  }
}

@compose(Description)
class Child is Base {}
```

Then:

```
Child.new().describe
// "mixed"
```

A directly declared host method wins over both:

```
@compose(Description)
class Child is Base {
  describe -> String {
    return "child"
  }
}
```

---

# 17. Selector identity

Mixin conflicts and overrides are determined by complete selector identity.

Phalcom selector identity includes:

- base method name;
    
- positional structure;
    
- labels;
    
- arity;
    
- getter or callable form;
    
- operator structure where applicable.
    

These selectors do not conflict:

```
write(value:)
write(_, to:)
write()
write
```

These selectors conflict:

```
write(value:)
write(value:)
```

Type annotations do not create separate runtime overload slots when the selectors are otherwise identical.

---

# 18. Method conflicts

## 18.1 Conflict definition

A conflict occurs when two distinct effective mixin contributions provide the same complete selector to one host.

```
@mixin
class JsonPrintable {
  toString -> String {
    ...
  }
}

@mixin
class DebugPrintable {
  toString -> String {
    ...
  }
}
```

This declaration contains a conflict:

```
@compose(
  JsonPrintable,
  DebugPrintable
)
class Document {
  ...
}
```

## 18.2 Conflict resolution

A host explicitly resolves the conflict by declaring the selector itself:

```
@compose(
  JsonPrintable,
  DebugPrintable
)
class Document {
  toString -> String {
    return self.toJson
  }
}
```

The host method replaces all conflicting mixin candidates for that selector.

## 18.3 No source-order precedence

Mixin order must not select a winner.

These declarations are semantically equivalent:

```
@compose(A, B)
class C {}
```

```
@compose(B, A)
class C {}
```

If `A` and `B` conflict, both declarations are invalid unless `C` declares the selector.

## 18.4 Compatible identical origins

The same mixin method reached through multiple transitive paths is not a conflict when it represents the same canonical mixin application and method origin.

Example:

```
A includes Common
B includes Common
C composes A and B
```

`Common` should be deduplicated in `C`’s effective mixin set.

## 18.5 Duplicate direct applications

Exact duplicate direct applications are invalid:

```
@compose(
  DebugPrintable,
  DebugPrintable
)
class User {}
```

Diagnostic:

```
duplicate mixin application DebugPrintable
```

This is rejected rather than silently deduplicated because it is most likely a declaration error.

## 18.6 Different applications of the same generic mixin

Different generic applications are distinct:

```
@compose(
  Converter<Int>,
  Converter<String>
)
class Value {}
```

This is valid only if:

- both applications’ requirements are satisfiable;
    
- their contributed selector sets do not conflict;
    
- their effective type relationships are coherent.
    

Because runtime selectors are not distinguished only by type annotations, generic applications will often conflict when they contribute the same selectors.

The compiler must diagnose those conflicts normally.

---

# 19. `self` semantics

Inside a contributed mixin method, `self` is the receiving host instance.

```
@mixin(Named)
class NamedDescription {
  description -> String {
    return self.name
  }
}
```

When applied to `User`, `self` is a `User`.

Mixin methods are not invoked on mixin descriptor instances.

A method may dynamically dispatch to a host override:

```
@protocol
class Named {
  name -> String
}

@mixin(Named)
class Greeting {
  greeting -> String {
    return "Hello, \(self.name)"
  }
}
```

```
@compose(Greeting)
class User {
  name -> String {
    return "Altun"
  }
}
```

Calling `User.new().greeting` sends `name` to the `User` object.

---

# 20. `super` semantics

A mixin method must not contain a `super` send.

```
@mixin
class LoggingClose {
  close() {
    super.close()
  }
}
```

This is a compile-time error.

A mixin has no nominal parent and does not participate in the class inheritance chain. Defining `super` for mixins would require ordering mixins into a method-resolution chain, which the feature explicitly avoids.

A mixin that wraps host behavior must require an explicitly named primitive selector:

```
@protocol
class RawClosable {
  closeUnderlying() -> None
}

@mixin(RawClosable)
class LoggingClose {
  close() {
    Log.info("closing")
    self.closeUnderlying()
  }
}
```

A normal class method retains ordinary `super` semantics and searches the superclass established by `is`.

---

# 21. Privacy and member access

Mixin methods must not receive privileged direct access to the host’s fields merely because they are flattened into the host.

A mixin should interact with the host through selectors declared by protocols or otherwise available dynamically.

This should be invalid or inaccessible:

```
@mixin
class CounterHelpers {
  increment() {
    _count++
  }
}
```

when `_count` is a private host field not declared by the mixin.

The correct design is:

```
@protocol
class MutableCount {
  count -> Int
  count(value: Int) -> None
}

@mixin(MutableCount)
class CounterHelpers {
  increment() {
    self.count(self.count + 1)
  }
}
```

This preserves:

- encapsulation;
    
- structural requirements;
    
- independent host representation;
    
- reliable static checking;
    
- meaningful reflection.
    

Private methods declared by a mixin remain private implementation details of that contribution. Their collision rules follow the language’s ordinary private-selector identity rules.

---

# 22. Protocol conformance

Applying a mixin does not by itself create nominal protocol membership.

Phalcom protocols remain structural.

A class may structurally conform because of methods supplied by a mixin:

```
@protocol
class Equatable<T> {
  ==(other: T) -> Bool
}
```

```
@mixin(Comparable<T>)
class EqualityFromComparison<T> {
  ==(other: T) -> Bool {
    return self.compare(other) == Ordering.equal
  }
}
```

```
@compose(
  EqualityFromComparison<Version>,
  conforms: [
    Comparable<Version>,
    Equatable<Version>
  ]
)
class Version {
  compare(other: Version) -> Ordering {
    ...
  }
}
```

The mixed method satisfies the `Equatable<Version>` requirement.

The explicit `conforms:` list requests validation and records design intent. It does not alter dispatch or object representation.

A class may satisfy `Equatable<Version>` without listing it in `conforms:`.

---

# 23. Abstract classes

An abstract class may consume mixins.

```
@abstract(
  with: [
    CloseOnce
  ]
)
class BaseResource {
  ...
}
```

Mixin requirements may be satisfied by abstract methods:

```
@protocol
class CloseState {
  closed -> Bool
  markClosed() -> None
  closeUnderlying() -> None
}

@mixin(CloseState)
class CloseOnce {
  ...
}
```

```
@abstract(
  with: [CloseOnce]
)
class BaseResource {
  closed -> Bool {
    ...
  }

  markClosed() -> None {
    ...
  }

  @abstract
  closeUnderlying() -> None
}
```

This is valid because `BaseResource` represents `closeUnderlying()` as an explicit abstract obligation.

A concrete subclass must implement all remaining abstract selectors.

---

# 24. Inheritance of composed methods

Mixin methods composed into a class become part of that class’s effective method table and are inherited by ordinary subclasses.

```
@compose(DebugPrintable)
class BaseEntity {
  ...
}

class User is BaseEntity {
  ...
}
```

`User` inherits the effective `DebugPrintable` methods through `BaseEntity`.

Reflection must distinguish:

- a mixin applied directly to `BaseEntity`;
    
- a method inherited nominally by `User`;
    
- a mixin applied directly to `User`.
    

A subclass does not need to repeat the mixin application.

If a subclass directly applies the same canonical mixin application already inherited from a superclass, the compiler should reject it as redundant unless a future explicit reapplication feature is introduced.

```
@compose(DebugPrintable)
class BaseEntity {}

@compose(DebugPrintable)
class User is BaseEntity {}
```

Recommended diagnostic:

```
redundant mixin application DebugPrintable

User already inherits this mixin application from BaseEntity.
```

---

# 25. Interaction with `@data`

A data class may consume mixins:

```
@data
@compose(DebugPrintable)
class Point {
  const _x: Int
  const _y: Int
}
```

Generated `@data` methods participate in conflict analysis.

If both `@data` and a mixin contribute the same selector, the compiler must not resolve the conflict based on decorator source order.

Example:

```
@mixin
class StructuralEquality {
  ==(other) -> Bool {
    ...
  }
}
```

```
@data
@compose(StructuralEquality)
class Point {
  const _x: Int
  const _y: Int
}
```

If `@data` also generates `==(_)`, compilation must:

- apply a feature-specific explicit override rule, if one is specified elsewhere; or
    
- diagnose the duplicate selector.
    

The default is to diagnose it.

The user may resolve it with a directly declared method when the relevant derivation system permits overriding generated methods.

---

# 26. Interaction with `@immutable`

Mixins have no storage and therefore do not directly violate host immutability.

```
@immutable
@compose(DebugPrintable)
class Identifier {
  const _value: String
}
```

is valid.

A mixin may still require mutating host capabilities:

```
@mixin(MutableCollection<T>)
class MutableCollectionOperations<T> {
  ...
}
```

Applying this mixin to an immutable class must fail when the requirements or contributed methods are incompatible with immutability.

The compiler should validate immutability after mixin composition so contributed methods cannot bypass immutable-class restrictions.

Mixins must not create hidden mutable state.

---

# 27. Interaction with `@sealed`

Mixins do not affect nominal hierarchy membership.

```
@sealed
@abstract
class Expression {
  ...
}
```

A subclass may consume mixins:

```
@compose(DebugPrintable)
class Literal is Expression {
  ...
}
```

`DebugPrintable` is not:

- a superclass;
    
- a sealed-family member;
    
- a variant;
    
- an exhaustiveness branch.
    

Sealed-family analysis only follows nominal `is` relationships and declared variant semantics.

A mixin declaration itself must not be marked `@sealed`.

---

# 28. Interaction with variants

A variant class may consume mixins when variant expansion permits class-level composition metadata.

```
@compose(DebugPrintable)
class Ok<T> is Result<T, Nothing> {
  ...
}
```

Mixin application does not change:

- the variant tag;
    
- the sealed parent;
    
- pattern-matching exhaustiveness;
    
- generated deconstruction behavior.
    

Generated variant methods and mixin methods participate in ordinary selector-conflict analysis.

---

# 29. Interaction with dispatch interception

A contributed mixin method is an ordinary effective method for dispatch purposes.

If a host defines `intercept`, a call to a mixed method passes through the host’s normal interception semantics.

Mixins themselves must not declare `intercept`.

This restriction ensures that applying a mixin cannot silently change the behavior of every message sent to the host.

A host class may explicitly declare interception while also consuming mixins:

```
@compose(DebugPrintable)
class Proxy {
  intercept(message, proceed) {
    ...
  }
}
```

The interception mechanism sees the mixed method according to the same rules as any other host method.

---

# 30. Interaction with `doesNotUnderstand`

Mixed methods are installed before runtime fallback dispatch.

The lookup order is:

```
class-declared method
mixin-contributed method
superclass method
doesNotUnderstand
```

A mixin must not declare `doesNotUnderstand`.

A host may declare it explicitly.

This prevents a mixin from implicitly claiming an open-ended selector set or weakening static diagnostics.

---

# 31. Interaction with contracts

Mixin methods may use ordinary method contracts:

```
@mixin(Sized)
class NonEmptyOperations {
  @requires(self.size > 0)
  firstRequired() {
    ...
  }
}
```

Contracts belong to the contributed method and remain visible through reflection.

Mixin host requirements must not use `@requires`. They are descriptor-level protocol constraints declared positionally in `@mixin(...)`.

These concepts remain distinct:

```
@mixin(Iterable<T>)
    Composition-time structural host requirement.

@requires(condition)
    Runtime or contract-weaving precondition on a method.
```

---

# 32. Declaration processing

Decorator source order must not determine mixin semantics.

These declarations are equivalent:

```
@immutable
@compose(DebugPrintable)
class Identifier {
  ...
}
```

```
@compose(DebugPrintable)
@immutable
class Identifier {
  ...
}
```

The compiler should process class-shaped declarations in the following semantic phases:

1. Parse the class declaration and attributes.
    
2. Determine whether the declaration represents a class, protocol, or mixin.
    
3. Resolve generic parameters and bounds.
    
4. Resolve the optional nominal superclass through `is`.
    
5. Resolve protocol inclusion.
    
6. Expand structural derivations such as `@data`.
    
7. Resolve direct mixin applications.
    
8. Recursively flatten transitive mixin applications.
    
9. Detect duplicate and cyclic applications.
    
10. Substitute generic arguments.
    
11. Collect contributed methods.
    
12. Merge effective selectors.
    
13. Diagnose unresolved method conflicts.
    
14. Validate mixin host requirements.
    
15. Validate explicit protocol conformance.
    
16. Validate abstract obligations.
    
17. Validate immutability, sealing, and variant constraints.
    
18. Construct the final method table.
    
19. Emit reflection metadata.
    

A compiler may internally combine phases for efficiency, but observable semantics must match this ordering.

---

# 33. Runtime representation

A conforming runtime may implement mixin flattening through:

- copied method-table entries;
    
- shared method bodies with host installation metadata;
    
- compiler-generated forwarding entries;
    
- another optimized representation.
    

The representation must preserve these observable properties:

1. mixed methods participate in ordinary host dispatch;
    
2. `self` is the host instance;
    
3. no mixin object is allocated for each host instance;
    
4. mixins create no instance fields;
    
5. mixins create no nominal parent relationship;
    
6. reflection identifies the declaring mixin;
    
7. method lookup follows the specified precedence;
    
8. source-order mixin precedence does not exist;
    
9. conflicts are diagnosed before ordinary execution;
    
10. a mixed method can be optimized like a class method.
    

The runtime should avoid physically duplicating machine code for every host. Multiple method-table entries may reference one compiled method body with specialized type metadata.

---

# 34. Reflection

## 34.1 Mixin descriptor reflection

A mixin descriptor should expose:

```
Mixin.name
Mixin.typeParameters
Mixin.directRequirements
Mixin.effectiveRequirements
Mixin.directMixins
Mixin.effectiveMixins
Mixin.directMethods
Mixin.effectiveMethods
Mixin.attributes
Mixin.source
```

For an applied generic mixin:

```
const applied = Enumerable<Int>

applied.origin
// Enumerable

applied.arguments
// const [Int]

applied.requirements
// const [Iterable<Int>, Sized]
```

## 34.2 Class reflection

A class should expose:

```
Class.directMixins
Class.effectiveMixins
Class.inheritedMixins
Class.mixinApplications
Class.methodsFrom(mixin:)
Class.declaredConformances
Class.effectiveConformances
```

Suggested semantics:

```
IntegerBag.directMixins
// const [Enumerable<Int>, DebugPrintable]
```

```
IntegerBag.effectiveMixins
// flattened and deduplicated list
```

```
IntegerBag.methodsFrom(Enumerable<Int>)
// contributed methods installed on IntegerBag
```

## 34.3 Method reflection

A contributed method should expose both source and installation metadata:

```
const method = IntegerBag.methodFor(#toList)

method.declaredBy
// Enumerable

method.installedOn
// IntegerBag

method.mixinApplication
// Enumerable<Int>

method.isMixed
// true
```

A subclass inheriting that method should expose:

```
method.declaredBy
// Enumerable

method.installedOn
// IntegerBag

method.inheritedBy
// subclass-dependent reflection, if requested
```

Reflection must not pretend that the method was textually declared by the host.

---

# 35. Diagnostics

Mixin diagnostics should identify:

- the consuming declaration;
    
- the relevant mixin applications;
    
- the complete selector involved;
    
- the source location of each contribution;
    
- the failed protocol requirement;
    
- the generic substitution;
    
- the composition path for transitive mixins.
    

## 35.1 Unsatisfied requirement

```
error: mixin requirement is not satisfied

IntegerBag composes Enumerable<Int>.
Enumerable<Int> requires Iterable<Int>.

Missing selector:
  #each(_)

Required signature:
  each(block: [Int] -> Any) -> None
```

## 35.2 Method conflict

```
error: conflicting mixed selector #toString

Document receives #toString from:
  JsonPrintable at json_printable.ph:8
  DebugPrintable at debug_printable.ph:11

Declare #toString directly on Document to resolve the conflict.
```

## 35.3 Invalid descriptor kind

```
error: @compose positional arguments must be mixins

Iterable<Int> is a protocol.
Place it in conforms: [...] instead.
```

## 35.4 Forbidden mixin state

```
error: mixins cannot declare instance fields

Field:
  var _cache: Map

Declare host state through a protocol requirement or use object composition.
```

## 35.5 Forbidden `super`

```
error: super is unavailable in mixin methods

LoggingClose is not part of a nominal superclass chain.
Require and invoke an explicit host operation instead.
```

## 35.6 Cycle

```
error: cyclic mixin composition

A → B → C → A

Composition edges:
  A with B at a.ph:4
  B with C at b.ph:7
  C with A at c.ph:3
```

---

# 36. Positive examples

## 36.1 Enumerable behavior

```
@protocol
class Iterable<out T> {
  each(block: [T] -> Any) -> None
}

@protocol
class Sized {
  size -> Int
}

@mixin(Iterable<T>, Sized)
class Enumerable<T> {
  toList -> List<T> {
    const result = List.new()

    self.each { item =>
      result.add(item)
    }

    return result
  }

  empty -> Bool {
    return self.size == 0
  }

  any(predicate: [T] -> Bool) -> Bool {
    self.each { item =>
      if predicate.call(item) {
        return true
      }
    }

    return false
  }
}
```

```
@compose(
  Enumerable<Int>,
  conforms: [
    Iterable<Int>,
    Sized
  ]
)
class IntegerRange {
  const _start: Int
  const _end: Int

  @constructor
  new(start: Int, end: Int) {
    _start = start
    _end = end
  }

  size -> Int {
    return (_end - _start).max(0)
  }

  each(block: [Int] -> Any) -> None {
    var current = _start

    while current < _end {
      block.call(current)
      current++
    }
  }
}
```

## 36.2 Equality from comparison

```
@protocol
class Comparable<T> {
  compare(other: T) -> Ordering
}

@protocol
class Equatable<T> {
  ==(other: T) -> Bool
  !=(other: T) -> Bool
}
```

```
@mixin(Comparable<T>)
class EqualityFromComparison<T> {
  ==(other: T) -> Bool {
    return self.compare(other) == Ordering.equal
  }

  !=(other: T) -> Bool {
    return not (self == other)
  }
}
```

```
@compose(
  EqualityFromComparison<Version>,
  conforms: [
    Comparable<Version>,
    Equatable<Version>
  ]
)
class Version {
  const _major: Int
  const _minor: Int

  compare(other: Version) -> Ordering {
    ...
  }
}
```

## 36.3 Strategy combinators

```
@protocol
class DrawableStrategy<out T> {
  draw(data: DrawData) -> T
}

@protocol(DrawableStrategy<T>)
class Strategy<out T> {
  map<U>(transform: [T] -> U) -> Strategy<U>

  filter(
    predicate: [T] -> Bool
  ) -> Strategy<T>

  flatMap<U>(
    transform: [T] -> Strategy<U>
  ) -> Strategy<U>
}
```

```
@mixin(DrawableStrategy<T>)
class StrategyCombinators<T> {
  map<U>(
    transform: [T] -> U
  ) -> Strategy<U> {
    return MappedStrategy.new(
      source: self,
      transform: transform
    )
  }

  filter(
    predicate: [T] -> Bool
  ) -> Strategy<T> {
    return FilteredStrategy.new(
      source: self,
      predicate: predicate
    )
  }

  flatMap<U>(
    transform: [T] -> Strategy<U>
  ) -> Strategy<U> {
    return FlatMappedStrategy.new(
      source: self,
      transform: transform
    )
  }
}
```

```
@compose(
  StrategyCombinators<Int>,
  conforms: [
    Strategy<Int>
  ]
)
class IntStrategy {
  draw(data: DrawData) -> Int {
    return data.drawInteger(
      min: Int.min,
      max: Int.max
    )
  }
}
```

## 36.4 Mixin composition

```
@mixin(Comparable<T>)
class EqualityFromComparison<T> {
  ...
}
```

```
@mixin(Comparable<T>)
class MinMaxOperations<T> {
  min(other: T) -> T {
    if self.compare(other) == Ordering.greater {
      return other
    }

    return self
  }

  max(other: T) -> T {
    if self.compare(other) == Ordering.less {
      return other
    }

    return self
  }
}
```

```
@mixin(
  Comparable<T>,
  with: [
    EqualityFromComparison<T>,
    MinMaxOperations<T>
  ]
)
class OrderingOperations<T> {
  <(other: T) -> Bool {
    return self.compare(other) == Ordering.less
  }
}
```

---

# 37. Negative examples

## 37.1 Field declaration

```
@mixin
class Invalid {
  var _state: Int
}
```

Invalid: mixins cannot declare instance storage.

## 37.2 Constructor declaration

```
@mixin
class Invalid {
  @constructor
  new() {}
}
```

Invalid: mixins cannot participate in construction.

## 37.3 Class inheritance

```
@mixin
class Invalid is Parent {}
```

Invalid: mixins have no nominal superclass.

## 37.4 Abstract method

```
@mixin
class Invalid {
  @abstract
  run() -> None
}
```

Invalid: host requirements must be expressed through protocols.

## 37.5 Protocol passed as composed implementation

```
@compose(Iterable<Int>)
class Invalid {}
```

Invalid: positional `@compose` items must be mixins.

## 37.6 Mixin passed as requirement

```
@mixin(DebugPrintable)
class Invalid {}
```

Invalid when `DebugPrintable` is a mixin rather than a protocol.

## 37.7 `super` in a mixin

```
@mixin
class Invalid {
  run() {
    super.run()
  }
}
```

Invalid: mixins do not participate in `super`.

## 37.8 Unresolved conflict

```
@compose(A, B)
class Invalid {}
```

Invalid when both `A` and `B` contribute the same selector and the class does not declare it.

---

# 38. Static-checking obligations

A static checker should:

1. validate every mixin decorator argument category;
    
2. validate generic arity and bounds;
    
3. substitute generic arguments through methods and requirements;
    
4. detect cyclic mixin composition;
    
5. detect duplicate direct applications;
    
6. flatten transitive mixin applications;
    
7. compute the effective selector set;
    
8. diagnose conflicts;
    
9. verify host protocol requirements;
    
10. include generated and inherited methods in validation;
    
11. verify explicit `conforms:` claims;
    
12. reject forbidden mixin members;
    
13. reject `super` inside mixin methods;
    
14. enforce immutable-host restrictions after composition;
    
15. preserve method-origin metadata.
    

In a dynamic or unchecked mode, declaration-shape errors, cycles, forbidden members, and selector conflicts remain compile-time errors. Protocol signature compatibility may be deferred only where the type information is unavailable.

---

# 39. Interpreter and VM obligations

The interpreter or VM must:

1. represent mixin descriptors as first-class reflective objects;
    
2. reject attempts to instantiate a mixin;
    
3. resolve and specialize generic mixin applications;
    
4. install contributed methods into host method tables;
    
5. preserve host `self`;
    
6. preserve declaring-mixin metadata;
    
7. use ordinary dispatch for mixed methods;
    
8. avoid adding mixins to nominal ancestry;
    
9. avoid per-instance mixin allocation;
    
10. invalidate method caches when dynamically constructed classes install mixins, if dynamic class construction is supported;
    
11. preserve conflict and requirement diagnostics in dynamic declaration construction;
    
12. expose direct and effective mixin metadata through reflection.
    

---

# 40. Bootstrap obligations

The bootstrap implementation must provide sufficient primitive support for:

- constructing mixin descriptors;
    
- applying generic arguments;
    
- reading mixin requirements;
    
- reading direct composed mixins;
    
- reading mixin methods;
    
- flattening composition graphs;
    
- installing method entries;
    
- preserving source metadata;
    
- querying method origin;
    
- preventing mixin instantiation.
    

The visible Phalcom reflection contract is normative even when selected operations are implemented natively.

---

# 41. Conformance tests

A conforming implementation should include tests for at least the following.

## 41.1 Declaration

- parameterless mixin;
    
- mixin with one requirement;
    
- mixin with multiple requirements;
    
- generic mixin;
    
- requirement-free composed mixin;
    
- mixin composing several mixins.
    

## 41.2 Validation

- class used as requirement is rejected;
    
- mixin used as requirement is rejected;
    
- protocol used in `@compose` positional arguments is rejected;
    
- class used in `@compose` positional arguments is rejected;
    
- empty `with:` list is rejected;
    
- duplicate direct application is rejected;
    
- cyclic composition is rejected.
    

## 41.3 Member restrictions

- field declaration is rejected;
    
- constructor is rejected;
    
- abstract method is rejected;
    
- nominal superclass is rejected;
    
- `super` is rejected;
    
- `intercept` is rejected;
    
- `doesNotUnderstand` is rejected.
    

## 41.4 Dispatch

- mixed method is callable;
    
- `self` is the host object;
    
- host method overrides mixed method;
    
- mixed method overrides superclass method;
    
- subclass inherits mixed method;
    
- subclass may override inherited mixed method;
    
- `doesNotUnderstand` is not called for an installed mixed selector.
    

## 41.5 Conflicts

- distinct selectors do not conflict;
    
- same selector from two mixins conflicts;
    
- host declaration resolves conflict;
    
- mixin order does not affect conflict result;
    
- shared transitive mixin is deduplicated;
    
- different generic applications conflict when selectors collide.
    

## 41.6 Requirements

- direct host method satisfies requirement;
    
- inherited method satisfies requirement;
    
- generated method satisfies requirement;
    
- another mixin satisfies requirement;
    
- abstract method provisionally satisfies requirement;
    
- missing requirement is diagnosed;
    
- incompatible signature is diagnosed;
    
- generic substitution appears correctly in diagnostics.
    

## 41.7 Reflection

- direct mixins are reported;
    
- transitive mixins are reported;
    
- inherited mixins are distinguished;
    
- method declaring mixin is reported;
    
- installation host is reported;
    
- applied type arguments are reported;
    
- effective requirements are reported.
    

---

# 42. Canonical examples

## 42.1 Simple mixin

```
@mixin
class DebugPrintable {
  debugString -> String {
    return self.class.name
  }
}
```

```
@compose(DebugPrintable)
class User {
  ...
}
```

## 42.2 Required capability

```
@protocol
class Named {
  name -> String
}
```

```
@mixin(Named)
class NamedDescription {
  description -> String {
    return "\(self.class.name)(name: \(self.name))"
  }
}
```

```
@compose(
  NamedDescription,
  conforms: [Named]
)
class User {
  name -> String {
    return "Altun"
  }
}
```

## 42.3 Stateful host

```
@protocol
class MutableCount {
  count -> Int
  count(value: Int) -> None
}
```

```
@mixin(MutableCount)
class CounterOperations {
  increment() {
    self.count(self.count + 1)
  }

  decrement() {
    self.count(self.count - 1)
  }
}
```

```
@compose(
  CounterOperations,
  conforms: [MutableCount]
)
class Counter {
  var _count: Int

  @constructor
  new() {
    _count = 0
  }

  count -> Int {
    return _count
  }

  count(value: Int) -> None {
    _count = value
  }
}
```

---

# 43. Final semantic model

The mixin feature is defined by the following invariants:

```
A class has zero or one nominal superclass through is.

A mixin is a first-class implementation descriptor, not a class.

A mixin has no instances, fields, constructors, or nominal ancestry.

Positional @mixin arguments are required host protocols.

The with: argument composes other mixins.

Concrete classes consume mixins through @compose(...).

Abstract classes consume mixins through @abstract(with: [...]).

Mixin methods are flattened into the host’s effective method table.

self inside a mixed method is the host instance.

Mixin methods cannot use super.

Mixin order never establishes method precedence.

Selector conflicts are resolved only by an explicit host method.

Mixin application does not create nominal subtyping.

Protocol conformance remains structural.

Mixins may help a host satisfy protocols.

Mixins do not alter sealed-family membership.

Mixins do not own state or participate in construction.

Reflection preserves both method origin and installation host.
```

This design gives Phalcom the principal benefit of multiple inheritance—horizontal implementation reuse—while preserving a simple nominal class tree, predictable dispatch, explicit dependencies, and deterministic composition.
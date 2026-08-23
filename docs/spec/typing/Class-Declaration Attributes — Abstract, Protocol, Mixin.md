# Phalcom Class-Declaration Attributes

## 1. Design decision

Phalcom retains one deliberately small class grammar:

```
class Name<TypeParameters> is OptionalSuperclass {
  ...
}
```

The `is` clause has exactly one meaning:

> Establish the declaration’s single nominal superclass.

Everything else is expressed through class attributes.

The canonical declaration attributes are:

```
@protocol(...)   structural capability declaration
@mixin(...)      reusable implementation declaration
@abstract(...)   incomplete nominal class declaration
@compose(...)    composition metadata for a concrete class
```

The following standalone helper attributes are not part of the canonical design:

```
@needs(...)
@includes(...)
@with(...)
@conforms(...)
```

Their semantics are folded into the primary declaration attributes.

This minimizes declaration noise without combining unrelated descriptor types in one heterogeneous argument list.

---

# 2. Declaration categories

Phalcom has four relevant class-declaration categories.

## 2.1 Ordinary concrete class

```
class User is Object {
  ...
}
```

A concrete class:

- may be instantiated;

- may declare storage and constructors;

- may inherit from zero or one class;

- may consume mixins;

- may explicitly declare protocol conformance;

- must have no unresolved abstract methods.


Composition and explicit conformance use `@compose(...)`.

## 2.2 Protocol declaration

```
@protocol
class Iterable<out T> {
  each(block: [T] -> Any) -> None
}
```

A protocol describes structural behavior. It has no instances, storage, constructors, superclass, or method implementations.

## 2.3 Mixin declaration

```
@mixin(Iterable<T>)
class Enumerable<T> {
  ...
}
```

A mixin supplies concrete methods to consuming classes. It has no instances, storage, constructors, or nominal ancestry.

## 2.4 Abstract class

```
@abstract
class BaseStream<T> is Object {
  ...
}
```

An abstract class is a real nominal class. It may contain state, constructors, concrete methods, and abstract methods, but it cannot be instantiated until all abstract requirements are implemented by a concrete subclass.

---

# 3. Cardinality summary

|Construct|Positional items|Labeled items|Minimum total|Maximum|
|---|---|---|---|---|
|`is Parent`|One class|None|0|1|
|`@protocol(...)`|Included protocols|None|0|Unbounded|
|`@mixin(...)`|Required protocols|`with:` mixins|0|Unbounded|
|`@abstract(...)`|None|`with:`, `conforms:`|0|Unbounded|
|`@compose(...)`|Mixed-in implementations|`conforms:` protocols|1|Unbounded|
|Method `@abstract`|None|None|0|0|

“Unbounded” means no language-level small limit. Implementations may impose a general collection or metadata-size limit, but Phalcom should not specify an arbitrary maximum such as eight mixins or sixteen protocols.

Each declaration attribute may occur at most once on the same declaration.

Invalid:

```
@compose(DebugPrintable)
@compose(Enumerable<Int>)
class Numbers {
  ...
}
```

Valid:

```
@compose(
  DebugPrintable,
  Enumerable<Int>
)
class Numbers {
  ...
}
```

---

# 4. `@protocol`

## 4.1 Purpose

`@protocol` transforms a class-shaped declaration into a first-class structural protocol descriptor.

```
@protocol
class Hashable {
  hash -> Int
}
```

The descriptor records:

- its name;

- generic parameters;

- directly declared requirements;

- included protocols;

- effective requirements;

- attributes and documentation.


It does not create an instantiable class.

```
Hashable.new()
```

is invalid.

## 4.2 Parameters

Positional parameters are protocols whose requirements are included.

```
@protocol(Iterable<T>, Sized)
class Collection<out T> {
  first -> Option<T>
}
```

This means:

```
Collection<T> requires:
  all requirements of Iterable<T>
  all requirements of Sized
  first -> Option<T>
```

The former `@includes(...)` helper is therefore unnecessary.

## 4.3 Cardinality

```
Included protocols: 0..unbounded
Labeled parameters: none
Decorator occurrences: exactly one
```

Valid:

```
@protocol
class Sized {
  size -> Int
}
```

Valid:

```
@protocol(Iterable<T>)
class Sequence<out T> {
  at(index: Int) -> T
}
```

Valid:

```
@protocol(
  Iterable<T>,
  Sized,
  Reversible<T>
)
class Collection<out T> {
  ...
}
```

Invalid:

```
@protocol()
class Sized {
  ...
}
```

The canonical empty form is `@protocol` without parentheses.

Invalid:

```
@protocol(BaseCollection<T>)
class Collection<T> {
  ...
}
```

when `BaseCollection<T>` is a class.

Invalid:

```
@protocol(Enumerable<T>)
class Collection<T> {
  ...
}
```

when `Enumerable<T>` is a mixin.

## 4.4 Allowed contents

A protocol may declare:

```
@protocol
class Mapping<K, out V> {
  get(key: K) -> Option<V>
  contains(key: K) -> Bool

  map<U>(
    transform: [V] -> U
  ) -> Mapping<K, U>
}
```

Protocol methods have signatures but no executable bodies.

A protocol may not declare:

- instance fields;

- constructors;

- concrete instance methods;

- a nominal superclass through `is`;

- mixin composition;

- `intercept`;

- `doesNotUnderstand`;

- allocation or destruction hooks.


## 4.5 Included requirement conflicts

Compatible duplicate requirements are merged.

```
@protocol
class A {
  size -> Int
}

@protocol
class B {
  size -> Int
}

@protocol(A, B)
class C {}
```

This is valid.

Incompatible requirements are rejected:

```
@protocol
class A {
  value -> Int
}

@protocol
class B {
  value -> String
}

@protocol(A, B)
class C {}
```

The compiler must not select one based on argument order.

---

# 5. `@mixin`

## 5.1 Purpose

`@mixin` transforms a class-shaped declaration into a reusable implementation descriptor.

```
@mixin(Named)
class NamedDescription {
  description -> String {
    return "\(self.class)(name: \(self.name))"
  }
}
```

The mixin contributes methods to another declaration. It does not create another superclass or a nominal subtype relationship.

## 5.2 Positional parameters: host requirements

Positional parameters are protocols the eventual consuming class must satisfy.

```
@mixin(
  Iterable<T>,
  Sized
)
class Enumerable<T> {
  ...
}
```

This means:

> `Enumerable<T>` may only be installed on a class whose final effective method set satisfies `Iterable<T>` and `Sized`.

The former `@needs(...)` helper is unnecessary.

Requirements are checked after:

1. inherited methods are resolved;

2. class-declared methods are collected;

3. generated methods are produced;

4. all mixin methods are composed.


This permits one mixin to help satisfy another mixin’s requirements.

## 5.3 Labeled `with:` parameter

A mixin may compose other mixins:

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

The `with:` value is a list of mixin applications.

The former class-level `@with(...)` helper is therefore unnecessary for mixin declarations.

The effective host requirements are accumulated transitively:

```
OrderingOperations<T> requirements =
  its direct Comparable<T> requirement
  + requirements of EqualityFromComparison<T>
  + requirements of MinMaxOperations<T>
```

Equivalent requirements are deduplicated.

## 5.4 Cardinality

```
Required protocols: 0..unbounded
with: mixins: 0..unbounded
Decorator occurrences: exactly one
```

The decorator is valid when both collections are empty:

```
@mixin
class ClassDescription {
  classDescription -> String {
    return self.class.name
  }
}
```

The empty call is not canonical:

```
@mixin()
class ClassDescription {
  ...
}
```

Use `@mixin`.

A requirement-free composed mixin is valid:

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

When `with:` is present, its list must contain at least one item.

Invalid:

```
@mixin(with: [])
class M {
  ...
}
```

## 5.5 Allowed contents

A mixin may declare:

- concrete instance methods;

- private helper methods;

- generic methods;

- method attributes;

- documentation;

- generic type parameters.


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

A mixin may not declare:

- instance fields;

- constructors;

- a superclass through `is`;

- abstract methods;

- class variants;

- `intercept`;

- `doesNotUnderstand`;

- instance initialization or finalization hooks.


Invalid:

```
@mixin
class Cached {
  var _cache: Map
}
```

Invalid:

```
@mixin
class Cached {
  @constructor
  new() {
    ...
  }
}
```

Stateful reusable behavior must use composition or require state-access protocols from its host.

## 5.6 `self` and `super`

Inside a mixin method, `self` is the consuming object.

```
@mixin(Named)
class NamedDescription {
  description -> String {
    return self.name
  }
}
```

A mixed method may dynamically call other host methods through `self`.

A mixin method may not use `super`.

```
@mixin
class LoggingClose {
  close() {
    super.close()
  }
}
```

This is invalid because mixins are not in the nominal superclass chain.

The mixin must require an explicit primitive operation instead:

```
@protocol
class UnderlyingClosable {
  closeUnderlying() -> None
}

@mixin(UnderlyingClosable)
class LoggingClose {
  close() {
    Log.info("closing")
    self.closeUnderlying()
  }
}
```

---

# 6. `@abstract`

## 6.1 Purpose

Class-level `@abstract` declares an incomplete nominal class.

```
@abstract
class BaseStream<T> {
  ...
}
```

It remains a real class:

- it may have storage;

- it may have constructors;

- it may inherit through `is`;

- it may supply concrete methods;

- it participates in nominal subtype checks;

- it may call `super`.


It cannot be instantiated directly.

## 6.2 Labeled `with:` parameter

Abstract classes may consume mixins directly through `@abstract`.

```
@abstract(
  with: [
    CloseOnce,
    DebugPrintable
  ]
)
class BaseResource {
  ...
}
```

There is no separate `@with(...)` decorator.

## 6.3 Labeled `conforms:` parameter

Abstract classes may explicitly declare protocol conformance:

```
@abstract(
  conforms: [
    Stream<T>,
    Closeable
  ]
)
class BaseStream<T> {
  ...
}
```

There is no separate `@conforms(...)` decorator.

An abstract method may satisfy a protocol requirement provisionally:

```
@protocol
class Stream<out T> {
  next -> Option<T>
  close() -> None
}

@abstract(
  conforms: [Stream<T>]
)
class BaseStream<T> {
  close() -> None {
    ...
  }

  @abstract
  next -> Option<T>
}
```

The abstract class is valid. A concrete subclass must implement `next`.

## 6.4 Combined usage

```
@abstract(
  with: [
    CloseOnce,
    DebugPrintable
  ],
  conforms: [
    Stream<T>,
    Closeable
  ]
)
class BaseStream<T> is Resource {
  ...
}
```

## 6.5 Cardinality

```
Positional parameters: none
with: mixins: 0..unbounded
conforms: protocols: 0..unbounded
Decorator occurrences: exactly one
```

`@abstract` is valid with no parameters:

```
@abstract
class BaseNode {
  ...
}
```

`@abstract()` is noncanonical and should be rejected.

When a label is supplied, its list must contain at least one item.

Invalid:

```
@abstract(with: [])
class BaseNode {
  ...
}
```

Invalid:

```
@abstract(conforms: [])
class BaseNode {
  ...
}
```

Positional arguments are deliberately forbidden:

```
@abstract(Stream<T>)
class BaseStream<T> {
  ...
}
```

This could be misread as an abstract parent, protocol requirement, or abstract type parameter. The explicit label is required:

```
@abstract(conforms: [Stream<T>])
```

## 6.6 Method-level `@abstract`

The same attribute name is used on methods:

```
@abstract
next -> Option<T>
```

Method-level `@abstract` accepts no parameters.

```
Arguments: exactly 0
Occurrences per method: at most 1
Method body: forbidden
```

A class containing an abstract method must itself be marked `@abstract`.

Invalid:

```
class BaseStream<T> {
  @abstract
  next -> Option<T>
}
```

Diagnostic:

```
class BaseStream declares abstract methods but is not marked @abstract
```

---

# 7. `@compose`

## 7.1 Purpose

Ordinary concrete classes have no declaration-kind attribute. Adding a redundant `@concrete` or `@class` marker would make simple classes noisier.

When a concrete class needs mixins or explicit conformance, it uses the neutral `@compose(...)` attribute.

```
@compose(
  Enumerable<Int>,
  DebugPrintable,
  conforms: [
    Iterable<Int>,
    Sized
  ]
)
class IntegerBag is Collection {
  ...
}
```

`@compose` means:

> Compose these implementation providers into the class and validate these explicit structural claims.

## 7.2 Positional parameters

Positional parameters are mixin applications.

```
@compose(
  Enumerable<Int>,
  DebugPrintable
)
class IntegerBag {
  ...
}
```

This replaces `@with(...)`.

## 7.3 Labeled `conforms:` parameter

The optional `conforms:` list contains protocols the class intentionally claims.

```
@compose(
  conforms: [
    Iterable<Int>,
    Sized
  ]
)
class IntegerBag {
  ...
}
```

This replaces `@conforms(...)`.

A class can structurally satisfy protocols without listing them. `conforms:` requests eager validation and preserves explicit design intent in reflection.

## 7.4 Cardinality

```
Positional mixins: 0..unbounded
conforms: protocols: 0..unbounded
Minimum combined item count: 1
Decorator occurrences: at most 1
```

Valid with only mixins:

```
@compose(DebugPrintable)
class User {
  ...
}
```

Valid with only protocols:

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

Valid with both:

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

Invalid:

```
@compose
class User {
  ...
}
```

Invalid:

```
@compose()
class User {
  ...
}
```

A class with no composition metadata simply omits the attribute.

## 7.5 Why `@compose`

The name is deliberately neutral.

Using `@with(...)` would make a conformance-only declaration awkward:

```
@with(conforms: [Hashable])
```

Using `@conforms(...)` would make a mixin-only declaration awkward:

```
@conforms(with: [DebugPrintable])
```

Using `@implements(...)` would suggest nominal protocol implementation and would poorly describe classes that only consume mixins.

`@compose(...)` covers both operations without obscuring them because protocol claims retain the explicit `conforms:` label.

---

# 8. Composition and conflict rules

Mixin methods are flattened into the consuming class’s effective method table.

The effective lookup precedence is:

```
1. Methods explicitly declared by the class
2. Methods contributed by directly or transitively applied mixins
3. Methods inherited through is
4. doesNotUnderstand
```

Mixin argument order never establishes precedence.

```
@compose(A, B)
class C {}
```

and:

```
@compose(B, A)
class C {}
```

have the same meaning.

If `A` and `B` contribute the same complete selector, compilation fails unless `C` explicitly declares that selector.

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

Invalid:

```
@compose(
  JsonPrintable,
  DebugPrintable
)
class Document {
  ...
}
```

Valid:

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

Conflicts use complete selector identity. These do not conflict:

```
write(value:)
write(_, to:)
```

Duplicate identical mixin applications are rejected:

```
@compose(
  DebugPrintable,
  DebugPrintable
)
class User {
  ...
}
```

Mixin composition cycles are rejected:

```
A with B
B with C
C with A
```

---

# 9. Type restrictions

Every relationship position has one accepted descriptor category.

|   |   |
|---|---|
|Position|Accepted type|
|`is Parent`|Class type|
|`@protocol(P...)`|Protocol types|
|`@mixin(P...)` positional|Protocol types|
|`@mixin(with: [...])`|Mixin types|
|`@abstract(with: [...])`|Mixin types|
|`@abstract(conforms: [...])`|Protocol types|
|`@compose(...)` positional|Mixin types|
|`@compose(conforms: [...])`|Protocol types|

The compiler should not silently reinterpret incorrect descriptor kinds.

```
@compose(Iterable<Int>)
class Numbers {
  ...
}
```

is invalid when `Iterable<Int>` is a protocol.

Diagnostic:

```
@compose positional arguments must be mixins.

Iterable<Int> is a protocol.
Place it in conforms: [...] instead.
```

Similarly:

```
@mixin(BaseCollection<T>)
class Enumerable<T> {
  ...
}
```

is invalid when `BaseCollection<T>` is a class.

Diagnostic:

```
@mixin positional arguments must be protocol requirements.

BaseCollection<T> is a class.
Describe the required behavior through a protocol.
```

---

# 10. Decorator-order independence

Attribute source order must not determine semantic processing.

These declarations are equivalent:

```
@sealed
@immutable
@compose(
  DebugPrintable,
  conforms: [Hashable]
)
class Identifier {
  ...
}
```

```
@compose(
  DebugPrintable,
  conforms: [Hashable]
)
@immutable
@sealed
class Identifier {
  ...
}
```

The compiler processes declaration metadata in fixed semantic phases:

1. Determine the declaration kind.

2. Resolve generic parameters.

3. Resolve the optional superclass from `is`.

4. Resolve protocol inclusions.

5. Expand derivations such as `@data`.

6. Resolve and flatten mixins.

7. Merge effective selectors.

8. Diagnose method conflicts.

9. Validate mixin host requirements.

10. Validate explicit protocol conformance.

11. Validate abstract methods.

12. Validate sealing and immutability.

13. produce reflection metadata and method tables.


---

# 11. Orthogonal attributes remain separate

Not every class attribute should be folded into the declaration-kind decorator.

These remain independent:

```
@data
@immutable
@sealed
@variant
```

They express object-model or derivation semantics rather than protocol, mixin, or class composition relationships.

Trying to absorb them would create boolean-option declarations such as:

```
@abstract(
  sealed: true,
  immutable: true,
  data: true,
  ...
)
```

That is compact only numerically. It is harder to read, harder to validate, and makes each declaration-kind decorator responsible for unrelated language features.

The preferred form remains:

```
@sealed
@data
@immutable
@compose(
  DebugPrintable,
  conforms: [Hashable]
)
class Identifier {
  const _value: String
}
```

There is one decorator for each independent semantic dimension:

- `@sealed`: hierarchy restriction;

- `@data`: derived value behavior;

- `@immutable`: write-once state;

- `@compose`: mixins and explicit conformance.


---

# 12. Complete examples

## 12.1 Protocol refinement

```
@protocol
class Iterable<out T> {
  each(block: [T] -> Any) -> None
}

@protocol
class Sized {
  size -> Int
}

@protocol(Iterable<T>, Sized)
class Collection<out T> {
  first -> Option<T>
}
```

## 12.2 Generic mixin

```
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
}
```

## 12.3 Composed mixin

```
@mixin(Comparable<T>)
class EqualityFromComparison<T> {
  ==(other: T) -> Bool {
    return self.compare(other) == Ordering.equal
  }
}

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

## 12.4 Concrete class

```
@compose(
  OrderingOperations<Version>,
  DebugPrintable,
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

## 12.5 Abstract class

```
@protocol
class Stream<out T> {
  next -> Option<T>
  close() -> None
}

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
  next -> Option<T>

  @abstract
  closeUnderlying() -> None
}
```

## 12.6 Concrete subclass

```
class FileStream is BaseStream<Bytes> {
  next -> Option<Bytes> {
    ...
  }

  closeUnderlying() -> None {
    ...
  }
}
```

The subclass inherits nominal behavior through `is` and receives the mixin methods already composed into `BaseStream`.

---

# 13. Rejected alternatives

## 13.1 Separate helper decorators

Rejected:

```
@mixin
@needs(Iterable<T>)
@with(EqualityOperations<T>)
class Enumerable<T> {
  ...
}
```

Problems:

- excessive declaration noise;

- duplicated decorators;

- possible ordering questions;

- related mixin metadata is scattered;

- reflection must merge several attribute instances.


Preferred:

```
@mixin(
  Iterable<T>,
  with: [
    EqualityOperations<T>
  ]
)
class Enumerable<T> {
  ...
}
```

## 13.2 One universal declaration decorator

Rejected:

```
@declaration(
  kind: #mixin,
  requires: [...],
  with: [...]
)
class Enumerable<T> {
  ...
}
```

Problems:

- declaration kinds become enum values rather than language concepts;

- diagnostics become less direct;

- invalid combinations become easier to construct;

- `@protocol`, `@mixin`, and `@abstract` disappear from the visible language vocabulary.


## 13.3 Heterogeneous positional lists

Rejected:

```
@compose(
  Enumerable<Int>,
  Iterable<Int>,
  DebugPrintable,
  Sized
)
class Numbers {
  ...
}
```

The user must visually determine which arguments are protocols and which are mixins. The compiler could inspect descriptor types, but the declaration would still obscure two fundamentally different relationships.

Preferred:

```
@compose(
  Enumerable<Int>,
  DebugPrintable,
  conforms: [
    Iterable<Int>,
    Sized
  ]
)
class Numbers {
  ...
}
```

## 13.4 Positional arguments on `@abstract`

Rejected:

```
@abstract(
  CloseOnce,
  Stream<T>
)
class BaseStream<T> {
  ...
}
```

It is unclear which item is a mixin, protocol, abstract dependency, or parent.

Preferred:

```
@abstract(
  with: [CloseOnce],
  conforms: [Stream<T>]
)
class BaseStream<T> {
  ...
}
```

---

# 14. Ratified surface

The final recommended declaration surface is:

```
// Structural protocol
@protocol(OptionalIncludedProtocols...)
class P<TypeParameters> {
  requirements
}
```

```
// Reusable concrete implementation
@mixin(
  OptionalRequiredProtocols...,
  with: OptionalMixinList
)
class M<TypeParameters> {
  concreteMethods
}
```

```
// Incomplete nominal class
@abstract(
  with: OptionalMixinList,
  conforms: OptionalProtocolList
)
class A<TypeParameters> is OptionalSuperclass {
  fields
  constructors
  concreteMethods
  abstractMethods
}
```

```
// Concrete nominal class with composition metadata
@compose(
  OptionalMixins...,
  conforms: OptionalProtocolList
)
class C<TypeParameters> is OptionalSuperclass {
  fields
  constructors
  concreteMethods
}
```

```
// Plain concrete nominal class
class C<TypeParameters> is OptionalSuperclass {
  ...
}
```

The final conceptual vocabulary is:

```
is
    One nominal superclass.

@protocol(P...)
    A structural capability including optional parent protocols.

@mixin(P..., with: [...])
    Reusable concrete methods requiring optional host protocols
    and optionally composing other mixins.

@abstract(with: [...], conforms: [...])
    A non-instantiable nominal class with optional implementation
    composition and explicit structural claims.

@compose(M..., conforms: [...])
    Optional mixin composition and explicit protocol claims for
    an ordinary concrete class.
```

This is the minimum-decorator design that remains explicit about every relationship. It removes four helper decorators without introducing a universal option bag, heterogeneous positional lists, source-order behavior, or overloaded `is` semantics.
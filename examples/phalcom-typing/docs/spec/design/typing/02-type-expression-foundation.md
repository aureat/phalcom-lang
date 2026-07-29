# Type Expression Foundation

- **Status:** Proposed normative design; not a claim of current compiler or VM support
- **Date:** 2026-07-23
- **Depends on:** Document 01 — Protocol Foundation; class and protocol descriptor identity; immutable collections; method, class, module, and source reflection; trusted bootstrap shells
- **Supersedes:** the `Type` and `TypeDescriptor` fragments in the Phase 1 reference package wherever they conflict with this document
- **Superseded by:** none
- **Related ADRs and specifications:** `docs/spec/design/typing/01-protocol-foundation.md`, the current object-model, selector, attribute, module, reflection, `@native`, and immutable-value specifications, and Documents 03–07, 10, 13, 16–21 of this series

This document is the second normative part of the Phalcom typing specification series. It defines the common object protocol for type expressions, makes existing class and protocol descriptors type expressions directly, establishes the implementation base for synthetic descriptors, fixes normalization and equivalence rules, and reserves the public `Type.currentApplication` surface.

The visible Phalcom source in Section 6 is normative. Native implementations may replace selected `@native` methods only when they preserve that source contract exactly.

## 1. Purpose and scope

### 1.1 Purpose

Phalcom requires one reflective vocabulary for objects that denote types without introducing a parallel hierarchy that wraps every existing class and protocol. A class object already has stable identity, a name, methods, fields, inheritance metadata, and module ownership. A protocol object already has stable identity, requirements, generic metadata, and source ownership. Creating redundant `ClassType` and `ProtocolType` wrappers would split identity, complicate reflection, and force tools to unwrap objects before reaching the descriptors they actually need.

The common foundation is therefore a signature-only protocol:

```phalcom
@protocol
class Type {
  displayName -> String
  origin -> Type
  arguments -> const List<Type>
  typeParameters -> const List<TypeParameter>
  freeParameters -> const List<TypeParameter>
  substitute(using: TypeEnvironment) -> Type
  equivalentTo(other: Type) -> Bool
}
```

Existing `Class` and `Protocol` objects satisfy this surface directly. Synthetic type expressions—such as future applied types, type parameters, block types, intersections, aliases, and special singleton types—use the ordinary abstract implementation base `TypeDescriptor` where appropriate.

This architecture preserves the central typing invariant:

> Type-expression objects and metadata are explicitly observable, but they do not implicitly participate in selector encoding, ordinary value-method lookup, overload resolution, allocation, layout, ordinary inline-cache identity, or automatic value validation. Direct messages sent to descriptor objects remain ordinary explicit reflection; the reserved `Type.currentApplication` intrinsic is the sole singleton operation introduced here.

### 1.2 In scope

This document specifies:

- `Type` as a first-class signature-only protocol descriptor;
- class objects as type expressions without `ClassType` wrappers;
- protocol objects as type expressions without `ProtocolType` wrappers;
- the shared meanings of `displayName`, `origin`, `arguments`, `typeParameters`, `freeParameters`, `substitute(using:)`, and `equivalentTo(_:)`;
- the distinction between declaration-owned parameters and free parameters occurring in an expression;
- `TypeDescriptor` as the ordinary implementation base for trusted synthetic descriptors;
- identity, equivalence, hashing, immutability, normalization, and collection-order rules;
- the minimum compiler and VM recognition rules for type expressions;
- the public read-only `Type.currentApplication` API surface;
- the trusted singleton-intrinsic mechanism used to expose that API without turning it into a protocol requirement or default implementation;
- bootstrap ordering for the `Protocol` shell, the canonical `Type` descriptor, built-in class/protocol type behavior, and synthetic descriptors;
- reflection and metadata normalization requirements;
- stable diagnostics and conformance fixtures.

### 1.3 Out of scope

The following are assigned to later documents:

- exact `TypeParameter`, `TypeParameterSpec`, `TypeParameterOwner`, variance, bounds, and constraints: Document 03;
- angle application, `TypeConstructor`, `AppliedType`, interning, and reserved `<...>`: Document 04;
- concrete `TypeEnvironment`, `TypeBinding`, recursive substitution, and applied member views: Document 05;
- class-side forwarding frames and full `Type.currentApplication` propagation/restoration semantics: Document 06;
- `Any`, `Dynamic`, `Nothing`, `Self`, `Unit`, and the complete type-relation lattice: Document 07;
- structural protocol conformance and protocol composition: Document 10;
- block-type descriptor fields and callable relations: Document 13;
- aliases and intersections: Document 16;
- final source/normalized metadata encoding: Document 17;
- hardened bootstrap and native security: Document 18;
- checker modes, diagnostics presentation, LSP behavior, and compatibility: Document 19.

This document reserves the shared interfaces required by those later parts. Later documents may specialize descriptor behavior but may not silently redefine the meanings fixed here.

### 1.4 Design alternatives and final decisions

#### 1.4.1 Wrapper objects versus direct descriptor participation

Serious options:

1. Wrap each class in `ClassType` and each protocol in `ProtocolType`.
2. Use classes directly but wrap protocols.
3. Make both existing descriptor kinds satisfy `Type` directly.

Decision: **Option 3.** Bare type identity is the existing descriptor identity. `String` as a class object and `Drawable` as a protocol object are already the canonical bare type expressions.

#### 1.4.2 Common abstract superclass versus protocol

Serious options:

1. Require all type objects to inherit a common `Type` class.
2. Make `Type` a protocol and provide `TypeDescriptor` only for synthetic implementation reuse.
3. Use no shared behavioral abstraction and branch on descriptor kind everywhere.

Decision: **Option 2.** `Class` and `Protocol` cannot sensibly change inheritance merely to become type expressions. Structural behavior unifies them without changing their object-model role.

#### 1.4.3 Open user-defined descriptor kinds versus trusted core descriptor kinds

Serious options:

1. Any object structurally satisfying `Type` is accepted as authoritative compiler metadata.
2. Only built-in trusted kinds are accepted forever.
3. The runtime `Type` protocol remains structurally observable, while first-version compiler normalization accepts existing class/protocol descriptors and trusted `TypeDescriptor` kinds; a controlled extension policy may be specified later.

Decision: **Option 3.** Arbitrary libraries may consume objects through the `Type` protocol, but compiler-emitted normalized metadata must use recognized stable descriptor kinds. This prevents mutable or effectful objects from entering trusted metadata while preserving a future extension path.

#### 1.4.4 Meaning of bare generic origins

Serious options:

1. Treat `Box` as if it were the open application `Box<T>`.
2. Treat `Box` as the declaration/type-constructor object itself.

Decision: **Option 2.** `Box.typeParameters` may contain `T`, but `Box.freeParameters` is empty and `Box.substitute(using:)` returns `Box`. The expression `Box<T>` is a distinct applied expression introduced in Document 04.

#### 1.4.5 `Type.currentApplication` placement

Serious options:

1. Put an executable class-side body inside the `@protocol` declaration.
2. Put `currentApplication` on every `Protocol` instance.
3. Rename the public API to `TypeRuntime.currentApplication` only.
4. Install one trusted, reflectable singleton intrinsic on the canonical `Type` descriptor, backed by `TypeRuntime.currentApplication`.

Decision: **Option 4.** Document 01 forbids executable protocol requirements and states that class-side requirements describe candidate class objects rather than methods on a protocol descriptor. The singleton intrinsic preserves `Type.currentApplication` without creating a default method, requirement, per-protocol metaclass, user extension point, or second dispatch mechanism.

#### 1.4.6 Equivalence versus identity

Serious options:

1. Require all equivalent types to be identical objects.
2. Define equivalence separately and permit canonical identity only where promised.
3. Use ordinary `==` without a type-specific relation.

Decision: **Option 2.** Bare classes and protocols use identity equivalence. Synthetic kinds define semantic equivalence and may additionally be interned. `equivalentTo(_:)` is not subtyping, consistency, conformance, assignment acceptance, or ordinary value equality.

## 2. Terminology

### 2.1 Type expression

A recognized immutable object denoting a type-level concept and satisfying the `Type` behavioral surface.

### 2.2 Bare class type

An existing `Class` descriptor used directly as a type expression, such as `String` or `Box`.

### 2.3 Bare protocol type

An existing `Protocol` descriptor used directly as a type expression, such as `Drawable` or the canonical `Type` protocol itself.

### 2.4 Synthetic descriptor

A type-expression object that is not merely an existing `Class` or `Protocol`. Synthetic descriptors normally inherit `TypeDescriptor` and include future `TypeParameter`, `AppliedType`, block, intersection, alias, and special-type objects.

### 2.5 Origin

The stable descriptor from which an expression is formed. A non-applied bare descriptor is its own origin. An applied type’s origin is its type constructor. Later descriptor kinds define an origin only where a natural constructor decomposition exists.

### 2.6 Arguments

The immutable ordered type-expression operands applied to an origin. Bare class and protocol descriptors have no arguments. This field is not a general-purpose child-node traversal API.

### 2.7 Declared type parameters

The immutable parameters owned by a declaration or other type constructor. They are exposed through `typeParameters` and are not automatically free occurrences in the bare origin expression.

### 2.8 Free parameters

The immutable, duplicate-free, first-occurrence-ordered list of `TypeParameter` objects occurring unbound in a type expression.

### 2.9 Closed type expression

An expression whose `freeParameters` list is empty.

### 2.10 Open type expression

An expression whose `freeParameters` list is non-empty.

### 2.11 Substitution

The immutable operation that replaces free type parameters according to a `TypeEnvironment` and returns a type expression.

### 2.12 Type equivalence

A reflexive, symmetric, and transitive relation exposed by `equivalentTo(_:)`. Equivalence means that two descriptors denote the same normalized type expression under the rules for their descriptor kinds.

### 2.13 Normalization

Validation and conversion of a source-resolved annotation value into a recognized stable type-expression object. Normalization never wraps bare classes or protocols.

### 2.14 Trusted descriptor kind

A descriptor class whose construction, immutability, equivalence, hash, and metadata shape are accepted by the compiler/VM authority boundary.

### 2.15 Current application

The optional `AppliedType` associated with an active applied-type class-side invocation frame. The complete frame semantics are deferred to Document 06.

### 2.16 Singleton intrinsic

A reserved method entry installed on one canonical object identity by trusted bootstrap. It is visible to ordinary reflection and direct message sending but cannot be declared, replaced, reopened, or intercepted by user code.

## 3. User-facing syntax

### 3.1 The canonical `Type` declaration

The shared protocol is declared using the protocol foundation from Document 01:

```phalcom
@protocol
class Type {
  displayName -> String
  origin -> Type
  arguments -> const List<Type>
  typeParameters -> const List<TypeParameter>
  freeParameters -> const List<TypeParameter>
  substitute(using: TypeEnvironment) -> Type
  equivalentTo(other: Type) -> Bool
}
```

Every member is a bodyless instance-side requirement. No member provides a default body. The declaration binds `Type` to a first-class `Protocol` descriptor.

### 3.2 Class objects are type expressions

```phalcom
const type = String

System.print(type.displayName)
// String

System.assert(type.origin === String)
System.assert(type.arguments.isEmpty)
System.assert(type.freeParameters.isEmpty)
```

No wrapper allocation occurs. The exact object stored in `type` is the class descriptor `String`.

### 3.3 Protocol objects are type expressions

```phalcom
@protocol
class Drawable {
  draw() -> Unit
}

const type = Drawable

System.assert(type.class === Protocol)
System.assert(type.origin === Drawable)
System.assert(type.arguments.isEmpty)
```

The protocol remains a `Protocol`; it does not become a class or a wrapper instance.

### 3.4 The `Type` descriptor is itself a type expression

Because every protocol descriptor receives the built-in protocol type-expression surface:

```phalcom
System.assert(Type.class === Protocol)
System.assert(Type.origin === Type)
System.assert(Type.equivalentTo(Type))
```

This self-description does not imply nominal self-conformance and does not run the Document 10 structural-conformance algorithm during bootstrap. Trusted bootstrap establishes the built-in surface before general conformance queries are available.

### 3.5 Bare generic declarations

```phalcom
class Box<T> {}

Box.typeParameters
// const [T]

Box.freeParameters
// const []

Box.origin
// Box
```

`Box` is the generic declaration/type-constructor object. It is not shorthand for `Box<T>`, and substitution does not implicitly apply its declared parameters.

### 3.6 Synthetic descriptors

Synthetic descriptors appear as ordinary first-class objects:

```phalcom
const type = Box<Int>

System.print(type.displayName)
// Box<Int>
```

`AppliedType` and angle application are defined in Document 04. The shared operations defined here apply once that descriptor exists.

### 3.7 Explicit reflection only

A type annotation is observable through reflection:

```phalcom
save(value: Box<Int>) -> Unit {
  // Ordinary execution is unchanged.
}

const method = Repository.methodFor(#save(value:)).unwrap
const annotation = method.parameters.at(0).type.unwrap

System.print(annotation.displayName)
// Box<Int>
```

The annotation does not change the selector, dispatch, parameter passing, or runtime acceptance of values.

### 3.8 Absent annotation is not an implicit special type

```phalcom
untyped(value) {
  return value
}
```

Reflection reports `None` for the missing parameter and result annotation. It does not manufacture `Dynamic`, `Any`, `Object`, or another type expression. Explicit special-type semantics are defined in Document 07.

### 3.9 Current application

The public API is:

```phalcom
Type.currentApplication -> Option<AppliedType>
```

Outside an applied class-side invocation:

```phalcom
System.assert(Type.currentApplication == None)
```

Inside a future applied send:

```phalcom
class Box<T> {
  @class
  from(value: Object) -> Box<T> {
    const application = Type.currentApplication
    // During Box<Int>.from(value), application is Some(Box<Int>).
    return Box.new(value: value)
  }
}
```

The API is read-only. There is no setter, dynamic assignment, or user-accessible push/pop operation.

### 3.10 No executable member is added to the protocol body

The following is illegal under Document 01 and remains illegal:

```phalcom
@protocol
class Type {
  currentApplication -> Option<AppliedType> {
    return TypeRuntime.currentApplication
  }
}
```

The canonical source anchor is `TypeRuntime.currentApplication`; trusted bootstrap exposes a singleton alias on the canonical `Type` object.

## 4. Semantic model

### 4.1 Recognized type-expression categories

The first-version normalized universe includes:

1. any ordinary `Class` descriptor;
2. any `Protocol` descriptor;
3. trusted synthetic descriptors introduced by this and later typing documents, normally inheriting `TypeDescriptor`;
4. trusted declaration-owned `TypeParameter` descriptors after Document 03.

A runtime library may use the `Type` protocol structurally against other objects, but the compiler and VM are not required to serialize, intern, compare, or trust arbitrary user-defined descriptor kinds in normalized annotation metadata.

### 4.2 Bare descriptor table

For a non-generic class or protocol `D`:

| Operation | Result |
|---|---|
| `D.displayName` | declaration display name |
| `D.origin` | `D` |
| `D.arguments` | `const []` |
| `D.typeParameters` | `const []` |
| `D.freeParameters` | `const []` |
| `D.substitute(using: env)` | `D` |
| `D.equivalentTo(other)` | `D === other` |
| `D.hash` | identity-compatible hash |

For a generic declaration `G<T...>`, only `typeParameters` changes. `freeParameters` remains empty because the bare declaration object contains no applied occurrences.

### 4.3 No redundant wrappers

Normalization of a class or protocol is idempotent and identity-preserving:

```phalcom
TypeRuntime.normalize(String) === String
TypeRuntime.normalize(Drawable) === Drawable
```

The runtime must not allocate hidden wrappers, adapter objects, proxy types, or sidecar identity objects for these cases.

Internal VM metadata may store compact tagged references, but reflection must return the original descriptor object.

### 4.4 Declaration parameters versus free parameters

`typeParameters` answers:

> Which parameters does this descriptor declare as a type constructor or owner?

`freeParameters` answers:

> Which parameter objects occur unbound inside this particular expression?

These are intentionally different:

```text
Box.typeParameters       = [T]
Box.freeParameters       = []
T.typeParameters         = []
T.freeParameters         = [T]
Box<T>.typeParameters    = []
Box<T>.freeParameters    = [T]
Box<Int>.typeParameters  = []
Box<Int>.freeParameters  = []
```

Document 03 fixes the identity of `T`; Document 04 fixes the applied rows.

### 4.5 Free-parameter ordering

`freeParameters` must be:

- immutable;
- duplicate-free by `TypeParameter` identity/equivalence;
- ordered by first left-to-right occurrence in the normalized expression;
- stable across repeated calls;
- independent of hash-map iteration order.

For a future expression `Pair<T, List<U>, T>`, the result is `const [T, U]`.

### 4.6 Origin and arguments

For bare classes, protocols, type parameters, and singleton special descriptors:

```text
origin    = self
arguments = const []
```

For an applied descriptor, `origin` is the unapplied type constructor and `arguments` is the exact immutable source-order argument list after normalization.

`origin` must itself be a recognized type expression. Every element of `arguments` must be a recognized type expression. Cycles not explicitly permitted by a later descriptor specification are invalid metadata.

### 4.7 Substitution

`substitute(using:)` is:

- pure from the caller’s perspective;
- non-mutating;
- deterministic for one descriptor and environment;
- identity-preserving when no occurrence changes;
- recursive for compound descriptor kinds;
- cycle-guarded when recursive descriptors are introduced;
- required to return a normalized `Type` object.

Bare `Class` and `Protocol` descriptors return themselves. A type parameter resolves itself through the environment in Document 03. Applied and compound descriptors recursively substitute their contained expressions in Documents 04–05.

Substitution never changes executable code, ordinary class inheritance, layout, or value identity.

### 4.8 Equivalence

For every valid type expression `A`:

```text
A.equivalentTo(A) == true
```

For valid `A`, `B`, and `C`:

```text
A.equivalentTo(B) == B.equivalentTo(A)
A.equivalentTo(B) and B.equivalentTo(C) implies A.equivalentTo(C)
```

Bare class and protocol equivalence is descriptor identity. Two separately declared protocols with the same name and requirements are not equivalent. Two distinct classes with the same display name are not equivalent.

Synthetic descriptors define kind-specific semantic equivalence. Equivalent synthetic descriptors must have equal type hashes. Canonical interning may additionally make them identical, but only where a later specification promises that identity.

### 4.9 Equivalence is not another relation

`equivalentTo(_:)` must not be used as a synonym for:

- subclassing;
- protocol conformance;
- generic variance;
- consistency with `Dynamic`;
- assignment acceptance;
- runtime value validation;
- exact runtime class identity;
- ordinary `==` for arbitrary values.

Document 07 separates equivalence, subtype, consistency, and acceptance. Document 10 defines structural protocol conformance.

### 4.10 Hash contract

A descriptor used as a type-key must satisfy:

```text
A.equivalentTo(B) implies A.hash == B.hash
```

The converse is not required. Bare classes and protocols use identity-compatible hashes. Synthetic descriptor hashes combine descriptor kind, origin identity where applicable, and equivalent component hashes.

A descriptor whose `equivalentTo` or `hash` result changes after construction is invalid.

### 4.11 Display names

`displayName` is stable human-facing rendering. It must not be used as identity or as the only persistent metadata key.

Bare classes and protocols return their declaration display name. Qualified source rendering, aliases, and source-preserving spellings are separate reflection concerns. `toString` should normally delegate to `displayName` for synthetic descriptors, while existing class/protocol `toString` behavior may remain more qualified.

### 4.12 Immutability

Every synthetic descriptor is observationally immutable after successful construction:

- descriptor fields cannot change;
- contained collections are immutable;
- equality/equivalence and hash do not change;
- origin and arguments do not change;
- free-parameter results do not change;
- source metadata does not change;
- no user operation can complete or reopen a descriptor.

Class and protocol descriptors may participate in Phalcom’s open method model, but their identity as bare type expressions is stable. Method-table mutation must not change `origin`, `arguments`, or bare type equivalence. Cache invalidation for relations dependent on members is specified later.

### 4.13 Trusted normalization

`TypeRuntime.normalize(candidate:)` validates that a candidate is a recognized type-expression kind and that its exposed shape is stable. It returns the same canonical object for bare classes and protocols.

Normalization may canonicalize synthetic descriptors only where their specification permits. It must not call arbitrary conversion methods such as `asType`, `toType`, or `coerce` on unknown values.

### 4.14 Custom descriptor policy

The first version does not provide a public registration API for new authoritative descriptor kinds. User code may subclass or emulate typing objects only for ordinary library-level experimentation where accepted by that library. The compiler, metadata emitter, and VM may reject such objects with `type.expression.untrusted_descriptor`.

This restriction protects compiler determinism and metadata integrity. Document 17 or a later extension specification may define a portable custom-descriptor protocol, but it must not weaken the invariants in this document.

### 4.15 `Type` satisfies itself

The canonical `Type` object is a `Protocol` descriptor, and all protocol descriptors receive the built-in type-expression behavior. Therefore the object bound to `Type` has the required selectors.

This is a bootstrap fact, not a circular structural-conformance proof. General conformance queries remain unavailable until the descriptor and built-in surfaces are complete.

### 4.16 Current-application semantics reserved here

This document fixes only the public contract:

- `Type.currentApplication` returns `Option<AppliedType>`;
- outside an active applied invocation it returns `None`;
- user code cannot set it;
- the query never participates in method selection;
- the result, when present, is the active applied type view and not hidden data on the constructed value;
- reading the query has no side effects.

Document 06 defines fiber locality, stack scoping, same-origin propagation, nested contexts, exception-safe restoration, and DNU interaction.

## 5. Object model

### 5.1 Descriptor relationships

The foundational relationships are:

```text
Type                    instance of Protocol
TypeDescriptor          instance of Class
String                   instance of Class; satisfies Type
Drawable                 instance of Protocol; satisfies Type
AppliedType instance     instance of AppliedType; satisfies Type through TypeDescriptor
```

`TypeDescriptor` does not inherit from `Type` because `Type` is a protocol, not a class. Its instances satisfy `Type` structurally.

### 5.2 Existing descriptor identity is retained

For a class `C`:

```phalcom
C.class === Class
TypeRuntime.normalize(C) === C
```

For a protocol `P`:

```phalcom
P.class === Protocol
TypeRuntime.normalize(P) === P
```

No per-type wrapper class, proxy metaclass, hidden allocation origin, or duplicate method table is introduced.

### 5.3 Built-in type surface

Trusted core definitions of `Class` and `Protocol` must expose the `Type` selectors. Implementations may place the method bodies directly in those core classes or install equivalent trusted native entries during bootstrap.

The observable behavior must match Section 6. No implementation may expose wrapper objects merely because its internal core classes cannot share source code.

### 5.4 `TypeDescriptor`

`TypeDescriptor` is an ordinary abstract trusted standard-library class used for synthetic descriptor implementation reuse. It provides identity defaults:

- origin is `self`;
- arguments, type parameters, and free parameters are empty;
- substitution returns `self`;
- equivalence is identity;
- hash is identity-compatible;
- `toString` delegates to `displayName`.

A concrete synthetic descriptor overrides only the operations whose semantics differ.

### 5.5 Construction authority

The base `TypeDescriptor` constructor requires a trusted token. This prevents user code from instantiating a meaningless base descriptor and prevents untrusted subclasses from constructing authoritative metadata through the standard path.

Later trusted subclasses receive the same token through internal constructors. The token is not reflectively forgeable, serializable, or comparable by user code.

### 5.6 Descriptor graph and GC

Synthetic descriptors strongly trace:

- their origin where present;
- every argument;
- every declared or free parameter reference;
- source and owner metadata;
- any environment or component descriptors introduced later.

Interning caches must use GC-safe weak keys/values or another strategy that does not keep every historically formed type alive forever. Exact applied-type cache behavior is specified in Document 04.

### 5.7 Singleton intrinsic for `Type.currentApplication`

The canonical `Type` object receives one trusted intrinsic entry for selector `currentApplication`. Lookup is conceptually:

```text
1. reserved singleton intrinsic table for the exact receiver identity
2. ordinary methods supplied by Protocol
3. ordinary doesNotUnderstand path
```

Only the canonical `Type` descriptor has this entry. An inline cache for this explicit intrinsic may guard the exact canonical receiver identity, as Document 01 permits for reflective APIs sent to descriptor objects. That guard must never leak into ordinary value dispatch. Other protocols do not respond through this intrinsic:

```phalcom
Drawable.currentApplication
// ordinary doesNotUnderstand
```

The intrinsic forwards to `TypeRuntime.currentApplication`. It does not create a per-protocol metaclass, alter protocol requirements, or mutate the protocol after publication.

### 5.8 Reflection of the intrinsic

Reflection reports the intrinsic as an executable method-like descriptor with:

- receiver identity: canonical `Type` protocol object;
- selector: getter `currentApplication`;
- result annotation: `Option<AppliedType>`;
- implementation anchor: `TypeRuntime.currentApplication`;
- native status: true;
- reserved status: true;
- source location: the standard-library source anchor where available.

It does not appear in `Type.requirements` and does not appear in another protocol’s method list.

### 5.9 Ordinary dispatch preservation

Type-expression participation does not affect dispatch on values. Sending `add(_:)` to a list does not inspect its annotation. Sending a class-side message to `Box` does not inspect a type environment. Only explicit messages sent to type-expression objects invoke this API.

### 5.10 Bootstrap circularity

The required order is:

1. create trusted minimal shells for `Class`, `Protocol`, and required core reflection objects;
2. make class and protocol descriptors capable of the minimal bare type-expression operations internally;
3. parse and index the signature-only `Type` protocol declaration using Document 01’s trusted protocol shell;
4. complete and bind the canonical `Type` descriptor;
5. install the reserved `Type.currentApplication` singleton intrinsic;
6. load and freeze `TypeDescriptor`, diagnostics, and `TypeRuntime` source models;
7. enable general normalized annotation resolution;
8. load later type-parameter and applied-type descriptors.

User reflection must not observe an incomplete `Type` descriptor or an installed intrinsic whose runtime anchor is unavailable.

## 6. Complete standard-library source model

### 6.1 Status of forward references

The source below is normative for this document but references names defined later in the series:

- `TypeParameter` and its ownership semantics: Document 03;
- `AppliedType`: Document 04;
- `TypeEnvironment`: Document 05.

A bootstrap compiler may resolve these through trusted declaration indexing. Their use here fixes API names and positions but does not pull their complete semantics into this document.

The `Class` and `Protocol` integration blocks in Section 6.7 are normative augmentations to the existing complete core definitions. They are not replacement declarations for those large built-in classes.

`TypeDescriptor` uses the existing `@abstract` marker only to prohibit direct construction as a concrete descriptor base. Document 11 supplies the complete general abstract-class and obligation semantics; this document does not depend on abstract member inheritance.

### 6.2 Diagnostics and errors

```phalcom
@data
@immutable
class TypeExpressionDiagnostic {
  const _code: String
  const _message: String
  const _details: const Map<Symbol, Object>
  const _sourceLocation: Option<SourceLocation>

  @constructor
  new(
    code: String,
    message: String,
    details: const Map<Symbol, Object>,
    sourceLocation: Option<SourceLocation>
  ) {
    _code = code
    _message = message
    _details = details.freeze
    _sourceLocation = sourceLocation
  }
}

class TypeExpressionError is Error {
  const _diagnostic: TypeExpressionDiagnostic

  @constructor
  new(diagnostic: TypeExpressionDiagnostic) {
    _diagnostic = diagnostic
  }

  diagnostic -> TypeExpressionDiagnostic {
    return _diagnostic
  }

  message -> String {
    return _diagnostic.message
  }
}

class InvalidTypeExpressionError is TypeExpressionError {}
class UntrustedTypeDescriptorError is TypeExpressionError {}
class TypeDescriptorAuthorityError is TypeExpressionError {}
class TypeSubstitutionError is TypeExpressionError {}
class TypeEquivalenceError is TypeExpressionError {}
class ReservedTypeIntrinsicError is TypeExpressionError {}
class MalformedTypeMetadataError is TypeExpressionError {}
```

### 6.3 Trusted construction token

```phalcom
@immutable
class TypeDescriptorConstructionToken {
  @constructor
  @native
  _trustedNew() {}
}

const _TYPE_DESCRIPTOR_CONSTRUCTION_TOKEN =
  TypeDescriptorConstructionToken._trustedNew()
```

The constructor is not callable with user-forgeable authority. The exact token representation is private and may be a VM capability rather than an ordinary heap object.

### 6.4 `Type` protocol

```phalcom
@protocol
class Type {
  displayName -> String
  origin -> Type
  arguments -> const List<Type>
  typeParameters -> const List<TypeParameter>
  freeParameters -> const List<TypeParameter>
  substitute(using: TypeEnvironment) -> Type
  equivalentTo(other: Type) -> Bool
}
```

`currentApplication` is deliberately absent from this requirement list. It is an API on the canonical descriptor object, not behavior required from every type expression.

### 6.5 `TypeDescriptor`

```phalcom
@abstract
@immutable
class TypeDescriptor {
  const _constructionToken: TypeDescriptorConstructionToken

  @constructor
  _trustedNew(token: TypeDescriptorConstructionToken) {
    TypeRuntime.requireConstructionAuthority(token)
    _constructionToken = token
  }

  displayName -> String {
    throw TypeRuntime.subclassResponsibility(
      receiver: self,
      selector: #displayName
    )
  }

  origin -> Type {
    return self
  }

  arguments -> const List<Type> {
    return const []
  }

  typeParameters -> const List<TypeParameter> {
    return const []
  }

  freeParameters -> const List<TypeParameter> {
    return const []
  }

  substitute(using: TypeEnvironment) -> Type {
    if self.freeParameters.isEmpty {
      return self
    }

    throw TypeRuntime.invalidSubstitutionImplementation(
      descriptor: self
    )
  }

  equivalentTo(other: Type) -> Bool {
    return self === other
  }

  hash -> Int {
    return self.identityHash
  }

  toString -> String {
    return self.displayName
  }
}
```

The field storing the token is conceptually sufficient to show construction authority. A VM may validate authority without retaining the token in each instance.

### 6.6 Built-in reference semantics

```phalcom
class BuiltinTypeSemantics {
  @class
  displayName(of: Type) -> String {
    if of.isA(Class) or of.isA(Protocol) {
      return of.name.toString
    }

    throw TypeRuntime.invalidBuiltinReceiver(of)
  }

  @class
  origin(of: Type) -> Type {
    if of.isA(Class) or of.isA(Protocol) {
      return of
    }

    throw TypeRuntime.invalidBuiltinReceiver(of)
  }

  @class
  arguments(of: Type) -> const List<Type> {
    if of.isA(Class) or of.isA(Protocol) {
      return const []
    }

    throw TypeRuntime.invalidBuiltinReceiver(of)
  }

  @class
  typeParameters(of: Type) -> const List<TypeParameter> {
    if of.isA(Class) or of.isA(Protocol) {
      return TypeRuntime.declaredTypeParameters(of)
    }

    throw TypeRuntime.invalidBuiltinReceiver(of)
  }

  @class
  freeParameters(of: Type) -> const List<TypeParameter> {
    if of.isA(Class) or of.isA(Protocol) {
      return const []
    }

    throw TypeRuntime.invalidBuiltinReceiver(of)
  }

  @class
  substitute(of: Type, using: TypeEnvironment) -> Type {
    if of.isA(Class) or of.isA(Protocol) {
      return of
    }

    throw TypeRuntime.invalidBuiltinReceiver(of)
  }

  @class
  equivalent(left: Type, to: Type) -> Bool {
    if left.isA(Class) or left.isA(Protocol) {
      return left === to
    }

    throw TypeRuntime.invalidBuiltinReceiver(left)
  }
}
```

### 6.7 Normative core integration

The following method bodies are merged into the trusted complete definitions of `Class` and `Protocol`. They are shown as augmentation blocks because those core definitions contain unrelated object-model behavior outside this specification.

```phalcom
// Normative augmentation of Class.
displayName -> String {
  return BuiltinTypeSemantics.displayName(of: self)
}

origin -> Type {
  return BuiltinTypeSemantics.origin(of: self)
}

arguments -> const List<Type> {
  return BuiltinTypeSemantics.arguments(of: self)
}

typeParameters -> const List<TypeParameter> {
  return BuiltinTypeSemantics.typeParameters(of: self)
}

freeParameters -> const List<TypeParameter> {
  return BuiltinTypeSemantics.freeParameters(of: self)
}

substitute(using: TypeEnvironment) -> Type {
  return BuiltinTypeSemantics.substitute(of: self, using: using)
}

equivalentTo(other: Type) -> Bool {
  return BuiltinTypeSemantics.equivalent(self, to: other)
}
```

```phalcom
// Normative augmentation of Protocol.
displayName -> String {
  return BuiltinTypeSemantics.displayName(of: self)
}

origin -> Type {
  return BuiltinTypeSemantics.origin(of: self)
}

arguments -> const List<Type> {
  return BuiltinTypeSemantics.arguments(of: self)
}

typeParameters -> const List<TypeParameter> {
  return BuiltinTypeSemantics.typeParameters(of: self)
}

freeParameters -> const List<TypeParameter> {
  return BuiltinTypeSemantics.freeParameters(of: self)
}

substitute(using: TypeEnvironment) -> Type {
  return BuiltinTypeSemantics.substitute(of: self, using: using)
}

equivalentTo(other: Type) -> Bool {
  return BuiltinTypeSemantics.equivalent(self, to: other)
}
```

An implementation may use native core methods instead of literal forwarding bodies. Reflection and behavior must remain equivalent.

### 6.8 `TypeRuntime`

```phalcom
class TypeRuntime {
  @class
  details(*items: Object) -> const Map<Symbol, Object> {
    let result = Map.new()
    let index = 0

    while index < items.size {
      const key = items.at(index)
      const value = items.at(index + 1)
      result.at(key, put: value)
      index += 2
    }

    return result.freeze
  }

  @class
  @native
  _constructionToken -> TypeDescriptorConstructionToken {
    return _TYPE_DESCRIPTOR_CONSTRUCTION_TOKEN
  }

  @class
  @native
  requireConstructionAuthority(
    token: TypeDescriptorConstructionToken
  ) -> None {
    if token !== _TYPE_DESCRIPTOR_CONSTRUCTION_TOKEN {
      throw TypeDescriptorAuthorityError.new(
        TypeExpressionDiagnostic.new(
          code: "type.expression.construction_authority",
          message: "type descriptor construction requires trusted authority",
          details: self.details(),
          sourceLocation: None
        )
      )
    }
  }

  @class
  @native
  isTrustedSynthetic(candidate: Object) -> Bool {
    // The VM recognizes only descriptor classes registered by the trusted
    // typing bootstrap. Merely inheriting TypeDescriptor is not sufficient.
    return false
  }

  @class
  isRecognized(candidate: Object) -> Bool {
    return candidate.isA(Class) or
      candidate.isA(Protocol) or
      self.isTrustedSynthetic(candidate)
  }

  @class
  @native
  unrecognized(candidate: Object) -> TypeExpressionError {
    // If the object exposes a Type-shaped selector surface but its descriptor
    // kind is not trusted, return UntrustedTypeDescriptorError with code
    // type.expression.untrusted_descriptor. Otherwise return
    // InvalidTypeExpressionError with code type.expression.invalid.
    return InvalidTypeExpressionError.new(
      TypeExpressionDiagnostic.new(
        code: "type.expression.invalid",
        message: "value is not a recognized type expression",
        details: self.details(#candidate, candidate),
        sourceLocation: None
      )
    )
  }

  @class
  normalize(candidate: Object) -> Type {
    if self.isRecognized(candidate).not {
      throw self.unrecognized(candidate)
    }

    self.validateShape(candidate)
    return candidate
  }

  @class
  @native
  validateShape(candidate: Type) -> None {
    // The authoritative implementation validates that every required selector
    // exists, all returned collections are immutable, origin and arguments are
    // recognized, free parameters are valid and duplicate-free, and repeated
    // observations are stable. It must not invoke arbitrary coercion hooks.
  }

  @class
  normalizeAll(types: List<Object>) -> const List<Type> {
    return types.map { type =>
      self.normalize(type)
    }.freeze
  }

  @class
  collectFreeParameters(
    types: const List<Type>
  ) -> const List<TypeParameter> {
    let result = const []

    types.each { type =>
      type.freeParameters.each { parameter =>
        const exists = result.any { existing =>
          existing.equivalentTo(parameter)
        }

        if exists.not {
          result = result.appending(parameter).freeze
        }
      }
    }

    return result
  }

  @class
  equivalent(left: Type, right: Type) -> Bool {
    return left.equivalentTo(right)
  }

  @class
  equivalentCollections(
    left: const List<Type>,
    right: const List<Type>
  ) -> Bool {
    if left.size != right.size {
      return false
    }

    let index = 0
    while index < left.size {
      if left.at(index).equivalentTo(right.at(index)).not {
        return false
      }
      index++
    }

    return true
  }

  @class
  @native
  declaredTypeParameters(
    owner: Object
  ) -> const List<TypeParameter> {
    // Before Document 03 is loaded this returns an empty frozen list for all
    // non-generic declarations and trusted unresolved placeholders internally.
    // User reflection never observes unresolved placeholders.
    return const []
  }

  @class
  @native
  currentApplication -> Option<AppliedType> {
    return None
  }

  @class
  @native
  installCurrentApplicationIntrinsic(on: Protocol) -> None {
    // Installs the reserved singleton selector only on the exact canonical
    // Type descriptor before it is published. Repeated installation, a wrong
    // receiver, or installation after publication is rejected.
  }

  @class
  @native
  subclassResponsibility(
    receiver: TypeDescriptor,
    selector: Selector
  ) -> TypeExpressionError {
    return InvalidTypeExpressionError.new(
      TypeExpressionDiagnostic.new(
        code: "type.expression.subclass_responsibility",
        message: "concrete type descriptor must implement \(selector)",
        details: self.details(
          #receiver, receiver,
          #selector, selector
        ),
        sourceLocation: None
      )
    )
  }

  @class
  invalidSubstitutionImplementation(
    descriptor: TypeDescriptor
  ) -> TypeSubstitutionError {
    return TypeSubstitutionError.new(
      TypeExpressionDiagnostic.new(
        code: "type.expression.invalid_substitution",
        message: "open descriptor does not implement recursive substitution",
        details: self.details(#descriptor, descriptor),
        sourceLocation: None
      )
    )
  }

  @class
  invalidBuiltinReceiver(receiver: Object) -> InvalidTypeExpressionError {
    return InvalidTypeExpressionError.new(
      TypeExpressionDiagnostic.new(
        code: "type.expression.invalid_builtin_receiver",
        message: "built-in type semantics require Class or Protocol",
        details: self.details(#receiver, receiver),
        sourceLocation: None
      )
    )
  }
}
```

### 6.9 Canonical bootstrap operation

The trusted bootstrap performs the following conceptual source operation after the `Type` protocol descriptor is complete and before it is published:

```phalcom
TypeRuntime.installCurrentApplicationIntrinsic(on: Type)
```

The operation is not available as a general user API. Calling it after bootstrap or with another protocol fails with `type.current_application.reserved_intrinsic`.

### 6.10 Source-model notes

1. `Type` contains requirements only.
2. `TypeDescriptor` is an ordinary class and contains executable reusable behavior.
3. `BuiltinTypeSemantics` supplies the reference behavior for existing descriptor classes without creating wrappers.
4. `TypeRuntime` is the authority seam for recognition, validation, construction capabilities, built-in integration, and the current-application intrinsic.
5. The `@native` bodies are readable reference semantics. Native Rust may use compact representations but may not alter the observable result.

## 7. Compiler and AST requirements

### 7.1 Phase separation

A conforming implementation must distinguish these phases:

1. **Lexing and parsing:** recognize annotation syntax and ordinary expression syntax without deciding type identity.
2. **Declaration indexing:** reserve class, protocol, and future type-parameter identities so recursive references can resolve.
3. **Attribute expansion:** produce the `Type` protocol descriptor through Document 01’s `@protocol` path; no executable protocol body is generated.
4. **Name and annotation resolution:** resolve source names and applications to descriptor references or deferred metadata nodes.
5. **Normalization:** validate resolved annotation objects through `TypeRuntime.normalize` and preserve bare class/protocol identity.
6. **Checking:** consume type expressions explicitly; no checker result changes runtime dispatch.
7. **Metadata emission:** encode stable references to recognized descriptor kinds and source-preserving annotation information.
8. **Bootstrap/module load:** materialize descriptors, attach metadata, install the `Type.currentApplication` intrinsic, and publish complete bindings.
9. **Runtime reflection:** return normalized type-expression objects and source metadata without recreating wrappers.

These phases may be fused internally for performance, but diagnostics and observable semantics must reflect the separation above.

### 7.2 Annotation AST

Before resolution, an annotation is represented by ordinary source-expression structure plus its source location. At minimum, the AST must distinguish:

- unqualified names;
- qualified names;
- future angle application;
- future block types;
- future intersections and aliases;
- explicit special-type tokens such as `?` once introduced;
- absence of an annotation.

The parser must not create `ClassType` or `ProtocolType` AST nodes merely because a name later resolves to a class or protocol. Descriptor-kind classification occurs during resolution.

### 7.3 Resolved type-expression reference

A resolved annotation record must contain at least:

```text
descriptor reference or deferred descriptor key
source range
source spelling / source-form node
normalized-kind tag
resolution state
```

The normalized-kind tag may be an internal compact representation, but reflection returns the first-class object described by this specification.

### 7.4 Absence representation

The AST and metadata model must distinguish:

```text
no annotation
```

from:

```text
an explicit annotation whose descriptor is Dynamic, Any, Object, Unit, None's type, or another type expression
```

No phase may fill an absent annotation with a special type unless a later checker mode explicitly requests a derived view. Raw reflection continues to report absence as `None`.

### 7.5 Class resolution

When a source annotation resolves to a class declaration:

- the normalized descriptor is that exact `Class` object;
- no wrapper is created;
- module qualification and source alias information remain separate metadata;
- repeated resolution within one module epoch returns the same class identity;
- unresolved generic arguments are not invented.

### 7.6 Protocol resolution

When a source annotation resolves to a protocol declaration:

- the normalized descriptor is that exact `Protocol` object;
- the protocol remains non-instantiable;
- its requirements are not copied into the annotation record;
- no conformance check occurs merely because it is referenced;
- repeated resolution returns the same protocol identity.

### 7.7 Generic-origin resolution

A bare generic declaration resolves to its declaration object. The compiler records that it declares parameters, but it must not rewrite:

```phalcom
Box
```

as:

```phalcom
Box<T>
```

Document 04 decides where an unapplied generic origin is legal as an annotation and which diagnostics apply. This document fixes only its descriptor behavior.

### 7.8 Type protocol bootstrap declaration

The compiler parses the canonical `Type` declaration exactly as another signature-only protocol declaration. It must not special-case executable bodies or silently transform `currentApplication` into a class-side requirement.

The compiler or bootstrap linker identifies the canonical descriptor using a trusted module/binding identity, not by searching for any protocol whose display name is `"Type"`.

### 7.9 Singleton intrinsic metadata

The compiler/VM must represent the `Type.currentApplication` intrinsic as a reserved executable entry distinct from protocol requirements. The minimum internal record contains:

```text
receiver descriptor identity = canonical Type
selector = currentApplication getter
implementation anchor = TypeRuntime.currentApplication
result type = Option<AppliedType>
reserved = true
native = true
source location = standard-library anchor when available
```

It must not appear in the requirement table emitted for the `Type` protocol.

### 7.10 Built-in surface integration

The compiler/bootstrap must ensure that the trusted `Class` and `Protocol` definitions expose the seven `Type` requirements. The methods may be:

- ordinary core Phalcom methods;
- native methods with the same source anchors;
- trusted intrinsic entries reflected as methods.

An implementation must choose one coherent route. It may not make `Class` or `Protocol` appear to conform only inside the checker while direct runtime messages fail.

### 7.11 Recognition and normalization checks

At metadata attachment time the implementation validates:

- recognized descriptor kind;
- all required selectors are available;
- `origin` is a recognized type expression;
- `arguments`, `typeParameters`, and `freeParameters` are immutable lists;
- argument elements are recognized type expressions;
- type-parameter elements are trusted `TypeParameter` descriptors once Document 03 is loaded;
- free parameters contain no duplicates;
- bare class/protocol shapes match the fixed table;
- repeated observations are stable during construction;
- the descriptor is complete and not a bootstrap shell.

The compiler must reject malformed metadata rather than silently replacing values with `Dynamic` or an opaque placeholder.

### 7.12 Compile-time execution limits

Normalizing a built-in descriptor must not execute arbitrary user conversion hooks. The compiler may invoke trusted descriptor methods whose classes are recognized by the typing runtime. It must not send speculative messages such as `asType`, `typeExpression`, or `normalize` to unknown objects.

This keeps annotation resolution deterministic and prevents a source annotation from becoming an unrestricted compile-time code-execution hook.

### 7.13 Incremental compilation and module reload

Class and protocol descriptor identities are scoped to a module/declaration epoch according to the module system. Recompiling or reloading a declaration may produce a new descriptor identity. Tools must not equate old and new descriptors solely by qualified name.

Caches keyed by descriptors must either:

- be epoch-local;
- observe declaration invalidation;
- use weak identity keys;
- or be rebuilt.

Document 17 defines persistent metadata identities and versioning in detail.

### 7.14 Checker obligations at this stage

A checker implementing only Documents 01–02 can:

- resolve and display bare class/protocol annotations;
- test type equivalence;
- report invalid or absent annotations accurately;
- preserve generic declaration metadata without applying it;
- expose type-expression reflection.

It cannot yet soundly decide generic application, subtyping, protocol conformance, variance, inference, block compatibility, or special-type consistency.

## 8. Interpreter and VM requirements

### 8.1 Trusted shells

The interpreter/VM must provide minimal shells for `Class` and `Protocol` before loading the typing module. Those shells must support enough name, identity, method installation, and protocol construction behavior to create the canonical `Type` descriptor.

The minimal shell is not the public final descriptor. User code must not observe unresolved fields, placeholder parameters, or missing type-expression methods.

### 8.2 Bare class operations

For every completed class object, the VM must preserve:

```text
origin = exact class object
arguments = immutable empty list
freeParameters = immutable empty list
substitute = exact class object
equivalence = receiver identity
```

The declared type-parameter list is attached after Document 03 metadata resolution. Before publication it must be complete or hidden.

### 8.3 Bare protocol operations

For every completed protocol object, including `Type`, the VM must preserve the same bare operations with protocol identity. Adding the `Type` surface must not:

- make protocols instantiable;
- create a superclass;
- create executable requirement bodies;
- mutate requirement lists;
- add per-protocol metaclasses;
- affect candidate classes.

### 8.4 Synthetic descriptor authority

Trusted synthetic descriptor constructors must validate authority before publishing an object. Native allocation may bypass the literal token field, but the effective capability check is mandatory.

An untrusted subclass or forged metadata record must not be able to enter an interning cache or compiled annotation table as if it were a built-in descriptor kind.

### 8.5 Intrinsic lookup

For the exact canonical `Type` receiver, lookup of the getter selector `currentApplication` resolves the singleton intrinsic before the ordinary `Protocol` method table.

For every other receiver, the ordinary object-model lookup path applies. Name equality is insufficient:

```phalcom
const OtherType = Protocol.new(
  owner: CurrentModule,
  name: #Type,
  typeParameters: const [],
  requirements: const [],
  attributes: const [],
  documentation: None,
  sourceLocation: None
)

OtherType.currentApplication
// ordinary doesNotUnderstand
```

### 8.6 Intrinsic reservation

User code must not:

- install the intrinsic on another object;
- replace its implementation;
- remove it;
- intercept it through protocol reopening;
- shadow it with a requirement;
- monkey-patch the canonical `Type` object;
- invoke the bootstrap installation primitive successfully after publication.

Attempts fail with `type.current_application.reserved_intrinsic` or the general reserved-selector diagnostic defined by later security specifications.

### 8.7 Current-application storage floor

Document 02 requires only a VM seam capable of returning `None` or the active `AppliedType`. Until applied forwarding is implemented, returning `None` is conforming for all ordinary execution.

Once Document 06 is implemented, the same method must expose the full fiber-local stack semantics without changing its selector or result type.

### 8.8 GC tracing

The VM traces:

- the canonical `Type` binding;
- the intrinsic method descriptor and implementation anchor;
- every synthetic descriptor edge;
- source metadata and owner references;
- type-parameter collections once present.

The intrinsic must not keep completed fibers or historical invocation frames alive. Current-application frame roots exist only while frames are active.

### 8.9 Exception behavior

Calls to `displayName`, `origin`, `arguments`, `typeParameters`, `freeParameters`, `substitute(using:)`, and `equivalentTo(_:)` may report malformed trusted metadata, but they must not leave partial normalization state in global caches.

`Type.currentApplication` itself does not throw during normal operation. If runtime frame metadata is corrupted, the VM reports a trusted-runtime error rather than returning an arbitrary descriptor.

### 8.10 Interpreter parity

A tree-walking interpreter and bytecode VM must return the same descriptor identities and reflective results. The interpreter may use direct object fields while the VM uses compact tags; this difference is not observable.

### 8.11 No implicit runtime validation

The VM must not consult annotations or `Type` descriptors when:

- entering an ordinary method;
- assigning a local or field;
- passing arguments;
- returning a value;
- allocating an object;
- resolving a selector;
- probing an inline cache.

Explicit library messages and optional checker/tooling phases are the only consumers introduced here.

## 9. Reflection and metadata

### 9.1 Common reflection surface

Every normalized type expression exposes:

```phalcom
displayName -> String
origin -> Type
arguments -> const List<Type>
typeParameters -> const List<TypeParameter>
freeParameters -> const List<TypeParameter>
substitute(using: TypeEnvironment) -> Type
equivalentTo(other: Type) -> Bool
hash -> Int
```

`hash` is an object-model requirement for keyed use even though it is not a `Type` protocol requirement. The same applies to ordinary identity and `class` reflection.

### 9.2 Bare class reflection

Reflecting a bare class annotation returns the class object itself:

```phalcom
method.returnType.unwrap === String
```

Tools can immediately navigate class methods, fields, superclass, documentation, and source through the existing class reflection APIs.

### 9.3 Bare protocol reflection

Reflecting a bare protocol annotation returns the protocol object itself:

```phalcom
method.returnType.unwrap === Drawable
method.returnType.unwrap.requirements === Drawable.requirements
```

No requirement copy or protocol-view wrapper is created.

### 9.4 Source form versus normalized descriptor

Reflection must preserve both concepts:

- **normalized descriptor:** the object used for equivalence and checker semantics;
- **source form:** original spelling, aliases, qualification, and source range.

For example, an imported alias may preserve source spelling `Text` while the normalized descriptor is `String`. `displayName` belongs to the descriptor and does not reproduce every source alias.

Document 17 defines the final APIs and encoding. This document forbids discarding source range merely because the normalized descriptor already has a name.

### 9.5 Absent annotations

Method, parameter, field, and constructor reflection must use `Option<Type>` or the later source/normalized annotation record. `None` means no annotation was written or generated. It does not mean the `None` value type.

### 9.6 Type descriptor class reflection

For a synthetic descriptor `D`:

```phalcom
D.class
```

returns its concrete descriptor implementation class, such as future `AppliedType`. This runtime class identity is not automatically the type expression’s `origin`.

### 9.7 Intrinsic reflection

The canonical `Type` descriptor reports `currentApplication` through ordinary method reflection APIs that support singleton intrinsics. The reflection object must make its reserved/native nature visible.

It must not be returned by:

```phalcom
Type.requirements
```

because it is not a conformance requirement.

### 9.8 Deterministic collections

`arguments`, `typeParameters`, and `freeParameters` are immutable and deterministic. Repeated reflection returns either the same frozen list object or equivalent frozen lists with identical order. Implementations should prefer stable cached collections for built-ins.

### 9.9 Display and documentation tools

Tools should render:

- bare classes using class names and optional qualification;
- bare protocols using protocol names and a visible protocol marker where helpful;
- synthetic descriptors using `displayName` plus kind-specific structure;
- open descriptors with free-parameter navigation;
- absent annotations distinctly from explicit `Dynamic` once available.

### 9.10 Metadata integrity boundary

Raw bytecode or native metadata must not create a descriptor that lies about its kind, origin, arguments, or free parameters. The loader validates records before attaching them to reflective members.

Malformed records fail module loading; they are not exposed as partially working reflection objects.

## 10. Validation and diagnostics

### 10.1 Diagnostic contract

Every diagnostic has:

- stable code;
- human-readable message;
- primary source range when source exists;
- structured details;
- zero or more secondary ranges;
- deterministic selection when multiple violations exist.

Runtime errors without source still carry `location: None` and structured descriptor/value details.

### 10.2 Diagnostic codes

| Code | Meaning | Primary range / key fields |
|---|---|---|
| `type.expression.invalid` | resolved value is not a recognized type expression | annotation expression; `candidate`, `actualClass` |
| `type.expression.untrusted_descriptor` | descriptor kind is structurally plausible but not trusted for normalized metadata | annotation expression; `descriptorClass` |
| `type.expression.incomplete_descriptor` | bootstrap shell or partial descriptor escaped | declaration/metadata record; `descriptor` |
| `type.expression.invalid_origin` | `origin` is absent or not a recognized `Type` | descriptor declaration/metadata; `origin` |
| `type.expression.invalid_arguments` | arguments are mutable, malformed, or contain non-types | offending arguments range; `index`, `value` |
| `type.expression.invalid_type_parameters` | declared parameter list is malformed | generic declaration range; `index`, `value` |
| `type.expression.invalid_free_parameters` | free list is mutable, duplicated, unstable, or wrong | descriptor range; `parameter`, `firstIndex`, `duplicateIndex` |
| `type.expression.mutable_descriptor` | synthetic descriptor changes observable type state | descriptor construction; `field` or `selector` |
| `type.expression.invalid_substitution` | open descriptor does not substitute recursively or returns a non-type | substitution call/source descriptor; `descriptor`, `result` |
| `type.expression.equivalence_contract` | equivalence violates reflexivity, symmetry, or trusted kind rules | descriptor metadata; `left`, `right` |
| `type.expression.hash_contract` | equivalent descriptors have incompatible hashes | descriptor metadata; `leftHash`, `rightHash` |
| `type.expression.construction_authority` | trusted descriptor constructor used without authority | constructor call; `descriptorClass` |
| `type.expression.subclass_responsibility` | concrete descriptor omitted required behavior | descriptor class/member; `selector` |
| `type.expression.invalid_builtin_receiver` | built-in class/protocol semantics used on another object | runtime call; `receiver` |
| `type.current_application.reserved_intrinsic` | user attempted to install, replace, or remove the intrinsic | declaration/mutation site; `selector`, `receiver` |
| `type.current_application.invalid_receiver` | bootstrap attempted installation on a non-canonical descriptor | bootstrap record; `receiver`, `expected` |
| `type.metadata.malformed` | encoded type metadata failed integrity validation | metadata record/source annotation; `reason`, `kind` |

Later documents may add more specific subcodes but must preserve these meanings.

### 10.3 Validation order

Normalization validates in this deterministic order:

1. candidate is non-absent and of a recognized trusted kind;
2. descriptor is complete and published or in an authorized bootstrap completion path;
3. `displayName` is a stable string;
4. `origin` is a recognized complete type expression;
5. `arguments` is immutable and every element is valid;
6. `typeParameters` is immutable and every element is valid when Document 03 is active;
7. `freeParameters` is immutable, valid, ordered, and duplicate-free;
8. descriptor-kind-specific shape rules hold;
9. substitution identity rule holds for closed built-in descriptors;
10. equivalence/hash invariants hold for trusted debug/conformance validation.

Production implementations may omit repeated expensive contract probes after trusted construction, but loaders and constructors must perform equivalent validation once.

### 10.4 Source ranges

For an invalid annotation expression, the primary range covers the narrowest expression that failed normalization. For a malformed generated member annotation, the primary range covers the generating declaration and a secondary range identifies the generator attribute where available.

For intrinsic reservation violations, the primary range covers the attempted declaration or mutation selector.

### 10.5 Runtime exception mapping

Runtime normalization failures map diagnostics to:

- `InvalidTypeExpressionError` for invalid objects and shapes;
- `UntrustedTypeDescriptorError` for unsupported descriptor kinds;
- `TypeDescriptorAuthorityError` for construction authority violations;
- `TypeSubstitutionError` for substitution contract violations;
- `TypeEquivalenceError` for trusted equivalence/hash failures;
- `ReservedTypeIntrinsicError` for current-application intrinsic mutation.

The diagnostic code remains the stable machine interface; exception subclasses are convenience categories.

### 10.6 No fallback coercion

The implementation must not recover from an invalid type expression by:

- replacing it with `Dynamic`;
- using its runtime class as the intended type;
- accepting its display string;
- wrapping it in an opaque descriptor;
- ignoring malformed arguments;
- dropping duplicate free parameters silently.

Such repair would make reflection and checking dependent on implementation accidents.

## 11. Interaction with earlier specifications

### 11.1 Document 01 — Protocol Foundation

This document uses `@protocol` exactly as Document 01 defines it. `Type` is a first-class `Protocol` descriptor, has signature-only requirements, cannot be instantiated, and does not install methods into conformers.

The `Type.currentApplication` singleton intrinsic does not alter `Type.requirements`, create an executable requirement, or generalize protocol descriptor methods. It is one trusted API on one canonical object.

### 11.2 Protocol type-expression behavior

Adding `origin`, `arguments`, `typeParameters`, `freeParameters`, `substitute(using:)`, and `equivalentTo(_:)` to the trusted `Protocol` class does not change protocol identity or conformance semantics. Every protocol remains a named descriptor with declaration-object identity.

### 11.3 Class declarations and object model

Class objects participate directly. This does not make class instances retain annotations, does not add fields to instances, and does not alter superclass lookup.

### 11.4 Selectors

The `Type` protocol selectors are ordinary canonical selectors. Type annotations appearing in their signatures are metadata and are not part of selector identity.

The singleton `currentApplication` selector is a getter selector. Its reservation concerns mutation/installation on the canonical descriptor, not general use of the same selector name by unrelated classes.

### 11.5 Attributes and `@native`

`@native` marks trusted source anchors but does not permit semantic divergence. `@protocol` remains a declaration-product attribute. No new user-facing decorator is introduced by this document.

### 11.6 Modules

The canonical `Type` descriptor is identified by trusted standard-library module binding plus declaration identity. A user module may bind another value named `Type` without acquiring the singleton intrinsic.

### 11.7 Reflection

Existing member reflection should return normalized type objects directly. Source-form annotation reflection is expanded in Document 17.

### 11.8 Immutability

Synthetic descriptors and their collections are immutable. The exact `@immutable` inheritance and enforcement rules follow the language’s immutable-object specification. Native storage must provide at least the same guarantees.

### 11.9 Phase 1 reference package

The Phase 1 package correctly proposed `Type` as a protocol, direct class/protocol participation, and `TypeDescriptor` as a synthetic base. This document changes or clarifies:

- executable `currentApplication` is removed from the protocol body;
- bare generic origins have no free parameter occurrences;
- compiler metadata accepts trusted descriptor kinds rather than arbitrary structural objects;
- built-in integration and bootstrap order are explicit;
- equivalence, hash, normalization, absence, and intrinsic reflection are normative;
- `isGeneric` and `isApplied` are not foundation protocol requirements because they conflate declaration and expression questions.

## 12. Examples

### 12.1 Bare class identity

```phalcom
const first = TypeRuntime.normalize(String)
const second = TypeRuntime.normalize(String)

System.assert(first === String)
System.assert(second === String)
System.assert(first === second)
```

### 12.2 Bare protocol identity

```phalcom
@protocol
class Serializable {
  serialize() -> Bytes
}

const normalized = TypeRuntime.normalize(Serializable)

System.assert(normalized === Serializable)
System.assert(normalized.class === Protocol)
```

### 12.3 Separately declared protocols are not equivalent

```phalcom
const First = Protocol.new(
  owner: CurrentModule,
  name: #Readable,
  typeParameters: const [],
  requirements: const [],
  attributes: const [],
  documentation: None,
  sourceLocation: None
)

const Second = Protocol.new(
  owner: CurrentModule,
  name: #Readable,
  typeParameters: const [],
  requirements: const [],
  attributes: const [],
  documentation: None,
  sourceLocation: None
)

System.assert(First !== Second)
System.assert(First.equivalentTo(Second).not)
```

### 12.4 Class and protocol with the same display name

```phalcom
class Item {}

const ItemProtocol = Protocol.new(
  owner: CurrentModule,
  name: #Item,
  typeParameters: const [],
  requirements: const [],
  attributes: const [],
  documentation: None,
  sourceLocation: None
)

System.assert(Item.displayName == ItemProtocol.displayName)
System.assert(Item.equivalentTo(ItemProtocol).not)
```

### 12.5 Bare generic origin

```phalcom
class Pair<A, B> {}

System.assert(Pair.typeParameters.size == 2)
System.assert(Pair.freeParameters.isEmpty)
System.assert(Pair.origin === Pair)
System.assert(Pair.arguments.isEmpty)
```

### 12.6 Closed substitution is identity-preserving

```phalcom
const environment = TypeEnvironment.empty
const result = String.substitute(using: environment)

System.assert(result === String)
```

### 12.7 Type is self-describing

```phalcom
System.assert(Type.class === Protocol)
System.assert(Type.origin === Type)
System.assert(Type.arguments.isEmpty)
System.assert(Type.equivalentTo(Type))
```

### 12.8 The intrinsic is not a requirement

```phalcom
const selector = Selector.getter(#currentApplication)

System.assert(
  Type.requirementFor(
    selector: selector,
    side: ProtocolRequirementSide.ClassSide
  ) == None
)

System.assert(Type.methodFor(selector) != None)
```

The exact method-reflection API may use a singleton-member lookup introduced in Document 17; the semantic distinction is normative now.

### 12.9 Other protocols do not receive the intrinsic

```phalcom
@protocol
class Drawable {
  draw() -> Unit
}

Drawable.currentApplication
// error: Drawable does not understand currentApplication
```

### 12.10 Outside applied forwarding

```phalcom
System.assert(Type.currentApplication == None)
```

### 12.11 Annotation reflection returns the descriptor

```phalcom
class Service {
  parse(text: String) -> Serializable {
    throw NotImplementedError.new()
  }
}

const method = Service.methodFor(#parse(text:)).unwrap

System.assert(method.parameters.at(0).type.unwrap === String)
System.assert(method.returnType.unwrap === Serializable)
```

### 12.12 Missing annotation remains absent

```phalcom
class Identity {
  value(input) {
    return input
  }
}

const method = Identity.methodFor(#value(_:)).unwrap

System.assert(method.parameters.at(0).type == None)
System.assert(method.returnType == None)
```

### 12.13 Invalid annotation object

```phalcom
const NotAType = Object.new()

class Invalid {
  value(input: NotAType) {
    return input
  }
}
```

Compilation fails with `type.expression.invalid` at `NotAType`.

### 12.14 Display name is not identity

```phalcom
System.assert(String.displayName == "String")
System.assert(String.equivalentTo(String))

// No code may conclude equivalence merely from the display string.
```

### 12.15 Synthetic identity default

A trusted singleton synthetic descriptor that does not override equivalence uses identity:

```phalcom
System.assert(FirstSpecial.equivalentTo(FirstSpecial))
System.assert(FirstSpecial.equivalentTo(SecondSpecial).not)
```

Later special-type declarations normally use canonical singleton identities.

### 12.16 No runtime argument enforcement

```phalcom
class Box<T> {
  var _value

  @constructor
  new(value) {
    _value = value
  }
}

const value: Box<Int> = Box.new(value: "text")

// Ordinary runtime execution permits this. An optional checker may diagnose it.
System.assert(value.class === Box)
```

### 12.17 No type-directed dispatch

```phalcom
class Printer {
  print(value: Int) {
    return "int"
  }
}

// The annotation does not create an overload key or alternate selector.
System.assert(Printer.methodFor(#print(value:)) != None)
```

### 12.18 User-defined object is not automatically trusted metadata

```phalcom
class PretendType {
  displayName -> String { return "Pretend" }
  origin -> Type { return self }
  arguments -> const List<Type> { return const [] }
  typeParameters -> const List<TypeParameter> { return const [] }
  freeParameters -> const List<TypeParameter> { return const [] }
  substitute(using: TypeEnvironment) -> Type { return self }
  equivalentTo(other: Type) -> Bool { return self === other }
}

const pretend = PretendType.new()
TypeRuntime.normalize(pretend)
// UntrustedTypeDescriptorError in the first-version compiler/runtime contract.
```

## 13. Conformance tests

### 13.1 Test categories

A conforming implementation must provide tests for:

- class and protocol direct participation;
- generic-origin behavior;
- normalization identity;
- descriptor immutability;
- equivalence/hash laws;
- absence preservation;
- invalid and untrusted descriptors;
- singleton intrinsic identity and reservation;
- bootstrap ordering and failure isolation;
- GC tracing;
- cross-engine parity;
- non-interference with dispatch and allocation.

### 13.2 Positive fixture: class type surface

```phalcom
class Plain {}

System.assert(Plain.displayName == "Plain")
System.assert(Plain.origin === Plain)
System.assert(Plain.arguments == const [])
System.assert(Plain.typeParameters == const [])
System.assert(Plain.freeParameters == const [])
System.assert(Plain.substitute(using: TypeEnvironment.empty) === Plain)
System.assert(Plain.equivalentTo(Plain))
```

### 13.3 Positive fixture: protocol type surface

```phalcom
@protocol
class EmptyProtocol {}

System.assert(EmptyProtocol.displayName == "EmptyProtocol")
System.assert(EmptyProtocol.origin === EmptyProtocol)
System.assert(EmptyProtocol.arguments == const [])
System.assert(EmptyProtocol.typeParameters == const [])
System.assert(EmptyProtocol.freeParameters == const [])
System.assert(
  EmptyProtocol.substitute(using: TypeEnvironment.empty) === EmptyProtocol
)
```

### 13.4 Positive fixture: normalization identity

```phalcom
System.assert(TypeRuntime.normalize(Int) === Int)
System.assert(TypeRuntime.normalize(Type) === Type)
System.assert(TypeRuntime.normalize(EmptyProtocol) === EmptyProtocol)
```

### 13.5 Positive fixture: generic origin

```phalcom
class Mapping<K, V> {}

System.assert(Mapping.typeParameters.size == 2)
System.assert(Mapping.freeParameters.isEmpty)
System.assert(Mapping.substitute(using: TypeEnvironment.empty) === Mapping)
```

### 13.6 Positive fixture: intrinsic surface

```phalcom
System.assert(Type.currentApplication == None)
System.assert(Type.respondsTo(#currentApplication))
System.assert(EmptyProtocol.respondsTo(#currentApplication).not)
```

If `respondsTo` does not include singleton intrinsics in the repository’s reflection model, the equivalent exact member lookup must be used.

### 13.7 Negative fixture: non-type annotation

```phalcom
const Value = 42

class Bad {
  method(input: Value) {}
}
```

Expected diagnostic:

```text
code: type.expression.invalid
primary: Value annotation expression
candidate: 42
```

### 13.8 Negative fixture: mutable synthetic descriptor

Construct a malformed trusted test descriptor whose `arguments` changes between observations. Normalization must fail with:

```text
code: type.expression.mutable_descriptor
selector: arguments
```

The production public API need not expose a way to create this fixture; a VM conformance harness may inject it.

### 13.9 Negative fixture: invalid origin

A synthetic descriptor returning an ordinary object from `origin` fails with:

```text
code: type.expression.invalid_origin
```

### 13.10 Negative fixture: mutable arguments collection

A synthetic descriptor returning a mutable list from `arguments` fails with:

```text
code: type.expression.invalid_arguments
```

### 13.11 Negative fixture: duplicate free parameter

After Document 03 supplies parameter objects, a descriptor returning `const [T, T]` fails with:

```text
code: type.expression.invalid_free_parameters
firstIndex: 0
duplicateIndex: 1
```

### 13.12 Negative fixture: wrong substitution result

A trusted test descriptor with a free parameter that returns a non-type from `substitute(using:)` fails with:

```text
code: type.expression.invalid_substitution
```

### 13.13 Equivalence law fixture

For every built-in descriptor sample `A`, `B`, and `C`, the suite checks:

```phalcom
System.assert(A.equivalentTo(A))
System.assert(A.equivalentTo(B) == B.equivalentTo(A))

if A.equivalentTo(B) and B.equivalentTo(C) {
  System.assert(A.equivalentTo(C))
}

if A.equivalentTo(B) {
  System.assert(A.hash == B.hash)
}
```

### 13.14 Negative fixture: forged construction authority

```phalcom
TypeDescriptor._trustedNew(token: Object.new())
```

Expected diagnostic:

```text
code: type.expression.construction_authority
```

### 13.15 Negative fixture: intrinsic installation on another protocol

```phalcom
TypeRuntime.installCurrentApplicationIntrinsic(on: EmptyProtocol)
```

Expected diagnostic:

```text
code: type.current_application.invalid_receiver
```

### 13.16 Negative fixture: intrinsic replacement

Attempting to define or install a singleton `currentApplication` member on canonical `Type` after bootstrap must fail with:

```text
code: type.current_application.reserved_intrinsic
```

### 13.17 Bootstrap fixture

A bootstrap conformance test asserts this order:

```text
Protocol shell available
→ Type declaration indexed
→ Type descriptor completed
→ built-in Type surfaces complete
→ currentApplication intrinsic installed
→ Type published
→ general annotation resolution enabled
```

No user callback or module initializer may observe an intermediate step.

### 13.18 Bootstrap failure isolation

If intrinsic installation fails, the standard typing module must not publish a partially functioning `Type` binding. A subsequent import reports module initialization failure rather than returning the incomplete descriptor.

### 13.19 GC fixture

Create temporary synthetic descriptors and metadata graphs, remove all strong user references, force collection, and verify:

- unreachable descriptors are collectible unless intentionally interned;
- the canonical `Type` descriptor remains rooted;
- no historical current-application frame remains rooted after return;
- owner/component cycles are traced without use-after-free.

### 13.20 Dispatch non-interference fixture

Run the same ordinary message-send corpus with annotation metadata enabled and disabled. Selector resolution, selected executable, runtime receiver, allocation class, and inline-cache key must be identical.

### 13.21 Cross-engine fixture

For interpreter and VM, compare object identity/equivalence results for:

```text
Class
Protocol
Type
non-generic user class
non-generic user protocol
generic class origin
generic protocol origin
```

The engines must agree on all observable results.

## 14. Native implementation latitude

### 14.1 General rule

Native code may implement trusted descriptor recognition, bootstrap integration, construction authority, singleton intrinsics, shape validation, compact metadata, and caching. The visible behavior remains the source contract in Section 6.

### 14.2 Operations that may be native

The following are suitable native responsibilities:

- creation and validation of `TypeDescriptorConstructionToken`;
- trusted descriptor-kind recognition;
- built-in `Class` and `Protocol` method integration;
- stable empty-list reuse;
- descriptor-shape validation;
- singleton-intrinsic installation and lookup;
- current-application frame read access;
- metadata loader validation;
- GC tracing and weak-cache integration;
- debug equivalence/hash contract probes.

### 14.3 Operations that should remain ordinary source when practical

These can remain ordinary Phalcom code:

- `TypeDescriptor` default methods;
- list-based free-parameter collection;
- collection equivalence;
- diagnostic construction;
- display delegation;
- most explicit reflection helpers.

### 14.4 Allowed internal representations

A VM may represent a type reference as:

- a tagged pointer to `Class`, `Protocol`, or a synthetic descriptor;
- a compact handle into a GC-managed arena;
- an interned descriptor ID resolved to an object for reflection;
- a constant-pool entry with lazy object materialization.

Reflection and identity rules must remain exact. In particular, a bare class/protocol annotation must materialize as the original descriptor object.

### 14.5 Forbidden native divergences

Native code must not:

- return class/protocol wrappers;
- treat display names as identity;
- make absent annotations equal `Dynamic`;
- expose mutable argument or parameter lists;
- accept arbitrary user objects as trusted metadata silently;
- install `currentApplication` on every protocol;
- include the intrinsic in protocol requirements;
- consult type descriptors during ordinary value dispatch;
- retain current-application contexts after frame exit;
- change equivalence according to cache or compilation mode.

### 14.6 Performance expectations

Bare descriptor operations should be constant-time and allocation-free after bootstrap. Repeated `arguments` and `freeParameters` calls should reuse a canonical immutable empty list where possible. Normalization of a previously validated trusted descriptor should be constant-time or amortized constant-time.

These are performance requirements, not permission to weaken validation at trust boundaries.

## 15. Non-goals and deferred work

### 15.1 Complete type lattice

This document does not define subtype, consistency, acceptance, joins, meets, or special-type ordering. Document 07 owns those relations.

### 15.2 Generic application

`Box<Int>` syntax, `AppliedType`, `TypeConstructor`, arity validation, interning, and application identity are Document 04 concerns.

### 15.3 Substitution environment implementation

The `TypeEnvironment` name and `substitute(using:)` selector are fixed, but environment representation and recursive algorithms are specified in Document 05.

### 15.4 Structural protocol conformance

That a class or descriptor has the `Type` selectors is a built-in bootstrap fact here. The general algorithm for testing arbitrary protocol conformance is deferred to Document 10.

### 15.5 User-extensible authoritative descriptor kinds

No registration, serialization, or checker-extension API for arbitrary descriptor kinds is defined. A future proposal must address determinism, immutability, hashing, versioning, security, and portable encoding.

### 15.6 Type-directed dispatch

Types do not become selector components, overload keys, multimethod guards, or inline-cache dimensions.

### 15.7 Runtime value enforcement

No automatic parameter, return, field, collection-element, or constructor validation is introduced.

### 15.8 Raw generic legality

Whether bare generic origins may appear in all annotation positions, produce warnings, or require explicit application is deferred to Documents 03–04 and checker modes in Document 19.

### 15.9 Source alias semantics

Alias preservation and normalization are specified in Document 16, with metadata encoding in Document 17.

### 15.10 Full current-application behavior

Frame pushing, propagation, nesting, fiber isolation, exception restoration, and DNU forwarding are Document 06 concerns.

### 15.11 Pattern matching over type descriptors

This document provides object/reflection APIs only. It does not introduce dedicated type-pattern syntax.

### 15.12 Descriptor serialization

Persistent and cross-module encoding details are deferred to Document 17. This document only requires validated recognized identities at runtime.

## 16. Normative invariants

A conforming implementation must preserve every invariant below.

1. `Type` is a first-class signature-only `Protocol` descriptor.
2. `Type` contains no executable default implementations.
3. Existing `Class` objects are type expressions directly.
4. Existing `Protocol` objects are type expressions directly.
5. No `ClassType` or `ProtocolType` wrapper is observable for bare descriptors.
6. Normalizing a bare class returns the exact class object.
7. Normalizing a bare protocol returns the exact protocol object.
8. Bare class and protocol `origin` is `self`.
9. Bare class and protocol `arguments` is an immutable empty list.
10. Bare class and protocol `freeParameters` is an immutable empty list.
11. A generic origin’s `typeParameters` may be non-empty while its `freeParameters` remains empty.
12. A bare generic origin is not implicitly rewritten to an open application.
13. Bare class and protocol substitution returns the exact receiver.
14. Bare class and protocol equivalence is descriptor identity.
15. Display names are not identity or equivalence keys.
16. Separately declared protocols remain non-equivalent even when structurally identical.
17. `TypeDescriptor` is abstract, and synthetic descriptors use it where appropriate while remaining ordinary objects.
18. `TypeDescriptor` does not change `Class` or `Protocol` inheritance.
19. Trusted synthetic descriptors are observationally immutable.
20. Descriptor argument, type-parameter, and free-parameter collections are immutable and deterministic.
21. Free parameters are duplicate-free and ordered by first occurrence.
22. Every `origin` and every `arguments` element is a recognized type expression.
23. Substitution is pure, deterministic, and identity-preserving when no component changes.
24. Substitution returns a normalized type expression.
25. Equivalence is reflexive, symmetric, and transitive for valid descriptors.
26. Equivalent descriptors have equal hashes.
27. Equivalence is distinct from subtyping, consistency, acceptance, conformance, and value equality.
28. Missing annotations remain reflectively absent.
29. Missing annotations are not silently converted to `Dynamic`, `Any`, or `Object`.
30. Normalization never invokes arbitrary user coercion hooks.
31. First-version compiler metadata accepts only recognized trusted descriptor kinds.
32. Untrusted descriptor-like objects may be rejected without violating structural protocol semantics.
33. The canonical `Type` descriptor is itself a type expression through built-in protocol behavior.
34. Bootstrap self-description does not require the general structural-conformance algorithm.
35. `Type.currentApplication` is a public getter on the exact canonical `Type` descriptor.
36. `Type.currentApplication` is not a `Type` protocol requirement.
37. `Type.currentApplication` is not a protocol default method.
38. The intrinsic is backed by the visible `TypeRuntime.currentApplication` source anchor.
39. The intrinsic is installed before `Type` is published and cannot be installed again.
40. Other protocol descriptors do not acquire the intrinsic.
41. User code cannot replace, remove, shadow, or intercept the intrinsic on canonical `Type`.
42. Outside active applied forwarding, `Type.currentApplication` returns `None`.
43. Reading `Type.currentApplication` never changes method selection.
44. Class and protocol type behavior is available through direct runtime messages, not checker-only fiction.
45. Type-expression metadata never implicitly changes ordinary selector identity.
46. Type-expression metadata never implicitly changes method lookup or overload resolution.
47. Type-expression metadata never implicitly changes layout, allocation, or ordinary value inline-cache identity; the explicit `Type.currentApplication` intrinsic may guard the exact canonical descriptor receiver.
48. Type-expression metadata never automatically validates runtime values.
49. Native representations preserve original bare descriptor identity through reflection.
50. Malformed metadata is rejected rather than repaired heuristically.
51. Failed bootstrap publishes no partial `Type` descriptor or half-installed intrinsic.
52. GC traces all live descriptor graphs and does not retain historical invocation contexts.
53. Interpreter and VM agree on descriptor identity and equivalence.
54. Later documents may add descriptor kinds and relations but may not reintroduce bare wrappers or type-directed dispatch.

---

**End of Document 02.** The next specification is `03-type-parameters-and-generic-signatures.md`, which defines declaration-site generic grammar, `TypeParameter`, owner/index identity, variance metadata, bounds, finite constraints, and `GenericSignature` while preserving the type-expression rules established here.

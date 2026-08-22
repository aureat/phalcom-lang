# Type Parameters and Generic Signatures

- **Status:** Proposed normative design; not a claim of current compiler or VM support
- **Date:** 2026-07-23
- **Depends on:** Document 01 — Protocol Foundation; Document 02 — Type Expression Foundation; declaration identity; immutable collections; class, protocol, method, parameter, module, and source reflection; trusted bootstrap shells
- **Supersedes:** the `Variance`, `TypeParameter`, `TypeParameterOwner`, and `GenericSignature` fragments in the Phase 1 reference package wherever they conflict with this document
- **Superseded by:** none
- **Related ADRs and specifications:** `docs/spec/design/typing/01-protocol-foundation.md`, `docs/spec/design/typing/02-type-expression-foundation.md`, the current selector, class declaration, method declaration, module, reflection, `@native`, `@protocol`, and immutable-value specifications, and Documents 04–12, 14–21 of this series

This document is the third normative part of the Phalcom typing specification series. It defines generic declaration headers, first-class type parameters, declaration-site variance metadata, upper bounds, finite constraint sets, generic signatures, ownership, identity, lexical resolution, trusted construction, diagnostics, and the reflection surface shared by generic classes, protocols, and methods.

The visible Phalcom source in Section 6 is normative. Native implementations may replace selected `@native` methods only when they preserve that source contract exactly.

## 1. Purpose and scope

### 1.1 Purpose

A generic declaration introduces names such as `T`, `K`, and `V` that are used inside type annotations. Those names cannot be represented as strings or compiler-only variables if Phalcom intends typing metadata to remain reflectively observable. Each declaration therefore owns immutable first-class `TypeParameter` objects.

For example:

```phalcom
class Pair<A, B> {
  first -> A
  second -> B
}
```

creates one ordinary class descriptor, `Pair`, and two parameter descriptors. Reflection observes:

```phalcom
const a = Pair.typeParameters.at(0)
const b = Pair.typeParameters.at(1)

System.assert(a.name == #A)
System.assert(a.owner === Pair)
System.assert(a.index == 0)
System.assert(b.name == #B)
System.assert(b.owner === Pair)
System.assert(b.index == 1)
```

The parameter name is descriptive. Identity is the pair:

```text
owner descriptor identity + zero-based declaration index
```

This rule is required because two unrelated declarations may both use the source name `T`, because a nested generic method may shadow an outer name, and because renaming a parameter must not change the meaning of already normalized metadata within the same declaration identity.

Generic metadata remains non-dispatching:

> Type parameters, variance, bounds, constraints, and generic signatures are explicitly reflectable metadata. They never implicitly alter selector identity, ordinary method lookup, overload resolution, allocation, instance layout, inline-cache identity, or automatic value validation.

### 1.2 In scope

This document specifies:

- generic parameter-list grammar on class-shaped declarations and methods;
- generic classes, protocols, instance methods, class-side methods, and protocol requirements as type-parameter owners;
- `Variance.Invariant`, `Variance.Covariant`, and `Variance.Contravariant` as real immutable reflected values;
- invariance as the default for unmarked declaration-site parameters;
- the first-version restriction that method-owned parameters are invariant;
- `T: Bound` as an upper-bound declaration;
- `T in (A, B)` as a finite constraint-set declaration;
- the prohibition on declaring both a bound and constraints on one parameter;
- `TypeParameterSpec` as the ownerless public manual-construction input used by Document 01;
- `TypeParameter` as an immutable synthetic `Type` expression based on `TypeDescriptor`;
- owner-plus-index identity, equivalence, hashing, and GC ownership;
- `TypeParameterOwner` and `GenericSignature`;
- source ordering, lexical scope, shadowing, and annotation resolution;
- first-version recursive-restriction rules and cycle diagnostics;
- argument validation hooks consumed by Document 04;
- compiler, AST, metadata, interpreter, VM, bootstrap, reflection, and diagnostic obligations;
- positive, negative, reflection, identity, bootstrap, and malformed-metadata fixtures.

### 1.3 Out of scope

The following are assigned to later documents:

- angle application, `TypeConstructor`, `AppliedType`, arity enforcement at `<...>`, and interning: Document 04;
- `TypeEnvironment`, `TypeBinding`, complete recursive substitution, and applied member views: Document 05;
- applied class-side forwarding and invocation context: Document 06;
- the complete subtype, consistency, and acceptance relations used by bound validation: Document 07;
- variance-position checking and generic subtyping: Document 08;
- guarded F-bounds, intersection bounds, full constraint solving, inference, and promotion behavior: Document 09;
- structural protocol bounds and conformance: Document 10;
- generic abstract obligations: Document 11;
- generic superclass applications and inherited substitution: Document 12;
- generic generated declarations for `@data`, `@immutable`, `@sealed`, and `@variant`: Documents 14–15;
- final metadata encoding and lazy/eager resolution choices: Document 17;
- hardened bootstrap authority and malformed-metadata security: Document 18;
- checker modes, warnings, LSP presentation, and compatibility: Document 19.

This document defines stable representation and declaration rules that those later documents consume. Later documents may add relations and supported restriction forms but may not change parameter identity, source order, or the default variance rule.

### 1.4 Design alternatives and final decisions

#### 1.4.1 Compiler-only variables versus first-class objects

Serious options:

1. Erase type parameters after checking and preserve only textual annotations.
2. Preserve names and indexes as passive metadata records but provide no `Type` behavior.
3. Construct immutable `TypeParameter` objects that are themselves type expressions.

Decision: **Option 3.** A type parameter may occur anywhere a type expression is expected, must participate in substitution and equivalence, and must be available to reflective libraries.

#### 1.4.2 Parameter identity

Serious options:

1. Textual name.
2. Globally generated opaque identifier.
3. Owner identity plus declaration index.

Decision: **Option 3.** It is deterministic, locally derivable, stable for the lifetime of the declaration descriptor, independent of spelling, and sufficient to distinguish every parameter owned by a declaration. Implementations may additionally store an opaque identifier, but it cannot replace the normative owner/index identity.

#### 1.4.3 Variance defaults

Serious options:

1. Infer variance from use sites.
2. Make every unmarked parameter covariant.
3. Make every unmarked parameter invariant and permit explicit `out` and `in`.

Decision: **Option 3.** Published variance remains visible in source and cannot silently change when a member is edited. Document 08 checks whether declared variance is legal.

#### 1.4.4 Variance on method-owned parameters

Serious options:

1. Permit `in` and `out` on generic methods even though methods are not subtype-forming generic constructors.
2. Parse and preserve the markers but ignore them.
3. Require method-owned parameters to be invariant.

Decision: **Option 3.** `in` and `out` are declaration-site variance declarations for class and protocol type constructors. Generic methods may declare bounds and constraints but their parameters are invariant. A future higher-rank callable design may introduce separate callable variance concepts without overloading this syntax.

#### 1.4.5 Bounds and finite constraints

Serious options:

1. Use one syntax and infer whether a list means a union-like bound or a finite solver domain.
2. Use `:` for both bounds and constraints, as some languages do.
3. Use `:` for one upper bound and `in (...)` for a finite constraint set.

Decision: **Option 3.** The two declarations have different semantics and different inference behavior. They must remain distinct in source and reflection.

#### 1.4.6 Defaults

Serious options:

1. Implement default type arguments immediately.
2. Omit defaults from the object model and add storage later.
3. Reserve a reflected `default` slot now, require it to be `None`, and reject source/manual defaults until a later specification.

Decision: **Option 3.** The long-term shape is stable, but Document 04 need not solve omitted-argument application or partial/default interactions.

#### 1.4.7 Same-signature recursive restrictions

Serious options:

1. Permit arbitrary recursive bounds immediately.
2. Reject every restriction that mentions any type parameter.
3. Permit references to enclosing-owner parameters, but reject references to parameters owned by the same signature until Document 09 defines guarded F-bounds and recursive solving.

Decision: **Option 3.** This supports useful nested generic declarations without introducing an underspecified recursive constraint solver. `T: Comparable<T>` is reserved syntax and is rejected by the first-version validator with a diagnostic directing the implementation to Document 09.

#### 1.4.8 Lexical shadowing

Serious options:

1. Reject a nested parameter whose name matches any enclosing type parameter.
2. Permit lexical shadowing and resolve the nearest declaration.
3. Merge parameters with matching names.

Decision: **Option 2.** Type parameter names follow ordinary lexical shadowing. Identity prevents accidental merging. Tooling may offer a style warning later, but shadowing is not a type-system error.

#### 1.4.9 Empty generic signatures

Serious options:

1. Give every declaration a `GenericSignature`, including an empty one.
2. Return `None` from `genericSignature` for a non-generic declaration and `Some(signature)` for a generic declaration.

Decision: **Option 2.** Absence remains explicit, avoids allocating empty signatures for every declaration, and agrees with Phalcom's reflective `Option` conventions.

## 2. Terminology

### 2.1 Generic declaration

A class-shaped declaration or method declaration that owns one or more type parameters.

### 2.2 Type-parameter owner

A stable declaration descriptor that owns an ordered parameter list. First-version owners are `Class`, `Protocol`, `Method`, and `ProtocolRequirement` descriptors. A class-side method remains a `Method` descriptor and is not a separate owner kind.

### 2.3 Type-parameter specification

An immutable ownerless `TypeParameterSpec` used as validated construction input. A specification is not a type expression and has no declaration identity.

### 2.4 Type parameter

An immutable owned `TypeParameter` descriptor. It is a synthetic type expression and has identity `(owner identity, index)`.

### 2.5 Declaration index

The zero-based source-order position of a parameter inside its owner's parameter list.

### 2.6 Generic signature

The immutable descriptor pairing one owner with exactly its ordered type-parameter collection.

### 2.7 Declaration-site variance

A property of a class- or protocol-owned type parameter describing how applied forms may participate in subtyping. The values are invariant, covariant, and contravariant. This document stores the declaration; Document 08 validates positions and defines subtype consequences.

### 2.8 Upper bound

A type expression declared with `:`. An explicit type argument is admissible only when it is a subtype of the bound under the type relation defined by Documents 07 and 09.

### 2.9 Finite constraint set

A non-empty ordered set of distinct type expressions declared with `in (...)`. For explicit application, an argument must be equivalent to one member. Inference-specific promotion is deferred to Document 09.

### 2.10 Unrestricted parameter

A parameter whose `bound` is `None` and whose `constraints` collection is empty.

### 2.11 Enclosing parameter

A parameter owned by a lexically enclosing declaration, such as class parameter `T` referenced by method parameter bound `U: Container<T>`.

### 2.12 Same-signature recursive restriction

A bound or constraint expression whose `freeParameters` contains a parameter owned by the signature currently being declared. First-version declarations reject this form.

### 2.13 Parameter shell

A trusted mutable bootstrap-only allocation used while resolving a declaration header. It is never observable through user reflection. Completion assigns resolved restriction metadata and freezes the public descriptor.

### 2.14 Raw generic origin

A class or protocol descriptor that declares type parameters but has not been applied. Document 02 establishes that it is the closed declaration/type-constructor object, not an implicit open application.

## 3. User-facing syntax

### 3.1 Generic classes

```phalcom
class Box<T> {
  const _value: T

  @constructor
  new(value: T) {
    _value = value
  }

  value -> T {
    return _value
  }
}
```

`T` is invariant because no variance marker appears.

### 3.2 Declaration-site variance

```phalcom
class Producer<out T> {
  next -> Option<T> {
    ...
  }
}
```

```phalcom
class Consumer<in T> {
  accept(value: T) -> Unit {
    ...
  }
}
```

The parser and reflection preserve the variance markers from the first implementation. Document 08 determines whether all uses of `T` are legal.

### 3.3 Generic protocols

```phalcom
@protocol
class Repository<K, out V> {
  get(key: K) -> Option<V>
  contains(key: K) -> Bool
}
```

`Repository` is a `Protocol` descriptor whose `typeParameters` list contains two owned `TypeParameter` objects. Requirements refer to those exact objects in their resolved annotations.

### 3.4 Generic instance methods

```phalcom
class Sequence<T> {
  map<U>(transform: (T) -> U) -> Sequence<U> {
    ...
  }
}
```

`U` is owned by the `Method` descriptor for `map(_:)`, not by `Sequence`. The method annotation may refer to both the enclosing `T` and method-owned `U`.

### 3.5 Generic class-side methods

```phalcom
class Parser<T> {
  @class
  from<U: TextSource>(source: U) -> Parser<T> {
    ...
  }
}
```

The class-side method descriptor owns `U`. The method remains selected by its ordinary selector; `U` is not part of dispatch identity.

### 3.6 Method-owned parameters are invariant

```phalcom
class Transformer<T> {
  convert<U>(value: T) -> U {
    ...
  }
}
```

The following is illegal:

```phalcom
class Transformer<T> {
  convert<out U>(value: T) -> U {
    ...
  }
}
```

Diagnostic: `type.variance.method_parameter`.

### 3.7 Upper bounds

```phalcom
class Garage<T: Vehicle> {
  store(vehicle: T) -> Unit {
    ...
  }
}
```

The bound is reflected as `Some(Vehicle)`. It is not a superclass clause for `Garage`, does not affect layout, and does not insert runtime guards into `store(_:)`.

### 3.8 Finite constraint sets

```phalcom
class DatabaseId<T in (Int, String)> {
  const _value: T
}
```

The constraints are reflected in source order:

```phalcom
DatabaseId.typeParameters.first.constraints
// const [Int, String]
```

The set is semantically duplicate-free even though order is retained for reflection and diagnostics.

### 3.9 Bounds and constraints are mutually exclusive

No combined form is legal. The following conceptual declaration is rejected:

```phalcom
// Invalid conceptual combination; there is no legal grammar for it.
class Invalid<T: Object in (Int, String)> {}
```

A manually constructed `TypeParameterSpec` that supplies both is rejected with `type.parameter.bound_and_constraints`.

### 3.10 Defaults are reserved but unsupported

The following is not accepted in this version:

```phalcom
class Box<T = Object> {}
```

Diagnostic: `type.parameter.default_not_supported`.

Every reflected parameter nevertheless exposes:

```phalcom
parameter.default
// None
```

### 3.11 Multiple parameters and source order

```phalcom
class Mapping<K, V, out View> {}
```

Reflection preserves exactly `K`, `V`, `View` in that order. Reordering parameters is a declaration-shape change and changes owner/index identity for affected positions.

### 3.12 Lexical shadowing

```phalcom
class Outer<T> {
  identity<T>(value: T) -> T {
    return value
  }
}
```

Inside `identity`, unqualified `T` resolves to the method-owned parameter. `Outer.typeParameters.first` and `Outer.methodFor(#identity).typeParameters.first` are distinct even though both are named `T`.

### 3.13 References to enclosing parameters

```phalcom
class Outer<T> {
  wrap<U: Wrapper<T>>(value: U) -> Pair<T, U> {
    ...
  }
}
```

This representation is legal once `Wrapper<T>` is a valid type expression. The method-owned signature may contain free occurrences of enclosing `T` in its restriction metadata. The method-owned parameter `U` remains identified only by the method descriptor and its index.

### 3.14 Same-signature F-bounds are deferred

```phalcom
class Ordered<T: Comparable<T>> {}
```

The grammar preserves this form, but the first-version declaration validator rejects it with `type.parameter.recursive_restriction_deferred`. Document 09 may make guarded F-bounds legal without changing the AST or descriptor fields.

### 3.15 Manual specifications

Document 01's manual protocol construction uses public ownerless specifications:

```phalcom
const t = TypeParameterSpec.new(
  name: #T,
  variance: Variance.Invariant,
  bound: None,
  constraints: const [],
  default: None,
  sourceLocation: None
)
```

Convenience forms are also provided:

```phalcom
const key = TypeParameterSpec.invariant(name: #K)
const value = TypeParameterSpec.covariant(name: #V)
const vehicle = TypeParameterSpec.bounded(
  name: #T,
  variance: Variance.Invariant,
  by: Vehicle
)
const id = TypeParameterSpec.constrained(
  name: #T,
  variance: Variance.Invariant,
  to: const [Int, String]
)
```

A `TypeParameterSpec` does not become the owned parameter. `Protocol.new(...)` validates it and creates a fresh owned `TypeParameter` with the protocol or requirement as owner.

## 4. Semantic model

### 4.1 Declaration product

A generic declaration does not create a family of runtime classes. It creates:

1. the ordinary declaration descriptor (`Class`, `Protocol`, `Method`, or `ProtocolRequirement`);
2. one immutable `TypeParameter` object per source parameter;
3. one immutable `GenericSignature` for the owner;
4. resolved annotation metadata that refers to those parameter objects.

Instances remain instances of the ordinary origin class. Generic metadata is not retained on values unless some unrelated explicit library chooses to store it.

### 4.2 Parameter identity

For parameter `P`:

```text
identity(P) = (identity(P.owner), P.index)
```

The following do not determine identity:

- `name`;
- variance;
- bound;
- constraints;
- source location;
- display name;
- structural similarity of the owner;
- module-qualified spelling alone.

Within one completed owner, there is exactly one parameter object at each valid index. Repeated reflection returns the same object identity:

```phalcom
System.assert(Box.typeParameters.first === Box.typeParameters.first)
```

A module reload that creates a new class descriptor creates new parameter identities even when source text is unchanged.

### 4.3 Type-expression behavior

A `TypeParameter` is a synthetic `Type` expression:

- `displayName` is the source name as a string;
- `origin` is the parameter itself;
- `arguments` is empty;
- `typeParameters` is empty because a parameter does not declare parameters;
- `freeParameters` is exactly `const [self]`;
- `substitute(using:)` resolves the parameter by identity and returns itself when unbound;
- `equivalentTo(_:)` compares owner identity and index;
- `hash` is compatible with that equivalence.

Restriction metadata does not change the free-parameter result. `T.freeParameters` is `(T)`, not the union of `T` and parameters occurring in `T.bound`.

### 4.4 Variance semantics in this document

This document records variance but defines only these legality rules:

- unmarked class/protocol parameters are invariant;
- `out` maps to `Variance.Covariant`;
- `in` maps to `Variance.Contravariant`;
- method-owned parameters are always invariant;
- variance is immutable after declaration completion.

This document does not infer variance and does not decide whether a parameter is used in legal positions. Document 08 performs that analysis.

### 4.5 Upper-bound semantics

A parameter has at most one upper-bound expression. For explicit application, `parameter.validate(argument)` succeeds only when the argument is a subtype of the bound.

Because the complete subtype relation is established later, this document fixes the API and the eventual result, while the compiler/VM may defer invocation of the check until the type-relation bootstrap is ready. Document 04 may form an applied type only after validation is available.

The bound itself is metadata. Declaring `T: Vehicle` does not:

- change ordinary method dispatch;
- insert a runtime `is` check on values;
- create an inheritance edge between the generic owner and `Vehicle`;
- make `T` nominally equal to `Vehicle`;
- replace `T` in reflected member signatures.

### 4.6 Finite-constraint semantics

A finite constraint set is non-empty and duplicate-free by `Type.equivalentTo(_:)`.

For explicit application, validation succeeds when:

```text
exists C in parameter.constraints such that argument.equivalentTo(C)
```

Subtypes of a listed constraint do not automatically satisfy explicit application in this document. Document 09 defines whether inference may promote a discovered subtype to a listed constraint solution. That inference rule must not silently change explicit application semantics.

### 4.7 Bound/constraint exclusivity

The valid states are exactly:

| Bound | Constraints | Meaning |
|---|---|---|
| `None` | empty | unrestricted |
| `Some(type)` | empty | upper-bounded |
| `None` | non-empty | finitely constrained |
| `Some(type)` | non-empty | invalid |

The `default` field must be `None` in all first-version states.

### 4.8 Source-order invariants

For each owner:

- parameter indexes are contiguous from zero;
- collection order equals declaration order;
- `parameter.index` equals its collection position;
- no two parameters share the same source name within that owner;
- no two parameters share the same owner/index pair;
- `GenericSignature.parameters` is the same immutable collection exposed by `owner.typeParameters` or is an identity-preserving frozen view of it.

### 4.9 Lexical resolution

When resolving a type name inside a generic declaration:

1. consult the innermost method-owned type-parameter scope;
2. consult the immediately enclosing declaration's type-parameter scope;
3. continue through outer lexical declaration scopes;
4. consult ordinary local/module type bindings;
5. report the normal unresolved-type diagnostic.

A nested parameter may shadow an outer parameter. Resolution is by lexical scope, not by searching all owner names globally.

### 4.10 Recursive restrictions and cycles

The first-version validator computes `restriction.freeParameters` for every bound and constraint member after resolution.

It rejects the restriction when any free parameter has the same owner as the parameter being declared. This rejects:

```phalcom
class Direct<T: T> {}
class Mutual<T: U, U: T> {}
class Guarded<T: Comparable<T>> {}
```

The third form is intentionally reserved for Document 09 rather than declared meaningless. The diagnostic distinguishes deferred guarded recursion from malformed direct cycles when the implementation can classify it:

- `type.parameter.recursive_restriction` for direct or mutual unguarded cycles;
- `type.parameter.recursive_restriction_deferred` for a same-signature occurrence nested inside another type expression.

Restrictions may refer to parameters from enclosing owners because those cannot create a cycle back into the nested signature under lexical declaration ordering.

### 4.11 Generic signature semantics

A `GenericSignature`:

- belongs to exactly one owner;
- contains one or more parameters;
- preserves source order;
- validates owner and index consistency at construction;
- exposes `arity`, `isEmpty`, `parameterAt(index:)`, `validate(arguments:)`, and `environmentFor(arguments:)`;
- is immutable and identity-stable for the lifetime of the owner.

Although the public constructor is trusted, reflection may compare signatures by owner identity. There is no structural merging of unrelated signatures.

### 4.12 Argument validation

`GenericSignature.validate(arguments:)` performs, in order:

1. require the argument collection to be immutable or freeze a private snapshot;
2. require exact arity;
3. normalize every argument as a recognized `Type` expression;
4. call the corresponding parameter's `validate(_:)` in source order;
5. return normally on success;
6. throw the first stable diagnostic on failure.

Partial application and default filling are not supported. Document 04 consumes this exact rule.

### 4.13 Generic metadata and dispatch

The following selectors remain identical regardless of annotations:

```phalcom
box.put(value)
box.put(value: value)
```

Type parameter lists and method-owned generic parameters do not form overload keys. Two methods whose only difference is type annotations or type-parameter declarations collide under the ordinary selector duplicate rule.

### 4.14 Manual versus declaration construction

Decorator/compiler construction and manual construction produce behaviorally equivalent owned parameter descriptors when given equivalent specifications.

Differences:

- compiler construction has exact lexical source ranges and declaration identities;
- compiler construction can allocate parameter shells before annotation resolution;
- manual construction accepts already resolved type-expression objects;
- first-version manual construction cannot express same-signature recursive restrictions because no owned parameter exists before binding;
- manual construction never binds a declaration name automatically.

The recursive restriction difference does not reduce first-version expressiveness because source recursive restrictions are also rejected until Document 09.

## 5. Object model

### 5.1 `Variance`

`Variance` is an immutable sealed value family with three canonical variants:

- `Variance.Invariant`;
- `Variance.Covariant`;
- `Variance.Contravariant`.

The variants are stable singleton-like values suitable for identity comparison, equality, hashing, serialization, and reflection. They do not contain executable checking policy.

### 5.2 `TypeParameterSpec`

A specification stores:

- `name: Symbol`;
- `variance: Variance`;
- `bound: Option<Type>`;
- `constraints: const List<Type>`;
- `default: Option<Type>`;
- `sourceLocation: Option<SourceLocation>`.

It is ownerless. It may be freely reused to create parameters for different owners; each use creates distinct owned parameter identities.

### 5.3 `TypeParameterOwner`

`TypeParameterOwner` is a signature-only protocol:

```phalcom
@protocol
class TypeParameterOwner {
  typeParameters -> const List<TypeParameter>
  genericSignature -> Option<GenericSignature>
}
```

The protocol does not make arbitrary structural lookalikes trusted compiler owners. Trusted owner kinds are registered by bootstrap. Libraries may use the protocol reflectively, while compiler metadata requires a recognized declaration descriptor.

### 5.4 `TypeParameter`

`TypeParameter` is an immutable trusted subclass of `TypeDescriptor`. It stores:

- owner;
- index;
- name;
- variance;
- bound;
- constraints;
- default;
- source location.

Its public creation path is indirect. User code constructs `TypeParameterSpec`; trusted declaration binding constructs the owned object. A public unconstrained `TypeParameter.new(...)` would permit forged owner/index identities and is therefore not provided.

### 5.5 `GenericSignature`

`GenericSignature` is an immutable owned descriptor, but it is not a `Type` expression. It describes a declaration's parameter binder.

The owner strongly references its signature and parameters. The signature strongly references its owner and parameters. Parameters strongly reference the owner. These cycles are ordinary GC-managed object graphs.

### 5.6 Owner integration

The following descriptor kinds expose the owner surface:

| Owner kind | Parameters describe |
|---|---|
| `Class` | class declaration parameters |
| `Protocol` | protocol declaration parameters |
| `Method` | generic method parameters |
| `ProtocolRequirement` | generic requirement parameters |

For non-generic owners:

```phalcom
owner.typeParameters
// const []

owner.genericSignature
// None
```

For generic owners:

```phalcom
owner.genericSignature.unwrap.parameters === owner.typeParameters
// true, or an identity-preserving immutable view
```

### 5.7 Ownership and lifetime

A parameter is alive while either its owner or any external reference to the parameter is alive. Holding a parameter may keep its owner and signature alive. Implementations must not store only a weak owner reference because owner identity is part of parameter identity and reflection.

### 5.8 Equality, equivalence, and hashing

`TypeParameter.equivalentTo(other)` is true exactly when:

```text
other is TypeParameter
and self.owner === other.owner
and self.index == other.index
```

Its hash must be derived from owner identity and index. Ordinary `==` may delegate to the same rule for immutable descriptor usability, but `equivalentTo(_:)` remains the normative type-expression relation.

`GenericSignature` equality may use owner identity. Parameters from structurally identical but separately allocated declarations are not equivalent.

### 5.9 Immutability

After publication, none of the following may change:

- parameter owner;
- index;
- name;
- variance;
- bound;
- constraints;
- default;
- source location;
- signature owner;
- signature parameter order.

Class reopening may add ordinary methods under Phalcom's open object model, but it cannot add, remove, reorder, rename, or mutate type parameters. Doing so would invalidate normalized metadata and owner/index identity.

### 5.10 Trusted shells

Compiler-created recursive declarations use unobservable shells. A shell is not a user-visible partially initialized `TypeParameter`. The VM must prevent:

- reflection before completion;
- hashing before owner/index assignment;
- substitution before freeze;
- publication of an owner whose signature failed validation;
- reuse of a failed shell.

## 6. Complete standard-library source model

### 6.1 Status of forward references

The source below is normative for this document and references:

- `Type`, `TypeDescriptor`, `TypeRuntime`, and type-expression diagnostics from Document 02;
- `TypeEnvironment`, whose complete implementation is Document 05;
- the subtype relation whose complete semantics are Documents 07 and 09.

`TypeParameterRuntime.argumentSatisfiesBound` is a native/reference boundary until those relation documents are loaded. Its final behavior must match the formal rule in Section 4.5.

The owner integration blocks in Section 6.11 are normative augmentations to the complete core descriptors. They are not replacement declarations for those larger classes.

### 6.2 Variance

```phalcom
@data
@immutable
@sealed
class Variance {
  @variant Invariant
  @variant Covariant
  @variant Contravariant
}
```

### 6.3 Diagnostics and errors

```phalcom
@data
@immutable
class TypeParameterDiagnostic {
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

class TypeParameterError is Error {
  const _diagnostic: TypeParameterDiagnostic

  @constructor
  new(diagnostic: TypeParameterDiagnostic) {
    _diagnostic = diagnostic
  }

  diagnostic -> TypeParameterDiagnostic {
    return _diagnostic
  }

  message -> String {
    return _diagnostic.message
  }
}

class InvalidTypeParameterError is TypeParameterError {}
class TypeParameterOwnerError is TypeParameterError {}
class TypeParameterRestrictionError is TypeParameterError {}
class TypeBoundError is TypeParameterError {}
class TypeConstraintError is TypeParameterError {}
class GenericSignatureError is TypeParameterError {}
class TypeArgumentCountError is TypeParameterError {}
class TypeParameterAuthorityError is TypeParameterError {}
class TypeParameterMutationError is TypeParameterError {}
```

### 6.4 `TypeParameterSpec`

```phalcom
@data
@immutable
class TypeParameterSpec {
  const _name: Symbol
  const _variance: Variance
  const _bound: Option<Type>
  const _constraints: const List<Type>
  const _default: Option<Type>
  const _sourceLocation: Option<SourceLocation>

  @constructor
  new(
    name: Symbol,
    variance: Variance,
    bound: Option<Type>,
    constraints: const List<Type>,
    default: Option<Type>,
    sourceLocation: Option<SourceLocation>
  ) {
    const frozenConstraints = constraints.freeze

    TypeParameterRuntime.validateSpecification(
      name: name,
      variance: variance,
      bound: bound,
      constraints: frozenConstraints,
      default: default,
      sourceLocation: sourceLocation
    )

    _name = name
    _variance = variance
    _bound = TypeParameterRuntime.normalizeOptional(bound)
    _constraints = TypeParameterRuntime.normalizeTypes(frozenConstraints)
    _default = default
    _sourceLocation = sourceLocation
  }

  @class
  invariant(name: Symbol) -> TypeParameterSpec {
    return TypeParameterSpec.new(
      name: name,
      variance: Variance.Invariant,
      bound: None,
      constraints: const [],
      default: None,
      sourceLocation: None
    )
  }

  @class
  covariant(name: Symbol) -> TypeParameterSpec {
    return TypeParameterSpec.new(
      name: name,
      variance: Variance.Covariant,
      bound: None,
      constraints: const [],
      default: None,
      sourceLocation: None
    )
  }

  @class
  contravariant(name: Symbol) -> TypeParameterSpec {
    return TypeParameterSpec.new(
      name: name,
      variance: Variance.Contravariant,
      bound: None,
      constraints: const [],
      default: None,
      sourceLocation: None
    )
  }

  @class
  bounded(
    name: Symbol,
    variance: Variance,
    by: Type
  ) -> TypeParameterSpec {
    return TypeParameterSpec.new(
      name: name,
      variance: variance,
      bound: Some.new(by),
      constraints: const [],
      default: None,
      sourceLocation: None
    )
  }

  @class
  constrained(
    name: Symbol,
    variance: Variance,
    to: const List<Type>
  ) -> TypeParameterSpec {
    if to.isEmpty {
      throw TypeParameterRuntime.invalid(
        code: "type.parameter.empty_constraints",
        message: "a constrained type parameter requires at least one alternative",
        details: TypeParameterRuntime.details(#name, name),
        sourceLocation: None
      )
    }

    return TypeParameterSpec.new(
      name: name,
      variance: variance,
      bound: None,
      constraints: to,
      default: None,
      sourceLocation: None
    )
  }
}
```

### 6.5 `TypeParameterOwner`

```phalcom
@protocol
class TypeParameterOwner {
  typeParameters -> const List<TypeParameter>
  genericSignature -> Option<GenericSignature>
}
```

### 6.6 Trusted construction token

```phalcom
@immutable
class TypeParameterConstructionToken {
  @constructor
  @native
  _trustedNew() {}
}

const _TYPE_PARAMETER_CONSTRUCTION_TOKEN =
  TypeParameterConstructionToken._trustedNew()
```

The token is an unforgeable capability. A VM may represent it without allocating an ordinary user-visible object.

### 6.7 `TypeParameter`

```phalcom
@immutable
class TypeParameter is TypeDescriptor {
  const _owner: TypeParameterOwner
  const _index: Int
  const _name: Symbol
  const _variance: Variance
  const _bound: Option<Type>
  const _constraints: const List<Type>
  const _default: Option<Type>
  const _sourceLocation: Option<SourceLocation>

  @constructor
  @native
  _ownedNew(
    token: TypeParameterConstructionToken,
    owner: TypeParameterOwner,
    index: Int,
    specification: TypeParameterSpec
  ) {
    TypeParameterRuntime.requireConstructionAuthority(token)

    if index < 0 {
      throw TypeParameterRuntime.invalid(
        code: "type.parameter.invalid_index",
        message: "type parameter index must be non-negative",
        details: TypeParameterRuntime.details(#index, index),
        sourceLocation: specification.sourceLocation
      )
    }

    _owner = owner
    _index = index
    _name = specification.name
    _variance = specification.variance
    _bound = specification.bound
    _constraints = specification.constraints
    _default = specification.default
    _sourceLocation = specification.sourceLocation
  }

  owner -> TypeParameterOwner {
    return _owner
  }

  index -> Int {
    return _index
  }

  name -> Symbol {
    return _name
  }

  variance -> Variance {
    return _variance
  }

  bound -> Option<Type> {
    return _bound
  }

  constraints -> const List<Type> {
    return _constraints
  }

  default -> Option<Type> {
    return _default
  }

  sourceLocation -> Option<SourceLocation> {
    return _sourceLocation
  }

  displayName -> String {
    return _name.toString
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
    return const [self]
  }

  substitute(using: TypeEnvironment) -> Type {
    return using.resolve(self).orElse { self }
  }

  equivalentTo(other: Type) -> Bool {
    if other.is(TypeParameter).not {
      return false
    }

    return _owner === other.owner and _index == other.index
  }

  hash -> Int {
    return (_owner.identityHash * 31) + _index.hash
  }

  isBounded -> Bool {
    return _bound != None
  }

  isConstrained -> Bool {
    return _constraints.isEmpty.not
  }

  isUnrestricted -> Bool {
    return _bound == None and _constraints.isEmpty
  }

  validate(argument: Type) -> None {
    TypeParameterRuntime.validateArgument(
      parameter: self,
      argument: argument
    )
  }

  toString -> String {
    return _name.toString
  }
}
```

The native constructor must also establish the trusted `TypeDescriptor` base authority required by Document 02. An implementation may do so in VM allocation rather than through an observable superclass constructor send.

### 6.8 `GenericSignature`

```phalcom
@immutable
class GenericSignature {
  const _owner: TypeParameterOwner
  const _parameters: const List<TypeParameter>

  @constructor
  @native
  _ownedNew(
    token: TypeParameterConstructionToken,
    owner: TypeParameterOwner,
    parameters: const List<TypeParameter>
  ) {
    TypeParameterRuntime.requireConstructionAuthority(token)

    const frozenParameters = parameters.freeze
    TypeParameterRuntime.validateOwnedParameters(
      owner: owner,
      parameters: frozenParameters
    )

    _owner = owner
    _parameters = frozenParameters
  }

  owner -> TypeParameterOwner {
    return _owner
  }

  parameters -> const List<TypeParameter> {
    return _parameters
  }

  arity -> Int {
    return _parameters.size
  }

  isEmpty -> Bool {
    return _parameters.isEmpty
  }

  parameterAt(index: Int) -> Option<TypeParameter> {
    if index < 0 or index >= _parameters.size {
      return None
    }

    return Some.new(_parameters.at(index))
  }

  validate(arguments: const List<Type>) -> None {
    const normalized = TypeParameterRuntime.normalizeArguments(arguments)

    if normalized.size != _parameters.size {
      throw TypeParameterRuntime.argumentCount(
        owner: _owner,
        expected: _parameters.size,
        received: normalized.size
      )
    }

    let index = 0
    while index < _parameters.size {
      _parameters.at(index).validate(normalized.at(index))
      index++
    }
  }

  environmentFor(arguments: const List<Type>) -> TypeEnvironment {
    const normalized = TypeParameterRuntime.normalizeArguments(arguments)
    self.validate(arguments: normalized)

    return TypeEnvironment.from(
      signature: self,
      arguments: normalized
    )
  }

  equivalentTo(other: Object) -> Bool {
    if other.is(GenericSignature).not {
      return false
    }

    return _owner === other.owner
  }

  hash -> Int {
    return _owner.identityHash
  }

  toString -> String {
    const names = _parameters.map { parameter =>
      parameter.displayName
    }

    return "<\(names.join(\", \"))>"
  }
}
```

A completed public generic signature always has at least one parameter. `isEmpty` remains part of the source model because trusted bootstrap and future synthetic owners may use an empty internal signature while constructing metadata; ordinary non-generic declarations expose `None` instead.

### 6.9 `TypeParameterRuntime`

```phalcom
class TypeParameterRuntime {
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
  requireConstructionAuthority(
    token: TypeParameterConstructionToken
  ) -> None {
    if token !== _TYPE_PARAMETER_CONSTRUCTION_TOKEN {
      throw TypeParameterAuthorityError.new(
        TypeParameterDiagnostic.new(
          code: "type.parameter.construction_authority",
          message: "type parameter construction requires trusted authority",
          details: self.details(),
          sourceLocation: None
        )
      )
    }
  }

  @class
  invalid(
    code: String,
    message: String,
    details: const Map<Symbol, Object>,
    sourceLocation: Option<SourceLocation>
  ) -> InvalidTypeParameterError {
    return InvalidTypeParameterError.new(
      TypeParameterDiagnostic.new(
        code: code,
        message: message,
        details: details,
        sourceLocation: sourceLocation
      )
    )
  }

  @class
  normalizeOptional(type: Option<Type>) -> Option<Type> {
    if type == None {
      return None
    }

    return Some.new(TypeRuntime.normalize(type.unwrap))
  }

  @class
  normalizeTypes(types: const List<Type>) -> const List<Type> {
    return types.map { type =>
      TypeRuntime.normalize(type)
    }.freeze
  }

  @class
  validateSpecification(
    name: Symbol,
    variance: Variance,
    bound: Option<Type>,
    constraints: const List<Type>,
    default: Option<Type>,
    sourceLocation: Option<SourceLocation>
  ) -> None {
    if name.toString.isEmpty {
      throw self.invalid(
        code: "type.parameter.invalid_name",
        message: "type parameter name must not be empty",
        details: self.details(#name, name),
        sourceLocation: sourceLocation
      )
    }

    if variance.is(Variance).not {
      throw self.invalid(
        code: "type.parameter.invalid_variance",
        message: "type parameter variance must be a Variance value",
        details: self.details(#variance, variance),
        sourceLocation: sourceLocation
      )
    }

    if bound != None and constraints.isEmpty.not {
      throw TypeParameterRestrictionError.new(
        TypeParameterDiagnostic.new(
          code: "type.parameter.bound_and_constraints",
          message: "a type parameter cannot declare both a bound and constraints",
          details: self.details(
            #name, name,
            #bound, bound,
            #constraints, constraints
          ),
          sourceLocation: sourceLocation
        )
      )
    }

    if default != None {
      throw TypeParameterRestrictionError.new(
        TypeParameterDiagnostic.new(
          code: "type.parameter.default_not_supported",
          message: "default type arguments are not supported",
          details: self.details(#name, name, #default, default),
          sourceLocation: sourceLocation
        )
      )
    }

    if bound != None {
      TypeRuntime.normalize(bound.unwrap)
    }

    const normalizedConstraints = self.normalizeTypes(constraints)
    let index = 0
    while index < normalizedConstraints.size {
      const constraint = normalizedConstraints.at(index)
      let previous = 0

      while previous < index {
        if constraint.equivalentTo(normalizedConstraints.at(previous)) {
          throw TypeParameterRestrictionError.new(
            TypeParameterDiagnostic.new(
              code: "type.parameter.duplicate_constraint",
              message: "type parameter constraints must be unique",
              details: self.details(
                #name, name,
                #constraint, constraint,
                #firstIndex, previous,
                #duplicateIndex, index
              ),
              sourceLocation: sourceLocation
            )
          )
        }
        previous++
      }

      index++
    }
  }

  @class
  validateSpecifications(
    specifications: const List<TypeParameterSpec>,
    sourceLocation: Option<SourceLocation>
  ) -> None {
    self.validateSpecificationList(
      specifications: specifications,
      ownerKind: #unspecified,
      sourceLocation: sourceLocation
    )
  }

  @class
  validateSpecifications(
    specifications: const List<TypeParameterSpec>,
    ownerKind: Symbol
  ) -> None {
    self.validateSpecificationList(
      specifications: specifications,
      ownerKind: ownerKind,
      sourceLocation: None
    )
  }

  @class
  validateSpecificationList(
    specifications: const List<TypeParameterSpec>,
    ownerKind: Symbol,
    sourceLocation: Option<SourceLocation>
  ) -> None {
    let index = 0

    while index < specifications.size {
      const specification = specifications.at(index)

      const methodOwned =
        ownerKind == #method or ownerKind == #protocolRequirement

      if methodOwned and specification.variance != Variance.Invariant {
        throw TypeParameterRestrictionError.new(
          TypeParameterDiagnostic.new(
            code: "type.variance.method_parameter",
            message: "method-owned type parameters must be invariant",
            details: self.details(
              #name, specification.name,
              #variance, specification.variance,
              #ownerKind, ownerKind
            ),
            sourceLocation: specification.sourceLocation.orElse {
              sourceLocation
            }
          )
        )
      }

      let previous = 0
      while previous < index {
        if specifications.at(previous).name == specification.name {
          throw self.invalid(
            code: "type.parameter.duplicate_name",
            message: "type parameter names must be unique within one declaration",
            details: self.details(
              #name, specification.name,
              #firstIndex, previous,
              #duplicateIndex, index
            ),
            sourceLocation: specification.sourceLocation.orElse {
              sourceLocation
            }
          )
        }
        previous++
      }

      index++
    }
  }

  @class
  @native
  ownerKind(of: TypeParameterOwner) -> Symbol {
    if of.is(Class) {
      return #class
    }

    if of.is(Protocol) {
      return #protocol
    }

    if of.is(Method) {
      return #method
    }

    if of.is(ProtocolRequirement) {
      return #protocolRequirement
    }

    throw TypeParameterOwnerError.new(
      TypeParameterDiagnostic.new(
        code: "type.parameter.invalid_owner",
        message: "type parameter owner is not a trusted declaration descriptor",
        details: self.details(#owner, of),
        sourceLocation: None
      )
    )
  }

  @class
  bindOwned(
    owner: TypeParameterOwner,
    specifications: const List<TypeParameterSpec>
  ) -> const List<TypeParameter> {
    return self.bindOwned(
      owner: owner,
      ownerKind: self.ownerKind(of: owner),
      specifications: specifications
    )
  }

  @class
  @native
  bindOwned(
    owner: TypeParameterOwner,
    ownerKind: Symbol,
    specifications: const List<TypeParameterSpec>
  ) -> const List<TypeParameter> {
    self.validateSpecifications(
      specifications: specifications,
      ownerKind: ownerKind
    )

    let parameters = List.new()
    let index = 0

    while index < specifications.size {
      parameters.add(
        TypeParameter._ownedNew(
          token: _TYPE_PARAMETER_CONSTRUCTION_TOKEN,
          owner: owner,
          index: index,
          specification: specifications.at(index)
        )
      )
      index++
    }

    const frozen = parameters.freeze
    self.validateRestrictions(
      owner: owner,
      parameters: frozen
    )

    const signature = self.signatureFor(
      owner: owner,
      parameters: frozen
    )

    GenericDeclarationRuntime.attach(
      owner: owner,
      parameters: frozen,
      signature: signature
    )

    return frozen
  }

  @class
  @native
  signatureFor(
    owner: TypeParameterOwner,
    parameters: const List<TypeParameter>
  ) -> Option<GenericSignature> {
    if parameters.isEmpty {
      return None
    }

    return Some.new(
      GenericSignature._ownedNew(
        token: _TYPE_PARAMETER_CONSTRUCTION_TOKEN,
        owner: owner,
        parameters: parameters
      )
    )
  }

  @class
  validateOwnedParameters(
    owner: TypeParameterOwner,
    parameters: const List<TypeParameter>
  ) -> None {
    let index = 0

    while index < parameters.size {
      const parameter = parameters.at(index)

      if parameter.owner !== owner {
        throw TypeParameterOwnerError.new(
          TypeParameterDiagnostic.new(
            code: "type.signature.foreign_parameter",
            message: "generic signature contains a parameter owned by another declaration",
            details: self.details(
              #owner, owner,
              #parameter, parameter,
              #parameterOwner, parameter.owner,
              #index, index
            ),
            sourceLocation: parameter.sourceLocation
          )
        )
      }

      let previous = 0
      while previous < index {
        const prior = parameters.at(previous)

        if prior === parameter {
          throw TypeParameterOwnerError.new(
            TypeParameterDiagnostic.new(
              code: "type.signature.duplicate_parameter",
              message: "generic signature contains the same parameter more than once",
              details: self.details(
                #owner, owner,
                #parameter, parameter,
                #firstIndex, previous,
                #duplicateIndex, index
              ),
              sourceLocation: parameter.sourceLocation
            )
          )
        }

        if prior.name == parameter.name {
          throw InvalidTypeParameterError.new(
            TypeParameterDiagnostic.new(
              code: "type.parameter.duplicate_name",
              message: "type parameter names must be unique within one declaration",
              details: self.details(
                #owner, owner,
                #name, parameter.name,
                #firstIndex, previous,
                #duplicateIndex, index
              ),
              sourceLocation: parameter.sourceLocation
            )
          )
        }

        previous++
      }

      if parameter.index != index {
        throw TypeParameterOwnerError.new(
          TypeParameterDiagnostic.new(
            code: "type.signature.index_mismatch",
            message: "type parameter index must match declaration order",
            details: self.details(
              #owner, owner,
              #parameter, parameter,
              #expectedIndex, index,
              #actualIndex, parameter.index
            ),
            sourceLocation: parameter.sourceLocation
          )
        )
      }

      index++
    }
  }

  @class
  validateRestrictions(
    owner: TypeParameterOwner,
    parameters: const List<TypeParameter>
  ) -> None {
    parameters.each { parameter =>
      if parameter.bound != None {
        self.validateRestriction(
          owner: owner,
          parameter: parameter,
          restriction: parameter.bound.unwrap
        )
      }

      parameter.constraints.each { constraint =>
        self.validateRestriction(
          owner: owner,
          parameter: parameter,
          restriction: constraint
        )
      }
    }
  }

  @class
  validateRestriction(
    owner: TypeParameterOwner,
    parameter: TypeParameter,
    restriction: Type
  ) -> None {
    const sameOwner = restriction.freeParameters.filter { free =>
      free.owner === owner
    }

    if sameOwner.isEmpty {
      return
    }

    if restriction.is(TypeParameter) {
      throw TypeParameterRestrictionError.new(
        TypeParameterDiagnostic.new(
          code: "type.parameter.recursive_restriction",
          message: "same-signature direct type-parameter restrictions are not supported in this version",
          details: self.details(
            #owner, owner,
            #parameter, parameter,
            #restriction, restriction,
            #recursiveParameters, sameOwner
          ),
          sourceLocation: parameter.sourceLocation
        )
      )
    }

    throw TypeParameterRestrictionError.new(
      TypeParameterDiagnostic.new(
        code: "type.parameter.recursive_restriction_deferred",
        message: "guarded same-signature type-parameter restrictions are deferred to Document 09",
        details: self.details(
          #owner, owner,
          #parameter, parameter,
          #restriction, restriction,
          #recursiveParameters, sameOwner
        ),
        sourceLocation: parameter.sourceLocation
      )
    )
  }

  @class
  normalizeArguments(arguments: const List<Type>) -> const List<Type> {
    return arguments.map { argument =>
      TypeRuntime.normalize(argument)
    }.freeze
  }

  @class
  validateArgument(
    parameter: TypeParameter,
    argument: Type
  ) -> None {
    const normalized = TypeRuntime.normalize(argument)

    if parameter.bound != None and
      self.argumentSatisfiesBound(
        normalized,
        bound: parameter.bound.unwrap
      ).not {
      throw TypeBoundError.new(
        TypeParameterDiagnostic.new(
          code: "type.argument.bound_violation",
          message: "type argument does not satisfy the parameter bound",
          details: self.details(
            #parameter, parameter,
            #argument, normalized,
            #bound, parameter.bound.unwrap
          ),
          sourceLocation: parameter.sourceLocation
        )
      )
    }

    if parameter.constraints.isEmpty.not {
      const accepted = parameter.constraints.any { constraint =>
        normalized.equivalentTo(constraint)
      }

      if accepted.not {
        throw TypeConstraintError.new(
          TypeParameterDiagnostic.new(
            code: "type.argument.constraint_violation",
            message: "type argument is not a member of the finite constraint set",
            details: self.details(
              #parameter, parameter,
              #argument, normalized,
              #constraints, parameter.constraints
            ),
            sourceLocation: parameter.sourceLocation
          )
        )
      }
    }
  }

  @class
  @native
  argumentSatisfiesBound(
    argument: Type,
    bound: Type
  ) -> Bool {
    // Reference result: TypeRelations.isSubtype(argument, of: bound).
    // Documents 07 and 09 define the complete relation and recursive guards.
    return argument.equivalentTo(bound)
  }

  @class
  argumentCount(
    owner: TypeParameterOwner,
    expected: Int,
    received: Int
  ) -> TypeArgumentCountError {
    return TypeArgumentCountError.new(
      TypeParameterDiagnostic.new(
        code: "type.application.argument_count",
        message: "generic declaration received the wrong number of type arguments",
        details: self.details(
          #owner, owner,
          #expected, expected,
          #received, received
        ),
        sourceLocation: None
      )
    )
  }
}
```

The visible fallback in `argumentSatisfiesBound` is a bootstrap floor, not the final subtype relation. A conforming full implementation must replace it once Document 07 is loaded and must produce the same result as `TypeRelations.isSubtype(argument, of: bound)`.

### 6.10 Compiler-facing declaration binder

```phalcom
class GenericDeclarationRuntime {
  @class
  @native
  bindSpecifications(
    owner: TypeParameterOwner,
    ownerKind: Symbol,
    specifications: const List<TypeParameterSpec>
  ) -> None {
    TypeParameterRuntime.bindOwned(
      owner: owner,
      ownerKind: ownerKind,
      specifications: specifications
    )
  }

  @class
  @native
  attach(
    owner: TypeParameterOwner,
    parameters: const List<TypeParameter>,
    signature: Option<GenericSignature>
  ) -> None {
    // Trusted one-time attachment before owner publication. Repeated
    // attachment, mutation after publication, or mismatched owner metadata
    // raises type.parameter.mutation or malformed-metadata diagnostics.
  }

  @class
  @native
  typeParameters(of: TypeParameterOwner) -> const List<TypeParameter> {
    return const []
  }

  @class
  @native
  genericSignature(of: TypeParameterOwner) -> Option<GenericSignature> {
    return None
  }
}
```

### 6.11 Normative owner augmentations

`Class` and `Method` receive both owner accessors through trusted augmentation:

```phalcom
// Normative augmentation of Class and Method.
typeParameters -> const List<TypeParameter> {
  return GenericDeclarationRuntime.typeParameters(of: self)
}

genericSignature -> Option<GenericSignature> {
  return GenericDeclarationRuntime.genericSignature(of: self)
}
```

`Protocol` and `ProtocolRequirement` retain the concrete `typeParameters` accessors established by Document 01 and receive only the signature accessor:

```phalcom
// Normative augmentation of Protocol and ProtocolRequirement.
genericSignature -> Option<GenericSignature> {
  return GenericDeclarationRuntime.genericSignature(of: self)
}
```

`TypeParameterRuntime.bindOwned(owner:, specifications:)` is the compatible completion of Document 01's forward-referenced selector. It infers the trusted owner kind, performs method-variance validation for requirements, and registers the canonical signature without changing Document 01's public protocol APIs.

### 6.12 Public manual-construction example

```phalcom
const repository = Protocol.new(
  name: #Repository,
  module: currentModule,
  typeParameters: const [
    TypeParameterSpec.invariant(name: #K),
    TypeParameterSpec.covariant(name: #V)
  ],
  requirements: const [
    ProtocolRequirementDraft.instanceMethod(
      selector: Selector.method(#get, labels: const [#key]),
      parameters: const [
        ProtocolParameterDraft.labeled(
          name: #key,
          label: #key,
          type: None
        )
      ],
      resultType: None
    )
  ],
  attributes: const [],
  documentation: None,
  sourceLocation: None
)

System.assert(repository.typeParameters.size == 2)
System.assert(repository.typeParameters.at(0).owner === repository)
System.assert(repository.typeParameters.at(1).variance == Variance.Covariant)
```

The parameter and result annotations in a manually constructed recursive generic protocol require already resolved type-expression objects. Document 05 supplies environments and Document 09 may add a public recursive specification facility. This limitation is explicit and does not affect ordinary compiler declarations.

## 7. Compiler and AST requirements

### 7.1 Phase separation

A conforming implementation distinguishes:

1. **Parsing:** preserve generic header syntax and source ranges without resolving names.
2. **Declaration indexing:** allocate owner identity and bind the declaration name.
3. **Parameter-shell creation:** allocate one hidden shell per source parameter and bind names in the generic scope.
4. **Restriction resolution:** resolve bounds, constraints, and defaults against lexical type scopes.
5. **Header validation:** validate duplicate names, owner-kind variance rules, exclusivity, defaults, and recursive restrictions.
6. **Parameter completion:** complete and freeze owned `TypeParameter` objects.
7. **Signature creation:** construct and attach the unique `GenericSignature`.
8. **Member annotation resolution:** resolve fields, parameters, and result annotations using completed parameter identities.
9. **Variance and relation checking:** later documents consume the metadata; this document does not perform position checking.
10. **Metadata emission:** serialize stable owner references and indexes.
11. **Bootstrap/module publication:** expose only completed immutable owners and parameters.

Implementations may fuse internal operations for performance but must preserve these observable dependencies and failure boundaries.

### 7.2 Grammar

The generic parameter list follows the declared name and precedes superclass clauses or method parameter lists.

Normative grammar sketch:

```text
generic-parameter-list
  ::= "<" generic-parameter ("," generic-parameter)* ">"

generic-parameter
  ::= variance-marker? identifier generic-restriction? generic-default?

variance-marker
  ::= "in" | "out"

generic-restriction
  ::= ":" type-expression
   |  "in" "(" type-expression ("," type-expression)* ")"

generic-default
  ::= "=" type-expression
```

The parser recognizes `generic-default` solely to produce the stable `type.parameter.default_not_supported` diagnostic. It must not accept the declaration silently.

A finite constraint set requires at least one source alternative. A one-element set remains semantically distinct from an upper bound: it accepts only an equivalent explicit argument, while a bound accepts subtypes.

`in` is contextual:

- before a parameter name, it means contravariance;
- after a parameter name, it introduces a finite constraint set;
- elsewhere it retains its ordinary language meaning.

Examples:

```phalcom
class Consumer<in T> {}
class Identifier<T in (Int, String)> {}
```

The parser distinguishes the forms by position.

### 7.3 Declaration contexts

Generic parameter lists are accepted on:

- ordinary class declarations;
- `@protocol` class-shaped declarations;
- instance method declarations;
- class-side method declarations;
- bodyless protocol requirement declarations.

They are not accepted on:

- fields;
- local bindings;
- parameters;
- constructors as a separate declaration category beyond their method descriptor;
- module declarations;
- attributes;
- individual `@variant` payload fields in this document.

Constructor methods may be generic only if ordinary methods with `@constructor` are permitted to own type parameters by the constructor specification. If permitted, the parameters belong to the constructor's `Method` descriptor and remain non-dispatching.

### 7.4 AST shape

A conforming AST may use different internal names but must preserve at least:

```text
TypeParameterAst {
  name: Symbol
  variance: Absent | In | Out
  restriction: Unrestricted
             | Bound(TypeExpressionAst)
             | Constraints([TypeExpressionAst])
  default: Option<TypeExpressionAst>
  sourceRange: SourceRange
  nameRange: SourceRange
  varianceRange: Option<SourceRange>
  restrictionRange: Option<SourceRange>
  defaultRange: Option<SourceRange>
}

ClassDeclarationAst {
  name: Symbol
  typeParameters: [TypeParameterAst]
  ...
}

MethodDeclarationAst {
  selector: SelectorAst
  typeParameters: [TypeParameterAst]
  ...
}
```

The AST must not collapse a bound and constraints into one untagged list. It must retain exact source order and range information for every alternative.

### 7.5 Header indexing and shells

For a declaration:

```phalcom
class Pair<A, B> {}
```

required ordering is:

```text
allocate Pair declaration shell
→ reserve two parameter indexes
→ allocate hidden A and B shells with owner Pair
→ bind A and B in the generic header scope
→ resolve restrictions
→ validate and freeze A and B
→ create Pair GenericSignature
→ resolve member annotations
→ publish Pair
```

The shell mechanism permits all member annotations to refer to canonical parameter identities. User code cannot observe the shell phase.

### 7.6 Duplicate names

Duplicate names are rejected within one owner:

```phalcom
class Pair<T, T> {}
```

Diagnostic: `type.parameter.duplicate_name` on the second name, with fields:

```text
owner
name
firstIndex
firstRange
duplicateIndex
duplicateRange
```

The duplicate does not create a second shell.

### 7.7 Shadowing

A nested method may shadow an outer parameter. The AST retains no special aliasing. The resolver creates a new owner/index identity and selects it by nearest lexical scope.

Documentation and LSP rendering should qualify ambiguous reflected parameters as needed, for example:

```text
Outer.T
Outer#identity.T
```

The display name of each parameter remains only `T`.

### 7.8 Restriction resolution

Present bound and constraint expressions are resolved through the ordinary type-expression resolver after parameter shells are in scope. The resolver must preserve absent restrictions as `None`/empty rather than rewriting them to `Object`, `Any`, or `Dynamic`.

Every resolved restriction is normalized through `TypeRuntime.normalize` before attachment.

### 7.9 Variance lowering

Compiler lowering maps:

| Source | Reflected value |
|---|---|
| no marker | `Variance.Invariant` |
| `out` | `Variance.Covariant` |
| `in` before name | `Variance.Contravariant` |

A method-owned non-invariant marker is diagnosed during header validation. The compiler must not silently erase it to invariant.

### 7.10 Restriction lowering

Compiler lowering produces one `TypeParameterSpec` per parameter in source order. Conceptually:

```phalcom
class Garage<T: Vehicle> {}
```

produces:

```phalcom
TypeParameterSpec.new(
  name: #T,
  variance: Variance.Invariant,
  bound: Some.new(Vehicle),
  constraints: const [],
  default: None,
  sourceLocation: locationOfT
)
```

The compiler may use a trusted unresolved record internally before type-expression resolution. User reflection observes only resolved `TypeParameter` objects.

### 7.11 Member resolution

After signature completion, member annotations resolve to exact parameter objects:

```phalcom
class Box<T> {
  value -> T { ... }
}
```

The reflected return type must be the same object as `Box.typeParameters.first`:

```phalcom
System.assert(
  Box.methodFor(#value).returnType.unwrap ===
    Box.typeParameters.first
)
```

String equality of names is insufficient.

### 7.12 Duplicate methods and selector identity

Generic parameter lists do not distinguish method selectors:

```phalcom
class Invalid {
  convert<T>(value: T) -> T { ... }
  convert<U>(value: U) -> U { ... }
}
```

Both declarations have the same ordinary selector and are duplicates. The existing duplicate-selector diagnostic applies. The compiler must not create type-directed overloads.

### 7.13 Recursive restriction classification

The validator must inspect free parameters after restriction resolution:

- a restriction that is directly a same-owner parameter is an unguarded recursive restriction;
- a restriction containing a same-owner parameter beneath another type-expression node is a deferred guarded recursive restriction;
- a restriction containing only enclosing-owner parameters is legal;
- closed restrictions are legal.

This classification exists so Document 09 can admit guarded F-bounds without changing source metadata.

### 7.14 Metadata emission

At minimum, emitted records preserve:

```text
GenericSignatureRecord {
  ownerReference
  parameterCount
  parameters[]
}

TypeParameterRecord {
  ownerReference
  index
  name
  variance
  restrictionKind: unrestricted | bound | constraints
  boundReference: optional
  constraintReferences[]
  defaultReference: optional, must be absent in version 1
  sourceLocation: optional
}
```

Owner references and parameter references must be relocatable across module loading. Name alone cannot be used to reconstruct identity.

Document 17 defines final encoding, versioning, aliases, and lazy/eager resolution.

## 8. Interpreter and VM requirements

### 8.1 Runtime responsibilities

The interpreter/VM must provide or preserve:

- trusted owner recognition;
- hidden parameter-shell allocation during declaration loading;
- one-time owner/index assignment;
- one-time restriction completion;
- immutable parameter publication;
- stable `GenericSignature` attachment;
- trusted descriptor registration under Document 02;
- exact reflected object identity for annotation references;
- argument-validation hooks;
- malformed-metadata rejection;
- GC tracing through owner/signature/parameter cycles.

### 8.2 Trusted owner kinds

The first-version trusted owners are:

- `Class`;
- `Protocol`;
- `Method`;
- `ProtocolRequirement`.

An arbitrary object that structurally satisfies `TypeParameterOwner` may be consumed by ordinary libraries but cannot be used as authoritative compiler metadata unless a later trusted extension mechanism registers its descriptor kind.

### 8.3 Construction authority

Only trusted declaration loading and validated manual APIs may call `TypeParameter._ownedNew` and `GenericSignature._ownedNew`.

The VM must reject:

- a forged token;
- a negative index;
- a foreign owner;
- duplicate indexes;
- repeated attachment;
- attachment after owner publication;
- mutation after freeze;
- a parameter object whose trusted `TypeDescriptor` base is incomplete.

### 8.4 Bootstrap sequence

Minimum bootstrap ordering:

```text
trusted Protocol and Type shells
→ trusted TypeDescriptor shell
→ Variance values
→ TypeParameterSpec
→ TypeParameter and GenericSignature shells
→ TypeParameterOwner protocol
→ owner-side metadata attachment slots
→ TypeParameterRuntime native anchors
→ canonical Type protocol completion
→ generic core declaration indexing
→ parameter binding and annotation resolution
→ descriptor freezing and publication
```

The `Type` protocol in Document 02 references `TypeParameter` before the latter is fully loaded. Bootstrap may use trusted unresolved references internally, but user reflection must not observe them.

### 8.5 Two-phase completion

For compiler declarations, the VM may allocate parameter shells whose restrictions are temporarily unresolved. These shells must not be reachable from ordinary module bindings or reflection until completion.

If completion fails:

- the owner declaration fails atomically;
- no signature is published;
- no parameter shell remains reachable from user code;
- metadata caches are cleared;
- a later module reload creates fresh owner and parameter identities.

### 8.6 Manual construction

Manual `Protocol.new(...)` and future manual owner APIs call `TypeParameterRuntime.bindOwned` with already resolved specifications.

Manual binding is eager and atomic:

```text
validate all specifications
→ allocate all parameters
→ validate owner/index order
→ validate restrictions
→ create signature
→ attach once
→ return completed owner
```

No partially completed owner is returned on failure.

### 8.7 Argument validation timing

Forming a generic declaration does not validate values. `GenericSignature.validate(arguments:)` validates type-expression arguments only when explicitly requested, primarily by Document 04's application primitive.

The VM must not consult generic signatures during ordinary instance method dispatch.

### 8.8 Bound relation bootstrap

Until the subtype relation is loaded, the runtime may:

- delay public type application;
- use a trusted bootstrap relation for core declarations;
- invoke the exact-equivalence fallback only for bootstrap-internal declarations whose bounds are known safe.

It must not expose a public application result that bypasses a required upper-bound check.

### 8.9 GC requirements

The GC must trace:

```text
owner → parameters
owner → signature
signature → owner
signature → parameters
parameter → owner
parameter → bound
parameter → constraints
```

Source locations and names follow their ordinary ownership rules. The runtime may intern `Variance` variants permanently. Parameter descriptors are not globally interned because owner identity already canonicalizes them.

### 8.10 Open-class mutation

Reopening a class or adding methods does not change its generic signature. Attempts to attach a second signature, reorder parameters, or replace a restriction must fail with `type.parameter.mutation` or a hardened malformed-metadata error.

### 8.11 Thread and fiber safety

Declaration completion and metadata attachment must be synchronized so concurrent reflection observes either:

- no published declaration because initialization is incomplete; or
- the complete immutable signature.

No fiber may observe a partially populated parameter list.

## 9. Reflection and metadata

### 9.1 Owner reflection

Every trusted owner exposes:

```phalcom
owner.typeParameters -> const List<TypeParameter>
owner.genericSignature -> Option<GenericSignature>
```

The list is immutable, source ordered, and identity stable.

### 9.2 Parameter reflection

Every `TypeParameter` exposes at least:

```phalcom
name -> Symbol
owner -> TypeParameterOwner
index -> Int
variance -> Variance
bound -> Option<Type>
constraints -> const List<Type>
default -> Option<Type>
sourceLocation -> Option<SourceLocation>
displayName -> String
freeParameters -> const List<TypeParameter>
substitute(using: TypeEnvironment) -> Type
equivalentTo(other: Type) -> Bool
validate(argument: Type) -> None
```

### 9.3 Signature reflection

Every `GenericSignature` exposes at least:

```phalcom
owner -> TypeParameterOwner
parameters -> const List<TypeParameter>
arity -> Int
isEmpty -> Bool
parameterAt(index: Int) -> Option<TypeParameter>
validate(arguments: const List<Type>) -> None
environmentFor(arguments: const List<Type>) -> TypeEnvironment
```

### 9.4 Source versus normalized metadata

The compiler may retain both:

- source syntax/ranges, including absent markers and exact spelling;
- normalized descriptor references.

Reflection defined here returns normalized objects. Document 17 may add APIs exposing source-form annotations without changing normalized identity.

### 9.5 Variance reflection

Variance is reflected even before Document 08 performs legality checking. Tools may render:

```text
T
out T
in T
```

based on the value. They must not infer or rewrite variance based on member usage.

### 9.6 Restriction reflection

An unrestricted parameter reports:

```phalcom
parameter.bound == None
parameter.constraints.isEmpty
```

A bounded parameter reports one bound and an empty constraint list. A constrained parameter reports `None` and a non-empty list.

No reflection API fabricates `Object`, `Any`, or `Dynamic` for an absent bound.

### 9.7 Annotation identity

When a member annotation names an owner parameter, reflection must return the same object:

```phalcom
const t = Box.typeParameters.first
const returnType = Box.methodFor(#value).returnType.unwrap

System.assert(returnType === t)
```

For a method-owned parameter, its uses likewise return that method's owned object.

### 9.8 Documentation rendering

Recommended rendering:

```text
class Box<T>
class Producer<out T>
class Consumer<in T>
class Garage<T: Vehicle>
class DatabaseId<T in (Int, String)>
map<U>(transform: (T) -> U) -> Sequence<U>
```

When shadowing is ambiguous in cross-links, documentation should display owner qualification while retaining source spelling in code excerpts.

### 9.9 Metadata stability

Renaming a parameter in a recompilation creates a new source spelling on a newly loaded owner descriptor. Within one descriptor lifetime, name is immutable. Replacing a module creates new owner and parameter identities; serialized caches must resolve through declaration records rather than assume pointer stability across runs.

### 9.10 Attribute interaction

Attributes attached directly to a type parameter are not introduced by this document. The AST and metadata encoding may reserve a field for future parameter attributes, but user-facing syntax is deferred. Attributes on the owning class, protocol, method, or requirement remain governed by their existing specifications.

## 10. Validation and diagnostics

### 10.1 Diagnostic shape

Every diagnostic contains:

```text
code
message
primary source range
owner identity or declaration context when available
parameter name and index when available
structured detail fields
related ranges when another declaration caused the conflict
```

Manual-construction errors use `sourceLocation` from `TypeParameterSpec` when supplied and otherwise report no source range.

### 10.2 Stable diagnostic codes

| Code | Condition | Primary range | Required fields |
|---|---|---|---|
| `type.parameter.invalid_name` | empty or invalid manual name | parameter/specification | `name` |
| `type.parameter.duplicate_name` | duplicate name in one owner | duplicate name | `owner`, `name`, `firstIndex`, `duplicateIndex` |
| `type.parameter.invalid_variance` | non-`Variance` manual metadata | variance/specification | `variance` |
| `type.variance.method_parameter` | `in`/`out` on method-owned parameter | variance marker | `owner`, `name`, `variance` |
| `type.parameter.bound_and_constraints` | both restriction forms supplied manually or malformed metadata | restriction/specification | `name`, `bound`, `constraints` |
| `type.parameter.empty_constraints` | `TypeParameterSpec.constrained` receives no alternatives | specification | `name` |
| `type.parameter.duplicate_constraint` | equivalent alternatives repeat | duplicate alternative | `name`, `firstIndex`, `duplicateIndex`, `constraint` |
| `type.parameter.default_not_supported` | source or manual default is present | default clause | `name`, `default` |
| `type.parameter.invalid_index` | negative owned index | metadata record | `index` |
| `type.parameter.invalid_owner` | owner is not trusted | owner record | `owner`, `ownerKind` |
| `type.parameter.recursive_restriction` | direct or mutual same-signature recursion | restriction | `owner`, `parameter`, `restriction` |
| `type.parameter.recursive_restriction_deferred` | guarded same-signature occurrence reserved for Document 09 | occurrence/restriction | `owner`, `parameter`, `restriction` |
| `type.signature.foreign_parameter` | parameter belongs to another owner | parameter record | `owner`, `parameter`, `parameterOwner`, `index` |
| `type.signature.index_mismatch` | parameter index differs from list position | parameter record | `owner`, `expectedIndex`, `actualIndex` |
| `type.signature.duplicate_parameter` | same parameter appears more than once | duplicate record | `owner`, `parameter`, `firstIndex`, `duplicateIndex` |
| `type.application.argument_count` | exact argument arity mismatch | application argument list | `owner`, `expected`, `received` |
| `type.argument.bound_violation` | argument is not subtype of bound | offending argument | `parameter`, `argument`, `bound` |
| `type.argument.constraint_violation` | argument is not equivalent to a finite alternative | offending argument | `parameter`, `argument`, `constraints` |
| `type.parameter.construction_authority` | forged trusted-construction call | call/metadata | none required |
| `type.parameter.mutation` | signature or parameter changed after publication | mutation site | `owner`, `operation` |
| `type.metadata.malformed_parameter` | malformed serialized parameter record | metadata record | `ownerReference`, `index`, `reason` |

### 10.3 Duplicate constraints

Duplicate detection uses type equivalence, not display text. Two aliases that normalize to equivalent descriptors may be diagnosed as duplicates once alias normalization is defined in Document 16.

### 10.4 Invalid type expressions

Bounds, constraints, and future defaults must normalize through Document 02's trusted type-expression boundary. Invalid or untrusted objects use the corresponding `type.expression.*` diagnostics rather than being converted into a generic parameter error.

### 10.5 Constraint arity

The source grammar requires at least one finite alternative. `T in (Int)` is legal and differs from `T: Int`: the finite constraint requires explicit equivalence to `Int`, while the bound admits subtypes. `TypeParameterSpec.constrained` rejects an empty list; direct `TypeParameterSpec.new(... constraints: const [])` denotes an unrestricted parameter.

### 10.6 Error precedence

When several errors are present in one parameter, preferred order is:

1. parse error;
2. invalid/duplicate name;
3. illegal variance marker;
4. unsupported default;
5. bound/constraint shape conflict;
6. invalid type-expression resolution;
7. duplicate constraints;
8. recursive restriction;
9. owner/index/signature integrity.

Compilers may recover and report multiple independent parameters, but diagnostics for one malformed parameter must remain deterministic.

### 10.7 No runtime value diagnostics

Assigning or storing an ordinary value does not trigger these diagnostics merely because a generic annotation exists. Only explicit type-expression construction, checking, reflection validation, or trusted metadata loading invokes them.

## 11. Interaction with earlier specifications

### 11.1 Document 01 — Protocol Foundation

This document completes Document 01's forward references:

- `TypeParameterSpec` is the public ownerless input used by `Protocol.new(...)` and `ProtocolRequirementDraft`;
- `TypeParameterRuntime.bindOwned` supplies exact owner/index objects;
- protocol-owned parameters use the `Protocol` descriptor as owner;
- requirement-owned parameters use the `ProtocolRequirement` descriptor as owner;
- `Protocol#typeParameters` and `ProtocolRequirement#typeParameters` remain immutable source-ordered collections;
- protocol requirement signatures may refer to protocol-owned and method-owned parameters by exact object identity.

No protocol requirement gains executable behavior. Generic metadata does not turn requirements into inherited methods.

### 11.2 Document 02 — Type Expression Foundation

`TypeParameter` is the first fully defined synthetic `TypeDescriptor` kind.

It preserves Document 02 invariants:

- the parameter itself is its origin;
- it has no arguments or declared parameters;
- its free parameters are `[self]`;
- substitution uses declaration identity;
- equivalence is not subtyping;
- trusted normalized metadata accepts the registered `TypeParameter` descriptor kind;
- bare generic origins remain closed and report their declaration parameters separately.

### 11.3 Selector identity

Generic parameter headers and type annotations never enter selector encoding. Duplicate method rules remain selector-based.

### 11.4 Modules and declaration identity

The owner descriptor's module binding and declaration identity establish the lifetime and uniqueness domain for parameter identity. Manual protocols require the explicit module semantics from Document 01.

### 11.5 Reflection

Existing `Method`, `Class`, `Protocol`, and `ProtocolRequirement` reflection gains generic signature access without changing executable identity, method table entries, or requirement identity.

## 12. Examples

### 12.1 Invariant default

```phalcom
class Cell<T> {}

const t = Cell.typeParameters.first
System.assert(t.variance == Variance.Invariant)
System.assert(t.isUnrestricted)
```

### 12.2 Covariant protocol parameter

```phalcom
@protocol
class Source<out T> {
  next -> Option<T>
}

const t = Source.typeParameters.first
System.assert(t.owner === Source)
System.assert(t.variance == Variance.Covariant)
```

### 12.3 Contravariant class parameter

```phalcom
class Sink<in T> {
  accept(value: T) -> Unit {
    ...
  }
}

System.assert(
  Sink.typeParameters.first.variance ==
    Variance.Contravariant
)
```

Document 08 may reject an illegal use of this parameter elsewhere; reflection is established here.

### 12.4 Bound reflection

```phalcom
class Garage<T: Vehicle> {}

const t = Garage.typeParameters.first
System.assert(t.bound.unwrap === Vehicle)
System.assert(t.constraints.isEmpty)
System.assert(t.isBounded)
```

### 12.5 Constraint reflection

```phalcom
class DatabaseId<T in (Int, String)> {}

const t = DatabaseId.typeParameters.first
System.assert(t.bound == None)
System.assert(t.constraints.size == 2)
System.assert(t.constraints.at(0) === Int)
System.assert(t.constraints.at(1) === String)
System.assert(t.isConstrained)
```

### 12.6 Owner/index identity

```phalcom
class Left<T> {}
class Right<T> {}

const leftT = Left.typeParameters.first
const rightT = Right.typeParameters.first

System.assert(leftT.name == rightT.name)
System.assert(leftT.owner === Left)
System.assert(rightT.owner === Right)
System.assert(leftT.equivalentTo(rightT).not)
```

### 12.7 Stable repeated reflection

```phalcom
class Box<T> {}

const first = Box.typeParameters.first
const second = Box.typeParameters.first

System.assert(first === second)
System.assert(first.hash == second.hash)
```

### 12.8 Member annotation identity

```phalcom
class Box<T> {
  value -> T {
    ...
  }
}

const t = Box.typeParameters.first
const reflected = Box.methodFor(#value).returnType.unwrap

System.assert(reflected === t)
```

### 12.9 Method-owned parameter identity

```phalcom
class Sequence<T> {
  map<U>(transform: (T) -> U) -> Sequence<U> {
    ...
  }
}

const method = Sequence.methodFor(#map)
const u = method.typeParameters.first

System.assert(u.owner === method)
System.assert(u.index == 0)
System.assert(u.variance == Variance.Invariant)
```

### 12.10 Shadowing

```phalcom
class Outer<T> {
  identity<T>(value: T) -> T {
    return value
  }
}

const outerT = Outer.typeParameters.first
const methodT = Outer.methodFor(#identity).typeParameters.first

System.assert(outerT.name == methodT.name)
System.assert(outerT.equivalentTo(methodT).not)
```

### 12.11 Generic signature

```phalcom
class Mapping<K, V> {}

const signature = Mapping.genericSignature.unwrap

System.assert(signature.owner === Mapping)
System.assert(signature.arity == 2)
System.assert(signature.parameters === Mapping.typeParameters)
System.assert(signature.parameterAt(0).unwrap.name == #K)
System.assert(signature.parameterAt(2) == None)
```

### 12.12 Non-generic owner

```phalcom
class Point {}

System.assert(Point.typeParameters.isEmpty)
System.assert(Point.genericSignature == None)
```

### 12.13 Constraint validation success

```phalcom
class Identifier<T in (Int, String)> {}

const signature = Identifier.genericSignature.unwrap
signature.validate(arguments: const [Int])
signature.validate(arguments: const [String])
```

### 12.14 Constraint validation failure

```phalcom
class Identifier<T in (Int, String)> {}

Identifier.genericSignature.unwrap.validate(
  arguments: const [Float]
)
// throws TypeConstraintError, code type.argument.constraint_violation
```

### 12.15 Bound validation

```phalcom
class Garage<T: Vehicle> {}

Garage.genericSignature.unwrap.validate(
  arguments: const [Car]
)
```

This succeeds when `Car <: Vehicle` under Document 07's subtype relation.

### 12.16 Wrong arity

```phalcom
class Pair<A, B> {}

Pair.genericSignature.unwrap.validate(
  arguments: const [Int]
)
// throws TypeArgumentCountError, code type.application.argument_count
```

### 12.17 Manual protocol parameters

```phalcom
const protocol = Protocol.new(
  name: #Producer,
  module: currentModule,
  typeParameters: const [
    TypeParameterSpec.covariant(name: #T)
  ],
  requirements: const [],
  attributes: const [],
  documentation: None,
  sourceLocation: None
)

const t = protocol.typeParameters.first
System.assert(t.name == #T)
System.assert(t.owner === protocol)
System.assert(t.index == 0)
```

### 12.18 Specification reuse creates distinct identities

```phalcom
const spec = TypeParameterSpec.invariant(name: #T)

const left = Protocol.new(
  name: #Left,
  module: currentModule,
  typeParameters: const [spec],
  requirements: const [],
  attributes: const [],
  documentation: None,
  sourceLocation: None
)

const right = Protocol.new(
  name: #Right,
  module: currentModule,
  typeParameters: const [spec],
  requirements: const [],
  attributes: const [],
  documentation: None,
  sourceLocation: None
)

System.assert(
  left.typeParameters.first.equivalentTo(
    right.typeParameters.first
  ).not
)
```

### 12.19 Substitution identity

```phalcom
class Box<T> {}

const t = Box.typeParameters.first
const empty = TypeEnvironment.empty

System.assert(t.substitute(using: empty) === t)
```

Document 05 defines bound environments that replace `t` by owner/index identity.

### 12.20 Illegal method variance

```phalcom
class Invalid {
  transform<out T>(value: Object) -> T {
    ...
  }
}
```

Diagnostic: `type.variance.method_parameter` on `out`.

### 12.21 Duplicate parameter name

```phalcom
class Invalid<T, T> {}
```

Diagnostic: `type.parameter.duplicate_name` on the second `T`.

### 12.22 Duplicate constraint

```phalcom
class Invalid<T in (Int, Int)> {}
```

Diagnostic: `type.parameter.duplicate_constraint` on the second `Int`.

### 12.23 Unsupported default

```phalcom
class Invalid<T = Object> {}
```

Diagnostic: `type.parameter.default_not_supported` on `= Object`.

### 12.24 Deferred F-bound

```phalcom
class Ordered<T: Comparable<T>> {}
```

Diagnostic: `type.parameter.recursive_restriction_deferred` until Document 09.

### 12.25 Generic annotations do not enforce values

```phalcom
class Box<T> {
  var value: T
}

const box = Box.new()
box.value = "text"
box.value = 42
```

The ordinary runtime does not reject either assignment solely because `T` exists. A checker or explicit validator may report mismatches when it has an applied type context.

## 13. Conformance tests

The following fixture names are normative suggestions. Equivalent harness names are allowed, but each behavior must be covered.

### 13.1 Parser fixtures

#### Positive

- `generic/class_invariant.ph`: parses `class Box<T> {}`.
- `generic/class_covariant.ph`: parses `class Producer<out T> {}`.
- `generic/class_contravariant.ph`: parses `class Consumer<in T> {}`.
- `generic/protocol_multiple.ph`: parses `@protocol class Mapping<K, out V> {}`.
- `generic/method_parameter.ph`: parses `map<U>(...)`.
- `generic/bound.ph`: parses `T: Vehicle`.
- `generic/constraints.ph`: parses `T in (Int, String)`.
- `generic/contextual_in.ph`: distinguishes contravariance and constraints.

#### Negative

- `generic/missing_parameter_name.ph`.
- `generic/trailing_comma_policy.ph` according to the shared list grammar.
- `generic/empty_constraint_list.ph` reports a parse error or `type.parameter.empty_constraints`.
- `generic/default_not_supported.ph`.
- `generic/malformed_bound.ph`.
- `generic/malformed_constraint_separator.ph`.

### 13.2 Ownership fixtures

```phalcom
class Left<T> {}
class Right<T> {}

const left = Left.typeParameters.first
const right = Right.typeParameters.first

assert(left.owner === Left)
assert(right.owner === Right)
assert(left.index == 0)
assert(right.index == 0)
assert(left.equivalentTo(right).not)
```

Required additional coverage:

- two parameters in one owner have indexes `0`, `1`;
- repeated reflection returns identical objects;
- method parameter owner is the exact method descriptor;
- protocol requirement parameter owner is the exact requirement descriptor;
- module reload creates new owner and parameter identities.

### 13.3 Variance fixtures

Positive:

```phalcom
class Invariant<T> {}
class Output<out T> {}
class Input<in T> {}

assert(Invariant.typeParameters.first.variance == Variance.Invariant)
assert(Output.typeParameters.first.variance == Variance.Covariant)
assert(Input.typeParameters.first.variance == Variance.Contravariant)
```

Negative:

```phalcom
class Invalid {
  method<in T>() {}
}
```

Expected code: `type.variance.method_parameter`.

Position legality is not asserted until Document 08.

### 13.4 Restriction fixtures

Positive:

- unrestricted parameter reports no bound and empty constraints;
- bounded parameter reports exact bound object;
- constrained parameter preserves source order;
- method bound may reference an enclosing class parameter without changing ownership.

Negative:

- both bound and constraints in manual metadata;
- duplicate equivalent constraints;
- an empty list passed to `TypeParameterSpec.constrained`;
- untrusted object as bound;
- direct self-bound;
- mutual same-signature bound cycle;
- guarded same-signature bound receives deferred diagnostic;
- default receives unsupported diagnostic.

### 13.5 Signature fixtures

```phalcom
class Pair<A, B> {}
const signature = Pair.genericSignature.unwrap

assert(signature.owner === Pair)
assert(signature.arity == 2)
assert(signature.parameters === Pair.typeParameters)
assert(signature.parameterAt(0).unwrap === Pair.typeParameters.at(0))
assert(signature.parameterAt(1).unwrap === Pair.typeParameters.at(1))
assert(signature.parameterAt(2) == None)
```

Negative trusted-construction probes must reject:

- foreign parameter owner;
- index mismatch;
- duplicate parameter object;
- repeated signature attachment;
- attachment after publication.

### 13.6 Annotation identity fixtures

For every owner kind, a source annotation reference must resolve to the same parameter object:

- class field annotation;
- method parameter annotation;
- method result annotation;
- protocol requirement parameter annotation;
- protocol requirement result annotation;
- bound expression containing an enclosing parameter.

### 13.7 Argument validation fixtures

- exact arity succeeds;
- too few arguments uses `type.application.argument_count`;
- too many arguments uses the same code;
- constraint member succeeds;
- non-member fails with `type.argument.constraint_violation`;
- subtype of bound succeeds once Document 07 relation is loaded;
- unrelated type fails with `type.argument.bound_violation`;
- arguments normalize through Document 02 trusted type-expression rules.

### 13.8 Non-dispatch fixtures

Two generic methods with the same ordinary selector are duplicates even when parameter names, bounds, or annotations differ.

A runtime send to an ordinary instance must produce the same selector lookup with and without generic metadata.

### 13.9 Manual construction fixtures

- reusable `TypeParameterSpec` creates distinct parameters for distinct owners;
- manual generic protocol returns a non-empty signature;
- manual non-generic protocol returns `None` signature;
- invalid specification aborts owner construction atomically;
- manual source location appears in diagnostics;
- constraints and bound collections are frozen snapshots.

### 13.10 Bootstrap fixtures

- `Type` can reference `TypeParameter` before full typing bootstrap without exposing unresolved placeholders;
- generic core declarations publish complete signatures only;
- failed header resolution leaves no visible owner;
- retry/reload allocates fresh identities;
- concurrent reflection never sees a partial parameter list.

### 13.11 GC fixtures

- owner retains parameters and signature;
- retained parameter keeps owner identity valid;
- bound and constraint descriptors remain alive while parameter is alive;
- unreachable owner/signature/parameter cycles are collectible;
- canonical `Variance` values remain stable.

### 13.12 Malformed metadata fixtures

A hardened loader rejects records with:

- negative index;
- duplicate index;
- gap in indexes;
- owner mismatch;
- unknown variance tag;
- both bound and constraints;
- malformed metadata that marks a finite constraint form but contains no alternatives;
- default in metadata version 1;
- untrusted type-expression reference;
- same-signature recursion unsupported by the active metadata version;
- parameter count inconsistent with signature record.

### 13.13 Acceptance matrix

A conforming implementation must cover at least:

| Area | Positive | Negative |
|---|---:|---:|
| Grammar | 8 | 7 |
| Identity/ownership | 6 | 4 |
| Variance metadata | 3 | 2 |
| Bounds/constraints | 5 | 8 |
| Generic signatures | 5 | 5 |
| Annotation identity | 6 | 2 |
| Argument validation | 4 | 4 |
| Manual construction | 5 | 4 |
| Bootstrap/GC | 6 | 5 |
| Malformed metadata | 0 | 11 |

Tests may overlap categories, but every normative rule requires at least one passing and one failing fixture where failure is meaningful.

## 14. Native implementation latitude

### 14.1 Permitted native responsibilities

Rust/VM code may implement or accelerate:

- hidden parameter-shell allocation and completion;
- trusted owner recognition;
- construction-token authority;
- owner/index canonical attachment;
- `TypeParameter` and `GenericSignature` allocation;
- immutable metadata freezing;
- lexical parameter binding tables;
- metadata decoding and validation;
- bound-check calls into the subtype relation;
- owner/signature side tables when core object layouts cannot yet store fields;
- GC tracing and cycle handling;
- compact variance encoding;
- source-location attachment.

### 14.2 Required observable contract

Native implementations must preserve:

- exact owner/index identity;
- source order;
- object identity on repeated reflection;
- immutable public collections;
- exact variance values;
- absent bound/default representation;
- finite constraint order and equivalence uniqueness;
- stable diagnostic codes and detail fields;
- atomic publication;
- `TypeParameter.freeParameters == const [self]`;
- substitution by declaration identity;
- non-participation in ordinary dispatch.

### 14.3 Forbidden native shortcuts

A conforming implementation must not:

- use parameter name as identity;
- recreate new parameter objects on each reflection call;
- encode generic parameter lists into selectors;
- attach type arguments to ordinary instances implicitly;
- treat absent bounds as `Object` or `Any`;
- accept method variance and silently erase it;
- skip constraint duplicate checks;
- expose partially initialized shells;
- mutate signatures after class reopening;
- accept forged structural owners as trusted metadata;
- use a different explicit constraint rule than equivalence membership.

### 14.4 Side-table implementations

A bootstrap VM may store generic metadata in side tables keyed by owner identity. Side tables must:

- be traced or otherwise kept consistent with GC;
- preserve stable parameter object identity;
- reject duplicate attachment;
- release entries when unreachable owners are collected unless the owner is a permanent core root;
- be indistinguishable from direct immutable fields through reflection.

### 14.5 Diagnostics

Native metadata rejection may wrap low-level decoding failures, but the surfaced diagnostic must use the stable code taxonomy in Section 10. Security-sensitive details may be redacted from the human message while structured safe fields remain available.

## 15. Non-goals and deferred work

### 15.1 No type application

This document does not define evaluation of `Box<Int>`. Document 04 defines `<...>`, arity validation, canonical `AppliedType`, and interning.

### 15.2 No partial application or defaults

Generic arguments must eventually be complete and exact in first-version application. Defaults are represented as `None` and source/manual defaults are rejected.

### 15.3 No variance checking

`out` and `in` are reflected, but this document does not inspect member positions, mutable fields, constructors, nested variance, or inheritance. Document 08 is authoritative.

### 15.4 No inference

Method-generic inference, constraint solving, promotions, ambiguity, and inferred substitutions are Document 09.

### 15.5 No F-bounds yet

The representation can contain arbitrary type expressions, but same-signature recursive restrictions are rejected until Document 09 defines guarded recursion and solver termination.

### 15.6 No structural bound checking yet

Protocol bounds ultimately use structural conformance and subtyping rules from Documents 07, 09, and 10. This document only fixes the reflected field and validation hook.

### 15.7 No generic inheritance

Superclass applications, specialization by subclasses, inherited annotations, and cycle detection are Document 12.

### 15.8 No parameter attributes

Annotations such as per-parameter variance attributes, documentation attributes, or generation constraints are deferred. They must not be smuggled into `TypeParameterSpec.attributes` because no such public field is specified here.

### 15.9 No runtime value enforcement

Generic declarations remain optional reflective typing metadata. They do not automatically validate assignments, arguments, returns, collection contents, or fields.

### 15.10 No user-defined trusted owner kinds

Structural `TypeParameterOwner` behavior is public, but compiler metadata accepts only trusted built-in owner kinds in this version. Document 17 or 18 may define controlled extension.

## 16. Normative invariants

1. Every source generic parameter produces exactly one immutable `TypeParameter` object.
2. Every parameter identity is owner identity plus zero-based declaration index.
3. Parameter name is descriptive and never identity.
4. Parameter order exactly matches source order.
5. Indexes are contiguous from zero and equal collection positions.
6. Repeated reflection on one owner returns the same parameter objects.
7. Parameters from different owners are not equivalent even when all visible fields match.
8. `TypeParameter` is a trusted synthetic `TypeDescriptor` kind.
9. `TypeParameter.displayName` is its name string.
10. `TypeParameter.origin` is itself.
11. `TypeParameter.arguments` and `typeParameters` are empty.
12. `TypeParameter.freeParameters` is exactly `const [self]`.
13. Type-parameter substitution resolves by owner/index identity and leaves an unbound parameter unchanged.
14. Bounds and constraints do not contribute additional entries to the parameter's `freeParameters` result.
15. Unmarked class and protocol parameters are invariant.
16. `out` denotes `Variance.Covariant`; `in` before a name denotes `Variance.Contravariant`.
17. Method-owned type parameters are invariant and reject `in`/`out` markers.
18. One parameter may declare at most one upper bound.
19. One parameter may declare either a bound or a finite constraint set, never both.
20. A finite constraint set contains at least one type expression, and all members are pairwise non-equivalent.
21. Constraint reflection preserves source order.
22. Explicit finite-constraint validation uses type equivalence membership.
23. Explicit upper-bound validation uses the subtype relation defined by later documents.
24. Every first-version parameter default is `None`; source or manual defaults are rejected.
25. Same-signature recursive restrictions are rejected until Document 09.
26. Restrictions may refer to enclosing-owner parameters.
27. Duplicate names are rejected within one owner.
28. Nested declarations may lexically shadow enclosing parameter names.
29. Generic parameter metadata never changes selector identity or dispatch.
30. A generic owner has one identity-stable `GenericSignature`.
31. A non-generic owner exposes an empty parameter list and `genericSignature == None`.
32. A signature's owner is the same object that owns every parameter in the signature.
33. A signature's parameter list is source ordered and immutable.
34. Exact application arity equals signature arity; partial application is not supported.
35. Member annotations naming a parameter reflect the exact owned parameter object.
36. Compiler declarations use hidden shells and publish only completed immutable descriptors.
37. Manual specifications are ownerless inputs and create fresh owned identities per owner.
38. Parameter/signature construction requires trusted authority.
39. Reopening a declaration cannot mutate its type parameters or signature.
40. GC traces owner, signature, parameter, bound, and constraint references consistently.
41. Malformed serialized metadata is rejected before publication.
42. Native implementations preserve the visible Phalcom source contract exactly.

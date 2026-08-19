# Protocol Foundation

- **Status:** Proposed normative design; not a claim of current compiler or VM support
- **Date:** 2026-07-23
- **Depends on:** class-shaped declarations, attributes/decorators, selector identity, modules and lexical bindings, immutable collections, basic method/parameter/source reflection
- **Supersedes:** the protocol fragments in earlier experimental typing notes and the Phase 1 reference package wherever they conflict with this document
- **Superseded by:** none
- **Related ADRs and specifications:** `docs/spec/current/decorators/README.md`, `docs/spec/current/decorators/native.md`, `docs/spec/current/decorators/constructor.md`, `docs/spec/current/syntax/statements-and-declarations.md`, `docs/spec/current/selectors.md`, the current module and object-model specifications, and Documents 02, 03, 10, 17, 18, 19, 20, and 21 of this series

This document is the first normative part of the Phalcom typing specification series. It establishes first-class protocol descriptors and the `@protocol` declaration product. Later documents add the common `Type` protocol, generic signatures, applied types, substitution, structural conformance, metadata encoding, bootstrap hardening, checker modes, and the complete conformance suite.

The visible Phalcom source in Section 6 is normative. A native implementation may replace selected `@native` methods only when it preserves the source contract exactly.

## 1. Purpose and scope

### 1.1 Purpose

Phalcom dispatches ordinary messages by selector identity and receiver behavior. Optional type metadata must not create a second method-selection mechanism. Protocols therefore describe message capabilities without changing ordinary lookup, inheritance, layout, allocation, or inline-cache identity.

The declaration:

```phalcom
@protocol
class Drawable {
  draw() -> Unit
  bounds() -> Rect
}
```

creates one first-class `Protocol` descriptor and binds it to `Drawable`. It does not create an ordinary class, an abstract superclass, a trait, a mixin, a set of installed stubs, or a wrapper around conforming values.

This document answers three foundation questions normatively:

1. **What does the declaration create?** A distinct immutable `Protocol` descriptor whose requirements are owned reflective descriptors.
2. **How can the same protocol be constructed without the decorator?** By calling the public validated `Protocol.new(...)` constructor with explicit owner, requirement drafts, attributes, documentation, and source metadata, then binding the returned descriptor through an ordinary module binding.
3. **Which parts require compiler or VM authority?** Declaration classification, bodyless-signature parsing, lexical declaration identity, recursive declaration shells, trusted ownership attachment, metadata emission, source-range fidelity, descriptor freezing, and malformed-metadata rejection require compiler or VM authority. Ordinary reflection, filtering, lookup, display, and most validation semantics are standard-library behavior.

### 1.2 In scope

This document specifies:

- `@protocol class Name { ... }` as class-shaped syntax producing a `Protocol` object;
- signature-only instance-side and class-side requirements;
- the exact legal and illegal member forms in a protocol declaration;
- protocol descriptor identity, ownership, immutability, hashing, module binding, and non-instantiability;
- `Protocol`, `ProtocolRequirement`, `ProtocolParameter`, their input-draft classes, and the `ProtocolAttribute` source model;
- a public direct-construction API with the same behavioral semantics as decorator construction;
- selector, label, annotation, attribute, documentation, and source-location retention;
- parser, AST, expansion, declaration-indexing, metadata-resolution, compilation, interpreter, VM, bootstrap, and GC obligations;
- reflection APIs and normalized metadata shapes;
- validation rules and stable diagnostic codes;
- positive, negative, reflection, bootstrap, and manual-construction fixtures;
- the exact boundary between protocols, ordinary classes, and abstract classes.

### 1.3 Out of scope

The following are deliberately assigned to later documents:

- the complete `Type` protocol and synthetic type-expression architecture: Document 02;
- full generic parameter semantics, variance, bounds, finite constraints, and method-owned type parameters: Document 03;
- angle application and applied protocol types: Document 04;
- applied requirement views and annotation substitution: Document 05;
- special types such as `Dynamic`, `Any`, `Nothing`, and `Self`: Document 07;
- structural conformance, protocol inheritance/composition, recursive conformance, explicit conformance declarations, and open-world cache invalidation: Document 10;
- bytecode and metadata encoding details beyond the minimum record shape established here: Document 17;
- hardened bootstrap, trusted shells, selector protection, and security boundaries: Document 18;
- checker modes, LSP rendering, and diagnostic presentation policy: Document 19.

This document reserves stable fields and interfaces needed by those later parts; later documents may refine their semantics but must not silently rename or remove them.

### 1.4 Design alternatives and final decisions

#### 1.4.1 Dedicated `protocol` keyword versus `@protocol class`

Serious options:

1. Add a separate `protocol` declaration grammar.
2. Parse an ordinary class and attach a protocol flag after construction.
3. Reuse class-shaped grammar but let `@protocol` replace the declaration product.

Decision: **Option 3.** It preserves one declaration grammar and the existing attribute/decorator model without pretending that the result is an ordinary class.

#### 1.4.2 Flagged class versus distinct descriptor

Serious options:

1. Represent a protocol as a `Class` with `isProtocol = true`.
2. Represent a protocol as an abstract class with no implementation.
3. Represent a protocol as a distinct `Protocol` object.

Decision: **Option 3.** A protocol has no instance layout, superclass chain, constructors, inherited stubs, allocation path, or concrete method table. Modeling it as a class would expose invalid operations and force every class operation to branch on a flag.

#### 1.4.3 Signature-only requirements versus defaults

Serious options:

1. Signature-only requirements.
2. Trait-like default implementations installed into conformers.
3. Callable defaults stored on the protocol and used through protocol-directed dispatch.

Decision: **Option 1.** Defaults would introduce code reuse, conflict resolution, installation order, a second dispatch path, or both. Those concerns belong to a future trait or mixin design.

#### 1.4.4 Class-side requirements

Serious options:

1. Defer class-side requirements and support only instance-side requirements.
2. Include class-side requirements in the first descriptor model and syntax.

Decision: **Option 2.** Phalcom already has class-side selectors. Deferring them would force a later metadata shape change and would prevent protocols from describing factories, parsers, registries, and other class-object capabilities. Class-side requirements remain signature-only and do not make the protocol itself instantiable.

#### 1.4.5 Manual construction API

Serious options:

1. A mutable `ProtocolBuilder` finalized by `build`.
2. A public direct constructor receiving immutable requirement drafts.
3. A compiler-only constructor with no public manual equivalent.

Decision: **Option 2.** `Protocol.new(...)` is the canonical low-level public API. It validates immutable drafts and returns a complete frozen descriptor. A builder may be added later as convenience, but it must lower to this constructor and may not create different semantics.

## 2. Terminology

### 2.1 Protocol declaration

A class-shaped declaration carrying the declaration-product attribute `@protocol`.

### 2.2 Protocol descriptor

An instance of the standard-library class `Protocol`. A protocol descriptor is named, immutable, reflectable, non-instantiable, and structurally interpreted by later conformance algorithms.

### 2.3 Ordinary class

A `Class` descriptor that participates in inheritance, owns or inherits instance layout, owns executable instance and class-side methods, and may allocate instances unless abstract or otherwise restricted.

### 2.4 Abstract class

An ordinary class with unresolved abstract obligations. It still has a superclass, layout, executable methods, possible constructors, and inherited behavior. Abstract-class semantics are specified in Document 11.

### 2.5 Requirement

An immutable owned descriptor stating that a candidate must provide a compatible selector on either its instance side or class side. A requirement contains metadata only and has no executable body.

### 2.6 Requirement draft

An immutable ownerless input record accepted by `Protocol.new(...)`. Construction validates the draft, creates owned `ProtocolRequirement` descriptors, assigns declaration-order indexes, and freezes the result.

### 2.7 Parameter draft and parameter descriptor

A parameter draft is ownerless input metadata. A `ProtocolParameter` is an owned immutable descriptor attached to one `ProtocolRequirement` at a declaration-order index.

### 2.8 Requirement side

One of:

- `ProtocolRequirementSide.Instance`: a selector required on values of a candidate class;
- `ProtocolRequirementSide.ClassSide`: a selector required on the candidate class object.

The side is metadata. It does not alter selector encoding.

### 2.9 Selector identity

The normal Phalcom selector identity: base selector name, arity, positional structure, and labels. Type annotations are never part of selector identity.

### 2.10 Declaration identity

The identity of the descriptor object produced by one declaration evaluation. Names are descriptive and are not identity. Two separately evaluated declarations named `Drawable` produce distinct protocols.

### 2.11 Lexical owner

The module or other declaration scope that owns the protocol binding and qualifies its display name. The first version requires a module owner for top-level protocols.

### 2.12 Declaration-product attribute

An attribute that changes the kind of object produced by a class-shaped declaration. `@protocol` is such an attribute. It is not merely passive metadata and is mutually exclusive with other declaration-product attributes unless a later specification explicitly composes them.

### 2.13 Retain-tier requirement attribute

An attribute attached to a requirement that is retained as metadata but does not install, wrap, synthesize, replace, or weave executable behavior. Only retain-tier attributes are legal on first-version protocol requirements, apart from the structural `@class` side marker.

### 2.14 Absent annotation

A parameter or result without a source type annotation. Reflection preserves absence as `None`; it does not invent a source annotation. Later checking relations may interpret absence as `Dynamic`, but source reflection remains lossless.

### 2.15 Trusted declaration shell

A VM-allocated, one-time-initializable `Protocol` object used so recursive declarations can be indexed and referenced before all annotations are resolved. Manual construction does not expose shells.

## 3. User-facing syntax

### 3.1 Canonical declaration

```phalcom
@protocol
class Drawable {
  draw() -> Unit
  bounds() -> Rect
}
```

After declaration evaluation:

```phalcom
Drawable.class === Protocol
Drawable.name == #Drawable
Drawable.owner === Module.current
Drawable.requirements.size == 2
```

`Drawable` is not a class and cannot allocate values:

```phalcom
Drawable.new()
// raises ProtocolInstantiationError
```

### 3.2 Bodyless requirement grammar

The first-version grammar is conceptually:

```text
protocol-declaration
  ::= attribute-list "class" identifier generic-parameters? protocol-body

protocol-body
  ::= "{" protocol-member* "}"

protocol-member
  ::= documentation? attribute-list? method-signature terminator

method-signature
  ::= selector-signature method-type-parameters? parameter-clause? result-annotation?

terminator
  ::= line-terminator | ";" | before-"}"
```

The parser must preserve an absent method body in the AST. It must not synthesize an empty block. A requirement ends before the next member at a valid declaration boundary. A `{` following a requirement signature begins an executable body and is rejected during protocol validation.

This syntax is proposed even if the current parser does not yet accept it. The compiler must not claim conformance until it implements the grammar and fixtures in this document.

### 3.3 Instance-side requirements

An unmarked signature is instance-side:

```phalcom
@protocol
class CollectionView {
  size -> Int
  at(index: Int) -> Object
  includes(value: Object) -> Bool
}
```

Getter, unary, positional, labeled, and rest-selector forms are legal when they are otherwise legal Phalcom selectors.

### 3.4 Class-side requirements

`@class` on a bodyless signature creates a class-side requirement:

```phalcom
@protocol
class Parsable {
  @class
  parse(text: String) -> Self

  @class
  canParse(text: String) -> Bool
}
```

`@class` is normalized into `ProtocolRequirement.side`. It is not an executable decorator and does not install a method on the protocol descriptor.

Class-side requirements are checked against the candidate class object by Document 10. Calling `Parsable.parse(...)` does not dispatch to a requirement and does not search for a conforming class.

### 3.5 Selector labels remain decisive

These are distinct requirements:

```phalcom
@protocol
class Lookup {
  get(index: Int) -> Object
  get(key: String) -> Object
  get(_: Int) -> Object
}
```

The exact selector model, not the textual base name alone, distinguishes them. Type annotations do not distinguish overloads and cannot participate in ordinary dispatch.

### 3.6 Optional annotations

Typing remains optional:

```phalcom
@protocol
class Printable {
  printOn(stream)
}
```

The parameter and result annotation fields are `None`. The requirement still records the selector and parameter metadata. A later checker may treat missing annotations consistently with `Dynamic`; reflection must continue to report that no annotation was written.

### 3.7 Requirement attributes and documentation

Retain-tier attributes and documentation are legal:

```phalcom
@protocol
class Repository {
  /// Returns the value associated with `key`, or `None`.
  @deprecated("Use valueFor(key:) in new code")
  get(key: String) -> Option<String>
}
```

The requirement stores the instantiated `deprecated` attribute, documentation text, and exact source location. Behavior-changing attributes are illegal because there is no executable body to transform or wrap.

### 3.8 Generic protocol syntax

The declaration grammar reserves generic protocols and generic requirements:

```phalcom
@protocol
class Strategy<T> {
  draw(data: DrawData) -> T
  map<U>(transform: [T] -> U) -> Strategy<U>
}
```

This document requires the protocol and requirement descriptors to retain type-parameter collections. Document 03 defines their grammar, identity, variance, bounds, finite constraints, and validation. Until Document 03 is implemented, a compiler may parse and retain this syntax but must not claim full generic typing conformance.

### 3.9 Manual construction

The same behavioral descriptor can be created without `@protocol`:

```phalcom
const Drawable = Protocol.new(
  name: #Drawable,
  owner: Module.current,
  typeParameters: const [],
  requirements: const [
    ProtocolRequirementDraft.new(
      side: ProtocolRequirementSide.Instance,
      selector: Selector.method(#draw, labels: const []),
      typeParameters: const [],
      parameters: const [],
      resultType: Some.new(Unit),
      attributes: const [],
      documentation: None,
      sourceLocation: None
    ),
    ProtocolRequirementDraft.new(
      side: ProtocolRequirementSide.Instance,
      selector: Selector.method(#bounds, labels: const []),
      typeParameters: const [],
      parameters: const [],
      resultType: Some.new(Rect),
      attributes: const [],
      documentation: None,
      sourceLocation: None
    )
  ],
  attributes: const [],
  documentation: None,
  sourceLocation: None
)
```

The ordinary `const Drawable = ...` binding is what gives the manually created descriptor a name in the current module. `Protocol.new(...)` does not mutate a module namespace and does not infer an owner from the call stack.

### 3.10 Decorator and manual construction equivalence

Both paths produce a `Protocol` with identical behavioral semantics:

- same descriptor class;
- same non-instantiability;
- same selector and side interpretation;
- same reflection APIs;
- same structural-conformance meaning once Document 10 is implemented;
- same identity-based equality and hashing rules;
- same immutability guarantees.

They differ intentionally:

| Property | Decorator declaration | Manual construction |
|---|---|---|
| Name binding | Automatic declaration binding | Ordinary explicit binding by user code |
| Lexical owner | Compiler-derived | Explicit `owner:` argument |
| Source location | Exact compiler span | Caller-supplied or `None` |
| Documentation | Parsed automatically | Caller-supplied or `None` |
| Runtime `attributes` | Compiler-derived retain-tier attributes; `@protocol` is normalized as the declaration product | Caller-supplied retain-tier attributes |
| Declaration identity | One descriptor for one declaration evaluation | Fresh descriptor for each constructor call |
| Recursive references | Trusted shell and later completion | Inputs must already be resolved |
| Malformed metadata | Rejected before or during module loading | Rejected synchronously by constructor |

Behavioral equivalence does not imply object identity:

```phalcom
const A = Protocol.new(
  name: #Drawable,
  owner: Module.current,
  typeParameters: const [],
  requirements: const [],
  attributes: const [],
  documentation: None,
  sourceLocation: None
)

const B = Protocol.new(
  name: #Drawable,
  owner: Module.current,
  typeParameters: const [],
  requirements: const [],
  attributes: const [],
  documentation: None,
  sourceLocation: None
)

assert(A !== B)
assert(A != B)
```

### 3.11 Legal members

The following are legal in a first-version protocol body:

- instance-side bodyless method signatures;
- instance-side bodyless getter signatures;
- class-side bodyless method or getter signatures marked with `@class`;
- legal selector forms, including labels and supported rest parameters;
- optional parameter and result annotations;
- declared method type parameters, subject to Document 03;
- documentation comments;
- retain-tier attributes;
- repeated base names when complete selectors differ;
- the same complete selector once on the instance side and once on the class side.

### 3.12 Illegal members and combinations

The following are rejected:

- any executable method or getter body;
- expression-bodied requirements;
- fields, including `const`, `let`, or mutable instance fields;
- class fields or other stored declaration state;
- constructors and `@constructor`;
- initializers;
- nested classes, protocols, aliases, or executable declarations in the body;
- superclass clauses in the first version;
- protocol-composition clauses before Document 10;
- duplicate complete selectors on the same side;
- `@native`, `@abstract`, contract-weaving attributes, or any other behavior-changing requirement attribute;
- combining `@protocol` with `@data`, `@sealed`, `@variant`, `@abstract`, `@immutable`, or another declaration-product attribute;
- applying `@protocol` to a method, field, local binding, or any non-class-shaped declaration;
- using `@class` on anything other than a bodyless requirement inside a protocol declaration;
- constructor-like requirements marked with `@constructor`; use an ordinary class-side factory selector instead.

## 4. Semantic model

### 4.1 Declaration product

Evaluating a valid `@protocol class Name { ... }` declaration performs the following semantic operation:

```text
parse class-shaped declaration
→ classify @protocol as the declaration-product attribute
→ allocate or obtain a trusted Protocol declaration shell
→ bind the shell to the lexical name
→ validate signature-only protocol legality
→ resolve selectors, parameters, annotations, attributes, docs, and locations
→ create owned requirement and parameter descriptors
→ complete and freeze the Protocol descriptor
```

The binding value is the completed `Protocol` object. The declaration does not first create a `Class` and then convert it.

### 4.2 Requirement semantics

A requirement is a declarative obligation record. It has no bytecode, executable block, native function pointer, method-table slot, or inherited stub.

For an instance-side requirement, later conformance asks whether candidate values respond to the same selector with a compatible reflective signature.

For a class-side requirement, later conformance asks whether the candidate class object responds to the same selector with a compatible reflective signature.

The complete algorithm, annotation compatibility, inherited members, recursive protocols, and cache invalidation are specified in Document 10.

### 4.3 Requirements do not participate in inheritance

Declaring:

```phalcom
@protocol
class Hashable {
  hash -> Int
}
```

must not:

- create `Hashable` as a superclass;
- insert `hash` into any candidate class;
- create a stub that throws;
- change `doesNotUnderstand` behavior;
- alter ordinary method lookup;
- add a nominal conformance bit;
- add storage or allocation metadata.

A candidate either already has compatible behavior or does not. Explicit conformance declarations, when introduced, request eager verification and express intent; they do not substitute for structural checking.

### 4.4 Protocols are named but conformance is structural

A `Protocol` has a stable runtime identity for the lifetime of the descriptor. That identity is used for reflection, caches, diagnostics, and ownership. The eventual conformance relation is structural rather than nominal.

Two protocol descriptors may have identical names and requirements while remaining distinct descriptors:

```phalcom
assert(FirstDrawable !== SecondDrawable)
```

Document 10 may define a structural comparison helper, but it must not redefine `==`, `===`, or `hash` away from declaration identity.

### 4.5 Identity, equality, and hash

Normative rules:

- `Protocol#===` uses object identity.
- `Protocol#==` uses the same declaration identity semantics as `===`.
- `Protocol#hash` is derived from stable object identity for the descriptor lifetime.
- A protocol name, owner, or requirement list is never sufficient to recreate identity.
- Re-evaluating a module declaration after reload creates a new descriptor unless the module system explicitly preserves declaration objects; old descriptors remain valid while referenced.
- `ProtocolRequirement` identity is its owner protocol identity plus requirement index, although the runtime may use direct object identity because each owner creates exactly one descriptor at each index.
- `ProtocolParameter` identity is its owner requirement identity plus parameter index.

### 4.6 Module binding and qualified names

A decorator declaration receives its lexical module owner from the compiler. The module binds the source name to the protocol shell before annotation resolution, enabling recursive references.

The first version permits only top-level module-owned protocol declarations. Nested protocol declarations are rejected. A later specification may generalize `owner` to classes or local scopes, but it must preserve identity and qualified-name rules.

`Protocol#qualifiedName` is derived from the owner module's qualified name and the protocol's source name. It is descriptive, not identity.

Manual construction requires an explicit `Module` owner and performs no automatic namespace mutation.

### 4.7 Annotations

Parameter and result annotations are retained as type-expression objects after resolution. Their complete architecture is specified in Document 02.

The following rules apply now:

- an absent source annotation is stored as `None`;
- a present annotation is stored as `Some<Type>` after successful resolution;
- annotations never enter selector identity;
- annotations never trigger automatic value checks;
- annotations never create overloads;
- annotations never change ordinary message dispatch;
- malformed or unresolved annotations in decorator declarations fail metadata resolution;
- manual construction accepts only already-resolved type-expression objects.

### 4.8 Attributes

Protocol-level attributes are separated into:

1. `@protocol`, the declaration-product attribute;
2. compatible retain-tier attributes stored on the descriptor;
3. incompatible declaration-product or behavior-changing attributes, which are rejected.

Requirement-level attributes must be retain-tier metadata. `@class` is normalized into the side field and is not duplicated in `ProtocolRequirement#attributes`.

The compiler must preserve source order for retained attributes.

### 4.9 Documentation and source locations

Documentation and source locations are first-class metadata:

- protocol documentation covers the declaration;
- requirement documentation covers the signature;
- parameter source locations cover the parameter declaration;
- requirement source locations cover the full signature, including attributes and result annotation;
- compiler diagnostics use the narrowest relevant range;
- manual construction may supply `None` for any location or documentation field.

The runtime must not synthesize false source locations for manually constructed descriptors.

### 4.10 Protocol descriptors versus abstract classes

| Property | Protocol descriptor | Abstract class |
|---|---|---|
| Runtime descriptor class | `Protocol` | `Class` |
| Instance layout | None | Yes, possibly inherited |
| Superclass | None in first-version protocol model | Required except root cases |
| Executable instance methods | No | Yes, concrete and abstract obligations |
| Constructors | No | May declare constructors |
| Allocation | Never | Rejected while abstract; allowed for concrete subclasses |
| Code reuse | No | Yes through inheritance |
| Conformance/substitutability | Structural selector/signature check | Nominal subclass relationship plus obligations |
| Method stubs installed | Never | Abstract method metadata belongs to class hierarchy |
| Ordinary dispatch effect | None | Inherited method lookup applies |

### 4.11 Class-side requirements do not become descriptor methods

A source member:

```phalcom
@class
parse(text: String) -> Self
```

is metadata about candidate class objects. It is not a method installed on the protocol descriptor. Therefore:

```phalcom
Parsable.parse("value")
```

follows normal dispatch on the `Parsable` descriptor and ordinarily reaches `doesNotUnderstand` unless `Protocol` itself defines such an API. It never invokes a conformer and never performs type-directed dispatch.

### 4.12 Non-instantiability

A protocol descriptor cannot be used as an allocation origin. The runtime must reject:

```phalcom
Drawable.new()
```

with `ProtocolInstantiationError` and diagnostic code `type.protocol.instantiation`.

The runtime must also reject lower-level attempts to allocate an instance whose behavior/layout descriptor is a `Protocol`. This check is not ordinary type enforcement; it protects the object model from a descriptor that has no layout.

Manual construction of the descriptor itself remains legal because `Protocol.new(...)` is a constructor on the ordinary standard-library class `Protocol`.

### 4.13 Structural conformance dependency

This document establishes the descriptor data required for structural conformance but does not define the complete relation. Until Document 10 is implemented:

- `Protocol#conformedBy` and `Protocol#satisfiedBy` may exist only as explicit `@native` seams that report unsupported operation, or they may remain absent;
- a compiler must not infer structural conformance merely because a protocol descriptor exists;
- tools may inspect requirements but must not invent compatibility rules.

Document 10 must consume the exact selector, side, parameter, result, type-parameter, attribute, documentation, and source-location metadata defined here.

## 5. Object model

### 5.1 Class relationship

`Protocol` is an ordinary standard-library class backed by a trusted bootstrap shell. Protocol descriptors are instances of that class:

```phalcom
Drawable.class === Protocol
Protocol.class === Class
```

The exact metaclass tower follows the general Phalcom object-model specification. This document adds no special metaclass category for individual protocols.

A protocol descriptor does not own an associated instance class or per-protocol metaclass. Its class-side requirements are metadata, not methods in a generated metaclass.

### 5.2 Descriptor graph

A completed descriptor owns the following graph strongly:

```text
Protocol
├── owner Module
├── typeParameters[]
├── requirements[]
│   ├── owner Protocol
│   ├── side
│   ├── selector
│   ├── typeParameters[]
│   ├── parameters[]
│   │   ├── owner ProtocolRequirement
│   │   ├── name / label / kind
│   │   ├── annotation
│   │   ├── attributes[]
│   │   └── sourceLocation
│   ├── resultType
│   ├── attributes[]
│   ├── documentation
│   └── sourceLocation
├── attributes[]
├── documentation
└── sourceLocation
```

Owner back-references create cycles. The GC must trace them normally. No requirement or parameter descriptor may outlive its owner merely because a native side table forgot to trace the edge.

### 5.3 Immutability

After completion:

- descriptor fields are read-only;
- requirement and parameter collections are immutable;
- attributes collections are immutable;
- no public API adds, removes, replaces, or reorders requirements;
- no public API changes names, owners, sides, selectors, annotations, docs, or locations;
- monkey-patching a `Protocol` instance is forbidden if ordinary objects otherwise support per-object method mutation;
- reopening the `Protocol` standard-library class does not mutate already stored metadata.

Open-world mutation of candidate classes is addressed by conformance-cache invalidation in Document 10, not by mutating protocols.

### 5.4 Requirement ownership and indexing

Requirement indexes are zero-based source order across both sides. Side filtering preserves relative source order.

```phalcom
protocol.requirements.at(i).owner === protocol
protocol.requirements.at(i).index == i
```

Parameters are indexed zero-based in source parameter order:

```phalcom
requirement.parameters.at(i).owner === requirement
requirement.parameters.at(i).index == i
```

Index identity allows stable applied views and diagnostics without relying on names.

### 5.5 Selector storage

The requirement stores the canonical `Selector` object produced by the normal selector subsystem. It does not store an ad hoc text string.

The constructor validates that:

- selector arity equals parameter count, excluding any language-defined rest encoding rules;
- labels and parameter drafts agree with the selector's positional/labeled structure;
- getter selectors have no parameters;
- unsupported selector forms are rejected;
- duplicate selector identity on the same side is rejected.

### 5.6 Parameter storage

Each `ProtocolParameter` records:

- source name;
- optional selector label;
- declaration index;
- parameter kind;
- optional resolved type annotation;
- retained attributes;
- optional source location.

The source name is descriptive and is not part of selector identity unless the language's selector grammar explicitly makes it a label.

### 5.7 Type-parameter storage

`Protocol#typeParameters` and `ProtocolRequirement#typeParameters` are immutable collections. Document 03 defines their owner/index identity and validates that they belong to the protocol or requirement respectively.

The bootstrap may temporarily hold unresolved type-parameter metadata while the declaration shell is incomplete. User reflection must never observe the partially initialized state.

### 5.8 Allocation and rooting

Decorator declaration:

- allocates a protocol shell before recursive annotation resolution;
- stores it in the module declaration table and lexical binding as a strong root;
- completes it exactly once;
- makes it visible to user code only after successful module initialization;
- discards or poisons the shell if completion fails so no partial descriptor escapes.

Manual construction:

- allocates a normal `Protocol` instance through `Protocol.new(...)`;
- validates and completes it synchronously;
- is rooted according to ordinary object references and bindings;
- creates no global registry entry unless user code stores it.

### 5.9 Display and string conversion

`Protocol#displayName` returns the unqualified source name. `Protocol#qualifiedName` includes the module owner. `Protocol#toString` returns the qualified name when available.

Display is not identity and must not be used as a cache key without the descriptor identity.

### 5.10 Manual construction validation order

`Protocol.new(...)` validates in this order so diagnostics are deterministic:

1. owner is a module accepted by the first-version model;
2. name is a valid non-empty declaration symbol;
3. collections are immutable or are defensively frozen;
4. protocol attributes are retain-tier and compatible;
5. protocol type-parameter metadata is valid when Document 03 is present;
6. every requirement draft has valid side, selector, annotations, attributes, docs, and source location;
7. selector/parameter shape agrees;
8. no duplicate selector exists on the same side;
9. requirement type-parameter ownership can be established;
10. owned requirement and parameter descriptors are created in source order;
11. fields are assigned and the descriptor is frozen.

No partially completed descriptor is returned on failure.

## 6. Complete standard-library source model

### 6.1 Status of forward references

The following source is normative for the protocol foundation. It assumes the ordinary core reflection names `Attribute`, `AttributeTarget`, `AttributeTier`, `Selector`, `SourceLocation`, `Module`, and `DeclarationIdentity`.

It also uses the forward-referenced typing names `Type`, `TypeParameter`, `TypeParameterSpec`, and `TypeParameterRuntime`. Their exact definitions are established by Documents 02 and 03. The stable protocol API is fixed here, and those later documents are required to define compatible objects under these names.

The source is presented as one conceptual `core/typing/protocol.ph` module. Document 20 may split it into physical files without changing public names or behavior.

### 6.2 Normative Phalcom source

```phalcom
// core/typing/protocol.ph
//
// First-class protocol descriptors. Protocol requirements are metadata only.
// Native anchors protect trusted declaration shells and owned descriptor
// construction; their visible source bodies define the required semantics.

@data
@immutable
@sealed
class ProtocolRequirementSide {
  @variant Instance
  @variant ClassSide
}

@data
@immutable
@sealed
class ProtocolParameterKind {
  @variant Positional
  @variant Labeled
  @variant Rest
}

@data
@immutable
class ProtocolDiagnostic {
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

class ProtocolError is Error {
  const _diagnostic: ProtocolDiagnostic

  @constructor
  new(diagnostic: ProtocolDiagnostic) {
    _diagnostic = diagnostic
  }

  diagnostic -> ProtocolDiagnostic {
    return _diagnostic
  }

  message -> String {
    return _diagnostic.message
  }
}

class InvalidProtocolDeclarationError is ProtocolError {}
class InvalidProtocolMemberError is ProtocolError {}
class InvalidProtocolMetadataError is ProtocolError {}
class ProtocolInstantiationError is ProtocolError {}
class ProtocolMutationError is ProtocolError {}
class ProtocolMetadataAuthorityError is ProtocolError {}

// Ownerless immutable input accepted by Protocol.new(...).
@data
@immutable
class ProtocolParameterDraft {
  const _name: Symbol
  const _label: Option<Symbol>
  const _kind: ProtocolParameterKind
  const _type: Option<Type>
  const _attributes: const List<Attribute>
  const _sourceLocation: Option<SourceLocation>

  @constructor
  new(
    name: Symbol,
    label: Option<Symbol>,
    kind: ProtocolParameterKind,
    type: Option<Type>,
    attributes: const List<Attribute>,
    sourceLocation: Option<SourceLocation>
  ) {
    const frozenAttributes = attributes.freeze

    ProtocolRuntime.validateParameterDraft(
      name: name,
      label: label,
      kind: kind,
      type: type,
      attributes: frozenAttributes,
      sourceLocation: sourceLocation
    )

    _name = name
    _label = label
    _kind = kind
    _type = type
    _attributes = frozenAttributes
    _sourceLocation = sourceLocation
  }

  @class
  positional(name: Symbol, type: Option<Type>) -> ProtocolParameterDraft {
    return ProtocolParameterDraft.new(
      name: name,
      label: None,
      kind: ProtocolParameterKind.Positional,
      type: type,
      attributes: const [],
      sourceLocation: None
    )
  }

  @class
  labeled(
    name: Symbol,
    label: Symbol,
    type: Option<Type>
  ) -> ProtocolParameterDraft {
    return ProtocolParameterDraft.new(
      name: name,
      label: Some.new(label),
      kind: ProtocolParameterKind.Labeled,
      type: type,
      attributes: const [],
      sourceLocation: None
    )
  }
}

// Ownerless immutable input accepted by Protocol.new(...).
@data
@immutable
class ProtocolRequirementDraft {
  const _side: ProtocolRequirementSide
  const _selector: Selector
  const _typeParameters: const List<TypeParameterSpec>
  const _parameters: const List<ProtocolParameterDraft>
  const _resultType: Option<Type>
  const _attributes: const List<Attribute>
  const _documentation: Option<String>
  const _sourceLocation: Option<SourceLocation>

  @constructor
  new(
    side: ProtocolRequirementSide,
    selector: Selector,
    typeParameters: const List<TypeParameterSpec>,
    parameters: const List<ProtocolParameterDraft>,
    resultType: Option<Type>,
    attributes: const List<Attribute>,
    documentation: Option<String>,
    sourceLocation: Option<SourceLocation>
  ) {
    const frozenTypeParameters = typeParameters.freeze
    const frozenParameters = parameters.freeze
    const frozenAttributes = attributes.freeze

    ProtocolRuntime.validateRequirementDraft(
      side: side,
      selector: selector,
      typeParameters: frozenTypeParameters,
      parameters: frozenParameters,
      resultType: resultType,
      attributes: frozenAttributes,
      documentation: documentation,
      sourceLocation: sourceLocation
    )

    _side = side
    _selector = selector
    _typeParameters = frozenTypeParameters
    _parameters = frozenParameters
    _resultType = resultType
    _attributes = frozenAttributes
    _documentation = documentation
    _sourceLocation = sourceLocation
  }

  @class
  instanceMethod(
    selector: Selector,
    parameters: const List<ProtocolParameterDraft>,
    resultType: Option<Type>
  ) -> ProtocolRequirementDraft {
    return ProtocolRequirementDraft.new(
      side: ProtocolRequirementSide.Instance,
      selector: selector,
      typeParameters: const [],
      parameters: parameters,
      resultType: resultType,
      attributes: const [],
      documentation: None,
      sourceLocation: None
    )
  }

  @class
  classMethod(
    selector: Selector,
    parameters: const List<ProtocolParameterDraft>,
    resultType: Option<Type>
  ) -> ProtocolRequirementDraft {
    return ProtocolRequirementDraft.new(
      side: ProtocolRequirementSide.ClassSide,
      selector: selector,
      typeParameters: const [],
      parameters: parameters,
      resultType: resultType,
      attributes: const [],
      documentation: None,
      sourceLocation: None
    )
  }
}

// Unforgeable capability supplied only by ProtocolRuntime while binding owned
// requirement and parameter descriptors. The only constructor is a reserved
// native anchor used once by this trusted core module during bootstrap.
@immutable
class ProtocolConstructionToken {
  @constructor
  @native
  _trustedNew() {}
}

const _PROTOCOL_CONSTRUCTION_TOKEN =
  ProtocolConstructionToken._trustedNew()

@immutable
class ProtocolParameter {
  const _owner: ProtocolRequirement
  const _index: Int
  const _name: Symbol
  const _label: Option<Symbol>
  const _kind: ProtocolParameterKind
  const _type: Option<Type>
  const _attributes: const List<Attribute>
  const _sourceLocation: Option<SourceLocation>

  @constructor
  @native
  _ownedNew(
    token: ProtocolConstructionToken,
    owner: ProtocolRequirement,
    index: Int,
    draft: ProtocolParameterDraft
  ) {
    ProtocolRuntime.requireConstructionAuthority(token)

    if index < 0 {
      throw ProtocolRuntime.invalidMetadata(
        code: "type.protocol.manual.invalid_parameter",
        message: "protocol parameter index must be non-negative",
        details: ProtocolRuntime.details(#index, index),
        sourceLocation: draft.sourceLocation
      )
    }

    _owner = owner
    _index = index
    _name = draft.name
    _label = draft.label
    _kind = draft.kind
    _type = draft.type
    _attributes = draft.attributes
    _sourceLocation = draft.sourceLocation
  }

  owner -> ProtocolRequirement {
    return _owner
  }

  index -> Int {
    return _index
  }

  name -> Symbol {
    return _name
  }

  label -> Option<Symbol> {
    return _label
  }

  kind -> ProtocolParameterKind {
    return _kind
  }

  type -> Option<Type> {
    return _type
  }

  attributes -> const List<Attribute> {
    return _attributes
  }

  sourceLocation -> Option<SourceLocation> {
    return _sourceLocation
  }

  equivalentTo(other: Object) -> Bool {
    if other.is(ProtocolParameter).not {
      return false
    }

    return _owner === other.owner and _index == other.index
  }

  hash -> Int {
    return (_owner.identityHash * 31) + _index.hash
  }
}

@immutable
class ProtocolRequirement {
  const _owner: Protocol
  const _index: Int
  const _side: ProtocolRequirementSide
  const _selector: Selector
  const _typeParameters: const List<TypeParameter>
  const _parameters: const List<ProtocolParameter>
  const _resultType: Option<Type>
  const _attributes: const List<Attribute>
  const _documentation: Option<String>
  const _sourceLocation: Option<SourceLocation>

  @constructor
  @native
  _ownedNew(
    token: ProtocolConstructionToken,
    owner: Protocol,
    index: Int,
    draft: ProtocolRequirementDraft
  ) {
    ProtocolRuntime.requireConstructionAuthority(token)

    if index < 0 {
      throw ProtocolRuntime.invalidMetadata(
        code: "type.protocol.manual.invalid_requirement",
        message: "protocol requirement index must be non-negative",
        details: ProtocolRuntime.details(#index, index),
        sourceLocation: draft.sourceLocation
      )
    }

    _owner = owner
    _index = index
    _side = draft.side
    _selector = draft.selector
    _typeParameters = ProtocolRuntime.bindRequirementTypeParameters(
      token: token,
      owner: self,
      specifications: draft.typeParameters
    )
    _resultType = draft.resultType
    _attributes = draft.attributes
    _documentation = draft.documentation
    _sourceLocation = draft.sourceLocation

    let boundParameters = const []
    let parameterIndex = 0

    while parameterIndex < draft.parameters.size {
      const parameter = ProtocolParameter._ownedNew(
        token: token,
        owner: self,
        index: parameterIndex,
        draft: draft.parameters.at(parameterIndex)
      )

      boundParameters = boundParameters.appending(parameter).freeze
      parameterIndex++
    }

    _parameters = boundParameters
  }

  owner -> Protocol {
    return _owner
  }

  index -> Int {
    return _index
  }

  side -> ProtocolRequirementSide {
    return _side
  }

  selector -> Selector {
    return _selector
  }

  typeParameters -> const List<TypeParameter> {
    return _typeParameters
  }

  parameters -> const List<ProtocolParameter> {
    return _parameters
  }

  resultType -> Option<Type> {
    return _resultType
  }

  attributes -> const List<Attribute> {
    return _attributes
  }

  documentation -> Option<String> {
    return _documentation
  }

  sourceLocation -> Option<SourceLocation> {
    return _sourceLocation
  }

  isInstanceSide -> Bool {
    return _side == ProtocolRequirementSide.Instance
  }

  isClassSide -> Bool {
    return _side == ProtocolRequirementSide.ClassSide
  }

  parameterAt(index: Int) -> Option<ProtocolParameter> {
    if index < 0 or index >= _parameters.size {
      return None
    }

    return Some.new(_parameters.at(index))
  }

  equivalentTo(other: Object) -> Bool {
    if other.is(ProtocolRequirement).not {
      return false
    }

    return _owner === other.owner and _index == other.index
  }

  hash -> Int {
    return (_owner.identityHash * 31) + _index.hash
  }

  toString -> String {
    let prefix = ""
    if self.isClassSide {
      prefix = "class "
    }

    return "\(prefix)\(_selector)"
  }
}

@immutable
class Protocol {
  const _name: Symbol
  const _owner: Module
  const _typeParameters: const List<TypeParameter>
  const _requirements: const List<ProtocolRequirement>
  const _attributes: const List<Attribute>
  const _documentation: Option<String>
  const _sourceLocation: Option<SourceLocation>
  const _declarationIdentity: Option<DeclarationIdentity>

  // Canonical public manual-construction API.
  @constructor
  new(
    name: Symbol,
    owner: Module,
    typeParameters: const List<TypeParameterSpec>,
    requirements: const List<ProtocolRequirementDraft>,
    attributes: const List<Attribute>,
    documentation: Option<String>,
    sourceLocation: Option<SourceLocation>
  ) {
    const frozenTypeParameters = typeParameters.freeze
    const frozenRequirements = requirements.freeze
    const frozenAttributes = attributes.freeze

    ProtocolRuntime.validateManualProtocol(
      name: name,
      owner: owner,
      typeParameters: frozenTypeParameters,
      requirements: frozenRequirements,
      attributes: frozenAttributes,
      documentation: documentation,
      sourceLocation: sourceLocation
    )

    _name = name
    _owner = owner
    _attributes = frozenAttributes
    _documentation = documentation
    _sourceLocation = sourceLocation
    _declarationIdentity = None

    const token = ProtocolRuntime._constructionToken
    _typeParameters = ProtocolRuntime.bindProtocolTypeParameters(
      token: token,
      owner: self,
      specifications: frozenTypeParameters
    )
    _requirements = ProtocolRuntime.bindRequirements(
      token: token,
      owner: self,
      drafts: frozenRequirements
    )
  }

  // VM/compiler-only declaration path. The native implementation may allocate
  // an indexed shell before metadata resolution and complete it exactly once.
  @constructor
  @native
  _declaredNew(
    token: ProtocolConstructionToken,
    declarationIdentity: DeclarationIdentity,
    name: Symbol,
    owner: Module,
    typeParameters: const List<TypeParameterSpec>,
    requirements: const List<ProtocolRequirementDraft>,
    attributes: const List<Attribute>,
    documentation: Option<String>,
    sourceLocation: Option<SourceLocation>
  ) {
    ProtocolRuntime.requireConstructionAuthority(token)

    const frozenTypeParameters = typeParameters.freeze
    const frozenRequirements = requirements.freeze
    const frozenAttributes = attributes.freeze

    ProtocolRuntime.validateDeclaredProtocol(
      declarationIdentity: declarationIdentity,
      name: name,
      owner: owner,
      typeParameters: frozenTypeParameters,
      requirements: frozenRequirements,
      attributes: frozenAttributes,
      documentation: documentation,
      sourceLocation: sourceLocation
    )

    _name = name
    _owner = owner
    _attributes = frozenAttributes
    _documentation = documentation
    _sourceLocation = sourceLocation
    _declarationIdentity = Some.new(declarationIdentity)
    _typeParameters = ProtocolRuntime.bindProtocolTypeParameters(
      token: token,
      owner: self,
      specifications: frozenTypeParameters
    )
    _requirements = ProtocolRuntime.bindRequirements(
      token: token,
      owner: self,
      drafts: frozenRequirements
    )
  }

  name -> Symbol {
    return _name
  }

  owner -> Module {
    return _owner
  }

  typeParameters -> const List<TypeParameter> {
    return _typeParameters
  }

  requirements -> const List<ProtocolRequirement> {
    return _requirements
  }

  attributes -> const List<Attribute> {
    return _attributes
  }

  documentation -> Option<String> {
    return _documentation
  }

  sourceLocation -> Option<SourceLocation> {
    return _sourceLocation
  }

  declarationIdentity -> Option<DeclarationIdentity> {
    return _declarationIdentity
  }

  displayName -> String {
    return _name.toString
  }

  qualifiedName -> String {
    return "\(_owner.qualifiedName).\(_name)"
  }

  isDeclared -> Bool {
    return _declarationIdentity != None
  }

  isGeneric -> Bool {
    return _typeParameters.isEmpty.not
  }

  instanceRequirements -> const List<ProtocolRequirement> {
    return _requirements.select { requirement =>
      requirement.isInstanceSide
    }.freeze
  }

  classRequirements -> const List<ProtocolRequirement> {
    return _requirements.select { requirement =>
      requirement.isClassSide
    }.freeze
  }

  requirementFor(
    selector: Selector,
    side: ProtocolRequirementSide
  ) -> Option<ProtocolRequirement> {
    return _requirements.find { requirement =>
      requirement.side == side and requirement.selector == selector
    }
  }

  requirementsFor(selector: Selector) -> const List<ProtocolRequirement> {
    return _requirements.select { requirement =>
      requirement.selector == selector
    }.freeze
  }

  // Protocol equality is declaration identity, represented by object identity.
  ==(other: Object) -> Bool {
    return self === other
  }

  hash -> Int {
    return self.identityHash
  }

  toString -> String {
    return self.qualifiedName
  }

  // A conventional construction send receives a specific error instead of a
  // generic DNU. Lower-level allocation is rejected by the VM as well.
  new(*arguments: Object) -> Nothing {
    throw ProtocolRuntime.instantiationError(
      protocol: self,
      selector: Selector.method(#new, labels: const []),
      arguments: arguments.freeze,
      sourceLocation: None
    )
  }

  doesNotUnderstand(message: Message) -> Object {
    if message.selector.baseName == #new {
      throw ProtocolRuntime.instantiationError(
        protocol: self,
        selector: message.selector,
        arguments: message.arguments.freeze,
        sourceLocation: message.sourceLocation
      )
    }

    return super.doesNotUnderstand(message)
  }
}

// Runtime representation of the @protocol declaration-product attribute.
@data
@immutable
class ProtocolAttribute is Attribute {
  @constructor
  new() {}

  name -> Symbol {
    return #protocol
  }

  target -> AttributeTarget {
    return AttributeTarget.ClassDeclaration
  }

  tier -> AttributeTier {
    return AttributeTier.DeclarationProduct
  }

  @native
  expand(declaration: ClassDeclaration) -> ProtocolDeclarationPlan {
    return ProtocolRuntime.expand(
      attribute: self,
      declaration: declaration
    )
  }
}

@data
@immutable
class ProtocolDeclarationPlan {
  const _declarationIdentity: DeclarationIdentity
  const _name: Symbol
  const _owner: Module
  const _typeParameters: const List<TypeParameterSpec>
  const _requirements: const List<ProtocolRequirementDraft>
  const _attributes: const List<Attribute>
  const _documentation: Option<String>
  const _sourceLocation: Option<SourceLocation>

  @constructor
  new(
    declarationIdentity: DeclarationIdentity,
    name: Symbol,
    owner: Module,
    typeParameters: const List<TypeParameterSpec>,
    requirements: const List<ProtocolRequirementDraft>,
    attributes: const List<Attribute>,
    documentation: Option<String>,
    sourceLocation: Option<SourceLocation>
  ) {
    _declarationIdentity = declarationIdentity
    _name = name
    _owner = owner
    _typeParameters = typeParameters.freeze
    _requirements = requirements.freeze
    _attributes = attributes.freeze
    _documentation = documentation
    _sourceLocation = sourceLocation
  }
}

class ProtocolRuntime {
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

  // Returns an unforgeable VM capability. The selector is reserved to the core
  // module and cannot be replaced or invoked successfully by user code.
  @class
  @native
  _constructionToken -> ProtocolConstructionToken {
    return _PROTOCOL_CONSTRUCTION_TOKEN
  }

  @class
  @native
  requireConstructionAuthority(token: ProtocolConstructionToken) -> None {
    if token !== self._constructionToken {
      throw ProtocolMetadataAuthorityError.new(
        ProtocolDiagnostic.new(
          code: "type.protocol.metadata_authority",
          message: "trusted protocol metadata construction authority is required",
          details: self.details(),
          sourceLocation: None
        )
      )
    }
  }

  @class
  validateParameterDraft(
    name: Symbol,
    label: Option<Symbol>,
    kind: ProtocolParameterKind,
    type: Option<Type>,
    attributes: const List<Attribute>,
    sourceLocation: Option<SourceLocation>
  ) -> None {
    if name.toString.isEmpty {
      throw self.invalidMetadata(
        code: "type.protocol.manual.invalid_parameter",
        message: "protocol parameter name must not be empty",
        details: self.details(#name, name),
        sourceLocation: sourceLocation
      )
    }

    if kind == ProtocolParameterKind.Labeled and label == None {
      throw self.invalidMetadata(
        code: "type.protocol.manual.invalid_parameter",
        message: "a labeled protocol parameter requires a selector label",
        details: self.details(#name, name),
        sourceLocation: sourceLocation
      )
    }

    if kind != ProtocolParameterKind.Labeled and label != None {
      throw self.invalidMetadata(
        code: "type.protocol.manual.invalid_parameter",
        message: "only a labeled protocol parameter may carry a label",
        details: self.details(#name, name, #label, label.unwrap),
        sourceLocation: sourceLocation
      )
    }

    self.validateRetainedAttributes(
      attributes: attributes,
      ownerKind: #parameter,
      sourceLocation: sourceLocation
    )
  }

  @class
  validateRequirementDraft(
    side: ProtocolRequirementSide,
    selector: Selector,
    typeParameters: const List<TypeParameterSpec>,
    parameters: const List<ProtocolParameterDraft>,
    resultType: Option<Type>,
    attributes: const List<Attribute>,
    documentation: Option<String>,
    sourceLocation: Option<SourceLocation>
  ) -> None {
    self.validateSelectorParameters(
      selector: selector,
      parameters: parameters,
      sourceLocation: sourceLocation
    )

    self.validateRetainedAttributes(
      attributes: attributes,
      ownerKind: #requirement,
      sourceLocation: sourceLocation
    )

    self.validateTypeParameterSpecifications(
      specifications: typeParameters,
      sourceLocation: sourceLocation
    )
  }

  @class
  validateManualProtocol(
    name: Symbol,
    owner: Module,
    typeParameters: const List<TypeParameterSpec>,
    requirements: const List<ProtocolRequirementDraft>,
    attributes: const List<Attribute>,
    documentation: Option<String>,
    sourceLocation: Option<SourceLocation>
  ) -> None {
    self.validateProtocolCommon(
      name: name,
      owner: owner,
      typeParameters: typeParameters,
      requirements: requirements,
      attributes: attributes,
      sourceLocation: sourceLocation,
      declared: false
    )
  }

  @class
  validateDeclaredProtocol(
    declarationIdentity: DeclarationIdentity,
    name: Symbol,
    owner: Module,
    typeParameters: const List<TypeParameterSpec>,
    requirements: const List<ProtocolRequirementDraft>,
    attributes: const List<Attribute>,
    documentation: Option<String>,
    sourceLocation: Option<SourceLocation>
  ) -> None {
    self.validateProtocolCommon(
      name: name,
      owner: owner,
      typeParameters: typeParameters,
      requirements: requirements,
      attributes: attributes,
      sourceLocation: sourceLocation,
      declared: true
    )
  }

  @class
  validateProtocolCommon(
    name: Symbol,
    owner: Module,
    typeParameters: const List<TypeParameterSpec>,
    requirements: const List<ProtocolRequirementDraft>,
    attributes: const List<Attribute>,
    sourceLocation: Option<SourceLocation>,
    declared: Bool
  ) -> None {
    if name.toString.isEmpty {
      throw self.invalidMetadata(
        code: "type.protocol.manual.invalid_name",
        message: "protocol name must not be empty",
        details: self.details(#name, name),
        sourceLocation: sourceLocation
      )
    }

    if owner.is(Module).not {
      throw self.invalidMetadata(
        code: "type.protocol.manual.invalid_owner",
        message: "first-version protocols require a module owner",
        details: self.details(#owner, owner),
        sourceLocation: sourceLocation
      )
    }

    self.validateRetainedAttributes(
      attributes: attributes,
      ownerKind: #protocol,
      sourceLocation: sourceLocation
    )

    self.validateTypeParameterSpecifications(
      specifications: typeParameters,
      sourceLocation: sourceLocation
    )

    let index = 0
    while index < requirements.size {
      const draft = requirements.at(index)
      self.validateRequirementDraft(
        side: draft.side,
        selector: draft.selector,
        typeParameters: draft.typeParameters,
        parameters: draft.parameters,
        resultType: draft.resultType,
        attributes: draft.attributes,
        documentation: draft.documentation,
        sourceLocation: draft.sourceLocation
      )
      index++
    }

    self.validateUniqueRequirements(
      requirements: requirements,
      sourceLocation: sourceLocation
    )
  }

  @class
  validateUniqueRequirements(
    requirements: const List<ProtocolRequirementDraft>,
    sourceLocation: Option<SourceLocation>
  ) -> None {
    let left = 0

    while left < requirements.size {
      let right = left + 1

      while right < requirements.size {
        const first = requirements.at(left)
        const second = requirements.at(right)

        if first.side == second.side and first.selector == second.selector {
          throw self.invalidMetadata(
            code: "type.protocol.duplicate_requirement",
            message: "duplicate protocol requirement on the same side",
            details: self.details(
              #selector, first.selector,
              #side, first.side,
              #firstIndex, left,
              #secondIndex, right
            ),
            sourceLocation: second.sourceLocation.orElse {
              sourceLocation
            }
          )
        }

        right++
      }

      left++
    }
  }

  @class
  validateSelectorParameters(
    selector: Selector,
    parameters: const List<ProtocolParameterDraft>,
    sourceLocation: Option<SourceLocation>
  ) -> None {
    if selector.parameterCount != parameters.size {
      throw self.invalidMetadata(
        code: "type.protocol.manual.selector_parameter_mismatch",
        message: "selector parameter count does not match protocol parameter metadata",
        details: self.details(
          #selector, selector,
          #selectorCount, selector.parameterCount,
          #parameterCount, parameters.size
        ),
        sourceLocation: sourceLocation
      )
    }

    let index = 0
    while index < parameters.size {
      const parameter = parameters.at(index)
      const selectorLabel = selector.labelAt(index)

      if parameter.kind == ProtocolParameterKind.Labeled {
        if selectorLabel == None or selectorLabel.unwrap != parameter.label.unwrap {
          throw self.invalidMetadata(
            code: "type.protocol.manual.selector_parameter_mismatch",
            message: "protocol parameter label does not match selector label",
            details: self.details(
              #selector, selector,
              #index, index,
              #parameterLabel, parameter.label,
              #selectorLabel, selectorLabel
            ),
            sourceLocation: parameter.sourceLocation.orElse {
              sourceLocation
            }
          )
        }
      } else if selectorLabel != None {
        throw self.invalidMetadata(
          code: "type.protocol.manual.selector_parameter_mismatch",
          message: "unlabeled protocol parameter does not match labeled selector position",
          details: self.details(
            #selector, selector,
            #index, index,
            #selectorLabel, selectorLabel
          ),
          sourceLocation: parameter.sourceLocation.orElse {
            sourceLocation
          }
        )
      }

      index++
    }
  }

  @class
  validateRetainedAttributes(
    attributes: const List<Attribute>,
    ownerKind: Symbol,
    sourceLocation: Option<SourceLocation>
  ) -> None {
    attributes.each { attribute =>
      if attribute.tier != AttributeTier.Retain {
        throw self.invalidMetadata(
          code: "type.protocol.invalid_attribute",
          message: "protocol metadata accepts only retain-tier attributes",
          details: self.details(
            #attribute, attribute,
            #tier, attribute.tier,
            #ownerKind, ownerKind
          ),
          sourceLocation: attribute.sourceLocation.orElse {
            sourceLocation
          }
        )
      }
    }
  }

  @class
  @native
  validateTypeParameterSpecifications(
    specifications: const List<TypeParameterSpec>,
    sourceLocation: Option<SourceLocation>
  ) -> None {
    return TypeParameterRuntime.validateSpecifications(
      specifications: specifications,
      sourceLocation: sourceLocation
    )
  }

  @class
  @native
  bindProtocolTypeParameters(
    token: ProtocolConstructionToken,
    owner: Protocol,
    specifications: const List<TypeParameterSpec>
  ) -> const List<TypeParameter> {
    self.requireConstructionAuthority(token)

    // Create exactly one immutable TypeParameter per specification in source
    // order with owner = protocol and index = position.
    return TypeParameterRuntime.bindOwned(
      owner: owner,
      specifications: specifications
    )
  }

  @class
  @native
  bindRequirementTypeParameters(
    token: ProtocolConstructionToken,
    owner: ProtocolRequirement,
    specifications: const List<TypeParameterSpec>
  ) -> const List<TypeParameter> {
    self.requireConstructionAuthority(token)

    // Create exactly one immutable TypeParameter per specification in source
    // order with owner = requirement and index = position.
    return TypeParameterRuntime.bindOwned(
      owner: owner,
      specifications: specifications
    )
  }

  @class
  bindRequirements(
    token: ProtocolConstructionToken,
    owner: Protocol,
    drafts: const List<ProtocolRequirementDraft>
  ) -> const List<ProtocolRequirement> {
    self.requireConstructionAuthority(token)

    let requirements = const []
    let index = 0

    while index < drafts.size {
      requirements = requirements.appending(
        ProtocolRequirement._ownedNew(
          token: token,
          owner: owner,
          index: index,
          draft: drafts.at(index)
        )
      ).freeze
      index++
    }

    return requirements
  }

  @class
  @native
  expand(
    attribute: ProtocolAttribute,
    declaration: ClassDeclaration
  ) -> ProtocolDeclarationPlan {
    // Compiler-authoritative reference semantics:
    // 1. verify the attribute target and declaration-product compatibility;
    // 2. preserve the class-shaped name, generic parameter syntax, docs, attrs,
    //    and source ranges;
    // 3. convert each bodyless method signature to a requirement draft;
    // 4. reject fields, constructors, bodies, nested declarations, and extends;
    // 5. assign a declaration identity and lexical module owner;
    // 6. return a plan without compiling requirement bodies.
    return ProtocolDeclarationPlan.new(
      declarationIdentity: declaration.identity,
      name: declaration.name,
      owner: declaration.module,
      typeParameters: declaration.typeParameterSpecifications,
      requirements: declaration.protocolRequirementDrafts,
      attributes: declaration.retainedAttributes,
      documentation: declaration.documentation,
      sourceLocation: Some.new(declaration.sourceLocation)
    )
  }

  @class
  @native
  declare(plan: ProtocolDeclarationPlan) -> Protocol {
    // Production VMs may allocate and bind a trusted shell before annotations
    // resolve. User code observes only the completed object. This direct body is
    // the non-recursive reference semantics.
    return Protocol._declaredNew(
      token: self._constructionToken,
      declarationIdentity: plan.declarationIdentity,
      name: plan.name,
      owner: plan.owner,
      typeParameters: plan.typeParameters,
      requirements: plan.requirements,
      attributes: plan.attributes,
      documentation: plan.documentation,
      sourceLocation: plan.sourceLocation
    )
  }

  @class
  invalidMetadata(
    code: String,
    message: String,
    details: const Map<Symbol, Object>,
    sourceLocation: Option<SourceLocation>
  ) -> InvalidProtocolMetadataError {
    return InvalidProtocolMetadataError.new(
      ProtocolDiagnostic.new(
        code: code,
        message: message,
        details: details,
        sourceLocation: sourceLocation
      )
    )
  }

  @class
  instantiationError(
    protocol: Protocol,
    selector: Selector,
    arguments: const List<Object>,
    sourceLocation: Option<SourceLocation>
  ) -> ProtocolInstantiationError {
    return ProtocolInstantiationError.new(
      ProtocolDiagnostic.new(
        code: "type.protocol.instantiation",
        message: "protocol \(protocol.qualifiedName) cannot be instantiated",
        details: self.details(
          #protocol, protocol,
          #selector, selector,
          #argumentCount, arguments.size
        ),
        sourceLocation: sourceLocation.orElse {
          protocol.sourceLocation
        }
      )
    )
  }
}
```

### 6.3 Source-model notes

1. `Protocol.new(...)` is intentionally a real public constructor rather than compiler pseudocode.
2. `ProtocolRequirementDraft` and `ProtocolParameterDraft` are public because they are the safe manual-construction boundary.
3. Owned descriptors require the module-private `_PROTOCOL_CONSTRUCTION_TOKEN`. Its `_trustedNew()` constructor and the `_constructionToken` selector are reserved native anchors; a native implementation may instead use VM-only allocation entry points or sealed metadata handles, provided user code cannot forge ownership.
4. `ProtocolRuntime.bindProtocolTypeParameters` and `bindRequirementTypeParameters` return empty lists only as visible bootstrap-floor reference bodies before Document 03 supplies the full implementation. Their selector, ownership, ordering, and immutability contracts are already normative.
5. `ProtocolAttribute.expand` is compiler-facing. A production compiler may implement expansion directly in Rust rather than instantiate compiler AST objects at runtime, but reflected `@protocol` metadata must still be represented by `ProtocolAttribute`.
6. The conventional `new` interception supplies a useful diagnostic. The VM-level allocation guard remains mandatory because constructor selectors are ordinary methods and lower-level allocation paths may bypass `new`.
7. The standard library may add convenience constructors that lower to the full `Protocol.new(...)`; it may not weaken validation or create mutable protocols.

## 7. Compiler and AST requirements

### 7.1 Phase separation

A conforming implementation must distinguish these phases:

1. **Lexing and parsing:** recognize class-shaped syntax, attributes, bodyless signatures, selector structure, annotations, docs, and source ranges. No protocol descriptor is created.
2. **Declaration-product classification:** determine that `@protocol` changes the declaration product and reject incompatible product decorators.
3. **Declaration indexing:** allocate a declaration identity, create or reserve a protocol shell, and bind the lexical name so recursive annotations can resolve.
4. **Protocol legality validation:** reject bodies, fields, constructors, nested declarations, superclass clauses, illegal attributes, and duplicates.
5. **Metadata resolution:** resolve type expressions, attributes, documentation, module references, and source locations into immutable metadata.
6. **Checking:** later checker phases may consume the descriptor; this document performs no implicit conformance check.
7. **Compilation and metadata emission:** emit a protocol declaration record and no requirement bytecode.
8. **Bootstrap or module loading:** allocate/complete the descriptor and publish it after successful initialization.
9. **Runtime reflection:** expose only completed immutable objects.

Conflating these phases is non-conforming when it causes partial descriptors, accidental body compilation, or inconsistent diagnostics.

### 7.2 Parser requirements

The parser must:

- parse `@protocol` through the ordinary attribute syntax;
- parse the following declaration through the shared class-shaped declaration grammar;
- preserve whether a method body is absent, block-bodied, or expression-bodied;
- preserve `@class` as a member modifier before normalization;
- preserve complete selector structure and source ranges;
- preserve parameter names, labels, kinds, annotations, attributes, and ranges;
- preserve method-owned generic parameter syntax;
- preserve result annotation presence or absence;
- preserve documentation independently from ordinary comments;
- accept a signature terminator at newline, semicolon, or closing brace as defined in Section 3.2;
- recover after malformed protocol members sufficiently to report more than one independent diagnostic when safe.

The parser must not decide structural conformance and must not synthesize executable blocks for bodyless members.

### 7.3 AST shape

A conforming AST may use equivalent internal names, but it must preserve at least this information:

```text
ClassDeclarationAst
  attributes: [AttributeAst]
  name: Symbol
  typeParameters: [TypeParameterAst]
  superclassClause: Option<SuperclassAst>
  members: [ClassMemberAst]
  documentation: Option<DocumentationAst>
  sourceRange: SourceRange

MethodDeclarationAst
  attributes: [AttributeAst]
  sideMarker: Instance | ClassSide
  selector: SelectorAst
  typeParameters: [TypeParameterAst]
  parameters: [ParameterAst]
  resultAnnotation: Option<TypeExpressionAst>
  body: Absent | Block(BlockAst) | Expression(ExpressionAst)
  documentation: Option<DocumentationAst>
  sourceRange: SourceRange
```

The preferred representation is an optional or tagged body on the shared method AST rather than a parser-only `ProtocolRequirementAst`. Expansion then converts a valid bodyless method into protocol metadata. This keeps parsing independent of decorator evaluation and allows future `@abstract` bodyless members to reuse the same syntax.

### 7.4 `@protocol` resolution

`@protocol` is a compiler-known declaration-product attribute whose reflected runtime object is `ProtocolAttribute`.

Resolution rules:

- spelling is exactly lowercase `@protocol`;
- zero arguments are accepted;
- arguments are rejected with `type.protocol.invalid_attribute_arguments`;
- the target must be a class-shaped declaration;
- exactly one `@protocol` product attribute may appear;
- duplicate `@protocol` is rejected rather than silently deduplicated;
- `@protocol` may coexist with retain-tier declaration attributes;
- incompatible product decorators are rejected before member lowering.

### 7.5 Expansion and lowering

Expansion produces a `ProtocolDeclarationPlan`, not a `ClassDeclaration` with a flag.

For every valid member, the compiler must:

1. normalize the side to `ProtocolRequirementSide.Instance` or `.ClassSide`;
2. construct the canonical `Selector` using the normal selector subsystem;
3. convert source parameters to `ProtocolParameterDraft` metadata;
4. preserve absent annotations as `None`;
5. resolve present annotations after declaration indexing;
6. retain method-owned type-parameter specifications for Document 03;
7. instantiate and retain compatible attributes in source order;
8. preserve documentation and exact source location;
9. create one `ProtocolRequirementDraft` in source order;
10. emit no executable method.

The lowering must not install a `doesNotUnderstand` stub, abstract method, trait forwarding method, or method-table placeholder.

### 7.6 Declaration indexing and recursive references

A protocol may refer to itself in annotations:

```phalcom
@protocol
class Node {
  parent -> Option<Node>
}
```

The compiler therefore must assign a declaration identity and bind a shell before resolving `Node` in the result annotation.

Required sequence:

```text
index header and name
→ allocate trusted Protocol shell
→ bind Node to shell in module declaration environment
→ resolve member annotation Node to that shell
→ create requirement metadata
→ complete and freeze shell
→ publish module initialization success
```

If resolution or completion fails, user code must not observe a partially initialized descriptor. The module loader must roll back the binding or mark module initialization failed according to the module specification.

### 7.7 Duplicate requirement detection

Duplicates are keyed by:

```text
(requirement side, canonical selector identity)
```

Types, parameter names, result annotations, documentation, and attributes do not distinguish duplicates.

The same selector on opposite sides is legal:

```phalcom
@protocol
class Sized {
  size -> Int

  @class
  size -> Int
}
```

Two same-side declarations that differ only by annotation are illegal because types are not dispatch keys.

### 7.8 Requirement attribute classification

The compiler must classify each member attribute before lowering:

- `@class`: structural side marker; legal and normalized;
- retain-tier: legal and retained;
- declaration-product: illegal on a requirement;
- install, synthesize, wrap, weave, or native-binding tiers: illegal;
- `@constructor`: illegal;
- `@abstract`: illegal and redundant;
- `@requires`, `@ensures`, and other contract-weaving attributes: illegal in the first version because their relationship to structural substitutability is not yet specified.

The diagnostic must point at the offending attribute, not the entire protocol.

### 7.9 Compilation obligations

For a valid protocol declaration, the compiler must not generate:

- instance method bytecode for requirements;
- class-side method bytecode for requirements;
- field descriptors or slot indexes;
- an instance layout;
- constructor tables;
- superclass links;
- ordinary class method-table entries corresponding to requirements;
- inline-cache identities for requirements.

It must generate enough metadata to reconstruct the descriptor exactly, including declaration identity, owner, name, side, selector, parameter metadata, annotations, attributes, docs, and source locations.

### 7.10 Minimum metadata record

Document 17 defines the final encoding. Until then, any encoding is conforming only if it is lossless with respect to this conceptual record:

```text
ProtocolDeclarationRecord {
  formatVersion
  declarationIdentity
  moduleReference
  nameSymbol
  typeParameterRecords[]
  requirementRecords[]
  attributeRecords[]
  documentationRecord?
  sourceLocationRecord?
}

ProtocolRequirementRecord {
  index
  side
  selectorConstant
  typeParameterRecords[]
  parameterRecords[]
  resultTypeRecord?
  attributeRecords[]
  documentationRecord?
  sourceLocationRecord?
}

ProtocolParameterRecord {
  index
  nameSymbol
  labelSymbol?
  kind
  typeRecord?
  attributeRecords[]
  sourceLocationRecord?
}
```

Unknown format versions or malformed indexes must be rejected during load; they must not be repaired heuristically.

### 7.11 Checker obligations at this stage

Before Document 10, the checker may validate declaration legality and metadata well-formedness but must not claim complete protocol conformance.

It may still report:

- duplicate requirements;
- malformed selectors;
- invalid annotations;
- illegal attributes;
- invalid type-parameter syntax once Document 03 is available;
- attempts to instantiate a protocol when statically evident.

It must not use protocol annotations for overload resolution or selector encoding.

### 7.12 Incremental compilation and reload

Incremental compilers must use declaration identity rather than only the qualified name when deciding whether metadata objects may be reused.

A descriptor may be reused only when the module system guarantees that the declaration object itself survives and its metadata is unchanged. Otherwise a new descriptor is created and caches keyed by the old identity must not be silently transferred.

## 8. Interpreter and VM requirements

### 8.1 Trusted standard-library shell

`Protocol` is ordinary Phalcom source backed by a trusted runtime shell so the language can create protocol descriptors while loading the typing library itself.

The minimal shell must provide enough authority to:

- allocate a `Protocol` object without an ordinary user constructor path;
- reserve one-time field initialization for declaration shells;
- allocate owned requirement and parameter descriptors;
- expose an unforgeable construction capability or equivalent VM-only path;
- prevent reflection from observing incomplete objects;
- freeze the descriptors after completion;
- reject malformed metadata.

### 8.2 Interpreter execution

A tree-walking interpreter may implement a protocol declaration directly from the AST instead of emitting bytecode. It must still follow the same phases:

1. index and bind shell;
2. validate members;
3. resolve annotations and attributes;
4. build drafts;
5. complete descriptor;
6. expose it after success.

Interpreting a protocol requirement must never execute a body because a valid requirement has no body.

### 8.3 VM declaration operation

A bytecode VM should use a dedicated declaration operation or metadata-loader path, conceptually:

```text
DECLARE_PROTOCOL protocol_record_index
```

It must not lower to `DECLARE_CLASS` followed by mutation. The descriptor product and allocation invariants differ.

### 8.4 Non-instantiability enforcement

The VM must reject any allocation request whose layout/behavior origin is a `Protocol` descriptor.

This includes:

- conventional `new` sends recognized by `Protocol#new` or DNU;
- direct allocation primitives;
- reflective construction APIs;
- deserialization paths that attempt to use a protocol as a class;
- malformed bytecode or metadata requesting protocol allocation.

The error must be `ProtocolInstantiationError` with code `type.protocol.instantiation`, unless malformed trusted metadata requires a harder bootstrap/security failure under Document 18.

### 8.5 Ordinary dispatch preservation

Protocol metadata must never be consulted by the VM's ordinary send path. Specifically, the VM must not:

- search protocol requirements during message lookup;
- select a method based on parameter or result types;
- inject protocol identities into inline-cache keys;
- wrap conforming values;
- add nominal conformance tables to ordinary lookup;
- consult a protocol merely because a local variable carries that annotation.

Explicit reflection or checker operations may query protocols through dedicated APIs.

### 8.6 Descriptor freezing

After successful completion, writes to protocol metadata must fail with `ProtocolMutationError` or the platform's standard immutable-field error carrying diagnostic code `type.protocol.mutation`.

The VM must prevent:

- field writes through reflection;
- collection mutation through aliased mutable lists;
- replacing owned requirement objects;
- changing owner back-references;
- changing declaration identity;
- adding per-object methods to a protocol descriptor if the object model otherwise supports such mutation.

Inputs from manual construction must be frozen or defensively copied into immutable collections.

### 8.7 GC tracing

The GC must trace all strong edges listed in Section 5.2, including cycles from requirements to protocol and parameters to requirement.

The module binding strongly roots declared protocols while the module is live. Manual protocols are rooted only by ordinary references.

Native caches introduced later must not be the sole owner of a descriptor unless their ownership semantics are explicitly strong. Conformance caches in Document 10 should normally use weak keys or otherwise avoid retaining unloaded modules indefinitely.

### 8.8 Exception and failure paths

On any validation, annotation-resolution, attribute-instantiation, metadata-allocation, or completion failure:

- no partially completed descriptor may become visible to user code;
- module initialization state must be restored or failed consistently;
- temporary native roots must be released;
- owner/index side tables must be discarded;
- a later declaration must not accidentally reuse poisoned metadata;
- source ranges and diagnostic fields must remain available for the thrown error.

### 8.9 Manual construction path

`Protocol.new(...)` is ordinary runtime behavior with validation assistance. It must:

- allocate one descriptor;
- validate before exposing it;
- bind owned descriptors using trusted authority;
- return a fully frozen object;
- leave no global registry entry;
- raise synchronous `InvalidProtocolMetadataError` on malformed input;
- never infer `Module.current` or mutate the module namespace.

### 8.10 Bootstrap sequence

The protocol-specific minimum bootstrap sequence is:

```text
trusted Object/Class/Module/Attribute/Selector shells
→ trusted Protocol/ProtocolRequirement/ProtocolParameter shells
→ register ProtocolAttribute as compiler-known @protocol
→ declaration indexing for typing-library source
→ allocate protocol declaration shells, including future Type
→ resolve annotations and attributes against the full index
→ complete and freeze descriptors
→ replace @native anchors with runtime implementations
→ enable user reflection
```

This avoids circularity: a minimal trusted `Protocol` shell exists before the full typing source is loaded, while the later `Type` protocol is declared as an ordinary `@protocol` once that foundation is operational.

### 8.11 Malformed metadata

Malformed compiler or bytecode metadata is not ordinary user-level protocol invalidity. The loader must reject:

- duplicate indexes;
- missing owner references;
- invalid side tags;
- selector/parameter mismatch;
- duplicate same-side selectors;
- out-of-range constant-pool references;
- invalid attribute tiers;
- mutable collections where immutable metadata is required;
- recursive initialization that exposes an incomplete descriptor;
- unsupported metadata versions.

Document 18 defines whether these failures abort module load, process bootstrap, or the entire VM.

## 9. Reflection and metadata

### 9.1 Protocol reflection surface

A completed `Protocol` exposes at least:

```phalcom
name -> Symbol
owner -> Module
displayName -> String
qualifiedName -> String
typeParameters -> const List<TypeParameter>
requirements -> const List<ProtocolRequirement>
instanceRequirements -> const List<ProtocolRequirement>
classRequirements -> const List<ProtocolRequirement>
attributes -> const List<Attribute>
documentation -> Option<String>
sourceLocation -> Option<SourceLocation>
declarationIdentity -> Option<DeclarationIdentity>
isDeclared -> Bool
isGeneric -> Bool
requirementFor(selector:, side:) -> Option<ProtocolRequirement>
requirementsFor(selector:) -> const List<ProtocolRequirement>
```

Fields exposed through generated getters or explicit methods are read-only.

### 9.2 Requirement reflection surface

`ProtocolRequirement` exposes at least:

```phalcom
owner -> Protocol
index -> Int
side -> ProtocolRequirementSide
selector -> Selector
typeParameters -> const List<TypeParameter>
parameters -> const List<ProtocolParameter>
resultType -> Option<Type>
attributes -> const List<Attribute>
documentation -> Option<String>
sourceLocation -> Option<SourceLocation>
isInstanceSide -> Bool
isClassSide -> Bool
parameterAt(index:) -> Option<ProtocolParameter>
```

It does not expose executable bytecode, a method-table slot, or an invocation API because no executable exists.

### 9.3 Parameter reflection surface

`ProtocolParameter` exposes at least:

```phalcom
owner -> ProtocolRequirement
index -> Int
name -> Symbol
label -> Option<Symbol>
kind -> ProtocolParameterKind
type -> Option<Type>
attributes -> const List<Attribute>
sourceLocation -> Option<SourceLocation>
```

### 9.4 Source versus normalized metadata

Reflection distinguishes source-preserving and normalized facts:

- the canonical selector is normalized;
- requirement side is normalized from the optional `@class` marker;
- absent annotations remain absent;
- attribute instances remain in source order after removing structural markers such as `@class`;
- documentation text is normalized according to the documentation specification while preserving source location;
- source locations point to original source, not generated lowering code;
- `Protocol#attributes` contains normalized retain-tier runtime attributes and excludes structural markers such as `@protocol`; compiler/declaration reflection represents the source marker with `ProtocolAttribute`.

Document 17 defines a broader source-versus-normalized reflection API. This document fixes the minimum observable values.

### 9.5 Manual reflection differences

Manual descriptors report exactly what the caller supplied:

- `declarationIdentity == None`;
- `isDeclared == false`;
- source locations may be `None`;
- documentation may be `None`;
- `attributes` contains only the caller-supplied retain-tier attributes;
- owner and name are explicit constructor inputs.

Tools must not pretend that a manual descriptor corresponds to a source declaration when no declaration identity or source location exists.

### 9.6 Requirement lookup

`requirementFor(selector:, side:)` returns the single exact same-side selector match or `None`.

`requirementsFor(selector:)` may return zero, one, or two elements because the same selector may occur once on each side. Results preserve source order.

Lookup compares canonical selector identity. It does not compare string renderings or type annotations.

### 9.7 Reflection and ordinary dispatch

Reflecting on a protocol is explicit behavior:

```phalcom
Drawable.requirements
```

This must not cause the runtime to modify any candidate class or value. Conversely, sending an ordinary message to a value must not scan `Protocol` objects or requirement lists.

### 9.8 Future applied views

Document 05 introduces `AppliedProtocolRequirement` and applied parameter views. Those are substituted views over the descriptors defined here.

They must preserve:

- owner declaration identity;
- requirement index;
- selector;
- side;
- attributes;
- documentation;
- source location.

Only type-expression fields are substituted.

### 9.9 LSP and documentation tooling

Even before full conformance checking, tools can use protocol metadata to:

- render protocol signatures;
- navigate from a requirement to source;
- distinguish instance and class-side requirements;
- show documentation and attributes;
- report duplicate or illegal requirements;
- list protocols in a module;
- show manual descriptors as runtime-only objects without fake source navigation.

Tooling must not present requirements as inherited methods or runnable definitions.

## 10. Validation and diagnostics

### 10.1 Diagnostic contract

Every diagnostic defined here has:

- a stable symbolic code;
- a primary source range when source exists;
- a human-readable message;
- structured fields listed below;
- zero or more secondary ranges;
- a phase classification.

Messages may improve without compatibility impact. Codes, field meanings, and primary-range selection are compatibility-sensitive.

### 10.2 Declaration diagnostics

| Code | Phase | Primary range | Required fields | Meaning |
|---|---|---|---|---|
| `type.protocol.invalid_target` | expansion | `@protocol` | `targetKind` | `@protocol` is attached to a non-class-shaped declaration |
| `type.protocol.invalid_attribute_arguments` | expansion | `@protocol(...)` argument list | `argumentCount` | `@protocol` received arguments |
| `type.protocol.duplicate_attribute` | expansion | second `@protocol` | `firstLocation`, `secondLocation` | the declaration repeats `@protocol` |
| `type.protocol.conflicting_decorator` | expansion | conflicting decorator | `protocolLocation`, `decorator`, `decoratorTier` | another declaration-product decorator cannot compose with `@protocol` |
| `type.protocol.superclass` | validation | superclass clause | `protocol`, `superclassText` | first-version protocols may not declare `extends` or a superclass |
| `type.protocol.invalid_member` | validation | complete invalid member | `protocol`, `memberKind` | a member category is not legal in a protocol body |
| `type.protocol.executable_body` | validation | body opening brace or expression body | `protocol`, `selector`, `side`, `bodyKind` | a requirement has executable implementation |
| `type.protocol.field` | validation | field declaration | `protocol`, `field`, `isClassSide` | protocols may not declare stored state |
| `type.protocol.constructor` | validation | `@constructor` or constructor declaration | `protocol`, `selector` | protocols may not declare constructors or constructor requirements |
| `type.protocol.nested_declaration` | validation | nested declaration | `protocol`, `nestedKind`, `nestedName` | nested declarations are deferred and illegal in the first version |
| `type.protocol.duplicate_requirement` | validation | second requirement signature | `protocol`, `selector`, `side`, `firstLocation`, `secondLocation` | same side contains the same canonical selector twice |
| `type.protocol.invalid_attribute` | validation | offending attribute | `protocol`, `selector?`, `attribute`, `tier`, `ownerKind` | attribute is not legal retain-tier protocol metadata |
| `type.protocol.invalid_side_marker` | validation | `@class` | `protocol`, `memberKind` | `@class` is not attached to a bodyless protocol requirement |
| `type.protocol.invalid_selector` | validation | selector range | `protocol`, `selectorText`, `reason` | selector form is malformed or unsupported |
| `type.protocol.annotation_resolution` | metadata resolution | failing annotation | `protocol`, `selector?`, `annotationText`, `resolutionReason` | a present type annotation cannot be resolved |

### 10.3 Runtime and manual-construction diagnostics

| Code | Phase | Primary range | Required fields | Meaning |
|---|---|---|---|---|
| `type.protocol.instantiation` | checking or runtime | construction send / allocation site | `protocol`, `selector`, `argumentCount` | a protocol descriptor is used as an allocation origin |
| `type.protocol.mutation` | runtime | attempted write or reflective mutation site | `protocol`, `member` | immutable protocol metadata is being mutated |
| `type.protocol.manual.invalid_name` | runtime construction | supplied location or call site | `name` | manual protocol name is invalid |
| `type.protocol.manual.invalid_owner` | runtime construction | supplied location or call site | `owner`, `expectedOwnerKind` | first-version manual protocol owner is not a module |
| `type.protocol.manual.invalid_requirement` | runtime construction | requirement location or call site | `index`, `reason` | requirement draft is malformed |
| `type.protocol.manual.invalid_parameter` | runtime construction | parameter location or call site | `index?`, `name?`, `reason` | parameter draft is malformed |
| `type.protocol.manual.selector_parameter_mismatch` | runtime construction | parameter/requirement location or call site | `selector`, `index?`, `selectorCount?`, `parameterCount?`, `selectorLabel?`, `parameterLabel?` | canonical selector structure disagrees with parameter metadata |
| `type.protocol.metadata_authority` | runtime/security | attempted trusted constructor call | `operation` | user code attempted to forge owned metadata or a declaration shell |
| `type.protocol.malformed_metadata` | module load/bootstrap | metadata record or module declaration | `recordKind`, `recordIndex`, `reason`, `formatVersion` | compiler/bytecode metadata violates trusted format invariants |
| `type.protocol.incomplete_descriptor` | bootstrap/runtime reflection | declaration or reflection site | `protocol`, `initializationState` | an incomplete shell would have escaped or been reflected upon |

### 10.4 Primary and secondary range rules

- Conflicting decorators: primary range is the later conflicting decorator; the first decorator is secondary.
- Duplicate requirements: primary range is the second signature; the first signature is secondary.
- Executable body: primary range is the body delimiter or expression introducer, not the entire method.
- Invalid attribute: primary range is the attribute.
- Selector/parameter mismatch: primary range is the mismatching parameter; the selector is secondary when source exists.
- Instantiation: primary range is the construction send or allocation call; the protocol declaration may be secondary.
- Manual construction without supplied source locations: primary range is the `Protocol.new(...)` call site when caller-location reflection is available; otherwise the error carries `None` and tools render it as runtime-created metadata.

### 10.5 Deterministic validation order

When one declaration contains multiple errors, implementations should report all independent errors that do not require invalid metadata to continue. Within one member, report the earliest rule in this order:

1. illegal member kind;
2. illegal side marker;
3. constructor marker;
4. executable body;
5. invalid selector;
6. invalid attribute;
7. annotation resolution;
8. duplicate requirement.

A compiler may suppress dependent diagnostics, but it must not emit contradictory alternatives for the same source.

### 10.6 Runtime exception mapping

The standard-library errors map to codes as follows:

| Error class | Primary code family |
|---|---|
| `InvalidProtocolDeclarationError` | `type.protocol.*` declaration/expansion codes |
| `InvalidProtocolMemberError` | member, body, field, constructor, selector, and attribute codes |
| `InvalidProtocolMetadataError` | manual-construction codes |
| `ProtocolInstantiationError` | `type.protocol.instantiation` |
| `ProtocolMutationError` | `type.protocol.mutation` |
| `ProtocolMetadataAuthorityError` | `type.protocol.metadata_authority` |

The error class is not a substitute for the diagnostic code.

## 11. Interaction with earlier specifications

### 11.1 Class declarations

This document reuses the class-shaped declaration grammar but changes the declaration product. Shared parsing does not imply shared runtime object kind.

Any class-declaration rule that assumes every `class` keyword produces a `Class` must be amended to allow declaration-product attributes to produce another trusted descriptor kind.

### 11.2 Attributes and decorators

`@protocol` is a declaration-product attribute. It must participate in the existing attribute ordering and compatibility checks.

Retain-tier attributes remain reflected metadata. Install, wrap, weave, synthesize, and native-binding attributes are rejected on requirements because no implementation exists to transform.

The compiler-specific expansion hook may be implemented in Rust, but the reflected attribute object remains `ProtocolAttribute`.

### 11.3 `@constructor`

`@constructor` is illegal inside a protocol body. Protocol requirements describe ordinary selectors, not allocation authority.

A protocol may require an ordinary class-side factory:

```phalcom
@protocol
class Decodable {
  @class
  decode(bytes: Bytes) -> Self
}
```

That requirement does not mark `decode` as a constructor and does not alter allocation semantics.

### 11.4 Selectors

The canonical selector subsystem remains authoritative. Protocols do not create a parallel selector representation.

Type annotations, generic arguments, and protocol identity never enter selector encoding. Duplicate detection and later conformance use the same canonical selector identity as ordinary dispatch.

### 11.5 Modules

Decorator declarations are automatically bound by the module loader. Manual construction returns a value; an ordinary binding such as `const Drawable = ...` names it.

The first version requires module owners and rejects nested protocol declarations. Qualified names follow module naming rules but do not define identity.

### 11.6 Object model and metaclasses

`Protocol` is a standard-library class. A protocol descriptor is an instance of `Protocol`, not a `Class`, and does not generate a per-protocol metaclass.

Class-side requirements describe selectors expected on candidate class objects. They do not create methods on the protocol descriptor.

### 11.7 Reflection

Protocol reflection extends the existing method/parameter/attribute/source model with non-executable requirement descriptors.

Tools must not coerce a `ProtocolRequirement` into a `Method` because it lacks executable identity and method-table ownership. Shared interfaces may later expose common signature metadata, but object kinds remain distinct.

### 11.8 `@native`

`@native` is legal only on the trusted standard-library source anchors in Section 6. User-authored protocol requirements marked `@native` are rejected because a requirement has no implementation binding.

### 11.9 `@data`, `@sealed`, `@variant`, and `@immutable`

These declaration-product or layout-affecting decorators cannot combine with `@protocol` in the first version.

The protocol descriptor implementation itself may use `@data`, `@sealed`, and `@immutable` on supporting standard-library value classes, as Section 6 does. That is unrelated to decorating a user protocol declaration with them.

### 11.10 `@abstract`

Protocols are not abstract classes. `@abstract` on a protocol declaration or requirement is rejected as a conflicting or invalid attribute.

Document 11 defines abstract-class obligations and their interaction with protocols without changing this boundary.

### 11.11 Contracts

`@requires`, `@ensures`, and invariant-weaving attributes are rejected on first-version requirements. Their executable weaving semantics do not apply to metadata-only signatures, and behavioral-subtyping rules are not yet specified.

A later specification may introduce retained protocol contract metadata, but it must define substitutability and runtime behavior explicitly rather than silently enabling existing weaving.

### 11.12 Experimental Phase 1 typing package

The Phase 1 reference package is informative only. Its sound principles retained here include:

- type metadata must not alter ordinary dispatch;
- descriptors are immutable reflective objects;
- native anchors may enforce trusted construction while visible source remains normative;
- bootstrap may require declaration indexing before annotation resolution.

Its `@protocol` examples that contain concrete class-side bodies are not normative under this document. A signature-only protocol body may not contain executable methods. Descriptor-side utility APIs must be defined outside requirement syntax or through later explicitly specified mechanisms.

### 11.13 No earlier typing-series document

This is Document 01, so it has no dependency on an earlier document in this series. It deliberately establishes the descriptor foundation required by all later typing documents.

## 12. Examples

### 12.1 Minimal protocol

```phalcom
@protocol
class Hashable {
  hash -> Int
}

assert(Hashable.class === Protocol)
assert(Hashable.requirements.size == 1)
assert(Hashable.requirements.first.selector == Selector.getter(#hash))
assert(Hashable.requirements.first.isInstanceSide)
assert(Hashable.requirements.first.resultType.unwrap === Int)
```

### 12.2 Instance and class-side requirements

```phalcom
@protocol
class Codec {
  encode(value: Object) -> Bytes

  @class
  decode(bytes: Bytes) -> Object
}

assert(Codec.instanceRequirements.size == 1)
assert(Codec.classRequirements.size == 1)
assert(Codec.classRequirements.first.selector ==
  Selector.method(#decode, labels: const [#bytes]))
```

The class-side requirement does not make this legal:

```phalcom
Codec.decode(bytes: Bytes.empty)
// ordinary DNU unless Protocol itself defines decode(bytes:)
```

### 12.3 Labels distinguish selectors

```phalcom
@protocol
class IndexedLookup {
  get(index: Int) -> Object
  get(key: String) -> Object
  get(_: Int) -> Object
}

assert(IndexedLookup.requirements.size == 3)
```

### 12.4 Types do not distinguish duplicates

```phalcom
@protocol
class InvalidLookup {
  get(value: Int) -> Object
  get(value: String) -> Object
}
```

Both signatures have the same side and canonical selector. The compiler reports `type.protocol.duplicate_requirement` at the second declaration.

### 12.5 Same selector on opposite sides

```phalcom
@protocol
class Sized {
  size -> Int

  @class
  size -> Int
}

assert(Sized.requirementsFor(Selector.getter(#size)).size == 2)
```

### 12.6 Optional annotations

```phalcom
@protocol
class Sink {
  write(value)
}

const write = Sink.requirements.first
assert(write.parameters.first.type == None)
assert(write.resultType == None)
```

### 12.7 Documentation and passive attributes

```phalcom
/// A source of timestamped events.
@documented
@protocol
class EventSource {
  /// Returns the next event, or None when exhausted.
  @deprecated("Use poll(timeout:) for non-blocking code")
  next -> Option<Event>
}
```

A conforming compiler retains protocol documentation, requirement documentation, the `documented` attribute, the `deprecated` attribute, and source locations.

### 12.8 Manual construction with parameters

```phalcom
const key = ProtocolParameterDraft.labeled(
  name: #key,
  label: #key,
  type: Some.new(String)
)

const Repository = Protocol.new(
  name: #Repository,
  owner: Module.current,
  typeParameters: const [],
  requirements: const [
    ProtocolRequirementDraft.instanceMethod(
      selector: Selector.method(#get, labels: const [#key]),
      parameters: const [key],
      resultType: Some.new(Object)
    )
  ],
  attributes: const [],
  documentation: Some.new("A manually constructed repository protocol."),
  sourceLocation: None
)

assert(Repository.class === Protocol)
assert(Repository.requirements.first.owner === Repository)
assert(Repository.requirements.first.parameters.first.owner ===
  Repository.requirements.first)
```

### 12.9 Decorator/manual behavioral equivalence

```phalcom
@protocol
class DeclaredPingable {
  ping() -> Unit
}

const ManualPingable = Protocol.new(
  name: #ManualPingable,
  owner: Module.current,
  typeParameters: const [],
  requirements: const [
    ProtocolRequirementDraft.instanceMethod(
      selector: Selector.method(#ping, labels: const []),
      parameters: const [],
      resultType: Some.new(Unit)
    )
  ],
  attributes: const [],
  documentation: None,
  sourceLocation: None
)

assert(DeclaredPingable !== ManualPingable)
assert(DeclaredPingable.requirements.first.selector ==
  ManualPingable.requirements.first.selector)
assert(DeclaredPingable.requirements.first.side ==
  ManualPingable.requirements.first.side)
```

### 12.10 No stub injection

```phalcom
@protocol
class Runnable {
  run() -> Unit
}

class Empty {}

const selector = Selector.method(#run, labels: const [])
assert(Empty.methodFor(selector) == None)
```

The declaration of `Runnable` does not mutate `Empty` or any other class.

### 12.11 Protocol versus abstract class

```phalcom
@protocol
class Readable {
  read() -> Bytes
}

@abstract
class BaseReader {
  const _buffer: Bytes

  @constructor
  new(buffer: Bytes) {
    _buffer = buffer
  }

  @abstract
  read() -> Bytes

  remaining -> Int {
    return _buffer.size
  }
}
```

`Readable` is a structural descriptor with no implementation. `BaseReader` is an ordinary class with storage, constructor behavior, a concrete method, and an abstract obligation.

### 12.12 Recursive protocol annotation

```phalcom
@protocol
class TreeNode {
  parent -> Option<TreeNode>
  children -> List<TreeNode>
}
```

The compiler must index and bind the `TreeNode` shell before resolving both annotations.

### 12.13 Generic protocol metadata

```phalcom
@protocol
class Producer<out T> {
  next -> Option<T>
}
```

This document requires `Producer.typeParameters` and the requirement annotation metadata to be retained. Document 03 defines `out T`; Document 10 defines conformance.

### 12.14 Illegal body

```phalcom
@protocol
class InvalidRunnable {
  run() -> Unit {
    System.print("running")
  }
}
```

Diagnostic: `type.protocol.executable_body` at `{`.

### 12.15 Illegal field

```phalcom
@protocol
class InvalidCounter {
  var _count: Int
  count -> Int
}
```

Diagnostic: `type.protocol.field` at `var _count: Int`.

### 12.16 Illegal constructor

```phalcom
@protocol
class InvalidFactory {
  @constructor
  new(value: Object)
}
```

Diagnostic: `type.protocol.constructor` at `@constructor`.

### 12.17 Illegal composition before Document 10

```phalcom
@protocol
class Ordered is Comparable {
  compare(other: Object) -> Int
}
```

Diagnostic: `type.protocol.superclass` at `extends Comparable`.

### 12.18 Illegal behavior-changing requirement attribute

```phalcom
@protocol
class InvalidNative {
  @native
  hash -> Int
}
```

Diagnostic: `type.protocol.invalid_attribute` at `@native`.

### 12.19 Instantiation failure

```phalcom
@protocol
class Drawable {
  draw() -> Unit
}

Drawable.new()
```

Runtime error fields include:

```text
code = type.protocol.instantiation
protocol = Drawable
selector = new()
argumentCount = 0
```

### 12.20 Explicit reflection does not alter dispatch

```phalcom
const requirements = Drawable.requirements
const value = Canvas.new()
value.draw()
```

The `draw()` send uses ordinary lookup on `Canvas`. The VM does not consult `requirements` or `Drawable` during that send.

## 13. Conformance tests

### 13.1 Test categories

A conforming implementation must provide tests in these categories:

- parser and AST preservation;
- declaration-product expansion;
- legality validation;
- descriptor object model;
- manual construction;
- reflection fidelity;
- non-instantiability;
- no-dispatch-effect invariants;
- recursive declaration bootstrap;
- GC ownership and cycles;
- malformed metadata rejection;
- diagnostic codes, fields, and source ranges.

The fixtures below are normative even if the repository uses different physical test directories.

### 13.2 Legality-rule fixture matrix

| Rule | Positive fixture | Negative fixture | Expected result |
|---|---|---|---|
| `@protocol` target is class-shaped | `protocol/minimal.ph` | `protocol/invalid_target_method.ph` | negative: `type.protocol.invalid_target` |
| `@protocol` has zero arguments | `protocol/minimal.ph` | `protocol/attribute_arguments.ph` | negative: `type.protocol.invalid_attribute_arguments` |
| `@protocol` appears once | `protocol/minimal.ph` | `protocol/duplicate_attribute.ph` | negative: `type.protocol.duplicate_attribute` |
| product decorators are exclusive | `protocol/retained_declaration_attribute.ph` | `protocol/conflicting_data.ph` | negative: `type.protocol.conflicting_decorator` |
| protocol has no superclass | `protocol/minimal.ph` | `protocol/superclass.ph` | negative: `type.protocol.superclass` |
| bodyless instance requirement is legal | `protocol/instance_requirement.ph` | `protocol/instance_body.ph` | negative: `type.protocol.executable_body` |
| bodyless class requirement is legal | `protocol/class_requirement.ph` | `protocol/class_body.ph` | negative: `type.protocol.executable_body` |
| fields are illegal | `protocol/getter_requirement.ph` | `protocol/field.ph` | negative: `type.protocol.field` |
| class fields are illegal | `protocol/class_requirement.ph` | `protocol/class_field.ph` | negative: `type.protocol.field` |
| constructors are illegal | `protocol/class_factory_requirement.ph` | `protocol/constructor.ph` | negative: `type.protocol.constructor` |
| nested declarations are illegal | `protocol/minimal.ph` | `protocol/nested_class.ph` | negative: `type.protocol.nested_declaration` |
| same-side selector is unique | `protocol/distinct_labels.ph` | `protocol/duplicate_selector_by_type.ph` | negative: `type.protocol.duplicate_requirement` |
| same selector on opposite sides is legal | `protocol/same_selector_opposite_sides.ph` | `protocol/duplicate_class_selector.ph` | negative: `type.protocol.duplicate_requirement` |
| retain-tier requirement attributes are legal | `protocol/retained_requirement_attribute.ph` | `protocol/native_requirement.ph` | negative: `type.protocol.invalid_attribute` |
| `@class` marks only a requirement | `protocol/class_requirement.ph` | `protocol/class_marker_on_field.ph` | negative: `type.protocol.invalid_side_marker` |
| selector and parameter metadata agree | `protocol/manual_valid_selector.ph` | `protocol/manual_selector_parameter_mismatch.ph` | negative: `type.protocol.manual.selector_parameter_mismatch` |
| missing annotations are legal | `protocol/unannotated.ph` | `protocol/unresolved_annotation.ph` | negative: `type.protocol.annotation_resolution` |
| protocol is non-instantiable | `protocol/reflection_only.ph` | `protocol/instantiate.ph` | negative runtime: `type.protocol.instantiation` |
| metadata is immutable | `protocol/read_reflection.ph` | `protocol/mutate_requirements.ph` | negative runtime: `type.protocol.mutation` |
| manual owner is explicit module | `protocol/manual_valid.ph` | `protocol/manual_invalid_owner.ph` | negative runtime: `type.protocol.manual.invalid_owner` |
| owned metadata cannot be forged | `protocol/manual_valid.ph` | `protocol/forge_requirement_owner.ph` | negative runtime: `type.protocol.metadata_authority` |

### 13.3 Positive acceptance fixture

```phalcom
// protocol/protocol_foundation_acceptance.ph

@protocol
class Renderable {
  /// Draws the receiver into the supplied context.
  @stable
  render(context: RenderContext) -> Unit

  bounds -> Rect

  @class
  from(resource: Resource) -> Self
}

assert(Renderable.class === Protocol)
assert(Renderable.name == #Renderable)
assert(Renderable.owner === Module.current)
assert(Renderable.requirements.size == 3)
assert(Renderable.instanceRequirements.size == 2)
assert(Renderable.classRequirements.size == 1)

const renderSelector = Selector.method(#render, labels: const [#context])
const render = Renderable.requirementFor(
  selector: renderSelector,
  side: ProtocolRequirementSide.Instance
).unwrap

assert(render.owner === Renderable)
assert(render.index == 0)
assert(render.parameters.size == 1)
assert(render.parameters.first.owner === render)
assert(render.parameters.first.index == 0)
assert(render.parameters.first.name == #context)
assert(render.parameters.first.label.unwrap == #context)
assert(render.parameters.first.type.unwrap === RenderContext)
assert(render.resultType.unwrap === Unit)
assert(render.attributes.size == 1)
assert(render.documentation != None)
assert(render.sourceLocation != None)

const bounds = Renderable.requirementFor(
  selector: Selector.getter(#bounds),
  side: ProtocolRequirementSide.Instance
).unwrap

assert(bounds.index == 1)
assert(bounds.parameters.isEmpty)
assert(bounds.resultType.unwrap === Rect)

const from = Renderable.requirementFor(
  selector: Selector.method(#from, labels: const [#resource]),
  side: ProtocolRequirementSide.ClassSide
).unwrap

assert(from.index == 2)
assert(from.isClassSide)
assert(from.parameters.first.type.unwrap === Resource)

// Requirements are metadata, not methods on the protocol descriptor.
assert(Renderable.methodFor(renderSelector) == None)
```

### 13.4 Manual-construction acceptance fixture

```phalcom
// protocol/manual_protocol_acceptance.ph

const parameter = ProtocolParameterDraft.new(
  name: #value,
  label: Some.new(#value),
  kind: ProtocolParameterKind.Labeled,
  type: Some.new(Object),
  attributes: const [],
  sourceLocation: None
)

const draft = ProtocolRequirementDraft.new(
  side: ProtocolRequirementSide.Instance,
  selector: Selector.method(#accept, labels: const [#value]),
  typeParameters: const [],
  parameters: const [parameter],
  resultType: Some.new(Bool),
  attributes: const [],
  documentation: Some.new("Accepts a value."),
  sourceLocation: None
)

const Accepting = Protocol.new(
  name: #Accepting,
  owner: Module.current,
  typeParameters: const [],
  requirements: const [draft],
  attributes: const [],
  documentation: Some.new("Runtime-created protocol."),
  sourceLocation: None
)

assert(Accepting.class === Protocol)
assert(Accepting.isDeclared.not)
assert(Accepting.declarationIdentity == None)
assert(Accepting.requirements.size == 1)
assert(Accepting.requirements.first.owner === Accepting)
assert(Accepting.requirements.first.parameters.first.owner ===
  Accepting.requirements.first)
assert(Accepting.sourceLocation == None)
```

### 13.5 Identity fixture

```phalcom
// protocol/identity.ph

const EmptyA = Protocol.new(
  name: #Empty,
  owner: Module.current,
  typeParameters: const [],
  requirements: const [],
  attributes: const [],
  documentation: None,
  sourceLocation: None
)

const EmptyB = Protocol.new(
  name: #Empty,
  owner: Module.current,
  typeParameters: const [],
  requirements: const [],
  attributes: const [],
  documentation: None,
  sourceLocation: None
)

assert(EmptyA !== EmptyB)
assert(EmptyA != EmptyB)
assert(EmptyA.hash != EmptyB.hash or EmptyA.identityHash != EmptyB.identityHash)
```

The last assertion does not require globally collision-free integer hashes. It requires identity semantics; a test harness should primarily verify map/set behavior with both descriptors as distinct keys.

### 13.6 No method installation fixture

```phalcom
// protocol/no_stub_installation.ph

class Candidate {}

const selector = Selector.method(#run, labels: const [])
assert(Candidate.methodFor(selector) == None)

@protocol
class Runnable {
  run() -> Unit
}

assert(Candidate.methodFor(selector) == None)
assert(Runnable.methodFor(selector) == None)
```

### 13.7 Class-side metadata fixture

```phalcom
// protocol/class_side_requirement.ph

@protocol
class Parseable {
  @class
  parse(text: String) -> Object
}

const selector = Selector.method(#parse, labels: const [#text])
const requirement = Parseable.requirementFor(
  selector: selector,
  side: ProtocolRequirementSide.ClassSide
).unwrap

assert(requirement.isClassSide)
assert(Parseable.requirementFor(
  selector: selector,
  side: ProtocolRequirementSide.Instance
) == None)
```

### 13.8 Parser/AST fixture

Given:

```phalcom
@protocol
class Example {
  @class
  make(value: Int) -> Example
}
```

The parser-level golden representation must preserve:

```text
ClassDeclarationAst(
  attributes = [protocol],
  name = Example,
  superclass = None,
  members = [
    MethodDeclarationAst(
      sideMarker = ClassSide,
      selector = make(value:),
      parameters = [value: Int],
      resultAnnotation = Example,
      body = Absent
    )
  ]
)
```

The AST must not contain an empty block for `make(value:)`.

### 13.9 Negative fixture: executable body

```phalcom
// protocol/negative/executable_body.ph

@protocol
class Bad {
  run() -> Unit {
    return Unit.value
  }
}
```

Expected:

```text
code: type.protocol.executable_body
primary: opening `{` of run()
fields:
  protocol: Bad
  selector: run()
  side: instance
  bodyKind: block
```

### 13.10 Negative fixture: duplicate by annotation only

```phalcom
// protocol/negative/duplicate_by_annotation.ph

@protocol
class BadLookup {
  get(value: Int) -> Object
  get(value: String) -> Object
}
```

Expected:

```text
code: type.protocol.duplicate_requirement
primary: second get(value:) signature
secondary: first get(value:) signature
fields:
  protocol: BadLookup
  selector: get(value:)
  side: instance
```

### 13.11 Negative fixture: conflicting product decorator

```phalcom
// protocol/negative/conflicting_data.ph

@data
@protocol
class BadRecord {
  value -> Int
}
```

Expected code: `type.protocol.conflicting_decorator` at the later conflicting product decorator according to attribute expansion order, with the other decorator as a secondary range.

### 13.12 Negative fixture: illegal native requirement

```phalcom
// protocol/negative/native_requirement.ph

@protocol
class BadHash {
  @native
  hash -> Int
}
```

Expected code: `type.protocol.invalid_attribute` at `@native`, with `tier = NativeBinding` or the equivalent implementation tier.

### 13.13 Negative fixture: instantiation

```phalcom
// protocol/runtime-errors/instantiation.ph

@protocol
class Empty {}

Empty.new()
```

Expected runtime exception:

```text
class: ProtocolInstantiationError
code: type.protocol.instantiation
protocol: Empty
selector: new()
argumentCount: 0
```

### 13.14 Negative fixture: manual selector mismatch

```phalcom
// protocol/runtime-errors/manual_selector_mismatch.ph

const wrong = ProtocolParameterDraft.positional(
  name: #value,
  type: Some.new(Int)
)

Protocol.new(
  name: #Bad,
  owner: Module.current,
  typeParameters: const [],
  requirements: const [
    ProtocolRequirementDraft.instanceMethod(
      selector: Selector.method(#put, labels: const [#value]),
      parameters: const [wrong],
      resultType: Some.new(Unit)
    )
  ],
  attributes: const [],
  documentation: None,
  sourceLocation: None
)
```

Expected runtime exception:

```text
class: InvalidProtocolMetadataError
code: type.protocol.manual.selector_parameter_mismatch
index: 0
selectorLabel: value
parameterLabel: None
```

### 13.15 Recursive bootstrap fixture

```phalcom
// protocol/bootstrap/recursive_annotation.ph

@protocol
class Recursive {
  next -> Option<Recursive>
}

const next = Recursive.requirements.first
assert(next.resultType.unwrap.arguments.first === Recursive)
```

The test must run through both interpreter and compiled/module-load paths when both exist.

### 13.16 Bootstrap failure isolation fixture

A module containing an unresolved annotation:

```phalcom
@protocol
class Broken {
  value -> MissingType
}
```

must fail with `type.protocol.annotation_resolution`. After failure:

- `Broken` is not available through normal module import;
- no incomplete descriptor is returned through reflection;
- reloading a corrected module succeeds with a fresh declaration identity;
- temporary shells and metadata are collectible.

### 13.17 GC cycle fixture

Construct a manual protocol, retain only one parameter descriptor, force GC, and verify its owner chain remains valid:

```phalcom
let protocol = makeManualProtocol()
const parameter = protocol.requirements.first.parameters.first

protocol = None
GC.collect()

assert(parameter.owner.owner.name == #Manual)
```

Then release `parameter`, force GC, and verify a weak reference to the protocol clears. The test must not rely on a native metadata side table as an untraced owner.

### 13.18 Mutation fixture

Attempts to mutate any of these must fail:

```phalcom
protocol._name = #Other
protocol.requirements.add(requirement)
protocol.requirements.first._selector = otherSelector
protocol.requirements.first.parameters.first._name = #other
```

The exact surface of reflective writes follows the object model. Every reachable path must preserve immutability and report `type.protocol.mutation` or the standard immutable-field error carrying that code.

### 13.19 Malformed metadata fixtures

The loader test suite must synthesize or inject records with:

- duplicate requirement index;
- invalid side tag;
- selector count mismatch;
- missing owner module;
- invalid type constant reference;
- non-retain requirement attribute;
- duplicate same-side selector;
- unsupported format version;
- attempted reflection of an incomplete shell.

Each must fail deterministically with `type.protocol.malformed_metadata` or `type.protocol.incomplete_descriptor`, never panic, execute arbitrary code, or publish a partial descriptor.

### 13.20 Cross-engine acceptance

Where Phalcom provides an AST interpreter, bytecode VM, bootstrap interpreter, and optimized runtime, the observable results of all positive fixtures must agree on:

- descriptor class and identity behavior;
- requirement order and indexes;
- selector identity;
- annotation presence;
- docs, attributes, and locations;
- exception classes and diagnostic codes;
- non-instantiability;
- no method installation.

## 14. Native implementation latitude

### 14.1 General rule

Native code is permitted for authority, bootstrap, GC integration, integrity, or performance. It is not permitted to redefine the visible object model.

The Phalcom source remains the behavioral contract.

### 14.2 Operations that may be native

A conforming runtime may implement these operations natively:

- `ProtocolAttribute.expand`;
- allocation and one-time completion of recursive declaration shells;
- creation and verification of the construction capability;
- owned `ProtocolRequirement` and `ProtocolParameter` construction;
- binding of protocol-owned and requirement-owned type parameters;
- descriptor freezing and mutation prevention;
- compiler metadata decoding and validation;
- source-location attachment;
- fast exact-selector lookup tables inside a protocol;
- module-root registration and rollback;
- VM-level allocation rejection for protocol descriptors.

### 14.3 Operations that should remain ordinary source when practical

The following semantics are straightforward standard-library behavior and should remain visible even if optimized:

- `displayName`, `qualifiedName`, and `toString`;
- `isDeclared` and `isGeneric`;
- filtering instance and class-side requirements;
- exact requirement lookup;
- owner/index equivalence helpers;
- identity-based equality and hash contract;
- public draft constructors and their validation surface;
- construction of diagnostic objects.

### 14.4 Native equivalence obligations

A native implementation must preserve:

- exact public selectors and labels;
- exception classes and diagnostic codes;
- deterministic validation order;
- source-range fidelity;
- requirement and parameter source order;
- owner/index identity;
- immutable collection behavior;
- identity-based protocol equality and hashing;
- absence versus presence of annotations;
- no executable requirement bodies;
- no effect on ordinary dispatch;
- no exposure of partial shells;
- correct GC tracing of all descriptor edges.

### 14.5 Allowed internal representations

The VM may use:

- compact tagged enums for side and parameter kind;
- contiguous immutable arrays for requirements and parameters;
- interned symbols and selectors;
- hidden declaration IDs;
- one-time initialization cells for shells;
- side-indexed hash tables for requirement lookup;
- compressed source-location tables;
- native handles instead of a literal user-visible construction-token object.

These are unobservable except through performance and memory use.

### 14.6 Forbidden native divergences

The VM must not:

- model protocols as ordinary classes with hidden flags visible through class APIs;
- allocate instances of a protocol;
- compile requirement bodies;
- treat type annotations as dispatch keys;
- insert requirement stubs into candidate classes;
- infer nominal conformance from descriptor identity;
- merge separately constructed equivalent protocols;
- intern manual protocols by name and requirement list;
- drop documentation, attributes, parameter names, labels, or locations that reflection promises;
- return mutable metadata collections;
- let a native side table outlive or lose track of GC ownership;
- permit user code to invoke trusted constructors successfully.

### 14.7 Performance expectations

Exact requirement lookup should be implementable in expected constant time after descriptor completion, but source-order iteration remains normative. A runtime may lazily construct lookup indexes as long as it does not mutate observable descriptor state or retain unloaded modules incorrectly.

Conformance caches are not part of this document and must wait for the invalidation rules in Document 10.

## 15. Non-goals and deferred work

### 15.1 Default implementations, traits, and mixins

Protocols do not provide code reuse. Default bodies, conflict resolution, method copying, explicit delegation, and trait composition require a separate future design.

### 15.2 Protocol inheritance and composition

`extends`, protocol inclusion, intersections, and composed protocol descriptors are deferred to Documents 10 and 16. The first version rejects superclass/composition clauses rather than assigning provisional semantics.

### 15.3 Structural conformance algorithm

Selector matching, parameter contravariance, result covariance, generic substitution, inherited members, recursive protocols, explicit conformance declarations, and cache invalidation are deferred to Document 10.

### 15.4 Generic semantics

This document stores protocol and requirement type-parameter metadata. Invariance defaults, `in`/`out`, bounds, finite constraints, owner/index identity, cycles, and defaults are defined in Document 03.

### 15.5 Applied protocols

`Protocol<A>`, reserved angle application, applied requirement views, and substitution are deferred to Documents 04 and 05.

### 15.6 Special types and `Self`

The meaning of `Any`, `Dynamic`, `Nothing`, `Self`, `Unit`, absent annotations, and consistency relations is deferred to Document 07. This document only preserves source annotation presence and object references.

### 15.7 Abstract classes

Abstract class declarations and obligations are deferred to Document 11. This document only fixes their conceptual distinction from protocols.

### 15.8 Runtime automatic validation

Protocols do not automatically validate assignments, arguments, fields, returns, collection elements, or constructor results. Runtime checks are explicit library or reflection operations.

### 15.9 Type-directed dispatch

Protocols and their annotations never alter selector encoding, overload resolution, method selection, inline-cache identity, or DNU behavior.

### 15.10 Nested and local protocols

The first version supports module-owned top-level protocols only. Nested class-owned or local protocols require a later owner and lifetime design.

### 15.11 Mutable or open protocol descriptors

Protocols cannot be reopened, extended in place, or monkey-patched. Future protocol composition creates new descriptor views rather than mutating existing identities.

### 15.12 Protocol constructor requirements

A protocol cannot require that a class expose a method marked `@constructor`. It may describe ordinary class-side factory selectors. Constructor obligation semantics would entangle structural capability with allocation policy and are not part of the first version.

### 15.13 Contract metadata on requirements

Behavioral contracts on protocol requirements are deferred. Existing executable contract decorators are rejected rather than silently reinterpreted as passive metadata.

### 15.14 Serialization identity

This document does not define stable cross-process protocol identity. Metadata may serialize declaration references, but deserialization and module versioning are specified in Document 17.

### 15.15 Automatic nominal declarations

A class does not gain conformance merely by naming a protocol. Explicit declarations introduced later express intent and request validation; structural verification remains authoritative.

## 16. Normative invariants

A conforming implementation must preserve every invariant below.

1. `@protocol class Name { ... }` binds `Name` to a `Protocol`, not a `Class`.
2. A protocol descriptor has no instance layout, superclass, constructor table, or per-protocol metaclass.
3. `Protocol` itself is an ordinary trusted standard-library class.
4. Every valid protocol member is a signature-only requirement.
5. No protocol requirement owns executable bytecode or a native function.
6. Requirement bodies are rejected; they are never silently ignored or compiled.
7. Instance-side and class-side requirements are both supported from the first version.
8. `@class` normalizes requirement side and does not install a descriptor method.
9. Requirements never install stubs or behavior into candidate classes.
10. Protocols do not participate in inheritance or code reuse.
11. Selector identity is the ordinary canonical selector, including arity, positional structure, and labels.
12. Type annotations never enter selector identity or ordinary dispatch.
13. Duplicate requirements are keyed by side plus canonical selector.
14. The same selector may appear once per side.
15. Missing annotations remain reflectively absent.
16. Present annotations are resolved type-expression objects and are never automatic runtime guards.
17. Requirement and parameter metadata preserve source order.
18. Requirement identity is owner protocol identity plus declaration index.
19. Parameter identity is owner requirement identity plus declaration index.
20. Protocol equality and hashing use declaration object identity, not name or structural content.
21. Separately constructed equivalent protocols remain distinct objects.
22. Decorator and manual construction produce the same behavioral descriptor kind.
23. Manual construction requires an explicit module owner and does not mutate module bindings.
24. Decorator construction derives owner, binding, declaration identity, documentation, and locations from the compiler.
25. Manual construction creates a fresh identity on every successful call.
26. Protocol, requirement, parameter, attribute, and metadata collections are immutable after completion.
27. User code cannot forge owned requirement or parameter descriptors.
28. Recursive declaration shells are invisible until complete.
29. Failed completion publishes no partial descriptor.
30. The GC traces protocol-owner cycles and all reflected metadata.
31. A protocol descriptor cannot allocate values through any path.
32. Instantiation failure reports `type.protocol.instantiation`.
33. Protocol metadata is never consulted by ordinary message lookup.
34. Protocol identities never become inline-cache dispatch keys unless an explicit reflective API is being called on the descriptor itself.
35. Retain-tier attributes are preserved; behavior-changing requirement attributes are rejected.
36. Protocol declaration-product decorators are mutually exclusive in the first version.
37. `@constructor`, `@abstract`, `@native`, executable contracts, fields, nested declarations, and superclass clauses are illegal in a protocol body.
38. Reflection exposes complete selectors, side, parameters, annotations, type parameters, attributes, docs, and source locations.
39. Reflection does not misrepresent a requirement as an executable `Method`.
40. Native implementations preserve the visible source contract exactly.
41. Malformed trusted metadata is rejected rather than repaired heuristically.
42. Later typing documents may refine the meaning of stored type metadata but may not retroactively make protocols affect ordinary dispatch.
43. Later protocol composition creates new descriptors or views and does not mutate existing protocol identities.
44. Structural conformance remains a later explicit relation and is not implied by the existence of this descriptor foundation.

---

**End of Document 01.** The next specification is `02-type-expression-foundation.md`, which defines `Type` as a protocol, makes class and protocol objects valid type expressions, establishes `TypeDescriptor`, and reserves the public `Type.currentApplication` surface without altering the protocol invariants above.

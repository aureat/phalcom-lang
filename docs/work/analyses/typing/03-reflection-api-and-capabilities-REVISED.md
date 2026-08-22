# 03 — User-Facing Typing Reflection API and Capabilities

**Date:** 2026-08-22  
**Status:** Ratified API architecture and implementation specification  
**Authority:** Normative runtime reflection behavior and semantic-query projection. This document does not redefine the canonical generic/type calculus, metadata representation, or parser grammar.  
**Primary owners:** `phalcom-core` universe/runtime reflection, `phalcom-semantic` read-only semantic query facade, metadata/runtime registry from Spec 02  
**Dependencies:** [01 — Implementation Architecture](01-implementation-architecture.md), [01.5 — Canonical Generic Type Semantics and Declaration Model](01.5-canonical-generic-type-semantics-and-declaration-model.md), [02 — Runtime Reification, Semantic Metadata, and Artifact Contract](02-runtime-reification-and-metadata.md)  
**Syntax dependency:** [04 — User-Facing Type Syntax and Lowering](04-user-facing-type-syntax-and-lowering.md) defines how source expressions produce the semantic forms reflected here; Spec 03 does not block parser implementation.  
**Advanced dependency:** revised Spec 05 owns effects, totality, contracts-as-proof-input, proof-result semantics, and advanced kind/row domains; this document reserves projection seams without freezing those payloads early.

---

## 0. Revision contract

This revision replaces the earlier Spec 03. Its purpose is not to expand reflection indiscriminately. Its purpose is to make runtime reflection a **lazy, explicit projection** of the canonical semantic system defined by Specs 01 and 01.5 and transported by Spec 02.

The central rule is:

> **Static semantics remain compiler-owned. Runtime reflection may observe or explicitly manipulate validated semantic forms, but runtime objects never become the authority that defines those semantics.**

This revision makes the following corrections to the previous document:

1. it consumes the complete generic model from Spec 01.5, including type lambdas, signature-owned constraints, declaration-site variance, generic inheritance templates, owner-relative `Self`, and lazy specialization;
2. it consumes the revised Spec 02 runtime architecture: immutable loaded metadata, a VM-owned `RuntimeTypingRegistry`, bounded context overlays, one boxed `Object::Typing` payload family, weak synthetic-descriptor caches, and an external method-to-semantic side table;
3. it removes the old `Object::SemanticDescriptor` assumption;
4. it replaces `Kind` / `ArrowKind` descriptor class names with `KindDescriptor` / `FunctionKind`, keeping `Type` as the atomic-kind singleton;
5. it replaces `displayName` with the ratified `.display` vocabulary and kind `.parameters` with `.arguments`;
6. it adds `.remainingParameters` and `.subtypeOf` to the common TypeForm reflection vocabulary;
7. it removes canonical `TypeParameter.default` and `TypeParameter.upperBound`; constraints are derived from the owning generic signature;
8. it replaces status-plus-optional-payload result objects with **sealed variant classes**;
9. it keeps cancellation and budget exhaustion distinct from `Blocked` and `Unknown`;
10. it introduces a dedicated `MemberLookupResult` rather than abusing `TypeRelationResult` for member queries;
11. it removes public `normalize`, `substitute`, and `applyKind` from the core runtime API; canonicalization and specialization are semantic implementation details unless a future feature demonstrates a user need;
12. it adds explicit, bounded runtime `matches` and `validate` operations for user-defined typed-dispatch/reflection libraries without adding per-instance generic tokens;
13. it changes world-mutation semantics: pinned static semantic queries remain valid as statements about their semantic snapshot; only operations crossing into the live runtime world require world reconciliation;
14. it places the public typing reflection package under the existing universe `reflection` package rather than inventing a parallel top-level universe package;
15. it makes indexed/lazy accessors first-class performance APIs so inspecting one parameter or type argument does not allocate an entire reflected object graph.

Where this document conflicts with the earlier Spec 03 on these points, this revision supersedes it.

---

# 1. Scope and semantic boundary

## 1.1 Questions owned by this specification

This document answers six questions:

1. Which runtime values represent canonical type forms and kinds?
2. How do existing class objects participate in the TypeForm role without being wrapped or reclassified?
3. How are generic declarations, type parameters, type lambdas, callable signatures, fields, and source type uses reflected?
4. Which operations are pure structural observation, bounded semantic relations, source/tooling queries, or live runtime operations?
5. Which capability and result types make unavailable, invalid, dynamic, blocked, cancelled, budget-exhausted, and internal states explicit?
6. How can runtime libraries perform type-aware inspection or validation without changing Phalcom's ordinary dispatch, object layout, allocation, or instance identity?

## 1.2 What this document does not own

This specification does **not** own:

- canonical type/kind identity or normalization rules — Spec 01.5;
- generic application, variance, constraints, type-lambda beta reduction, generic inheritance, `Self`, or method inference — Spec 01.5;
- durable metadata layout, validation, profiles, registry ownership, weak-cache mechanics, or artifact carriage — Spec 02;
- source grammar — Spec 04;
- advanced row/effect/proof semantics — revised Spec 05;
- selector identity, ordinary runtime dispatch, DNU, access-control rules, allocation semantics, or metaclass wiring — existing runtime specifications and implementation.

## 1.3 Non-goals

The following are explicitly rejected:

- no `Type.currentApplication`;
- no hidden ambient generic application in call frames, fibers, native re-entry, or thread-local state;
- no forwarding `List<Int>.new(...)` as if `List<Int>` were a new runtime class;
- no specialized runtime class/metaclass for generic applications;
- no per-instance generic argument token;
- no `typeOf(value)` operation that guesses erased static generic information from an arbitrary live value;
- no type metadata in selector identity, method dictionary keys, inline-cache keys, field layout, or ordinary allocation;
- no public constructor accepting raw metadata IDs, `TypeId`, `KindId`, numeric node IDs, or unchecked descriptor payloads;
- no reflection API that bypasses existing private/protected authority;
- no result API that turns `DynamicBoundary`, `Blocked`, `Cancelled`, `BudgetExceeded`, proof-unknown, or metadata unavailability into success;
- no VM objects inside `phalcom-semantic`;
- no routing LSP formal queries through runtime descriptor objects;
- no requirement that a type be heap-reified merely because the compiler knows it.

---

# 2. Repository-grounded current state

This section records the current `main` implementation at repository commit `a43f26e0ddd6b1d6e37ddf7a0b9588769bb41f3e`. These are observations, not claims that the target reflection architecture already exists.

## 2.1 `Behavior` reflection is currently minimal

`phalcom-core/core/universe/src/object/behavior.ph:1-7` currently reopens `Behavior` only for attribute reflection:

```phalcom
class Behavior {
  attributes { self._$attributes }
  attributesOfType(_ cls) { self._$attributes.filter |a| { a.is(cls) } }
}
```

Therefore type-form observation on class objects is additive. The implementation must not replace or fork the existing `Behavior` semantics.

## 2.2 Method reflection already has an authority-preserving runtime boundary

`phalcom-core/src/primitive/method.rs:1-105` implements reified `Method` behavior including:

- direct construction prohibition;
- `Method#invokeOn`;
- `Method#bind`;
- `Method#selector`;
- `Method#holder`.

`Method#invokeOn` authorizes access before activating the captured method. This is the correct precedent for any future typing-reflection bridge involving live invocation: typing metadata cannot become an alternative access path.

The runtime `MethodObject` itself contains runtime calling signature, holder, visibility, contracts, and attributes, but no complete static generic signature. This is intentional and aligns with Spec 01.5 §14.5 and Spec 02 §20.

## 2.3 Runtime member reflection is live-world reflection

`phalcom-core/src/primitive/object.rs:160-280` implements ordinary runtime:

- `perform`;
- `respondsTo`;
- `methodFor`;
- `doesNotUnderstand`.

These operations query the **live runtime class hierarchy and method tables**. `respondsTo` and `methodFor` also enforce current method-access authorization.

A static semantic query such as “does declared type `T` have selector `s`?” is therefore a different operation. Spec 03 must preserve that distinction instead of redefining `respondsTo` as a type-checker query.

## 2.4 Static relation APIs are currently not rich enough for public reflection

`phalcom-semantic/src/types/relation.rs:1-170` currently exposes:

- boolean `is_subtype`;
- coarse `Assignability::{Assignable, Refuted, Uncertain}`;
- hard-coded invariant applied-generic subtyping by identical origin/argument list.

Spec 01 is already responsible for replacing these with bounded reasoned outcomes. Spec 03 must project the **new** relation API; it must not build a second runtime relation engine around today's boolean helper.

## 2.5 Member and callable surfaces currently duplicate semantic data

`phalcom-semantic/src/surface.rs:1-80` currently stores several overlapping maps:

```rust
fields: HashMap<String, TypeKnowledge>,
field_ids: HashMap<FieldId, TypeKnowledge>,
callables: HashMap<CallableId, TypeKnowledge>,
callable_signatures: HashMap<Selector, CallableSignature>,
```

`phalcom-semantic/src/dispatch.rs:25-67` separately defines a materialized `CallableSignature`, and `resolve_dispatch_on_owner` clones that signature when returning a result.

Spec 01.5 requires convergence toward one `CallableId -> CallableSemanticSignature` registry and selector surfaces mapping to IDs, with specialized lookup represented by a small `(CallableId, environment)` view. Reflection must consume that canonical model rather than making the current duplication permanent.

## 2.6 Current snapshots are immutable values but not yet the final SemanticDb publication model

`phalcom-semantic/src/snapshot.rs:1-35` contains an immutable `SemanticSnapshot` with generation, `Arc<TypeStore>`, sources, declaration surfaces, dispatch resolver, declaration table, hierarchy, diagnostics, and semantic graph.

Spec 01 is being implemented to establish the long-lived compiler-owned database, stamped identities, explicit outcomes, cancellation, and publication rules. The reflection facade defined here must sit on top of whatever published-snapshot API Spec 01 delivers. It must not own independent caches, solve relations separately, or retain mutable checker state.

## 2.7 The universe already has a reflection package

`phalcom-core/core/universe/src/package.ph:1-18` exposes and imports `.reflection` alongside object, scalar, callable, collections, errors, and concurrency.

`phalcom-core/core/universe/src/reflection/package.ph:1-22` already owns first-class reflection modules for modules, projects, exports, identities, selectors, messages, and attributes.

`phalcom-modules/src/builtin.rs:1-280` explicitly describes every builtin universe package/module and enumerates the children of `reflection`.

Therefore the typing reflection surface belongs under:

```text
universe.reflection.typing
```

not as a second unrelated top-level universe package.

## 2.8 Runtime core classes are explicitly wired

`phalcom-core/src/universe/core_classes.rs:1-260` allocates and wires the kernel class/metaclass graph explicitly and then adds ordinary core classes through the existing parallel metaclass rule.

This creates an important implementation constraint: adding dozens of typing descriptor classes to the fixed `CoreClasses` struct would permanently enlarge bootstrap state. Revised Spec 02 already permits `Object::Typing` to carry its runtime class handle directly. Therefore source-defined typing descriptor classes should be preferred; only types genuinely required before universe source materialization may be added to fixed bootstrap structures.

## 2.9 Existing reflected objects demonstrate that tiny source shells plus native payloads are viable

The current `reflection/selector.ph` is effectively a documentation/source declaration shell, while its native runtime representation and primitives live in Rust. Existing reflection modules use this split repeatedly.

The typing package should follow the same principle: source classes define normal object-model identity and ergonomic pure methods; native primitives only bridge validated compact payloads to the runtime registry or compiler-owned semantic query products.

---

# 3. Non-negotiable reflection laws

## 3.1 Runtime class and static TypeForm remain different questions

For a runtime value `v`:

```text
v.class
```

answers the live runtime class used for ordinary dispatch.

A static judgment:

```text
v : T
```

answers a compiler semantic fact.

A reflected type-form value:

```text
T :: K
```

represents a semantic form classified by a kind.

No reflection method collapses these axes.

## 3.2 Reifying a nominal form returns the existing class object

If the semantic form is the nominal declaration `Int`, reification yields the existing class object `Int`.

There is no separate `NominalType(Int)` heap wrapper.

Consequences:

```phalcom
Typing.current.typeOfDeclaration(Int).value === Int
```

when the query succeeds and metadata is available.

The semantic category “nominal type” remains real in the compiler and metadata schema; it simply does not require an allocated runtime descriptor object.

## 3.3 Synthetic TypeForms are ordinary immutable runtime objects

Applied, union, tuple, record, callable, type-lambda, special, `Self`, parameter, and signature-view forms may require synthetic runtime objects.

They are:

- immutable;
- VM/context-local in identity;
- backed by compact validated handles;
- canonical while their weak cache entry remains live;
- semantically comparable by canonical meaning, not by object identity.

`===` therefore remains runtime object identity. Semantic equivalence is `equivalentTo` / `TypingContext.equivalent`.

## 3.4 Descriptor existence has no effect on execution

Creating or collecting a descriptor cannot:

- create or remove a class;
- change a superclass/metaclass edge;
- install or remove a method;
- bump runtime dispatch world version merely because of descriptor memoization;
- alter object layout;
- add per-instance fields;
- change selector encoding;
- modify an existing value's `class`;
- specialize executable code observably.

## 3.5 Reflection is demand-driven

If a compiled program never asks for a type as a runtime value and never uses typing reflection, ordinary execution allocates **zero typing descriptors**.

Reifying a root such as:

```text
Map<String, List<Option<Int>>>
```

allocates at most the root descriptor initially. Its children remain metadata handles until queried.

## 3.6 No runtime value can manufacture semantic authority

A runtime-created TypeForm may be valid and useful for relation/validation APIs. It does not become:

- a compiler declaration;
- a source type annotation;
- a proof that a live value has that type;
- a token that changes dispatch;
- a declaration interface;
- a proof artifact.

Authority/provenance remains attached to compiler/native metadata and semantic query results.

## 3.7 Reflection cannot recover erased generic arguments

For an ordinary `List` instance, runtime class identity proves at most what the runtime representation actually knows:

```text
list.class == List
```

It does **not** prove:

```text
list : List<Int>
```

or any other applied generic type. Applied generic identity is not stored per instance.

---

# 4. Runtime object catalog

## 4.1 Class hierarchy

The public runtime vocabulary is ordinary Phalcom object-model vocabulary:

```text
Object
├── KindDescriptor                         abstract
│   ├── AtomicKind
│   └── FunctionKind
│
├── TypeDescriptor                         abstract synthetic TypeForm base
│   ├── AppliedType
│   ├── UnionType
│   ├── TupleType
│   ├── RecordType
│   ├── CallableType
│   ├── TypeLambda
│   ├── SpecialType
│   └── SelfType
│
├── TypeParameter
├── GenericSignature
├── GenericConstraint
├── CallableSignature
├── CallableParameter
├── FieldSignature
├── TypeUse
├── TypingContext
├── Typing                              utility/factory namespace
│
├── TypingResult                         sealed
│   ├── TypingKnown
│   ├── TypingUnknown
│   ├── TypingInvalid
│   ├── TypingUnavailable
│   ├── TypingCancelled
│   ├── TypingBudgetExceeded
│   └── TypingInternalFailure
│
├── TypeRelationResult                    sealed
│   ├── RelationSatisfied
│   ├── RelationRejected
│   ├── RelationDynamicBoundary
│   ├── RelationBlocked
│   ├── RelationCancelled
│   ├── RelationBudgetExceeded
│   └── RelationInternalFailure
│
├── MemberLookupResult                    sealed
│   ├── MemberFound
│   ├── MemberMissing
│   ├── MemberDynamicBoundary
│   ├── MemberBlocked
│   ├── MemberCancelled
│   ├── MemberBudgetExceeded
│   └── MemberInternalFailure
│
├── RelationEvidence
├── RelationFailure
├── DynamicBoundary
├── ReflectionCapability                 optional public capability descriptor
└── Future Spec-05 projection classes
```

All of these are ordinary classes under the existing metaclass rule. None creates a new meta-level.

### Why there is no `NominalType` descriptor class

Spec 01.5's conceptual type taxonomy can speak of a nominal type. At runtime, however, direct nominal reification is cheaper and more truthful:

```text
semantic Nominal(Int)  ──reify──> existing class object Int
```

Adding a `NominalType` runtime wrapper would duplicate identity and force needless cache/GC complexity. It is therefore forbidden in the initial implementation.

## 4.2 Physical heap representation

Per Spec 02, synthetic typing objects should share one boxed heap arm:

```rust
pub enum Object {
    // existing arms ...
    Typing(Box<TypingObject>),
}

pub struct TypingObject {
    pub class: ClassId,
    pub payload: TypingPayload,
}

pub enum TypingPayload {
    Context(TypingContextData),
    Descriptor {
        context: ObjRef,
        handle: RuntimeSemanticHandle,
    },
    // narrowly justified payloads for result/evidence objects may be added
    // only when ordinary InstanceObject fields would be materially worse.
}
```

The **surface class** distinguishes `AppliedType`, `TypeLambda`, `CallableSignature`, and so forth. The heap does not need a separate fat Rust enum arm for every descriptor class.

Result variants may be ordinary sealed Phalcom data classes/instances when their allocation profile is acceptable. They are not semantic authority objects and do not need to share the metadata-handle payload representation merely for uniformity.

## 4.3 Fixed bootstrap classes should remain minimal

The implementation must not automatically add every class above to `CoreClasses`.

Preferred order:

1. define ordinary reflection typing classes in universe source;
2. materialize them through the existing builtin module pipeline;
3. let `TypingObject.class` point at those ordinary runtime class handles;
4. add a fixed `CoreClasses` slot only if a primitive must construct that class before the source class can exist.

This preserves the current compact kernel bootstrap surface.

---

# 5. The `TypeForm` runtime role

`TypeForm` is a **semantic/behavioral role**, not the runtime superclass of every value that denotes a type.

Class objects already inherit through `Behavior`; synthetic descriptors inherit through `TypeDescriptor`; type parameters have their own class. A future protocol declaration may formalize this shared surface, but protocol implementation is not a prerequisite for this specification.

## 5.1 Required common surface

The ratified common vocabulary is:

```phalcom
# Logical role, not necessarily an immediately declared protocol.
TypeForm {
  kind -> KindDescriptor
  display -> String

  freeParameterCount -> Int
  freeParameterAt(_ index: Int) -> TypeParameter
  freeParameters -> Tuple

  remainingParameterCount -> Int
  remainingParameterAt(_ index: Int) -> TypeParameter
  remainingParameters -> Tuple

  equivalentTo(_ other: TypeForm) -> Bool
  subtypeOf(_ other: TypeForm) -> TypeRelationResult
}
```

### `kind`

Returns the canonical reflected kind of the form.

Examples:

```text
Int.kind                      => Type
List.kind                     => Type -> Type
Map.kind                      => Type -> Type -> Type
Map<String>.kind              => Type -> Type
(<T> =>> List<T>).kind        => Type -> Type
```

### `display`

Returns stable human-facing semantic rendering under the active semantic-model version. It is presentation, not identity and not a serialization key.

### `freeParameters`

Returns every free stable type binder referenced by the form, in deterministic canonical occurrence order.

### `remainingParameters`

Returns unsatisfied application slots for a partially applied constructor.

These two APIs are intentionally different:

```text
Pair<T, Int>
```

may contain free parameter `T` while having no remaining application slots if `Pair` is fully applied.

Conversely:

```text
Map<String>
```

has a remaining constructor parameter even when `String` itself contains no free binder.

### `equivalentTo`

For two already validated, compatible TypeForm runtime values, returns the semantic equivalence predicate directly as `Bool`.

It is not:

- `==`;
- `===`;
- subtyping;
- assignability;
- consistency;
- conformance;
- runtime class equality.

The loader rejects incompatible semantic-model versions before forms can share a context. Cross-context comparison delegates through stable semantic fingerprints/normalized structure as defined by Specs 01.5 and 02.

### `subtypeOf`

Returns a full `TypeRelationResult`, because subtyping may encounter dynamic boundaries, blocked dependencies, cancellation, or budgets. It is a convenience delegation to the active `TypingContext`, not an independent implementation on each descriptor class.

## 5.2 Why `hash` is not part of the TypeForm contract

The earlier Spec 03 required semantic `hash` parity across class objects and descriptors. That would create unnecessary pressure on existing class hash/identity semantics.

This revision does not require overriding ordinary runtime `hash` to mean semantic-type hashing. Internal metadata/type-form fingerprints remain available to the registry and semantic engine. A future dedicated `semanticHash` may be added only if a demonstrated public use case justifies it.

## 5.3 Why declaration-specific methods are not on the common role

A union, callable type, or type lambda may have no declaration object. A class declaration has a generic signature; an arbitrary structural form does not.

Therefore APIs such as declaration, declared generic parameters, superclass template, methods, and fields live on declaration/signature reflection or `TypingContext`, not on every TypeForm.

This avoids a protocol full of semantically meaningless `None` values.

---

# 6. Kinds

## 6.1 Runtime kind classes

Kinds are reflected through:

```text
KindDescriptor
├── AtomicKind
└── FunctionKind
```

`Type` is the canonical atomic-kind singleton. It is **not** a class object and does not imply `Type :: Type`.

Future atomic kinds such as `RecordRow` may be added by their owning specification. Their existence does not make them TypeForms.

## 6.2 Kind surface

```phalcom
class KindDescriptor {
  display -> String

  argumentCount -> Int
  argumentAt(_ index: Int) -> KindDescriptor
  arguments -> Tuple

  result -> Option<KindDescriptor>
  equivalentTo(_ other: KindDescriptor) -> Bool
}
```

Laws:

```text
Type.argumentCount == 0
Type.arguments == ()
Type.result == None

(Type -> Type).argumentCount == 1
(Type -> Type).argumentAt(0) === Type
(Type -> Type).result == Some(Type)
```

For a multi-argument canonical constructor kind, `.arguments` returns the canonical argument lane and `.result` returns the final result kind.

`KindDescriptor` intentionally has no `.kind` selector. Runtime reflected kinds are values describing classifiers; the static calculus does not derive `Type :: Type` through reflection.

## 6.3 No public arbitrary kind construction in the core API

The earlier `TypingContext.applyKind` API is removed from the core surface.

Type constructor application already checks kinds through `TypingContext.apply`. Advanced kind schemes or future kind-level abstractions remain owned by revised Spec 05. There is no reason to expose a second user-facing kind-calculus constructor before such a use case exists.

---

# 7. Applied, structural, special, and `Self` forms

## 7.1 AppliedType

```phalcom
class AppliedType is TypeDescriptor {
  origin -> TypeForm

  argumentCount -> Int
  argumentAt(_ index: Int) -> TypeForm
  arguments -> Tuple

  remainingParameterCount -> Int
  remainingParameterAt(_ index: Int) -> TypeParameter
  remainingParameters -> Tuple
}
```

`origin` is the normalized base constructor, not the immediately preceding partial application wrapper.

Canonical flattening therefore makes:

```text
Map<String><Int>
Map<String, Int>
```

reflect with the same origin and complete argument sequence when they normalize to the same form.

`argumentAt(i)` reifies only the requested child. `.arguments` is an explicit convenience operation and may allocate the tuple and previously unseen child descriptors.

## 7.2 UnionType

```phalcom
class UnionType is TypeDescriptor {
  memberCount -> Int
  memberAt(_ index: Int) -> TypeForm
  members -> Tuple
}
```

Member order follows canonical semantic normalization, never source spelling order if normalization changes it.

## 7.3 TupleType

```phalcom
class TupleType is TypeDescriptor {
  elementCount -> Int
  elementAt(_ index: Int) -> TypeForm
  labelAt(_ index: Int) -> Option<Symbol>
  elements -> Tuple
}
```

The complete tuple convenience view may use lightweight element-view objects if labels must travel with types. The indexed accessors are the performance contract.

## 7.4 RecordType

Initial closed-record reflection:

```phalcom
class RecordType is TypeDescriptor {
  fieldCount -> Int
  fieldNameAt(_ index: Int) -> Symbol
  fieldTypeAt(_ index: Int) -> TypeForm
  fields -> Object
}
```

Record-row tails and row descriptors are deferred to revised Spec 05. Closed-record support must not fake an open row using a sentinel field or a TypeForm.

## 7.5 CallableType

A structural callable **type** is distinct from a declared method's `CallableSignature` descriptor:

```phalcom
class CallableType is TypeDescriptor {
  parameterCount -> Int
  parameterTypeAt(_ index: Int) -> TypeForm
  parameterLabelAt(_ index: Int) -> Option<Symbol>
  parameterRestModeAt(_ index: Int) -> Symbol
  parameters -> Tuple
  returnType -> TypeForm
}
```

`CallableType` answers the structural type. `CallableSignature` additionally carries declaration identity, local parameter names, generic binders, side, source, and selector.

## 7.6 SpecialType

`Never` and future non-nominal proper-type constants that cannot directly reuse a class object may reify as canonical `SpecialType` instances.

`Dynamic` and `Unknown` are **not** `SpecialType` values. They are semantic knowledge/status states and must not become ordinary TypeForms merely for API convenience.

## 7.7 SelfType

Unspecialized signature-level `Self` may be reflected when a user inspects a declared generic signature without providing a receiver specialization:

```phalcom
class SelfType is TypeDescriptor {
  owner -> Object
  side -> Symbol
  role -> Symbol
}
```

The semantic owner/side/role model is defined by Spec 01.5 §13.

A specialized member view should normally expose the specialized result instead of leaking `SelfType` when receiver information is known.

---

# 8. Type parameters, generic signatures, and constraints

## 8.1 TypeParameter

```phalcom
class TypeParameter {
  owner -> Object
  index -> Int
  name -> Symbol
  kind -> KindDescriptor
  variance -> Option<Symbol>

  constraintCount -> Int
  constraintAt(_ index: Int) -> GenericConstraint
  constraints -> Tuple
}
```

Identity is semantic owner plus zero-based index. Binder names are presentation/resolution provenance only.

### Variance

For a nominal declaration parameter:

```text
Some(#covariant)
Some(#contravariant)
Some(#invariant)
```

For method-generic and type-lambda parameters, variance is not a declaration feature and returns `None`.

No `out`/`in` compatibility vocabulary is introduced.

### Constraints

Constraints are retrieved from the owning `GenericSignature`. They are not stored canonically on the parameter record itself. `TypeParameter.constraints` is a derived convenience filter for constraints mentioning that parameter.

### Removed fields

There is no canonical:

```text
TypeParameter.default
TypeParameter.upperBound
```

Generic defaults are not yet designed. Upper/lower/equality relationships are all represented through general signature constraints.

A later ergonomic `.upperBounds` convenience view may be added without changing canonical semantics, but it is not part of the initial required API.

## 8.2 GenericSignature

```phalcom
class GenericSignature {
  owner -> Object

  parameterCount -> Int
  parameterAt(_ index: Int) -> TypeParameter
  parameters -> Tuple

  constraintCount -> Int
  constraintAt(_ index: Int) -> GenericConstraint
  constraints -> Tuple
}
```

Owner may be:

- class/declaration object;
- `CallableSignature` descriptor;
- `TypeLambda` descriptor.

The metadata/runtime implementation may encode owner identity differently internally, but the public result must resolve to the corresponding reflected owner when available.

## 8.3 GenericConstraint

The initial reflected constraint vocabulary exactly mirrors Spec 01.5:

```phalcom
class GenericConstraint {
  relation -> Symbol      # #subtype | #equivalent
  left -> TypeForm
  right -> TypeForm
  source -> Option<Object>
}
```

Examples:

```phalcom
where T <: Number
```

reflects as:

```text
relation = #subtype
left     = T
right    = Number
```

A lower bound:

```phalcom
where Number <: T
```

uses the same relation with operands reversed.

Semantic equality:

```phalcom
where T == U
```

reflects as `#equivalent`.

There is no finite-set `#in` constraint in the initial API.

## 8.4 Indexed access is normative

The tuple-returning convenience APIs above are not permission to eagerly construct every child descriptor.

For a signature with 30 parameters, a call to:

```phalcom
signature.parameterTypeAt(17)
```

must not require allocation of 29 unrelated `CallableParameter` or TypeForm objects.

---

# 9. Type lambdas

## 9.1 Runtime role

A reflected type lambda is a semantic TypeForm, not a `Closure` and not executable program code.

```phalcom
class TypeLambda is TypeDescriptor {
  kind -> KindDescriptor

  parameterCount -> Int
  parameterAt(_ index: Int) -> TypeParameter
  parameters -> Tuple

  body -> TypeForm

  freeParameterCount -> Int
  freeParameterAt(_ index: Int) -> TypeParameter
  freeParameters -> Tuple
}
```

Example source:

```phalcom
<T> =>> Result<T, Error>
```

may be reified as a `TypeLambda` only when the value escapes into runtime space or reflection explicitly requests it.

## 9.2 Alpha equivalence

Binder spelling is not semantic identity:

```text
<T> =>> List<T>
<U> =>> List<U>
```

must be `equivalentTo` one another.

Runtime descriptors carry source binder names when retained for presentation, but the metadata/semantic handle uses alpha-normalized scoped structure from Spec 02 §9.

## 9.3 Application

The source/operator form:

```phalcom
lambda.<>(Int)
```

is written in actual source according to Spec 04's `<>` type-form application syntax; conceptually this document denotes it as:

```text
lambda.<>(Int)
```

Application invokes the trusted semantic application operation, performs beta reduction, kind checking, and canonicalization, and returns a validated TypeForm result.

It does **not** dispatch an overridable ordinary method whose behavior can redefine type semantics.

At runtime the descriptor class may expose the selector needed for uniform syntax, but the primitive behind that selector is compiler/runtime-authoritative and sealed against semantic override in exactly the same sense as other language semantic gateways.

## 9.4 No closure capture

A `TypeLambda` cannot capture runtime locals by value as a closure does. Its free parameters are semantic binders/type forms. It has no bytecode body, closure environment, home-frame token, or executable effect.

---

# 10. Declaration, callable, and field reflection

This section is central to typed dispatch, documentation tooling, metaprogramming, and rich IDE support.

## 10.1 Declaration queries

Class objects remain ordinary declarations/behaviors. The initial public typed-reflection accessors should be context-mediated so profile, authority, and availability remain explicit:

```phalcom
class TypingContext {
  typeOfDeclaration(_ declaration: Object) -> TypingResult
  genericSignatureOf(_ declaration: Object) -> TypingResult
  declaredSupertypeOf(_ declaration: Object) -> TypingResult
}
```

Convenience methods may later be mirrored directly onto `Behavior`:

```phalcom
Int.kind
List.genericSignature
```

provided those methods delegate to the same registry/query layer and return honest unavailable states when metadata is absent.

`declaredSupertypeOf` returns the semantic generic supertype template, not merely the runtime superclass class object.

Example:

```phalcom
class Names<T> is Sequence<Option<T>>
```

reflects the declared supertype template `Sequence<Option<T>>`.

## 10.2 CallableSignature

A declared callable signature is reflected through:

```phalcom
class CallableSignature {
  owner -> Object
  side -> Symbol                    # #instance | #class
  selector -> Selector
  genericSignature -> Option<GenericSignature>

  parameterCount -> Int
  parameterAt(_ index: Int) -> CallableParameter
  parameterTypeAt(_ index: Int) -> TypeForm
  parameters -> Tuple

  returnType -> TypeForm
  source -> Option<Object>
  documentation -> Option<String>
}
```

`documentation` is available only if the active profile retains phaldoc/documentation metadata. Its absence is not a typing failure.

`parameterTypeAt` is a mandatory direct fast path. It must not create a `CallableParameter` object when only the type is required.

The descriptor represents one semantic callable identity. A specialized lookup may return a view over the same callable with a bound specialization environment; it does not synthesize a new declaration or runtime method.

## 10.3 CallableParameter

```phalcom
class CallableParameter {
  index -> Int
  localName -> Symbol
  externalLabel -> Option<Symbol>
  restMode -> Symbol
  type -> TypeForm
  source -> Option<Object>
}
```

`restMode` uses the language's canonical parameter-lane vocabulary. It is not inferred from selector text at reflection time.

## 10.4 FieldSignature

```phalcom
class FieldSignature {
  owner -> Object
  side -> Symbol
  name -> Symbol
  mutable -> Bool
  type -> TypingResult
  source -> Option<Object>
}
```

A field without a known declared/inferred publishable type must not fabricate `Object` or `Dynamic`. `type` therefore returns `TypingResult` rather than an unconditional TypeForm.

## 10.5 Generated declarations must be indistinguishable semantically from equivalent written declarations

When attributes/compiler transforms synthesize a getter, setter, constructor, data member, variant member, or visitor and the type is derivable exactly, the reflected signature must contain that exact type.

Example:

```phalcom
@get
const _name: String
```

must eventually expose a generated getter semantically equivalent to:

```text
name -> String
```

Reflection must not reveal `Unknown` merely because the source member was synthesized after parsing.

## 10.6 Method object to static signature bridge

```phalcom
class TypingContext {
  signatureOf(_ method: Method) -> TypingResult
}
```

Implementation rule:

```text
Method ObjRef
   │ O(1) VM side-table lookup
   ▼
RuntimeCallableRef
   │
   ▼
immutable semantic metadata record
```

Do not reconstruct the static signature by:

- converting the runtime holder/name to strings;
- reparsing a selector;
- searching every declaration;
- copying signature information into `MethodObject`.

### Method replacement

If a previously reified `Method` handle remains alive after the class's live method table is replaced, its exact side-table entry may continue to describe that **method object**. `signatureOf(oldMethod)` is therefore not automatically stale merely because the method is no longer the current live implementation for the selector.

A dynamically installed method with no semantic mapping returns an honest unavailable/unknown result. It must never inherit the old method's static signature merely because selector and holder match.

---

# 11. Result algebra: sealed variants, not nullable bags

## 11.1 General rule

Compiler-internal queries use compact Rust enums or equivalent typed internal results and allocate no VM objects.

Only an explicit runtime reflection boundary projects those results into Phalcom sealed variant instances.

Public result classes must encode mutually exclusive states structurally. They must not be one mutable-looking object with a status symbol plus several optional payloads that can form impossible combinations.

## 11.2 TypingResult

```text
TypingResult
├── TypingKnown(value)
├── TypingUnknown(reason)
├── TypingInvalid(diagnostics)
├── TypingUnavailable(reason)
├── TypingCancelled
├── TypingBudgetExceeded(report)
└── TypingInternalFailure(incidentId)
```

Suggested logical source declarations, using the project's sealed-variant mechanism when typed variant payloads are supported:

```phalcom
@sealed
class TypingResult {}

# conceptual variants; exact generated syntax follows the variant feature
TypingKnown(value)
TypingUnknown(reason)
TypingInvalid(diagnostics)
TypingUnavailable(reason)
TypingCancelled
TypingBudgetExceeded(report)
TypingInternalFailure(incidentId)
```

No-payload variants such as `TypingCancelled` should be canonical singleton instances.

Convenience predicates may exist:

```phalcom
result.isKnown
result.isUnavailable
```

but are derived from variant identity; they are not the primary representation.

### Meaning of states

- `TypingKnown` — the requested semantic value/fact is available and validated.
- `TypingUnknown` — the semantic system genuinely lacks enough knowledge.
- `TypingInvalid` — source/metadata/request is semantically invalid; diagnostics explain why.
- `TypingUnavailable` — the build/profile/context does not retain or expose this category of metadata.
- `TypingCancelled` — caller/system cancelled the operation.
- `TypingBudgetExceeded` — explicit resource limit stopped the operation.
- `TypingInternalFailure` — invariant/implementation failure identified by a stable incident token.

## 11.3 TypeRelationResult

```text
TypeRelationResult
├── RelationSatisfied(evidence)
├── RelationRejected(failure)
├── RelationDynamicBoundary(boundary)
├── RelationBlocked(reason)
├── RelationCancelled
├── RelationBudgetExceeded(report)
└── RelationInternalFailure(incidentId)
```

`RelationSatisfied` is the only positive proof of the requested relation.

`RelationDynamicBoundary` means static/runtime-erased or open-world information prevents a sound conclusion and a runtime boundary remains. It is **not** satisfied.

`RelationBlocked` means a dependency or knowledge precondition is not presently available under the chosen policy. It does not include cancellation or budget exhaustion.

## 11.4 MemberLookupResult

Member lookup is not a binary type relation and therefore receives its own result algebra:

```text
MemberLookupResult
├── MemberFound(signature)
├── MemberMissing(failure)
├── MemberDynamicBoundary(boundary)
├── MemberBlocked(reason)
├── MemberCancelled
├── MemberBudgetExceeded(report)
└── MemberInternalFailure(incidentId)
```

This avoids awkward states such as pretending a callable signature is “relation evidence.”

`MemberFound(signature)` contains a possibly specialized `CallableSignature` or `FieldSignature` view.

## 11.5 Evidence and failure objects

Evidence/failure objects are immutable and bounded. They may carry:

- relation kind;
- normalized left/right semantic identities;
- concise derivation/reason path;
- declaration/source provenance when retained;
- world/snapshot identity when relevant.

They are diagnostic/explanatory projections, not proof-system certificates unless revised Spec 05 explicitly says so.

## 11.6 Proof results remain a downstream projection gate

The earlier Spec 03 prematurely froze a proof object hierarchy/status bag. This revision does not.

Spec 03 reserves:

```phalcom
TypingContext.proofsOf(_ declaration)
```

behind `INSPECT_PROOFS`, but the exact `ProofResult` sealed variants, trust tiers, counterexample payloads, certificates, and artifact semantics are owned by revised Spec 05.

Until that specification is revised and implemented, proof reflection returns `TypingUnavailable` rather than an invented placeholder proof state.

---

# 12. TypingContext

## 12.1 Purpose

`TypingContext` is the explicit runtime handle that combines:

- validated loaded semantic metadata;
- semantic-model/schema compatibility;
- a snapshot/program identity;
- capability set;
- bounded runtime-created TypeForm overlay;
- weak synthetic-descriptor cache;
- live-world stamp/reference required by runtime bridge operations.

It is **not** ambient generic state for currently executing methods.

## 12.2 Acquisition

```phalcom
class Typing {
  @class current -> TypingResult
  @class contextFor(_ module: Module) -> TypingResult
}
```

`Typing.current` returns the program's active public typing context when the build/profile permits public typing reflection.

A `RuntimeMinimal` artifact may contain enough semantic roots to execute explicit type-form constants while still declining arbitrary discovery through `Typing.current`. Metadata presence and public discoverability are separate Spec 02 concepts.

## 12.3 Context observation

```phalcom
class TypingContext {
  profile -> Symbol
  capabilities -> Tuple
  semanticModel -> Object
  snapshot -> Object
  world -> Object

  restrictTo(_ capabilities: Tuple) -> TypingContext
  refresh -> TypingResult
}
```

The context is semantically immutable. Its overlay/cache may memoize canonical runtime forms, but this does not change the meaning of already-returned forms.

`restrictTo` can only remove authority. It cannot mint capabilities absent from the parent context.

`refresh` returns a **new** context. It never mutates old descriptors, proof artifacts, or type-use observations into a newer semantic world.

---

# 13. Capabilities and metadata profiles

## 13.1 Capability set

Initial internal/public capability identities:

```text
OBSERVE_PUBLIC_TYPES
OBSERVE_SIGNATURES
CONSTRUCT_TYPE_FORMS
EVALUATE_RELATIONS
OBSERVE_SOURCE_USES
OBSERVE_PRIVATE_TYPES
VALIDATE_RUNTIME_VALUES
INSPECT_PROOFS
INVOKE_REFLECTIVELY
```

Capabilities are unforgeable runtime authority values or equivalent VM-owned tokens. A symbol with the same spelling is not authority.

## 13.2 Meaning

### `OBSERVE_PUBLIC_TYPES`

Inspect public declaration type forms, kinds, generic signatures, public superclass templates, and structural children already reachable from an observed public TypeForm.

### `OBSERVE_SIGNATURES`

Inspect public callable/field semantic signatures and map authorized runtime `Method` handles to retained static signatures.

### `CONSTRUCT_TYPE_FORMS`

Create validated runtime TypeForms in the context overlay via `apply`, `unionOf`, tuple/record/callable constructors, and type-lambda application.

This does **not** grant authority to construct runtime instances.

### `EVALUATE_RELATIONS`

Run bounded equivalence/subtype/assignability/consistency/conformance/member semantic queries.

### `OBSERVE_SOURCE_USES`

Read source-occurrence metadata such as type uses, source ranges, source spelling, inference provenance, and related tooling-only facts.

### `OBSERVE_PRIVATE_TYPES`

Observe non-public declaration/member metadata subject to normal caller/module authority. This is not a universal privacy bypass.

### `VALIDATE_RUNTIME_VALUES`

Permit explicit runtime representation/deep validation of a live value against a TypeForm under budgets.

### `INSPECT_PROOFS`

Read proof artifacts/results when revised Spec 05 and the active profile provide them.

### `INVOKE_REFLECTIVELY`

Cross from a type-form/static query into actual runtime construction/invocation. Existing runtime access-control and dispatch semantics still apply.

## 13.3 Default profile mapping

Recommended default capability envelopes:

| Profile | Default public capabilities |
|---|---|
| `RuntimeMinimal` | none for global discovery; only internal operations necessary for emitted runtime TypeForm constants/validators |
| `RuntimePublic` | `OBSERVE_PUBLIC_TYPES`, `OBSERVE_SIGNATURES`, `CONSTRUCT_TYPE_FORMS`, bounded `EVALUATE_RELATIONS` |
| `ToolingDebug` | RuntimePublic + `OBSERVE_SOURCE_USES`; private observation only according to explicit tooling/debug policy |
| `Proof` | ToolingDebug + proof inspection according to trust/policy |

`VALIDATE_RUNTIME_VALUES` and `INVOKE_REFLECTIVELY` should remain explicit policy grants rather than silently appearing merely because public metadata is present.

## 13.4 Possession versus enumeration

Possessing a valid TypeForm object permits structural observation necessary for that value's ordinary TypeForm behavior even if the context does not allow global declaration enumeration.

For example, a `RuntimeMinimal` program may contain a reified executable type-lambda constant. Reading its `kind` and `body` may be necessary for normal use. That does not imply permission to enumerate every private method signature in the program.

---

# 14. Type-form construction API

## 14.1 Required constructors

```phalcom
class TypingContext {
  apply(_ origin: TypeForm, arguments: Tuple) -> TypingResult
  unionOf(_ members: Tuple) -> TypingResult
  tupleOf(_ elements: Tuple) -> TypingResult
  recordOf(_ fields: Record) -> TypingResult
  callable(_ parameters: Tuple, returns result: TypeForm) -> TypingResult
}
```

Type-lambda `.<>(...)` delegates to the same semantic application engine.

## 14.2 `apply`

Application performs the Spec 01.5 pipeline:

```text
validated origin + validated argument forms
        ↓
kind applicability
        ↓
parameter binding
        ↓
residual kind
        ↓
known-argument substitution into constraints
        ↓
constraint evaluation: satisfied / rejected / deferred
        ↓
canonical normalization
        ↓
bounded overlay intern/reuse
        ↓
lazy root reification
```

Partial application is valid when the residual kind remains an arrow and all currently decidable constraints succeed.

The operation does not allocate intermediate partial descriptors merely because a semantic implementation conceptually applies arguments one at a time.

## 14.3 Union/tuple/record/callable construction

All inputs must already denote validated TypeForms of appropriate kinds.

- union members must be proper `Type` forms;
- tuple element types must be proper `Type` forms;
- initial `recordOf` constructs closed records only;
- callable parameter and return forms must satisfy the canonical callable type rules.

Normalization is implicit. There is no separate public `normalize` call because every successfully returned TypeForm is already canonical under the context's semantic model.

## 14.4 Public substitution is deferred

The earlier Spec 03 exposed:

```text
substitute(form, using environment)
```

This is removed from the core public API.

Substitution environments are a central compiler implementation mechanism for lazy specialization, but exposing arbitrary binder substitution creates a second programmable type transformation language without a demonstrated need.

Users already have:

- constructor application;
- type-lambda application;
- specialized member lookup.

A future explicit substitution API may be designed if reflective metaprogramming requires it.

## 14.5 Raw descriptor construction is impossible

There are no public constructors such as:

```text
AppliedType.new(...)
TypeLambda.new(rawNodeId)
TypeParameter.new(owner, index)
```

Descriptor classes reject direct construction. Trusted primitives create them only from validated registry handles.

---

# 15. Semantic relation API

## 15.1 Context operations

```phalcom
class TypingContext {
  equivalent(_ left: TypeForm, to right: TypeForm) -> TypeRelationResult
  subtype(_ candidate: TypeForm, of supertype: TypeForm) -> TypeRelationResult
  assignable(_ actual: TypeForm, to expected: TypeForm) -> TypeRelationResult
  consistent(_ left: TypeForm, with right: TypeForm) -> TypeRelationResult
  conforms(_ candidate: TypeForm, to protocol: Object) -> TypeRelationResult
}
```

Every operation delegates to compiler-owned/shared semantic relation logic or the validated runtime equivalent generated from the same canonical rules. It does not copy current `relation.rs` into `phalcom-core`.

## 15.2 Relations remain distinct

The following are not interchangeable:

- semantic equivalence;
- subtyping;
- assignability;
- gradual/dynamic consistency;
- protocol/structural conformance.

In particular, a dynamic/open boundary may permit execution while failing to establish subtype evidence.

## 15.3 `equivalentTo` convenience

`TypeForm.equivalentTo` is the total convenience predicate for two valid reflected forms under compatible semantic-model versions.

`TypingContext.equivalent` exists when callers need evidence, budget/cancellation visibility, or reasoned failure.

## 15.4 Bounded execution

Relation queries consume context/query budgets from Spec 01. A hostile recursive or extremely wide TypeForm graph cannot cause unbounded runtime work.

Budget exhaustion returns `RelationBudgetExceeded`, not `RelationBlocked`, `TypingUnknown`, `false`, or `true`.

Cancellation returns `RelationCancelled`.

---

# 16. Static member lookup

## 16.1 API

```phalcom
class TypingContext {
  member(
    on receiver: TypeForm,
    selector: Selector,
    side: Symbol,
    lookup: LookupMode
  ) -> MemberLookupResult
}
```

Initial sides:

```text
#instance
#class
```

Lookup modes:

```text
LookupMode.normal
LookupMode.superFrom(definingDeclaration, side)
```

## 16.2 Exact selector identity

Member lookup consumes the canonical `Selector` value, including:

- base selector name;
- positional arity/lane;
- labels;
- setter/index shape where applicable.

Typing metadata never adds parameter types to selector identity.

## 16.3 Generic specialization

For a receiver:

```text
Box<String>
```

and declared member:

```text
value -> T
```

`member(...)` returns a specialized signature view whose `returnType` is `String`.

The implementation should retain the compact representation:

```rust
struct SpecializedMemberView {
    callable: CallableId,
    environment: EnvironmentRef,
}
```

or equivalent, and reify parameter/result forms lazily.

It should **not** eagerly clone an entire substituted signature for every lookup.

## 16.4 `super`

`superFrom(definingDeclaration, side)` changes lookup start while retaining the actual receiver specialization exactly as source `super` semantics require.

## 16.5 Access control

Static member reflection obeys visibility authority. A caller lacking permission receives a `MemberMissing`/rejected-access failure or a capability/availability result according to the public API policy; it never receives a hidden signature merely because metadata exists.

## 16.6 Static lookup versus live runtime lookup

These are intentionally different:

```text
TypingContext.member(...)  -> declared/published semantic world
obj.respondsTo(...)        -> live runtime world
obj.methodFor(...)         -> live runtime world
obj.perform(...)           -> live runtime dispatch
```

A class may be reflectively modified after metadata publication. In that case both answers can be truthful even when they differ.

---

# 17. Source TypeUse reflection

## 17.1 Purpose

A `TypeUse` describes what the compiler knew about a specific source occurrence. It is not itself a TypeForm.

The need comes directly from the two-axis model: a source expression such as `Int` has a runtime value type associated with the class object and separately denotes the nominal TypeForm `Int`.

## 17.2 API

```phalcom
class TypingContext {
  typeUseAt(_ module: Module, range: SourceRange) -> TypingResult
  typeUsesOf(_ declaration: Object) -> TypingResult
}

class TypeUse {
  valueType -> TypingResult
  denotation -> TypingResult
  source -> Object
  spelling -> Option<String>
  evidence -> Option<Object>
  inference -> Option<Object>
  constant -> Option<Object>
}
```

The exact evidence/inference provenance classes may grow with the compiler's semantic presentation layer. Their absence must not alter the core type result.

## 17.3 Availability

Source occurrence reflection requires `ToolingDebug`-level metadata and `OBSERVE_SOURCE_USES`.

A release/runtime profile that omitted occurrence metadata returns `TypingUnavailable`, not `TypingUnknown`.

The compiler/LSP itself does **not** need runtime `TypeUse` objects. It reads the same formal snapshot facts directly.

---

# 18. Runtime matching and explicit validation

This section defines the safe bridge needed for runtime typed-dispatch libraries without violating erasure.

## 18.1 APIs

```phalcom
class TypingContext {
  matches(_ value: Object, against form: TypeForm) -> TypeRelationResult
  validate(_ value: Object, as form: TypeForm) -> TypeRelationResult
}
```

For surface typing, these selectors accept any Phalcom value even if the example declaration uses `Object` as a placeholder type annotation.

## 18.2 `matches`: cheap runtime evidence only

`matches` performs only checks supported directly by runtime representation and authoritative metadata.

Examples:

```text
matches(42, Int)             -> Satisfied
matches(42, Number)          -> Satisfied
matches("x", Int)           -> Rejected
```

For an ordinary erased generic instance:

```text
matches(list, List<Int>)
```

cannot conclude that the list's elements satisfy `Int` merely from `list.class == List`.

The correct result is normally:

```text
RelationDynamicBoundary(...)
```

unless that specific runtime representation carries separately trusted evidence sufficient for the relation.

`matches` never traverses an arbitrary object graph to manufacture deeper evidence.

## 18.3 `validate`: explicit bounded deep validation

`validate` is an opt-in operation requiring `VALIDATE_RUNTIME_VALUES`.

It may inspect trusted runtime representations to determine whether the **current value state** satisfies the requested structural/applied form.

For standard containers, an implementation may support bounded validation such as:

```text
validate([1, 2, 3], as: List<Int>) -> Satisfied
```

subject to:

- element budget;
- recursion-depth budget;
- cycle detection;
- representation-specific rules;
- dynamic/opaque element boundaries;
- cancellation.

## 18.4 Validation does not install a type

Even after successful validation:

```text
list
```

is still an ordinary `List` instance with no per-instance generic token.

For mutable containers, successful validation means only:

> the checked value state satisfied the requested form at validation time under the stated policy.

It does not prove that future writes preserve that applied type.

## 18.5 User-defined erased generics

For arbitrary user-defined generic classes whose runtime representation does not preserve enough structure to validate type arguments safely, `validate` returns `RelationDynamicBoundary` or `RelationBlocked` with an explicit reason.

It never assumes that a nominal origin match implies applied generic match.

## 18.6 Typed dispatch use case

A user-space `@typecase` or multimethod implementation can therefore:

1. inspect a method's declared parameter types with `signatureOf(method)`;
2. use `parameterTypeAt(i)` without materializing the complete signature tree;
3. compare those TypeForms semantically;
4. run `matches` for cheap nominal/runtime checks;
5. optionally request `validate` when the library deliberately wants deep validation;
6. choose its method using ordinary user-space logic;
7. invoke the chosen method through the existing method invocation boundary.

The compiler's own static typed-dispatch optimization does **not** use these runtime objects. It operates directly on semantic IDs/views.

---

# 19. Explicit type-directed construction

## 19.1 API

```phalcom
class TypingContext {
  construct(_ form: TypeForm, arguments: Tuple) -> TypingResult
}
```

This operation requires `INVOKE_REFLECTIVELY` and is deliberately separate from ordinary message sends.

## 19.2 Semantics

For a nominal or applied nominal form, `construct`:

1. validates the form;
2. identifies the runtime origin class;
3. validates any requested static argument expectations according to the selected construction policy;
4. applies ordinary runtime constructor lookup/access rules to the origin class;
5. invokes through the existing runtime gateway;
6. returns the runtime result.

It does not install an applied class context in the constructor body.

## 19.3 Ordinary dispatch remains unchanged

Given:

```text
listOfInt = List<Int>
```

this specification does not make:

```text
listOfInt.new(...)
```

silently forward to `List.new(...)`.

Applied TypeForm descriptors are not proxy class objects.

Explicit construction exists precisely so this type-directed behavior has an unmistakable boundary:

```text
Typing.current.construct(listOfInt, arguments)
```

## 19.4 Access control and DNU

The final constructor call uses existing runtime semantics. Reflection does not bypass:

- private/protected access;
- selector shape;
- argument-lane validation;
- `doesNotUnderstand` behavior;
- native/bytecode activation rules.

---

# 20. World, snapshot, and mutation semantics

## 20.1 Two different worlds must not be conflated

A `TypingContext` can simultaneously refer to:

1. a **published semantic snapshot** describing declared/analyzed program meaning;
2. the **live runtime world** whose method dictionaries can change reflectively.

The two are related but not identical.

## 20.2 Static structural queries remain pinned and truthful

Queries such as:

```text
kind
freeParameters
remainingParameters
equivalent
subtype
declaredSupertypeOf
member(...)
```

answer the semantic snapshot they are pinned to.

A later runtime monkey patch does not retroactively make those answers “stale lies.” They remain statements about the published static world.

## 20.3 Live bridge operations reconcile runtime world state

Operations that cross into live execution may require a world check:

- `construct`;
- optional future reflective member invocation;
- runtime validation relying on mutable method/protocol behavior;
- any bridge whose correctness assumes a current runtime method mapping.

If the runtime world diverged in a way that invalidates the bridge's premise, the API returns an explicit boundary/unavailable/failure state rather than using stale static evidence as live authority.

## 20.4 `signatureOf(method)` describes an exact method object

The side table maps an exact `Method` handle to its semantic callable record. If that method is later replaced in a class dictionary but the old method object remains live, `signatureOf(oldMethod)` may continue to describe it accurately.

This is distinct from asking what method is **currently** installed for a selector.

## 20.5 Refresh creates a new view

`TypingContext.refresh` may acquire current metadata/runtime-world alignment and produce a new context. Existing descriptor objects continue to denote the old context's semantic forms.

No mutation-in-place is allowed.

---

# 21. Static semantic query facade

## 21.1 Location

Create or evolve a thin facade in:

```text
phalcom-semantic/src/reflection.rs
```

or an equivalent focused module under the eventual Spec 01 query package.

Conceptual API:

```rust
pub struct ReflectionQuery<'s> {
    snapshot: &'s PublishedSemanticSnapshot,
    capabilities: ReflectionCapabilities,
    budget: QueryBudget,
}
```

The exact snapshot type follows Spec 01 implementation.

## 21.2 Responsibility

The facade exposes read-only projections for:

- declaration TypeForm;
- kind/generic signature;
- callable/field signatures;
- specialized member lookup;
- type-use/source occurrence facts;
- relations;
- contracts/proofs once their owning specs exist.

It does not:

- parse runtime objects;
- allocate VM descriptors;
- own a `VM` reference;
- maintain a second relation solver;
- own a second declaration cache;
- recompute the workspace;
- convert advisory LSP `ValueShape` into formal typing truth.

## 21.3 Runtime adaptation

`phalcom-core` adapts the semantic/metadata result into runtime objects:

```text
semantic result / RuntimeSemanticHandle
        ↓
RuntimeTypingRegistry
        ↓
TypingContext capability + budget check
        ↓
lazy descriptor/result projection
```

This keeps `phalcom-semantic` VM-independent.

---

# 22. LSP, CLI, compiler, and REPL integration

## 22.1 Compiler

The compiler consumes canonical semantic IDs and views directly. It must never create runtime `AppliedType`, `CallableSignature`, `TypeParameter`, or result objects merely to continue compilation.

## 22.2 `phalcom check`

CLI diagnostics consume the same formal snapshot/query outcomes. Runtime reflection metadata is irrelevant to ordinary checking.

A stripped runtime profile therefore cannot make `phalcom check` less semantically capable when source/compiler information is available.

## 22.3 LSP

Formal hover, completion, signature help, navigation, and future inference provenance consume compiler-owned semantic facts directly.

Runtime descriptors are not an LSP transport layer.

The existing LSP `ValueShape` system remains advisory and may enrich UI when formal facts are absent, but it cannot be exposed as a validated TypeForm.

## 22.4 REPL

The REPL should eventually use Spec 01's persistent semantic workspace/database for formal facts across cells.

Runtime typing reflection in the REPL uses the same loaded metadata/context architecture as ordinary execution. It must not create a second “REPL-only” type system or infer generic arguments from runtime values differently.

---

# 23. Runtime package layout

## 23.1 Universe source layout

Use the existing reflection package:

```text
phalcom-core/core/universe/src/reflection/typing/
├── package.ph
├── kind.ph
├── type-descriptor.ph
├── type-parameter.ph
├── generic-signature.ph
├── signature.ph
├── type-use.ph
├── result.ph
├── evidence.ph
└── context.ph
```

Type-lambda and specialized descriptor classes may live in `type-descriptor.ph` initially or be split when file size warrants it.

Modify:

```text
phalcom-core/core/universe/src/reflection/package.ph
```

to expose `.typing`.

Modify:

```text
phalcom-modules/src/builtin.rs
```

to add the `reflection/typing` package and its child modules to `UNIVERSE_NODES`.

## 23.2 Native primitives

Create focused modules rather than one monolith, for example:

```text
phalcom-core/src/primitive/typing/
├── mod.rs
├── context.rs
├── descriptor.rs
├── signature.rs
├── relation.rs
├── runtime_validation.rs
└── result.rs
```

The primitive layer:

- accepts only trusted runtime descriptor/context objects, never raw IDs;
- validates capabilities;
- enforces budgets;
- retrieves compact runtime semantic handles;
- delegates structural/semantic meaning to validated metadata/query helpers;
- creates only requested runtime child objects;
- preserves normal caller authority for any live invocation.

## 23.3 Heap integration

Modify:

```text
phalcom-core/src/heap/object.rs
```

with the Spec 02 `Object::Typing(Box<TypingObject>)` arm.

Modify:

```text
phalcom-core/src/heap/trace.rs
```

with an explicit exhaustive arm. Descriptor payload traces its context handle; a context traces owned strong roots but **not** weak descriptor-cache entries.

Modify the `Value::class` implementation so:

```rust
Object::Typing(obj) => obj.class
```

No new `Value` tag is added.

## 23.4 VM integration

The VM owns:

- `RuntimeTypingRegistry`;
- loaded immutable metadata pools;
- declaration/class bindings;
- method semantic `SecondaryMap` or equivalent;
- context roots/registry state required by loaded programs.

Do not put a broad `Arc<RwLock<...>>` behind every descriptor.

## 23.5 Native metadata/catalog integration

Native surface definitions must eventually include typing reflection signatures through the same authoritative native metadata mechanism as other primitives.

However, do not bulk-add every descriptor class to the fixed kernel `CoreClasses` merely to make native metadata convenient. Prefer ordinary source-defined classes and stable runtime class handles, introducing fixed bootstrap roots only when startup ordering genuinely requires them.

---

# 24. Weak descriptor identity and garbage collection

## 24.1 Same-context live identity

While a canonical synthetic descriptor remains live in one context:

```text
reify(handle) === reify(handle)
```

should normally hold because the weak cache returns the existing object.

## 24.2 Collection is allowed

The cache is weak. If no strong reference retains the descriptor, GC may reclaim it. Reifying the same semantic handle later may produce a different `ObjRef`:

```text
oldDescriptor === newDescriptor     # may be false after collection
oldDescriptor.equivalentTo(newDescriptor)   # true
```

Semantic meaning never depends on stable descriptor object identity across reclamation.

## 24.3 Required weak-cache lookup

Because `ObjRef` is a generational handle, stale weak entries are expected.

The lookup algorithm must use the non-panicking heap probe:

```rust
if let Some(obj_ref) = weak_cache.get(&handle).copied() {
    if heap.try_get(obj_ref).is_some() {
        return obj_ref;
    }
    weak_cache.remove(&handle);
}
```

Using `heap.get(obj_ref)` for this path is forbidden because stale cache entries are ordinary lifecycle events, not VM invariant failures.

## 24.4 Descriptor -> context edge

A descriptor strongly retains its `TypingContext`. This ensures the metadata pool/overlay that interprets its compact handle remains alive for as long as the descriptor is observable.

The inverse cache edge remains weak, preventing cycles from making every created TypeForm immortal.

---

# 25. Diagnostics and reason vocabulary

Runtime result objects carry stable reason categories/codes; human strings are presentation.

Initial reflection codes should include at least:

| Code | Result family | Meaning |
|---|---|---|
| `reflection.metadata.unavailable` | `TypingUnavailable` | Requested category omitted/not loaded |
| `reflection.capability.denied` | unavailable/failure | Context lacks requested authority |
| `reflection.form.invalid` | `TypingInvalid` | Value is not a validated TypeForm for operation |
| `reflection.kind.not_applicable` | `TypingInvalid` | Applied an atomic/non-constructor form |
| `reflection.application.arity` | `TypingInvalid` | Too many/otherwise invalid type arguments |
| `reflection.application.kind_mismatch` | `TypingInvalid` | Type argument has wrong kind |
| `reflection.application.constraint` | rejected/invalid | Generic `where` constraint cannot be satisfied |
| `reflection.member.missing` | `MemberMissing` | No declared member under static lookup |
| `reflection.member.access_denied` | `MemberMissing`/failure | Visibility authority rejects lookup |
| `reflection.dynamic_boundary` | dynamic boundary | Open/runtime-erased fact prevents proof |
| `reflection.runtime.erased_generic` | dynamic boundary | Runtime class does not prove applied generic args |
| `reflection.runtime.validation_unsupported` | dynamic/blocked | Representation cannot be deeply validated |
| `reflection.world.diverged` | boundary/failure | Live bridge premise differs from pinned semantic world |
| `reflection.cancelled` | dedicated cancelled variant | Query cancelled |
| `reflection.budget_exceeded` | dedicated budget variant | Explicit bounded operation stopped |
| `reflection.internal_failure` | internal failure | Invariant failure with incident ID |

Cancellation and budget exhaustion must never be encoded as `reflection.context.stale_world`, `Blocked`, or `Unknown` simply to reduce variant count.

---

# 26. Security and authority

## 26.1 Metadata existence is not authorization

A metadata bundle may physically contain private declaration/signature information needed by tooling or proof profiles. Runtime APIs still enforce capability and language access policy before exposing it.

## 26.2 Raw handles never cross into user code

Public code never receives:

- metadata node numbers;
- store-local IDs;
- `RuntimeSemanticHandle` integer representation;
- VM registry indices.

Opaque runtime descriptor objects are the boundary.

## 26.3 Runtime-created TypeForms are untrusted as evidence

Creating:

```text
Typing.current.apply(List, (Int,))
```

proves only that `List<Int>` is a valid semantic TypeForm under the context. It does not prove any arbitrary `List` value satisfies it.

## 26.4 Reflective invocation preserves caller authority

Any `construct` or future signature-based invocation path must reuse the existing authorization machinery demonstrated by `Method#invokeOn`, `perform`, and normal dispatch.

Typing capability is not private/protected authority.

## 26.5 Resource-exhaustion resistance

All user-triggerable semantic construction/relation/validation work has hard context budgets. Deep recursive TypeForms, enormous unions, huge container validation, and hostile metadata cannot trigger unbounded recursion or allocation.

---

# 27. Performance contract

Performance is a normative part of this API architecture.

## 27.1 Zero-cost when unused

For a program that does not use runtime typing reflection:

- zero synthetic typing descriptor allocations;
- zero per-instance generic metadata;
- zero type checks on ordinary dispatch;
- zero selector-key changes;
- no expanded `Value` representation;
- no full static signatures added to every `MethodObject`;
- no descriptor cache traffic on ordinary object operations.

## 27.2 O(1) metadata observations

After metadata validation/load, operations such as:

```text
kind
argumentCount
parameterCount
fieldCount
constraintCount
```

should normally be indexed metadata reads plus class/context validation, with no global lock and no child-object allocation.

## 27.3 Lazy child reification

`parameterTypeAt(i)` / `argumentAt(i)` / `constraintAt(i)` reify at most the requested child chain required by that result.

Full convenience collections such as `.parameters`, `.arguments`, `.constraints`, and `.members` are explicitly allowed to allocate because the user requested the complete collection.

They must not be computed eagerly at parent descriptor creation.

## 27.4 No eager specialization copies

Specialized member reflection retains a compact environment/view. Repeated lookup of `Box<String>.value` should not clone whole signature trees.

## 27.5 Deep root reification

Reifying a root of a deeply nested TypeForm should allocate O(1) descriptor objects initially, not O(number of DAG nodes).

## 27.6 Result allocation boundary

Compiler, CLI, and LSP internal semantic queries allocate no VM result variants.

A runtime call such as:

```text
ctx.subtype(A, of: B)
```

may allocate one result/evidence projection because the user explicitly requested a runtime result.

## 27.7 Weak-cache metrics

Debug/performance builds should expose internal counters for:

- descriptor-cache hit/miss/stale-hit;
- root descriptors allocated;
- child descriptors allocated;
- full collection materializations;
- overlay node count/high-water;
- relation/validation budget consumption.

These counters are tooling metrics, not semantic behavior.

---

# 28. Exact implementation changes by repository area

This section describes the expected target seams. Exact line numbers may shift while Spec 01 is being implemented; named files/symbols are the authoritative resume points.

## 28.1 `phalcom-semantic`

### Modify / consume

```text
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/parameter.rs
phalcom-semantic/src/declarations.rs
phalcom-semantic/src/surface.rs
phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/identity.rs
phalcom-semantic/src/lib.rs
```

Most structural changes are owned by Specs 01/01.5 rather than Spec 03. This specification requires their published APIs to be consumable without cloning or VM coupling.

### Add

```text
phalcom-semantic/src/reflection.rs
```

or a focused equivalent under the eventual Spec 01 query tree.

It should define read-only projection structs/enums over:

- TypeForm handles/views;
- declaration generic signatures;
- callable/field signatures;
- specialized member views;
- relations;
- TypeUse/source facts.

It must not introduce another `TypeStore`, solver, hierarchy, or dispatch table.

## 28.2 `phalcom-core` universe source

### Create

```text
phalcom-core/core/universe/src/reflection/typing/package.ph
phalcom-core/core/universe/src/reflection/typing/kind.ph
phalcom-core/core/universe/src/reflection/typing/type-descriptor.ph
phalcom-core/core/universe/src/reflection/typing/type-parameter.ph
phalcom-core/core/universe/src/reflection/typing/generic-signature.ph
phalcom-core/core/universe/src/reflection/typing/signature.ph
phalcom-core/core/universe/src/reflection/typing/type-use.ph
phalcom-core/core/universe/src/reflection/typing/result.ph
phalcom-core/core/universe/src/reflection/typing/evidence.ph
phalcom-core/core/universe/src/reflection/typing/context.ph
```

### Modify

```text
phalcom-core/core/universe/src/reflection/package.ph
```

Add `expose .typing` and, if the package convention requires it, matching import/export declarations.

Do not modify `Behavior` with a parallel type hierarchy. If convenience type selectors are added to `Behavior`, they must delegate to the typing registry/context and preserve existing attribute methods.

## 28.3 `phalcom-modules`

Modify:

```text
phalcom-modules/src/builtin.rs
```

Update `UNIVERSE_NODES`:

- add `"typing"` as a child of `reflection`;
- add a package node for `reflection/typing`;
- add child module nodes matching the actual source files.

Builtin provider interface/source tests must prove the new nodes are discoverable and deterministic.

## 28.4 `phalcom-core` heap/runtime

Modify:

```text
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/mod.rs
phalcom-core/src/heap/accessors.rs
phalcom-core/src/heap/trace.rs
phalcom-core/src/value/mod.rs
```

Add the single boxed `TypingObject` family from Spec 02. Ensure `Value::class` resolves via `TypingObject.class`.

The tracer must be exhaustive and must not trace weak descriptor-cache entries.

## 28.5 VM/runtime registry

Create or add focused modules under:

```text
phalcom-core/src/modules/typing_registry.rs
```

or an equivalent ownership location selected during Spec 02 implementation.

Modify the VM struct to own the registry and method semantic side table. Do not place it behind a process-global singleton.

## 28.6 Method bridge

Do **not** add full generic fields to:

```text
phalcom-core/src/method/object.rs
```

Instead install/remove method semantic mappings when methods are materialized/replaced according to Spec 02 §20.

Extend:

```text
phalcom-core/src/primitive/method.rs
```

only with small convenience bridges if desirable; the primary rich lookup stays on `TypingContext.signatureOf` so availability/capability remains explicit.

## 28.7 Runtime primitives

Create:

```text
phalcom-core/src/primitive/typing/
```

with focused files for context, descriptors, signatures, relations, results, and runtime validation.

Update primitive registration/native surface catalog according to current project conventions.

## 28.8 Native metadata

Update native type/signature metadata only after Spec 01.5 native convergence shapes are available. Do not encode obsolete parameter defaults, finite-set bounds, or old TypeParameter-owned constraints just to register reflection primitives.

---

# 29. Dependency-ordered implementation units

The implementation should not be attempted as one giant “typing reflection” change.

## C0 — Readiness/rebase gate

**Depends on:** stable Spec 01 semantic-core contracts, enough Spec 01.5 implementation to publish canonical generic/type-lambda/signature products, and Spec 02 B1–B5 metadata/registry substrate.

Before implementation:

- rebase against current `main`;
- inspect changes made by Spec 01 agent to snapshots, relation outcomes, IDs, cancellation, and SemanticDb;
- update exact symbol names in this plan rather than building compatibility copies;
- verify Spec 02 registry/metadata implementation if it has already landed.

**Stop condition:** if canonical callable IDs/generic signatures/type-lambda metadata are not available, do not invent reflection-only substitutes.

## C1 — Reflection typing package, capabilities, and sealed result variants

**Files:**

```text
phalcom-core/core/universe/src/reflection/package.ph
phalcom-core/core/universe/src/reflection/typing/*
phalcom-modules/src/builtin.rs
phalcom-core/src/primitive/typing/result.rs
phalcom-core/src/primitive/typing/context.rs
```

Implement:

- package/module discovery;
- `Typing`, `TypingContext` shells;
- capability identities/restriction;
- `TypingResult` sealed variants;
- `TypeRelationResult` sealed variants;
- `MemberLookupResult` sealed variants;
- singleton no-payload variants.

**Tests first:** impossible payload combinations cannot be constructed; cancelled/budget are distinct classes; capabilities cannot be forged/escalated; builtin typing package loads deterministically.

**Acceptance:** no semantic descriptor object is required yet except a minimal context/result bridge.

## C2 — Kind and TypeForm runtime descriptors

**Depends on:** Spec 02 runtime metadata loader + context/weak cache.

Implement:

- `KindDescriptor`, `AtomicKind`, `FunctionKind`;
- `Type` singleton;
- `TypeDescriptor` base;
- `AppliedType`, structural descriptor classes, `SpecialType`, `SelfType` as available;
- common TypeForm selectors;
- indexed child access;
- weak descriptor identity.

**Tests first:**

- `Int.kind === Type`;
- `List.kind` displays `Type -> Type`;
- nominal reification `===` existing class object;
- no nominal wrapper;
- deep root allocates one descriptor;
- `argumentAt(0)` allocates only requested child;
- descriptor GC and stale weak cache recover cleanly through `try_get`;
- no core class/metaclass invariant changes.

## C3 — Generic signatures, type parameters, constraints, and type lambdas

Implement:

- `TypeParameter` owner/index/name/kind/variance/constraint access;
- `GenericSignature`;
- `GenericConstraint` subtype/equivalence operands;
- `TypeLambda` alpha-normalized descriptor and body/parameter reflection;
- type-lambda application gateway;
- free vs remaining parameters.

**Tests first:**

- declaration/method/lambda same-name binders remain distinct;
- lambda alpha equivalence;
- no method/lambda variance;
- lower bound operand direction preserved;
- no default field;
- no finite-set constraint;
- beta application canonicalizes to same semantic result as compiler form;
- no `Closure` allocation for type lambdas.

## C4 — Declaration/callable/field reflection and method side table

**Depends on:** Spec 01.5 canonical signature registry and Spec 02 method semantic index.

Implement:

- declaration generic/supertype queries;
- `CallableSignature` and `CallableParameter`;
- mandatory `parameterTypeAt` fast path;
- `FieldSignature`;
- `signatureOf(method)` O(1) bridge;
- generated declaration exact-type preservation after corresponding 01.5 compiler work.

**Tests first:**

- source method signature;
- native method signature;
- generated getter/setter signature;
- generic method signature;
- replaced old method retains its own mapping;
- dynamically installed untyped method does not borrow previous signature;
- stripped profile returns unavailable;
- MethodObject size/regression gate shows no full static signature was embedded.

## C5 — Type-form construction, relations, and static member lookup

Implement:

- `apply`, union/tuple/closed-record/callable construction;
- semantic relation projections;
- `MemberLookupResult`;
- generic specialized member views;
- `superFrom` lookup;
- access checks;
- budgets/cancellation.

**Tests first:**

- kind mismatch;
- too many args;
- partial application;
- constraint failure/defer;
- variance-aware subtype;
- `DynamicBoundary` not satisfied;
- cancelled and budget distinct;
- specialized `Box<String>` member type;
- inherited generic substitution;
- `super` specialization;
- private member authority.

## C6 — TypeUse and semantic presentation facts

**Depends on:** ToolingDebug occurrence metadata from Spec 02 and formal provenance from semantic pipeline.

Implement:

- `typeUseAt`;
- `typeUsesOf`;
- separate value type and denotation;
- source spelling/range when retained;
- inference/evidence provenance adapters.

**Tests first:** source occurrence with class-object value type + nominal denotation; omitted metadata => unavailable; unresolved/invalid => proper states; no advisory LSP fact enters result.

## C7 — Runtime `matches`, deep `validate`, and explicit construction

Implement:

- nominal cheap matches;
- erased applied generic dynamic boundary;
- bounded standard-container validation where sound;
- cycle/element/depth budgets;
- explicit `construct` live-world bridge;
- world reconciliation/access control.

**Tests first:**

- Int/Number nominal relation;
- List<Int> erased cheap match boundary;
- bounded list deep validation;
- mutable list validation creates no permanent token;
- cyclic container terminates;
- unsupported user generic boundary;
- validation cancellation/budget;
- construct uses origin class normal constructor;
- no descriptor forwarding to class-side methods;
- no ambient generic context inside constructor.

## C8 — Compiler/CLI/LSP/REPL adapters

Implement only adapters to formal semantic snapshot APIs.

Acceptance:

- no VM descriptor object construction in compiler/LSP;
- CLI and LSP formal relation/signature answers agree for same snapshot;
- RuntimePublic stripping does not affect source checker capability;
- advisory LSP `ValueShape` remains labeled separately.

## C9 — Contracts/proofs reflection gate

Wait for revised Spec 05.

Only after that spec fixes result/trust/artifact semantics should `contractsOf` / `proofsOf` project those products.

Do not resurrect the old Spec 03 proof status bag as an interim API.

## C10 — Fuzzing, GC, security, and performance hardening

Required:

- hostile metadata/descriptor graph fuzzing;
- weak-cache stale-entry stress;
- GC stress with descriptor/context lifetimes;
- deep/wide relation budget tests;
- runtime validation cycle/depth tests;
- object-model invariant suite;
- benchmark zero-reflection overhead;
- benchmark indexed access versus full materialization;
- code search proving rejected APIs/representations absent.

---

# 30. Verification and acceptance matrix

## 30.1 Object-model invariants

- Reifying `Int` returns existing `Int` class object.
- No `NominalType` wrapper exists.
- Generic applications create no class/metaclass.
- `value.class` never changes because a TypeForm was reflected or validated.
- Descriptor classes obey ordinary metaclass wiring.
- `Type` is an `AtomicKind`, not a `Class` and not a TypeForm classified by itself.
- `Value` remains its existing compact representation.

## 30.2 TypeForm laws

- `.display` does not participate in semantic equality.
- `.freeParameters` and `.remainingParameters` differ according to Spec 01.5 definitions.
- `.equivalentTo` is alpha-invariant for type lambdas.
- partial applications reflect residual kinds/remaining slots correctly.
- applied origin/argument flattening matches canonical application law.

## 30.3 Result laws

- only `RelationSatisfied` establishes a positive relation;
- `RelationDynamicBoundary` is never implicitly true;
- cancelled and budget-exceeded are separate variants;
- member lookup uses `MemberLookupResult`, never relation-result payload abuse;
- unavailable metadata is distinct from unknown semantic information;
- internal failure never fabricates a semantic value.

## 30.4 Lazy allocation laws

- ordinary execution allocates zero typing descriptors when unused;
- root reification of deep graph allocates O(1) descriptors initially;
- count/index getters do not materialize full child collections;
- `parameterTypeAt(i)` does not allocate unrelated parameters;
- full convenience tuple APIs allocate only on explicit request;
- weak cache does not root synthetic descriptors;
- stale weak entry uses `Heap::try_get` and never panics.

## 30.5 Method/signature laws

- `MethodObject` receives no complete generic static signature payload;
- `signatureOf(method)` uses exact method-handle side table;
- method replacement does not mutate semantic identity of an already reified old method;
- dynamically installed untyped methods cannot steal prior metadata;
- static selector identity remains type-independent.

## 30.6 Runtime matching laws

- nominal runtime class evidence may satisfy nominal match;
- erased generic origin alone cannot satisfy an applied generic match;
- deep validation is explicit and bounded;
- successful validation installs no runtime generic token;
- validation of mutable state is not permanent typing evidence;
- unsupported deep validation returns boundary/blocked, never success.

## 30.7 Static/live world laws

- pinned static semantic queries remain stable after runtime method mutation;
- live `respondsTo`/`methodFor` may differ from static `member` truthfully;
- live invocation/construction reconciles world state before relying on static premises;
- `refresh` creates a new context rather than mutating old descriptors.

## 30.8 Cross-tool laws

For the same published semantic snapshot:

- compiler, CLI, and LSP formal signatures agree;
- compiler/LSP do not need runtime metadata/descriptors;
- runtime reflection of emitted metadata agrees with the published source semantic form for retained facts;
- advisory LSP facts are never promoted to runtime TypeForms.

---

# 31. Rejected alternatives and failure modes

## 31.1 Wrapping every class in a nominal descriptor

Rejected because it duplicates class identity, adds allocations/cache edges, and makes `Int` versus “type of Int” runtime identity confusing.

Direct nominal reification is both cheaper and semantically cleaner.

## 31.2 Adding semantic type IDs to `Value`

Rejected because `Value` is a hot representation used by every runtime operation. A rare reflection feature cannot justify a permanent cost to all values.

## 31.3 Storing static signatures in every MethodObject

Rejected because it bloats every method, including programs/builds where static reflection is stripped or unused. The VM side table is sparse and profile-dependent.

## 31.4 Strong immortal descriptor cache

Rejected because arbitrary runtime TypeForm construction could retain an unbounded graph for VM lifetime. Context-local weak canonicalization preserves convenient live identity without memory leaks.

## 31.5 Status bags instead of variants

Rejected because they permit impossible states such as `status=#known` with no value or a rejected relation carrying dynamic-boundary payload simultaneously. Sealed variants make invalid combinations unrepresentable.

## 31.6 Treating cancellation/budget as blocked

Rejected because those are operational terminal states with different retry and observability policies. They are not missing semantic knowledge.

## 31.7 One `TypeRelationResult` for every query

Rejected. Member lookup, typing lookup, and binary type relations have distinct positive payloads and failure meanings. Shared status philosophy does not require one lossy universal result class.

## 31.8 Exposing arbitrary substitution

Deferred because it unnecessarily exposes compiler-internal binder-environment machinery and invites a second user-programmable type transformation layer. Application/type lambdas/specialized lookup cover the current use cases.

## 31.9 Treating runtime method mutation as invalidating static truth

Rejected because a semantic snapshot remains a valid description of the program version that produced it. Static and live runtime reflection answer different questions; world checking belongs at bridge operations.

## 31.10 Inferring `List<Int>` from `List` runtime class

Rejected as unsound under erasure. Runtime class identity proves only the nominal origin. Deep validation is explicit and state-specific.

---

# 32. Cross-spec amendments

## 32.1 Spec 01

Spec 01 must provide or preserve:

- compiler-owned relation outcomes with cancellation/budget distinction;
- published immutable snapshots/query facade;
- stable declaration/callable/field identities;
- source-owned diagnostics;
- semantic generation/store ownership needed by reflection result provenance.

Spec 03 does not dictate Spec 01's internal DB representation.

## 32.2 Spec 01.5

Spec 01.5 remains authoritative for:

- TypeForm/kind semantics;
- type lambdas;
- free/remaining parameters;
- constraints/variance;
- generic inheritance;
- `Self`;
- callable/field canonical declaration model;
- lazy specialization;
- runtime erasure.

This document merely gives those products a runtime projection.

## 32.3 Spec 02

Revised Spec 02 remains authoritative for:

- metadata schema;
- profiles;
- root-driven export;
- immutable runtime metadata pools;
- `RuntimeTypingRegistry`;
- context overlay;
- `Object::Typing` physical representation;
- weak cache;
- method semantic side table;
- lazy descriptor reification.

This document decides which **surface classes/selectors/results** users see on top of that substrate.

## 32.4 Spec 04

Spec 04 remains authoritative for source type-form syntax and the contextual boundary between annotations and runtime TypeForm expressions.

Runtime `TypeLambda.<>(...)` and reflected generic application must agree with Spec 04 semantic lowering; neither can redefine parser/application precedence.

## 32.5 Revised Spec 05

Spec 05 must consume, not redefine, the capabilities/result principles here for:

- effect reflection;
- contract reflection;
- totality results;
- proof/trust/artifact reflection;
- future row-kind descriptors.

Spec 05 must use sealed honest terminal states and cannot collapse proof budget/cancellation into proof success.

## 32.6 Spec 06

Rationale must be updated to reflect:

- `KindDescriptor` / `FunctionKind` naming;
- no nominal wrapper;
- sealed result variants;
- dedicated member lookup result;
- cheap matches versus deep validation;
- static snapshot truth versus live runtime mutation;
- no public arbitrary substitution in the initial reflection API.

## 32.7 Spec 07

The consolidated implementation plan must replace its old G1/G2 reflection task cards with C0–C10 or equivalent dependencies from this revision and the revised Spec 02.

---

# 33. Migration and deletion ledger

| Transitional/old design | Required final state | Delete/forbid when |
|---|---|---|
| old `Object::SemanticDescriptor` concept | Spec 02 `Object::Typing` family | typing heap arm lands |
| `Kind`/`ArrowKind` descriptor names | `KindDescriptor`/`FunctionKind` | reflection package API lands |
| `displayName` | `.display` | common TypeForm/Kind API lands |
| kind `.parameters` | kind `.arguments` | kind API lands |
| common TypeForm `.declaration`/`.typeParameters` | declaration-specific query APIs | reflection API lands |
| common semantic `.hash` requirement | internal semantic fingerprints only | reflection API lands |
| `TypeParameter.upperBound` | derived signature constraints | generic reflection lands |
| `TypeParameter.default` | absent until generic defaults designed | generic reflection lands |
| status/optional-payload `TypingResult` | sealed variants | C1 |
| four-state `TypeRelationResult` | full seven-state sealed variants | C1 |
| cancellation/budget inside blocked | dedicated variants | C1 |
| member query returning relation result | `MemberLookupResult` | C5 |
| public `applyKind` | no core public operation | C5 |
| public `normalize` | successful forms canonical by construction | C5 |
| public arbitrary `substitute` | internal lazy specialization only | C5 |
| static queries blocked solely by runtime world mutation | pinned static truth; bridge-specific reconciliation | C5/C7 |
| top-level universe `typing` package proposal | `reflection/typing` package | C1 |
| strong synthetic descriptor cache | context-local weak cache | Spec 02 B5/C2 |
| nominal TypeDescriptor wrapper | reuse class object | C2 |
| static signature fields in MethodObject | sparse method semantic side table | Spec 02 B7/C4 |
| runtime generic tokens | forbidden | permanent invariant |
| `Type.currentApplication` | explicit TypingContext | permanent invariant |

Deletion is part of completion. A phase report must include repository search evidence for obsolete API names/representations it claims removed.

---

# 34. Completion gate

Spec 03 is implemented only when all of the following are true:

1. the public typing reflection package loads through the existing builtin reflection package;
2. nominal forms reify to existing class objects;
3. synthetic forms use lazy context-backed descriptors and weak canonicalization;
4. `Type`, kinds, applied forms, type parameters, type lambdas, generic constraints/signatures, callable/field signatures, and TypeUse expose the exact ratified vocabulary;
5. full collection access is optional convenience while indexed access is available for hot reflective inspection;
6. result APIs are sealed variants with cancellation and budget exhaustion distinct;
7. member lookup has its own result family;
8. static relations/member lookup delegate to compiler-owned/shared semantics rather than duplicate them in the VM;
9. `signatureOf(method)` uses the sparse exact method side table;
10. runtime typed-dispatch libraries can inspect signature parameter types without eager graph allocation;
11. `matches` never invents erased generic evidence;
12. `validate` is explicit, capability-checked, bounded, and leaves no permanent type token;
13. explicit `construct` crosses to the existing runtime constructor/authority path without forwarding descriptor methods or installing ambient generic context;
14. runtime world mutation does not rewrite pinned static semantic truth;
15. compiler, CLI, LSP, and REPL formal consumers do not require runtime descriptors;
16. ordinary non-reflective runtime benchmarks show no meaningful regression attributable to typing reflection infrastructure;
17. GC stress confirms descriptor/context edges are correct and weak caches tolerate stale handles;
18. the migration/deletion ledger is satisfied or every retained transitional path has an explicit owner and deletion gate;
19. no code path changes selector identity, class/metaclass identity, object layout, `Value` representation, or ordinary dispatch because of static typing;
20. cross-spec terminology matches Specs 01.5, revised 02, and revised 04.

The final architecture is therefore:

```text
compiler-owned canonical semantics
              │
              │ publish/export
              ▼
    immutable semantic metadata
              │
              ▼
      RuntimeTypingRegistry
              │
              │ explicit context
              ▼
         TypingContext
        /      |       \
       /       |        \
observe    relate     construct/validate
  │            │             │
  ▼            ▼             ▼
lazy         sealed        explicit live
forms         results       runtime boundary

ordinary execution ───────────────────────────────► unchanged
```

That is the intended Phalcom model: **deep static semantic information, first-class reflection when requested, and no compulsory runtime tax for information that ordinary execution does not need.**

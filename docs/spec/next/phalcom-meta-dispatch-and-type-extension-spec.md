# Phalcom Meta-Dispatch and Type-Driven Extension Specification

**Status:** Proposed normative specification  
**Audience:** Language implementers, standard-library authors, compiler and VM developers, tool authors  
**Scope:** Multimethods; predicate and value dispatch; protocol instances; reflective pattern matching; capability composition through richer mixins; units and dimensions as first-class types

---

## Contents

1. Shared foundations
2. Multimethods
3. Predicate and value dispatch
4. Protocol instances
5. Reflective pattern matching
6. Capability composition and richer mixins
7. Units and dimensions as first-class types
8. Cross-feature integration
9. Compiler, runtime, and VM obligations
10. Required diagnostics
11. Conformance and acceptance tests
12. Recommendations, deferred questions, and integrated examples

---

## 1. Purpose

This specification defines a coherent family of facilities that allow Phalcom programs to extend the language through Phalcom itself without weakening the language's selector-based object model or the soundness of its reflective type system.

The facilities share one foundation:

1. types are first-class runtime objects;
2. generic specialization produces canonical `SpecializedType` values;
3. selector identity includes arity, positional structure, and labels;
4. protocols are first-class structural capability descriptors;
5. reflection exposes methods, parameters, annotations, specializations, mixins, patterns, and dispatch cases;
6. tuples represent ordered heterogeneous products;
7. records represent labeled heterogeneous products;
8. maps provide dynamic keyed registries and caches;
9. spread arguments preserve the positional and labeled lanes of a call;
10. open extension is scoped and diagnosable rather than process-global and invisible.

The specification deliberately avoids six independent mini-frameworks. Multimethod resolution, protocol-instance resolution, pattern specificity, mixin composition, and unit algebra reuse the same type relations, normalization rules, reflection model, diagnostics, and canonicalization machinery.

---

## 2. Normative language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** are normative.

Examples use proposed Phalcom syntax. Where a surface form depends on future parser work, the specification also provides a stable ordinary-object API.

---

## 3. Shared foundations

### 3.1 Selectors

A selector is determined by its base name and complete argument shape.

These are distinct selectors:

```phalcom
collide(_,_)
collide(left:right:)
collide(_,right:)
collide(*)
```

Generic specialization is represented by reserved selectors:

```text
<>(_)
<>(_,_)
<>(_,_,_)
```

A declared generic type with `N` generic parameters receives exactly one compiler/runtime-defined specialization selector with `N` positional generic arguments.

User code MUST NOT define, replace, remove, intercept, or synthesize any `<>(...)` selector.

### 3.2 `SpecializedType`

Generic application returns a canonical `SpecializedType`:

```phalcom
Var<Int>.class
// SpecializedType

Var<Int>.origin
// Var

Var<Int>.arguments
// const (Int,)
```

Conceptually:

```phalcom
@data
@immutable
class SpecializedType is Type {
  const _origin: TypeConstructor
  const _arguments: Tuple<Type>
  const _substitution: Map<TypeParameter, Type>
}
```

Equivalent specializations MUST be canonical:

```phalcom
Var<Int> === Var<Int>
// true
```

A specialized instance remembers the specialization through which it was constructed:

```phalcom
const x = Var<Int>(#x)

x.class
// Var

x.type
// Var<Int>
```

`class` identifies allocation and ordinary implementation ownership. `type` identifies the runtime type, including specialization provenance.

### 3.3 Type relations

The shared type-relation service MUST support at least:

```phalcom
left.equivalentTo(right)
left.subtypeOf(right)
left.overlaps(right)
left.disjointWith(right)
left.join(right)
left.meet(right)
```

Open-world questions MUST permit an `Unknown` result. Failure to prove overlap is not proof of disjointness.

The relation service SHOULD produce structured explanations or proof objects rather than only booleans.

### 3.4 Variance

Generic variance is declared using:

```phalcom
+T    // covariant
-T    // contravariant
T     // invariant
```

Unary `+` and `-` remain ordinary overridable operators on ordinary objects. Their meaning on compiler-created `TypeParameter` descriptors is reserved and final. Variance is recorded as immutable metadata when the declaration is compiled; the type checker MUST NOT dynamically invoke user methods to recover variance later.

### 3.5 Protocols

An `@protocol class` declaration creates a first-class structural descriptor, not an instantiable class.

```phalcom
@protocol
class Encoder<in T, out Output> {
  encode(value: T) -> Output
}
```

Protocols define substitutability. They do not automatically provide implementation, storage, inheritance, or method installation.

### 3.6 Reflection

The following information MUST be reflectable where applicable:

- selector;
- parameter positions and labels;
- rest captures;
- declared parameter and result types;
- generic parameters and variance;
- specialized origin and arguments;
- protocol requirements;
- multimethod cases;
- value patterns and guards;
- instance declarations;
- mixin requirements, provisions, conflicts, and contributions;
- pattern accepted types and captures;
- units, dimensions, conversion rules, and normalization metadata.

### 3.7 Stable versus sugared APIs

Every facility in this specification MUST expose an ordinary-object API that remains stable independently of parser sugar.

Examples:

```phalcom
multimethod.invoke(*positional, **labeled)
Instances.resolve(Encoder<User, Json>)
Match.on(value).with(*cases)
Formula.all(*terms)
quantity.convert(to: Meter)
```

Surface syntax MAY lower to these APIs or to optimized equivalent VM operations.

---

# Part I — Multimethods

## 4. Overview

A multimethod is a first-class callable object whose implementation is selected from an ordered set of applicable cases using several arguments, their runtime types, optional exact-value constraints, and optional symbolic predicates.

Multimethods extend dispatch without modifying the method tables of participating classes.

## 5. Declaration

A multimethod uses class-shaped syntax but produces a `MultiMethod` descriptor rather than an ordinary class:

```phalcom
@multimethod
class collide {
  @case
  call(left: Circle, right: Circle) -> Collision {
    return circleCircle(left, right)
  }

  @case
  call(left: Circle, right: Rectangle) -> Collision {
    return circleRectangle(left, right)
  }

  @case
  call(left: Shape, right: Shape) -> Collision {
    return Collision.none
  }
}
```

The name `collide` binds to a first-class `MultiMethod` value.

```phalcom
collide.class
// MultiMethod

collide.selector
// #call(_,_)
```

Calling:

```phalcom
collide(circle, rectangle)
```

is equivalent to:

```phalcom
collide.invoke(circle, rectangle)
```

## 6. Case descriptors

Each `@case` method produces an immutable descriptor:

```phalcom
@data
@immutable
class MultiCase {
  const _owner: MultiMethod
  const _method: Method
  const _callableType: CallableType
  const _typeSignature: Tuple<Type>
  const _labeledTypes: Record<Type>
  const _valuePattern: Option<DispatchPattern>
  const _guard: Option<Formula>
  const _priorityClass: CasePriorityClass
  const _originModule: Module
}
```

The method body remains ordinary Phalcom code. Case selection occurs before the body is invoked.

## 7. Open extension and scope

Third-party modules MAY add cases:

```phalcom
@casesFor(collide)
module polygon_collision {
  @case
  call(left: Polygon, right: Circle) -> Collision {
    ...
  }
}
```

Case openness is scoped.

A call site sees:

1. cases declared with the multimethod;
2. cases declared in the current module;
3. cases from directly imported case-providing modules;
4. explicitly re-exported cases.

Transitive imports MUST NOT silently activate cases unless the intermediate module explicitly re-exports them.

A dispatch environment is therefore part of resolution:

```phalcom
@data
@immutable
class DispatchEnvironment {
  const _module: Module
  const _activeCaseModules: Set<Module>
  const _version: Int
}
```

Caches MUST include the dispatch environment or an equivalent version token.

## 8. Applicability

A case is applicable when all of the following hold:

1. its selector shape accepts the call's positional and labeled arguments;
2. every runtime argument type is compatible with the corresponding declared dispatch type;
3. every exact-value or structural dispatch pattern matches;
4. its guard evaluates to true;
5. any declared protocol requirement is satisfied.

Dispatch MUST NOT invoke the case body merely to determine applicability.

## 9. Specificity

Case specificity is a partial order.

Case `A` is more specific than case `B` when:

1. every type region accepted by `A` is a subset of the corresponding region accepted by `B`;
2. every value pattern accepted by `A` is a subset of the corresponding value pattern accepted by `B`;
3. `A`'s guard implies `B`'s guard, where that implication is provable;
4. at least one relation is strict.

For ordinary nominal types:

```text
(Circle, Rectangle) < (Shape, Shape)
```

For protocol types, structural conformance participates in the same relation engine.

Declaration order MUST NOT resolve incomparable maximal cases.

## 10. Ambiguity

If two or more maximal applicable cases are incomparable, invocation fails:

```text
× Ambiguous multimethod dispatch

  Multimethod:
    collide(_,_)

  Runtime argument types:
    (Capsule, Asteroid)

  Applicable maximal cases:
    collide(PhysicalBody, Asteroid)
    collide(Capsule, PhysicalBody)

  Neither case is more specific than the other.
```

The diagnostic SHOULD include a witness region or explanation of the overlap.

## 11. Fallback cases

A case MAY be marked as a fallback:

```phalcom
@fallback
@case
call(left: Any, right: Any) {
  ...
}
```

A fallback participates only when no nonfallback case is applicable. Multiple applicable fallback cases are still subject to ambiguity rules.

Fallback MUST NOT be represented by arbitrary declaration priority.

## 12. Dispatch caching

Implementations SHOULD cache successful resolution using a key including:

```phalcom
(
  multimethod: collide,
  environment: environment.version,
  positionalTypes: (Circle, Rectangle),
  labeledTypes: (),
  relevantValues: valueFingerprint
)
```

Exact values and predicates may make full caching impossible. A case descriptor MUST declare the value information relevant to its applicability so the runtime does not accidentally cache an invalid result.

## 13. Reflection

```phalcom
collide.cases
collide.casesIn(environment)
collide.resolve(types: (Circle, Rectangle))
collide.explain(circle, rectangle)
```

A resolution explanation SHOULD show rejected and accepted candidates.

## 14. Negative cases

The implementation MUST reject:

- duplicate equivalent cases in one scope;
- cases with incompatible selectors;
- non-type dispatch annotations where a type is required;
- opaque guards where a statically analyzable guard is required;
- incomparable cases declared as unconditionally coherent;
- case installation into a frozen environment.

---

# Part II — Predicate and Value Dispatch

## 15. Value dispatch

A multimethod case may constrain arguments to literal or structural values:

```phalcom
@case(values: (#GET, "/health", _))
call(method: Symbol, path: String, request: Request) -> Response {
  return HealthResponse.ok
}
```

The `_` in a dispatch pattern means “no additional value constraint.” It is pattern notation, not generic wildcard specialization.

Equivalent stable API:

```phalcom
route.defineCase(
  values: DispatchPattern.tuple(
    DispatchPattern.literal(#GET),
    DispatchPattern.literal("/health"),
    DispatchPattern.any
  ),
  implementation: ...
)
```

Value dispatch MUST use value semantics explicitly defined by the pattern. It MUST NOT assume arbitrary `==` methods return `Bool`.

## 16. Supported dispatch patterns

The initial standard library SHOULD provide:

```text
AnyPattern
LiteralPattern
TypePattern
TuplePattern
RecordPattern
VariantPattern
ProtocolPattern
SetMembershipPattern
RangePattern
```

Examples:

```phalcom
@case(values: ((#POST, "/orders"), _))
```

```phalcom
@case(values: ((status: 404, ...),))
```

```phalcom
@case(values: (Range.closed(0, 9),))
```

## 17. Predicate guards

A case may include a symbolic guard:

```phalcom
@case
@when(
  Case.arg(#order).field(#total) > Money.usd(100)
  and Case.arg(#destination).field(#country) == #US
)
call(order: Order, destination: Destination) -> Money {
  return Money.zero(#USD)
}
```

Because symbolic `and` and symbolic equality may not yet be integrated into the language, the stable form is:

```phalcom
@when(
  Formula.all(
    Case.arg(#order).field(#total).gt(Money.usd(100)),
    Case.arg(#destination).field(#country).eq(#US)
  )
)
```

A guard used for static specificity MUST be a reified `Formula`.

An opaque executable predicate MAY be supported explicitly:

```phalcom
@whenOpaque { order, destination =>
  externalService.approves(order, destination)
}
```

Opaque guards:

- MAY determine runtime applicability;
- MUST NOT be used to claim static disjointness or implication;
- MAY force ambiguity checks to remain runtime-only;
- MUST be reported as opaque through reflection.

## 18. Guard evaluation

Guards are evaluated only after selector, type, and value-pattern filtering.

A guard MUST be side-effect-free to participate in cached dispatch. The runtime MAY refuse caching when purity cannot be established.

A guard exception is a dispatch error, not a rejected case:

```text
× Multimethod guard failed

  Case:
    calculateShipping(Order, Destination)

  Guard:
    external approval predicate

  Cause:
    NetworkTimeout
```

## 19. Predicate specificity

If guard `P` provably implies guard `Q`, then `P` is at least as specific as `Q` for otherwise equal case signatures.

```text
x > 100  implies  x > 0
```

If implication is unknown, the cases are not ordered by guards.

An explicit user priority MUST NOT be allowed to convert an unknown logical relation into a typing guarantee. A separate runtime-only ordered dispatcher MAY exist, but it is not a coherent multimethod under this specification.

## 20. Exhaustiveness

For sealed or finite domains, tools MAY analyze whether cases cover the dispatch domain.

```phalcom
route.coverage(
  domain: (HttpMethod, Path)
)
```

Coverage results MUST be one of:

```text
Complete
Incomplete(counterexample or residual pattern)
Unknown(reason)
```

---

# Part III — Protocol Instances

## 21. Purpose

A protocol instance provides an external implementation of a protocol for a type combination without modifying the participating types and without establishing inheritance.

Protocol instances support typeclass-like behavior while preserving Phalcom's first-class protocols and scoped extension model.

## 22. Declaration

```phalcom
@protocol
class Encode<T, Format> {
  encode(value: T, format: Format) -> Bytes
}
```

```phalcom
@instance(Encode<User, Json>)
class UserJsonEncoding {
  encode(value: User, format: Json) -> Bytes {
    ...
  }
}
```

The class-shaped declaration creates an implementation object and an `InstanceDescriptor`.

It does not:

- inject methods into `User`;
- modify `Encode`;
- establish subclassing;
- make conformance nominal;
- activate globally merely because the package is installed.

## 23. Resolution

Stable API:

```phalcom
const encoder =
  Instances.resolve(Encode<User, Json>)

const bytes =
  encoder.encode(user, Json.standard)
```

A protocol MAY expose forwarding sugar:

```phalcom
Encode<User, Json>.instance
```

but resolution semantics belong to `Instances` and the active instance environment.

## 24. Instance environment

Instances use the same scoped-openness model as multimethod cases.

Visible instances include:

1. instances defined with the protocol;
2. instances defined in the current module;
3. instances from directly imported instance modules;
4. explicitly re-exported instances.

Transitive activation is forbidden unless explicitly re-exported.

## 25. Coherence

For any requested protocol specialization and active environment, resolution MUST produce at most one maximal applicable instance.

If multiple incomparable maximal instances apply, resolution fails:

```text
× Ambiguous protocol instance

  Requested:
    Encode<User, Json>

  Applicable instances:
    GenericDataJsonEncoding<T>
    UserJsonEncoding

  The resolver could not prove that one instance is more specific.
```

## 26. Generic instances

```phalcom
@instance(Encode<List<T>, Json>)
class ListJsonEncoding<T>
  @requiresInstance(Encode<T, Json>) {

  encode(value: List<T>, format: Json) -> Bytes {
    const elementEncoder =
      Instances.resolve(Encode<T, Json>)

    ...
  }
}
```

A generic instance has:

- a protocol pattern;
- generic parameters;
- type constraints;
- required subordinate instances;
- an implementation factory.

Resolution solves the instance's type constraints and recursively resolves requirements.

Cycles MUST be detected and diagnosed.

## 27. Orphan policy

A normal instance SHOULD be declared in a module that owns at least one of:

- the protocol;
- a nominal head type in the protocol arguments.

An orphan instance MAY be declared explicitly:

```phalcom
@localInstance(Encode<ExternalUser, ExternalFormat>)
class LocalExternalEncoding {
  ...
}
```

A local orphan instance:

- participates only in the declaring module and explicit importers;
- MUST NOT be re-exported implicitly;
- SHOULD trigger a warning when exported publicly;
- MUST be reflected as orphaned.

This policy reduces accidental ecosystem-wide conflicts without eliminating local adaptation.

## 28. Instance identity and lifetime

Stateless instances SHOULD be canonical singletons.

Stateful instances MAY be factories scoped to a container, request, or explicit context. Instance descriptors MUST declare lifetime:

```text
Singleton
Environment
Request
Transient
```

Resolution MUST NOT silently instantiate a stateful service as a process-global singleton.

## 29. Relationship to structural conformance

Structural protocol conformance answers:

> Does this object already provide the required selectors?

Protocol-instance resolution answers:

> Is there an external implementation object providing this protocol operation for these types?

These are distinct.

```phalcom
Encoder<User, Json>.conformedBy(User)
// possibly false

Instances.resolve(Encoder<User, Json>)
// may still succeed
```

## 30. Reflection

```phalcom
Instances.visibleFor(Encode<User, Json>)
Instances.explain(Encode<User, Json>)
instance.protocol
instance.arguments
instance.requirements
instance.originModule
instance.orphaned
instance.lifetime
```

## 31. Diagnostics

The resolver MUST distinguish:

- no instance exists;
- candidates exist but constraints fail;
- subordinate instance is missing;
- resolution cycle;
- ambiguity;
- inaccessible instance due to scope;
- instance implementation does not satisfy the protocol.

---

# Part IV — Reflective Pattern Matching

## 32. Purpose

Reflective pattern matching provides a library-level matching system based on first-class pattern objects. It is designed to support future native syntax without making the semantics dependent on parser changes.

## 33. Pattern protocol

```phalcom
@protocol
class Pattern<in Input, out Captures> {
  acceptedType -> Type
  captureType -> Type
  match(value: Input) -> Option<Captures>
  residualFrom(subject: Type) -> ResidualType
  describe -> String
}
```

A pattern is an ordinary immutable object. Matching does not mutate the subject.

## 34. Stable matching API

```phalcom
const result =
  Match.on(value).with(
    Case.of(
      Patterns.variant(
        Ok,
        value: Capture(#value)
      )
    ) { captures =>
      handle(captures.value)
    },

    Case.of(
      Patterns.variant(
        Err,
        error: Capture(#error)
      )
    ) { captures =>
      recover(captures.error)
    }
  )
```

Captures may be spread into labeled parameters:

```phalcom
Case.of(pattern) { **captures =>
  ...
}
```

The block's reflected callable type MUST be compatible with the pattern's capture record.

## 35. Standard patterns

The initial standard library SHOULD provide:

```text
AnyPattern
NeverPattern
LiteralPattern
TypePattern
ProtocolPattern
VariantPattern
TuplePattern
RecordPattern
CapturePattern
GuardedPattern
OrPattern
AndPattern
NotPattern
RestPattern
```

Examples:

```phalcom
Patterns.literal(0)
Patterns.type(Int)
Patterns.protocol(Serializable)
```

```phalcom
Patterns.tuple(
  Patterns.type(Int),
  Capture(#name, Patterns.type(String)),
  rest: Capture(#tail)
)
```

```phalcom
Patterns.record(
  name: Capture(#name, Patterns.type(String)),
  age: Capture(#age, Patterns.range(0, 150)),
  rest: Capture(#extra)
)
```

## 36. Variant patterns

Sealed variants SHOULD expose pattern constructors through reflection:

```phalcom
Patterns.variant(
  Result.Ok,
  value: Capture(#value)
)
```

The matcher MUST validate the variant's reflected fields and capture labels when the pattern is constructed.

## 37. Guarded patterns

```phalcom
Patterns.type(Int)
  .where(Match.value > 0)
```

Stable form:

```phalcom
Patterns.guard(
  Patterns.type(Int),
  Match.value.gt(0)
)
```

A symbolic guard contributes to exhaustiveness and specificity when its logical relation is decidable.

An opaque guard may be used for runtime matching but cannot prove coverage.

## 38. Case selection

Cases are tested in declaration order only after static validation confirms that ordering is intentional.

Two modes exist:

### 38.1 Ordered matching

The first matching case wins. Overlap is legal but SHOULD be diagnosed when a later case is provably unreachable.

### 38.2 Unordered exhaustive matching

Cases form a set. Overlapping incomparable cases are errors. This mode is appropriate for declarative rule tables and compiler analyses.

The selected mode MUST be explicit:

```phalcom
Match.ordered(value)
Match.exhaustive(value, subjectType: Result<Int, Error>)
```

## 39. Exhaustiveness and residual types

Each pattern may subtract an accepted region from the subject type.

```text
subject: Option<Int>
case Some<Int>  → residual None
case None       → residual Never
```

A match is exhaustive when the final residual is `Never`.

Results:

```text
Exhaustive
NonExhaustive(residual, witness)
Unknown(reason)
```

Arbitrary opaque guards MUST produce `Unknown`, not a false claim of exhaustiveness.

## 40. Unreachable cases

```phalcom
Match.ordered(value).with(
  Case.of(Patterns.type(Int)) { ... },
  Case.of(Patterns.range(1, 10)) { ... }
)
```

The second case is unreachable because its accepted set is contained by the first.

The diagnostic SHOULD provide the containment proof.

## 41. Pattern reflection

```phalcom
pattern.acceptedType
pattern.captureType
pattern.children
pattern.guard
pattern.explain(value)
pattern.residualFrom(subjectType)
```

## 42. Future native syntax

A future native `match` syntax MAY lower to the same `Pattern`, `Case`, and exhaustiveness model. Native syntax MUST NOT establish a second incompatible matching semantics.

---

# Part V — Capability Composition and Richer Mixins

## 43. Purpose

A richer mixin is a first-class capability-composition descriptor. It can contribute methods and namespaced state while declaring requirements, provisions, parameters, conflicts, and ordering constraints.

Mixins provide implementation reuse. Protocols provide substitutability. The two concepts remain distinct.

## 44. Mixin declaration

```phalcom
@mixin
class Cached<T> {
  @requires(Repository<T>, Clock)
  @provides(CachedRepository<T>, CacheMetrics)

  const _capacity: Int
  const _expiration: Duration

  @constructor
  configure(capacity: Int, expiration: Duration) {
    _capacity = capacity
    _expiration = expiration
  }

  get(id: Id) -> Option<T> {
    ...
  }
}
```

The declaration produces a `Mixin` descriptor. It is not independently instantiated as an ordinary domain object.

## 45. Composition

```phalcom
@with(
  Cached<User>(
    capacity: 1_000,
    expiration: Duration.minutes(5)
  ),
  Retried(maxAttempts: 3),
  Traced
)
class UserRepository is SqlRepository<User>
  is Repository<User> {
  ...
}
```

The compiler creates and validates a composition plan before the class becomes usable.

## 46. Mixin descriptor

```phalcom
@data
@immutable
class MixinDescriptor {
  const _name: Symbol
  const _typeParameters: Tuple<TypeParameter>
  const _requirements: Set<CapabilityRequirement>
  const _provisions: Set<CapabilityProvision>
  const _methods: Map<Selector, MethodContribution>
  const _state: RecordType
  const _conflicts: Set<CapabilityPattern>
  const _ordering: Set<CompositionOrdering>
}
```

## 47. Requirements

Requirements may be:

- protocols;
- nominal base classes;
- selectors;
- fields or state capabilities;
- subordinate mixins;
- protocol instances;
- constructor or lifecycle capabilities.

Examples:

```phalcom
@requires(Repository<T>)
@requiresSelector(#clock)
@requiresInstance(Encode<T, Json>)
```

Requirements are resolved against:

1. the base class;
2. the target class body;
3. earlier or dependency-ordered mixins;
4. visible protocol instances where explicitly requested.

## 48. Provisions

A mixin may declare that it provides:

- protocol conformance;
- method selectors;
- named capabilities;
- lifecycle hooks;
- instrumentation events.

A provision is a claim verified by the compiler. Merely writing `@provides(Repository<T>)` does not bypass conformance checking.

## 49. State isolation

Mixin state MUST be namespaced to avoid accidental slot collisions.

Conceptually:

```text
UserRepository storage
├── base::SqlRepository
├── mixin::Cached
│   ├── capacity
│   └── expiration
└── mixin::Retried
    └── maxAttempts
```

Ordinary unqualified field access inside a mixin resolves first to that mixin's state namespace, then to explicitly required host state.

Reflection MUST expose the final physical and logical layout.

## 50. Method conflicts

If two mixins contribute the same selector and neither contribution refines or delegates to the other, composition fails.

```text
× Mixin method conflict

  Target class:
    UserRepository

  Selector:
    get(_:)

  Contributions:
    Cached<User>.get(_:)
    AuditedRepository<User>.get(_:)
```

The class may resolve the conflict explicitly:

```phalcom
@resolve(
  #get(_:),
  using: Cached<User>
)
```

Or define an explicit combination:

```phalcom
@combine(#get(_:))
get(id: Id) {
  return AuditedRepository.call {
    Cached.call(id)
  }
}
```

Implicit “last mixin wins” behavior is forbidden.

## 51. Dependency ordering

Mixin dependencies form a directed graph. The composition planner MUST topologically order initialization and method availability.

Cycles fail:

```text
× Cyclic mixin dependency

  A requires B
  B requires C
  C requires A
```

Declaration order MAY break ties only between otherwise independent mixins. It MUST NOT override dependency ordering.

## 52. Lifecycle

Mixins MAY define controlled lifecycle hooks:

```phalcom
initializeMixin(context: CompositionContext) -> Unit
finalizeMixin(context: CompositionContext) -> Unit
```

Hooks execute in dependency order and reverse dependency order respectively.

Hooks MUST NOT alter type identity, specialization arguments, selector identity, or the composition plan after the class is finalized.

## 53. Parametric mixins

A generic mixin specialization is a `SpecializedType`:

```phalcom
Cached<User>
```

Configuration produces a `ConfiguredMixin` value:

```phalcom
Cached<User>(
  capacity: 1_000,
  expiration: Duration.minutes(5)
)
```

The distinction is:

```text
Cached<User>                  type-level mixin specialization
Cached<User>(capacity: ...)  configured composition operand
```

The reserved generic specialization selector remains compiler/runtime-controlled.

## 54. Capability queries

```phalcom
UserRepository.capabilities
UserRepository.composition
UserRepository.providedBy(Cached<User>)
UserRepository.satisfies(CachedRepository<User>)
```

Objects MAY expose their class composition reflectively:

```phalcom
repository.capabilities
```

## 55. Dynamic composition

Dynamic mutation of an existing class's mixin composition is outside the initial specification.

A runtime MAY create a new anonymous composed class through an explicit builder, but the resulting class MUST receive a frozen composition plan and distinct class identity.

## 56. Interactions with multimethods and instances

A mixin MAY contribute multimethod cases or protocol instances only when the contribution is explicitly listed in its descriptor.

Such contributions become active only when the containing class or module is active in the relevant dispatch environment. They MUST NOT leak process-globally.

---

# Part VI — Units and Dimensions as First-Class Types

## 57. Purpose

The unit system defines dimensions, units, quantities, conversions, and dimensional algebra as first-class reflective types implemented primarily in Phalcom.

The system MUST distinguish:

- dimensions from units;
- scalar quantities from affine measurement points;
- runtime numeric values from unit metadata;
- exact mathematical conversions from context-dependent conversions.

## 58. Base dimensions

```phalcom
@dimension
class Length {}

@dimension
class Time {}

@dimension
class Mass {}
```

Each declaration produces a canonical `DimensionType`.

```phalcom
Length.class
// DimensionType
```

A dimension is normalized as an exponent map over base dimensions:

```phalcom
@data
@immutable
class DimensionType is Type {
  const _powers: Map<BaseDimensionType, Rational>
}
```

Examples:

```text
Length       = {Length: 1}
Time         = {Time: 1}
Speed        = {Length: 1, Time: -1}
Acceleration = {Length: 1, Time: -2}
```

## 59. Dimension algebra

Dimensions support canonical multiplication, division, and rational powers:

```phalcom
const Speed = Length / Time
const Acceleration = Length / (Time ** 2)
const Area = Length ** 2
```

Equivalent dimensions are canonical:

```phalcom
(Length / Time) === (Length * (Time ** -1))
// true
```

Dimension operations are type-level ordinary operators on `DimensionType`, not generic specialization selectors.

## 60. Units

A unit has a dimension and a conversion to a canonical unit of that dimension.

```phalcom
@unit(
  symbol: #m,
  dimension: Length,
  scale: Rational.one
)
class Meter {}
```

```phalcom
@unit(
  symbol: #cm,
  dimension: Length,
  scale: Rational.new(1, 100)
)
class Centimeter {}
```

The declarations produce canonical `UnitType` objects.

```phalcom
Meter.class
// UnitType

Meter.dimension
// Length
```

## 61. Derived units

Unit multiplication and division create canonical derived units:

```phalcom
const MeterPerSecond = Meter / Second
const Newton = Kilogram * Meter / (Second ** 2)
```

A derived unit contains:

```phalcom
@data
@immutable
class UnitType is Type {
  const _dimension: DimensionType
  const _scale: ExactScale
  const _offset: Option<ExactOffset>
  const _components: Map<UnitType, Rational>
  const _symbol: Option<Symbol>
}
```

Named units may alias a structurally equivalent derived unit while retaining a preferred display name.

```phalcom
Newton.equivalentTo(
  Kilogram * Meter / (Second ** 2)
)
// true
```

Identity and equivalence remain distinct if display aliases are preserved.

## 62. Quantities

```phalcom
@data
@immutable
class Quantity<Unit> {
  const _magnitude: Number
}
```

`Quantity<Meter>` is a `SpecializedType`.

```phalcom
const distance = Quantity<Meter>(100)

distance.class
// Quantity

distance.type
// Quantity<Meter>

distance.unit
// Meter

distance.dimension
// Length
```

The unit is recovered from `distance.type.argument(#Unit)`. It is not copied into an ordinary mutable field.

## 63. Quantity construction

Conveniences MAY include:

```phalcom
100.of(Meter)
Meter.quantity(100)
Quantity<Meter>(100)
```

All forms MUST produce equivalent `Quantity<Meter>` values.

The stable constructor is:

```phalcom
Quantity<Meter>(100)
```

## 64. Addition and subtraction

Addition and subtraction require compatible dimensions.

The strict initial rule is:

- ordinary `+` and `-` require the same unit specialization;
- cross-unit arithmetic requires explicit conversion.

```phalcom
Quantity<Meter>(1) + Quantity<Meter>(2)
// Quantity<Meter>(3)
```

```phalcom
Quantity<Meter>(1) + Quantity<Centimeter>(50)
// error: explicit conversion required
```

```phalcom
Quantity<Meter>(1)
  + Quantity<Centimeter>(50).convert(to: Meter)
// Quantity<Meter>(1.5)
```

This rule avoids hidden precision, rounding, and preferred-unit decisions.

A library MAY provide explicit convenience:

```phalcom
left.addConverting(right, to: Meter)
```

## 65. Multiplication and division

```phalcom
const distance = Quantity<Meter>(100)
const duration = Quantity<Second>(10)

const speed = distance / duration

speed.type
// Quantity<Meter / Second>
```

Multiplication and division combine unit types canonically.

Dimensionless results normalize to `Quantity<One>` or a configured scalar-unwrapping policy. The stable type-preserving rule is `Quantity<One>`.

## 66. Powers

```phalcom
const area = Quantity<Meter>(3) ** 2

area.type
// Quantity<Meter ** 2>
```

Noninteger powers are permitted only when the resulting unit exponents are representable and the numeric magnitude operation is valid.

## 67. Comparisons

Equality of quantities is domain equality, not object identity.

The stable comparison API is:

```phalcom
left.equivalentQuantity(right)
left.compareConverting(right)
```

Whether `==` performs cross-unit conversion remains a broader object-model decision. Implementations MUST provide a guaranteed-Boolean stable method independent of symbolic `==` discussions.

Ordering requires equal dimensions and a deterministic conversion policy.

## 68. Conversion

```phalcom
const centimeters =
  Quantity<Meter>(1.5)
    .convert(to: Centimeter)
```

Conversion results in a new specialization:

```phalcom
centimeters.type
// Quantity<Centimeter>
```

Exact linear conversions SHOULD use rational scale factors internally.

Lossy conversion MUST be explicit in the result or policy:

```phalcom
quantity.convert(
  to: Unit,
  rounding: Rounding.nearestEven
)
```

## 69. Affine units and measurement points

Offset units such as Celsius are not ordinary vector units.

The specification distinguishes:

```phalcom
Quantity<CelsiusDelta>
MeasurePoint<Celsius>
```

Valid:

```phalcom
MeasurePoint<Celsius>(20)
  - MeasurePoint<Celsius>(10)
// Quantity<CelsiusDelta>(10)
```

```phalcom
MeasurePoint<Celsius>(20)
  + Quantity<CelsiusDelta>(5)
// MeasurePoint<Celsius>(25)
```

Invalid:

```phalcom
MeasurePoint<Celsius>(20)
  + MeasurePoint<Celsius>(10)
```

Affine units MUST NOT be treated as simple multiplicative units.

## 70. Unit protocols and protocol instances

Domain-specific numeric representations can participate through protocols:

```phalcom
@protocol
class Scalable<in Scale, out Self> {
  scale(by: Scale) -> Self
}
```

Conversions MAY use protocol instances:

```phalcom
@instance(ConvertUnit<Meter, Centimeter, Decimal>)
class DecimalMeterToCentimeter {
  ...
}
```

Built-in exact conversions SHOULD remain canonical unit metadata rather than instance lookups.

## 71. Unit patterns and dispatch

Pattern matching:

```phalcom
Patterns.quantity(
  dimension: Length,
  capture: #distance
)
```

Multimethod dispatch:

```phalcom
@case
render(value: Quantity<Meter>) {
  ...
}

@case
render(value: Quantity<Centimeter>) {
  ...
}

@case
render(value: Quantity<U>)
  @where(U.dimension == Length) {
  ...
}
```

Specificity follows ordinary `SpecializedType` and refinement relations.

## 72. Reflection

```phalcom
Meter.dimension
Meter.scale
Meter.symbol
Meter.components

Quantity<Meter>.origin
Quantity<Meter>.argument(#Unit)

speed.unit
speed.dimension
```

Tools MAY derive formatting, schemas, validators, serializers, and property-based strategies from this metadata.

## 73. Diagnostics

Examples:

```text
× Incompatible quantity addition

  Left:
    Quantity<Meter>

  Right:
    Quantity<Second>

  Dimensions:
    Length
    Time
```

```text
× Explicit unit conversion required

  Left unit:
    Meter

  Right unit:
    Centimeter

  Both values have dimension Length.
  Convert one operand before addition.
```

```text
× Invalid affine-unit operation

  Cannot add two MeasurePoint<Celsius> values.
  Subtract points to obtain Quantity<CelsiusDelta>,
  or add a temperature difference to one point.
```

---

# Part VII — Cross-feature integration

## 74. Shared specificity engine

The following MUST reuse one type and logical specificity service:

- multimethod cases;
- protocol instances;
- unordered pattern cases;
- mixin requirement resolution;
- quantity dispatch;
- generic instance selection.

A framework MUST NOT invent a conflicting local definition of “more specific.”

## 75. Shared scoped extension environment

Multimethod cases and protocol instances use the same rules for direct imports, explicit re-exports, orphans, caching, and diagnostics.

Mixin-contributed cases and instances become part of that environment only through explicit composition metadata.

## 76. Reflection-driven tooling

The combined model enables tools such as:

```phalcom
Compatibility.compare(oldApi, newApi)
Dispatch.explain(call)
Instances.explain(protocolType)
Match.exhaustiveness(patterns, subjectType)
Composition.explain(class)
Dimensions.explain(unit)
```

## 77. Symbolic predicates

Predicate guards and guarded patterns SHOULD share one symbolic expression framework.

Stable logical composition remains:

```phalcom
Formula.all(*terms)
Formula.any(*terms)
Formula.not(term)
```

Language-integrated `and`, `or`, and `not` MAY later support lazily dispatched logical values, but none of the facilities in this specification depend on that future decision.

## 78. Equality discipline

Because symbolic expression equality may eventually return formulas, all framework internals MUST use explicit guaranteed-Boolean operations for:

- canonicalization;
- cache keys;
- map and set membership;
- pattern literal comparison;
- unit equivalence;
- type identity and equivalence.

Recommended distinctions:

```text
===                    object identity
sameExpressionAs(_)    expression structure
sameValueAs(_)         guaranteed-Boolean value equality
==                     domain operator, subject to language decision
```

---

# Part VIII — Compiler, Runtime, and VM Obligations

## 79. Compiler obligations

The compiler MUST:

1. generate reserved `<>(...)` selectors for declared generics;
2. prohibit user definitions and overrides of specialization selectors;
3. preserve complete selector and label metadata;
4. validate multimethod case signatures;
5. validate protocol-instance implementations;
6. validate generic variance positions;
7. construct mixin composition plans and reject conflicts;
8. retain pattern and guard metadata;
9. emit enough unit and dimension metadata for runtime reflection;
10. detect statically knowable arity, bound, coherence, and exhaustiveness failures.

## 80. Runtime obligations

The runtime MUST:

1. canonicalize `SpecializedType` values;
2. preserve specialized runtime type identity on constructed instances;
3. provide scoped dispatch and instance environments;
4. maintain cache invalidation through environment versions;
5. enforce runtime generic argument validation for dynamic specialization;
6. execute multimethod and pattern resolution deterministically;
7. preserve mixin state isolation;
8. canonicalize dimensions and derived units;
9. expose reflective descriptors without permitting mutation of reserved semantic metadata.

## 81. VM optimization latitude

The VM MAY use specialized instructions and caches rather than literal ordinary message sends, provided observable semantics remain equivalent.

Examples:

```text
SPECIALIZE N
MULTI_DISPATCH
INSTANCE_RESOLVE
PATTERN_MATCH
QUANTITY_CONVERT
```

Reserved operations may bypass generic interception to preserve guarantees and diagnostics.

---

# Part IX — Required Diagnostics

## 82. Generic arity

```text
× Var does not understand '<>(_,_)'

  Var accepts exactly 1 positional generic argument,
  but 2 were provided.

  Expected:
    Var<_>

  Received:
    Var<Int, String>
```

## 83. Multimethod ambiguity

Must list the call shape, runtime types, applicable maximal cases, and why no candidate dominates.

## 84. Instance ambiguity

Must list the requested protocol specialization, active environment, applicable instances, and failed specificity relation.

## 85. Pattern nonexhaustiveness

Should display the residual type or a concrete witness:

```text
× Non-exhaustive match

  Subject type:
    Option<Int>

  Uncovered case:
    None
```

## 86. Mixin conflict

Must identify the selector or capability, all contributors, and available explicit resolutions.

## 87. Unit mismatch

Must distinguish dimensional incompatibility from compatible dimensions requiring explicit unit conversion.

---

# Part X — Conformance and Acceptance Tests

## 88. Multimethod tests

An implementation conforms when it demonstrates:

1. exact type case beats superclass case;
2. protocol case participates structurally;
3. third-party cases are active only when imported;
4. transitive imports do not activate cases accidentally;
5. incomparable maximal cases produce ambiguity;
6. value patterns filter cases before guards;
7. symbolic guard implication affects specificity when provable;
8. opaque guards do not create false static guarantees;
9. cache invalidation follows environment changes.

## 89. Protocol-instance tests

Required tests:

1. exact instance resolution;
2. generic instance resolution;
3. subordinate instance requirements;
4. orphan scoping;
5. missing instance diagnostics;
6. recursive instance-cycle detection;
7. ambiguous instances;
8. implementation conformance verification;
9. lifetime handling.

## 90. Pattern tests

Required tests:

1. literal, type, tuple, record, variant, and protocol patterns;
2. positional and labeled captures;
3. rest captures;
4. ordered overlap behavior;
5. unordered ambiguity rejection;
6. exhaustive sealed matching;
7. residual witness generation;
8. unreachable-case diagnostics;
9. opaque-guard `Unknown` coverage.

## 91. Mixin tests

Required tests:

1. requirements satisfied by base class;
2. requirements satisfied by another mixin;
3. missing requirement diagnostics;
4. namespaced state isolation;
5. selector conflict detection;
6. explicit resolution;
7. dependency ordering;
8. cycle detection;
9. generic mixin specialization;
10. configured mixin reflection.

## 92. Units and dimensions tests

Required tests:

1. canonical base and derived dimensions;
2. equivalent dimension expressions are identical or canonically equivalent;
3. `Quantity<Meter>` remembers specialization;
4. same-unit addition;
5. explicit cross-unit conversion;
6. dimension mismatch rejection;
7. multiplication and division produce correct derived units;
8. powers normalize exponents;
9. affine point/difference rules;
10. exact and lossy conversion diagnostics;
11. reflection and formatting.

---

# Part XI — Recommendations and Deferred Questions

## 93. Ratified recommendations

The recommended initial design is:

1. multimethods are first-class descriptors with import-scoped open cases;
2. declaration order never resolves unordered multimethod ambiguity;
3. predicate dispatch uses reified formulas where static reasoning is required;
4. opaque predicates remain runtime-only and explicitly marked;
5. protocol instances are external implementation objects, not injected methods;
6. instance coherence is scoped and specificity-based;
7. reflective pattern matching ships as an ordinary-object API before native syntax;
8. ordered and unordered matching are distinct explicit modes;
9. richer mixins use frozen composition plans, namespaced state, and explicit conflict resolution;
10. dimensions and units are canonical first-class types;
11. quantities remember their unit through `SpecializedType` runtime identity;
12. affine units use measurement points and differences rather than pretending offsets are multiplicative;
13. all facilities share the same type relation, reflection, and diagnostic infrastructure.

## 94. Deferred questions

The following remain deferred:

1. native `match` syntax;
2. lazily dispatchable `and`, `or`, and `not` keywords;
3. symbolic `==` and `!=` versus guaranteed-Boolean equality;
4. user-defined refinement relations beyond registered certified rules;
5. dynamic mutation of dispatch environments;
6. dynamic mutation of class mixin composition;
7. higher-kinded protocol instances;
8. automatic cross-unit arithmetic policy;
9. implicit scalar unwrapping for dimensionless quantities;
10. specialization-specific instance method tables or layouts.

These questions MUST NOT block the stable APIs defined here.

---

# Appendix A — Compact integrated example

```phalcom
@dimension
class Length {}

@dimension
class Time {}

@unit(symbol: #m, dimension: Length, scale: Rational.one)
class Meter {}

@unit(symbol: #s, dimension: Time, scale: Rational.one)
class Second {}

@protocol
class Render<T> {
  render(value: T) -> String
}

@instance(Render<Quantity<Meter>>)
class MeterRenderer {
  render(value: Quantity<Meter>) -> String {
    return "\\(value.magnitude) m"
  }
}

@multimethod
class describe {
  @case
  call(value: Quantity<Meter>) -> String {
    return "distance: \\(value.magnitude) m"
  }

  @case
  call(value: Quantity<Meter / Second>) -> String {
    return "speed: \\(value.magnitude) m/s"
  }
}

@mixin
class Traced {
  @provides(Traceable)

  trace(message: String) {
    System.print(message)
  }
}

@with(Traced)
class MeasurementService {
  measure(
    distance: Quantity<Meter>,
    duration: Quantity<Second>
  ) {
    const speed = distance / duration
    trace(describe(speed))
    return speed
  }
}

const result =
  Match.on(
    MeasurementService.new().measure(
      Quantity<Meter>(100),
      Quantity<Second>(10)
    )
  ).with(
    Case.of(
      Patterns.quantity(
        unit: Meter / Second,
        capture: #speed
      )
    ) { **captures =>
      captures.speed
    }
  )
```

This example uses:

- first-class specialized types;
- protocol instances;
- multimethod dispatch;
- units and dimensions;
- richer mixins;
- reflective pattern matching;
- tuples, records, labels, and spread capture.

---

# Appendix B — Semantic summary

```text
Types and formulas define regions of possible values.

Multimethods choose one implementation from applicable regions.
Protocol instances choose one external capability implementation.
Patterns decompose and subtract regions.
Mixins compose implementation capabilities under explicit constraints.
Units and dimensions form a canonical type algebra.

All five depend on:
  canonical SpecializedType values,
  selector identity,
  scoped openness,
  proof-aware specificity,
  immutable reflection metadata,
  and explicit ambiguity diagnostics.
```

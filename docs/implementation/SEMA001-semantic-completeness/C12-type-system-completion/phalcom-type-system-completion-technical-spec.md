# Phalcom Type-System Completion — Technical Specification

**Status:** Proposed authoritative completion specification  
**Repository baseline:** `aureat/phalcom-lang` remote `main` at `e17f2733f98cb20e2a8ead5794d75ca647a950ce`  
**Prepared:** 2026-09-04  
**Scope:** Callable-local generics, generic constructors/accessors/index members, variant-local generics, generic variant constructors, full GADT existential elimination, exact-case interaction, rank-1 Families, durable publication, incremental parity, and static semantics required for reified applied generic types with per-application class-side storage.

---

## 1. Purpose

This specification closes the remaining planned gaps in Phalcom's static generic and GADT semantics without introducing a second generic calculus.

The governing architectural rule is:

> Methods, constructors, getters, setters, index getters, index setters, and variant constructors are callable semantic entities. Callable-local type parameters use one canonical ownership model, one `GenericSignature` representation, one constraint/inference pipeline, and one specialization model. Full GADT semantics extend that model at elimination time with scoped rigid variables; they do not introduce a GADT-specific inference engine.

The target architecture is:

```text
                    GENERIC TYPE SYSTEM
                           │
             ┌─────────────┴─────────────┐
             │                           │
     DECLARATION APPLICATION      CALLABLE APPLICATION
             │                           │
      declaration-owned            callable-owned
      type parameters              type parameters
             │                           │
             └─────────────┬─────────────┘
                           │
                      one solver
                           │
               application substitutions
                           │
                   specialized view
```

GADTs add one further layer:

```text
           GENERIC VARIANT CONSTRUCTION

                 ∀U at introduction
                         │
                  ordinary inference
                         │
                         ▼
                 constructed value


              GADT ELIMINATION

              hidden local binder U
                         │
                         ▼
                       ∃U
                         │
                  fresh rigid κ
                         │
          ┌──────────────┴──────────────┐
          │                             │
   payload specialization       result-index proof
          │                             │
          └──────────────┬──────────────┘
                         │
                  branch environment
                         │
                         ▼
                 non-escape checking
```

---

## 2. Repository Baseline and Current Capabilities

The current remote repository already contains the majority of the required infrastructure.

### 2.1 Canonical semantic identities

Current code includes:

- `TypeId`, `KindId`, `TypeParameterId`;
- `DeclarationId`;
- `CallableOwnerId::{Declaration, Variant}`;
- `CallableId`;
- `VariantId`;
- `VariantConstructorId`;
- `InvocationTargetId`;
- exact-case type identities;
- canonical applied type forms.

Important current identity definitions are in:

```text
phalcom-semantic/src/identity.rs
```

`CallableOwnerId` already supports exact variants:

```rust
pub enum CallableOwnerId {
    Declaration(DeclarationId),
    Variant(VariantId),
}
```

and `CallableId` already has:

```rust
pub struct CallableId {
    pub owner: CallableOwnerId,
    pub selector: Selector,
    pub side: DispatchSide,
}
```

This means variant-local type parameters can remain `TypeParameterOwner::Callable(...)`; a new `TypeParameterOwner::Variant` category is not required.

### 2.2 Generic signatures

Current generic declaration/callable semantics already include:

```text
GenericSignature
TypeParameterOwner
GenericConstraint
TypeTerm
```

and publication validation rejects owner mismatch and solver-local inference variables.

The key current invariant in `phalcom-semantic/src/types/parameter.rs` is:

> A published `GenericSignature` owns a contiguous sequence of type parameters whose `TypeParameterData.owner` is exactly the signature owner.

This invariant is normative and must be preserved.

### 2.3 Generic call inference

The repository already supports:

- argument-derived generic inference;
- expected-result-derived generic inference;
- declaration-owned receiver specialization;
- callable-owned generic parameters;
- higher-kinded parameter kinds where supported;
- generic constraints;
- transformed receiver inheritance;
- `Self` specialization;
- rank-1 Family retention/application.

Representative current tests include:

```text
phalcom-semantic/tests/semantic/foundations/generic_application.rs
phalcom-semantic/tests/semantic/foundations/receiver_specialization.rs
phalcom-semantic/tests/semantic/capabilities/generics.rs
phalcom-semantic/tests/semantic/capabilities/getters.rs
```

### 2.4 Generic getters are already implemented

Current `main` supports getter-local generic binders and `where` clauses. The repository includes parser and semantic tests verifying:

- expected-result inference;
- underconstrained getter access;
- constraint satisfaction/failure;
- inherited/transformed receiver specialization;
- enum generic getters;
- callable-owned getter type parameters;
- stable getter selector identity.

The historical file:

```text
docs/work/deferred/generic-on-getter.md
```

is stale relative to the investigated baseline.

### 2.5 Existing GADT support

Phalcom already supports declaration-indexed GADT semantics.

The central current product is:

```rust
pub struct CaseTypeEnvironment {
    pub bindings: BTreeMap<TypeParameterId, TypeId>,
    pub equalities: Box<[GenericConstraint]>,
}
```

in:

```text
phalcom-semantic/src/types/case_environment.rs
```

Current match/GADT machinery includes:

```text
phalcom-semantic/src/checker/gadt_proof.rs
phalcom-semantic/src/checker/exhaustiveness.rs
phalcom-semantic/src/checker/pattern_space.rs
```

It already proves equalities such as:

```text
Expr<T> matched with Expr<Int>
⇒ T = Int
```

The missing GADT dimension is constructor-local polymorphism and existential elimination.

### 2.6 Existing exact-case representation

Current canonical representation is:

```rust
TypeData::ExactCase {
    variant,
    enum_type,
}
```

This representation is used consistently in substitution, type environments, GADT proofing, inference, relations, associated lookup, and exhaustiveness.

### 2.7 Existing applied-type reification

Canonical applied semantic types already survive into metadata:

```rust
TypeNode::Applied {
    origin,
    arguments,
}
```

The runtime typing/reflection layer can materialize applied forms as `AppliedType` descriptors. The missing runtime feature is not semantic applied-type identity; it is executable applied-class context and per-applied-type class storage.

---

## 3. Normative Terminology

### 3.1 Declaration-owned type parameter

A binder introduced by a generic type declaration.

Example:

```phalcom
class Box<T> { ... }
```

`T` is declaration-owned.

### 3.2 Callable-owned type parameter

A binder introduced by a callable semantic declaration.

Example:

```phalcom
convert<U>(_ value: U) -> U { ... }
```

`U` is callable-owned.

The callable may be a method, constructor, getter, setter, index getter, index setter, or variant constructor.

### 3.3 Application domain

One ownership domain participating in a generic application.

A constructor on a generic class can have two domains:

```text
declaration domain: Box::T
callable domain:    Box::new::U
```

### 3.4 Flexible inference variable

A transient solver metavariable whose value may be solved from constraints.

Notation:

```text
α, β, ...
```

### 3.5 Rigid type variable

A scoped, fixed-but-unknown type identity introduced when eliminating an existentially hidden constructor-local binder.

Notation:

```text
κ₁, κ₂, ...
```

A rigid variable is not assignable by ordinary inference.

### 3.6 Proper applied receiver

A saturated proper type application used as the semantic receiver specialization for a generic declaration.

Example:

```text
Box<Int>
```

### 3.7 Invocation specialization

The durable semantic information describing one resolved invocation:

```text
canonical target
+ applied receiver when relevant
+ declaration substitution
+ callable substitution
```

This is semantically distinct from executable code specialization/monomorphization.

---

## 4. Fundamental Ownership Model

### CGC-01 — Callable-local generic ownership

Every callable-local type parameter is owned by the callable's canonical `CallableId`.

This includes:

```text
method<T>
constructor<T>
getter<T>
setter<T>
index-getter<T>
index-setter<T>
variant-constructor<T>
```

A new syntax-specific type-parameter owner is forbidden unless a future feature is not semantically callable.

### CGC-02 — Declaration and callable parameters are different identities

Given:

```phalcom
class Box<T> {
    @constructor
    new<U>(value: T, metadata: U) { ... }
}
```

these are distinct:

```text
Box::T
Box::new::U
```

The compiler must preserve that distinction in:

- ownership;
- substitutions;
- diagnostics;
- incremental fingerprints;
- metadata;
- reflection;
- future applied-runtime environments.

### CGC-03 — One solver may solve multiple ownership domains

Separate ownership does not imply separate solving.

A generic application may simultaneously use:

```text
receiver/declaration constraints
callable-local constraints
source argument constraints
expected-result constraints
where/bound constraints
GADT result/index equations
```

through one inference session.

### CGC-04 — Canonical generic signatures are homogeneous by owner

A durable `GenericSignature` may contain only parameters owned by its declared owner.

Therefore a generic constructor on a generic class must not canonically publish one synthetic signature such as:

```text
owner = Callable(Box::new)
parameters = [Box::T, Box::new::U]
```

because `Box::T` is declaration-owned.

The current `merge_constructor_generic_signatures` behavior may be used as historical evidence of an application-time need, but its result must not remain the canonical published signature.

### CGC-05 — Generic application composition is an application product

The canonical model is:

```text
Declaration GenericSignature
        +
Callable GenericSignature
        ↓
GenericApplicationPlan / equivalent
        ↓
one InferenceSession
        ↓
separate substitutions
```

The exact Rust product name is implementation-defined.

### CGC-06 — Generic instantiation never changes callable identity

Instantiation does not manufacture a new `CallableId`.

Examples:

```text
identity<Int>
identity<String>
```

share the same canonical callable identity.

### CGC-07 — Generic instantiation never changes selector identity

Type arguments are never encoded into selectors.

### CGC-08 — Bounds constrain candidates but do not invent candidates

From:

```text
T <: Number
```

alone, the compiler may not infer:

```text
T = Number
```

This law already matches current expected-result inference tests and remains normative.

### CGC-09 — Generic signatures are immutable declaration templates

Application creates substitutions and specialized views; it does not mutate the declaration signature.

### CGC-10 — Durable products contain no flexible inference variables

Solver-local metavariables may not be published in:

- semantic metadata;
- snapshot interfaces;
- callable signatures;
- declaration signatures;
- runtime type metadata.

---

## 5. Generic Receiver Application

### APP-01 — Unsaturated generic receivers are not proper instance types

For:

```text
Box :: Type -> Type
```

bare `Box` is a type constructor, not the final proper receiver application represented by `Box<Int>`.

### APP-02 — Generic construction may infer receiver saturation

Given:

```phalcom
class Box<T> {
    @constructor
    new(value: T) { ... }
}
```

then:

```phalcom
Box.new(value: 10)
```

must conceptually perform:

```text
1. Load constructor template:
   Box<T>::new(value: T) -> Box<T>

2. Introduce declaration-domain inference variable for Box::T.

3. Constrain from argument:
   T = Int

4. Canonicalize receiver application:
   Box<Int>

5. Publish invocation specialization with:
   receiver_application = Box<Int>

6. Publish result:
   Box<Int>
```

### APP-03 — Explicit application remains authoritative evidence

For:

```phalcom
Box<Int>.new(value: 10)
```

the explicit receiver fixes:

```text
Box::T = Int
```

before callable-local inference.

Conflicting arguments must fail rather than respecialize the receiver.

### APP-04 — Applied receiver specialization must survive semantic publication

If generic receiver inference produces:

```text
Box<Int>
```

that receiver application is semantically relevant beyond result typing and must be available to lowering/future runtime class-side storage.

It may not be silently erased to bare `Box` in the invocation product.

---

## 6. Generic Constructors

### CTOR-01 — Constructors are ordinary semantic callables

A constructor uses the same callable declaration and generic machinery as methods.

Constructor-specific semantics are limited to:

- construction result semantics;
- receiver/application inference;
- lowering/runtime construction target.

### CTOR-02 — Constructor-local parameters are callable-owned

For:

```phalcom
@constructor
new<U>(...)
```

`U` is owned by the constructor `CallableId`.

### CTOR-03 — Constructor declaration and callable substitutions remain separate

For:

```phalcom
Box.new(value: 10, metadata: "literal")
```

with:

```phalcom
class Box<T> {
    @constructor
    new<U>(value: T, metadata: U) { ... }
}
```

publish:

```text
declaration substitution:
    Box::T -> Int

callable substitution:
    Box::new::U -> String

receiver application:
    Box<Int>

result:
    Box<Int>
```

### CTOR-04 — Constructor local constraints use the ordinary generic constraint pipeline

Example:

```phalcom
@constructor
new<U>(value: T, metadata: U)
where U <: Serializable
{
    ...
}
```

No constructor-specific bound solver shall exist.

### CTOR-05 — Constructor expected-result inference may participate where sound

If the existing application solver can derive declaration or constructor-local parameters from the expected result without ambiguity, it may do so.

No arbitrary precedence rule such as "arguments always win" is normative; conflicting evidence is a constraint conflict.

---

## 7. Generic Getters

Generic getters are already supported by the investigated repository; the following laws document and preserve that behavior.

### GET-01 — Generic getter is a zero-value-argument generic callable

Example:

```phalcom
default<T> -> T
where T <: Default
{
    T.default
}
```

Conceptual type:

```text
∀T. () -> T
```

### GET-02 — Expected result is valid inference evidence

```phalcom
let value: Int = Factory.default
```

may infer:

```text
T = Int
```

### GET-03 — Context-free access remains underconstrained

```phalcom
let value = Factory.default
```

is underconstrained unless another source of evidence uniquely determines `T`.

### GET-04 — Bounds do not default a generic getter

A bound such as:

```text
T <: Number
```

does not choose `Number` in the absence of inference evidence.

### GET-05 — Getter selector identity remains unchanged by instantiation

A generic getter remains one selector and one `CallableId`.

---

## 8. Generic Setters

### SET-01 — Generic setter is an independent generic callable

A setter is not modeled as getter mutation.

Illustrative syntax:

```phalcom
value<T>=(newValue: T)
where T <: Serializable
{
    ...
}
```

The exact grammar remains an implementation/surface decision, but the semantics are fixed.

### SET-02 — Assigned value is ordinary argument evidence

For:

```phalcom
object.value = 42
```

the RHS supplies an argument constraint exactly as if the setter were applied to `42`.

### SET-03 — Setter selector identity is independent of type arguments

Instantiations do not create selectors such as:

```text
value=<Int>
value=<String>
```

### SET-04 — Assignment expression result is `Unit`

Property assignment expressions are `Unit`-typed.

The canonical setter callable should use `Unit` as its language-level return unless another independently ratified setter-send return rule requires otherwise.

---

## 9. Generic Index Getters

### IDXG-01 — Index getter is an ordinary callable

Illustrative syntax:

```phalcom
[_ field]<T>(field: Field<T>) -> T {
    ...
}
```

### IDXG-02 — Index getter inference combines all available evidence

Potential evidence includes:

```text
applied receiver
index arguments
expected result
```

Example:

```phalcom
class Store<T> {
    [_ key]<U>(key: Key<U>) -> Pair<T, U> {
        ...
    }
}
```

with:

```phalcom
let store: Store<Int>
let result = store[StringKey]
```

must specialize:

```text
Store::T  -> Int
index::U  -> String
result    -> Pair<Int, String>
```

### IDXG-03 — Index selector identity is resolved before generic instantiation

Resolution order is:

```text
index syntax / labels / arity
        ↓
canonical index selector
        ↓
CallableId
        ↓
generic application
```

Type arguments never become overload identity.

---

## 10. Generic Index Setters

### IDXS-01 — Index setter is a separate callable

The current architecture already correctly treats index set as its own callable selector.

### IDXS-02 — Assigned value participates in inference as an argument

For:

```phalcom
record[AgeKey] = 42
```

where:

```text
AgeKey : Key<Int>
```

both key and value may constrain the same local parameter.

### IDXS-03 — Conflicting evidence is an ordinary inference conflict

```phalcom
record[AgeKey] = "wrong"
```

with `AgeKey : Key<Int>` must not pick one side arbitrarily or recover to `Dynamic` merely to continue.

### IDXS-04 — Getter and setter generic signatures need not match

Legal model:

```text
[] getter:
    ∀T. Column<T> -> T

[]= setter:
    ∀T,U. (Column<T>, U) -> Unit
    where U <: Convertible<T>
```

---

## 11. Variant-Local Generic Parameters

### VGEN-01 — Variant-local parameters are callable-owned

Example:

```phalcom
enum Expr<T> {
    @variant Equal<U>(
        _ left: Expr<U>,
        _ right: Expr<U>
    ) -> Expr<Bool>
}
```

`U` belongs to the variant constructor callable, not to `Expr`.

### VGEN-02 — Variant constructor callable identity

Phalcom already supports `CallableOwnerId::Variant(VariantId)`.

A generic variant constructor shall use a deterministic callable identity derived from the exact `VariantId`, while retaining `VariantConstructorId` as the executable construction identity.

The recommended encoding is:

```text
owner    = CallableOwnerId::Variant(variant)
side     = DispatchSide::Class
selector = variant.selector
```

or an equivalent dedicated helper producing a collision-free canonical `CallableId`.

This identity is used for:

```text
TypeParameterOwner::Callable
GenericSignature ownership
metadata
incremental identity
diagnostics
```

It does not replace `VariantConstructorId` in lowering/runtime construction.

### VGEN-03 — Variant-local generic syntax is first-class AST data

`VariantDecl` shall gain generic binder and `where` data analogous to generic methods/getters.

### VGEN-04 — Variant payload/result resolution sees both enum and local parameters

For:

```phalcom
enum Expr<T> {
    @variant Cast<U>(_ value: T, _ target: U) -> Expr<U>
}
```

payload/result annotations resolve under nested scopes:

```text
parent enum scope: T
variant callable scope: U
```

Local shadowing/duplicate-name rules follow the existing generic binder rules.

### VGEN-05 — Variant-local `where` constraints use ordinary generic constraints

Example:

```phalcom
@variant Showable<U>(_ value: U)
where U <: Show
```

construction validates through the existing generic constraint system.

---

## 12. Generic Variant Construction

### VCALL-01 — Generic variant construction is universal introduction

At construction, local variant parameters are universally quantified:

```text
Equal : ∀U. (Expr<U>, Expr<U>) -> Expr<Bool>
```

### VCALL-02 — Generic variant construction uses ordinary inference

For:

```phalcom
@variant Literal<U>(_ value: U) -> Expr<U>
```

then:

```phalcom
Expr::Literal(42)
```

uses ordinary argument-derived inference:

```text
U = Int
result = Expr<Int>
```

No GADT-specific constructor solver is permitted.

### VCALL-03 — Enum declaration and variant callable domains remain distinct

A generic enum containing a generic variant may require simultaneous solving across:

```text
enum declaration parameters
variant-local callable parameters
```

These domains compose through the same application-domain mechanism used for generic constructors.

### VCALL-04 — Variant result and exact-case forms end in canonical type identities

Any solved result application must be canonicalized through existing `TypeStore` application machinery.

---

## 13. Existing GADT Model

### GADT-BASE-01 — `CaseTypeEnvironment` remains declaration-index authority

`CaseTypeEnvironment` continues to represent equations over canonical declaration-owned enum parameters.

It must not be overloaded into a container for branch-local existential allocation.

### GADT-BASE-02 — Existing branch proof solver remains equality authority

The current GADT equality solver remains the canonical mechanism for proving/refuting indexed cases.

Superclass/subtype relationships do not automatically prove type equality.

### GADT-BASE-03 — Existing exact-case reachability/exhaustiveness remains reusable

Full GADT completion extends existing pattern-space and proof machinery; it does not replace it.

---

## 14. Rigid Type Variables

### RIGID-01 — Full GADT elimination requires a distinct rigid semantic category

A rigid variable means:

> some fixed type identity exists, but the checker is not free to choose which type it is.

This differs from a flexible inference variable.

### RIGID-02 — Rigid variables are scoped

Each rigid carries a defining scope.

Recommended conceptual shape:

```rust
struct RigidTypeVariable {
    id: RigidTypeVariableId,
    kind: KindId,
    scope: RigidScopeId,
    origin: RigidOrigin,
}

enum RigidOrigin {
    GadtConstructorExistential,
}
```

The names are illustrative; semantics are normative.

### RIGID-03 — Rigids preserve binder kind

If local parameter `U` has kind `K`, the corresponding rigid `κ` has kind `K`.

### RIGID-04 — Rigids are not ordinary `TypeParameterId`s

Reusing a normal type parameter would permit existing substitution/proof paths to accidentally solve or publish it as if it were a declaration binder.

### RIGID-05 — Rigids are not ordinary inference variables

The solver may compare a rigid and propagate equations involving it, but may not bind the rigid to a candidate merely to satisfy a constraint.

### RIGID-06 — Rigids should remain scoped/local rather than globally canonical

The preferred representation is an analysis-local/scoped type-term layer rather than unrestricted global `TypeStore` interning.

This preserves the existing invariant that ephemeral solver/branch identities do not leak into durable metadata.

### RIGID-07 — Rigid allocation numbers are not durable semantic identity

Cold and incremental analyses may allocate different raw rigid IDs while remaining semantically equivalent.

Any durable comparison must alpha-normalize rigid binders/scopes.

---

## 15. Constructor-Local Existentials at Elimination

### GADT-01 — Construction-local universal becomes elimination-local existential

A constructor declared as:

```text
∀U. Payload<U> -> Result
```

is observed after construction as containing an existentially hidden `U` unless the result type externally determines it.

### GADT-02 — Each case instantiation allocates fresh rigids

For:

```phalcom
match expr {
    Equal(left, right) => ...
}
```

create a new elimination scope and fresh rigid identities for the variant-local binders.

### GADT-03 — One local binder maps to one rigid per case instantiation

Given:

```phalcom
@variant Equal<U>(
    left: Expr<U>,
    right: Expr<U>
)
```

inside one branch:

```text
U -> κ₁
left  : Expr<κ₁>
right : Expr<κ₁>
```

The implementation must never instantiate the two payload occurrences separately.

### GADT-04 — Independent observations are fresh

Two independently eliminated values generally receive distinct rigids:

```text
first match:  κ₁
second match: κ₂
```

unless equality is independently proven.

### GADT-05 — Payload/result/constraints share one case-local substitution

The binder-to-rigid substitution is allocated once and reused for:

- payload field types;
- constructor parameter types;
- result type template;
- local generic constraints.

### GADT-06 — Result compatibility feeds existing GADT proofs

Example:

```phalcom
enum Expr<T> {
    @variant Wrap<U>(_ value: U) -> Expr<List<U>>
}
```

matching a value:

```text
Expr<X>
```

creates:

```text
U -> κ₁
variant result = Expr<List<κ₁>>
```

and may prove:

```text
X = List<κ₁>
```

through the existing equality proof machinery extended to understand rigid-containing local types.

### GADT-07 — Rigid identity cannot be guessed from payload use

The checker must not infer:

```text
κ₁ = Int
```

without actual equality evidence.

---

## 16. Case Instantiation Product

A separate branch-local product is required.

Recommended conceptual shape:

```rust
struct CaseInstantiation {
    variant: VariantId,
    scope: RigidScopeId,
    local_rigids: BTreeMap<TypeParameterId, RigidTypeVariableId>,
    payload_substitution: ...,
    instantiated_result: ...,
    local_constraints: ...,
    index_equalities: ...,
}
```

The exact representation is implementation-defined.

### CASE-01 — `CaseInstantiation` and `CaseTypeEnvironment` have different authority

`CaseInstantiation` owns:

```text
fresh local existential identities
instantiated payload/result templates
local constraints
```

`CaseTypeEnvironment` owns:

```text
declaration-index equations attached to the variant semantics
```

### CASE-02 — Branch proof construction consumes both

Conceptual flow:

```text
VariantConstructorSignature
        ↓
fresh CaseInstantiation
        ↓
instantiate result/payload/local constraints
        ↓
combine with CaseTypeEnvironment
        ↓
existing GADT branch equality solver
        ↓
BranchProofEnvironment
```

---

## 17. Variant-Local Constraints During Elimination

### GADT-C-01 — Static constraints survive existential opening

For:

```phalcom
@variant Showable<U>(_ value: U)
where U <: Show
```

matching introduces:

```text
U -> κ₁
```

and branch evidence:

```text
κ₁ <: Show
```

### GADT-C-02 — Static proposition and runtime witness remain distinct

This completion project only requires static proof evidence.

If a future protocol/typeclass model needs runtime witness passing, variant storage may need explicit evidence fields. That is out of scope here.

---

## 18. Existential Escape

### EXI-01 — Branch-local rigids may not escape their defining scope

General rule:

> A type leaving scope `S` is invalid if its externally published type contains a rigid variable defined by `S`, unless an explicit existential packaging construct exists.

Phalcom currently has no general first-class existential package.

### EXI-02 — Escape checking is structural

The check walks all relevant local type forms, including:

```text
applied types
unions
tuples
records
callables
exact-case outward forms
other composite forms supported by the type kernel
```

### EXI-03 — Returning hidden value directly is rejected

Example:

```phalcom
match packed {
    Pack(value) => value
}
```

is rejected if the outward result would contain `κ`.

### EXI-04 — Assignment to an outer variable is an escape boundary

A branch may not update an outer binding so that its durable type acquires a branch-local rigid.

### EXI-05 — Wrapping does not automatically hide the existential

These still escape if the outward type contains the rigid:

```text
List<κ>
Option<κ>
Pair<Int, κ>
```

### EXI-06 — Safe widening/abstraction is allowed

If a value is soundly widened to a type that contains no branch-local rigid, the value may leave the branch.

For example, if branch evidence proves:

```text
κ <: Object
```

then publishing the value as `Object` may be valid.

### EXI-07 — Escaping closures capturing rigids are rejected initially

A closure capturing a value typed by a branch-local rigid is effectively an existential package.

Until Phalcom explicitly specifies existential closure packaging, such an escaping capture is rejected.

### EXI-08 — Durable metadata is a hard publication barrier

Any rigid reaching semantic metadata/export is an internal semantic publication failure even if an earlier user-facing escape diagnostic was missed.

---

## 19. Exact-Case Interaction

### XCASE-01 — Preserve canonical exact-case shape

The canonical representation remains:

```text
variant identity
+
enum result type
```

No branch-local rigid is stored in a global exact-case `TypeId`.

### XCASE-02 — Hidden constructor-local parameters are reconstructed freshly

Given:

```phalcom
enum Packed {
    @variant Pack<U>(_ value: U)
}
```

an exact `Pack` case does not globally record the hidden `U` as a canonical applied argument.

When the payload is eliminated/observed, the checker loads the canonical variant constructor signature and creates a fresh case instantiation.

### XCASE-03 — Exact case narrowing does not make hidden locals declaration-owned

Constructor-local parameters remain existentially scoped at elimination even if the exact variant identity is known.

---

## 20. Families and Rank-1 Polymorphism

### FAM-01 — Generic callable preservation remains rank-1

A generic callable may be:

```text
instantiated immediately
or
preserved as a Family target
```

No ordinary `forall` type constructor is introduced.

### FAM-02 — Family target identity is canonical

A Family retains the exact callable/variant target. Generic instantiation occurs when that target is invoked.

### FAM-03 — Generic variant constructor Families reuse the canonical variant signature

No Family-only variant generic signature is permitted.

### FAM-04 — Bound Family receiver specialization is semantically relevant

If a Family is captured from an applied receiver such as:

```text
Box<Int>
```

its bound receiver application must not be erased back to raw `Box` when the target's semantics depend on declaration substitution or future applied class-side storage.

### FAM-05 — Rank boundary

This project remains rank-1.

It does not introduce:

```text
(∀T. T -> T) -> X
```

or nested/impredicative universal types.

Rigid infrastructure should nevertheless remain reusable for future higher-rank checking.

---

## 21. Applied Generic Types and Per-Application Class Storage

This section supersedes the older repository direction that ordinary class-side operations do not carry declaration-generic application context.

### APCL-01 — Saturated generic applications are distinct canonical proper types

For:

```phalcom
class Box<T> { ... }
```

these are distinct:

```text
Box         :: Type -> Type
Box<Int>    :: Type
Box<String> :: Type
```

### APCL-02 — Class-side declarations of a generic class are parameterized templates

Example:

```phalcom
class Box<T> {
    @class
    const _instances: List<Box<T>>

    @class
    instances -> List<Box<T>> {
        _instances
    }
}
```

defines templates:

```text
_instances : List<Box<T>>
instances  : () -> List<Box<T>>
```

### APCL-03 — Applied receivers specialize class-side templates

```text
Box<Int>.instances
    : List<Box<Int>>

Box<String>.instances
    : List<Box<String>>
```

### APCL-04 — Applied proper types own separate class-side storage

Future executable/runtime semantics must treat:

```text
Box<Int>
Box<String>
```

as distinct class-storage owners.

The storage values are independent even when implementation code/layout is shared.

### APCL-05 — Raw unsaturated receiver does not fabricate declaration arguments

If a raw class-side member's signature depends on `T`, then:

```phalcom
Box.instances
```

is underconstrained unless surrounding inference determines a proper application.

### APCL-06 — Static declaration formation may refer to class declaration parameters

The current helper:

```text
checker/declaration_signature.rs::declaration_type_level_bindings_for_side
```

returns an empty map for class side. That current rule is superseded for generic class-side templates.

The replacement semantics are:

```text
declaration formation:
    class-side template may name declaration binders

use site:
    receiver application supplies or infers arguments
```

### APCL-07 — Applied receiver identity must survive invocation publication

For:

```phalcom
Box.new(value: 10)
```

semantic completion must retain:

```text
receiver_application = Box<Int>
```

not only:

```text
target = Box::new
result = Box<Int>
```

### APCL-08 — Selector/callable identity remains shared

`Box<Int>.instances` and `Box<String>.instances` use the same selector and canonical callable declaration.

The differing semantic context is the receiver application/substitution/storage owner.

### APCL-09 — Reification does not imply monomorphization

Canonical/reified applied type identity is compatible with:

```text
one shared method body
one shared slot layout description
multiple applied storage frames
```

### APCL-10 — Runtime implementation is out of scope for this completion program

The current project must preserve enough semantic/lowering information to permit future runtime implementation, but does not need to build applied metaclass objects or storage tables.

---

## 22. Applied Metaclass / Runtime Direction

The future runtime model should conceptually support:

```text
Box
    generic class/type-constructor object

Box.class
    generic class-side behavior template

Box<Int>
    canonical applied proper-type descriptor/object

Box<Int>.class
    applied class-side specialization/view
```

An efficient implementation may use an internal key rather than one fully materialized metaclass object per application:

```text
AppliedClassRuntimeKey {
    origin_class,
    canonical_applied_type
}
```

with lazily allocated state:

```text
AppliedClassState {
    key,
    shared_layout,
    slot_values,
}
```

Dispatch can remain selector-driven against shared behavior while class-slot operations use the applied storage owner.

No `Type.currentApplication`-style ambient user API is required by this specification.

---

## 23. Source / Native / Generated / Intrinsic Parity

### PAR-01 — All semantically equivalent callable surfaces converge on canonical semantic products

The final semantic authority is:

```text
CallableId
GenericSignature
CallableSemanticSignature
TypeParameterOwner
```

regardless of source origin.

### PAR-02 — Native metadata must be able to express callable-local generics

Current native import currently converges on `CallableSemanticSignature` but publishes `generics: None`.

Native metadata/import must eventually support:

- callable-local binders;
- kinds;
- constraints;
- callable-owned stable parameter identity.

### PAR-03 — Native inference is not separate

Once imported, native generic callables use the same application/inference pipeline.

### PAR-04 — Generated members initialize the complete AST/semantic shape

Generated getters/setters/indexers created by attributes/compiler transformations must initialize new generic/where fields consistently.

For non-generic generated accessors:

```text
generic_parameters = []
where_clause = None
```

### PAR-05 — Core ADTs should not bypass source-equivalent semantics

`Option`, `Result`, and future core enums should use the same generic variant/GADT semantics where their declarations require those features.

---

## 24. Incremental Analysis and Fingerprints

### INC-01 — Generic binder/constraint edits are semantic interface changes

Changes to callable/variant generic binders or constraints must invalidate consumers.

### INC-02 — New accessor/index generic syntax participates in fingerprints

Fingerprint inputs must include:

```text
setter generic binders
setter where clause
index getter generic binders
index getter where clause
index setter generic binders
index setter where clause
```

### INC-03 — Variant generic edits invalidate introduction and elimination consumers

Changes to:

```text
variant local binders
kinds
constraints
payload types
result type template
```

invalidate:

```text
construction sites
Family applications
pattern matching
GADT proofs
exhaustiveness
exact-case consumers
metadata
```

### INC-04 — Local rigid allocation IDs are not fingerprint inputs

Cold and incremental analyses compare alpha-equivalent branch semantics, not raw rigid IDs.

### INC-05 — Durable products remain canonical

No flexible inference variable or branch-local rigid may survive into a stable published product.

---

## 25. Metadata and Reflection

### META-01 — Existing applied type metadata remains canonical

`TypeNode::Applied` remains the durable applied-type representation.

### META-02 — Generic callable metadata preserves ownership

Declaration and callable generic signatures must remain separately representable; metadata must not publish an ownership-invalid merged constructor signature.

### META-03 — Variant constructor generic metadata is first-class

The exact variant constructor generic signature must be available through durable semantic products sufficient for:

- downstream checking;
- reflection/tooling where exposed;
- incremental fingerprints.

### META-04 — Rigids are not exportable type nodes

Branch-local rigids are never durable public type expressions.

### META-05 — Runtime reflection remains descriptive, not inference authority

Reflection reports canonical generic/applied semantic facts but does not decide inference or GADT proofs.

---

## 26. Diagnostics

The completion should expose ownership-accurate diagnostics.

Required diagnostic categories include at least:

```text
unresolved declaration-owned generic
unresolved callable-owned generic
ambiguous generic receiver application
ambiguous callable instantiation
generic constraint failure
setter/index-setter argument conflict
variant payload mismatch
variant-local constraint failure
GADT result-index incompatibility
impossible GADT case
rigid-variable incompatibility
existential escape
```

### DIAG-01 — Ownership must be visible

For:

```phalcom
class Box<T> {
    @constructor
    new<U>(value: T) { ... }
}
```

if `T` is solved and `U` is not, the diagnostic must identify the unresolved constructor-local parameter rather than reporting a generic undifferentiated failure.

### DIAG-02 — Generic getter remains underconstrained, not defaulted

Context-free generic getter failures continue to use the existing generic-inference-underconstrained behavior.

### DIAG-03 — Existential escape identifies origin and boundary

An existential escape diagnostic should identify:

```text
variant/case origin
local binder when available
escape boundary
outward type that contains the rigid
```

---

## 27. Efficiency Requirements

### PERF-01 — No declaration cloning per instantiation

Use:

```text
one declaration
+ canonical signature
+ compact substitution
+ specialized view
```

not duplicated semantic declarations per type argument.

### PERF-02 — Applied types remain canonicalized

Solved applications end in canonical `TypeId`s through existing store machinery.

### PERF-03 — Inference variables remain transient

Do not intern solver metavariables into the canonical type store merely for convenience.

### PERF-04 — Case instantiation is signature-driven

Pattern elimination should load the canonical variant constructor signature and allocate one compact rigid substitution.

It must not re-resolve source syntax for every match.

### PERF-05 — Rigid scope checks are compact

A rigid should carry a small scope/generation identifier so escape checks can test type free-rigid sets efficiently.

Free-rigid caching may be added only if profiling demonstrates repeated walks are material.

### PERF-06 — Semantic specialization and code specialization remain separate

This specification does not require monomorphization.

---

## 28. Required Positive Examples

### 28.1 Constructor declaration + callable generics

```phalcom
class Box<T> {
    @constructor
    new<U>(value: T, metadata: U) { }
}

let x = Box.new(value: 42, metadata: "literal")
```

Required semantic result:

```text
Box::T      -> Int
Box::new::U -> String
receiver    -> Box<Int>
result      -> Box<Int>
```

### 28.2 Generic getter

```phalcom
class Factory {
    @class
    default<T> -> T { ... }
}

let value: Int = Factory.default
```

Required:

```text
T -> Int
```

### 28.3 Generic setter

```phalcom
object.value = 42
```

Required generic setter behavior:

```text
T -> Int
assignment result -> Unit
```

### 28.4 Generic index getter

```phalcom
let x = store[StringKey]
```

for `store : Store<Int>` and `StringKey : Key<String>`:

```text
Store::T -> Int
index::U -> String
result   -> Pair<Int, String>
```

### 28.5 Generic variant

```phalcom
enum Expr<T> {
    @variant Literal<U>(_ value: U) -> Expr<U>
}

let x = Expr::Literal(42)
```

Required:

```text
U -> Int
x : Expr<Int>
```

### 28.6 Existential GADT branch

```phalcom
enum Expr<T> {
    @variant Equal<U>(_ left: Expr<U>, _ right: Expr<U>) -> Expr<Bool>
}
```

Inside:

```phalcom
match expr {
    Equal(left, right) => ...
}
```

required:

```text
U -> κ₁
left  : Expr<κ₁>
right : Expr<κ₁>
```

---

## 29. Required Negative / Hostile Examples

### 29.1 Constructor owner confusion

A test must fail if implementation rewrites declaration-owned `Box::T` to callable ownership merely to combine inference.

### 29.2 Bound defaulting

```phalcom
let x = Factory.default
```

must remain underconstrained even if `T <: Number`.

### 29.3 Index conflict

```phalcom
record[AgeKey] = "wrong"
```

with `AgeKey : Key<Int>` must reject.

### 29.4 Shared-rigid bug

One `Equal<U>` occurrence must not produce different rigids for left/right.

### 29.5 Cross-match rigid aliasing

Two independently eliminated existential cases must not accidentally share one rigid.

### 29.6 Rigid guessing

A branch must not become valid by assigning `κ := Int` without proof.

### 29.7 Escape by wrapper

Returning `Option<κ>` or `List<κ>` remains an escape.

### 29.8 Closure capture

Escaping closure capture of `κ` is rejected in the first version.

### 29.9 Raw generic class-side access

If `Box.instances` needs declaration `T` and no context determines it, the checker must not silently choose or erase `T`.

---

## 30. Explicitly Out of Scope

This completion does not introduce:

```text
general `forall` syntax
rank-2/rank-N polymorphism
impredicative polymorphism
general first-class existential packages
dependent types
runtime typeclass witness passing
monomorphization
specialized machine-code generation
runtime applied-class storage implementation
runtime applied metaclass object implementation
new collection representations
```

It also does not require rewriting the existing `AppliedType` reflection layer.

---

## 31. Superseded Current/Historical Rules

This specification deliberately supersedes two repository-era decisions.

### 31.1 Constructor merged generic signature

Current `merge_constructor_generic_signatures` combines declaration and callable parameters into one callable-owned `GenericSignature`.

That representation conflicts with the current publication invariant and is superseded by multi-domain application composition.

### 31.2 Empty generic scope for ordinary class-side members

Current `declaration_type_level_bindings_for_side` returns an empty map on `DispatchSide::Class`.

That rule is superseded for declaration template formation because per-application class-side storage requires class-side signatures/fields to be parameterized by the generic declaration.

This does **not** imply an ambient unsaturated application at use sites. Raw class-side access still requires saturation/inference when the member depends on declaration parameters.

---

## 32. Conformance Criteria

The type-system completion is semantically complete for this feature set only when all of the following hold:

- methods, constructors, getters, setters, index getters, index setters, and variant constructors use canonical callable-local generic ownership;
- declaration and callable generic domains remain independently publishable;
- constructors no longer publish an ownership-invalid merged generic signature;
- generic setters and index members use the ordinary application solver;
- variant-local binders and `where` constraints are represented canonically;
- generic variant construction is ordinary universal instantiation;
- GADT elimination opens constructor-local parameters as fresh rigids;
- one local binder maps to one shared rigid per case instantiation;
- independent matches receive fresh rigids;
- local constraints survive existential opening;
- declaration-index proofs continue through the existing GADT proof system;
- branch-local rigids cannot escape;
- exact-case representation remains canonical and does not persist local rigids;
- Families preserve rank-1 generic targets without introducing ordinary `forall` types;
- native/generated/intrinsic semantic inputs can converge on the same canonical products;
- cold/incremental publication is alpha-equivalent for branch-local rigids;
- applied receiver inference retains the canonical applied proper type;
- class-side generic templates specialize under applied receivers;
- static/lowering products preserve enough applied receiver identity for future per-application class storage;
- no type argument alters selector identity;
- no semantic rule requires monomorphization.

---

## 33. Repository Evidence Anchors

The implementation plan should treat the following current files/symbols as primary evidence anchors:

```text
phalcom-ast/src/ast.rs
    EnumDef
    VariantDecl
    GenericParameterSyntax
    WhereClauseSyntax
    GetterDef / SetterDef / IndexMethodDef

phalcom-ast/src/parser.rs
    generic binder parsing
    getter/setter/index member parsing
    enum/variant parsing

phalcom-ast/src/selector.rs
    selector_from_setter
    index/variant selector formation

phalcom-semantic/src/identity.rs
    VariantId
    VariantConstructorId
    CallableOwnerId
    CallableId
    InvocationTargetId

phalcom-semantic/src/types/parameter.rs
    TypeParameterOwner
    GenericSignature
    GenericSignature::validate_publishable

phalcom-semantic/src/checker/declaration_signature.rs
    resolve_callable_local_generics
    merge_constructor_generic_signatures
    declaration_type_level_bindings_for_side
    callable_id_for_syntax

phalcom-semantic/src/checker/call.rs
    canonical generic application/inference path

phalcom-semantic/src/checker/inference.rs
    InferenceSession / term conversion / constraint solving

phalcom-semantic/src/enum_semantics.rs
    VariantConstructorSignature
    VariantInfo
    EnumInfo

phalcom-semantic/src/checker/enum_declaration.rs
    enum/variant semantic construction

phalcom-semantic/src/types/case_environment.rs
    CaseTypeEnvironment

phalcom-semantic/src/checker/gadt_proof.rs
    solve_gadt_branch_proof
    equality unification

phalcom-semantic/src/checker/exhaustiveness.rs
    exact-case/pattern-space integration

phalcom-semantic/src/types/store.rs
    TypeData::Applied
    TypeData::ExactCase

phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/instantiation.rs
    canonical type materialization/substitution

phalcom-semantic/src/types/family.rs
phalcom-semantic/src/checker/associated.rs
    Family/associated target semantics

phalcom-semantic/src/db/fingerprint.rs
    declaration/member semantic fingerprints

phalcom-semantic/src/metadata/export.rs
    durable generic/type export

phalcom-type-meta/src/type_node.rs
    TypeNode::Applied

phalcom-core/src/typing/reify.rs
    runtime semantic type descriptor reification

phalcom-semantic/tests/semantic/foundations/generic_application.rs
phalcom-semantic/tests/semantic/foundations/receiver_specialization.rs
phalcom-semantic/tests/semantic/capabilities/getters.rs
phalcom-semantic/tests/semantic/adts/
phalcom-semantic/tests/semantic/families/
phalcom-semantic/tests/semantic/incremental/
```

---

## 34. Final Architectural Statement

The completed Phalcom model is:

> A gradual static type system with canonical type constructors/applications, owner-qualified declaration and callable generics, one rank-1 callable application calculus, one Family preservation mechanism, and an indexed-GADT proof system extended by scoped rigid variables for constructor-local existential elimination. Proper generic applications remain canonical/reifiable identities, and applied generic class-side state is semantically owned by the saturated application without requiring monomorphized code.

The implementation must preserve that coherence rather than adding syntax-specific mini-solvers.

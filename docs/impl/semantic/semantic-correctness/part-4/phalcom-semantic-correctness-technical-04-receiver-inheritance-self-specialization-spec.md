# Phalcom Semantic Correctness Program — Technical Specification 04
## Receiver, Inheritance, and `Self` Generic Specialization

**Status:** Repository-grounded implementation specification  
**Repository:** `aureat/phalcom-lang`  
**Verified source baseline:** `main` at `9b30ec324d4361128f285154fe236e25746df750`  
**Implementation baseline:** must be re-pinned after the corrective single-semantic-world retirement lands  
**Preceded by:** Technical Specification 03 — Generic Inference and Proof Integrity  
**Architectural prerequisite:** Part 3 single-semantic-world retirement  
**Primary golden acceptance scenario:** `golden_01_generic_self_chain`

## 1. Purpose

Technical Specifications 01–03 established three increasingly strong invariants:

```text
Technical 01
required operands cannot disappear from semantic composition

Technical 02
all call-like syntax uses one canonical callable-application law

Technical 03
generic substitution solvability does not by itself prove a result
```

Technical 04 establishes the next missing law:

> A callable selected through a generic receiver must be interpreted in the receiver's actual generic environment, including every generic inheritance edge between the receiver and the declaration that owns the selected member.

For:

```phalcom
class Maker<T> {
  wrap(_ value: T) -> Box<T> { ... }
}

class AnimalMaker<T> is Maker<T> {}

let maker: AnimalMaker<Cat>
maker.wrap(Cat.new())
```

dispatch may select:

```text
CallableId(Maker, wrap(_), Instance)
```

That callable identity remains correct.

But the raw declaration signature:

```text
(T) -> Box<T>
```

is not the call-site signature.

The call-site contract must be:

```text
(Cat) -> Box<Cat>
```

because the actual receiver:

```text
AnimalMaker<Cat>
```

projects through:

```text
AnimalMaker<T> is Maker<T>
```

to:

```text
Maker<Cat>
```

Likewise:

```phalcom
class SelfNode {
  boxed() -> Box<Self> { ... }
}

class CatNode is SelfNode {}

CatNode.new().boxed()
```

must produce:

```text
Box<CatNode>
```

not:

```text
Box<SelfNode>
Box<Self>
Unknown
```

This slice makes receiver specialization a complete compiler-owned operation instead of a collection of partial substitution helpers.

## 2. Why this is specifically Technical 04

Technical 03 explicitly excludes complete receiver/class generic substitution through:

```text
method parameters
method returns
method-generic bounds
enclosing where constraints
```

and assigns that boundary to Technical 04.

The repository's remaining golden coverage makes the same boundary concrete. `golden_01_generic_self_chain` remains ignored specifically because it waits for:

```text
generic inheritance
nested Self specialization
```

Its source combines all of the required mechanisms:

```phalcom
class Box<+T> { ... }

class Maker<T> {
  wrap(_ value: T) -> Box<T> { ... }
}

class AnimalMaker<T> is Maker<T> { ... }

class SelfNode {
  boxed() -> Box<Self> { ... }
}

class CatNode is SelfNode {}
```

and then exercises them compositionally.

That is therefore the correct semantic boundary for 04.

## 3. Existing architecture that must be preserved

Technical 04 is not starting from zero.

Several important foundations already exist and should be completed rather than replaced.

### 3.1 Generic parameter identity is already owner-correct

The canonical parameter model distinguishes:

```rust
TypeParameterOwner::Declaration(DeclarationId)
TypeParameterOwner::Callable(CallableId)
```

and `TypeParameterData` carries owner, index, name, kind, variance, and source metadata. `GenericSignature` also owns canonical constraints.

This means 04 must always substitute using `TypeParameterId`.

It must never implement substitution by matching parameter names such as `"T"`.

For:

```phalcom
class Box<T> {
  convert<T>(_ value: T) -> ...
}
```

the class `T` and callable `T` are different semantic identities even though their source spellings coincide.

### 3.2 Generic superclass templates already exist

`DeclarationTypeInfo` already publishes:

```rust
generic_signature: Option<GenericSignature>
supertype_template: Option<GenericSupertypeTemplate>
```

and the template contains the canonical specialized superclass type expression.

For:

```phalcom
class Child<T> is Parent<List<T>>
```

the declaration product can therefore represent:

```text
Child<T>
    ↓ template
Parent<List<T>>
```

This is exactly the representation 04 needs.

### 3.3 Workspace declaration construction already resolves generic superclass templates

Current `SemanticWorkspaceSession` already resolves class generics and evaluates the superclass annotation under those parameters, producing a `GenericSupertypeTemplate`.

So the parser/declaration layer is not the main missing piece.

The problem is that the resulting type-level inheritance information is not carried far enough into call-site specialization.

### 3.4 `TypeEnvironment` and `TypeView` already implement specialization mechanics

Current code has:

```rust
TypeEnvironment {
    bindings: HashMap<TypeParameterId, TypeId>,
    self_binding: Option<TypeId>,
}

TypeView {
    root: TypeId,
    environment: TypeEnvironment,
}
```

and `TypeView::materialize()` recursively handles parameters, `Self`, applied types, unions, tuples, records, and callables.

This is the right underlying abstraction.

Technical 04 should build correct environments.

It should not write another unrelated recursive type-substitution engine.

### 3.5 Low-level generic inheritance already works in isolation

The existing foundational tests manually construct:

```text
Names<T> is Sequence<T>
```

install the generic supertype template, and prove:

```text
Names<Int> <: Sequence<Int>
```

through the canonical relation engine.

So generic inheritance is already partially real.

The source/workspace/call path is what remains incomplete.

### 3.6 Direct receiver specialization already exists

Current `CheckingContext` already contains a first-generation receiver-specialization path. It recognizes an applied receiver, builds a direct substitution for that declaration, applies it to parameters and return type, and then specializes `Self`.

This is an important implementation finding.

04 must therefore not be implemented as:

> “add generic receiver specialization.”

It must be implemented as:

> “replace direct-owner-only receiver specialization with complete owner-relative specialization.”

## 4. Current correctness gaps

### RSI-01 — Receiver substitution only understands the receiver's own declaration

The present helper can turn:

```text
Box<Cat>
raw Box<T>.value() -> T
```

into:

```text
value() -> Cat
```

because the receiver and callable owner are both `Box`.

It cannot correctly derive:

```text
AnimalMaker<Cat>
        ↓
Maker<Cat>
        ↓
Maker<T>.wrap(T) -> Box<T>
        ↓
wrap(Cat) -> Box<Cat>
```

because the member is owned by a different declaration.

The missing concept is:

```text
actual receiver
    projected to
member-owning ancestor
```

### RSI-02 — The workspace hierarchy discards type-level superclass specialization

Current workspace construction computes `supertype_template`, but later builds the runtime checking hierarchy primarily with:

```rust
hierarchy.insert(class_decl, super_decl)
```

which preserves only:

```text
Child -> Parent
```

rather than:

```text
Child<T> -> Parent<List<T>>
```

This explains the disconnect between low-level generic-supertype tests and source-level generic inheritance.

### RSI-03 — Generic hierarchy topology and generic specialization are conflated in `MapTypeHierarchy`

The current hierarchy implementation has support for generic templates, and the relation engine knows how to specialize one when comparing applied types.

But this support has never become the authoritative workspace inheritance environment.

Technical 04 needs a clear distinction:

```text
nominal topology:
Child -> Parent

type projection:
Child<T> -> Parent<List<T>>
```

Both describe the same edge but answer different questions.

### RSI-04 — Only callable parameter and return types are currently receiver-specialized

A complete callable signature also contains:

```rust
GenericSignature {
    parameters,
    constraints,
}
```

Current direct specialization handles callable parameter/return `TypeKnowledge`, but not the generic constraints themselves.

Consider:

```phalcom
class Holder<T> {
  select<U>(_ value: U) -> U
    where U <: T
}
```

For:

```text
Holder<Animal>
```

the method's actual callable constraint is:

```text
U <: Animal
```

not:

```text
U <: Holder::T
```

before Technical 03 instantiates `U`.

### RSI-05 — Method-local inference currently receives unspecialized enclosing parameters

Technical 03's canonical generic path correctly creates inference variables only for the callable's own `GenericSignature`.

That is correct.

But any remaining class-owned type parameter inside:

```text
parameters
return
where constraints
```

is not a method inference variable.

It is a canonical declaration-owned parameter `TypeId`.

Therefore the correct order must be:

```text
specialize enclosing declaration environment
        BEFORE
instantiate method-local inference variables
```

Never the reverse.

### RSI-06 — Inherited `Self` and generic substitution are not one coherent operation

`Self` has explicit canonical roles already:

```rust
SelfRole::InstanceType
SelfRole::ReceiverValue
```

The important law is:

> An inherited member's `Self` denotes the actual receiver, not the declaration that supplied the method.

For:

```text
receiver = CatNode
member owner = SelfNode
```

the inherited raw:

```text
Box<Self>
```

must materialize as:

```text
Box<CatNode>
```

The generic ancestor environment and `Self` binding therefore belong in one specialization environment.

### RSI-07 — Generic member specialization must cover all member contracts, not methods only

Generic receiver substitution also affects:

```text
getters
setters
indexers
field contracts
constructors
```

A generic field:

```phalcom
class Cell<T> {
  _value: T
}
```

cannot legitimately be read from `Cell<Int>` as raw declaration parameter `T`.

Callable dispatch already gives getters/setters/indexers the opportunity to share one signature-specialization path.

Direct field access must consume the same receiver environment.

### RSI-08 — Constructor `Self` must preserve class specialization

For:

```phalcom
Box<Cat>.new(cat)
```

the constructor result proposition is not merely:

```text
Box
```

and it is not an unsaturated:

```text
Box<_>
```

It is:

```text
Box<Cat>
```

The constructor remains semantically special because its return authority is `ConstructorSemantics`, but its result type still needs the same receiver/class specialization environment.

This must be solved without creating a second constructor-specific generic inference engine.

### RSI-09 — Generic-supertype changes need ordinary incremental dependency tracking

Changing:

```phalcom
class Child<T> is Parent<T>
```

to:

```phalcom
class Child<T> is Parent<List<T>>
```

can change the effective signature of every inherited generic member on `Child`.

Therefore callable body analysis that uses such a member must depend on the relevant canonical declaration/hierarchy products.

No session-local “invalidate generic subclasses” list should be introduced.

## 5. End-state semantic pipeline

The canonical call pipeline after Technical 04 is:

```text
receiver expression
        ↓
Known receiver TypeId
        ↓
dispatch
        ↓
raw canonical callable identity + raw declaration signature
        ↓
receiver-to-member-owner specialization
        │
        ├─ declaration generic bindings
        ├─ inherited generic projection
        └─ actual Self binding
        ↓
receiver-specialized CallableSignature
        │
        ├─ specialized parameters
        ├─ specialized return
        └─ specialized method-generic constraints
        ↓
Technical 02 canonical callable application
        ↓
if callable has method-local generics:
        Technical 03 inference + proof accounting
        ↓
CallCheckResult
```

The distinction between identity and specialization is crucial.

This:

```text
AnimalMaker<Cat>.wrap
```

still identifies:

```text
CallableId {
    owner = Maker,
    selector = wrap(_),
    side = Instance
}
```

It does not create:

```text
CallableId<Maker<Cat>>
```

or:

```text
SpecializedCallableId
```

Type arguments are not selector identity.

## 6. Canonical receiver-specialization product

Introduce a protocol-independent specialization model inside `phalcom-semantic`.

Recommended location:

```text
phalcom-semantic/src/types/specialization.rs
```

Recommended core shape:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReceiverSpecialization {
    pub receiver: TypeId,
    pub receiver_declaration: DeclarationId,
    pub member_owner: DeclarationId,
    pub environment: TypeEnvironment,
    pub path: Box<[ReceiverSpecializationStep]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReceiverSpecializationStep {
    pub declaration: DeclarationId,
    pub specialized_type: TypeId,
}
```

The important product is `environment`.

`path` exists for:

```text
dependency recording
debugging
explanations
verification
```

not because semantic identity depends on the traversal spelling.

Failure should be structured:

```rust
pub enum ReceiverSpecializationFailure {
    UnsupportedReceiver(TypeId),
    MissingDeclarationMetadata(DeclarationId),
    MissingInheritanceProjection {
        declaration: DeclarationId,
        superclass: DeclarationId,
    },
    MalformedGenericApplication {
        declaration: DeclarationId,
    },
    OwnerNotReachable {
        receiver: DeclarationId,
        member_owner: DeclarationId,
    },
    Cycle {
        declaration: DeclarationId,
    },
}
```

Do not return raw `Option`.

A missing specialization proof is semantically meaningful.

## 7. Direct receiver environment law

For:

```text
Box<Cat>
```

where declaration metadata says:

```text
Box<T>
```

construct:

```text
T -> Cat
Self -> Box<Cat>
```

The direct bindings are based on `TypeParameterId`, not names.

For a non-generic receiver:

```text
CatNode
```

the declaration binding map is empty but:

```text
Self -> CatNode
```

still exists.

## 8. Generic inheritance projection law

Consider:

```phalcom
class Parent<A> {}

class Middle<B> is Parent<List<B>> {}

class Child<C> is Middle<Option<C>> {}
```

and receiver:

```text
Child<Int>
```

To specialize a member owned by `Parent`, compute:

```text
step 0
Child<C>
C -> Int

materialize Child's supertype template:
Middle<Option<C>>
    ↓
Middle<Option<Int>>

step 1
Middle<B>
B -> Option<Int>

materialize Middle's supertype template:
Parent<List<B>>
    ↓
Parent<List<Option<Int>>>

step 2
Parent<A>
A -> List<Option<Int>>
```

The resulting member-owner environment is:

```text
Parent::A -> List<Option<Int>>
Self      -> Child<Int>
```

This algorithm works for arbitrary finite inheritance depth.

It must not assume that child parameter position `0` corresponds to parent parameter position `0`.

## 9. `Self` law

The actual receiver binding is invariant across ancestor projection.

For:

```text
Child<Int>
  → Middle<Option<Int>>
  → Parent<List<Option<Int>>>
```

an inherited `Parent` method returning:

```text
Self
```

still returns:

```text
Child<Int>
```

not:

```text
Parent<List<Option<Int>>>
```

This is the difference between:

```text
member-owner generic environment
```

and:

```text
actual receiver identity
```

Both are needed simultaneously.

## 10. Callable signature specialization

Given:

```rust
CallableSignature {
    parameters,
    return_type,
    generics,
    ...
}
```

specialization must produce a new call-site signature without mutating the declaration surface.

For each parameter:

```text
parameter.ty
    ↓ TypeView(receiver environment)
specialized parameter.ty
```

For return:

```text
return_type
    ↓
specialized return
```

Evidence metadata must be preserved.

The transformation changes the canonical `TypeId` inside `TypeKnowledge`.

It must not change:

```text
Established ↔ Assumed
EvidenceOrigin
causal invalidity
callable identity
```

because substitution is not new evidence.

## 11. Method generic constraints

Suppose:

```phalcom
class Container<T> {
  convert<U>(_ value: U) -> Pair<T, U>
    where U <: T
}
```

on:

```text
Container<Animal>
```

the raw method signature contains two distinct parameters:

```text
Declaration(Container)::T
Callable(convert)::U
```

Receiver specialization must transform only the enclosing declaration references:

```text
parameter:
U                          // untouched

return:
Pair<T,U>
    ↓
Pair<Animal,U>

constraint:
U <: T
    ↓
U <: Animal
```

Only after that does Technical 03 instantiate:

```text
Callable(convert)::U
    ↓
InferVar(...)
```

This gives the required composition law:

```text
receiver substitution
then
callable inference
```

## 12. Raw surfaces remain generic

`DeclarationSurface` remains declaration-owned.

For:

```phalcom
class Box<T> {
  value() -> T
}
```

the surface should continue to publish:

```text
Box<T>.value() -> T
```

It must not be overwritten after observing:

```text
Box<Int>
```

or:

```text
Box<String>
```

Call-site specialization is a view.

It is not mutation of declaration truth.

Consequently no cache may be keyed only by:

```text
CallableId
```

and hold a receiver-specialized signature.

If specialized signatures are eventually cached, the key must include the relevant canonical receiver specialization identity.

For this slice, ephemeral specialization during dispatch is preferable.

## 13. Hierarchy architecture

`MapTypeHierarchy` should own both:

```text
direct nominal parent
generic direct-supertype template
```

without conflating them.

Recommended API:

```rust
pub fn insert(
    &mut self,
    subclass: DeclarationId,
    superclass: DeclarationId,
);

pub fn insert_template(
    &mut self,
    template: GenericSupertypeTemplate,
);
```

but `insert_template` must only register the template.

It must not manufacture a fake superclass.

Workspace construction should perform:

```rust
hierarchy.insert(
    class_decl.clone(),
    super_decl.clone(),
);

if let Some(template) =
    declarations.supertype_template(&class_decl).cloned()
{
    hierarchy.insert_template(template);
}
```

That makes source-level analysis use the same generic-supertype machinery already proven by the low-level generic tests.

## 14. Dispatch architecture

Dispatch continues to answer:

> Which canonical member is selected?

It does not answer:

> What is this member's type under the current receiver?

Thus:

```text
dispatch
    returns raw ResolvedDispatch
```

then:

```text
specialization
    returns receiver-specialized signature
```

then:

```text
call application
```

The current placement inside `CheckingContext::resolve_dispatch_target` is already close to the right boundary.

The existing direct-only helpers should be replaced rather than supplemented by a second inherited path.

## 15. Direct field specialization

Field access needs the same owner-relative environment.

For:

```phalcom
class Parent<T> {
  _value: T
}

class Child<U> is Parent<List<U>> {}
```

inside a context where the receiver is:

```text
Child<Int>
```

the inherited field contract is:

```text
List<Int>
```

not raw:

```text
Parent::T
```

The field identity remains:

```text
FieldId(Parent, "_value", Instance)
```

Again:

```text
identity belongs to declaration
type belongs to receiver-specialized view
```

## 16. Constructor/class-side specialization

Constructor handling must respect two separate facts.

First:

```text
constructor callable identity
```

is a class-side declaration member.

Second:

```text
constructor result
```

is an instance-side `Self`.

Therefore for:

```text
Box<Cat>.new(cat)
```

the result must materialize as:

```text
Box<Cat>
```

while retaining:

```text
EvidenceOrigin::ConstructorSemantics
```

Technical 04 should not infer class parameters through a new constructor-only solver.

Explicit class specialization is receiver context.

Method-local generics remain Technical 03's solver domain.

Any already-supported constructor class-parameter inference must be routed through the canonical class/type-application model rather than duplicated.

## 17. Formal failure behavior

The checker must never handle failed specialization by applying the raw declaration signature as if it were already concrete.

Forbidden:

```text
cannot project Child<Int> to Parent<T>
    ↓
just use Parent<T>.member(...)
```

That turns missing proof into apparent generic semantics.

Instead:

```text
dispatch may have identified the member
but
signature specialization is incomplete
```

must produce a structured blocked/incomplete semantic outcome or an internal incident if the hierarchy and declaration products contradict one another.

Examples:

```text
unresolved source superclass
    -> source diagnostic + blocked specialization

hierarchy says Parent but declaration product has impossible malformed template
    -> internal semantic incident

inheritance cycle
    -> bounded/cycle failure

unknown receiver
    -> existing Unknown path before dispatch

dynamic receiver
    -> existing DynamicBoundary path
```

## 18. Incremental dependency law

A callable body using an inherited specialized member depends on:

```text
the selected callable signature
+
the receiver declaration shell
+
every superclass declaration shell traversed
+
every hierarchy edge traversed
+
the member owner's declaration surface
```

Current callable analysis already has canonical `SemanticDependency` categories for:

```text
DeclarationShell
CallableSignature
DeclarationSurface
HierarchyEdge
LinkedInterface
```

Use those.

Do not add session-local invalidation logic.

An edit to:

```phalcom
class Child<T> is Parent<T>
```

must invalidate dependent inherited-member analysis if changed to:

```phalcom
class Child<T> is Parent<List<T>>
```

through changed query/product fingerprints.

Unrelated callables should continue to reuse their products.

## 19. Relationship to source positions and incremental reuse

Technical 04 must preserve the established separation between semantic input identity and incidental source movement.

Generic receiver specialization depends on:

```text
canonical TypeIds
declaration metadata
hierarchy templates
canonical callable identity
```

not absolute source offsets.

A trivia-only movement of a superclass declaration must not change specialized semantic meaning.

## 20. Relationship to LSP

Technical 04 adds no LSP semantic implementation.

After the retirement:

```text
phalcom-semantic
    computes specialized receiver facts

SemanticSnapshot
    publishes resulting expression/call/binding facts

phalcom-lsp
    renders them
```

No generic-inheritance or `Self` substitution code belongs in:

```text
phalcom-lsp
```

This is why the single-world retirement is a hard execution prerequisite.

## 21. Non-goals

Technical 04 does not implement:

```text
type-lambda parser completion
HKT inference expansion
Family activation
iteration protocol stabilization
closure contextual typing
recursive generic fixed points
row/effect contracts
nominal branch product joins
broad variance recovery
new generic syntax
global generic argument inference
```

Those are independently gated areas.

It also does not redesign Technical 03's:

```text
InferenceSession
InferenceProofState
required generic premise accounting
expected-result selection law
```

The only interaction with Technical 03 is supplying it a correctly receiver-specialized method signature.

## 22. Definition of done

Technical 04 is complete when all of these are true:

```text
1. Receiver generic specialization is owner-relative, not direct-owner-only.

2. Generic inheritance templates from source declarations participate in
   the workspace hierarchy.

3. Multi-hop generic inheritance specializes correctly.

4. Generic superclass transformations such as Parent<List<T>> specialize.

5. Inherited Self denotes the actual receiver.

6. Nested Self inside Box<Self>, tuples, callables, records, unions, etc.
   specializes recursively.

7. Method-local generics remain separate from declaration generics.

8. Enclosing class parameters inside method-generic where constraints are
   specialized before Technical 03 inference.

9. Generic getters/setters/indexers/direct fields receive the same receiver
   environment.

10. Generic constructors preserve applied class Self.

11. Raw declaration surfaces remain generic and immutable.

12. CallableId and FieldId never include type arguments.

13. No parameter-name-based generic substitution exists.

14. Missing specialization cannot silently fall back to a raw signature.

15. Generic-supertype edits invalidate dependent callable products through
    canonical query dependencies.

16. Independent callable products remain reusable.

17. golden_01_generic_self_chain is unignored and green.

18. No unrelated golden ignore is removed as part of this slice.

19. Technical 01–03 suites remain green.

20. phalcom-lsp remains a consumer only; no 04 logic exists there.
```

# SC-2 — Generic Callable Application, Receiver Specialization, Constraint Solving, and Executable Typing Closure

**Project:** Phalcom  
**Deliverable:** Technical specification  
**Date:** 2026-09-01  
**Repository:** `aureat/phalcom-lang`  
**Repository baseline inspected:** `main@01e19adb86186d67212b558ba76f54f79e2b5d9f` (`feat(core,semantic,vm): canonical native enums and associated lookup implementation`)  
**Status:** implementation-ready after SC-1 acceptance; repository-grounded against the baseline above  
**Primary owner:** `phalcom-semantic`  
**Depends on:** SC-1 — Type Formation, Kinds, Generic Declarations, and Type-Level Source Semantics  
**Feeds:** SC-3 rows; SC-6 convergence/incrementality; SC-7 generic getters; SC-8 semantic-completeness certification

---

## 0. Executive contract

SC-2 does **not** introduce a new generic type system. Phalcom already has a substantial generic-call inference substrate on current `main`:

- `CallableApplicationTarget`, `CallPremise`, normalized application arguments, static call-shape binding, and the canonical `apply_resolved_callable(...)` funnel;
- session-local `InferenceSession` / `InferVarId` rather than canonical inference nodes;
- kind-carrying inference variables;
- expected-type-aware generic argument analysis;
- generic declaration constraints with provenance;
- explicit solved / underconstrained / conflicting / blocked / cancelled / budget / internal outcomes;
- proof/evidence weakening separate from canonical `TypeId` identity;
- canonical generic superclass templates and variance-aware canonical type relations;
- canonical enum/variant constructor products and GADT case environments.

SC-2 closes the remaining semantic gaps around that substrate so that **every executable generic application uses one sound model**.

The target law is:

> **Resolve one ordinary runtime callable identity, build an owner-relative static specialization view, instantiate only the residual generic binders as solver-local variables, solve with argument/context/semantic constraints, validate declaration restrictions, materialize only publishable canonical results, and preserve every non-success state without guessing.**

The target pipeline is:

```text
canonical declarations/signatures from SC-1
                ↓
ordinary selector / constructor / variant resolution
                ↓
selected stable callable/constructor identity
                ↓
owner-relative receiver / owner specialization
                ↓
residual generic instantiation
                ↓
static argument binding exactly once
                ↓
argument + expected-result + exact-semantic constraints
                ↓
declaration restrictions / GADT obligations
                ↓
bounded local inference
                ↓
Solved | Underconstrained | Ambiguous | Conflicting
       | Blocked | DynamicBoundary | Cancelled
       | BudgetExceeded | InternalFailure
                ↓
canonical materialization + evidence classification
                ↓
CallCheckResult / ExpressionAnalysis / explanation / dependencies
```

No stage may invent a different selector, runtime method identity, specialized runtime class, or per-instance generic representation.

---

# Part I — Authority and scope

## 1. Normative source authority

SC-2 specializes and rebases the ordinary-call portions of the already-ratified generic architecture. On semantic conflict, authority is:

```text
ratified semantic rules in 01.5 / 04.5 / revised 05 / revised 06
        ↓
this SC-2 closure specification
        ↓
current canonical compiler representation
        ↓
current implementation accident / historical plan text
```

Historical implementation-state statements such as “`TypeData::Infer` is still active” are not carried forward when current source disproves them. Current `main` uses session-local ordinary inference; searches at the inspected baseline find no production `TypeData::Infer`, `TypeStore::infer`, or `LocalConstraintSolver` ordinary-inference path.

SC-2 therefore starts from the **current implementation**, not from the older migration baseline.

## 2. What SC-2 owns

SC-2 is authoritative for:

1. executable callable-application typing after canonical signature publication;
2. owner-relative generic receiver specialization, including transformed and multi-hop inheritance;
3. owner-relative `Self` specialization at call sites;
4. residual declaration-generic and callable-generic instantiation at one application site;
5. argument-to-parameter binding and generic constraint generation;
6. bidirectional expected-result participation in local generic inference;
7. contextual closure/callable argument typing when generic terms occur in their expected types;
8. variance-aware solver-local subtype/equivalence decomposition;
9. higher-kinded local inference over explicit monomorphic arrow kinds and type lambdas;
10. generic declaration restriction semantics at application time;
11. F-bound handling as relational obligations rather than recursive equality;
12. underconstraint, finite ambiguity, conflict, blocked, dynamic, cancelled, budget, and internal outcomes;
13. generic constructor application for ordinary classes;
14. generic enum/ADT variant construction, including nullary constructors and residual owner parameters;
15. GADT constructor/case specialization constraints at construction sites;
16. generic callable recovery for first-class family/associated values whose denotation retains a concrete target;
17. union-receiver call validity and result joining;
18. generic call explanation/provenance and exact dependency capture;
19. source/native/generated/constructor conformance through the same application algorithm;
20. publication rules ensuring solver-local state never escapes.

## 3. Explicit non-goals

SC-2 does not introduce:

- first-class `forall` or polymorphic values;
- higher-rank/rank-N or impredicative inference;
- public kind polymorphism or kind variables;
- dependent types;
- generic defaults;
- finite exact-set generic bounds;
- intersection types unless separately ratified;
- implicit/given/type-class search as hidden call input;
- type-directed runtime overload selection;
- generic setter syntax;
- generic indexer syntax unless separately ratified;
- generic getter syntax/AST/signature support — SC-7;
- open record-row inference — SC-3;
- effect inference — SC-4;
- proof/contract inference — SC-5;
- runtime generic monomorphization as language semantics;
- specialized runtime class/metaclass identity.

---

# Part II — Repository-grounded baseline

## 4. Current architecture to preserve

### 4.1 Canonical application funnel

`phalcom-semantic/src/checker/call.rs` already contains the correct application ownership seam:

```text
CallableApplicationTarget
CallPremise
ApplicationArgument
ArgumentBindingPlan
apply_resolved_callable(...)
```

Both generic and non-generic application pass through `apply_resolved_callable`. SC-2 strengthens this path; it must not create parallel application engines such as:

```text
infer_source_generic_call
infer_native_generic_call
infer_variant_generic_call
infer_family_generic_call
```

### 4.2 Session-local inference

`phalcom-semantic/src/checker/inference.rs` already makes `InferVarId` a solver-local entity. Fresh inference variables carry their declared `KindId`, and successful materialization interns only canonical solved types.

This is now a protected invariant:

```text
InferVarId != TypeId
```

SC-2 has no “delete canonical infer nodes” implementation task because that ordinary-inference migration is already complete on the inspected baseline.

### 4.3 Canonical relations are richer than solver-local relations

`phalcom-semantic/src/types/relation.rs` already implements bounded canonical subtyping for:

- nominal inheritance;
- generic supertype templates;
- declaration-site variance;
- unions;
- exact cases;
- tuples;
- callable contravariant parameters / covariant return;
- records under the current record relation policy;
- callable/family structural relations;
- cancellation and budgets.

The solver-local relation calculus is currently less expressive. SC-2 must make inference-term relations **conform to** the canonical relation semantics rather than creating a second inconsistent relation language.

### 4.4 Canonical signature ownership exists

`CallableSemanticSignature` / `CallableSignatureTable` in `phalcom-semantic/src/signature.rs` are the canonical declaration products. `CallableSignature` in dispatch is a checking projection.

SC-2 must preserve stable `CallableId` identity and treat specialization as a view. It should reduce eager copied specialization where practical, but it need not redesign all signature storage—that broader convergence remains SC-6.

### 4.5 Generic enum products exist

`EnumInfo`, `VariantInfo`, `VariantConstructorSignature`, `CaseTypeEnvironment`, result templates, exact-case templates, and stable constructor identities already exist. The remaining gap is application-time residual generic inference, not enum declaration representation.

---

# Part III — Non-negotiable semantic laws

## 5. Runtime identity law

Static specialization never changes:

- selector identity;
- `CallableId`/variant constructor declaration identity;
- runtime method-dictionary keys;
- dispatch-cache keys;
- class/metaclass identity;
- instance layout;
- allocation layout.

For a generic method:

```phalcom
map<U>(...)
```

`U = String` is a static call-site specialization. It is not encoded into the runtime selector.

## 6. Receiver specialization precedes local inference

For:

```phalcom
class Pairer<T> {
    pair<U>(_ value: U) -> (T, U)
}

const p: Pairer<Int> = ...
p.pair("x")
```

the environments are:

```text
declaration/receiver environment:
    Pairer::T := Int

call-local inference environment:
    pair::U := ?u0
```

They are different identity domains even when represented by the same generic `TypeParameterId` mechanism.

The operation order is mandatory:

```text
project receiver to selected declaring owner
    ↓
specialize declaration-owned type parameters and Self
    ↓
instantiate unresolved declaration parameters, if the invocation owns any
    ↓
instantiate callable-local generic parameters
    ↓
solve call-local constraints
```

## 7. Owner-relative inheritance law

A selected inherited member is specialized against the generic arguments of **its declaring owner**, not merely the root receiver declaration.

For:

```phalcom
class Parent<T> {
    value() -> T
}

class Child<T> is Parent<List<T>> {}

const x: Child<Int> = ...
x.value()
```

SC-2 must derive:

```text
Child<Int>
  -> Parent<List<Int>>
  -> Parent::T := List<Int>
  -> value() -> List<Int>
```

A direct substitution `Child::T := Int` is insufficient because the selected signature contains `Parent::T`.

## 8. Expected context is selection, not value evidence

Expected-result context may uniquely determine otherwise-unsolved generic parameters.

Ratified SC-2 behavior:

```phalcom
make<T>() -> T

const value: Int = make()
```

may solve:

```text
T := Int
```

Likewise:

```phalcom
empty<T>() -> List<T>
const xs: List<Int> = empty()
```

may solve `T := Int`.

However, the annotation is not runtime evidence. A result depending only on contextual selection cannot become `Established` merely because the context requested it. Under the current evidence lattice it is published at most as `Assumed`/contextually derived.

Without a selecting context:

```phalcom
const value = make()
```

remains underconstrained.

This replaces the current regression policy that diagnoses expected-only `<T>() -> T` as underconstrained. The replacement is intentional and required by the ratified 01.5/04.5 inference model and by the later generic-getter design.

## 9. Context cannot overwrite stronger type evidence

Given:

```phalcom
identity<T>(_ value: T) -> T
const x: Int = identity("wrong")
```

argument evidence selects `T := String`. Expected `Int` may then produce a contextual contradiction, but cannot silently replace the precise argument-derived solution with `T := Int`.

A valid implementation may publish:

```text
knowledge = String
status    = Invalid(...)
```

when the `String` result proposition was independently complete before the contradictory expected-result constraint was added.

## 10. Declaration restrictions are not arbitrary defaults

A `where` constraint participates formally, but a one-sided bound does not by itself nominate a call-site specialization.

Example:

```phalcom
make<T>() -> T
    where T <: Number
```

With no argument, explicit type argument, exact semantic constraint, or expected result, SC-2 must **not** infer:

```text
T := Number
```

merely because `Number` is the only stored upper bound.

The call is underconstrained.

The restriction may reject a selected candidate:

```text
T := String
String <: Number  // refuted
```

or confirm a selected candidate:

```text
T := Int
Int <: Number     // proven
```

Exact/equivalence restrictions may propagate a uniquely anchored solution. The implementation must distinguish “candidate selection” from “candidate admissibility” instead of using raw bound cardinality as a defaulting rule.

## 11. F-bounds are relations, not recursive equalities

For:

```text
where T <: Comparable<T>
```

SC-2 must not transform the relation into:

```text
T == Comparable<T>
```

and then report an occurs-check failure.

Correct behavior:

```text
select candidate T := User
        ↓
validate User <: Comparable<User>
```

If no candidate is selected, the call remains underconstrained; the bound does not manufacture one.

## 12. No generic default to `Object`, `Dynamic`, or the first bound

An unsolved generic parameter remains explicitly unsolved. No compatibility fallback may turn it into:

- `Object`;
- `Dynamic`;
- `Any`;
- `Never`;
- the first upper bound;
- the first lower bound when no principal join rule applies;
- the first candidate encountered by traversal order.

## 13. No partial specialization after terminal failure

For:

```text
Result<T, U>
```

if `T` is solved and `U` is not, SC-2 does not publish `Result<Int, U>` as ordinary known expression knowledge.

Only a return contract genuinely independent of failed/unsolved generic variables may survive a terminal inference outcome.

## 14. Determinism

For semantically identical input, successful generic substitution and final expression knowledge are independent of:

- hash-map iteration order;
- constraint insertion order;
- inheritance traversal incidental allocation order;
- diagnostic collection order.

Stable ordering may affect presentation ordering only.

---

# Part IV — Receiver specialization model

## 15. Canonical owner-relative specialization product

Create `phalcom-semantic/src/types/specialization.rs` and centralize generic receiver/owner projection there.

Recommended semantic shape:

```rust
pub struct ReceiverSpecialization {
    pub receiver: TypeId,
    pub receiver_owner: DeclarationId,
    pub target_owner: DeclarationId,
    pub environment: TypeEnvironment,
    pub path: Box<[ReceiverSpecializationStep]>,
}

pub struct ReceiverSpecializationStep {
    pub owner: DeclarationId,
    pub specialized_form: TypeId,
}

pub enum ReceiverSpecializationFailure {
    UnsupportedReceiver,
    TargetNotReachable,
    InvalidSupertypeTemplate,
    InheritanceCycle,
    Blocked(BlockReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure,
}
```

Exact Rust names are implementation choices. The semantic requirements are:

1. retain the original actual receiver type;
2. identify the root receiver declaration;
3. identify the selected declaring owner;
4. project generic arguments through every superclass template until that owner;
5. bind `TypeParameterId`s belonging to the selected owner;
6. bind `Self` to the actual receiver according to `SelfRole`/side;
7. retain enough traversal information for dependency capture and explanations;
8. detect cycles and malformed templates explicitly.

## 16. Reuse current generic-supertype projection

`checker/associated.rs::project_supertype_arguments(...)` already performs much of the required multi-hop projection using `TypeEnvironment` and `TypeView`.

SC-2 extracts/generalizes that algorithm into the type specialization module. Ordinary dispatch, associated lookup, inherited field/member viewing, and later consumers must call the same implementation.

After migration there must not be independent algorithms named conceptually:

```text
substitution_for_applied_receiver       // complete dispatch algorithm
project_supertype_arguments             // associated-only complete algorithm
specialize_inherited_member_elsewhere   // third algorithm
```

A direct substitution helper may remain as a low-level primitive, but it is not the definition of inherited specialization.

## 17. Specialize the entire callable contract

Receiver specialization applies to all declaration-owned type facts consumed by a call:

- fixed parameters;
- labeled/rest parameters;
- return type;
- nested callable types;
- nested tuples/unions/applications;
- `Self` occurrences;
- callable-local `where` constraints that reference declaration parameters;
- constructor semantic return templates where appropriate.

Current `CheckingContext::specialize_dispatch_signature` specializes parameter/return types but not `signature.generics.constraints`. SC-2 closes that hole.

## 18. Prefer views over eager subtree reconstruction

The semantic model is:

```text
canonical signature + ReceiverSpecialization environment
```

not:

```text
clone entire canonical signature
recursively substitute every nested node
intern every rebuilt subtree
```

A materialized projected `CallableSignature` may remain at compatibility boundaries, but the application solver should consume lazy/environment-aware terms wherever practical.

The recommended integration is an environment-aware lifting operation:

```rust
fn type_id_to_inference_under_environment(
    ty: TypeId,
    receiver_env: &TypeEnvironment,
    inference_params: &HashMap<TypeParameterId, InferenceTerm>,
    store: &TypeStore,
) -> Result<InferenceTerm, InferenceLiftFailure>;
```

This simultaneously:

- substitutes fixed declaration parameters;
- substitutes `Self`;
- replaces residual/callable generic parameters with solver variables;
- leaves unrelated canonical parameters untouched or reports malformed ownership as appropriate.

---

# Part V — Generic instantiation domains

## 19. Callable-local generics

For an ordinary method call, each callable-owned generic parameter receives one fresh solver-local variable with its declared kind.

Example:

```phalcom
identity<T>(_ value: T) -> T
identity(42)
```

becomes:

```text
T -> ?t0 : Type
Int <: ?t0
```

## 20. Residual declaration generics

Some invocation surfaces operate on a declaration owner whose generic arguments are not fully supplied.

Examples:

```phalcom
Result::Ok(1)
Option::None()
GenericClass(...)
```

where owner-level parameters may remain residual.

SC-2 may instantiate those **declaration-owned** `TypeParameterId`s as solver-local variables for that invocation. They remain distinct from callable-local generic variables.

For partially supplied owner forms:

```text
Result<Int>
```

fixed prefix:

```text
Result::T := Int
```

residual:

```text
Result::E := ?e0
```

Only residual parameters become fresh variables.

## 21. Domain support and SC-3 boundary

SC-2 supports ordinary type-form inference where a generic parameter's kind can be represented by canonical type-form IDs:

```text
Type
Type -> Type
(Type, Type) -> Type
... explicit monomorphic arrow kinds
```

A `RecordRow` parameter belongs to the SC-3 row domain. It must not be represented as an ordinary `InferenceTerm::Var` whose solution is a `TypeId`.

Before SC-3 integration, attempting to instantiate a row-kind parameter in ordinary generic call inference must return an explicit blocked/unsupported-domain result—not a fake proper type.

---

# Part VI — Constraint roles and solving

## 22. Four semantic constraint roles

SC-2 distinguishes the *relation* from the *role* that relation plays in choosing a substitution.

Recommended roles:

```text
ValueSelection
ContextSelection
ExactSemanticSelection
DeclarationRestriction
```

### 22.1 Value selection

Comes from actual value expressions bound to formal parameters.

It can:

- nominate candidates;
- propagate lower/upper/equality information;
- carry `Established` or `Assumed` value support;
- weaken return evidence when return-relevant variables depend on it.

### 22.2 Context selection

Comes from expected result context.

It can:

- nominate candidates;
- disambiguate an otherwise unsolved result variable;
- participate in consistency checking.

It does **not** provide runtime value support.

### 22.3 Exact semantic selection

Comes from compiler-owned exact information such as:

- explicit owner generic arguments;
- GADT case environment equalities;
- exact variant result relationships;
- future explicit generic arguments if syntax is separately ratified.

It may exactly fix variables but does not inherit value-expression epistemic status unless it actually depends on such a premise.

### 22.4 Declaration restriction

Comes from the canonical `where`/generic signature.

It constrains admissible solutions. It does not automatically nominate a candidate simply because the internal solver currently has one upper bound.

## 23. Recommended implementation representation

The existing `ConstraintOrigin` remains valuable. Add either an explicit role field or a deterministic origin-to-role mapping.

A recommended shape is:

```rust
pub enum InferenceConstraintRole {
    ValueSelection,
    ContextSelection,
    ExactSemanticSelection,
    DeclarationRestriction,
}

pub struct InferenceConstraint {
    pub relation: InferenceRelation,
    pub origin: ConstraintOrigin,
    pub role: InferenceConstraintRole,
    pub explanation: Option<ExplanationId>,
    pub support: Option<InferenceSupport>,
}
```

Do not infer role later from a partially collapsed bound table if provenance has already been lost.

## 24. Candidate selection versus admissibility

The solver may use declaration restrictions during propagation, but final candidate selection must obey:

```text
no selecting evidence
+ one-sided declaration upper bound
-----------------------------------
Underconstrained
```

Exact declaration equivalence can propagate a unique selection when anchored by fixed canonical information. For example:

```text
where T == Int
```

may determine `T` if such a constraint is valid in the language's canonical constraint grammar.

For a relation between unsolved parameters:

```text
where T == U
```

it aliases them but does not, by itself, select a canonical proper type for either.

---

# Part VII — Inference-term relation completion

## 25. Current gap

Current `InferenceSession::subtype_terms` handles variable/canonical cases but falls back to `unify_terms` for most compound terms.

That can accidentally convert a subtype constraint into equality-style structural matching.

Example:

```text
List<?T> <: List<Number>
```

must not generally become:

```text
?T == Number
```

The correct relation depends on `List`'s declared variance.

## 26. Applied constructor decomposition

For the same applied origin:

```text
C<A1...An> <: C<B1...Bn>
```

decompose according to each declaration parameter's variance:

```text
covariant:      Ai <: Bi
contravariant:  Bi <: Ai
invariant:      Ai == Bi
```

For different origins, if the left side has a generic supertype template, project it owner-relatively and continue against the target, matching canonical relation behavior.

## 27. Callable decomposition

For equivalent call shapes:

```text
(A1...An) -> R <: (B1...Bn) -> S
```

require:

```text
Bi <: Ai    for each parameter
R  <: S
```

Labels and rest modes are part of callable structural compatibility.

## 28. Tuple/product decomposition

Tuple arity and labels must agree according to canonical tuple semantics. Element subtype constraints decompose positionally.

## 29. Exact-case decomposition

Exact cases with the same variant identity compare their enum carriers. An exact case may participate in the same carrier-subtype relation already implemented canonically.

## 30. Union decomposition

Inference-term union behavior follows canonical relation laws:

```text
(A | B) <: T      iff A <: T and B <: T
T <: (A | B)      iff T is proven under an admissible union-arm rule
```

Do not add solver-only union heuristics that disagree with `types/relation.rs`.

## 31. Closed structural types

A canonical type may contain a generic variable under a closed record field or family member. Treating the whole record/family as `InferenceTerm::Canonical` hides that variable.

SC-2 must either:

1. add solver-local compound terms for the closed structural shapes required by the ordinary generic language; or
2. make inference lifting/viewing recursively expose generic variables through those canonical shapes without creating a parallel canonical representation.

Open row-tail variables remain SC-3.

## 32. Higher-kinded inference

SC-2 must support explicit monomorphic higher-kinded parameters:

```phalcom
<F: Type -> Type, T>
apply(_ value: F<T>) -> F<T>
```

Given `List<Int>`, a valid local solution is:

```text
F := List
T := Int
```

Likewise a type lambda may be a constructor candidate where kinds match:

```text
F := <X> =>> Result<X, Error>
```

No public kind-variable inference is implied.

## 33. Kind checking

Every variable binding and compound decomposition preserves kinds.

A variable declared:

```text
F : Type -> Type
```

cannot bind to:

```text
Int : Type
```

The failure remains a structured kind mismatch. The solver must never default a missing kind to `Type` internally.

---

# Part VIII — Expected-result inference and evidence

## 34. Two-phase solving is allowed; sticky underconstraint is not

Current generic call analysis first solves argument/declaration constraints, then adds expected-result context if the first outcome is solved or underconstrained.

That architecture may remain, but an interim underconstrained outcome is **not terminal** if contextual constraints are subsequently allowed to solve it.

The current pattern that retains `argument_underconstrained` and later diagnoses it even when contextual solving succeeds must be removed.

Final diagnosis is based on the final admissible solution state.

## 35. Context-only result publication

If a return-relevant variable was selected solely by expected context:

```phalcom
make<T>() -> T
const value: Int = make()
```

publish:

```text
type      = Int
status    = at most Assumed
origin    = GenericInference / ContextualDerivation as represented by the existing evidence model
```

The precise `EvidenceOrigin` spelling is an implementation choice; the critical law is that the expected annotation does not become `Established` value evidence.

## 36. Required-premise preservation

If a value argument is `Unknown` or `Dynamic`, expected result context cannot erase that missing premise merely because it can select a syntactic substitution.

The substitution and the proof of the return proposition are distinct.

Example:

```text
argument requires T but argument value is Unknown
expected result selects T = Int
```

The type substitution may be structurally determined, while the result's admissible evidence/status remains blocked/unknown according to the required premise.

---

# Part IX — Underconstraint, ambiguity, conflict

## 37. Underconstrained

Use `Underconstrained` when required variables have an open solution space and no unique finite candidate is established.

Example:

```phalcom
make<T>() -> T
const x = make()
```

## 38. Ambiguous

Add an explicit ambiguity outcome for a **finite, known set of multiple incomparable admissible substitutions**.

Recommended shape:

```rust
pub struct AmbiguousInference {
    pub variables: Box<[InferVarId]>,
    pub candidates: Box<[InferenceCandidate]>,
    pub constraint_indices: Box<[u32]>,
}

pub enum InferenceOutcome {
    Solved(InferenceSolution),
    Underconstrained(UnderconstrainedInference),
    Ambiguous(AmbiguousInference),
    Conflicting(InferenceConflict),
    Blocked(BlockReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(InferenceFailureReason),
}
```

Do not fabricate a finite candidate list merely to relabel ordinary underconstraint as ambiguity.

Dispatch ambiguity and generic-substitution ambiguity remain distinct semantic causes.

## 39. Conflict

Use `Conflicting` when no substitution satisfies all required relations/obligations. Preserve the actual failing constraints and their origins.

## 40. Structured materialization failure

`InferenceSession::materialize` currently maps failed type application into an empty `UnderconstrainedInference` in some cases. SC-2 replaces that information loss.

Recommended domain:

```rust
pub enum InferenceMaterializationFailure {
    Unsolved(UnderconstrainedInference),
    TypeApplication(TypeApplicationError),
    InvalidExactCase,
    UnsupportedDomain,
    InternalInvariant,
}
```

A kind/application error is not underconstraint.

---

# Part X — `where` constraints and F-bounds

## 41. Specialize declaration references before solving

For:

```phalcom
class Holder<T> {
    convert<U>(_ value: U) -> U
        where U <: T
}

const h: Holder<Animal> = ...
```

the local solver must see:

```text
U <: Animal
```

not an unspecialized `Holder::T`.

Receiver specialization applies to constraint operands before callable-local inference.

## 42. Formal participation without defaulting

Declaration restrictions can:

- alias/propagate variables;
- reject candidate assignments;
- narrow a candidate space;
- produce a structured generic-constraint diagnostic.

They cannot:

- invent a concrete candidate solely because one upper bound exists;
- upgrade evidence status;
- rescue an already contradictory value/context constraint set.

## 43. F-bound validation

Once a candidate exists, instantiate the F-bound and query the canonical bounded relation engine.

```text
T := User
where T <: Comparable<T>

validate:
User <: Comparable<User>
```

Relation terminal states propagate honestly:

```text
Refuted         -> generic constraint unsatisfied
Blocked         -> blocked call analysis
DynamicBoundary -> dynamic boundary
Cancelled       -> cancelled
BudgetExceeded  -> budget exceeded
InternalFailure -> internal incident
```

Only genuine refutation becomes an ordinary type rejection.

---

# Part XI — Constructors

## 44. Delete positional runtime-argument generic guessing

Current unqualified type-name call handling contains a compatibility heuristic that:

1. resolves a declaration by call name;
2. obtains declaration generic parameters;
3. analyzes runtime arguments without canonical constructor parameter expectations;
4. treats argument types positionally as declaration generic arguments;
5. calls `apply_type_form` directly.

That is not generic constructor inference.

SC-2 removes this rule.

## 45. Ordinary class constructor application

If source syntax `TypeName(args...)` denotes runtime construction, resolve the canonical constructor/allocator behavior and use the same application funnel.

Generic inference comes from formal constructor signature relations.

For:

```phalcom
class Box<T> {
    @constructor
    new(_ value: T)
}
```

conceptually:

```text
owner residual T -> ?t0
formal parameter -> ?t0
argument          -> Int
constraint        -> Int <: ?t0
constructor Self  -> Box<?t0>
solution          -> ?t0 := Int
result            -> Box<Int>
```

If syntax instead denotes type-form application, it must be represented by explicit type-form syntax/AST, not guessed from ordinary runtime argument types.

## 46. Class-side `Self`

Constructor return semantics use the exact receiver/owner specialization and the canonical `SelfRole::InstanceType` semantics. No `Unit`/`Object` sentinel is permitted for unresolved `Self`.

---

# Part XII — ADT/enum variant constructors and GADTs

## 47. Residual owner generic inference

A variant constructor belongs to an enum declaration whose generic signature owns its type parameters.

For:

```phalcom
enum Result<T, E> {
    @variant Ok(_ value: T) -> Result<T, E>
}
```

`Result::Ok(1)` starts with:

```text
T := ?t0
E := ?e0
argument: Int <: ?t0
result: Result<?t0, ?e0>
```

With no expected result, `E` remains result-relevant and unsolved. The call is underconstrained; it must not choose `Object`, `Dynamic`, or a guessed error type.

With:

```phalcom
const x: Result<Int, Error> = Result::Ok(1)
```

expected result may select:

```text
T := Int
E := Error
```

subject to all other constraints.

## 48. Nullary constructors

Nullary constructors are an essential expected-result case.

Conceptually:

```phalcom
const x: Option<Int> = Option::None()
```

must allow the result relation:

```text
Option<?T> <: Option<Int>
```

to select `T := Int` according to `Option` variance/invariance rules.

Without expected or explicit owner specialization, the constructor remains underconstrained if `T` is result-relevant.

## 49. No `Object` fallback

Current associated variant invocation can recover an unavailable constructor parameter canonical type with `Object`. SC-2 deletes that fallback.

Missing canonical constructor type information is a blocked/malformed semantic input, not a broad type.

## 50. GADT case constraints

`CaseTypeEnvironment` provides exact relationships between enum declaration parameters and a variant's result specialization.

At construction:

- fixed owner arguments are checked against case constraints;
- residual owner variables receive exact semantic/equivalence constraints;
- contradictions are structured GADT/generic conflicts;
- constraints feed the same inference/admissibility model rather than a disconnected manual solver where practical.

## 51. Result publication

Do not use an already-materialized variant `fallback_result_type` to hide terminal generic failure when that result depends on unsolved variables.

A fallback is permitted only when it is formally independent of the failed inference variables, matching the ordinary fixed-return law.

---

# Part XIII — First-class family and associated values

## 52. Monomorphic callable types are not generic declaration authority

`TypeData::Callable` is a first-class callable **value type**. It does not contain a `GenericSignature` and must remain monomorphic unless first-class polymorphism is separately ratified.

A generic declaration captured in a family/associated value must therefore not be permanently reduced to a monomorphic callable type and then inferred from that projection alone.

## 53. Behavioral family re-resolution

`AssociatedValueDenotation::BehavioralFamily` retains:

- the captured receiver type;
- the behavioral family spec;
- captured operation/target identities.

When invoking a selected generic behavioral member, re-resolve/recover its canonical dispatch target using retained receiver + operation/target identity, then call `apply_resolved_callable` with the recovered generic signature.

The cached `callable_type` remains useful for monomorphic family typing/presentation, but it is not the authority for lost generic binders.

## 54. Associated enum family recovery

Associated-family denotation retains owner form and variant member/constructor identity. Recover the canonical `VariantConstructorSignature` and owner generic signature for generic application rather than reconstructing generic semantics from a monomorphic callable projection.

## 55. No implicit `forall`

A generic declaration captured as a family candidate does not make arbitrary lexical bindings polymorphic.

Ordinary:

```phalcom
const f = someCallableValue
```

remains monomorphic unless its semantic denotation explicitly retains a declaration/family target that the language defines as re-instantiable on each family application.

---

# Part XIV — Union receiver calls

## 56. Required semantics

04.5 requires statically reachable union receiver arms to be checked exhaustively.

For:

```text
receiver : A | B
receiver.m(args...)
```

SC-2 must:

1. enumerate canonical union arms deterministically;
2. resolve the same runtime selector against each arm;
3. require a valid call on every reachable static arm unless the language explicitly marks a dynamic/open boundary;
4. solve each arm's receiver/generic specialization;
5. join successful result knowledge conservatively;
6. preserve failing-arm evidence in explanations/diagnostics;
7. record dependencies for every consumed arm/callable/hierarchy edge.

A single arm missing the selector is not silently ignored.

## 57. Analyze source arguments once

Union-arm checking must not duplicate source expression evaluation or expression identity.

A conservative v1 contextual rule is:

- bind call shape once;
- compute specialized parameter expectations for each receiver arm;
- if all arm expectations for an argument are semantically equivalent, analyze that argument once under the common expectation;
- otherwise analyze the argument once without a branch-specific expectation, then check its synthesized knowledge against every arm's formal parameter;
- if a closure requires incompatible branch-specific contextual parameter types and cannot be synthesized independently, report a structured ambiguity/blocked call rather than analyze the closure multiple times as if it executed multiple times.

This rule is sound and preserves one expression-analysis product per source expression.

## 58. Result join

If all arms succeed, join canonical result types using ordinary union/canonicalization rules.

Evidence status is the meet/weakest admissible evidence across arms. Dynamic/blocked/cancel/budget states propagate according to their semantic category rather than being coerced into a union type.

---

# Part XV — Contextual closures and higher-order calls

## 59. Bidirectional callable argument flow

For:

```phalcom
transform<T, U>(
    _ value: T,
    _ f: (T) -> U
) -> U
```

with:

```phalcom
transform(10, |x| x.name)
```

SC-2 must support:

```text
argument 0 selects T := Int
        ↓
formal argument 1 becomes (Int) -> ?U
        ↓
closure parameter x checked as Int
        ↓
closure result constrains ?U
```

The existing `ExpectedType::Inference` machinery is the integration point.

## 60. No repeated source analysis for solver progress

Constraint refinement must not repeatedly re-run arbitrary argument expressions as a solver worklist technique.

If a contextual closure needs information unavailable during the first pass, use a bounded explicit contextual-analysis protocol/product, not uncontrolled repeated evaluation of the AST with different guessed types.

---

# Part XVI — Dynamic shapes and open-world calls

## 61. Static call-shape law

Dynamic labels and expansion packs do not produce speculative generic equations because the formal parameter binding is not statically known.

Their expressions are still analyzed for their own semantic effects/diagnostics, but generic substitutions are not established from guessed slots.

## 62. Dynamic receiver boundary

A dynamic/open-world receiver does not become a generic inference failure. It remains an explicit dynamic application boundary according to current `TypeKnowledge`/`AnalysisStatus` policy.

---

# Part XVII — Solver boundedness and cancellation

## 63. Shared control

Generic inference consumes the caller's `CheckerControl` cancellation token and query budget.

No nested solver may silently reset budgets.

## 64. Iteration policy

Current inference additionally uses a local `max_passes = 16` and reports nonconvergence as `RecursiveFixpoint`.

SC-2 must replace or formalize this behavior:

- derive the convergence limit from shared query policy (for example the existing SCC/iteration budget dimension), or
- expose one named inference-iteration policy that is accounted for in the shared budget.

Nonconvergence unrelated to an actual recursive semantic dependency should not be mislabeled `RecursiveFixpoint`.

## 65. Terminal publication

After cancellation or budget exhaustion:

- no newly solved substitution is published as successful;
- no partially materialized generic signature escapes;
- no fallback broad type is inserted;
- independent fixed return knowledge may survive only under the existing fixed-return independence law.

---

# Part XVIII — Diagnostics and explanations

## 66. Required diagnostic categories

Retain existing:

```text
type.generic.inference_conflict
type.generic.underconstrained
type.generic.constraint_unsatisfied
```

Add at least:

```text
type.generic.inference_ambiguous
```

Kind mismatch/occurs/materialization failures may either receive dedicated stable diagnostic codes or remain structured causes under a generic conflict code, but their semantic identity must be preserved in the explanation graph.

## 67. Explanation requirements

A generic call explanation should be able to answer:

- which runtime callable/constructor identity was selected;
- which receiver arm and declaring owner were used;
- how receiver generic arguments projected through inheritance;
- how `Self` specialized;
- which generic parameters were fixed, residual, or callable-local;
- which arguments constrained which parameters;
- which expected-result constraint participated;
- which declaration/GADT restrictions were checked;
- why a candidate was selected;
- why a candidate was rejected;
- which evidence weakened the final result;
- which arm failed for a union receiver.

No diagnostic must reconstruct these facts from display strings after solving.

---

# Part XIX — Incremental semantics

## 68. Dependency requirements

Owner-relative receiver specialization consumes every relevant generic superclass template and declaration shell along the projection path.

Current `TrackingTypeHierarchy` already records `HierarchyEdge` dependencies for `superclass`, `is_subclass`, and `supertype_template` reads. The specialization utility must operate through the tracking hierarchy/context boundary so these dependencies remain visible.

For:

```text
Child<Int>
 -> Middle<Option<Int>>
 -> Parent<List<Option<Int>>>
```

changing only the `Middle -> Parent` generic template must invalidate a dependent call to a `Parent` method through `Child<Int>`.

## 69. Signature-body invalidation separation

Changing a selected callable's semantic signature must invalidate callers.

Changing only its body, while its published callable signature fingerprint remains identical, must not force unrelated caller retyping merely because the callee body changed.

## 70. Cold/incremental equivalence

After every SC-2 incremental scenario:

```text
incremental final semantic result == cold analysis final semantic result
```

for:

- selected call target identity;
- generic substitution/materialized result;
- diagnostics/status;
- explanation semantics modulo local IDs/order where allowed.

---

# Part XX — Source/native/generated conformance

## 71. One generic mathematics

Given semantically identical canonical signatures, source/native/generated callables must produce the same:

- shape binding;
- receiver specialization;
- generic substitution;
- where-constraint result;
- materialized return type;
- inference terminal category.

Implementation provenance may affect evidence origin/authority (`CallableSignature`, `NativeSignature`, `ConstructorSemantics`) but not substitution mathematics.

## 72. Variant constructors

Variant constructor provenance is constructor semantics, not a reason to create a second generic solver.

---

# Part XXI — Performance requirements

## 73. Runtime cost

SC-2 adds no per-object runtime generic metadata and no new runtime dispatch key.

## 74. Static hot-path requirements

Benchmark/measure:

1. repeated generic member lookup on saturated receiver;
2. multi-hop transformed generic inheritance;
3. generic call with 1/2/4/8 local variables;
4. higher-kinded `F<T>` inference;
5. expected-result-only inference;
6. contextual closure inference;
7. union receiver with 2/4/8 arms;
8. generic variant constructor with residual owner variables;
9. cold versus warm/incremental call analysis.

Track at least:

```text
InferenceSession variables
constraints
solver iterations/steps
canonical TypeStore nodes created
receiver environments/views created
relation pairs
call-resolution time
```

Key invariant:

> Temporary inference complexity must not directly inflate canonical `TypeStore` node count except for successfully materialized canonical results that are actually required.

---

# Part XXII — Required behavioral matrix

## 75. Ordinary generic calls

| Case | Required result |
|---|---|
| `id<T>(T)->T`, `id(1)` | `T=Int`, result `Int` |
| two independent variables | independently solved |
| nested `List<T>` parameter | infer nested `T` |
| repeated `T` positions | one consistent variable |
| contradictory arguments | conflict, not arbitrary first binding |
| assumed value argument | return-relevant result at most assumed |
| unknown required argument | proof/result remains unavailable |
| dynamic required argument | dynamic boundary retained |

## 76. Expected-result inference

| Case | Required result |
|---|---|
| argument solves `Int`, expected `Number` | retain precise `Int` if valid |
| argument solves `String`, expected `Int` | preserve precise independent knowledge + invalid status |
| `<T>() -> T`, expected `Int` | `T=Int`, known contextual/assumed result |
| `<T>() -> T`, no expected | underconstrained |
| expected context + unknown required value premise | substitution may be selected; proof not fabricated |

## 77. Constraints

| Case | Required result |
|---|---|
| candidate `Int`, `where T <: Number` | accepted |
| candidate `String`, same bound | constraint unsatisfied |
| only `where T <: Number`, no selector evidence | underconstrained, not `T=Number` |
| `where T == U` | aliases/propagates, no arbitrary concrete type |
| F-bound after candidate | canonical relation validation |
| F-bound without candidate | underconstrained |

## 78. Receiver specialization

| Case | Required result |
|---|---|
| `Box<Int>` direct member | owner parameter `Int` |
| `Child<Int> is Parent<T>` | inherited parameter specialized |
| `Child<Int> is Parent<List<T>>` | `Parent::T=List<Int>` |
| multi-hop transformed chain | fully projected arguments |
| nested `Self` return | actual receiver substituted |
| class generic + method generic | independent environments |
| method `where U <: T` | `T` specialized before solve |
| `super` | owner-relative projected parent environment |

## 79. HKT

| Case | Required result |
|---|---|
| infer `F=List`, `T=Int` from `F<T>` | solved |
| infer constructor lambda candidate | solved if kinds match |
| `F: Type->Type` vs `Int: Type` | structured kind mismatch |
| public kind variable required | out of scope / blocked by formation policy |

## 80. Constructors and variants

| Case | Required result |
|---|---|
| generic class constructor argument infers class parameter | canonical constructor inference |
| constructor arg count != generic count | governed by constructor shape, never declaration arity guessing |
| `Result::Ok(1)` with unsolved `E` | underconstrained |
| expected `Result<Int,Error>` + `Ok(1)` | solves both owner parameters |
| nullary `Option::None()` expected `Option<Int>` | expected-only owner inference |
| GADT owner conflict | structured conflict |
| missing constructor type fact | blocked/malformed, never `Object` fallback |

## 81. Union receivers

| Case | Required result |
|---|---|
| both arms support same method | both checked, result join |
| one arm missing method | static call invalid |
| per-arm generic solutions differ | joined result if each valid |
| common closure expectation | closure analyzed once contextually |
| incompatible closure expectations | structured ambiguity/blocked, no duplicated AST execution |

## 82. Terminal outcomes

| Case | Required result |
|---|---|
| ordinary open solution space | Underconstrained |
| finite incomparable candidates | Ambiguous |
| inconsistent relations | Conflicting |
| cancelled | Cancelled |
| budget exhausted | BudgetExceeded |
| malformed solver invariant | InternalFailure |
| fixed return independent of failed generic | fixed knowledge may survive; status remains terminal |

---

# Part XXIII — Acceptance gates

## 83. SC2-A — specialization correctness

Pass when:

- one canonical owner-relative specialization utility exists;
- transformed and multi-hop generic inheritance work;
- `Self` uses actual receiver semantics;
- method constraints are receiver-specialized;
- ordinary dispatch and associated lookup share projection logic.

## 84. SC2-B — solver relation correctness

Pass when:

- compound subtype constraints do not collapse to equality;
- applied variance matches canonical relations;
- callable variance matches canonical relations;
- higher-kinded inference works with kind checks;
- F-bounds do not become recursive equality;
- materialization failure classes remain explicit.

## 85. SC2-C — selection/evidence correctness

Pass when:

- expected-only inference succeeds when unique;
- contextual selection does not become established value evidence;
- one-sided declaration bounds do not default variables;
- argument-derived precision survives contradictory expected context when independently complete;
- unknown/dynamic required premises are not erased by context.

## 86. SC2-D — executable-surface closure

Pass when:

- ordinary generic methods use the canonical funnel;
- generic class constructors use the canonical funnel;
- generic variant constructors use the canonical inference model;
- generic family/associated targets recover canonical generic declaration semantics;
- no `Object` generic fallback remains;
- no positional runtime-argument-to-generic-parameter guessing remains.

## 87. SC2-E — union call closure

Pass when union receiver calls check all arms, analyze source arguments once, join results soundly, and preserve failing-arm evidence.

## 88. SC2-F — outcome/diagnostic closure

Pass when Underconstrained, Ambiguous, Conflicting, DynamicBoundary, Blocked, Cancelled, BudgetExceeded, and InternalFailure remain distinct from creation through presentation.

## 89. SC2-G — incremental/conformance closure

Pass when:

- generic supertype template edits invalidate exact dependent calls;
- unrelated bodies/signatures reuse;
- cold and incremental results match;
- source/native/generated/variant signatures produce the same generic solution mathematics.

---

# Part XXIV — Deletion / prohibition ledger

## 90. Must be removed by SC-2 completion

Production patterns equivalent to:

```text
runtime call argument #N -> declaration generic parameter #N
```

for ordinary class construction.

Production variant-constructor fallback equivalent to:

```text
missing parameter type -> Object
```

Dependent result recovery equivalent to:

```text
terminal generic failure -> use already-specialized fallback result anyway
```

Direct-only inherited dispatch specialization treated as complete semantics.

Generic family invocation whose only authority is a monomorphic `TypeData::Callable` when a retained declaration target is available.

Sticky pre-context `Underconstrained` diagnostics after a later expected-result solve succeeds.

## 91. May remain as compatibility helpers

- direct `TypeSubstitution` materialization utilities;
- structural List/Map application-target constructors, provided they only construct a target and always use `apply_resolved_callable` for checking/publication;
- projected `CallableSignature` on dispatch surfaces until SC-6 retires remaining duplication.

They may not own independent generic semantics.

---

# Part XXV — SC-3 and SC-7 extension seams

## 92. SC-3 row inference seam

SC-2 must leave the inference architecture capable of adding a separate row-domain solver/constraint term without pretending a row variable is a `TypeId` variable.

Ordinary call application should be able to coordinate:

```text
type inference session
record-row inference session/domain
```

through typed constraints when SC-3 lands.

## 93. SC-7 generic getter seam

SC-2 intentionally does not modify `GetterDef` or getter generic signature publication.

It does guarantee the hard semantic prerequisite:

```text
generic callable with zero value arguments
+ expected result
-> unique contextual generic solution
```

SC-7 can therefore be limited to:

```text
getter AST generic binders/where
canonical generic getter signature
property access forwards ExpectedType
getter access uses zero-argument apply_resolved_callable
```

without changing generic inference mathematics.

---

# Part XXVI — Final normative summary

1. SC-2 completes generic application; it does not replace the existing inference engine wholesale.
2. `apply_resolved_callable` remains the canonical executable application funnel.
3. Receiver/declaration specialization is owner-relative and precedes local inference.
4. Generic inheritance projection is one shared algorithm.
5. `Self` is specialized from actual receiver semantics, never a sentinel.
6. Residual declaration generics and callable-local generics are distinct binders even when solved in one invocation session.
7. Expected-result context may uniquely select generic arguments.
8. Expected context is selection information, not runtime value evidence.
9. One-sided declaration restrictions do not silently select their bound as a default.
10. F-bounds are relational admissibility obligations, not recursive equality equations.
11. Solver-local subtyping must respect canonical variance/callable/union/inheritance semantics.
12. Explicit arrow-kind HKT inference is in scope; public kind polymorphism is not.
13. Underconstrained and finite ambiguous outcomes are distinct.
14. Materialization failures preserve their actual category.
15. Ordinary generic class constructors use canonical callable semantics, not positional generic guessing.
16. Generic enum/ADT variant constructors infer residual owner parameters through the same generic model.
17. `Result::Ok(1)` with an unsolved result-relevant `E` is underconstrained unless context/other evidence determines it.
18. Nullary generic constructors may use expected-result inference.
19. No missing generic type becomes `Object`/`Dynamic`/first-bound by fallback.
20. Family/associated values recover retained generic declaration targets rather than treating monomorphic callable projections as generic authority.
21. Union receiver calls check every reachable arm and analyze source arguments once.
22. Cancellation/budgets never become semantic success.
23. Solver-local variables never escape snapshots/metadata/reflection.
24. Static specialization never changes runtime selector/class/object semantics.
25. SC-2 is complete only when ordinary methods, constructors, variant constructors, family calls, inherited calls, expected-only calls, HKT calls, and union-receiver calls obey the same formal laws.

---

# Appendix A — Repository evidence inspected

Pinned baseline: `01e19adb86186d67212b558ba76f54f79e2b5d9f`.

Primary current-source evidence:

```text
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/checker/declaration_signature.rs
phalcom-semantic/src/checker/expected.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/parameter.rs
phalcom-semantic/src/types/outcome.rs
phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/signature.rs
phalcom-semantic/src/enum_semantics.rs
phalcom-semantic/src/types/denotation.rs
phalcom-semantic/src/diagnostic.rs
```

Primary current tests inspected:

```text
phalcom-semantic/tests/semantic/foundations/bidirectional_calls.rs
phalcom-semantic/tests/semantic/foundations/generics_core.rs
phalcom-semantic/tests/semantic/capabilities/generics.rs
phalcom-semantic/tests/semantic/adts/constructors.rs
phalcom-semantic/tests/semantic/adts/generics.rs
phalcom-semantic/tests/semantic/incremental/mod.rs
```

Normative/planning basis reconciled:

```text
docs/impl/semantic/semantic-completeness/part-1/01.5-canonical-generic-type-semantics-and-declaration-model.md
docs/impl/semantic/semantic-completeness/part-4/04.5-expression-typing-generic-inference-flow-semantics-and-diagnostic-explanations.md
docs/impl/semantic/semantic-completeness/part-5/05-advanced-kinds-constraints-effects-and-proofs-REVISED.md
docs/impl/semantic/semantic-completeness/part-6/06-language-comparisons-and-design-rationale-REVISED.md
docs/impl/semantic/semantic-completeness/part-7/07-consolidated-implementation-plan-and-decision-register-REVISED.md
docs/spec/semantic-analyzer/07-generic-inference-engine.md
docs/work/deferred/generic-on-getter.md
docs/impl/adt-gadt-associated-lookup/part-3/03-associated-resolution-family-values-generic-specialization-invocation-typing-technical-spec.md
docs/impl/adt-gadt-associated-lookup/part-3/03-associated-resolution-family-values-generic-specialization-invocation-typing-implementation-plan.md
```

The attached revised specifications are used for semantic intent. Their older repository-state observations are superseded where current `main` materially differs.

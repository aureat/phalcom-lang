For Phalcom’s currently ratified typing direction, “full type-system closure” does not mean implementing every advanced type-system feature imaginable. It means that every type-forming surface, every executable expression, every refinement/proof path, and every public semantic consumer is governed by one coherent static model, with `Dynamic`/`Unknown` handled explicitly rather than by unsound fallback.

SC-2 is a very large part of that closure, but it is specifically the **generic executable-application layer**. It assumes canonical declarations already exist, and it deliberately leaves structural rows, proof/contract inference, generic getters, broader representation convergence, and final semantic certification elsewhere. The attached spec says exactly that: SC-2 depends on SC-1 and feeds SC-3, SC-6, SC-7, and SC-8.

I would organize the complete remaining system as follows.

## I. Features covered by SC-2

These are not merely “generic syntax.” Together they form the executable generic inference calculus.

### 1. One canonical generic-call application model

Every executable generic surface must converge on one application pipeline:

```text
canonical callable declaration
        ↓
runtime target/selector resolution
        ↓
receiver/owner specialization
        ↓
fresh residual generic variables
        ↓
argument binding
        ↓
bidirectional constraints
        ↓
declaration/GADT restrictions
        ↓
bounded inference
        ↓
canonical materialization
        ↓
evidence/status/explanation
```

Ordinary methods, constructors, enum constructors, inherited methods, families, native declarations, and generated declarations must not each acquire their own solver.

This is the core SC-2 architectural law.

### 2. Owner-relative generic receiver specialization

SC-2 closes the difference between:

```phalcom
Parent<Int>
```

and transformed inheritance such as:

```phalcom
class Child<T> is Parent<List<T>> {}
```

For:

```text
receiver = Child<Int>
selected declaration owner = Parent
```

the checker must derive:

```text
Parent::T = List<Int>
```

rather than blindly substituting `Child::T = Int`.

This includes:

- direct generic receivers;
- transformed inheritance;
- multi-hop transformed inheritance;
- inherited generic methods;
- generic arguments appearing in superclass templates;
- selected declaring-owner projection;
- dependency tracking for every hierarchy template read.

SC-2 explicitly makes owner-relative specialization precede callable-local inference.

### 3. Correct `Self` specialization

`Self` must mean the actual semantic receiver in the appropriate role.

Examples:

```phalcom
class Parent {
    wrap() -> Box<Self>
}

class Child is Parent {}
```

Calling through `Child` must produce:

```text
Box<Child>
```

even when the method is declared on `Parent`.

This must work inside:

- direct return positions;
- nested applied types;
- callable types;
- inherited signatures;
- constructor result types;
- owner-relative constraints.

### 4. Separation of declaration generics and callable generics

Given:

```phalcom
class Container<T> {
    transform<U>(_ value: U) -> Pair<T, U>
}
```

there are two scopes:

```text
Container::T
transform::U
```

SC-2 requires them to remain distinct even if they are solved during one invocation.

The same distinction matters for:

- partially specialized constructors;
- enum owner parameters;
- method-local generics;
- residual declaration generics;
- GADT result parameters.

### 5. Solver-local inference variables

The protected law is:

```text
InferVarId != TypeId
```

Unsolved inference variables are temporary existential/metavariable state. They are not canonical semantic types and may never leak into:

- `TypeStore`;
- snapshots;
- reflection;
- metadata;
- runtime type objects.

That separation is explicitly part of SC-2.

As discussed in the previous turn, SC-2 still needs strengthening around **nested ownership of these variables**, but that is an SC-2 issue rather than a new typing domain.

### 6. Full inference-term structural decomposition

Inference cannot understand only:

```text
T
```

It must understand variables nested under:

```text
List<T>
Result<T, E>
(A) -> B
(A, B)
A | B
ExactVariant<T>
closed record fields
family member types
higher-kinded applications
```

SC-2 requires solver-local relations to conform to the canonical relation algebra instead of treating compound subtype constraints as equality.

### 7. Declaration-site variance during inference

For:

```text
C<A> <: C<B>
```

SC-2 requires decomposition according to the declaration:

```text
covariant      A <: B
contravariant B <: A
invariant      A == B
```

This must agree with the ordinary canonical subtype checker.

It is essential for:

- collections;
- `Option`;
- `Result`;
- user-defined generic classes;
- expected-result inference;
- nullary variant inference;
- generic inheritance.

### 8. Callable variance

Functions require:

```text
(A) -> R <: (B) -> S

iff

B <: A
R <: S
```

with callable shape, labels, and rest modes also checked.

This is especially important for higher-order inference and contextual closures.

### 9. Generic-supertype inference

The solver must understand relations across different generic nominal origins through their declared inheritance templates.

For example:

```text
Child<Int> <: Parent<List<Int>>
```

must participate in generic inference rather than being treated as a structural mismatch simply because the immediate origins differ.

### 10. Higher-kinded local inference

SC-2 explicitly includes monomorphic higher kinds such as:

```phalcom
F: Type -> Type
```

and inference such as:

```text
formal: F<T>
actual: List<Int>

F = List
T = Int
```

It also covers canonical type-lambda constructor candidates where kinds agree.

It does **not** require kind polymorphism.

### 11. Kind-correct solving

Inference variables carry kinds.

So:

```text
F : Type -> Type
```

cannot solve to:

```text
Int : Type
```

Errors must remain structured kind mismatches rather than degrading into ordinary underconstraint.

### 12. Bidirectional generic inference

SC-2 makes expected types first-class constraint sources.

For:

```phalcom
make<T>() -> T

let x: Int = make()
```

Phalcom should solve:

```text
T = Int
```

even though there is no value argument.

This is one of SC-2's major policy decisions.

### 13. Context is selection, not evidence

This is crucial to Phalcom's semantic model.

Expected context may determine:

```text
T = Int
```

but cannot pretend a runtime value proved `T`.

Therefore:

```text
contextual-only solution
→ at most Assumed
```

not automatically `Established`.

SC-2 explicitly distinguishes:

```text
ValueSelection
ContextSelection
ExactSemanticSelection
DeclarationRestriction
```

as different semantic roles.

### 14. Context cannot overwrite stronger intrinsic evidence

For:

```phalcom
identity<T>(_ x: T) -> T

let x: Int = identity("hello")
```

the call intrinsically establishes:

```text
T = String
```

The `Int` context may make the expression invalid, but cannot rewrite the inferred call result to `Int`.

So Phalcom can retain:

```text
knowledge = String
status    = Invalid(...)
```

This preservation of proposition versus contextual compatibility is part of SC-2.

### 15. Required-premise proof weakening

If a generic solution depends upon:

- an unknown argument;
- an assumed argument;
- a dynamic boundary;

then contextual inference must not fabricate stronger result evidence.

The generic substitution and the epistemic strength of the resulting type proposition are separate dimensions.

### 16. Underconstraint

SC-2 provides a real semantic state for:

```phalcom
make<T>() -> T
let x = make()
```

There is no legitimate solution.

It must remain:

```text
Underconstrained
```

rather than:

```text
Object
Dynamic
first bound
first candidate
```

### 17. Finite ambiguity

SC-2 distinguishes:

```text
Underconstrained
```

from:

```text
Ambiguous
```

Ambiguity is specifically a finite known set of multiple incomparable solutions.

Those are semantically different situations and should yield different explanations/diagnostics.

### 18. Generic conflicts

Contradictory constraints must retain their actual provenance.

For instance:

```text
T = Int
T = String
```

or:

```text
T = String
where T <: Number
```

must produce structured conflicts rather than arbitrary first-choice inference.

### 19. `where` constraints as restrictions

SC-2 makes a critical distinction:

```phalcom
where T <: Number
```

does not mean:

```text
T defaults to Number
```

It means:

```text
if T is selected, T must satisfy T <: Number
```

Without selecting evidence:

```phalcom
make<T>() -> T where T <: Number
```

remains underconstrained.

### 20. Equality constraints without defaulting

A relation:

```text
T == U
```

can alias two generic variables.

But:

```text
T == U
```

with neither side otherwise anchored does not produce a concrete type.

### 21. F-bounds

SC-2 explicitly models:

```text
T <: Comparable<T>
```

as an admissibility relation.

Correct:

```text
select User
validate User <: Comparable<User>
```

Incorrect:

```text
solve T = Comparable<T>
→ occurs-check failure
```



### 22. Structured materialization

Solving and materializing are different.

A malformed type application, kind mismatch, invalid exact case, and unresolved variable cannot all collapse into:

```text
Underconstrained([])
```

SC-2 requires structured terminal failures.

### 23. Solver convergence and boundedness at the call level

SC-2 already requires:

- cancellation;
- shared budgets;
- bounded solving;
- no success after cancellation/budget exhaustion;
- no hidden magic iteration count masquerading as semantics.

This overlaps later broader convergence work but generic solver correctness itself belongs here.

### 24. Generic class constructors

Generic constructor inference must come from the constructor signature:

```phalcom
class Pair<A, B> {
    @constructor
    new(_ second: B, _ first: A)
}
```

not:

```text
runtime arg #0 → generic parameter #0
runtime arg #1 → generic parameter #1
```

SC-2 removes positional generic guessing.

### 25. Residual generic enum/ADT constructors

For:

```phalcom
enum Result<T, E> {
    @variant Ok(_ value: T)
}
```

this:

```phalcom
Result::Ok(1)
```

solves:

```text
T = Int
```

but leaves:

```text
E = ?
```

and therefore remains underconstrained unless something else selects `E`.

### 26. Nullary generic constructors

Result context can solve generic variables even when no constructor payload exists:

```phalcom
let x: Option<Int> = Option::None()
```

must infer `T = Int`.

### 27. GADT construction constraints

Variant case environments can contribute exact equations/refinements to constructor inference.

Those GADT constraints must enter the same generic solver rather than a parallel manual subsystem.

### 28. No constructor `Object` fallback

Missing constructor typing information must remain malformed/blocked.

Task 9 explicitly removes:

```text
unknown parameter → Object
```

which is essential for soundness.

### 29. Generic family/associated invocation

If a family captures a generic declaration, calling the family member must recover the canonical generic declaration rather than relying only on a monomorphic projected callable type.

SC-2 deliberately does not introduce arbitrary first-class polymorphism; it allows reinstantiation only where semantic denotation retains an actual declaration target.

### 30. Monomorphic ordinary callable values

Conversely:

```phalcom
let f = someCallable
```

does not silently create a hidden `forall`.

This is an important boundary around first-class polymorphism.

### 31. Union-receiver calls

For:

```text
receiver : A | B
receiver.method(...)
```

SC-2 requires every reachable static arm to be checked.

It must support:

- same result;
- different result → union;
- per-arm generic specialization;
- missing method on one arm → static invalidity;
- conservative evidence joining;
- dependencies for all arms.

### 32. Analyze union-call arguments once

Union call checking may not execute semantic analysis once per arm.

This is both a correctness and identity law:

```text
one source expression
→ one ExpressionAnalysis
```

Then arm-specific compatibility is checked against that result.

### 33. Contextual closures

The canonical SC-2 higher-order case is:

```text
argument 1 selects T
        ↓
closure expected (T) -> U
        ↓
closure parameter receives T
        ↓
closure body determines U
```

The Either package now exercises this.

### 34. Partial symbolic specialization

This is the mechanism exposed by the Either problem:

```text
(A) -> B
A = Int
B unresolved

→

(Int) -> B
```

Useful solved structure must remain available even when the entire term cannot yet become a canonical `TypeId`.

This is firmly within SC-2's inference-term/contextual-closure responsibilities, although its nested form should be specified more explicitly.

### 35. Nested generic calls

Task 12 explicitly requires a generic call inside a contextually typed closure to receive an inference-shaped expected type without solver sessions leaking into one another.

For full closure, SC-2 should strengthen this into explicit:

- inference-frame ownership;
- cross-frame metavariable identity;
- ancestor-variable references;
- child-variable non-escape;
- cross-frame constraint propagation.

Those are **missing architectural details inside SC-2**, not a separate type-system feature.

### 36. Dynamic/open call boundaries

Dynamic calls remain explicitly dynamic.

SC-2 does not turn:

```text
cannot prove statically
```

into:

```text
generic inference failed
```

`DynamicBoundary` remains separate.

### 37. Source/native/generated generic conformance

Given equivalent canonical signatures, the inference mathematics must be identical whether the declaration came from:

- Phalcom source;
- native surface;
- generated declaration;
- enum constructor.

Evidence origin can differ, substitution semantics cannot.

### 38. Generic call explanations

The explanation graph must retain:

- selected callable;
- actual receiver;
- declaring owner;
- specialization path;
- fixed/residual generic parameters;
- argument constraints;
- expected-result constraints;
- declaration restrictions;
- selected solution;
- failed relation;
- union arm.

This is part of SC-2 closure, not merely diagnostic polish.

### 39. Generic-call incremental dependencies

SC-2 includes exact dependency capture for generic superclass templates, signatures, and selected targets.

It also demands cold/incremental equivalence for its own generic call semantics.

---

# II. Necessary for full type-system closure, but not owned by SC-2

This is the rest of the type system.

Some of these features already exist substantially in Phalcom; “not covered by SC-2” means they need their own closure/certification, not necessarily that they are absent.

## A. Canonical type formation — SC-1 domain

SC-2 assumes canonical declarations already exist. Its implementation plan explicitly refuses to compensate for missing SC-1 formation semantics.

Full closure therefore requires the following independently.

### 40. Canonical type expression resolution

Every source-level type expression must resolve into one canonical semantic type/form, including:

```text
Int
List<Int>
Map<String, Int>
Option<T>
(A, B)
(A) -> B
A | B
Self
exact variant types
type aliases
type lambdas
record types
```

No parser spelling should survive as a second semantic authority.

### 41. Type-level versus value-level name resolution

Phalcom must distinguish:

```text
type expression resolution
value expression resolution
runtime class/type object values
associated lookup
```

without using ad hoc spelling tests.

### 42. Generic declaration formation

Classes, enums, aliases, methods, constructors, and other legal generic owners need canonical:

```text
GenericSignature
TypeParameterId
kind
variance
where constraints
owner identity
```

### 43. Kind formation

The canonical kind universe must include at least:

```text
Type
RecordRow
Arrow(...)
```

with kind checking for all type applications.

### 44. Type-lambda formation

Type lambdas need:

- binder scope;
- capture safety;
- canonical body representation;
- result kind;
- substitution;
- beta/application semantics.

SC-2 consumes type-lambda candidates but does not own their declaration correctness.

### 45. Variance declaration validation

If a type parameter is declared covariant or contravariant, its use in the declaration must satisfy the language's variance placement rules.

SC-2 consumes validated variance.

### 46. `Self` formation

Before SC-2 specializes `Self`, declarations must publish a correct owner-relative semantic `Self` form, including class/instance role.

### 47. Transparent aliases

Aliases need coherent:

- identity;
- target form;
- expansion;
- cycle detection;
- kind checking;
- canonical equality/presentation policy.

### 48. Malformed type formation outcomes

Invalid type formation must remain:

```text
Invalid
Blocked
Unknown
...
```

as appropriate and must never become a normal `Object`/`Dynamic` type just to keep analysis going.

These are SC-1 prerequisites explicitly identified by the SC-2 plan.

---

# III. Canonical type-relation closure outside generic application

SC-2 requires its local solver to conform to the canonical relation algebra, but it does not define that entire algebra.

Full closure requires the global relation system itself to be complete.

### 49. Nominal subtyping

Including:

- superclass chains;
- mixins/interfaces/protocols according to Phalcom semantics;
- generic superclass templates;
- actual applied arguments.

### 50. Generic variance at canonical type level

Not merely inference decomposition, but ordinary:

```text
List<Int> <: List<Number>
```

according to the canonical declaration variance.

### 51. Union algebra

A canonical system for:

- union construction;
- flattening;
- duplicate elimination;
- subtype redundancy;
- member ordering;
- `Never` interactions;
- relation checks.

### 52. Callable structural relations

Ordinary callable type comparison needs:

- parameter contravariance;
- result covariance;
- labels;
- rest modes;
- arity/shape.

### 53. Tuple relations

Including:

- labels;
- arity;
- element relations;
- canonical equality.

### 54. Exact enum-case relations

The system needs a coherent relationship between:

```text
Option<Int>
Option::Some<Int>
Option::None<Int>
```

or the exact Phalcom variant type representation.

### 55. Record structural relations

Closed record structural relations exist as a canonical relation concern; open-row completeness belongs to SC-3 below.

### 56. Family/callable structural types

First-class family and callable structural relation rules need canonical agreement with application semantics.

### 57. Dynamic consistency versus subtyping

Because Phalcom is gradual, it must not conflate:

```text
subtyping
type equality
type equivalence
dynamic consistency
unknownness
```

This is essential for the `Dynamic` escape hatch.

### 58. Type equivalence/isomorphism operators if ratified

If Phalcom exposes value-side type relations such as:

```text
<:
>:
equivalence
isomorphism
```

their semantics must delegate to canonical type relations rather than creating alternate reflection/runtime rules.

---

# IV. Gradual typing and epistemic closure

SC-2 touches this heavily, but whole-language closure is broader.

### 59. `Dynamic`

`Dynamic` is a deliberate escape hatch, not analyzer failure.

The semantics must define:

- what operations statically pass;
- where runtime checks occur;
- how evidence degrades;
- whether refinement can recover precision;
- how generic application crosses the boundary.

### 60. `Unknown`

`Unknown` means the analyzer cannot currently establish a type because of:

- missing information;
- unsupported analysis;
- blocked dependency;
- budget;
- explicit unknown propagation.

It must not silently behave as `Dynamic`.

### 61. Evidence status

Phalcom's static facts require a consistent epistemic lattice such as:

```text
Established
Assumed
Unknown
Dynamic
```

with causal invalidity/status kept distinct.

SC-2 handles generic result propagation, but all expressions and bindings need the same rules.

### 62. No fabricated facts

Across every language construct:

```text
analysis failure
≠
Object
≠
Dynamic
≠
success
```

This must be an invariant of the whole checker.

---

# V. Flow-sensitive expression typing and refinement

SC-2 assumes the 04.5 expression/flow machinery.

Full type-system closure requires it to be complete across the language.

### 63. Bidirectional expression typing generally

Expected types should participate in:

- literals;
- collection literals;
- closures;
- return expressions;
- assignments;
- constructor calls;
- nested expressions;
- properties/getters.

Not just generic calls.

### 64. Binding state

Every binding needs a distinction between:

```text
declared constraint
current inferred knowledge
flow-refined knowledge
```

### 65. Assignment compatibility

Assignments must preserve the declared contract while updating flow knowledge.

### 66. Branch refinement

Conditions should refine facts in their respective branches.

Examples include:

```phalcom
if x is SomeType { ... }
if x != nil { ... }
if case/pattern succeeds { ... }
```

### 67. Refinement invalidation

Mutations and opaque calls must invalidate facts they could make false.

### 68. Join semantics

After:

```text
if/else
match
loops
exceptional control flow
```

flow states need principled joins rather than last-branch wins or arbitrary widening.

### 69. Loop fixed points

Loops require bounded fixed-point analysis with:

- stable header states;
- backedges;
- break/continue;
- termination of analysis;
- conservative widening if required.

### 70. `Never`

`Never` must mean no normal return from an expression path, not generic failure and not automatically divergence.

### 71. Return-flow typing

All explicit and implicit returns must reconcile with declared/inferred callable results.

### 72. Closure capture typing

Closures need stable analysis of:

- captured variables;
- mutable captures;
- contextual parameters;
- returns;
- escaping semantics.

---

# VI. ADT/GADT elimination and refinement

SC-2 covers **construction-time generic inference** for ADTs/GADTs. It does not own the complete elimination side.

For a complete type system, Phalcom also needs:

### 73. Exact variant identity

Every case must have canonical nominal/exact identity.

### 74. Pattern typing

Patterns must be checked against the scrutinee type.

### 75. Payload binding

A case pattern must bind payload fields at their specialized types.

### 76. Exhaustiveness

For closed ADTs:

```text
covered pattern space
==
scrutinee value space
```

must be proven rather than guessed.

### 77. Usefulness/unreachable arms

The checker should identify arms that cannot match because prior patterns already cover their space.

### 78. Missing-case witnesses

Non-exhaustive matches should produce meaningful missing-case witnesses.

### 79. Guards

Guards need semantics for:

- usefulness;
- exhaustiveness;
- branch refinement;
- proof conservatism.

### 80. GADT elimination refinement

This is the other half of GADTs.

Matching a constructor can establish local equalities such as:

```text
T = Int
```

inside one branch.

These refinements must enter the ordinary type/proof environment and disappear outside the branch.

### 81. GADT impossible-case reasoning

Contradictory case environments should allow statically impossible branches to be recognized.

### 82. Pattern-space algebra

Tuple, record, literal, union, enum, and exact-case pattern spaces need a coherent mathematical algebra.

---

# VII. Open structural records and row polymorphism — SC-3

This is the largest clearly deferred **typing domain**.

The SC-2 technical spec explicitly leaves open row inference to SC-3 and requires row variables to remain a domain distinct from ordinary `InferVarId`.

The current SC-3 plan describes the goal as completing immutable structural records, open row syntax, row-polymorphic callables, correlated remainders, structural subtyping, query-local row solver state, and canonical publication.

Full closure therefore requires:

### 83. One canonical record representation

Closed and open records should share one canonical semantic representation.

Not:

```text
ClosedRecordType
OpenRecordType
```

as unrelated equality domains.

### 84. Open record syntax

The type system must preserve the distinction between:

```text
{ x: Int, y: String }
```

and something conceptually like:

```text
{ x: Int | R }
```

using Phalcom's eventual syntax.

### 85. Row kinds

Row binders are:

```text
RecordRow
```

not ordinary:

```text
Type
```

### 86. Row variables

Use a distinct domain such as:

```text
RecordRowVarId
```

rather than pretending an unsolved row is a normal type metavariable.

### 87. Row unification

Inference must solve relationships such as:

```text
{ name: String | R }
```

against:

```text
{ name: String, age: Int }
```

giving:

```text
R = { age: Int }
```

### 88. Correlated row remainder

For:

```text
input  { known fields | R }
return { other fields | R }
```

the same remainder must flow through the callable.

### 89. Row lacks constraints

If necessary to avoid duplicate-field ambiguities:

```text
R lacks name
```

or equivalent representation.

### 90. Open-record structural subtyping

Width/depth behavior needs exact rules for:

- required fields;
- extra fields;
- open tails;
- closed tails;
- field type variance.

### 91. Simultaneous row + ordinary generic inference

This is the important SC-2/SC-3 bridge:

```text
T
R
```

may need solving during the same generic application.

They should cooperate without sharing identity domains. The SC-2 specification explicitly leaves this seam open.

### 92. Row materialization and non-escape

Like ordinary metavariables:

```text
RecordRowVarId
```

must never leak into durable canonical snapshots.

---

# VIII. Generic getters — SC-7

SC-2 intentionally defers the source surface.

Full closure requires:

### 93. Getter generic binders

Conceptually:

```phalcom
value<T> -> T
```

or whatever final syntax is ratified.

### 94. Getter `where` clauses

Generic getter declarations must publish the same canonical generic constraint products as ordinary methods.

### 95. Expected type propagation through property access

This is essential because a generic getter has no value arguments:

```phalcom
let x: Int = obj.value
```

may need expected result context to infer its generic parameter.

### 96. Getter access through the canonical application engine

Property access should eventually reduce to:

```text
resolve getter
apply zero-argument generic callable
```

rather than implementing a separate generic getter solver.

SC-2 explicitly provides this prerequisite.

### 97. Generic setters/indexers only if ratified

SC-2 explicitly excludes them.

They are **not necessary for closure of the currently ratified system** unless you decide those declaration forms should support generics.

---

# IX. Contracts and proof inference — non-effect portion of Spec/SC-5

The attached SC-2 explicitly excludes proof/contract inference.

This domain is necessary if “full type-system closure” includes the already-designed semantic contracts/proof layer.

Effects can be excluded exactly as requested.

### 98. Canonical contract declarations

Source constructs such as:

```text
requires
ensures
invariant
```

need canonical semantic identities and expressions.

They must not remain raw AST fragments interpreted independently by multiple consumers.

### 99. Contract admissibility

A runtime-valid assertion is not automatically statically provable.

The semantic system needs to classify a contract as:

```text
Admissible
Unsupported
Blocked
Invalid
```

rather than boolean “verified/not verified.”

The advanced-semantics plan explicitly separates canonical contract identity from proof admissibility.

### 100. Proof procedure IR

Static verification should consume the already typed/refined flow graph and lower it into a deterministic proof procedure.

It must **not** re-type the AST.

### 101. Verification-condition generation

Contracts and refinements need deterministic VCs representing propositions such as:

```text
preconditions
path conditions
postconditions
invariants
type refinements
```

### 102. Logical normalization

Equivalent semantic conditions should generate stable normalized proof terms/fingerprints.

### 103. Baseline deterministic reasoner

The plan explicitly calls for a backend-independent proof core before optional SMT integration.

### 104. Proof result algebra

Proof states need to distinguish at least:

```text
Proven
Disproven
Unknown
Blocked
Cancelled
BudgetExceeded
InternalFailure
```

Failure to prove is not proof of falsehood, and definitely not success.

### 105. Proof evidence and trust

An external solver's answer must not automatically become trusted compiler truth.

Proof evidence requires an explicit trust policy.

### 106. Proof dependency tracking

Proof results depend on:

- callable semantics;
- contracts;
- called declarations;
- type relations;
- branch predicates;
- invariants.

They need exact invalidation.

### 107. GADT/refinement proof integration

The proof system must consume branch refinements established by:

- patterns;
- type tests;
- exact cases;
- GADT equalities.

This is where the type system and formal proof layer must actually meet.

### 108. Runtime contract versus static proof separation

A runtime contract check executing successfully is not static proof evidence.

That distinction is explicitly part of the advanced-semantics architecture.

---

# X. Termination/totality if considered part of typing/proof closure

This is not an effect system, so I would include it under your “excluding effects” request if `@total` remains part of the intended language.

### 109. Termination knowledge

A separate product:

```text
Proven terminates
May diverge
Unknown
...
```

### 110. `@total`

A source-level requirement that the callable is proven terminating under the selected semantic model.

The current advanced plan explicitly says:

```text
@total ≠ pure
```

and treats termination as independent from effects.

### 111. Loop termination reasoning

Potentially including:

- obvious finite loops;
- ranking arguments;
- recursive SCC reasoning;
- conservative unknown when proof is unavailable.

### 112. Do not infer totality from `Never`

```text
Never
```

describes normal return behavior; it does not prove termination.

---

# XI. Canonical semantic convergence — SC-6 class of work

SC-2 deliberately does not finish all representation convergence. It explicitly says projected callable signatures may remain until SC-6 retires remaining duplication.

For full closure this matters enormously.

### 113. One canonical declaration/signature authority

There should not be competing semantic representations in:

```text
declaration tables
dispatch signatures
LSP projections
native surfaces
runtime reflection metadata
```

Each consumer should project from compiler-owned semantic facts.

### 114. Specialization as a view

Instead of repeatedly cloning and rewriting complete type trees, canonical declarations should remain stable while specialization environments/views represent:

```text
receiver substitutions
Self substitutions
generic substitutions
```

SC-2 moves in this direction; full convergence belongs later.

### 115. Remove stale compatibility semantic copies

Any projected structures that can independently drift from canonical facts must become:

- views;
- cached projections with canonical authority;
- or eliminated.

### 116. Query ownership

Every major semantic product needs a canonical query key and dependency graph ownership.

### 117. Stable fingerprints

Canonical products need fingerprints so a change to:

```text
body only
```

does not invalidate:

```text
unchanged signature users
```

while a signature/constraint change does.

### 118. Cold/incremental equivalence globally

SC-2 requires it for generic calls; full closure requires it for all semantic facts.

### 119. Budget/cancellation consistency

All expensive semantic analyses need shared policies rather than individual magic constants.

### 120. Deterministic publication

Canonical output should not depend on:

- hash iteration;
- allocation order;
- incidental traversal order.

---

# XII. Public semantic projection, metadata, reflection, and tooling

A type system is not closed if the compiler internally knows the answer but every external consumer reconstructs a different one.

### 121. Snapshot completeness

Published snapshots should expose compiler-owned facts for:

- canonical types;
- callable signatures;
- generic signatures;
- selected calls;
- exact variants;
- refinements where appropriate;
- row types;
- proof/contract status;
- evidence/status.

No solver-local variables.

### 122. Metadata projection

Compiled metadata should preserve the semantic facts needed by downstream compilation/tooling without creating another type system.

### 123. Reflection projection

For reified types, reflection should correctly expose:

- applied arguments;
- exact variant specialization;
- generic declaration information;
- kind information where public;
- callable signatures where permitted.

### 124. LSP semantic reuse

Hover, completion, signature help, diagnostics, and go-to-definition must consume canonical analysis rather than implementing miniature type checkers.

### 125. Cross-module type identity

Imported types and generic declarations must preserve stable identity through module resolution.

This includes the import/member resolution work you've already identified elsewhere in the project.

### 126. Native/source semantic parity

Core/native declarations must expose the same semantic shape as equivalent source declarations.

---

# XIII. Final semantic-completeness certification — SC-8 class

SC-2 explicitly feeds SC-8 semantic-completeness certification.

This is not a new type feature. It is the proof that no holes remain.

### 127. Expression-kind coverage

Every AST expression variant must have an explicit semantic typing owner.

No silent:

```text
Unknown(UncheckedExpression)
```

for otherwise supported language constructs.

### 128. Declaration-kind coverage

Every declaration surface must publish the required semantic products.

### 129. Relation coverage

Every canonical type form must have defined behavior under:

```text
equality
subtyping
consistency
substitution
kind
presentation
serialization/projection
```

where applicable.

### 130. Control-flow coverage

Every control form must correctly:

- propagate type knowledge;
- refine;
- join;
- invalidate;
- publish exits.

### 131. Generic-surface coverage

SC-2's completion matrix must include:

```text
methods
constructors
variants
families
inherited calls
HKT
expected-only calls
nested generic calls
union receiver calls
```

### 132. Hostile negative testing

For every success law there should be:

- malformed formation;
- underconstraint;
- ambiguity;
- contradiction;
- kind mismatch;
- inaccessible member;
- dynamic boundary;
- budget/cancellation;
- invalid metadata if reachable.

### 133. Proof-path testing, not only final type testing

Tests must assert not only:

```text
result = Int
```

but where important:

```text
why Int?
which substitution?
which constraint?
what evidence status?
which owner specialization?
what refinement?
```

### 134. Full source/native/generated parity

Equivalent declaration semantics should yield equivalent typing mathematics regardless of provenance.

### 135. Incremental parity

Every important semantic class needs:

```text
cold result == incremental result
```

### 136. Deletion ledger

Closure means proving obsolete semantic authorities are gone.

For example:

```text
no TypeData::Infer ordinary inference
no LocalConstraintSolver ordinary inference
no Object generic fallbacks
no positional generic guessing
no duplicated hierarchy-specialization algorithm
no LSP-local type checker
```

---

# XIV. Important things explicitly not required for this closure

SC-2 lists several advanced ideas as non-goals. They should not be accidentally treated as blockers for the current Phalcom type system.

You do **not** need, unless separately ratified:

### 137. First-class `forall`

Generic declarations can remain instantiated at declaration-aware call sites rather than making every function value polymorphic.

### 138. Rank-N / higher-rank inference

No requirement for:

```text
((forall T. T -> T) -> Int)
```

style inference.

### 139. Impredicative polymorphism

Not necessary.

### 140. Public kind variables / kind polymorphism

SC-2 supports explicit monomorphic arrow kinds; that is sufficient for the current HKT model.

### 141. Dependent types

GADTs and flow refinement do not require a general dependent type system.

### 142. Generic defaults

Explicitly not part of SC-2 and not necessary for closure.

### 143. Intersection types

Not needed unless separately ratified.

### 144. Implicit/given/type-class search

Typeclasses/interfaces may eventually exist, but hidden implicit search is not required for the present closure.

### 145. Type-directed runtime overload selection

Static typing must not silently mutate runtime selector identity.

### 146. Runtime monomorphization

An optimization strategy, not a semantic prerequisite.

### 147. Specialized runtime class identity

`List<Int>` does not need a physically distinct runtime class object merely for static type-system closure.

---

# XV. The whole closure as a dependency stack

I would now model Phalcom's type-system completion like this:

```text
┌──────────────────────────────────────────────┐
│ SC-8 / Certification                        │
│ prove every semantic surface is closed      │
├──────────────────────────────────────────────┤
│ Public projection / tooling / reflection    │
│ snapshots, metadata, LSP, reification       │
├──────────────────────────────────────────────┤
│ SC-6 / Semantic convergence + incrementality│
│ one authority, stable queries/fingerprints  │
├───────────────────────┬──────────────────────┤
│ Contracts + proofs    │ Generic getters     │
│ (+ totality)          │ SC-7                │
├───────────────────────┼──────────────────────┤
│ SC-3 Rows             │ ADT/GADT elimination│
│ structural records    │ match/refinement    │
├───────────────────────┴──────────────────────┤
│ SC-2 Generic executable application         │
│ inference, specialization, HKT, constructors│
├──────────────────────────────────────────────┤
│ 04.5 Expression / flow / evidence semantics │
│ refinement, joins, binding knowledge        │
├──────────────────────────────────────────────┤
│ Canonical relation algebra                  │
│ subtype, variance, union, callable, cases   │
├──────────────────────────────────────────────┤
│ SC-1 Canonical type formation               │
│ kinds, generics, lambdas, Self, aliases     │
└──────────────────────────────────────────────┘
```

The key point is that **SC-2 occupies the central application/inference layer, not the entire type system**.

## XVI. What remains after SC-2 specifically

Once SC-2 itself is completely correct—including the nested inference-frame issue we identified—I would regard these as the major remaining typing closures, excluding effects:

1. **SC-3:** open rows, row polymorphism, row inference, and full structural-record typing.
2. **ADT/GADT elimination certification:** match typing, exhaustiveness, refinements, GADT equality propagation, impossible cases.
3. **Whole-language flow/refinement completion:** every expression/control construct participates correctly in bidirectional typing and flow facts.
4. **Contract/proof layer:** canonical contracts, admissibility, VC generation, deterministic proof results, GADT/refinement integration.
5. **Termination/`@total`**, assuming this remains part of the intended static language.
6. **SC-7:** generic getter declaration + expected-type property access.
7. **SC-6-style convergence:** eliminate duplicate semantic representations and finish stable incremental query ownership.
8. **Snapshot/metadata/reflection/LSP projection:** make all consumers use the one semantic truth.
9. **SC-8 certification:** exhaustive positive/negative, proof-path, cold/incremental, and source/native parity testing.

And inside SC-2 itself, I would still add the explicit nested-inference laws we identified:

```text
scoped inference-variable ownership
ancestor/child frame references
cross-frame constraints
no child-variable escape
result-context → closure-input feedback
partial structural zonking through deferred terms
```

With those pieces completed, Phalcom would have what I would consider a genuinely closed static type system for its currently ratified language design—without requiring effects, rank-N polymorphism, dependent types, implicit typeclass search, or runtime monomorphization.
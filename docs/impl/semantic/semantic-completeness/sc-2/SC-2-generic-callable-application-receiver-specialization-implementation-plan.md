# SC-2 — Generic Callable Application, Receiver Specialization, Constraint Solving, and Executable Typing Closure
## Repository-Grounded Implementation Plan

**Project:** Phalcom  
**Date:** 2026-09-01  
**Repository:** `aureat/phalcom-lang`  
**Pinned planning baseline:** `main@3a7300082368214c04552f425edf4649ba1597b5`
**Companion specification:** `SC-2-generic-callable-application-receiver-specialization-technical-spec.md`  
**Execution prerequisite:** SC-1 acceptance gate complete on the implementation branch; re-pin SHA before coding if `main` has moved  
**Method:** tests-first, one semantic owner, no parallel compatibility solver

---

# 0. Goal

Complete Phalcom's ordinary generic call semantics by closing the gaps around the already-implemented `InferenceSession` and `apply_resolved_callable` architecture.

The final implementation must support one coherent algorithm for:

```text
ordinary methods
inherited generic methods
methods combining class + method generics
expected-result-only inference
higher-kinded generic calls
source constructors
enum/ADT variant constructors
GADT constructor constraints
first-class family/associated invocation
union receiver calls
source/native/generated signatures
```

without changing runtime selector identity or introducing first-class polymorphism.

---

# 1. Global implementation constraints

1. Keep `phalcom-semantic` as the sole static semantic authority.
2. Keep `apply_resolved_callable(...)` as the canonical executable application funnel.
3. Do not add a second generic solver for constructors, variants, native callables, or families.
4. Do not reintroduce `TypeData::Infer`, `TypeStore::infer`, or `LocalConstraintSolver` for ordinary type inference.
5. `InferVarId` remains query/session-local.
6. `RecordRowVarId` remains a distinct SC-3 domain; do not encode row variables as ordinary type variables.
7. Receiver/declaration specialization occurs before callable-local generic inference.
8. Generic specialization never changes `Selector` or `CallableId` identity.
9. Preserve current explanation/cause/status architecture; extend it instead of creating flat ad hoc diagnostics.
10. Preserve current semantic dependency capture; specialization must record all generic-supertype reads.
11. Do not use `Object`, `Dynamic`, `Never`, `Unit`, or first-bound sentinels for missing generic solutions.
12. Expected context may select a solution, but is not value evidence.
13. A one-sided `where` bound is not a type-argument default.
14. No source expression is analyzed multiple times merely because a union receiver has several arms.
15. No implementation task is complete until focused RED tests are GREEN and its deletion/audit search passes.

---

# 2. Repository state at the pinned baseline

## 2.1 Already implemented — do not rebuild

Current source already contains:

```text
checker/call.rs
    CallableApplicationTarget
    CallPremise
    ApplicationArgument
    ArgumentBindingPlan
    apply_resolved_callable

checker/inference.rs
    InferenceSession
    InferenceTerm
    InferVarId-local state
    kind-aware variables
    lower/upper/equality handling
    required-premise proof state
    shared cancellation/step budget

checker/expected.rs
    ExpectedType::None
    ExpectedType::Proper
    ExpectedType::Inference

types/relation.rs
    bounded canonical relation outcomes
    declaration-site variance
    generic superclass templates
    callable/tuple/union/exact-case relations

signature.rs
    CallableSemanticSignature
    CallableSignatureTable

enum_semantics.rs
    EnumInfo / VariantInfo
    VariantConstructorSignature
    CaseTypeEnvironment
```

Searches against the pinned baseline show no production ordinary-inference use of:

```text
TypeData::Infer
TypeStore::infer
LocalConstraintSolver
```

Do not resurrect tasks from older plans that assume those are still active.

## 2.2 Confirmed remaining gaps

Current source still has all of the following:

1. direct receiver substitution rather than owner-relative inherited specialization in `CheckingContext::specialize_dispatch_signature`;
2. generic constraints not receiver-specialized there;
3. associated-only generic-supertype projection in `checker/associated.rs`;
4. compound inference subtyping falling back to equality/unification;
5. declaration upper bounds capable of becoming implicit candidates through solver bound reconciliation;
6. sticky pre-context underconstraint behavior in generic call analysis;
7. no explicit generic `Ambiguous` solver outcome;
8. materialization errors collapsed into empty underconstraint in some paths;
9. unqualified type-name call logic that maps runtime argument types positionally to declaration generic arguments;
10. variant invocation `Object` fallback and manually pre-materialized non-generic target;
11. family application paths that can lose generic signatures through `TypeData::Callable` projection;
12. no union-receiver method-call closure in the ordinary dispatch path;
13. generic getter AST/signature prohibition (deliberately SC-7, not SC-2).

---

# 3. File map

## 3.1 Create

```text
phalcom-semantic/src/types/specialization.rs

phalcom-semantic/tests/semantic/foundations/receiver_specialization.rs
phalcom-semantic/tests/semantic/foundations/generic_application.rs
phalcom-semantic/tests/semantic/foundations/union_calls.rs
```

Optional if implementation size warrants extraction rather than growing `call.rs`:

```text
phalcom-semantic/src/checker/call_union.rs
phalcom-semantic/src/checker/generic_application.rs
```

Do not create these optional modules merely to satisfy the plan. Create them only if `call.rs` ownership becomes materially clearer.

## 3.2 Modify — core production

```text
phalcom-semantic/src/types/mod.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/substitution.rs          # only low-level helpers if required
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/outcome.rs
phalcom-semantic/src/types/denotation.rs

phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/checker/analysis.rs            # only if a new call outcome product is retained
phalcom-semantic/src/checker/body.rs                # only if canonical signature table access is needed
phalcom-semantic/src/checker/mod.rs                 # module registration if extracted files land

phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/signature.rs                   # only if view/recovery API is needed
phalcom-semantic/src/enum_semantics.rs              # only if constructor application view helper belongs here
phalcom-semantic/src/diagnostic.rs
phalcom-semantic/src/explain.rs
phalcom-semantic/src/session.rs                     # dependency/input plumbing if canonical signature recovery requires it
phalcom-semantic/src/db/fingerprint.rs              # only if a published product shape changes
```

## 3.3 Modify — focused tests

```text
phalcom-semantic/tests/semantic/foundations/mod.rs
phalcom-semantic/tests/semantic/capabilities/generics.rs
phalcom-semantic/tests/semantic/capabilities/higher_order.rs
phalcom-semantic/tests/semantic/capabilities/type_lambdas.rs
phalcom-semantic/tests/semantic/capabilities/constraints.rs
phalcom-semantic/tests/semantic/capabilities/dispatch.rs
phalcom-semantic/tests/semantic/capabilities/callable_publication.rs

phalcom-semantic/tests/semantic/adts/constructors.rs
phalcom-semantic/tests/semantic/adts/generics.rs
phalcom-semantic/tests/semantic/adts/associated.rs

phalcom-semantic/tests/semantic/incremental/callable_dependencies.rs
phalcom-semantic/tests/semantic/incremental/checker_dependencies.rs
```

Modify only the files whose test category actually owns the scenario; do not scatter duplicate tests.

---

# 4. Execution dependency graph

```text
Task 0  Re-pin and characterize baseline
   ↓
Task 1  RED SC-2 semantic matrix
   ↓
Task 2  Canonical owner-relative receiver specialization
   ↓
Task 3  Dispatch/application integration of specialization
   ↓
Task 4  Complete inference-term lifting and subtype decomposition
   ↓
Task 5  Selection roles + expected-only inference + evidence policy
   ↓
Task 6  Ambiguity + materialization outcomes + convergence policy
   ↓
Task 7  `where`/F-bound admissibility semantics
   ↓
Task 8  Ordinary source constructor application closure
   ↓
Task 9  ADT/variant residual generic + GADT construction closure
   ↓
Task 10 Generic family/associated target recovery
   ↓
Task 11 Union-receiver call closure
   ↓
Task 12 HKT / closure / structural regression closure
   ↓
Task 13 Incremental + source/native/generated conformance
   ↓
Task 14 Deletion ledger, full verification, docs sync
```

Tasks 8–10 may overlap after Tasks 4–7 are stable. Task 11 should wait until the ordinary single-receiver application product is stable.

---

# Task 0 — Re-pin, verify prerequisites, and record baseline

## Goal

Prevent implementation against stale assumptions.

## Step 0.1 — Re-pin implementation SHA

Run:

```bash
git status --short
git rev-parse HEAD
git log -1 --oneline
```

If HEAD differs from:

```text
01e19adb86186d67212b558ba76f54f79e2b5d9f
```

update both SC-2 documents with the actual implementation SHA before editing production code.

## Step 0.2 — Confirm SC-1 gate

Do not start SC-2 production changes until SC-1 proves:

- source generic declaration signatures are canonical;
- source method generic signatures and `where` constraints are canonical;
- generic superclass templates are published;
- `Self` owner/side is correct;
- type lambdas are capture-safe;
- malformed formation outcomes do not collapse to ordinary unknowns.

If any is missing, file/record it as an SC-1 blocker instead of compensating in SC-2 call analysis.

## Step 0.3 — Confirm ordinary inference migration is already complete

Run:

```bash
rg -n 'TypeData::Infer|TypeStore::infer|LocalConstraintSolver' phalcom-semantic/src
```

Expected:

```text
no production ordinary-inference ownership
```

Do not count row-solver-local `RecordRowVarId` as a failure.

## Step 0.4 — Record current focused baseline

Before changing tests:

```bash
cargo check -p phalcom-semantic
cargo test -p phalcom-semantic --test semantic semantic::capabilities::generics -- --nocapture
cargo test -p phalcom-semantic --test semantic semantic::adts::constructors -- --nocapture
```

Record actual failures; do not normalize them into the SC-2 expected RED set.

Suggested commit if only docs/characterization notes change:

```text
docs(semantic): re-pin SC-2 execution baseline
```

---

# Task 1 — Add the RED SC-2 semantic matrix

## Goal

Make all changed/added semantics executable before implementation.

Create and register:

```text
phalcom-semantic/tests/semantic/foundations/receiver_specialization.rs
phalcom-semantic/tests/semantic/foundations/generic_application.rs
phalcom-semantic/tests/semantic/foundations/union_calls.rs
```

Modify:

```text
phalcom-semantic/tests/semantic/foundations/mod.rs
```

## 1.1 Receiver specialization RED cases

Add:

### Direct receiver

```phalcom
class Box<T> {
    value(_ x: T) -> T { x }
}

class Probe {
    @class
    run(_ box: Box<Int>) {
        let x = box.value(1)
    }
}
```

Expected:

```text
parameter = Int
result    = Int
```

This may already be GREEN. Keep as a characterization guard.

### Transformed single-hop inheritance

```phalcom
class Parent<T> {
    value() -> T { ... }
}

class Child<T> is Parent<List<T>> {}
```

Receiver `Child<Int>` must yield `List<Int>`.

### Multi-hop transformation

Use a chain conceptually:

```text
Leaf<Int>
 -> Middle<Option<Int>>
 -> Base<List<Option<Int>>>
```

Assert the inherited `Base` method sees the fully materialized owner argument.

### Class generic + method generic

```phalcom
class Pairer<T> {
    pair<U>(_ value: U) -> (T, U) { ... }
}
```

`Pairer<Int>.pair("x")` -> `(Int, String)`.

### Receiver generic in method constraint

Use accepted source constraint syntax equivalent to:

```text
where U <: T
```

on receiver `Holder<Animal>` and prove the solver sees `U <: Animal`.

### Nested `Self`

```phalcom
class Parent {
    wrap() -> Box<Self> { ... }
}
class Child is Parent {}
```

`Child.wrap()` -> `Box<Child>`.

### `super`

Characterize generic projection for the actual parser's `super` representation. If source syntax is not executable yet, add a lower-level semantic test for owner-relative parent projection and record the source test as blocked by the owning earlier feature.

## 1.2 Generic application RED cases

Add tests for:

### Expected-only inference — intentional policy change

```phalcom
class Probe {
    @class
    make<T>() -> T { ... }

    @class
    run() {
        let value: Int = Probe.make()
    }
}
```

Expected:

```text
call type   = Int
call status = Ready
knowledge status <= Assumed
no GenericInferenceUnderconstrained
```

Update/replace the existing test named roughly:

```text
expected_context_cannot_fabricate_missing_generic_return
```

Do not delete the law; rewrite it as:

```text
expected_context_selects_but_does_not_establish_result_only_generic
```

### No context remains underconstrained

```phalcom
let value = Probe.make()
```

### Constraint-only upper bound does not default

```phalcom
make<T>() -> T where T <: Number
let value = Probe.make()
```

Expected underconstrained.

### Candidate + upper bound

`Int` candidate accepted against `Number`; `String` candidate rejected.

### F-bound

Use a fixture supported by current constraint syntax / hierarchy. Candidate first, then verify recursive relation.

### Compound covariant result constraint

Expected `List<Number>` against result `List<T>` must generate variance-aware `T <: Number`, not equality.

### Contravariant callable term

A generic variable under callable parameter position must use function variance.

### HKT inference

Use the accepted syntax for:

```text
F: Type -> Type
```

and infer `F=List`, `T=Int` from an `F<T>` occurrence.

### Kind mismatch

Attempt to solve constructor-kinded `F` with `Int` and assert structured failure.

### Context + unknown value premise

Prove context can choose a substitution without fabricating required value evidence.

## 1.3 Union-call RED cases

Add:

- both arms have same method, same return;
- both arms have same method, different return -> union result;
- one arm missing method -> invalid;
- per-arm generic solutions differ -> joined result;
- common contextual closure expectation -> closure analyzed once;
- incompatible contextual closure expectation -> structured blocked/ambiguous result, not duplicated analysis.

For every union scenario assert expression-analysis count/identity where practical to prove source arguments are not re-analyzed per arm.

## 1.4 ADT constructor RED cases

Modify `semantic/adts/constructors.rs` with:

```phalcom
Result::Ok(1)
```

Expected: result-relevant `E` underconstrained.

Add contextual:

```phalcom
const x: Result<Int, Error> = Result::Ok(1)
```

Expected exact result.

Add nullary:

```phalcom
const x: Option<Int> = Option::None()
```

Expected `T=Int` from result context.

Add no-context nullary underconstraint if its owner parameter is result-relevant.

## 1.5 Ordinary constructor RED cases

Add a source class generic constructor whose constructor value parameter order is deliberately different from declaration generic parameter order to prove the old positional heuristic is invalid.

Example shape:

```phalcom
class Pair<A, B> {
    @constructor
    new(_ second: B, _ first: A) { ... }
}
```

A call must infer from formal constructor types, not `[arg0 -> A, arg1 -> B]`.

## 1.6 Commit RED tests

Run focused tests and record exact RED failures.

Suggested commit:

```text
test(semantic): characterize SC-2 generic application gaps
```

Do not implement before this commit unless repository policy forbids RED commits.

---

# Task 2 — Add canonical owner-relative receiver specialization

## Goal

Replace direct-only receiver substitution as the definition of inherited generic member specialization.

## Files

Create:

```text
phalcom-semantic/src/types/specialization.rs
```

Modify:

```text
phalcom-semantic/src/types/mod.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/checker/context.rs
```

## Step 2.1 — Add specialization domain

Implement a result-rich API conceptually:

```rust
pub fn specialize_receiver_to_owner(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    receiver: TypeId,
    target_owner: &DeclarationId,
    control: &impl SpecializationControl,
) -> Result<ReceiverSpecialization, ReceiverSpecializationFailure>;
```

The concrete control API may reuse `CheckerControl` through an adapter or accept budget/cancellation primitives directly. Avoid a types module depending cyclically on checker internals.

## Step 2.2 — Decompose receiver

Support:

```text
Nominal D
Applied(Nominal D, args)
ExactCase(enum_type)  -> project from carrier where member semantics require it
ClassObject D         -> according to class-side dispatch semantics
```

Reject unsupported structural receiver forms here; union is handled at call resolution by arm decomposition.

## Step 2.3 — Project through templates

Move/generalize the algorithm currently in:

```text
checker/associated.rs::project_supertype_arguments
```

into `types/specialization.rs`.

At each hop:

1. bind current declaration's generic parameters to current concrete args;
2. materialize/view its `GenericSupertypeTemplate`;
3. decompose next owner + args;
4. charge budget/check cancellation;
5. record step;
6. stop at selected target owner.

Cycle -> structured failure.

## Step 2.4 — Build target owner environment

When target owner reached:

```text
target owner TypeParameterId #i -> projected arg #i
```

Bind `Self` to the original actual receiver according to role/side semantics when the view is later consumed.

## Step 2.5 — Remove associated-only projection owner

Refactor `checker/associated.rs` to call the new utility.

Delete or reduce `project_supertype_arguments` to a thin compatibility wrapper; final deletion preferred.

## Step 2.6 — Dependency capture

Ensure the specialization call in checker context goes through `TrackingTypeHierarchy`, not `.inner()`, when semantic dependencies must be recorded.

Test:

```text
changed generic superclass template
-> HierarchyEdge dependency invalidates affected call
```

## Focused tests

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::receiver_specialization -- --nocapture
```

Suggested commit:

```text
feat(semantic): centralize owner-relative receiver specialization
```

---

# Task 3 — Integrate receiver specialization into dispatch/application

## Goal

Make every selected callable carry an owner-relative specialization environment rather than only an eagerly substituted direct receiver signature.

## Files

Modify:

```text
phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/explain.rs
```

Potentially:

```text
phalcom-semantic/src/signature.rs
```

## Step 3.1 — Strengthen `DispatchSignatureSpecialization`

Current shape retains only:

```text
receiver
unspecialized_return
```

Change/reform it to retain enough specialization semantics:

```rust
pub struct DispatchSignatureSpecialization {
    pub receiver: TypeId,
    pub declaring_owner: DeclarationId,
    pub environment: TypeEnvironment,
    pub unspecialized_return: TypeKnowledge,
    // optional path/explanation payload
}
```

If copying `TypeEnvironment` into this compatibility projection is too expensive, store a compact owner/arg/path product from which the environment can be rebuilt deterministically.

## Step 3.2 — Resolve selected owner, then project

`resolve_dispatch_target(...)` already returns `resolved.callable`. Use:

```text
resolved.callable.declaration_owner()
```

as the target owner for `specialize_receiver_to_owner`.

Do not assume the root receiver declaration owns the selected method.

## Step 3.3 — Stop treating `specialize_dispatch_signature` as full semantics

Either:

A. keep an eagerly specialized projected `CallableSignature` for compatibility but build it from the new owner-relative environment; or

B. leave the signature canonical/projected and let `call.rs` apply the environment lazily.

Preferred transition:

1. make the existing projected signature correct using the new environment;
2. add lazy lifting in Task 4;
3. reduce eager substitution in hot paths after tests are stable.

## Step 3.4 — Specialize constraints

When a method-local generic constraint contains declaration-owned parameters:

```text
U <: Holder::T
```

application lifting must see:

```text
U <: Animal
```

for `Holder<Animal>`.

Do not mutate the canonical `GenericSignature`; produce a specialized inference view/term.

## Step 3.5 — Explanation

Extend callable selection explanation so it can retain:

```text
actual receiver
selected callable
selected declaring owner
specialization path
projected generic arguments
```

Avoid formatting strings as semantic data.

## Focused tests

Run receiver matrix plus existing dispatch/generics.

Suggested commit:

```text
feat(semantic): apply owner-relative specialization to dispatch
```

---

# Task 4 — Complete inference-term lifting and subtype decomposition

## Goal

Make solver-local relations obey canonical type-relation semantics when generic variables occur inside compound types.

## Files

Modify:

```text
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/environment.rs
```

Potentially create internal helper module if the file becomes unwieldy:

```text
phalcom-semantic/src/checker/inference_relation.rs
```

## Step 4.1 — Environment-aware inference lifting

Add a lifting function that accepts:

```text
canonical TypeId
fixed receiver/declaration TypeEnvironment
TypeParameterId -> InferenceTerm map
```

It must recursively expose generic variables through all ordinary SC-2 compound shapes.

Current `type_id_to_inference` already handles:

```text
Parameter
Applied
ExactCase
Union
Tuple
Callable
```

Audit and close generic-variable hiding under:

```text
closed Record fields
Family members
other canonical containers reachable from ordinary generic signatures
```

Do not add open row variables here.

## Step 4.2 — Applied subtype decomposition

Replace fallback unification for:

```text
Applied <: Applied
```

with variance-aware decomposition matching `types/relation.rs`.

For different nominal origins, reuse the canonical generic-supertype projection semantics rather than reporting structural mismatch prematurely.

## Step 4.3 — Callable subtype decomposition

Implement contravariant parameter / covariant return inference-term relations with exact label/rest shape compatibility.

## Step 4.4 — Tuple / exact case / union

Make these match canonical relation behavior.

## Step 4.5 — Materializable canonical shortcut

When both inference terms can be completely materialized, delegate to the bounded canonical relation engine rather than duplicating a second final relation decision.

Expose a helper through checking context so shared cancellation/budget is preserved.

## Step 4.6 — HKT application

Ensure applied terms can contain variables in constructor position:

```text
?F<?T>
```

Binding `?F` to a constructor-kinded canonical form must kind-check.

Add occurs checks for solver terms where constructor variables recursively contain themselves.

## Step 4.7 — Type-lambda candidates

When a canonical lambda/type-form candidate of the correct arrow kind can be bound to a constructor variable, permit it. Materialization/application continues through canonical `TypeStore` type-lambda/application rules.

## Tests

Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::generic_application -- --nocapture

cargo test -p phalcom-semantic --test semantic \
  semantic::capabilities::type_lambdas -- --nocapture

cargo test -p phalcom-semantic --test semantic \
  semantic::capabilities::higher_order -- --nocapture
```

Suggested commit:

```text
feat(semantic): make generic inference relations variance-aware
```

---

# Task 5 — Separate selection from admissibility and enable expected-only inference

## Goal

Implement the most important SC-2 semantic policy change without weakening evidence integrity.

## Files

Modify:

```text
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/expected.rs       # only if new origin/basis needed
phalcom-semantic/src/types/evidence.rs         # only if existing Assumed/origin cannot represent contextual-only selection
phalcom-semantic/src/explain.rs
phalcom-semantic/src/diagnostic.rs
phalcom-semantic/tests/semantic/capabilities/generics.rs
```

## Step 5.1 — Add constraint role / selection provenance

Preferred:

```rust
pub enum InferenceConstraintRole {
    ValueSelection,
    ContextSelection,
    ExactSemanticSelection,
    DeclarationRestriction,
}
```

Add role to stored constraints, or implement an equally explicit origin-backed policy that survives bound normalization.

## Step 5.2 — Preserve existing value support

Keep `InferenceSupport::{Established,Assumed}` for actual value support.

Do **not** fake:

```text
ExpectedResult -> Established support
```

Instead track contextual selection separately.

## Step 5.3 — Remove sticky first-pass underconstraint

Current generic call path saves the argument-only underconstrained outcome and can diagnose it even if expected-result solving later succeeds.

Change logic to:

```text
argument/declaration solve
    ↓
if eligible, add expected-result constraint
    ↓
final solve
    ↓
classify FINAL outcome
```

Retain the pre-context complete result only for the existing precision-preservation law when expected context creates a contradiction after a complete argument-derived solution.

## Step 5.4 — Expected-only result selection

Add RED->GREEN behavior:

```phalcom
make<T>() -> T
const x: Int = make()
```

Final substitution `T=Int`.

Publication:

```text
Known(Int)
status <= Assumed
```

No `GenericInferenceUnderconstrained` diagnostic.

## Step 5.5 — Keep no-context underconstraint

`make<T>() -> T` without expected context remains underconstrained.

## Step 5.6 — Keep expected contradiction precision

Do not regress current behavior where argument-derived `String` remains the precise call fact despite expected `Int` contradiction.

## Step 5.7 — Context cannot repair missing proof premises

Add a test where contextual substitution is structurally solved but a required value premise is Unknown/Dynamic. Ensure publication remains weakened/blocked according to current evidence laws.

Suggested commit:

```text
feat(semantic): distinguish contextual generic selection from value evidence
```

---

# Task 6 — Add ambiguity, structured materialization failure, and shared convergence policy

## Goal

Finish result-rich inference terminal semantics.

## Files

Modify:

```text
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/types/outcome.rs
phalcom-semantic/src/diagnostic.rs
phalcom-semantic/src/explain.rs
```

## Step 6.1 — Add `InferenceOutcome::Ambiguous`

Introduce a real finite-candidate ambiguity product.

Do not return Ambiguous merely because a variable is unsolved.

## Step 6.2 — Add diagnostic code

Add:

```text
DiagnosticCode::GenericInferenceAmbiguous
-> type.generic.inference_ambiguous
```

Include guidance only when actionable; do not invent explicit type-argument syntax if the language has not ratified it.

## Step 6.3 — Structured materialization

Replace:

```text
TypeApplicationError -> Underconstrained { [] }
```

with a materialization failure domain.

Map user-caused kind/application problems to structured generic/type-application failures and internal invariants to incidents.

## Step 6.4 — Convergence policy

Replace hard-coded unaccounted `max_passes = 16` with shared/named policy.

Preferred reuse:

```text
QueryBudget.max_scc_iterations
```

if semantically acceptable; otherwise add a generic-inference iteration field to shared `QueryBudget` rather than a hidden local constant.

## Step 6.5 — Correct blocked reason

Do not label ordinary solver nonconvergence as `RecursiveFixpoint` unless recursion/fixpoint semantics are actually involved.

Add a precise `BlockReason` if necessary, for example:

```rust
InferenceDidNotConverge
UnsupportedInferenceDomain
```

Names are implementation choices; semantic distinction is required.

Suggested commit:

```text
feat(semantic): complete generic inference terminal outcomes
```

---

# Task 7 — Correct `where` constraints and F-bounds

## Goal

Make declaration restrictions formal without turning them into type defaults.

## Files

Modify:

```text
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/explain.rs
```

## Step 7.1 — Preserve GenericWhere provenance

Current `ConstraintOrigin::GenericWhere` is good. Retain it.

Add declaration-restriction role.

## Step 7.2 — Prevent single-upper defaulting

Current solver can bind an unsolved variable to a sole upper bound when there is no lower bound.

Change candidate reconciliation so:

```text
only declaration-restriction upper bound
-> does not select candidate
```

A candidate selected/anchored by argument/context/exact-semantic constraints may still be checked against that upper bound.

## Step 7.3 — Exact/equivalence propagation

`where T == U` aliases variables. If one side is selected by call evidence, propagate to the other. If neither is selected, remain underconstrained.

## Step 7.4 — F-bound strategy

For recursive relational restrictions:

```text
T <: Comparable<T>
```

avoid fallback unification.

Preferred flow:

```text
selection phase finds T candidate
        ↓
materialize restriction under candidate
        ↓
canonical bounded relation check
```

If selection cannot proceed without interpreting the relation, store a deferred restriction obligation and revisit after progress.

## Step 7.5 — Terminal relation propagation

Map canonical relation outcomes exactly:

```text
Refuted          GenericConstraintUnsatisfied
Blocked          AnalysisStatus::Blocked
DynamicBoundary  DynamicBoundary
Cancelled        Cancelled
BudgetExceeded   BudgetExceeded
InternalFailure  Internal incident
```

Suggested commit:

```text
fix(semantic): treat generic bounds as admissibility constraints
```

---

# Task 8 — Close ordinary generic constructor application

## Goal

Eliminate unqualified type-name generic guessing and route construction through canonical callable semantics.

## Files

Modify:

```text
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/dispatch.rs
```

Potentially:

```text
phalcom-semantic/src/signature.rs
phalcom-semantic/src/session.rs
```

## Step 8.1 — Characterize source syntax

Confirm parser/runtime meaning of:

```text
TypeName(args...)
TypeName.new(args...)
```

Do not merge type-form application and runtime construction.

## Step 8.2 — Delete generic argument positional heuristic

Remove the `synthesize_unqualified_call` branch equivalent to:

```rust
for runtime arg:
    synthesize type
if arg count == declaration generic parameter count:
    apply_type_form(declaration, arg_types)
```

No replacement heuristic.

## Step 8.3 — Resolve canonical constructor

For construction syntax, resolve the canonical constructor/allocator target by ordinary selector/class-object semantics.

Create residual owner generic variables when the owner is not fully specialized.

## Step 8.4 — Compose owner residuals with constructor-local generics

If constructors themselves can own generic binders, maintain separate `TypeParameterId`s and instantiate both scopes in one application session.

## Step 8.5 — Constructor result

Materialize constructor `Self` after owner residual variables solve.

No broad fallback on unsolved owner parameters.

## Tests

Add reordered-parameter constructor test proving inference follows formal type positions rather than declaration parameter index.

Suggested commit:

```text
feat(semantic): route generic construction through canonical calls
```

---

# Task 9 — Close ADT/variant residual generic inference and GADT construction

## Goal

Turn the existing rich enum semantic products into ordinary application inputs.

## Files

Modify:

```text
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/enum_semantics.rs             # helper only if useful
phalcom-semantic/src/types/case_environment.rs     # helper only if useful
phalcom-semantic/src/explain.rs
phalcom-semantic/tests/semantic/adts/constructors.rs
phalcom-semantic/tests/semantic/adts/generics.rs
```

## Step 9.1 — Build variant application target from semantic products

Inputs:

```text
EnumInfo.generic_signature
owner supplied arguments
VariantConstructorSignature.parameters
VariantConstructorSignature.result_type_template
VariantConstructorSignature.exact_case_template
VariantInfo.case_environment
```

Do not pre-materialize away residual owner parameters.

## Step 9.2 — Seed fixed owner parameters

For:

```text
Result<Int>::Ok(...)
```

bind fixed owner prefix before fresh variables.

## Step 9.3 — Instantiate residual owner parameters

Create inference variables for remaining owner parameters.

Example:

```text
Result<Int, ?E>
```

## Step 9.4 — Remove `Object` fallback

Delete constructor parameter recovery equivalent to:

```rust
canonical_type().unwrap_or(object_ty)
```

If parameter declaration semantics are incomplete, propagate a structured blocked/malformed target.

## Step 9.5 — Generate argument constraints normally

Use `apply_resolved_callable`/generic application engine. Do not manually compare payload arguments in a separate generic solver.

## Step 9.6 — GADT exact constraints

Translate `CaseTypeEnvironment` bindings into exact semantic constraints against fixed/residual owner variables.

If explicit owner arguments contradict case constraints, produce structured GADT/generic conflict before publication.

## Step 9.7 — Expected result

Allow expected result to select residual owner parameters, including nullary constructors.

Required cases:

```text
Result::Ok(1)                      -> E underconstrained
Result::Ok(1) expected Result<Int,Error> -> solved
Option::None() expected Option<Int>      -> solved
```

## Step 9.8 — Remove dependent fallback result

Delete/restrict `fallback_result_type` usage where it could publish a result that depends on failed/unsolved variables.

Independent fixed result behavior remains allowed.

Suggested commit:

```text
feat(semantic): unify variant constructor generic inference
```

---

# Task 10 — Preserve generic semantics through family/associated invocation

## Goal

Avoid losing generic binders when a callable is captured inside a family value.

## Files

Modify:

```text
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/types/denotation.rs
phalcom-semantic/src/checker/context.rs          # only if canonical signature lookup attachment is chosen
phalcom-semantic/src/checker/body.rs             # only if attachment input needed
phalcom-semantic/src/session.rs                   # only if attachment input needed
```

## Step 10.1 — Behavioral family path

`AssociatedValueDenotation::BehavioralFamily` already retains:

```text
receiver_type
spec
operation/target identities
```

When selected member target is behavioral:

1. recover its selector/operation;
2. re-resolve canonical dispatch target against retained receiver;
3. obtain generic `CallableSignature` + receiver specialization;
4. call `apply_resolved_callable`.

Do not use the monomorphic `TypeData::Callable` projection as generic declaration authority.

## Step 10.2 — Associated variant family path

Use retained owner form + `AssociatedMemberId::Variant` + constructor identity to recover enum/variant semantic products and reuse Task 9 application target construction.

## Step 10.3 — Keep callable value semantics monomorphic

An ordinary lexical `TypeData::Callable` with no retained declaration/family target remains monomorphic. Do not infer a hidden `forall`.

## Step 10.4 — Canonical signature table attachment only if required

If target recovery cannot be implemented reliably through dispatch/enum tables, add an optional `CallableSignatureTable` reference to `CheckingContext` parallel to `FieldSignatureTable` and thread it through `BodyAnalysisContext`/session.

If you do this:

- canonical table is authority;
- dispatch remains selector/index surface;
- do not create another copied table.

Suggested commit:

```text
fix(semantic): retain generic declarations through family invocation
```

---

# Task 11 — Implement union-receiver call closure

## Goal

Satisfy the existing 04.5 law that a statically known union receiver is checked across every reachable arm.

## Files

Modify:

```text
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/explain.rs
```

Optional extraction:

```text
phalcom-semantic/src/checker/call_union.rs
```

## Step 11.1 — Detect canonical union receiver before nominal dispatch owner lookup

Do not teach `dispatch_owner_for_lookup` to pretend a union has one owner.

Create call-level decomposition:

```text
receiver TypeData::Union(arms)
-> per-arm dispatch target
```

## Step 11.2 — Resolve every arm

For each arm:

- use same selector shape;
- resolve target;
- build owner specialization;
- retain result/failure product.

Missing/ambiguous/dynamic states remain arm-specific evidence.

## Step 11.3 — Bind source arguments once

Static call shape is common and should be computed once.

For each argument position:

1. collect specialized formal expectations from successful arm targets;
2. if all are semantically equivalent, analyze argument once under common expectation;
3. otherwise analyze once without branch-specific contextual expectation;
4. check resulting knowledge against each arm.

Do not invoke `analyze_expression` once per receiver arm.

## Step 11.4 — Contextual closure incompatibility

If a closure cannot synthesize independently and arm contexts disagree, return a structured blocked/ambiguous call outcome. Do not type the same closure differently on multiple pseudo-executions and then merge arbitrary results.

## Step 11.5 — Per-arm generic solving

Each receiver arm has an independent generic application session. Reuse pre-analyzed argument products as `ApplicationArgument::PreAnalyzed` or introduce an explicit normalized typed-argument frame.

## Step 11.6 — Join successful results

Union/canonicalize result types.

Meet evidence status across arms.

Any static arm refutation makes the union call invalid unless a separate dynamic boundary explicitly applies.

## Step 11.7 — Explanations

Add arm identity and target/failure roots to the call explanation.

## Tests

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::union_calls -- --nocapture
```

Suggested commit:

```text
feat(semantic): type union receiver calls across all arms
```

---

# Task 12 — Close HKT, contextual closure, and structural generic regression matrix

## Goal

Prove the generic application algorithm works beyond simple `T` substitution.

## Files

Tests primarily:

```text
phalcom-semantic/tests/semantic/foundations/generic_application.rs
phalcom-semantic/tests/semantic/capabilities/higher_order.rs
phalcom-semantic/tests/semantic/capabilities/type_lambdas.rs
phalcom-semantic/tests/semantic/capabilities/generics.rs
```

Production only if tests expose gaps:

```text
checker/inference.rs
checker/expression.rs
checker/call.rs
types/environment.rs
```

## 12.1 HKT cases

Test:

```text
F : Type -> Type
F<T>
List<Int>
=> F=List, T=Int
```

Test a canonical type-lambda constructor candidate.

Test wrong-kind rejection.

## 12.2 Contextual closure cases

Test:

```text
T selected by earlier argument
-> closure param expected T
-> closure result selects U
```

Test assumed closure source weakens return-relevant `U` appropriately.

## 12.3 Nested generic calls

Closure result contains another generic call whose expected type is inference-shaped. Ensure expression identities remain stable and solver sessions do not leak across calls.

## 12.4 Closed record/family nested variables

If a generic type variable can appear under a closed structural type in source signatures, add a test proving inference lifting exposes it. If source formation currently forbids such shape, record the restriction and do not invent syntax.

## 12.5 Call isolation

Keep/extend existing tests proving repeated generic calls in one body do not share variables.

Suggested commit:

```text
test(semantic): close higher-order generic application matrix
```

---

# Task 13 — Incremental dependency correctness and conformance

## Goal

Prove SC-2 semantics are stable across incremental updates and implementation provenance.

## Files

Modify:

```text
phalcom-semantic/tests/semantic/incremental/callable_dependencies.rs
phalcom-semantic/tests/semantic/incremental/checker_dependencies.rs
phalcom-semantic/tests/semantic/capabilities/callable_publication.rs
phalcom-semantic/tests/semantic/adts/native_core.rs
```

Potential production changes:

```text
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/db/fingerprint.rs
```

## Step 13.1 — Generic superclass template dependency

Fixture:

```text
Child<Int> -> Middle<T> -> Parent<List<T>>
```

Analyze inherited parent call.

Edit only template:

```text
Parent<List<T>> -> Parent<Set<T>>
```

Assert caller recomputes and result changes exactly.

## Step 13.2 — Unrelated body edit reuse

Change selected callee body while keeping canonical signature fingerprint unchanged.

Assert caller's call typing is reused when dependency graph says signature unchanged.

## Step 13.3 — Signature edit invalidation

Change parameter/return/generic constraint and prove exact caller invalidation.

## Step 13.4 — Source/native conformance

Construct semantically equivalent source/native generic callable signatures and apply identical argument/expected contexts.

Assert:

```text
same substitution
same terminal inference class
same result TypeId structure
```

Evidence origin may differ.

## Step 13.5 — Variant/native enum parity

Where core `Option`/`Result` native enum surfaces exist, compare generic constructor application semantics with equivalent source enum declarations.

## Step 13.6 — Cold/incremental equivalence

For each incremental scenario, compare final incremental snapshot facts with a fresh cold rebuild.

Suggested commit:

```text
test(semantic): prove SC-2 incremental and provenance parity
```

---

# Task 14 — Deletion ledger, audits, full verification, and documentation sync

## Goal

Prove no old semantic authority remains and record the completion evidence.

## Step 14.1 — Search for constructor positional guessing

Run targeted searches, adjusting patterns to actual code after refactors:

```bash
rg -n 'arg_tys|apply_type_form\(' phalcom-semantic/src/checker/expression.rs
```

Manually verify no ordinary call path implements:

```text
runtime argument index -> declaration generic parameter index
```

## Step 14.2 — Search variant fallbacks

```bash
rg -n 'unwrap_or\(object_ty\)|fallback_result_type' \
  phalcom-semantic/src/checker
```

Expected:

- no `Object` fallback for missing variant constructor type facts;
- any remaining `fallback_result_type` is independently fixed and documented, or removed.

## Step 14.3 — Search specialization duplication

```bash
rg -n 'specialize_dispatch_signature|substitution_for_applied_receiver|project_supertype_arguments|specialize_receiver_to_owner' \
  phalcom-semantic/src
```

Expected architecture:

```text
one canonical owner-relative projection implementation
compatibility wrappers only where justified
```

## Step 14.4 — Search generic application funnels

```bash
rg -n 'InferenceSession::new|apply_resolved_callable|apply_generic_callable' \
  phalcom-semantic/src/checker
```

Manually classify every `InferenceSession::new`.

No constructor/variant/family-specific duplicate solver may remain.

## Step 14.5 — Search generic ambiguity

```bash
rg -n 'GenericInferenceAmbiguous|InferenceOutcome::Ambiguous' \
  phalcom-semantic/src phalcom-semantic/tests
```

Confirm finite ambiguity and underconstraint tests are separate.

## Step 14.6 — Search canonical inference regression

```bash
rg -n 'TypeData::Infer|TypeStore::infer|LocalConstraintSolver' phalcom-semantic/src
```

Expected no ordinary type inference use.

## Step 14.7 — Formatting/check

```bash
cargo fmt --all -- --check
cargo check -p phalcom-semantic
```

## Step 14.8 — Focused suites

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::receiver_specialization -- --nocapture

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::generic_application -- --nocapture

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::union_calls -- --nocapture

cargo test -p phalcom-semantic --test semantic \
  semantic::capabilities::generics -- --nocapture

cargo test -p phalcom-semantic --test semantic \
  semantic::capabilities::higher_order -- --nocapture

cargo test -p phalcom-semantic --test semantic \
  semantic::capabilities::type_lambdas -- --nocapture

cargo test -p phalcom-semantic --test semantic \
  semantic::adts::constructors -- --nocapture

cargo test -p phalcom-semantic --test semantic \
  semantic::adts::associated -- --nocapture

cargo test -p phalcom-semantic --test semantic \
  semantic::incremental::callable_dependencies -- --nocapture

cargo test -p phalcom-semantic --test semantic \
  semantic::incremental::checker_dependencies -- --nocapture
```

## Step 14.9 — Semantic package/full workspace

```bash
cargo test -p phalcom-semantic --test semantic
cargo test -p phalcom-semantic
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

If workspace cost is prohibitive in the implementation environment, run all semantically affected packages and explicitly record what was not run. Do not claim full verification without evidence.

## Step 14.10 — Performance baseline

Run the existing benchmark/metrics harness if available or add focused counters for:

```text
receiver specialization steps
InferenceSession variable/constraint count
solver iteration count
relation pairs
TypeStore node delta
union arm count
```

Record before/after numbers; do not put arbitrary thresholds in the language semantics.

## Step 14.11 — Documentation sync

Update the semantic-completeness status/decision register so it no longer reports stale states such as session-local ordinary inference being absent.

Add explicit SC-2 decisions:

```text
expected-only inference = yes
context is selection, not value evidence
one-sided bounds are not defaults
finite ambiguity distinct from underconstraint
owner-relative receiver specialization required
union receiver calls all arms
```

Suggested final commit:

```text
feat(semantic): complete SC-2 generic callable application semantics
```

---

# 5. Detailed implementation notes

## 5.1 Recommended specialization API placement

Do not put the canonical projection algorithm in `checker/context.rs`; that would make associated/reflection/other semantic consumers depend on checker state.

Preferred layering:

```text
types/specialization.rs
    pure/result-rich specialization over
    TypeStore + TypeHierarchy + explicit control

checker/context.rs
    records semantic dependencies
    adapts terminal failures to AnalysisStatus/explanations
```

## 5.2 Avoid cyclic dependencies with `CheckerControl`

`types/*` should not import checker internals. Use one of:

```text
QueryBudget + CancellationToken parameters
small SpecializationControl trait in types layer
closure/adaptor passed by caller
```

The checker wrapper retains shared budget ownership.

## 5.3 Canonical relation reuse

Do not directly call the boolean compatibility helper if a result-rich bounded relation API exists.

At inference boundaries, preserve:

```text
Proven
Refuted
DynamicBoundary
Blocked
Cancelled
BudgetExceeded
InternalFailure
```

## 5.4 Candidate construction policy

Current lower-bound union construction can remain where it represents the canonical least-upper/principal local solution under Phalcom's union semantics.

Do not generalize this into “union all candidates” for:

- invariant variables;
- constructor-kinded variables;
- finite overload/family candidate selection;
- declaration restrictions.

Every candidate combination rule must be justified by the relation/kind being solved.

## 5.5 Selection provenance

A useful internal model is:

```text
variable solution
    type
    selection bases
        argument established/assumed
        expected context
        exact semantic/GADT/owner
        propagated from another selected variable
    required value proof state
```

Do not overload `InferenceSupport` to mean both “how selected” and “how strongly proven.”

## 5.6 Evidence publication

Recommended policy under the existing two-level `EvidenceStatus`:

```text
return-relevant variables selected only by established value/exact independent contract
    -> Established where declaration authority supports it

any return-relevant variable depends on assumed value evidence
    -> Assumed

any return-relevant variable selected only contextually
    -> Assumed

required return-relevant premise Unknown/Dynamic
    -> Unknown/Dynamic according to existing proof-state meet
```

Do not change the public evidence lattice merely to expose “Contextual” unless another project decision requires that new user-facing state.

## 5.7 Pre-context result fallback

The existing `pre_context_result` concept is useful for this precise case:

```text
arguments completely solve result
expected result later contradicts it
```

It must not be used when the pre-context result contains unsolved variables or when the only reason it materialized was a declaration-bound default.

## 5.8 Family invocation

Prefer re-resolution from retained semantic denotation over storing a copied generic signature in every family value. That keeps runtime/value representation compact and canonical declaration authority centralized.

## 5.9 Union receiver scalability

Avoid eager Cartesian products:

```text
receiver arms × argument union arms × generic candidates
```

Use:

- canonical union normalization;
- one source argument analysis;
- memoized relation checks;
- shared budgets;
- per-arm application sessions;
- early terminal failure when an arm is statically invalid.

## 5.10 SC-3 extension

If generic application encounters a `RecordRow` parameter before SC-3, return an explicit unsupported-domain block. Do not delay SC-2 by implementing row unification inside `InferenceSession` unless SC-3 is intentionally merged into the same branch.

---

# 6. Test ownership map

Use descriptive language-feature test organization rather than implementation-part numbering.

```text
semantic/foundations/receiver_specialization.rs
    low-level owner projection and receiver/call composition laws

semantic/foundations/generic_application.rs
    solver/application invariants, expected-result selection, constraints,
    ambiguity/materialization/HKT low-level laws

semantic/foundations/union_calls.rs
    union receiver argument-once and arm-join laws

semantic/capabilities/generics.rs
    source-level ordinary generic UX

semantic/capabilities/higher_order.rs
    contextual closures/callables

semantic/capabilities/type_lambdas.rs
    HKT/type-lambda source capability

semantic/capabilities/constraints.rs
    source-level where/F-bound behavior

semantic/adts/constructors.rs
    source-level variant construction/application

semantic/adts/generics.rs
    enum generic declaration/case relationships

semantic/adts/associated.rs
    family/associated invocation

semantic/incremental/callable_dependencies.rs
semantic/incremental/checker_dependencies.rs
    exact reuse/invalidation
```

Avoid new files named `sc2.rs`, `part2.rs`, or similar implementation-program terminology.

---

# 7. Completion checklist

SC-2 is complete only when every checkbox is true.

## Specialization

- [x] Direct generic receiver specialization remains correct.
- [x] Inherited generic receiver specialization uses selected declaring owner.
- [x] Transformed superclass arguments work.
- [x] Multi-hop transformed inheritance works.
- [x] `Self` uses actual receiver.
- [x] Receiver params inside callable `where` constraints specialize.
- [x] Associated lookup uses the same specialization implementation.

## Solver

- [x] Compound subtype no longer defaults to equality/unification.
- [x] Applied variance matches canonical relation semantics.
- [x] Callable variance matches canonical relation semantics.
- [x] Generic supertype projection participates in inference relations.
- [x] HKT constructor variables infer with correct kinds.
- [x] Type-lambda constructor candidates work where allowed.
- [x] Closed structural nested generic occurrences are not hidden.
- [x] Row-kind variables remain outside ordinary type inference.

## Selection/evidence

- [x] Expected-only generic selection works.
- [x] Expected-only result is not established merely by context.
- [x] No-context result-only generic remains underconstrained.
- [x] One-sided declaration bound does not default variable.
- [x] Argument-derived precision survives later contextual contradiction.
- [x] Unknown/Dynamic required premises are not erased by context.
- [x] F-bounds are relations, not recursive equalities.

## Outcomes

- [x] Underconstrained exists and is tested.
- [x] Finite Ambiguous exists and is tested.
- [x] Conflicting exists and is tested.
- [x] DynamicBoundary remains distinct.
- [x] Blocked remains distinct.
- [x] Cancelled remains distinct.
- [x] BudgetExceeded remains distinct.
- [x] InternalFailure remains distinct.
- [x] Materialization errors are not encoded as empty underconstraint.

## Executable surfaces

- [x] Ordinary generic methods use canonical application funnel.
- [x] Ordinary generic source constructors use canonical application funnel.
- [x] No runtime-arg/declaration-generic positional heuristic remains.
- [x] Variant constructors infer residual owner generics.
- [x] Nullary variant constructors can use expected context.
- [x] No variant `Object` generic fallback remains.
- [x] No dependent generic fallback result masks terminal failure.
- [x] Generic behavioral family invocation recovers declaration semantics.
- [x] Generic associated variant invocation recovers variant semantic products.
- [x] Ordinary callable values remain monomorphic.

## Union calls

- [x] Every receiver arm is checked.
- [x] One missing arm invalidates static call.
- [x] Source arguments are analyzed once.
- [x] Common contextual expectations are reused safely.
- [x] Incompatible closure expectations fail explicitly.
- [x] Results join canonically.
- [x] Evidence/status joins conservatively.

## Incremental/conformance

- [x] Generic superclass template edit invalidates affected callers.
- [x] Unrelated body edit does not invalidate unchanged call signatures.
- [x] Signature edits invalidate callers.
- [x] Cold/incremental final facts match.
- [x] Source/native/generated generic solution mathematics match.
- [x] Source/native enum constructor semantics match where equivalent.

## Deletion/audit

- [x] No ordinary production `TypeData::Infer` regression.
- [x] No ordinary production `LocalConstraintSolver` regression.
- [x] No positional constructor generic guessing.
- [x] No variant `Object` fallback.
- [x] No duplicated complete generic-supertype projection algorithm.
- [x] No generic family call relying only on lost monomorphic callable type when target is recoverable.
- [x] Old expected-only-underconstrained test replaced by new selection/evidence law.

## Verification

- [ ] `cargo fmt --all -- --check`
- [x] `cargo check -p phalcom-semantic`
- [x] focused SC-2 foundations GREEN
- [x] capability generic/HKT suites GREEN
- [x] ADT constructor/associated suites GREEN
- [x] incremental suites GREEN
- [x] `cargo test -p phalcom-semantic --test semantic` GREEN
- [x] `cargo test -p phalcom-semantic` GREEN
- [x] clippy affected/workspace target GREEN
- [x] workspace test GREEN or explicitly documented unavailable
- [x] deletion ledger searches reviewed
- [x] performance metrics recorded
- [x] decision/status documents updated

---

# 8. Final expected architecture

```text
source/native/generated/enum canonical declaration products
                         │
                         ▼
              selector / target identity
                         │
                         ▼
              owner-relative specialization
                         │
              ┌──────────┴───────────┐
              │                      │
         fixed owner params      residual params
         + actual Self           + callable params
              │                      │
              └──────────┬───────────┘
                         ▼
                 InferenceSession
        argument / expected / exact / restrictions
                         │
                         ▼
     rich bounded local solution / terminal outcome
                         │
                         ▼
             canonical materialization
                         │
                         ▼
        evidence + status + explanation + deps
                         │
                         ▼
                 CallCheckResult
```

The implementation is successful when a new generic executable surface can be made type-correct by supplying a canonical target/signature and specialization inputs—without adding another inference algorithm.

# Phalcom Semantic Correctness Program — Technical Specification 03
## Generic Inference and Proof Integrity

> **Status:** Technical implementation specification
> **Intended repository path:** `docs/impl/semantic/semantic-correctness/part-4/phalcom-semantic-correctness-technical-03-generic-inference-proof-integrity-spec.md`
> **Repository:** `aureat/phalcom-lang`
> **Verified baseline:** `main` at `4599dad282c014669f39e5f42d382e48f89aca9b` (`docs: log Technical 02 slice status`, 2026-08-26)
> **Program position:** Semantic Correctness Technical Specification 03 of 10
> **Preceded by:** Technical Specification 02 — Canonical Callable Application and Operation Semantics
> **Following boundary:** Technical Specification 04 owns complete generic receiver/class substitution and specialization. Its exact title is not ratified by this document.
> **Implementation discipline:** RED regression first. A solved substitution is never, by itself, proof of a call-result proposition.

---

# 1. Purpose

Technical Specification 02 established the canonical outer application law:

```text
syntax selects a callable target
        ↓
canonical argument binding/application
        ↓
generic or non-generic application sub-engine
        ↓
CallCheckResult
```

That removed syntax-specific weak call algorithms.

The remaining generic correctness problem is deeper.

The current generic engine can answer:

```text
Which substitution satisfies the constraints I happened to add?
```

without separately answering:

```text
Did I account for every required value premise needed to justify
the result proposition I am about to publish?
```

Those are different questions.

A concrete current failure shape is:

```phalcom
class Probe {
  @class
  first<T>(_ first: T, _ second: T) -> T {
    first
  }

  @class
  run() {
    let x = Probe.first(1, missing)
  }
}
```

The first argument can generate:

```text
Int <: T
```

while the unresolved second argument contributes no inference constraint at all.

The solver may therefore obtain:

```text
T = Int
```

and the current result-support code can classify `T` from the first argument alone.

That substitution is mathematically solvable.

The complete call-result proposition is not proven.

Technical Specification 03 establishes the missing distinction.

Its central rule is:

> Generic inference solves substitutions. Generic proof accounting determines whether a solved specialization is semantically publishable, and at what epistemic authority.

This slice also repairs two closely related solver-integrity defects that can otherwise fabricate stronger generic conclusions:

1. the generic call path currently performs its own argument-to-parameter walk instead of consuming Technical 02's canonical `ArgumentBindingPlan`;
2. a solver relation `Subtype(Var(T), Var(U))` currently falls back to unification when both variables are unresolved, strengthening `T <: U` into `T == U`.

The objective is not broader generic inference.

The objective is that every generic semantic claim the analyzer publishes is justified by the exact premises and relations the program supplied.

---

# 2. End-state invariant

After this specification, generic application has four explicit semantic layers:

```text
canonical argument binding
        ↓
constraint generation / substitution solving
        ↓
required-premise proof accounting
        ↓
result publication + call status
```

They must not collapse into one another.

The end-state model is:

```text
CallableApplicationTarget
        +
CallPremise
        +
ArgumentBindingPlan
        ↓
analyze each supplied argument exactly once
        ↓
for each bound generic parameter relation:
    record required value premise
    if Known:
        add solver relation with Established/Assumed support
    if Unknown:
        add no fake type constraint
        retain exact UnknownReason as missing proof premise
    if Dynamic:
        add no formal type constraint
        retain exact DynamicReason as dynamic proof premise
        ↓
solve value + declared-generic constraints
        ↓
optionally add expected-result selection constraints
        ↓
solve contextual selection
        ↓
classify return-influencing proof state
        ↓
materialize only a justified result
        ↓
combine with independent fixed-return knowledge
        ↓
CallCheckResult
```

The required separations are:

```text
InferenceOutcome
    answers whether a substitution solved

InferenceProofState
    answers what evidence supports return-influencing variables

TypeKnowledge
    answers what result fact may be published

AnalysisStatus
    answers whether the call operation is ready/invalid/blocked/dynamic/cancelled/budget-exhausted/internal-failure
```

No one axis substitutes for another.

---

# 3. Normative semantic basis

## 3.1 `docs/spec/semantic-analyzer/07-generic-inference-engine.md`

This is the primary normative source for this slice.

It already establishes:

- inference solves type equations; it does not create authority;
- inference variables carry real kinds;
- constraints retain relation and origin;
- argument matching precedes solving and is performed once;
- generic declaration constraints participate formally;
- solver failures remain structured;
- support is monotone and tracks Established versus Assumed value evidence;
- Unknown contributes no usable value constraint;
- Dynamic is a dynamic path, not formal generic evidence;
- expected-result context is selection, not value evidence;
- return authority depends only on return-influencing variables;
- fixed returns may remain established despite unrelated generic failure;
- partial specialization must not be published after terminal failure;
- underconstrained/conflicting/blocked/cancelled/budget outcomes remain distinguishable.

Technical Specification 03 implements these laws in the current checker architecture.

## 3.2 `docs/spec/semantic-analyzer/08-callable-analysis-and-publication.md`

Chapter 08 establishes the publication boundary:

```text
successful generic solution
    ↓
materialized return
    ↓
support classification
    ↓
Established/Assumed(..., GenericInference)
```

It also establishes:

- concrete return syntax alone is insufficient authority;
- result authority is weakened by premises that actually influence it;
- a failed argument judgment need not erase an independently established fixed return;
- a generic return depending on failed inference may instead become Unknown.

Technical Specification 03 produces the proof-complete generic result that Chapter 08 publishes.

## 3.3 `docs/spec/semantic-analyzer/06-relations-reconciliation-and-semantic-judgments.md`

Chapter 06 establishes that semantic relations are structured computations rather than booleans.

Therefore generic inference must preserve:

```text
solved
underconstrained
conflicting
blocked
cancelled
budget exceeded
internal failure
```

where applicable.

It must not turn inability to prove into success.

## 3.4 Technical Specification 01

Technical 01 established required-premise completeness and exact Unknown/Dynamic preservation for ordinary expression composition.

Technical 03 applies the same epistemic law to generic inference:

> A required premise that cannot contribute a concrete `TypeId` is still a semantic premise. Its absence must not disappear from the proof.

## 3.5 Technical Specification 02

Technical 02 owns canonical callable application and `ArgumentBindingPlan`.

Technical 03 must consume that architecture.

It must not create a second generic-only argument matching algorithm.

---

# 4. Scope

This technical slice owns:

1. required generic value-premise accounting;
2. the distinction between substitution solvability and proof completeness;
3. result authority for generic-specialized returns;
4. exact preservation of Unknown/Dynamic generic premises;
5. use of canonical Technical 02 argument binding by generic calls;
6. exact-once generic argument analysis in source order;
7. generic argument status/causal propagation;
8. expected-result constraints as selection only;
9. fixed-return independence from generic proof failure;
10. no partial generic return publication after terminal failure;
11. monotone support through aliases and solver dependencies;
12. directed subtype relations between inference variables;
13. solver outcome preservation;
14. production cancellation and query-budget wiring for the generic solver;
15. generic conflict provenance sufficient for diagnostics/explanations;
16. internal inference invariant failures using the analyzer incident channel;
17. solver-level and call-level regression coverage.

---

# 5. Non-goals

This specification does **not** own:

- complete receiver/class generic substitution through method parameters, returns, method generic bounds, or enclosing `where` clauses;
- complete specialization of enclosing class parameters inside method generic constraints;
- higher-kinded/type-lambda specialization coverage beyond preserving existing kinds/terms correctly;
- new explicit generic-argument syntax;
- broader HKT inference completeness;
- complete `*` / `**` / `***` parameter-pack inference;
- recursive/fixpoint inference completeness beyond honest blocked/budget behavior;
- global/Hindley–Milner inference;
- declaration-level generic model redesign;
- source/formal identity takeover;
- advisory/LSP authority;
- workspace/session transactional correctness;
- incremental publication correctness.

In particular, `phalcom-semantic/src/types/substitution.rs` remains the canonical substitution implementation for the type forms it currently owns. Technical 03 may consume method-local solved substitutions, but it must not absorb Technical 04's receiver/class specialization work.

The experimental document:

```text
docs/spec/typing/typing-inference.md
```

is not normative for this slice. It is explicitly marked Proposed and contains historical choices that differ from the current normative semantic-analyzer specifications.

---

# 6. Verified repository baseline

## 6.1 Current branch

Verified current branch:

```text
main
4599dad282c014669f39e5f42d382e48f89aca9b
```

Technical 02's substantive implementation is present in the parent implementation commit and the tip records its status.

## 6.2 `checker/inference.rs` already has a serious solver model

File:

```text
phalcom-semantic/src/checker/inference.rs
```

Current types include:

```rust
InferenceTerm
InferVarId
InferVarState
InferenceFailureReason
InferenceVariable
InferenceSupport
InferenceRelation
ConstraintOrigin
InferenceConstraint
InferenceSolution
InferenceOutcome
InferenceSession
```

Current `InferenceOutcome` already distinguishes:

```text
Solved
Underconstrained
Conflicting
Blocked
Cancelled
BudgetExceeded
```

Current support correctly distinguishes:

```text
Established
Assumed
```

and `InferenceSupport::join` is monotone.

This specification keeps those foundations.

## 6.3 Kinds are already real solver metadata

`InferenceSession::fresh_variable(kind)` stores the supplied `KindId`.

`instantiate_generic_signature(...)` obtains the kind from the canonical type-parameter metadata.

`bind(...)` checks canonical kind compatibility.

Do not replace this with a `Type` default.

## 6.4 Real conflict data is already partly repaired

Final lower/upper reconciliation records the actual upper bound that fails rather than blindly reporting `uppers[0]`.

Existing regression coverage verifies this.

Do not regress it.

## 6.5 Current result support is insufficient for missing premises

Current `term_support(term)` answers only:

```text
Established
Assumed
None
```

for variables occurring in a return term.

That is adequate only when every required value premise produced usable type evidence.

It cannot represent:

```text
Unknown(UnresolvedName(...))
Dynamic(ExplicitEscape)
```

that influenced a return-relevant variable but generated no solver constraint.

Therefore missing proof premises disappear before publication.

## 6.6 Current generic call path omits Unknown/Dynamic arguments

File:

```text
phalcom-semantic/src/checker/call.rs
```

Inside `apply_generic_callable_inner(...)`, the current shape is effectively:

```rust
let argument_typed = analyze_application_argument(...);

if let Some(argument_ty) = argument_typed.knowledge.ty()
    && let Some(support) = inference_support(&argument_typed.knowledge)
{
    session.add_constraint_with_support(...);
}
```

Consequences:

```text
Known Established -> contributes
Known Assumed     -> contributes
Unknown           -> disappears
Dynamic           -> disappears
```

This directly violates required-premise completeness.

## 6.7 Current generic call path duplicates argument mapping

Technical 02 introduced:

```rust
bind_static_arguments(...)
ArgumentBindingPlan
ArgumentShapeFailure
```

The non-generic path consumes it.

The generic path currently performs a second manual positional/labeled walk with its own `positional_idx` and label search.

That creates correctness drift:

- generic shape failure can differ from non-generic shape failure;
- an unmatched argument can be analyzed and then ignored;
- partial mappings can still feed a substitution;
- one syntax can again select a weaker application algorithm.

This specification removes the duplicate generic binder.

## 6.8 Current generic path does not mirror non-generic argument dependency/status capture

The non-generic path explicitly records:

```text
argument causal invalidity
argument explanation
non-Ready argument status
```

into the surrounding call capture.

The generic path currently lacks equivalent explicit handling in the argument loop.

Required generic premises must not disappear from status or causality merely because their types are unavailable.

## 6.9 Expected context is already modeled correctly as control information

File:

```text
phalcom-semantic/src/checker/expected.rs
```

The file explicitly states that expectations are contextual control information, never value evidence.

`ExpectedType` distinguishes:

```text
None
Proper
Inference
```

with an `ExpectationOrigin`.

Technical 03 preserves this architecture.

Expected result constraints may enter the solver, but they never seed generic value support or required-premise proof authority.

## 6.10 Current solver strengthens variable subtype into equivalence

Inside `InferenceSession::subtype_terms(...)`:

```text
Var vs unresolved Var
    ↓
term cannot materialize
    ↓
fallback to unify_terms(...)
```

`unify_terms(Var, Var)` aliases the variables.

Therefore:

```text
T <: U
```

can become:

```text
T == U
```

depending on constraint order.

That is an unsound strengthening of the declared relation.

It also makes inference order-sensitive.

Technical 03 introduces directed inference-variable subtype edges instead.

## 6.11 Solver terminal variants are not fully reachable in production

`InferenceOutcome` contains:

```text
Cancelled
BudgetExceeded
```

but `InferenceSession::solve(...)` currently takes only:

```text
store
hierarchy
```

and uses a hard-coded pass bound.

Meanwhile `CheckingContext` already owns a shared `CheckerControl` containing:

```text
QueryBudget
CancellationToken
```

and all ordinary relation checks consume that shared control.

Generic inference must join the same query control domain instead of carrying nominally structured but unreachable terminal states.

## 6.12 Internal inference failures already have an analyzer incident domain

`checker/incident.rs` defines:

```rust
InternalSemanticIncidentKind::InferenceInvariantViolation
```

and `CheckingContext::record_internal_incident(...)` publishes structured analyzer incidents separately from source diagnostics.

Malformed solver state must use this channel rather than pretending to be a source-level generic conflict.

## 6.13 Existing tests cover basic inference but not proof completeness

Existing suites include:

```text
phalcom-semantic/tests/semantic/foundations/inference.rs
phalcom-semantic/tests/semantic/foundations/generics_core.rs
phalcom-semantic/tests/semantic/foundations/bidirectional_calls.rs
phalcom-semantic/tests/semantic/capabilities/generics.rs
phalcom-semantic/tests/semantic/foundations/semantic_correctness_regressions.rs
```

They already test:

- fresh variables;
- occurs check;
- kind mismatch;
- aliasing;
- lower-bound union;
- basic generic call specialization;
- established/assumed support;
- expected-result contextual checking;
- real failed upper bound.

They do not comprehensively test:

- one known + one unresolved required generic premise;
- dynamic required generic premise;
- exact UnknownReason preservation through generic return proof;
- expected context solving a substitution without supplying proof;
- generic shape failure refusing partial specialization;
- constraint-order independence for variable subtype;
- reachable cancellation/budget outcomes in production generic solving.

---

# 7. Identified correctness defects

| ID | Current defect | Incorrect semantic consequence |
|---|---|---|
| GPI-01 | Unknown generic argument generates no constraint and no proof-premise record | Remaining arguments can establish an unjustifiably precise generic result |
| GPI-02 | Dynamic generic argument generates no formal constraint and no proof-premise record | Other arguments/context can launder a dynamic boundary into known generic result |
| GPI-03 | `term_support` represents only known support | Exact Unknown/Dynamic reason cannot survive return-influence classification |
| GPI-04 | Generic call manually rematches arguments | Generic application can diverge from Technical 02 shape semantics |
| GPI-05 | Unmatched generic arguments can be analyzed and skipped | Invalid shape can still contribute to/permit partial specialization |
| GPI-06 | Generic argument status/causal capture is weaker than non-generic capture | Invalid/blocked/dynamic argument dependencies can disappear from call product |
| GPI-07 | Expected result and value support share the same solver without an explicit proof boundary | A context-selected substitution can be mistaken for value-established result authority |
| GPI-08 | Terminal generic failure accepts arbitrary fallback knowledge | Partial specialization can survive a failure without proving independence |
| GPI-09 | `Subtype(Var, Var)` can alias variables | Declared subtype relation is strengthened to equality; result becomes order-sensitive |
| GPI-10 | `Cancelled` / `BudgetExceeded` exist but production solve does not consume checker control | Structured outcome domain is partly nominal rather than executable |
| GPI-11 | Missing variable metadata is currently representable as ordinary inference failure | Analyzer invariant failure can be misclassified as source conflict |
| GPI-12 | Generic conflict provenance is richer in the solver than in call diagnostics | User-facing failure can lose the exact argument/constraint that generated it |

---

# 8. Architectural decision

Technical 03 introduces an explicit generic **proof domain** adjacent to, but distinct from, the existing solver support domain.

The solver continues to solve types.

The proof domain accounts for required value premises even when no `TypeId` exists.

The architecture is:

```text
InferenceSupport
    = strength of usable Known value evidence

InferenceProofState
    = complete proof status of a return-influencing inference variable
```

Conceptually:

```rust
pub enum InferenceProofState {
    Established,
    Assumed,
    Unknown(UnknownReason),
    Dynamic(DynamicReason),
}
```

This is not a replacement for `TypeKnowledge`.

It is solver-local epistemic metadata used before a result `TypeId` is materialized.

The meet law mirrors Technical 01 required-premise composition:

```text
Unknown(reason) + anything
    -> Unknown(joined exact reason)

Dynamic(reason) + Known
    -> Dynamic(reason)

Dynamic(a) + Dynamic(b)
    -> Dynamic(join(a, b))

Assumed + Established
    -> Assumed

Established + Established
    -> Established
```

Unknown outranks Dynamic when both are required premises, matching the existing required-composition law.

A missing proof state for a return-influencing variable means:

```text
the solver may have selected a type,
but no value premise justifies publishing it as a runtime value fact
```

and is projected as:

```text
Unknown(UnderconstrainedTypeVariable)
```

unless a more specific recorded Unknown/Dynamic state exists.

---

# 9. New and modified semantic types

## 9.1 `InferenceProofState`

Add in:

```text
phalcom-semantic/src/checker/inference.rs
```

Conceptual API:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InferenceProofState {
    Established,
    Assumed,
    Unknown(UnknownReason),
    Dynamic(DynamicReason),
}

impl InferenceProofState {
    pub fn from_knowledge(knowledge: &TypeKnowledge) -> Self;
    pub fn meet(self, other: Self) -> Self;
}
```

The meet must reuse the same deterministic Unknown/Dynamic reason-join policy as `types/evidence.rs`.

Do not duplicate a second reason precedence table in `inference.rs`.

## 9.2 `RequiredInferencePremise`

Add conceptually:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequiredInferencePremise {
    pub term: InferenceTerm,
    pub origin: ConstraintOrigin,
    pub proof: InferenceProofState,
    pub explanation: Option<ExplanationId>,
}
```

A required premise is recorded for every successfully bound generic argument relation, regardless of whether the argument is Known, Unknown, or Dynamic.

It is not itself necessarily a solver constraint.

## 9.3 Per-variable proof summary

Each inference representative must retain the meet of required proof states that can influence it.

This may be stored directly on `InferenceVariable`:

```rust
pub proof: Option<InferenceProofState>
```

or in an equivalent representative-indexed table.

The key law is semantic, not storage-specific.

Aliases must merge proof state.

Directed subtype dependency propagation must propagate proof state when one variable's value-supported bound is used to solve another variable.

## 9.4 `InferenceSession` APIs

Add APIs equivalent to:

```rust
pub fn record_required_premise(
    &mut self,
    term: &InferenceTerm,
    origin: ConstraintOrigin,
    knowledge: &TypeKnowledge,
    explanation: Option<ExplanationId>,
);

pub fn proof_state_for_term(
    &self,
    term: &InferenceTerm,
) -> InferenceProofState;
```

`record_required_premise(...)`:

1. converts full `TypeKnowledge` to `InferenceProofState`;
2. records the explicit premise for explanation/debugging;
3. applies its proof state to every inference variable occurring in the parameter term;
4. does **not** invent a solver constraint for Unknown/Dynamic.

`proof_state_for_term(...)`:

1. finds return-influencing variables by representative;
2. returns `Established` if the term contains no inference variables;
3. meets every influencing variable's proof summary;
4. treats an unproved influencing variable as `Unknown(UnderconstrainedTypeVariable)`.

## 9.5 Directed subtype edges

Add a solver-local representation equivalent to:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct InferenceSubtypeEdge {
    sub: InferVarId,
    sup: InferVarId,
}
```

For unresolved variables:

```text
Subtype(Var(T), Var(U))
```

records:

```text
T -> U
```

rather than aliasing the representatives.

The solver propagates:

```text
lower(T) -> lower(U)
upper(U) -> upper(T)
```

and checks solved endpoints with the real subtype relation.

Where a propagated bound derives from value-supported variable state, the corresponding support/proof summary propagates monotonically along the dependency actually used.

## 9.6 Generic application result

`checker/call.rs` should make the substitution/proof separation visible.

Use an internal carrier equivalent to:

```rust
struct GenericApplicationResult {
    knowledge: TypeKnowledge,
    outcome: InferenceOutcome,
    proof: InferenceProofState,
}
```

Exact field layout is flexible.

What is not flexible is returning a bare materialized `TypeId` from the generic engine and reconstructing proof authority afterward.

## 9.7 Controlled solving

Add a bounded production entry point equivalent to:

```rust
InferenceSession::solve_with_control(
    store,
    hierarchy,
    control,
) -> InferenceOutcome
```

or a `CheckingContext::solve_inference(...)` wrapper delegating to it.

The existing unbounded/test convenience `solve(...)` may remain only if it delegates to the same implementation with an explicit default control.

There must not be two solver algorithms.

## 9.8 Internal failure outcome

Add:

```rust
InferenceOutcome::InternalFailure(...)
```

or an equivalent explicit path for malformed solver invariants such as missing variable metadata.

At the call layer, convert it to:

```text
InternalSemanticIncidentKind::InferenceInvariantViolation
AnalysisStatus::InternalFailure(id)
```

not `ArgumentMismatch`.

---

# 10. Canonical generic application algorithm

## 10.1 Phase A — bind arguments once

Generic application calls Technical 02:

```rust
bind_static_arguments(arguments, &target.signature.parameters)
```

before generating generic constraints.

Use the resulting `ArgumentBindingPlan`.

Do not repeat positional/labeled matching.

If the binding plan contains static shape failures:

1. emit the canonical shape diagnostics;
2. analyze every supplied argument exactly once, in source order;
3. do not publish a partial generic specialization;
4. an independently fixed return may survive the invalid call;
5. a return depending on generic variables becomes `Unknown(InferenceBlocked)` or the more specific available proof state.

## 10.2 Phase B — instantiate generic variables

Instantiate the callable generic signature once:

```text
TypeParameterId -> InferenceTerm::Var
```

using the canonical kind stored for every type parameter.

Declared generic constraints become ordinary solver constraints with `ConstraintOrigin::GenericWhere`.

They do not seed value support or proof authority.

## 10.3 Phase C — analyze bound arguments

For each source argument in source order:

1. find its bound parameter from `ArgumentBindingPlan`;
2. convert the parameter type to an inference term;
3. propagate that term through `ExpectedType::Inference`;
4. analyze the argument exactly once;
5. record causal invalidity/explanation/status into call capture exactly as the non-generic path does;
6. record the argument as a `RequiredInferencePremise`;
7. if `Known`, add the subtype constraint with `InferenceSupport`;
8. if `Unknown`, add no fake type constraint;
9. if `Dynamic`, add no formal type constraint.

This produces two parallel products:

```text
solver constraints
proof-premise coverage
```

## 10.4 Phase D — project unavailable premises to call status

For a required generic argument:

```text
Unknown(reason)
```

means the argument/parameter relation cannot be proven.

The call records:

```text
AnalysisStatus::Blocked(
    BlockReason::UnknownType(reason)
)
```

while preserving the argument's exact knowledge.

For:

```text
Dynamic(reason)
```

the call records:

```text
AnalysisStatus::DynamicBoundary(reason)
```

Known Established/Assumed arguments can remain Ready if their relations are proven.

This status projection is orthogonal to result knowledge.

Therefore a fixed return may legally produce:

```text
Established(Int)
+
Blocked(UnknownType(UnresolvedName("missing")))
```

## 10.5 Phase E — solve value and declared constraints first

Run the solver **before** expected-result selection is allowed to influence it.

This first solve answers:

```text
What does value evidence + declaration constraints establish/select?
```

If it is terminally:

```text
Conflicting
Blocked
Cancelled
BudgetExceeded
InternalFailure
```

do not add expected-result constraints afterward.

Expected context cannot rescue a prior contradiction or execution failure.

## 10.6 Phase F — compute pre-context result proof

After the first solve:

- if the return is inference-independent and fixed, retain the fixed return;
- if the return depends on inference variables and all necessary variables solved, compute `proof_state_for_term(return_term)`;
- materialize a generic result only when the proof state is `Established` or `Assumed`;
- if proof state is Unknown or Dynamic, publish that state rather than the selected `TypeId`.

This produces a result that is independent of expected-result context when value evidence was already sufficient.

## 10.7 Phase G — add expected-result selection constraint

Only after Phase E is non-terminal may an expected proper type contribute:

```text
return_term <: expected_type
```

with:

```text
ConstraintOrigin::ExpectedResult
support = None
required proof premise = none
```

Expected context may change the selected substitution.

It must not mutate `InferenceSupport`.

It must not mutate `InferenceProofState`.

## 10.8 Phase H — solve contextual selection

Solve again with the expected-result constraint.

Possible consequences:

### Compatible expected result

Retain the precise value-supported result.

Example:

```phalcom
let x: Number = identity(42)
```

The call may remain:

```text
Established(Int)
```

while the binding validates:

```text
Int <: Number
```

### Expected-only solution

Example:

```phalcom
make<T>() -> T

let x: Int = make()
```

The solver may select:

```text
T = Int
```

but no value premise supports return-relevant `T`.

Therefore:

```text
knowledge = Unknown(UnderconstrainedTypeVariable)
```

The result must not become `Established(Int)`.

### Expected context plus unresolved argument

Example:

```phalcom
id<T>(missing) -> T
```

under expected `Int`.

Even if expected context selects `T = Int`, the proof state retains:

```text
Unknown(UnresolvedName("missing"))
```

and that exact reason wins.

### Expected contradiction after independently proven result

If value evidence already established:

```text
identity("wrong") -> String
```

and context expects `Int`, the actual `String` result remains available as knowledge while the contextual relation owns invalidity.

Do not replace it with `Int`.

## 10.9 Phase I — publish result authority

For a solved return term:

```text
no return inference variables
    -> independent fixed-return path

proof = Established
    -> Established(materialized, GenericInference)

proof = Assumed
    -> Assumed(materialized, GenericInference)

proof = Unknown(reason)
    -> Unknown(reason)

proof = Dynamic(reason)
    -> Dynamic(reason)
```

After this generic derivation, Technical 02's outer call-premise authority cap still applies.

Thus an assumed callable-valued premise cannot become Established merely because generic argument proof is Established.

---

# 11. Exact semantic laws

## 11.1 Solvability is not proof

```text
Solved(substitution)
≠
Established(result)
```

A solved substitution is necessary for a specialized generic result but is not sufficient.

## 11.2 Required premise completeness

For every bound generic argument relation:

```text
parameter term P<T...>
argument knowledge K
```

the proof system records K even when:

```text
K.ty() == None
```

Forbidden:

```rust
if let Some(ty) = argument.knowledge.ty() {
    // only then remember the argument existed
}
```

## 11.3 Known support law

```text
Established argument
    -> Established support

Assumed argument
    -> Assumed support
```

No operation upgrades Assumed to Established.

## 11.4 Unknown law

```text
Unknown(reason)
```

contributes:

```text
no solver TypeId constraint
+
required proof state Unknown(reason)
```

The exact reason is preserved.

## 11.5 Dynamic law

```text
Dynamic(reason)
```

contributes:

```text
no formal solver TypeId constraint
+
required proof state Dynamic(reason)
+
DynamicBoundary(reason) call status
```

Dynamic is not Unknown and is not Established.

## 11.6 Return-influence law

Only premises that influence return variables weaken generic return authority.

Example:

```phalcom
fixed<T>(_ value: T) -> Int
```

An Assumed `value` does not weaken the exact fixed `Int` return.

Likewise an Unknown `value` blocks proof of the call argument relation but does not erase the independently fixed `Int` result.

## 11.7 Compound return law

For:

```phalcom
pair<A, B>(_ a: A, _ b: B) -> (A, B)
```

result proof is the meet of A and B proof states.

Examples:

```text
Established + Established -> Established
Established + Assumed     -> Assumed
Established + Unknown(R)  -> Unknown(R)
Assumed + Dynamic(D)      -> Dynamic(D)
Unknown(R) + Dynamic(D)   -> Unknown(R)
```

## 11.8 Alias law

When inference variables become equivalent:

```text
T == U
```

their proof/support summaries merge monotonically.

An alias must never discard weaker evidence.

## 11.9 Subtype direction law

```text
T <: U
```

must remain a directed relation.

It must not become:

```text
T == U
```

merely because both sides are unresolved variables.

Constraint insertion order must not change the semantic outcome.

## 11.10 Expected-result law

Expected result context may constrain substitution selection.

It is never value evidence.

Therefore it must not call:

```text
add_constraint_with_support(... Established ...)
```

and must not call:

```text
record_required_premise(...)
```

## 11.11 Expected context cannot rescue contradiction

If value/declaration constraints are already `Conflicting`, expected context is not added in an attempt to choose a different solution.

## 11.12 Fixed-return independence

For:

```phalcom
foo<T>(_ value: T) -> Int
```

a generic terminal failure may still preserve:

```text
knowledge = fixed Int
status = Invalid / Blocked / Cancelled / BudgetExceeded / InternalFailure
```

when exact callable authority independently establishes `Int`.

## 11.13 No partial specialization after terminal failure

For:

```phalcom
pair<T, U>(...) -> Result<T, U>
```

if inference terminates before the complete dependent return proposition is established, do not publish:

```text
Result<Int, U>
```

as formal type knowledge.

Phalcom does not currently have a separate public partial-type semantic product.

## 11.14 Argument shape law

A generic call cannot solve against a different source-to-parameter mapping than the non-generic canonical application engine.

One `ArgumentBindingPlan` is authoritative.

## 11.15 Child analysis completeness

Every supplied argument expression is analyzed exactly once in source order even when:

```text
shape invalid
premise unknown
premise dynamic
solver conflict
```

## 11.16 Causality law

A generic result may be independently known while carrying non-clean causal invalidity.

Do not derive epistemic status from causal cleanliness.

## 11.17 Cancellation law

If shared checker cancellation is observed before solving completes:

```text
InferenceOutcome::Cancelled
```

must be produced.

Do not convert it to conflict, underconstrained, or blocked.

## 11.18 Budget law

If shared query budget is exhausted during generic solving:

```text
InferenceOutcome::BudgetExceeded(report)
```

must be produced with the real report.

Do not use the fixed-point pass limit as a substitute for query budget.

## 11.19 Fixed-point law

Failure to converge under the solver's bounded fixed-point strategy remains:

```text
Blocked(RecursiveFixpoint)
```

or an equivalent exact block reason.

Do not guess a substitution.

## 11.20 Internal failure law

Malformed solver metadata or an impossible post-`Solved` materialization state is an analyzer invariant failure.

It becomes:

```text
InternalSemanticIncidentKind::InferenceInvariantViolation
```

not a source-owned `ArgumentMismatch`.

---

# 12. Result matrix

| Solver state | Return dependency | Proof state | Published knowledge | Call status |
|---|---|---|---|---|
| Solved | fixed/no return vars | — | fixed known result | accumulated operation status |
| Solved | dependent | Established | Established(materialized) | accumulated status |
| Solved | dependent | Assumed | Assumed(materialized) | accumulated status |
| Solved | dependent | Unknown(R) | Unknown(R) | Blocked when R comes from required unavailable argument; otherwise existing contextual policy |
| Solved | dependent | Dynamic(D) | Dynamic(D) | DynamicBoundary(D) |
| Underconstrained | fixed | — | fixed known result | Blocked/Ready according to independent call proof |
| Underconstrained | dependent | — | Unknown(UnderconstrainedTypeVariable) | Blocked |
| Conflicting | fixed | — | fixed known result | Invalid(C) |
| Conflicting | dependent | — | Unknown(InferenceConflict) | Invalid(C) |
| Blocked | fixed | — | fixed known result | Blocked(real reason) |
| Blocked | dependent | — | Unknown(InferenceBlocked) | Blocked(real reason) |
| Cancelled | fixed | — | fixed known result | Cancelled |
| Cancelled | dependent | — | Unknown(InferenceCancelled) | Cancelled |
| BudgetExceeded | fixed | — | fixed known result | BudgetExceeded(report) |
| BudgetExceeded | dependent | — | Unknown(InferenceBudgetExceeded) | BudgetExceeded(report) |
| InternalFailure | fixed | — | fixed known result if independently valid | InternalFailure(id) |
| InternalFailure | dependent | — | Unknown(InferenceBlocked) | InternalFailure(id) |

A result independently proven before a second expected-context solve may likewise survive a terminal failure in that later contextual-selection phase.

---

# 13. Migration rules

## 13.1 Keep Technical 02 outer architecture

Do not replace:

```rust
CallableApplicationTarget
CallPremise
ApplicationArgument
ArgumentBindingPlan
apply_resolved_callable(...)
```

Technical 03 modifies only the generic sub-engine behind that boundary.

## 13.2 Delete generic argument rematching

Remove the manual generic-only:

```text
positional_idx
label search
parameter_index Option
continue on missing mapping
```

Use `ArgumentBindingPlan`.

## 13.3 Record proof before filtering to `TypeId`

The correct order is:

```rust
record_required_premise(&parameter_term, &argument.knowledge, ...);

match &argument.knowledge {
    TypeKnowledge::Known(_) => add solver relation,
    TypeKnowledge::Unknown(_) => no solver relation,
    TypeKnowledge::Dynamic(_) => no solver relation,
}
```

Never filter first and attempt to reconstruct missing evidence later.

## 13.4 Separate first solve from expected solve

Keep two conceptual stages:

```text
value/declaration solve
expected-selection solve
```

A result proven before expected context enters must remain distinguishable from a result selected only because of context.

## 13.5 Restrict terminal fallback

The generic terminal helper may preserve only:

1. an inference-independent fixed return; or
2. a complete result already proven before a later contextual-selection phase.

It may not accept arbitrary partially specialized knowledge as fallback.

## 13.6 Preserve Technical 04 boundary

Do not “fix” enclosing receiver/class substitution opportunistically in this slice.

If a current test exposes:

```text
Box<T>.method<U> where U <: T
```

with unspecialized enclosing `T` inside method constraints, record it for Technical 04 unless it prevents Technical 03 from preserving proof honestly.

Technical 03 may return Unknown/Blocked rather than inventing a specialization.

---

# 14. Forbidden implementation patterns

The following are forbidden after this slice.

## 14.1 Missing-premise omission

```rust
if let Some(ty) = argument.knowledge.ty() {
    add_constraint(ty);
}
// no semantic record of the else branch
```

## 14.2 Expected-context laundering

```rust
add_constraint_with_support(
    expected_relation,
    InferenceSupport::Established,
)
```

Expected context is not value support.

## 14.3 Generic-only argument binder

```rust
for arg in args {
    if labeled { search parameters }
    else { positional_idx += 1 }
}
```

when `ArgumentBindingPlan` already exists.

## 14.4 Subtype-as-equality fallback

```rust
Subtype(Var(a), Var(b))
    => unify_terms(a, b)
```

## 14.5 Arbitrary terminal fallback

```rust
terminal_generic_return(
    outcome,
    partially_specialized_result,
)
```

unless that result was independently and completely proven before the terminal phase.

## 14.6 Sentinel proof evidence

Do not manufacture:

```text
Never
Unit
Object
InferVarId(0)
Established
```

to fill missing generic proof data.

## 14.7 Unknown reason collapse

Do not rewrite:

```text
Unknown(UnresolvedName("x"))
```

to:

```text
Unknown(InferenceBlocked)
```

when the exact required-premise reason is available and return-influencing.

## 14.8 Dynamic collapse

Do not rewrite Dynamic into Unknown merely because the solver cannot consume a `TypeId`.

## 14.9 Constraint-order semantic dependence

Equivalent sets of constraints may differ in work order, but not in final semantic meaning.

---

# 15. Diagnostics and explanations

Generic diagnostics should use the structured provenance already present in:

```rust
ConstraintOrigin
InferenceConflict
RequiredInferencePremise
```

At minimum, an explanation for a generic failure must be able to identify:

```text
call expression
argument expression, when applicable
parameter index
generic where-constraint index, when applicable
inference variable
actual failed lower/upper relation
whether value support was Established or Assumed
whether expected-result context participated
which return variables were proof-relevant
```

For `ConstraintOrigin::Argument`, prefer the argument source range as the primary diagnostic range when the expression product is available.

For `ConstraintOrigin::GenericWhere`, keep the call as the owning source site if the generic constraint declaration does not yet carry a canonical source-site attachment. Do not invent a source range.

For `ExpectedResult`, preserve the value-supported call result and ensure the contextual contradiction is not rendered as evidence that the call produced the expected type.

Internal inference invariant failures are analyzer incidents, not user type diagnostics.

---

# 16. Required regression families

## 16.1 Solver-level

Required tests:

1. fresh variable kind preservation;
2. occurs-check failure;
3. kind mismatch;
4. alias support weakening;
5. alias proof-state weakening;
6. lower/upper bound solving;
7. actual second/later failed upper bound;
8. structural mismatch;
9. underconstrained variable;
10. required Unknown premise survives despite a solvable substitution from another constraint;
11. required Dynamic premise survives despite a solvable substitution;
12. expected-only constraint can solve selection but proof remains unsupported;
13. `Subtype(Var, Var)` remains directed;
14. subtype-variable result is permutation-stable under constraint insertion order;
15. cancellation yields `Cancelled`;
16. budget exhaustion yields `BudgetExceeded`;
17. non-convergence remains Blocked;
18. missing variable metadata uses internal-failure semantics rather than source conflict.

## 16.2 Call-level

Required tests:

1. Established argument -> Established generic return;
2. Assumed argument -> Assumed generic return;
3. weakest return-variable support wins;
4. assumed non-return variable does not weaken fixed return;
5. unresolved return-relevant argument prevents known result even when another argument solves the same variable;
6. exact unresolved reason is preserved;
7. unresolved non-return variable blocks the call but does not erase fixed return;
8. Dynamic return-relevant argument produces Dynamic result;
9. Dynamic non-return variable leaves fixed result known but marks DynamicBoundary;
10. expected result cannot upgrade an unresolved required premise;
11. expected-only inference cannot fabricate generic result evidence;
12. expected context does not widen/replace independently precise result;
13. expected context cannot rescue value/declaration conflict;
14. shape failure does not produce partial generic specialization;
15. every supplied argument is analyzed exactly once on generic shape failure;
16. generic conflict retains real argument/constraint provenance;
17. dependent conflict publishes Unknown rather than partial type;
18. fixed result survives conflict;
19. fixed result survives blocked/cancel/budget;
20. callable/receiver outer authority still caps generic result through Technical 02.

---

# 17. Acceptance gates

Technical Specification 03 is complete only when all of the following are true.

## Gate 03-A — Required premise completeness

A repository search and regressions establish that no generic result proof depends solely on `knowledge.ty()` filtering.

For every bound generic argument, Known/Unknown/Dynamic state is accounted for.

## Gate 03-B — Solvability/proof separation

A test equivalent to:

```phalcom
first<T>(1, unresolved)
```

does not publish Established/Assumed `T` merely because `1` solves the substitution.

## Gate 03-C — Exact reason preservation

The result retains the exact return-influencing `UnknownReason` or `DynamicReason`.

## Gate 03-D — Canonical argument binding

Generic application consumes Technical 02's `ArgumentBindingPlan`.

There is no second production generic-only argument matcher.

## Gate 03-E — Expected context safety

Expected-result constraints can select substitutions but never seed support/proof authority.

Expected-only solving cannot create a known generic result.

## Gate 03-F — Return authority

Established/Assumed generic result authority is determined only from return-influencing value evidence and Technical 02's outer callee/receiver authority.

## Gate 03-G — Fixed-result independence

Independent fixed returns survive unrelated generic invalidity/blockage/cancellation/budget exhaustion with the correct sibling status.

## Gate 03-H — No partial specialization

No terminal generic failure publishes a partially substituted dependent return as formal type knowledge.

## Gate 03-I — Relation integrity

`Subtype(Var, Var)` remains directed and constraint-order permutations yield equivalent solver outcomes.

## Gate 03-J — Terminal outcome fidelity

Production generic solving can actually observe shared cancellation and query budget and publishes exact terminal statuses.

## Gate 03-K — Internal failure integrity

Malformed solver invariants use `InferenceInvariantViolation` incidents rather than user type diagnostics.

## Gate 03-L — Regression closure

Focused solver tests, call-level correctness tests, generic capability tests, semantic integration tests, formatting, and structural scans all pass, subject only to separately documented unrelated repository failures.

---

# 18. Structural audit queries

Before closure, run repository searches equivalent to:

```bash
rg -n \
  'knowledge\.ty\(\).*add_constraint|if let Some\(.*knowledge\.ty\(\)' \
  phalcom-semantic/src/checker/call.rs

rg -n \
  'positional_idx|position\(\|p\|.*external_label' \
  phalcom-semantic/src/checker/call.rs

rg -n \
  'Subtype.*unify_terms|subtype_terms' \
  phalcom-semantic/src/checker/inference.rs

rg -n \
  'ExpectedResult.*add_constraint_with_support|ExpectedResult.*record_required_premise' \
  phalcom-semantic/src

rg -n \
  'terminal_generic_return' \
  phalcom-semantic/src/checker/call.rs
```

Every remaining match must be reviewed.

The desired interpretation is not necessarily zero textual matches; it is zero semantic occurrences of the forbidden patterns.

---

# 19. Handoff boundary to Technical Specification 04

Technical 03 finishes when method-local generic inference is honest.

It intentionally does not guarantee that every enclosing receiver/class generic term has already been fully specialized through every nested method constraint.

The next slice may assume:

```text
generic argument mapping is canonical
solver subtype relations are not strengthened
required value premises are complete
substitution solving is distinct from proof
result authority is monotone
expected context is non-evidentiary
terminal outcomes are structured
```

Technical 04 can then focus on:

```text
receiver/class substitution
method parameter specialization
return specialization
method generic bounds
where-clause specialization
nested applied types
higher-kinded/type-lambda structures
```

without needing to reopen generic proof integrity.

---

# 20. Final correctness statement

After Technical Specification 03, Phalcom may claim:

> A generic result is published as known only when the analyzer both solves the required specialization and has complete admissible evidence for the return proposition. Missing, dynamic, assumed, contextual, conflicting, blocked, cancelled, budget-exhausted, and internally malformed paths remain semantically distinct.

That is the correctness foundation required before improving generic inference completeness.

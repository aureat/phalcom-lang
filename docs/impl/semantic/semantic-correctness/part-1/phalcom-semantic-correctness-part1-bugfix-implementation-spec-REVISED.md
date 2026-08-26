# Phalcom Semantic Correctness Part 1 — Correctness Repair Implementation Specification (Revised)

> **Status:** Implementation specification for the remaining semantic-correctness defects identified during the Part 1 code review.
>
> **Target repository:** `aureat/phalcom-lang`
>
> **Verified implementation baseline:** GitHub `main` re-reviewed through commit `9b5873025b47dc7addb826f165530391fa93e171` (`Merge PR #4: fix semantic dispatch and test oracle corrections`, 2026-08-25). The canonical semantic target reported at that revision is 362 passed, 0 failed, 10 intentionally ignored; this document still requires every new regression to be observed RED before its fix.
>
> **Execution discipline:** Test-driven. Every production behavior change in this document begins with a regression that fails for the intended semantic reason on the implementation baseline.

---

# 1. Goal

This specification repairs the remaining defects in Phalcom's Part 1 formal semantic epistemic foundation without redesigning the type system or replacing the existing checker architecture. The objective is to make the current implementation faithfully preserve the semantic dimensions already defined by the Part 1 foundation, its Corrections and Amendments, and the implementation-level semantic analyzer specification set.

The repaired analyzer must preserve, end-to-end, the distinction between:

- type knowledge and analysis completion;
- established and assumed evidence;
- unknown and dynamic knowledge;
- causal invalidity and suppression;
- persistent binding contracts and current flow facts;
- successful relation proof and non-success terminal relation outcomes;
- mathematical generic substitution and the epistemic support of that substitution;
- semantic product identity and incidental allocator/source identity.

The implementation must remain fail-closed. When the analyzer cannot establish a proposition, it must publish an explicit unknown, blocked, dynamic, cancelled, budget-exceeded, or internal-failure state as appropriate. It must not invent a normal language type, silently drop a contract, or reinterpret a terminal result as successful proof.

---

# 2. Normative semantic basis

The following documents define the behavior implemented here, in precedence order:

1. `phalcom_semantic_correctness_part1_corrections_and_amendments.md`;
2. `phalcom_semantic_correctness_single_world_takeover_part1_formal_epistemic_foundation_spec.md`;
3. the implementation-level semantic analyzer specification set:
   - `01-semantic-analysis-model.md`;
   - `02-type-knowledge-and-evidence.md`;
   - `03-analysis-status-causality-and-recovery.md`;
   - `04-expression-analysis-and-contextual-typing.md`;
   - `05-binding-and-flow-analysis.md`;
   - `06-relations-reconciliation-and-semantic-judgments.md`;
   - `07-generic-inference-engine.md`;
   - `08-callable-analysis-and-publication.md`;
   - `09-semantic-products-incrementality-and-fingerprints.md`.

The old Part 1 implementation checklist is implementation history and test inventory. A checked item in that file is not normative evidence that the current source satisfies the semantic contract.

---

# 3. Verified current repository state

The fresh re-review of current `main` confirms that the core defects covered by this repair remain present after the later semantic dispatch/test-oracle work. Several objections raised against the first version of this implementation plan are also correct and materially change the repair architecture; those verified gaps are incorporated below rather than treated as optional follow-up work.

## 3.1 Expression status is still reconstructed from causal invalidity

**File:** `phalcom-semantic/src/checker/expression.rs`

**Anchor:** `analyze_expression`

Current logic conceptually does:

```rust
let status = if let Some(cause_id) = owned_cause {
    AnalysisStatus::Invalid(cause_id)
} else if let Some(cause) = typed.causal_invalidity.suppression_cause() {
    AnalysisStatus::Suppressed(cause)
} else if typed.knowledge.is_dynamic() {
    AnalysisStatus::DynamicBoundary(...)
} else {
    AnalysisStatus::Ready
};
```

This still equates non-clean causal invalidity with suppression.

**File:** `phalcom-semantic/src/checker/typed_expr.rs`

**Anchor:** `TypedExpression`

`TypedExpression` carries `knowledge`, `denotation`, `callable`, constraints, provenance, and `causal_invalidity`, but no first-class `AnalysisStatus`. Therefore nested expression analysis cannot preserve a terminal status directly and publication is forced to reconstruct it later.

## 3.2 Relation enforcement still erases terminal outcomes

**File:** `phalcom-semantic/src/checker/context.rs`

**Anchors:**

- `enforce_assignability`;
- `enforce_knowledge_against_type`;
- `enforce_knowledge_against_type_owned`.

Each helper treats only `Assignability::Refuted` as non-success and returns `true` for every other variant. Consequently `DynamicBoundary`, `Blocked`, `Cancelled`, `BudgetExceeded`, `InternalFailure`, and `Uncertain` are all flattened at these consumer boundaries.

## 3.3 Binding reconciliation still destroys terminal relation meaning

**File:** `phalcom-semantic/src/checker/binding.rs`

**Anchor:** `reconcile_binding_contract`

Current mappings include:

```text
Cancelled       -> Blocked(SuppressedDependency)
BudgetExceeded  -> Blocked(BudgetExceeded(...))
InternalFailure -> Blocked(OpaqueNative(...))
Uncertain       -> Blocked(RecursiveFixpoint)
```

These are not semantically equivalent states.

## 3.4 Normal return summarization still converts non-concrete results to generic Unknown

**File:** `phalcom-semantic/src/checker/analysis.rs`

**Anchor:** `normal_return_summary`

Current logic joins full `TypeKnowledge`, then replaces every joined result without a concrete `TypeId` with `Unknown(UncheckedExpression)`. This destroys `Dynamic` and classified `Unknown` reasons.

## 3.5 Assignment contextual typing still falls back to current flow knowledge

**File:** `phalcom-semantic/src/checker/expression.rs`

**Anchor:** `Expr::Assignment`

After checking the persistent contract, the expected-RHS computation falls back to `flow.get_current_type(...)` and labels that current type as `ExpectationOrigin::AssignmentContract`. A transient current fact can therefore influence a context-sensitive RHS as though it were a persistent assignment contract.

## 3.6 Same-binding contract disagreement still erases the contract at a flow join

**File:** `phalcom-semantic/src/checker/flow/state.rs`

**Anchor:** `FlowState::join_impl`

When incoming states for the same `BindingId` disagree on `BindingContract`, the current implementation selects:

```rust
let contract = if contracts_match {
    sample_binding.contract.clone()
} else {
    None
};
```

It later marks consistency blocked, but the persistent contract itself has already been erased. `None` means unconstrained and is therefore a fail-open representation of an internal invariant violation.

## 3.7 Loop widening still does not reconcile widened current knowledge against the contract

**File:** `phalcom-semantic/src/checker/flow/state.rs`

**Anchor:** `FlowState::widen_loop_state`

The widening path joins `current`, `denotation`, and causal invalidity. It only changes consistency when the incoming consistency values differ; it does not recompute consistency from the widened current fact and persistent contract.

## 3.8 Generic lower/upper reconciliation still reports the wrong upper bound

**File:** `phalcom-semantic/src/checker/inference.rs`

**Anchor:** `InferenceSession::solve`

The solver checks every upper bound, breaks on the first failed one, then constructs:

```rust
InferenceFailureReason::ConflictingBounds {
    var: rep,
    lower: candidate,
    upper: uppers[0],
}
```

If `uppers[1]` or later is the actual failed bound, the published conflict is false.

## 3.9 Generic kind lookup still silently defaults missing solver metadata to `Type`

**File:** `phalcom-semantic/src/checker/inference.rs`

**Anchors:**

- `InferenceSession::bind`;
- variable-variable branch of `unify_terms`.

Both use `unwrap_or(KindId::TYPE)` when the solver cannot find variable metadata. Missing solver metadata is an internal malformed-state condition, not evidence that the variable has kind `Type`.

## 3.10 Call analysis still has no terminal analysis status

**File:** `phalcom-semantic/src/checker/call.rs`

**Anchor:** `CallCheckResult`

The result carries knowledge, causal invalidity, explanation parents, and callable identity, but no analysis status. Generic inference has terminal outcome variants, yet call publication can only encode them indirectly as `UnknownReason` or lose them.

## 3.11 Exact return promotion still flattens return provenance

**File:** `phalcom-semantic/src/checker/call.rs`

**Anchor:** `promote_exact_return`

Every known exact return becomes:

```text
Established(..., CallableSignature)
```

regardless of whether the return fact originates from constructor semantics, trusted native semantics, or another exact formal origin.

## 3.12 Callable body parameters still enter with the wrong epistemic provenance

**Files:**

- `phalcom-semantic/src/checker/body.rs` — `analyze_callable_body`;
- `phalcom-semantic/src/checker/context.rs` — `bind_callable_parameter_with_causal`.

The body passes `sig.parameters[*].ty` directly into binding creation. Because signature parameter type knowledge can still retain `DeveloperAnnotation` origin, body-entry current knowledge can remain developer-origin evidence instead of being reclassified as a callable-contract assumption.

## 3.13 Callable return contract remains represented as `Option<TypeKnowledge>`

**File:** `phalcom-semantic/src/checker/context.rs`

**Anchor:** `CheckingContext::expected_return`

**File:** `phalcom-semantic/src/checker/body.rs`

**Anchor:** body-entry initialization of `expected_return`.

The return contract is context/contract state, but the context stores it as `TypeKnowledge`. This is semantically misleading and makes it easier for contract knowledge to be treated as value evidence.

## 3.14 Binding declaration type is duplicated in semantic state and lexical metadata

**Files:**

- `phalcom-semantic/src/checker/analysis.rs` — `BindingState::declared` and `BindingState::contract`;
- `phalcom-semantic/src/checker/context.rs` — `LocalBindingInfo::declared`;
- `phalcom-semantic/src/checker/flow/state.rs` — compatibility `declare*` APIs and `get_declared_type`.

The same persistent type can be represented independently in multiple places. Today most writes derive the mirror from the contract, but the architecture still permits divergence and keeps a legacy semantic read path alive.

## 3.15 `TypeEvidence` remains unrestricted public construction surface

**File:** `phalcom-semantic/src/types/evidence.rs`

**Anchor:** `TypeEvidence`

All fields remain `pub`, and production code such as pattern decomposition directly constructs `TypeKnowledge::Known(TypeEvidence { ... })`. This means ordinary code can mint `Established` facts without passing through a trusted semantic constructor.

## 3.16 Unknown/Dynamic reason join remains input-order-sensitive

**File:** `phalcom-semantic/src/types/evidence.rs`

**Anchor:** `join_type_knowledge`

The function returns the first encountered `UnknownReason` or `DynamicReason`. The type/epistemic class is conservative, but explanatory reason semantics can depend on incoming iteration order.

## 3.17 Flow summary fingerprinting remains epistemically lossy

**Files:**

- `phalcom-semantic/src/checker/analysis.rs` — `FlowStateSummary`;
- `phalcom-semantic/src/db/fingerprint.rs` — `hash_flow_summary`.

The summary stores only `BindingId -> TypeId` plus `fact_count`. It cannot distinguish `Established(Int)` from `Assumed(Int)`, or other semantic binding-state differences, if that summary is relied upon as a semantic dependency/product boundary.

## 3.18 Diagnostic product hashing still includes raw diagnostic cause allocator identity

**File:** `phalcom-semantic/src/db/fingerprint.rs`

**Anchor:** `hash_semantic_diagnostic`

The fingerprint hashes `diagnostic.root_cause` directly even though analysis-status and causal-invalidity hashing correctly ignore raw cause numbers. This violates the rule that pure cause-ID renumbering is not a semantic change.

---


## 3.19 Bounded relation outcomes exist below the checker but are not threaded into `CheckingContext`

**Files:**

- `phalcom-semantic/src/types/relation.rs`;
- `phalcom-semantic/src/types/outcome.rs`;
- `phalcom-semantic/src/checker/context.rs`;
- `phalcom-semantic/src/checker/body.rs`.

The type-relation layer already exposes bounded APIs using `QueryBudget` and `CancellationToken`:

```text
check_subtype_bounded(..., &mut QueryBudget, &CancellationToken)
check_assignability_bounded(..., &mut QueryBudget, &CancellationToken)
check_knowledge_against_type_bounded(..., &mut QueryBudget, &CancellationToken)
```

`analyze_callable_body` also already receives a real query budget and cancellation token. However, it creates `CheckingContext` without passing either one into the context. The body loop itself checks cancellation and charges statement steps, while `CheckingContext::enforce_assignability` and its sibling helpers call the unbounded compatibility wrappers. Those wrappers create/use independent default relation control and return the compatibility `Assignability` enum.

Consequently a relation performed inside return checking, argument checking, expected-type checking, or binding reconciliation cannot currently consume the callable query's cancellation/budget state. `Cancelled` and `BudgetExceeded` are therefore largely unreachable at the checker consumer boundaries that the first plan intended to test.

**Required repair:** production checker analysis must use the bounded relation APIs with relation control derived from the same callable/query analysis control. A test-only injection seam can supplement this for exhaustive variant testing, but it is not a substitute for wiring real production cancellation and budget semantics.

## 3.20 The bounded relation engine itself collapses nested terminal outcomes

**File:** `phalcom-semantic/src/types/relation.rs`

This is an additional gap discovered while verifying §3.19. Recursive subtype cases currently contain patterns equivalent to:

```rust
let outcome = check_subtype_impl(...);
if !outcome.is_proven() {
    all_ok = false;
    break;
}
...
RelationOutcome::Refuted(...)
```

This occurs in union, tuple, applied-generic, and callable recursion. A nested `BudgetExceeded`, `Cancelled`, or `InternalFailure` can therefore be converted into an ordinary `Refuted` result by the outer structural relation.

Threading the budget/cancellation into `CheckingContext` without fixing this recursion would make terminal states reachable only at shallow relation depths. The first repair slice must therefore make recursive relation composition terminal-preserving before checker integration.

## 3.21 `Assignability::DynamicBoundary` is already a lossy compatibility projection

**File:** `phalcom-semantic/src/types/relation.rs`

The canonical bounded carrier is richer:

```rust
RelationOutcome::DynamicBoundary(DynamicBoundaryObligation { reason })
```

but conversion to `Assignability` currently performs:

```text
RelationOutcome::DynamicBoundary(_) -> Assignability::DynamicBoundary
```

and the `Assignability` variant carries no obligation. The reason is lost before `CheckingContext` sees the result.

The checker repair must therefore use `RelationOutcome` as the canonical internal relation carrier, not `Assignability`. If `Assignability` remains as a public compatibility API, its dynamic variant must also become non-lossy or it must be explicitly documented/deprecated as unsuitable for checker-internal semantic transfer.

The existing `Assignability::is_assignable()` helper is also suspect because it treats `DynamicBoundary` as assignable. A dynamic boundary is not formal proof. All uses must be audited.

## 3.22 Flow join has four production callers and no invariant-failure publication path

**Files:**

- `phalcom-semantic/src/checker/flow/state.rs`;
- `phalcom-semantic/src/checker/expression.rs`;
- `phalcom-semantic/src/checker/statement.rs`.

`FlowState::join_with_hierarchy` currently returns `FlowState`, not `Result`. Current production callers include three expression/control-flow joins in `checker/expression.rs` and the `Statement::For` join in `checker/statement.rs`.

Therefore changing only the flow API to `Result<FlowState, FlowJoinFailure>` is incomplete. The checker needs one semantic publication path for invariant failure, and every caller must use it. An expression-level failure must preserve any independently computed expression knowledge while publishing `AnalysisStatus::InternalFailure`; a statement-level failure must poison/terminate the affected flow path and make callable analysis observe an internal failure rather than continuing with a fabricated state.

## 3.23 Suppression currently has no genuine producer

**Files:**

- `phalcom-semantic/src/checker/expression.rs`;
- `phalcom-semantic/src/checker/context.rs`.

`CheckingContext::mark_suppressed` is currently called only by `analyze_expression`, and there it is called when the current expression owns a diagnostic. `analyze_expression` then separately reconstructs `Suppressed` whenever `typed.causal_invalidity.suppression_cause()` is non-empty.

There is no expression-specific semantic rule today that says: “this child lost a premise because of upstream invalidity, therefore this parent operation is suppressed.” Task 1C in the first plan therefore had no real producer to exercise.

The repair must add an explicit **required-premise propagation rule** at operations such as method/property/index dispatch. A child that is invalid/suppressed and has no usable receiver type because of an owning upstream cause can suppress the dependent operation. A child that is `Ready + Known + causal invalidity` must not suppress it.

## 3.24 Raw diagnostic cause identity affects the module diagnostics product, not callable-body fingerprints

**File:** `phalcom-semantic/src/db/fingerprint.rs`

`hash_semantic_diagnostic` hashes `diagnostic.root_cause`, and `module_diagnostics_product_fingerprint` hashes every diagnostic through that helper. `callable_body_product_fingerprint`, by contrast, does not hash `analysis.diagnostics`.

Therefore the cause-renumbering regression must target `module_diagnostics_product_fingerprint`. Callable-body fingerprint tests remain appropriate for flow epistemic-state changes, but not for raw diagnostic cause-number stability.

The shared diagnostic hashing helper is also used by some query-input fingerprints. The implementation must distinguish semantic diagnostic identity from allocator identity without accidentally changing unrelated input-fingerprint contracts.

## 3.25 Constructor provenance has a stale passing test oracle

**Files:**

- `docs/spec/semantic-analyzer/02-type-knowledge-and-evidence.md`;
- `docs/spec/semantic-analyzer/08-callable-analysis-and-publication.md`;
- `phalcom-semantic/tests/semantic/capabilities/authority.rs`;
- `phalcom-semantic/src/checker/call.rs`;
- `phalcom-semantic/src/dispatch.rs`.

The normative implementation-level semantic specification requires exact `@constructor` results to use `EvidenceOrigin::ConstructorSemantics`. Existing authority tests currently use a helper that asserts `EvidenceOrigin::CallableSignature` for every method call, including `CellNum.new()`, and several constructor-derived binding assertions repeat that expectation.

This is a test-oracle conflict, not evidence that the normative rule should be weakened. The implementation task must first split the test helper so the expected origin is explicit, change constructor-specific oracles to `ConstructorSemantics`, observe the resulting RED failures, and only then change production call-result provenance. Ordinary exact methods must continue to expect `CallableSignature`; generic specialization expects `GenericInference`; trusted native calls expect `NativeSignature` where exposed.

`CallableSignature` currently carries selector, parameters, return knowledge, and generics, but no constructor/native call-kind metadata. The production fix therefore needs an explicit origin source at exact dispatch/call publication rather than guessing from the return type alone.

## 3.26 `checker/policy.rs` duplicates the boolean-laundering defect

**File:** `phalcom-semantic/src/checker/policy.rs`

`handle_relation_outcome(...) -> bool` returns false only for `Refuted` and true for every other `RelationOutcome`. `enforce_assignability(...) -> bool` repeats the same policy after calling the unbounded compatibility relation API.

The relation repair must include this file. Prefer deleting or replacing duplicate policy helpers so there is one structured checker relation-application path. A static audit limited to `CheckingContext::enforce_*` is insufficient.

## 3.27 Disposition of the raised objections

| Objection | Verification | Plan consequence |
|---|---|---|
| terminal outcomes cannot reach checker consumers | **Confirmed** | thread bounded control into `CheckingContext`; test seam is supplementary only |
| dynamic obligation is dropped | **Confirmed** | use `RelationOutcome` internally; make compatibility projection non-lossy |
| flow join `Result` lacked caller publication semantics | **Confirmed** | add context-level invariant-failure publisher and migrate all four production callers |
| no genuine suppression producer | **Confirmed** | implement required-premise suppression at receiver-dependent operations and test a concrete nested-invalid scenario |
| fingerprint 10D targeted wrong product | **Confirmed** | retarget to `module_diagnostics_product_fingerprint` |
| constructor provenance conflicts with tests | **Confirmed** | deliberately change stale constructor test oracle before production change |
| `policy.rs` duplicate boolean helpers omitted | **Confirmed** | include deletion/migration and static audit |
| nested bounded relation recursion preserves terminal outcomes | **Additional defect found** | add relation-engine terminal-preservation tests before checker wiring |


# 4. Architecture of the repair

The repair should be staged around information-preserving boundaries, not around individual bug reports. The revised dependency order is stricter than the first plan because terminal relation results must exist end-to-end before expression/call status can faithfully consume them.

```text
Slice A0 — make the relation engine genuinely terminal-preserving
    recursive RelationOutcome composition
    preserve DynamicBoundaryObligation
    checker analysis control (budget + cancellation)
    remove/deprecate lossy boolean policy adapters

Slice A1 — preserve semantic operation outcomes end-to-end
    TypedExpression.status
    CallCheckResult.status
    structured relation application using RelationOutcome
    binding terminal outcomes
    explicit required-premise suppression
    status/causality orthogonality
    return-summary preservation

Slice B — restore binding and flow invariants
    persistent-contract-only assignment context
    Result-based flow joins
    one invariant-failure publication path
    migrate all expression/statement join callers
    widening reconciliation
    remove declaration mirrors

Slice C — harden inference and provenance
    actual failed upper bound
    no kind fallback
    callable parameter body-entry provenance
    return contract representation
    exact return origin, including constructor test-oracle migration
    trusted evidence construction
    deterministic reason join

Slice D — repair incremental semantic identity
    epistemic flow summaries
    module-diagnostics cause-number-insensitive product fingerprint
    differential/product-stability regressions
```

Do not begin A1 by merely adding a `status` field while relation terminal states are still unrecoverable. Do not start Slice B until expression/call terminal status and the context-level failure publication seam are stable. Do not change semantic fingerprints until the semantic products themselves carry the repaired state.

The implementation should prefer one canonical carrier per semantic domain:

```text
relation engine/checker boundary  -> RelationOutcome<()>
expression/call operation         -> AnalysisStatus + TypeKnowledge + CausalInvalidity
flow structural validity          -> Result<FlowState, FlowJoinFailure>
binding contract relation         -> BindingConsistency plus terminal relation identity where needed
```

Compatibility wrappers may remain for external callers, but checker core must not use a lossy compatibility carrier.

# 5. Target internal result model

## 5.1 `TypedExpression` must carry analysis status

Modify:

`phalcom-semantic/src/checker/typed_expr.rs`

Target conceptual representation:

```rust
pub struct TypedExpression {
    pub expression_id: Option<ExpressionId>,
    pub callable: Option<CallableId>,
    pub explanation_parents: Vec<ExplanationId>,
    pub knowledge: TypeKnowledge,
    pub status: AnalysisStatus,
    pub denotation: Option<SemanticDenotation>,
    pub dispatch_lookup: DispatchLookup,
    pub constraints: Vec<TypeConstraint>,
    pub provenance: EvidenceSet,
    pub causal_invalidity: CausalInvalidity,
}
```

Constructors `new`, `established`, `assumed`, `unknown`, and `dynamic` should initialize semantically appropriate status:

```text
Known/Unknown ordinary synthesis -> Ready
Dynamic(...)                     -> DynamicBoundary(the actual dynamic reason)
```

However, callers must be able to override status when the operation is invalid, suppressed, blocked, cancelled, budget-exceeded, or internally failed.

Add a builder/mutator such as:

```rust
pub fn with_status(mut self, status: AnalysisStatus) -> Self
```

and ensure `From<ExpressionAnalysis> for TypedExpression` preserves `analysis.status`.

Do **not** derive status from causal invalidity in this conversion.

## 5.2 Expression publication becomes preservation, not reconstruction

Modify:

`phalcom-semantic/src/checker/expression.rs`

`analyze_expression` should determine only ownership-local changes after `analyze_expression_inner` returns.

Target algorithm:

```text
inner returns TypedExpression { knowledge, status, invalidity, ... }
        ↓
if this expression owner acquired an owning diagnostic:
    status = Invalid(owning cause)
    invalidity += owning cause
else:
    retain inner.status
        ↓
record ExpressionAnalysis with typed.status
```

Critically, delete the rule:

```text
non-clean causal invalidity -> Suppressed
```

Suppression must be assigned only at the semantic operation that actually loses a required premise.

`typed.knowledge.is_dynamic()` should not be used as a generic late publication heuristic if expression-specific analysis has already chosen a status. Dynamic constructors or dynamic dispatch paths should set the appropriate status when the dynamic boundary occurs.

## 5.3 Call results must preserve status

Modify:

`phalcom-semantic/src/checker/call.rs`

Target:

```rust
pub struct CallCheckResult {
    pub knowledge: TypeKnowledge,
    pub status: AnalysisStatus,
    pub causal_invalidity: CausalInvalidity,
    pub explanation_parents: Vec<ExplanationId>,
    pub callable: Option<CallableId>,
}
```

Prefer changing `resolve_call_inner` from returning bare `TypeKnowledge` to returning an internal call outcome containing at least `knowledge + status`.

Suggested internal type:

```rust
struct CallSemanticOutcome {
    knowledge: TypeKnowledge,
    status: AnalysisStatus,
}
```

`resolve_call` then adds causal capture/explanation/callable identity.

Do not encode cancellation or budget exhaustion solely as `UnknownReason::InferenceCancelled` / `InferenceBudgetExceeded`. Those reasons may remain useful as type-knowledge detail when the return itself is unavailable, but `AnalysisStatus` must still carry the terminal operational state.

---

# 6. Structured relation execution and application

## 6.1 Repair recursive bounded-relation composition first

Modify:

`phalcom-semantic/src/types/relation.rs`

Before wiring bounded outcomes into the checker, audit every recursive `check_subtype_impl` composition that currently tests only `is_proven()`.

For conjunction-like structural checks (source union members, tuple elements, callable parameters, same-origin applied arguments):

```text
Proven             -> continue
Refuted            -> produce the enclosing relation's real refutation
DynamicBoundary    -> propagate DynamicBoundary obligation
Blocked            -> propagate Blocked
Cancelled          -> propagate Cancelled
BudgetExceeded     -> propagate BudgetExceeded
InternalFailure    -> propagate InternalFailure
```

For disjunction-like checks such as checking a type against members of a target union:

```text
Proven member      -> overall Proven
Refuted member     -> continue to the next member
terminal outcome   -> do not reinterpret as Refuted; preserve according to the relation algorithm's fail-closed policy
all members Refuted-> overall Refuted
```

A cancellation or exhausted budget inside the second tuple element must not emerge as `TypeMismatch` for the entire tuple.

Add low-budget and pre-cancelled direct relation tests before changing checker context. These tests are required because otherwise later checker tests may falsely appear to validate end-to-end propagation while only exercising shallow relations.

## 6.2 Make `RelationOutcome` the checker-internal carrier

The bounded relation APIs already return the semantically complete carrier:

```rust
RelationOutcome<T> {
    Proven { value, evidence },
    Refuted(RelationFailure),
    DynamicBoundary(DynamicBoundaryObligation),
    Blocked(BlockReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(String),
}
```

Use this carrier directly inside `checker/*`.

Do **not** define the first plan's `RelationApplication { outcome: Assignability, ... }`, because `Assignability` is already lossy at `DynamicBoundary`.

Use a small checker application wrapper only for checker-owned side effects:

```rust
pub struct RelationApplication {
    pub outcome: RelationOutcome<()>,
    pub cause: Option<DiagnosticCauseId>,
}
```

`cause` is populated only when this semantic judgment owns a source diagnostic, normally on `Refuted`.

## 6.3 Thread real query budget and cancellation into `CheckingContext`

Modify:

- `phalcom-semantic/src/checker/body.rs`;
- `phalcom-semantic/src/checker/context.rs`.

Current body analysis owns `QueryBudget` and receives `&CancellationToken`, but the context is constructed without them. Production checking must stop calling unbounded relation wrappers.

Introduce a checker analysis-control object or semantically equivalent fields. Recommended conceptual shape:

```rust
#[derive(Clone)]
pub struct CheckerControl {
    budget: SharedQueryBudget,
    cancellation: CancellationToken,
}
```

`SharedQueryBudget` may be implemented as a context-owned budget with controlled mutable access, or as a small shared mutable cell if nested `with_resolver` contexts must consume one budget. The exact container is non-normative. The invariants are:

1. the callable query has one logical relation/step budget, not a fresh default budget for every relation;
2. nested checker contexts share/continue that budget rather than resetting counters;
3. cancellation is the actual query token, not a newly created never-cancelled token;
4. low-level/direct checker tests can construct a default control explicitly;
5. tests can construct a deliberately tiny budget or cancelled token without mocking the whole analyzer.

Move the body loop's `charge_step`/cancellation checks behind context/control methods if necessary so the same budget can be owned by the context without aliasing mutable borrows.

Recommended production construction:

```text
analyze_callable_body receives budget + cancel
        ↓
CheckingContext receives/shares CheckerControl
        ↓
body statement charging and relation checking consume the same logical control
        ↓
check_*_bounded returns terminal RelationOutcome directly
```

## 6.4 Bounded enforcement APIs return structured applications

Replace the boolean APIs in `checker/context.rs`:

```text
enforce_assignability(...) -> bool
enforce_knowledge_against_type(...) -> bool
enforce_knowledge_against_type_owned(...) -> bool
```

with structured operations that call the bounded relation functions using context control:

```text
apply_assignability(...) -> RelationApplication
apply_knowledge_against_type(...) -> RelationApplication
apply_knowledge_against_type_owned(...) -> RelationApplication
```

Naming may follow repository conventions; the contract is normative.

On `Refuted`, the helper may allocate the owning semantic diagnostic and return its cause. On every other terminal outcome it emits no fake type-mismatch diagnostic and returns the terminal result unchanged.

The `_owned` form updates the already-published owning expression when the relation runs after initial expression synthesis. It must be able to set:

```text
Invalid(cause)
DynamicBoundary(obligation-derived reason)
Blocked(reason)
Cancelled
BudgetExceeded(report)
InternalFailure(incident)
```

without changing independently known type knowledge.

## 6.5 Preserve dynamic-boundary obligations

Modify `types/relation.rs` compatibility projection as part of this slice.

Preferred:

```rust
Assignability::DynamicBoundary(DynamicBoundaryObligation)
```

and conversion:

```text
RelationOutcome::DynamicBoundary(obligation)
    -> Assignability::DynamicBoundary(obligation)
```

If changing the public enum immediately is judged too disruptive, checker core still must not use that projection. In that transitional case, mark the compatibility adapter as lossy/deprecated and add a follow-up gate. The final Part 1 semantic surface should be non-lossy.

Also change/audit `Assignability::is_assignable()`: only actual proof may return true. A dynamic boundary is not static assignability proof.

## 6.6 Remove duplicate boolean policy laundering

Modify or delete:

`phalcom-semantic/src/checker/policy.rs`

The current `handle_relation_outcome(...) -> bool` and `enforce_assignability(...) -> bool` duplicate the same “anything except Refuted succeeds” defect.

Preferred outcome: one structured relation-application implementation in `CheckingContext`/a focused checker relation-policy module, and no parallel boolean policy API.

If `policy.rs` remains, it must return a structured policy result and preserve every terminal variant including `DynamicBoundaryObligation`.

## 6.7 Consumer mapping

Every checker consumer must explicitly interpret `RelationOutcome`:

| Outcome | Required checker consequence |
|---|---|
| `Proven` | relation succeeds; no status override |
| `Refuted` | own one contradiction diagnostic; preserve actual facts; status becomes `Invalid(cause)` where operation-owned |
| `DynamicBoundary(obligation)` | preserve obligation; publish dynamic-boundary status/contract obligation |
| `Blocked(reason)` | publish `Blocked(reason)` |
| `Cancelled` | publish `Cancelled` |
| `BudgetExceeded(report)` | publish `BudgetExceeded(report)` |
| `InternalFailure(message)` | allocate/publish internal analysis incident; do not emit a user type mismatch |

Known consumers to migrate include:

- `checker/expression.rs::check_typed_expr`;
- return checks in `checker/statement.rs` and `checker/body.rs`;
- ordinary call argument checks in `checker/call.rs`;
- assignment/collection/index checks that currently use context enforcement;
- binding reconciliation;
- every helper found by the required repository audit.

No consumer may merely ask “was this Refuted?” when a terminal outcome is semantically observable.

## 6.8 Structured injection seam for exhaustive tests

Even after real budget/cancellation wiring, not every terminal variant is easy to trigger from source syntax (especially `InternalFailure`). Provide a narrow testable seam at the checker relation-application boundary.

Preferred seam:

```text
apply_relation_outcome(outcome, judgment metadata, owner) -> RelationApplication
```

Production calls compute the bounded `RelationOutcome` then pass it through this same function. Direct tests can pass a constructed outcome to validate policy without mocking `CheckingContext` or relation internals.

The injection seam is for exhaustive policy testing; production source analysis must still use bounded relations.

# 7. Binding reconciliation terminal fidelity

Modify:

`phalcom-semantic/src/checker/binding.rs`

Binding reconciliation must consume the same structured bounded relation semantics as expression/call checking. It must not call an unbounded compatibility wrapper and then relabel terminal outcomes.

Preferred target:

```rust
pub enum BindingConsistency {
    Unconstrained,
    Validated,
    Assumed { basis: AssumptionBasis },
    Refuted { failure: RelationFailure, /* optional normalized operands */ },
    DynamicBoundary { obligation: DynamicBoundaryObligation },
    Blocked(BlockReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(Box<str>),
}
```

If the external/public shape of `Refuted { actual, expected, reason }` must remain temporarily for diagnostics, retain it only if every `RelationFailure` can be represented without inventing operands. Otherwise migrate to the real `RelationFailure` as the authoritative failure object.

`reconcile_binding_contract` should either:

1. receive a precomputed `RelationOutcome` from the context/transfer layer; or
2. receive bounded relation control explicitly and call the bounded function.

Prefer option 1 for purity: relation execution belongs to the controlled checker boundary, while reconciliation remains a pure projection of `(contract, actual, relation outcome)` into binding state.

Required one-to-one semantics:

```text
Proven                  -> Validated / Assumed according to actual evidence strength
Refuted(real failure)   -> Refuted(real failure)
DynamicBoundary(o)      -> DynamicBoundary(o)
Blocked(reason)         -> Blocked(reason)
Cancelled               -> Cancelled
BudgetExceeded(report)  -> BudgetExceeded(report)
InternalFailure(message)-> InternalFailure(message)
```

Do not retain these current aliases:

```text
Cancelled       -> Blocked(SuppressedDependency)
InternalFailure -> Blocked(OpaqueNative)
BudgetExceeded  -> generic Blocked when a dedicated variant/status exists
```

Update:

- `db/fingerprint.rs::hash_binding_consistency`;
- explanation rendering/matches;
- source-level binding tests;
- pure binding reconciliation matrix tests.

The expression/statement transfer that installs a binding state must also propagate the terminal relation outcome into the owning operation/callable status when appropriate. Binding consistency alone is not a substitute for `AnalysisStatus`.

# 8. Return summary repair

Modify:

`phalcom-semantic/src/checker/analysis.rs`

Replace `normal_return_summary` with a direct semantic join:

```rust
pub fn normal_return_summary(
    store: &mut TypeStore,
    values: &[TypeKnowledge],
) -> TypeKnowledge {
    if values.is_empty() {
        return TypeKnowledge::established(store.never(), EvidenceOrigin::Flow);
    }
    join_type_knowledge(store, values.iter().cloned())
}
```

The summary must not inspect only `joined.ty()`.

Expected results:

```text
[Dynamic(RuntimeReflection)]
    -> Dynamic(RuntimeReflection)

[Unknown(InferenceConflict)]
    -> Unknown(InferenceConflict)

[Established(Int), Assumed(Number)]
    -> Assumed(Int | Number)
```

If multiple unknown/dynamic reasons require deterministic aggregation, solve that in `join_type_knowledge`; do not erase the reason at callable summary level.

---

# 9. Assignment expected-context repair

Modify:

`phalcom-semantic/src/checker/expression.rs`

In `Expr::Assignment`, the RHS expected context must be derived **only** from the persistent binding contract.

Delete the fallback:

```text
FlowState.current.ty() -> ExpectedType::AssignmentContract
```

Target:

```rust
let target_expected = binding_state
    .contract
    .as_ref()
    .map(|contract| {
        ExpectedType::proper_from(
            contract.ty,
            ExpectationOrigin::AssignmentContract,
        )
    })
    .unwrap_or_default();
```

Current flow knowledge remains useful after RHS synthesis for recovery and flow transfer. It is simply not a source of persistent expected context.

Regression must use a context-sensitive RHS; a plain literal will not expose this bug because it synthesizes independently.

---

# 10. Flow invariant repair

## 10.1 Same-identity contract mismatch must fail closed

Modify:

`phalcom-semantic/src/checker/flow/state.rs`

Do not represent divergent contracts as `None`.

Target API:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FlowJoinFailure {
    DivergentBindingContract {
        binding: BindingId,
        // compact details sufficient for incident/explanation; allocate vectors only on failure
    },
    DivergentMutability {
        binding: BindingId,
    },
}

pub fn join_with_hierarchy(...) -> Result<FlowState, FlowJoinFailure>
```

A same-identity contract disagreement is not a user-level flow union. `None` means unconstrained and is therefore a false, fail-open state; picking the first branch is order-dependent and equally false.

## 10.2 Add one checker-level flow-invariant publication path

Modify:

`phalcom-semantic/src/checker/context.rs`

Add one operation equivalent to:

```text
publish_flow_join_failure(failure, source range, optional expression owner)
    -> AnalysisStatus::InternalFailure(incident)
```

The context should allocate a stable local `AnalysisIncidentId` (or use the repository's chosen incident allocation mechanism), retain enough incident detail for debugging/telemetry, and mark the current checker control as terminal-internal-failure for callable aggregation.

Do **not** emit an ordinary user `TypeMismatch` diagnostic for an analyzer invariant failure.

Fail-closed flow recovery after invariant failure:

```text
ctx.flow = FlowState::unreachable() / explicit poisoned flow state
```

so later statements cannot consume a fabricated binding state.

If the implementation already has or introduces a dedicated invalid-flow state instead of `unreachable`, that is acceptable provided it cannot be mistaken for valid normal flow.

## 10.3 Migrate every production join caller explicitly

Current production callers on the verified baseline are:

### Expression/control-flow callers in `checker/expression.rs`

1. if-let branch merge;
2. if/else/control branch merge;
3. control-loop method/body + continue/break merge.

For expression callers:

```text
match FlowState::join_with_hierarchy(...) {
    Ok(flow) => ctx.flow = flow,
    Err(failure) => {
        incident = ctx.publish_flow_join_failure(...)
        ctx.flow = fail_closed_flow
        result.status = InternalFailure(incident)
    }
}
```

Preserve independently computed expression knowledge/causal information when it does not depend on the validity of the joined binding state. For example, if both branch expression values were already analyzed, their value-knowledge join may remain available while status reports internal failure. Do not manufacture `Unknown` solely to encode the invariant failure.

### Statement caller in `checker/statement.rs`

`Statement::For` currently joins `before`, `body_flow`, continues, and breaks directly into `ctx.flow`.

On join failure:

1. publish the internal incident through the context helper;
2. install fail-closed flow;
3. return from the statement transfer without inventing a binding state;
4. make `analyze_callable_body` observe the context terminal condition after the statement and publish callable-level internal failure.

## 10.4 Callable-level status must represent flow internal failure

`CallableAnalysisStatus` currently has `Complete`, `Partial`, `Blocked`, `Cancelled`, and `BudgetExceeded`, but no internal-failure state.

Add a dedicated callable-level internal-failure representation or another semantically equivalent terminal carrier. Do not map flow invariant failure to `Blocked` or `Partial` merely to fit the enum.

Update callable-status fingerprinting exhaustively.

## 10.5 Mutability disagreement is also invariant failure

The same `BindingId` cannot be mutable in one incoming state and immutable in another under ordinary control flow. Current `all(incoming mutable)` behavior silently weakens disagreement to immutable.

Return `FlowJoinFailure::DivergentMutability` instead.

## 10.6 Widening must validate invariants and reconcile

Change the widening API to be hierarchy-aware and fail-closed:

```rust
pub fn widen_loop_state_with_hierarchy(
    header: &FlowState,
    next_header: &FlowState,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
) -> Result<FlowState, FlowJoinFailure>
```

or migrate the existing name to that contract.

For every binding whose current knowledge changes:

1. validate stable contract equality;
2. validate mutability equality;
3. join current knowledge;
4. join denotation and causal invalidity;
5. reconcile widened current against persistent contract;
6. store the new consistency;
7. increment version.

Current semantic production does not appear to call `widen_loop_state` directly; current hits are the definition, direct semantic tests, and legacy LSP flow. Therefore this slice primarily hardens the semantic API/tests now. Any future production caller must consume the `Result` contract rather than reintroducing infallibility.

## 10.7 Flow failure tests must include caller publication

Direct `FlowState` tests are necessary but insufficient. Add at least:

- one direct divergent-contract `Err` test;
- one expression branch test verifying published `InternalFailure` and fail-closed subsequent flow;
- one statement/`for`-path test or focused context seam test verifying callable-level internal failure propagation.

# 11. Generic inference hardening

## 11.1 Preserve the actual failed upper bound

Modify:

`phalcom-semantic/src/checker/inference.rs`

In the lower-bound candidate reconciliation loop, retain the exact failing upper:

```rust
let failed_upper = uppers
    .iter()
    .copied()
    .find(|&upper| !is_subtype(store, hierarchy, candidate, upper));

if let Some(upper) = failed_upper {
    let failure = InferenceFailureReason::ConflictingBounds {
        var: rep,
        lower: candidate,
        upper,
    };
    ...
}
```

If upper bounds can retain originating constraint IDs, improve the representation further so this final reconciliation can identify the real originating constraint. Do not invent an index if the solver has already collapsed the bound and no stable origin remains; `constraint_index: None` is better than false provenance.

## 11.2 Remove `KindId::TYPE` solver fallback

Do not use:

```rust
...find(...).map(...).unwrap_or(KindId::TYPE)
```

inside solver operations.

Introduce a malformed-solver-state failure, for example:

```rust
InferenceFailureReason::MissingVariableMetadata {
    var: InferVarId,
}
```

or a dedicated internal solver error if inference failures are intentionally user-semantic only.

Then:

```rust
let expected_kind = self.variable(rep)
    .ok_or(InferenceFailureReason::MissingVariableMetadata { var: rep })?
    .kind;
```

Apply the same rule in variable-variable unification.

If metadata absence is statically impossible after construction, a checked internal invariant error is also acceptable, but it must not silently change the kind to `Type` in release builds.

## 11.3 Keep cancellation/budget outcome capability real

Today `InferenceOutcome` includes cancellation/budget variants, but the solver itself has no cancellation/budget inputs. Do not remove the variants merely because current solver execution is bounded by passes.

Instead, after call-result status propagation is in place, decide one of two explicit architectures:

1. wire `QueryBudget`/`CancellationToken` into `InferenceSession::solve`; or
2. document and test that inference receives terminal cancellation/budget from its enclosing query layer rather than producing those outcomes itself.

The current half-state—variants that call code handles but solver cannot emit—should not remain indefinitely.

For this bugfix series, terminal call-result propagation is required; direct solver cancellation wiring may be deferred if broader query ownership makes it inappropriate.

---

# 12. Callable provenance and contract representation

## 12.1 Reclassify callable parameters at body entry

Modify:

`phalcom-semantic/src/checker/context.rs`

`bind_callable_parameter_with_causal` should not reuse incoming known evidence unchanged.

If the exact signature provides a concrete parameter type `T`, construct body-entry knowledge as:

```rust
TypeKnowledge::assumed(T, EvidenceOrigin::CallableSignature)
```

while retaining the persistent contract:

```text
BindingContractOrigin::CallableParameter
```

Expected binding consistency:

```text
Assumed(CallableParameterContract)
```

If the signature parameter is `Unknown` or `Dynamic`, preserve that non-known semantic state rather than manufacturing an assumption.

The function should conceptually take a **parameter contract**, not arbitrary current value knowledge. Consider changing its input from `TypeKnowledge` to the exact signature parameter representation plus its causal/provenance context.

## 12.2 Replace `expected_return: Option<TypeKnowledge>`

Modify:

`phalcom-semantic/src/checker/context.rs`

Preferred representation:

```rust
pub struct CallableReturnContract {
    pub ty: TypeId,
    pub origin: EvidenceOrigin,
    pub source: Option<SourceRange>,
}

pub expected_return: Option<CallableReturnContract>
```

or reuse an existing contract/context type if one already carries these semantics cleanly.

Then update:

- `checker/body.rs` body-entry initialization;
- `checker/statement.rs` return checking;
- tail-expression expected-context construction;
- block/nested callable save/restore logic.

Return checking obtains an `ExpectedType` from the contract. It does not treat the contract as a value fact.

## 12.3 Preserve exact return provenance

`promote_exact_return` currently receives only `&TypeKnowledge` and a source range, so it cannot distinguish an ordinary callable contract from constructor or trusted-native semantics. `CallableSignature` currently contains selector, parameters, return knowledge, and generic signature, but no explicit semantic call kind.

The target must make exact-result provenance an input to promotion rather than guessing from the return type.

Preferred architecture:

```rust
pub(crate) enum ExactReturnOrigin {
    CallableSignature,
    ConstructorSemantics,
    NativeSignature,
}

pub(crate) fn promote_exact_return(
    return_type: &TypeKnowledge,
    origin: ExactReturnOrigin,
    range: SourceRange,
) -> TypeKnowledge
```

The origin is derived from resolved callable/declaration metadata at the exact-dispatch boundary. It is **not** inferred from the `TypeKnowledge` stored in the signature.

If the current dispatch path cannot expose this without changing `CallableSignature`, introduce a small resolved-call contract wrapper carrying:

```text
CallableId
CallableSignature
ExactReturnOrigin / callable semantic kind
```

and use it only at checker/call publication. Avoid adding constructor/native flags to every generic type/signature object if the metadata already exists on declaration/member surfaces.

Normative mapping:

```text
ordinary exact callable -> CallableSignature
exact @constructor       -> ConstructorSemantics
trusted native/intrinsic -> NativeSignature
generic substituted      -> GenericInference
```

### Required stale-test migration

`tests/semantic/capabilities/authority.rs` currently hardcodes `CallableSignature` in `assert_method_call_evidence` and in constructor-derived binding assertions. Those tests must be changed **before** production code:

1. make the helper accept an expected origin;
2. change only constructor-specific expectations to `ConstructorSemantics`;
3. leave ordinary methods at `CallableSignature`;
4. observe the constructor tests RED;
5. then implement origin-aware promotion.

This is an intentional test-oracle correction. Existing passing tests do not override the normative implementation specification.

Do not preserve raw `DeveloperAnnotation` at call site as if developer syntax alone established the runtime return. Exact resolved callable semantics are what justify call-site established knowledge.

Constructor bodies and class-side constructor signatures require explicit coverage because their callable identity/side handling differs from ordinary methods.

---

# 13. Evidence construction trust boundary

Modify:

`phalcom-semantic/src/types/evidence.rs`

Goal: ordinary semantic code should not be able to construct arbitrary `TypeEvidence { status: Established, ... }` without going through trusted constructors.

Recommended API:

```rust
pub struct TypeEvidence {
    ty: TypeId,
    status: EvidenceStatus,
    origin: EvidenceOrigin,
    provenance: EvidenceSet,
}
```

Expose read accessors:

```rust
pub fn ty(&self) -> TypeId
pub fn status(&self) -> EvidenceStatus
pub fn origin(&self) -> EvidenceOrigin
pub fn provenance(&self) -> &EvidenceSet
```

Keep semantic constructors on `TypeKnowledge`:

```text
established
assumed
map_type
with_range
```

Add a focused trusted derivation constructor for transformations such as tuple/pattern decomposition that need to preserve status while changing origin:

```rust
pub(crate) fn derive_known_type(
    source: &TypeKnowledge,
    ty: TypeId,
    origin: EvidenceOrigin,
) -> TypeKnowledge
```

or equivalent.

Migrate direct `TypeEvidence` struct literals, including known cases in:

- `checker/statement.rs` tuple/pattern decomposition;
- annotation/type helpers discovered by search;
- tests that construct raw evidence for convenience.

Tests may use explicit test helpers behind `#[cfg(test)]` rather than keeping production fields public.

This change is architectural containment: it prevents future subsystems from bypassing evidence authority.

---

# 14. Remove binding declaration mirrors

This cleanup should happen only after flow/contract tests are green.

## 14.1 `BindingState::declared`

Modify:

`phalcom-semantic/src/checker/analysis.rs`

Remove `declared` as an independent semantic field.

Provide a compatibility read method if consumers still need the declared type:

```rust
impl BindingState {
    pub fn declared_type(&self) -> Option<TypeId> {
        self.contract.as_ref().map(|contract| contract.ty)
    }
}
```

This guarantees one source of truth.

## 14.2 `LocalBindingInfo::declared`

Modify:

`phalcom-semantic/src/checker/context.rs`

Lexical scope should retain only lookup identity and metadata that cannot be obtained from the flow binding state.

Preferred:

```rust
pub struct LocalBindingInfo {
    pub id: BindingId,
}
```

If denotation is needed for declarations not represented in flow, justify it separately. For ordinary local bindings, current denotation belongs to `FlowState`.

Update `lookup_local` and related readers to resolve current semantic facts through `BindingId -> FlowState`.

## 14.3 Compatibility APIs

Remove or deprecate flow APIs whose signatures require both `declared` and `contract` independently:

```text
BindingState::new_with_contract(... declared, contract ...)
FlowState::declare_with_contract(... declared, contract ...)
get_declared_type
```

Prefer `BindingSeed`/`BindingContract` as the construction path.

---

# 15. Deterministic `Unknown` and `Dynamic` joins

Modify:

`phalcom-semantic/src/types/evidence.rs`

The join's semantic class is already conservative, but reason selection should not depend on incoming vector order.

Do not solve this with arbitrary enum discriminant ordering unless that ordering has a documented semantic meaning.

Preferred design: introduce deterministic reason-combination functions:

```rust
fn join_unknown_reasons(reasons: impl Iterator<Item = UnknownReason>) -> UnknownReason
fn join_dynamic_reasons(reasons: impl Iterator<Item = DynamicReason>) -> DynamicReason
```

Options, in preferred order:

1. add aggregate variants (`MultipleUnknownReasons`, `MixedDynamicBoundary`) if explanation needs to preserve multiplicity;
2. define an explicit semantic precedence table based on information/recovery significance;
3. retain a normalized small set/summary outside hot `TypeKnowledge` if richer explanation is required.

The key conformance law is:

```text
join(a, b) == join(b, a)
```

at the externally observable reason level for equivalent reachable branch sets.

This is P2 compared with status/flow defects. Do not block the core repair if the reason lattice requires separate design ratification; if deferred, add a tracked failing/ignored regression and document the deferral explicitly.

---

# 16. Semantic fingerprint repairs

## 16.1 Expand `FlowStateSummary`

Modify:

`phalcom-semantic/src/checker/analysis.rs`

Do not make the summary a full clone of `BindingState`; it is a compact semantic boundary.

Recommended compact representation:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowBindingSummary {
    pub knowledge: TypeKnowledge,
    pub contract: Option<BindingContract>,
    pub consistency: BindingConsistency,
    pub causal_invalidity: CausalInvalidity,
    pub mutable: bool,
    pub denotation: Option<SemanticDenotation>,
}

pub struct FlowStateSummary {
    pub bindings: BTreeMap<BindingId, FlowBindingSummary>,
    pub fact_count: usize,
}
```

If some fields are proven irrelevant to all consumers of entry/exit summaries, omit them only with an explicit product-contract justification and regression demonstrating equivalence.

At minimum, the summary must distinguish:

```text
Established(Int)
Assumed(Int)
Unknown(...)
Dynamic(...)
```

and any contract/consistency facts that affect downstream semantics.

Update the flow-state summarizer and every construction/read site.

## 16.2 Hash epistemic flow semantics

Modify:

`phalcom-semantic/src/db/fingerprint.rs`

`hash_flow_summary` should hash full summary semantics using existing helpers:

```text
hash_type_knowledge
hash_binding_contract
hash_binding_consistency
hash_causal_invalidity
hash_denotation
mutability
```

Do not hash incidental binding version counters unless version itself is an externally meaningful semantic property. Version is primarily an internal mutation generation and should usually not make equivalent semantic states different.

## 16.3 Stop hashing raw diagnostic root-cause IDs in the correct product

Modify:

`phalcom-semantic/src/db/fingerprint.rs`

The verified defect is specifically observable through:

```text
module_diagnostics_product_fingerprint(module, diagnostics)
```

because that product calls `hash_semantic_diagnostic`, and the helper currently performs:

```rust
diagnostic.root_cause.hash(hasher);
```

`callable_body_product_fingerprint` does **not** hash `analysis.diagnostics` on the verified baseline, so raw cause-number stability must not be tested against callable-body fingerprints.

For semantic diagnostic product identity, replace raw numeric cause identity with semantic cause shape/presence where that is part of the product contract. A minimal shape-preserving implementation is conceptually:

```rust
match diagnostic.root_cause {
    Some(_) => 1u8.hash(hasher),
    None => 0u8.hash(hasher),
}
```

provided the diagnostics product does not need richer causal-graph structure. If richer structure is semantically observable, hash normalized structure through stable semantic identities—not the local `DiagnosticCauseId` number.

### Shared helper caution

`hash_semantic_diagnostic` is also called by diagnostic-bearing query-input fingerprint paths. Do not blindly change every input/product contract together.

Preferred choices:

```text
A. split semantic-product diagnostic hashing from source/query-input diagnostic hashing; or
B. add an explicit hashing mode whose contract is tested.
```

Even for input fingerprints, raw cause allocator numbers are usually incidental; however the implementation agent must verify the intended input-product contract before removing any additional fields.

Required regression:

```text
same module + same diagnostics + root cause id 17
same module + same diagnostics + root cause id 91
        -> equal module diagnostics product fingerprint
```

and a separate guard for root-cause presence/semantic shape if that is intentionally observable.

## 16.4 Revisit diagnostic ranges only as a separate product-contract question

`hash_semantic_diagnostic` currently hashes source spans/ranges. Whether those ranges belong in the semantic *product* fingerprint depends on whether callable body product identity intentionally includes diagnostic presentation location.

Do not broaden this bugfix unnecessarily. The required correction in this series is cause-number insensitivity.

If existing incrementality architecture distinguishes input fingerprints from semantic product fingerprints, add a follow-up audit to determine whether range-only diagnostic movement should propagate semantic invalidation. Do not silently change range semantics without tests.

---

# 17. Test plan — RED before GREEN

The tests below are part of the implementation, not optional validation after the fact.

The current repository organizes semantic integration tests under:

```text
phalcom-semantic/tests/semantic/
    foundations/
    capabilities/
    advanced/
    integration/
    incremental/
```

Prefer extending focused existing files rather than creating one new file per defect.

---

# 18. Task 1 — Status/causality orthogonality

**Production files:**

- `phalcom-semantic/src/checker/typed_expr.rs`
- `phalcom-semantic/src/checker/expression.rs`
- `phalcom-semantic/src/checker/context.rs`

**Test files:**

- extend `phalcom-semantic/tests/semantic/capabilities/authority.rs`;
- extend `phalcom-semantic/tests/semantic/foundations/causal.rs`;
- optionally `foundations/expression_analysis.rs` for pure product behavior.

## RED 1A — invalid binding remains analyzable downstream

Use the existing canonical source fixture or extend it:

```phalcom
class CellNum {
  @constructor
  new() {}

  cellOnly() -> Int { 1 }
}

class Probe {
  @class
  run() {
    let x: Int = CellNum.new()
    let y = x.cellOnly()
  }
}
```

Assert all of:

```text
x.current.ty()                     == CellNum
x.current.status()                 == Established
x.consistency                      == Refuted(...)
x.causal_invalidity                == One(C1)

expression("x.cellOnly()").ty      == Int
expression("x.cellOnly()").status  == Ready
expression("x.cellOnly()").causal_invalidity == One(C1)
resolved callable                  == CellNum.cellOnly
```

**Expected baseline failure:** downstream expression status is `Suppressed` because causal invalidity is non-clean.

This test is the primary guard for the Part 1 recovery model.

## RED 1B — causally invalid variable read is Ready

Assert the expression product for the `x` read itself is:

```text
knowledge  Established(CellNum)
status     Ready
invalidity One(C1)
```

## RED 1C — genuine suppressed premise has a real producer

Task 1C must not fabricate `TypedExpression { status: Suppressed(...) }` directly and call that sufficient. It must exercise the production rule that decides a required premise is unavailable because of upstream invalidity.

Preferred source-level fixture: create a generic call whose return depends on a generic variable and whose real constraint is refuted, then immediately use the failed result as a method receiver.

Conceptual shape:

```phalcom
class Allowed {
  allowedOnly() -> Int { 1 }
}

class Bad {
  @constructor
  new() {}
}

class Generic {
  @class
  id<T>(value: T) -> T where T <: Allowed {
    value
  }
}

class Probe {
  @class
  run() {
    Generic.id(Bad.new()).allowedOnly()
  }
}
```

Exact syntax may be adapted to the currently passing generic-constraint fixtures; do not invent parser syntax just for this test.

Required semantics:

```text
inner Generic.id(...):
    owns generic-constraint diagnostic C1
    return depends on failed T
    knowledge has no usable receiver TypeId
    status = Invalid(C1)
    invalidity = One(C1)

outer .allowedOnly():
    requires receiver type to perform dispatch
    receiver premise unavailable because of C1
    emits no duplicate missing-method/type-mismatch diagnostic
    status = Suppressed(One(C1))
    invalidity = One(C1)
```

If a stable source fixture cannot trigger this exact path before the producer exists, add a focused test of the new required-premise helper **in addition to**, not instead of, the final source-level composition test.

The helper/producer rule must distinguish four cases:

```text
Ready + Known + causal invalidity
    -> premise available; parent remains analyzable

Invalid/Suppressed + no required type + owning upstream cause
    -> parent Suppressed(same cause)

Ready + Unknown(no upstream invalid cause)
    -> parent remains honest Unknown/Ready-or-blocked according to expression rule; not Suppressed

Blocked/Cancelled/BudgetExceeded/InternalFailure
    -> propagate that terminal state; never relabel as Suppressed
```

Apply the first production use at receiver-required operations such as method dispatch; extend to property/index operations where they have the same semantic premise dependency.

## GREEN

Add `TypedExpression.status`, migrate constructors/conversions, and change expression publication to preserve the inner status.

## REFACTOR

Centralize helpers for joining/overriding status without coupling them to causal invalidity.

---

# 19. Task 2 — Bounded relation execution and terminal propagation

**Production files:**

- `types/relation.rs`;
- `types/outcome.rs` if carrier extensions are required;
- `checker/context.rs`;
- `checker/body.rs`;
- `checker/policy.rs`;
- `checker/expression.rs`;
- `checker/statement.rs`;
- `checker/call.rs`;
- `checker/binding.rs`.

**Tests:**

- `foundations/type_model.rs` or the existing direct relation test module;
- `foundations/binding_contracts.rs`;
- `foundations/bidirectional_calls.rs`;
- `foundations/expression_analysis.rs`;
- capability/integration tests where source composition is possible.

## RED 2A — nested bounded relation preserves budget exhaustion

Construct a structural relation that requires more than one nested relation step (tuple, callable, applied type, or union-source relation). Use a deliberately tiny `QueryBudget` so exhaustion occurs inside the nested check.

Assert:

```text
check_*_bounded(...) == BudgetExceeded(report)
```

not `Refuted(TypeMismatch)`.

## RED 2B — nested bounded relation preserves cancellation

Use a cancelled token on the same structural relation. Assert `Cancelled` survives recursive composition.

If cooperative mid-relation cancellation cannot be deterministically triggered without threads, pre-cancelled entry plus nested budget tests are sufficient for this slice; do not add timing-sensitive tests.

## RED 2C — dynamic obligation survives compatibility projection

Construct dynamic type knowledge with a bounded assignability relation and assert the returned `DynamicBoundaryObligation.reason` survives through whichever compatibility projection remains public.

If checker core is migrated away from `Assignability`, also test that checker application receives the full obligation.

## RED 2D — binding reconciliation variant matrix

Use the structured relation-application/reconciliation seam to verify one-to-one preservation of:

```text
Blocked(reason)
Cancelled
BudgetExceeded(report)
InternalFailure(message)
DynamicBoundary(obligation)
```

Do not manufacture `Assignability::Uncertain` as a normative bounded outcome unless it remains a real reachable carrier after the relation cleanup.

## RED 2E — real callable budget reaches return checking

Analyze a callable with a deliberately small budget such that body entry succeeds but a nested return relation exhausts the shared relation budget.

Assert callable/expression status is `BudgetExceeded`, not `Complete`, `Invalid(TypeMismatch)`, or `Blocked(BudgetExceeded(...))`.

This is the proof that `CheckingContext` is using the caller's budget rather than a fresh default relation budget.

## RED 2F — real cancellation reaches checker relation consumer

Use a cancelled query token with callable analysis and assert cancellation reaches the appropriate callable/expression boundary without a type-mismatch diagnostic.

Where outer body cancellation preempts expression analysis, use the structured relation-application injection seam to cover expression-level cancellation policy as a separate direct test.

## RED 2G — call argument terminal outcome propagates

Use the structured injection seam or a low-budget source fixture to make an argument relation terminal. Assert the call does not remain `Ready` merely because the outcome is non-`Refuted`.

## GREEN

1. preserve terminal outcomes in recursive bounded relations;
2. make `RelationOutcome` the checker carrier;
3. thread shared query control into `CheckingContext`;
4. replace boolean context enforcement helpers;
5. delete/migrate `checker/policy.rs` boolean helpers;
6. preserve dynamic obligations;
7. extend binding consistency;
8. propagate terminal status through expression/call/callable products.

## Static gate

```bash
rg 'handle_relation_outcome|enforce_assignability|enforce_knowledge_against_type' phalcom-semantic/src/checker
rg '->\s*bool' phalcom-semantic/src/checker/policy.rs phalcom-semantic/src/checker/context.rs
rg 'is_assignable\(' phalcom-semantic/src
rg 'RelationOutcome::DynamicBoundary\(_\)\s*=>\s*Assignability::DynamicBoundary' phalcom-semantic/src/types/relation.rs
rg '!.*is_proven\(\)' phalcom-semantic/src/types/relation.rs
```

Every surviving hit must be classified. In particular, recursive `!is_proven()` checks are acceptable only if their match logic explicitly distinguishes real refutation from terminal outcomes rather than converting both to the same result.

# 20. Task 3 — Return-summary preservation

**Production file:** `checker/analysis.rs`

**Tests:** add to a callable-analysis/foundations test file; if no focused file exists, create `foundations/callable_returns.rs` and register it in the semantic test module.

## RED 3A — Dynamic return remains Dynamic

Direct unit-level test:

```rust
let result = normal_return_summary(
    &mut store,
    &[TypeKnowledge::Dynamic(DynamicReason::RuntimeReflection)],
);
assert_eq!(result, TypeKnowledge::Dynamic(DynamicReason::RuntimeReflection));
```

## RED 3B — Unknown reason survives

```text
Unknown(InferenceConflict) -> Unknown(InferenceConflict)
```

not `UncheckedExpression`.

## RED 3C — mixed known evidence still joins normally

Guard against regression:

```text
Established(Int) + Assumed(Number)
-> Assumed(Int | Number)
```

## GREEN

Remove the `joined.ty().is_none()` rewrite.

---

# 21. Task 4 — Assignment-context contract/current separation

**Production:** `checker/expression.rs`

**Tests:** `foundations/expression_engine.rs` or `foundations/bidirectional_calls.rs` depending on existing test helpers.

## RED 4A — previous current fact is not assignment context

The fixture must use a context-sensitive RHS.

One suitable shape is an unconstrained/no-persistent-contract binding available through a test-level flow setup, followed by assignment of an empty collection or generic construct whose type would differ depending on expected context.

Assert:

```text
binding.contract == None
binding.current  == Known(previous type)
RHS expected context == None
```

Do not merely assert the final assignment type if the language's current unannotated-binding policy normally creates `InferredInitializer` contracts. If source syntax cannot naturally create a contract-free local with known current type, this regression belongs at the checker/flow unit level.

## RED 4B — real persistent contract still supplies context

Guard the positive case:

```phalcom
let xs: List<Int> = []
...
xs = []
```

The second empty list should receive the `List<Int>` assignment contract.

## GREEN

Delete current-type fallback from assignment expected-context construction.

---

# 22. Task 5 — Flow invariant, caller publication, and widening correctness

**Production:**

- `checker/flow/state.rs`;
- `checker/context.rs`;
- `checker/expression.rs`;
- `checker/statement.rs`;
- `checker/body.rs`;
- `checker/analysis.rs` if `CallableAnalysisStatus::InternalFailure` is added there.

**Tests:** extend `foundations/flow_graph.rs`, direct flow-state tests, and at least one source/composition control-flow test.

## RED 5A — divergent same-ID contracts do not become unconstrained

Construct two reachable `FlowState`s containing the same `BindingId` but different persistent contracts.

Assert:

```text
join_with_hierarchy(...) == Err(FlowJoinFailure::DivergentBindingContract { ... })
```

Explicitly reject:

```text
Ok(binding.contract == None)
Ok(binding.consistency == Unconstrained)
Ok(first incoming contract chosen)
```

## RED 5B — divergent mutability is invariant failure

Same `BindingId`, same contract, different mutability -> `Err(DivergentMutability)`.

## RED 5C — expression branch caller publishes internal failure

Use the narrowest existing control expression that invokes `join_with_hierarchy`, or a focused context test around the join-publication helper.

Assert:

```text
expression.status == InternalFailure(incident)
ctx.flow is fail-closed
no ordinary type-mismatch diagnostic is emitted for the invariant failure
```

If branch expression value knowledge was independently computed before the join failure, assert it is retained rather than rewritten to a fake unknown.

## RED 5D — statement/for caller makes callable fail internally

Exercise the `Statement::For` join publication path through a focused constructed flow fixture if source syntax cannot naturally create divergent same-ID declaration metadata.

Assert the callable-level result is an explicit internal failure and subsequent normal statement analysis does not continue on a fabricated flow state.

## RED 5E — widening recomputes consistency

Construct:

```text
contract = Number
header.current = Established(Int)
next.current   = Established(Float)
```

and assert consistency is recomputed from `join(Int, Float) <: Number`.

Also test invariant mismatch in widening returns `FlowJoinFailure` rather than silently weakening state.

## GREEN

1. make flow joins/wideners return `Result`;
2. add one context flow-invariant publisher;
3. add callable-level internal-failure status if absent;
4. migrate all three expression joins and the statement `for` join;
5. fail closed after invariant failure;
6. recompute widening consistency.

## Static gate

```bash
rg 'join_with_hierarchy\(' phalcom-semantic/src/checker
rg 'widen_loop_state' phalcom-semantic/src phalcom-lsp/src
```

Every semantic caller must explicitly handle `Result`; no `.unwrap()`, `.unwrap_or_default()`, first-branch fallback, or `contract=None` recovery is permitted.

# 23. Task 6 — Generic conflict and kind invariants

**Production:** `checker/inference.rs`

**Tests:** existing generic inference session tests, likely under `foundations/generics_core.rs` or advanced inference modules.

## RED 6A — second upper bound is the actual conflict

Create types such that:

```text
candidate <: UpperA == true
candidate <: UpperB == false
```

Store upper bounds in that order.

Assert:

```text
ConflictingBounds.upper == UpperB
```

This should fail on the current `uppers[0]` implementation.

## RED 6B — third upper bound conflict

Use three upper bounds with first two compatible and third incompatible. This ensures the fix is not hard-coded to index one.

## RED 6C — missing inference variable metadata never defaults to Type

Exercise the lowest legitimate malformed-state test seam. If internal fields prevent creating this state through public APIs, add a `#[cfg(test)]` helper in `InferenceSession` that removes/invalidates a variable record, then call `bind`/`unify_terms` and assert explicit malformed-state failure.

Do not weaken production encapsulation to make the test easy.

## GREEN

Retain the actual failed upper. Remove `unwrap_or(KindId::TYPE)` from solver internals.

## Static gate

```bash
rg 'unwrap_or\(KindId::TYPE\)' phalcom-semantic/src/checker/inference.rs
```

Expected: zero solver-internal fallback hits.

---

# 24. Task 7 — Callable boundary and provenance repair

**Production:**

- `checker/body.rs`
- `checker/context.rs`
- `checker/call.rs`
- possibly signature/declaration helpers if origin metadata must be exposed.

**Tests:**

- `capabilities/authority.rs`;
- generic/call tests;
- constructor tests;
- add focused callable-boundary tests if needed.

## RED 7A — callable parameter body-entry origin

Source:

```phalcom
class Probe {
  use(value: Int) {
    value
  }
}
```

Assert body binding:

```text
current.ty      == Int
current.status  == Assumed
current.origin  == CallableSignature
contract.origin == CallableParameter
consistency     == Assumed(CallableParameterContract)
```

This catches the current pass-through of developer-origin signature knowledge.

## RED 7B — contextual block parameter remains contextual

Guard existing semantics:

```text
origin = ContextualDerivation
basis  = ContextualParameterContract
```

## RED 7C — first correct the stale constructor test oracle

Current `tests/semantic/capabilities/authority.rs` has a shared `assert_method_call_evidence` helper that hardcodes:

```text
origin == CallableSignature
```

for every method call, including `CellNum.new()`. Constructor-derived binding assertions also hardcode `CallableSignature`.

Before touching production code:

1. change the helper to accept `expected_origin`;
2. keep ordinary calls passing `CallableSignature`;
3. change exact `@constructor` calls and bindings directly derived from them to expect `ConstructorSemantics`;
4. run the constructor-focused test and observe RED because production still promotes it as `CallableSignature`.

The passing old assertion is a stale oracle that contradicts the normative semantic-analyzer specification; do not preserve it for compatibility.

## RED 7D — constructor call result origin

For exact `CellNum.new()` after the oracle correction:

```text
knowledge.status == Established
knowledge.origin == ConstructorSemantics
explanation.origin == ConstructorSemantics
```

## RED 7E — ordinary exact callable remains CallableSignature

Guard against overcorrecting all exact returns to constructor semantics.

## RED 7F — native exact return preserves NativeSignature

Add if the current test universe exposes a trusted native callable. If not, add an internal signature-level test using the same call promotion function.

## GREEN

Reclassify body-entry parameter knowledge, replace return contract representation, and make exact result promotion origin-aware. Because `CallableSignature` currently has no constructor/native call-kind field, carry exact call semantics from resolved callable/declaration metadata into promotion explicitly; do not infer constructor/native provenance from the return `TypeKnowledge` alone.

---

# 25. Task 8 — Trusted evidence construction

**Production:** `types/evidence.rs` plus all direct `TypeEvidence` constructors.

**Tests:** knowledge invariant/foundation tests and compile-time repository audit.

## RED 8A — pattern decomposition preserves status without raw struct minting

Existing behavior must remain:

```text
source Established(Tuple<Int, String>)
    -> components Established(Int), Established(String)
       origin PatternDecomposition

source Assumed(Tuple<Int, String>)
    -> components Assumed(Int), Assumed(String)
       origin PatternDecomposition
```

Write tests first so the encapsulation change cannot accidentally upgrade/downgrade evidence.

## GREEN

Make `TypeEvidence` fields non-public outside the trusted module boundary and migrate production struct literals to semantic constructors/derivation helpers.

## Static gate

Search for raw construction:

```bash
rg 'TypeEvidence\s*\{' phalcom-semantic/src
```

Expected: only definitions/trusted constructors inside `types/evidence.rs`, unless an explicitly audited trusted module is documented.

---

# 26. Task 9 — Remove binding mirrors

**Production:**

- `checker/analysis.rs`
- `checker/context.rs`
- `checker/flow/state.rs`
- presentation/snapshot code discovered by search.

**Tests:** binding/flow integration plus any LSP compatibility tests that read `.declared`.

## RED 9A — declaration type read derives from contract

Before removing the field, add a test through the intended read API:

```text
binding.declared_type() == binding.contract.map(|c| c.ty)
```

## RED 9B — contract mutation/construction cannot diverge from lexical metadata

After refactor there should be no independent lexical declared-type value to assert. The test should exercise lookup + flow and prove the same contract is observed.

## GREEN

Remove `BindingState::declared` and `LocalBindingInfo::declared`; migrate reads to `contract`/flow.

## Static gate

```bash
rg '\.declared\b' phalcom-semantic/src
```

Classify every surviving hit. No local-binding semantic read should depend on a mirrored declared type.

---

# 27. Task 10 — Semantic fingerprint correctness

**Production:**

- `checker/analysis.rs` — flow summary representation;
- flow summarization producer(s);
- `db/fingerprint.rs`.

**Tests:**

- existing incremental fingerprint tests under `tests/semantic/incremental/`;
- product-stability/differential tests using the current semantic DB harness.

## RED 10A — Established vs Assumed flow summary changes callable-body fingerprint

Construct otherwise-identical callable products whose hashed flow summary differs only in:

```text
A: x = Established(Int)
B: x = Assumed(Int)
```

Assert `callable_body_product_fingerprint(A) != callable_body_product_fingerprint(B)`.

The mutation must affect the actual `FlowStateSummary` representation hashed by callable-body fingerprinting, not merely some redundant binding field.

## RED 10B — Unknown vs Dynamic flow summary changes fingerprint

Same absence of concrete `TypeId`, different semantics -> different callable-body semantic fingerprint.

## RED 10C — binding consistency/contract changes alter flow semantic identity

Where flow summary owns these dimensions, `Validated` vs `Refuted` or distinct persistent contracts must differ.

## RED 10D — raw cause renumbering does not change **module diagnostics product fingerprint**

Create two semantically identical module diagnostic lists whose only difference is:

```text
root_cause = DiagnosticCauseId(17)
root_cause = DiagnosticCauseId(91)
```

Call:

```text
module_diagnostics_product_fingerprint(module, diagnostics)
```

and assert equality.

This is the correct target. `callable_body_product_fingerprint` does not hash `analysis.diagnostics` on the verified baseline.

## RED 10E — root-cause presence/semantic shape still matters where specified

Guard against over-removal. If the diagnostics product contract distinguishes an owning root from no owning root, then:

```text
root_cause = None
root_cause = Some(_)
```

must remain distinguishable without hashing the numeric allocator identity.

Prefer hashing presence/semantic causal structure, not `DiagnosticCauseId.0`.

## RED 10F — shared diagnostic hashing does not accidentally destabilize input/product contracts

`hash_semantic_diagnostic` is also used by diagnostic-bearing query input fingerprints. Add/adjust a test that demonstrates the intended input-vs-product behavior after the helper is changed or split.

Do not globally delete diagnostic information from input fingerprints merely to fix the module diagnostics product.

Preferred implementation:

```text
hash diagnostic semantic content
hash causal root presence/shape if semantically relevant
never hash raw allocator number in semantic product identity
```

If input and product fingerprints genuinely need different treatment, split the helper or add an explicit hashing mode rather than relying on one ambiguous function.

## GREEN

1. expand flow summaries to include epistemically relevant state;
2. hash full semantic flow summary;
3. stop raw root-cause ID from defining module diagnostics product identity;
4. preserve intentional diagnostic/input semantics;
5. run cold/incremental differential tests.

# 28. Task 11 — Deterministic reason joins

This is lower priority than the correctness repairs above but should be included if the reason-lattice design is accepted during the same series.

**Production:** `types/evidence.rs`

**Tests:** knowledge invariant tests.

## RED

For pairs of distinct reasons:

```rust
assert_eq!(
    join_type_knowledge(&mut store, [Unknown(A), Unknown(B)]),
    join_type_knowledge(&mut store, [Unknown(B), Unknown(A)]),
);
```

Equivalent test for dynamic reasons.

Use several permutations for three inputs.

## GREEN

Implement explicit deterministic reason combination.

If this requires new public reason variants and wider design approval, defer this task rather than choosing arbitrary enum ordering. Mark it explicitly as deferred in the completion report.

---

# 29. Implementation order and commit boundaries

The revised sequence moves bounded relation correctness ahead of expression-status plumbing because downstream status tests are otherwise testing unreachable synthetic states.

## Commit 1 — `test(semantic): expose bounded relation terminal loss`

Tests only:

- nested budget exhaustion remains `BudgetExceeded`;
- nested cancellation remains `Cancelled`;
- dynamic obligation survives projection/policy.

## Commit 2 — `fix(semantic): preserve bounded relation terminal outcomes`

- fix recursive `RelationOutcome` composition;
- make compatibility dynamic carrier non-lossy or remove it from checker core;
- audit `is_assignable()` semantics.

## Commit 3 — `test(semantic): expose checker relation-control gaps`

Tests only:

- shared callable budget reaches relation checking;
- structured policy matrix;
- `policy.rs` duplicate boolean behavior.

## Commit 4 — `fix(semantic): thread relation control through checker`

- checker budget/cancellation control;
- bounded context relation APIs;
- structured relation application;
- delete/migrate `policy.rs` boolean helpers;
- binding terminal fidelity.

## Commit 5 — `test(semantic): expose status causality and suppression regressions`

Tests only:

- invalid-but-analyzable downstream `Ready + causal`;
- variable read remains `Ready`;
- concrete required-premise suppression fixture.

## Commit 6 — `fix(semantic): carry expression and call status explicitly`

- `TypedExpression.status`;
- `CallCheckResult.status`;
- explicit required-premise suppression producer;
- expression publication preservation;
- no causal->suppressed reconstruction.

## Commit 7 — `fix(semantic): preserve callable return knowledge states`

- return summary dynamic/unknown preservation;
- focused tests.

## Commit 8 — `test(semantic): expose assignment and flow invariant regressions`

- current-fact-as-context;
- divergent contract/mutability;
- expression and statement flow-failure publication;
- widening reconciliation.

## Commit 9 — `fix(semantic): harden binding and flow invariants`

- assignment context fix;
- `FlowJoinFailure`;
- context incident publication;
- all production join caller migration;
- callable internal-failure status;
- widening reconciliation.

## Commit 10 — `test(semantic): expose inference evidence regressions`

- actual failed second/third upper;
- missing-kind metadata.

## Commit 11 — `fix(semantic): harden generic solver evidence`

- failed-upper capture;
- remove kind fallback.

## Commit 12 — `test(semantic): correct callable provenance oracles`

- split authority helper expected origin;
- change constructor oracle to `ConstructorSemantics` and observe RED;
- parameter body-entry origin;
- ordinary/native/generic guards.

## Commit 13 — `fix(semantic): normalize callable contract provenance`

- body parameter reclassification;
- return contract representation;
- explicit call-kind/origin-aware exact promotion.

## Commit 14 — `refactor(semantic): restrict established evidence construction`

- evidence encapsulation;
- pattern decomposition migration.

## Commit 15 — `refactor(semantic): remove binding contract mirrors`

- remove `declared` duplicates;
- flow/scope read migration.

## Commit 16 — `test(semantic): expose semantic fingerprint identity gaps`

- flow epistemic fingerprints;
- module diagnostics cause-renumbering regression;
- diagnostic input/product contract guard.

## Commit 17 — `fix(semantic): fingerprint full semantic identity`

- expanded flow summary;
- callable-body fingerprint updates;
- module diagnostics root-cause allocator removal.

## Optional Commit 18 — `fix(semantic): make unknown reason joins deterministic`

Only after the reason-combination policy is ratified.

# 30. Cross-cutting migration rules

## 30.1 Never fix a semantic-status bug by weakening the test

If a test asserts:

```text
Ready + One(C1)
```

and the implementation currently produces `Suppressed`, the repair is not to assert only the type or causal state.

## 30.2 Keep actual value facts through contradictions

Across binding, return, argument, and assignment fixes, do not regress to overwriting actual `TypeKnowledge` with contract types.

## 30.3 Do not turn operational failures into `Unknown` only

`UnknownReason` can describe why value type knowledge is absent. It does not replace `AnalysisStatus` for cancellation, budget, internal failure, or suppression.

## 30.4 Do not let the advisory layer participate

No Part 2 advisory fact is required to repair these formal semantics. Formal products remain authoritative.

## 30.5 Do not broaden the patch into AST completeness

The aggregate capability suite has broader parser/flow/structural gaps. Those are not justification for fabricating semantic fallbacks in these fixes.

---

# 31. Required repository audits

Run these after the relevant slices. Every surviving hit must be classified; zero hits are not required where a compatibility API remains intentionally public.

## Status reconstruction / suppression

```bash
rg 'suppression_cause\(\)|mark_suppressed\(' phalcom-semantic/src/checker
```

Every producer must correspond to a real required-premise suppression rule. No surviving code may implement “non-clean causal invalidity => Suppressed.”

## Boolean relation laundering — context **and policy**

```bash
rg 'handle_relation_outcome|enforce_(assignability|knowledge_against_type)' phalcom-semantic/src/checker
rg '->\s*bool' phalcom-semantic/src/checker/policy.rs phalcom-semantic/src/checker/context.rs
rg 'Assignability::Refuted.*=>.*false|_\s*=>\s*true' phalcom-semantic/src/checker phalcom-semantic/src/types/relation.rs
rg 'is_assignable\(' phalcom-semantic/src
```

No checker path may treat “not refuted” as “proved.”

## Bounded relation wiring

```bash
rg 'check_(assignability|knowledge_against_type)\(' phalcom-semantic/src/checker
rg 'check_(assignability|knowledge_against_type)_bounded' phalcom-semantic/src/checker phalcom-semantic/src/types
rg 'QueryBudget::default\(\)|CancellationToken::new\(\)' phalcom-semantic/src/types/relation.rs phalcom-semantic/src/checker
```

Unbounded wrappers may remain as public convenience APIs, but production checker transfer must use the caller's real control.

## Recursive relation terminal preservation

```bash
rg '!.*is_proven\(\)|\.is_proven\(\)' phalcom-semantic/src/types/relation.rs
```

Classify every recursive use. A nested terminal result must not be converted to `Refuted` merely because it is non-proven.

## Dynamic obligation preservation

```bash
rg 'DynamicBoundary' phalcom-semantic/src/types/relation.rs phalcom-semantic/src/checker
```

There must be no unchecked `DynamicBoundary(_) -> DynamicBoundary` projection that loses `DynamicBoundaryObligation` on a path used by formal checking.

## Flow joins

```bash
rg 'join_with_hierarchy\(' phalcom-semantic/src/checker
rg 'widen_loop_state' phalcom-semantic/src phalcom-lsp/src
```

All semantic callers must handle invariant failure explicitly.

## Inference kind fallback

```bash
rg 'unwrap_or\(KindId::TYPE\)' phalcom-semantic/src/checker/inference.rs
```

Expected zero fallback uses in solver metadata lookup.

## Raw evidence construction

```bash
rg 'TypeEvidence\s*\{' phalcom-semantic/src
```

Only trusted construction sites may remain.

## Declared mirrors

```bash
rg '\bdeclared\b' phalcom-semantic/src/checker
```

Classify every hit. No independent current semantic authority may remain.

## Raw diagnostic cause fingerprinting

```bash
rg 'root_cause\.hash|DiagnosticCauseId.*hash' phalcom-semantic/src/db/fingerprint.rs
```

Raw local cause IDs must not define `module_diagnostics_product_fingerprint` semantic identity. Classify input-fingerprint uses separately rather than deleting them indiscriminately.

## Constructor provenance stale oracles

```bash
rg 'CallableSignature' phalcom-semantic/tests/semantic/capabilities/authority.rs
rg 'ConstructorSemantics' phalcom-semantic/tests phalcom-semantic/src/checker
```

Every constructor-specific assertion must use the normative constructor origin; ordinary method assertions must remain callable-signature origin.

## Sentinel/fabricated fallback audit

```bash
rg 'TypeId::DUMMY|store\.unit\(\)|store\.never\(\)' phalcom-semantic/src/checker phalcom-semantic/src/types
```

Classify every hit as real language semantics or remove it if used as missing-evidence filler.

# 32. Verification commands

The implementation agent must run targeted RED/GREEN commands after each task and a broad suite before completion.

Because the repository now registers semantic tests through `phalcom-semantic/tests/semantic.rs`, the broad focused command is:

```bash
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic --test semantic
```

Use test-name filters during TDD, for example:

```bash
cargo test -p phalcom-semantic --test semantic invalid_but_analyzable -- --nocapture
cargo test -p phalcom-semantic --test semantic conflicting_bounds -- --nocapture
cargo test -p phalcom-semantic --test semantic fingerprint -- --nocapture
```

Exact names should match the tests added by the implementation.

Then run:

```bash
cargo check -p phalcom-semantic
RUST_MIN_STACK=8388608 cargo test -p phalcom-semantic
RUST_MIN_STACK=8388608 cargo test -p phalcom-lsp
cargo fmt --all -- --check
git diff --check
```

Run clippy:

```bash
cargo clippy -p phalcom-semantic --all-targets -- -D warnings
```

If repository baseline remains red on known pre-existing warnings, record the exact baseline before the changes and require **no new warnings in changed code**. Do not silently mark a red clippy run as passing.

Record the verified baseline before adding regressions. At `9b5873025b47dc7addb826f165530391fa93e171` the canonical semantic target is reported as 362 passed, 0 failed, 10 intentionally ignored. The repair must introduce no unrelated regression. New tests intentionally make the target red until their matching fix is applied; restore the full baseline plus new tests before each fix commit is considered green.

---

# 33. Performance and efficiency acceptance

These fixes should improve semantic precision without creating pathological hot-path overhead.

## 33.1 Status fields

Adding one compact status enum to `TypedExpression` and `CallCheckResult` is acceptable. It replaces reconstruction work and preserves information already produced by the analyzer.

Do not attach unbounded diagnostic/proof graphs to every typed expression.

## 33.2 Relation execution and application

`RelationApplication` is a small wrapper around the already-computed bounded `RelationOutcome` plus optional owning diagnostic cause. It must not trigger a second relation computation.

The checker should consume one logical query budget rather than constructing a default budget per relation. This improves correctness and makes worst-case relation work genuinely bounded at the callable/query level.

A critical performance requirement is:

```text
compute bounded relation once using shared query control
consume structured outcome many ways
```

not:

```text
compute unbounded/default-budget bool check
then recompute relation to recover detail/status
```

If a shared mutable budget cell is used so nested checking contexts share counters, keep the borrow narrow around each bounded relation call; do not hold runtime mutable borrows across recursive expression analysis.

## 33.3 Flow joins

Invariant validation adds equality comparisons on declaration-stable binding properties already traversed at join. This is negligible relative to type-knowledge union/join work.

Do not clone unbounded vectors of contracts on every normal join; `FlowJoinFailure` only needs detailed incoming contracts on the exceptional invariant-failure path.

## 33.4 Flow summaries

The expanded flow summary increases product size. Keep it compact and semantic:

- no full provenance vectors unless downstream product identity truly depends on them;
- no source ranges unless the summary's product contract requires them;
- no binding version counters in semantic identity;
- use existing compact `TypeKnowledge`, contract, consistency, denotation, causal summaries.

Measure callable-body product fingerprint time if the benchmark/test harness already provides it. Do not introduce a new benchmark framework solely for this patch.

## 33.5 Generic solver

Capturing the actual failed upper is O(upper bounds) exactly as current checking already is. Do not add an extra full upper-bound pass if the existing loop can capture the failed value directly.

Missing-kind lookup should remain O(number of inference vars) with the current vector implementation; a map/index optimization is out of scope unless profiling shows this path dominates.

---

# 34. Incremental correctness acceptance

The following differential properties must hold after Slice D:

1. cold and incremental analysis of identical final source states publish semantically equivalent callable/expression/binding products;
2. changing only evidence strength (`Established` -> `Assumed`) invalidates dependent semantic products where that strength is observable;
3. changing only a raw diagnostic cause allocator number does not invalidate semantic products;
4. changing analysis status (`Ready` -> `Suppressed`/`Invalid`) changes product identity;
5. changing binding contract/consistency changes product identity;
6. range-only/source-presentation behavior remains consistent with the existing input-vs-product fingerprint architecture.

Do not claim incrementality is fixed based solely on unit hashes; include at least one cold-vs-incremental product-stability test using the existing DB/session test harness.

---

# 35. Non-goals

This repair series does **not** require:

- completing every currently failing `semantic_capabilities` fixture;
- redesigning the language's unannotated-binding monomorphism policy;
- changing the runtime type-object model;
- redesigning higher-kinded type syntax;
- replacing union-find/inference architecture;
- completing Part 2 advisory migration;
- completing Part 3 compiler/LSP lifecycle takeover;
- introducing full theorem-proof objects into hot semantic state;
- optimizing code generation from established evidence;
- rewriting the semantic database.

If one of these becomes necessary to satisfy a specific regression, stop and document why rather than silently widening scope.

---

# 36. Completion criteria by bug

| Defect | Required completion evidence |
|---|---|
| causal invalidity -> suppression | downstream invalid-but-analyzable expression is `Ready + causal`; genuine suppression still works |
| `TypedExpression` lacks status | internal carrier preserves status through publication and conversion |
| relation bool collapse | bounded terminal relation matrix survives recursive relation engine and checker consumer boundaries |
| binding terminal collapse | cancellation/budget/internal/uncertain are not relabeled as unrelated block reasons |
| return summary destroys Dynamic/Unknown | direct return summary regressions pass |
| assignment current fact becomes context | contract-free known current does not create expected RHS type |
| divergent contract erasure | same-ID divergence produces explicit invariant failure, never `contract=None` |
| widening stale consistency | widened current is reconciled against contract |
| wrong generic upper evidence | second/third failing upper tests report actual bound |
| kind fallback to `Type` | zero solver `unwrap_or(KindId::TYPE)` fallbacks; malformed metadata is explicit |
| call terminal status missing | real shared budget/cancellation and injected terminal policy both reach call products distinctly |
| callable parameter provenance | body parameter is assumed from callable contract |
| return contract as value knowledge | body context uses explicit return contract/context representation |
| exact return origin collapse | constructor/native/ordinary/generic origins are distinguished |
| unrestricted `TypeEvidence` | raw production struct minting eliminated outside trusted boundary |
| duplicate declared mirrors | contract is single authoritative persistent type source |
| order-sensitive reason join | deterministic/commutative reason result or explicitly deferred ratified design |
| flow summary fingerprint loss | epistemic flow-state changes alter semantic fingerprint |
| raw diagnostic cause hash | cause renumbering alone preserves `module_diagnostics_product_fingerprint`; diagnostic input-fingerprint behavior is explicitly classified |

---

# 37. Final acceptance checklist

The implementation is complete only when all applicable items below are true.

## Semantic result model

- [ ] `TypedExpression` carries analysis status end-to-end.
- [ ] `CallCheckResult` carries analysis status end-to-end.
- [ ] `Ready + CausalInvalidity::One/Multiple` is representable and tested.
- [ ] suppression is assigned only when a required premise is unavailable because of upstream invalidity.
- [ ] dynamic boundaries are not reconstructed from mere absence of a `TypeId`.

## Relation and binding judgments

- [ ] recursive bounded relation composition preserves cancellation/budget/internal terminal outcomes.
- [ ] `CheckingContext` consumes the callable/query budget and cancellation token for formal relations.
- [ ] relation enforcement no longer returns a lossy boolean contract.
- [ ] `checker/policy.rs` no longer duplicates boolean laundering.
- [ ] `DynamicBoundaryObligation` survives every formal checker path.
- [ ] every structured relation terminal variant has a documented consumer mapping.
- [ ] binding reconciliation preserves terminal outcome identity.
- [ ] refuted contracts preserve actual current knowledge.

## Callable returns and calls

- [ ] normal return summary preserves full `TypeKnowledge` variants.
- [ ] call invalidity can coexist with independent fixed result knowledge.
- [ ] cancellation/budget/blockage can reach call expression status.
- [ ] stale constructor `CallableSignature` test oracles have been deliberately migrated to `ConstructorSemantics`.
- [ ] constructor/native/ordinary/generic return provenance is correct.

## Binding and flow

- [ ] assignment expected context comes only from persistent contract/context.
- [ ] same-binding contract disagreement fails closed.
- [ ] all three expression join callers and the statement `for` join publish invariant failure explicitly.
- [ ] callable analysis can publish internal flow-invariant failure.
- [ ] same-binding mutability disagreement fails closed.
- [ ] widening reconciles widened current knowledge against contract.
- [ ] no flow operation overwrites current with contract merely to converge.
- [ ] contract is the single authoritative persistent binding type source.

## Generic inference

- [ ] actual failed upper bound is reported.
- [ ] no missing-kind fallback to `Type` remains in solver internals.
- [ ] support weakening behavior remains green.
- [ ] fixed-return independence remains green.
- [ ] expected-result context does not fabricate value support.

## Evidence authority

- [ ] production code cannot arbitrarily construct established `TypeEvidence`.
- [ ] pattern/structural derivations preserve status through trusted helpers.
- [ ] body parameter assumptions use callable-contract provenance.

## Incrementality

- [ ] flow summary retains epistemically relevant state.
- [ ] `Established(Int)` and `Assumed(Int)` flow summaries differ semantically.
- [ ] raw diagnostic cause-ID renumbering does not change the module diagnostics semantic product fingerprint.
- [ ] analysis status and causal shape changes do change fingerprint where observable.
- [ ] cold/incremental differential tests pass.

## Repository verification

- [ ] all new tests were observed failing before production fixes.
- [ ] focused RED tests fail for semantic reasons, not parser/setup errors.
- [ ] focused GREEN tests pass after each slice.
- [ ] `cargo check -p phalcom-semantic` passes.
- [ ] full `phalcom-semantic` test suite passes except explicitly classified unrelated baseline failures, if any.
- [ ] `phalcom-lsp` compatibility suite has no new failures.
- [ ] formatting/diff checks pass for changed files.
- [ ] no new clippy warnings are introduced.
- [ ] audit searches are clean or every surviving hit is explicitly classified.

---

# 38. Handoff note to the implementation agent

Do not begin by editing `expression.rs` or `flow/state.rs` directly.

First run the current focused semantic test suite and record the baseline. Then begin with Slice A0 bounded-relation regressions, not expression.rs edits. Verify that nested budget/cancellation tests fail for the specific terminal-collapse reasons documented here before wiring checker control. After A0 is green, add the Slice A1 status/suppression regressions and observe them RED before production changes. The highest-risk failure mode in this repair is producing code that makes the final `TypeId` look right while still losing status, causality, provenance, or terminal outcome information. Tests must therefore assert complete semantic products at the boundaries that matter.

The intended result of this series is not a new type checker architecture. It is the existing Part 1 architecture made internally consistent: facts remain facts, contracts remain contracts, status remains status, causality remains causality, terminal outcomes remain terminal outcomes, and incremental identity reflects semantic meaning rather than incidental representation.

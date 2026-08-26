# Phalcom Semantic Correctness Program — Technical Specification 01
## Formal Knowledge, Required-Premise Composition, and Expression-Result Integrity

> **Status:** Implementation specification  
> **Intended path:** `docs/impl/semantic/semantic-correctness/technical/01-formal-knowledge-and-expression-composition-implementation-spec.md`  
> **Repository:** `aureat/phalcom-lang`  
> **Verified baseline:** `main` at `6ced2afd83ee89d2a09f45b8ba3821482abf3752`  
> **Implementation discipline:** RED regression first; production change second; focused test pass; semantic suite pass.  
> **Depends on:** the normative semantic-analyzer specifications for type knowledge/evidence and analysis status/causality.  
> **Does not implement:** canonical callable application or generic proof integrity. Those are Technical Specs 02 and 03. This document does define the expression-composition contracts those later implementations must consume.

---

## 1. Purpose

This specification completes the first technical slice of the semantic-correctness program.

The repository already has the correct high-level semantic model:

```text
TypeKnowledge
AnalysisStatus
CausalInvalidity
```

are independent dimensions.

`TypeKnowledge` already distinguishes `Known`, `Unknown`, and `Dynamic`; known evidence distinguishes `Established` and `Assumed`; unknown-reason joining is deterministic; `TypedExpression` now carries first-class status and causal invalidity; bounded relation outcomes are preserved at the checker boundary.

The remaining problem is not primarily the representation. It is that several expression producers do not consistently use that representation.

The current analyzer still contains local patterns equivalent to:

```rust
if let Some(ty) = operand.knowledge.ty() {
    collected.push(ty);
}
```

which means:

```text
known required operand    -> participates
unknown required operand  -> silently disappears
dynamic required operand  -> silently disappears
```

List, set, and map literals currently behave this way. Tuple and record literals fail closed when a direct component is unknown, but replace the real reason with `Unknown(UncheckedExpression)` and still ignore expansions.

There are related leaks in statement/pattern transfer:

- a `for` iterable with `Unknown(UnresolvedName(...))` becomes `Unknown(UnannotatedDeclaration)`;
- any dynamic iterable is rewritten to `Dynamic(RuntimeReflection)`;
- tuple pattern decomposition rewrites unknown or dynamic parent knowledge into fresh `Unknown(NoTypeEvidence)`;
- loop pattern bindings discard the iterable expression's causal invalidity;
- binding-initializer mismatch can set `AnalysisStatus::Invalid(C)` on the initializer while failing to insert `C` into the initializer's `causal_invalidity`.

This specification replaces those local policies with canonical implementation primitives.

The target is:

```text
one epistemic composition implementation
+
one required-expression dependency implementation
+
atomic expression publication
+
explicit projection/decomposition operations
```

Every expression producer must either use those primitives or explicitly justify why its result is independent of one of its operands.

---

# 2. Required outcome

After this specification is implemented, the following classes of mistakes must be structurally difficult to write.

This:

```rust
if let Some(ty) = child.knowledge.ty() {
    types.push(ty);
}
```

must no longer be an acceptable implementation of a required-premise operation.

This:

```rust
TypeKnowledge::Unknown(_) =>
    TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)
```

must no longer be an acceptable decomposition rule.

This:

```rust
typed.status = AnalysisStatus::Invalid(cause);
```

without updating causal invalidity must no longer be the normal API.

And this:

```rust
ctx.record_expression(... status ...);
analysis.causal_invalidity = typed.causal_invalidity;
ctx.expressions.insert(...);
```

must no longer publish an expression through several independently mutable fields.

The implementation must make these states canonical:

```text
[1, missing]

knowledge = Unknown(UnresolvedName("missing"))
status = Ready
causalInvalidity = Clean
```

```text
[establishedInt, assumedInt]

knowledge = Assumed(List<Int>)
status = Ready
```

```text
[invalidButKnownInt, 2]

knowledge = Established(List<Int>)
status = Ready
causalInvalidity = One(C1)
```

and, when a required component lost its type because an upstream invalid operation made the premise unavailable:

```text
[invalidUnknownOperand, 2]

knowledge = Unknown(SuppressedByInvalidCause)
status = Suppressed(...)
causalInvalidity = One(C1)
```

The last two examples deliberately demonstrate the distinction:

```text
causally invalid required input with usable type
    !=
required input whose type is unavailable because of invalidity
```

That distinction is already normative in the analyzer specification.

---

# 3. Scope

This technical slice owns five implementation concerns:

1. canonical required-knowledge composition;
2. canonical required-expression state propagation;
3. atomic expression-product publication and status/cause coherence;
4. aggregate literal and decomposition correctness;
5. iteration/pattern-transfer preservation.

It also adds verification gates for behavior that current `main` already implements correctly:

- stable Unknown joining;
- weakest-evidence flow joins;
- normal return summarization;
- flow contract reconciliation;
- loop widening.

Those existing behaviors must not be rewritten unnecessarily. `join_type_knowledge()` already implements the relevant flow-alternative algebra, including Unknown absorption, Dynamic preservation, and Assumed weakening. `normal_return_summary()` delegates directly to it.

---

# 4. Non-goals

This specification must not expand itself into the next semantic-correctness slices.

Do not implement here:

- canonical operator/call argument checking;
- subscript argument checking;
- setter argument checking;
- generic inference integration;
- generic receiver constraint specialization;
- advisory lookup;
- source/formal identity repair;
- module transactions;
- semantic workspace transactions.

Those areas consume the primitives introduced here, but they get separate implementation specifications.

In particular:

```phalcom
1 + "hello"
```

remains Technical Spec 02 even though this spec provides the result-integrity APIs the call path will later use.

---

# 5. Current repository architecture to preserve

## 5.1 `TypeKnowledge` is already the canonical epistemic carrier

`phalcom-semantic/src/types/evidence.rs` currently provides:

```rust
pub enum TypeKnowledge {
    Known(TypeEvidence),
    Unknown(UnknownReason),
    Dynamic(DynamicReason),
}
```

and:

```rust
pub enum EvidenceStatus {
    Established,
    Assumed,
}
```

`TypeEvidence` fields are private, so ordinary external code cannot directly mint arbitrary evidence internals. `map_type()` also correctly preserves evidence status, origin, and provenance during a pure canonical type transformation.

Do not redesign this representation.

---

## 5.2 `join_type_knowledge()` is a flow-alternative join, not a universal composition operation

Current code correctly implements:

```text
Known(A) + Known(B)
    -> Known(A | B)

Known + Unknown
    -> Unknown

Known + Dynamic
    -> Dynamic
```

with the weakest known evidence strength and stable Unknown/Dynamic reason selection.

This operation answers:

> What proposition is true after execution may have arrived through any of these reachable alternatives?

It does not answer:

> What proposition can I establish for a product requiring all of these components?

The second operation needs a distinct name and implementation.

Do not overload or rename `join_type_knowledge()` to perform both jobs.

---

## 5.3 `TypedExpression` already carries the correct dimensions

Current:

```rust
pub struct TypedExpression {
    pub expression_id: Option<ExpressionId>,
    pub callable: Option<CallableId>,
    pub explanation_parents: Vec<ExplanationId>,
    pub knowledge: TypeKnowledge,
    pub status: AnalysisStatus,
    ...
    pub causal_invalidity: CausalInvalidity,
}
```

is the correct basic carrier.

This specification strengthens its mutation/publication APIs rather than replacing it.

---

# 6. New implementation boundary: required knowledge composition

## 6.1 Add one canonical function

Modify:

```text
phalcom-semantic/src/types/evidence.rs
```

Add:

```rust
pub(crate) fn compose_required_knowledge(
    inputs: impl IntoIterator<Item = TypeKnowledge>,
    origin: EvidenceOrigin,
    build_type: impl FnOnce(&[TypeId]) -> Result<TypeId, UnknownReason>,
) -> TypeKnowledge
```

The name `compose_required_knowledge` is intentional.

Do not call it:

```text
join
merge
unify
infer
```

because those words already have different meanings.

Its semantic meaning is:

> The result proposition requires every supplied input proposition.

---

## 6.2 Required algorithm

The function must first collect all inputs.

It must then perform these phases in this exact semantic order.

### Phase A — Unknown detection

If one or more inputs are `Unknown`, the composite is Unknown.

If several are Unknown, combine their reasons using the same deterministic Unknown reason algebra already used by `join_type_knowledge()`.

Do not independently reimplement reason precedence.

Refactor the existing private reason-reduction implementation only as much as necessary to reuse it.

Conceptually:

```rust
let unknown_reason = inputs
    .iter()
    .filter_map(...)
    .reduce(join_unknown_reason);

if let Some(reason) = unknown_reason {
    return TypeKnowledge::Unknown(reason);
}
```

Example:

```text
Established(Int)
Unknown(UnresolvedName("x"))
Established(String)
```

produces:

```text
Unknown(UnresolvedName("x"))
```

The known inputs must not survive as a partial composite proposition.

---

## 6.3 Dynamic detection

After proving that no required input is Unknown, detect Dynamic.

If any required input is Dynamic:

```text
composite = Dynamic(stable joined dynamic reason)
```

Example:

```text
Established(Int)
Dynamic(RuntimeReflection)
```

does not establish a concrete tuple/list/record containing the dynamic component.

The current type universe has no canonical “Dynamic component type” to place inside such a product.

Do not substitute:

```text
Object
Any
expected type
first known member
```

---

## 6.4 Known composition

Only if every input is `Known` may `build_type()` run.

Collect every `TypeId` in source/component order.

Then:

```rust
let final_ty = build_type(&types)?;
```

The result evidence status is:

```text
Established iff every required input is Established
Assumed otherwise
```

Specifically:

| Inputs | Result status |
|---|---|
| Established + Established | Established |
| Established + Assumed | Assumed |
| Assumed + Assumed | Assumed |

Agreement is not proof.

Thus:

```text
Assumed(Int)
Assumed(Int)
```

does not permit:

```text
Established(...)
```

This directly enforces the epistemic-support rule already specified normatively.

---

## 6.5 Provenance

The composite must retain the component provenance.

After constructing the result evidence:

```text
result.provenance.ranges
    += every known input provenance range

result.provenance.descriptions
    += every known input provenance description
```

Do not copy only the first operand.

Do not deduplicate ranges as part of this patch unless an existing bounded-provenance invariant requires it.

That is an optimization/presentation concern.

---

## 6.6 Result origin

The caller supplies `origin`.

For literal construction in this specification, use:

```rust
EvidenceOrigin::Syntax
```

even when the resulting evidence status is `Assumed`.

This is valid:

```text
Assumed(List<Int>, origin = Syntax)
```

because:

```text
status
```

answers how strongly the proposition is justified, while:

```text
origin
```

answers which semantic rule derived the outer proposition.

Do not change the origin to `DeveloperAnnotation` merely because an assumed child ultimately came from a declaration contract.

The child provenance/explanation graph retains that dependency.

---

## 6.7 Empty input semantics

`compose_required_knowledge()` itself must permit zero operands.

The caller decides whether its operation is typable without components.

For example:

```text
empty record
```

may have a canonical empty-record type.

But:

```phalcom
[]
```

cannot currently synthesize its element type without context.

Therefore list/set/map helpers retain explicit empty-literal handling.

Do not bake “empty means Unknown” into the generic composition primitive.

---

# 7. Unit tests for `compose_required_knowledge`

Add unit tests in:

```text
phalcom-semantic/src/types/evidence.rs
```

or a focused existing evidence test module if repository organization requires external tests.

Required tests:

### 7.1 Known established

```rust
Established(Int) + Established(String)
```

with a tuple-building closure must produce known tuple type, Established.

### 7.2 Assumption weakening

```rust
Established(Int) + Assumed(String)
```

must produce Assumed composite.

### 7.3 Two assumptions do not prove

```rust
Assumed(Int) + Assumed(Int)
```

must remain Assumed.

### 7.4 Unknown absorption

```rust
Established(Int)
Unknown(UnresolvedName("x"))
Established(String)
```

must be Unknown with the unresolved-name reason.

### 7.5 Unknown order stability

Forward and reversed required-input order must choose the same merged Unknown reason when component order is irrelevant to reason priority.

### 7.6 Dynamic absorption

Known + Dynamic must be Dynamic.

### 7.7 Builder not invoked under uncertainty

Use a closure that panics:

```rust
|_| panic!("builder must not run")
```

and prove Unknown/Dynamic inputs return without invoking it.

This test matters: the implementation must not construct a false canonical aggregate type and then overwrite the knowledge state afterward.

---

# 8. New implementation boundary: required expression dependencies

Knowledge composition alone is insufficient.

Each operand also contributes:

```text
AnalysisStatus
CausalInvalidity
ExplanationId
```

Create:

```text
phalcom-semantic/src/checker/composition.rs
```

Register it in:

```text
phalcom-semantic/src/checker/mod.rs
```

The module should remain `pub(crate)` unless an existing external test boundary forces wider visibility.

---

# 9. `propagate_required_dependencies`

Add:

```rust
pub(crate) fn propagate_required_dependencies(
    result: &mut TypedExpression,
    operands: &[TypedExpression],
)
```

Its responsibilities are deliberately narrow:

1. join causal invalidity;
2. add child explanation parents;
3. propagate genuine terminal analysis states;
4. turn unavailable-invalid required premises into suppression;
5. not turn usable causally-invalid inputs into suppression.

It does not build types.

---

# 10. Causal invalidity propagation

Always join every required operand's causal invalidity:

```rust
result.causal_invalidity =
    operands
        .iter()
        .map(|operand| operand.causal_invalidity)
        .fold(
            result.causal_invalidity,
            CausalInvalidity::join,
        );
```

This occurs regardless of whether the result remains Known, Unknown, or Dynamic.

Example:

```text
operand:
    Established(Int)
    Ready
    One(C1)

other:
    Established(Int)
    Ready
    Clean
```

can produce:

```text
result:
    Established(List<Int>)
    Ready
    One(C1)
```

The outer collection did not create `C1`.

Therefore it is not `Invalid(C1)`.

It simply depends on a value whose upstream semantics are compromised.

---

# 11. Explanation-parent propagation

For each operand:

```rust
operand.expression_id
    -> ctx.explanation_for_expression(...)
```

or existing `explanation_parents` where appropriate.

Do not reconstruct explanation relationships by source ranges.

Do not add duplicate parent IDs.

Where the caller already has direct child explanation IDs, append those through one helper.

The exact helper may be:

```rust
pub(crate) fn push_unique_explanation(
    parents: &mut Vec<ExplanationId>,
    explanation: ExplanationId,
)
```

Do not make explanation correctness depend on the order a `HashMap` happens to enumerate.

---

# 12. Required-premise status propagation

This is the subtle part.

The parent result does not blindly copy the worst child status.

The parent's operation asks:

> Did the child leave the semantic premise required by this operation available?

The implementation must obey the following cases.

---

## 12.1 Child is Invalid but still Known

Example child:

```text
knowledge = Established(Int)
status = Invalid(C1)
causal = One(C1)
```

If the parent only requires the child's established type proposition, that required premise remains available.

The parent does not become Invalid or Suppressed solely from that child.

Parent:

```text
status = Ready
causal includes C1
```

This is the invalid-but-analyzable rule.

---

## 12.2 Child is Invalid and has no usable type

Example:

```text
knowledge = Unknown(...)
status = Invalid(C1)
causal = One(C1)
```

The parent requires a type proposition which is unavailable because of an invalid upstream operation.

Parent becomes:

```text
Suppressed(One(C1))
```

or:

```text
Suppressed(Multiple)
```

when the joined causal summary is multiple.

Do not emit another source diagnostic.

---

## 12.3 Child already Suppressed

Required dependent operation remains suppressed.

Use the joined causal summary for the parent suppression payload.

---

## 12.4 Blocked

Required child:

```text
Blocked(reason)
```

causes the dependent composition to be:

```text
Blocked(reason)
```

unless a higher-priority infrastructure terminal state also exists.

Do not translate it into Unknown-only state.

Knowledge may independently be Unknown, but the status must remain observable.

---

## 12.5 Cancelled

Any required operand whose analysis is Cancelled makes the composition Cancelled.

Do not continue building a normal Ready semantic product.

---

## 12.6 BudgetExceeded

Likewise:

```text
BudgetExceeded(report)
```

must propagate.

---

## 12.7 InternalFailure

Internal failure has highest propagation priority.

The outer operation must not publish a Ready value pretending analysis completed.

---

## 12.8 DynamicBoundary

If the required knowledge composition becomes:

```text
Dynamic(reason)
```

the normal `TypedExpression::new()` behavior already creates:

```text
DynamicBoundary(reason)
```

Keep this.

Do not overwrite a stronger infrastructure terminal state such as cancellation or internal failure with DynamicBoundary.

---

# 13. Terminal precedence

When more than one required operand has a terminal analysis state, use a deterministic precedence.

Implement this policy in one helper, not at each expression site:

```text
InternalFailure
    >
Cancelled
    >
BudgetExceeded
    >
Blocked
    >
Suppressed
    >
DynamicBoundary
    >
Ready
```

`Invalid` is not placed into that simple ranking because its propagation depends on whether the child still exposes the required proposition.

That is deliberate.

For `Blocked` ties or multiple budget reports, preserve source operand order unless the corresponding domain already exposes a canonical join operation.

Do not sort semantic failures by formatted debug strings unless the normative reason algebra specifically requires order independence.

Source operand order is deterministic and semantically meaningful for evaluation.

---

# 14. Add status/cause integrity operations to `TypedExpression`

Modify:

```text
phalcom-semantic/src/checker/typed_expr.rs
```

Add:

```rust
pub(crate) fn invalidate(
    &mut self,
    cause: DiagnosticCauseId,
)
```

with implementation semantics:

```rust
self.status = AnalysisStatus::Invalid(cause);
self.causal_invalidity = self
    .causal_invalidity
    .join(CausalInvalidity::One(cause));
```

There should be no normal production site that performs only:

```rust
typed.status = AnalysisStatus::Invalid(cause);
```

after this method exists.

---

# 15. Add `CausalInvalidity::contains`

Modify:

```text
phalcom-semantic/src/checker/causal.rs
```

Add:

```rust
pub fn contains(self, cause: DiagnosticCauseId) -> bool
```

Semantics:

```rust
Clean       -> false
One(actual) -> actual == cause
Multiple    -> true
```

`Multiple` is deliberately cardinality-bounded; it means the exact individual IDs are not retained in the hot state.

This helper exists to verify representation coherence, not to recover exact cause sets.

---

# 16. Add expression integrity assertion

In `TypedExpression`:

```rust
pub(crate) fn debug_assert_coherent(&self)
```

At minimum:

```rust
if let AnalysisStatus::Invalid(cause) = self.status {
    debug_assert!(
        self.causal_invalidity.contains(cause),
        "Invalid expression status must include its owning cause"
    );
}
```

For:

```rust
AnalysisStatus::Suppressed(...)
```

assert causal invalidity is non-clean.

Do not assert:

```text
Ready => causalInvalidity == Clean
```

because that would violate the central recovery model.

---

# 17. Atomic expression publication

Current `analyze_expression()` constructs an `ExpressionAnalysis`, then mutates `causal_invalidity`, inserts it, possibly mutates `explanation`, and inserts it again.

This is unnecessary split publication.

Replace it.

---

## 17.1 Replace piecemeal `record_expression`

In:

```text
phalcom-semantic/src/checker/context.rs
```

introduce:

```rust
pub(crate) fn publish_expression_analysis(
    &mut self,
    id: ExpressionId,
    range: SourceRange,
    typed: &TypedExpression,
    explanation: Option<ExplanationId>,
) -> ExpressionAnalysis
```

This method constructs the complete product in one place:

```rust
let analysis = ExpressionAnalysis {
    id,
    range,
    knowledge: typed.knowledge.clone(),
    callable: typed.callable.clone(),
    denotation: typed.denotation,
    status: typed.status.clone(),
    causal_invalidity: typed.causal_invalidity,
    explanation,
    call: None, // preserve existing call plumbing if populated elsewhere
};
```

If existing call-resolution product population requires `call`, include it in the helper rather than dropping it.

The executor must inspect current call-resolution ownership before finalizing the signature.

The important requirement is atomicity of the fields in this specification, not removal of existing metadata.

Call:

```rust
typed.debug_assert_coherent();
```

before insertion.

---

## 17.2 Add post-analysis synchronization

Some judgments legitimately happen after expression synthesis.

Binding initializer reconciliation is one such case.

Add:

```rust
pub(crate) fn sync_expression_outcome(
    &mut self,
    typed: &TypedExpression,
)
```

Require `typed.expression_id`.

Update at least:

```text
knowledge
status
causal_invalidity
callable
denotation
```

where the corresponding product is designed to reflect mutable post-analysis judgment.

Do not update explanation identity merely because status changes unless a new explanation node is explicitly allocated.

Run coherence assertion before synchronization.

---

# 18. Rewrite `analyze_expression()` publication flow

Target structure:

```rust
pub fn analyze_expression(
    ctx: &mut CheckingContext<'_>,
    expr: &Expr,
    expected: &ExpectedType,
) -> TypedExpression {
    let expr_id = ctx.alloc_expression_id();
    ctx.push_expression_owner(expr_id);

    let mut typed =
        analyze_expression_inner(ctx, expr, expected);

    if let Some(cause) =
        ctx.pop_expression_owner(expr_id)
    {
        typed.invalidate(cause);
    }

    typed.expression_id = Some(expr_id);

    let explanation =
        build_expression_explanation(...);

    ctx.record_call_dependency(
        typed.causal_invalidity,
        explanation,
    );

    ctx.publish_expression_analysis(
        expr_id,
        expr.range(),
        &typed,
        explanation,
    );

    typed
}
```

Do not retain two independent variables for:

```text
typed.status
published status
```

The `TypedExpression` is the immediate semantic result.

`ExpressionAnalysis` is its published immutable body product.

---

# 19. Repair binding-initializer invalidity

Current `Statement::Let` performs:

```rust
causal_invalidity =
    causal_invalidity.join(One(cause));

val_typed.status =
    AnalysisStatus::Invalid(cause);
```

but later copies:

```rust
analysis.causal_invalidity =
    val_typed.causal_invalidity;
```

which does not contain the new cause.

Replace the mismatch branch with:

```rust
val_typed.invalidate(cause);
ctx.sync_expression_outcome(&val_typed);

let causal_invalidity =
    val_typed
        .causal_invalidity
        .join(annotation_invalidity);
```

Do not separately maintain two causal values before this point.

The binding's causal invalidity may additionally include annotation invalidity because the binding judgment depends on both.

The initializer expression itself must not inherit annotation invalidity unless its own expression analysis actually depended on the annotation through contextual typing in a way represented elsewhere.

---

# 20. Collection literal migration

Modify:

```text
phalcom-semantic/src/checker/expression.rs
```

The helpers:

```text
synthesize_list_literal
synthesize_set_literal
synthesize_map_literal
synthesize_tuple_literal
synthesize_record_literal
```

must all follow one pattern:

```text
analyze children
    ↓
derive each child's contribution knowledge
    ↓
compose_required_knowledge
    ↓
construct TypedExpression
    ↓
propagate_required_dependencies
```

Do not let each helper implement its own evidence algebra.

---

# 21. List literals

Current implementation collects only `TypeId`s from known elements and ignores unknown elements and expansions.

Replace:

```rust
let mut elem_tys = Vec::new();
```

with:

```rust
let mut operands = Vec<TypedExpression>;
let mut contributions = Vec<TypeKnowledge>;
```

For an ordinary element:

```rust
let typed =
    analyze_expression(ctx, expr, &expected_elem);

contributions.push(typed.knowledge.clone());
operands.push(typed);
```

Then:

```rust
let knowledge = compose_required_knowledge(
    contributions,
    EvidenceOrigin::Syntax,
    |element_types| {
        if element_types.is_empty() {
            return Err(UnknownReason::NoTypeEvidence);
        }

        let elem_ty =
            ctx.store.union(element_types);

        ctx.store
            .list_of(list_form, elem_ty)
            .map_err(|_| UnknownReason::UncheckedExpression)
    },
);
```

Finally:

```rust
let mut result = TypedExpression::new(
    knowledge.with_range(list.range)
);

propagate_required_dependencies(
    &mut result,
    &operands,
);

result
```

If `with_range()` only affects Known evidence, preserve current behavior for Unknown/Dynamic.

---

# 22. Required list examples

### 22.1 Homogeneous established

```phalcom
[1, 2]
```

Result:

```text
Established(List<Int>)
Ready
Clean
```

### 22.2 Heterogeneous established

```phalcom
[1, "x"]
```

Result:

```text
Established(List<Int | String>)
Ready
Clean
```

assuming canonical union behavior.

### 22.3 Assumed component

```phalcom
class Probe {
    @class
    run(_ value: Int) {
        let xs = [1, value]
    }
}
```

`value` is a callable-contract assumption.

Result:

```text
Assumed(List<Int>)
```

It must not become Established because the two `TypeId`s happen to agree.

### 22.4 Unknown component

```phalcom
[1, missing]
```

Result:

```text
Unknown(UnresolvedName("missing"))
```

not:

```text
Established(List<Int>)
```

### 22.5 Dynamic component

Dynamic required component makes the list Dynamic unless a later collection semantics explicitly supplies an independent static element proposition.

### 22.6 Invalid-but-known component

If child is:

```text
Established(Int)
Invalid(C1)
One(C1)
```

result may remain:

```text
Established(List<Int>)
Ready
One(C1)
```

No duplicate diagnostic.

---

# 23. Set literals

Use exactly the same evidence algorithm as lists.

The only shape difference is:

```rust
ctx.store.set_of(form, elem_ty)
```

Do not copy/paste the epistemic loop.

Share a local helper for homogeneous collection construction if this can be done without hiding syntax-specific behavior.

Possible private helper:

```rust
fn synthesize_homogeneous_literal(
    ctx: &mut CheckingContext<'_>,
    operands: Vec<TypedExpression>,
    contributions: Vec<TypeKnowledge>,
    range: SourceRange,
    build: impl FnOnce(
        &mut TypeStore,
        TypeId,
    ) -> Result<TypeId, UnknownReason>,
) -> TypedExpression
```

Do not introduce this abstraction if Rust borrowing makes the implementation materially harder.

The required abstraction is the knowledge/state algebra, not necessarily one literal-builder function.

---

# 24. Map literals

Map literals have two independent required type lanes:

```text
key
value
```

Every association contributes to both aggregate proofs.

Maintain:

```rust
let mut operands = Vec<TypedExpression>;
let mut key_knowledge = Vec<TypeKnowledge>;
let mut value_knowledge = Vec<TypeKnowledge>;
```

For computed keys:

```rust
let key_typed =
    analyze_expression(ctx, expr, &expected_key);

key_knowledge.push(
    key_typed.knowledge.clone()
);

operands.push(key_typed);
```

For bare-symbol keys, construct a synthetic established key knowledge from canonical `String` semantics if that remains the language rule.

Do not pretend a missing canonical `String` declaration is a known key.

For values:

```rust
let value_typed =
    analyze_expression(ctx, value, &expected_val);
```

Store both knowledge and expression dependency.

First compose keys:

```text
key_result
```

Then compose values:

```text
value_result
```

Then compose those two required results into `Map<K,V>`.

A simpler implementation is permitted:

1. determine whether either lane contains Unknown;
2. determine whether either contains Dynamic;
3. only construct `Map<K,V>` after both lanes are Known.

But do not write a second epistemic policy. Reuse `compose_required_knowledge()`.

---

# 25. Map failure examples

These must be explicitly tested separately:

```phalcom
#{ missing: 1 }
```

if bare symbol semantics do not refer to a variable is not an unknown-key case.

Use a computed key for the actual test:

```phalcom
#{ [missing]: 1 }
```

with the repository's real computed-key syntax.

Also test:

```phalcom
#{ "a": missing }
```

Both must fail closed.

A known key lane must not allow an unknown value lane to disappear, and vice versa.

---

# 26. Tuple literals

Tuple composition differs from list composition because component types stay positional rather than being unioned.

Current tuple synthesis does fail closed for direct unknown elements, but collapses every non-known reason to:

```text
Unknown(UncheckedExpression)
```

and ignores expansions.

Rewrite direct components using `compose_required_knowledge()`.

The builder receives types in source order and constructs:

```rust
TupleTypeElement {
    label,
    ty,
}
```

The labels must be collected separately from evidence.

Do not encode labels into the evidence-composition function.

---

# 27. Tuple direct component behavior

For:

```phalcom
(1, missing)
```

publish:

```text
Unknown(UnresolvedName("missing"))
```

For:

```phalcom
(1, assumedValue)
```

publish an Assumed tuple.

For:

```phalcom
(1, dynamicValue)
```

publish Dynamic.

---

# 28. Record literals

Record behavior mirrors tuple product construction.

Current record synthesis likewise rewrites any non-known direct field value to `Unknown(UncheckedExpression)` and ignores expansions.

Collect:

```text
field metadata
field TypeKnowledge
field TypedExpression
```

separately.

Only invoke:

```rust
ctx.store.record(...)
```

after every required field value is Known.

For:

```phalcom
#{ name: missing }
```

or the canonical record syntax in the current parser, preserve:

```text
Unknown(UnresolvedName("missing"))
```

Do not publish a record omitting the field.

---

# 29. Expansion entries: correctness-first policy

The current AST exposes expansion forms in list, set, map, tuple, and record literal helpers, while those helpers currently analyze expansion expressions and then discard the result.

That is forbidden after this patch.

This technical spec does not authorize inventing new spread semantics.

Therefore the implementation follows:

> Precisely project an expansion only when the canonical static type representation proves what it contributes. Otherwise fail closed.

This is intentionally conservative.

---

# 30. Exact homogeneous collection projection

Add private helpers in:

```text
checker/composition.rs
```

such as:

```rust
pub(crate) fn project_applied_argument(
    store: &TypeStore,
    knowledge: &TypeKnowledge,
    expected_origin: TypeId,
    argument_index: usize,
) -> TypeKnowledge
```

Rules:

For:

```rust
TypeKnowledge::Known(...)
```

and exact:

```rust
TypeData::Applied {
    origin == expected_origin,
    arguments,
}
```

project the requested argument using `map_type()` so status, origin and provenance are preserved.

For Unknown:

```text
preserve exact Unknown(reason)
```

For Dynamic:

```text
preserve exact Dynamic(reason)
```

For Known but statically non-projectable type:

```text
Unknown(UncheckedExpression)
```

unless an existing language-semantic diagnostic explicitly proves the expansion operand invalid.

This is a legitimate new downstream Unknown reason because the operand itself is known; what is missing is a sound static projection for this operation.

Do not reuse the operand's unrelated type as the element type.

---

# 31. List/set/map expansion

For a known exact:

```text
List<Int>
```

list expansion may contribute:

```text
Int
```

to list element inference.

Likewise:

```text
Set<T> -> T
Map<K,V> -> K and V
```

for exact same-form expansions.

This is only the statically obvious case.

Do not infer arbitrary iterable expansion from the first generic argument.

If current language semantics permit broader cursor-protocol expansion, that precision requires an explicit protocol projection operation and must use canonical dispatch. Do not reproduce iterable inference ad hoc inside the literal helper.

Until that exists:

```text
known non-exact expansion operand
    -> Unknown(UncheckedExpression)
```

is acceptable and sound.

---

# 32. Tuple expansion

If the expansion operand has exact:

```rust
TypeData::Tuple(elements)
```

splice its canonical element types into the output shape.

Every spliced component inherits the expansion operand's evidence strength.

If expansion knowledge is:

```text
Assumed((Int, String))
```

then the resulting tuple can be at most Assumed.

If Unknown or Dynamic, preserve that state for the whole required composition.

If known non-tuple, fail closed.

---

# 33. Record expansion

Only statically decompose:

```rust
TypeData::Record(row_id)
```

when:

```rust
store.record_row(row_id).tail
    == RecordRowTail::Closed
```

The current row model explicitly distinguishes closed rows from parameter/open rows.

For a closed row, incorporate every canonical field.

For an open row:

```text
Unknown(UncheckedExpression)
```

because unknown tail fields prevent the analyzer from claiming an exact closed record product.

Do not drop the tail.

Do not convert the row to a closed record.

Duplicate field resolution remains governed by the language's record semantics; do not silently choose source-first or source-last inside this correctness patch.

If the current canonical record builder rejects duplicate fields, preserve that failure instead of inventing override semantics.

---

# 34. Pattern decomposition

Modify in:

```text
phalcom-semantic/src/checker/statement.rs
```

both:

```text
bind_declaration_pattern
bind_pattern
```

The current pattern:

```rust
match fact.knowledge {
    Known(tuple) => ...
    _ => None,
}

...
.unwrap_or_else(|| {
    Unknown(NoTypeEvidence)
})
```

destroys information.

Replace it with an explicit decomposition helper.

---

# 35. Add `decompose_tuple_component`

Prefer a helper in:

```text
checker/composition.rs
```

with semantics equivalent to:

```rust
pub(crate) fn decompose_tuple_component(
    store: &TypeStore,
    parent: &TypeKnowledge,
    index: usize,
    expected_len: usize,
) -> TypeKnowledge
```

Rules:

### Known exact tuple, correct arity

Return the component type using a pure type transformation preserving epistemic strength and provenance.

Change semantic origin to:

```rust
EvidenceOrigin::PatternDecomposition
```

only if the current provenance model expects the new derivation step to become primary.

The current code already intentionally uses `PatternDecomposition` for successful tuple decomposition; preserve that behavior.

### Unknown parent

Preserve exact reason:

```text
Unknown(R) -> Unknown(R)
```

### Dynamic parent

Preserve:

```text
Dynamic(D) -> Dynamic(D)
```

### Known non-tuple

Use:

```text
Unknown(UncheckedExpression)
```

unless this patch also adds a canonical pattern-shape diagnostic.

Do not claim `NoTypeEvidence`: evidence exists; it proves the parent is the wrong/non-decomposable form.

### Known tuple with wrong arity

Likewise fail closed.

A later pattern-diagnostics project may refine this to an explicit contradiction diagnostic.

---

# 36. Causal invalidity through pattern decomposition

Current recursive binding helpers accept no child causal invalidity in ordinary `bind_pattern()`, and `bind_pattern_binding()` always creates a `BindingSeed` with `Clean`.

Add:

```rust
pub fn bind_pattern_binding_with_causal(
    &mut self,
    name: impl Into<String>,
    fact: ValueSemanticFact,
    range: SourceRange,
    causal_invalidity: CausalInvalidity,
) -> BindingDeclarationResult
```

Then retain compatibility:

```rust
pub fn bind_pattern_binding(...) {
    self.bind_pattern_binding_with_causal(
        ...,
        CausalInvalidity::Clean,
    )
}
```

This keeps existing call sites source-compatible.

Change recursive `bind_pattern()` to accept causal invalidity:

```rust
fn bind_pattern(
    ctx: &mut CheckingContext<'_>,
    pattern: &Pattern,
    fact: ValueSemanticFact,
    causal_invalidity: CausalInvalidity,
)
```

All child pattern bindings inherit the parent value's causal dependency.

Do not convert causal invalidity into Invalid status.

The pattern binding did not necessarily create the upstream contradiction.

---

# 37. `for` iterable transfer

Current code:

```rust
let iter_k =
    synthesize_expr(ctx, &lane.iter);
```

loses status and causal information immediately.

Replace it with:

```rust
let iter_typed =
    analyze_expression(
        ctx,
        &lane.iter,
        &ExpectedType::None,
    );
```

Then derive element knowledge by matching the complete knowledge state.

---

# 38. Preserve Unknown exactly in `for`

Replace:

```rust
else {
    Unknown(UnannotatedDeclaration)
}
```

with:

```rust
TypeKnowledge::Unknown(reason) =>
    TypeKnowledge::Unknown(reason.clone())
```

Thus:

```phalcom
for x in missing {
    ...
}
```

keeps:

```text
UnresolvedName("missing")
```

It does not become “unannotated declaration.”

This is explicitly forbidden by the normative Unknown-preservation law.

---

# 39. Preserve Dynamic exactly in `for`

Replace:

```rust
if iter_k.is_dynamic() {
    Dynamic(RuntimeReflection)
}
```

with:

```rust
TypeKnowledge::Dynamic(reason) =>
    TypeKnowledge::Dynamic(reason.clone())
```

Do not rewrite:

```text
ExplicitEscape
DynamicRestPack
```

into:

```text
RuntimeReflection
```

unless the iteration operation itself actually introduces runtime reflection.

---

# 40. Known iterable path

For:

```rust
TypeKnowledge::Known(evidence)
```

call:

```rust
resolve_iteration_element(ctx, evidence.ty())
```

The returned `TypeKnowledge` already carries whatever evidence status the canonical iterator/member surface provides.

Do not upgrade it.

---

# 41. Causal invalidity for loop bindings

Each lane fact must carry:

```rust
iter_typed.causal_invalidity
```

into its pattern binding.

Conceptually:

```text
iterable fact
        ↓
iteration protocol projection
        ↓
loop element
        ↓
pattern binding
```

If the iterable is:

```text
Established(List<Int>)
Ready
One(C1)
```

then `x` can be:

```text
Established(Int)
...
One(C1)
```

if the iteration protocol establishes the element type.

Do not make `x` clean merely because element extraction succeeded.

---

# 42. Iteration projection's own causal state

`resolve_iteration_element()` currently returns only `TypeKnowledge`.

For this technical slice, do not redesign it into another large result carrier unless implementation demonstrates a real producer of new causal/status state inside this helper.

Instead:

```text
loop binding causal invalidity
    = iterable expression causal invalidity
```

plus any future explicit iteration-operation invalidity once such an operation exists.

Technical Spec 02 may eventually route iteration protocol use through canonical callable application; at that point this function should consume the canonical call result rather than growing a parallel semantics.

Add a comment marking that integration boundary.

---

# 43. Assignment result verification

Ordinary local assignment currently returns:

```text
Established(Unit, Syntax)
```

and carries causal invalidity separately.

Do not redesign assignment return semantics here.

Add regression coverage proving:

```phalcom
let x: Int = 1
x = "bad"
```

preserves the RHS's actual binding/current fact according to current mutation rules while the assignment operation owns its mismatch diagnostic/status.

Technical Spec 02 will audit property/index setters as canonical call-like operations.

---

# 44. Existing correct behavior: flow joins

`FlowState::join_with_hierarchy()` now:

- rejects divergent persistent binding contracts;
- rejects divergent mutability;
- joins current knowledge via `join_type_knowledge`;
- rejoins causal invalidity;
- reconciles the joined current fact against the persistent contract.

This is a verification-only area for this patch.

Do not replace it.

Add tests if coverage is insufficient, but no production refactor is required unless a new regression exposes a real defect.

---

# 45. Existing correct behavior: loop widening

Current `widen_loop_state_with_hierarchy()` now validates persistent contract/mutability and reconciles widened current knowledge against the contract.

Again: verification gate, not rewrite target.

---

# 46. Existing correct behavior: normal return summary

Current:

```rust
normal_return_summary()
```

uses `join_type_knowledge()` directly and therefore preserves Dynamic and specific Unknown reason classes.

The existing RED-regression suite already asserts preservation of Dynamic and Unknown reasons.

Keep those tests.

Do not reintroduce a “collect only known `TypeId`s” summarizer.

---

# 47. Explanation requirements

This patch must improve correctness without making diagnostics inexplicable.

For a composite expression that becomes Unknown because of a child:

```phalcom
[1, missing]
```

the outer expression need not invent a separate “list inference failed” diagnostic.

Its explanation graph should retain the child expression as a parent.

Future inference diagnostics can then explain:

```text
Could not establish List element type.

Known contribution:
  element 1 -> Int

Missing contribution:
  element 2 -> unresolved name `missing`
```

This technical spec does not require that exact UI text.

It does require retaining enough parent relationships to produce it later.

---

# 48. Do not use expected types as replacement evidence

Collection components may still receive expected/contextual types through existing bidirectional analysis.

For example:

```phalcom
let xs: List<Int> = []
```

the existing behavior is:

```text
[] itself -> Unknown(NoTypeEvidence)
binding contract -> supplies Assumed(List<Int>)
```

and the current test explicitly asserts that outcome.

Do not change this in this specification.

In particular, do not implement:

```rust
if elements.is_empty() {
    return TypeKnowledge::assumed(expected_ty, ...);
}
```

inside the list literal.

That would conflate context with evidence.

Generic/context-sensitive collection inference can be improved later through a real typing rule, but not through assignment of the expected type.

---

# 49. New integration test file

Create:

```text
phalcom-semantic/tests/semantic/correctness/
    expression_composition.rs
```

if the test organization already supports the `correctness` subdirectory.

Otherwise place the tests under the existing:

```text
phalcom-semantic/tests/semantic/foundations/
```

without restructuring the test harness as part of this patch.

Do not move unrelated tests.

Use the existing `Fixture` API, which already supports exact assertions for:

- type shape;
- knowledge state;
- EvidenceStatus;
- EvidenceOrigin;
- binding state;
- expression products;
- diagnostics;
- internal incidents.

---

# 50. Required source-level regressions

At minimum add the following.

## EPI-COMP-01 — list unknown member does not disappear

```phalcom
class Probe {
    @class
    run() {
        let xs = [1, missing]
    }
}
```

Assert outer list expression:

```text
Unknown(UnresolvedName("missing"))
```

Assert `xs.current` is likewise Unknown.

Do not merely assert “not Int.”

---

## EPI-COMP-02 — list assumed member weakens aggregate

```phalcom
class Probe {
    @class
    run(_ value: Int) {
        let xs = [1, value]
    }
}
```

Assert:

```text
xs = Assumed(List<Int>)
```

not Established.

---

## EPI-COMP-03 — set unknown member

Equivalent set case.

---

## EPI-COMP-04 — map unknown value

Use canonical map syntax.

Assert the entire map is Unknown with the child's reason.

---

## EPI-COMP-05 — map unknown computed key

Use actual parser-supported computed-key syntax.

Assert the map is Unknown.

---

## EPI-COMP-06 — tuple preserves Unknown reason

```phalcom
let x = (1, missing)
```

must publish:

```text
Unknown(UnresolvedName("missing"))
```

not `UncheckedExpression`.

---

## EPI-COMP-07 — record preserves Unknown reason

Equivalent record-field test.

---

## EPI-COMP-08 — tuple decomposition preserves Unknown reason

Create an unknown tuple-valued source and destructure it.

Every resulting component whose exact type cannot be established from the same unknown parent must preserve the parent's Unknown reason.

Do not rewrite it to `NoTypeEvidence`.

---

## EPI-COMP-09 — dynamic decomposition remains Dynamic

Bind/destructure a Dynamic source through a path the current language supports.

Assert:

```text
Dynamic(reason)
```

on decomposed bindings.

---

## EPI-COMP-10 — `for` preserves unresolved iterable reason

```phalcom
for value in missing {
    let copy = value
}
```

Assert loop binding/read retains the appropriate unresolved reason.

---

## EPI-COMP-11 — `for` preserves dynamic reason

Use a source explicitly carrying one non-runtime-reflection DynamicReason if the test harness can construct it directly.

This may be a lower-level checker test if source syntax cannot select the desired reason.

---

## EPI-COMP-12 — invalid initializer coherence

Use:

```phalcom
class CellNum {
    @constructor
    new() {}
}

class Probe {
    @class
    run() {
        let x: Int = CellNum.new()
    }
}
```

Get initializer expression:

```text
CellNum.new()
```

Assert:

```rust
let AnalysisStatus::Invalid(cause) =
    expression.status
else { ... };

assert!(
    expression
        .causal_invalidity
        .contains(cause)
);
```

This closes the concrete coherence bug in `Statement::Let`.

---

# 51. Required lower-level status tests

Source syntax will not conveniently produce every terminal state.

Add focused tests around `propagate_required_dependencies()` for:

```text
Blocked
Cancelled
BudgetExceeded
InternalFailure
Suppressed
Invalid + Known
Invalid + Unknown
```

These tests may construct `TypedExpression` values directly.

Required assertions:

### Invalid + Known

```text
parent status = Ready
parent causal includes cause
```

### Invalid + Unknown

```text
parent status = Suppressed
```

### Cancelled child

```text
parent status = Cancelled
```

### InternalFailure + Cancelled

```text
parent status = InternalFailure
```

according to the deterministic precedence.

---

# 52. Required algebra/property tests

Add at least these laws.

## Law A — Assumption monotonicity

Replacing one Established required input by the same Assumed type must never strengthen output knowledge.

```text
compose(E, E) >= compose(E, A)
```

in epistemic strength.

---

## Law B — Unknown monotonicity

Replacing a required known input with Unknown may reduce precision, never increase it.

The output cannot become Known if the replaced premise remains required.

---

## Law C — Required operand completeness

For all required inputs:

```text
output Known
    =>
every required input was Known
```

unless the operation documents an independent derivation.

`compose_required_knowledge()` itself has no independent-derivation mode.

That is deliberate.

---

## Law D — Cause coherence

For every published expression:

```text
Invalid(C)
    =>
causalInvalidity.contains(C)
```

Scan every expression in a `Fixture` analysis for this invariant in a generic helper.

This should become a broad test, not just one regression.

---

## Law E — Suppression coherence

For every published expression:

```text
Suppressed(_)
    =>
causalInvalidity != Clean
```

---

# 53. Extend the test Fixture with invariant assertions

Modify:

```text
phalcom-semantic/tests/semantic/support/fixture.rs
```

Add:

```rust
pub fn assert_expression_product_invariants(
    &self,
)
```

It should iterate all callable analyses and expressions.

For every expression:

```rust
match expression.status {
    AnalysisStatus::Invalid(cause) => {
        assert!(
            expression
                .causal_invalidity
                .contains(cause)
        );
    }

    AnalysisStatus::Suppressed(_) => {
        assert_ne!(
            expression.causal_invalidity,
            CausalInvalidity::Clean
        );
    }

    _ => {}
}
```

Do not assert Ready is clean.

Call this helper from the new semantic-correctness integration fixtures.

Consider eventually making it part of `Fixture::new()`, but do not do so in the first RED step: an already-existing unrelated violation could make every semantic test fail and obscure the targeted migration.

After this spec's test suite is green, audit whether enabling it globally is appropriate.

---

# 54. Exact file map

## Create

```text
phalcom-semantic/src/checker/composition.rs
```

Responsibility:

```text
required-expression dependency propagation
terminal-state propagation
static component projection helpers
```

Do not put generic inference or dispatch logic here.

---

## Modify

```text
phalcom-semantic/src/checker/mod.rs
```

Register `composition`.

---

```text
phalcom-semantic/src/types/evidence.rs
```

Add:

```text
compose_required_knowledge
```

Reuse existing stable Unknown/Dynamic reason reducers.

No new TypeKnowledge variants.

No new EvidenceStatus.

No EvidenceOrigin additions required.

---

```text
phalcom-semantic/src/checker/causal.rs
```

Add:

```text
CausalInvalidity::contains
```

---

```text
phalcom-semantic/src/checker/typed_expr.rs
```

Add:

```text
invalidate
debug_assert_coherent
```

Potentially add a small unique-explanation/dependency utility if it belongs naturally here.

---

```text
phalcom-semantic/src/checker/context.rs
```

Add:

```text
publish_expression_analysis
sync_expression_outcome
bind_pattern_binding_with_causal
```

Make existing `bind_pattern_binding` delegate with `Clean`.

Retire or narrow old piecemeal `record_expression()` once no production call needs it.

---

```text
phalcom-semantic/src/checker/expression.rs
```

Modify:

```text
analyze_expression
list literal
set literal
map literal
tuple literal
record literal
expansion handling
```

Do not alter canonical call logic in this task.

---

```text
phalcom-semantic/src/checker/statement.rs
```

Modify:

```text
Statement::Let mismatch publication
Statement::For
bind_pattern
bind_declaration_pattern
tuple decomposition
```

---

```text
phalcom-semantic/tests/semantic/support/fixture.rs
```

Add semantic-product invariant helpers.

---

## Add or extend tests

```text
phalcom-semantic/tests/semantic/foundations/
```

or existing semantic-correctness test directory.

Do not reorganize the full test tree during this implementation.

---

# 55. Implementation sequence

The executor should implement this as seven independently reviewable slices.

## Task 1 — Add product-integrity primitives

Files:

```text
checker/causal.rs
checker/typed_expr.rs
tests
```

RED first:

```text
Invalid(C) without C in causal state
```

Then implement:

```text
contains
invalidate
coherence assertion
```

Commit independently.

---

## Task 2 — Make expression publication atomic

Files:

```text
checker/context.rs
checker/expression.rs
checker/statement.rs
```

Introduce:

```text
publish_expression_analysis
sync_expression_outcome
```

Migrate `analyze_expression()`.

Repair `Statement::Let`.

Run targeted binding/causality tests.

Commit.

---

## Task 3 — Add required-knowledge composition

Files:

```text
types/evidence.rs
```

Write all low-level algebra tests first.

Implement one primitive.

No expression migration yet.

Commit.

---

## Task 4 — Add required-expression state propagation

Files:

```text
checker/composition.rs
checker/mod.rs
```

RED unit tests for every status combination.

Implement deterministic terminal propagation.

Commit.

---

## Task 5 — Migrate direct aggregate literals

Migrate in order:

```text
list
set
map
tuple
record
```

For each kind:

1. write source regression;
2. verify it fails for the expected old behavior;
3. migrate only that helper;
4. run targeted test;
5. proceed to next helper.

Do not rewrite all five and then discover which policy failed.

Commit once the aggregate family is coherent.

---

## Task 6 — Expansion fail-closed/projectable semantics

Implement static projections separately from direct literal elements.

Test:

```text
exact known projectable
unknown
dynamic
known non-projectable
assumed projectable
```

Do not add generic iterable protocol inference here.

Commit.

---

## Task 7 — Pattern and iteration preservation

Repair:

```text
tuple pattern reason preservation
pattern causal propagation
for Unknown preservation
for Dynamic preservation
for causal propagation
```

Run flow and semantic foundation suites.

Commit.

---

# 56. Commands and test gates

An implementation agent must determine the repository's canonical Cargo package/test invocation from the current workspace, but the expected targeted form is approximately:

```bash
cargo test -p phalcom-semantic \
  semantic::foundations::semantic_correctness_regressions
```

and focused new tests by exact test name.

Before production modification for each regression:

```text
test must fail
for the specific semantic assertion being repaired
```

A parser error, fixture setup failure, missing type declaration, or unrelated panic does not count as the required RED state.

After each task:

```bash
cargo fmt --check
cargo test -p phalcom-semantic <targeted suite>
```

At closure:

```bash
cargo test -p phalcom-semantic
cargo test --workspace
```

if workspace cost is acceptable under the repository's established verification process.

No test may be changed from a stronger semantic assertion to a weaker one merely to obtain green.

---

# 57. Explicit forbidden implementations

The implementation agent must treat the following as code-review blockers.

### Forbidden 1

```rust
if let Some(ty) = knowledge.ty() {
    contributions.push(ty);
}
```

for a required operand.

---

### Forbidden 2

```rust
Unknown(_) =>
    Unknown(NoTypeEvidence)
```

during decomposition.

---

### Forbidden 3

```rust
Dynamic(_) =>
    Dynamic(RuntimeReflection)
```

without an actual runtime-reflection semantic operation.

---

### Forbidden 4

```rust
typed.status = Invalid(cause);
```

without causal update.

Use the canonical invalidation operation.

---

### Forbidden 5

```rust
analysis.status = ...;
analysis.causal_invalidity = ...;
```

at arbitrary call sites after this patch.

Post-analysis updates must go through the synchronization helper.

---

### Forbidden 6

```rust
Known + Unknown -> Known
```

because the unknown operand “did not supply useful type information.”

That is exactly the defect this spec removes.

---

### Forbidden 7

```rust
Assumed + Established -> Established
```

merely because all resulting `TypeId`s agree.

---

### Forbidden 8

```rust
Unknown expansion -> ignore expansion
```

A required expansion operand either contributes soundly or prevents a precise aggregate result.

---

### Forbidden 9

Using:

```text
Object
Unit
expected type
first generic argument
first known element
```

as a recovery substitute for unavailable required knowledge.

---

### Forbidden 10

Treating non-clean causal invalidity as automatic suppression.

A usable known premise may remain fully analyzable despite upstream invalidity.

---

# 58. Acceptance matrix

The implementation is complete only when all rows are satisfied.

| Area | Required result |
|---|---|
| `TypeKnowledge` representation | unchanged |
| stable Unknown join | preserved |
| stable Dynamic join | preserved |
| required-composition primitive | implemented once |
| Established + Assumed required operands | Assumed |
| Known + Unknown required operands | Unknown |
| Known + Dynamic required operands | Dynamic |
| list unknown element | cannot establish partial list |
| set unknown element | cannot establish partial set |
| map unknown key/value | cannot establish partial map |
| tuple unknown reason | preserved |
| record unknown reason | preserved |
| expansions | participate or fail closed |
| pattern Unknown reason | preserved |
| pattern Dynamic reason | preserved |
| pattern causal invalidity | propagated |
| `for` Unknown reason | preserved |
| `for` Dynamic reason | preserved |
| `for` causal invalidity | propagated |
| `Invalid(C)` product | causal state includes C |
| causally-invalid Known child | does not automatically suppress parent |
| invalid unavailable child | suppresses required dependent operation |
| Cancelled child | parent cancellation preserved |
| BudgetExceeded child | parent budget status preserved |
| InternalFailure child | parent internal failure preserved |
| normal-return summary | current semantics preserved |
| flow joins | current contract-aware semantics preserved |
| loop widening | current reconciliation semantics preserved |

---

# 59. Closure laws

Before this specification is marked implemented, the executor must be able to state that the current code enforces these laws by construction.

### Law 1 — Required premise completeness

```text
A derived composite proposition cannot be Known
if a required component proposition is Unknown or Dynamic.
```

Unless the operation has an explicitly modeled independent derivation.

The generic primitive in this spec does not.

---

### Law 2 — Evidence monotonicity

```text
weaker required premise
    cannot
produce stronger composite evidence
```

---

### Law 3 — Unknown preservation

```text
Unknown(R)
```

remains `R` through a pure operation unless a genuine semantic transformation creates a more accurate downstream reason.

---

### Law 4 — Dynamic preservation

Dynamic reason is not a generic fallback bucket and is not silently rewritten.

---

### Law 5 — Cause coherence

```text
Invalid(C)
    =>
causalInvalidity includes C
```

for every published expression.

---

### Law 6 — Causal invalidity is not suppression

```text
Ready
+
non-clean causalInvalidity
```

is a valid and important expression state.

---

### Law 7 — Suppression requires a lost premise

A parent is suppressed only when the upstream invalidity is why a semantic premise required by the parent is unavailable.

---

### Law 8 — Product publication is atomic

A published `ExpressionAnalysis` must represent one coherent `TypedExpression` result, not a combination assembled by separately updating status, knowledge, and causal fields.

---

# 60. Relationship to Technical Spec 02

The most important downstream consequence of this implementation is that Technical Spec 02 can use exactly the same result discipline for:

```text
method calls
operators
subscript get
subscript set
property set
callable values
constructors
```

Technical Spec 02 must not invent another way to propagate:

```text
knowledge
status
causal invalidity
explanation dependencies
```

Its canonical application result should use or compose with the primitives introduced here.

Likewise, Technical Spec 03 for generics must treat generic arguments as required call premises instead of reverting to:

```rust
if let Some(arg_ty) = arg.knowledge.ty() {
    add_constraint(arg_ty);
}
```

and forgetting the operand when no `TypeId` exists.

That is why this implementation slice comes first.

---

## Completion condition

I would consider Technical Spec 01 complete only when an implementation agent can no longer produce a false `Established` aggregate merely by filtering out inconvenient operands, cannot erase an Unknown/Dynamic reason during ordinary projection, and cannot publish `Invalid(C)` with a clean causal product.

The repository already contains most of the semantic types needed to achieve that. The implementation change is therefore intentionally surgical: centralize the missing algebra, migrate the known violating producers, add atomic publication helpers, and lock the laws down with adversarial integration tests rather than redesigning the semantic model.

This is the first technical implementation specification. The next should be considerably more involved: **Technical Spec 02 — Canonical Callable Application and Operation Semantics**, where binary/unary operators, indexing, setters, ordinary calls, constructor calls, and callable-valued locals are forced through one relation/application engine.
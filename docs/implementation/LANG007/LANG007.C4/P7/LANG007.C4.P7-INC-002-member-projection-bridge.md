# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: LANG007
- Checkpoint: LANG007.C4
- Plan: LANG007.C4.P7
- Task/Gate: T6 / G6
- Repository: /Users/altunhasanli/dev/phalcom/phalcom
- Branch: main
- Starting revision: 3f8cd713
- Current revision: 3f8cd713 (uncommitted working tree)

## 2. Escalation trigger

Triggers 1, 8, and 16 from the shared consultation protocol apply: the planned
member-projection route is absent from the live checked-body/evaluator
interfaces, implementing it requires a cross-layer contract, and the plan's
expected route conflicts with current source structure.

## 3. Required invariant

P5's canonical member identity and projection result domain remain authoritative.
The evaluator must preserve the distinction between kinded terms, ordinary
static values, residual projections, and terminal Unknown/Blocked/Dynamic
outcomes. It must not reconstruct member identity from source spelling or add a
second projection authority.

## 4. Expected architecture

P7.T6 requires the evaluator to turn Stuck(MemberProjection) directly into a
typed residual and to reuse the canonical structural/member projector after
specialization. Tests must cover an open projection and an exact projection
reducing to an ordinary StaticValue.

## 5. Observed behavior

The live StaticEvaluator has no call path to
StructuralReflectionProjector::project_member. Its checked static body stores
proof-predicate resolutions, but no member-projection resolution. TypeForm
completion requires result_as_kind() and only publishes a kinded TypeId;
the structural projector's Product::Shape result instead has a
ValueType(ProductShape) domain and can produce a StaticValueRef.

The current continuation maps canonical AssociatedProjection and
TypeMemberProjection values returned by a kinded static definition into
typed member-projection residuals. It does not connect Stuck(MemberProjection)
from the projector or reduce exact structural projections to ordinary static
values.

## 6. Minimal reproduction

Focused command:

    RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic static_recursive_evaluation

Observed: 7 selected tests pass, including the new typed residual case.

Expected T6 acceptance: a generic structural member projection must residualize,
then specialize through the same canonical projector to an ordinary static
value. The latter path has no source/evaluator product to exercise in the live
tree.

## 7. Relevant code path

    CheckedStaticBody::proof_predicates
      -> StaticEvaluator::EvaluateTypeForm
      -> complete_checked_type_form
      -> ReducedKinded(TypeId) or Residual

    StructuralReflectionProjector::project_product_shape / project_member
      -> ReducedValue(StaticValueRef) or Stuck(MemberProjection)

There is no edge between these paths today. Relevant definitions:

- phalcom-semantic/src/static_definition.rs:138-169
- phalcom-semantic/src/static_eval/machine.rs:2365-2406
- phalcom-semantic/src/member_projection.rs:91-114
- phalcom-semantic/src/structural_reflection.rs:139-203,529-535

## 8. Scoped implementation diff

The current T6 slice adds typed proof residuals for open proof goals, uses
canonical proof engines for exact subtype and conformance applications, and
maps canonical kinded associated/type-member projections into member residuals.
It adds focused tests for open/exact proof behavior and an associated projection.
The working tree also contains earlier P7 changes and unrelated existing dirty
files; no files were staged or committed.

## 9. Evidence

- rg -n 'project_member\\(' phalcom-semantic/src --glob '*.rs' finds only the
  method definition in structural_reflection.rs, with no production call site.
- CheckedStaticBody has no member-projection resolution field.
- complete_checked_type_form rejects bodies without a kinded result domain.
- project_product_shape returns ReducedValue for concrete products and
  Stuck(MemberProjection) for abstract/symbolic products; both projections
  carry ValueType(ProductShape).
- RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic
  static_recursive_evaluation passed 7/7.
- RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic
  static_definition_bodies passed 8/8.

## 10. Implementer's causal hypothesis

T6 assumes a checked canonical member-projection product exists or can be
derived at the evaluator boundary. Live code exposes the projector as a
standalone API but does not publish projection sites in static bodies. A correct
bridge likely needs a checker-owned projection resolution carrying canonical
member identity and result domain, plus an evaluator result path that can
accept a reduced value, reduced kinded term, or residual.

## 11. Alternative hypotheses

- The intended contract may be to derive structural projections solely from
  TypeData and add a value-result completion path to the evaluator.
- The intended consumer may be outside CheckedStaticBody, with a caller
  responsible for invoking the projector and passing its typed outcome into
  residual normalization.

These alternatives differ in where canonical projection-site identity and
result-domain authority live.

## 12. Attempts already made

- Mapped returned canonical AssociatedProjection/TypeMemberProjection forms
  into typed residuals. The focused residual test passes.
- Did not invoke StructuralReflectionProjector from TypeForm completion:
  its concrete Product::Shape result is an ordinary static value with a
  ValueType domain, while the current path requires Kind and returns
  ReducedKinded. Ad hoc conversion would violate the result-domain invariant.

## 13. Why local authority is insufficient

Choosing between a new checked-body product, an evaluator-facing projection
request, or TypeData-only reconstruction changes the semantic ownership and
cross-layer contract that later T7 specialization will depend on. The active
plan does not choose among these interfaces or state how the ValueType result
crosses the current evaluator completion boundary.

## 14. Decision requested

For T6, where should canonical structural/member projection sites and their
StaticValueDomain live, and what exact evaluator interface should route
Stuck and reduced outcomes through the existing projector without rebuilding
member identity or losing the distinction between ReducedValue,
ReducedKinded, and terminal Unknown/Blocked/Dynamic outcomes?

Should the present TypeData-to-residual path remain as a separate kinded case,
or be replaced by the selected canonical projection route?

## Adopted adviser decision — 2026-10-12

**Verdict: T6 requires a Level 2 clarification/amendment.** Add a compact
canonical member-projection product/index to `CheckedStaticBody`, keyed by
checked expression/site and derived from ordinary associated/type-member
resolution plus the canonical signature. It carries existing `MemberProjection`
identity, arguments/evidence, and `StaticValueDomain`. The evaluator consumes
this product and does not resolve names or infer domains.

Route the product through `StructuralReflectionProjector::project_member` and
preserve its typed outcome. Validate reduced results against the checked site
domain; route `ReducedValue` to ordinary value completion, `ReducedKinded` to
kinded completion, and `Stuck` to a typed residual using canonical identity and
domain. Keep `Unknown`, `Blocked`, and `Dynamic` as distinct terminal evaluator
outcomes. `Cancelled` and `BudgetExceeded` use existing terminal paths.

Keep TypeData-to-residual conversion as a separate kinded case for canonical
`AssociatedProjection` and `TypeMemberProjection` forms. It does not replace
checked static-value projection sites such as `Product::Shape`. Resume
implementation once the amendment is recorded; no further architectural fork
is required. Focused G6 coverage must include open `Product::Shape<T>`, exact
specialization to `ReducedValue`, identity/domain preservation, and distinct
Unknown/Blocked/Dynamic outcomes.

## Implementation resolution — 2026-10-12

The amended bridge is implemented in `CheckedStaticBody`, its checker builder,
and `StaticEvaluator`. The checker-side unit test proves a canonical Product
associated projection publishes its exact requirement identity, `ValueType`
domain, and trait-surface dependency. The focused evaluator regression injects
the resulting checked-site contract and proves an open subject residualizes
with identity/domain intact while an exact subject reduces to an ordinary
reflected static value. Separate terminal mapping coverage proves Unknown,
Blocked, and Dynamic remain distinct. Focused tests passed: checker index 1/1,
`static_recursive_evaluation` 8/8, `static_definition_bodies` 8/8, and terminal
outcome 1/1. T6/G6 is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`; broader P7
acceptance remains open.

# Phalcom Formal Knowledge and Expression Composition Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
********
**Goal:** Implement Technical Spec 01 so required semantic operands can no longer disappear from aggregate inference, expression products are published coherently, and Unknown/Dynamic/causal evidence survives collection construction, pattern decomposition, and iteration transfer.

**Architecture:** Keep the current `TypeKnowledge` / `AnalysisStatus` / `CausalInvalidity` model. Add one canonical required-knowledge composition primitive in `types/evidence.rs`, one checker-local required-expression propagation module in `checker/composition.rs`, and a small set of atomic publication/invalidation helpers. Then migrate existing aggregate, pattern, and `for` producers one family at a time under RED regression tests.

**Tech Stack:** Rust 2024, Cargo workspace, `phalcom-semantic` canonical integration-test binary (`tests/semantic.rs`), current semantic `Fixture` test harness.

**Spec:** `docs/impl/semantic/semantic-correctness/technical/01-formal-knowledge-and-expression-composition-implementation-spec.md`

**Verified baseline:** `aureat/phalcom-lang` `main` at commit `6ced2afd83ee89d2a09f45b8ba3821482abf3752`.

## Global Constraints

- Do not redesign `TypeKnowledge`, `EvidenceStatus`, `AnalysisStatus`, or `CausalInvalidity`.
- Do not fold canonical callable application, operator checking, subscript checking, setters, or generic proof integrity into this plan.
- Every behavior change starts with a regression that fails for the intended semantic reason.
- Never weaken a semantic assertion merely to obtain green tests.
- `UnknownReason` and `DynamicReason` are semantic data; preserve them unless the operation itself creates a more accurate reason.
- A required operand with no concrete `TypeId` must not disappear from a composite operation.
- `Assumed` required evidence must never become `Established` merely because all known `TypeId`s agree.
- `AnalysisStatus::Invalid(cause)` must always be represented with causal invalidity containing that cause.
- Non-clean causal invalidity does not imply suppression.
- Keep existing correct flow-join, loop-widening, bounded-relation, and normal-return-summary behavior intact.
- Do not add broad refactors unrelated to this semantic slice.
- Use the existing `phalcom-semantic` integration test binary. Focus tests with the concrete Rust test path after `--test semantic`, for example: `cargo test -p phalcom-semantic --test semantic semantic::foundations::expression_composition`.
- Run `cargo fmt --check` before every review/commit gate.
- Commits below are suggested review boundaries; if the repository owner prefers squash-only history, preserve the task boundaries in the working tree/review anyway.

---

# 1. Baseline Map: Files and Current Anchors

The executor must begin from the current `main` implementation and locate these exact anchors before editing.

| File | Existing anchor | Current role / defect |
|---|---|---|
| `phalcom-semantic/src/types/evidence.rs` | `TypeKnowledge`, `join_type_knowledge`, `join_unknown_reason`, `join_dynamic_reason` | Correct flow-alternative join exists; no required-premise composition primitive |
| `phalcom-semantic/src/checker/causal.rs` | `CausalInvalidity::join`, `suppression_cause` | Needs a bounded `contains(cause)` coherence helper |
| `phalcom-semantic/src/checker/typed_expr.rs` | `TypedExpression` | Carries all three dimensions; lacks a canonical local invalidation/coherence operation |
| `phalcom-semantic/src/checker/context.rs` | `emit_diagnostic`, `apply_relation_outcome`, `record_expression`, `bind_pattern_binding` | Expression publication is piecemeal; pattern binding defaults causal state to `Clean` |
| `phalcom-semantic/src/checker/expression.rs` | `analyze_expression` | Reconstructs/publishes product fields in multiple steps |
| `phalcom-semantic/src/checker/expression.rs` | `synthesize_list_literal` | Drops Unknown/Dynamic elements by filtering `.knowledge.ty()` |
| `phalcom-semantic/src/checker/expression.rs` | `synthesize_set_literal` | Same |
| `phalcom-semantic/src/checker/expression.rs` | `synthesize_map_literal` | Drops Unknown/Dynamic keys/values |
| `phalcom-semantic/src/checker/expression.rs` | `synthesize_tuple_literal` | Fails closed, but rewrites all non-known direct elements to `UncheckedExpression`; ignores expansions |
| `phalcom-semantic/src/checker/expression.rs` | `synthesize_record_literal` | Same class of defect |
| `phalcom-semantic/src/checker/statement.rs` | `Statement::Let` | Can set initializer `Invalid(cause)` without synchronizing that cause into initializer causal state |
| `phalcom-semantic/src/checker/statement.rs` | `Statement::For` | Uses `synthesize_expr`; rewrites Unknown/Dynamic reason and discards causal state |
| `phalcom-semantic/src/checker/statement.rs` | `bind_declaration_pattern`, `bind_pattern` | Tuple decomposition rewrites Unknown/Dynamic into `NoTypeEvidence` |
| `phalcom-semantic/tests/semantic/foundations/mod.rs` | module list | Add `mod expression_composition;` |
| `phalcom-semantic/tests/semantic/support/fixture.rs` | `Fixture` | Add reusable expression-product invariant assertion |

Current test binary wiring is already:

```rust
// phalcom-semantic/tests/semantic.rs
#[path = "semantic/mod.rs"]
mod semantic;
```

and:

```rust
// phalcom-semantic/tests/semantic/mod.rs
pub(crate) mod foundations;
```

Therefore this plan creates:

```text
phalcom-semantic/tests/semantic/foundations/expression_composition.rs
```

and adds only:

```rust
mod expression_composition;
```

to `phalcom-semantic/tests/semantic/foundations/mod.rs`.

Do **not** create a new `correctness/` test directory in this slice.

---

# 2. Execution Strategy

Implement in this order:

```text
Task 1  Product coherence primitive
Task 2  Atomic expression publication + initializer fix
Task 3  Required-knowledge algebra
Task 4  Required-expression dependency propagation
Task 5  List + Set migration
Task 6  Map migration
Task 7  Tuple + Record direct-member migration
Task 8  Expansion projection/fail-closed behavior
Task 9  Pattern decomposition + causal propagation
Task 10 for-loop transfer preservation
Task 11 Fixture-wide invariant checks + regression consolidation
Task 12 Cleanup, audit, and full verification
```

Why this order matters:

- Tasks 1–4 create reusable semantic primitives.
- Tasks 5–10 migrate producers; they must not invent local replacements.
- Task 11 turns local regressions into broad product invariants.
- Task 12 searches the repository for residual forbidden patterns and proves no old path was left behind.

---

# 3. Task 1 — Add Expression Product Coherence Primitives

**Deliverable:** There is exactly one normal way to mark a `TypedExpression` invalid, and tests can verify that `Invalid(C)` contains `C` in the bounded causal state.

**Files:**
- Modify: `phalcom-semantic/src/checker/causal.rs`
- Modify: `phalcom-semantic/src/checker/typed_expr.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/expression_composition.rs`
- Modify test module: `phalcom-semantic/tests/semantic/foundations/mod.rs`

**Interfaces:**
- Produces: `CausalInvalidity::contains(DiagnosticCauseId) -> bool`
- Produces: `TypedExpression::invalidate(DiagnosticCauseId)`
- Produces: `TypedExpression::debug_assert_coherent()`

## Step 1.1 — Create the test module

- [ ] Create `phalcom-semantic/tests/semantic/foundations/expression_composition.rs` with the imports needed for low-level product tests:

```rust
use phalcom_common::range::SourceRange;
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::checker::causal::CausalInvalidity;
use phalcom_semantic::checker::typed_expr::TypedExpression;
use phalcom_semantic::identity::DiagnosticCauseId;
use phalcom_semantic::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
use phalcom_semantic::types::store::TypeStore;

use crate::semantic::support::Fixture;
```

If `DiagnosticCauseId` is not publicly constructible from this integration test, keep the product-coherence unit test in `checker/causal.rs` / `checker/typed_expr.rs` and use source-level coherence tests in the integration file. Do not widen identity visibility merely to simplify a test.

- [ ] Add this line to `phalcom-semantic/tests/semantic/foundations/mod.rs`:

```rust
mod expression_composition;
```

## Step 1.2 — Write RED/unit tests for causal containment

- [ ] In `checker/causal.rs`, extend the existing `#[cfg(test)]` module with:

```rust
#[test]
fn contains_reports_bounded_cause_membership() {
    let c1 = DiagnosticCauseId(4);
    let c2 = DiagnosticCauseId(5);

    assert!(!CausalInvalidity::Clean.contains(c1));
    assert!(CausalInvalidity::One(c1).contains(c1));
    assert!(!CausalInvalidity::One(c1).contains(c2));

    // Multiple intentionally means "at least two causes"; exact membership
    // is not retained in the hot representation.
    assert!(CausalInvalidity::Multiple.contains(c1));
    assert!(CausalInvalidity::Multiple.contains(c2));
}
```

- [ ] Run:

```bash
cargo test -p phalcom-semantic --lib contains_reports_bounded_cause_membership
```

Expected RED: method `contains` does not exist.

## Step 1.3 — Implement `CausalInvalidity::contains`

- [ ] In `phalcom-semantic/src/checker/causal.rs`, add immediately after `join`:

```rust
/// Returns whether this bounded causal summary can contain `cause`.
///
/// `Multiple` deliberately does not retain exact cause identity, so any
/// concrete cause is conservatively considered contained.
pub fn contains(self, cause: DiagnosticCauseId) -> bool {
    match self {
        Self::Clean => false,
        Self::One(actual) => actual == cause,
        Self::Multiple => true,
    }
}
```

- [ ] Run the focused unit test again; expect PASS.

## Step 1.4 — Add `TypedExpression::invalidate`

- [ ] In `phalcom-semantic/src/checker/typed_expr.rs`, import `DiagnosticCauseId`:

```rust
use crate::identity::{CallableId, DiagnosticCauseId, ExplanationId, ExpressionId};
```

- [ ] Add inside `impl TypedExpression`:

```rust
/// Marks this expression as owning `cause` while keeping independently
/// available type knowledge intact.
pub(crate) fn invalidate(&mut self, cause: DiagnosticCauseId) {
    self.status = AnalysisStatus::Invalid(cause);
    self.causal_invalidity = self
        .causal_invalidity
        .join(CausalInvalidity::One(cause));
}
```

Do not change `knowledge`.

## Step 1.5 — Add `debug_assert_coherent`

- [ ] Add:

```rust
pub(crate) fn debug_assert_coherent(&self) {
    if let AnalysisStatus::Invalid(cause) = self.status {
        debug_assert!(
            self.causal_invalidity.contains(cause),
            "Invalid expression status must include its owning diagnostic cause"
        );
    }

    if matches!(self.status, AnalysisStatus::Suppressed(_)) {
        debug_assert!(
            !matches!(self.causal_invalidity, CausalInvalidity::Clean),
            "Suppressed expression must have non-clean causal invalidity"
        );
    }
}
```

Do **not** add any assertion that `Ready` implies `Clean`.

## Step 1.6 — Run format and focused tests

- [ ] Run:

```bash
cargo fmt --all
cargo test -p phalcom-semantic --lib causal
```

- [ ] Commit:

```bash
git add \
  phalcom-semantic/src/checker/causal.rs \
  phalcom-semantic/src/checker/typed_expr.rs \
  phalcom-semantic/tests/semantic/foundations/mod.rs \
  phalcom-semantic/tests/semantic/foundations/expression_composition.rs

git commit -m "fix(semantic): add expression product coherence primitives"
```

---

# 4. Task 2 — Make Expression Publication Atomic and Fix Initializer Cause Drift

**Deliverable:** `analyze_expression()` publishes one coherent `ExpressionAnalysis` from one `TypedExpression`, and `Statement::Let` cannot create `Invalid(C)` with stale/clean initializer causal state.

**Files:**
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Modify: `phalcom-semantic/src/checker/statement.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/expression_composition.rs`

**Interfaces:**
- Produces: `CheckingContext::publish_expression_analysis(&mut self, ExpressionId, SourceRange, &TypedExpression, Option<ExplanationId>) -> ExpressionAnalysis`
- Produces: `CheckingContext::sync_expression_outcome(&mut self, &TypedExpression)`
- Consumes: `TypedExpression::invalidate`

## Step 2.1 — Add a source-level RED regression for initializer coherence

- [ ] Add:

```rust
#[test]
fn invalid_initializer_expression_contains_its_own_cause() {
    let fixture = Fixture::new(
        r#"
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
"#,
    );

    let run = fixture.callable("Probe", "run", phalcom_semantic::identity::DispatchSide::Class);
    let initializer = fixture.expression(run, "CellNum.new()");

    let AnalysisStatus::Invalid(cause) = initializer.status else {
        panic!("initializer mismatch must own Invalid status: {initializer:#?}");
    };

    assert!(
        initializer.causal_invalidity.contains(cause),
        "Invalid(cause) must be represented in the initializer causal product: {initializer:#?}",
    );
}
```

- [ ] Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::expression_composition::invalid_initializer_expression_contains_its_own_cause
```

Expected RED: initializer is `Invalid(cause)` but its stored `causal_invalidity` does not contain the newly allocated mismatch cause.

## Step 2.2 — Add atomic publication API to `CheckingContext`

- [ ] In `context.rs`, locate existing:

```rust
pub fn record_expression(
    &mut self,
    id: ExpressionId,
    range: SourceRange,
    knowledge: TypeKnowledge,
    callable: Option<CallableId>,
    denotation: Option<SemanticDenotation>,
    status: AnalysisStatus,
) -> ExpressionAnalysis
```

- [ ] Add a new checker-internal method **next to it**:

```rust
pub(crate) fn publish_expression_analysis(
    &mut self,
    id: ExpressionId,
    range: SourceRange,
    typed: &crate::checker::typed_expr::TypedExpression,
    explanation: Option<crate::identity::ExplanationId>,
) -> ExpressionAnalysis {
    typed.debug_assert_coherent();

    let mut analysis = ExpressionAnalysis::ready(id, range, typed.knowledge.clone());
    analysis.callable = typed.callable.clone();
    analysis.denotation = typed.denotation;
    analysis.status = typed.status.clone();
    analysis.causal_invalidity = typed.causal_invalidity;
    analysis.explanation = explanation;

    self.expressions.insert(id, analysis.clone());
    analysis
}
```

- [ ] Do **not** delete `record_expression()` yet. First migrate its production call sites, then remove/narrow it in Task 12 if unused.

## Step 2.3 — Add post-analysis synchronization API

- [ ] Add:

```rust
pub(crate) fn sync_expression_outcome(
    &mut self,
    typed: &crate::checker::typed_expr::TypedExpression,
) {
    typed.debug_assert_coherent();

    let Some(id) = typed.expression_id else {
        return;
    };
    let Some(analysis) = self.expressions.get_mut(&id) else {
        return;
    };

    analysis.knowledge = typed.knowledge.clone();
    analysis.callable = typed.callable.clone();
    analysis.denotation = typed.denotation;
    analysis.status = typed.status.clone();
    analysis.causal_invalidity = typed.causal_invalidity;
}
```

Do not allocate a new expression ID and do not replace the existing explanation ID.

## Step 2.4 — Rewrite `analyze_expression()` publication

- [ ] In `expression.rs`, locate the block that currently:

1. calls `analyze_expression_inner`;
2. pops the expression owner;
3. builds a separate `status` variable;
4. calls `record_expression`;
5. mutates `analysis.causal_invalidity`;
6. inserts it;
7. optionally adds explanation and inserts it again.

- [ ] Replace that publication sequence with this shape:

```rust
pub fn analyze_expression(
    ctx: &mut CheckingContext<'_>,
    expr: &Expr,
    expected: &ExpectedType,
) -> TypedExpression {
    let expr_id = ctx.alloc_expression_id();
    ctx.push_expression_owner(expr_id);

    let mut typed = analyze_expression_inner(ctx, expr, expected);

    if let Some(cause_id) = ctx.pop_expression_owner(expr_id) {
        typed.invalidate(cause_id);
    }

    typed.expression_id = Some(expr_id);

    let explanation_id = if let Some(ty) = typed.knowledge.ty() {
        // KEEP the existing ExplanationStep selection and EvidenceRef
        // construction here. Only remove the duplicate publication logic.
        let step = match expr {
            Expr::Int { .. }
            | Expr::Float { .. }
            | Expr::String { .. }
            | Expr::Boolean { .. } => {
                crate::explain::ExplanationStep::Literal {
                    expression: expr_id,
                    ty,
                }
            }
            Expr::MethodCall(_) => match typed.callable.clone() {
                Some(callable) => crate::explain::ExplanationStep::MethodCall {
                    call: expr_id,
                    callable,
                    return_ty: ty,
                },
                None => crate::explain::ExplanationStep::UnresolvedCall {
                    call: expr_id,
                    return_ty: ty,
                },
            },
            _ => crate::explain::ExplanationStep::Literal {
                expression: expr_id,
                ty,
            },
        };

        let rule = step.derivation_rule();
        let evidence = vec![
            crate::explain::EvidenceRef::SourceSpan(expr.range()),
            crate::explain::EvidenceRef::TypeId(ty),
        ];
        let status = typed
            .knowledge
            .status()
            .unwrap_or(EvidenceStatus::Established);
        let origin = typed
            .knowledge
            .origin()
            .unwrap_or(EvidenceOrigin::Syntax);

        Some(ctx.record_derivation(
            step,
            rule,
            status,
            origin,
            evidence,
            typed.explanation_parents.clone(),
        ))
    } else {
        None
    };

    ctx.record_call_dependency(typed.causal_invalidity, explanation_id);

    ctx.publish_expression_analysis(
        expr_id,
        expr.range(),
        &typed,
        explanation_id,
    );

    typed
}
```

The executor must preserve any current call-resolution ID population not shown in this simplified block. If a later/current field is populated between baseline inspection and execution, carry it through the atomic helper rather than dropping it.

## Step 2.5 — Fix `Statement::Let`

- [ ] Find this current mismatch pattern in `statement.rs`:

```rust
causal_invalidity = causal_invalidity.join(CausalInvalidity::One(cause));
val_typed.status = AnalysisStatus::Invalid(cause);
```

- [ ] Replace the expression-local mutation with:

```rust
val_typed.invalidate(cause);
ctx.sync_expression_outcome(&val_typed);
```

- [ ] Then derive binding causal state from the synchronized initializer:

```rust
let causal_invalidity = val_typed
    .causal_invalidity
    .join(annotation_invalidity);
```

The binding can depend on annotation invalidity. The initializer expression must not be mutated with unrelated annotation invalidity after the fact.

- [ ] Remove the later manual block:

```rust
if let Some(expression_id) = val_typed.expression_id {
    if let Some(analysis) = ctx.expressions.get_mut(&expression_id) {
        analysis.status = val_typed.status.clone();
        analysis.causal_invalidity = val_typed.causal_invalidity;
    }
}
```

and replace it with a single:

```rust
ctx.sync_expression_outcome(&val_typed);
```

after all expression-local status changes are finalized.

## Step 2.6 — Verify

- [ ] Run the RED regression again; expect PASS.

- [ ] Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::semantic_correctness_regressions

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::causal

cargo fmt --check
```

- [ ] Commit:

```bash
git add \
  phalcom-semantic/src/checker/context.rs \
  phalcom-semantic/src/checker/expression.rs \
  phalcom-semantic/src/checker/statement.rs \
  phalcom-semantic/tests/semantic/foundations/expression_composition.rs

git commit -m "fix(semantic): publish coherent expression outcomes"
```

---

# 5. Task 3 — Implement Canonical Required-Knowledge Composition

**Deliverable:** One reusable function implements the epistemic algebra for operations whose result requires all component type propositions.

**Files:**
- Modify: `phalcom-semantic/src/types/evidence.rs`
- Unit test: same file

**Interfaces:**
- Produces:

```rust
pub(crate) fn compose_required_knowledge(
    inputs: impl IntoIterator<Item = TypeKnowledge>,
    origin: EvidenceOrigin,
    build_type: impl FnOnce(&[TypeId]) -> Result<TypeId, UnknownReason>,
) -> TypeKnowledge
```

## Step 3.1 — Add RED unit tests before production code

- [ ] Add tests covering these exact cases:

```rust
#[cfg(test)]
mod required_composition_tests {
    use super::*;
    use crate::types::store::TypeStore;
    use phalcom_common::range::SourceRange;
    use phalcom_modules::DeclarationId;
    use phalcom_modules::identity::ModuleId;

    fn nominal(store: &mut TypeStore, name: &str) -> TypeId {
        store.nominal(DeclarationId::new(
            ModuleId::core(),
            name.into(),
        ))
    }

    #[test]
    fn required_composition_is_established_only_from_established_inputs() {
        let mut store = TypeStore::new();
        let int_ty = nominal(&mut store, "Int");
        let string_ty = nominal(&mut store, "String");

        let result = compose_required_knowledge(
            [
                TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax),
                TypeKnowledge::established(string_ty, EvidenceOrigin::Syntax),
            ],
            EvidenceOrigin::Syntax,
            |types| Ok(store.union(types)),
        );

        assert_eq!(result.status(), Some(EvidenceStatus::Established));
        assert!(result.ty().is_some());
    }

    #[test]
    fn required_composition_weakens_when_any_input_is_assumed() {
        let mut store = TypeStore::new();
        let int_ty = nominal(&mut store, "Int");

        let result = compose_required_knowledge(
            [
                TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax),
                TypeKnowledge::assumed(int_ty, EvidenceOrigin::DeveloperAnnotation),
            ],
            EvidenceOrigin::Syntax,
            |types| Ok(store.union(types)),
        );

        assert_eq!(result.ty(), Some(int_ty));
        assert_eq!(result.status(), Some(EvidenceStatus::Assumed));
        assert_eq!(result.origin(), Some(EvidenceOrigin::Syntax));
    }

    #[test]
    fn required_composition_unknown_is_absorbing() {
        let mut store = TypeStore::new();
        let int_ty = nominal(&mut store, "Int");
        let unknown = UnknownReason::UnresolvedName("missing".into());

        let result = compose_required_knowledge(
            [
                TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax),
                TypeKnowledge::Unknown(unknown.clone()),
            ],
            EvidenceOrigin::Syntax,
            |_| panic!("builder must not run when a required input is Unknown"),
        );

        assert_eq!(result, TypeKnowledge::Unknown(unknown));
    }

    #[test]
    fn required_composition_dynamic_is_absorbing_after_unknown_check() {
        let mut store = TypeStore::new();
        let int_ty = nominal(&mut store, "Int");
        let reason = DynamicReason::ExplicitEscape;

        let result = compose_required_knowledge(
            [
                TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax),
                TypeKnowledge::Dynamic(reason.clone()),
            ],
            EvidenceOrigin::Syntax,
            |_| panic!("builder must not run when a required input is Dynamic"),
        );

        assert_eq!(result, TypeKnowledge::Dynamic(reason));
    }

    #[test]
    fn required_composition_does_not_call_builder_for_unknown_input() {
        let result = compose_required_knowledge(
            [TypeKnowledge::Unknown(
                UnknownReason::UnresolvedName("missing".into()),
            )],
            EvidenceOrigin::Syntax,
            |_| panic!("Unknown input reached the builder"),
        );

        assert!(matches!(
            result,
            TypeKnowledge::Unknown(UnknownReason::UnresolvedName(_))
        ));
    }

    #[test]
    fn required_composition_does_not_call_builder_for_dynamic_input() {
        let result = compose_required_knowledge(
            [TypeKnowledge::Dynamic(DynamicReason::RuntimeReflection)],
            EvidenceOrigin::Syntax,
            |_| panic!("Dynamic input reached the builder"),
        );

        assert_eq!(
            result,
            TypeKnowledge::Dynamic(DynamicReason::RuntimeReflection),
        );
    }

    #[test]
    fn required_composition_preserves_component_provenance() {
        let mut store = TypeStore::new();
        let int_ty = nominal(&mut store, "Int");

        let left = TypeKnowledge::established(
            int_ty,
            EvidenceOrigin::Syntax,
        )
        .with_range(SourceRange::default());

        let right = TypeKnowledge::established(
            int_ty,
            EvidenceOrigin::Syntax,
        )
        .with_range(SourceRange::default());

        let result = compose_required_knowledge(
            [left, right],
            EvidenceOrigin::Syntax,
            |types| Ok(store.union(types)),
        );

        let TypeKnowledge::Known(evidence) = result else {
            panic!("expected known required composition");
        };

        assert_eq!(evidence.provenance().ranges.len(), 2);
    }
}
```

Use nominal test types from a local `TypeStore` exactly as existing `evidence.rs` or semantic-foundation tests do.

Example structure for assumption weakening:

```rust
let established =
    TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax);
let assumed =
    TypeKnowledge::assumed(int_ty, EvidenceOrigin::DeveloperAnnotation);

let result = compose_required_knowledge(
    [established, assumed],
    EvidenceOrigin::Syntax,
    |types| Ok(store.union(types)),
);

assert_eq!(result.ty(), Some(int_ty));
assert_eq!(result.status(), Some(EvidenceStatus::Assumed));
assert_eq!(result.origin(), Some(EvidenceOrigin::Syntax));
```

- [ ] Run:

```bash
cargo test -p phalcom-semantic --lib required_composition
```

Expected RED: function does not exist.

## Step 3.2 — Refactor reason reducers only enough for reuse

- [ ] Keep current `join_unknown_reason` and `join_dynamic_reason` semantics unchanged.

- [ ] If necessary, extract these two local helpers:

```rust
fn joined_unknown_reason(inputs: &[TypeKnowledge]) -> Option<UnknownReason> {
    inputs
        .iter()
        .filter_map(|knowledge| match knowledge {
            TypeKnowledge::Unknown(reason) => Some(reason.clone()),
            _ => None,
        })
        .reduce(join_unknown_reason)
}

fn joined_dynamic_reason(inputs: &[TypeKnowledge]) -> Option<DynamicReason> {
    inputs
        .iter()
        .filter_map(|knowledge| match knowledge {
            TypeKnowledge::Dynamic(reason) => Some(reason.clone()),
            _ => None,
        })
        .reduce(join_dynamic_reason)
}
```

- [ ] Replace the duplicate extraction inside `join_type_knowledge()` with these helpers so the reason algebra has one implementation.

## Step 3.3 — Implement `compose_required_knowledge`

- [ ] Add:

```rust
pub(crate) fn compose_required_knowledge(
    inputs: impl IntoIterator<Item = TypeKnowledge>,
    origin: EvidenceOrigin,
    build_type: impl FnOnce(&[TypeId]) -> Result<TypeId, UnknownReason>,
) -> TypeKnowledge {
    let inputs = inputs.into_iter().collect::<Vec<_>>();

    if let Some(reason) = joined_unknown_reason(&inputs) {
        return TypeKnowledge::Unknown(reason);
    }

    if let Some(reason) = joined_dynamic_reason(&inputs) {
        return TypeKnowledge::Dynamic(reason);
    }

    let evidence = inputs
        .into_iter()
        .map(|knowledge| match knowledge {
            TypeKnowledge::Known(evidence) => evidence,
            TypeKnowledge::Unknown(_) | TypeKnowledge::Dynamic(_) => {
                unreachable!("Unknown/Dynamic handled before known composition")
            }
        })
        .collect::<Vec<_>>();

    let types = evidence
        .iter()
        .map(TypeEvidence::ty)
        .collect::<Vec<_>>();

    let ty = match build_type(&types) {
        Ok(ty) => ty,
        Err(reason) => return TypeKnowledge::Unknown(reason),
    };

    let status = if evidence
        .iter()
        .all(|item| item.status() == EvidenceStatus::Established)
    {
        EvidenceStatus::Established
    } else {
        EvidenceStatus::Assumed
    };

    let mut result = match status {
        EvidenceStatus::Established => TypeKnowledge::established(ty, origin),
        EvidenceStatus::Assumed => TypeKnowledge::assumed(ty, origin),
    };

    if let TypeKnowledge::Known(result_evidence) = &mut result {
        for input in evidence {
            result_evidence
                .provenance
                .ranges
                .extend(input.provenance.ranges);
            result_evidence
                .provenance
                .descriptions
                .extend(input.provenance.descriptions);
        }
    }

    result
}
```

Because this code is inside `evidence.rs`, it can access private `TypeEvidence` fields. Do not make those fields public.

## Step 3.4 — Empty input test

- [ ] Add a unit test proving the builder owns empty-input semantics:

```rust
#[test]
fn required_composition_delegates_empty_shape_to_builder() {
    let mut store = TypeStore::new();
    let unit = store.unit();

    let result = compose_required_knowledge(
        std::iter::empty(),
        EvidenceOrigin::Syntax,
        |_| Ok(unit),
    );

    assert_eq!(result.ty(), Some(unit));
    assert_eq!(result.status(), Some(EvidenceStatus::Established));
}
```

This prevents the helper from hard-coding list/set empty-literal behavior.

## Step 3.5 — Verify

- [ ] Run:

```bash
cargo test -p phalcom-semantic --lib required_composition
cargo test -p phalcom-semantic --test semantic semantic::foundations::knowledge
cargo fmt --check
```

- [ ] Commit:

```bash
git add phalcom-semantic/src/types/evidence.rs
git commit -m "fix(semantic): add required knowledge composition algebra"
```

---

# 6. Task 4 — Add Required-Expression Dependency Propagation

**Deliverable:** Aggregate producers can combine child causal/status/explanation information without implementing their own terminal-state rules.

**Files:**
- Create: `phalcom-semantic/src/checker/composition.rs`
- Modify: `phalcom-semantic/src/checker/mod.rs`
- Unit tests: `checker/composition.rs`

**Interfaces:**
- Produces: `propagate_required_dependencies(result, operands)`
- Produces internal helpers for terminal precedence
- Later produces static projection helpers in Task 8

## Step 4.1 — Register module

- [ ] Add to `checker/mod.rs`:

```rust
pub(crate) mod composition;
```

Keep it checker-internal.

## Step 4.2 — Create `composition.rs`

- [ ] Start with:

```rust
use crate::checker::analysis::AnalysisStatus;
use crate::checker::causal::CausalInvalidity;
use crate::checker::typed_expr::TypedExpression;
```

## Step 4.3 — Implement deterministic non-Invalid terminal precedence helper

- [ ] Add:

```rust
fn terminal_priority(status: &AnalysisStatus) -> u8 {
    match status {
        AnalysisStatus::InternalFailure(_) => 7,
        AnalysisStatus::Cancelled => 6,
        AnalysisStatus::BudgetExceeded(_) => 5,
        AnalysisStatus::Blocked(_) => 4,
        AnalysisStatus::Suppressed(_) => 3,
        AnalysisStatus::DynamicBoundary(_) => 2,
        AnalysisStatus::Invalid(_) => 1,
        AnalysisStatus::Ready => 0,
    }
}
```

`Invalid` cannot simply be selected by this priority. It is handled conditionally in the operand loop.

## Step 4.4 — Implement explanation de-duplication helper

- [ ] Add:

```rust
fn push_unique<T: Eq + Copy>(items: &mut Vec<T>, value: T) {
    if !items.contains(&value) {
        items.push(value);
    }
}
```

If `ExplanationId` is not `Copy`, use `Clone` rather than changing the identity type.

## Step 4.5 — Implement `propagate_required_dependencies`

- [ ] Add the function in this form:

```rust
pub(crate) fn propagate_required_dependencies(
    result: &mut TypedExpression,
    operands: &[TypedExpression],
) {
    let mut selected_terminal: Option<AnalysisStatus> = None;

    for operand in operands {
        result.causal_invalidity = result
            .causal_invalidity
            .join(operand.causal_invalidity);

        for parent in &operand.explanation_parents {
            push_unique(&mut result.explanation_parents, *parent);
        }

        match &operand.status {
            AnalysisStatus::Invalid(_) if operand.knowledge.ty().is_some() => {
                // The required type premise survived. Keep the parent
                // analyzable and carry the causal dependency only.
            }

            AnalysisStatus::Invalid(_) => {
                if let Some(cause) = result.causal_invalidity.suppression_cause() {
                    let candidate = AnalysisStatus::Suppressed(cause);
                    select_terminal(&mut selected_terminal, candidate);
                }
            }

            AnalysisStatus::Ready => {}

            status => {
                select_terminal(&mut selected_terminal, status.clone());
            }
        }
    }

    if let Some(status) = selected_terminal {
        if terminal_priority(&status) >= terminal_priority(&result.status) {
            result.status = status;
        }
    }

    result.debug_assert_coherent();
}
```

- [ ] Add:

```rust
fn select_terminal(
    selected: &mut Option<AnalysisStatus>,
    candidate: AnalysisStatus,
) {
    match selected {
        None => *selected = Some(candidate),
        Some(current)
            if terminal_priority(&candidate) > terminal_priority(current) =>
        {
            *selected = Some(candidate);
        }
        Some(_) => {}
    }
}
```

## Step 4.6 — Add low-level tests

- [ ] Add direct unit tests in `composition.rs` for:

1. `Invalid + Known` -> parent remains `Ready`, causal is non-clean.
2. `Invalid + Unknown` -> parent becomes `Suppressed`.
3. `Suppressed` -> parent remains suppressed.
4. `Cancelled` -> parent cancelled.
5. `BudgetExceeded` -> parent budget exceeded.
6. `InternalFailure + Cancelled` -> internal failure wins.
7. causal states from all required operands are joined.

Construct known/unknown `TypedExpression`s directly.

If creation of internal incident IDs/reports requires constructors, use the repository's existing constructors rather than exposing fields.

## Step 4.7 — Verify

- [ ] Run:

```bash
cargo test -p phalcom-semantic --lib checker::composition
cargo fmt --check
```

- [ ] Commit:

```bash
git add \
  phalcom-semantic/src/checker/composition.rs \
  phalcom-semantic/src/checker/mod.rs

git commit -m "fix(semantic): add required expression dependency propagation"
```

---

# 7. Task 5 — Migrate List and Set Literals

**Deliverable:** Unknown/Dynamic/Assumed required elements cannot disappear from homogeneous collection inference.

**Files:**
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/expression_composition.rs`

**Interfaces consumed:**
- `compose_required_knowledge`
- `propagate_required_dependencies`

## Step 5.1 — Add list RED tests

- [ ] Add:

```rust
#[test]
fn list_unknown_element_does_not_disappear() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let xs = [1, missing]
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        phalcom_semantic::identity::DispatchSide::Class,
    );

    let list = fixture.expression(run, "[1, missing]");
    assert_eq!(
        list.knowledge,
        TypeKnowledge::Unknown(
            UnknownReason::UnresolvedName("missing".into())
        ),
    );

    let xs = fixture.binding(run, "xs");
    assert_eq!(
        xs.current,
        TypeKnowledge::Unknown(
            UnknownReason::UnresolvedName("missing".into())
        ),
    );
}
```

Expected current RED: list is established from the known `Int` element.

- [ ] Add:

```rust
#[test]
fn assumed_list_element_weakens_aggregate() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run(_ value: Int) {
    let xs = [1, value]
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        phalcom_semantic::identity::DispatchSide::Class,
    );
    let xs = fixture.binding(run, "xs");

    assert_eq!(
        xs.current.status(),
        Some(phalcom_semantic::types::evidence::EvidenceStatus::Assumed),
    );
}
```

Expected RED: current list helper always calls `TypedExpression::established`.

## Step 5.2 — Rewrite `synthesize_list_literal`

- [ ] Locate and delete this old local policy:

```rust
let mut elem_tys = Vec::new();

for el in &list.elements {
    match el {
        ListLiteralElement::Element { expr, .. } => {
            let k = analyze_expression(ctx, expr, &expected_elem);
            if let Some(ty) = k.knowledge.ty() {
                elem_tys.push(ty);
            }
        }
        ListLiteralElement::Expansion { expr, .. } => {
            analyze_expression(ctx, expr, &ExpectedType::None);
        }
    }
}
```

- [ ] Replace the direct-element branch with collection of full products:

```rust
let mut operands = Vec::new();
let mut contributions = Vec::new();

for el in &list.elements {
    match el {
        ListLiteralElement::Element { expr, .. } => {
            let typed = analyze_expression(ctx, expr, &expected_elem);
            contributions.push(typed.knowledge.clone());
            operands.push(typed);
        }

        ListLiteralElement::Expansion { .. } => {
            // Task 8 replaces this branch. Until then, keep existing
            // expansion handling unchanged so this commit changes only direct
            // element semantics.
        }
    }
}
```

- [ ] Preserve the existing empty-list behavior:

```rust
if contributions.is_empty() && list.elements.is_empty() {
    return TypedExpression::unknown(UnknownReason::NoTypeEvidence);
}
```

Do not use expected type as substitute evidence.

- [ ] Resolve `List` form once, then build knowledge:

```rust
let Some(decl) = list_decl else {
    return TypedExpression::unknown(UnknownReason::UnannotatedDeclaration);
};

let kind = ctx.store.arrow_kind(
    vec![KindId::TYPE].into_boxed_slice(),
    KindId::TYPE,
);
let form = ctx.store.nominal_form(decl, kind);

let knowledge = crate::types::evidence::compose_required_knowledge(
    contributions,
    EvidenceOrigin::Syntax,
    |types| {
        if types.is_empty() {
            return Err(UnknownReason::NoTypeEvidence);
        }

        let element = ctx.store.union(types);
        ctx.store
            .list_of(form, element)
            .map_err(|_| UnknownReason::UncheckedExpression)
    },
);
```

- [ ] Construct and propagate:

```rust
let mut result = TypedExpression::new(
    match knowledge {
        TypeKnowledge::Known(_) => knowledge.with_range(list.range),
        other => other,
    },
);

crate::checker::composition::propagate_required_dependencies(
    &mut result,
    &operands,
);

result
```

If borrow checking rejects capturing `ctx.store` from inside the builder while calling the free function, split type aggregation from constructor application using a small local enum/result but keep the semantic decision in `compose_required_knowledge`. Do not fall back to filtering known `TypeId`s.

## Step 5.3 — Add Set RED tests

- [ ] Mirror the two list tests for the parser's canonical set literal syntax.

Use an already-valid set literal fixture from `expression_engine.rs` / existing tests rather than guessing syntax.

Required assertions:

```text
known + unknown -> Unknown(reason)
known + assumed -> Assumed(Set<joined element type>)
```

## Step 5.4 — Rewrite `synthesize_set_literal`

- [ ] Replace the `.knowledge.ty()` filter exactly as for list.
- [ ] Use `ctx.store.set_of(form, element_ty)`.
- [ ] Preserve empty-set behavior currently defined by the analyzer.
- [ ] Use `propagate_required_dependencies`.

## Step 5.5 — Verify

- [ ] Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::expression_composition::list_unknown_element_does_not_disappear

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::expression_composition::assumed_list_element_weakens_aggregate

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::expression_composition

cargo fmt --check
```

- [ ] Commit:

```bash
git add \
  phalcom-semantic/src/checker/expression.rs \
  phalcom-semantic/tests/semantic/foundations/expression_composition.rs

git commit -m "fix(semantic): preserve list and set element evidence"
```

---

# 8. Task 6 — Migrate Map Literals

**Deliverable:** Key and value lanes are both required premises; an unknown/dynamic lane cannot be dropped while the other lane establishes a partial `Map<K,V>`.

**Files:**
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/expression_composition.rs`

## Step 6.1 — Add RED tests for both lanes

- [ ] Find the parser-supported computed-map-key syntax in existing map tests; use it exactly.

- [ ] Add:

```rust
#[test]
fn map_unknown_value_does_not_disappear() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let values = { key: missing }
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        phalcom_semantic::identity::DispatchSide::Class,
    );
    let map = fixture.expression(run, "{ key: missing }");

    assert_eq!(
        map.knowledge,
        TypeKnowledge::Unknown(
            UnknownReason::UnresolvedName("missing".into()),
        ),
    );
}

#[test]
fn map_unknown_computed_key_does_not_disappear() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let values = { [missing]: 1 }
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        phalcom_semantic::identity::DispatchSide::Class,
    );
    let map = fixture.expression(run, "{ [missing]: 1 }");

    assert_eq!(
        map.knowledge,
        TypeKnowledge::Unknown(
            UnknownReason::UnresolvedName("missing".into()),
        ),
    );
}

#[test]
fn assumed_map_value_weakens_map_evidence() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run(_ value: Int) {
    let values = { key: value }
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        phalcom_semantic::identity::DispatchSide::Class,
    );
    let map = fixture.expression(run, "{ key: value }");

    assert_eq!(
        map.knowledge.status(),
        Some(
            phalcom_semantic::types::evidence::EvidenceStatus::Assumed,
        ),
    );
}
```

For the first test, use a known literal key and unresolved value.

For the second, use a computed key expression, not a bare-symbol key if bare symbols are canonical string/symbol syntax.

Expected RED: current helper filters missing key/value type out of its corresponding `Vec<TypeId>`.

## Step 6.2 — Replace `key_tys` / `val_tys`

- [ ] Delete:

```rust
let mut key_tys = Vec::new();
let mut val_tys = Vec::new();
```

- [ ] Introduce:

```rust
let mut operands = Vec::new();
let mut key_knowledge = Vec::new();
let mut value_knowledge = Vec::new();
```

## Step 6.3 — Computed key handling

- [ ] Replace:

```rust
MapLiteralKey::Computed { expr, .. } =>
    analyze_expression(ctx, expr, &expected_key).knowledge.ty()
```

with:

```rust
MapLiteralKey::Computed { expr, .. } => {
    let typed = analyze_expression(ctx, expr, &expected_key);
    key_knowledge.push(typed.knowledge.clone());
    operands.push(typed);
}
```

## Step 6.4 — Bare symbol key handling

- [ ] Preserve the current language rule that a bare-symbol map key supplies a canonical string/symbol key type.

- [ ] Instead of pushing a raw `TypeId`, push an established `TypeKnowledge`:

```rust
let key_ty = ctx.nominal_type_of(string_decl);
key_knowledge.push(
    TypeKnowledge::established(
        key_ty,
        EvidenceOrigin::Syntax,
    ),
);
```

If the canonical key type is `Symbol` on the actual baseline rather than `String`, use the implementation's existing declaration resolution; do not change language semantics in this plan.

## Step 6.5 — Value handling

- [ ] Replace raw type filtering with:

```rust
let typed = analyze_expression(ctx, value, &expected_val);
value_knowledge.push(typed.knowledge.clone());
operands.push(typed);
```

## Step 6.6 — Compose key and value lane knowledge

- [ ] Use `compose_required_knowledge` once per lane to compute the union type with correct evidence status/reason.

The lane result can be represented as a `TypeKnowledge` whose `TypeId` is the union of key/value contributions:

```rust
let key_lane = compose_required_knowledge(
    key_knowledge,
    EvidenceOrigin::Syntax,
    |types| {
        if types.is_empty() {
            Err(UnknownReason::NoTypeEvidence)
        } else {
            Ok(ctx.store.union(types))
        }
    },
);
```

Do the same for values.

## Step 6.7 — Compose the two lanes into `Map<K,V>`

- [ ] Then call `compose_required_knowledge` again with:

```rust
[key_lane, value_lane]
```

and a builder that requires exactly two `TypeId`s:

```rust
|types| {
    let [key, value] = types else {
        return Err(UnknownReason::UncheckedExpression);
    };

    ctx.store
        .map_of(form, *key, *value)
        .map_err(|_| UnknownReason::UncheckedExpression)
}
```

This second composition is what guarantees an Assumed key or value lane weakens the whole map.

## Step 6.8 — Propagate operand state

- [ ] Build `TypedExpression::new(map_knowledge)`.
- [ ] Call `propagate_required_dependencies(&mut result, &operands)`.

## Step 6.9 — Verify and commit

- [ ] Run all new map tests and the complete `expression_composition` module.

- [ ] Run:

```bash
cargo fmt --check
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::expression_engine
```

- [ ] Commit:

```bash
git add \
  phalcom-semantic/src/checker/expression.rs \
  phalcom-semantic/tests/semantic/foundations/expression_composition.rs

git commit -m "fix(semantic): require complete map key and value evidence"
```

---

# 9. Task 7 — Migrate Tuple and Record Direct Members

**Deliverable:** Direct tuple/record members preserve exact Unknown/Dynamic state and weakest evidence instead of replacing all non-known values with `UncheckedExpression`.

**Files:**
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/expression_composition.rs`

## Step 7.1 — Add tuple RED tests

- [ ] Add:

```rust
#[test]
fn tuple_preserves_unresolved_member_reason() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let pair = (1, missing)
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        phalcom_semantic::identity::DispatchSide::Class,
    );
    let pair = fixture.expression(run, "(1, missing)");

    assert_eq!(
        pair.knowledge,
        TypeKnowledge::Unknown(
            UnknownReason::UnresolvedName("missing".into()),
        ),
    );
}

#[test]
fn assumed_tuple_member_weakens_tuple_evidence() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run(_ value: Int) {
    let pair = (1, value)
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        phalcom_semantic::identity::DispatchSide::Class,
    );
    let pair = fixture.expression(run, "(1, value)");

    assert_eq!(
        pair.knowledge.status(),
        Some(
            phalcom_semantic::types::evidence::EvidenceStatus::Assumed,
        ),
    );
}
```

Use:

```phalcom
let pair = (1, missing)
```

and:

```phalcom
class Probe {
  @class
  run(_ value: Int) {
    let pair = (1, value)
  }
}
```

Expected RED:
- current unknown path returns `UncheckedExpression`;
- current known path always returns `Established`.

## Step 7.2 — Rewrite direct tuple member collection

- [ ] Keep label extraction exactly as currently implemented.
- [ ] Replace the current early-return pattern `let Some(ty) = k.knowledge.ty() else { return TypedExpression::unknown(UnknownReason::UncheckedExpression); };` with:

```rust
let typed = analyze_expression(ctx, expr, &ExpectedType::None);
labels.push(None); // or existing extracted label
knowledge.push(typed.knowledge.clone());
operands.push(typed);
```

Maintain a parallel label vector so evidence composition stays concerned only with `TypeId`s.

- [ ] Build tuple type from the `types` slice:

```rust
let knowledge = compose_required_knowledge(
    knowledge,
    EvidenceOrigin::Syntax,
    |types| {
        let elements = labels
            .iter()
            .cloned()
            .zip(types.iter().copied())
            .map(|(label, ty)| TupleTypeElement { label, ty })
            .collect::<Vec<_>>();

        Ok(ctx.store.tuple(elements.into_boxed_slice()))
    },
);
```

- [ ] Propagate required dependencies.

## Step 7.3 — Add record RED tests

- [ ] Add unresolved-field and assumed-field tests.

Use an existing valid record literal syntax from the repository.

Assert exact `UnknownReason::UnresolvedName("missing".into())`.

## Step 7.4 — Rewrite direct record member collection

- [ ] Preserve exact label/name extraction.
- [ ] Collect child `TypeKnowledge` and `TypedExpression`.
- [ ] Build `RecordTypeField { name, ty }` only inside the all-Known builder.
- [ ] Use `EvidenceOrigin::Syntax`.
- [ ] Propagate dependencies.

Do not silently manufacture `"field"` for a label form that current parser/language considers invalid if an existing diagnostic path exists. Preserve current behavior outside the evidence change.

## Step 7.5 — Verify

- [ ] Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::expression_composition

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::expression_engine

cargo fmt --check
```

- [ ] Commit:

```bash
git add \
  phalcom-semantic/src/checker/expression.rs \
  phalcom-semantic/tests/semantic/foundations/expression_composition.rs

git commit -m "fix(semantic): preserve tuple and record member evidence"
```

---

# 10. Task 8 — Implement Expansion Projection and Fail-Closed Semantics

**Deliverable:** Expansion operands are never analyzed and ignored. Exact statically projectable expansions contribute soundly; uncertain/non-projectable expansions make the aggregate honestly unknown/dynamic.

**Files:**
- Modify: `phalcom-semantic/src/checker/composition.rs`
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/expression_composition.rs`

**Important:** This task must follow the current AST/language grammar. Do not broaden expansion syntax. The collection specification has historical/superseded sections; use parser-supported constructs on current `main`.

## Step 8.1 — Add a pure applied-argument projector

- [ ] In `composition.rs`, import:

```rust
use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
use crate::types::id::TypeId;
use crate::types::store::{TypeData, TypeStore};
```

- [ ] Add:

```rust
pub(crate) fn project_applied_argument(
    store: &TypeStore,
    knowledge: &TypeKnowledge,
    expected_origin: TypeId,
    argument_index: usize,
) -> TypeKnowledge {
    match knowledge {
        TypeKnowledge::Known(_) => {
            let Some(source_ty) = knowledge.ty() else {
                unreachable!("Known knowledge has a type");
            };

            match store.get(source_ty) {
                TypeData::Applied { origin, arguments }
                    if *origin == expected_origin =>
                {
                    let Some(argument) = arguments.get(argument_index).copied() else {
                        return TypeKnowledge::Unknown(
                            UnknownReason::UncheckedExpression,
                        );
                    };

                    knowledge.derive_known_type(
                        argument,
                        EvidenceOrigin::PatternDecomposition,
                    )
                }

                _ => TypeKnowledge::Unknown(
                    UnknownReason::UncheckedExpression,
                ),
            }
        }

        TypeKnowledge::Unknown(reason) => {
            TypeKnowledge::Unknown(reason.clone())
        }

        TypeKnowledge::Dynamic(reason) => {
            TypeKnowledge::Dynamic(reason.clone())
        }
    }
}
```

If `derive_known_type` is inaccessible from `checker`, use `map_type(|_| argument)` and preserve the existing origin rather than widening visibility. Do not make evidence internals public. The technical spec permits origin preservation for a pure projection; explanation nodes can represent the projection separately.

## Step 8.2 — Add pure tuple expansion projector

- [ ] Add a function returning a projected list of component knowledge:

```rust
pub(crate) fn project_tuple_elements(
    store: &TypeStore,
    knowledge: &TypeKnowledge,
) -> Result<Vec<TypeKnowledge>, TypeKnowledge>
```

Semantics:

- Known exact tuple -> return one derived `TypeKnowledge` per element, preserving parent evidence status/provenance.
- Unknown -> `Err(Unknown(reason.clone()))`.
- Dynamic -> `Err(Dynamic(reason.clone()))`.
- Known non-tuple -> `Err(Unknown(UncheckedExpression))`.

Do not return an empty vector for non-projectable input; that would recreate the disappearance bug.

## Step 8.3 — Add closed-record projector

- [ ] Add a helper that returns `(field name, TypeKnowledge)` pairs only when the source type is a closed canonical record row.

Inspect current `TypeData::Record` and `TypeStore` row-access API at execution time and use the existing accessors. Required behavior:

```text
Known(closed record) -> exact fields
Known(open row)      -> Unknown(UncheckedExpression)
Known(non-record)    -> Unknown(UncheckedExpression)
Unknown(R)           -> Unknown(R)
Dynamic(D)           -> Dynamic(D)
```

Do not close an open row by dropping its tail.

## Step 8.4 — Write RED tests before migrating each expansion kind

- [ ] For every expansion syntax currently accepted in literal construction, add at least:

1. exact known projectable expansion;
2. unknown expansion;
3. assumed known projectable expansion;
4. known non-projectable expansion if parser/type setup permits it.

The key assertion for every family:

```text
unknown expansion must not be observationally equivalent to no expansion
```

## Step 8.5 — Migrate list/set expansion branches

- [ ] Replace:

```rust
ListLiteralElement::Expansion { expr, .. } => {
    analyze_expression(ctx, expr, &ExpectedType::None);
}
```

with:

1. analyze full `TypedExpression`;
2. project element contribution from the exact expected collection form if sound;
3. push projected knowledge into the same `contributions` vector as direct members;
4. push the original typed expression into `operands`.

If the current list `*` semantics are generic iterable/cursor based rather than exact `List<T>` spread, do **not** invent `first generic argument == element type`. In that case, use a fail-closed projection for non-exact List/Set until Technical Spec 02 routes protocol extraction through canonical callable application.

## Step 8.6 — Migrate map expansion branch

- [ ] Replace analyze-and-ignore with exact `Map<K,V>` lane projection where statically provable.

Unknown/Dynamic/non-projectable expansion contributes uncertainty to both map lanes.

## Step 8.7 — Migrate tuple expansion

- [ ] For exact tuple knowledge, splice projected elements and labels into the local tuple shape arrays.

- [ ] For Unknown/Dynamic/non-projectable tuple expansion, add that knowledge as a required blocker for the entire result; do not skip it.

## Step 8.8 — Migrate record expansion

- [ ] For a closed record row, splice exact fields.
- [ ] For open row, Unknown, Dynamic, or non-record, fail closed.
- [ ] Preserve duplicate-field behavior from the existing canonical record builder/diagnostic path; do not invent source-order override semantics.

## Step 8.9 — Verify

- [ ] Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::expression_composition

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::expression_engine

cargo fmt --check
```

- [ ] Commit:

```bash
git add \
  phalcom-semantic/src/checker/composition.rs \
  phalcom-semantic/src/checker/expression.rs \
  phalcom-semantic/tests/semantic/foundations/expression_composition.rs

git commit -m "fix(semantic): make aggregate expansion evidence explicit"
```

---

# 11. Task 9 — Preserve Pattern Decomposition Knowledge and Causal State

**Deliverable:** Tuple pattern decomposition preserves parent Unknown/Dynamic evidence and causal invalidity instead of synthesizing fresh `NoTypeEvidence`.

**Files:**
- Modify: `phalcom-semantic/src/checker/composition.rs`
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/checker/statement.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/expression_composition.rs`

## Step 9.1 — Add decomposition helper

- [ ] Add:

```rust
pub(crate) fn decompose_tuple_component(
    store: &TypeStore,
    parent: &TypeKnowledge,
    index: usize,
    expected_len: usize,
) -> TypeKnowledge {
    match parent {
        TypeKnowledge::Known(_) => {
            let Some(parent_ty) = parent.ty() else {
                unreachable!("Known knowledge has a type");
            };

            match store.get(parent_ty) {
                TypeData::Tuple(elements)
                    if elements.len() == expected_len =>
                {
                    let Some(element) = elements.get(index) else {
                        return TypeKnowledge::Unknown(
                            UnknownReason::UncheckedExpression,
                        );
                    };

                    parent.derive_known_type(
                        element.ty,
                        EvidenceOrigin::PatternDecomposition,
                    )
                }

                _ => TypeKnowledge::Unknown(
                    UnknownReason::UncheckedExpression,
                ),
            }
        }

        TypeKnowledge::Unknown(reason) => {
            TypeKnowledge::Unknown(reason.clone())
        }

        TypeKnowledge::Dynamic(reason) => {
            TypeKnowledge::Dynamic(reason.clone())
        }
    }
}
```

Again, if checker visibility blocks `derive_known_type`, preserve evidence through `map_type` and use explanation provenance rather than widening visibility.

## Step 9.2 — Add causal-aware pattern binding API

- [ ] In `context.rs`, add:

```rust
pub fn bind_pattern_binding_with_causal(
    &mut self,
    name: impl Into<String>,
    fact: ValueSemanticFact,
    range: SourceRange,
    causal_invalidity: CausalInvalidity,
) -> BindingDeclarationResult {
    let contract = fact.knowledge.ty().map(|ty| BindingContract {
        ty,
        origin: BindingContractOrigin::PatternBinding,
        source: Some(range),
    });

    self.declare_binding(BindingSeed {
        name: name.into(),
        range,
        contract,
        current: fact.knowledge,
        denotation: fact.denotation,
        causal_invalidity,
        mutable: true,
    })
}
```

- [ ] Replace current `bind_pattern_binding` body with delegation:

```rust
pub fn bind_pattern_binding(
    &mut self,
    name: impl Into<String>,
    fact: ValueSemanticFact,
    range: SourceRange,
) -> BindingDeclarationResult {
    self.bind_pattern_binding_with_causal(
        name,
        fact,
        range,
        CausalInvalidity::Clean,
    )
}
```

## Step 9.3 — Rewrite `bind_pattern`

- [ ] Change signature from:

```rust
fn bind_pattern(
    ctx: &mut CheckingContext<'_>,
    pattern: &Pattern,
    fact: ValueSemanticFact,
)
```

to:

```rust
fn bind_pattern(
    ctx: &mut CheckingContext<'_>,
    pattern: &Pattern,
    fact: ValueSemanticFact,
    causal_invalidity: CausalInvalidity,
)
```

- [ ] Name branch:

```rust
Pattern::Name { name, range, .. } => {
    ctx.bind_pattern_binding_with_causal(
        name.clone(),
        fact,
        *range,
        causal_invalidity,
    );
}
```

- [ ] Tuple branch: delete the current `component_facts: Option<Vec<_>>` + `unwrap_or(NoTypeEvidence)` structure.

- [ ] Replace with one component call per element:

```rust
for (index, element) in elements.iter().enumerate() {
    let component = crate::checker::composition::decompose_tuple_component(
        ctx.store,
        &fact.knowledge,
        index,
        elements.len(),
    );

    bind_pattern(
        ctx,
        element,
        ValueSemanticFact::new(component),
        causal_invalidity,
    );
}
```

Preserve denotation decomposition only where the current implementation actually has component denotations; do not fabricate them.

## Step 9.4 — Rewrite `bind_declaration_pattern`

- [ ] Apply the same knowledge-decomposition logic.

- [ ] Keep its existing explicit contract/mutability behavior.

- [ ] Every recursively created binding gets the same incoming `causal_invalidity` unless a component operation adds a new cause.

## Step 9.5 — Add RED tests

- [ ] Add a test where an unresolved/unknown tuple source is destructured and verify each resulting binding retains the original `UnknownReason`.

- [ ] Add a low-level test for:

```text
Dynamic(D) -> decomposed Dynamic(D)
```

if source syntax cannot directly produce the needed DynamicReason.

- [ ] Add a causal propagation test. Reuse an invalid-but-known tuple-valued binding if possible; otherwise construct a focused checker/flow test. Assert decomposed children are analyzable but non-clean, not automatically suppressed.

## Step 9.6 — Update all `bind_pattern` call sites

- [ ] Search:

```bash
rg "bind_pattern\(" phalcom-semantic/src/checker
```

- [ ] Update every call to pass the source fact's causal invalidity explicitly.

There must be no compiler-driven hidden default to `Clean` for semantic pattern transfer.

## Step 9.7 — Verify

- [ ] Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::expression_composition

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::binding_contracts

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::flow_graph

cargo fmt --check
```

- [ ] Commit:

```bash
git add \
  phalcom-semantic/src/checker/composition.rs \
  phalcom-semantic/src/checker/context.rs \
  phalcom-semantic/src/checker/statement.rs \
  phalcom-semantic/tests/semantic/foundations/expression_composition.rs

git commit -m "fix(semantic): preserve evidence through pattern decomposition"
```

---

# 12. Task 10 — Preserve `for` Iterable Knowledge, Status, and Causal State

**Deliverable:** `for` no longer loses iterable Unknown/Dynamic reason or causal dependence by calling `synthesize_expr`.

**Files:**
- Modify: `phalcom-semantic/src/checker/statement.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/expression_composition.rs`

## Step 10.1 — Add RED unresolved-iterable test

- [ ] Add a parser-valid loop fixture:

```rust
#[test]
fn for_loop_preserves_iterable_unknown_reason() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    for value in missing {
      let copy = value
    }
  }
}
"#,
    );

    // Locate loop-bound `value` using the existing Fixture binding APIs.
    // Assert Unknown(UnresolvedName("missing")), not
    // Unknown(UnannotatedDeclaration) or NoTypeEvidence.
}
```

If the fixture binding index contains multiple same-named bindings, use `bindings_named` and select by range/order as the support API already permits.

Expected RED: current `Statement::For` rewrites to `UnannotatedDeclaration`.

## Step 10.2 — Replace `synthesize_expr`

- [ ] In `Statement::For`, replace:

```rust
let iter_k = synthesize_expr(ctx, &lane.iter);
```

with:

```rust
let iter_typed = analyze_expression(
    ctx,
    &lane.iter,
    &ExpectedType::None,
);
```

- [ ] Remove the now-unused `synthesize_expr` import from `statement.rs` if no other statement branch uses it.

## Step 10.3 — Replace knowledge-state rewrite

- [ ] Replace current:

```rust
let elem_knowledge = if let Some(iter_ty) = iter_k.ty() {
    resolve_iteration_element(ctx, iter_ty)
} else if iter_k.is_dynamic() {
    TypeKnowledge::Dynamic(DynamicReason::RuntimeReflection)
} else {
    TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration)
};
```

with:

```rust
let elem_knowledge = match &iter_typed.knowledge {
    TypeKnowledge::Known(evidence) => {
        resolve_iteration_element(ctx, evidence.ty())
    }

    TypeKnowledge::Unknown(reason) => {
        TypeKnowledge::Unknown(reason.clone())
    }

    TypeKnowledge::Dynamic(reason) => {
        TypeKnowledge::Dynamic(reason.clone())
    }
};
```

Do not convert Dynamic to `RuntimeReflection` unless `resolve_iteration_element` itself crosses that boundary.

## Step 10.4 — Carry causal state alongside lane facts

- [ ] Replace:

```rust
lane_facts.push((&lane.pattern, elem_fact));
```

with:

```rust
lane_facts.push((
    &lane.pattern,
    elem_fact,
    iter_typed.causal_invalidity,
));
```

- [ ] Replace:

```rust
for (pat, fact) in lane_facts {
    bind_pattern(ctx, pat, fact);
}
```

with:

```rust
for (pat, fact, causal_invalidity) in lane_facts {
    bind_pattern(
        ctx,
        pat,
        fact,
        causal_invalidity,
    );
}
```

## Step 10.5 — Add Dynamic reason test

- [ ] If current source syntax supports an explicit Dynamic escape that produces a distinguishable `DynamicReason`, add a source-level test.

- [ ] Otherwise add a lower-level test around the transfer helper/path. Do not change language syntax just to make this test possible.

Required assertion:

```text
Dynamic(reason_in) -> Dynamic(reason_in)
```

unless iteration dispatch itself explicitly produces a new boundary.

## Step 10.6 — Add comment marking Technical Spec 02 boundary

- [ ] Directly above `resolve_iteration_element`, add/update its documentation:

```rust
/// Derives the iteration element type through the current iteration protocol.
///
/// Semantic-correctness note: this helper returns only TypeKnowledge. It must
/// not grow a parallel callable-application semantics. Technical Spec 02
/// should eventually route protocol application through the canonical call
/// application result so status/causal/evidence authority are composed once.
```

## Step 10.7 — Verify

- [ ] Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::expression_composition

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::flow_graph

cargo fmt --check
```

- [ ] Commit:

```bash
git add \
  phalcom-semantic/src/checker/statement.rs \
  phalcom-semantic/tests/semantic/foundations/expression_composition.rs

git commit -m "fix(semantic): preserve iterable evidence in for analysis"
```

---

# 13. Task 11 — Add Fixture-Wide Product Invariant Checks

**Deliverable:** Tests can assert coherence over every published expression, not just hand-picked regressions.

**Files:**
- Modify: `phalcom-semantic/tests/semantic/support/fixture.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/expression_composition.rs`

## Step 11.1 — Add helper to `Fixture`

- [ ] Import `CausalInvalidity` / `AnalysisStatus` if not already in scope.

- [ ] Add:

```rust
pub fn assert_expression_product_invariants(&self) {
    for callable in self.analysis.snapshot.callable_analyses.values() {
        for expression in callable.expressions.values() {
            match expression.status {
                AnalysisStatus::Invalid(cause) => {
                    assert!(
                        expression.causal_invalidity.contains(cause),
                        "Invalid expression must contain owning cause: {expression:#?}",
                    );
                }

                AnalysisStatus::Suppressed(_) => {
                    assert!(
                        !matches!(
                            expression.causal_invalidity,
                            CausalInvalidity::Clean
                        ),
                        "Suppressed expression must have non-clean causal state: {expression:#?}",
                    );
                }

                _ => {}
            }
        }
    }
}
```

Do not add:

```rust
AnalysisStatus::Ready => assert_eq!(causal_invalidity, Clean)
```

because invalid-but-analyzable downstream expressions are intentionally Ready + non-clean.

## Step 11.2 — Exercise helper over diverse fixtures

- [ ] Add one test that creates several semantically interesting constructs in one fixture:

```phalcom
class CellNum {
  @constructor
  new() {}

  value() -> Int { 1 }
}

class Probe {
  @class
  run(_ assumed: Int) {
    let bad: Int = CellNum.new()
    let known = bad.value()
    let list = [1, assumed]
    let unresolved = [1, missing]
  }
}
```

- [ ] Call:

```rust
fixture.assert_expression_product_invariants();
```

The fixture already reports source diagnostics; this helper is asserting internal product coherence, not “program has no errors.”

## Step 11.3 — Consider global enablement only after focused suite is green

- [ ] Search all current semantic tests for use of `Fixture::new_allowing_internal_incidents`.

- [ ] If the invariant helper passes across the normal fixture corpus, add this call at the end of `Fixture::new()`:

```rust
fixture.assert_expression_product_invariants();
```

- [ ] If unrelated pre-existing product violations remain outside this technical slice, **do not weaken the helper**. Leave global enablement out and record the failing test names in the commit/review notes for the appropriate later semantic-correctness spec.

The final code must retain the helper either way.

## Step 11.4 — Verify

- [ ] Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::expression_composition

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::semantic_correctness_regressions

cargo fmt --check
```

- [ ] Commit:

```bash
git add \
  phalcom-semantic/tests/semantic/support/fixture.rs \
  phalcom-semantic/tests/semantic/foundations/expression_composition.rs

git commit -m "test(semantic): enforce expression product invariants"
```

---

# 14. Task 12 — Remove Residual Unsafe Patterns and Run Full Verification

**Deliverable:** No production path in the technical-spec scope still silently filters required evidence or mutates expression status/causal state independently.

**Files:** Repository audit; modify only files in this plan when a residual scoped pattern is found.

## Step 12.1 — Search for required-operand filtering

- [ ] Run:

```bash
rg -n \
  'knowledge\.ty\(\)|\.ty\(\).*push|if let Some\(.*ty.*knowledge|if let Some\(ty\)' \
  phalcom-semantic/src/checker
```

Review every hit.

For each hit, classify it explicitly:

```text
A. optional precision optimization — safe to omit
B. required semantic premise — must use composition/preservation
C. canonical call/generic work — belongs to Technical Spec 02/03
D. unrelated
```

Any category-B hit remaining in collections, patterns, or iteration is a blocker.

Do not mechanically replace every `.ty()` call; many are legitimate canonical dispatch/type-access operations.

## Step 12.2 — Search for direct Invalid status mutation

- [ ] Run:

```bash
rg -n \
  'status\s*=\s*AnalysisStatus::Invalid|with_status\(AnalysisStatus::Invalid' \
  phalcom-semantic/src/checker
```

For every hit inside this plan's scope, replace separate status/cause mutation with:

```rust
typed.invalidate(cause);
```

If a hit is not a `TypedExpression` but a published relation/application product, verify that its corresponding causal product is already updated by the owning API; do not force `TypedExpression` helpers into unrelated types.

## Step 12.3 — Search for analyze-and-ignore expansion branches

- [ ] Run:

```bash
rg -n \
  'Expansion.*|Expand.*' \
  phalcom-semantic/src/checker/expression.rs
```

Inspect all literal expansion branches covered by this plan.

No covered branch may contain only:

```rust
analyze_expression(ctx, expr, &ExpectedType::None);
```

with no semantic contribution to the aggregate result.

## Step 12.4 — Search for reason rewriting

- [ ] Run:

```bash
rg -n \
  'UnknownReason::UnannotatedDeclaration|UnknownReason::NoTypeEvidence|UnknownReason::UncheckedExpression|DynamicReason::RuntimeReflection' \
  phalcom-semantic/src/checker/statement.rs \
  phalcom-semantic/src/checker/expression.rs \
  phalcom-semantic/src/checker/composition.rs
```

For each occurrence, verify the reason is created because the **current operation** has that reason, not merely because an upstream value lacked a `TypeId`.

## Step 12.5 — Remove/narrow obsolete expression publication API

- [ ] Search:

```bash
rg -n 'record_expression\(' phalcom-semantic
```

- [ ] If `record_expression()` is no longer used by production code, remove it.
- [ ] If a legitimate compatibility/test caller remains, make it `pub(crate)` and add documentation that production expression analysis uses `publish_expression_analysis`.

No two production publication paths should remain without a reason.

## Step 12.6 — Run focused semantic suites

- [ ] Run:

```bash
cargo test -p phalcom-semantic --lib
```

- [ ] Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::expression_composition
```

- [ ] Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::semantic_correctness_regressions
```

- [ ] Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::knowledge
```

- [ ] Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::binding_contracts
```

- [ ] Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::flow_graph
```

- [ ] Run:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::bidirectional_calls
```

This last suite is a regression guard. Do not “fix” generic/call authority behavior here if new epistemic assertions expose Technical Spec 02/03 gaps; isolate them into explicit later RED regressions.

## Step 12.7 — Run package and workspace verification

- [ ] Run:

```bash
cargo fmt --check
cargo test -p phalcom-semantic
```

Because `phalcom-semantic/Cargo.toml` has `autotests = false` and explicitly declares the `semantic` test binary, the package command must still include that declared test.

- [ ] Then run the workspace suite:

```bash
cargo test --workspace
```

If workspace tests are intentionally excluded or too costly in the project's current CI workflow, run the repository's canonical CI-equivalent commands and record exactly what was run. Do not claim workspace verification without executing it.

## Step 12.8 — Review the diff against scope

- [ ] Run:

```bash
git diff --stat
git diff -- \
  phalcom-semantic/src/types/evidence.rs \
  phalcom-semantic/src/checker/causal.rs \
  phalcom-semantic/src/checker/typed_expr.rs \
  phalcom-semantic/src/checker/composition.rs \
  phalcom-semantic/src/checker/context.rs \
  phalcom-semantic/src/checker/expression.rs \
  phalcom-semantic/src/checker/statement.rs \
  phalcom-semantic/tests/semantic/foundations/mod.rs \
  phalcom-semantic/tests/semantic/foundations/expression_composition.rs \
  phalcom-semantic/tests/semantic/support/fixture.rs
```

Reject unrelated edits before final commit.

## Step 12.9 — Final commit

- [ ] If Task 12 contains code cleanup:

```bash
git add \
  phalcom-semantic/src \
  phalcom-semantic/tests/semantic

git commit -m "chore(semantic): close expression composition correctness slice"
```

If no production changes were needed after the audit, do not create an empty commit.

---

# 15. Detailed Regression Inventory

The implementation is not finished until this matrix is represented by executable tests.

| ID | Test | Expected semantic product |
|---|---|---|
| EPI-COMP-01 | `[1, missing]` | `Unknown(UnresolvedName("missing"))` |
| EPI-COMP-02 | `[1, assumedParam]` | `Assumed(List<Int>)` |
| EPI-COMP-03 | Set with unknown element | exact Unknown reason |
| EPI-COMP-04 | Map with unknown value | map Unknown |
| EPI-COMP-05 | Map with unknown computed key | map Unknown |
| EPI-COMP-06 | `(1, missing)` | exact Unknown reason, not `UncheckedExpression` |
| EPI-COMP-07 | record with unresolved field | exact Unknown reason |
| EPI-COMP-08 | tuple pattern over Unknown | components preserve parent reason |
| EPI-COMP-09 | tuple pattern over Dynamic | components preserve Dynamic reason |
| EPI-COMP-10 | `for x in missing` | loop binding preserves unresolved reason |
| EPI-COMP-11 | Dynamic iterable | element path preserves Dynamic reason unless protocol operation introduces a documented new boundary |
| EPI-COMP-12 | invalid initializer | `Invalid(C)` and `causal_invalidity.contains(C)` |
| EPI-COMP-13 | invalid-but-known required child | parent remains analyzable, causal non-clean |
| EPI-COMP-14 | invalid + unavailable required child | parent suppressed |
| EPI-COMP-15 | cancelled required child | parent Cancelled |
| EPI-COMP-16 | budget-exceeded required child | parent BudgetExceeded |
| EPI-COMP-17 | internal-failure required child | parent InternalFailure |
| EPI-COMP-18 | exact known expansion | projected contribution participates |
| EPI-COMP-19 | unknown expansion | aggregate not equivalent to expansion omission |
| EPI-COMP-20 | assumed expansion | aggregate no stronger than Assumed |

---

# 16. Code-Review Checklist for Each Task

A reviewer should reject a task if any answer below is “yes” without an explicit semantic justification.

## Evidence

- [ ] Does new code call `.knowledge.ty()` and then silently skip the operand on `None`?
- [ ] Does any new operation convert `Assumed` to `Established` just because a concrete `TypeId` exists?
- [ ] Does any projection replace an existing `UnknownReason` with a generic reason?
- [ ] Does any projection replace an existing `DynamicReason` with `RuntimeReflection` without a new runtime-reflection operation?
- [ ] Does expected/contextual type become result evidence merely by assignment?

## Status and causality

- [ ] Does code set `AnalysisStatus::Invalid(cause)` without joining `cause` into causal invalidity?
- [ ] Does code equate non-clean causal invalidity with suppression?
- [ ] Does a child with known type + invalid status prevent a parent operation that only needs that type?
- [ ] Does a suppressed product have clean causal invalidity?

## Aggregates

- [ ] Can an unknown collection element disappear from the result?
- [ ] Can an unknown map key/value lane disappear independently?
- [ ] Can a tuple/record omit an unknown member and still produce a known product?
- [ ] Can an expansion be analyzed solely for diagnostics and then ignored?

## Patterns and loops

- [ ] Does tuple decomposition manufacture `NoTypeEvidence` from a more specific parent state?
- [ ] Does pattern binding reset causal invalidity to `Clean`?
- [ ] Does `for` still use `synthesize_expr` where full expression state is required?
- [ ] Does iteration overwrite the incoming Unknown/Dynamic reason?

---

# 17. Explicit Scope Boundaries for the Next Plans

When implementation exposes one of the following defects, add/retain a RED regression but do not solve it opportunistically here.

## Technical Spec 02 — Canonical Callable Application

Defer:

```text
1 + "hello"
list["wrong"]
obj.field = wrong
list[0] = wrong
ordinary method call argument relation
callable-valued local application
constructor application
iteration protocol canonical call result
```

This plan may provide composition helpers those paths later consume.

## Technical Spec 03 — Generic Proof Integrity

Defer:

```text
generic argument Unknown omission
receiver/callable authority contribution
substitution support vs call validity
expected-result constraints vs value evidence
generic terminal outcome publication
```

Do not “fix” generic calls by calling `compose_required_knowledge` directly inside inference; generic inference needs its own proof-integrity plan.

## Technical Spec 04 — Generic Specialization/Relations

Defer:

```text
class generic constraints specialized under applied receiver
higher-kinded substitution
type-lambda capture/substitution audit
```

---

# 18. Expected Final Diff Shape

The final implementation should be approximately:

```text
CREATE
  phalcom-semantic/src/checker/composition.rs
  phalcom-semantic/tests/semantic/foundations/expression_composition.rs

MODIFY
  phalcom-semantic/src/types/evidence.rs
  phalcom-semantic/src/checker/causal.rs
  phalcom-semantic/src/checker/typed_expr.rs
  phalcom-semantic/src/checker/context.rs
  phalcom-semantic/src/checker/expression.rs
  phalcom-semantic/src/checker/statement.rs
  phalcom-semantic/tests/semantic/foundations/mod.rs
  phalcom-semantic/tests/semantic/support/fixture.rs
```

A substantially broader diff requires justification before merge.

---

# 19. Completion Gate

Do not mark Technical Spec 01 implemented until all of these are true:

- [ ] `compose_required_knowledge` is the only evidence algebra used by required aggregate-member composition.
- [ ] Direct list/set/map/tuple/record members no longer disappear because they lack `TypeId`.
- [ ] Aggregate evidence status is the weakest required known premise.
- [ ] Unknown/Dynamic reasons survive pure aggregate and decomposition operations.
- [ ] Literal expansion branches covered by this spec either project soundly or fail closed.
- [ ] `TypedExpression::invalidate` is used for expression-local source contradictions.
- [ ] Every published `Invalid(C)` expression contains `C` in its causal state.
- [ ] `Ready + non-clean causal invalidity` remains legal and tested.
- [ ] Required invalid-but-known inputs do not automatically suppress parents.
- [ ] Required invalid-and-unavailable inputs do suppress parents without duplicate diagnostics.
- [ ] Pattern decomposition preserves parent epistemic state and causal dependency.
- [ ] `for` analysis preserves iterable Unknown/Dynamic reason and causal state.
- [ ] Existing normal-return summary regressions remain green.
- [ ] Existing flow join and loop widening regressions remain green.
- [ ] No scoped analyze-and-ignore expansion branch remains.
- [ ] No scoped direct `Invalid(cause)` mutation remains without causal synchronization.
- [ ] `cargo fmt --check` passes.
- [ ] `cargo test -p phalcom-semantic` passes.
- [ ] Workspace/CI-equivalent verification has been executed and recorded.
- [ ] Diff contains no canonical-call, generic-proof, advisory, identity, or transaction implementation accidentally pulled into this slice.

---

# 20. Handoff to the Next Implementation Plan

After this plan lands, Technical Spec 02 should start from these completed interfaces:

```rust
compose_required_knowledge(
    inputs,
    origin,
    build_type,
)

propagate_required_dependencies(
    &mut result,
    &operands,
)

typed.invalidate(cause)
typed.debug_assert_coherent()

ctx.publish_expression_analysis(
    expression_id,
    range,
    &typed,
    explanation_id,
)

ctx.sync_expression_outcome(&typed)
causal_invalidity.contains(cause)
```

Technical Spec 02 must route call-like operations through a canonical application result that reuses these product/coherence rules rather than duplicating them.

The semantic-correctness program should not proceed to generic proof-integrity implementation until ordinary call application has one canonical argument/relation path.

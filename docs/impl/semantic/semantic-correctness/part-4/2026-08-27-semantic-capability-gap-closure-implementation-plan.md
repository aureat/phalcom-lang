# Semantic Capability Gap Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the five remaining semantic capability gaps—trusted branch refinement, higher-order closure invocation, formal field lifecycle facts, list/rest destructuring, and incremental callable reuse—and then remove the separate imported-binding identity failure without weakening Phalcom's formal evidence rules.

**Architecture:** Reuse existing semantic infrastructure rather than building parallel mechanisms. Branch refinement must wire the existing predicate/transfer engine into real branch execution; closure `.call()` must route into the post-Technical-03 canonical callable application engine; list/rest patterns must extend existing formal pattern decomposition; fields need a persistent contract plus path-sensitive current state and a lifecycle proof; incremental repair must preserve semantic-product stability through the inferred-return refresh phase rather than bypassing the DB's reuse laws.

**Tech Stack:** Rust, `phalcom-semantic`, `TypeKnowledge`, `EvidenceStatus`, `FlowState`, `FlowPredicate`, `CallableApplicationTarget`, `SemanticWorkspaceSession`, semantic DB query/product fingerprints, `FieldId`, `CallableId`, `SourceScopeIndex`, Rust integration tests.

**Spec:** `docs/superpowers/plans/2026-08-24-semantic-complex-analysis-scenarios.md` (gap-producing scenarios), `docs/impl/semantic/semantic-correctness/part-4/phalcom-semantic-correctness-technical-02-canonical-callable-application-spec.md`, and the Technical Specification 03 — Generic Inference and Proof Integrity implementation currently in progress.

## Repository grounding

This plan was grounded against `aureat/phalcom-lang` `main` at:

```text
1ffa7c1d12f637114eddeabefde72f76926b2a7c
test(semantic): remove redundant branch assertion
```

The exact committed gap probes at this revision are:

```text
RED
phalcom-semantic/tests/semantic/capabilities/flow_branches.rs
  refined_branch_with_abrupt_else_publishes_only_normal_value

GATED
phalcom-semantic/tests/semantic/capabilities/higher_order.rs
  higher_order_block_call_propagates_captured_result

GATED
phalcom-semantic/tests/semantic/capabilities/fields.rs
  field_facts_survive_constructor_and_general_writes

GATED
phalcom-semantic/tests/semantic/capabilities/patterns.rs
  collection_and_destructure_facts_preserve_element_shapes

RED
phalcom-semantic/tests/semantic/incremental/callable_dependencies.rs
  dependency_edit_remove_readd_recomputes_affected_summary_deterministically

SEPARATE EXISTING FAILURE
phalcom-semantic/tests/semantic/integration/imported_resolution.rs
  imported_binding_use_resolves_to_exported_declaration_not_local_import_site
```

The ignored reflective dynamic-pack test is not part of these five gaps. Do not broaden this plan into reflection semantics.

## Global Constraints

- Technical Specification 03 must land first. Do not implement this plan against the pre-Spec-03 generic call path and then forward-port duplicate logic.
- Preserve `TypeKnowledge::{Known, Unknown, Dynamic}` and `EvidenceStatus::{Established, Assumed}` distinctions. No gap may be closed by upgrading an unavailable or assumed premise to established knowledge without a new formal proof.
- Use the canonical callable application path after Technical 03. No new syntax-specific parameter matcher, generic solver, return promotion shortcut, or argument loop may be introduced.
- Keep persistent contracts separate from current flow knowledge. Refinement and field writes change current facts; they do not rewrite source contracts.
- Unknown and Dynamic reasons remain exact whenever a decomposition, call, or field operation lacks proof.
- A semantic predicate may refine a branch only when its semantic identity is trusted. Method spelling alone is not proof.
- `Arc::ptr_eq` is a legitimate acceptance check for a semantically reused callable product. Do not "fix" the incremental test by removing pointer-stability assertions.
- Incremental statistics must describe the final semantic result of the revision, not merely the first body-query pass.
- Keep the separate imported-binding fix isolated from the five capability implementations so its source-identity change can be reviewed independently.
- Do not implement broad record/map/variant pattern decomposition under the list/rest task. Those are adjacent capability work, not required to close this plan.
- Do not redesign static/class-field initialization under the instance-field lifecycle task. The committed field gap is instance-side.
- Each task uses RED → implementation → GREEN and lands as an independently reviewable commit.

---

# File/ownership map

The plan intentionally keeps new responsibilities narrow.

| Area | Existing owner | Planned responsibility |
|---|---|---|
| Branch predicate syntax | `checker/flow/predicate.rs` | Extract candidate predicates and validate canonical type-test identity |
| Branch transfer | `checker/flow/transfer.rs` | Relation-aware positive/negative refinement |
| Structured branch execution | `checker/expression.rs` | Apply true/false predicates before each branch body |
| Callable-value application | `checker/expression.rs`, post-Spec-03 `checker/call.rs` | One helper shared by `f(...)` and `f.call(...)` |
| Pattern decomposition | `checker/composition.rs`, `checker/statement.rs` | Homogeneous List fixed/rest projection |
| Field state | `checker/analysis.rs`, `checker/flow/state.rs` | Persistent field contract + current fact + initialization state |
| Field transfer | `checker/expression.rs`, `checker/context.rs` | Read/write formal field facts |
| Field lifecycle | new `checker/field_lifecycle.rs` | Default/constructor initialization proof |
| Incremental reuse | `session.rs`, `db/*` | Prevent self-reanalysis and preserve final product identity/stats |
| Import identity | `source_index/scope.rs`, `source_index/occurrence.rs` | Dereference import bindings to canonical linked targets |

Because Technical 03 is changing callable application and may alter `call.rs`, `expression.rs`, `context.rs`, `evidence.rs`, and fingerprints, all line numbers must be re-resolved after its merge. Function/type names in this plan are the stable anchors.

---

# Execution order

```text
Task 0  Spec-03 merge/rebase gate
   |
   +--> Task 1  trusted branch refinement
   |
   +--> Task 2  callable-value .call()
   |
   +--> Task 3  list/rest formal decomposition
   |
   +--> Task 4  field state model
             |
             v
          Task 5  lifecycle proof
             |
             v
          Task 6  field read/write publication
   |
   +--> Task 7  inferred-return refresh stability
             |
             v
          Task 8  final incremental statistics
   |
   +--> Task 9  imported canonical identity
             |
             v
          Task 10 full closure gate
```

Tasks 1, 3, and 9 are logically independent after Task 0. Task 2 must use the merged Technical-03 application API. Tasks 4–6 are one ordered field workstream. Tasks 7–8 should be done last because they touch the session/incremental layer and must observe the final product shapes produced by the other workstreams.

---

## Task 0: Rebase onto completed Technical 03 and freeze the post-merge baseline

**Files:**
- Read: merged Technical-03 spec and implementation plan.
- Read: `phalcom-semantic/src/checker/call.rs`
- Read: `phalcom-semantic/src/checker/inference.rs`
- Read: `phalcom-semantic/src/checker/expression.rs`
- Read: `phalcom-semantic/src/checker/context.rs`
- Read: `phalcom-semantic/src/db/fingerprint.rs`
- Test: the six committed scenarios listed in Repository grounding.

**Interfaces:**
- Consumes: the final Technical-03 versions of `CallableApplicationTarget`, `apply_resolved_callable`, generic argument binding, `TypeKnowledge`, inference proof-state publication, and call-result authority capping.
- Produces: a written baseline in the work log naming the exact post-Spec-03 commit and the observed RED/GATED state of each gap.

- [ ] **Step 1: Start from a clean post-Spec-03 revision.**

Run:

```sh
git status --short
git rev-parse HEAD
git log -1 --oneline
```

Expected: no unrelated source edits are mixed into the gap-closure branch.

- [ ] **Step 2: Verify Technical 03 before touching gap code.**

Run the Technical-03 focused tests from its implementation plan, followed by:

```sh
cargo test -p phalcom-semantic --test semantic generics -- --nocapture
```

Expected: Technical-03 generic proof-integrity coverage is GREEN. If it is not, this plan does not begin; fix Technical 03 on its own branch.

- [ ] **Step 3: Probe the five gaps without changing assertions.**

Run:

```sh
cargo test -p phalcom-semantic --test semantic refined_branch_with_abrupt_else_publishes_only_normal_value -- --nocapture
cargo test -p phalcom-semantic --test semantic higher_order_block_call_propagates_captured_result -- --ignored --nocapture
cargo test -p phalcom-semantic --test semantic field_facts_survive_constructor_and_general_writes -- --ignored --nocapture
cargo test -p phalcom-semantic --test semantic collection_and_destructure_facts_preserve_element_shapes -- --ignored --nocapture
cargo test -p phalcom-semantic --test semantic dependency_edit_remove_readd_recomputes_affected_summary_deterministically -- --nocapture
cargo test -p phalcom-semantic --test semantic imported_binding_use_resolves_to_exported_declaration_not_local_import_site -- --nocapture
```

Expected: capture the exact failure shapes. Do not update expected values to match current behavior.

- [ ] **Step 4: Record post-Spec-03 overlap.**

In the implementation work log, explicitly record whether Technical 03 changed any of these anchors:

```text
checker/call.rs:
  CallableApplicationTarget
  apply_resolved_callable
  static_call_shape / ArgumentBindingPlan

checker/expression.rs:
  synthesize_method_call
  synthesize_unqualified_call

checker/context.rs:
  call-status/evidence helpers

types/evidence.rs:
  EvidenceOrigin / TypeKnowledge rules

db/fingerprint.rs:
  callable body product fingerprint
```

Expected: subsequent tasks use the merged APIs, not this document's pre-merge assumptions.

- [ ] **Step 5: Commit only the baseline log if one is maintained.**

Suggested message:

```sh
git commit -m "docs(semantic): record post-spec03 gap baseline"
```

---

## Task 1: Wire trusted branch predicates into real branch execution

The repository already contains `FlowPredicate`, `extract_predicate`, inversion, `apply_predicate`, `FactSet`, and mutation invalidation. The capability gap exists because `extract_predicate` is not wired into the structured branch path.

A second correctness issue must be fixed at the same time: the current positive `IsInstance` transfer can assign the target type unconditionally. A trusted branch transfer may narrow; it must not fabricate a type for an incompatible current fact.

**Files:**
- Modify: `phalcom-semantic/src/checker/flow/predicate.rs`
- Modify: `phalcom-semantic/src/checker/flow/transfer.rs`
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/flow_branches.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/flow_graph.rs`

**Interfaces:**
- Consumes:
  - `extract_predicate(ctx, expr, truth) -> Option<FlowPredicate>`
  - `TypedExpression.callable`
  - `CheckingContext::lookup_binding_info`
  - `TypeHierarchy`
- Produces:
  - `extract_trusted_predicate(...) -> Option<FlowPredicate>`
  - relation-aware `apply_predicate(..., hierarchy)`
  - branch-local refined `FlowState` before `analyze_control_block`.

- [ ] **Step 1: Add a RED safety test proving method spelling alone cannot refine.**

Add beside the composed branch test:

```rust
#[test]
fn overridden_is_method_does_not_gain_builtin_refinement_authority() {
    let f = Fixture::new(
        r#"
class Liar {
  is(_ type) -> Bool { true }
}

class Probe {
  @class
  run(_ value: Liar) {
    if (value.is(Int)) {
      return value
    }
    0
  }
}
"#,
    );

    let run = f.callable("Probe", "run", DispatchSide::Class);
    let liar = f.ty("Liar");
    let branch_value = f.expression_n(run, "value", 1);
    assert_eq!(branch_value.knowledge.ty(), Some(liar));
}
```

Run:

```sh
cargo test -p phalcom-semantic --test semantic overridden_is_method_does_not_gain_builtin_refinement_authority -- --nocapture
```

Expected: RED if syntactic extraction is wired without semantic trust; GREEN before wiring is also acceptable because no refinement is currently applied. Keep it as the future soundness guard.

- [ ] **Step 2: Add a trusted-predicate wrapper.**

Keep `extract_predicate` as syntax extraction. Add a wrapper in `flow/predicate.rs` or `expression.rs` with this contract:

```rust
pub(crate) fn extract_trusted_predicate(
    ctx: &mut CheckingContext<'_>,
    condition: &Expr,
    condition_typed: &TypedExpression,
    truth: bool,
) -> Option<FlowPredicate>;
```

For `FlowPredicate::IsInstance` / `IsNotInstance`, require the analyzed condition to resolve to the canonical core `Object.is(_)` or `Object.is!(_)` callable identity. Build the canonical IDs from:

```rust
DeclarationId::new(ModuleId::core(), "Object".into())
Selector::method("is", vec![SelectorSlot::Positional])
Selector::method("is!", vec![SelectorSlot::Positional])
DispatchSide::Instance
```

If the condition resolved to an override or has no exact callable identity, return `None`.

Other already-modeled compiler predicates such as direct binary/nil comparisons may pass through unchanged because their semantics come from compiler-recognized operators, not an overridable user method.

- [ ] **Step 3: Make positive type refinement relation-aware.**

Change the transfer entry point to accept hierarchy:

```rust
pub fn apply_predicate(
    state: &mut FlowState,
    predicate: &FlowPredicate,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
);
```

Implement positive `IsInstance(binding, target)` as:

```text
current == target
    -> target

target <: current
    -> target

current is Union
    -> keep members compatible with target;
       if at least one member survives, join survivors

otherwise
    -> keep current unchanged
```

Do not mark an unrelated nominal path unreachable in this task; Phalcom does not yet expose a general formal disjointness proof sufficient for that transformation.

For negative `IsNotInstance`:

```text
current is Union
    -> remove members proven to be target/subtypes of target
broad nominal current
    -> keep current unchanged
```

A broad `Object` negative branch therefore remains `Object`; do not invent `Object - Int`.

Use `EvidenceOrigin::Flow` for the new branch fact. Do not alter `BindingState.contract`.

- [ ] **Step 4: Apply true/false predicates before branch bodies.**

In the structured `ifTrue(... ifFalse: ...)` path inside `synthesize_control_method_call`:

```rust
let before = ctx.flow.clone();

ctx.flow = before.clone();
if let Some(predicate) =
    extract_trusted_predicate(ctx, &call.object, &recv_typed, true)
{
    apply_predicate(&mut ctx.flow, &predicate, ctx.store, &ctx.hierarchy);
}
let then_typed = analyze_control_block(...);
let then_flow = ctx.flow.clone();

ctx.flow = before.clone();
if let Some(predicate) =
    extract_trusted_predicate(ctx, &call.object, &recv_typed, false)
{
    apply_predicate(&mut ctx.flow, &predicate, ctx.store, &ctx.hierarchy);
}
let else_typed = analyze_control_block(...);
let else_flow = ctx.flow.clone();
```

Resolve borrow details according to the post-Spec-03 `CheckingContext` shape; do not clone or construct a second hierarchy.

- [ ] **Step 5: Restore the strong assertion in the committed composed test.**

The current main removed the direct branch-expression assertion while retaining the `Int` normal-return expectation. Reinstate:

```rust
f.assert_expression_established(f.expression_n(run, "value", 1), int_ty);
```

This proves the fact is refined at the use site, not only accidentally narrowed by return summarization.

- [ ] **Step 6: Run focused branch suites.**

```sh
cargo test -p phalcom-semantic --test semantic refined_branch_with_abrupt_else_publishes_only_normal_value -- --nocapture
cargo test -p phalcom-semantic --test semantic overridden_is_method_does_not_gain_builtin_refinement_authority -- --nocapture
cargo test -p phalcom-semantic --test semantic flow_branches -- --nocapture
cargo test -p phalcom-semantic --test semantic flow_graph -- --nocapture
```

Expected: trusted `Object.is(Int)` narrows the positive branch to established `Int`; abrupt false path contributes no normal return; user overrides do not receive refinement authority.

- [ ] **Step 7: Commit.**

```sh
git add phalcom-semantic/src/checker/flow/predicate.rs \
        phalcom-semantic/src/checker/flow/transfer.rs \
        phalcom-semantic/src/checker/expression.rs \
        phalcom-semantic/tests/semantic/capabilities/flow_branches.rs \
        phalcom-semantic/tests/semantic/foundations/flow_graph.rs
git commit -m "feat(semantic): apply trusted branch refinements"
```

---

## Task 2: Route structural callable `.call()` through canonical application

Direct lexical invocation already recognizes `TypeData::Callable`; `increment.call()` currently falls through nominal dispatch. The fix is to unify the two syntactic forms at the callable-value target layer.

**Files:**
- Modify: post-Spec-03 `phalcom-semantic/src/checker/call.rs`
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/higher_order.rs`
- Test: post-Spec-03 call/application foundation tests.

**Interfaces:**
- Consumes:
  - `TypeData::Callable`
  - `CallableApplicationTarget::callable_value`
  - post-Spec-03 canonical `apply_resolved_callable`
  - post-Spec-03 `ArgumentBindingPlan`
- Produces:

```rust
pub(crate) fn callable_value_target(
    store: &TypeStore,
    callable_ty: TypeId,
    authority: EvidenceStatus,
) -> Option<CallableApplicationTarget>;
```

Name may be adjusted to match Technical 03, but there must be exactly one implementation shared by direct and `.call()` syntax.

- [ ] **Step 1: Add an equivalence RED test.**

Extend `higher_order.rs`:

```rust
#[test]
fn direct_and_explicit_call_on_same_block_publish_same_result() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ value: Int) {
    const direct = || { value + 1 }
    const explicit = || { value + 1 }
    let a = direct()
    let b = explicit.call()
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_established(run, "a", int_ty);
    f.assert_binding_established(run, "b", int_ty);
}
```

Expected before implementation: `a` succeeds while `b` is `Unknown(DynamicMessageSend)` or otherwise unresolved.

- [ ] **Step 2: Extract callable-type target construction from `synthesize_unqualified_call`.**

Move the existing structural signature construction into one helper. It must copy from `CallableType`:

```rust
CallableParameterType {
    label,
    ty,
    rest,
}
```

into canonical `CallableParameter` values and a `call` selector, while preserving receiver authority:

```rust
let signature = CallableSignature::new(
    Selector::method("call", slots)?,
    parameters,
    TypeKnowledge::assumed(callable.return_type, EvidenceOrigin::CallableSignature),
);

CallableApplicationTarget::callable_value(signature, authority)
```

Do not perform argument matching in this helper. Technical 03 owns binding and proof participation.

- [ ] **Step 3: Use the helper from direct lexical invocation.**

Replace the hand-built `TypeData::Callable` block in `synthesize_unqualified_call` with the shared helper and `apply_resolved_callable`.

The direct `f(...)` behavior must remain unchanged.

- [ ] **Step 4: Intercept `.call()` on a structural callable before nominal dispatch.**

In `synthesize_method_call`, after receiver analysis and before `resolve_dispatch_target`:

```rust
if call.method == "call" {
    if let Some(receiver_ty) = recv_typed.knowledge.ty() {
        if let Some(target) = callable_value_target(
            ctx.store,
            receiver_ty,
            recv_typed.knowledge.status().unwrap_or(EvidenceStatus::Assumed),
        ) {
            return apply_resolved_callable(
                ctx,
                &target,
                &premise,
                &arguments,
                expected,
                call.range,
            )
            .into();
        }
    }
}
```

Only structural `TypeData::Callable` values take this path. A nominal object with a user-defined method named `call` must continue through ordinary dispatch.

- [ ] **Step 5: Remove the ignore marker from the committed higher-order probe.**

Change:

```rust
#[ignore = "GATED: source-level closure invocation and callable-result publication are not formal yet"]
```

to an ordinary `#[test]`.

Do not weaken its expected `Int` normal return.

- [ ] **Step 6: Add a non-callable/nominal guard.**

Add a test proving a nominal class's `call()` method still resolves nominally:

```phalcom
class Fun {
  call() -> String { "nominal" }
}
```

The structural shortcut must not capture `Fun.call()`.

- [ ] **Step 7: Run focused tests.**

```sh
cargo test -p phalcom-semantic --test semantic higher_order -- --nocapture
cargo test -p phalcom-semantic --test semantic call -- --nocapture
```

Expected: direct invocation and `.call()` share return knowledge, status, argument diagnostics, and post-Spec-03 proof authority.

- [ ] **Step 8: Commit.**

```sh
git add phalcom-semantic/src/checker/call.rs \
        phalcom-semantic/src/checker/expression.rs \
        phalcom-semantic/tests/semantic/capabilities/higher_order.rs
git commit -m "feat(semantic): unify callable value invocation"
```

---

## Task 3: Implement formal List/rest pattern decomposition

Source indexing already creates list and rest bindings. Formal binding currently supports only `Name` and `Tuple`, so this task extends the checker without changing parser or source identity.

**Files:**
- Modify: `phalcom-semantic/src/checker/composition.rs`
- Modify: `phalcom-semantic/src/checker/statement.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/patterns.rs`
- Test: unit tests in `checker/composition.rs`

**Interfaces:**
- Consumes:
  - `project_applied_argument`
  - `TypeKnowledge::derive_known_type`
  - canonical `List` type constructor
- Produces:

```rust
pub(crate) fn decompose_list_element(
    store: &TypeStore,
    parent: &TypeKnowledge,
    list_origin: TypeId,
) -> TypeKnowledge;

pub(crate) fn decompose_list_rest(
    store: &TypeStore,
    parent: &TypeKnowledge,
    list_origin: TypeId,
) -> TypeKnowledge;
```

- [ ] **Step 1: Add unit tests for epistemic preservation.**

For a known `List<Int>`:

```text
element -> same authority as parent, type Int, origin PatternDecomposition
rest    -> same authority as parent, type List<Int>, origin PatternDecomposition
```

For `Unknown(UnresolvedName("x"))`, both projections retain that exact unknown reason.

For `Dynamic(ExplicitEscape)`, both remain that exact dynamic reason.

- [ ] **Step 2: Implement the two List decomposition helpers.**

`decompose_list_element` may delegate to:

```rust
project_applied_argument(store, parent, list_origin, 0)
```

`decompose_list_rest` must first verify that the parent is an application of the expected List origin, then derive the same canonical list type:

```rust
parent.derive_known_type(parent_ty, EvidenceOrigin::PatternDecomposition)
```

Do not manufacture a list type from an Unknown/Dynamic parent.

- [ ] **Step 3: Add `Pattern::List` to declaration-pattern binding.**

In `bind_declaration_pattern`:

```rust
Pattern::List { elements, rest, .. } => {
    let Some(list_decl) = ctx.resolve_type_name("List") else {
        // Bind leaves using the parent's exact unavailable knowledge.
        ...
    };
    let list_origin = ctx.nominal_type_of(&list_decl);
    let element_fact = ValueSemanticFact::new(
        decompose_list_element(ctx.store, &fact.knowledge, list_origin)
    );

    for element in elements {
        bind_declaration_pattern(
            ctx,
            element,
            element_fact.clone(),
            None,
            causal_invalidity,
            mutable,
            range,
        );
    }

    if let Some(rest) = rest {
        let rest_fact = ValueSemanticFact::new(
            decompose_list_rest(ctx.store, &fact.knowledge, list_origin)
        );
        bind_declaration_pattern(
            ctx,
            rest,
            rest_fact,
            None,
            causal_invalidity,
            mutable,
            range,
        );
    }
}
```

Use the post-Spec-03/current `ValueSemanticFact` API exactly; preserve causal invalidity.

- [ ] **Step 4: Add the same List case to runtime-pattern binding.**

`bind_pattern` is used by `if let`, `while let`, and `for`. It must project the same List facts instead of silently dropping rest bindings.

Do not add Record/Map/Variant implementation in this task.

- [ ] **Step 5: Remove the ignore marker from the committed composed pattern test.**

Keep these requirements:

```text
head == Established(Int)
tail is not Dynamic
pair preserves [Int, tail_type]
record remains structural
```

- [ ] **Step 6: Add an Unknown propagation regression.**

Use an unavailable list source and prove both `head` and `tail` preserve the same Unknown reason instead of disappearing or becoming Dynamic.

- [ ] **Step 7: Run focused tests.**

```sh
cargo test -p phalcom-semantic --test semantic collection_and_destructure_facts_preserve_element_shapes -- --nocapture
cargo test -p phalcom-semantic --test semantic patterns -- --nocapture
cargo test -p phalcom-semantic checker::composition --lib -- --nocapture
```

- [ ] **Step 8: Commit.**

```sh
git add phalcom-semantic/src/checker/composition.rs \
        phalcom-semantic/src/checker/statement.rs \
        phalcom-semantic/tests/semantic/capabilities/patterns.rs
git commit -m "feat(semantic): decompose list rest patterns formally"
```

---

## Task 4: Add a formal field-flow state domain

A field declaration's surface type is a persistent contract. It must not double as the current field fact. Introduce a separate flow state keyed by canonical `FieldId`.

**Files:**
- Modify: `phalcom-semantic/src/checker/analysis.rs`
- Modify: `phalcom-semantic/src/checker/flow/state.rs`
- Modify: `phalcom-semantic/src/types/evidence.rs`
- Test: new low-level field-flow tests in `checker/flow/state.rs` or a focused foundations file.

**Interfaces:**
- Produces:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldInitialization {
    Uninitialized,
    MaybeInitialized,
    DefinitelyInitialized,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldState {
    pub field: FieldId,
    pub contract: TypeKnowledge,
    pub current: TypeKnowledge,
    pub initialization: FieldInitialization,
    pub version: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowFieldSummary {
    pub contract: TypeKnowledge,
    pub current: TypeKnowledge,
    pub initialization: FieldInitialization,
}
```

Extend:

```rust
FlowState {
    bindings: ...,
    fields: BTreeMap<FieldId, FieldState>,
    ...
}

FlowStateSummary {
    bindings: ...,
    fields: BTreeMap<FieldId, FlowFieldSummary>,
    ...
}
```

Add a formal origin:

```rust
EvidenceOrigin::FieldLifecycle
```

- [ ] **Step 1: Write state tests before changing production code.**

Tests must prove:

1. a seeded field preserves contract and current knowledge separately;
2. `write_field` changes current, not contract;
3. branch join uses `join_type_knowledge` for current field facts;
4. `DefinitelyInitialized + DefinitelyInitialized -> DefinitelyInitialized`;
5. `DefinitelyInitialized + Uninitialized -> MaybeInitialized`;
6. unreachable incoming paths do not weaken initialization.

- [ ] **Step 2: Implement field state types.**

Keep field contract as `TypeKnowledge`, not only `TypeId`, because an unannotated/unknown field contract is epistemically meaningful.

- [ ] **Step 3: Extend `FlowState` APIs.**

Add:

```rust
pub fn seed_field(&mut self, state: FieldState);
pub fn get_field(&self, field: &FieldId) -> Option<&FieldState>;
pub fn get_field_current(&self, field: &FieldId) -> Option<&TypeKnowledge>;
pub fn write_field(
    &mut self,
    field: &FieldId,
    current: TypeKnowledge,
    initialization: FieldInitialization,
);
```

A write increments `version`.

- [ ] **Step 4: Extend flow join.**

For every field present in all reachable states:

```text
contract:
    must be identical;
    divergence is an analyzer invariant failure

current:
    join_type_knowledge across reachable states

initialization:
    Definitely iff all reachable states Definitely
    Uninitialized iff all reachable states Uninitialized
    otherwise Maybe
```

Add:

```rust
FlowInvariantFailure::DivergentFieldContract { ... }
```

and route it through the existing internal flow-invariant incident path.

- [ ] **Step 5: Extend flow summaries/fingerprints.**

Any callable product fingerprint that includes entry/exit flow summaries must include field state deterministically. Sort by `FieldId` through the existing `BTreeMap`.

This is required so field-semantic changes invalidate dependents correctly.

- [ ] **Step 6: Run unit/foundation tests.**

```sh
cargo test -p phalcom-semantic checker::flow --lib -- --nocapture
cargo test -p phalcom-semantic --test semantic flow -- --nocapture
```

- [ ] **Step 7: Commit.**

```sh
git add phalcom-semantic/src/checker/analysis.rs \
        phalcom-semantic/src/checker/flow/state.rs \
        phalcom-semantic/src/types/evidence.rs \
        phalcom-semantic/src/db/fingerprint.rs
git commit -m "feat(semantic): add formal field flow state"
```

---

## Task 5: Prove instance-field initialization and publish lifecycle read facts

This task establishes a class invariant without rewriting source annotations. A lifecycle proof is new formal evidence:

```text
known field contract
+
proven initialization before any normally constructed instance escapes
+
all field assignments remain checked against the contract
=
Established(contract_type, FieldLifecycle) read seed
```

The declaration annotation remains Assumed/declared metadata; the established read fact is a separate proof product.

**Files:**
- Create: `phalcom-semantic/src/checker/field_lifecycle.rs`
- Modify: `phalcom-semantic/src/checker/mod.rs`
- Modify: `phalcom-semantic/src/checker/declaration.rs`
- Modify: `phalcom-semantic/src/checker/body.rs`
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/session.rs`
- Test: new field-lifecycle foundation tests
- Test: `phalcom-semantic/tests/semantic/capabilities/fields.rs`

**Interfaces:**
- Produces:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldLifecycleFact {
    pub field: FieldId,
    pub contract: TypeKnowledge,
    pub read_knowledge: TypeKnowledge,
    pub initialization: FieldInitialization,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FieldLifecycleTable {
    pub fields: BTreeMap<FieldId, FieldLifecycleFact>,
}
```

Helper contracts:

```rust
pub(crate) fn default_field_seeds(...) -> FieldLifecycleTable;

pub(crate) fn finalize_instance_field_lifecycle(
    defaults: &FieldLifecycleTable,
    constructors: impl IntoIterator<Item = &CallableAnalysis>,
) -> FieldLifecycleTable;
```

- [ ] **Step 1: Add default-initializer lifecycle tests.**

Use:

```phalcom
class Counter {
  _value: Int = 0
  read() -> Int { _value }
}
```

Expected lifecycle fact:

```text
contract        = Assumed/declared Int
initialization  = DefinitelyInitialized
read_knowledge  = Established(Int, FieldLifecycle)
```

The proof comes from the checked initializer plus the field contract; do not mutate the declaration surface's epistemic origin.

- [ ] **Step 2: Add constructor-only definite-initialization tests.**

Positive:

```phalcom
class Cell {
  _value: Int
  @constructor new(_ value: Int) {
    _value = value
  }
  read() -> Int { _value }
}
```

Negative:

```phalcom
class Cell {
  _value: Int
  @constructor new(_ flag: Bool, _ value: Int) {
    if flag { _value = value }
  }
  read() -> Int { _value }
}
```

The first is `DefinitelyInitialized`; the second is `MaybeInitialized` and must not publish established lifecycle read knowledge.

- [ ] **Step 3: Produce default field seeds without executing methods.**

Refactor field initializer checking so it can return formal seed facts while continuing to own initializer diagnostics.

For a known contract and a known initializer relation-proven assignable to it:

```rust
FieldState {
    contract: declared.clone(),
    current: TypeKnowledge::established(
        contract_ty,
        EvidenceOrigin::FieldLifecycle,
    ),
    initialization: FieldInitialization::DefinitelyInitialized,
    ...
}
```

For no initializer:

```rust
current = TypeKnowledge::Unknown(UnknownReason::MissingInitializer)
initialization = Uninitialized
```

Do not use the annotation itself as the current fact.

- [ ] **Step 4: Seed constructor body flow with defaults.**

When analyzing an instance constructor, populate `ctx.flow.fields` before statements execute.

Field assignment within a constructor must therefore update the canonical `FieldId` state and appear in normal exit summaries.

- [ ] **Step 5: Finalize lifecycle across constructor normal exits.**

Rules:

```text
No source constructors:
    default seed determines initialization.

One or more constructors:
    for each constructor that has a normal exit,
    field must be DefinitelyInitialized on every normal exit.

Constructor with no normal exit:
    contributes no constructed instance and does not weaken the proof.

Any normal exit with Uninitialized/MaybeInitialized:
    lifecycle is not established.

Known contract + DefinitelyInitialized:
    read_knowledge = Established(contract_ty, FieldLifecycle)

Unknown contract:
    lifecycle cannot invent a type; preserve Unknown.
```

- [ ] **Step 6: Keep the lifecycle table compiler-owned and immutable for ordinary method entry.**

Do not write the established read fact back into `DeclarationSurface.fields`.

Pass the table into ordinary instance body analysis as a seed product. The persistent declaration surface continues to answer "what contract was declared"; the lifecycle table answers "what field fact has the analyzer proved at method entry."

- [ ] **Step 7: Add diagnostics only where semantics already require them.**

This task is not a new "uninitialized field" diagnostic project. For `MaybeInitialized`, fail closed epistemically and preserve existing diagnostics. Do not invent a user-facing error code solely to make the test pass.

- [ ] **Step 8: Run focused lifecycle tests.**

```sh
cargo test -p phalcom-semantic --test semantic fields -- --nocapture
cargo test -p phalcom-semantic field_lifecycle --lib -- --nocapture
```

The existing composed field test may still be ignored until Task 6 routes all reads/writes through the new state.

- [ ] **Step 9: Commit.**

```sh
git add phalcom-semantic/src/checker/field_lifecycle.rs \
        phalcom-semantic/src/checker/mod.rs \
        phalcom-semantic/src/checker/declaration.rs \
        phalcom-semantic/src/checker/body.rs \
        phalcom-semantic/src/checker/context.rs \
        phalcom-semantic/src/session.rs
git commit -m "feat(semantic): prove instance field lifecycle facts"
```

---

## Task 6: Route formal field reads and writes through `FieldState`

This task actually closes the field capability. Surface lookup remains the contract source; `FlowState.fields` becomes the current formal read/write source.

**Files:**
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Modify: `phalcom-semantic/src/checker/body.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/fields.rs`
- Test: branch/flow field regressions added here.

**Interfaces:**
- Consumes: `FieldLifecycleTable`, `FlowState::get_field_current`, `FlowState::write_field`.
- Produces:

```rust
pub(crate) fn resolve_current_field(
    &self,
    owner: &DeclarationId,
    side: DispatchSide,
    name: &str,
) -> Option<(FieldId, TypeKnowledge)>;
```

and a field-write helper that owns contract reconciliation.

- [ ] **Step 1: Change `Expr::Field` read synthesis.**

Current behavior reads the surface contract directly. New behavior:

```text
resolve canonical FieldId
    |
    +-- current FlowState field fact exists
    |      -> return current fact
    |
    +-- no current fact
           -> fall back to declaration contract only as Assumed/Unknown
```

The fallback is necessary for unsupported static-field/lifecycle cases.

- [ ] **Step 2: Change field assignment transfer.**

Current assignment validates the RHS and returns `Unit` without updating field flow. New behavior:

1. resolve `FieldId`;
2. read persistent contract from declaration surface;
3. analyze RHS using the contract as expected type;
4. apply assignability/reconciliation;
5. write RHS-derived current knowledge into `FlowState.fields`;
6. set initialization to `DefinitelyInitialized` on that path;
7. return `Unit` with the relation status/causal invalidity.

Do not write the contract itself as current knowledge.

- [ ] **Step 3: Preserve authority after general writes.**

Examples:

```text
Established Int RHS -> field current Established Int
Assumed Int RHS     -> field current Assumed Int
Unknown             -> field current exact Unknown reason
Dynamic             -> field current exact Dynamic reason
```

A later read in the same method must observe that current fact.

At ordinary method entry, the lifecycle seed may establish the field contract as an invariant. That is a distinct proof from any later local write.

- [ ] **Step 4: Add branch field-write regression.**

Use:

```phalcom
class Counter {
  _value: Number = 0

  choose(_ flag: Bool) {
    if flag { _value = 1 } else { _value = 2.5 }
    _value
  }
}
```

Expected current read after join: established `Int | Float` (or canonical joined subtype representation), while the persistent field contract remains `Number`.

This proves contract/current separation for fields mirrors bindings.

- [ ] **Step 5: Remove the ignore marker from `field_facts_survive_constructor_and_general_writes`.**

Keep all existing established-read assertions.

- [ ] **Step 6: Run field plus branch regression suites.**

```sh
cargo test -p phalcom-semantic --test semantic field_facts_survive_constructor_and_general_writes -- --nocapture
cargo test -p phalcom-semantic --test semantic fields -- --nocapture
cargo test -p phalcom-semantic --test semantic flow_branches -- --nocapture
```

- [ ] **Step 7: Commit.**

```sh
git add phalcom-semantic/src/checker/context.rs \
        phalcom-semantic/src/checker/expression.rs \
        phalcom-semantic/src/checker/body.rs \
        phalcom-semantic/tests/semantic/capabilities/fields.rs
git commit -m "feat(semantic): publish formal field read write facts"
```

---

## Task 7: Stop inferred-return refresh from replacing semantically reusable callables

The DB already supports product-stability reuse. The current inferred-return refresh violates the intended result identity by rechecking a callable merely because its own newly inferred return was published.

The immediate law is:

```text
publishing callable C's inferred return
does not require reanalyzing C
unless C actually consumes that return through a dependency cycle.
```

Only consumers of a changed inferred return need reanalysis.

**Files:**
- Modify: `phalcom-semantic/src/session.rs`
- Test: `phalcom-semantic/tests/semantic/incremental/callable_dependencies.rs`
- Test: existing callable-publication chain tests.

**Interfaces:**
- Consumes: `CallableAnalysis.dependencies`, `dependency_fingerprint`, `Arc<CallableAnalysis>`.
- Produces: refresh logic that rechecks dependency consumers only and preserves prior `Arc` when the semantic product is unchanged.

- [ ] **Step 1: Keep the committed incremental scenario RED.**

Do not remove:

```rust
assert!(Arc::ptr_eq(&client_v1, &client_v2));
```

Run:

```sh
cargo test -p phalcom-semantic --test semantic dependency_edit_remove_readd_recomputes_affected_summary_deterministically -- --nocapture
```

Record the exact failing revision.

- [ ] **Step 2: Remove unconditional self-affecting refresh.**

Current conceptual predicate:

```rust
changed_callables.contains(&callable)
    || analysis.dependencies.iter().any(|dep| changed_callables.contains(dep))
```

Change to dependency consumption:

```rust
analysis
    .dependencies
    .iter()
    .any(|dependency| changed_callables.contains(dependency))
```

If recursion explicitly records self as a dependency, recursive callables still re-enter the fixed point. Do not special-case the callable's own ID.

- [ ] **Step 3: Preserve prior `Arc` on semantically stable refresh.**

When a dependent must be reanalyzed outside the ordinary body query:

```rust
let refreshed = analyze_callable_body(...);
let refreshed_fp = callable_body_product_fingerprint(&refreshed);

let replacement = match callable_analyses.get(&callable) {
    Some(previous) if previous.dependency_fingerprint == refreshed_fp => {
        previous.clone()
    }
    _ => Arc::new(refreshed),
};
```

Before comparing, set the refreshed analysis's `dependency_fingerprint` exactly as the body query path does.

This makes the cache-bypassing refresh honor the same semantic-stability law as `SemanticDb`.

- [ ] **Step 4: Add a call-chain regression.**

Use unannotated:

```text
Leaf.value -> Int
Middle.value -> Leaf.value
Top.value -> Middle.value
```

Change only the leaf body from `1` to `2` while keeping the same inferred type. Expected:

```text
Leaf body recomputed
Middle/Top final semantic products pointer-stable when their products are unchanged
```

If the actual body expression fact contains literal value identity and therefore changes the product fingerprint, assert the exact intended product boundary rather than forcing pointer equality. The callable return type alone is not sufficient reason for reuse; the semantic product fingerprint is authoritative.

- [ ] **Step 5: Run publication and incremental suites.**

```sh
cargo test -p phalcom-semantic --test semantic callable_publication -- --nocapture
cargo test -p phalcom-semantic --test semantic callable_dependencies -- --nocapture
```

- [ ] **Step 6: Commit.**

```sh
git add phalcom-semantic/src/session.rs \
        phalcom-semantic/tests/semantic/incremental/callable_dependencies.rs
git commit -m "fix(semantic): preserve callable identity across inferred returns"
```

---

## Task 8: Make incremental recompute/reuse statistics describe final revision products

The first body-query pass can report "reused" and a later inferred-return refresh can reanalyze that same callable. Statistics must classify the callable by its final revision behavior.

**Files:**
- Modify: `phalcom-semantic/src/session.rs`
- Modify: incremental stats type if it is defined outside `session.rs`
- Test: `phalcom-semantic/tests/semantic/incremental/callable_dependencies.rs`

**Interfaces:**
- Produces a per-callable final classification:

```rust
enum CallableRevisionDisposition {
    Reused,
    Recomputed,
}
```

Use a map/set internally; the public stats may remain numeric.

- [ ] **Step 1: Add explicit stats assertions around refresh-sensitive chains.**

The committed scenario already requires:

```text
v2 body-only Api edit:
    recomputed = 1
    reused     = 1

v3 signature edit:
    recomputed = 2
```

Add one inferred-return chain where the first query pass reuses a consumer but refresh later genuinely reanalyzes it. The final stats must classify that consumer as recomputed, not both reused and recomputed.

- [ ] **Step 2: Stop incrementing counters eagerly as the source of truth.**

Track:

```rust
BTreeMap<CallableId, CallableRevisionDisposition>
```

During the normal body-query pass:

```text
cache hit  -> Reused unless later upgraded
cache miss -> Recomputed
```

During inferred-return refresh:

```text
actual reanalysis -> Recomputed
```

`Recomputed` is monotone and overrides `Reused`.

- [ ] **Step 3: Derive public counters after refresh convergence.**

At the end of the update:

```rust
stats.callables_recomputed =
    dispositions.values().filter(|v| **v == Recomputed).count();

stats.callables_reused =
    dispositions.values().filter(|v| **v == Reused).count();
```

Build `recomputed_keys` from the same final set. Do not count repeated fixed-point passes twice.

- [ ] **Step 4: Verify removal/re-addition does not retain stale dispositions.**

Each workspace revision starts a fresh disposition map. Removed callables must not appear in current counters or current snapshot products.

- [ ] **Step 5: Run the full incremental category.**

```sh
cargo test -p phalcom-semantic --test semantic incremental -- --nocapture
```

Then the exact scenario:

```sh
cargo test -p phalcom-semantic --test semantic dependency_edit_remove_readd_recomputes_affected_summary_deterministically -- --nocapture
```

Expected: deterministic counters and pointer identity.

- [ ] **Step 6: Commit.**

```sh
git add phalcom-semantic/src/session.rs \
        phalcom-semantic/tests/semantic/incremental/callable_dependencies.rs
git commit -m "fix(semantic): report final callable reuse disposition"
```

---

## Task 9: Fix the separate imported-binding canonical identity failure

This is not one of the five capability gaps, but it must be closed before claiming the full semantic target is GREEN.

The source builder already attaches a canonical linked target to an import declaration site. `SourceScopeIndex::resolve_name` currently returns the lexical `Binding` before dereferencing that import target.

**Files:**
- Modify: `phalcom-semantic/src/source_index/scope.rs`
- Review: `phalcom-semantic/src/source_index/builder.rs`
- Review: `phalcom-semantic/src/source_index/occurrence.rs`
- Test: `phalcom-semantic/tests/semantic/integration/imported_resolution.rs`

**Interfaces:**
- Consumes: `SourceBindingKind::Import`, `SourceScopeIndex.targets`.
- Produces: imported use-site name resolution to the external canonical `SemanticTargetId`.

- [ ] **Step 1: Keep the existing failure unchanged.**

Run:

```sh
cargo test -p phalcom-semantic --test semantic imported_binding_use_resolves_to_exported_declaration_not_local_import_site -- --nocapture
```

Expected before fix:

```text
actual:   SemanticTargetId::Binding(local_import_site)
expected: SemanticTargetId::Declaration(external_export)
```

- [ ] **Step 2: Dereference import bindings during name resolution.**

In `SourceScopeIndex::resolve_name`, after finding a visible lexical binding:

```rust
if binding.kind == SourceBindingKind::Import {
    if let Some(target) = self.targets.get(site) {
        return SourceNameResolution::Target(target.clone());
    }
}
return SourceNameResolution::Binding(site.clone());
```

This preserves ordinary local binding identity while treating an imported alias as a lexical name that denotes an external canonical semantic target.

- [ ] **Step 3: Add an alias regression.**

Test:

```phalcom
from .shapes import Circle as C
C
```

The declaration spelling `C` remains the local import declaration site for source navigation metadata, while the read occurrence resolves to the external canonical declaration target.

- [ ] **Step 4: Verify module imports remain modules.**

A module import that already has `SemanticTargetId::Module` must continue resolving to that canonical module target.

- [ ] **Step 5: Run source-index/integration tests.**

```sh
cargo test -p phalcom-semantic --test semantic imported_resolution -- --nocapture
cargo test -p phalcom-semantic --test semantic source_index -- --nocapture
```

- [ ] **Step 6: Commit separately.**

```sh
git add phalcom-semantic/src/source_index/scope.rs \
        phalcom-semantic/tests/semantic/integration/imported_resolution.rs
git commit -m "fix(semantic): preserve canonical imported target identity"
```

---

## Task 10: Close the capability ledger and run the full verification matrix

**Files:**
- Modify: only the existing semantic work log / capability ledger used by the project.
- Review: all files touched by Tasks 1–9.
- Test: complete semantic target plus Technical-03 focused tests.

**Interfaces:**
- Produces: no ignored markers for the three GATED tests in this plan; both RED scenarios GREEN; separate imported identity failure GREEN; no new internal incidents.

- [ ] **Step 1: Confirm the three gap ignores are gone.**

Search:

```sh
rg -n 'higher_order_block_call_propagates_captured_result|field_facts_survive_constructor_and_general_writes|collection_and_destructure_facts_preserve_element_shapes' \
  phalcom-semantic/tests/semantic
```

Inspect each definition and confirm there is no `#[ignore]`.

Do not remove unrelated intentional ignores such as reflection simply to improve counts.

- [ ] **Step 2: Run the six exact closure scenarios.**

```sh
cargo test -p phalcom-semantic --test semantic refined_branch_with_abrupt_else_publishes_only_normal_value -- --nocapture
cargo test -p phalcom-semantic --test semantic higher_order_block_call_propagates_captured_result -- --nocapture
cargo test -p phalcom-semantic --test semantic field_facts_survive_constructor_and_general_writes -- --nocapture
cargo test -p phalcom-semantic --test semantic collection_and_destructure_facts_preserve_element_shapes -- --nocapture
cargo test -p phalcom-semantic --test semantic dependency_edit_remove_readd_recomputes_affected_summary_deterministically -- --nocapture
cargo test -p phalcom-semantic --test semantic imported_binding_use_resolves_to_exported_declaration_not_local_import_site -- --nocapture
```

Expected: six PASS.

- [ ] **Step 3: Re-run Technical 03.**

Run the Technical-03 focused command matrix from its implementation plan.

Expected: all proof-integrity tests still GREEN, especially:

```text
Unknown generic premise preservation
Assumed generic support
expected-result context does not fabricate value evidence
terminal generic failure behavior
directed variable subtype/order stability
```

The higher-order callable task must not have reintroduced a pre-Spec-03 application shortcut.

- [ ] **Step 4: Run capability category.**

```sh
cargo test -p phalcom-semantic --test semantic capabilities -- --nocapture
```

Expected: the three formerly gated gap tests now execute normally. Any remaining ignored test must be documented as outside this plan.

- [ ] **Step 5: Run incremental category.**

```sh
cargo test -p phalcom-semantic --test semantic incremental -- --nocapture
```

Expected: pointer-stability and final recompute/reuse counts are deterministic.

- [ ] **Step 6: Run the full semantic target.**

```sh
cargo test -p phalcom-semantic --test semantic -- --nocapture
```

Expected: zero failures. Do not describe the plan as complete while the imported-resolution test or any newly introduced failure remains.

- [ ] **Step 7: Run formatting and compiler checks used by the repository.**

At minimum:

```sh
cargo fmt --check
cargo check -p phalcom-semantic
```

Run the repository's established clippy/workspace commands only on a toolchain where the pinned components are actually available; do not misclassify hosted-runner/toolchain infrastructure failures as semantic regressions.

- [ ] **Step 8: Check for internal incidents.**

Capability, field, branch, and incremental fixtures should report no unexpected `InternalSemanticIncidentKind` values.

No invariant failure may be converted into a user diagnostic to make the suite green.

- [ ] **Step 9: Update the semantic work log.**

Record:

```text
Technical 03 base SHA
gap-closure final SHA
focused capability result
incremental result
full semantic result
remaining ignored tests and why each is outside this plan
```

Do not copy stale historical counts.

- [ ] **Step 10: Commit closure documentation.**

```sh
git add docs
git commit -m "docs(semantic): close remaining capability gaps"
```

---

# Acceptance laws

The implementation is complete only when all of these laws hold.

## R1 — Trusted refinement

```text
Object.is(Int) resolved to canonical core predicate
+ true branch
=> current binding fact may refine to Int

user-defined/overridden `is`
=> no built-in refinement authority
```

The source contract remains unchanged.

## R2 — Abrupt reachability

A `throw`/`return` branch does not contribute a value to a continuing/normal join. Refinement must be visible before abrupt exit summarization.

## H1 — One callable-value application engine

```text
f(...)
f.call(...)
```

for the same structural `Callable` value use the same post-Spec-03 application semantics, parameter binding, proof-state participation, and result-authority rules.

## P1 — Rest decomposition preserves epistemic authority

```text
Known(List<T>)   -> Known(T), Known(List<T>)
Unknown(reason)  -> Unknown(reason), Unknown(reason)
Dynamic(reason)  -> Dynamic(reason), Dynamic(reason)
```

No rest binding silently disappears.

## F1 — Field contract/current separation

```text
field annotation -> persistent contract
lifecycle proof  -> method-entry current fact
field assignment -> path-local current fact
branch join      -> joined current fact
```

No field write rewrites the source contract.

## F2 — Lifecycle proof is explicit

A field read becomes `Established(contract_ty, FieldLifecycle)` only when initialization has been formally proved. An annotation by itself remains an assumption/declaration fact.

## F3 — Constructor coverage

Every normally exiting constructor path must initialize a constructor-only field before lifecycle publication may establish it. Abrupt-only constructor paths do not create instances and therefore do not weaken normal-construction proof.

## I1 — Semantic stability preserves identity

If a callable's direct input and dependency product fingerprints are unchanged and no refresh consumer relation requires reanalysis, the existing `Arc<CallableAnalysis>` remains reusable.

## I2 — Publishing your own inferred return is not self-invalidation

A callable is not rechecked solely because its body-derived return was written into the current dispatch view. Only actual consumers of that changed return participate in the next fixed-point step.

## I3 — Stats describe final products

A callable reanalyzed during refresh is `Recomputed` for the revision even if the first DB pass hit cache. A callable is never counted as both reused and recomputed.

## M1 — Imported use identity is external canonical identity

The lexical import declaration remains source metadata, while a read through that import resolves to the exported declaration/module target.

---

# Recommended commit sequence

```text
docs(semantic): record post-spec03 gap baseline
feat(semantic): apply trusted branch refinements
feat(semantic): unify callable value invocation
feat(semantic): decompose list rest patterns formally
feat(semantic): add formal field flow state
feat(semantic): prove instance field lifecycle facts
feat(semantic): publish formal field read write facts
fix(semantic): preserve callable identity across inferred returns
fix(semantic): report final callable reuse disposition
fix(semantic): preserve canonical imported target identity
docs(semantic): close remaining capability gaps
```

Keep field-state model, lifecycle proof, and field read/write routing in separate commits. They have distinct correctness claims and should be reviewable independently.

---

# Reviewer checkpoints

After Task 1, review one question only:

```text
Can any user-defined method named `is` cause formal type refinement?
```

Required answer: no.

After Task 2:

```text
Do `f(...)` and `f.call(...)` reach the same canonical callable application engine?
```

Required answer: yes.

After Task 3:

```text
Does a rest binding exist formally with the same unavailable/dynamic reason as its source when proof is absent?
```

Required answer: yes.

After Tasks 4–6:

```text
Can a reviewer point separately to field contract, lifecycle proof, and path-local current field fact?
```

Required answer: yes; none may be aliases of the same declaration-surface `TypeKnowledge`.

After Tasks 7–8:

```text
Can a body-only same-signature edit leave an unaffected caller's final Arc pointer stable, while a signature edit recomputes the caller and changes its semantic result?
```

Required answer: yes.

After Task 9:

```text
Does an imported read have the external canonical target without destroying the local import declaration site's source identity?
```

Required answer: yes.

---

# Non-goals

This plan deliberately does not include:

- Technical-03 generic proof work;
- complete generic receiver specialization;
- generalized intersection/difference types for negative refinement;
- arbitrary user-defined refinement predicates;
- record/map/variant formal destructuring;
- static/class-field lifecycle;
- object-sensitive heap analysis;
- closure allocation identity or escape analysis beyond callable-value invocation;
- full replacement of the inferred-return refresh pass with a new query family;
- reflection model expansion;
- unrelated LSP/UI work.

If later work replaces inferred-return refresh with a fully query-owned result product, it should preserve the laws established here rather than reopening this capability closure.

---

# Definition of done

The gap-closure program is done when:

1. Technical 03 is merged and remains GREEN.
2. `refined_branch_with_abrupt_else_publishes_only_normal_value` proves the branch-local `Int` fact.
3. `higher_order_block_call_propagates_captured_result` is no longer ignored and passes.
4. `field_facts_survive_constructor_and_general_writes` is no longer ignored and passes from a genuine lifecycle/current-field model.
5. `collection_and_destructure_facts_preserve_element_shapes` is no longer ignored and passes with a real `tail` binding.
6. `dependency_edit_remove_readd_recomputes_affected_summary_deterministically` passes its pointer and count assertions.
7. `imported_binding_use_resolves_to_exported_declaration_not_local_import_site` passes without changing ordinary local binding resolution.
8. The full semantic test target has zero failures.
9. No implementation achieves GREEN by weakening `TypeKnowledge`, removing causal/status checks, deleting pointer-stability assertions, broadening Dynamic, or bypassing the canonical post-Spec-03 call path.

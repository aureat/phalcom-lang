# Phalcom Canonical Callable Application and Operation Semantics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace Phalcom's parallel call-like semantic shortcuts with one canonical callable-application engine so methods, callable values, operators, getters, setters, subscripts, constructors, and protocol sends share argument mapping, relation checking, result authority, status, causality, identity, and explanation semantics.

**Architecture:** Keep `checker/call.rs` as the sole owner of callable application. Introduce explicit application targets, receiver/callee premises, normalized arguments, static-shape binding, and a single `apply_resolved_callable(...) -> CallCheckResult` funnel. `expression.rs` becomes syntax/target-selection glue; structural field and transitional List/Map indexing paths either construct complete operation contracts or remain direct non-call operations with the same result/status discipline.

**Tech Stack:** Rust 2024, Cargo workspace, `phalcom-semantic`, canonical integration-test binary `tests/semantic.rs`, existing `Fixture` semantic test harness.

**Spec:** `docs/impl/semantic/semantic-correctness/technical/02-canonical-callable-application-and-operation-semantics-implementation-spec.md`

**Verified repository baseline:** `aureat/phalcom-lang` `main` at commit `6ced2afd83ee89d2a09f45b8ba3821482abf3752`.

## Global Constraints

- Technical Specification 01 is a hard prerequisite. Before starting this plan, Part 01 must already provide `CausalInvalidity::contains`, `TypedExpression::invalidate`, `TypedExpression::debug_assert_coherent`, atomic expression publication, and required-expression dependency propagation.
- If Part 01 lands with slightly different helper names, resolve that naming delta once before Task 1. Do not re-implement Part 01 logic here.
- `checker/call.rs` is the only production owner of fixed callable return derivation, callable target authority, fixed argument-to-parameter mapping, and call-local relation aggregation.
- Syntax may select a callable target; syntax may not select a weaker application algorithm.
- Do not implement Technical Specification 03 generic-proof fixes here. Generic inference remains delegated behind the new outer call boundary.
- Do not implement complete `*` / `**` / `***` parameter-shape semantics. Current `CallableParameter::rest: bool` cannot prove the full public rest model.
- Unsupported dynamic/rest shapes must fail closed or publish a deliberate dynamic boundary. Never approximate expansion as one positional slot.
- Every supplied argument expression is analyzed exactly once, in source order, even when target resolution fails.
- Every fixed non-generic bound argument receives one structured assignability judgment.
- A fixed return proposition may survive an invalid argument relation when the return is independent.
- Receiver/callee epistemic authority caps fixed result authority.
- Ordinary assumed arguments do not automatically weaken an independent fixed return.
- Causal invalidity is orthogonal to epistemic authority; `Ready + non-clean causal invalidity` remains legal.
- Every source assignment expression returns `Unit`, including direct field, property-setter, and subscript-setter syntax.
- Underlying setter/indexer callable result semantics remain separate from source assignment value semantics.
- Preserve constructor result origin as `EvidenceOrigin::ConstructorSemantics`.
- Preserve native result origin as `EvidenceOrigin::NativeSignature`.
- Do not broaden language surface while fixing correctness.
- Do not weaken existing tests to make migrations pass.
- Each task is RED → minimal implementation → focused verification → commit.
- Use `cargo test -p phalcom-semantic --test semantic <test-path>` for semantic integration tests.
- Run `cargo fmt --check` at every review gate.

---

# 1. Pre-Execution Gate

- [ ] Verify Part 01 helpers exist:

```bash
rg -n \
  'fn invalidate|debug_assert_coherent|publish_expression_analysis|propagate_required_dependencies' \
  phalcom-semantic/src/checker
```

- [ ] Verify Part 01 tests:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::expression_composition
```

Expected: PASS. If the module is absent, land Part 01 first.

---

# 2. Current Baseline Map

| File | Anchor | Current behavior |
|---|---|---|
| `phalcom-semantic/src/checker/call.rs` | `CallCheckResult` | Keep rich result carrier |
| `phalcom-semantic/src/checker/call.rs` | `promote_exact_return` | Restrict behind canonical application |
| `phalcom-semantic/src/checker/call.rs` | `resolve_call` / `resolve_call_inner` | Refactor into canonical outer application + generic/non-generic sub-engines |
| `phalcom-semantic/src/checker/context.rs` | `resolve_dispatch` | Currently strips `ResolvedDispatch` identity after side-table publication |
| `phalcom-semantic/src/checker/context.rs` | `apply_relation_outcome` | Reuse structured relation → status/cause integration |
| `phalcom-semantic/src/checker/expression.rs` | `synthesize_method_call` | Closest existing canonical path |
| `phalcom-semantic/src/checker/expression.rs` | `synthesize_unqualified_call` | Callable-value authority leak + non-callable invocation bug |
| `phalcom-semantic/src/checker/expression.rs` | `synthesize_binary_expr` | Resolves and promotes return without RHS relation |
| `phalcom-semantic/src/checker/expression.rs` | `synthesize_unary_expr` | Resolves and promotes return directly |
| `phalcom-semantic/src/checker/expression.rs` | `synthesize_get_property` | Getter callable bypasses call engine |
| `phalcom-semantic/src/checker/expression.rs` | `synthesize_set_property` | Ad hoc relation + RHS-valued result |
| `phalcom-semantic/src/checker/expression.rs` | `synthesize_index_expr` | Direct List/Map result shortcut + callable-return shortcut |
| `phalcom-semantic/src/checker/expression.rs` | `synthesize_set_index_expr` | Checks only List value relation + RHS-valued result |
| `phalcom-semantic/src/checker/statement.rs` | `resolve_iteration_element` | Direct protocol signature projection seam |
| `phalcom-semantic/src/dispatch.rs` | `ResolvedDispatch` / `ResolvedDispatchResult` | Reuse |
| `phalcom-semantic/src/diagnostic.rs` | `DiagnosticCode` | Add call-shape and not-callable codes |
| `phalcom-semantic/tests/semantic/foundations/mod.rs` | module list | Register new suite |

Parser/runtime syntax verified by repository tests:

```phalcom
class Box {
  _val
  val { _val }
  val=(put v) { _val = v }
}

class Storage {
  _m
  [_ k] { return _m[k] }
  [_ k]=(put v) { _m[k] = v }
}

b.val = 99
s[100] = 777
receiver.target(*values)
```

---

# 3. Task Decomposition

```text
Task 1   Test module + diagnostics + pure application domain models
Task 2   Static argument-shape binding and dynamic-shape classification
Task 3   Explicit resolved-dispatch target API
Task 4   Fixed-return authority and call-result conversion
Task 5   Canonical non-generic application engine
Task 6   Unresolved/dynamic application and child-analysis completeness
Task 7   Ordinary method and implicit-self migration
Task 8   Callable-valued local application and NotCallable semantics
Task 9   Binary and unary operator migration
Task 10  Getter, setter, and direct field-write migration
Task 11  Subscript-get migration + structural List/Map targets
Task 12  Subscript-set migration + Unit assignment projection
Task 13  Constructor/native and outer generic-authority regressions
Task 14  Iteration protocol application seam
Task 15  Legacy API deletion, structural audits, full verification
```

---

# 4. Task 1 — Add Canonical Call Domain Models and Diagnostics

**Files:**
- Modify: `phalcom-semantic/src/checker/call.rs`
- Modify: `phalcom-semantic/src/diagnostic.rs`
- Create: `phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs`
- Modify: `phalcom-semantic/tests/semantic/foundations/mod.rs`

**Produces:**

```rust
CallableApplicationTarget
CallTargetAuthority
CallPremise
ApplicationArgument<'a>
StaticCallShape
ArgumentBinding
ArgumentBindingPlan
ArgumentShapeFailure
```

- [ ] Create `canonical_call_application.rs` with imports:

```rust
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::{
    EvidenceOrigin,
    EvidenceStatus,
    TypeKnowledge,
};

use crate::semantic::support::Fixture;
```

- [ ] Register:

```rust
mod canonical_call_application;
```

- [ ] Add RED diagnostic test:

```rust
#[test]
fn canonical_call_diagnostic_codes_are_stable() {
    assert_eq!(
        DiagnosticCode::CallShapeMismatch.as_str(),
        "type.call.shape_mismatch",
    );
    assert_eq!(
        DiagnosticCode::NotCallable.as_str(),
        "type.call.not_callable",
    );
}
```

Run:

```bash
cargo test -p phalcom-semantic --lib \
  canonical_call_diagnostic_codes_are_stable
```

Expected: FAIL because variants do not exist.

- [ ] Add variants in `diagnostic.rs` immediately after `ArgumentMismatch`:

```rust
CallShapeMismatch,
NotCallable,
```

and mappings:

```rust
Self::CallShapeMismatch => "type.call.shape_mismatch",
Self::NotCallable => "type.call.not_callable",
```

- [ ] In `call.rs` add:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CallTargetAuthority {
    ExactDispatch,
    CallableValue(EvidenceStatus),
    StructuralBuiltin,
}

#[derive(Clone, Debug)]
pub(crate) struct CallableApplicationTarget {
    pub signature: CallableSignature,
    pub callable: Option<crate::identity::CallableId>,
    pub authority: CallTargetAuthority,
}

impl CallableApplicationTarget {
    pub(crate) fn exact(
        callable: crate::identity::CallableId,
        signature: CallableSignature,
    ) -> Self {
        Self {
            signature,
            callable: Some(callable),
            authority: CallTargetAuthority::ExactDispatch,
        }
    }

    pub(crate) fn callable_value(
        signature: CallableSignature,
        status: EvidenceStatus,
    ) -> Self {
        Self {
            signature,
            callable: None,
            authority: CallTargetAuthority::CallableValue(status),
        }
    }

    pub(crate) fn structural(
        signature: CallableSignature,
    ) -> Self {
        Self {
            signature,
            callable: None,
            authority: CallTargetAuthority::StructuralBuiltin,
        }
    }
}
```

- [ ] Add:

```rust
#[derive(Clone, Debug)]
pub(crate) struct CallPremise {
    pub knowledge: TypeKnowledge,
    pub status: AnalysisStatus,
    pub causal_invalidity:
        crate::checker::causal::CausalInvalidity,
    pub explanation:
        Option<crate::identity::ExplanationId>,
}

impl CallPremise {
    pub(crate) fn from_typed(
        ctx: &CheckingContext<'_>,
        expression:
            &crate::checker::typed_expr::TypedExpression,
    ) -> Self {
        let explanation = expression
            .expression_id
            .and_then(|id| {
                ctx.explanation_for_expression(id)
            });

        Self {
            knowledge: expression.knowledge.clone(),
            status: expression.status.clone(),
            causal_invalidity:
                expression.causal_invalidity,
            explanation,
        }
    }

    pub(crate) fn established(
        knowledge: TypeKnowledge,
    ) -> Self {
        Self {
            knowledge,
            status: AnalysisStatus::Ready,
            causal_invalidity:
                crate::checker::causal::CausalInvalidity::Clean,
            explanation: None,
        }
    }
}
```

- [ ] Add normalized arguments:

```rust
#[derive(Clone, Copy, Debug)]
pub(crate) enum ApplicationArgument<'a> {
    Positional {
        expression: &'a Expr,
        range: SourceRange,
    },
    Labeled {
        label: &'a str,
        expression: &'a Expr,
        range: SourceRange,
    },
    DynamicLabel {
        expression: &'a Expr,
        range: SourceRange,
    },
    Expansion {
        expression: &'a Expr,
        range: SourceRange,
    },
}
```

- [ ] Add adapter:

```rust
pub(crate) fn application_arguments(
    args: &[PackItem],
) -> Vec<ApplicationArgument<'_>> {
    args.iter()
        .map(|item| match item {
            PackItem::Positional { expr, range } => {
                ApplicationArgument::Positional {
                    expression: expr,
                    range: *range,
                }
            }
            PackItem::Labeled {
                label: PackLabel::Static { text, .. },
                value,
                range,
            } => ApplicationArgument::Labeled {
                label: text.as_str(),
                expression: value,
                range: *range,
            },
            PackItem::Labeled {
                value,
                range,
                ..
            } => ApplicationArgument::DynamicLabel {
                expression: value,
                range: *range,
            },
            PackItem::Expand { expr, range } => {
                ApplicationArgument::Expansion {
                    expression: expr,
                    range: *range,
                }
            }
        })
        .collect()
}
```

- [ ] Add data types:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StaticCallShape {
    Exact(Vec<SelectorSlot>),
    Dynamic(DynamicReason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArgumentBinding {
    pub argument_index: usize,
    pub parameter_index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArgumentBindingPlan {
    pub bindings: Vec<ArgumentBinding>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ArgumentShapeFailure {
    MissingRequiredParameter {
        parameter_index: usize,
    },
    UnexpectedPositional {
        argument_index: usize,
    },
    UnknownLabel {
        argument_index: usize,
        label: String,
    },
    DuplicateParameterBinding {
        parameter_index: usize,
    },
    UnsupportedRestShape,
    DynamicShape,
}
```

- [ ] Verify:

```bash
cargo fmt --all
cargo test -p phalcom-semantic --lib \
  canonical_call_diagnostic_codes_are_stable
cargo check -p phalcom-semantic
```

- [ ] Commit:

```bash
git add \
  phalcom-semantic/src/checker/call.rs \
  phalcom-semantic/src/diagnostic.rs \
  phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs \
  phalcom-semantic/tests/semantic/foundations/mod.rs

git commit -m "refactor(semantic): add canonical call application models"
```

---

# 5. Task 2 — Implement Static Shape Classification and Argument Binding

**Files:**
- Modify: `phalcom-semantic/src/checker/call.rs`

**Produces:**

```rust
static_call_shape(...)
bind_static_arguments(...)
```

- [ ] Add RED tests for:
  - positional + labeled exact slots;
  - expansion => `DynamicRestPack`;
  - dynamic label => `DynamicRestPack`.

Use parser-extracted expressions if direct AST literal construction changes on the landed branch.

- [ ] Implement:

```rust
pub(crate) fn static_call_shape(
    arguments: &[ApplicationArgument<'_>],
) -> StaticCallShape {
    let mut slots = Vec::with_capacity(arguments.len());

    for argument in arguments {
        match argument {
            ApplicationArgument::Positional { .. } => {
                slots.push(SelectorSlot::Positional);
            }
            ApplicationArgument::Labeled {
                label,
                ..
            } => {
                slots.push(
                    SelectorSlot::Label(
                        (*label).to_string(),
                    ),
                );
            }
            ApplicationArgument::DynamicLabel { .. }
            | ApplicationArgument::Expansion { .. } => {
                return StaticCallShape::Dynamic(
                    DynamicReason::DynamicRestPack,
                );
            }
        }
    }

    StaticCallShape::Exact(slots)
}
```

- [ ] Add binding tests:
  - labeled maps to exact external label;
  - positional maps to next unlabeled parameter;
  - missing parameter;
  - extra positional;
  - unknown label;
  - duplicate label binding;
  - rest parameter fails closed.

- [ ] Implement binder:

```rust
pub(crate) fn bind_static_arguments(
    arguments: &[ApplicationArgument<'_>],
    parameters: &[CallableParameter],
) -> Result<
    ArgumentBindingPlan,
    Vec<ArgumentShapeFailure>,
> {
    if parameters.iter().any(|p| p.rest) {
        return Err(vec![
            ArgumentShapeFailure::UnsupportedRestShape,
        ]);
    }

    if arguments.iter().any(|argument| matches!(
        argument,
        ApplicationArgument::DynamicLabel { .. }
            | ApplicationArgument::Expansion { .. }
    )) {
        return Err(vec![
            ArgumentShapeFailure::DynamicShape,
        ]);
    }

    let mut bindings = Vec::new();
    let mut bound = vec![false; parameters.len()];
    let mut failures = Vec::new();
    let mut positional_cursor = 0usize;

    for (argument_index, argument) in
        arguments.iter().enumerate()
    {
        let parameter_index = match argument {
            ApplicationArgument::Positional { .. } => {
                let mut found = None;
                while positional_cursor < parameters.len() {
                    let index = positional_cursor;
                    positional_cursor += 1;
                    if parameters[index]
                        .external_label
                        .is_none()
                        && !bound[index]
                    {
                        found = Some(index);
                        break;
                    }
                }
                found
            }

            ApplicationArgument::Labeled {
                label,
                ..
            } => parameters
                .iter()
                .enumerate()
                .find_map(|(index, parameter)| {
                    (parameter.external_label.as_deref()
                        == Some(*label))
                    .then_some(index)
                }),

            ApplicationArgument::DynamicLabel { .. }
            | ApplicationArgument::Expansion { .. } => {
                None
            }
        };

        let Some(parameter_index) = parameter_index else {
            match argument {
                ApplicationArgument::Positional { .. } => {
                    failures.push(
                        ArgumentShapeFailure::UnexpectedPositional {
                            argument_index,
                        },
                    );
                }
                ApplicationArgument::Labeled {
                    label,
                    ..
                } => {
                    failures.push(
                        ArgumentShapeFailure::UnknownLabel {
                            argument_index,
                            label: (*label).to_string(),
                        },
                    );
                }
                ApplicationArgument::DynamicLabel { .. }
                | ApplicationArgument::Expansion { .. } => {
                    failures.push(
                        ArgumentShapeFailure::DynamicShape,
                    );
                }
            }
            continue;
        };

        if bound[parameter_index] {
            failures.push(
                ArgumentShapeFailure::DuplicateParameterBinding {
                    parameter_index,
                },
            );
            continue;
        }

        bound[parameter_index] = true;
        bindings.push(ArgumentBinding {
            argument_index,
            parameter_index,
        });
    }

    for (parameter_index, parameter) in
        parameters.iter().enumerate()
    {
        if !parameter.rest && !bound[parameter_index] {
            failures.push(
                ArgumentShapeFailure::MissingRequiredParameter {
                    parameter_index,
                },
            );
        }
    }

    if failures.is_empty() {
        Ok(ArgumentBindingPlan { bindings })
    } else {
        Err(failures)
    }
}
```

- [ ] Verify:

```bash
cargo test -p phalcom-semantic --lib static_shape
cargo test -p phalcom-semantic --lib static_binding
cargo fmt --check
```

- [ ] Commit:

```bash
git add phalcom-semantic/src/checker/call.rs
git commit -m "refactor(semantic): add canonical call shape binding"
```

---

# 6. Task 3 — Preserve Explicit `ResolvedDispatch` Identity

**Files:**
- Modify: `phalcom-semantic/src/checker/context.rs`
- Test: `phalcom-semantic/src/checker/context.rs`

**Produces:**

```rust
resolve_dispatch_target(...)
```

- [ ] Write a context test that registers a simple `Owner.value()` signature, calls `resolve_dispatch_target`, and asserts both exact `CallableId` and return type survive.

- [ ] Extract current receiver-to-owner logic into:

```rust
fn dispatch_owner_for_lookup(
    &self,
    receiver: TypeId,
    lookup: DispatchLookup,
) -> Option<(DeclarationId, DispatchSide)>
```

Preserve current `Super`, `ClassObject`, `Nominal`, and nested `Applied` behavior exactly.

- [ ] Extract current applied-receiver and `Self` specialization into:

```rust
fn specialize_dispatch_signature(
    &mut self,
    receiver: TypeId,
    mut signature: CallableSignature,
) -> CallableSignature
```

Apply substitution to all parameter `TypeKnowledge` and return `TypeKnowledge`, then apply `specialize_self_type`. Do not specialize `signature.generics` here.

- [ ] Implement:

```rust
pub(crate) fn resolve_dispatch_target(
    &mut self,
    receiver: TypeId,
    selector: &Selector,
    lookup: DispatchLookup,
) -> ResolvedDispatchResult
```

It must:
1. use `dispatch_owner_for_lookup`;
2. call `resolve_dispatch_with_trace`;
3. record every visited declaration-surface dependency;
4. record consumed callable-signature dependency;
5. retain exact `CallableId`;
6. specialize the returned signature;
7. keep current side-table publication only as compatibility.

- [ ] Rewrite legacy `resolve_dispatch` as a projection over `resolve_dispatch_target`.

- [ ] Verify:

```bash
cargo test -p phalcom-semantic --lib \
  dispatch_target_preserves_callable_identity

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::expression_engine

cargo test -p phalcom-semantic --test semantic \
  semantic::incremental::checker_dependencies

cargo fmt --check
```

- [ ] Commit:

```bash
git add phalcom-semantic/src/checker/context.rs
git commit -m "refactor(semantic): preserve resolved dispatch targets"
```

---

# 7. Task 4 — Enforce Fixed-Return Authority and One Result Conversion

**Files:**
- Modify: `phalcom-semantic/src/checker/call.rs`
- Modify: `phalcom-semantic/src/checker/typed_expr.rs`
- Conditional: `phalcom-semantic/src/types/evidence.rs`

**Produces:**

```rust
derive_fixed_return(...)
weaken_known_to_status(...)
From<CallCheckResult> for TypedExpression
```

- [ ] Add tests proving:
  - Established receiver + exact fixed return => Established;
  - Assumed receiver + exact fixed return => Assumed;
  - Assumed callable-value target => result at most Assumed;
  - constructor origin remains `ConstructorSemantics`.

- [ ] Add:

```rust
fn minimum_evidence_status(
    left: EvidenceStatus,
    right: EvidenceStatus,
) -> EvidenceStatus {
    if left == EvidenceStatus::Assumed
        || right == EvidenceStatus::Assumed
    {
        EvidenceStatus::Assumed
    } else {
        EvidenceStatus::Established
    }
}
```

- [ ] Add:

```rust
fn target_base_authority(
    target: &CallableApplicationTarget,
) -> EvidenceStatus {
    match target.authority {
        CallTargetAuthority::ExactDispatch
        | CallTargetAuthority::StructuralBuiltin => {
            EvidenceStatus::Established
        }
        CallTargetAuthority::CallableValue(status) => status,
    }
}
```

- [ ] Add origin mapping:

```rust
fn target_fixed_return_origin(
    target: &CallableApplicationTarget,
) -> EvidenceOrigin {
    match target.authority {
        CallTargetAuthority::ExactDispatch => {
            match target.signature.kind {
                CallableSemanticKind::Ordinary => {
                    EvidenceOrigin::CallableSignature
                }
                CallableSemanticKind::Constructor => {
                    EvidenceOrigin::ConstructorSemantics
                }
                CallableSemanticKind::Native => {
                    EvidenceOrigin::NativeSignature
                }
            }
        }
        CallTargetAuthority::CallableValue(_) => {
            EvidenceOrigin::CallableSignature
        }
        CallTargetAuthority::StructuralBuiltin => {
            EvidenceOrigin::DeclarationSemantics
        }
    }
}
```

- [ ] Implement `weaken_known_to_status`. If provenance must be retained, add crate-private `TypeKnowledge::with_status_and_origin(...)` in `evidence.rs` and keep `TypeEvidence` fields private.

- [ ] Implement:

```rust
fn derive_fixed_return(
    target: &CallableApplicationTarget,
    premise: &CallPremise,
    range: SourceRange,
) -> TypeKnowledge {
    let Some(premise_status) =
        premise.knowledge.status()
    else {
        return premise.knowledge.clone();
    };

    let authority = minimum_evidence_status(
        target_base_authority(target),
        premise_status,
    );

    weaken_known_to_status(
        target.signature.return_type.clone(),
        authority,
        target_fixed_return_origin(target),
        range,
    )
}
```

- [ ] Delete or privatize `promote_exact_return` so `expression.rs` cannot call it.

- [ ] Add one conversion:

```rust
impl From<crate::checker::call::CallCheckResult>
    for TypedExpression
{
    fn from(
        result: crate::checker::call::CallCheckResult,
    ) -> Self {
        let mut typed = Self::new(result.knowledge);
        typed.status = result.status;
        typed.causal_invalidity =
            result.causal_invalidity;
        typed.explanation_parents =
            result.explanation_parents;
        typed.callable = result.callable;
        typed.debug_assert_coherent();
        typed
    }
}
```

- [ ] Verify:

```bash
cargo test -p phalcom-semantic --lib \
  assumed_receiver_caps_exact_fixed_return
cargo test -p phalcom-semantic --lib \
  assumed_callable_value_caps_fixed_return
cargo fmt --check
```

- [ ] Commit:

```bash
git add \
  phalcom-semantic/src/checker/call.rs \
  phalcom-semantic/src/checker/typed_expr.rs \
  phalcom-semantic/src/types/evidence.rs

git commit -m "fix(semantic): enforce callable result authority"
```

Only stage `evidence.rs` if modified.

---

# 8. Task 5 — Build Canonical Non-Generic Application

**Files:**
- Modify: `phalcom-semantic/src/checker/call.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs`

**Produces:**

```rust
apply_resolved_callable(...)
apply_non_generic_callable(...)
```

- [ ] Add integration regression:

```rust
#[test]
fn fixed_method_return_survives_argument_mismatch() {
    let fixture = Fixture::new(
        r#"
class Probe {
  accept(_ value: Int) -> Int {
    1
  }

  run() {
    self.accept("wrong")
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        DispatchSide::Instance,
    );

    let call = fixture.expression(
        run,
        r#"self.accept("wrong")"#,
    );

    assert_eq!(
        call.knowledge.ty(),
        Some(fixture.ty("Int")),
    );

    let AnalysisStatus::Invalid(cause) = call.status else {
        panic!("call must be Invalid: {call:#?}");
    };

    assert!(
        call.causal_invalidity.contains(cause),
        "{call:#?}",
    );

    fixture.assert_diagnostic(
        DiagnosticCode::ArgumentMismatch,
        1,
    );
}
```

- [ ] Add argument accessors:

```rust
impl<'a> ApplicationArgument<'a> {
    fn expression(self) -> &'a Expr {
        match self {
            Self::Positional { expression, .. }
            | Self::Labeled { expression, .. }
            | Self::DynamicLabel { expression, .. }
            | Self::Expansion { expression, .. } => expression,
        }
    }

    fn range(self) -> SourceRange {
        match self {
            Self::Positional { range, .. }
            | Self::Labeled { range, .. }
            | Self::DynamicLabel { range, .. }
            | Self::Expansion { range, .. } => range,
        }
    }
}
```

- [ ] Add:

```rust
fn parameter_for_argument<'a>(
    plan: &ArgumentBindingPlan,
    argument_index: usize,
    parameters: &'a [CallableParameter],
) -> Option<&'a CallableParameter> {
    let parameter_index = plan
        .bindings
        .iter()
        .find(|binding| {
            binding.argument_index == argument_index
        })?
        .parameter_index;

    parameters.get(parameter_index)
}
```

- [ ] Add shape diagnostic emitter. Emit `CallShapeMismatch` only for statically refuted missing/extra/unknown-label/duplicate-binding failures. `DynamicShape` and `UnsupportedRestShape` are not source contradictions.

- [ ] Implement `apply_non_generic_callable`:
  1. bind shape;
  2. retain plan when valid;
  3. emit static shape diagnostics when invalid;
  4. iterate supplied arguments in source order;
  5. derive parameter expected type from plan;
  6. analyze argument once;
  7. apply assignability exactly once for matched parameter;
  8. derive fixed return independently from relation outcome.

Core loop:

```rust
for (argument_index, argument) in
    arguments.iter().copied().enumerate()
{
    let parameter = plan
        .as_ref()
        .and_then(|plan| {
            parameter_for_argument(
                plan,
                argument_index,
                &target.signature.parameters,
            )
        });

    let expected = parameter
        .and_then(|p| p.ty.ty())
        .map(|ty| {
            ExpectedType::proper_from(
                ty,
                ExpectationOrigin::CallableSignature,
            )
        })
        .unwrap_or_default();

    let typed = analyze_expression(
        ctx,
        argument.expression(),
        &expected,
    );

    if let Some(parameter) = parameter {
        ctx.apply_assignability(
            &typed.knowledge,
            &parameter.ty,
            DiagnosticCode::ArgumentMismatch,
            argument_relation_message(
                argument_index,
                argument,
                parameter,
            ),
            argument.range(),
        );
    }
}
```

- [ ] Add `argument_relation_message(...)` with labeled/positional wording.

- [ ] Refactor current generic branch into:

```rust
fn apply_generic_callable(
    ctx: &mut CheckingContext<'_>,
    target: &CallableApplicationTarget,
    premise: &CallPremise,
    arguments: &[ApplicationArgument<'_>],
    expected: &ExpectedType,
    call_range: SourceRange,
) -> TypeKnowledge
```

Do not change generic solver laws yet.

- [ ] Implement top-level:

```rust
pub(crate) fn apply_resolved_callable(
    ctx: &mut CheckingContext<'_>,
    target: &CallableApplicationTarget,
    premise: &CallPremise,
    arguments: &[ApplicationArgument<'_>],
    expected: &ExpectedType,
    call_range: SourceRange,
) -> CallCheckResult
```

It must:
1. begin call causal capture;
2. delegate generic/non-generic;
3. end capture;
4. add premise explanation once;
5. join premise causal invalidity;
6. select canonical status;
7. copy explicit target `CallableId`;
8. assert product coherence.

- [ ] Use Part 01 terminal-precedence helper if available. Do not infer suppression merely from non-clean causal state.

- [ ] Verify:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application::fixed_method_return_survives_argument_mismatch

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::bidirectional_calls

cargo test -p phalcom-semantic --lib
cargo fmt --check
```

- [ ] Commit:

```bash
git add \
  phalcom-semantic/src/checker/call.rs \
  phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs

git commit -m "refactor(semantic): add canonical resolved call application"
```


---

# 9. Task 6 — Analyze Unresolved and Dynamic Applications Without Dropping Children

**Files:**
- Modify: `phalcom-semantic/src/checker/call.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs`

**Produces:**

```rust
UnresolvedApplicationReason
analyze_unresolved_application(...)
```

- [ ] Add RED dispatch-miss child-analysis test:

```rust
#[test]
fn dispatch_miss_still_analyzes_argument_expressions() {
    let fixture = Fixture::new(
        r#"
class Probe {
  run() {
    self.noSuchMethod(missing)
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        DispatchSide::Instance,
    );

    let argument =
        fixture.expression(run, "missing");

    assert!(matches!(
        argument.knowledge,
        TypeKnowledge::Unknown(
            phalcom_semantic::types::evidence::UnknownReason::UnresolvedName(_)
        )
    ));

    let call = fixture.expression(
        run,
        "self.noSuchMethod(missing)",
    );

    assert!(call.knowledge.is_unknown());
}
```

Expected current failure: `missing` may not be analyzed because dispatch fails before the call engine is entered.

- [ ] Add unresolved reason model:

```rust
#[derive(Clone, Debug)]
pub(crate) enum UnresolvedApplicationReason {
    PremiseUnknown,
    PremiseInvalidUnavailable,
    PremiseDynamic(DynamicReason),
    DispatchMissing,
    DispatchAmbiguous,
    DynamicShape(DynamicReason),
}
```

- [ ] Add source-order no-target analyzer:

```rust
fn analyze_unbound_arguments(
    ctx: &mut CheckingContext<'_>,
    arguments: &[ApplicationArgument<'_>],
) -> (
    CausalInvalidity,
    Vec<crate::identity::ExplanationId>,
    Option<AnalysisStatus>,
) {
    ctx.begin_call_causal_capture();

    for argument in arguments.iter().copied() {
        analyze_expression(
            ctx,
            argument.expression(),
            &ExpectedType::None,
        );
    }

    ctx.end_call_causal_capture()
}
```

- [ ] Implement:

```rust
pub(crate) fn analyze_unresolved_application(
    ctx: &mut CheckingContext<'_>,
    premise: &CallPremise,
    arguments: &[ApplicationArgument<'_>],
    reason: UnresolvedApplicationReason,
) -> CallCheckResult {
    let (
        argument_invalidity,
        mut explanation_parents,
        argument_status,
    ) = analyze_unbound_arguments(
        ctx,
        arguments,
    );

    if let Some(explanation) = premise.explanation {
        if !explanation_parents.contains(&explanation) {
            explanation_parents.push(explanation);
        }
    }

    let causal_invalidity = premise
        .causal_invalidity
        .join(argument_invalidity);

    let knowledge = match &reason {
        UnresolvedApplicationReason::PremiseUnknown => {
            premise.knowledge.clone()
        }

        UnresolvedApplicationReason::PremiseInvalidUnavailable => {
            TypeKnowledge::Unknown(
                UnknownReason::SuppressedByInvalidCause,
            )
        }

        UnresolvedApplicationReason::PremiseDynamic(reason)
        | UnresolvedApplicationReason::DynamicShape(reason) => {
            TypeKnowledge::Dynamic(reason.clone())
        }

        UnresolvedApplicationReason::DispatchMissing
        | UnresolvedApplicationReason::DispatchAmbiguous => {
            TypeKnowledge::Unknown(
                UnknownReason::DynamicMessageSend,
            )
        }
    };

    let status = if let Some(status) = argument_status {
        status
    } else {
        match &reason {
            UnresolvedApplicationReason::PremiseInvalidUnavailable => {
                causal_invalidity
                    .suppression_cause()
                    .map(AnalysisStatus::Suppressed)
                    .unwrap_or(AnalysisStatus::Ready)
            }

            UnresolvedApplicationReason::PremiseDynamic(reason)
            | UnresolvedApplicationReason::DynamicShape(reason) => {
                AnalysisStatus::DynamicBoundary(
                    reason.clone(),
                )
            }

            _ => AnalysisStatus::Ready,
        }
    };

    let result = CallCheckResult {
        knowledge,
        status,
        causal_invalidity,
        explanation_parents,
        callable: None,
    };

    debug_assert_call_result_coherent(&result);
    result
}
```

Use the Part 01 status-precedence helper if it exists so argument InternalFailure/Cancelled/BudgetExceeded outrank a dynamic shape boundary.

- [ ] Add dynamic spread regression:

```rust
#[test]
fn spread_call_shape_is_dynamic_not_one_positional_slot() {
    let fixture = Fixture::new(
        r#"
class Receiver {
  target(_ value: Int) -> Int {
    value
  }
}

class Probe {
  run(receiver: Receiver, values) {
    receiver.target(*values)
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        DispatchSide::Instance,
    );

    let call = fixture.expression(
        run,
        "receiver.target(*values)",
    );

    assert!(matches!(
        call.knowledge,
        TypeKnowledge::Dynamic(
            phalcom_semantic::types::evidence::DynamicReason::DynamicRestPack
        )
    ));
}
```

- [ ] Verify:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application::dispatch_miss_still_analyzes_argument_expressions

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application::spread_call_shape_is_dynamic_not_one_positional_slot

cargo fmt --check
```

- [ ] Commit:

```bash
git add \
  phalcom-semantic/src/checker/call.rs \
  phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs

git commit -m "fix(semantic): preserve children on unresolved calls"
```

---

# 10. Task 7 — Migrate Ordinary Method and Implicit-Self Calls

**Files:**
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs`

**Consumes:**
- `application_arguments`
- `static_call_shape`
- `CallPremise`
- `CallableApplicationTarget`
- `apply_resolved_callable`
- `analyze_unresolved_application`

- [ ] Add exact callable identity regression:

```rust
#[test]
fn explicit_method_call_publishes_exact_callable_identity() {
    let fixture = Fixture::new(
        r#"
class Probe {
  value() -> Int {
    1
  }

  run() {
    self.value()
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        DispatchSide::Instance,
    );

    let call =
        fixture.expression(run, "self.value()");

    let expected = fixture.callable_id(
        "Probe",
        "value",
        DispatchSide::Instance,
    );

    assert_eq!(
        call.callable.as_ref(),
        Some(&expected),
        "{call:#?}",
    );
}
```

- [ ] Add assumed receiver RED regression:

```rust
#[test]
fn assumed_receiver_caps_fixed_method_result() {
    let fixture = Fixture::new(
        r#"
class Worker {
  value() -> Int {
    1
  }
}

class Probe {
  run(worker: Worker) {
    worker.value()
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        DispatchSide::Instance,
    );

    let call =
        fixture.expression(run, "worker.value()");

    assert_eq!(
        call.knowledge.ty(),
        Some(fixture.ty("Int")),
    );
    assert_eq!(
        call.knowledge.status(),
        Some(EvidenceStatus::Assumed),
        "{call:#?}",
    );
}
```

Expected current failure: exact return promotion may establish the return even though body-entry `worker` is Assumed.

- [ ] In `synthesize_method_call`, keep:
  - receiver analysis;
  - sacred control-flow recognition.

Delete:
  - local selector-slot loop;
  - legacy `resolve_call` conversion;
  - manual receiver causal join.

- [ ] Replace with:

```rust
let arguments =
    application_arguments(&call.args);

let premise =
    CallPremise::from_typed(ctx, &recv_typed);
```

- [ ] Handle unavailable receiver before static dispatch:

```rust
let Some(receiver_ty) = recv_typed.knowledge.ty() else {
    let reason = match &recv_typed.knowledge {
        TypeKnowledge::Unknown(_) => {
            if matches!(
                recv_typed.status,
                AnalysisStatus::Invalid(_)
                    | AnalysisStatus::Suppressed(_)
            ) {
                UnresolvedApplicationReason::PremiseInvalidUnavailable
            } else {
                UnresolvedApplicationReason::PremiseUnknown
            }
        }

        TypeKnowledge::Dynamic(reason) => {
            UnresolvedApplicationReason::PremiseDynamic(
                reason.clone(),
            )
        }

        TypeKnowledge::Known(_) => unreachable!(),
    };

    return analyze_unresolved_application(
        ctx,
        &premise,
        &arguments,
        reason,
    )
    .into();
};
```

- [ ] Derive shape:

```rust
let slots = match static_call_shape(&arguments) {
    StaticCallShape::Exact(slots) => slots,

    StaticCallShape::Dynamic(reason) => {
        return analyze_unresolved_application(
            ctx,
            &premise,
            &arguments,
            UnresolvedApplicationReason::DynamicShape(
                reason,
            ),
        )
        .into();
    }
};
```

- [ ] Build selector and resolve explicit target:

```rust
let Ok(selector) =
    Selector::method(&call.method, slots)
else {
    return analyze_unresolved_application(
        ctx,
        &premise,
        &arguments,
        UnresolvedApplicationReason::DispatchMissing,
    )
    .into();
};

match ctx.resolve_dispatch_target(
    receiver_ty,
    &selector,
    recv_typed.dispatch_lookup.clone(),
) {
    ResolvedDispatchResult::Found(resolved) => {
        let target =
            CallableApplicationTarget::exact(
                resolved.callable,
                resolved.signature,
            );

        apply_resolved_callable(
            ctx,
            &target,
            &premise,
            &arguments,
            expected,
            call.range,
        )
        .into()
    }

    ResolvedDispatchResult::Missing { .. } => {
        analyze_unresolved_application(
            ctx,
            &premise,
            &arguments,
            UnresolvedApplicationReason::DispatchMissing,
        )
        .into()
    }

    ResolvedDispatchResult::Ambiguous(_) => {
        analyze_unresolved_application(
            ctx,
            &premise,
            &arguments,
            UnresolvedApplicationReason::DispatchAmbiguous,
        )
        .into()
    }

    ResolvedDispatchResult::Dynamic => {
        analyze_unresolved_application(
            ctx,
            &premise,
            &arguments,
            UnresolvedApplicationReason::PremiseDynamic(
                DynamicReason::RuntimeReflection,
            ),
        )
        .into()
    }
}
```

- [ ] Migrate the implicit-self branch in `synthesize_unqualified_call` to the same exact sequence:
  - current class/self type computation unchanged;
  - `CallPremise::established`;
  - normalized arguments;
  - static shape;
  - `resolve_dispatch_target`;
  - `apply_resolved_callable`.

- [ ] Do not change lexical-value-before-implicit-self precedence.

- [ ] Verify:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application::assumed_receiver_caps_fixed_method_result

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application::explicit_method_call_publishes_exact_callable_identity

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::bidirectional_calls

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::expression_engine

cargo fmt --check
```

- [ ] Commit:

```bash
git add \
  phalcom-semantic/src/checker/expression.rs \
  phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs

git commit -m "refactor(semantic): route method calls through canonical application"
```

---

# 11. Task 8 — Fix Callable-Valued Local Application and `NotCallable`

**Files:**
- Modify: `phalcom-semantic/src/checker/call.rs`
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs`

- [ ] Add RED regression:

```rust
#[test]
fn invoking_known_non_callable_is_invalid_not_a_value_read() {
    let fixture = Fixture::new(
        r#"
class Probe {
  run() {
    let value = 1
    value()
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        DispatchSide::Instance,
    );

    let call =
        fixture.expression(run, "value()");

    assert!(
        call.knowledge.ty()
            != Some(fixture.ty("Int")),
        "invocation must not degrade to value read: {call:#?}",
    );

    assert!(matches!(
        call.status,
        AnalysisStatus::Invalid(_)
    ));

    fixture.assert_diagnostic(
        DiagnosticCode::NotCallable,
        1,
    );
}
```

- [ ] Add callable-value target builder in `call.rs`.

For the exact type behind `TypeData::Callable`, build:
  - selector base `call`, not lexical variable name;
  - parameter contracts `Assumed(..., CallableSignature)`;
  - return contract `Assumed(..., CallableSignature)`;
  - target authority `CallableValue(callee_status)`.

Concrete logic:

```rust
let mut parameters =
    Vec::with_capacity(callable.parameters.len());
let mut slots =
    Vec::with_capacity(callable.parameters.len());

for parameter in callable.parameters.iter() {
    let mut formal = CallableParameter::new(
        "argument",
        TypeKnowledge::assumed(
            parameter.ty,
            EvidenceOrigin::CallableSignature,
        ),
    )
    .with_rest(parameter.rest);

    if let Some(label) = &parameter.label {
        formal =
            formal.with_label(label.to_string());
        slots.push(
            SelectorSlot::Label(
                label.to_string(),
            ),
        );
    } else {
        slots.push(SelectorSlot::Positional);
    }

    parameters.push(formal);
}

let signature = CallableSignature::new(
    Selector::method("call", slots)
        .expect("callable type selector"),
    parameters,
    TypeKnowledge::assumed(
        callable.return_type,
        EvidenceOrigin::CallableSignature,
    ),
);
```

- [ ] In lexical local branch, construct `CallPremise` from:
  - current binding knowledge;
  - flow-state causal invalidity;
  - binding explanation if available.

- [ ] For `TypeData::Callable`, call `apply_resolved_callable`.

- [ ] For known non-callable, call new helper:

```rust
pub(crate) fn analyze_non_callable_invocation(
    ctx: &mut CheckingContext<'_>,
    premise: &CallPremise,
    args: &[PackItem],
    call_range: SourceRange,
) -> CallCheckResult
```

Implementation:
1. analyze supplied arguments using `analyze_unresolved_application`;
2. emit `NotCallable`;
3. set `Invalid(cause)`;
4. join `CausalInvalidity::One(cause)`.

- [ ] For lexical Unknown/Dynamic, preserve lexical shadowing and call unresolved/dynamic application. Do not continue to implicit-self fallback.

- [ ] Add lexical shadowing regression:

```rust
#[test]
fn lexical_unknown_callee_does_not_fall_back_to_self_method() {
    let fixture = Fixture::new(
        r#"
class Probe {
  helper() -> Int {
    1
  }

  run(value) {
    let helper = value
    helper()
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        DispatchSide::Instance,
    );

    let call =
        fixture.expression(run, "helper()");

    assert!(
        call.callable.is_none(),
        "lexical helper binding must shadow self.helper(): {call:#?}",
    );
}
```

- [ ] Add lower-level/unit regression proving an Assumed callable target cannot produce Established fixed return.

- [ ] Verify:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application::invoking_known_non_callable_is_invalid_not_a_value_read

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application::lexical_unknown_callee_does_not_fall_back_to_self_method

cargo fmt --check
```

- [ ] Commit:

```bash
git add \
  phalcom-semantic/src/checker/call.rs \
  phalcom-semantic/src/checker/expression.rs \
  phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs

git commit -m "fix(semantic): canonicalize callable value invocation"
```

---

# 12. Task 9 — Migrate Binary and Unary Operators

**Files:**
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs`

- [ ] Add RED binary mismatch regression:

```rust
#[test]
fn binary_operator_checks_rhs_parameter_relation() {
    let fixture = Fixture::new(
        r#"
class Probe {
  run() {
    1 + "wrong"
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        DispatchSide::Instance,
    );

    let expr =
        fixture.expression(run, r#"1 + "wrong""#);

    assert_eq!(
        expr.knowledge.ty(),
        Some(fixture.ty("Int")),
        "{expr:#?}",
    );

    let AnalysisStatus::Invalid(cause) =
        expr.status
    else {
        panic!(
            "operator application must be Invalid: {expr:#?}"
        );
    };

    assert!(
        expr.causal_invalidity.contains(cause),
        "{expr:#?}",
    );

    fixture.assert_diagnostic(
        DiagnosticCode::ArgumentMismatch,
        1,
    );
}
```

- [ ] Rewrite binary operand order:
  1. analyze left;
  2. construct left premise;
  3. construct one positional application argument referencing RHS;
  4. resolve operator target;
  5. let canonical application analyze RHS with parameter context.

Do not eagerly analyze RHS before target resolution.

- [ ] Keep the existing `BinaryOp -> selector base` mapping exactly.

- [ ] Build:

```rust
let selector = Selector::method(
    op_name,
    vec![SelectorSlot::Positional],
)
.expect("binary operator selector");
```

- [ ] Resolve through `resolve_dispatch_target` and apply through `apply_resolved_callable`.

- [ ] On missing/ambiguous/dynamic target, call `analyze_unresolved_application` so RHS is still analyzed.

- [ ] Delete direct `promote_exact_return` usage and manual left/right causal join.

- [ ] Add assertion that a resolved operator carries `CallableId`. If core bootstrap surfaces do not expose one for the chosen operator, define a user operator fixture using the parser-supported declaration form; do not weaken the identity requirement.

- [ ] Rewrite unary path:
  - analyze operand once;
  - construct premise;
  - resolve getter/operator selector;
  - call canonical application with `&[]`;
  - no direct return promotion.

- [ ] Add authority regression:

```rust
#[test]
fn unary_operator_respects_receiver_authority() {
    let fixture = Fixture::new(
        r#"
class Probe {
  run(value: Int) {
    -value
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        DispatchSide::Instance,
    );

    let expr =
        fixture.expression(run, "-value");

    assert_eq!(
        expr.knowledge.status(),
        Some(EvidenceStatus::Assumed),
        "{expr:#?}",
    );
}
```

- [ ] Verify:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application::binary_operator_checks_rhs_parameter_relation

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application::unary_operator_respects_receiver_authority

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::expression_engine::test_binary_and_unary_as_message_sends

cargo fmt --check
```

- [ ] Commit:

```bash
git add \
  phalcom-semantic/src/checker/expression.rs \
  phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs

git commit -m "fix(semantic): route operators through canonical application"
```

---

# 13. Task 10 — Migrate Getter, Setter, and Direct Field Write Semantics

**Files:**
- Modify: `phalcom-semantic/src/checker/call.rs`
- Modify: `phalcom-semantic/src/checker/context.rs` only if `RelationApplication` gains a status field
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs`

- [ ] Add direct field Unit regression:

```rust
#[test]
fn direct_field_assignment_expression_is_unit() {
    let fixture = Fixture::new(
        r#"
class Box {
  value: Int = 0

  run() {
    let result = (self.value = 1)
  }
}
"#,
    );

    let run = fixture.callable(
        "Box",
        "run",
        DispatchSide::Instance,
    );

    let assignment =
        fixture.expression(run, "self.value = 1");

    assert_eq!(
        assignment.knowledge.ty(),
        Some(fixture.ty("Unit")),
        "{assignment:#?}",
    );
}
```

- [ ] Add setter Unit + callable identity regression. The current parser explicitly accepts `name=(put local: Type)`; use that exact typed setter form:

```rust
#[test]
fn setter_assignment_expression_is_unit_and_keeps_callable() {
    let fixture = Fixture::new(
        r#"
class Box {
  _value: Int = 0

  value {
    _value
  }

  value=(put next: Int) {
    _value = next
  }

  run() {
    let result = (self.value = 1)
  }
}
"#,
    );

    let run = fixture.callable(
        "Box",
        "run",
        DispatchSide::Instance,
    );

    let assignment =
        fixture.expression(run, "self.value = 1");

    assert_eq!(
        assignment.knowledge.ty(),
        Some(fixture.ty("Unit")),
    );

    assert!(
        assignment.callable.is_some(),
        "{assignment:#?}",
    );
}
```

- [ ] Add setter mismatch regression:

```rust
#[test]
fn setter_assignment_checks_value_and_keeps_unit() {
    let fixture = Fixture::new(
        r#"
class Box {
  _value: Int = 0

  value=(put next: Int) {
    _value = next
  }

  run() {
    self.value = "wrong"
  }
}
"#,
    );

    let run = fixture.callable(
        "Box",
        "run",
        DispatchSide::Instance,
    );

    let assignment = fixture.expression(
        run,
        r#"self.value = "wrong""#,
    );

    assert_eq!(
        assignment.knowledge.ty(),
        Some(fixture.ty("Unit")),
    );

    assert!(matches!(
        assignment.status,
        AnalysisStatus::Invalid(_)
    ));

    fixture.assert_diagnostic(
        DiagnosticCode::ArgumentMismatch,
        1,
    );
}
```

- [ ] Add assignment projection helper in `call.rs`:

```rust
pub(crate) fn assignment_result_from_call(
    ctx: &mut CheckingContext<'_>,
    operation: CallCheckResult,
    range: SourceRange,
) -> TypedExpression {
    let mut typed = TypedExpression::established(
        ctx.store.unit(),
        EvidenceOrigin::Syntax,
        range,
    );

    typed.status = operation.status;
    typed.causal_invalidity =
        operation.causal_invalidity;
    typed.callable = operation.callable;
    typed.explanation_parents =
        operation.explanation_parents;

    typed.debug_assert_coherent();
    typed
}
```

- [ ] Migrate getter callable branch:
  - preserve field-first precedence;
  - direct field remains structural;
  - getter selector uses `resolve_dispatch_target`;
  - exact getter uses canonical application with zero arguments.

- [ ] For direct field writes, use field contract as RHS expected type and apply exactly one relation.

- [ ] Ensure direct field relation terminal status is not discarded. Preferred implementation: extend `RelationApplication` to carry:

```rust
pub status: Option<AnalysisStatus>
```

inside `context.rs::apply_relation_outcome`.

Then add:

```rust
fn apply_relation_application_to_typed(
    typed: &mut TypedExpression,
    application: &RelationApplication,
) {
    if let Some(status) = &application.status {
        match status {
            AnalysisStatus::Invalid(cause) => {
                typed.invalidate(*cause);
            }
            other => {
                typed.status = other.clone();
            }
        }
    }

    typed.debug_assert_coherent();
}
```

This avoids reconstructing terminal status from `RelationOutcome`.

- [ ] Direct field write shape:

```rust
let expected = field_k
    .ty()
    .map(|ty| {
        ExpectedType::proper_from(
            ty,
            ExpectationOrigin::AssignmentContract,
        )
    })
    .unwrap_or_default();

let value_typed =
    analyze_expression(ctx, &set.value, &expected);

let application = ctx.apply_assignability(
    &value_typed.knowledge,
    &field_k,
    DiagnosticCode::FieldMismatch,
    format!(
        "assigned value does not match field `{}` type",
        set.property,
    ),
    set.range,
);

let mut result = TypedExpression::established(
    ctx.store.unit(),
    EvidenceOrigin::Syntax,
    set.range,
);

propagate_required_dependencies(
    &mut result,
    &[recv_typed, value_typed],
);

apply_relation_application_to_typed(
    &mut result,
    &application,
);

return result;
```

- [ ] Setter callable branch:
  - do not pre-analyze RHS;
  - create one positional `ApplicationArgument`;
  - resolve setter target;
  - call `apply_resolved_callable`;
  - wrap via `assignment_result_from_call`.

Delete:
  - `sig.parameters.first()`;
  - ad hoc `AssignmentMismatch`;
  - `TypedExpression::new(val_k)` result.

- [ ] Verify:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application::direct_field_assignment_expression_is_unit

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application::setter_assignment_expression_is_unit_and_keeps_callable

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application::setter_assignment_checks_value_and_keeps_unit

cargo fmt --check
```

- [ ] Commit:

```bash
git add \
  phalcom-semantic/src/checker/call.rs \
  phalcom-semantic/src/checker/context.rs \
  phalcom-semantic/src/checker/expression.rs \
  phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs

git commit -m "fix(semantic): canonicalize getter and property assignment"
```

Only stage `context.rs` if `RelationApplication.status` was added.


---

# 14. Task 11 — Migrate Subscript Get and Replace Result-Only List/Map Shortcuts

**Files:**
- Modify: `phalcom-semantic/src/checker/call.rs`
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs`

- [ ] Add List wrong-index RED regression:

```rust
#[test]
fn list_subscript_checks_index_contract() {
    let fixture = Fixture::new(
        r#"
class Probe {
  run() {
    let values: List<Int> = [1]
    values["wrong"]
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        DispatchSide::Instance,
    );

    let index = fixture.expression(
        run,
        r#"values["wrong"]"#,
    );

    assert_eq!(
        index.knowledge.ty(),
        Some(fixture.ty("Int")),
        "{index:#?}",
    );

    assert!(
        !index.status.is_ready(),
        "wrong index must not be Ready: {index:#?}",
    );

    fixture.assert_diagnostic(
        DiagnosticCode::ArgumentMismatch,
        1,
    );
}
```

Expected current failure: direct `List<T>` shortcut returns `T` without checking index.

- [ ] Add Map wrong-key regression:

```rust
#[test]
fn map_subscript_checks_key_contract() {
    let fixture = Fixture::new(
        r#"
class Probe {
  run() {
    let values: Map<String, Int> = { key: 1 }
    values[1]
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        DispatchSide::Instance,
    );

    let index =
        fixture.expression(run, "values[1]");

    assert_eq!(
        index.knowledge.ty(),
        Some(fixture.ty("Int")),
        "{index:#?}",
    );

    assert!(!index.status.is_ready());

    fixture.assert_diagnostic(
        DiagnosticCode::ArgumentMismatch,
        1,
    );
}
```

- [ ] Add transitional List get target in `call.rs`:

```rust
pub(crate) fn structural_list_index_get_target(
    ctx: &mut CheckingContext<'_>,
    receiver_ty: TypeId,
) -> Option<CallableApplicationTarget> {
    let TypeData::Applied {
        origin,
        arguments,
    } = ctx.store.get(receiver_ty).clone()
    else {
        return None;
    };

    let TypeData::Nominal { declaration } =
        ctx.store.get(origin)
    else {
        return None;
    };

    if declaration.name.as_ref() != "List"
        || arguments.len() != 1
    {
        return None;
    }

    let int_decl = ctx.resolve_type_name("Int")?;
    let int_ty = ctx.nominal_type_of(&int_decl);
    let element_ty = arguments[0];

    let selector = Selector::subscript_get(
        vec![SelectorSlot::Positional],
    )
    .ok()?;

    let signature = CallableSignature::new(
        selector,
        vec![CallableParameter::new(
            "index",
            TypeKnowledge::established(
                int_ty,
                EvidenceOrigin::DeclarationSemantics,
            ),
        )],
        TypeKnowledge::established(
            element_ty,
            EvidenceOrigin::DeclarationSemantics,
        ),
    );

    Some(
        CallableApplicationTarget::structural(
            signature,
        ),
    )
}
```

Add comment immediately above:

```rust
/// Transitional semantic fallback.
/// Delete when canonical universe dispatch surfaces publish this indexer.
/// This helper may construct a target only; it must never construct the
/// final subscript expression result.
```

- [ ] Add Map target:

```rust
pub(crate) fn structural_map_index_get_target(
    ctx: &mut CheckingContext<'_>,
    receiver_ty: TypeId,
) -> Option<CallableApplicationTarget> {
    let TypeData::Applied {
        origin,
        arguments,
    } = ctx.store.get(receiver_ty).clone()
    else {
        return None;
    };

    let TypeData::Nominal { declaration } =
        ctx.store.get(origin)
    else {
        return None;
    };

    if declaration.name.as_ref() != "Map"
        || arguments.len() != 2
    {
        return None;
    }

    let key_ty = arguments[0];
    let value_ty = arguments[1];

    let selector = Selector::subscript_get(
        vec![SelectorSlot::Positional],
    )
    .ok()?;

    let signature = CallableSignature::new(
        selector,
        vec![CallableParameter::new(
            "key",
            TypeKnowledge::established(
                key_ty,
                EvidenceOrigin::DeclarationSemantics,
            ),
        )],
        TypeKnowledge::established(
            value_ty,
            EvidenceOrigin::DeclarationSemantics,
        ),
    );

    Some(
        CallableApplicationTarget::structural(
            signature,
        ),
    )
}
```

- [ ] Rewrite `synthesize_index_expr`:
  1. analyze receiver only;
  2. create `CallPremise`;
  3. normalize `idx.args`;
  4. derive static shape;
  5. try canonical `subscript_get` dispatch first;
  6. on exact target, canonical application;
  7. on Missing only, try List then Map structural targets;
  8. if no target, unresolved application;
  9. delete eager index-analysis loop;
  10. delete direct `TypedExpression::established(elem_ty, Flow, ...)` and Map equivalent.

- [ ] Exact canonical target branch:

```rust
let selector = Selector::subscript_get(slots)
    .expect("static subscript shape");

match ctx.resolve_dispatch_target(
    recv_ty,
    &selector,
    recv_typed.dispatch_lookup.clone(),
) {
    ResolvedDispatchResult::Found(resolved) => {
        let target =
            CallableApplicationTarget::exact(
                resolved.callable,
                resolved.signature,
            );

        return apply_resolved_callable(
            ctx,
            &target,
            &premise,
            &arguments,
            &ExpectedType::None,
            idx.range,
        )
        .into();
    }

    ResolvedDispatchResult::Missing { .. } => {}

    ResolvedDispatchResult::Ambiguous(_) => {
        return analyze_unresolved_application(
            ctx,
            &premise,
            &arguments,
            UnresolvedApplicationReason::DispatchAmbiguous,
        )
        .into();
    }

    ResolvedDispatchResult::Dynamic => {
        return analyze_unresolved_application(
            ctx,
            &premise,
            &arguments,
            UnresolvedApplicationReason::PremiseDynamic(
                DynamicReason::RuntimeReflection,
            ),
        )
        .into();
    }
}
```

- [ ] Missing fallback:

```rust
if let Some(target) =
    structural_list_index_get_target(
        ctx,
        recv_ty,
    )
    .or_else(|| {
        structural_map_index_get_target(
            ctx,
            recv_ty,
        )
    })
{
    return apply_resolved_callable(
        ctx,
        &target,
        &premise,
        &arguments,
        &ExpectedType::None,
        idx.range,
    )
    .into();
}
```

- [ ] Add receiver authority regression:

```rust
#[test]
fn assumed_list_receiver_caps_structural_index_result() {
    let fixture = Fixture::new(
        r#"
class Probe {
  run(values: List<Int>) {
    values[0]
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        DispatchSide::Instance,
    );

    let index =
        fixture.expression(run, "values[0]");

    assert_eq!(
        index.knowledge.status(),
        Some(EvidenceStatus::Assumed),
        "{index:#?}",
    );
}
```

- [ ] Verify:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application::list_subscript_checks_index_contract

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application::map_subscript_checks_key_contract

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application::assumed_list_receiver_caps_structural_index_result

cargo fmt --check
```

- [ ] Commit:

```bash
git add \
  phalcom-semantic/src/checker/call.rs \
  phalcom-semantic/src/checker/expression.rs \
  phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs

git commit -m "fix(semantic): canonicalize subscript get application"
```

---

# 15. Task 12 — Migrate Subscript Set and Enforce Unit Assignment Result

**Files:**
- Modify: `phalcom-semantic/src/checker/call.rs`
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs`

- [ ] Add wrong-index RED regression:

```rust
#[test]
fn list_subscript_set_checks_index_contract_and_returns_unit() {
    let fixture = Fixture::new(
        r#"
class Probe {
  run() {
    let values: List<Int> = [1]
    values["wrong"] = 2
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        DispatchSide::Instance,
    );

    let assignment = fixture.expression(
        run,
        r#"values["wrong"] = 2"#,
    );

    assert_eq!(
        assignment.knowledge.ty(),
        Some(fixture.ty("Unit")),
    );

    assert!(!assignment.status.is_ready());

    fixture.assert_diagnostic(
        DiagnosticCode::ArgumentMismatch,
        1,
    );
}
```

- [ ] Add wrong-value regression:

```rust
#[test]
fn list_subscript_set_checks_value_contract_and_returns_unit() {
    let fixture = Fixture::new(
        r#"
class Probe {
  run() {
    let values: List<Int> = [1]
    values[0] = "wrong"
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        DispatchSide::Instance,
    );

    let assignment = fixture.expression(
        run,
        r#"values[0] = "wrong""#,
    );

    assert_eq!(
        assignment.knowledge.ty(),
        Some(fixture.ty("Unit")),
    );

    assert!(matches!(
        assignment.status,
        AnalysisStatus::Invalid(_)
    ));

    fixture.assert_diagnostic(
        DiagnosticCode::ArgumentMismatch,
        1,
    );
}
```

- [ ] Add structural List set target:

```rust
pub(crate) fn structural_list_index_set_target(
    ctx: &mut CheckingContext<'_>,
    receiver_ty: TypeId,
) -> Option<CallableApplicationTarget> {
    let TypeData::Applied {
        origin,
        arguments,
    } = ctx.store.get(receiver_ty).clone()
    else {
        return None;
    };

    let TypeData::Nominal { declaration } =
        ctx.store.get(origin)
    else {
        return None;
    };

    if declaration.name.as_ref() != "List"
        || arguments.len() != 1
    {
        return None;
    }

    let int_decl = ctx.resolve_type_name("Int")?;
    let int_ty = ctx.nominal_type_of(&int_decl);
    let element_ty = arguments[0];

    let selector = Selector::subscript_set(
        vec![SelectorSlot::Positional],
    )
    .ok()?;

    let signature = CallableSignature::new(
        selector,
        vec![
            CallableParameter::new(
                "index",
                TypeKnowledge::established(
                    int_ty,
                    EvidenceOrigin::DeclarationSemantics,
                ),
            ),
            CallableParameter::new(
                "put",
                TypeKnowledge::established(
                    element_ty,
                    EvidenceOrigin::DeclarationSemantics,
                ),
            ),
        ],
        TypeKnowledge::established(
            element_ty,
            EvidenceOrigin::DeclarationSemantics,
        ),
    );

    Some(
        CallableApplicationTarget::structural(
            signature,
        ),
    )
}
```

The underlying synthetic return mirrors the current put-type convention; source assignment projection discards it.

- [ ] Add helper:

```rust
fn subscript_set_arguments<'a>(
    args: &'a [PackItem],
    value: &'a Expr,
) -> Vec<ApplicationArgument<'a>> {
    let mut arguments =
        application_arguments(args);

    arguments.push(
        ApplicationArgument::Positional {
            expression: value,
            range: value.range(),
        },
    );

    arguments
}
```

- [ ] Rewrite `synthesize_set_index_expr`:
  - analyze receiver only;
  - build premise;
  - derive selector shape from index arguments only;
  - build application arguments from index args + put value;
  - canonical dispatch first;
  - structural List fallback only on Missing;
  - canonical application;
  - wrap result with `assignment_result_from_call`;
  - dynamic shape also gets Unit result with dynamic operation status;
  - delete eager index analysis;
  - delete List value-only relation;
  - delete RHS-valued result.

- [ ] Keep selector construction based on index lanes only:

```rust
let index_arguments =
    application_arguments(&set_idx.args);

let all_arguments =
    subscript_set_arguments(
        &set_idx.args,
        &set_idx.value,
    );

let slots = match static_call_shape(
    &index_arguments,
) {
    StaticCallShape::Exact(slots) => slots,

    StaticCallShape::Dynamic(reason) => {
        let operation =
            analyze_unresolved_application(
                ctx,
                &premise,
                &all_arguments,
                UnresolvedApplicationReason::DynamicShape(
                    reason,
                ),
            );

        return assignment_result_from_call(
            ctx,
            operation,
            set_idx.range,
        );
    }
};
```

- [ ] For exact target:

```rust
let operation = apply_resolved_callable(
    ctx,
    &target,
    &premise,
    &all_arguments,
    &ExpectedType::None,
    set_idx.range,
);

return assignment_result_from_call(
    ctx,
    operation,
    set_idx.range,
);
```

- [ ] Add user-defined index setter regression. The parser accepts `[_ key: Int]=(put value: Int) -> ReturnType`; use that exact typed indexer/setter grammar. Required assertions:
  - assignment expression type `Unit`;
  - exact index-set `CallableId` retained.

- [ ] Verify:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application::list_subscript_set_checks_index_contract_and_returns_unit

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application::list_subscript_set_checks_value_contract_and_returns_unit

cargo fmt --check
```

- [ ] Commit:

```bash
git add \
  phalcom-semantic/src/checker/call.rs \
  phalcom-semantic/src/checker/expression.rs \
  phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs

git commit -m "fix(semantic): canonicalize subscript assignment"
```

---

# 16. Task 13 — Lock Down Constructor, Native, and Generic Outer Authority

**Files:**
- Modify: `phalcom-semantic/src/checker/call.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs`
- Extend: `phalcom-semantic/tests/semantic/foundations/bidirectional_calls.rs`

- [ ] Add constructor origin regression:

```rust
#[test]
fn constructor_result_keeps_constructor_semantics_origin() {
    let fixture = Fixture::new(
        r#"
class CellNum {
  @constructor
  new() {
  }
}

class Probe {
  run() {
    CellNum.new()
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        DispatchSide::Instance,
    );

    let call =
        fixture.expression(run, "CellNum.new()");

    assert_eq!(
        call.knowledge.ty(),
        Some(fixture.ty("CellNum")),
    );

    assert_eq!(
        call.knowledge.origin(),
        Some(EvidenceOrigin::ConstructorSemantics),
    );
}
```

- [ ] Add invalid constructor argument regression:

```rust
#[test]
fn constructor_keeps_instance_result_when_argument_relation_is_invalid() {
    let fixture = Fixture::new(
        r#"
class CellNum {
  @constructor
  new(_ value: Int) {
  }
}

class Probe {
  run() {
    CellNum.new("wrong")
  }
}
"#,
    );

    let run = fixture.callable(
        "Probe",
        "run",
        DispatchSide::Instance,
    );

    let call = fixture.expression(
        run,
        r#"CellNum.new("wrong")"#,
    );

    assert_eq!(
        call.knowledge.ty(),
        Some(fixture.ty("CellNum")),
    );

    assert_eq!(
        call.knowledge.origin(),
        Some(EvidenceOrigin::ConstructorSemantics),
    );

    assert!(matches!(
        call.status,
        AnalysisStatus::Invalid(_)
    ));

    fixture.assert_diagnostic(
        DiagnosticCode::ArgumentMismatch,
        1,
    );
}
```

- [ ] Add outer generic cap helper:

```rust
fn cap_result_to_premise_authority(
    target: &CallableApplicationTarget,
    premise: &CallPremise,
    result: TypeKnowledge,
    range: SourceRange,
) -> TypeKnowledge {
    let Some(premise_status) =
        premise.knowledge.status()
    else {
        return result;
    };

    let maximum = minimum_evidence_status(
        target_base_authority(target),
        premise_status,
    );

    let origin = result
        .origin()
        .unwrap_or(
            EvidenceOrigin::GenericInference,
        );

    weaken_known_to_status(
        result,
        maximum,
        origin,
        range,
    )
}
```

- [ ] Apply this exactly once to the result of `apply_generic_callable` in `apply_resolved_callable`.

Do not apply it again to the non-generic result already derived with premise authority.

- [ ] Add generic receiver regression to `bidirectional_calls.rs`:

```phalcom
class Box {
  echo<T>(_ value: T) -> T {
    value
  }
}

class Probe {
  run(box: Box) {
    box.echo(1)
  }
}
```

Assert the generic result EvidenceStatus is not stronger than Assumed because the receiver premise is Assumed.

- [ ] Do not fix Unknown generic argument omission or generic expected-result authority here. Those become Part 03 work.

- [ ] Verify:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application::constructor_result_keeps_constructor_semantics_origin

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application::constructor_keeps_instance_result_when_argument_relation_is_invalid

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::bidirectional_calls

cargo fmt --check
```

- [ ] Commit:

```bash
git add \
  phalcom-semantic/src/checker/call.rs \
  phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs \
  phalcom-semantic/tests/semantic/foundations/bidirectional_calls.rs

git commit -m "fix(semantic): preserve callable authority across constructors and generics"
```

---

# 17. Task 14 — Remove Parameterized Iteration Signature Projection Bypass

**Files:**
- Modify: `phalcom-semantic/src/checker/statement.rs`
- Modify: `phalcom-semantic/src/checker/call.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs`
- Preserve: `expression_engine::test_for_loop_protocol_custom_iterable`

- [ ] Inspect current protocol calls:

```bash
rg -n \
  'resolve_iteration_element|iteratorValue|iterate' \
  phalcom-semantic/src/checker/statement.rs
```

Classify each resolved selector as:
- zero-argument protocol getter;
- parameterized call;
- structural applied-type projection.

- [ ] Keep existing zero-argument custom iterable behavior:

```phalcom
class MyCustomStream {
  iteratorValue -> String {
    "item"
  }
}
```

- [ ] For zero-argument resolved protocol getter, construct an exact application target and invoke `apply_resolved_callable` with no arguments rather than reading the signature return directly.

- [ ] For a parameterized protocol selector when loop analysis does not model the required argument expression, fail closed:

```rust
TypeKnowledge::Unknown(
    UnknownReason::UncheckedExpression,
)
```

or return a `Blocked` application product if `resolve_iteration_element` is upgraded to return `CallCheckResult`.

- [ ] Do not fabricate a dummy protocol argument.

- [ ] Do not consume the parameterized callable return contract without application.

- [ ] Add a unit test around `resolve_iteration_element` if source syntax cannot naturally force the parameterized path.

- [ ] Verify existing custom iterable:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::expression_engine::test_for_loop_protocol_custom_iterable
```

- [ ] Verify canonical call suite:

```bash
cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application

cargo fmt --check
```

- [ ] Commit:

```bash
git add \
  phalcom-semantic/src/checker/call.rs \
  phalcom-semantic/src/checker/statement.rs \
  phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs

git commit -m "fix(semantic): canonicalize iteration callable application"
```

---

# 18. Task 15 — Delete Legacy Paths and Run Repository-Wide Audit

**Files:** audit all `phalcom-semantic/src/checker` files; modify only scoped files needed to eliminate bypasses.

- [ ] Search fixed-return promotion:

```bash
rg -n \
  'promote_exact_return|exact_return_origin' \
  phalcom-semantic/src
```

Expected: no production `expression.rs` use.

- [ ] Search direct signature return publication:

```bash
rg -n \
  'sig\.return_type|signature\.return_type' \
  phalcom-semantic/src/checker
```

Allowed:
- `call.rs` fixed/generic return derivation;
- dispatch specialization;
- callable declaration/body verification.

Forbidden:
- expression syntax creates final result from signature return;
- statement protocol helper consumes callable return without application.

- [ ] Search spread-shape approximation:

```bash
rg -n \
  'PackItem::Expand.*SelectorSlot::Positional|Expand.*slots\.push' \
  phalcom-semantic/src/checker
```

Expected: no semantic static-selector approximation.

- [ ] Search result-only List/Map indexing:

```bash
rg -n \
  'arguments\[0\]|arguments\[1\]' \
  phalcom-semantic/src/checker/expression.rs \
  phalcom-semantic/src/checker/call.rs
```

Allowed only in structural target construction or unrelated type decomposition.

- [ ] Search ad hoc setter/index relations:

```bash
rg -n \
  'parameters\.first\(\)|AssignmentMismatch|value assigned to List index' \
  phalcom-semantic/src/checker/expression.rs
```

Expected:
- no resolved setter parameter peeking;
- no List value-only special case;
- no RHS-valued assignment result.

- [ ] Search callable-value laundering:

```bash
rg -n \
  'TypeData::Callable|c\.return_type' \
  phalcom-semantic/src/checker/expression.rs \
  phalcom-semantic/src/checker/call.rs
```

Reject any `Established(..., Flow)` conversion of callable-value return contracts.

- [ ] Search legacy call APIs:

```bash
rg -n \
  'resolve_call\(|match_callable_arguments\(|check_arguments\(' \
  phalcom-semantic
```

Required:
- `resolve_call` deleted or private compatibility wrapper not used by production expression analysis;
- `match_callable_arguments` deleted or test-only projection;
- `check_arguments` deleted or canonical adapter.

- [ ] Search canonical funnel coverage:

```bash
rg -n \
  'apply_resolved_callable\(' \
  phalcom-semantic/src/checker
```

Confirm coverage for:
- explicit method;
- implicit self;
- callable value;
- binary;
- unary;
- getter callable;
- setter callable;
- subscript get;
- subscript set;
- constructor via method dispatch;
- iteration callables in this scope.

- [ ] Search assignment results:

```bash
rg -n \
  'synthesize_set_property|synthesize_set_index_expr|SetPropertyExpr|SetIndexExpr' \
  phalcom-semantic/src/checker/expression.rs
```

Inspect final results. Every source assignment path must project `Unit`.

- [ ] Run focused tests:

```bash
cargo test -p phalcom-semantic --lib

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::canonical_call_application

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::expression_composition

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::bidirectional_calls

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::expression_engine

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::semantic_correctness_regressions

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::causal

cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::explanations
```

- [ ] Run package/workspace:

```bash
cargo fmt --check
cargo test -p phalcom-semantic
cargo test --workspace
```

- [ ] Review diff:

```bash
git diff --stat

git diff -- \
  phalcom-semantic/src/checker/call.rs \
  phalcom-semantic/src/checker/context.rs \
  phalcom-semantic/src/checker/expression.rs \
  phalcom-semantic/src/checker/typed_expr.rs \
  phalcom-semantic/src/checker/statement.rs \
  phalcom-semantic/src/diagnostic.rs \
  phalcom-semantic/src/types/evidence.rs \
  phalcom-semantic/tests/semantic/foundations/mod.rs \
  phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs \
  phalcom-semantic/tests/semantic/foundations/bidirectional_calls.rs
```

Reject unrelated advisory, identity-takeover, transaction, or generic-specialization edits.

- [ ] If audit removes legacy code:

```bash
git add \
  phalcom-semantic/src \
  phalcom-semantic/tests/semantic

git commit -m "chore(semantic): close canonical call application slice"
```

Do not create an empty commit.

---

# 19. Required Regression Matrix

| ID | Scenario | Required product |
|---|---|---|
| CALL-01 | ordinary fixed call, correct arg | fixed return + Ready |
| CALL-02 | ordinary fixed call, wrong arg | fixed return retained + Invalid(C) |
| CALL-03 | assumed receiver fixed call | result at most Assumed |
| CALL-04 | invalid-but-known receiver | call analyzable + causal non-clean |
| CALL-05 | dispatch miss with unresolved arg | argument still analyzed |
| CALL-06 | dynamic spread shape | DynamicRestPack, not one positional |
| CALL-07 | callable local valid | canonical application result |
| CALL-08 | assumed callable local | result at most Assumed |
| CALL-09 | non-callable local | NotCallable + not value-read result |
| CALL-10 | lexical unknown callee shadows self | no implicit-self fallback |
| CALL-11 | binary wrong RHS | ArgumentMismatch + fixed return retained |
| CALL-12 | unary assumed receiver | result authority capped |
| CALL-13 | getter callable | exact callable identity |
| CALL-14 | direct field assignment | Unit |
| CALL-15 | setter assignment | Unit + setter CallableId |
| CALL-16 | setter wrong value | Unit + Invalid(C) |
| CALL-17 | List get wrong index | return known if independent + non-Ready |
| CALL-18 | Map get wrong key | return known if independent + non-Ready |
| CALL-19 | assumed List receiver get | result at most Assumed |
| CALL-20 | List set wrong index | Unit + index relation failure |
| CALL-21 | List set wrong value | Unit + value relation failure |
| CALL-22 | user-defined index set | Unit + CallableId |
| CALL-23 | constructor valid | ConstructorSemantics origin |
| CALL-24 | constructor wrong arg | instance result retained + Invalid(C) |
| CALL-25 | generic call on assumed receiver | generic result <= Assumed |
| CALL-26 | zero-arg iteration protocol getter | existing behavior preserved |
| CALL-27 | unmodeled parameterized iteration protocol | fail closed |

---

# 20. Code-Review Checklist

## Target and dispatch

- [ ] `CallableId` travels explicitly in `CallableApplicationTarget`.
- [ ] Syntax does not re-query dispatch to recover identity.
- [ ] `resolve_dispatch_target` preserves dependency recording.
- [ ] Applied receiver substitution still happens.
- [ ] `Self` specialization still happens.
- [ ] Generic constraints are not accidentally specialized here.

## Arguments

- [ ] Selector shape is derived before value analysis when parameter context is useful.
- [ ] Supplied arguments are analyzed once.
- [ ] Analysis remains source-ordered.
- [ ] Every fixed bound non-generic argument receives one relation.
- [ ] Unmatched supplied arguments still get analyzed.
- [ ] Missing parameters become explicit shape failures.
- [ ] Dynamic expansion is never one positional slot.

## Evidence

- [ ] Assumed receiver/callee cannot produce Established fixed result.
- [ ] Independent fixed result is not weakened merely because an ordinary argument is Assumed.
- [ ] Constructor origin is preserved.
- [ ] Native origin is preserved.
- [ ] Structural fallback origin is explicit.
- [ ] Callable-value conversion does not establish assumed return contracts.

## Status and causality

- [ ] Refuted argument relation makes call non-Ready.
- [ ] Fixed return may remain known.
- [ ] `Invalid(C)` causal state contains C.
- [ ] Known causally-invalid receiver remains analyzable.
- [ ] Unresolved target does not erase child diagnostics.
- [ ] Cancelled/BudgetExceeded/InternalFailure are preserved.

## Assignment

- [ ] Field/property/subscript assignment returns Unit.
- [ ] Callable-backed assignment retains underlying CallableId.
- [ ] Direct field write retains relation terminal status.
- [ ] Setter syntax does not duplicate mismatch diagnostics.
- [ ] Subscript set checks index and put value.

## Structural fallbacks

- [ ] List/Map fallback constructs a complete signature target.
- [ ] Canonical surface dispatch is attempted first.
- [ ] Structural fallback never directly creates final index expression result.
- [ ] No new language feature is introduced via fallback.

---

# 21. Explicit Part 03 Boundary

Do not solve during this plan:

```text
first<T>(1, unresolved)
Unknown generic argument omission
substitution solved != call proven
generic expected-result over-authorization
generic result authority from incomplete argument support
generic where-clause conflict dependency semantics
```

Part 02 success is:

```text
all call-like syntax reaches one generic sub-engine
```

not:

```text
generic sub-engine is fully proof-correct
```

---

# 22. Explicit Part 04 Boundary

Do not solve receiver specialization of generic constraints such as:

```text
Box<T>.method<U> where U <: T
```

`resolve_dispatch_target` preserves current parameter/return substitution behavior. Full `signature.generics` specialization is Part 04.

---

# 23. Expected Final Diff Shape

Create:

```text
phalcom-semantic/tests/semantic/foundations/canonical_call_application.rs
```

Modify:

```text
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/typed_expr.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/diagnostic.rs
phalcom-semantic/tests/semantic/foundations/mod.rs
phalcom-semantic/tests/semantic/foundations/bidirectional_calls.rs
```

Conditional:

```text
phalcom-semantic/src/types/evidence.rs
```

only if provenance-preserving evidence weakening needs a crate-private helper.

A broader diff requires explicit justification.

---

# 24. Completion Gate

Do not mark Technical Specification 02 implemented until every item is true.

- [ ] `apply_resolved_callable` is the single production resolved-call application funnel.
- [ ] Explicit method calls use it.
- [ ] Implicit-self calls use it.
- [ ] Callable-valued locals use it.
- [ ] Binary operators use it.
- [ ] Unary operators use it.
- [ ] Getter callables use it.
- [ ] Setter callables use it.
- [ ] Subscript getter callables use it.
- [ ] Subscript setter callables use it.
- [ ] Constructors reach it through class-side method dispatch.
- [ ] Parameterized iteration callables cannot project returns without application.
- [ ] Exact `CallableId` is explicit target data.
- [ ] Fixed argument mapping is a distinct stage.
- [ ] Fixed non-generic relations are centralized.
- [ ] Every supplied argument is analyzed exactly once.
- [ ] Dispatch miss does not erase child analysis.
- [ ] Expansion/dynamic labels do not masquerade as positional slots.
- [ ] Assumed receiver/callee caps fixed result authority.
- [ ] Assumed callable value cannot establish its return.
- [ ] Independent fixed return survives argument mismatch when semantically independent.
- [ ] Constructor origin remains `ConstructorSemantics`.
- [ ] Native origin remains `NativeSignature`.
- [ ] Field/property/subscript assignment returns Unit.
- [ ] Setter/indexer assignment retains exact underlying CallableId when available.
- [ ] Direct field relation terminal status is retained.
- [ ] List get checks index contract.
- [ ] Map get checks key contract.
- [ ] List set checks index and value.
- [ ] Structural List/Map fallback returns a target, never final result directly.
- [ ] No production `expression.rs` fixed-return promotion remains.
- [ ] No production syntax path uses legacy `resolve_call`.
- [ ] No type-only `match_callable_arguments` projection is used for expression analysis.
- [ ] No resolved setter/index parameter-peeking fast path remains.
- [ ] Canonical call suite passes.
- [ ] Part 01 expression composition suite remains green.
- [ ] Bidirectional call suite remains green.
- [ ] Expression engine suite remains green.
- [ ] Semantic correctness regressions remain green.
- [ ] `cargo fmt --check` passes.
- [ ] `cargo test -p phalcom-semantic` passes.
- [ ] Workspace/CI-equivalent verification is executed.
- [ ] Diff contains no advisory, identity-takeover, transaction, or full generic-proof implementation.

---

# 25. Handoff to Technical Specification 03

Technical Specification 03 begins only after this plan lands.

It consumes:

```rust
CallableApplicationTarget
CallPremise
ApplicationArgument
ArgumentBindingPlan
apply_resolved_callable
apply_generic_callable
```

and repairs generic proof integrity inside this one architecture.

Part 03 must not introduce separate:

```text
generic method engine
generic operator engine
generic subscript engine
```

The architectural success condition of Part 02 is that there is exactly one place left where generic call proof semantics can be wrong.

# Proposed generic Iterable foundation: compiler boundary

Scope: math proposal 13 A1 / 16 / 22 G6. Updated 2026-10-08 after the
architectural response and inline compiler correction.

## Resolved projection progress defect

`InferenceSession::extract_and_normalize_projections` now reports a constraint
rewrite as progress and leaves equality adjudication to ordinary propagation.
Residual obligations survive normalization and are cleared/re-established on
canonical retry. The exact law-level and uncontextualized local-binding helper
regressions failed before the correction and pass afterward.

Associated-equality calls with known bindings and insufficient caller equality
evidence now report `GenericConstraintUnsatisfied`. Contradictory `Item = Int`
evidence is also rejected. The residual remains mandatory; no specialization is
published for the rejected call.

Direct helper forwarding and the supported local/imported/re-exported Family
routes pass semantic tests. Published callee bindings retain rigid caller
parameters and `GenericCallConformanceSource::Forwarded` provenance. A trivial
exported generic identity Family also passes.

The original incident and containment record below describe the starting state.
The remaining incident is recorded after them.

## Required contract

A generic method accepting I with `I |= Iterable` and
`<I as Iterable>::Item == Float` consumes the canonical iterator once. It may
call a generic snapshot helper under the identical constraints. Public Families
retain the generic callable contract across package exports.

## Observed

The following source-owned delegation reports GenericInferenceUnderconstrained
on `FloatReduction.snapshot(input)` despite value-producing typed input evidence:

```text
@class
snapshot<I>(_ input: I) -> List<Float>
  where I |= Iterable, <I as Iterable>::Item == Float
{ ... }

@class
logSumExp<I>(_ input: I) -> Float
  where I |= Iterable, <I as Iterable>::Item == Float
{
  let values = FloatReduction.snapshot(input)
  ...
}
```

Adding `<I>` to the method call produces AnnotationUnresolved for the interpreted
`FloatReduction.snapshot<I>` type path. This explicit-call spelling is not a
supported escape hatch. The draft probability package shows the same helper
failure, and exported generic Family calls report GenericInferenceUnderconstrained.

## Bounded action

Restored the previously tested List<Float> logSumExp entry point, preserving its
implemented behavior. The probability package and its iterator acceptance tests
remain a draft pending the generic decision. No tests are weakened or skipped.
No Dynamic dispatch, consumer-specific compiler patches, or secondary inference
state were introduced.

## Decision requested

Investigate canonical call inference/constraint evidence under the current
specification, or explicitly limit this implementation pass to partial List
entry points while leaving the required Iterable API unimplemented. Any fix
must remain compiler-owned and general; it must not special-case math packages.

# IMPLEMENTATION INCIDENT — remaining integration boundary

## 1. Identity

- Program: math proposal implementation (no numbered program assigned).
- Task/Gate: restore generic reduction and pass probability iterator acceptance.
- Repository: `/Users/altunhasanli/.codex/worktrees/math-proposals/phalcom`.
- Branch: detached HEAD, worktree `math-proposals`.
- Starting/current revision: c4db36bb; uncommitted numerical and semantic work.

## 2. Escalation trigger

The semantic correction passes, but generic integration still fails. Completing
the generic surface now involves contextual collection inference and executable
evidence publication/transport. These cross the supplied narrow patch boundary
and trigger the repository's unexpected-subsystem/cross-layer-contract rules.
No subagent or external adviser was invoked.

## 3. Required invariant

Generic `Iterable<Item = Float>` APIs must accept statically Float inputs,
consume the canonical iterator once, and receive compiler-selected runtime
evidence. Residual equalities cannot be erased, and assumptions cannot be
replaced by Dynamic or runtime conformance search.

## 4. Expected architecture

Argument evidence selects callee generics. Canonical projection normalization
and obligation discharge establish specialization. Compiler lowering projects
the published specialization into executable type/conformance environments.

## 5. Observed behavior

- The generic probability implementation's local helper calls no longer report
  the original error. Its client calls with Float list literals and iterators
  made from those literals still fail associated-equality proof.
- A bounded discriminator using `List<Float>.new()` accepts both the typed List
  and its iterator. Adding `Probe.accept([1.0])` to that same source fails with
  `GenericConstraintUnsatisfied`. The diagnostic snapshot nevertheless contains
  `List<Float>` knowledge for `[1.0]`.
- Temporarily restoring generic `logSumExp` made `math.logSumExp([])` reach
  `snapshot(input)` and fail at `input.iterator` with `no active conformance
  target`. This does not establish a general failure for typed inputs.

## 6. Minimal reproduction

Collection inference discriminator, compiled as a package with `main.ph`:

```phalcom
from universe.collections.iterator_laws import Iterable
class Probe {
  @class accept<I>(_ input: I) -> Float
    where I |= Iterable, <I as Iterable>::Item == Float { 0.0 }
}
let values = List<Float>.new()
values.append(1.0)
let direct = Probe.accept(values)
let iterated = Probe.accept(values.iterator)
// Adding this fails, despite final literal knowledge being List<Float>:
let literal = Probe.accept([1.0])
```

The temporary discriminator was removed after evidence collection; the existing
probability acceptance tests retain the failing literal-based route:

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core audit_probes::math::probability_helpers_normalize_extremes_without_replaying_or_mutating_inputs
```

Runtime probe under the temporary generic reduction signature:

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core audit_probes::math::log_sum_exp_preserves_extreme_values_and_small_corrections
```

It failed on the first expression, `math.logSumExp([])`, before numerical work.
The temporary List compatibility signature has since been restored.

## 7. Relevant code path

```text
checker/expression.rs::synthesize_list_literal
  -> checker/call.rs::apply_generic_callable_in_context
  -> checker/context.rs::solve_inference_in_frame
  -> checker/call.rs::publish_call_specialization
  -> core/modules/compile.rs::project_analyzed
  -> core/compiler/lib/scope.rs::emit_invoke
  -> core/vm/dispatch.rs::invoke_at / invoke_trait_requirement
```

## 8. Scoped implementation diff

Retained changes are in semantic inference, the coordinator comment, narrow
known-binding diagnostic classification, and semantic regression tests.
The generic reduction experiment and VM transport experiment were reverted
with scoped patches. No runtime or collection-inference correction is retained.

## 9. Evidence

- Inference unit tests: 16 passed.
- Inference foundation laws: 61 passed.
- Generic associated constraints: 14 passed.
- Generic conformance: 21 passed on the final completed run.
- Family invocation: 13 passed, including explicit Forwarded-product checks.
- Math audit: 16 passed, 2 probability tests failed; completed in 195.97 seconds.
- During runtime probing, the empty-list client invocation had no attached call
  recipe. The inner helper call had Forwarded slot recipes, but its caller
  provided no usable Iterable witness.
- `try_module_export_send_with_environment` currently accepts only the type
  environment and uses an activation helper that supplies an empty conformance
  environment. That is an additional observed transport seam to audit, not a
  confirmed sole cause of the empty-list failure.

## 10. Implementer's causal hypothesis

Literal analysis inside an active generic inference context reaches a different
projection/substitution lifecycle than an already-canonical typed List argument.
Empty literals additionally lack direct item evidence and may reach execution
without a completed specialization under bootstrap/project analysis paths.

## 11. Alternative hypotheses

Canonical Universe bootstrap deliberately projects despite diagnostics. Missing
specialization at the outer empty-list call, Family call transport, and module
export environment transport may independently contribute to the runtime symptom.

## 12. Attempts already made

The supplied normalization correction fixed the exact semantic reproducer.
A local diagnostic correction fixed insufficient/contradictory caller rejection.
A tentative module-export environment pass-through did not fix the empty-list
runtime failure; it was removed. Temporary logging and discriminator source were
also removed.

## 13. Why local authority is insufficient

Further implementation requires a decision on contextual collection inference,
empty generic Iterable arguments, and fail-closed executable evidence publication.
Changing these contracts was not covered by the supplied architectural correction.

## 14. Decision requested

Authorize a follow-up correction at the canonical collection/call inference
boundary and specify the empty-input inference/evidence contract. Then audit
Family/module-export conformance transport with a typed, nonempty runtime
discriminator. Keep the generic API as the intended final boundary.

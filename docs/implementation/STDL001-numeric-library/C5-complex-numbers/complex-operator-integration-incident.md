# IMPLEMENTATION INCIDENT: conditional generic operators do not dispatch

## 1. Identity

- Program: STDL001
- Checkpoint: STDL001.C5
- Plan: STDL001.C5.P1
- Task/Gate: T1 / core arithmetic RED test
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/complex-math`
- Branch: `feat/complex-math`
- Starting revision: `ad1db231`
- Current revision: working tree, source/test/design changes uncommitted

## 2. Escalation trigger

The approved operation-specific capability architecture crosses the compiler
operator-lowering/runtime evidence boundary. The same semantic failure remains
after one direct source implementation attempt. This is a cross-layer contract
decision, so implementation stops for user approval.

## 3. Required invariant

`Complex<T>` operators must be available only when the component capability is
proven. The implementation must use canonical semantic evidence. It must not
use Dynamic, runtime type probing, or a second capability registry.

## 4. Expected architecture

Conditional inherent implementations on `Complex<T>` contribute `+`, `-`, `*`,
and unary `-` under `where T |= ...`. A statically selected generic method call
should lower through the existing conditional-invocation/evidence transport,
while ordinary dispatch remains the runtime primitive.

## 5. Observed behavior

The following source compiles and constructs `Complex<Int>` successfully, and
data equality works. At execution, ordinary binary operator lowering reports:

```text
unsupported operands for `+`: Complex and Complex (tried `+(_)` then `+(from)`)
unsupported operands for `*`: Complex and Complex (tried `*(_)` then `*(from)`)
```

The same test's `Complex<Float>` equality/signed-zero case passes because it does
not invoke a conditional operator member.

## 6. Minimal reproduction

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core audit_probes::complex -- --nocapture
```

The exact failure is in:

```text
audit_probes::complex::complex_int_arithmetic_preserves_exact_components_and_generic_identity
audit_probes::complex::complex_third_party_components_use_selected_operation_capabilities
```

## 7. Relevant code path

```text
complex.ph: impl<T> Complex<T> where T |= Addable
  -> semantic conditional inherent member product
  -> compiler scope.rs::emit_operator_send
  -> Bytecode::Invoke
  -> VM ordinary bilateral operator dispatch
  -> no unconditional Complex +(_) method
```

Existing conditional named member behavior is handled by the separate
`InvokeConditional` path in `compiler/lib/impl_decl.rs`; operator sends currently
always use ordinary `Invoke`.

## 8. Scoped implementation diff

Added the initial `universe.numbers` package, capability traits, generic
`Complex<T>` data owner, Universe provider registration, and three red runtime
regressions. No compiler workaround or Dynamic fallback was added.

## 9. Evidence

- The pre-feature Float backend regression passed.
- The new package resolves and generic data construction executes.
- Complex equality and signed-zero/hash behavior test passes.
- Complex operator sends fail at ordinary bilateral dispatch.
- `impl_decl.rs` contains conditional method preparation and `InvokeConditional`
  support for semantically selected callables.
- `scope.rs::emit_operator_send` emits `Bytecode::Invoke` unconditionally.

## 10. Implementer's causal hypothesis

The compiler's operator lowering does not consume canonical static receiver/member
products for conditional inherent operators. It therefore never selects or
transports the `Complex<T>` component capability evidence before VM dispatch.

## 11. Alternative hypotheses

- Built-in Int/Float capability conformance declarations may not be accepted as
  evidence for a source-owned conditional operator.
- The conditional implementation product may omit operator members during
  Universe package linking.
- Generic operator methods may require a reflected selector route rather than
  the ordinary conditional invocation mechanism.

## 12. Attempts already made

- Added the source-owned conditional operator implementation and red tests.
  Prediction: the compiler/runtime would select the proven capability member.
  Result: construction succeeds, ordinary operator dispatch misses the member.

## 13. Why local authority is insufficient

The choice changes compiler lowering and the contract between semantic generic
evidence and VM operator dispatch. A local library patch cannot preserve the
approved capability boundary without choosing how operator expressions consume
conditional callable products.

## 14. Decision requested

Which direction should govern T1?

1. **Extend operator lowering (recommended):** when canonical semantic products
   prove a conditional operator member, emit the existing `InvokeConditional`
   form with the selected evidence. This preserves capability-dependent
   availability and makes operator syntax consistent with named conditional
   members. It is a compiler-layer feature and will need semantic/compiler
   regressions.
2. **Use named capability methods for generic Complex:** expose `add`, `subtract`,
   `multiply`, and `negate` through conditional members, while defining operators
   only for a later concrete Float specialization. Smaller compiler scope, but
   the approved generic operator ergonomics are deferred and the public API is
   less mathematical.
3. **Install unconditional Complex operators:** let operator bodies send to
   component values dynamically and treat capabilities as documentation. This
    avoids compiler work but weakens the approved invariant and is not recommended.

## Decision outcome

The user approved option 1: extend operator lowering using canonical conditional
callable/evidence products and the existing InvokeConditional mechanism.
The existing bilateral preference, decline/fallback, and once-only operand
evaluation must remain intact. Operator bodies also consume existing trait-slot
products for component operations. No Dynamic fallback is authorized.

## Component Self specialization diagnosis

Observed: explicit component reads retain rigid impl T and publish an assumed
operator slot, but its result is `SelfType(ClosedAdd)` instead of T. Constructing
Cell<T> from that result raises GenericInferenceConflict. Implicit component
reads hid this by producing unavailable knowledge, so the source now uses
explicit `self.real`/`self.imag`.

Cause: `specialize_self_type`'s InstanceType branch handles nominal/applied
receivers but does not substitute a rigid Parameter receiver. Assumed trait
dispatch therefore needs to bind the existing TypeEnvironment Self slot to the
proven instance subject, as exact trait dispatch already does.

Prediction: instance requirement parameters/results specialize to the same
rigid T, without inferring or strengthening evidence. The semantic identity
discriminator checks exact subject/result equality, then the compiler test
checks runtime trait-slot execution. Class-side specialization is preserved.

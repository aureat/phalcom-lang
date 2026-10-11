# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: LANG007
- Checkpoint: LANG007.C4
- Plan: algebraic-literals-follow-on
- Task/Gate: T5 assumed generic class-side realization receiver
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `feat/complex-math`
- Starting revision: `fdbc2c41e807e2af9b80142c1a4ba91bea3a61bf`
- Current revision: working tree at the starting revision with scoped, uncommitted algebraic-literal changes

## 2. Escalation trigger

Trigger 15 and trigger 11 from the shared consultation protocol: generic
reification is nonlocal and uncertain, and two plausible runtime representations
have different architectural consequences.

## 3. Required invariant

Preserve the plan's canonical target type and the caller's exact runtime type and
conformance environments. The compiler must consume the semantic `Assumed`
selection and must not perform conformance discovery or infer the target from
source spelling. Class-side dispatch must receive the class object corresponding
to the exact generic target type.

## 4. Expected architecture

The follow-on plan requires runtime class-object projection for assumed generic
receivers, preserving the caller's canonical type/conformance environment. The
existing class-side trait decision keeps ordinary nominal conformance evidence;
it does not introduce metatype conformance.

## 5. Observed behavior

`project_contextual_realization` accepts only `RealizationCapabilitySelection::Evidenced`
and returns `AssumedContextualRealizationReceiver` for an assumed generic
selection. Direct evidence already projects through the selected trait call.

The compiler's class receiver helper emits canonical declaration imports or
Universe bindings. It has no path for a generic type parameter. Runtime type
environments map stable type parameters to `RuntimeTypeRef`; runtime conformance
slots carry a method handle and type/conformance environment IDs, but no class
receiver. `InvokeTraitRequirement` expects a receiver value on the stack.

No generic source acceptance probe was run in this continuation.

## 6. Minimal reproduction

No source reproduction was run. The projection boundary is directly visible in
`phalcom-core/src/modules/semantic_lowering.rs::project_contextual_realization`:
an assumed capability is rejected before a lowering specification is created.

Expected plan behavior: contextual basis/suffix realization inside a generic
function uses the exact assumed class-side capability and the runtime class
object for its type parameter.

## 7. Relevant code path

```text
phalcom-semantic/src/checker/algebraic_realization.rs::apply_contextual_realization
  -> publishes RealizationCapabilitySelection::Assumed
phalcom-core/src/modules/semantic_lowering.rs::project_contextual_realization
  -> rejects AssumedContextualRealizationReceiver
phalcom-core/src/typing/environment.rs::RuntimeTypeEnvironment
  -> stores StableTypeParameterRef -> RuntimeTypeRef
phalcom-core/src/typing/conformance_environment.rs::RuntimeConformanceSlot
  -> stores method + type/conformance environment IDs
phalcom-core/src/vm/dispatch.rs::invoke_trait_requirement
  -> consumes an explicit receiver from the operand stack
```

## 8. Scoped implementation diff

The current partial work adds side-aware class trait requirements and witnesses,
canonical contextual realization products, direct-evidence lowering, and
selected conformance runtime-environment roots. It does not change bytecode or
runtime type/conformance environment representation.

## 9. Evidence

- `project_contextual_realization` pattern-matches only the `Evidenced` selection.
- `RuntimeTypeEnvironment` stores stable type-parameter to `RuntimeTypeRef`
  bindings.
- `RuntimeConformanceSlot` stores a method handle and type/conformance
  environment IDs; it has no receiver field.
- `InvokeTraitRequirement` reads the receiver from the operand stack and invokes
  the slot's method with that receiver.
- `reify_type_form` returns a class object for a directly nominal base type;
  other handles fall through to a typing descriptor.

## 10. Implementer's causal hypothesis

The assumed semantic selection is available, but the runtime representation
does not currently define how a generic type binding becomes the class object
required by class-side dispatch. The plan names the outcome but not the carrier
or projection mechanism.

## 11. Alternative hypotheses

- Lower the target from the current frame's canonical runtime type binding,
  resolving its nominal origin to the existing class object; likely requires a
  semantic lowering recipe and a VM operation/helper for a stable type-parameter
  reference.
- Carry the class receiver with the runtime conformance slot, so forwarded and
  selected generic evidence transports the dispatch receiver beside its method
  and environments.

These choices differ in the owner and lifetime of the receiver and in which
call-entry paths must construct or forward it.

## 12. Attempts already made

No speculative runtime change was attempted. The existing direct-evidence path
remains intact and fail-closed on assumed generic selections.

## 13. Why local authority is insufficient

Choosing a carrier affects generic type reification, runtime conformance
environments, VM dispatch, and GC/rooting. It changes a cross-layer contract
required by later contextual-realization work, so a local compiler-only patch
would be unsafe.

## 14. Decision requested

On 2026-10-10 the user chose to derive `T`'s nominal class object from the
active runtime type environment. The conformance slot remains the owner of the
selected method and its type/conformance environments; it does not duplicate
the receiver. This clarifies the existing follow-on plan and does not require a
new plan.

## 15. Implementation update — 2026-10-10

The agreed path is implemented in the current worktree. Lowering stores stable
type-parameter receiver references in the executable semantic pool. The new
`LoadTypeParameterClass` instruction obtains the exact binding from the active
frame type environment and resolves its nominal class/origin; absent bindings
and structural or erased types fail closed. The subsequent
`InvokeTraitRequirement` continues to consume the existing conformance slot,
whose shape is unchanged.

`RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core` passed, scoped
rustfmt passed, and `git diff --check` passed. No source-level tests were run,
so this runtime path remains `PARTIAL / UNVERIFIED` until focused acceptance
coverage is acquired.

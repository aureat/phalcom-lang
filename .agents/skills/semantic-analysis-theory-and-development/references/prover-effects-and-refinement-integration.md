# Prover, Effects, and Refinement Integration

## 1. The semantic engine feeds proof; it does not manufacture proof

A future static prover can reuse semantic identities, CFG, call graph, type facts, effects, and provenance. It must independently establish proof obligations under explicit assumptions.

Never treat the following as proof:

```text
no counterexample found by testing
finite loop unrolling succeeded
optimizer inferred a constant
LSP shape inference is exact-looking
solver timed out / returned unknown
```

## 2. Proof bridge

A verification pipeline may look like:

```text
resolved semantic body
 -> typed/effect-annotated CFG/HIR
 -> contracts + path conditions
 -> verification conditions
 -> SMT/other solver
 -> proved | disproved(model) | unknown(timeout/unsupported)
```

This skill owns the reliable semantic inputs and their assumptions. `static-prover-development` owns VC generation, heap encoding, solver tactics, model reconstruction, and proof policy.

## 3. Effects as proof boundaries

For call `f()`, a prover needs to know which state may change. An effect summary can include:

```text
reads/writes fields or globals
may throw
may perform dynamic send
may invoke callback parameters
may allocate / interact with native world
may yield/suspend
```

A missing effect summary must be conservative. Treating an unresolved call as pure is unsound.

Frame conditions express what remains unchanged. For a contract:

```text
{P} f {Q}
```

the prover also needs a frame/effect relation to preserve unrelated heap facts across the call.

## 4. Refinements and path conditions

Semantic CFG branches provide control structure. The prover enriches them with logical conditions:

```text
path_true  = path ∧ condition
path_false = path ∧ ¬condition
```

Flow-sensitive type/runtime-shape refinements may help simplify these conditions but only if their soundness is sufficient. Advisory editor facts cannot be axioms.

## 5. Dynamic dispatch

A proof of a message send must cover all possible runtime targets under the proof world's assumptions. Options include:

- closed-world finite target set and prove each target contract;
- declared protocol/type contract guaranteeing common behavior;
- runtime guard separating proved targets from dynamic fallback;
- mark obligation unprovable/unknown across fully dynamic dispatch.

Reflection/method mutation must be prohibited, modeled, or guarded if it can change target set after proof assumptions are established.

## 6. Native/FFI trust

A native contract used by proof is part of the trusted computing base. Record its source/version and test it at runtime where possible. Unspecified Rust/native code is opaque, not automatically trusted because it is memory-safe Rust.

## 7. Concurrency/fibers

Once fibers/futures can suspend and shared mutable state exists, a refinement established before suspension may not hold after resumption if another execution context can mutate the state. The semantic effect model should expose yield/suspend points and shared-state effects so the prover can apply interference rules or restrict proofs.

## 8. Contracts and recursion

`requires`/`ensures`/`invariant` metadata should resolve to semantic declaration IDs and source provenance. Recursive methods require contracts/invariants suitable for induction; interprocedural abstract summaries are not a replacement for recursive proof rules.

## 9. Solver outcomes

Preserve:

```text
Proved
Disproved(counterexample/model)
Unknown(reason)
Unsupported(reason)
Cancelled
```

Do not collapse unknown into success or ordinary type uncertainty.

## 10. Review questions

1. Which semantic facts are trusted premises, and why?
2. Does every call have a sufficiently conservative effect/frame model?
3. Can dynamic dispatch/reflection invalidate target assumptions?
4. Does suspension invalidate refinements?
5. Is native behavior backed by a trusted contract?
6. Are solver unknown/timeouts reported distinctly from proof?

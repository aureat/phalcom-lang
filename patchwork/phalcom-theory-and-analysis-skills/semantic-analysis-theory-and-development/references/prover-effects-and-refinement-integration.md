# Prover, Effects, and Refinement Integration

## Shared body semantics

Static prover should consume the same normalized body/CFG and semantic IDs as type/flow analysis. Otherwise evaluation order/control discrepancies become soundness bugs.

## Fact layering

```text
TypeFact: x : Int
FlowFact: x came from Some payload
ProofFact: x > 0
EffectFact: call may write field f
```

Each fact contributes differently to verification.

## Refinement bridge

A proved proposition can narrow a flow/type alternative at a program point. Keep the proof evidence separate so the base type remains canonical.

## Effect bridge

Prover uses effect summaries to decide which facts survive calls/yields. Optimizer can reuse trusted effects only when soundness tier is sufficient.

## Contract summaries

Callable summary can expose both:

```text
operational effects/return
logical requires/ensures/invariant obligations
```

Do not encode a postcondition only as a return type.

## Unknown proof

`Unknown` does not poison semantic analysis globally. It means proof-specific consumer cannot establish property; LSP can still show type/shape facts and typed-runner may keep runtime check.

## Interprocedural staging

A sensible dependency:

```text
resolve bodies/calls
-> infer sound type/effect facts
-> generate VCs
-> solve proof
```

A proof may then feed refined diagnostics/optimization, but avoid cyclicly assuming its own result during VC generation.

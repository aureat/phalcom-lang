---
name: static-analysis-and-abstract-interpretation
description: Use when designing or implementing Phalcom control/data-flow analyses, abstract domains, fixed-point solvers, flow-sensitive narrowing, reachability, effect analysis, interprocedural summaries, alias/escape reasoning, semantic lints, checker support, optimizer facts, or analyses that must trade precision, soundness, termination, and performance.
compatibility: Designed for Phalcom semantic-analysis, checker, static-prover, lint, optimizer, and LSP work in Rust.
---

# Static Analysis and Abstract Interpretation for Phalcom

Static analysis approximates program behavior without executing every possible run. This skill defines how to build those approximations without confusing "unknown" with "safe," traversal order with semantics, or heuristics with proofs.

**REQUIRED BACKGROUND:** Understand `programming-language-semantics`, `phalcom-semantic-model`, and relevant `type-theory`. Pair current code work with `semantic-analysis-development`.

## Required analysis contract

Before code, define:

```text
concrete property
abstract domain
precision/order relation
join/top/bottom
transfer functions
fixed-point/termination policy
soundness or advisory status
provenance/invalidation/performance budget
```

## Non-negotiable principles

- Define the concrete question first; "infer more" is not a specification.
- Branch merge uses the abstract domain's join.
- Unknown/top and unreachable/bottom are different.
- Loops and recursion require fixed points or a conservative designed cutoff.
- State whether the analysis is may/must and sound/advisory.
- Facts belong to program points/edges when control flow changes them.
- Unknown calls, reflection, FFI and fibers need effects/havoc or trusted summaries.
- Interprocedural analysis uses summaries/dependency edges rather than recursive AST expansion.
- Precision upgrades (path/context/heap/relational sensitivity) need a cost justification.
- Every widening/loss of precision is explicit and testable.
- Cached facts need dependency-driven invalidation and generation coherence.
- Editor heuristics are not optimizer/prover evidence unless separately validated.

## Development workflow

1. State exact query and consumer.
2. State may/must and sound/advisory contract.
3. Choose CFG/structured-flow representation.
4. Define domain/order/join/top/bottom.
5. Define transfer functions and edge refinements.
6. Define call/effect/heap model.
7. Define loop/recursion convergence.
8. Define provenance/uncertainty reasons.
9. Define dependency/invalidation keys.
10. Test domain laws, control flow, recursion and incremental equivalence.
11. Measure time, memory and rebuild frontier.

## Reference map

- Lattices/fixed points: [references/orders-lattices-and-fixed-points.md](references/orders-lattices-and-fixed-points.md)
- Abstract interpretation: [references/abstract-interpretation-foundations.md](references/abstract-interpretation-foundations.md)
- Dataflow: [references/dataflow-analysis-frameworks.md](references/dataflow-analysis-frameworks.md)
- CFG/SSA: [references/cfg-dominators-and-ssa.md](references/cfg-dominators-and-ssa.md)
- Transfer/state: [references/transfer-functions-and-state-modeling.md](references/transfer-functions-and-state-modeling.md)
- Widening: [references/widening-narrowing-and-termination.md](references/widening-narrowing-and-termination.md)
- Path refinement: [references/path-sensitivity-and-refinement.md](references/path-sensitivity-and-refinement.md)
- Interprocedural: [references/interprocedural-analysis-and-call-graphs.md](references/interprocedural-analysis-and-call-graphs.md)
- Heap/alias: [references/heap-alias-and-escape-analysis.md](references/heap-alias-and-escape-analysis.md)
- Effects/fibers: [references/effects-closures-and-concurrency-analysis.md](references/effects-closures-and-concurrency-analysis.md)
- Domain examples: [references/domain-design-examples.md](references/domain-design-examples.md)
- Dynamic/reflection: [references/dynamic-language-and-reflection-analysis.md](references/dynamic-language-and-reflection-analysis.md)
- Incrementality: [references/incremental-and-demand-driven-analysis.md](references/incremental-and-demand-driven-analysis.md)
- Soundness/cost: [references/soundness-precision-and-cost.md](references/soundness-precision-and-cost.md)
- Testing: [references/testing-static-analyses.md](references/testing-static-analyses.md)
- Phalcom map: [references/phalcom-analysis-domains.md](references/phalcom-analysis-domains.md)
- Precedents: [references/comparative-analysis-and-reading.md](references/comparative-analysis-and-reading.md)
- Review tests: [references/review-and-validation-scenarios.md](references/review-and-validation-scenarios.md)

## Result discipline

Do not let `Option<T>` erase important reasons. When relevant, distinguish exact/approximate, unknown reason, unreachable, contradiction/error, unsupported feature, and dynamic boundary.

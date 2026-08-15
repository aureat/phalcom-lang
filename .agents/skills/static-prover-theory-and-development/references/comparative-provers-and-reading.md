# Comparative Provers and Reading

## Theory

- Hoare: axiomatic basis for program correctness.
- Dijkstra: weakest preconditions and guarded commands.
- Floyd: inductive assertions for flowcharts.
- Cousot & Cousot: abstract interpretation.
- Clarke/Grumberg/Peled for model checking if temporal/state systems arise.
- de Moura/Bjørner and SMT literature for Z3-style solving.

## Systems to study

### Dafny / Boogie

Excellent model of contracts -> intermediate verification language -> VCs -> SMT. Study framing, loop invariants, diagnostics and trigger/quantifier pain.

### Why3 / WhyML

Verification-condition generation with multiple provers and rich logic transformation.

### SPARK/Ada

Industrial modular proof, contracts, flow analysis and soundness-focused diagnostics.

### Frama-C / ACSL

C contracts, abstract interpretation, weakest-precondition plugin and explicit memory-model challenges.

### Liquid Haskell / refinement typing

Shows how SMT refinements can integrate with types when logic is carefully restricted.

### Rust verification projects (Prusti, Creusot, Kani, Verus)

Study MIR/borrow-aware verification and the value of lowering before proof. Do not inherit Rust ownership assumptions into Phalcom.

### Java tools (OpenJML, KeY)

Contracts over OO heaps, exceptions, dynamic dispatch and frame conditions.

### Python-oriented symbolic tools

Useful for dynamic-language modeling and limitations, but many are testing-oriented rather than sound whole-program provers.

## Practical lesson

Mature verifiers rely heavily on:

```text
contracts
modular summaries
loop invariants
restricted logic
explicit unknown/timeout
careful heap/effect models
```

They do not "automatically prove arbitrary code" by brute-force SMT.

---

## Comparative study matrix

Use precedent by problem, not prestige.

| System/family | Study for | Assumptions to inspect before borrowing |
|---|---|---|
| Dafny/Boogie | contract lowering, IVL, VCs, framing, SMT diagnostics | more static/closed semantics than Phalcom; explicit verification-oriented language |
| Why3/WhyML | proof task transformations, multi-prover architecture | ML-like typed semantics; different dynamic/reflection surface |
| SPARK | industrial contracts, flow/proof layering, explicit unknowns | restrictive Ada subset and strong static discipline |
| Frama-C | memory models, WP + abstract interpretation cooperation | C pointer/UB model unlike Phalcom object semantics |
| OpenJML/KeY | OO heap, Java exceptions/dynamic dispatch, contracts | Java's class-loading/reflection and static typing differ |
| Liquid Haskell | SMT-backed refinements and decidable fragments | refinement typing is not automatically Phalcom's intended type system |
| Prusti/Creusot/Verus | Rust/MIR lowering, contracts, ownership-enabled framing | Rust ownership is not a Phalcom language invariant |
| Kani/model checking tools | bounded/exhaustive state exploration, counterexamples | bounded/model-checking guarantees differ from modular deductive proof |
| Rosette/symbolic systems | symbolic execution and solver-aided language design | often rely on restricted host-language semantics |

### What to extract from each paper/tool

For any external system, record:

1. semantic fragment verified;
2. trusted computing base;
3. contract/annotation burden;
4. heap/effect model;
5. dynamic dispatch/open-world assumptions;
6. loop/recursion treatment;
7. solver theories and unknown policy;
8. incremental/IDE architecture if any;
9. diagnostic model reconstruction;
10. which assumptions do not transfer to Phalcom.

The goal is a Phalcom prover, not a renamed Boogie/Dafny architecture.

# Recursion and Procedure Summaries

## Modular verification

Verify callers against callee contracts rather than inlining bodies. This naturally handles recursion when contracts are strong enough.

For recursive method `f`:

- assume `f`'s contract for recursive calls;
- verify body establishes the same contract;
- this is an induction principle over calls under appropriate termination/partial-correctness assumptions.

## Mutual recursion

Verify an SCC of mutually recursive callables using declared candidate contracts for calls within SCC, then prove each body against its contract.

## Inferred summaries

If no explicit contract exists, abstract interpretation can infer conservative effect/return summaries. Those summaries may support checking but are not automatically logical postconditions strong enough for proof.

## Termination of recursion

Total-correctness proof needs a decreasing measure across recursive calls. Partial correctness can omit it.

## Recursive data

Proofs over recursive ADTs may need induction. SMT can reason about finite datatypes but automatic inductive proofs are limited. Keep first-version prover scope modest.

## Dynamic dispatch recursion

A virtual send may target overrides with distinct contracts. Static proof needs a contract guaranteed by the receiver type/protocol, not the body of one guessed target.

## `Self`

Recursive fluent APIs may state postconditions involving `Self`. Substitute according to typed-language rules, not runtime class guesses.

## Summary trust

A declared but unverified `ensures` can be:

- rejected as assumption in strict mode;
- treated as runtime-checked contract in typed-runner mode;
- trusted only for core/native code signed/marked by system policy.

Make mode explicit.

---

## Deep treatment: recursive proof as modular induction

### Recursive contract rule

For recursive callable `f` with candidate contract `C_f`, verify its body under an environment in which recursive *calls* to `f` may use `C_f`:

```text
Assume C_f for recursive call edges
Prove body of f establishes C_f for current invocation
```

This is an induction principle only when the chosen correctness semantics justify it. For partial correctness, it establishes that any terminating recursive execution satisfies the postcondition. Total correctness additionally needs termination.

### Mutual recursion SCC

For SCC `{f,g,h}`:

1. collect candidate contracts for all members;
2. within the SCC, calls may assume those candidates;
3. verify each body against its own candidate;
4. accept the SCC only if every required obligation succeeds.

A failed member invalidates use of the SCC as a verified summary.

### Summary fixed points versus logical contracts

Interprocedural abstract analysis can compute return/effect summaries by fixed point even without user contracts. Those summaries are sound approximations if the analysis is sound, but they may be too weak or not logically expressive enough to serve as `ensures`. Keep distinct:

```text
AnalysisSummary
VerifiedContract
TrustedContract
```

### Termination measures

For recursive call `f(args')` from `f(args)`, prove a well-founded decrease:

```text
V(args') < V(args)
```

Lexicographic tuples can support multi-argument recursion. Dynamic dispatch complicates this because the actual recursive target may vary; termination contract must hold across all targets/cycles.

### Higher-order recursion

A method accepting blocks/callables can recurse indirectly through callbacks. Call-graph SCCs based only on named direct calls may miss this. Use conservative higher-order call summaries or treat unknown callback recursion as a boundary.

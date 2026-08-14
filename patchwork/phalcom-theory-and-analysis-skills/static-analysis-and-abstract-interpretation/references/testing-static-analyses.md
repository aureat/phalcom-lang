# Testing Static Analyses

## Unit-test the algebra

Property tests for domain laws:

```text
join idempotent
join commutative
join associative
bottom identity
widening upper-bounds inputs
normalization deterministic
```

## Transfer tests

Test one statement/expression at a time with hand-built input state and expected output.

## CFG tests

Cover:

- diamond branch;
- terminating branch;
- nested loops;
- break/continue;
- throw/return;
- unreachable code;
- exceptional edge;
- block capture/invocation.

## Interprocedural tests

Cover:

- simple call;
- polymorphic receiver targets;
- recursion;
- mutual recursion;
- changed callee invalidates caller;
- dynamic call conservative fallback;
- higher-order block effects.

## Metamorphic tests

Program transformations that should preserve analysis answer:

- rename a local consistently;
- add unreachable dead branch;
- reorder independent declarations where semantics permits;
- format source;
- replace syntactic sugar by canonical desugaring.

Transformations that should predictably weaken facts also make good tests.

## Differential tests

Compare analyzer predictions with runtime observations across generated programs. Runtime samples cannot prove soundness, but discrepancies reveal bugs.

## Fuzzing

Generate/reduce ASTs stressing:

- scopes/shadowing;
- selector shapes;
- nested control flow;
- packs/collections;
- closures/non-local returns;
- module graphs.

Assert no panic, deterministic result, fixed-point termination and domain invariants.

## Incremental tests

Edit/remove/re-add source and compare incremental snapshot result against clean full rebuild. They should be semantically identical.

## Performance regression tests

Track fixed-point iterations, rebuild frontier and allocations on representative projects, not only microbenchmarks.

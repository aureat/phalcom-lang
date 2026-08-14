# Transfer Functions and State Modeling

## Transfer function purpose

A transfer function maps abstract pre-state to post-state:

```text
F_stmt : A -> A
```

or produces abrupt successors/events as well.

## Assignment

For lexical assignment:

```text
x = e
```

1. abstract-evaluate `e` in input state;
2. update the specific `BindingId`;
3. preserve/update provenance;
4. invalidate relational facts involving old `x` as necessary.

## Calls

Call transfer depends on resolved/possible target summaries:

- return abstract value;
- field/global mutations;
- captured writes;
- may throw/yield;
- dynamic/reflection havoc.

Unknown calls require a conservative policy based on what dynamic code can mutate.

## Conditions

A condition can generate edge refinements only when the analyzer trusts its semantics.

Examples:

- pattern match on sealed ADT;
- runtime class test;
- known Option predicate;
- numeric comparison in an interval domain.

A random user method named `isPositive` should not refine unless its contract is trusted/proven.

## Havoc

When an operation may change a state component unpredictably, `havoc` forgets affected facts rather than keeping stale precision.

Examples:

- reflective method mutation invalidates member-surface assumptions;
- unknown FFI call may mutate passed buffers/objects;
- dynamic global operation may invalidate module/global facts.

Havoc scope should be as narrow as soundness allows.

## Strong versus weak update

Strong update replaces old abstract value when analysis knows exactly one storage location.

Weak update joins with previous value when abstract address may represent several concrete locations.

Local `BindingId` assignment can often be strong; heap alias sets often need weak update.

## Provenance

Transfer should retain why a fact changed:

```text
initializer
assignment
branch test
call summary
native contract
widening
```

A widened fact should not be presented to users as exact syntax knowledge.

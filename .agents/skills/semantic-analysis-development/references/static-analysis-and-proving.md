# Static Analysis and Proving Development

## Analysis toolbox

Know which tool matches the question.

| Question | Typical technique |
|---|---|
| reachable? | CFG reachability |
| definitely assigned? | forward must dataflow |
| possible values/classes? | forward may abstract interpretation |
| used after definition? | def-use/liveness |
| constant? | constant propagation/SCCP |
| numeric bound? | interval domain |
| Option/variant narrowed? | flow refinement |
| call effect? | interprocedural effect summary |
| contract logically follows? | path predicates + abstract interpretation / SMT |
| exhaustiveness? | pattern usefulness/coverage algorithm |

Do not use a type solver to solve every dataflow problem.

## Monotone framework

A classic dataflow analysis needs:

```text
CFG direction (forward/backward)
domain/lattice
boundary condition
merge operator
transfer function
fixed-point algorithm
```

Write these in the design/spec/test before implementation.

## Definite assignment

Domain per binding can be:

```text
Unassigned
Assigned
MaybeAssigned
```

Merge across branches is must-like. Reads in `Unassigned`/forbidden `MaybeAssigned` produce a
diagnostic according to language rule.

Field definite initialization is more complex: constructor paths, superclass initialization,
delegating constructors, early `self` escape and reflective writes may matter.

## Reachability

An explicit bottom/unreachable state improves:

- dead-code linting;
- return completeness;
- type bottom propagation;
- proof path pruning.

Do not let unreachable assignments broaden reachable facts.

## Constant propagation

Domain example:

```text
Bottom(unreachable)
Constant(v)
Overdefined/Unknown
```

Sparse conditional constant propagation (SCCP) can combine reachability and constants once an
explicit CFG exists.

## Interval analysis

For numeric proofs:

```text
x in [a, b]
```

Transfer arithmetic conservatively; widen loops. Useful for bounds/preconditions without SMT.

Be careful with Phalcom numeric tower, overflow semantics and dynamic operator dispatch. Only use
native arithmetic laws when the type/operation is known to obey them.

## Option/Result analysis

Phalcom's direction toward explicit `Option`/`Result` handling makes these high-value analyses.

Potential facts:

```text
Option<T> state = Some | None | Maybe
Result<T,E> state = Ok | Err | Maybe
```

Pattern/case predicates can refine. Do not assume arbitrary user-overridable `isSome` methods are
logical unless core semantics seals/protects the predicate.

## Contracts

For a call:

```text
callee @requires P(params)
caller state S
obligation: S => P(actuals)
```

For return:

```text
callee body path condition S
@ensures Q(params, result)
obligation: S => Q(...)
```

Class invariant obligations need defined entry/exit points and mutation rules.

## Weakest preconditions

For straight-line pure-ish code, weakest-precondition reasoning can transform postconditions
backward through statements. It becomes more complex with dynamic dispatch, mutation, exceptions,
callbacks and loops.

Use only on a well-defined lowered semantic representation.

## Loops

Proofs need invariants. Sources:

- user-supplied invariant attributes;
- automatically inferred simple intervals/variants;
- conservative unknown.

Do not unroll arbitrary loops indefinitely.

## SMT boundary

If an SMT backend is introduced:

### Encode only trusted semantics

Do not translate dynamic method `+` into integer addition unless analysis proves it is the native
numeric operation with corresponding semantics.

### Keep unsupported expressions explicit

Return `Unknown(Unsupported)` rather than making uninterpreted assumptions that accidentally prove
false contracts.

### Timeouts

Solver timeout -> `Unknown(Timeout)`, not pass/fail.

### Counterexamples

When solver finds refutation, map model values back to source-level explanation where practical.

### Cache

Proof query cache keys must include normalized obligation + relevant semantic/type generation.

## Exhaustiveness

For sealed ADTs/patterns, use a dedicated usefulness/coverage algorithm rather than general SMT.
Guards generally do not provide unconditional coverage unless statically true.

## Effects and proofs

Unknown mutation/dynamic calls can invalidate path predicates. A prover needs effect summaries to
know whether facts survive calls.

This is why effect analysis belongs in shared semantics.

## Testing

Every proof analysis needs:

```text
provable true
provable false with evidence
unknown due to dynamic/unsupported
branch merge
loop
call summary
mutation invalidation
malformed code
solver timeout if applicable
```

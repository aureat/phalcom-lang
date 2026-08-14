# Widening, Narrowing, and Termination

## Infinite chains

Interval analysis of:

```text
x = 0
while ...:
  x = x + 1
```

can produce:

```text
[0,0] -> [0,1] -> [0,2] -> ...
```

No finite fixed point is reached by naive iteration.

## Widening

A widening operator `▽` accelerates ascending chains by jumping to a coarser state:

```text
[0,0] ▽ [0,1] = [0,+∞]
```

Widening sacrifices precision to guarantee termination under appropriate conditions.

## Narrowing

After reaching a widened post-fixpoint, a few narrowing iterations can recover precision.

## Union caps

The current LSP shape domain uses bounded unions and widens to `Unknown` beyond a cap. This is a valid advisory-analysis engineering policy.

A correctness checker cannot simply copy that cap if widening to `Unknown` would cause it to accept an unsafe operation. It may need:

- explicit union type;
- conservative rejection/dynamic boundary;
- more precise representation;
- source annotation.

## Iteration caps

A raw "stop after 10 iterations" is acceptable only if the fallback is conservative and the result records loss of precision. It is not a proof of convergence.

## SCCs

For recursive call graphs, solve strongly connected components together. Acyclic callers can be processed in dependency order.

## Monotonic summaries

Design summary updates to move monotonically toward less/more precise states according to solver convention. Oscillation often means transfer/join is not monotone or canonicalization is unstable.

## Canonical equality

Fixed-point termination checks need semantic equality, not pointer allocation identity. Normalize ordering and union representation before comparing.

## Diagnostics

Internal tracing should reveal:

```text
iteration number
old state
new state
widening applied
reason/cycle
```

This makes convergence bugs debuggable.

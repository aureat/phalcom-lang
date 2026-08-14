# Path Sensitivity and Refinement

## Path-insensitive merge

A basic analysis joins at every control-flow merge. It scales but loses correlations.

Example:

```text
if cond:
  x = 1
  y = 2
else:
  x = "a"
  y = "b"
```

After merge, independent unions lose the fact that numeric `x` correlates with numeric `y`.

## Path-sensitive analysis

Retain separate states keyed by path condition/partition. This improves precision but can grow exponentially.

Bound it by:

- selected predicates only;
- maximum partitions;
- merge heuristics;
- demand-driven splitting;
- trace partitioning.

## Flow typing

For a value:

```text
x : Option<String>
```

a trusted branch test can refine:

```text
true edge -> Some<String>
false edge -> None
```

After merge, recover original union/Option type unless one branch terminated.

## Terminating branch precision

```text
if x is None:
  return
# here x is Some
```

Because the false/normal path is the only continuing path, refinement survives after the conditional. CFG reachability handles this naturally.

## Relational predicates

Comparisons can introduce constraints:

```text
x < 10
x == y
```

A simple type/shape domain may ignore these. A static prover/interval/relational domain can retain them separately.

## User predicates

Do not infer refinements from method names. Refinement-capable predicates need:

- built-in trusted semantics;
- declared type guard/refinement contract;
- proven `ensures` relation.

## Contradictions

A path condition proved inconsistent becomes unreachable/bottom. An unsupported/unknown condition does not.

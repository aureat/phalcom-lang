# Abstract Knowledge, Dataflow, Joins, and Fixed Points

## Abstract interpretation mental model

Semantic analysis usually cannot execute the program exactly. It executes an abstract
version of the program over a finite/controlled domain of facts.

Concrete execution:

```text
program state -> exact runtime values -> next exact state
```

Abstract execution:

```text
abstract state -> possible value/type/effect facts -> next abstract state
```

Correctness usually requires the abstract result to conservatively cover the concrete
possibilities relevant to the analysis.

## Precision order

Define a relation "is at least as precise as". Example for runtime shape knowledge:

```text
Instance(Point) <= Union(Point, Circle) <= Unknown
```

The exact ordering depends on the domain. Write it down before implementing joins.

## Join

At a control-flow merge, join facts from reachable predecessor paths.

Example:

```phalcom
let x
if cond {
  x = Point.new()
} else {
  x = Circle.new()
}
use(x)
```

Post-merge shape should be `Point | Circle` (bounded according to policy), not whichever
branch happened to be analyzed last.

## Structural joins

Useful structural policies:

- same nominal shape -> unchanged;
- list/set/range -> join element shape;
- map -> join key and value independently;
- tuple -> elementwise join when arity matches, otherwise union/widen;
- record -> fieldwise join only when label structure is compatible;
- callable -> exact identity only when the same callable; otherwise union/unknown as domain permits;
- family -> retain base selector only when equal, join receiver shape;
- incompatible forms -> bounded union;
- union above cap -> `Unknown`/widened top.

The current LSP follows many of these policies already.

## Why union caps exist

Without widening, this loop or recursive call pattern can grow without bound:

```text
T0
T0 | T1
T0 | T1 | T2
...
```

A bounded union is a pragmatic finite-height approximation for editor inference.
A future checker type lattice may use different canonicalization and cannot blindly reuse
the LSP union cap.

## Bottom and unreachable

A complete dataflow domain often benefits from explicit bottom:

```text
Bottom = no reachable concrete states
Top    = reachable but no useful information
```

Current `ValueShape::Unknown` acts like top for shape knowledge. Do not treat it as bottom.

When future analyses need unreachable-state precision, model reachability separately or
introduce bottom explicitly.

## Transfer functions

Every statement/expression has a transfer function over abstract state.

Examples:

```text
x = expr:
  value = analyze(expr, state)
  state[x] = value

return expr:
  emit return evidence
  terminate normal path

if condition:
  optionally refine true/false input states
  analyze branches
  join reachable normal exits
```

Transfer functions should be deterministic and monotone with respect to the chosen domain.

## Worklist/fixed-point solving

Loops and recursive call graphs require repeated analysis until facts stop changing.

Generic worklist:

```text
seed states
enqueue affected nodes
while worklist not empty:
  node = pop
  out = transfer(node, join(predecessor outs))
  if out changed:
     store out
     enqueue dependents
```

For call graphs, strongly connected components (SCCs) can isolate recursive groups and
reduce repeated global passes.

## Widening

Use widening when the domain can ascend indefinitely or practical latency matters.

Possible widening policies:

- cap union alternatives;
- cap tuple/record depth;
- collapse recursive container shapes to `Unknown` at a depth threshold;
- summarize repeated mutation as a stable top-like fact;
- cap provenance samples.

Widening is a deliberate precision loss. Preserve enough metadata to diagnose why a fact
became imprecise when useful.

## Narrowing/refinement

After a conservative fixed point, some analyses can regain precision using branch facts.
Do not call ordinary type narrowing "widening/narrowing" unless using the abstract-
interpretation terms carefully; they are related but distinct ideas.

## Flow-sensitive versus flow-insensitive

Flow-insensitive:

```text
x has join of every assignment anywhere
```

Flow-sensitive:

```text
x has fact valid at this program point
```

LSP completion and static proving frequently require flow sensitivity. Whole-program
summary facts may intentionally be flow-insensitive.

## Path sensitivity

Full path sensitivity is expensive. Prefer staged precision:

1. structured/CFG flow with joins;
2. trusted local refinements for simple predicates;
3. limited path predicates for contracts/exhaustiveness;
4. symbolic/SMT reasoning only when justified.

Do not build an SMT solver into ordinary completion inference.

## May versus must analyses

Know which one you need.

- "may call dynamically" joins with OR;
- "definitely initialized" joins with AND/intersection across predecessors;
- "possible runtime shapes" joins with union;
- "definitely satisfies predicate" retains only predicates true on all paths.

Using the wrong join polarity makes an analysis unsound.

## Provenance under joins

Keep a bounded representative set of origins. More provenance is not always better: a
fact copied through thousands of calls can explode memory.

Useful strategies:

- retain first few distinct origins;
- collapse to a summary origin after cap;
- store parent evidence IDs in a compact arena only for diagnostic-capable modes;
- keep fast/editor mode cheaper than deep-check mode.

## Fixed-point equality

Convergence depends on semantic equality, not pointer identity. Normalize domains so
logically equivalent facts compare equal:

```text
A | B == B | A
A | A == A
```

The current `ValueShape` ordering/canonicalization should remain deterministic. Future
type unions need stronger canonicalization rules.

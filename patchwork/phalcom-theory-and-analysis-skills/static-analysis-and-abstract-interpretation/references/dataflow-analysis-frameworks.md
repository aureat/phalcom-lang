# Dataflow Analysis Frameworks

## Forward dataflow equations

For CFG block `B`:

```text
IN[B]  = join(OUT[P] for P in predecessors(B))
OUT[B] = transfer_B(IN[B])
```

For backward analysis:

```text
OUT[B] = join(IN[S] for S in successors(B))
IN[B]  = transfer_B(OUT[B])
```

## May versus must

May property: path merge typically uses union/join.

Examples:

- may be class A or B;
- may throw;
- may write field f.

Must property: merge keeps only facts true on every incoming path, often intersection-like.

Examples:

- definitely initialized;
- definitely closed resource;
- definitely Some after condition.

## Gen/kill framework

Classic bitvector analyses use:

```text
OUT = GEN ∪ (IN - KILL)
```

Examples include reaching definitions and liveness variants. Rich Phalcom value/effect analysis usually needs structured transfer functions, but bitvectors remain ideal for sets of IDs.

## Worklist solver

Canonical algorithm:

```text
initialize states
push entry/affected blocks
while worklist not empty:
    b = pop
    new = transfer(join(pred states))
    if new != old:
        store new
        push successors
```

Use stable block IDs and deterministic queue policy in tests.

## Sparse analyses

SSA/def-use graphs can avoid propagating facts through every CFG edge. Sparse conditional constant propagation is a classic example.

Do not jump to SSA merely for fashion; use it when the analyses/optimizer need value-version precision.

## Edge-sensitive transfer

Conditionals refine differently per edge:

```text
if isSome(x)
  true edge:  x = Some(T)
  false edge: x = None
```

Attach refinement to edges before merging successor state.

## Exceptional edges

Calls/operations that may throw have exceptional successors. Ignoring them can produce unsound definite-state/proof results.

## Non-local return and fiber edges

Phalcom control features may require edges beyond ordinary structured local CFG. Model them explicitly or summarize their effect conservatively.

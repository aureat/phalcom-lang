# CFGs, Dominators, and SSA

## Control-flow graph

A CFG contains basic blocks connected by possible control transfers.

Nodes should make explicit:

- branch successors;
- loop back-edges;
- return/throw exits;
- break/continue targets;
- exceptional edges when needed;
- non-local control summaries/targets if modeled.

## Basic blocks

A basic block is a maximal straight-line region with one entry and control transfer at the end. Keeping source range / semantic expression IDs allows diagnostics to map facts back to syntax.

## Dominance

Block `D` dominates `B` if every path from entry to `B` passes through `D`.

Uses:

- definite facts;
- SSA construction;
- redundant check elimination;
- proving that a refinement/assignment holds at a use.

Post-dominance reverses the idea toward exits and is useful for cleanup/control reasoning.

## Dominance frontier

Used to place SSA phi nodes where definitions from different paths merge.

## SSA

Static Single Assignment gives each variable version one definition:

```text
x1 = 1
if cond:
  x2 = 2
x3 = phi(x1, x2)
```

Advantages:

- precise def-use;
- constant propagation;
- easier value facts;
- optimizer infrastructure.

Costs:

- lowering complexity;
- source mapping;
- mutable fields/heap still require memory abstraction;
- closures/upvalues complicate promotion.

## Memory SSA

Heap/global mutation can be versioned conceptually, but full MemorySSA is advanced. Start with effect summaries/havoc sets unless optimizer/prover needs more precision.

## Structured flow versus CFG

Current Phalcom LSP analysis uses structured statement flow. That is appropriate while constructs remain naturally recursive and analyses are modest. Introduce a reusable CFG/semantic IR when multiple analyses need dominance, loops, exceptional edges or common program-point identity.

Do not maintain several incompatible CFGs in checker, prover, linter and optimizer.

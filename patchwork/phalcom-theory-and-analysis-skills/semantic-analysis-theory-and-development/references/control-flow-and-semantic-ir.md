# Control Flow and Semantic IR

## When structured AST flow stops scaling

Move toward reusable body IR/CFG when you need several of:

- dominance/post-dominance;
- definite assignment/liveness;
- loop invariants;
- SSA/value numbering;
- exceptional edges;
- non-local control targets;
- optimizer passes;
- proof VCs;
- shared program-point IDs.

## Body IR properties

Prefer:

```text
explicit basic blocks
explicit terminators
resolved operands/IDs
explicit calls/sends
explicit field access
source map
well-defined control outcomes
```

Avoid embedding LSP protocol objects or runtime heap references.

## Terminators

Possible terminators:

```text
Goto
Branch
Return
Throw
Break/Continue after lowering to targets
Switch/Match
NonLocalReturn
Yield/Suspend depending representation
Unreachable
```

## Expression granularity

Three-address style:

```text
v1 = LoadBinding(x)
v2 = Const(1)
v3 = Send(v1, +, [v2])
StoreBinding(x, v3)
```

is easier for SSA/prover but can be verbose. A hybrid expression tree within blocks may be enough initially.

## Exceptional edges

Calls/sends that may throw need explicit exceptional successor only for analyses that care. An effect summary plus block terminator model can defer full EH CFG until needed.

## Non-local return

Represent target home callable/frame identity explicitly at semantic level. Static analysis can treat escaping target as abrupt edge/effect even if runtime uses frame metadata.

## CFG ownership

One shared body graph should serve type flow, static analysis, prover and semantic lints. Consumer-specific annotations attach to node/block IDs.

## Relation to bytecode

Semantic IR is not bytecode and does not need stack operations. Compiler may lower HIR/body IR to stack bytecode later; VM remains independent.

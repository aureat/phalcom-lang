# Flow and Interprocedural Semantic Analysis

## Local flow

Facts change by program point. Use structured flow or CFG state keyed by `BindingId`/field identity.

Branch merge uses domain join. Terminating branches do not contribute normal continuation.

## Current structured flow

Phalcom LSP already represents normal continuation, returns, breaks, continues, throws, tail values, calls, field writes and block effects. Preserve this semantic richness if/when lowering to CFG.

## Call summaries

A reusable summary can include:

```text
parameters
return runtime shape/type
throws/yields/dynamic-send
field/global read-write effects
invoked block parameters
call dependencies
contracts/proof status
revision
```

Different consumers may attach additional summary domains to one `CallableId`.

## Solver separation

Do not make one giant interprocedural solver that simultaneously mutates shapes, types, effects and proofs with accidental iteration ordering.

Prefer staged or mutually defined queries with explicit dependencies:

```text
surface/call graph
-> shape/effect summaries
-> type obligations
-> proof summaries
```

Where cycles genuinely cross domains, define a joint monotone fixed point deliberately.

## Recursion

SCC-based solving is the default mental model. Annotations/contracts can break inference cycles and improve diagnostics.

## Higher-order blocks

Block summary captures latent effects and parameter/return facts. Callee summary records whether/how it invokes callable parameter. Propagate effects when call semantics justify it.

## Dynamic calls

Unknown target summary must be conservative. Advisory LSP can keep a heuristic return fact separately but checker/prover should see the dynamic boundary.

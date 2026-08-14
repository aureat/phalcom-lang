# Semantic IR and Lowering

Phalcom currently performs substantial live semantic analysis over the recovered AST
with structured flow. Do not introduce a new IR because other compilers have one.
Introduce it only when a class of semantic questions becomes simpler, safer, or more
reusable after lowering.

This document defines the decision boundary and design requirements for that future
step.

## 1. Three different representations

Keep these concepts separate:

1. **Source AST** — preserves source constructs and source ranges.
2. **Semantic IR/CFG** — optional analysis representation with explicit identities and
   control-flow edges.
3. **VM bytecode** — executable stack-machine representation.

A semantic IR is not automatically execution bytecode and should not be distorted to
match stack-machine details that are irrelevant to analysis.

Likewise, bytecode optimizations must not become the only place where source semantic
facts exist if LSP/checker/lints need them before compilation.

## 2. Current structured-flow model

Current semantic flow already makes control results explicit through concepts such as:

- normal continuation;
- returns;
- break/continue;
- throw;
- tail value;
- binding state;
- call and field-write events;
- block effects.

That is enough for many analyses. Preserve it while it remains understandable and
correct.

## 3. Signals that a semantic IR is justified

Consider introducing a semantic CFG/IR when several of these appear:

- many analyses independently recreate branch/loop reachability;
- dominance/postdominance becomes necessary;
- definite assignment becomes difficult to express over nested AST traversal;
- flow-sensitive type refinement needs explicit program points;
- exception edges are repeatedly approximated differently;
- `break`/`continue`/non-local return interactions become fragile;
- dataflow requires repeated rescanning of syntax;
- proof obligations need normalized expressions;
- effects need edge-sensitive propagation;
- optimizer and checker need the same definition-use information;
- source constructs lower to a smaller set of semantic operations;
- control-flow bugs repeatedly arise from traversal order.

Do not use code size alone as the trigger.

## 4. Desired semantic properties

A future IR should make semantic identity explicit.

Possible identifiers:

```rust
BodyId
BlockId
InstrId
ValueId
BindingId
DefinitionId
CallSiteId
```

These are conceptual names, not a mandate for exact Rust types.

The IR should preserve mappings back to source:

```text
source range -> semantic instruction/value/body
semantic instruction -> source range(s)
```

Diagnostics and LSP require this provenance.

## 5. Suggested normalized operations

A minimal semantic IR might represent operations such as:

```text
Const
ReadBinding
WriteBinding
ReadField
WriteField
ConstructCollection
Send
SuperSend
DynamicSend
MakeClosure
CallCallable
Branch
Jump
Return
Throw
Yield
```

Do not lower away semantic distinctions that downstream analyses need. For example,
`SuperSend` should remain distinguishable from an ordinary send even if runtime
bytecode eventually shares machinery.

## 6. Lowering invariants

Lowering must be:

- deterministic;
- source-order preserving where evaluation order is observable;
- explicit about side effects;
- explicit about exceptional/non-local control flow;
- conservative under recovered/malformed syntax;
- independent of LSP request type;
- testable without the VM.

Never synthesize semantic facts that the source cannot justify merely to obtain a
well-formed IR.

## 7. Evaluation order

Phalcom evaluation order is part of language semantics. Lower operands and argument
packs in the exact observable order used by the runtime compiler.

A lowering pass must preserve:

- receiver evaluation before message arguments when specified;
- lexical order of positional/labeled/dynamic pack elements;
- closure construction timing;
- assignment target/value ordering;
- short-circuit semantics;
- collection element evaluation order;
- decorator/attribute effects where semantically relevant.

Differential tests against AST-to-bytecode behavior are valuable here.

## 8. CFG construction

For each callable/module body:

```text
entry -> basic blocks -> exits
```

Edges should distinguish when useful:

- normal;
- true/false branch;
- loop back edge;
- break/continue;
- return;
- throw;
- exceptional edge;
- non-local return;
- future yield/suspend.

A single generic edge kind is simpler initially, but analyses that need exceptional or
suspension semantics must not infer those from syntax after lowering.

## 9. Values: named bindings versus SSA values

Do not confuse lexical binding identity with flow value version.

`BindingId` answers:

> Which source variable declaration is this?

SSA-like `ValueId` would answer:

> Which definition/version reaches this use?

If SSA is introduced, preserve both.

Example:

```phalcom
let x = 1
x = "s"
use(x)
```

One lexical `BindingId` may correspond to multiple flow values.

## 10. Phi/merge semantics

If using SSA, branch merges may require phi-like values.

Conceptually:

```text
then: v1 = Int
else: v2 = String
merge: v3 = phi(v1, v2)
shape(v3) = Int | String
```

The semantic join still belongs to the abstract domain. A phi node does not determine
the type/shape lattice by itself.

## 11. Exceptions and non-local return

These are the fastest way to make a superficially correct CFG unsound.

The IR design must decide:

- which operations can throw;
- whether every send conservatively has an exceptional edge;
- how catch/ensure/finally regions are represented;
- how non-local block returns target their home callable;
- what happens when a block escapes its home activation;
- how future fibers/yields interact with cleanup.

Do not add exception edges selectively based on current stdlib behavior unless the
runtime contract makes those operations non-throwing.

## 12. Closures and captures

Closure construction should identify:

- closure body identity;
- captured binding identities;
- capture read/write capability;
- home callable for non-local return;
- invocation effects when known.

Do not inline a closure body into the constructing path merely because it is syntactic
child AST. Construction is not execution.

## 13. Desugaring policy

Desugar only when doing so preserves diagnostic and semantic distinctions.

Good candidate:

- multiple surface spellings that have exactly one semantic operation.

Poor candidate:

- `super` into a normal receiver send, because lookup origin changes;
- dynamic pack sends into static calls, because selector certainty changes;
- protocol/type metadata into runtime dispatch, because that changes language model.

Retain source-origin metadata for desugared operations.

## 14. Recovery

Editor analysis receives incomplete programs. A semantic IR builder should support an
explicit recovery operation or missing operand rather than panic.

Examples:

```text
UnknownValue
InvalidTarget
UnresolvedSend
RecoveredInstruction
```

These must not be mistaken for normative language values or types.

## 15. Incrementality

If IR bodies are cached, their identity and invalidation should be at least body- or
module-scoped.

Questions to answer:

- Is a body rebuilt whenever its source file revision changes?
- Can unchanged bodies reuse lowering results?
- Which global surface changes force dispatch re-resolution without re-lowering?
- Can type/proof analyses rerun over the same IR after annotations change?

Do not cache cross-generation references to snapshot-local IDs without remapping.

## 16. Relationship to typing

A semantic IR may be an excellent substrate for:

- bidirectional type checking;
- flow refinements;
- definite assignment;
- proof obligations;
- effect checking.

But it must not embed the current `ValueShape` enum as if it were the future type IR.
Prefer references to separate stores/domains:

```text
ValueId -> ValueShapeFact
ValueId -> TypeFact
ValueId -> ProofFact
ValueId -> EffectFact
```

## 17. Relationship to bytecode

The source compiler can continue lowering AST directly to stack bytecode initially.
Later, both compiler and analyzer may share a semantic lowering layer if that improves
correctness.

Do not force this convergence prematurely. The criterion is semantic duplication:
when compiler and analyzer each maintain complex independent logic for the same
observable behavior, a shared lowering deserves consideration.

## 18. Migration strategy

A safe migration is incremental:

1. Define the IR for one callable body.
2. Build it alongside current structured flow.
3. Run one analysis over both paths.
4. Differential-test facts.
5. Move one consumer at a time.
6. Keep source mappings exact.
7. Measure update latency and allocations.
8. Delete old traversal only after parity is demonstrated.

Never rewrite parser, semantic analysis, checker, and bytecode compiler around a new IR
in one step unless a formal migration project explicitly requires it.

## 19. Tests

Required classes of tests:

- straight-line lowering golden tests;
- nested branch/loop CFG tests;
- return/throw/break/continue tests;
- non-local return tests;
- closure capture tests;
- dynamic send/pack tests;
- source range mapping tests;
- malformed/recovery tests;
- deterministic lowering tests;
- differential semantic-fact tests;
- incremental invalidation tests;
- performance benchmarks on large bodies.

## 20. Review questions

Before accepting a semantic IR change:

1. Which current semantic problem does this representation simplify?
2. Why is AST structured flow insufficient?
3. Which distinctions are preserved/lost?
4. Is evaluation order exact?
5. Are exceptional/non-local edges sound?
6. Are lexical identities distinct from value versions?
7. Can diagnostics map back to precise source?
8. What is the recovery representation?
9. What invalidates the IR?
10. Does the design unnecessarily couple analysis to VM bytecode?
11. Can typing/proving use it without being hardwired into it?
12. Is the runtime/compiler behavior covered by differential tests?

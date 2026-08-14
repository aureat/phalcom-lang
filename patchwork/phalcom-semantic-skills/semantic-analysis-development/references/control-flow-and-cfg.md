# Control Flow, Structured Flow, and a Future Semantic CFG

## Current state

Phalcom LSP currently performs structured recursive flow in `semantic/flow.rs` rather than
materializing a general CFG. This is valid while one shared walker can correctly model the
needed analyses.

Do not replace it merely for architectural fashion.

## What structured flow must model

A statement sequence has multiple exits:

```text
normal continuation
return
throw
break
continue
non-local return from invoked block (where semantics permits)
```

The current `StatementFlow` makes several of these explicit. Preserve explicit exits rather
than using booleans like `did_return` that cannot compose.

## Branches

Algorithm:

```text
input state
  -> optional condition refinement
  -> true branch state
  -> false branch state
  -> join reachable normal exits
  -> concatenate/merge abrupt exits by category
```

Do not mutate one state through true then false branch; clone/fork from the same pre-branch
state.

## Loops

Naive one-pass loop analysis misses loop-carried assignments.

When precision matters:

```text
header_state = incoming
repeat:
    body_result = analyze(body, header_state)
    backedge = join(body normal, continues)
    next_header = join(incoming, backedge)
    next_header = widen(header_state, next_header) if needed
until stable
exit = join(condition-false state, breaks)
```

Exact details depend on `while`/`for` semantics.

If current analysis intentionally uses a cheaper approximation, document/tests should capture
its precision limits rather than implying a fixed point.

## `return`

Return should:

- evaluate expression in current state;
- emit `ReturnEvidence` to the correct home callable;
- terminate normal path;
- preserve source range/provenance.

Future type checker uses same reachable returns but checks them against declared result type.

## Tail value

If Phalcom functions/methods have tail-expression result semantics, distinguish reachable tail
value from explicit returns. Unreachable tail expressions must not influence normative return
inference.

## `throw`

A throw terminates normal flow and contributes may-throw effect. Future catch/handler syntax
requires exceptional edges with handler-specific states.

Do not model throw as return.

## `break` / `continue`

Associate with correct loop target. If nested/labeled loops are introduced, structured flow may
need target IDs rather than anonymous vectors.

## Blocks/closures

Constructing a block evaluates/captures according to closure semantics but does not necessarily
run its body.

Analyze block body for a reusable `BlockEffects` summary:

- captured writes;
- non-local returns;
- invoked callable parameters;
- dynamic sends;
- future throw/yield/block/escape effects.

Apply those effects at call sites only when callee semantics says the block is invoked.

## Condition refinements

Future refinement API should look conceptually like:

```text
refine(condition, input) -> (true_state, false_state)
```

Supported predicates must be trusted semantic operations, for example:

- exact equality/inequality to `None` if semantics are stable;
- Option variant predicates/pattern matching;
- sealed ADT cases;
- type/protocol tests once specified;
- numeric comparisons for interval analysis.

Arbitrary user-overridable methods cannot be assumed to encode a logical predicate unless the
language contract says so.

## When to introduce explicit CFG

Strong signals:

- definite assignment;
- unreachable-code diagnostics;
- flow typing across many constructs;
- contract/static proving;
- liveness/escape analysis;
- common analysis framework for lints/checker;
- exception/catch/finally edges;
- complex loop/labeled control flow;
- dominance/post-dominance queries;
- repeated duplicated branch walkers.

## CFG design goals

A future semantic CFG should be analysis-oriented, source-mapped, and independent of VM
bytecode representation.

Possible concepts:

```text
BodyId / CallableId
BasicBlockId
ProgramPoint
Instruction/Terminator
Local/Binding semantic references
SourceRange per operation
Successor edge kind
```

Keep source mapping so diagnostics/LSP can translate facts back.

## Lowering versus AST

Lower syntax sugar before analyses when different syntax has identical semantics. Examples may
include:

- compound assignments;
- loop sugar;
- pattern desugaring;
- implicit returns;
- high-level collection iteration if the language defines an exact lower semantic form.

Do not lower away reflective/selector distinctions that are observable.

## Dominators/post-dominators

Needed for some future analyses:

- a fact established in dominating block may hold downstream absent kills;
- post-dominance helps cleanup/must-execute reasoning;
- loop headers/natural loops become explicit.

Do not compute them for every editor request unless a consumer needs them; cache per body/generation.

## SSA

SSA is optional. It can simplify dataflow/type refinement but introduces phi nodes and requires
mapping back to mutable source bindings.

Phalcom can have a useful CFG without SSA. Choose based on analyses, not compiler fashion.

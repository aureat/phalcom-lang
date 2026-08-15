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

## CFG as a semantic normalization

A control-flow graph should exist only when it removes duplicated semantic reasoning from multiple analyses. Its nodes/edges represent possible control transfer, not source nesting. A basic block has one entry and transfers control only at its terminator.

Conceptually:

```rust
struct Cfg {
    entry: BlockId,
    blocks: IndexVec<BlockId, BasicBlock>,
}

struct BasicBlock {
    ops: Vec<SemOp>,
    term: Terminator,
}

enum Terminator {
    Goto(BlockId),
    Branch { cond: ValueId, then_bb: BlockId, else_bb: BlockId },
    Return(Option<ValueId>),
    Throw(ValueId),
    Unreachable,
}
```

This is a design sketch, not a mandate for exact types. Preserve source ranges/provenance on operations and terminators so diagnostics and LSP queries can map normalized semantics back to syntax.

## Edge kinds matter

As Phalcom gains exceptions, non-local control, patterns, and fibers, a generic successor list can become insufficient. Distinguish edges when analyses need their meaning:

```text
Normal
True / False
LoopBack
Break(target)
Continue(target)
Exceptional(handler/unwind)
NonLocalReturn(target home context)
```

An analysis can ignore distinctions it does not need, but the CFG builder must not erase semantics required by another shared consumer.

## Dominance

Block `d` dominates block `n` if every path from entry to `n` passes through `d`:

```text
d dom n
```

Dominance is useful for:

- establishing where a definition/refinement is guaranteed to have executed;
- SSA construction;
- code motion/optimization preconditions;
- proof/path reasoning;
- determining loop headers via back-edges in reducible CFGs.

Do not use textual containment as a substitute. A statement nested lexically in a branch does not dominate code after a merge.

Post-dominance reverses the property toward exit and helps reason about cleanup/ensures/control completion.

## Reachability is separate from unknown value

Represent unreachable control explicitly. At a block with no reachable predecessors:

```text
reachable[B] = false
```

Do not encode this as every binding having `Unknown`; `Unknown` means executions may reach the point but value information is imprecise. Unreachable means no execution reaches the point under modeled semantics.

This distinction affects diagnostics:

```text
unreachable statement warning
missing return analysis
exhaustiveness
proof obligations
return-shape joins
```

## Condition refinement as edge transfer

For condition `x != None`, branch-sensitive flow is naturally modeled on edges:

```text
OUT_true  = refine(IN, x != None)
OUT_false = refine(IN, x == None)
```

The condition expression itself may also have effects before the branch. Evaluation order is therefore:

```text
state_after_condition_eval
   -> true edge refinement
   -> false edge refinement
```

If evaluating the condition invokes user code, do not reason as though the predicate were a pure symbolic formula unless its semantics guarantee that.

## Loops and fixed points

A loop header receives state from preheader and back-edge:

```text
IN[H] = OUT[preheader] ⊔ OUT[backedge]
```

The body must iterate until the header state stabilizes (or widens). One pass is insufficient whenever body facts can affect the next iteration.

For must analyses such as definite assignment, the join is intersection of guarantees across reachable predecessors. For may-shape analysis, it is union-like.

## Break and continue

`break` and `continue` are targeted abrupt completions. Their flow must bypass ordinary fallthrough:

```text
continue -> loop latch/header as semantics define
break    -> loop exit merge
```

Nested loops require target identity, not a boolean `did_break` flag. If labeled control is introduced later, the representation should naturally carry target IDs.

## Return and tail completion

Phalcom callable return summaries must distinguish:

```text
explicit return exits
normal tail completion
throw/non-local exits
non-termination/no-normal-return
```

Only normal returning exits contribute to normal return value. A method with `return 1` followed by unreachable `"x"` must not acquire `String` in its return shape merely because an AST walker saw the literal.

## Exceptions

If/when language `throw`/catch semantics are implemented, construct exceptional edges according to actual handler/unwind rules. Do not initially route every potentially throwing instruction to every handler; that is sound but may be prohibitively imprecise. Use effect summaries to determine which operations may throw, while unresolved/dynamic calls conservatively may throw unless a trusted contract says otherwise.

An exceptional edge carries the state after evaluation up to the throw point, not after subsequent normal operations.

## Non-local returns and blocks

Smalltalk-style non-local return from a block is not an ordinary return from the block invocation. It targets the lexical/home activation according to Phalcom's normative semantics. A future semantic IR may represent:

```text
NonLocalReturn { home: HomeContextId, value }
```

The analysis must account for invalid/non-live home contexts according to runtime semantics. Do not rewrite non-local return to an ordinary closure return merely to simplify CFG construction.

## Closure construction versus execution

When encountering a block literal:

```phalcom
let f = |x| { mutate(x) }
```

construction evaluates/captures according to language rules; the body does not execute merely because it appears in source. A caller/invocation analysis executes the block body semantics only when invocation is modeled.

This distinction prevents false effects, false field writes, and incorrect definite-assignment conclusions.

## CFG lowering invariants

For every lowering rule preserve:

1. operand evaluation order;
2. exactly-once evaluation unless source semantics duplicate it;
3. source-visible abrupt completion;
4. dynamic dispatch points;
5. capture/home-context semantics;
6. source provenance for diagnostics;
7. recovery boundaries for malformed syntax.

A useful differential test is to instrument side effects in source and verify CFG interpretation/lowered bytecode preserves their order.

## SSA is optional, not synonymous with CFG

SSA assigns each value definition once and introduces phi-like merges. It is powerful for optimizer/dataflow work but can complicate source-level mutable binding identity and editor explanations. Phalcom semantic analysis can first use a CFG with mutable binding-state maps. Introduce SSA only if repeated analyses materially benefit.

If SSA is added, keep a bridge:

```text
source BindingId <-> reaching SSA ValueId(s)
```

so rename/navigation/provenance remain source-semantic rather than compiler-temporary based.

## CFG verification

Before running analyses, a debug/test verifier can assert:

```text
entry exists
all successor IDs valid
terminators are total
no instructions after terminator
phi inputs match predecessors (if SSA)
source ranges belong to owning source snapshot
targeted break/continue/home IDs exist
```

For recovered malformed syntax, permit designated recovery nodes/terminators rather than violating graph invariants.

## Pressure tests

- branch where one arm returns: merge must contain only continuing arm state;
- loop whose second iteration adds a new shape: requires fixed point;
- nested loop with inner break: must not exit outer loop;
- condition call mutates a field used by refinement: refinement starts only after condition evaluation;
- block body writes a field but block is never called: construction must not record runtime write effect;
- non-local return inside block: must target lexical home, not block caller;
- unknown throwing call inside try/catch: exceptional successor remains possible;
- malformed branch missing an expression: unrelated predecessor/successor facts remain usable.

## Review questions

- What runtime control transfers does each edge represent?
- Are abrupt paths excluded from normal joins?
- Where does loop iteration reach a fixed point?
- Is reachability distinct from value unknownness?
- Are branch refinements edge-specific?
- Does the CFG preserve evaluation order and user-code invocation points?
- Are blocks analyzed as values at construction and code at invocation?
- Can source identity/provenance be recovered from normalized operations?
- Would introducing SSA improve multiple analyses enough to justify its mapping cost?

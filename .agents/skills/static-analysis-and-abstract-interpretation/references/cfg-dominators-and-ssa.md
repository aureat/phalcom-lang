# CFGs, Dominators, and SSA

A control-flow graph (CFG) makes execution paths explicit. Dominance and Static Single Assignment (SSA) make value-flow relations easier to query. They are powerful implementation tools, not semantic goals. Phalcom should introduce shared CFG/HIR/SSA infrastructure when it removes repeated control-flow reasoning across checker, prover, linter, optimizer, or advanced LSP analysis—not merely because mature compilers have one.

The current LSP semantic engine uses structured statement flow and already models branches, loops, breaks, continues, throws, returns, block effects, and bounded loop fixed points. That is a valid CURRENT architecture. A future CFG should preserve those semantics rather than silently replacing them with a simpler C-like control model.

## 1. Control-flow graph

A CFG is:

```text
G = (B, E, entry, exits)
```

where:

- `B` is a set of basic blocks;
- `E ⊆ B × EdgeKind × B` is possible control transfer;
- `entry` is the initial block;
- `exits` may include normal return, throw, non-local return, cancellation, etc.

A basic block is normally a maximal straight-line sequence with one entry and one terminal control transfer. For semantic tooling, blocks should carry stable-ish semantic/source identities rather than only bytecode offsets.

## 2. Edge kinds matter

Do not erase semantically distinct control transfers into unlabeled graph edges when downstream analyses need them. Possible kinds include:

```text
Normal
BranchTrue(predicate)
BranchFalse(predicate)
LoopBack
Break(loop_id)
Continue(loop_id)
Return(callable_id)
Throw
NonLocalReturn(home_callable)
Exceptional
Cancellation          # future, if normative
YieldResume            # only if concurrency analysis explicitly models it
```

Not every analysis needs every edge, but the shared CFG must not make important control impossible to reconstruct.

## 3. Build CFG from semantics, not syntax shape alone

For source:

```text
if x.isReady {
    return x.value
}
use(x)
```

a useful graph is:

```text
          [test]
          /    \
       T /      \ F
        v        v
   [return]    [use]
      |          |
 [return-exit] [next]
```

The return edge never reaches the merge before `use`, so facts refined on the false edge survive.

A syntax-tree walker that “joins then/else states” without abrupt reachability can accidentally lose this precision.

## 4. CFG construction obligations for Phalcom

When lowering source semantics into CFG, preserve:

- lexical evaluation order of receiver and arguments;
- short-circuit boolean/control sends;
- block invocation versus block construction;
- non-local return target;
- `break`/`continue` target identity;
- throw/exceptional paths required by consumer;
- `super` lookup context if calls are represented in IR;
- selector identity/dynamic selector family;
- source ranges and semantic IDs;
- evaluation of spreads/packs and their abrupt behavior;
- future pattern-binding scopes;
- malformed/recovery nodes without inventing valid semantics.

If an IR makes any of these unrepresentable, it is too weak.

## 5. Dominance

Block `D` dominates block `B`, written:

```text
D dom B
```

iff every path from `entry` to `B` passes through `D`.

Immediate dominator `idom(B)` is the strict dominator closest to `B`. The dominator relation forms a tree rooted at entry.

Why it matters:

- a definition in `D` can be known to execute before uses in dominated blocks;
- a guard dominating a use can justify a refinement, provided no invalidating mutation occurs;
- SSA placement uses dominance frontiers;
- optimizer check elimination often asks whether an equivalent check dominates the use.

Dominance is a control property, not proof that a mutable fact remains unchanged. Effects/aliasing can kill the fact between dominating guard and use.

## 6. Dominance equation

For all blocks except entry:

```text
Dom(entry) = {entry}
Dom(B) = {B} ∪ ⋂_{P ∈ pred(B)} Dom(P)
```

This iterative equation demonstrates that dominance is a must-property: a dominator must occur on every predecessor path.

Production implementations typically use more efficient algorithms (e.g. Lengauer-Tarjan or modern variants) rather than storing full sets for large CFGs.

## 7. Post-dominance

`D` post-dominates `B` if every path from `B` to an appropriate exit passes through `D`.

Useful for:

- cleanup/finalization reasoning;
- proving a release operation runs after all paths;
- control dependence;
- code motion.

Multiple exit kinds complicate the definition. Decide whether throw/non-local return/cancellation are included in the exit set. A resource proof that ignores exceptional exits is not a proof of cleanup.

## 8. Dominance frontier

The dominance frontier of `D` contains blocks where paths dominated and not dominated by `D` meet. It identifies where SSA φ-functions may be needed.

Intuition:

```text
      entry
      /   \
    x=1  x=2
      \   /
       join     <- dominance frontier of both defining blocks
```

## 9. SSA form

SSA gives each scalar variable version exactly one definition:

```text
x1 = 1
if cond {
    x2 = 2
}
x3 = φ(x1, x2)
use(x3)
```

A φ-node is not a runtime call. It selects the version corresponding to the predecessor edge.

Benefits:

- direct def-use chains;
- sparse propagation of value facts;
- constant propagation/SCCP;
- easier liveness/use queries;
- optimizer transformations over immutable value IDs.

## 10. SSA construction outline

Classic pruned SSA construction:

1. Build CFG.
2. Compute dominators and dominance frontiers.
3. For each source variable, find defining blocks.
4. Place φ-nodes iteratively in dominance frontiers, optionally pruned by liveness.
5. Rename variables along dominator tree using per-variable stacks.
6. Record source binding identity separately from SSA value identity.

Pseudo-code for placement:

```text
work = blocks_with_definitions(x)
while work not empty:
    n = pop(work)
    for y in DF(n):
        if phi_x not yet in y:
            insert phi_x in y
            if y did not originally define x:
                push y
```

## 11. Source binding identity is not SSA value identity

Phalcom tooling needs stable semantic `BindingId`s for definition/reference/rename. SSA creates *versions* of a binding:

```text
BindingId(x) -> {ValueId(x1), ValueId(x2), ValueId(x3), ...}
```

Do not replace source identity with SSA IDs. Rename operates on `BindingId`; constant propagation may operate on `ValueId`.

Likewise, source range identity and revision identity are separate from both.

## 12. Mutable captured bindings

A captured mutable binding is not a simple scalar SSA variable once closures share a cell. Options:

```text
promote only non-address-taken/non-captured locals to SSA
represent captured binding as load/store of CellId
use memory SSA for cell versions
use effect/alias abstraction and keep explicit cell operations
```

The simplest correct strategy is often to keep captured mutable cells out of scalar SSA until escape/alias machinery exists.

## 13. Fields and heap memory

Scalar SSA does not solve object fields:

```text
obj._x = 1
callUnknown(obj)
use(obj._x)
```

Heap mutation requires alias/effect reasoning. MemorySSA can version memory-def/use relationships, but it still depends on alias analysis to know which memory operations interfere.

**RECOMMENDATION:** use field/effect summaries and explicit heap abstraction first. Add MemorySSA only when optimization/proving use cases justify it.

## 14. Sparse Conditional Constant Propagation

SCCP combines:

- executable-edge reachability;
- SSA value lattice such as `Unknown/Const/Overdefined`.

It propagates only along executable edges and can discover unreachable branches. This illustrates why SSA plus CFG can outperform dense per-block propagation for value-centric analyses.

A typical lattice:

```text
⊥ / Undef
   |
Const(c)
   |
⊤ / Overdefined
```

Different implementations use different naming/order conventions; define them explicitly.

## 15. Exceptional and non-local control in SSA

SSA construction remains possible with exceptional edges, but φ-placement and dominance reflect those edges. Omitting an edge can make a definition appear to dominate a use when a concrete execution bypasses it.

For Phalcom, especially audit:

```text
block non-local return
throwing sends
future cancellation
control-flow library methods recognized specially by semantic lowering
```

If a source-level construct is implemented as a message send but semantically acts like control flow, the IR must encode the effective control, not just an opaque call, when an analysis relies on it.

## 16. Structured flow versus shared CFG

Structured flow is sufficient when:

- syntax remains structured;
- only a few analyses need flow;
- recursive transfer functions are clear;
- program-point identity is local;
- dominance/liveness are not required.

A shared CFG becomes compelling when checker/prover/linter/optimizer repeatedly reimplement:

```text
reachability
loop exits/back-edges
definite assignment
branch refinement
return/throw propagation
liveness
dominance
control dependence
proof paths
```

The threshold is duplication and semantic drift, not project prestige.

## 17. IR layering recommendation

If Phalcom introduces a semantic IR, a useful split is:

```text
Source AST/CST
    preserves spelling/recovery/source structure

Semantic HIR
    resolved identities, desugared semantically equivalent constructs,
    source provenance retained

CFG / analysis IR
    explicit control edges and normalized operations

Bytecode
    VM execution representation
```

Not every layer must exist immediately. Avoid forcing the LSP to reverse-engineer source meaning from bytecode; bytecode may erase distinctions needed for diagnostics/refactoring.

## 18. Incremental identity

CFG block indices are often rebuilt after edits. Do not expose them as durable semantic identities across revisions unless a stable identity design exists.

Cache keys can use:

```text
CallableId + BodyRevision + AnalysisMode
```

and keep `BlockId` local to that body generation. Source/semantic IDs map facts back to the editor.

## 19. Verification of CFG lowering

Strong invariants:

```text
all reachable blocks except exits have valid terminator
all edges point to existing blocks
entry has no ordinary predecessors
phi operand count/order matches predecessor relation
break/continue target correct enclosing loop
return/non-local return target correct callable/home
source provenance exists for diagnostic-relevant operations
```

Differential testing can compare structured-flow analysis with CFG-based analysis during migration on supported constructs.

## 20. Failure modes

- Introducing SSA and then pretending heap fields are SSA scalars.
- Using SSA value IDs for rename/reference identity.
- Omitting throw/non-local edges to simplify dominance.
- Treating φ as a runtime function call.
- Adding a CFG per consumer, each with slightly different edge semantics.
- Lowering away source provenance needed by diagnostics.
- Rebuilding a whole-project CFG for one local edit without evidence it is necessary.
- Assuming a dominating type test remains valid across arbitrary mutation.

## 21. Testing obligations

1. diamond branch + φ placement;
2. terminating branch and dominance;
3. nested loop break/continue targets;
4. loop-carried φ;
5. throw edge where required;
6. block non-local return target;
7. captured mutable binding not incorrectly promoted;
8. field mutation remains memory effect, not scalar SSA;
9. short-circuit control edges;
10. source mapping after lowering;
11. CFG invariants under malformed/recovered source if editor CFG is built;
12. incremental clean-rebuild equivalence;
13. deterministic block/value numbering within a generation for stable tests.

## 22. Review questions

1. Which repeated analysis problem justifies this CFG/SSA layer?
2. What source semantics are normalized, and which are preserved?
3. Are all abrupt control edges represented?
4. What is a source identity versus an SSA value identity?
5. Which values cannot safely be scalar-promoted due to alias/capture?
6. Does the consumer need dominance or only ordinary structured flow?
7. How do facts map back to precise source diagnostics?
8. What invalidates cached CFG/SSA products?
9. Is heap reasoning being accidentally smuggled into scalar SSA?
10. Can the new analysis be differentially checked against the old structured implementation during transition?

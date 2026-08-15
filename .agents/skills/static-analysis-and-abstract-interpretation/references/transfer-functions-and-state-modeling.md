# Transfer Functions and State Modeling

Transfer functions are where language semantics become analysis semantics. A mathematically sound lattice is useless if assignment, calls, blocks, mutation, abrupt completion, evaluation order, or dynamic boundaries are modeled incorrectly. This reference shows how to design state and transfer functions for Phalcom without confusing implementation convenience with semantic behavior.

## 1. Transfer is an abstraction of execution

For an abstract domain `A`, a simple statement transfer has the form:

```text
F_stmt : A -> A
```

but real language control often needs multiple successor classes:

```text
F_stmt : A -> TransferResult(A)
```

where conceptually:

```text
TransferResult(A) = {
    normal: A or unreachable,
    return exits,
    throw exits,
    break exits,
    continue exits,
    non-local returns,
    emitted effects/events
}
```

Do not add every category to every analyzer. Add the categories whose omission could change the property being analyzed. The current Phalcom LSP `StatementFlow` carries normal state, returns, breaks, continues, throws, and tail value; block effects separately carry non-local returns and captured writes. That is a source-oriented implementation of the same principle.

## 2. Separate semantic state components

A useful abstract state is usually a product:

```text
State = Lexical × Heap × Globals × Path × Effects × ControlMetadata
```

An analysis may project away components it does not need. For current LSP shape flow:

```text
State ≈ BindingId -> InferredValue
```

while field facts and call events are collected as separate products. A future proof/effect/alias analysis may need richer state.

### Design rule

For every state component, document:

```text
concrete meaning
abstract value
missing-key meaning
join
strong/weak update rules
what operations invalidate it
whether it is flow-sensitive
whether it participates in semantic equality
```

## 3. Evaluation order is part of transfer semantics

For an expression with subexpressions `e1`, `e2`, do not infer both from the same pre-state if concrete Phalcom evaluates `e1` first and `e1` can mutate state or invoke user code.

Concrete-style sequencing:

```text
⟨e1, σ0⟩ ⇓ ⟨v1, σ1⟩
⟨e2, σ1⟩ ⇓ ⟨v2, σ2⟩
```

The abstract transfer should reflect:

```text
(a1, σ1#) = eval#(e1, σ0#)
(a2, σ2#) = eval#(e2, σ1#)
```

This matters for:

- message receivers and arguments;
- dynamic packs/spreads;
- setters/index writes;
- collection literals whose element expressions invoke code;
- short-circuit operations;
- block construction if capture metadata changes;
- reflection/native calls.

A pure expression evaluator returning only an `InferredValue` can be safe only if side effects are modeled elsewhere in exactly the same evaluation order. Review this carefully when splitting “infer value” and “collect events” passes.

## 4. Lexical assignment

For:

```text
x = e
```

with a resolved `BindingId b`:

1. evaluate `e` under input state;
2. collect/apply effects caused by evaluating `e`;
3. update `b`;
4. invalidate relations depending on the previous value of `b`;
5. attach provenance for the assignment site;
6. emit normal successor unless evaluation completed abruptly.

A non-relational strong lexical update:

```text
σ' = σ[b ↦ value#(e)]
```

is valid when `BindingId` denotes exactly one lexical storage cell within the modeled execution. Captured mutable bindings may still denote one cell but can be modified by invoked blocks; the call/block transfer must model those writes.

### Mutable versus immutable bindings

Do not infer mutability from syntax text or name. Use the semantic binding record. An assignment to an immutable binding is a semantic error and should not be silently treated as an ordinary state update by a correctness analyzer.

## 5. Pattern binding/destructuring

A pattern transfer is not simply “copy the whole value to every name.” Define projection:

```text
project(pattern_position, value#) -> component#
```

For a tuple:

```text
(a, b) = value
```

if:

```text
value# = Tuple([Int, String])
```

then:

```text
a# = Int
b# = String
```

If shape information is insufficient, component facts widen to the appropriate unknown/top—not to unreachable.

For correctness-oriented destructuring, separately model the possibility that runtime shape does not satisfy the pattern, according to actual Phalcom runtime semantics.

## 6. Field and heap updates

For receiver-local field syntax such as `_x = e`, a source analyzer may know the declaration owner and storage side. But there are two distinct levels:

### Source evidence aggregation

Record that some executable write site writes shape/type/effect `V` to field identity `FieldId`. This is useful for LSP inference and declaration diagnostics.

### Runtime heap-state reasoning

Track the current value of a field on a particular abstract object across program points. This requires alias/object abstraction. A write through one alias can affect reads through another. See [heap-alias-and-escape-analysis.md](heap-alias-and-escape-analysis.md).

Do not treat an aggregated “field has observed writes Int | String” fact as if it were a precise heap store for a specific instance.

## 7. Strong and weak updates

A strong update replaces old information:

```text
Store[l] := v#
```

A weak update joins:

```text
Store[l] := Store[l] ⊔ v#
```

Strong update is sound only if abstract location `l` denotes exactly one concrete location in every represented execution. Lexical `BindingId` often satisfies this. An allocation-site heap location created inside a loop usually does not: it may summarize many concrete objects, so a field write requires weak update.

### Rule of thumb

```text
must-alias exactly one mutable cell -> strong update may be valid
may-alias / summary location        -> weak update
```

Prove the uniqueness condition from the alias abstraction; do not infer it because the Rust key is unique.

## 8. Calls are transfer functions over summaries

A call transfer needs more than a return value. Given possible targets `T = {t1, ... tn}`:

```text
Call#(state, args, T)
```

should combine:

```text
return fact
normal-state effects
field/global writes
captured binding writes
throws
non-local returns if relevant
may_yield / blocking
reflective mutation
dynamic-dispatch uncertainty
native/FFI effects
```

If multiple targets are possible, join their externally visible results/effects according to the analysis domain.

### Target unknown

Do not invent a target. Apply the conservative dynamic-call policy:

```text
return -> top/unknown appropriate to domain
writes -> havoc components reachable/mutable by unknown call
throws -> possible if language permits
may_yield -> possible if unknown callable can yield
reflection -> possible if callable may access reflection
```

You can narrow havoc with language capability rules. If unknown user code cannot access another object's private receiver-local fields except through public methods, that may bound direct mutation—but public methods and captured aliases may still mutate relevant state.

## 9. Havoc

`havoc` intentionally forgets facts invalidated by an operation whose exact behavior is unknown.

Formally, for state component `X`:

```text
havoc_X(σ) = σ[X ↦ ⊤_X]
```

or a narrower top for the affected subset.

Examples:

- unknown native call receiving a mutable buffer: havoc that buffer's contents/alias facts;
- reflective class-method mutation: invalidate dispatch/member-surface assumptions for affected class generations;
- unknown global mutation: havoc relevant module/global facts;
- fiber yield with shared mutable object accessible to other fibers: havoc or weaken facts that may be concurrently/interleavingly mutated.

### Havoc should be minimal but sound

“Forget everything” is sound but often useless. “Forget nothing” is precise but often unsound. Design an effect/alias model that can identify the narrowest sound invalidation set.

## 10. Condition transfer and edge refinement

A condition evaluation has two conceptually separate parts:

1. evaluate the condition expression and its effects;
2. refine successor states based on the outcome.

For a trusted type/class test:

```text
x.isExactly(String)
```

with `Shape(x) = {Int, String}`:

```text
true edge:  x -> {String}
false edge: x -> {Int}
```

If the condition is known true/false, the impossible edge is unreachable (`⊥`).

### Do not infer refinements from names

A method named `isString()` is ordinary user code unless the language gives it a trusted refinement contract or the prover establishes one. A static analyzer must not turn naming conventions into semantic axioms.

### Refinement invalidation

If a condition proves a fact about mutable heap state and a call/yield occurs before use, the fact may no longer hold. Flow refinements need effect/alias stability assumptions.

## 11. Short-circuit control

Operations such as `and`/`or` may be message-level syntax with block operands and short-circuit behavior. Analysis must model:

- left expression evaluated first;
- block constructed;
- block invocation only on the appropriate outcome;
- captured writes/effects only when invoked;
- result join across invoked/not-invoked paths.

Do not simply evaluate both operands and join. That can introduce effects from a block that concrete execution would not invoke.

## 12. Blocks and latent effects

Constructing a block and invoking it are distinct events.

Block construction may:

- capture lexical cells;
- allocate a closure/block object;
- establish home-callable identity for non-local returns.

It should **not** apply the block body's writes/effects as if the body ran.

Store latent effects:

```text
BlockSummary = {
    params,
    normal result,
    captured writes,
    nonlocal returns,
    throws,
    invokes parameters,
    dynamic send,
    may_yield,
    ...
}
```

When a trusted callee summary says it invokes block parameter `i`, compose the block summary then. Invocation timing/cardinality can affect precision and must-properties.

CURRENT Phalcom LSP already distinguishes block effects and records `invokes_parameters` in callable summaries. This is a foundation to generalize, not a complete effect system.

## 13. Non-local return

A block can potentially return to its home callable rather than its immediate invocation frame, depending on ratified Phalcom semantics. An analyzer must distinguish:

```text
local block result
callable local return
non-local return targeting home callable
```

A block summary can carry `nonlocal_returns: Vec<ReturnEvidence>`. When the block is invoked, those exits contribute to the target callable's return summary and terminate the appropriate control path.

Ignoring non-local return can make code after block invocation falsely reachable and corrupt return inference.

## 14. Loops as transfer/fixed-point composition

A loop transfer is not a one-shot function. It repeatedly applies condition/body transfer until header state stabilizes or widens.

Conceptually:

```text
H0 = Entry
Hi+1 = Entry ⊔ ContinueBackEdges(Body(Condition(Hi)))
```

Exit state joins:

```text
condition-false states
break states
other normal exits defined by concrete loop semantics
```

Return/throw/non-local return do not contribute to normal loop exit.

The current LSP flow has an explicit bounded loop fixed point and `widen_loop_state`. Any modification should be reviewed against these equations, especially zero iterations and back-edge projection of outer bindings.

## 15. Call argument evaluation and parameter evidence

Call-site parameter inference needs exact evaluation order and contribution ownership.

If arguments are:

```text
receiver.m(a(), b())
```

and `a()` can change state used by `b()`, the abstract facts for `b()` must use the post-`a()` state.

Interprocedural parameter facts should be contribution-indexed:

```text
ParameterSlot(target, name)
    <- ContributionSource(caller/top-level) -> InferredValue
```

Then changing one caller can remove only that caller's old evidence and rejoin the remaining sources. CURRENT Phalcom LSP already uses this design.

This is superior to monotonically appending observations forever, which cannot retract stale evidence after edits.

## 16. Dynamic packs and selectors

When a send includes dynamic argument expansion or computed labels, selector identity may be unknown or a bounded family.

Transfer policy:

```text
static selector exact      -> normal dispatch resolution
bounded selector family    -> resolve each feasible selector, join
unbounded/unknown selector -> dynamic-call effect + conservative return
```

Do not fabricate one selector from incomplete labels. That creates false call edges and stale parameter facts.

For advisory tooling, a family abstraction can preserve base name and receiver shape to offer useful completion while keeping exact-call resolution unknown.

## 17. Exceptions and throws

A `throw` evaluates its expression first, then produces no normal successor. Calls/operations that may throw create both normal and exceptional behavior.

A simple advisory shape analysis may only record `throws: bool`. A correctness resource/definite-state analysis may need an explicit exceptional edge state because effects before the throw matter.

Example:

```text
resource.open()
maybeThrow()
resource.close()
```

If `maybeThrow` can throw, “resource definitely closed” is false unless cleanup semantics cover the exceptional edge.

## 18. Fibers and yields

A transfer across `may_yield` may allow other fibers to run and mutate shared reachable state. The analysis should classify facts:

```text
fiber-local immutable fact       survives
lexical cell inaccessible elsewhere may survive
shared mutable heap fact         may need havoc/weaken
class/method surface             depends on whether reflection can mutate during yield
```

Do not treat cooperative concurrency as “no concurrency.” Cooperative scheduling makes interleaving points explicit, which can simplify the model, but facts spanning a yield still need interference reasoning.

## 19. Native and FFI transfers

Native code has no analyzable Phalcom body unless a semantic summary exists. A trusted native summary should specify relevant dimensions:

```text
parameter/return shape or type
throws
reads/writes through arguments
reads/writes globals/classes
allocates
IO/process/network
blocks OS thread
may_yield / invokes scheduler
retains references / escape
reflective mutation
invokes callbacks/blocks
```

If absent, use conservative effects. “Implemented in Rust” does not imply pure or safe for static reasoning.

## 20. Provenance through transfer

A value fact should be explainable:

```text
initializer -> assignment -> branch refinement -> call summary -> join -> widening
```

Avoid storing only a flat list of source ranges if diagnostics need causal chains. A scalable architecture can use provenance DAG nodes interned by ID:

```rust
struct ProvenanceId(u32);

enum ProvenanceNode {
    Syntax(SourceRange),
    Assignment { site: SourceRange, from: ProvenanceId },
    Branch { test: SourceRange, truth: bool, from: ProvenanceId },
    Call { site: SourceRange, callee: CallableId, from: ProvenanceId },
    Join(SmallVec<[ProvenanceId; 2]>),
    Widen { reason: WidenReason, from: ProvenanceId },
}
```

Bound depth/sample counts if needed, but preserve a `Truncated(reason)` marker rather than silently discarding explanation.

CURRENT `InferredValue` stores a compact provenance vector capped during join. That is appropriate for current LSP needs; a future checker diagnostic may need a richer causal structure.

## 21. State projection at lexical boundaries

When analyzing a nested block, local bindings declared inside it should not leak into outer state. Captured outer bindings may be updated.

Conceptually:

```text
project_outer(entry_scope_state, block_exit_state)
```

keeps only storage cells visible/owned by the outer context, applying captured writes where semantically valid.

Phalcom's current structured flow has explicit outer-state projection when joining invoked-block results back into enclosing control flow. Preserve this invariant when adding block parameters, nested closures, or new control selectors.

## 22. Semantic equality of transfer results

For fixed-point propagation, decide which fields matter semantically.

A summary's semantic equality might include:

```text
params
returns
dependencies
effects
```

but exclude:

```text
publication generation
bounded provenance samples that do not change semantic meaning
allocation identity
```

CURRENT `callable_summary_changed` in Phalcom explicitly compares semantic products while excluding revision provenance. That prevents unnecessary invalidation.

For diagnostic caches, provenance differences may still matter to rendering. This means “semantic equality” and “presentation equality” can be distinct relations.

## 23. Failure modes

### Evaluating subexpressions out of order

Can preserve stale facts or apply effects too early/late. Fix: model lexical evaluation order.

### Applying block body effects at block construction

Makes lazy/higher-order code look eagerly mutating. Fix: latent block effects.

### Unknown call returns unknown but has no effects

Unsound for field/global/alias/effect analyses. Fix: conservative call effect/havoc.

### Strong heap update without singleton proof

Excludes objects summarized by same abstract location. Fix: weak update.

### Branch refinement from arbitrary predicate name

Turns user code into an axiom. Fix: trusted built-in/refinement contract/proven result only.

### Treating all throws as global boolean when edge state matters

Can prove must-properties incorrectly. Use exceptional successor states for those analyses.

### Captured block writes leak from uninvoked block

Fix: apply captured writes only under invocation summary.

### Non-local return treated as local block return

Fix: explicit home target and abrupt control effect.

## 24. Testing obligations

Test transfer functions independently with hand-constructed states where possible.

### Assignment

- mutable local strong update;
- immutable binding rejection/unchanged state;
- assignment after effectful RHS;
- shadowed `BindingId`s.

### Branch/refinement

- true/false edge;
- exact versus subclass test;
- impossible edge becomes unreachable;
- terminating branch retains refinement on survivor;
- mutation/yield invalidates unstable refinement where required.

### Blocks

- construction has no body effect;
- invoked block applies captured write;
- never-invoked block does not;
- non-local return terminates correct path;
- nested captures project correctly.

### Calls

- one exact target;
- multiple receiver targets joined;
- unknown target conservatively havocs;
- native summary obeyed;
- changed caller contribution retracts old parameter fact.

### Evaluation order

Use side-effecting arguments/receivers so reversing order changes expected facts. These tests catch value/effect split bugs.

## 25. Review questions

1. What exact concrete operation does each transfer approximate?
2. Are subexpressions analyzed in concrete evaluation order?
3. Which state components can the operation mutate?
4. Can it invoke user code, throw, non-locally return, block, or yield?
5. Is an assignment strong or weak, and why?
6. Does a block's effect occur at construction or invocation?
7. What happens when dispatch or native behavior is unknown?
8. Which branch predicates are trusted to refine?
9. What state is projected out of nested lexical scopes?
10. Which transfer-result fields participate in fixed-point equality?
11. Does provenance retain enough causal structure for diagnostics?
12. Can an edit retract old evidence rather than only accumulating new facts?

If transfer answers are vague, the analyzer is not yet grounded in Phalcom semantics.

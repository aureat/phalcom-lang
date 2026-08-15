# Interprocedural Analysis and Call Graphs

Interprocedural analysis extends local reasoning across callable boundaries without recursively reinterpreting the entire program at every call. In Phalcom this is difficult because dispatch is receiver- and selector-driven, blocks are first class, selectors/packs may be dynamic, reflection and native code can obscure targets, and the call graph itself may become more precise as value analysis improves. The default architecture should therefore be summary-based, dependency-driven, and explicit about unknown edges.

## 1. Why naive recursive descent fails

A tempting implementation for a call expression is:

```text
analyze caller
  -> see call f()
     -> recursively analyze f body
        -> see call g()
           -> recursively analyze g body
```

This fails in several ways:

- direct/mutual recursion causes infinite recursion or ad hoc depth cutoffs;
- the same callee is reanalyzed at every call site;
- invalidation is difficult because facts are embedded in caller traversal;
- dynamic dispatch can have multiple targets;
- higher-order blocks require callback edges not syntactically nested in caller;
- native callables have no source body;
- editor latency scales with query fan-out.

Instead, analyze callable bodies into reusable summaries and solve dependencies to a fixed point.

## 2. What a callable summary means

A summary abstracts only behavior relevant outside the callable boundary.

A conceptual Phalcom summary:

```text
CallableSummary {
    callable: CallableId,

    // input abstraction
    params: [ValueFact],
    receiver_requirements_or_fact,

    // output abstraction
    returns: ValueFact,

    // externally visible behavior
    effects: EffectSummary,
    captured_writes,
    nonlocal_return_behavior,

    // dependencies for incremental solving
    callees: [CallableId],
    dynamic_dependencies,

    // explanation / publication metadata
    provenance,
    semantic_revision
}
```

Do not store every local binding fact in a summary unless a consumer genuinely needs it. Local facts belong to the callable/source analysis product; summary facts are the boundary interface.

CURRENT Phalcom LSP `CallableSummary` contains callable identity, inferred parameter values, return value, direct callable dependencies, `SummaryEffects`, and semantic generation. Its current `SummaryEffects` tracks `dynamic_send` and `invokes_parameters`. Treat this as CURRENT advisory scope, not a complete future effect system.

## 3. Parameter facts as contributions

Interprocedural input inference often collects evidence from call sites.

Suppose:

```text
f(1)
f("x")
```

Then a may-style parameter fact is:

```text
Param(f, 0) = Int ⊔ String
```

A mutable incremental database must know *which caller contributed which evidence*. Otherwise editing away `f("x")` cannot remove String.

Use:

```text
ParameterSlot(f, p)
    -> {
         ContributionSource(caller1): Int,
         ContributionSource(caller2): String
       }
    -> joined: Int | String
```

This contribution-indexed design is CURRENT in Phalcom's semantic engine. `ParameterContributions` keeps forward per-slot contributions, reverse slots-by-source, and cached joins so replacing one source recomputes only touched slots.

This is a major architectural invariant worth preserving as future types/effects are added: derived facts should retain dependency ownership so edits can retract stale evidence.

## 4. Call graph definitions

A call graph has:

```text
nodes = semantic callable identities
edges = possible calls
```

For a dynamic language, distinguish:

```text
must target: one target guaranteed by semantic resolution
may target set: bounded set of possible targets
dynamic edge: target not finitely/precisely known
callback edge: callee may invoke passed callable/block
reflective edge: target selected through reflection
native edge: native implementation / external boundary
```

Do not force unknown calls into fake edges to every callable in the workspace unless that is the deliberate conservative abstraction. “All methods” may be sound but catastrophically imprecise; an effect-based dynamic edge is often better.

## 5. Phalcom dispatch and call target construction

Phalcom's ordinary dispatch is selector-based. Types should not silently become selector keys. Interprocedural analysis therefore needs:

1. evaluate/infer receiver abstract value;
2. construct canonical selector from source/call shape;
3. map each possible receiver to the correct dispatch side/class start;
4. resolve lookup according to Phalcom inheritance/metaclass/access semantics;
5. collect possible `CallableId`s;
6. apply summaries and join;
7. model missing/dynamic cases separately.

If receiver shape is:

```text
Instance(A) | Instance(B)
```

and selector is exact `foo()`:

```text
Targets = lookup(A, foo()) ∪ lookup(B, foo())
```

If selector is dynamically constructed from packs/labels, target precision may collapse even when receiver is known.

### `super`

`super` dispatch must start lookup at the language-defined superclass of the lexical/holder context, not by pretending the receiver has a different runtime class. Use the shared dispatch resolver; do not duplicate lookup rules in the call graph.

### Class-side/metaclass dispatch

A class object receiver and an instance receiver have distinct dispatch sides. Preserve this in `CallableId`/receiver abstraction. Do not flatten `ClassObject(C)` and `Instance(C)`.

## 6. Call graph and value analysis are mutually recursive

Receiver facts determine targets; targets determine return facts; return facts determine later receiver facts.

Example:

```text
factory().build().run()
```

If `factory()` summary becomes more precise, `build()` target may resolve, making its return more precise, resolving `run()`.

This creates equations across:

```text
value facts
call targets
parameter facts
return summaries
```

A worklist/fixed-point solver should allow precision to propagate until semantic stability.

CURRENT Phalcom `solve_affected_callables_with_cancel` does this incrementally: it analyzes dirty callables against current summaries/parameters, updates dependency edges, replaces emitted parameter contributions, re-enqueues changed dependents/parameter targets, then performs bounded final source passes so source binding facts and callable facts share a coherent dispatch view.

That current mechanism is a useful base. A future checker may use separate formal type facts while reusing stable IDs, dependency machinery, callable surfaces, and scheduling concepts.

## 7. Summary equations

Let `S_f` be summary of callable `f`. A simplified equation:

```text
S_f = AnalyzeBody(f, Inputs_f, {S_g | g may be called by f})
```

Parameter inputs can themselves depend on callers:

```text
Inputs_f = ⊔ { Args_at_callsite(c -> f) }
```

So the system is mutually recursive:

```text
S_f = F_f(Inputs_f, S_targets)
Inputs_f = G_f(S_callers, local caller states)
```

A solver updates only nodes whose input summaries or contributions changed.

## 8. Worklist solving

A generic summary worklist:

```text
initialize boundary/native summaries
seed dirty callables

while worklist not empty:
    f = pop
    old = summary[f]
    new, emitted_contribs, deps = analyze_body(f, current_db)

    replace contributions from f
    update dependency edges

    if semantic(new) != semantic(old):
        summary[f] = new
        enqueue callers/dependents(f)

    for parameter slot changed by contribution replacement:
        enqueue owner(slot)
```

The exact schedule can differ, but dependency completeness is mandatory.

### Semantic equality

Compare summary meaning, not generation/provenance allocation. CURRENT Phalcom `callable_summary_changed` compares callable ID, param values, return, dependencies, and effects while excluding generation. Preserve this distinction.

## 9. Recursive SCCs

Strongly connected components identify recursive call clusters.

For graph:

```text
f -> g
g -> f
h -> f
```

`{f,g}` is an SCC. `h` is outside and depends on the SCC result.

A solver can:

1. compute SCC condensation DAG;
2. process acyclic SCCs in dependency order;
3. iterate within recursive SCCs to fixed point/widening;
4. propagate changed summaries to dependent SCCs.

### When SCCs help Phalcom

- large recursive graphs cause repeated global worklist churn;
- diagnostics need “recursive cycle caused widening” provenance;
- context-sensitive summaries are grouped by recursion cycles;
- proof/checker domains have expensive transfers.

### When a global worklist is enough

If current callable counts are modest and reverse-dependency scheduling already visits only dirty nodes, explicit SCC machinery may add complexity without material latency improvement. Measure first.

## 10. Context sensitivity

One summary per callable is context-insensitive:

```text
identity(x) called with Int and String
=> param Int|String, return Int|String
```

At each individual call site, this loses correlation.

### Call-site sensitivity

Context key includes recent call site(s):

```text
Summary(f, CallSite#123)
Summary(f, CallSite#456)
```

### Object sensitivity

Context includes abstract receiver allocation/class object:

```text
Summary(method, ReceiverAbstractObject)
```

### Type/generic sensitivity

Future typed analysis can parameterize summary by canonical type arguments or inferred instantiation:

```text
Summary(map, T=String, U=Int)
```

### Cost

If:

```text
N callables × C contexts × D domain size
```

becomes large, memory/solver cost can explode. Bound contexts and merge deliberately.

Start context-insensitive unless a concrete precision failure matters to checker/prover/optimizer/LSP.

## 11. Call strings and bounded contexts

A `k`-call-string context uses the last `k` call sites. For `k=1`:

```text
Context = current call site
```

This distinguishes immediate callers but merges deeper history. It guarantees finite contexts when call sites are finite.

Do not use unbounded call stacks as context keys; recursion creates infinitely many contexts.

## 12. Higher-order blocks and callback summaries

A callable receiving a block needs a contract for whether/how it invokes it.

At minimum:

```text
invokes_parameter(i)
```

More precise future summary:

```text
InvocationSummary {
    parameter_index,
    cardinality: Never | AtMostOnce | ExactlyOnce | Many | Unknown,
    timing: Synchronous | Deferred | Escapes | Unknown,
    on_paths: predicate/condition if modeled,
}
```

### Why cardinality matters

If a block increments captured `x`, one synchronous exactly-once invocation can strong-update a scalar relation; “zero or many” requires a join/widening.

### Why timing matters

If a block escapes for later execution, applying captured writes immediately is wrong. The closure's effects become latent/escaping and may interact with fibers/concurrency.

CURRENT Phalcom summaries only record invoked parameter positions. This is a good foundation but not enough for advanced effect/escape/proof reasoning.

## 13. Blocks passed through blocks

Higher-order propagation can be multi-level:

```text
outer(block) -> helper(block) -> block.call()
```

If `outer` summary is analyzed before `helper` knows it invokes parameter 0, the invocation fact must later propagate back and reanalyze `outer`/its callers as required.

This is exactly why invocation metadata belongs in summaries and dependencies, not as one local AST heuristic.

## 14. Dynamic dispatch target sets

For receiver abstract domain `R#`, define:

```text
targets(R#, selector) -> TargetSet | Dynamic
```

Properties:

- monotone in receiver uncertainty: adding possible receiver classes must not remove valid call targets;
- canonical target ordering;
- inheritance lookup centralized;
- missing-target behavior represented;
- method-table revision dependencies recorded if reflection can mutate dispatch.

If target set grows beyond a practical bound, widen at the *call abstraction*:

```text
KnownTargets({t1..tK}) -> DynamicDispatch(effect envelope)
```

rather than enumerating thousands of methods.

## 15. `doesNotUnderstand` / missing-message behavior

If Phalcom defines a missing-message hook, an unresolved ordinary selector is not automatically “VM error.” Interprocedural modeling should follow the actual language semantics:

```text
ordinary target exists -> call target
ordinary target missing -> missing-message protocol/fallback or runtime failure
```

A checker may diagnose “selector not guaranteed” while runtime still has a fallback. An optimizer cannot assume the call is impossible if fallback is observable.

Do not import Smalltalk/Python/Ruby missing-method semantics unless Phalcom's current spec says so.

## 16. Reflective invocation

For reflective `perform`/method invocation-like behavior:

### Exact selector object/string

Resolve as an ordinary send under the reflection/access rules.

### Bounded selector set

Resolve each and join.

### Unknown selector

Represent dynamic dispatch and conservative effects. Do not build an unbounded edge to every method if an effect envelope suffices.

If reflective APIs can invoke captured `MethodObject`s directly, the call graph may include pinned method identities separate from ordinary future dispatch. Preserve the distinction between “captured implementation” and “send this selector later.”

## 17. Method mutation and generation dependence

If reflection can install/replace methods, a call graph based on class surfaces is valid only for a dispatch generation/version.

Cache key/validity may need:

```text
receiver class identity
selector
class/method-table revision
access context
world/profile assumptions
```

An edit or runtime reflective mutation can invalidate previously known target sets.

Static offline checking may choose a closed-world profile that prohibits mutation while checking. LSP analysis of source can use workspace generation assumptions. Runtime optimizer devirtualization may require guards on method-table/class version.

## 18. Native callables

Native primitives need semantic summaries because no Phalcom AST body exists.

A native summary can seed:

```text
return fact
effects
callback invocation behavior
escape/retention behavior
throws/yields/blocks
```

A missing native summary is an unknown boundary, not “no dependency.”

Future formal typing should share canonical native signature metadata with the LSP/checker rather than maintaining a separate hand-coded return-shape table in each consumer.

## 19. Modules and call graphs

Callables are module-qualified. Import graph changes can affect:

- class/name resolution;
- dispatch surfaces;
- reachable target set;
- native/core availability;
- semantic identities if module provider resolution changes.

Module dependency invalidation therefore composes with callable dependency invalidation.

CURRENT Phalcom engine classifies source changes, computes module dependent closures for surface/import changes, and uses precise callable frontiers for body-only edits where possible. This layered invalidation is preferable to either full workspace rebuild or dangerously narrow callable-only invalidation when declarations/imports changed.

## 20. Final-source stabilization versus summary solve

A subtle current Phalcom pattern deserves explicit understanding. Callable worklist summaries are solver inputs, but final published local/field/parameter/summary products come from coherent source-backed flow analyses. After worklist propagation, source flow can discover more precise arguments using newly resolved return summaries; the current solver allows a bounded feedback cycle and final pass.

This is an example of a coupled analysis where:

```text
summary fixed point
  -> more precise source flow
  -> changed parameter contributions
  -> changed summaries
```

When extending this architecture, avoid two inconsistent “truths”: a summary database from one pass and local facts from another unrelated pass. Publication must be coherent.

## 21. Unknown-edge effect envelopes

Instead of a single `dynamic_send: bool`, future analyses may need a structured dynamic edge:

```text
DynamicCallEffect {
    return_domain: Top,
    may_throw: true,
    may_yield: true/depends profile,
    writes_globals: Top,
    writes_reachable_heap: Top or capability-bounded,
    reflective_mutation: possible,
    invokes_unknown_callbacks: possible,
}
```

Do not eagerly add maximal pessimism if Phalcom capability/access semantics bound what unknown code can do. But every excluded effect needs a semantic reason.

## 22. Precision recovery after edits

Incremental analysis is not monotonic across source revisions. Removing a call can make a parameter summary *more precise*.

Contribution replacement enables:

```text
rev1:
  caller A -> f(Int)
  caller B -> f(String)
  Param(f) = Int|String

rev2 removes B:
  Param(f) = Int
```

A cache that stores only joined `Int|String` with no source ownership cannot recover this without full recomputation.

The same principle applies to effects, call targets, and type constraints: retain dependency provenance at a granularity that supports retraction.

## 23. Interprocedural provenance

For diagnostics, a return fact may need a causal chain:

```text
expected String at use site
found Number
because call foo() returns Number
because reachable return in foo at line ... evaluates bar()
because bar summary returns Number
```

Do not retain only a `CallableId` origin if the future diagnostic needs return-site evidence. Use compact summary provenance with source anchors, and bound recursion cycles with explicit cycle markers rather than expanding forever.

## 24. Performance model

Track:

```text
callables visited per edit
callables changed per edit
summary equality hits
parameter contribution sources replaced
slots touched / slots changed
reverse dependency fan-out
worklist pushes / dedup hits
solver rounds / steps
final stabilization passes
module frontier size
snapshot product reuse
```

CURRENT Phalcom already records several of these counters. Use them before introducing context sensitivity, SCC scheduling, or new cache layers.

## 25. Failure modes

### Recursive AST descent

Causes recursion/nontermination/reanalysis. Use summary worklist.

### Call graph constructed once before value inference

Misses targets that become resolvable later. Couple target/value facts through dependencies or conservative dynamic edges.

### Unknown target treated as no call

Unsound effects/return reasoning. Use dynamic boundary summary.

### Parameter facts only accumulate

Stale evidence survives edits. Index contributions by source and replace/retract.

### Summary revision participates in semantic equality

Every generation propagates globally. Exclude nonsemantic revision fields.

### Unlimited context sensitivity

Memory/time explosion. Bound contexts and define merge.

### Block invocation assumed exactly once

Wrong for higher-order APIs unless summary proves cardinality/timing.

### Method mutation ignored

Call-target cache becomes stale. Add revision dependency or closed-world assumption.

### LSP heuristic target used by optimizer

Heuristic receiver confidence does not justify devirtualization without a runtime guard or sound proof.

## 26. Testing obligations

### Call graph

- exact instance target;
- class-side target;
- inherited target;
- `super` semantics;
- bounded union receiver multiple targets;
- selector missing on one alternative;
- dynamic pack/selector;
- reflective exact/unknown selector;
- method mutation invalidation where supported.

### Summaries

- parameter join from multiple callers;
- caller removal retracts fact;
- return propagation through call chain;
- unchanged summary stops propagation;
- changed summary reaches dependent callers;
- provenance/revision changes alone do not propagate.

### Recursion

- direct recursion;
- mutual recursion;
- recursive dynamic dispatch;
- budget/widening fallback;
- SCC/worklist termination.

### Higher-order

- literal block passed and invoked;
- block passed through helper;
- block never invoked;
- repeated invocation;
- block escapes/deferred if modeled;
- non-local return propagation.

### Incremental equivalence

For each edit sequence, compare final incremental summaries/parameter facts against a clean engine rebuilt from final source.

## 27. Review questions

1. What is the call-node identity?
2. How are ordinary Phalcom targets resolved without duplicating dispatch semantics?
3. How are unknown and bounded target sets represented?
4. What external behavior belongs in a summary?
5. How are call-site parameter contributions retracted after edits?
6. What re-enqueues a caller when a callee changes?
7. How does recursion converge?
8. Would SCC scheduling improve measured performance or only complexity?
9. What context sensitivity is used and why?
10. How are block invocation timing/cardinality represented?
11. How do reflection/native method mutation affect call graph validity?
12. Which summary fields participate in semantic equality?
13. How is final source/local fact publication kept coherent with summary solving?
14. What counters prove the invalidation frontier is proportional to semantic change?

A dependable interprocedural analyzer answers these before adding another inference heuristic.

# Flow and Interprocedural Semantics

## 1. From local flow to whole-program relationships

Local flow tracks abstract state within one callable. Interprocedural analysis explains how calls transmit facts between callers and callees without repeatedly inlining whole bodies. The key abstraction is a **summary**: a compact semantic function/property of a callable, keyed by a stable callable identity and explicit assumptions.

A summary is not merely “the inferred return shape.” It can contain:

```text
parameter input facts / requirements
return fact
normal/abrupt completion summary
effects: fields/globals/captured writes, throws, dynamic send, callback invocation
callees/dependencies
receiver/dispatch assumptions
confidence/provenance
revision/generation
```

Only include dimensions that have a consumer; nevertheless design the schema so uncertainty is not hidden.

## 2. Abstract interpretation view

Let concrete states be `C` and abstract states `A`. An abstraction/concretization relation can be written:

```text
α : P(C) -> A
γ : A -> P(C)
```

For concrete semantic transformer `F` and abstract transformer `F#`, a sound over-approximation satisfies:

```text
F(γ(a)) ⊆ γ(F#(a))
```

Intuition: every concrete result represented by `a` remains represented after abstract execution. This relationship matters for checker/prover/optimizer-strength facts. An editor heuristic may intentionally be non-sound, but its trust level must prevent escalation.

`ValueShape` is a good example of an abstract domain over runtime value categories. It is not automatically a language type because its abstraction relation, precision policies, and purpose differ from typing judgments.

## 3. Product domains

Real analysis often uses a product of domains:

```text
AbstractValue = Shape × Constant × Nullability? × Provenance × Confidence
State         = Reachability × Map<BindingId, AbstractValue> × Effects
```

Do not force every dimension into `ValueShape`. Product domains can define componentwise order/join, with intentional reduction between components when sound. For example, known boolean `true` implies runtime shape `Bool`; losing the constant at a join need not lose the shape.

## 4. Call transfer

For a statically resolved call `c(args)`, a context-insensitive flow may:

1. evaluate arguments in normative order;
2. resolve dispatch candidate(s);
3. emit call-site contributions to parameter slots;
4. read current callee summary return/effects;
5. apply effects to caller state;
6. record dependency caller -> callee;
7. join candidate returns if multiple targets are possible.

Conceptually:

```text
CallTransfer(call, S) = apply_effects(S, ⊔ Summary(target_i).effects)
ResultFact(call)       = ⊔ Summary(target_i).returns
```

If target resolution is dynamic/unbounded, the result/effects must conservatively represent that boundary rather than pretending “no target found” means no effect.

## 5. Contribution ownership and retraction

Incremental interprocedural analysis must be able to remove old evidence. Suppose callers `A` and `B` contribute argument facts to parameter `P`:

```text
P = contribution(A) ⊔ contribution(B)
```

If `A` changes, recomputing `P` as `old_P ⊔ new_A` cannot remove `A`'s old contribution. Store contributions by source:

```text
Contrib[P][A] = fact_A
Contrib[P][B] = fact_B
Joined[P] = ⊔ values(Contrib[P])
```

Then replacement is exact. **CURRENT:** Phalcom's `ParameterContributions` follows this model, including a reverse `slots_by_source` index and recomputation only of touched slots. This is an important pattern for fields, effects, call edges, and future type constraints where retraction matters.

## 6. Recursion and SCCs

The call graph may contain cycles. Compute strongly connected components (SCCs) and solve recursive summaries to a fixed point.

```text
for SCC in dependency order:
    initialize summaries conservatively
    worklist = members(SCC)
    while worklist not empty:
        f = pop()
        new = analyze_body(f, current summaries)
        if new != summary[f]:
            summary[f] = widen_if_needed(summary[f], new)
            enqueue dependents inside SCC
```

For mutually recursive functions, no topological ordering inside the SCC exists. The fixed point is the semantics of the abstract system.

If the domain has bounded unions/finite-height components, termination may be automatic. If effects/types produce infinite ascending chains, define widening explicitly. A hard round cap is a latency budget, not a mathematical convergence argument.

## 7. Context sensitivity

A single summary per callable is context-insensitive. It joins all callers and may lose correlations. Context-sensitive variants might key summaries by:

- bounded receiver class set;
- selected argument abstract shapes;
- call string of bounded length;
- generic/type instantiation in future typing;
- effect/closure invocation context.

Each extra key increases precision and state explosion. Use measurements and concrete misprecision cases before adopting context sensitivity. Keep a widening/merging policy so editor memory remains bounded.

## 8. Dispatch-sensitive summaries

For dynamic OOP, calls depend on receiver and selector. A summary keyed only by selector is wrong because different classes can implement the same selector. Phalcom's current `CallableId` includes owner class and dispatch side, which is an appropriate source callable identity.

When receiver analysis yields a union of classes, dispatch each class according to runtime lookup rules, deduplicate resolved callable targets, then join summaries. Do not type-direct dispatch unless language semantics explicitly choose it.

## 9. Higher-order/block parameters

If a callable invokes a block/callable parameter, its summary should express that effect rather than eagerly analyze any literal block passed at one call site.

Example conceptual summary:

```text
mapLike(block):
    effects.invokes_parameters = {0}
```

At a call site with literal block argument, the caller can then compose the block's effects. **CURRENT:** Phalcom's summary machinery already tracks invoked parameters and block effects such as captured writes and non-local returns. Future richer effect systems should generalize this model rather than execute blocks unconditionally during source traversal.

## 10. Heap/field facts and aliasing

Field facts are harder than locals because many objects/paths can write the same storage concept. “Field `x` has shape Number because constructors write Number” is only valid under assumptions about other writes, reflection/native mutation, subclass behavior, and initialization.

Use explicit evidence categories and may-write effects. **CURRENT:** current field facts distinguish declaration initializers, constructor initialization, and general writes. This is useful provenance, but a future correctness checker must define whether these observations establish a type invariant or merely advisory shape knowledge.

Alias analysis determines whether a write through one reference can affect another. Avoid claiming strong heap refinements without an alias/escape model. A local `let p = self` can make `p.field = ...` a write to `self.field`; arbitrary dynamic sends/native code may be opaque.

## 11. Precision boundaries

Keep different causes explicit:

```text
DynamicBoundary       language permits unknown target/behavior
MissingDependency     source/package unavailable
AmbiguousDispatch     bounded candidates but no unique target
Widened               domain exceeded precision budget
AnalysisBudget        solver stopped for latency
UnsupportedConstruct  analysis not implemented yet
```

All might eventually render as `?` in a compact hover, but checker/prover/invalidation logic needs the distinction.

## 12. Diagnostics and provenance

A future diagnostic should be able to traverse evidence:

```text
x inferred Number
  <- call result foo()
     <- summary foo returns Number
        <- reachable return at foo.ph:42
```

Store causal edges/compact origins during solving. Do not plan to recover this chain later by re-running heuristic searches through ASTs.

## 13. Testing

Include:

- single resolved call parameter/return propagation;
- two callers contributing incompatible facts;
- remove one caller and verify old evidence retracts;
- direct recursion requiring repeated rounds;
- mutual recursion SCC;
- bounded union widening;
- dynamic send conservative effects;
- higher-order callable invokes literal block versus does not invoke it;
- receiver-union dispatch to multiple overrides;
- field writes from constructor plus later general write;
- incremental/full summary equivalence;
- determinism independent of map iteration order.

## 14. Review questions

1. What concrete behaviors does this abstract fact represent?
2. Is the analysis intended to be sound or advisory?
3. What is the summary key and context sensitivity?
4. Can old contributions be removed exactly?
5. What dependency edges are created?
6. How are recursive SCCs solved and what guarantees termination?
7. What does an unresolved/dynamic call do to effects?
8. Are higher-order callbacks modeled as effects rather than eagerly executed?
9. What provenance is retained for explanation?
10. Which fact changes should invalidate callers versus only editor presentation?

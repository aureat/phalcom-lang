# Dynamic-Language and Reflection Analysis

Static analysis for Phalcom cannot begin from a closed-world assumption that every possible call target, method table, module, or native effect is visible in one source snapshot. The language is deliberately dynamic: dispatch starts from runtime receiver identity, methods and classes are first-class reflective objects, blocks are values, packages can cross a Rust boundary, and future reflective capabilities may mutate behavior. The correct response is not to abandon static analysis. It is to make every closed-world assumption explicit, attach it to the fact that depends on it, and conservatively model the operations that can invalidate it.

This reference owns the abstract-analysis consequences of dynamic language behavior. It does not define Phalcom's normative dispatch semantics; load the language/object-model and semantic-model skills for that. It also does not make `ValueShape` into a type system. Here, runtime-shape evidence is one possible abstract input to dispatch analysis.

## 1. Start from the observable send, not from a guessed target

A source send conceptually contains more than a callee name. A useful abstract model is:

```text
Send = (
    receiver expression,
    canonical selector identity or selector family,
    argument evaluation,
    access context,
    lookup-start rule,
    fallback semantics,
)
```

For a concrete runtime receiver `r` and selector `s`, normative dispatch determines a target or dynamic fallback:

```text
Dispatch(r, s, context, world) -> target | fallback | failure
```

Static analysis instead has an abstract receiver `r#` and perhaps an abstract selector `s#`:

```text
Dispatch#(r#, s#, context, world_assumptions)
    -> finite targets + dynamic remainder + effects
```

The result needs a **dynamic remainder** rather than pretending a finite target set is complete. A useful representation is:

```rust
struct DispatchApproximation {
    known_targets: SmallVec<[CallableId; 4]>,
    remainder: DynamicRemainder,
    assumptions: AssumptionSet,
    provenance: DispatchProvenance,
}

enum DynamicRemainder {
    None,
    ReceiverUnknown,
    SelectorUnknown,
    WorldOpen,
    ReflectiveMutationPossible,
    NativeBoundary,
    RecoveryBlocked,
}
```

The exact Rust representation may differ. The invariant matters: `known_targets = {A.m, B.m}` is not equivalent to “these are all possible targets” unless the analysis can justify `remainder == None` under its stated world assumptions.

## 2. Possible-target soundness

Let `Targets(c)` be the concrete target selected by an execution state `c`, and let `γ(r#)` denote concrete receivers represented by abstract receiver `r#`. For a sound may-target analysis, every concrete target must be covered:

```text
∀ r ∈ γ(r#).
    concrete dispatch of (r, s) = t
    ⇒ t ∈ Targets#(r#, s#) ∨ DynamicRemainder# permits t
```

A finite candidate set is therefore safe only when one of these is true:

1. the receiver abstraction denotes only a closed finite set of classes/objects and method lookup is stable;
2. a runtime guard checks the receiver/world assumption before an optimized dispatch;
3. the analysis retains a dynamic edge that conservatively covers anything not enumerated.

A completion engine may deliberately violate this completeness requirement for ranking usefulness. It must then label the result advisory and must not feed the finite candidate set into checker proof or unguarded optimization.

## 3. Receiver abstraction and dispatch sides

Phalcom distinguishes instance-side and class-side behavior. Therefore `ClassId` alone is insufficient to describe a dispatch receiver. A minimum receiver abstraction needs the dispatch side:

```text
Instance(C)
ClassObject(C)
Module(M)
Callable(F)
Family(receiver#, base)
Union({receiver#...})
Unknown
```

This matches the broad shape of the current LSP `ValueShape`: it has separate instance, class-object, module, callable, family, union, and unknown cases. That domain is advisory runtime-shape knowledge, not a formal Phalcom type.

For a bounded union receiver:

```text
R# = Instance(A) ⊔ Instance(B)
```

analysis may resolve the same selector separately:

```text
T# = Resolve(A, s) ∪ Resolve(B, s)
```

and join return/effect summaries over those targets. A missing method on one alternative is not erased by success on another. The consumer decides how to surface it:

- completion can show members supported by some candidates, with ranking or confidence;
- a checker may require the send to be valid for every possible receiver alternative unless the language has an explicit dynamic escape;
- an optimizer can devirtualize only if the target is unique under a sound assumption or it emits a guard/deopt/fallback path.

## 4. Selector identity versus selector uncertainty

Never derive selector identity from type metadata. Phalcom's current dispatch is selector-based; future optional typing must remain non-dispatching unless a separate language decision explicitly changes that.

Static selector states should distinguish at least:

```text
ExactSelector(s)
BoundedSelectorSet({s1, ..., sn})
SelectorFamily(base, known_shape_constraints)
DynamicSelector
MalformedOrBlocked
```

Dynamic argument packs and computed labels are particularly important. If a spread can change arity or labels, the analyzer cannot fabricate one canonical selector because it is convenient. It should retain a selector family or a dynamic-selector remainder.

Example:

```text
receiver.foo(*args)
```

If analysis only knows that `args` may be two or three positional values, a bounded analysis could preserve two selector shapes. If labels are unknown or the set grows beyond budget, widen to a dynamic selector state. That loss of selector precision is different from an unknown receiver.

## 5. A send is an effectful operation even when lookup is known

Resolving a target does not make the call pure. Analyze a send as two layers:

```text
lookup effects/assumptions
+
callee summary effects
```

Lookup itself may depend on mutable method/class state. Invocation may:

- mutate fields or globals;
- call supplied blocks;
- return non-locally through a block;
- throw;
- yield a fiber;
- invoke reflection;
- perform IO;
- cross FFI;
- mutate dispatch state, if the reflective API permits it.

Therefore “target known” and “state preserved” are unrelated facts.

## 6. Missing-message/fallback semantics

Many Smalltalk-like languages provide a missing-message mechanism. Do not assume Phalcom's exact fallback from precedent; inspect the normative object-model/runtime specification before implementation. The analysis architecture should nonetheless separate these cases:

```text
Definitely resolves ordinary target
Definitely reaches specified fallback
May resolve target or fallback
Definitely fails according to language semantics
Unknown because world/selector is dynamic
```

If Phalcom's governing semantics route failed lookup through a hook, the call graph should model that hook as an executable target/effect path. A linter may still diagnose a likely miss, but checker/prover reasoning must follow the actual dynamic behavior.

## 7. Reflective invocation

First-class method objects and reflective sends create different precision cases.

### Exact reflected method

If analysis has an exact `MethodId`/`CallableId`, reflected invocation can use that callable summary, subject to normative receiver/binding/access rules.

### Exact selector, abstract receiver

Equivalent to ordinary dispatch analysis over that receiver abstraction.

### Bounded selector set

Resolve all combinations within the budget:

```text
Targets# = ⋃_{r ∈ receivers#} ⋃_{s ∈ selectors#} Resolve#(r, s)
```

### Unknown selector or receiver

Introduce a dynamic-call effect. Do not interpret “not resolved by static analysis” as “does nothing.”

For correctness consumers, the dynamic-call effect should conservatively cover every state component callable through that boundary can mutate. For an LSP, a separate heuristic path may remain useful.

## 8. Reflective method-table mutation and world assumptions

If Phalcom permits adding/replacing/removing methods through reflection, a target fact must carry a stability condition. The abstract problem is temporal:

```text
At program point p:
    lookup(receiver_class, selector, world_revision = w) = method M
```

That does not imply the same lookup later if user code can mutate the method table between the points.

A robust analysis records an assumption such as:

```text
DependsOn(ClassSurfaceRevision(C))
DependsOn(MethodTableRevision(C))
DependsOn(CoreSurfaceRevision)
```

or, for intra-execution optimizer facts, a runtime guard/token:

```text
method_table_epoch(C) == expected_epoch
```

Any operation with `reflective_mutation` effect invalidates the relevant assumption set. The safest first implementation can globally havoc dispatch-surface assumptions; later work may narrow the invalidation scope to affected classes/selectors.

### Do not cache only by receiver class and selector

This is incomplete:

```text
key = (ClassId, Selector)
value = CallableId
```

unless cache validity also includes the dispatch-world state. A correct cache contract states:

```text
key: receiver lookup identity + selector + access/lookup context
value: resolved target
validity: all method-table/superclass inputs unchanged
invalidation: mutation of any governing dispatch surface
concurrency: publication/epoch policy
memory bound: bounded entries or owned by generation
```

The runtime inline-cache design and static semantic cache may use different representations, but both need explicit validity.

## 9. Open-world modules and packages

A workspace snapshot rarely proves that no additional module/package/native extension exists at runtime. Distinguish world policies:

```text
ClosedSnapshot
    exactly the declarations in a fixed semantic snapshot

ClosedProject
    all project/package dependencies resolved and sealed by policy

OpenRuntime
    runtime loading/reflection can introduce behavior outside analysis

GuardedClosedWorld
    optimizer assumes closed world under a runtime version/token check
```

The LSP normally works on a snapshot but should not silently treat it as a proof of runtime closure. A future checker/prover can define project modes that make stronger closure guarantees.

## 10. Native and FFI boundaries

Native Rust code is not “pure because the analyzer cannot see its body.” It is the opposite: absent a trusted summary, it is an opaque executable boundary.

A native summary may state:

```text
NativeSummary {
    return_fact,
    reads,
    writes,
    throws,
    may_yield,
    blocks_thread,
    escapes_arguments,
    retains_callbacks,
    mutates_dispatch,
    invokes_callback_positions,
    trust_origin,
    version,
}
```

The summary is trusted input to sound analyses only if native implementation and declaration contract are kept in sync and tested. At an unsummarized boundary, use a conservative envelope appropriate to the consumer.

## 11. Unknown-call havoc should be scoped, not magical

A common unsound shortcut is:

```text
unknown call -> return Unknown, keep everything else unchanged
```

The return shape is only one effect. Define a havoc function:

```text
Havoc : State# × DynamicEffectEnvelope -> State#
```

For example:

```text
Havoc(state, UnknownUserCall):
    preserve immutable lexical values
    preserve facts about unescaped fresh values when justified
    forget mutable globals reachable by call
    forget mutable heap cells reachable through passed/escaped aliases
    invalidate dispatch assumptions if reflection can occur
    mark throw/yield according to envelope
```

The initial implementation may be coarser. The key is that every preserved fact has a reason.

## 12. Provenance for dynamic uncertainty

Do not flatten all uncertainty into `Unknown`. Useful reasons include:

```text
ReceiverWidened
SelectorDynamic
TargetSetOpenWorld
MethodTableMayMutate
NativeSummaryMissing
ModuleDependencyMissing
MalformedSourceBlockedDispatch
AnalysisBudgetExceeded
ExplicitDynamicLanguageBoundary
```

A future diagnostic can then say why a send cannot be verified rather than misleadingly saying that the source “has no type information.”

## 13. Current Phalcom mapping

As of the inspected repository baseline (2026-08-14), the LSP semantic engine already provides several mechanisms this reference should build on:

- `ValueShape` distinguishes instance/class-object/module/callable/family/union/unknown runtime shapes;
- receiver-aware dispatch resolution is centralized in the semantic subsystem;
- dynamic packs are tracked as dynamic call-site conditions;
- callable summaries record a `dynamic_send` effect;
- summaries retain direct callable dependencies;
- source edits propagate through callable and module dependency structures;
- published semantic state is generation-coherent.

These are **CURRENT advisory-analysis mechanisms**. They are not a proof that all runtime dynamic behavior has been conservatively modeled for a future checker, prover, or optimizer. In particular, the current compact effect model is intentionally narrower than the effect envelope a sound optimization/proof analysis will eventually require.

## 14. Interaction matrix

| Dynamic feature | Shape/value analysis | Type/checker | Effect analysis | Optimizer | LSP |
|---|---|---|---|---|---|
| bounded receiver classes | join targets/results | verify send for required alternatives | join target effects | guarded/unique target possible | precise completion |
| unknown receiver | widen target space | dynamic/error policy | dynamic-call envelope | no unguarded devirtualization | heuristic fallback possible |
| dynamic selector | selector family/top | cannot assume one signature | dynamic-call envelope | no static direct call | broad suggestions/signature help degradation |
| reflective method mutation | invalidate target facts | respect open-world policy | reflective-mutation effect | epoch guard or abandon | invalidate member surfaces |
| native call | use trusted return summary | verify declared boundary | trusted/native conservative effects | optimize only with trusted contract | show declared/native surface |
| malformed source | recovery-specific uncertainty | do not reinterpret valid semantics | isolate blocked region | not optimizer input | remain useful locally |

## 15. Failure modes

Reject these arguments during review:

- “There are only two methods with this selector in the workspace, therefore the call has two targets.” Workspace enumeration is not necessarily runtime closure.
- “The receiver is probably `String`, so devirtualize.” Probability is not a sound dispatch assumption.
- “The method table has not changed during analysis.” That says nothing about whether executing intervening code can mutate it.
- “The reflected selector is a string, so resolve it statically.” Only if the abstract string is exact/bounded.
- “The native primitive has no source body, therefore no effects.” Missing information requires a summary or conservative boundary.
- “Dynamic send only makes the return unknown.” It may invalidate heap/global/dispatch facts and may throw/yield.
- “Type annotations can disambiguate methods.” That changes language dispatch semantics unless explicitly ratified.

## 16. Testing obligations

At minimum test:

1. receiver union resolves multiple targets and joins returns/effects;
2. one receiver alternative missing the selector remains visible to correctness analysis;
3. dynamic selector does not fabricate a precise target;
4. dynamic pack marks call uncertainty;
5. exact reflected callable reuses its summary;
6. unknown reflective call applies the expected havoc envelope;
7. dispatch-surface mutation invalidates cached target facts;
8. native summary absence is conservative;
9. native summary revision invalidates dependents;
10. class-side and instance-side dispatch do not merge accidentally;
11. malformed nearby syntax does not globally turn all calls dynamic;
12. incremental result after method-surface change equals a clean rebuild.

A particularly strong metamorphic test is:

```text
replace a statically direct-looking send by a semantically equivalent reflective
invocation with an exact selector/method

=> sound consumers should compute compatible target/result/effect facts,
   modulo provenance and deliberately different access semantics
```

## 17. Review questions

An implementation agent should be able to answer:

1. What proves that a finite target set is complete?
2. Which state can an unresolved call mutate?
3. Does this call invoke user code during lookup, argument conversion, or invocation?
4. Can executing this call change future dispatch?
5. What distinguishes an unknown receiver from an unknown selector?
6. Which world assumption makes the result valid?
7. How is that assumption invalidated incrementally and, if relevant, at runtime?
8. Is an LSP heuristic being promoted into checker/optimizer evidence?
9. Are type annotations affecting selector identity or lookup?
10. What provenance will explain why the target could not be resolved?

If those answers are absent, the dynamic-language analysis is incomplete.

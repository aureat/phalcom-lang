# Soundness, Precision, and Cost

Every static analysis is a contract among three constraints:

```text
soundness guarantee
precision target
resource budget
```

The mistake is not choosing a coarse or heuristic analysis. The mistake is failing to state which choice was made and then letting a stricter consumer act as though a stronger guarantee existed. Phalcom will have advisory editor inference, correctness-participating typing, proving, optimization, lints, and runtime modes. Those consumers can share semantic evidence while accepting different trust levels.

## 1. Soundness is relative to a modeled property

An analysis is not simply “sound” in the abstract. State:

```text
property being approximated
concrete semantics included
world assumptions
native/reflection assumptions
direction of approximation
consumer
```

For a may-reachability/value analysis:

```text
Concrete(p) ⊆ γ(Abstract(p))
```

Every concrete behavior is represented; the analysis may include extra behaviors.

For a must-property, the interpretation is different: reported fact must hold for all represented executions.

## 2. May versus must determines error direction

### May analysis

Question:

```text
Could this happen?
```

Examples:

```text
may throw
may call target M
may write field f
may be String
may escape to FFI
```

Over-approximation can cause false positives but must not omit real behaviors under sound assumptions.

### Must analysis

Question:

```text
Does this hold on every relevant execution?
```

Examples:

```text
definitely initialized
definitely non-None
definitely no alias
definitely unique target
definitely resource closed
```

At branch merge, must facts usually use intersection/meet-like behavior.

Confusing may and must is a semantic bug, not a small precision issue.

## 3. Soundness envelope

Document an explicit envelope:

```text
Sound provided that:
- source parses into supported semantic representation;
- modeled dispatch semantics match runtime;
- native summaries are correct;
- reflective mutation is either forbidden, modeled, or havoced;
- module world assumption holds;
- solver completes or conservative fallback is used.
```

If any assumption is violated, classify the result rather than silently continuing as though proved.

## 4. Advisory analysis

An advisory analysis can intentionally prioritize usefulness:

```text
receiver probably String -> offer String completions
```

This may use heuristics, bounded unions, incomplete workspace evidence, or confidence rankings. Its contract should be something like:

```text
best-effort editor assistance; not a proof of program correctness
```

The current Phalcom LSP `ValueShape` is explicitly in this category.

Advisory does not mean careless: deterministic results, coherent snapshots, provenance, and bounded cost still matter.

## 5. Correctness analysis

A checker/type analysis that rejects or accepts programs needs a more explicit policy:

```text
what does acceptance guarantee?
what happens at Dynamic/unchecked boundaries?
what does missing information do?
what does budget exhaustion do?
which declarations/native contracts are trusted?
```

It cannot copy an LSP widening policy if that widening causes invalid operations to be accepted.

## 6. Proof analysis

A prover needs a trivalent-or-richer outcome:

```text
Proved
Disproved(counterexample/model)
Unknown(reason)
```

Possible unknown reasons:

```text
Timeout
UnsupportedFeature
OpenWorldDispatch
MissingInvariant
NativeBoundary
Reflection
SolverUnknown
ResourceLimit
MalformedDependency
```

“Not disproved” is not “proved.”

## 7. Optimization analysis

An optimizer must preserve observable behavior. It may use:

```text
sound static fact
runtime profile + guard/deoptimization
language invariant
trusted closed-world assumption
```

but must encode the assumption. Example:

```text
receiver likely String
```

can enable speculative optimization only with a guard:

```text
if class(receiver) == String && dispatch_epoch == E:
    fast path
else:
    ordinary dispatch
```

Without the guard/proof, the fact is too weak.

## 8. Precision is not binary

Common dimensions:

```text
flow sensitivity
path sensitivity
context sensitivity
field sensitivity
heap/object sensitivity
relational numeric precision
selector precision
call-graph precision
module/world closure
provenance precision
```

An analysis can be precise in one dimension and coarse in another. State which dimension a proposed optimization improves.

## 9. Precision cliffs

Typical jumps:

```text
single class/Top
    -> finite class sets

path-insensitive
    -> selected branch refinement
    -> trace partitioning

context-insensitive summaries
    -> call-site context
    -> object-sensitive context

no heap
    -> escape only
    -> field-sensitive points-to
    -> context-sensitive heap

intervals
    -> octagons/polyhedra
    -> symbolic SMT reasoning
```

Each cliff multiplies state size or solver cost. Cross it because measured false positives/optimization misses justify it.

## 10. Precision budgets are semantic policy

A bound such as:

```text
MAX_SHAPE_UNION = 8
max path partitions = 16
max provenance origins = 4
solver step budget = derived bound
```

is not only a performance constant. It determines where precision is lost and how the fallback behaves.

Document:

```text
bound
why it exists
what widens at the bound
whether fallback is sound/advisory
how loss is recorded
which consumers may use the widened result
```

## 11. Deterministic budgets

Prefer deterministic resource limits over wall-clock cutoffs inside semantic algorithms:

```text
number of abstract states
worklist steps
union alternatives
path partitions
context depth
constraint nodes
provenance samples
```

Wall-clock cancellation is still necessary for editor responsiveness, but repeated analysis of unchanged input should not randomly produce different semantic answers because timing differed.

## 12. Conservative fallback

Suppose an analysis exceeds its budget. Correct fallback depends on direction.

May-analysis:

```text
widen to more possibilities / top
```

Must-analysis:

```text
drop unproven guarantees
```

Proof:

```text
Unknown(BudgetExceeded)
```

Optimization:

```text
abandon optimization or emit stronger runtime guard
```

LSP heuristic:

```text
show coarser result, perhaps with lower confidence
```

Never share one generic fallback without checking its meaning for the consumer.

## 13. False positive and false negative accounting

For a sound may analysis used to warn about possible errors:

```text
false positive possible
false negative should not occur within modeled envelope
```

For heuristic LSP completion:

```text
missing a valid suggestion is tolerable
showing an invalid suggestion may also be tolerable
```

For security analysis:

```text
silent false negative can be severe
```

Therefore the same abstract domain can need different thresholds and modes.

## 14. Precision provenance

Store where precision was lost:

```text
UnionCapExceeded(site)
LoopWidened(header)
CallTargetOpenWorld(site)
PathPartitionsMerged(point)
NativeSummaryMissing(call)
DependencyUnavailable(module)
SolverBudgetExceeded(SCC)
```

Diagnostics can then say:

```text
cannot establish `x` is String after loop
because the loop state widened at ...
```

instead of blaming absent annotations.

## 15. Trust lattice versus precision lattice

Trust and precision are different axes.

```text
Fact A: exact source annotation, but unverified at dynamic boundary
Fact B: coarse but sound may-shape from runtime invariant
Fact C: highly precise heuristic inferred from use sites
```

There is no one scalar “confidence” that totally orders these for every consumer. Model enough metadata that consumers can ask:

```text
is_trusted_for(Checker)
is_trusted_for(Optimizer)
is_precise_enough_for(Completion)
```

The current LSP confidence enum is useful for its advisory domain; future correctness consumers may require richer trust semantics.

## 16. Open-world precision

Closed-world assumptions can dramatically improve target analysis but are semantic assumptions, not free precision. Distinguish:

```text
workspace currently contains only A and B
```

from:

```text
runtime is guaranteed to contain no other applicable implementation
```

The first is an observation. The second requires a project/profile/module/reflection guarantee or runtime guard.

## 17. Cost model

Measure at least:

```text
CPU time
allocation count/bytes
hashing/interning cost
peak/retained memory
worklist iterations
summary recomputations
invalidated modules/callables
snapshot sharing/reuse
lock/wait time
query fan-out
p50/p95 editor latency
```

A “faster algorithm” that causes a wider invalidation frontier may lose overall.

## 18. Complexity versus semantic frontier

Incremental cost should trend with affected semantic dependencies:

```text
Cost(edit) ≈ O(changed body + changed summary dependents + necessary module frontier)
```

not always:

```text
O(entire workspace)
```

but precision in invalidation must remain correct. Rebuilding slightly too much is safe; failing to rebuild a dependent fact is stale-result unsoundness.

## 19. Caches need a full contract

Do not approve:

```text
cache[CallableId] = AnalysisResult
```

without:

```text
key
value
semantic dependencies
validity predicate
invalidation event
concurrency/publication policy
memory bound/eviction
semantic equality used to stop propagation
```

A cache is part of the analysis semantics because stale results change answers.

## 20. Performance optimization workflow

Use this order:

```text
1. Name observable semantic invariant.
2. Establish correctness tests.
3. Measure baseline.
4. Profile hot path / invalidation frontier.
5. Change algorithm or representation.
6. Re-run semantic equivalence tests.
7. Re-run benchmarks.
8. Add regression guard/counter where stable.
```

Do not optimize by weakening semantics without explicitly changing the analysis contract.

## 21. Security and robustness analyses

Security-sensitive analyses should prefer conservative models for sources/sinks/sanitizers and dynamic calls. Heuristic silence must never be represented as “safe.”

If malformed editor source blocks a taint path, record analysis incompleteness rather than certifying the value untainted.

## 22. Current Phalcom examples

The current semantic engine demonstrates several useful policies:

- bounded shape unions widen to `Unknown` for editor practicality;
- loop flow has a fixed iteration bound and a widening fallback;
- the interprocedural solver derives a deterministic budget and widens affected facts coherently if exceeded;
- semantic publication is atomic/coherent under cancellation;
- exact body invalidation can stop when callable summaries remain semantically unchanged;
- performance counters track solver/reuse/frontier behavior.

These are **CURRENT implementation choices for the advisory semantic engine**. They should be evaluated, not blindly copied, when future checker/prover/optimizer domains are added.

## 23. Failure modes

- Saying “sound” without naming the concrete property and assumptions.
- Using one `Unknown` for dynamicity, budget failure, missing dependency, contradiction, and bottom.
- Letting a union cap cause checker acceptance.
- Treating a timeout as proof success.
- Promoting heuristic target inference into unguarded optimization.
- Adding path sensitivity without measuring false-positive reduction.
- Adding a cache with no validity condition.
- Optimizing editor latency by publishing half-updated semantic products.
- Measuring only microbenchmarks while whole-workspace invalidation dominates.
- Increasing provenance detail until it dominates memory without a diagnostic consumer.

## 24. Review questions

1. Sound with respect to what concrete semantics?
2. Is this a may fact or a must fact?
3. What are the world/native/reflection assumptions?
4. What precision dimension is being improved?
5. What is the measured cost of that precision?
6. What deterministic bound guarantees termination?
7. What happens when the bound is exceeded?
8. Is the fallback conservative for this consumer?
9. What reason/provenance records the loss?
10. Can the fact be promoted to checker/prover/optimizer use, and why?
11. What invalidates a cached result?
12. Does incremental cost match the changed semantic frontier?

The review is incomplete until all three axes—soundness, precision, cost—have explicit answers.

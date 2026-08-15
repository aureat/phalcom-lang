# Testing Static Analyses

Static-analysis bugs are dangerous because a result can look plausible while being traversal-order dependent, stale, non-monotone, or unsound at one dynamic boundary. Testing must therefore cover more than example programs. Test the algebra, transfer functions, fixed-point solver, semantic lowering, interprocedural dependencies, incrementality, provenance, malformed input, and consumer trust boundaries.

A strong suite combines proof-like algebraic properties with executable examples and differential observations.

## 1. Test pyramid for an analysis

A useful layering is:

```text
Domain algebra/property tests
        ↓
Transfer-function unit tests
        ↓
CFG/structured-flow fixtures
        ↓
Interprocedural summary tests
        ↓
Incremental invalidation/retraction tests
        ↓
Consumer integration tests
        ↓
Fuzz/property/metamorphic tests
        ↓
Performance regression tests
```

A failing top-level hover test should not be the first signal that `join` is non-associative.

## 2. Algebraic laws

For join-semilattice domain `A`, property-test:

```text
join(a, a) = a                          # idempotence
join(a, b) = join(b, a)                 # commutativity
join(join(a,b), c) = join(a, join(b,c)) # associativity
join(bottom, a) = a                     # bottom identity
```

If top exists:

```text
join(top, a) = top
```

For order:

```text
a ⊑ a
(a ⊑ b and b ⊑ a) => semantic_equal(a,b)
(a ⊑ b and b ⊑ c) => a ⊑ c
```

For transfer `F`, sample monotonicity:

```text
a ⊑ b => F(a) ⊑ F(b)
```

This is not a mathematical proof over infinite domains but catches many implementation errors.

## 3. Widening properties

A widening should upper-bound the relevant inputs:

```text
a ⊑ a ▽ b
b ⊑ a ▽ b          # for common widening forms / according to documented contract
```

Test termination on representative ascending chains.

For interval widening:

```text
[0,0], [0,1], [0,2], ...
```

must stabilize under the implemented strategy.

For bounded unions, generate more than the cap and assert deterministic widening plus recorded reason if the domain supports it.

## 4. Canonical equality tests

Fixed-point and incremental propagation depend on semantic equality. Test that equivalent insertion orders canonicalize:

```text
Union[A,B] == Union[B,A]
EffectSet{f,g} == EffectSet{g,f}
Map labels canonical as required
```

unless source order is intentionally semantic.

Provenance-only changes should not trigger semantic change if the propagation contract excludes provenance from semantic equality. Test that distinction explicitly.

## 5. Transfer-function tests

Construct small abstract states and apply one operation:

```text
assignment
pattern bind
field write
known call
unknown call
branch refinement
return
throw
break/continue
block construction
block invocation
fiber yield
native call
```

Assert both value and effect/abrupt results.

Example:

```text
pre: x# = Int
stmt: x = "s"
post: x# = String
```

For mutable alias state, ensure old aliases observe weak/strong update policy correctly.

## 6. Evaluation-order tests

Static analysis must match language execution order. Build expressions where facts differ if order is swapped:

```text
f(x = 1, x)
receiverWithEffect().send(argWithEffect())
collection[*spreadWithEffect(), later]
```

Assert emitted events/provenance/state reflect lexical order exactly.

Do the same for abrupt completion: a throwing/returning earlier operand can make later operands unreachable.

## 7. Branch and reachability fixtures

Minimum set:

```text
simple diamond
diamond with one terminated branch
nested if
short-circuit and/or
trusted nominal test
contradictory test -> bottom
unsupported predicate -> no refinement
both branches write different values
```

Assert branch merge is independent of AST visitation order.

A powerful regression: deliberately reverse branch traversal in a test-only implementation and assert final join result is unchanged.

## 8. Loop fixtures

Cover:

```text
zero iterations
one iteration
multiple iterations
loop-carried value growth
break
continue
nested loop
return from loop body
throw from loop body
non-local return from invoked block
```

Assert:

- entry/zero-iteration path included where semantics permit;
- back-edge reaches fixed point/widening;
- exit state joins break + false-condition paths;
- no stale loop-body-only fact is treated as definite.

Instrument fixed-point iteration count in tests to catch accidental nontermination.

## 9. Interprocedural tests

Build a corpus with:

```text
A -> B
A -> B -> C
A -> {B,C} dynamic receiver union
self recursion
mutual recursion
higher-order callback
changed parameter contribution
callee return change
callee body change with unchanged summary
unknown/dynamic call
native summary
```

Assert worklist propagation reaches exactly the semantically affected callers where the architecture promises that granularity.

## 10. Stale contribution/retraction tests

This class of bug is easy to miss. Example:

```text
version 1:
    callerA -> f(1)
    callerB -> f("s")

parameter f.x = Int ⊔ String

version 2 removes callerB contribution:
    callerA -> f(1)

expected f.x = Int
```

A monotone accumulator that only adds facts will incorrectly retain `String`. Current Phalcom `ParameterContributions` explicitly supports source replacement/removal; keep regression tests for it.

## 11. Incremental equivalence is a first-class oracle

For every edit scenario:

```text
incremental_result(edit_sequence)
    ==
clean_full_analysis(final_source)
```

Compare semantic products, not allocation identity or generation counters.

Edit sequences should include:

```text
add file
remove file
rename declaration
body-only edit
source-range shift/comment insertion
import change
class/member surface change
core/native surface change
remove and re-add caller
edit recursion SCC
edit callback effect
```

This is one of the highest-value correctness tests for live semantic engines.

## 12. Snapshot coherence and cancellation

Simulate:

```text
start analysis generation N+1
cancel midway due to newer edit
publish generation N+2
```

Assert no consumer can observe a mixture such as:

```text
new class surface + old summary
new parameter facts + old call graph
```

The current engine uses candidate state plus atomic publication; test this invariant whenever new semantic products are added.

## 13. Differential testing against runtime observations

Generate or hand-write programs and compare sound may-predictions with observed runs:

```text
observed runtime class must be represented by shape fact
observed call target must be in target set/dynamic remainder
observed throw/write must be allowed by effect summary
```

Runtime samples cannot prove soundness because they cover only executions exercised. But a single observed behavior outside the abstract result is a concrete soundness bug.

## 14. Differential testing between analysis implementations

When migrating structured flow to CFG or replacing a solver:

```text
old_analysis(source) ≈ new_analysis(source)
```

for semantics both implementations support. Differences become intentional review items rather than silent behavior changes.

Keep old implementation as a temporary oracle or test-only reference where maintenance cost is reasonable.

## 15. Metamorphic properties

Metamorphic tests transform source in ways with known semantic effect.

### Alpha-renaming

```text
rename one local binding consistently
=> behavior and abstract values equivalent modulo BindingId/source provenance
```

### Formatting/whitespace

```text
semantically irrelevant formatting
=> semantic identities/facts equivalent where design promises identity stability
```

### Explicit annotation preservation

For future typing:

```text
insert a correctly inferred explicit annotation
=> checker result should remain valid; runtime dispatch unchanged
```

### Desugaring equivalence

```text
surface sugar
<-> canonical lowering
=> equivalent flow/effect result
```

### Unreachable branch

Adding a branch proven unreachable should not widen reachable-path facts, though diagnostics/provenance may change.

### Call extraction

Replace inline computation with helper carrying equivalent trusted summary; caller result/effect should remain equivalent.

## 16. Fuzzing domains and solvers

Generate random abstract elements and assert:

```text
join laws
canonicalization
no panic
bounded memory representation
widening termination
stable serialization/debug formatting if used in snapshots
```

For finite domains, exhaustive small-domain testing can prove algebraic laws over the enumerated set.

## 17. Fuzzing source/AST flow

Generate/reduce programs stressing:

```text
scope/shadowing
nested control flow
selectors/labels
packs/spreads
blocks/captures
non-local return
loops
module imports
reflection forms
collections/products
malformed syntax
```

Assertions:

```text
no panic
analysis terminates
snapshot coherent
result deterministic
all IDs/ranges valid under recovery contract
incremental final result equals clean rebuild where test harness can compare
```

## 18. Malformed-source tests

Live editor analysis needs cases such as:

```text
half-written call
missing closing block
incomplete selector labels
unknown import
partial class member
transient duplicate binding
incomplete future type arguments
```

Assert malformed region creates recovery uncertainty without poisoning unrelated declarations. Also assert complete invalid programs are not silently reinterpreted as valid semantics.

## 19. Dynamic/reflection tests

Cover:

```text
receiver union
unknown receiver
exact reflective selector
unknown reflective selector
dynamic pack
method surface mutation/invalidation if supported
class-side versus instance-side lookup
native dynamic boundary
```

Checker/optimizer tests should assert heuristic facts are rejected when insufficient.

## 20. Heap/alias/escape tests

When that analysis exists:

```text
must/may alias
strong versus weak update
loop allocation site
escape via return/global/closure/fiber/FFI
unknown-call havoc
fresh local preservation
```

Property-test monotone escape propagation.

## 21. Effects/concurrency tests

```text
closure construction != invocation
callback cardinality
synchronous vs deferred callback
non-local return propagation
may_yield vs blocks_thread
shared mutable havoc after yield
fiber-local fact preservation
native callback retention
```

These tests prevent value-only analysis from silently ignoring control/effect behavior.

## 22. Negative tests are essential

Test tempting invalid inferences:

```text
method named isString does NOT refine
workspace-only target enumeration does NOT prove closed world
one branch assignment does NOT imply definite assignment
solver timeout does NOT prove
unknown call does NOT preserve mutable global facts
heuristic receiver does NOT justify optimizer target
```

A good analysis suite tests what the analyzer refuses to conclude.

## 23. Provenance tests

For a derived diagnostic fact, assert explanation chain contains the relevant structural causes:

```text
parameter declaration
call-site contribution
callee return site
branch refinement
widening point
native contract
```

Do not snapshot entire verbose text if structured provenance can be asserted more robustly. Rendered diagnostics can have separate golden tests.

## 24. Determinism tests

Run same analysis with:

```text
different hash insertion order
different worklist seed order where semantics should be invariant
parallel query scheduling if supported
cold/warm cache
```

and compare canonical semantic results. Deterministic containers such as `BTreeMap/BTreeSet` can help but are not a substitute for algebraically deterministic joins.

## 25. Performance tests

Measure representative projects/fixtures, not only microbenchmarks:

```text
cold full analysis
single body edit
summary-changing body edit
surface edit
import edit
core/native edit
recursive SCC edit
large dynamic selector/union case
many editor queries against one snapshot
```

Counters worth tracking:

```text
callables visited
callables changed
modules recomputed
flow passes
solver rounds/steps
parameter slots touched/changed
published products reused
allocations/bytes
latency p50/p95
```

Set regression thresholds with noise awareness. Semantic tests remain the primary gate.

## 26. Soundness pressure harness

For selected small programs, enumerate bounded concrete executions where feasible and compare abstract collecting result:

```text
ConcreteReachable(p) ⊆ γ(Abstract(p))
```

This works especially well for tiny boolean/sign/finite-class domains and can catch missing branch/call effects.

## 27. Mutation testing

Deliberately inject solver mistakes in test builds or review thought experiments:

```text
replace join with right operand
skip zero-iteration path
ignore one predecessor
remove dynamic-call havoc
omit contribution retraction
stop after one recursive round
preserve field refinement across write
```

The suite should fail. If it does not, add a discriminating test.

## 28. Review checklist for a new analysis

Before merge, require evidence for:

```text
[ ] domain laws tested
[ ] transfer cases tested
[ ] branch/loop abrupt flow tested
[ ] termination/widening tested
[ ] dynamic/native boundary tested
[ ] interprocedural recursion tested if applicable
[ ] stale contribution removal tested
[ ] incremental = clean rebuild tested
[ ] malformed source tested for editor analysis
[ ] provenance/uncertainty reason tested
[ ] consumer rejects insufficient trust
[ ] performance counters/benchmarks checked where hot
```

## 29. Competency questions

1. Why can runtime differential tests find unsoundness but not prove soundness?
2. What algebraic law prevents branch visitation order from affecting join?
3. How do you test stale interprocedural evidence removal?
4. Which loop test catches forgetting the zero-iteration path?
5. Why should semantic equality ignore some provenance fields?
6. How do cancellation tests protect snapshot coherence?
7. What metamorphic transformation should preserve dispatch semantics after adding a type annotation?
8. How do malformed-source tests distinguish recovery uncertainty from real language error?
9. What negative test prevents heuristic LSP evidence from reaching the optimizer?
10. Which performance metric reveals an invalidation-frontier regression even if individual transfer functions got faster?

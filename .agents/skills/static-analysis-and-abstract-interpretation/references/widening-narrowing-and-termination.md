# Widening, Narrowing, and Termination

Termination is part of an analysis contract. “Usually converges” is not enough for an editor server, checker, or build tool. This reference explains why fixed-point iteration terminates on finite-height domains, why some useful domains have infinite chains, how widening and narrowing work, how recursive summaries should converge, and how a budget fallback can remain semantically conservative.

## 1. Why monotonicity alone does not guarantee practical termination

Suppose the abstract domain contains integer intervals ordered by inclusion:

```text
[0,0] ⊑ [0,1] ⊑ [0,2] ⊑ [0,3] ⊑ ...
```

The transfer for:

```text
x = 0
while condition {
  x = x + 1
}
```

can produce an infinite ascending chain. The least fixed point may be `[0,+∞]`, but naive Kleene iteration reaches it only in the limit.

A finite-height domain avoids this. If every strict ascending chain has bounded length, a monotone solver that only ascends must terminate after finitely many semantic changes.

## 2. Finite domains and engineered finite height

Many practical domains are engineered to be finite:

### Finite class set

If the semantic universe has `N` known classes:

```text
Shape = P(ClassId)
```

has finite height `N + 1` under subset inclusion, though the total number of elements is exponential.

### Bounded union

A capped union:

```text
Bottom
Singleton/Union(up to K alternatives)
Top
```

has bounded height even if the class universe can grow. When the cap is exceeded, the domain jumps to top.

CURRENT Phalcom LSP `ValueShape` uses this style with `MAX_SHAPE_UNION = 8`, widening oversized unions to `Unknown`. This is an explicit advisory performance policy.

### Bitvector domains

A finite set of facts represented by a bitset has finite height bounded by the number of bits.

## 3. Widening

A widening operator is written:

```text
a ▽ b
```

It is designed so that repeatedly widening an ascending sequence eventually stabilizes. A typical requirement is that it returns an upper bound of both operands:

```text
a ⊑ a ▽ b
b ⊑ a ▽ b
```

and satisfies a termination property for iterated widened chains.

Widening is not necessarily a lattice join. It may deliberately jump farther upward than the least upper bound.

### Interval widening

Classic interval widening:

```text
[l1, u1] ▽ [l2, u2]
  lower = l1 if l2 >= l1 else -∞
  upper = u1 if u2 <= u1 else +∞
```

Example:

```text
[0,0] ▽ [0,1] = [0,+∞]
```

The jump loses the upper bound but terminates.

## 4. Widening points

Do not widen everywhere. Common widening points are:

- loop headers/back-edge merges;
- recursive SCC summary heads;
- after a configured number of ordinary iterations;
- domain-specific growth thresholds.

Widening too early destroys useful precision. Widening too late can make editor/checker latency unpredictable.

### Delayed widening

A common strategy:

```text
perform k ordinary joins
then widen on further growth
```

This allows small finite loops to stabilize precisely.

Phalcom's current structured loop flow follows a related bounded strategy: iterate for a fixed number of flow iterations, stop if state equality stabilizes, otherwise widen loop state at the final iteration. When extending this mechanism, ensure the widening result covers entry/header/back-edge possibilities required by the domain.

## 5. Widening with thresholds

Intervals can retain common semantic bounds using thresholds:

```text
thresholds = {0, 1, 10, 100, ...}
```

Instead of jumping immediately from `[0,10]` to `[0,+∞]` when observing `11`, widen to the next threshold if appropriate.

Thresholds can be derived from program constants:

```text
x < 100
```

so the domain preserves `[0,100]`-like useful bounds.

Do not add threshold complexity to shape/union analyses that do not need it. This technique is for numeric/range domains.

## 6. Narrowing

After widening reaches a post-fixpoint, narrowing can recover precision.

If `y` is a widened safe over-approximation, apply a narrowing operator `△` or ordinary transfer iterations that move downward while remaining safe under the chosen scheme:

```text
y0 = widened post-fixpoint
y1 = y0 △ F(y0)
y2 = y1 △ F(y1)
...
```

Use a bounded number of narrowing iterations unless the domain has a termination proof.

### Example

A widening may produce:

```text
x ∈ [0,+∞]
```

while loop condition `x < 10` can narrow an exit fact to something closer to `[0,10]` or `[10,+∞]` depending on edge semantics.

Narrowing is a precision recovery phase, not a requirement for every analysis.

## 7. Widening bounded unions

Suppose shape alternatives grow:

```text
{A}
{A,B}
{A,B,C}
...
```

A cap-based widening:

```text
if |S| <= K: Classes(S)
else: Top
```

is simple and terminates.

### Advisory consumer

For completion/hover, top/unknown may mean “cannot narrow member surface.” Safe and useful enough.

### Correctness consumer

A checker cannot necessarily interpret overflow top as “all operations permitted.” If `Top` means any runtime value, a send valid only on String is not guaranteed. Conservative checking should reject/require dynamic boundary/guard unless the language type semantics explicitly allows it.

Same widening, different consumer policy.

## 8. Raw iteration caps are not convergence proofs

This is wrong:

```text
for _ in 0..10:
    state = transfer(state)
return state
```

unless `state` after iteration 10 is guaranteed to over-approximate every later iteration.

If the chain is ascending:

```text
a0 ⊑ a1 ⊑ ... ⊑ a10 ⊑ a11 ...
```

returning `a10` can exclude behaviors appearing at `a11`.

A valid budget policy needs a conservative fallback:

```text
if budget exceeded:
    return widen_to_safe_upper_bound(current, reason=BudgetExceeded)
```

and record the precision loss.

CURRENT Phalcom callable solver follows this principle for its advisory domain: when its derived work budget is exceeded, affected parameter/return facts widen to `ValueShape::Unknown` before coherent publication. A future correctness analysis must prove its own fallback is conservative in the required direction.

## 9. Derived budgets versus magic numbers

A budget can be based on domain structure:

```text
max semantic growth events
≈ number_of_nodes × domain_height
```

For interprocedural analysis, include:

```text
callables
parameter slots
dependency edges
abstract-domain height
context count
```

Phalcom's current solver derives a budget from callable count, parameter-slot count, possible dependency edges, and the shape-union bound. That is stronger engineering than an unexplained “100 iterations.”

Still, a derived budget is not a proof unless its formula actually bounds semantic growth. Treat it as an operational safeguard if that proof is not established, and keep the conservative overflow behavior.

## 10. Recursive call graphs

Recursion creates fixed-point equations over callable summaries.

Example:

```text
f(x) = if ... then 1 else g(x)
g(x) = if ... then "s" else f(x)
```

Summary equations:

```text
Ret(f) = Int ⊔ Ret(g)
Ret(g) = String ⊔ Ret(f)
```

A worklist seeded at bottom can converge to:

```text
Ret(f) = {Int, String}
Ret(g) = {Int, String}
```

### SCC solving

Compute strongly connected components in the call graph. Acyclic SCCs can be solved in dependency order. Recursive SCCs are iterated together until summaries stabilize/widen.

A dynamic call graph may itself evolve with value/receiver inference, so SCCs can change. Practical alternatives include a global deduplicating worklist with reverse dependencies, as CURRENT Phalcom LSP uses, or periodically recomputed SCCs when recursion/scale justifies it.

Do not claim SCC solving is mandatory if the existing worklist already provides correct convergence. SCCs are a scheduling/organization improvement, not the semantic essence.

## 11. Summary monotonicity

For ascending fixed-point iteration, summary updates should conceptually grow conservatively:

```text
S0 ⊑ S1 ⊑ S2 ...
```

However incremental source edits may remove old contributions and produce more precise/new summaries. This is a *new solve* with changed inputs, not one ascending chain across all repository history.

Within one solve, avoid unstable oscillation caused by noncanonical representation or non-monotone transfer.

### Common oscillation bugs

- union alternative order changes each round;
- provenance sample participates in equality and rotates;
- target resolution guesses differently as candidate count changes;
- summaries include current generation number in semantic equality;
- join alternates between structurally different but semantically equal forms.

Canonicalize and separate semantic fields from provenance/revision.

## 12. Widening effects and heap domains

Widening is not only for numeric values.

### Effect sets

Finite effect categories naturally terminate. But detailed sets of field IDs or call targets may grow without a practical bound. Widen:

```text
Writes({f1,...,fK}) -> WritesAnyFieldOf(ClassId) -> WritesAnyReachable
```

instead of immediately to “all effects.”

### Points-to sets

```text
Pts(x) = {Alloc1, Alloc2, ...}
```

can be capped and widened to region/class/unknown heap object. The chosen abstraction must preserve alias soundness.

### Provenance

Provenance growth does not usually belong in semantic convergence. Bound or intern it separately so an ever-growing explanation graph cannot prevent the analysis fixed point.

## 13. Context sensitivity and termination

Context-sensitive interprocedural analysis can create unbounded contexts if contexts include arbitrary call stacks.

Bound context abstraction:

```text
k-call-site context: last k call sites
object sensitivity: bounded receiver allocation abstraction
type context: canonical finite/limited type-argument abstraction
```

Context creation itself must have a termination/memory argument. A domain can be finite per context while total analysis diverges by generating new contexts forever.

## 14. Path partition bounds

Path-sensitive analysis has similar growth:

```text
2 branches -> 2 partitions
n independent branches -> up to 2^n partitions
```

Use:

- selected predicates;
- maximum partitions per program point;
- merge by similarity;
- dominance-scoped partitions;
- demand-driven splitting;
- widening to path-insensitive state.

When merging partitions, record that correlation was lost. A prover diagnostic can then distinguish “property false” from “analysis merged the relevant paths.”

## 15. Widening and malformed editor source

Incomplete code can create unstable semantic shapes during typing. Do not widen persistent workspace facts merely because a transient parse recovery generated many alternatives if those alternatives are recovery artifacts.

Separate:

```text
source recovery uncertainty
semantic domain top
budget widening
```

An editor analysis may quarantine facts from malformed regions or attach recovery provenance, preventing transient syntax from globally poisoning stable facts.

## 16. Widening and incremental analysis

A previously widened result should be recomputed when its dependencies change in a way that could restore precision. Do not treat top as permanently cached simply because it is a fixed point.

Example:

```text
callee previously dynamic -> caller return Unknown
callee becomes annotated/resolved -> caller can become String
```

Incremental invalidation must re-run the dependent even though `Unknown ⊔ String = Unknown` would not refine under a purely ascending solver seeded from old output. New revision solves should seed/recompute according to changed input semantics, not simply accumulate previous abstract output.

CURRENT contribution-indexed parameter facts in Phalcom are an example of enabling precision recovery after edits by retracting old contributions.

## 17. Proof obligations and solver timeout

For proving, three outcomes must remain distinct:

```text
Proved
Disproved / counterexample
Unknown (timeout, unsupported theory, abstraction too coarse, solver unknown)
```

Widening can produce an invariant too weak to prove a postcondition. That is not disproof.

If abstract interpretation supplies loop invariants to an SMT prover, retain provenance such as:

```text
Invariant widened at loop L after N iterations
```

so diagnostics can explain why proof failed.

## 18. Diagnostics for convergence

Make internal tracing inspectable:

```text
analysis ID
program point / callable ID
iteration or worklist step
old semantic state
new semantic state
changed components
widening applied?
widening reason
queue additions
dependency edge causing requeue
budget remaining
```

Do not log full huge maps on every production request. Provide structured counters and opt-in traces.

CURRENT Phalcom LSP already exposes performance counters such as solver rounds, callable steps, parameter slots touched/changed, recomputed callables, and reused published products. Extend counters with domain-specific widening/iteration metrics when new analyses are added.

## 19. Testing widening and termination

### Domain tests

```text
a ⊑ widen(a,b)
b ⊑ widen(a,b)
```

Construct known infinite chains and assert stabilization under widening.

### Loop tests

- exact fixed point before widening;
- widening-required numeric/union growth;
- zero iteration;
- break/continue;
- nested loops;
- widening does not make exit unreachable;
- changed source can recover precision from previously widened result.

### Recursive tests

- direct recursion;
- mutual recursion;
- recursive higher-order blocks;
- dynamic call inside recursive SCC;
- unchanged summary stops propagation;
- changed summary reaches all and only semantic dependents.

### Budget tests

Force budget exhaustion and assert:

```text
result is conservative
precision-loss reason recorded
publication coherent
no partial round leaks
consumer does not interpret fallback as proof
```

## 20. Failure modes

### “Ten iterations is enough in practice”

Not a semantic argument. Provide domain/widening fallback.

### Widening to a value that is not an upper bound

Unsound. Test order relation.

### Widening every join

Terminates but may collapse precision immediately. Restrict widening points.

### Never revisiting a widened fact after edit

Stale imprecision. New dependency revision must permit recomputation/refinement.

### Provenance causes nontermination

Exclude or separately bound nonsemantic metadata from fixed-point equality.

### Recursive analyzer calls itself directly

Can stack overflow and loses global convergence control. Use summaries/worklist/SCC.

### Timeout accepted as success

Invalid for checker/prover/security. Preserve unknown/budget result.

## 21. Review questions

1. What ascending chains can this domain produce?
2. Is domain height finite under real workspace growth?
3. Where exactly is widening applied?
4. Does widening upper-bound both inputs?
5. Is widening delayed or thresholded, and why?
6. Can narrowing recover useful precision?
7. What is the recursion convergence strategy?
8. Can context/path creation be unbounded even if each domain value is finite?
9. What happens when the resource budget is exceeded?
10. Is fallback conservative for every consumer?
11. Can a later edit recover precision from a widened cached result?
12. Are provenance/revision excluded from semantic fixed-point equality?

Termination is complete only when both semantic convergence and resource-bounded implementation behavior are specified.

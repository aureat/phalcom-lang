# Abstract Knowledge, Dataflow, Joins, and Fixed Points

This reference owns the mathematical model behind Phalcom semantic approximation. It is
not a substitute for the future language type system. Its job is to make an implementation
agent able to define a fact domain, prove that its merge behavior is conservative, solve
flow/interprocedural equations, recognize when widening is required, and distinguish
analysis convergence from incremental-edit retraction.

## 1. Concrete behavior and abstract knowledge

Let `Σ` be the set of concrete runtime states relevant to an analysis. Exact execution
moves between concrete states with a transfer function `F`:

```text
F : P(Σ) -> P(Σ)
```

A static analysis does not usually carry arbitrary sets of concrete states. It carries an
abstract value `a` from an abstract domain `A`. The abstraction and concretization maps are
conceptually:

```text
          α
P(Σ) ------------> A
  ^                |
  |                |
  +----------------+
          γ
```

`α` summarizes concrete states. `γ(a)` denotes the concrete states represented by an
abstract fact `a`. The implementation rarely materializes either mathematical function;
they define what the representation *means*.

An abstract transfer `F# : A -> A` is sound when it never omits a concrete behavior the
analysis claims to model. A useful soundness condition is:

```text
F(γ(a)) ⊆ γ(F#(a))
```

or, equivalently under a suitable Galois connection:

```text
α(F(S)) ⊑ F#(α(S))
```

The symbol `⊑` below means "at least as precise as / represents no more concrete
possibilities than". Thus a concrete class shape is below a union, and a union is below
unknown:

```text
Instance(Point) ⊑ Union(Point, Circle) ⊑ Unknown
```

Do not quote the equations decoratively. When introducing a new fact domain, say what its
concrete meaning is and what behavior the abstraction is allowed to forget.

## 2. A domain contract is mandatory

For each analysis domain `D`, define at least:

```text
carrier values        what facts can exist
concretization        what runtime/program states each fact represents
precision order ⊑     when one fact is no less precise than another
bottom ⊥              impossible/no reachable concrete state, if represented
top ⊤                 reachable but maximally imprecise, if represented
join ⊔                 least conservative fact covering two alternatives
transfer              how each construct transforms facts
fixed-point equality  when two computed facts count as semantically unchanged
widening ▽            termination accelerator, if needed
provenance policy     how evidence survives transfer/join
```

If the implementation cannot state these, it has not defined a flow analysis yet; it has
only defined a data structure.

### Current Phalcom example: `ValueShape`

**CURRENT:** `phalcom-lsp/src/semantic/facts.rs` defines `ValueShape` as advisory runtime
value knowledge and explicitly says it is not a language type. Its alternatives include
`Unknown`, module-qualified instances/class objects, modules, structural collection shapes,
callables/families, and bounded unions. `MAX_SHAPE_UNION` is currently `8`; oversized
unions widen to `Unknown`.

For this domain, `Unknown` behaves like a shape-top: it means no useful runtime-shape
knowledge. It is not an unreachable state and is not the future type system's `Any`,
`Dynamic`, or bottom.

## 3. Join is a semantic least upper bound

At a branch merge, a fact must cover every reachable predecessor. If `a` and `b` are
predecessor facts, their join `a ⊔ b` should satisfy:

```text
a ⊑ a ⊔ b
b ⊑ a ⊔ b

and for every c:
  if a ⊑ c and b ⊑ c, then a ⊔ b ⊑ c
```

The last condition makes the join the *least* upper bound: conservative, but no less
precise than necessary in the chosen domain.

Example:

```phalcom
let x
if cond {
  x = Point.new()
} else {
  x = Circle.new()
}
use(x)
```

The post-merge possible-shape fact is conceptually:

```text
Instance(Point) ⊔ Instance(Circle)
  = Union(Instance(Point), Instance(Circle))
```

It is not the last branch visited, and it is not "both classes definitely at once".

### Algebraic laws worth testing

A join used in order-independent dataflow should normally be:

```text
idempotent:   a ⊔ a = a
commutative:  a ⊔ b = b ⊔ a
associative:  (a ⊔ b) ⊔ c = a ⊔ (b ⊔ c)
```

If the representation stores a bounded sample of provenance, *the semantic value* can obey
these laws even when diagnostic evidence ordering is deliberately canonicalized or capped.
Do not let incidental vector insertion order determine fixed-point equality.

## 4. Structural joins and deliberate widening

Useful shape policies include:

- same nominal identity -> unchanged;
- list/set/range -> join element/bound shape;
- map -> join keys and values independently;
- tuple -> elementwise join when arity matches;
- record -> fieldwise join only when label structure is compatible;
- callable -> exact only for the same callable identity; otherwise union/widen;
- family -> preserve the base selector only when compatible, joining receiver knowledge;
- incompatible forms -> bounded union;
- union larger than the configured cap -> `Unknown`.

**CURRENT:** the existing `ValueShape::join` follows this general structure and caps a
union at eight alternatives. Current semantic tests exercise a callable with nine
incompatible return shapes and require widening to `Unknown`.

This is an editor/runtime-shape policy. A future language type union may need completely
different normalization, subtyping and canonicalization. Never reuse a union cap merely
because both representations contain a `|`-like concept.

## 5. Bottom, unknown, and reachability

The most damaging dataflow bug is often confusing "no state reaches here" with "a state
reaches here but we know little about it".

```text
⊥ / Unreachable     γ(⊥) = ∅
Unknown / shape-top γ(Unknown) = all represented runtime shapes
```

Suppose one branch returns:

```phalcom
if cond {
  return Point.new()
} else {
  x = Circle.new()
}
use(x)
```

Only the `else` branch contributes to the normal continuation. Joining the unreachable
`then` exit as if it contained `Unknown` would incorrectly destroy `Circle` precision.

**CURRENT:** Phalcom's semantic flow machinery represents reachability separately from
`ValueShape::Unknown`. Preserve that distinction as domains evolve.

## 6. Dataflow equations over a control-flow graph

For a forward analysis over basic blocks `B`, with predecessors `pred(B)`:

```text
IN[B]  = ⊔ { OUT[P] | P ∈ pred(B) and P reaches B }
OUT[B] = F_B(IN[B])
```

`F_B` is the block transfer function. For a backward analysis, the direction reverses and
successors feed the equation.

A domain over environments is usually pointwise. Let an environment map binding identities
to values:

```text
Env = BindingId -> ValueFact
```

A may-shape join can be defined:

```text
(E1 ⊔ E2)(x) = E1(x) ⊔ E2(x)
```

but only after deciding what *absence* means. If a binding absent on one reachable path
means "not definitely assigned", then treating absence as "ignore this predecessor" is
unsound for definite-assignment analysis. Different analyses need different products or
polarities.

## 7. Transfer functions and monotonicity

A transfer function `F` is monotone when:

```text
a ⊑ b  =>  F(a) ⊑ F(b)
```

Intuitively: giving the analyzer less precise input must not produce an unjustifiably more
precise output.

Examples:

```text
x = expr:
  v = analyze(expr, state)
  state[x] = v

return expr:
  emit return evidence for analyze(expr, state)
  normal successor = unreachable

if predicate:
  true_in  = refine_true(state, predicate)
  false_in = refine_false(state, predicate)
  analyze each reachable branch
  join normal exits
```

A transfer that says "if receiver is Unknown, assume String because that helps completion"
is not a semantic transfer. That may be a consumer ranking heuristic, but it cannot be
stored as exact semantic knowledge.

## 8. Worklist solving and least fixed points

Loops and recursive dependencies create equations in which outputs feed later inputs. A
standard forward worklist solver is:

```text
for each node n:
    IN[n]  = ⊥
    OUT[n] = ⊥

seed entry
worklist = [entry]

while worklist not empty:
    n = pop(worklist)
    new_in  = join(OUT[p] for p in pred(n))
    new_out = transfer_n(new_in)

    if new_in != IN[n] or new_out != OUT[n]:
        IN[n]  = new_in
        OUT[n] = new_out
        enqueue successors(n)
```

On a finite-height lattice with monotone transfers, ascending iteration terminates at a
least fixed point. In practical compiler domains, finite height may be created by design:
bounded unions, bounded structural depth, finite symbol sets, or explicit widening.

### Worked loop

Consider:

```phalcom
let x = A.new()
while cond {
  if p { x = B.new() }
}
use(x)
```

A simplified loop-header equation is:

```text
H = Instance(A) ⊔ BodyOut(H)
```

Iteration might be:

```text
H0 = ⊥
H1 = Instance(A)
H2 = Union(A, B)
H3 = Union(A, B)   fixed point
```

A single pass that sees only `A` at the header is insufficient because the back-edge
carries `B` into the next iteration.

## 9. Widening and narrowing

When an ascending chain can grow indefinitely, a widening operator `▽` accelerates
convergence:

```text
a_{n+1} = a_n ▽ F(a_n)
```

It must remain conservative, but it need not be the least upper bound.

Practical policies include:

- bound union alternatives;
- bound tuple/record/container nesting depth;
- summarize recursively expanding shapes;
- collapse repeated mutation to a top-like fact;
- cap proof/path predicates;
- cap provenance samples independently from the semantic value.

Widening is an explicit precision loss. Record enough debug metadata to explain where a
fact widened when that affects developer-visible behavior.

A later narrowing pass may regain precision after reaching a post-fixed point, but do not
confuse abstract-interpretation narrowing with ordinary branch type refinement. They are
related ideas with different contracts.

## 10. May and must analyses have opposite merge intuition

Ask whether the property is existential over paths (may) or universal over paths (must).

Examples:

```text
may throw                    join = OR
may have runtime class C     join = union
may write field f            join = set union

definitely initialized      merge = AND/intersection across reachable paths
definitely satisfies P       retain P only if every predecessor proves P
available expression         merge = intersection with compatible definitions
```

A common unsound shortcut is to use the shape-domain union rule for every fact. Products
of facts can contain components with different orders and joins.

## 11. Product domains

Real semantic states combine several facts:

```text
State = Reachability × Env × Effects × PathFacts × ...
```

Define product order componentwise:

```text
(a1, b1) ⊑ (a2, b2)
iff a1 ⊑A a2 and b1 ⊑B b2
```

and product join componentwise *only when each component's semantics permit it*.

This matters in Phalcom because a callable summary can carry return knowledge,
dependencies and effects. The return shape may join by runtime alternatives while
`can_throw` joins by logical OR. Treating the summary as one opaque "confidence" value
loses semantics.

## 12. Interprocedural fixed points

A callable summary can be modeled as an abstract transformer or a summary fact. A simple
summary form is:

```text
S_f = (ParamFacts_f, ReturnFact_f, Effects_f, Dependencies_f)
```

A call site uses the current summary of its target. If analyzing `f` changes `S_f`, every
caller whose analysis depends on `S_f` must be reconsidered.

Conceptual algorithm:

```text
queue = changed callables
while queue not empty:
    f = pop(queue)
    old = summary[f]
    new = analyze_body(f, summaries, parameter_facts)
    if new != old:
        summary[f] = join_or_replace_according_to_contract(old, new)
        enqueue callers_of(f)
```

For mutually recursive callables, SCC decomposition is often useful:

```text
A -> B -> C
     ^    |
     +----+
```

`B` and `C` form one SCC. Solve that component to stability before assuming its outputs are
final. A bounded iteration budget is a latency guard, not a proof that the abstract domain
converged. If the budget is exhausted, expose a conservative/budget-exhausted state or
widen according to an explicit policy.

**CURRENT:** Phalcom's semantic inference has an interprocedural worklist and a bounded
iteration budget. Current tests include recursive callables with concrete return evidence
that must converge.

## 13. Monotone solving is not the same as incremental editing

This distinction is essential.

Within one fixed source generation, an analysis often grows knowledge monotonically toward
a fixed point. Across *edits*, old evidence may disappear. For example:

```text
generation 10: caller contributes Cat to Service.consume(value)
generation 11: caller edited to contribute Dog instead
```

If the engine only performs monotone join:

```text
Cat ⊔ Dog = Cat | Dog
```

then the removed `Cat` observation becomes stale forever.

Incremental systems therefore need *contribution ownership* or recomputation from surviving
sources. Conceptually:

```text
Contrib[param_slot][source] = fact
Joined[param_slot] = ⊔ Contrib[param_slot].values()
```

Replacing one source is:

```text
remove old Contrib[*][source]
insert new contributions from source
recompute only touched joined slots
propagate deltas to dependents
```

**CURRENT:** `facts.rs` contains `ParameterContributions` indexed both by parameter slot and
`ContributionSource`; replacing a source removes its prior evidence and recomputes touched
slots. Current tests verify that editing a caller from `Cat` to `Dog` removes the stale
`Cat` contribution. This is a strong model for any future fact family that aggregates
retractable evidence across files or call sites.

## 14. Provenance is an evidence graph, not decoration

A useful model is a bounded DAG:

```text
FactId -> EvidenceNode
EvidenceNode = Source(range)
             | FromBinding(binding, parent)
             | FromCall(callsite, callee_summary)
             | Join(parents...)
             | Widen(reason, parents...)
```

The hot editor representation can use compact sampled origins, while a checker/prover mode
may retain a richer evidence arena. The important architectural rule is that provenance is
associated with the fact when derived; do not discard causal information and hope to
reconstruct it from source text when emitting a diagnostic.

**CURRENT:** `InferredValue` stores confidence plus a compact provenance vector capped at a
small number of origins. Preserve the principle even if the representation changes.

## 15. Fixed-point equality and canonicalization

Convergence must compare semantic values, not pointer identity or accidental insertion
order. Normalize domains so equivalent facts compare equal:

```text
A | B  == B | A     semantically
A | A  == A
join(A, Unknown) == Unknown     in the current shape domain
```

A vector-backed representation may still choose a deterministic canonical ordering.
Determinism matters for snapshots, tests, caches and editor stability.

Do not assume the same equality relation is appropriate for future language types. Type
semantic equivalence can involve alias expansion, normalization, alpha-equivalence,
subtyping or isomorphism—owned by type-system skills/specifications.

## 16. Precision budgeting

Editor analysis is latency-sensitive. A budget is legitimate only if its semantic meaning
is explicit.

Examples:

```text
union cap exceeded        -> widened shape + reason
recursive iteration cap   -> conservative summary / budget-exhausted marker
path predicate cap        -> drop selected predicates, never invent a proof
provenance cap            -> semantic fact unchanged; explanation sample truncated
```

Do not use a single `Unknown` to encode all four cases. They have different debugging,
diagnostic and future-checker consequences.

## 17. Testing obligations

For any new abstract domain or transfer:

1. Unit-test join laws where intended: idempotence, commutativity, associativity.
2. Test top/bottom behavior explicitly.
3. Test a branch whose predecessor order is permuted; the result must be semantically
   identical.
4. Test a loop requiring at least two iterations.
5. Test a recursive SCC with and without concrete evidence.
6. Test widening exactly at and above its threshold.
7. Test unreachable predecessors do not destroy precision.
8. Test may/must polarity with asymmetric branches.
9. Test an edit that *removes* evidence, not only adds evidence.
10. Test clean full analysis and incremental recomputation reach equivalent semantic facts.
11. Property-test domain normalization when practical.
12. Add performance guards for domains used on every editor keystroke.

## 18. Unsound shortcuts to reject

Reject designs that:

- merge branches by "last write wins";
- use `Unknown` for unreachable code;
- infer a must-property from one predecessor;
- assume one loop pass is enough;
- recurse directly through callees without a summary/termination policy;
- use a union cap without documenting what widening means;
- treat an iteration timeout as proof of stability;
- retain deleted call-site evidence because the join is monotone;
- store unbounded provenance chains in every fact;
- use heuristic completion evidence as checker proof;
- reuse `ValueShape` as the future language type lattice.

## 19. Review questions

An agent should be able to answer these before implementing an analysis:

- What concrete executions does each abstract fact represent?
- What is the precision order?
- What are top and bottom, and are they represented separately from reachability?
- Is the analysis may or must?
- What is the join and why is it conservative?
- Are transfers monotone?
- What makes the domain finite or what widening guarantees termination?
- Where is the fixed point: loop, callable SCC, module graph, or all three?
- What happens if the operational analysis budget is exhausted?
- Which evidence is retractable after an edit and who owns each contribution?
- How are provenance and widening reasons bounded?
- Can incremental recomputation be checked against a clean full rebuild?
- Why is this domain not automatically the Phalcom language type system?

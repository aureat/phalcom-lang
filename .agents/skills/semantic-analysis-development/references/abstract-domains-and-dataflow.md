# Abstract Domains and Dataflow Implementation

## Define the domain first

A dataflow implementation without a domain definition tends to become ad-hoc conditionals.
Write the domain contract in docs/tests before adding transfer code.

## Minimal domain interface

Conceptually:

```rust
trait AbstractDomain: Clone + Eq {
    fn bottom() -> Self;       // if analysis uses unreachable/no-state
    fn top() -> Self;          // unknown/no useful precision
    fn join(&self, other: &Self) -> Self;
    fn widen(&self, next: &Self) -> Self { self.join(next) }
}
```

Do not force a Rust trait if concrete code is simpler; preserve the semantics.

## Current shape domain

`ValueShape::Unknown` is top-like for runtime-shape knowledge.
`InferredValue::join` combines shape, lowers confidence, and bounds provenance.

Current union cap is an editor performance/termination policy, not a future language type
rule.

## Product domains

Many analyses are maps from identity to a value domain:

```text
FlowState = BindingId -> InferredValue
DefiniteAssignment = BindingId -> {Unassigned, Maybe, Assigned}
NullabilityRefinement = BindingId -> TypeRefinement
Effects = set/bitset of MayEffect
Intervals = BindingId -> [lower, upper]
```

The product join is pointwise, but missing-key semantics must be defined.

## May versus must

### May analysis

Example: possible runtime shapes.

Merge = union/join.

### Must analysis

Example: definitely initialized.

A binding is definitely initialized after merge only if initialized on *all* reachable
predecessors.

### Kill/gen analysis

Example: liveness, reaching definitions. Define transfer sets explicitly.

## Monotonicity

Fixed-point iteration assumes transfer functions do not oscillate unpredictably.
If adding a transfer can make a fact alternately more/less precise, reconsider state/order or
use a solver designed for non-monotone constraints.

## Widening policy

Potential resource limits:

- union alternatives;
- recursive shape nesting;
- provenance length;
- call graph iterations;
- path predicate count;
- constraint set size.

On cap, widen conservatively and record/debug that widening occurred where useful.

Never truncate by dropping arbitrary alternatives while still claiming exactness.

## Canonicalization

Deterministic equality is essential for convergence.

For set-like unions:

```text
flatten nested unions
remove duplicates
sort/canonicalize stable ordering
single element -> element
empty -> defined bottom/top policy
```

Current code may preserve insertion/order based on deterministic traversal; future type unions
will likely require stronger canonical form.

## Provenance domain

Treat provenance as metadata, not part of semantic precision equality unless necessary.
Otherwise every newly discovered path can prevent convergence even when the semantic value is
unchanged.

A summary may compare semantic value/effects while retaining a bounded latest/representative
evidence set separately.

## Confidence

Current confidence ordering can degrade under joins:

```text
Exact > Flow > Interprocedural > Heuristic
```

Verify actual enum ordering/`join` implementation before relying on comparison semantics.

Future checker judgments should not reuse heuristic confidence as proof strength.

## Transfer-function tests

Unit-test joins independently from AST walking:

- idempotence: `x join x = x`;
- commutativity where domain requires it;
- associativity where expected;
- top absorption;
- bottom identity where represented;
- union cap/widening;
- structural tuple/record/map joins;
- confidence/provenance bounds.

These tests make later flow failures easier to diagnose.

## Worklist implementation

For explicit CFG:

```rust
while let Some(block) = worklist.pop() {
    let input = join_predecessors(block);
    let output = transfer_block(block, input);
    if output != old_output {
        old_output = output;
        enqueue_successors(block);
    }
}
```

Use compact block IDs and vectors where block graph is dense.

## SCC implementation

For recursive callables:

1. build callable dependency graph from resolved calls;
2. find SCCs (Tarjan/Kosaraju or existing graph helper);
3. solve acyclic SCCs once after dependencies;
4. iterate recursive SCCs to fixed point;
5. widen/cap;
6. propagate changed summary to reverse dependents.

Do not recompute entire workspace if only one SCC changed unless correctness currently requires
that conservative fallback.

## Formal lattice model

An abstract domain is not merely an enum with a `join` method. For forward may-analysis, use a partially ordered set `(A, ⊑)` where `a ⊑ b` means every concrete behavior represented by `a` is also represented by `b`. A join `a ⊔ b` should be the least upper bound when the domain has one, or a documented conservative upper bound when implementation caps make the exact LUB impractical.

A complete lattice has `⊥`, `⊤`, arbitrary joins/meets; many implementation domains only need a finite-height join-semilattice plus an explicit widening policy. State what you actually rely on.

For finite CFG dataflow:

```text
IN[B]  = ⊔ { OUT[P] | P ∈ pred(B), P reachable }
OUT[B] = F_B(IN[B])
```

The global transfer functional `F` acts on the product of all block states. If the domain has ascending-chain condition and every transfer is monotone, iterative application from `⊥` terminates at the least fixed point:

```text
X₀ = ⊥
Xₙ₊₁ = F(Xₙ)
lfp(F) = ⋁ₙ Xₙ
```

For infinite-height domains, use widening to force stabilization.

### Why monotonicity matters

Monotonicity means:

```text
a ⊑ b  ⇒  F(a) ⊑ F(b)
```

As the analysis learns about more possible incoming behavior, transfer may not claim a strictly narrower set of possible outgoing behaviors. A non-monotone transfer can oscillate and invalidates standard worklist convergence reasoning.

A common accidental violation occurs when an implementation chooses “the first observed class” as a guess, then replaces it with another guess depending on traversal order. That is not a stable abstract interpretation. Use a join or explicitly classify the result as an order-dependent heuristic outside the sound domain.

## Galois connection intuition

For a concrete domain `C = P(Σ)` and abstract domain `A`, a Galois connection satisfies:

```text
α(c) ⊑ a    iff    c ⊆ γ(a)
```

You rarely encode `α` and `γ` in Rust. Their value is as a design test: if `ValueShape::Instance(C)` is intended to represent all runtime values whose runtime class is `C`, then `join` must return a shape whose concretization contains both operand sets.

This immediately explains why dropping an alternative from an oversized union is unsound while widening to `Unknown` is conservative.

## Bottom has two different uses

Do not overload one bottom-like value for unrelated meanings:

```text
Unreachable            // no execution reaches this program point
NoObservedEvidence      // analysis has not collected any evidence yet
ImpossibleType/Fact     // constraint set has no model
EmptyRuntimeSet         // semantic abstraction represents no values
```

They can coincide mathematically in a specific domain but often have different diagnostic and incremental behavior. Current `ValueShape::bounded_union([]) -> Unknown` demonstrates that current shape inference does not expose an explicit empty/bottom shape; future CFG reachability should therefore carry reachability separately rather than treating `Unknown` as unreachable.

## Worked example: branch and loop

Consider an advisory local shape analysis:

```phalcom
let x = 0
while cond {
    if choose {
        x = "s"
    }
}
use(x)
```

Let:

```text
D = {Int, String, Unknown, ... bounded unions ...}
State = BindingId -> D
```

At loop header `H`:

```text
IN[H] = OUT[entry] ⊔ OUT[backedge]
```

Iteration:

```text
round 0: IN[H] = Int
body may assign String
round 1: IN[H] = Int | String
body preserves Int | String
round 2: stable
```

The final fact after the loop is `Int | String` if `cond` may run zero or more times. A one-pass visitor that analyzes the body once and uses its final state would incorrectly report only `String` or depend on traversal choices.

## Product-state map semantics

For maps `M : BindingId -> D`, define missing keys explicitly. Two common choices differ:

```text
missing = ⊥  // no flow/evidence for this binding
missing = ⊤  // binding value is completely unknown
```

The choice changes pointwise join. In definite-assignment, an absent binding may mean “not declared in this scope,” which is neither unassigned nor unknown. Do not reuse one map representation across analyses without a domain-specific interpretation.

## Widening and narrowing

For a domain with infinite ascending chains, e.g. intervals:

```text
[0,0] ⊑ [0,1] ⊑ [0,2] ⊑ ...
```

naive iteration may not terminate. A widening `▽` can force:

```text
[0,0] ▽ [0,1] = [0,+∞]
```

After reaching a post-fixed point, an optional narrowing phase may recover precision while preserving safety.

For current bounded `ValueShape` unions, the cap-to-`Unknown` operation functions as a finite widening/resource policy. Document the distinction: it is not a normative union-type simplification and not evidence that the program truly has arbitrary runtime values.

## Worklist ordering and determinism

Correct monotone solvers should converge to the same semantic fixed point regardless of fair worklist order, but resource caps and bounded provenance can expose order effects. Therefore:

- canonicalize set-like semantic alternatives;
- avoid provenance in semantic equality when possible;
- use deterministic queues/IDs for reproducible diagnostics;
- test different source insertion/update orders when caps exist;
- record when widening occurs if diagnostics/debugging need to explain lost precision.

## Path sensitivity and partitioning

A path-insensitive join at every merge can lose useful correlations:

```phalcom
if flag {
    x = 1
    y = "a"
} else {
    x = "b"
    y = 2
}
```

Independent unions yield:

```text
x : Int | String
y : String | Int
```

and forget that `(x=Int)` correlates with `(y=String)`. Full path sensitivity can explode exponentially. Use bounded partitions only for semantic questions that need the correlation, e.g. discriminated variants or condition refinements.

A partitioned state can be modeled as:

```text
PartitionKey -> FlowState
```

with a cap that merges partitions conservatively. State the key and merge policy; never add unbounded path predicates to editor analysis.

## Reduced products

Combining two domains independently can lose cross-domain information. A reduced product adds a reduction operator `ρ`:

```text
ρ : A × B -> A × B
```

Example:

```text
Shape = Bool
KnownBoolean = true
```

is consistent, while `Shape = String, KnownBoolean = true` should be reduced rather than propagated as contradictory metadata. Future type/nullability + constant domains may similarly refine one another.

## Soundness of widening

If `▽` replaces an exact join for termination, require:

```text
a ⊔ b ⊑ a ▽ b
```

That is the crucial guarantee: widening may lose precision but cannot exclude represented behaviors in a may-analysis.

For a must-analysis, the polarity is reversed in implementation terms; reason from concretization/guarantees rather than copying a may-analysis operator.

## Dataflow implementation skeleton with reachability

Conceptually:

```rust
struct BlockState<D> {
    reachable: bool,
    facts: D,
}

while let Some(block) = work.pop_front() {
    let input = join_reachable_predecessors(block, &out_state);
    if !input.reachable {
        continue;
    }
    let next = transfer(block, input);
    if semantic_state_changed(&out_state[block], &next) {
        out_state[block] = next;
        for succ in cfg.successors(block) {
            work.push_back(succ);
        }
    }
}
```

Do not make provenance sampling or source-range movement trigger semantic re-enqueue unless a downstream semantic computation genuinely depends on it.

## Review questions

- What concrete set does each abstract value represent?
- Is `Unknown` top, “not analyzed,” or both? If both, should it be split?
- Does join satisfy idempotence/commutativity/associativity before widening?
- What is bottom, and is reachability separate?
- Why are transfers monotone?
- What bounds ascending chains?
- Is widening conservative in the correct direction?
- Does map absence have defined semantics?
- Are path partitions bounded?
- Can worklist order change semantic results under current caps?
- Which current Phalcom facts are advisory and which future consumers require soundness?

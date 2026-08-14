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

# Path Sensitivity and Refinement

A flow-sensitive analysis can know different facts before and after a statement while still being path-insensitive at merges. Path sensitivity goes further: it preserves selected distinctions between alternative control-flow histories. This is powerful for option/tag tests, class tests, contracts, numeric guards, and correlated values, but unrestricted path splitting is exponential. The engineering problem is therefore not “be path sensitive” but “which predicates deserve partitions, how are they represented, when are they merged, and what soundness guarantee survives budget exhaustion?”

## 1. Path-insensitive state

Suppose the value domain is independent per binding:

```text
State# = BindingId -> Value#
```

For:

```text
if cond {
    x = 1
    y = 2
} else {
    x = "a"
    y = "b"
}
```

a path-insensitive join yields:

```text
x# = Int ⊔ String
y# = Int ⊔ String
```

It loses the correlation:

```text
x is Int  <=>  y is Int
```

This is not unsound; it is imprecise.

## 2. Edge refinement

Most useful “flow typing” does not require preserving every full path. A conditional can refine the outgoing edges:

```text
              pre: x# = A ⊔ B
                  |
              test isA(x)
              /          \
      true edge            false edge
      x# = A                x# = B
```

Then each branch is analyzed independently and merged at its join point.

Formally, for predicate `p`:

```text
RefineTrue#(p, a#)  ⊑ a#
RefineFalse#(p, a#) ⊑ a#
```

and soundness requires every concrete state satisfying the corresponding branch condition to remain represented.

```text
{ σ ∈ γ(a#) | p(σ) = true }  ⊆  γ(RefineTrue#(p, a#))
```

Similarly for false.

## 3. Refinement is not arbitrary filtering

A refinement operation is safe only when the analyzer trusts the semantics of the predicate. Good sources include:

```text
built-in tag/presence tests with normative semantics
exact runtime class tests
pattern-match discriminants
compiler-known boolean operators
proven or trusted user predicate contracts
explicit type-guard declarations, if Phalcom adopts them
```

Bad source:

```text
method name looks like `isString`
```

Names are not semantic contracts.

The current Phalcom LSP structured flow follows this discipline for trusted nominal tests such as the recognized `is` / `isExactly` forms rather than inferring refinements from arbitrary method names.

## 4. Reachability is part of refinement

If refinement proves a branch impossible, represent the edge as unreachable/bottom, not as an “unknown” state.

```text
x# = Instance(String)
if x isExactly String {
    ...              # reachable
} else {
    ...              # bottom if class fact is exact/trusted
}
```

For a may-analysis:

```text
join(⊥, a) = a
```

This is why a terminating branch can make a refinement survive after an `if`:

```text
if x is None {
    return
}
# only the non-None edge reaches here
```

A CFG or equivalent structured abrupt-flow model naturally computes this. Do not manually “restore the pre-if type” after every branch.

## 5. Option/presence refinement

For a future Phalcom language type:

```text
x : Option<String>
```

a presence test could produce:

```text
true edge:  x : Some<String>
false edge: x : None
```

This must stay distinct from the current advisory runtime `ValueShape` domain. A checker refinement works over language types/proof facts; LSP shape refinement may separately exploit runtime-shape evidence. An explicit bridge can allow trusted type facts to constrain shape facts and vice versa where sound.

## 6. Union elimination

Given an abstract disjunction:

```text
x# = A ⊔ B ⊔ C
```

and a trusted predicate that selects `B`, refinement can be modeled as meet/filtering:

```text
RefineTrue(x#, IsB)  = B
RefineFalse(x#, IsB) = A ⊔ C
```

For nominal subtyping, exact-class and subtype-class tests differ:

```text
isExactly C
    select exactly runtime class C

is C
    select C and allowed subclasses according to language semantics
```

Do not substitute one for the other to gain precision.

## 7. Relational predicates

Predicates such as:

```text
x < 10
x == y
x + y <= 12
```

create relations between values. A non-relational shape domain cannot retain these precisely.

Possible domains:

```text
Intervals        x ∈ [lo, hi]
Difference bounds x - y <= c
Octagons         ±x ± y <= c
Polyhedra        arbitrary linear inequalities
SMT path formula richer logical constraints
```

Each precision increase has cost. The ordinary LSP should not acquire a polyhedral domain merely to improve one hint. The static prover may use richer path conditions separately while sharing identities and semantic facts.

## 8. Path partitions

A path-sensitive state can be represented as a finite map:

```text
PartitionedState# = PartitionKey -> State#
```

where `PartitionKey` records selected predicates:

```text
P1: x is String
P2: option is Some
P3: discriminant == VariantA
```

Instead of eagerly forming the Cartesian product of every branch, choose a bounded policy:

```text
max partitions per program point
predicate priority classes
demand-driven split only for queried value
merge oldest/least useful partitions
merge on loop back-edge unless predicate is loop-stable
```

When partitions merge, join their states. The fallback must remain conservative.

## 9. Trace partitioning as abstract interpretation

Path sensitivity can be understood by splitting collecting semantics into partitions:

```text
CollectingStates(p) = C1 ∪ C2 ∪ ... ∪ Cn
```

and abstracting each subset separately:

```text
α(C1), α(C2), ..., α(Cn)
```

This is more precise than:

```text
α(C1 ∪ ... ∪ Cn)
```

when the abstract domain loses correlations at join.

The analysis chooses the partitioning criteria; it never literally enumerates all execution traces.

## 10. Correlation example

Consider:

```text
if cond {
    x = 1
    y = "yes"
} else {
    x = 2
    y = "no"
}

if x == 1 {
    useString(y)
}
```

A non-relational join gives:

```text
x ∈ {1,2}
y ∈ {"yes","no"}
```

and cannot prove `y == "yes"` under `x == 1`.

A partition keyed by `cond` or a relational domain can retain it:

```text
Partition cond=true:  x=1, y="yes"
Partition cond=false: x=2, y="no"
```

When the second condition establishes `x=1`, only the first partition survives.

The important design question is whether any Phalcom consumer benefits enough from this precision to justify retaining such a relation.

## 11. Boolean short-circuit semantics

`and` / `or` are control flow, not ordinary eager binary operators when language semantics short-circuit. The right operand/block executes only on the appropriate left outcome.

For `A and B`:

```text
true output  = refine(A=true) then evaluate B=true
false output = refine(A=false)
             ⊔ refine(A=true) then evaluate B=false
```

For `A or B`, symmetric rules apply.

This affects both value facts and effects: latent effects of the right side are conditional.

## 12. Pattern matching

A future pattern matcher can generate multiple edges with pattern-specific refinements:

```text
match x {
    Some(v) => ...
    None    => ...
}
```

Each edge can bind new identities and refine the scrutinee. Exhaustiveness belongs to the typed-language/checker design, but the dataflow substrate should support:

```text
PatternRefine(scrutinee_fact, pattern)
    -> reachable? + refined scrutinee + new bindings + path propositions
```

Malformed or incomplete patterns in the editor must not create fake proof facts.

## 13. Negative information

A branch may know what a value is **not**:

```text
x# = A ⊔ B ⊔ C
if x is A { return }
# x# = B ⊔ C
```

Do not store an unbounded list of arbitrary negated classes/predicates in every value. For finite unions, eliminate alternatives directly. Rich logical negation belongs in a path/proof domain.

## 14. Refinement invalidation by mutation

A fact about a mutable location can become stale:

```text
if self._state is Ready {
    callUserCode()
    use self._state as Ready
}
```

If `callUserCode()` may mutate `_state`, the refinement cannot survive. Therefore refinement validity depends on effects and alias analysis.

For lexical immutable values, refinement can often survive calls. For mutable fields/globals/captured cells:

```text
RefinementFact = proposition + dependencies on locations/world state
```

A write/havoc to a dependency kills or weakens the fact.

## 15. Refinement across fiber yields

A yield can invalidate facts about shared mutable state even without an explicit local write:

```text
if shared.state is Ready {
    maybeYield()
    # another fiber may have changed shared.state
}
```

Preserve only facts protected by ownership/isolation/immutability or scheduler guarantees. This interaction is a major reason path facts cannot be divorced from effect/concurrency analysis.

## 16. Function/callable summaries and refinements

A callee contract can refine caller state. For example, a proven postcondition:

```text
ensures: result implies x is String
```

or a type-guard contract could emit an edge proposition at the call site. Keep trust explicit:

```text
BuiltinSemanticRule
DeclaredContractUnchecked
DeclaredContractRuntimeEnforced
StaticallyProvenContract
HeuristicPredicate
```

Only accepted trust levels may drive checker/prover correctness.

## 17. Widening path facts

Loops can accumulate ever more predicates or partitions. A termination policy can:

```text
drop loop-variant predicates
merge partitions by selected key subset
cap relational constraints
widen intervals
replace complex formula with TopPathCondition
```

Record the precision-loss reason. “Unable to prove after loop widening” is more useful than “unknown.”

## 18. Consumer-specific policy over one foundation

The same branch fact can feed multiple consumers at different trust thresholds:

```text
LSP hover
    may show probable narrowed shape

checker
    only uses sound/trusted refinement

prover
    turns trusted predicate into path formula / VC assumption

optimizer
    uses only sound refinement or emits runtime guard
```

Do not duplicate the condition parser in each consumer. Centralize semantic recognition of trusted predicate forms and expose facts with provenance/trust.

## 19. Failure modes

- Treating a method named `isFoo` as a type guard without a semantic contract.
- Joining branches by traversal order.
- Converting impossible branch to `Unknown` instead of bottom.
- Keeping every path indefinitely “for precision.”
- Retaining a field refinement across an unknown mutating call.
- Using a runtime-shape refinement as a formal type proof without a bridge.
- Treating exact-class and subtype tests as equivalent.
- Assuming a refinement survives a fiber yield over shared mutable state.
- Turning analysis budget exhaustion into success.

## 20. Testing obligations

1. simple true/false nominal refinement;
2. exact versus subtype test difference;
3. impossible branch becomes unreachable;
4. terminating branch preserves opposite refinement after merge;
5. branch join restores union when both paths continue;
6. short-circuit RHS effects occur only on reachable path;
7. mutation kills dependent refinement;
8. immutable local refinement survives unrelated call;
9. shared mutable refinement is invalidated across yield as required;
10. bounded path partitions merge conservatively;
11. loop path widening terminates;
12. incremental and clean analysis produce identical refinements;
13. malformed condition produces recovery uncertainty, not false proof.

Property tests should check:

```text
RefineTrue(p, a) ⊑ a
RefineFalse(p, a) ⊑ a
join(reachable refined branches) soundly covers pre-state behavior
```

## 21. Review questions

1. What exact predicate semantics justify the refinement?
2. Is the fact about an immutable value or mutable location?
3. Which effects can invalidate it?
4. Is this edge impossible or merely unsupported by the analyzer?
5. What correlation is path sensitivity preserving?
6. Why is that correlation worth its cost?
7. What bounds partition growth?
8. What conservative fallback occurs at the bound?
9. Which trust level may consume the fact?
10. How does the fact map to Phalcom type/proof domains without conflating them?

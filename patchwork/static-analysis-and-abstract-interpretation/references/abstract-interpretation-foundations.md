# Abstract Interpretation Foundations

Abstract interpretation gives a disciplined answer to a practical compiler question: how can an analyzer compute with a finite or tractable representation while conservatively describing many concrete program executions? This reference connects the formal framework to implementable Phalcom analyses. It does not require every implementation to encode Galois connections directly, but every sound analysis should be understandable in these terms.

## 1. Concrete semantics and collecting semantics

Let `C` be the concrete state space of the dynamic language. A concrete state might include more information than any particular static analysis wants to model:

```text
σ = (stack frames, lexical cells, heap objects, globals, module state,
     dispatch tables, fiber scheduler state, IO/native environment, ...)
```

A concrete small-step semantics can be written:

```text
⟨pc, σ⟩ -> ⟨pc', σ'⟩
```

or expression evaluation as:

```text
⟨e, ρ, σ⟩ ⇓ ⟨v, σ'⟩
```

where `ρ` is an environment mapping lexical names/IDs to locations or values and `σ` is mutable store/runtime state.

Static analysis normally does not approximate one concrete execution. It approximates the *collecting semantics*: the set of all concrete states that can reach a program point.

For program point `p`:

```text
Reach(p) ⊆ C
```

A branch creates multiple concrete successors; a merge point's collecting semantics contains states from every reachable predecessor; a loop includes states from zero or more iterations.

This is the most useful intuition for may-analysis. If the analyzer says a variable may be `Int | String`, it is summarizing the runtime values appearing in a set of executions.

## 2. Abstract domains

Static analysis replaces an intractable concrete domain—often `P(C)`, the powerset of concrete states—with an abstract domain `A`.

Conceptually:

```text
          α
P(C)  ---------->  A
      <----------
          γ
```

`α` is abstraction: it maps concrete-state sets to abstract facts.

`γ` is concretization: it maps an abstract fact to the concrete states it represents.

Example: a shape abstraction can forget exact object identity and retain only runtime class possibilities.

```text
concrete values:
  String object #104
  String object #880
  Int object #12

abstract shape:
  {String, Int}
```

The analyzer typically does not compute `α` or enumerate `γ` explicitly. They are specification devices: they tell us what an abstract value *means*.

## 3. Galois connections

A common formal relationship is a Galois connection between concrete and abstract orders:

```text
α(c) ⊑ a    iff    c ⊆ γ(a)
```

Here `c` is a set of concrete states, `a` is an abstract value, and `⊑` is the abstract precision/order relation.

This expresses a best-fit relationship: `α(c)` is the least abstract value sufficient to cover `c`, and `γ(a)` contains exactly the concrete states represented by `a` under the abstraction.

You do not need to force every engineering domain into a textbook-perfect Galois insertion. But if you claim soundness, you must be able to state a concretization meaning and show that abstract operations never exclude relevant concrete behavior.

## 4. Soundness of an abstract transformer

Suppose concrete execution of one statement is represented by:

```text
F : P(C) -> P(C)
```

and the analyzer implements:

```text
F# : A -> A
```

For a forward may-analysis, soundness requires:

```text
F(γ(a)) ⊆ γ(F#(a))
```

A commuting-diagram intuition:

```text
       concrete F
P(C) -------------> P(C)
 |                    |
 | α                  | α
 v                    v
 A   ------------->   A
       abstract F#
```

The abstract path may be less precise, but it must cover what the concrete path can do.

### Local versus whole-analysis soundness

Sound transfer functions are necessary but not sufficient. Whole-analysis soundness can also fail if:

- CFG/control edges are omitted;
- dynamic dispatch targets are missing;
- exceptions/non-local returns are ignored;
- aliasing makes a “strong update” invalid;
- reflection/native code mutates state the analysis assumes stable;
- the solver stops before reaching a conservative post-fixpoint;
- an incremental cache is stale;
- a consumer interprets an advisory result as guaranteed.

Soundness is a property of the entire modeling pipeline.

## 5. Best correct approximation

The ideal abstract transformer is often described as the best correct approximation:

```text
F#_best = α ∘ F ∘ γ
```

In practice, `F` and `γ` are not computable directly. Implementations choose a computable transformer that is at least as abstract as the ideal:

```text
F#_best(a) ⊑ F#_impl(a)
```

under the “more precise is lower” convention.

This frames performance engineering correctly. A coarser transformer is acceptable if it still over-approximates the best correct result. Precision is optional; soundness direction is not.

## 6. Worked example: class-shape analysis

Consider a simplified Phalcom-like fragment:

```text
let x = 1
if condition {
  x = "text"
}
x.foo()
```

Suppose the abstract domain tracks possible runtime classes:

```text
Shape = ⊥ | Classes(S) | ⊤
```

and:

```text
γ(Classes({Int}))    = all states where tracked value is an Int instance
gamma(Classes({String})) = all states where tracked value is a String instance
γ(⊤)                 = all runtime values
```

Initial transfer:

```text
x = 1
=> x ↦ {Int}
```

True branch:

```text
x = "text"
=> x ↦ {String}
```

False branch:

```text
x unchanged
=> x ↦ {Int}
```

Merge:

```text
{String} ⊔ {Int} = {Int, String}
```

To analyze `x.foo()`, resolve `foo()` against every possible receiver class. If both classes have a target, join return/effect summaries. If only one has a target, then a correctness analysis must represent the possibility of a missing-message/error path (subject to Phalcom's actual missing-message semantics). An advisory completion analysis might merely show members common to or likely across candidates.

The same receiver evidence can therefore feed multiple consumers, but the *consumer query* and trust policy differ.

## 7. Worked example: zero-iteration loops

Consider:

```text
let x = 0
|| { condition }.whileTrue || {
  x = "text"
}
use(x)
```

A one-pass analysis that uses only the body output after the loop reports `String`. The collecting semantics includes executions where the loop runs zero times, so the exit abstraction must include the entry value:

```text
Exit = Entry ⊔ ConditionFalseExit ⊔ BreakExits ⊔ ...
```

At minimum:

```text
x : {Int, String}
```

If later iterations can produce additional states, the loop header itself requires fixed-point iteration.

This example illustrates why abstract interpretation begins from concrete control semantics. The error is not “bad type inference”; it is an incorrect approximation of reachable executions.

## 8. Forward and backward abstraction

### Forward analysis

Forward analysis asks what states can occur after executing code:

```text
A_before --F#--> A_after
```

Examples:

- runtime shape/value inference;
- reaching definitions;
- constant propagation;
- may-effects;
- points-to propagation;
- taint propagation.

### Backward analysis

Backward analysis asks what states before an operation guarantee or may lead to a desired property after it.

A classic proof-oriented form is weakest precondition:

```text
WP(C, Q)
```

meaning the weakest precondition under which command `C` establishes postcondition `Q`.

Liveness is a conventional backward dataflow analysis: a variable is live before a statement if its value may be used later before being overwritten.

Do not assume abstract interpretation means forward analysis only.

## 9. Abstraction design: choose the question, then forget deliberately

An abstract domain is useful because it forgets concrete distinctions that do not matter to a query.

For completion:

```text
exact object identity -> unnecessary
runtime receiver classes -> useful
```

For escape analysis:

```text
exact numeric value -> often unnecessary
allocation site / alias set -> useful
```

For proving a range check:

```text
runtime class Int -> insufficient
numeric interval / relation -> useful
```

The domain is not “the truth about a value.” It is a projection of concrete behavior chosen for a property.

### Avoid the universal mega-domain

A tempting design is one `ValueFact` that carries every possible property forever:

```text
class + type + constant + range + taint + effects + aliases + proof propositions + ...
```

This creates expensive joins, difficult invalidation, mixed trust levels, and conceptual coupling. Prefer composable domains with explicit bridges and reduced products only when precision needs them.

## 10. Relational and non-relational domains

A non-relational domain tracks each variable independently:

```text
x ∈ [0, 10]
y ∈ [0, 10]
```

It cannot express `x < y`.

A relational domain can retain cross-variable relationships:

```text
x - y <= -1
```

Examples include difference-bound matrices, octagons, polyhedra, and symbolic path constraints.

Relational precision is expensive because joins and transfer affect combinations of variables. Use it only when the consumer needs those relationships. A future static prover may maintain symbolic relations while the LSP keeps a cheap non-relational shape domain.

## 11. Flow sensitivity, path sensitivity, and trace partitioning

### Flow sensitivity

A flow-sensitive analysis assigns facts to program points:

```text
before x = 1:  x unknown
 after x = 1:  x = Int
```

Phalcom's current LSP local-flow facts are flow-sensitive.

### Path sensitivity

Path-sensitive analysis keeps different states for selected path conditions:

```text
path p:     x = Int, y = String
path not p: x = String, y = Int
```

A path-insensitive join loses the correlation.

### Trace partitioning

Abstract interpretation describes bounded path sensitivity as partitioning the set of execution traces by selected predicates. Instead of globally “turning on path sensitivity,” choose partitions such as:

```text
Option tag of x
exact runtime-class test of receiver
small enum/pattern discriminant
selected null/presence predicates
```

Then merge partitions at defined boundaries or when a budget is exceeded. This makes precision policy explicit and predictable.

## 12. Disjunctive completion and bounded unions

A powerset-like disjunctive domain retains alternatives precisely:

```text
A = {Int-state} ∨ {String-state}
```

Full disjunctive completion can explode exponentially. Practical analyzers bound alternatives and widen.

Phalcom's current `ValueShape::Union` keeps at most `MAX_SHAPE_UNION` alternatives and widens to `Unknown` beyond that. This is an explicit precision/cost trade-off for advisory semantic intelligence. If a future sound checker reuses any bounded representation, its overflow must remain conservative *with respect to checker semantics*. “Unknown then allow everything” would not preserve correctness.

## 13. Abstract garbage collection

Heap analyses allocate abstract addresses. As analysis proceeds, abstract stores can retain addresses no longer reachable from abstract roots, increasing both memory use and imprecision. Abstract garbage collection computes reachability in the *abstract heap* and removes unreachable abstract addresses.

Conceptually:

```text
roots = abstract stack/env/global roots
reachable = transitive closure through abstract pointer edges
store = store restricted to reachable addresses
```

This does not mean runtime GC and static analysis are the same. Runtime GC discovers live concrete heap objects during execution; abstract GC prunes unreachable *abstract locations* in the analysis state.

Phalcom only needs this if a future points-to/abstract-store analysis becomes detailed enough to retain per-allocation abstract addresses.

## 14. Widening as an abstract-interpretation operator

When ascending chains are infinite, a widening `▽` forces convergence:

```text
x0 ⊑ x1 ⊑ x2 ⊑ ...
```

becomes:

```text
y0 = x0
y(n+1) = yn ▽ x(n+1)
```

with jumps to coarser states that stabilize. The widening must preserve an upper-bound relationship suitable for the soundness contract.

Widening is not simply “replace with Unknown after N iterations.” That can be a valid widening for a simple domain, but only if `Unknown` genuinely covers every relevant value/effect and the consumer can handle the loss safely.

See [widening-narrowing-and-termination.md](widening-narrowing-and-termination.md).

## 15. Abstract interpretation and language types are not identical

A type system can itself be viewed abstractly, but Phalcom should not identify all semantic approximations with formal language types.

Example bridges:

```text
exact runtime shape Instance(String)
    -> evidence supporting nominal type String

declared type String
    -> constraint on acceptable runtime values in typed mode

path fact x is Some
    -> refinement of Option<String> to payload String on that edge
```

But the following equivalence is invalid without a language decision:

```text
ValueShape::Unknown == Dynamic type
```

`Unknown` can mean the analysis lost information. `Dynamic`—if Phalcom adopts it—is a language-level typing construct with user-visible checking behavior. A union cap, missing dependency, or analysis budget must not silently grant dynamic permission.

## 16. Dynamic language soundness assumptions

Abstract interpretation is only as sound as the concrete semantics it models. For Phalcom, ask whether an analysis accounts for:

- message dispatch across possible receiver classes/metaclasses;
- selector identity and dynamic selector/packs;
- `super` lookup semantics;
- missing-message fallback if present;
- reflective method lookup/invocation;
- reflective method-table mutation if allowed;
- class-side/global/module mutation;
- native primitives and FFI;
- block invocation and non-local return;
- fibers/yields and shared mutable state;
- module initialization and dynamically available modules.

If an analysis profile explicitly forbids some capabilities, state the closed-world assumption. Do not quietly assume them absent.

## 17. Sound, conditionally sound, and advisory abstractions

Not every useful analysis needs a whole-language soundness guarantee. Classify it.

### Sound under modeled semantics

All concrete behaviors permitted by the model are represented. This is appropriate for optimizer guards, checker rejection reasoning, security analyses, and prover-support facts.

### Conditionally sound

Sound if explicit assumptions hold, for example:

```text
no reflective method mutation
native methods obey declared summaries
module graph is closed
no unmodeled shared-state mutation across yield
```

The assumptions must be machine-checkable or part of the profile/trusted base where possible.

### Advisory

Designed for utility rather than semantic guarantee. May use heuristics, caps, and guesses. LSP completion can be advisory, but diagnostic wording and downstream consumers must not imply proof.

## 18. Implementation representation in Rust

A clean separation is:

```rust
trait AbstractDomain: Clone + Eq {
    fn bottom() -> Self;
    fn leq(&self, other: &Self) -> bool;
    fn join(&self, other: &Self) -> Self;
}

struct Fact<T> {
    value: T,
    trust: Trust,
    provenance: ProvenanceId,
    precision_loss: SmallVec<[PrecisionLoss; 2]>,
}
```

This sketch is conceptual. Do not introduce a generic trait merely for abstraction purity if concrete enums are clearer or faster. The important separation is between:

- lattice semantics;
- provenance/trust metadata;
- solver storage;
- query/rendering policy.

Use typed semantic IDs (`BindingId`, `CallableId`, `FieldId`, `ClassId`) as keys. Avoid using source text/names when identity matters.

## 19. Failure modes and unsound shortcuts

### Guessing instead of abstracting

“Most objects here are String” is not a sound abstraction. It can be a heuristic with a separate confidence class.

### Ignoring unknown effects

If a call can mutate a field, preserving an exact field fact across the call excludes concrete executions.

### Treating finite testing as concretization proof

Generated/runtime tests can discover violations of the intended abstraction relation. They cannot prove `F(γ(a)) ⊆ γ(F#(a))` universally.

### Unmodeled abrupt control

Ignoring throw/non-local return can make an impossible fallthrough appear reachable or a must-property appear established.

### Unsound strong update

Replacing an abstract heap cell is valid only if it denotes exactly one concrete location. If it may represent several aliases/objects, use a weak update.

### Hidden closed-world assumption

Resolving only currently indexed method targets without modeling future/reflective mutation can be unsound in an open world.

## 20. Phalcom application: current `ValueShape`

CURRENT at the inspected 2026-08-14 baseline:

- `phalcom-lsp/src/semantic/facts.rs` explicitly calls `ValueShape` an “advisory runtime value shape” and says it is deliberately not a language type.
- Shapes include instances, class objects, modules, tuples, records, collections, callable/family shapes, bounded unions, and `Unknown`.
- `InferredValue` carries confidence and compact provenance.
- joins flatten bounded unions and widen over the cap.
- structured flow associates facts with lexical `BindingId`s and performs branch/loop reasoning.

This makes `ValueShape` a concrete example of an engineering abstract domain. It does **not** establish that the future checker, type system, proof engine, or optimizer should consume the same domain unchanged.

The right evolution is explicit bridges:

```text
ValueShape evidence -> formal type evidence
formal type constraint -> permitted shape set
path predicate -> type/shape refinement
sound effect fact -> optimizer/prover permission
```

Each arrow needs a trust rule and provenance.

## 21. Testing and validation

For an abstraction, test both algebra and concrete correspondence.

### Algebraic tests

- join laws;
- normalization;
- order relation;
- widening upper bounds;
- product/reduction invariants.

### Concrete differential tests

Generate small programs, execute them under the VM, and check that observed runtime states are contained in the analyzer's abstract result:

```text
for observed value v at point p:
    assert v ∈ γ(analysis[p])
```

This cannot prove soundness but is excellent at finding missing transfer/call/control cases.

### Metamorphic tests

- alpha-renaming should preserve abstract facts modulo IDs;
- formatting/irrelevant whitespace should not change semantic facts;
- adding an unreachable branch should not weaken reachable facts if reachability is modeled;
- replacing sugar with equivalent canonical syntax should preserve results;
- incremental analysis should equal clean analysis.

## 22. Competency questions

1. What is the concrete state space relevant to the proposed analysis?
2. What does `γ(a)` mean for each important abstract value?
3. Can you state the soundness condition for one representative transfer?
4. Which concrete distinctions does the abstraction deliberately forget?
5. Where could aliasing, reflection, FFI, or fibers break the abstraction relation?
6. Is the analysis sound, conditionally sound, or advisory?
7. What causes an abstract join to lose precision?
8. Which paths/trace partitions are retained, and what bounds them?
9. Why is `ValueShape` not automatically the Phalcom type system?
10. If the analysis widens to `Unknown`, what does `Unknown` concretize to, and can each consumer safely use it?

A designer who can answer these can reason from dynamic semantics to a tractable analyzer instead of treating “inference” as a collection of ad hoc guesses.

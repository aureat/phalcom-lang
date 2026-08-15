# Comparative Static Analysis and Reading

Use other systems as case studies in solved constraints, not as architecture templates. A precedent is useful only after stating the problem it solves, the assumptions under which it works, and which of those assumptions match Phalcom. Dynamic dispatch, first-class reflection, open modules, cooperative fibers, future optional correctness typing, and live-editor latency mean Phalcom often occupies a different design point from ahead-of-time statically dispatched compilers.

This reference is intentionally a reading map rather than another textbook. Load the theory references in this skill for derivations.

## 1. How to read a precedent

For every paper/compiler ask:

```text
What concrete property is analyzed?
Is the analysis may or must?
What is the abstract domain/order?
Where is the fixed point?
What guarantees termination?
What is the world assumption?
How are calls resolved?
How are heap aliases modeled?
How are exceptions/callbacks/concurrency modeled?
What is sound, heuristic, or trusted?
What representation makes the analysis tractable?
How is incrementality/caching validated?
What latency/memory budget shaped the design?
```

Then ask:

```text
Which Phalcom assumptions match?
Which do not?
What semantic feature would be lost by copying this design?
```

## 2. Abstract interpretation foundations

### Cousot & Cousot

Study for:

- collecting semantics;
- abstraction/concretization;
- Galois connections;
- sound abstract transformers;
- widening/narrowing;
- fixpoint approximation.

Phalcom lesson: make approximation direction explicit. The current advisory `ValueShape` can be understood as an abstract domain over runtime-shape possibilities, but that interpretation does not transform it into the language type system.

### Patrick Cousot's later surveys/lectures

Useful for reduced products, trace partitioning, widening design, and viewing many classic analyses as instances of one framework.

## 3. Monotone dataflow frameworks

### Kildall

Study for finite-height lattice worklist solving and unified dataflow equations.

Phalcom lesson: branch and loop results must be order-independent fixed points/joins, not artifacts of AST visitation.

### Kam & Ullman / monotone framework literature

Study for monotonicity, distributivity, MOP versus MFP, and convergence conditions.

Phalcom lesson: when a transfer is not monotone or equality is unstable, a worklist's “convergence” becomes implementation luck.

## 4. Interprocedural analysis

### Sharir & Pnueli

Study for functional/interprocedural dataflow frameworks and context-sensitive reasoning over call graphs.

Phalcom lesson: recursive calls need equations/summaries, not recursive AST expansion.

### Reps, Horwitz & Sagiv — IFDS

Study for distributive subset problems reducible to graph reachability with strong polynomial guarantees.

Good fits include finite fact-set problems such as certain taint/reaching-definition analyses. Poor fit for arbitrary rich Phalcom `ValueShape` joins or relational numeric domains unless transformed into a suitable finite distributive fact problem.

Do not label every interprocedural analysis “IFDS.”

## 5. SSA and compiler analysis

### Cytron et al.

Study for dominance-frontier SSA construction.

Phalcom lesson: SSA is useful when def-use precision, sparse propagation, or optimization justifies a semantic CFG/IR. It does not solve heap aliases or captured mutable cells automatically.

### Cooper & Torczon; Muchnick

Study for practical compiler-analysis engineering: dominators, liveness, reaching definitions, dataflow implementation, loop analysis, optimization interactions.

Use as engineering background, not as Phalcom semantics.

## 6. rustc

Study:

- HIR/MIR separation;
- MIR-based control/dataflow;
- move/initialization analyses;
- borrow checking;
- query/incremental infrastructure;
- diagnostics/provenance patterns.

Why it differs:

- Rust has a static type/ownership discipline and far more statically resolved behavior;
- reflection/open method mutation assumptions differ dramatically;
- borrow checking relies on Rust-specific language guarantees.

Phalcom lesson: lower into a representation that makes repeated semantic questions explicit, but do not import Rust's closed/static assumptions into dynamic dispatch.

## 7. rust-analyzer

Study:

- demand-driven semantic queries;
- stable identities/interning;
- immutable-ish snapshots;
- cancellation;
- incremental dependency tracking;
- latency-oriented semantic APIs.

Phalcom lesson: editor queries should consume coherent semantic facts rather than run independent inference. Current Phalcom LSP already moves in this direction with immutable published snapshots and dependency-directed rebuilds.

Difference: rust-analyzer serves Rust's statically typed semantics; Phalcom must retain advisory dynamic-shape knowledge and future correctness typing as separate abstractions.

## 8. Roslyn

Study:

- immutable syntax trees;
- semantic models;
- operation/control-flow APIs;
- incremental compilation concepts;
- analyzer contracts and diagnostics.

Phalcom lesson: a durable semantic-query surface can support many tooling consumers. Avoid making source ranges or syntax nodes the sole durable identity.

Difference: C# overload resolution and static type semantics should not be copied into selector-based Phalcom dispatch.

## 9. TypeScript

Study:

- flow-sensitive narrowing;
- union/intersection practicalities;
- control-flow graph driven type facts;
- handling dynamic JavaScript ecosystems;
- usefulness/soundness tradeoffs.

Phalcom lesson: sophisticated narrowing can coexist with dynamic runtime semantics, but Phalcom's future typing explicitly participates in correctness and may choose different soundness boundaries. TypeScript's particular `any`/structural rules are precedent, not defaults.

## 10. mypy and Pyright

Study:

- Python typing over a dynamic runtime;
- gradual/dynamic boundaries;
- flow narrowing/type guards;
- module/import graphs;
- incremental daemon/editor architecture;
- diagnostics under incomplete information.

Phalcom lesson: optional source annotations and dynamic execution can share a checker infrastructure, while editor inference remains useful. Do not assume Python's annotation erasure/runtime policies or nominal/structural compromises are Phalcom's choices.

## 11. Julia inference

Especially relevant because Julia combines dynamic execution, multiple dispatch, specialization, and abstract interpretation/type inference for optimization.

Study:

- lattice-based inference;
- method-instance specialization;
- abstract interpretation over dynamic calls;
- widening/union splitting;
- world-age and invalidation problems;
- optimizer use of inferred facts.

Phalcom lesson: dynamic-language analysis can be highly sophisticated and optimization-relevant. Critical difference: Phalcom's current dispatch identity is selector-based rather than Julia-style type tuple multiple dispatch. Do not import type-directed method selection accidentally.

Julia's world-age/invalidation work is a valuable case study if reflective method mutation becomes optimizer-relevant in Phalcom.

## 12. Smalltalk and Self VMs

Study:

- message-send semantics;
- inline caches and polymorphic inline caches;
- class/shape identity;
- reflective object model;
- speculative optimization and deoptimization.

Phalcom lesson: runtime profiles and class guards can optimize dynamic sends without changing source semantics. Static target evidence and runtime inline-cache evidence are related but not the same fact.

Self's maps/hidden-class-like techniques are runtime representation precedents, not formal type systems.

## 13. Python / CPython

Study:

- dynamic object protocol;
- descriptor/attribute lookup complexity;
- module loading;
- C extension/native boundary;
- runtime specialization/invalidation in modern CPython.

Phalcom lesson: native/reflective boundaries require explicit analysis contracts; runtime caches need invalidation tied to mutable object/class state.

Difference: Python's attribute lookup/descriptor semantics are not Phalcom's selector semantics.

## 14. Ruby

Study dynamic method lookup, blocks, open classes/metaprogramming, and JIT invalidation challenges. Useful as a cautionary case for reflection × optimization and blocks × control/effects.

Do not infer Phalcom block/non-local-return semantics from Ruby; inspect normative Phalcom rules.

## 15. Kotlin / Swift / Scala 3

These are useful for targeted type-system interactions:

- Kotlin smart casts: flow refinement invalidated by mutability/aliasing;
- Swift: generics/protocols and diagnostics;
- Scala 3: unions/intersections and advanced subtyping/inference.

The static-analysis skill should borrow flow/constraint ideas only after the concrete Phalcom type system is ratified.

## 16. OCaml / Haskell / GHC

Study for:

- HM-style inference roots;
- polymorphism/generalization;
- typed intermediate representations;
- GHC Core and optimization;
- effect/control abstractions in research/ecosystem.

These are primarily neighbors for `type-theory` and `type-checker-development`. Static analysis should know the concepts but not duplicate those skills.

## 17. LLVM

Study:

- dominator infrastructure;
- MemorySSA;
- alias analysis interfaces;
- abstract interpretation-like value analyses;
- pass invalidation/preservation contracts;
- optimization verification culture.

Phalcom lesson: analysis preservation is an explicit contract. Difference: LLVM IR has already lowered away most source-language dynamic semantics; Phalcom source tooling cannot use LLVM-style IR as the semantic truth for refactoring/diagnostics without retaining source meaning.

## 18. BEAM / Erlang

Study lightweight processes, immutable message passing, scheduling, and fault semantics as a contrasting concurrency model.

Phalcom fibers likely permit different shared-state assumptions. Do not import process isolation unless Phalcom explicitly adopts it. The useful lesson is that concurrency semantics dramatically determine which dataflow facts survive suspension/interleaving.

## 19. Abstract domains worth targeted reading

For future needs:

```text
intervals / widening        Cousot-style numeric AI
congruences                 modular arithmetic analysis
octagons/polyhedra          relational numeric domains
Andersen/Steensgaard        pointer/points-to analysis
escape analysis             compiler optimization literature
taint analysis              IFDS/dataflow/security literature
SCCP                        Wegman & Zadeck
MemorySSA                   LLVM literature/implementation
```

Only deepen the domain when a concrete Phalcom consumer requires it.

## 20. Primary-source discipline

When implementing from a precedent:

1. Read the original paper or current official implementation documentation.
2. Record the exact invariant/algorithm being borrowed.
3. Identify assumptions that do not hold in Phalcom.
4. Adapt the algorithm at the semantic boundary, not by name resemblance.
5. Add a Phalcom-specific counterexample test that would fail under naive cargo-culting.

## 21. Comparative review questions

For any proposed precedent, an agent should answer:

1. What exact problem does it solve?
2. What is its concrete/abstract domain?
3. Which language/runtime assumptions make it correct?
4. Is its world closed or open?
5. How are dynamic calls/reflection/native code handled?
6. What representation enables the algorithm?
7. What termination and complexity guarantees exist?
8. Which Phalcom assumption differs?
9. What must be changed to preserve Phalcom semantics?
10. What future design choice would copying it accidentally lock in?

If those answers are unavailable, the precedent is orientation, not design evidence.

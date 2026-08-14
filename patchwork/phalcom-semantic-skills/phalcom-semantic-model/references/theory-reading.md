# Theory and Primary-Source Reading Map

This reference is not a bibliography for its own sake. It maps the mathematical and
compiler concepts used by Phalcom semantic analysis to the canonical ideas an
implementer should know well enough to recognize, apply, and explain.

Do not copy another language's implementation merely because it uses one of these
techniques. Read for the underlying problem, assumptions, proof obligations, and
failure modes; then reconcile them with Phalcom's dynamic object model, selector
semantics, optional typing, reflective runtime, and editor-first semantic database.

## 1. Type systems and judgments

### Benjamin C. Pierce — *Types and Programming Languages*

Know these concepts:

- typing judgments and contexts: `Γ ⊢ e : T`;
- operational semantics and preservation/progress as proof patterns;
- substitution;
- simply typed lambda calculus as a minimal model;
- products, sums, records and variants;
- subtyping;
- recursive types;
- polymorphism;
- existential and universal types;
- references and state.

Phalcom use:

- defining checker judgments independently of runtime `ValueShape`;
- separating type equality, subtyping, assignability, and runtime membership;
- reasoning about generic substitution and applied member views;
- reasoning about Option/Result, tuples/records, variants, and callable types.

Do not infer from TAPL that Phalcom must become a purely statically typed lambda
calculus. The value is the precision of the vocabulary and proof techniques.

### Luca Cardelli / Peter Wegner — understanding types and polymorphism

Know the taxonomy of:

- universal polymorphism;
- ad-hoc polymorphism;
- inclusion/subtyping polymorphism;
- coercions and overloading.

Phalcom use: keep ordinary selector dispatch, future type-directed helper mechanisms,
generics, and structural protocol conformance conceptually separate.

## 2. Local inference and bidirectional typing

### Pierce & Turner — local type inference

Know:

- why full global inference becomes difficult in rich subtyping systems;
- local reconstruction from arguments and expected types;
- constraint bounds rather than only equality unification;
- explicit annotations at abstraction boundaries.

Phalcom use: future local generic type-argument inference and expected-type flow.

### Dunfield & Krishnaswami — bidirectional typing

Know the distinction:

- **synthesis**: infer a type from an expression;
- **checking**: validate an expression against an expected type.

Phalcom use:

- expected return/assignment/argument annotations can refine checking;
- blocks and empty collection literals often need inward expected-type information;
- the checker need not force every expression to infer a principal type in isolation.

## 3. Dataflow analysis

### Kildall — unified approach to global program optimization

Know:

- abstract states at program points;
- transfer functions;
- joins at control-flow merges;
- monotonicity;
- iterative fixed-point solving.

Phalcom use:

- local value-shape flow;
- definite assignment;
- reachability;
- null/Option refinements;
- effect propagation;
- later constant/range reasoning.

### Nielson, Nielson & Hankin — *Principles of Program Analysis*

Know:

- monotone frameworks;
- reaching definitions;
- available expressions;
- live variables;
- control-flow analyses;
- constraint-based analysis;
- interprocedural analysis.

Phalcom use: selecting the correct may/must lattice and direction for a semantic
analysis instead of inventing an ad-hoc traversal.

## 4. Abstract interpretation

### Cousot & Cousot — abstract interpretation

Know the core model:

- concrete semantic domain;
- abstract semantic domain;
- abstraction/concretization relationship;
- sound over-approximation;
- partial order;
- joins/meets;
- widening and narrowing;
- fixed points.

Phalcom use:

`ValueShape` is best understood as a small abstract domain over possible runtime
values. Future ranges, effects, proof facts, escape facts, and ownership facts may each
have their own abstract domain.

The practical rule is critical:

> A loss of precision must move toward a conservative abstraction, never toward a
> more specific unsupported fact.

## 5. Control-flow graphs and SSA

### Cytron et al. — Efficiently Computing Static Single Assignment Form

Know:

- dominance;
- dominance frontiers;
- phi functions;
- SSA as a representation of value versions.

Phalcom does not need SSA merely because mature compilers use it. SSA becomes useful
when analyses require explicit definition-use chains, optimizer-grade value reasoning,
or efficient sparse dataflow. Current structured flow can remain simpler while it is
sufficient.

### Cooper & Torczon / Muchnick — compiler analysis and optimization texts

Know:

- CFG construction;
- dominators/postdominators;
- loops and natural loops;
- dataflow worklists;
- call graphs;
- local versus global analyses;
- IR design tradeoffs.

Phalcom use: future semantic CFG/IR and optimization, not a requirement to copy a
particular compiler pipeline.

## 6. Interprocedural analysis and call graphs

Know these general techniques:

- call graph construction;
- strongly connected components (Tarjan/Kosaraju);
- summary-based analysis;
- context-insensitive versus context-sensitive analysis;
- call strings and bounded context sensitivity;
- conservative handling of unknown/dynamic calls.

Phalcom use:

- callable return/parameter summaries;
- recursion convergence;
- higher-order block effects;
- invalidation through reverse callable dependencies.

Dynamic dispatch means a call graph may be incomplete. Any analysis depending on a
closed call graph must state the closed-world assumptions explicitly.

## 7. Gradual typing

### Siek & Taha and subsequent gradual-typing work

Know the conceptual distinctions among:

- static type;
- dynamic type boundary;
- consistency relation;
- subtyping relation;
- casts/coercions;
- blame;
- runtime evidence.

Phalcom may not adopt the canonical gradual-typing calculus wholesale. The important
lesson is that `Dynamic`, `Unknown`, `Any`, and an absent annotation are not synonyms.
If Phalcom chooses different semantics, encode those differences explicitly.

## 8. Refinement and flow typing

Study Kotlin smart casts, TypeScript narrowing, Flow/Pyright/mypy narrowing, and
occurrence typing literature for:

- predicate-derived refinements;
- path-sensitive facts;
- invalidation by mutation or aliasing;
- union elimination;
- exhaustiveness and unreachable branches.

Phalcom use:

- Option/Result handling;
- pattern matching;
- `is`/class checks if present;
- sealed/variant exhaustiveness;
- contract facts.

Never preserve a refinement across a mutation or unknown call unless the semantic
model proves that the relevant value cannot change.

## 9. Contracts and proving

### Hoare logic

Know:

- `{P} C {Q}`;
- preconditions;
- postconditions;
- loop invariants;
- weakest preconditions.

Phalcom use:

- `@requires`;
- `@ensures`;
- class/object invariants;
- local proof obligations.

### SMT and symbolic reasoning

Know enough to distinguish:

- abstract interpretation from symbolic execution;
- syntactic simplification from theorem proving;
- satisfiable from valid;
- unknown/timeout from false;
- path explosion;
- theory selection.

The static prover must never report a contract violation merely because an SMT solver
returned unknown or timed out.

## 10. Operational semantics

Learn small-step and big-step operational semantics as reasoning tools even if Phalcom
specifications remain primarily prose and executable tests.

Use operational rules when a feature's behavior becomes ambiguous around:

- evaluation order;
- exceptions/non-local return;
- closures;
- mutation;
- message dispatch;
- pattern matching;
- concurrency/yield points.

A compact rule can reveal semantic ambiguity that prose hides.

## 11. Object semantics and dispatch

Study Smalltalk and Self for:

- message sends rather than direct function calls;
- class/metaclass relationships;
- method dictionaries;
- late binding;
- `self` and `super` semantics;
- inline caches and shape/class guards.

Study Julia for a contrasting model of dynamic dispatch and specialization. Do not
import Julia-style multiple dispatch into Phalcom's ordinary selector lookup unless a
separate language decision explicitly changes dispatch semantics.

## 12. Incremental and query-oriented compilers

Study rust-analyzer/Salsa-style query systems and Roslyn's immutable semantic model
for:

- immutable snapshots;
- dependency tracking;
- memoized derived facts;
- fine-grained invalidation;
- IDE concurrency;
- cancellation and stale-request handling.

Phalcom currently has an explicit mutable worker + immutable publication model. The
lesson is not "rewrite it in Salsa"; the lesson is that every cached semantic result
must have a dependency story and every query must observe a coherent generation.

## 13. Parser and recovery theory

Know:

- recursive descent / Pratt parsing;
- FIRST/FOLLOW intuition;
- precedence and associativity;
- error productions;
- synchronization sets;
- lossless versus lossy trees;
- recovery nodes.

Semantic analysis for an editor consumes recovered syntax. Therefore malformed trees
are normal input, not exceptional input.

## 14. Model checking versus program verification

Do not conflate:

- type checking;
- linting;
- abstract interpretation;
- theorem proving;
- model checking;
- testing.

They answer different questions with different completeness and soundness tradeoffs.
Phalcom should compose them instead of making one mechanism pretend to replace all
others.

## 15. Reading protocol for agents

When a task needs theory:

1. Name the semantic problem first.
2. Identify the minimal theory that solves that problem.
3. State the assumptions required by the technique.
4. Check those assumptions against Phalcom.
5. Define the abstract domain/judgment/constraint relation explicitly.
6. State soundness direction: what can be missed versus what can be falsely claimed?
7. Specify termination and widening if iterative.
8. Specify diagnostics/provenance.
9. Only then choose data structures and code organization.

The goal is not academic ornament. Theory is useful when it prevents an implementation
from silently becoming order-dependent, unsound, non-terminating, or semantically
inconsistent.

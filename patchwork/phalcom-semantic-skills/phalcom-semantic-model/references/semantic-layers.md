# Semantic Layers and Ownership

## Purpose

This reference defines *where truth lives*. Most compiler/LSP architecture failures
come from putting a correct idea in the wrong layer, then duplicating it elsewhere.

## Layer 0 — Source and syntax

Owns:

- source text;
- tokens/trivia;
- recovered AST/CST structure;
- exact source ranges;
- syntactic categories and grammar recovery.

Does not own:

- which declaration a name refers to;
- which method a send reaches;
- inferred runtime class/type;
- project/module identity beyond syntax;
- path-sensitive truth.

The parser must preserve enough source structure that semantic targets can be mapped
precisely. Semantic analysis must tolerate recovered nodes.

## Layer 1 — Source targets

Maps offsets/ranges to meaningful source constructs.

Examples:

- identifier token in a declaration;
- identifier/reference expression;
- member selector fragment;
- argument label;
- class name;
- import binding/path;
- field target;
- type annotation target (future).

A source target is still source-oriented. It becomes semantic only after resolution.

## Layer 2 — Lexical scope and declarations

Owns:

- scope nesting;
- declarations introduced by parameters, locals, patterns, loops, imports;
- shadowing;
- declaration-before-use rules where applicable;
- mutability/category metadata.

Output should be identities (`BindingId`, `ScopeId`) rather than repeated textual searches.

## Layer 3 — Named semantic identities

Owns:

- module identity;
- class/protocol/type declaration identity;
- field identity;
- callable identity;
- canonical selector identity;
- resolved imports/module namespaces;
- occurrence targets for navigation/refactoring.

This layer answers "what entity is this?" independent of value/type inference.

## Layer 4 — Declaration surfaces

A surface is the statically knowable behavior/shape declared by an entity:

- class members by dispatch side;
- fields;
- visibility;
- superclass relation;
- constructor status;
- method parameters;
- source/native declaration metadata;
- protocol requirements and type metadata in the future.

Surfaces are not execution traces and should not depend on arbitrary call-site inference.

## Layer 5 — Local runtime-value knowledge

Owns facts derivable from syntax and local propagation:

- literal/class-object shapes;
- initializer values;
- assignment results;
- collection element joins;
- local family/callable identities;
- known module values.

In current LSP code this is the `ValueShape`/`InferredValue` domain. It is advisory and
must remain distinct from future language types.

## Layer 6 — Control-flow knowledge

Owns facts whose validity depends on reachability/program point:

- binding value before/after assignments;
- branch merge results;
- loop fixed points;
- reachable return values;
- throw/break/continue paths;
- captured writes;
- future type narrowing/refinements;
- future definite assignment/unreachable analysis.

Implementation may use structured recursive flow today and an explicit CFG later.
The semantic contract is independent of the storage representation.

## Layer 7 — Dispatch and call-site semantics

Given a receiver semantic description and selector, owns:

- possible/known receiver side;
- selector canonicalization;
- lookup target/candidate set;
- declaration owner selected through inheritance;
- dynamic/reflective uncertainty;
- call argument mapping;
- constructor/class-side behavior.

Do not let language type annotations change this layer unless a separate explicit
feature changes dispatch semantics.

## Layer 8 — Interprocedural summaries

Owns compact cross-call information:

- inferred parameter knowledge;
- return knowledge;
- direct callable dependencies;
- conservative effects;
- invoked higher-order parameters;
- future throw/yield/block/mutation/escape summaries;
- revision/fixed-point state.

Summaries are public semantic contracts between caller and callee analysis. Callers
should not need the callee's internal flow graph for ordinary queries.

## Layer 9 — Project/module fixed point

Owns:

- import/module graph;
- dependency closure;
- callable dependency graph;
- SCC/fixed-point solving;
- affected-frontier recomputation;
- publication generation.

It must ensure the resulting snapshot is coherent across files.

## Layer 10 — Language typing (future)

Owns normative type semantics:

- parsed/resolved type expressions;
- `TypeId` or equivalent canonical representation;
- substitution;
- subtyping/assignability/consistency relations;
- generic constraints;
- bidirectional checking;
- flow-refined types;
- type errors/obligations;
- erased vs reified metadata.

Typing consumes lower semantic layers. It should not duplicate lexical resolution,
dispatch surfaces, or module identity.

## Layer 11 — Static proof/effect systems (future)

Owns:

- propositions/path predicates;
- contract obligations;
- invariant checking;
- abstract interpretation beyond value/type inference;
- SMT-backed obligations if introduced;
- proof status: proved/refuted/unknown;
- effect domains.

A proof engine must be conservative. "Unable to prove" is not "false".

## Layer 12 — Consumers

Consumers render, enforce, or optimize shared semantics:

- LSP;
- checker;
- typed-runner;
- lints;
- refactorings;
- diagnostics;
- optimizer;
- documentation tooling;
- REPL semantic features.

Consumers may choose different policies for uncertainty but must not create incompatible
underlying facts.

## Ownership test

When unsure where code belongs, ask:

1. Would two consumers need the same answer?
2. Is the answer independent of how it is displayed/enforced?
3. Does it require semantic identity or flow rather than protocol/UI state?

If yes, it belongs below the consumer layer.

## Anti-pattern: feature-owned semantics

Bad:

```text
completion.rs -> walks AST -> guesses receiver class
hover.rs      -> walks AST -> independently guesses receiver class
checker.rs    -> later builds another inference engine
```

Good:

```text
semantic engine -> one receiver fact / dispatch query
completion -> ranks members
hover -> renders fact + confidence
checker -> applies normative type relation
```

## When an explicit IR becomes necessary

Do not introduce an IR merely because compilers often have one. Introduce a semantic
representation when source AST structure makes a required analysis awkward or duplicated.

Signals:

- several analyses independently reconstruct control flow;
- implicit control edges are easy to miss;
- path predicates are required;
- dominance/post-dominance matters;
- loops need reusable fixed-point machinery;
- desugared constructs should share semantics;
- source AST ownership makes interprocedural queries borrow-heavy;
- static proving needs explicit program points.

A future semantic IR can coexist with stack bytecode. Execution IR choice and analysis
IR choice are separate decisions.

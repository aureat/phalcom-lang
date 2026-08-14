# Current Phalcom Semantic Implementation Map

This document is an orientation aid, not a frozen API contract. Inspect current source
before editing.

## Repository-level sources

- `CLAUDE.md`: workspace layout, commands, conventions.
- `AGENTS.md`: graphify-first rules for codebase questions.
- `docs/spec/current/`: current normative language behavior.
- `docs/adr/`, `docs/pdr/`, `docs/decisions/`: design decisions and deferrals.
- `docs/spec/typing/`: proposed/incremental typing design; status matters.

## Front end

`phalcom-ast` owns lexer/token/AST/error structures. Semantic code should consume its
recovered `Program` and `SourceRange`s rather than re-lex source ad hoc.

## Runtime/compiler truth

`phalcom-core` contains:

- AST -> bytecode compiler;
- stack VM and frames;
- class/object/method/signature semantics;
- universe/modules/interner;
- native primitives;
- bootstrap core source.

When semantic behavior conflicts with compiler/VM behavior, determine whether the runtime
or semantic layer violates the normative spec. Do not "fix" tooling by inventing a third
behavior.

## `phalcom-lsp/src/semantic`

### `ids.rs`

Current stable/module-qualified semantic identities:

- `ModuleId`;
- `ClassId`;
- `CallableId`;
- `FieldId`;
- `DispatchSide`;
- core pseudo-module URI.

### `scope.rs`

Builds lexical scope graph with snapshot-local IDs:

- `ScopeId`;
- `BindingId`;
- `BindingInfo`;
- `SemanticBindingKind`;
- `NameResolution`;
- source-order visible binding queries.

It visits method/setter/index/closure/for/pattern/import declarations.

### `surface.rs`

Represents source declaration surfaces:

- module surface;
- classes;
- members;
- fields;
- visibility;
- parameters;
- callable IDs and dispatch sides.

This is the right layer for declaration facts that do not depend on arbitrary flow.

### `occurrence.rs`

Maps exact source ranges to semantic targets/roles for navigation/refactoring and targeted
LSP behavior.

### `facts.rs`

Current advisory knowledge domain:

- `ValueShape` with instances, class objects, modules, tuples, records, collections,
  callable/family and bounded unions;
- `Confidence`;
- `FactOrigin` provenance;
- `InferredValue`;
- `LocalFacts` by binding and source order;
- `FieldFacts` + evidence kind;
- `ParameterFacts`.

Important: source explicitly documents `ValueShape` as *not a language type*.

### `analyzer.rs`

Expression-level semantic analysis over a context. Reuse it instead of reimplementing
literal/send/field semantics in feature modules.

### `dispatch.rs`

Resolves semantic dispatch over receiver abstraction and class surfaces.

### `flow.rs`

Structured statement flow shared by local facts, callable summaries, field analysis and
call-site analysis.

It models concepts such as:

- `FlowState`;
- normal/return/break/continue/throw exits;
- resolved calls and argument evidence;
- field writes;
- block effects/captured writes/non-local returns;
- summary extraction.

This sharing is a strong architectural precedent: new flow-dependent features should
extend common flow machinery or a future common CFG, not spawn feature-specific walkers.

### `callable.rs`

Callable summaries include parameter values, return value, dependencies, conservative
effects, and semantic generation.

### `infer.rs`

Interprocedural solve and collection helpers. Inspect before adding a new fixed-point loop.

### `module_graph.rs`

Module dependency/resolution graph. Future full module/package implementation should evolve
this layer rather than bypassing it in checker/LSP modules.

### `engine.rs`

Mutable single-threaded worker state. Updates file contributions, refreshes module graph,
computes affected frontiers, solves summaries/parameter facts, rebuilds local/field facts,
and publishes generations.

The current architecture deliberately separates mutation from query snapshots.

### `snapshot.rs` / `query.rs`

Immutable query-facing state and generation/stamp concepts. LSP features should prefer
queries here over reaching into mutable engine internals.

### `invalidation.rs`

Incremental invalidation utilities. New caches/facts need explicit invalidation behavior.

## Legacy/index code

`phalcom-lsp/src/index.rs` contains older/parallel workspace indexing infrastructure for
selector definitions/references/class metadata. Before adding new behavior there, determine
whether the live semantic database now owns the answer. Prefer migration toward one semantic
truth rather than perpetuating duplicate indices.

## Typing specifications

Current typing documents establish a crucial direction:

- typing is optional/reflection-aware;
- type metadata must not implicitly change ordinary selector identity or method lookup;
- protocols/type descriptors are distinct semantic objects, not flags that mutate the
  dynamic class model;
- later documents are expected to define applied types, substitution, lattice/special
  types, variance/subtyping, inference, structural conformance, checker modes and tooling.

Treat status labels carefully: proposed design is not current runtime behavior.

## Commands

Typical verification anchors from current repository guidance:

```sh
cargo test
cargo clippy --workspace
scripts/test.sh ast
scripts/test.sh core
scripts/test.sh lsp
scripts/test.sh workspace
scripts/test.sh full
```

Use more focused commands during iteration and the repository's required gate before
claiming completion.

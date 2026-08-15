# Current Semantic Architecture

This is a map of responsibilities, not a frozen API. Inspect current source before editing.

## Inputs

Semantic analysis consumes recovered `phalcom_ast::ast::Program` values and
`phalcom_common::range::SourceRange`s. It should not maintain an independent parser.

## File semantic snapshot

A current file contribution conceptually contains:

```text
FileRevision
ModuleId
Arc<Program>
ModuleSurface
ScopeGraph
OccurrenceIndex
LocalFacts
FieldFacts
ParameterFacts
DependencySet
```

This is a useful separation: syntax/source-owned data plus derived semantic products,
all tied to one file/module revision.

## IDs

`semantic/ids.rs` provides module-qualified identities:

```text
ModuleId
ClassId(ModuleId, name)
CallableId(owner ClassId, selector, DispatchSide)
FieldId(owner ClassId, name, DispatchSide)
```

`semantic/scope.rs` provides snapshot-local:

```text
ScopeId
BindingId
```

These lifetime differences matter for caches.

## Surface collection

`surface.rs` is where declaration-level facts belong. Surface facts are knowable without
executing arbitrary method flow:

- classes;
- superclass names/identities as resolved;
- members;
- fields;
- parameters;
- side/visibility/member kind;
- source ranges;
- callable IDs.

If you need a fact for completion and checking at every call site, ask whether it is really
a surface fact before putting it in flow analysis.

## Scope graph

`scope.rs` builds lexical scopes and bindings from the program.

Important current behaviors include:

- nested method/index/block/for scopes;
- pattern/destructure bindings;
- import bindings;
- source-order declaration visibility;
- nearest-scope resolution;
- classes resolved separately from lexical bindings;
- `self`/implicit-self handling as a distinct resolution state.

When syntax changes, update this centrally.

## Occurrence index

`occurrence.rs` converts source ranges into exact semantic targets and roles.
This is the primary foundation for targeted hover, definition, references and rename.

Do not ask every LSP feature to rediscover the token under the cursor differently.

## Advisory fact domain

`facts.rs` currently models:

```text
ValueShape
Confidence
FactOrigin
InferredValue
LocalFacts
FieldFacts + FieldEvidence
ParameterFacts
```

`ValueShape` includes unknown, class instances/class objects/modules, structural collections,
callables/families and bounded unions.

Its source documentation intentionally says it is *not* the language type system.

## Expression analyzer

`analyzer.rs` evaluates expressions abstractly from an `AnalysisContext`. The context can
supply:

- current class/side;
- local environment/binding values;
- scopes/local facts;
- known classes;
- callable returns;
- field values;
- dispatch resolver.

New expression semantics should be centralized here or in a shared helper it calls.

## Dispatch

`dispatch.rs` handles receiver/member lookup over class surfaces. Keep it aligned with runtime
lookup and selector canonicalization.

## Structured flow

`flow.rs` models reachable statement flow with concepts such as:

```text
FlowState { bindings }
StatementFlow { normal, returns, breaks, continues, throws, tail_value }
ReturnEvidence
AnalyzedArgument
ResolvedCall
AnalysisEvent
BlockEffects
SurfaceFlowAnalysis
```

One major architectural strength: local facts, return summaries, field writes and call-site
parameter evidence share the same flow traversal.

Preserve that. If it becomes too complex, replace with a shared lower representation rather
than splitting into several walkers.

## Callable summaries and solver

`callable.rs` defines summary shape. `infer.rs` performs interprocedural solving/collection.

Summaries currently expose:

- parameter facts;
- return fact;
- dependencies;
- effects (dynamic send, invoked callable parameter positions);
- generation.

The engine compares summaries/parameter facts to discover additional affected modules.

## Module graph

`module_graph.rs` owns dependency relationships and dependent closure.
Future full module/package semantics should evolve this component rather than create a
checker-only import graph.

## Engine

`engine.rs` owns mutable semantic state and update transactions.

Current pattern:

```text
update one/batch files
  -> rebuild surface/scope/occurrence for updated files
  -> refresh module graph
  -> compute initial affected modules
  -> solve affected callable/parameter state
  -> expand frontier if changed summaries affect dependents
  -> rebuild local/field facts
  -> rebuild reverse callable dependency map
  -> update file snapshots
  -> increment/publish semantic generation
```

## Snapshot/query

`SemanticDb` publishes `Arc<SemanticSnapshot>` behind an `RwLock` while the mutable
`SemanticEngine` lives behind a worker `Mutex`.

Consumers should access immutable query methods. Do not make an LSP request hold the engine
mutation lock while doing protocol rendering.

## Legacy index

`phalcom-lsp/src/index.rs` predates/overlaps some semantic-database responsibilities.
Before extending it, ask whether the new `semantic` layer already owns the concept.
Migrate consumers toward shared semantics where practical; avoid permanent dual truth.

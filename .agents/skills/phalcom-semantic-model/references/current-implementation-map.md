# Current Phalcom Semantic Implementation Map

This file is an orientation map to the repository state inspected while deepening this skill on **2026-08-15**. It is not a normative language specification and must be rechecked before repository work. Paths, names and representations can change.

Use status labels rigorously:

```text
CURRENT      observed in current repository source/tests
NORMATIVE    established by current spec/ratified decision
PROPOSED     documented design not yet current behavior
EXPERIMENTAL repository experiment without normative guarantee
FUTURE       expected direction without ratified semantics
RECOMMENDATION guidance from this skill, not existing behavior
```

A repository implementation can itself be wrong relative to a normative spec. When they disagree, investigate rather than silently declaring either one authoritative for all purposes.

## 1. Repository guidance and specification roots

**CURRENT repository structure:**

- `AGENTS.md` and `CLAUDE.md` provide repository/agent guidance.
- `docs/spec/current/` contains current specification material.
- `docs/spec/next/` and other design-oriented spec areas contain forward-looking work whose status must be read, not assumed.
- `docs/spec/typing/` contains typing work; do not treat typing proposals as current runtime semantics merely because they are detailed.
- `docs/adr/` and `docs/pdr/` contain architecture/product decisions.
- `docs/implementation/` contains implementation-oriented documentation.
- `phalcom-ast/` owns lexer/parser/AST/source structures.
- `phalcom-core/` owns compiler/bytecode/VM/object-runtime behavior and core/bootstrap implementation.
- `phalcom-lsp/` contains the current semantic engine and LSP adapter.

Before a repository-specific claim, inspect the relevant current file and tests. This skill intentionally does not freeze line numbers.

## 2. Front end boundary

`phalcom-ast` is the source representation producer. Semantic analysis should consume recovered AST/source ranges rather than lexing or reparsing independently in each feature.

The semantic layer needs to preserve:

```text
source revision
exact source ranges
recovered structure
syntax errors/recovery state where relevant
```

without treating recovery artifacts as dynamic language semantics.

## 3. Compiler/runtime boundary

`phalcom-core` is essential whenever semantic analysis models:

- canonical selector construction;
- instance/class-side behavior;
- inheritance and `super`;
- constructors;
- control flow/non-local behavior;
- reflective objects/methods;
- core/native primitives;
- module/runtime behavior.

The semantic engine describes those dynamic semantics approximately for tooling. It must not create a more convenient but different object model.

## 4. `phalcom-lsp/src/semantic/mod.rs`

**CURRENT:** module-level documentation explicitly frames this subsystem as the semantic source of truth for editor intelligence and emphasizes that advisory value inference is not language typing. The module wires the semantic database and query-facing behavior.

The file also contains substantial semantic tests. Current observed tests cover, among other behavior:

- return propagation across call chains;
- recursive callables with concrete evidence converging;
- oversized incompatible return-shape unions widening to `Unknown`;
- same selector under different classes maintaining independent summaries;
- cross-module return/parameter propagation;
- independent leaf edits avoiding unrelated module recomputation;
- provider edits recomputing consumers;
- creation/removal of imported providers repairing/invalidating import resolution;
- caller edits removing stale parameter contributions;
- multiple consumers joining parameter evidence;
- unimported workspace classes not becoming magically visible;
- same-named imported classes retaining module qualification.

Treat these as strong regression anchors for the semantic doctrine.

## 5. `ids.rs`

**CURRENT:** stable/module-qualified identity structures include:

```text
ModuleId(String canonical URI)
ClassId { module, name }
CallableId { owner, selector, side }
FieldId { owner, name, side }
DispatchSide::{Instance, Class}
CORE_MODULE_URI = phalcom://core
```

Important consequences:

- class identity is module-qualified;
- callable identity includes complete canonical selector and side;
- field identity includes owner and side;
- the current module identity policy is URI-based/file-oriented.

Future package/module identity can evolve behind these conceptual boundaries; do not spread URI assumptions into unrelated semantic logic.

## 6. `scope.rs`

**CURRENT:** owns lexical scope/binding structure and source-order-aware name resolution. `ScopeId` and `BindingId` are file-snapshot-local compact IDs.

Semantic clients should resolve spelling to a binding once and carry the ID. They should not keep global `String -> inferred value` maps or assume binding IDs survive reparses.

When adding declaration-bearing syntax, inspect scope construction and visible-binding tests before changing completion or hover directly.

## 7. `surface.rs`

**CURRENT:** owns source declaration surfaces: modules/classes/members/fields/parameters and source-backed metadata used by dispatch and queries.

A surface is declaration knowledge, not an execution trace. Flow-derived field values or call-site parameter observations belong to fact/analysis layers, not the declaration surface itself.

## 8. `occurrence.rs`

**CURRENT:** owns semantic source occurrences/targets used for identity-based navigation/refactoring. This is the preferred substrate for definition/reference/rename semantics rather than global text matching.

Occurrence identity is source-revision-sensitive; the semantic target can be stable at a broader lifetime.

## 9. `facts.rs`

This is one of the most important current files.

### `ValueShape`

**CURRENT:** `ValueShape` is documented as advisory runtime value knowledge and explicitly not a language type. Current variants include:

```text
Unknown
Instance(ClassId)
ClassObject(ClassId)
Module(ModuleId)
Tuple
Record
List
Set
Map
Range
Callable(CallableId)
Family { receiver, base }
Union
```

### Join and widening

**CURRENT:** structured shapes join recursively where compatible; otherwise a bounded union is built. `MAX_SHAPE_UNION` is `8`. `Unknown` is contagious under ordinary shape join.

This is a deliberately finite/controlled editor approximation. Future type unions must not inherit this cap by accident.

### Confidence and provenance

**CURRENT:** `Confidence` includes:

```text
Exact
Flow
Interprocedural
Heuristic
```

`FactOrigin` includes source/call-related origins, and `InferredValue` stores `shape`, optional known-boolean information, confidence and bounded provenance. Joins keep a bounded evidence sample.

### Revisions and fact families

**CURRENT:** `FileRevision`, local binding facts, field facts/evidence and parameter facts are represented here.

### Contribution-indexed parameter facts

**CURRENT:** `ParameterContributions` indexes facts by stable parameter slot and `ContributionSource` (`Callable` or top-level module contribution). Replacing one source removes old contributions, inserts new ones, recomputes only touched slots and returns deltas.

This is a major architectural pattern: incremental edits need retraction, not append-only monotone joins.

## 10. `analyzer.rs`

**CURRENT:** owns expression-level semantic analysis against an explicit semantic context. It is the place to inspect before implementing feature-local literal/member/send/value inference.

Expression inference must agree with binding resolution, dispatch, class surfaces and current callable/field facts. Do not create a second expression interpreter inside completion or checker adapters.

## 11. `dispatch.rs`

**CURRENT:** centralizes semantic member/dispatch resolution over receiver knowledge and class surfaces. It is the correct place to inspect when changing inheritance/selector/instance-vs-class-side resolution.

Always compare any change here with compiler/VM selector and lookup behavior.

## 12. `flow.rs`

**CURRENT:** contains shared structured flow machinery used for semantic facts and summaries. It models reachability/flow state rather than representing unreachable code as `ValueShape::Unknown`.

The current architecture uses structured traversal rather than requiring every analysis to own a separate CFG. A future explicit CFG becomes justified when several analyses need reusable program points/edges/dominance/loop solving. Do not introduce separate checker/lint/prover CFGs independently.

## 13. `callable.rs`

**CURRENT:** owns callable-summary representation. Summaries are the boundary for interprocedural knowledge rather than requiring callers to traverse callee ASTs.

Inspect this file before adding cross-call facts such as new effects, mutation summaries, declared/inferred types or closure behavior.

## 14. `infer.rs`

**CURRENT:** orchestrates interprocedural semantic inference and dependency propagation. Observed behavior includes:

- analysis units/callable bodies are reprocessed through a worklist;
- caller/callee dependencies drive re-enqueueing;
- parameter contributions are replaced per source rather than only appended;
- changed summaries propagate to dependent callers;
- analysis is bounded operationally to protect editor latency;
- partial/incomplete source should not abort all semantic analysis.

Do not add another fixed-point loop in a feature module before establishing why this owner cannot express the new fact.

## 15. `module_graph.rs`

**CURRENT:** owns module imports, resolved targets and reverse dependents, with deterministic graph structures. A changed provider's affected frontier includes transitive dependents.

This is currently module-granularity dependency infrastructure. Future package/project/module specifications may require richer edge kinds and identities rather than bypassing this layer.

## 16. `invalidation.rs`

**CURRENT:** owns bounded source-change classification. `SourceChangeKind` distinguishes `BodyOnly`, `ImportSurface`, `DeclarationSurface`, `FileAddedRemoved`, and `CoreSurface`; `SourceDelta` additionally identifies body-local `changed_callables` and whether top-level executable source changed. Declaration fingerprints intentionally exclude source ranges and debug formatting, so a declaration moving in the file does not become a different semantic declaration merely because its offsets changed. Current tests verify that range shifts do not spuriously classify an untouched callable body as changed.

This classifier seeds the narrowest recomputation frontier that the current engine can justify. Do not generalize its categories into universal semantic neutrality: documentation comments, literal contents, source maps, parser layout/newline sensitivity, or future consumers can create dependencies not represented by today's core semantic classifier.

## 17. `snapshot.rs`

**CURRENT:** `SemanticSnapshot` is immutable published semantic state. It stores `Arc`-shared maps of:

```text
files/source products
class surfaces
callable summaries
field facts
parameter facts
module graph
```

Query methods resolve classes/members, occurrences, visible bindings, callable returns, parameter facts, field facts, expression inference and completion surfaces from that coherent snapshot.

This is the current concurrency/consistency boundary: mutable analysis occurs before publication; requests read an immutable generation.

## 18. `query.rs`

**CURRENT:** contains semantic generation/snapshot stamp query concepts. Keep file revisions, semantic publication generations and identity lifetimes conceptually distinct.

## 19. `core_source.rs`

**CURRENT:** semantic analysis integrates bundled/core source so tooling can consume language-visible core declarations rather than hard-coding every core member independently.

This is an important direction for future primitive metadata: generated/trusted native semantic metadata should feed the same semantic truth used by compiler bootstrap, docs and checker rather than growing editor-only knowledge.

## 20. LSP consumer boundary

The semantic subsystem already exposes queryable data structures. LSP handlers should adapt these answers into protocol objects. Before adding inference to an LSP handler, search for an existing semantic query or extend the semantic owner.

Consumer-specific concerns still belong in LSP code:

```text
completion ranking
markdown rendering
protocol ranges/capabilities
request cancellation
UI fallback policy
```

but not a duplicate name resolver or type/shape inference engine.

## 21. Legacy/parallel indexes

The repository contains older/parallel indexing code outside the semantic subsystem. Before extending such an index, determine whether it is still authoritative for the requested behavior or whether the semantic database now owns the identity/fact.

If two indexes are temporarily necessary, document synchronization and migration ownership. Silent duplicate truth is technical debt with semantic consequences.

## 22. Typing status discipline

`docs/spec/typing/` exists and is substantial. Its presence does not mean all described typing semantics are implemented in runtime/checker/LSP.

For every typing claim, label it:

```text
CURRENT implementation?
RATIFIED/NORMATIVE specification?
PROPOSED design?
FUTURE direction?
```

This skill's non-negotiable boundary remains: current `ValueShape` is advisory runtime-shape analysis, not the future canonical language type representation.

## 23. Neighboring skill boundary

The repository contains `patchwork/phalcom-semantic-skills/semantic-analysis-development/`. That skill is the implementation companion to this one. Its ownership includes concrete Rust development workflow, source walkers, CFG introduction, interprocedural coding recipes, semantic-query implementation, repository gates and performance instrumentation.

This `phalcom-semantic-model` skill owns the semantic doctrine those implementations must preserve: identity meaning, fact-domain contracts, bridges, uncertainty/provenance, dependency ownership and cross-consumer coherence.

Do not duplicate the implementation companion into this reference.

## 24. Verification anchors

Before a semantic repository change, use current repository guidance to choose focused and full gates. Typical current anchors include:

```sh
cargo fmt --check
cargo test -p phalcom-lsp
scripts/test.sh lsp
cargo clippy --workspace
scripts/test.sh workspace
```

Use narrower tests during iteration and broader gates when change scope requires them. Commands can evolve; re-read repository guidance rather than treating this list as permanent.

## 25. Repository-review questions

Before making a CURRENT claim, answer:

- Which current source file implements it?
- Which test asserts it?
- Does a current normative spec agree?
- Is a similarly named typing/design document only proposed?
- Is the behavior semantic truth or merely one LSP rendering policy?
- Does `phalcom-core` execute the same selector/control/object semantics?
- Does the current identity/fact lifetime match the cache being proposed?
- Does a neighboring implementation skill already own the coding pattern?

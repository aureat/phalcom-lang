# Modules, Dependency Ownership, Incrementality, and Semantic Generations

Incremental semantic analysis is not "cache the answer and clear it sometimes." It is a dependency-maintenance problem over semantic contributions. This reference defines the invariants that make editor snapshots correct, explains retraction under edits, and separates module graph semantics from callable/dataflow fixed points.

## 1. Correctness criterion

Let:

```text
Full(S)        = semantic facts computed from source state S from scratch
Incremental(S0 -> S1, Facts0) = facts after applying edit S0 -> S1
```

The core correctness property is observational equivalence:

```text
Observe(Incremental(S0 -> S1, Full(S0)))
  = Observe(Full(S1))
```

for every semantic query whose result is contractually observable.

Incrementality is an optimization of recomputation. It must not change semantics. A stale result that looks plausible is incorrect.

## 2. Source contribution model

Treat each source unit as contributing declarations, occurrences, edges and facts to a larger semantic database:

```text
source file revision
  -> parsed/recovered source products
  -> declarations/surfaces/scopes/occurrences
  -> dependency edges
  -> local facts
  -> callable/field/parameter facts
  -> project fixed point
  -> published snapshot
```

An edit conceptually replaces a contribution:

```text
Database' = Database - OldContribution(file) + NewContribution(file)
```

Derived facts that depended on the removed contribution must either be recomputed or be maintained from contribution-indexed evidence. Pure append-only joins are insufficient when edits can retract evidence.

## 3. CURRENT Phalcom publication architecture

**CURRENT:** `phalcom-semantic::SemanticWorkspaceSession` owns the mutable compiler semantic workspace, including `SemanticDb`, `TypeStore`, persistent module lifecycle, dependency/query products, and retained immutable snapshots.

A successful source/module mutation produces one `Arc<phalcom_semantic::SemanticSnapshot>`. `phalcom-lsp/src/analysis_service.rs` owns one persistent semantic session on its worker thread and forwards accepted compiler publications into `phalcom-lsp/src/publication.rs`. That publication cell contains only the latest immutable snapshot; it does not own semantic queries, identity translation, invalidation, or mutation.

Each LSP `RequestContext` pins one live document snapshot and one compiler snapshot and classifies source coherence as `Exact`, `Stale`, or `Unmapped`. Semantic feature requests therefore observe one coherent compiler generation. Stale or absent compiler products do not authorize request-time reconstruction of semantics in the LSP.

This architecture establishes the invariant:

> A query observes one coherent compiler semantic generation, not a mixture of generations and not an LSP-local reconstruction.

Preserve that invariant even if future semantic storage or scheduling changes.

## 4. File revision versus semantic generation

These identifiers answer different questions:

```text
FileRevision:
  which version of one source document contributed this file state?

SemanticGeneration:
  which coherent project-wide publication does this fact/query belong to?
```

A batch can update several file revisions and publish one semantic generation. A non-semantic edit may update a file revision without requiring all derived semantic facts to change. Do not use one number for both concepts.

A future query stamp may include:

```text
SnapshotStamp = (semantic generation, relevant file revision(s), optional runtime/spec version)
```

according to the query's validity contract.

## 5. Dependency graphs need edge meaning

A graph node/edge must say *why* a fact depends on another fact. Future Phalcom can have several graph layers:

```text
module namespace/import dependency
class inheritance dependency
callable summary dependency
parameter-contribution dependency
field-fact dependency
type-signature dependency
proof obligation dependency
runtime initialization dependency
package/build/native dependency
```

Do not make every relation one untyped `depends_on` set if invalidation semantics differ. A typed edge kind can permit narrower recomputation and better debug traces.

## 6. CURRENT module and semantic dependency graphs

**CURRENT:** `phalcom-modules::WorkspaceModuleSession` owns persistent project/source/module identity, linking, canonical resolved imports, and module generation. `phalcom-semantic::SemanticWorkspaceSession` consumes those compiler module products and publishes the canonical `SemanticGraph` together with query dependency/reuse state.

The LSP owns neither an import graph nor a semantic dependency graph. Workspace discovery and file watching submit source mutations; `phalcom-modules` and `phalcom-semantic` decide module identity, import targets, invalidation, and semantic dependencies.

Current regression tests require that unresolved relative imports can survive until a provider appears, provider/source changes update canonical linked products, removed sources retract their semantic contributions, and imported declarations retain module-qualified identity.

Do not introduce an LSP-owned module graph to accelerate editor requests. Add the needed immutable compiler/module query product instead.

## 7. Unresolved dependencies are still dependencies

Suppose:

```phalcom
import "./provider" as Provider
```

and `provider.ph` does not exist. If the semantic graph simply omits the edge, later creation of `provider.ph` provides no reverse link telling the engine to revisit the consumer.

Represent unresolved dependencies with enough identity to match future providers:

```text
UnresolvedImport {
  importer,
  requested_specifier,
  source_range,
  local_alias,
}
```

A filesystem/package/module event can then re-resolve the edge.

## 8. Invalidation is reverse dependency reachability

Let `Dep(a, b)` mean fact/node `a` directly depends on `b`. If `b` changes semantically:

```text
Invalidate(b) = {b} ∪ {a | a ->* b in the reverse dependency graph}
```

In practice, the frontier is typed and conditional. A method-body-only change may leave the class member surface unchanged, so completion surfaces need not be invalidated even if callable summaries are. Conversely, changing a selector affects occurrences, class surface, dispatch, callers and perhaps docs/reflection.

The semantic contribution delta matters more than the text delta.

## 9. Change classification

A useful pipeline distinguishes:

```text
text changed
  -> parse/source products changed?
  -> declaration surface changed?
  -> dependency edges changed?
  -> flow/summary facts changed?
  -> consumer-visible semantic query changed?
```

**CURRENT:** `invalidation.rs` classifies replacement into `BodyOnly`, `ImportSurface`, `DeclarationSurface`, `FileAddedRemoved`, or `CoreSurface`. A `SourceDelta` separately records the exact changed callable seeds for body-only edits and whether top-level executable source changed. Declaration fingerprints compare semantic declaration properties—class identity/superclass, selector, side, member kind, visibility, constructor/native metadata, parameters, and fields—rather than source ranges or debug formatting. Body-local source comparison then identifies which callable bodies actually changed. Current tests explicitly verify that a range shift caused by inserting text before an otherwise unchanged callable does not spuriously mark that callable changed.

This is a useful architectural distinction: source position is evidence/location, not declaration identity. The classifier is still a validity proof for specific downstream facts, not permission to call arbitrary edits “semantic no-ops.” Comments may matter to documentation tooling, literal contents matter to value facts, imports change the module graph, and newline/layout changes can alter parsing. Change classification must therefore remain consumer- and grammar-aware.

## 10. Cache contract

Never propose a cache without all of:

| Component | Required question |
|---|---|
| Key | What semantic identity/request selects the entry? |
| Value | What derived fact is stored? |
| Dependencies | Which source/fact versions were read? |
| Validity | Under exactly what condition is the value reusable? |
| Invalidation | Which events make validity false? |
| Concurrency | Can readers observe mutation? |
| Memory bound | What evicts/limits entries? |
| Determinism | Can recomputation order change observable output? |

A cache keyed only by `(file, offset)` is invalid if its result depends on imported class surfaces that can change without editing the file.

## 11. Versioned dependency contract

A robust conceptual cache entry is:

```text
Entry<K, V> {
  key: K,
  value: V,
  deps: [(DependencyId, Version)],
  built_in_generation: G,
}

valid(entry, snapshot) =
  for every (dep, version) in entry.deps:
      snapshot.version(dep) == version
```

The current engine may use coarser generation/frontier replacement rather than this exact representation. The semantic contract remains useful: every derived fact has a validity condition owned by dependencies, not by elapsed time or hope.

## 12. Retraction: the hard part of incrementality

Fixed-point joins are often monotone inside one generation. Edits are not. If a source stops contributing evidence, that evidence must disappear.

Bad append-only maintenance:

```text
old callers: Cat
edit caller: Dog
joined parameter = Cat ⊔ Dog     // stale Cat survives
```

Correct contribution ownership:

```text
Contrib[slot][source] = fact
Joined[slot] = ⊔ source contributions

replace(source):
  remove all previous entries owned by source
  add new entries
  recompute touched joins
  propagate only changed joined facts
```

**CURRENT:** `ParameterContributions` in `facts.rs` uses this pattern and indexes slots by contribution source. Current tests verify stale caller parameter facts are removed after an edit.

Use this pattern for future cross-file aggregates when evidence can disappear: protocol-conformance witnesses, field-write sets, effect contributors, call-graph edges, inferred type constraints, proof assumptions, and similar facts.

## 13. Local recomputation versus transitive propagation

A good incremental update has two phases:

```text
1. Rebuild/replace local contribution for changed source.
2. Propagate semantic deltas through reverse dependencies until stable.
```

Do not enqueue every dependent merely because a source file changed if the semantic product they consume is equal after rebuilding. Delta propagation can stop when:

```text
new_fact == old_fact
```

under the fact domain's semantic equality.

This is how incremental cost approaches the changed semantic frontier instead of workspace size.

## 14. Callable dependency worklists

Interprocedural inference is another dependency graph:

```text
caller summary depends on callee summary
callee parameter fact depends on caller call-site contribution
```

When a summary changes:

```text
enqueue callers(summary_target)
```

When a caller edit changes parameter contributions:

```text
recompute touched parameter slots
if parameter fact changes:
    enqueue owning callable
```

**CURRENT:** Phalcom's semantic inference implements a worklist and contribution-aware parameter propagation. Treat this as semantic infrastructure, not LSP-specific convenience.

## 15. Cycles: module graph and callable graph are different semantics

A graph algorithm can find SCCs in both, but the meaning differs.

### Callable cycle

```text
A -> B -> A
```

Usually means solve summaries to a fixed point with conservative seeds/widening/budget.

### Module cycle

```text
module A imports B
module B imports A
```

May affect namespace availability, initialization ordering, partially initialized module state, type declarations and runtime side effects. A module SCC is not automatically valid just because a dataflow solver converges. Module-cycle legality/dynamic initialization behavior belongs to the module specification.

Do not cargo-cult callable SCC semantics into module loading.

## 16. Immutable publication and copy-on-write representation

The key query invariant is immutability after publication:

```text
worker mutable state
  -> finish all affected analysis
  -> construct snapshot G+1
  -> atomically publish Arc<Snapshot G+1>

request R1 keeps Arc<Snapshot G>
request R2 starts later and gets Arc<Snapshot G+1>
```

This prevents a request from observing half-rebuilt maps. Internally, `Arc`-shared maps/copy-on-write can keep publication cheap when much of a snapshot is unchanged. Representation choice may evolve; snapshot coherence must not.

## 17. Cancellation and stale requests

LSP requests can outlive edits. Two valid policies are:

```text
snapshot semantics:
  request completes against the snapshot it started with

cancellation semantics:
  request is cancelled/restarted when a newer relevant generation exists
```

Whichever policy is chosen, do not let one request read some facts from `G` and others from `G+1` unless the query API explicitly defines that behavior.

## 18. Source identity stability and cache validity are independent

A declaration ID can remain stable while its facts become invalid:

```text
same CallableId
body changed
return summary changed
```

Conversely, source ranges can move while semantic facts remain equivalent. Do not use "same ID" as a substitute for fact versioning, and do not use "range changed" as proof that semantic identity changed.

Keep separate:

```text
identity equality
structural/source equality
file revision
semantic generation
fact version/equality
cache validity
```

## 19. Whole-file replacement can be the right granularity

Fine-grained incremental parsing is not a prerequisite for semantic incrementality. A practical architecture can:

```text
reparse one changed file completely
replace its source contribution completely
recompute only semantic dependents
```

This is often easier to reason about and fast enough. Introduce finer-grained syntax/source incrementality only when measured costs justify the complexity.

Do not trade stale-fact risk for theoretical minimal recomputation.

## 20. Project/package future

Reserve conceptual room for:

```text
Workspace
  PackageInstance(name, version/source identity)
    LogicalModule
      SourceContribution(s)
      declarations
```

Before making package-aware IDs permanent, decide:

- whether two copies of one package version are the same semantic package instance;
- whether lockfile resolution participates in identity;
- whether generated/native modules share logical namespace with source modules;
- visibility across package boundaries;
- how core/std versioning participates;
- how package replacement invalidates semantic facts.

This skill owns the identity/invalidation questions; concrete package resolution belongs to module/package skills/specifications.

## 21. Core/native/FFI semantic floor

Tooling needs semantic declarations for behavior implemented natively. Prefer a shared source of truth:

```text
visible source declaration + trusted native metadata
or generated semantic stub from primitive declarations
or explicit native semantic signature consumed by all semantic clients
```

Do not hard-code special completion/type behavior that the checker/docs/compiler cannot see. Native metadata changes must invalidate facts derived from them just like source declaration changes.

## 22. Reflection and open-world invalidation

If Phalcom permits runtime mutation of method tables/classes/modules, static dependency graphs describe source-known behavior, not necessarily all runtime behavior. An optimizer or proof engine may need:

```text
dispatch-table version
global mutation epoch
sealed/frozen class guarantee
closed-world build mode
runtime guard + fallback
```

LSP advisory analysis can remain conservative. Do not claim a call graph is closed merely because every source declaration is indexed.

## 23. Diagnostic traces for incremental bugs

Incremental bugs are difficult because stale results often look valid. Provide debug/test observability such as:

```text
changed source contribution
change classification
modules recomputed
callables reanalyzed
parameter slots recomputed
summary deltas
reverse dependency reason for each enqueue
published generation
```

A "rebuild everything" fallback can restore correctness during debugging, but it should not hide the missing dependency permanently.

## 24. Performance model

Measure at least:

```text
parse/rebuild time for changed file
number of modules in affected frontier
number of callable summaries reanalyzed
number of parameter/field facts recomputed
allocations/clones during publication
snapshot memory retained by concurrent requests
query latency against immutable snapshot
```

A cache or finer invalidation scheme is justified only if the measured cost matters and the validity contract remains explicit.

## 25. Testing obligations

Incremental tests should include:

- independent leaf edit does not recompute unrelated module;
- provider surface change recomputes consumers;
- provider body-only change propagates only to semantic products that depend on body facts;
- provider creation repairs unresolved imports;
- provider removal retracts declarations/edges/facts;
- caller edit retracts old parameter contribution;
- adding and then deleting a call site returns to the pre-add fact;
- recursive callable SCC converges after one member changes;
- batch update publishes one coherent generation;
- recovery edit followed by valid source repairs blocked facts;
- semantically neutral edit preserves relevant facts/generation behavior according to policy;
- incremental final facts equal clean full rebuild;
- randomized edit sequences periodically compared with full rebuild;
- memory does not grow unbounded from retained obsolete snapshots/contributions.

The most important property test is:

```text
incremental(source_history.last) == full_rebuild(source_history.last)
```

for the semantic queries under test.

## 26. Failure modes to reject

Reject designs that cache without dependency/version ownership, invalidate only the edited file when imports/calls cross files, retain append-only joined evidence across edits, treat module and callable cycles as the same semantics, publish partially updated state, use one global mutable semantic map directly from request handlers, use timestamps as semantic validity, clone foreign facts into consumers without invalidation edges, or solve stale-state bugs permanently by rebuilding the entire workspace on every keystroke.

## 27. Review questions

Before approving incremental architecture, answer:

- What is the source contribution unit?
- What is replaceable/retractable?
- Which graph edges exist and what does each mean?
- What exactly causes each edge to invalidate?
- What fact equality stops propagation?
- What is file revision versus semantic generation?
- Can queries observe mixed generations?
- How are unresolved dependencies retained?
- How are old contributions removed after edits/deletion?
- What happens in callable and module cycles?
- Is runtime reflection outside the static dependency model, and if so what guard applies?
- Is final incremental state tested against a full rebuild?
- Is cost proportional to the changed semantic frontier in common cases?

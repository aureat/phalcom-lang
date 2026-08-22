# Phalcom transfer architecture and type philosophy

## 1. Scope and purpose

This dossier is the target architecture. Earlier dossiers explain individual Pyrefly mechanisms; this one decides how they fit Phalcom's language, runtime, native surface, modules, LSP, and semantic model.

The target is a stamped, demand-driven semantic database with immutable published generations and worker-owned mutable construction. It uses canonical semantic identities, indexed bindings, flow versions, a canonical type store, bounded constraints, SCC answer publication, explicit module interfaces, open-world dispatch, and separate diagnostic products.

## 2. Evidence boundary and pinned source

**OBSERVED / PYREFLY** observations use /tmp/pyrefly-analysis-20260822 at commit 43467e64e36550f232a18e89f24fda79b1020b6.

**CURRENT / PHALCOM** observations use the checkout at /Users/altunhasanli/dev/phalcom/phalcom.

| Boundary | Evidence |
| --- | --- |
| identity/equality/type store | completed dossier 04; phalcom-semantic type modules |
| constraints and SCCs | completed dossier 01 |
| staged semantic products | completed dossier 02; phalcom-lsp/src/semantic/engine.rs |
| bindings and flow | completed dossier 03; current semantic source |
| answer cells and publication | completed dossier 05 |
| invalidation | completed dossier 06; phalcom-lsp/src/semantic/invalidation.rs |
| worker and snapshots | phalcom-lsp/src/analysis_service.rs, semantic/snapshot.rs |
| modules and runtime graphs | phalcom-modules/src/resolver.rs, interface.rs, graph.rs |
| diagnostics and LSP | phalcom-semantic/src/diagnostic.rs, phalcom-lsp/src/diagnostics.rs, backend.rs |

Pyrefly source mirror: [commit 43467e64e36550f232a18e89f24fda79b1020b6](https://github.com/facebook/pyrefly/tree/43467e64e36550f232a18e89f24fda79b1020b6).

## 3. Executive conclusion

Transfer architecture:

~~~text
SourceStore + ProjectUniverse
    -> canonical ModuleId and SourceRevision
    -> module interfaces and import environment
    -> indexed bindings, scopes, flow versions
    -> demand-driven semantic queries
    -> canonical TypeStore and relation engine
    -> callable/dispatch/module products
    -> SCC-safe answer publication
    -> immutable SemanticGeneration
    -> diagnostics, traces, CLI, and LSP
~~~

Phalcom should transfer Pyrefly's ownership, staging, invalidation, and bounded solving ideas. It should not transfer Python's semantic assumptions, closed-world type identity, or a type checker that ignores runtime dispatch, reflection, native descriptors, and dynamic values.

## 4. Pyrefly execution path

Pyrefly gives Phalcom these architectural mechanisms:

1. module-oriented staged work;
2. demand-driven answer cells;
3. explicit query identity;
4. placeholders and SCC fixed points;
5. immutable or atomically replaced products;
6. dependency keys finer than “module changed”;
7. separate traces and errors;
8. bounded relation and equality work;
9. retention and eviction policies;
10. worker scheduling with stale-result rejection.

Phalcom maps them into language-specific products:

1. source surfaces and declaration shells;
2. binding and flow facts;
3. callable and selector dispatch summaries;
4. module interfaces and reflection exports;
5. runtime/native descriptors;
6. ValueShape and evidence facts;
7. type relations and constraints;
8. LSP snapshots and diagnostic products.

## 5. Concrete data structures

### Identity layer

Use separate identities:

- FileId for canonical source;
- ModuleId for logical module;
- BindingId for declaration/use binding;
- ScopeId for lexical scope;
- CallableId for callable declaration;
- SelectorId for raw message selector;
- ClassId for nominal class identity;
- TypeId or TypeRef for canonical type-store entries;
- QueryId for semantic computation;
- DiagnosticId for user facts.

Selector identity must not be derived from receiver type. A dynamic receiver can receive a known selector without a closed class hierarchy.

### Flow layer

Use binding-keyed flow versions:

~~~text
FlowVersion {
    callable,
    block,
    binding,
    program_point,
    value_shape,
    provenance,
}
~~~

Branch joins produce explicit phi facts. A flow fact is not a global inferred declaration type.

### Type layer

TypeStore owns canonical construction and semantic equality. It may contain nominal, structural, callable, union, intersection, dynamic, unknown, error, and runtime-backed descriptors.

Do not describe a TypeHeap-like structure as fully arena-interned unless allocation and lifetime evidence proves it. Keep equality, simplification, canonicalization, and relation solving separate.

### Query layer

Each query has:

~~~text
QueryKey {
    kind,
    semantic_identity,
    input_generation,
    dependency_fingerprint,
}
Answer {
    value,
    status,
    provenance,
    dependencies,
}
~~~

Status includes computed, placeholder, unknown, error, cancelled, and stale. These statuses are not interchangeable.

## 6. State machines and transitions

### Semantic generation

~~~text
source revision R
    -> candidate state
    -> affected closure
    -> query/SCC computation
    -> normalized products
    -> diagnostic/trace products
    -> published generation G
~~~

### Query answer

~~~text
Absent
    -> Computing
    -> Placeholder
    -> Converging
    -> Computed
    -> Stale/Evicted
~~~

### Module interface

~~~text
Source
    -> DeclarationSurface
    -> ExportSurface
    -> LinkedModuleInterface
    -> ModuleEnvironment
    -> SemanticSummary
~~~

Every transition carries source and project revisions.

## 7. Cache keys and validity

Cache validity rules:

1. semantic identity and source revision are separate;
2. a type-store ID does not prove a query answer is current;
3. a module interface does not prove callable bodies are current;
4. a diagnostic generation does not prove semantic facts are reusable;
5. an LSP request must pin both document revision and semantic generation;
6. native descriptors carry their own catalog revision;
7. reflection or dynamic summaries use an explicit open-world policy.

Dependency fingerprints should name the demanded facts, not only the module. A callable that reads selector existence, class inheritance, parameter facts, or a module export has different invalidation needs.

## 8. Ownership and concurrency

Target ownership:

~~~text
request threads  -> immutable SemanticSnapshot
analysis worker  -> mutable candidate SemanticState
solver context   -> local vars, constraints, SCC state
project model    -> source/module/interface revisions
publication      -> one generation commit
diagnostic store -> immutable per-generation facts
~~~

Worker ownership is the first safe implementation. Parallel module jobs may compute isolated products from immutable snapshots and return them to the owner for deterministic publication.

Do not share mutable type variables across jobs. Do not let LSP request code initiate an unbounded semantic computation while holding protocol state.

## 9. Memory and allocation

Recommended retention:

- canonical identities and compact indexes are long-lived;
- source and syntax products are revision-scoped;
- interfaces outlive bodies when dependents need exports;
- callable summaries outlive local flow tables when no body query needs them;
- traces and explanations are demand-retained;
- diagnostics are per-file and generation-scoped;
- old generations survive only while readers hold them.

Use structural sharing between snapshots. Add arena allocation, interning, or raw publication only behind measured seams.

Open-world data should be compact: selector summaries, native member descriptors, dynamic capability sets, and reflection metadata should not copy entire class or module surfaces.

## 10. Complexity and performance

Target complexity:

- identity lookup: O(1) indexed access;
- scope lookup: O(lexical depth) or indexed parent walk;
- flow join: O(changed bindings);
- interface update: O(changed declarations/exports plus reverse dependents);
- query cache lookup: O(1) average;
- SCC solve: proportional to SCC work times bounded iterations;
- type relation: memoized by relation key with depth/gas limits;
- snapshot publication: O(changed product maps) with sharing.

Measure per product and per invalidation reason. A single aggregate analysis time cannot decide whether to optimize identity, parsing, type relations, graph traversal, or publication.

## 11. Failure, cancellation, recursion, and cycles

Use:

- Unknown for insufficient but non-fatal knowledge;
- Dynamic for runtime-open behavior;
- Error for invalid source or contract violations;
- Placeholder only inside bounded recursive computation;
- Cancelled for uncommitted work;
- Stale for valid old work rejected by publication.

Semantic cycles may converge through SCC solving. Runtime initialization cycles remain governed by the module runtime graph. Dispatch cycles and reflection loops require bounded traversal and explicit unknown fallback.

An internal failure preserves the last published generation. A failed candidate must not partially overwrite maps, diagnostics, or graph indexes.

## 12. Phalcom mapping

| Pyrefly mechanism | Transfer classification | Phalcom target |
| --- | --- | --- |
| staged module products | Transfer directly | interface/body/summary products |
| calculation cells | Adapt | stamped QueryCell with safe mutex first |
| SCC iteration | Transfer directly | solver-local SCC work and batch commit |
| recursive placeholders | Adapt | unknown/placeholder facts with Phalcom type semantics |
| ModuleDeps | Adapt | demand-specific semantic dependency keys |
| ArcSwap product replacement | Adapt | worker-only immutable snapshot publication |
| raw AnswerSlot pointer tagging | Do not transfer initially | no unsafe publication contract |
| TypeHeap seam | Transfer as boundary | TypeStore, without claiming full interning |
| Python type rules | Do not transfer | Phalcom type/spec authority |
| trace side effects | Transfer directly | separate TraceFact products |
| error collector | Adapt | DiagnosticFact and policy store |
| module resolver | Transfer directly | Phalcom resolver/project universe |
| Python import fallback | Do not transfer | Phalcom module rules |
| open-world/dynamic summaries | Transfer as inspiration | selector/native/reflection facts |
| LSP stamped reads | Transfer directly | SemanticSnapshot and SnapshotStamp |
| worker cancellation | Transfer directly | WorkerShared epoch and cancellation |
| Pyrefly retention heuristics | Adapt | measured Phalcom retention policy |
| benchmark evidence discipline | Transfer directly | acceptance matrix and metrics |

This table is the transfer contract. Mechanisms marked Do not transfer require an explicit new design, not an implicit copy.

## 13. Mechanisms not copied

Do not copy:

- Python's nominal and structural rules as Phalcom semantics;
- a closed-world assumption for selectors, native members, or reflection;
- type identity as a raw allocation address;
- inference facts as runtime values;
- runtime values as static proof without evidence;
- diagnostics as cache validity;
- module paths as logical identity;
- a global solver cache that ignores flow version and dependency fingerprint;
- unsafe parallel mutation before ownership and memory ordering are proven.

## 14. Proposed Phalcom data structures

~~~text
SemanticDb {
    project: ProjectModel,
    sources: SourceStore,
    modules: ModuleStore,
    interfaces: InterfaceStore,
    identities: IdentityStore,
    types: TypeStore,
    queries: QueryStore,
    diagnostics: DiagnosticStore,
    traces: TraceStore,
}

SemanticSnapshot {
    stamp: SnapshotStamp,
    files,
    modules,
    interfaces,
    types,
    summaries,
    dispatch,
    diagnostics,
    traces,
}

CallableSummary {
    callable,
    parameters,
    return_knowledge,
    constraints,
    dispatch,
    dependencies,
}

DispatchSummary {
    selector,
    receiver_knowledge,
    candidates,
    native_fallback,
    dynamic_open,
}
~~~

The exact storage can evolve; the ownership and identity fields cannot be omitted.

## 15. Proposed APIs and module seams

Core seams:

- SemanticDb::snapshot();
- SemanticDb::query(key);
- SemanticEngine::apply(change, cancel);
- TypeStore::intern(type);
- TypeStore::semantic_eq(left, right);
- RelationEngine::relate(left, relation, right, context);
- BindingIndex::resolve(scope, name, point);
- FlowFacts::value_at(binding, point);
- ModuleStore::interface(module, stamp);
- DispatchStore::summary(selector, receiver, stamp);
- Publication::commit(candidate);
- DiagnosticStore::for_generation(stamp);
- LspQuery::request(snapshot, document_revision, params).

Keep relation solving separate from name lookup, dispatch, module resolution, and rendering.

## 16. Implementation order

1. Establish identity and stamp types.
2. Normalize module/source/interface products.
3. Index bindings and flow versions.
4. Introduce TypeStore and semantic equality.
5. Add relation interface and bounded constraints.
6. Add query identities and SCC answer publication.
7. Connect module interfaces and demand invalidation.
8. Add callable/dispatch summaries, preserving dynamic/native behavior.
9. Attach diagnostics and traces to generations.
10. Expose stamped LSP snapshots.
11. Measure, then optimize storage and parallelism.

## 17. Tests

Tests must prove:

- one source declaration has one stable BindingId within a revision;
- selector identity survives receiver uncertainty;
- type equality is separate from runtime object equality;
- dynamic/open-world dispatch is not narrowed into a false closed set;
- native descriptors participate in member lookup;
- reflection exports remain visible under the specified policy;
- flow versions distinguish branch facts;
- module interface changes invalidate only demanded dependents;
- query answers converge for recursive SCCs;
- stale generations cannot answer newer LSP documents;
- diagnostics and traces match the committed generation.

## 18. Benchmarks and metrics

Measure:

- identity lookup and index density;
- TypeStore hit rate and memory;
- type equality depth/cap fallbacks;
- relation cache hits/misses;
- constraint rounds and SCC iterations;
- callable summaries reused;
- dispatch candidate counts;
- module-interface invalidation breadth;
- snapshot bytes shared;
- LSP stale rejection and latency;
- diagnostic and trace retention.

Required comparison: current engine versus each new seam on clean build, body edit, declaration edit, import edit, and workspace scan.

## 19. Risks and open questions

- Which Phalcom type relations are specification-authoritative and which remain exploratory?
- How should TypeStore represent recursive nominal and structural types together?
- Which runtime/native facts may be promoted to static evidence?
- How much reflection can be modeled without false precision?
- Should selector-family summaries be a dispatch product or a type-store product?
- Which query identities must survive source movement?
- What is the first accepted dynamic/open-world fallback?
- Can one worker deliver required LSP latency before parallelism?

These are **OPEN / UNVERIFIED** until source/spec and benchmark decisions are recorded.

## 20. Final transfer checklist

- [x] Target architecture and ownership boundary defined.
- [x] Semantic identities separated from runtime values and rendering.
- [x] TypeStore/equality/relation boundaries specified.
- [x] Bindings, flow versions, query keys, callable summaries, and dispatch included.
- [x] Module interfaces and invalidation included.
- [x] Dynamic, reflection, native, and runtime semantics preserved.
- [x] Every major Pyrefly mechanism classified as direct transfer, adaptation, inspiration, or exclusion.
- [x] Worker-owned mutable state and immutable LSP snapshots retained.
- [ ] Type philosophy reconciled with all current specifications.
- [ ] Target APIs implemented behind tested seams.

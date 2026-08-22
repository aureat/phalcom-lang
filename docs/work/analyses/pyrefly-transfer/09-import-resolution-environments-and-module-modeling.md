# Pyrefly import resolution, environments, and module modeling

## 1. Scope and purpose

This dossier defines the module boundary that semantic analysis consumes. It covers canonical module identity, import tracking, interfaces, re-exports, aliases, missing environments, dependency phases, builtins, and runtime/native edges.

The transfer decision is that Phalcom already has a stronger module foundation than a generic “import graph.” Its resolver, interfaces, reference graph, semantic graph, runtime dependency graph, builtin catalog, and project universe should remain authoritative. Pyrefly supplies useful demand and invalidation ideas, but it must not replace Phalcom's logical module model.

## 2. Evidence boundary and pinned source

**OBSERVED / PYREFLY** observations use /tmp/pyrefly-analysis-20260822 at commit 43467e64e36550f232a18e89f24fda79b1020b6.

| Mechanism | Pinned source |
| --- | --- |
| Module dependency keys and change classification | pyrefly/lib/state/state.rs |
| Import tracking and dependency tests | pyrefly/lib/state/import_tracker.rs |
| Staged module products | pyrefly/lib/state/steps.rs |
| Module load and module state | pyrefly/lib/state/module.rs |
| Pyrefly LSP import/type resolution | pyrefly/lib/state/lsp.rs |
| Phalcom resolver and interface cache | phalcom-modules/src/resolver.rs |
| Phalcom interface and export products | phalcom-modules/src/interface.rs |
| Phalcom source canonicalization | phalcom-modules/src/source.rs |
| Phalcom import/reference/semantic/runtime graphs | phalcom-modules/src/graph.rs, module_graph.rs |
| Phalcom module and linker tests | phalcom-modules/tests/integration.rs, linker.rs, graph.rs, repair_regressions.rs |

Source mirror: [Pyrefly commit 43467e64e36550f232a18e89f24fda79b1020b6](https://github.com/facebook/pyrefly/tree/43467e64e36550f232a18e89f24fda79b1020b6).

## 3. Executive conclusion

An import is not one dependency. Pyrefly distinguishes existence, name existence, metadata, wildcard, re-export, class, type-alias, and export-origin dependencies. This lets a change to one module invalidate only consumers whose demanded fact can change.

Phalcom models several dimensions explicitly:

- canonical logical ModuleId;
- project universe and source provider;
- builtin and std roots;
- interface declarations and exports;
- unresolved and resolved import edges;
- interface versus runtime dependency phase;
- semantic SCCs distinct from runtime initialization cycles.

**PROPOSED / PHALCOM:** keep these products distinct and add an environment status to every module interface. A missing or partial environment should degrade dependent semantic facts to unresolved/unknown states while retaining the import edge and diagnostic explanation. It must not collapse all missing-dependency behavior into an empty module.

## 4. Pyrefly execution path

The relevant Pyrefly path is:

1. a module path and module name identify a load;
2. staged Load, Ast, Exports, Answers, and Solutions products are computed on demand;
3. import tracking records demanded properties of imported modules;
4. a change produces ModuleChanges with existence, type, metadata, class, wildcard, and related flags;
5. each ModuleDep decides whether the change invalidates it;
6. invalid modules re-enter the epoch loop;
7. missing modules and evicted answers have explicit lookup outcomes;
8. LSP queries reuse module products and answer traces.

The key behavior is demand-sensitive invalidation. An importer that only needs that a name exists should not necessarily invalidate when the name's inferred type changes. An importer that uses a wildcard or export origin is more sensitive.

## 5. Concrete data structures

### Pyrefly dependency keys

Pyrefly's ModuleDep variants include:

- Exists;
- Key;
- NameExists;
- NameMetadata;
- IsSpecialExport;
- ReexportSource;
- IsImplicitReexport;
- GetDeprecated;
- ExportOrigin;
- DocstringRange;
- IsSubmoduleImportedImplicitly;
- Wildcard;
- EveryExportUntracked;
- Class.

NameDep carries separate metadata and type flags. ModuleDeps stores name demands, wildcard demand, classes, type aliases, and framework-specific relations.

### Pyrefly module products

Module state combines:

- module identity and path;
- require level;
- epoch and dirty flags;
- computing state and waiters;
- staged products;
- negative or evicted outcomes.

The distinction between ModuleNotFound, Evicted, and Available matters to callers.

### Phalcom resolver and interfaces

ModuleResolver has:

- ProjectUniverse;
- SourceProvider;
- generation-scoped interface_cache;
- builtin provider support.

resolve_import handles universe and std roots, project roots, relative components, canonical path checks, source-provider lookup, and exposure errors. load_interface parses and builds an UnlinkedModuleInterface, then caches the result for the resolver generation.

UnlinkedModuleInterface contains:

- ModuleId and module kind;
- declaration surface;
- export surface;
- import surface;
- exposed children;
- metadata.

Linked interfaces resolve export targets to bindings or modules.

### Phalcom graph products

The module graph stores forward edges, reverse importers, and unresolved candidates. Edges preserve binding, import path, edge kind, phase, and source range.

The graph crate separately models:

- ReferenceGraph;
- SemanticGraph with deterministic SCC components;
- RuntimeDependencyGraph with acyclicity validation and initialization order.

This distinction should remain visible to semantic and runtime consumers.

## 6. State machines and transitions

### Module availability

~~~text
Unknown
    -> Resolving
    -> PresentInterface
    -> PresentLinked
    -> PresentAnalyzed

Resolving
    -> Missing
    -> ParseError
    -> InterfaceError
    -> Stale
~~~

Missing and error states are products with stable identity, not absence of a node.

### Import edge

~~~text
source syntax
    -> canonical unresolved edge
    -> resolved ModuleId or unresolved candidate set
    -> interface-linked binding/module target
    -> semantic or runtime dependency
~~~

### Interface publication

~~~text
source revision
    -> unlinked interface
    -> linked interface
    -> semantic graph update
    -> reverse invalidation update
~~~

An interface must never be published with a source revision different from its declarations and exports.

## 7. Cache keys and validity

Minimum module key:

~~~text
ModuleKey {
    project_id,
    logical_path,
    module_kind,
    universe_revision,
}
~~~

Minimum interface key:

~~~text
InterfaceKey {
    module_key,
    source_revision,
    provider_generation,
}
~~~

Minimum import-resolution key:

~~~text
ImportKey {
    importer_module,
    written_path,
    import_form,
    project_revision,
}
~~~

The key must include the importer for relative imports and package exposure. A path-only cache is incorrect when two projects or roots expose different logical modules.

Negative cache entries must be generation-scoped. Existing regression coverage in phalcom-modules tests verifies that a new provider generation does not retain an old missing-path result.

## 8. Ownership and concurrency

### Resolver ownership

The first implementation should keep resolver and interface-cache mutation behind the project model or analysis worker. Read-side semantic queries consume immutable interface products.

SourceProvider may perform I/O, but semantic publication must not retain an open file handle or borrow mutable provider state.

### Missing and partial environments

A dependent module may be analyzed when an import is:

- missing;
- found but not parsed;
- parsed but interface-invalid;
- found with stale source;
- found with a partial export surface.

The dependent environment should contain an explicit ImportBindingState:

~~~text
Resolved(module_id, target)
Missing(path)
UnresolvedCandidate(path_set)
InterfaceError(module_id, error_id)
Partial(module_id, available_exports)
~~~

The state can answer local names as Unknown or Unresolved while still preserving import provenance.

### Parallel extension

Interface extraction may run in parallel for independent modules if:

1. source reads are immutable;
2. every result carries ModuleKey and source revision;
3. graph mutation happens in a deterministic join;
4. negative results are generation-scoped;
5. runtime cycle validation runs after all edges are present.

## 9. Memory and allocation

Retain:

- compact ModuleId and logical path;
- declaration/export surfaces;
- import edges and reverse indexes;
- structured resolution errors;
- source revision and provider generation.

Evict:

- parsed body AST when interface and needed semantic products are retained;
- full source text if open-document or explanation policy permits;
- failed transient parse details after diagnostics are materialized.

Do not evict the canonical key or unresolved edge. Recomputing a missing module should update its state, not create a different node.

Cache large export maps by structural sharing. Keep aliases and re-export targets as compact references rather than copied declarations.

## 10. Complexity and performance

Expected costs:

- canonical import lookup: O(path components);
- interface cache lookup: O(1) average hash lookup;
- forward/reverse graph update: O(import edges changed);
- deterministic SCC computation: O(nodes plus edges);
- runtime initialization order: O(nodes plus runtime edges);
- wildcard or every-export dependency propagation: potentially broad and must be measured.

Measure:

- interface cache hit/miss/negative-hit;
- source-provider locate and read time;
- unresolved candidate count;
- imports by phase and edge kind;
- re-export chain length;
- invalidated importers per interface change;
- semantic versus runtime SCC sizes;
- memory retained by surfaces, paths, and edges.

## 11. Failure, cancellation, recursion, and cycles

Rules:

- import resolution cancellation leaves no partially linked interface;
- missing modules produce structured availability state;
- interface cycles may form semantic SCCs if the language permits them;
- runtime initialization cycles remain errors where ModuleRuntimeGraph requires acyclicity;
- re-export cycles must report a closed deterministic path;
- import outside package exposure is a resolution error, not a missing file;
- legacy or reserved roots remain explicit policy errors.

Phalcom already tests runtime self and multi-node cycles, semantic SCC retention, manifest cycle diagnostics, exposure, missing paths, and legacy core import rejection. Preserve those distinctions in any new incremental layer.

## 12. Phalcom mapping

| Pyrefly mechanism | Phalcom mapping |
| --- | --- |
| ModuleDeps and ModuleDep | ImportEdge plus named dependency facts |
| module require/epoch | project/source/interface revisions |
| staged Exports | Unlinked and LinkedModuleInterface |
| module existence outcome | ModuleAvailability |
| wildcard dependency | explicit wildcard/re-export edge kind |
| reverse dependency invalidation | ModuleGraph reverse importers and semantic graph |
| missing/evicted lookup states | availability state and product retention state |
| LSP module lookup | resolver-backed stamped snapshot query |
| deterministic module order | BTreeMap/SCC order and canonical ModuleId |

## 13. Mechanisms not copied

Do not copy:

- Python-specific import fallback or stub precedence without a Phalcom rule;
- an empty synthetic module for every missing import;
- Pyrefly's framework-specific dependency flags;
- one graph for interface, semantic, and runtime dependencies;
- path strings as semantic identity;
- a cache that ignores project or provider generation;
- wildcard invalidation hidden inside a generic “module changed” flag;
- re-export behavior inferred from runtime execution alone.

## 14. Proposed Phalcom data structures

~~~text
ModuleAvailability {
    Present,
    Missing { requested: ImportPath },
    ParseError { source: SourceId, diagnostic: DiagnosticId },
    InterfaceError { module: ModuleId, diagnostic: DiagnosticId },
    Partial { available: ExportSurfaceId },
    Stale { previous: InterfaceId },
}

ModuleEnvironment {
    module: ModuleId,
    bindings: BindingTable,
    imports: Vec<ImportBindingState>,
    interface_inputs: Vec<InterfaceDependency>,
    runtime_inputs: Vec<RuntimeDependency>,
}

InterfaceDependency {
    importer: ModuleId,
    target: ModuleKey,
    demand: InterfaceDemand,
    source_range: SourceRange,
}

InterfaceDemand {
    ModuleExists,
    NamedExport(SymbolId),
    ExportMetadata(SymbolId),
    WildcardExports,
    ReExportOrigin(SymbolId),
    RuntimeInitialization,
}
~~~

The environment should preserve all written aliases and the original import path so diagnostics and navigation can explain resolution.

## 15. Proposed APIs and module seams

Candidate APIs:

- ModuleResolver::resolve(importer, path, revision) -> Resolution
- InterfaceStore::load(key) -> InterfaceOutcome
- Environment::bind_import(edge, outcome) -> ImportBindingState
- ModuleGraph::record(edge)
- ModuleGraph::reverse_dependents(module, demand)
- SemanticGraph::sccs(affected)
- RuntimeDependencyGraph::initialization_order()
- ModuleAvailability::semantic_view()

Keep resolution, linking, semantic graph construction, and runtime validation as separate seams. A resolver should not run type inference; a type query should not reopen filesystem resolution.

## 16. Implementation order

1. Treat current ModuleId, source canonicalization, and resolver generation as the identity authority.
2. Add explicit ModuleAvailability to interface and semantic products.
3. Record import demands by written form, resolved target, source range, and phase.
4. Link aliases, selective imports, re-exports, and wildcard products through one environment builder.
5. Connect interface changes to reverse semantic invalidation.
6. Preserve runtime graph validation as a separate gate.
7. Add stamped interface snapshots to LSP requests.
8. Add parallel interface extraction only after deterministic join tests pass.

## 17. Tests

Required tests:

- absolute, relative, alias, selective, wildcard, and re-export imports;
- missing module with dependent semantic analysis;
- parse failure with available prior interface;
- partial export surface and unknown imported name;
- source and interface generation invalidates negative cache correctly;
- two projects with same path text have distinct ModuleKeys;
- exposed and unexposed project modules;
- builtin and std roots;
- semantic interface SCC accepted where allowed;
- runtime self and multi-node cycles rejected;
- deterministic re-export cycle diagnostics;
- stale interface cannot answer a newer source revision.

Current anchors:

- phalcom-modules/tests/integration.rs;
- linker.rs;
- interface_extraction.rs;
- graph.rs;
- repair_regressions.rs;
- universe_project_model.rs;
- package_semantic_contract.rs.

## 18. Benchmarks and metrics

Benchmark:

1. cold workspace resolution;
2. warm interface cache;
3. repeated missing imports;
4. deep re-export chain;
5. wildcard-heavy package;
6. large independent module set;
7. one interface edit with many semantic dependents;
8. runtime graph validation over a large project.

Acceptance metrics:

- cache hit and negative-hit rate;
- median and p95 import resolution;
- interface extraction throughput;
- number of invalidated dependents per change;
- graph update time;
- unresolved candidate memory;
- diagnostic count by availability state.

## 19. Risks and open questions

- Does Phalcom need source/stub/library precedence, or is ProjectUniverse already sufficient?
- Which builtins are module interfaces versus native descriptors?
- Should partial environments be publishable to LSP before all dependencies resolve?
- How are reflection exports represented without pretending they are statically closed?
- Which interface metadata changes are semantically relevant?
- Can a re-export target retain identity across module reload?
- What demand granularity is worth storing before reverse indexes dominate memory?

These remain **OPEN / UNVERIFIED** where current sources do not specify the final incremental policy.

## 20. Final transfer checklist

- [x] Pyrefly demand-sensitive module dependencies identified.
- [x] Phalcom resolver, interfaces, canonical keys, and graph layers mapped.
- [x] Missing and partial environments modeled explicitly.
- [x] Semantic and runtime cycle policies kept separate.
- [x] Negative cache generation scope recorded.
- [x] Re-exports, aliases, wildcard, exposure, and builtin boundaries included.
- [x] Parallel interface work constrained by deterministic join and stamped results.
- [ ] ModuleAvailability integrated into all semantic products.
- [ ] Import-demand reverse invalidation connected to the semantic engine.
- [ ] LSP stale interface behavior tested end to end.

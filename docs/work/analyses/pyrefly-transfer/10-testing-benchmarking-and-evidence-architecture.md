# Pyrefly testing, benchmarking, and evidence architecture

## 1. Scope and purpose

This dossier turns the transfer into measurable engineering work. It defines test strata, fixture shape, incremental scenarios, concurrency checks, determinism laws, benchmarks, and acceptance evidence.

The goal is not to copy Pyrefly's test count or test syntax. The goal is to make every semantic claim falsifiable at the correct layer: representation, query cell, solver, module graph, worker lifecycle, LSP publication, and user-visible diagnostics.

## 2. Evidence boundary and pinned source

**OBSERVED / PYREFLY** observations use /tmp/pyrefly-analysis-20260822 at commit 43467e64e36550f232a18e89f24fda79b1020b6.

| Evidence surface | Pinned source or directory |
| --- | --- |
| calculation-cell unit tests | crates/pyrefly_graph/src/calculation.rs |
| answer-slot and solver unit tests | pyrefly/lib/alt/answers.rs, answers_solver.rs |
| module state, eviction, and dependency tests | pyrefly/lib/state/module.rs, state.rs, import_tracker.rs |
| diagnostics and expectation tests | pyrefly/lib/state/errors.rs, error/collector.rs, error/expectation.rs |
| command-level and fixture tests | test/*.md, test/errors.md, test/baseline.md, test/suppress.md, test/sarif |
| LSP tests | lsp/src/test |
| Phalcom LSP integration | phalcom-lsp/tests/integration.rs and staged test modules |
| Phalcom module graph and resolver | phalcom-modules/tests |
| Phalcom semantic checker | phalcom-semantic/tests |
| Phalcom performance counters | phalcom-lsp/src/perf.rs, phalcom-lsp/tests/performance.rs |

Source mirror: [Pyrefly commit 43467e64e36550f232a18e89f24fda79b1020b6](https://github.com/facebook/pyrefly/tree/43467e64e36550f232a18e89f24fda79b1020b6).

## 3. Executive conclusion

Pyrefly evidence is layered:

- small Rust unit tests prove cell and solver invariants;
- module tests prove staged products, dependency invalidation, and eviction;
- markdown and fixture tests prove command-visible diagnostics;
- LSP tests prove protocol behavior;
- benchmarks and memory reports prove performance claims.

Phalcom should adopt the same separation. A focused checker test cannot prove snapshot publication. An LSP test cannot prove type equality. A benchmark cannot prove semantic correctness.

**PROPOSED / PHALCOM:** define a release gate with passing, baseline/unrelated, deferred, and unverified scope. Each new semantic product must have a property test or law, an incremental fixture, a stale/cancellation test where relevant, and a benchmark counter if it is intended to improve performance.

## 4. Pyrefly execution path

The evidence pipeline is:

1. unit test a local data structure;
2. test a solver state transition with controlled answers;
3. test module products through staged requirements;
4. test dependency changes and epoch rechecks;
5. test diagnostic collection and deterministic display;
6. test command fixtures and expected errors;
7. test LSP requests against a changing workspace;
8. benchmark cold, warm, incremental, and memory behavior;
9. compare results across repeated and concurrent runs.

This order matters. Integration tests should expose a broken contract, not be the only place where the contract is defined.

## 5. Concrete data structures

### Pyrefly test evidence

The pinned source includes direct tests for calculation cells and answer slots, many unit tests in the answer solver, state and error modules, import-tracker tests, markdown command fixtures, SARIF fixtures, and LSP tests. The solver tests include recursive and deterministic behavior close to implementation seams.

### Phalcom current evidence

Relevant current surfaces include:

- phalcom-semantic/tests/checker.rs;
- phalcom-semantic/tests/phase2_expression_engine.rs;
- phalcom-modules/tests/graph.rs;
- phalcom-modules/tests/linker.rs;
- phalcom-modules/tests/interface_extraction.rs;
- phalcom-modules/tests/integration.rs;
- phalcom-modules/tests/repair_regressions.rs;
- phalcom-lsp/tests/analysis_status.rs;
- phalcom-lsp/tests/performance.rs;
- phalcom-lsp/tests/semantic_consistency.rs;
- phalcom-lsp/tests/workspace_semantics.rs;
- phalcom-lsp/tests/stage1_diagnostics.rs.

The LSP integration crate uses a single integration target because its Cargo configuration disables automatic integration-test discovery. Filter the integration target rather than treating each included Rust module as an independent Cargo test target.

### Proposed evidence manifest

~~~text
EvidenceCase {
    id,
    layer,
    fixture,
    input_revision,
    expected_products,
    expected_diagnostics,
    expected_events,
    metrics,
    status,
}
~~~

## 6. State machines and transitions

Every incremental test should name its transition:

~~~text
initial source
    -> first build
    -> edit
    -> affected set
    -> recomputation
    -> publication or stale discard
    -> observable query
~~~

Lifecycle cases:

- no-op edit: no semantic product changes;
- body-only edit: callable frontier changes, interface remains stable;
- declaration edit: interface and dependents change;
- import edit: graph and environment change;
- deletion: module becomes unavailable and reverse dependents update;
- cancellation: in-flight result discarded;
- out-of-order completion: deterministic join preserves output;
- repeated same edit: second run converges to same generation products.

## 7. Cache keys and validity

Tests must assert keys, not only values:

| Product | Evidence requirement |
| --- | --- |
| type identity | structurally equal types have equal semantic identity where specified |
| query answer | same key returns valid answer for same generation |
| interface | source/provider generation changes invalidate old interface |
| diagnostic | source revision and generation prevent stale display |
| module graph | canonical ID avoids duplicate logical nodes |
| LSP result | document revision mismatch yields no semantic result |

Add negative tests for false cache hits:

- same path under two projects;
- same source text with different module identity;
- old interface after provider refresh;
- old diagnostic after edit;
- old callable summary after declaration change;
- answer after type-store reset.

## 8. Ownership and concurrency

### Determinism

Run the same fixture:

1. serially;
2. with different worker scheduling;
3. with delayed completion of independent jobs;
4. after cancellation and replay.

Compare canonical semantic products, diagnostic IDs/order, graph SCCs, and publication events. Timing and counter values may differ; semantic outputs must not.

### Race and lifecycle tests

Use a test-only scheduler or barriers to force:

- reader during candidate computation;
- shutdown during scan;
- update while old batch is computing;
- publication after epoch changes;
- two jobs targeting one module;
- one query waiting on another query;
- recursive query within one worker.

The first Phalcom implementation can remain mostly single-threaded while still testing these interleavings at the worker/publication boundary.

## 9. Memory and allocation

Tests should detect:

- retained old generations after all readers release;
- diagnostic vectors growing after repeated no-op edits;
- trace buffers retained when trace demand is disabled;
- source cache entries surviving provider generation incorrectly;
- eviction followed by a different semantic result;
- duplicate interned type entries for canonical-equal types.

Use counters and snapshots rather than unstable allocator-specific assertions for ordinary CI. Add a memory stress job for high-water marks and retained generation counts.

## 10. Complexity and performance

### Benchmark matrix

| Dimension | Cases |
| --- | --- |
| workspace size | 1, 10, 100, 1000 modules |
| dependency shape | chain, diamond, fan-in, fan-out, SCC |
| edit shape | body, declaration, import, deletion, no-op |
| demand | one query, open-file queries, full workspace |
| state | cold, warm, repeated incremental |
| execution | serial, bounded parallel |
| diagnostics | none, local, cross-file, many duplicates |

Record:

- wall time;
- p50/p95/p99 latency;
- modules visited;
- callables analyzed;
- solver rounds;
- query hits/misses;
- stale work;
- bytes allocated and retained;
- snapshot sharing;
- diagnostic and trace counts.

Do not call a benchmark a performance regression test until input, warmup, output validation, and noise policy are documented.

## 11. Failure, cancellation, recursion, and cycles

Failure fixtures must prove:

- recursive semantic analysis terminates;
- type inference gas or iteration limits produce stable fallback;
- interface cycles and runtime cycles obey different policies;
- a missing module does not panic dependent analysis;
- parse recovery does not publish malformed interface facts as valid;
- cancellation does not leave pending reservations or processing stuck;
- an internal error preserves last known good snapshot;
- stale result events are counted and not published.

Property-based cases should generate small graphs with:

- self edges;
- duplicate edges;
- diamonds;
- cycles;
- missing nodes;
- re-exports;
- independent components.

Then compare incremental recomputation with clean recomputation.

## 12. Phalcom mapping

| Pyrefly evidence pattern | Phalcom test layer |
| --- | --- |
| calculation cell tests | future query-cell unit tests |
| solver/SCC tests | phalcom-semantic solver tests |
| ModuleDeps and epoch tests | module graph and invalidation tests |
| error collector tests | diagnostic normalization tests |
| markdown command fixtures | Phalcom CLI/spec fixtures |
| LSP tests | phalcom-lsp integration and manual protocol checks |
| speed/memory comparisons | performance.rs and external benchmark harness |
| deterministic BTreeMap/SCC behavior | repeated-run property tests |

## 13. Mechanisms not copied

Do not copy:

- Pyrefly fixture syntax as a Phalcom language contract;
- exact expected message text when only diagnostic code is stable;
- benchmark numbers across different languages and workloads;
- a passing unit-test count as an architecture acceptance gate;
- concurrency tests that cannot force the intended interleaving;
- snapshots that omit source revision and module identity;
- property tests that compare only rendered strings.

## 14. Proposed Phalcom data structures

~~~text
FixtureWorkspace {
    files: BTreeMap<Path, SourceText>,
    project_model,
    open_documents,
}

EditScript {
    revisions: Vec<Edit>,
    expected_affected: Set<SemanticIdentity>,
}

AcceptanceRecord {
    case_id,
    clean_result,
    incremental_result,
    diagnostics,
    events,
    metrics,
}

BenchmarkProfile {
    name,
    workspace_shape,
    edit_script,
    demand_policy,
    warmup,
    repetitions,
}
~~~

Store expected semantic identities and diagnostic codes separately from rendered text.

## 15. Proposed APIs and module seams

Candidate test support:

- FixtureWorkspace::open();
- FixtureWorkspace::apply(edit);
- AnalysisHarness::build();
- AnalysisHarness::recheck();
- AnalysisHarness::snapshot();
- AnalysisHarness::events();
- AnalysisHarness::metrics();
- compare_clean_and_incremental();
- run_under_scheduler();
- benchmark_profile();

Keep harness code outside production semantic ownership. A test harness may inject a fake clock, source provider, scheduler, or cancellation barrier, but it must use the same publication APIs as production.

## 16. Implementation order

1. Create an evidence matrix for current behavior and known gaps.
2. Add invariant tests for identity, equality, graph SCCs, and diagnostics.
3. Add clean-versus-incremental fixture comparison.
4. Add worker stale/cancellation integration tests.
5. Add deterministic scheduling tests.
6. Add benchmark fixtures and counter output.
7. Add property-based graph and edit-script generation.
8. Add memory and long-session stress tests.
9. Add manual LSP acceptance for rebuilt server path and restart behavior.

## 17. Tests

Minimum test tiers:

### Unit

Type equality, canonical unions, query keys, dependency facts, diagnostic IDs, graph edges, and state transitions.

### Component

Solver fixed points, module interfaces, import environment, source revision invalidation, snapshot publication, and retention.

### Integration

Workspace edits, LSP requests, diagnostics, inlay hints, completion, hover, semantic tokens, and analysis status.

### Property and fuzz

Generated dependency graphs, edit scripts, expression fragments, cycle shapes, malformed source, and parser/semantic recovery boundaries.

### Manual

Rebuild server, configure phalcom.lsp.serverPath, restart language server, inspect output/status panel, open a multi-file fixture, edit provider and consumer, verify stale results do not remain.

## 18. Benchmarks and metrics

Acceptance thresholds must be established from a current baseline. Until then, report:

- cold and warm wall time;
- incremental wall time by edit class;
- p95 request latency;
- modules and callables visited;
- solver rounds and query hits;
- stale/cancelled work;
- allocations and retained bytes if available;
- diagnostic publication latency;
- memory high-water mark.

For every optimization, require one semantic equivalence test and one counter showing the intended mechanism changed.

## 19. Risks and open questions

- Which fixtures are stable enough for a long-lived acceptance suite?
- Should expected outputs use serialized semantic facts, diagnostic codes, or both?
- Which benchmark workloads represent real Phalcom projects?
- Can deterministic scheduling tests run reliably on CI hosts?
- Is a property-test crate already approved for the workspace?
- Which fuzz target should own parser, VM, module, or semantic generated input?
- How should baseline/unrelated failures be recorded without hiding regressions?

These are **OPEN / UNVERIFIED** until the project selects a harness and baseline policy.

## 20. Final transfer checklist

- [x] Pyrefly unit, fixture, LSP, and benchmark evidence surfaces identified.
- [x] Phalcom current test crates and integration-target constraint recorded.
- [x] Clean-versus-incremental comparison made a first-class gate.
- [x] Determinism, cycle, stale, cancellation, and memory cases specified.
- [x] Benchmark matrix and required counters specified.
- [x] Manual LSP validation included.
- [x] Passing, baseline, deferred, and unverified scope separated.
- [ ] Evidence manifest implemented.
- [ ] Current performance baseline captured.
- [ ] Property/stress/fuzz suite connected to acceptance commands.

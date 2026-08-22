# Pyrefly diagnostics, traces, and observability

## 1. Scope and purpose

This dossier separates three products that are often accidentally merged:

1. semantic answers used by later analysis;
2. user diagnostics derived from answers and source context;
3. traces and observability data used by LSP, tooling, debugging, and performance work.

The central rule is simple: diagnostics and traces may explain a semantic generation, but they must not silently decide whether that generation is cache-valid. They need their own identity, retention, stale-result, and rendering rules.

## 2. Evidence boundary and pinned source

**OBSERVED / PYREFLY** observations use /tmp/pyrefly-analysis-20260822 at commit 43467e64e36550f232a18e89f24fda79b1020b6.

| Mechanism | Pinned source |
| --- | --- |
| ErrorCollector, delayed/never styles, annotations, quick fixes | pyrefly/lib/error/collector.rs |
| Per-module error collection, filtering, sorting, baselines, LSP partitioning | pyrefly/lib/state/errors.rs |
| Error rendering and LSP conversion | pyrefly/lib/error/error.rs |
| Thread-local trace sinks and SCC trace commit | pyrefly/lib/alt/answers_solver.rs |
| Trace enablement from requirements | pyrefly/lib/state/steps.rs |
| Type-at-position and trace-backed LSP queries | pyrefly/lib/state/lsp.rs |
| Phalcom semantic diagnostic facts | phalcom-semantic/src/diagnostic.rs, checker/result.rs, checker/typed_expr.rs |
| Phalcom LSP conversion and publication | phalcom-lsp/src/diagnostics.rs, backend.rs |
| Phalcom generation/stale events and metrics | phalcom-lsp/src/analysis_service.rs, perf.rs |

Source mirror: [Pyrefly commit 43467e64e36550f232a18e89f24fda79b1020b6](https://github.com/facebook/pyrefly/tree/43467e64e36550f232a18e89f24fda79b1020b6).

## 3. Executive conclusion

Pyrefly does not treat an error message as a side effect of printing. Errors are collected in a module-owned collector, sorted and deduplicated, filtered through suppression and severity policy, optionally compared with baselines, and only then rendered for CLI or LSP. ErrorStyle::Never disables collection; ErrorStyle::Delayed permits collection without immediate output.

Traces are a separate product. The answer solver installs thread-local trace sinks, records side effects such as mapped types and invoked properties, discards cold-iteration traces, and merges traces only when the corresponding answer commits. This prevents speculative or placeholder state from leaking into user-visible queries.

Phalcom currently has structured SemanticDiagnostic values and an LSP converter, but the backend publication path currently publishes syntax diagnostics. Semantic diagnostic publication remains **CURRENT / PARTIAL** and must be wired through generation-stamped publication before it is treated as complete.

## 4. Pyrefly execution path

The diagnostic path is:

1. module loading creates an ErrorCollector with a selected style;
2. checking emits structured errors through an ErrorBuilder;
3. errors remain associated with module identity and source ranges;
4. module cleanup sorts by range and removes duplicate errors in the same range;
5. collection partitions ordinary, directive, suppressed, disabled, and baseline errors;
6. configuration applies severity, ignore, and baseline policy;
7. multi-module Errors sorts modules by name and path;
8. CLI and LSP renderers consume deterministic display lists.

The trace path is independent:

1. requirements decide whether answer traces are enabled;
2. solver work installs a thread-local sink;
3. semantic operations append trace side effects;
4. cold SCC iteration discards traces;
5. final answer publication merges trace side effects;
6. LSP queries read trace-backed products for positions, calls, and members.

The observability path is different again:

1. worker and solver counters record events;
2. spans optionally record elapsed time and generation/epoch context;
3. status and stale events are streamed to clients;
4. counters are snapshots, not semantic facts.

## 5. Concrete data structures

### Pyrefly diagnostics

ErrorCollector contains module information, an ErrorStyle, and mutex-protected ModuleErrors. An error can carry:

- primary range;
- error kind;
- header and detail text;
- context;
- secondary annotations;
- quick fixes;
- severity after configuration.

CollectedErrors partitions ordinary, directives, suppressed, disabled, and baseline entries. This is important: “not displayed” does not mean “never computed,” and “baseline” does not mean “successful semantic answer.”

### Pyrefly traces

TraceSideEffects records maps from ranges or keys to semantic information, overloaded callees, invoked properties, and expected types. The solver's thread-local trace sink prevents nested module computations from mixing traces. Traces are merged only with a committed answer.

### Phalcom diagnostics

SemanticDiagnostic currently contains:

- DiagnosticCode;
- severity;
- message;
- primary range;
- labeled secondary ranges.

TypeCheckReport returns a Vec<SemanticDiagnostic>. TypedExpression carries TypeKnowledge, constraints, and EvidenceSet; this is useful provenance, but it is not yet a generation-level trace store.

semantic_diagnostic_to_lsp_diagnostic maps severity, code, message, and labels to LSP values. The current related-information conversion uses a placeholder file URL for related locations and needs source-aware mapping before cross-file explanations are complete.

## 6. State machines and transitions

### Diagnostic lifecycle

~~~text
not computed
    -> collected in semantic pass
    -> normalized/deduplicated
    -> policy filtered
    -> attached to generation
    -> rendered for CLI/LSP
    -> retained or evicted with source revision
~~~

### Trace lifecycle

~~~text
disabled
    -> sink installed
    -> side effects recorded
    -> cold iteration discarded
    -> final answer committed
    -> traces merged into generation
~~~

### LSP result lifecycle

~~~text
request(document_revision, snapshot_generation)
    -> compute or read
    -> compare request stamp with current document
    -> publish only if still current
    -> otherwise discard as stale
~~~

No transition may render a result that belongs to an obsolete source revision.

## 7. Cache keys and validity

Diagnostic identity must not be only message text. Proposed identity:

~~~text
DiagnosticId {
    module_id,
    source_revision,
    code,
    primary_range,
    normalized_context_hash,
}
~~~

Trace identity must include its semantic owner:

~~~text
TraceKey {
    module_id,
    source_revision,
    semantic_generation,
    query_range_or_binding,
    trace_kind,
}
~~~

Observability event identity should include:

~~~text
EventStamp { generation, epoch, sequence }
~~~

A diagnostic cache hit means the same diagnostic product is valid for the same source and semantic dependency fingerprint. It does not mean that the message was previously printed. A trace cache hit means the requested semantic explanation is still attached to the same valid generation.

## 8. Ownership and concurrency

### Pyrefly

Errors are appended under a collector mutex, then cleaned up and rendered from controlled collection paths. Trace sinks are thread-local. SCC-local errors and traces remain private until commit. This avoids a speculative branch adding a diagnostic to a shared global list.

### Phalcom first implementation

Use worker-owned mutable diagnostic accumulation:

~~~text
semantic worker -> DiagnosticBuffer and TraceBuffer
publication     -> immutable DiagnosticSnapshot and TraceSnapshot
LSP request     -> read-only generation-stamped products
renderer        -> LSP/CLI values only
~~~

The backend must not mutate a published diagnostic vector in place. A later publication may replace the vector or structurally share per-file entries.

### Streaming

Status events may stream while analysis is running because they describe lifecycle. Semantic diagnostics should stream only as a complete per-generation replacement for each affected file, or use explicit partial-state markers. A consumer must never interpret an intermediate append as a complete diagnostic set.

## 9. Memory and allocation

Diagnostic memory is dominated by:

- repeated message strings;
- source ranges and labels;
- cross-file related information;
- quick-fix edits;
- retained generations during slow LSP clients.

Trace memory is dominated by range-keyed maps and repeated type/provenance values. Cold-iteration trace suppression is a direct allocation optimization.

Phalcom should:

- store codes and stable IDs compactly;
- share source and module identity through Arc;
- intern repeated labels only after measurement;
- retain diagnostics per file and generation, not in one unbounded global vector;
- make trace retention demand-driven;
- cap explanation depth and related-location count;
- count retained generations and diagnostic bytes.

Do not keep full solver provenance in every published diagnostic unless a query or explanation feature requires it.

## 10. Complexity and performance

Target costs:

- local diagnostic append: amortized O(1);
- per-range cleanup: O(n log n) sorting plus duplicate checks;
- module display ordering: O(m log m);
- diagnostic replacement for affected files: O(changed files plus diagnostics);
- trace lookup: O(1) or O(log n) by stable range/binding key;
- rendering: proportional to diagnostics and labels emitted.

Measure:

- diagnostics emitted before/after deduplication;
- diagnostics suppressed by style, config, baseline, and stale publication;
- trace events recorded/discarded/committed;
- bytes in messages, labels, fixes, and traces;
- rendering latency;
- stale LSP results discarded;
- diagnostics per generation and per file;
- repeated diagnostic identity across generations.

## 11. Failure, cancellation, recursion, and cycles

Rules:

- cancellation discards uncommitted diagnostics and traces;
- stale batches never replace current diagnostics;
- placeholder and cold SCC answers do not emit final user diagnostics;
- internal worker failures preserve previous diagnostics and emit a status/error event;
- a missing source or module can produce a structured resolution diagnostic without making dependent semantic facts impossible to compute;
- a trace lookup with no valid owner returns unavailable, not a fabricated explanation.

Pyrefly explicitly suppresses errors and discards traces during cold SCC iteration. Phalcom must mirror the policy at the solver boundary, not try to remove speculative messages later by string matching.

## 12. Phalcom mapping

| Pyrefly mechanism | Phalcom mapping |
| --- | --- |
| ErrorCollector | worker-local DiagnosticBuffer |
| delayed/never styles | analysis policy and demand flags |
| ModuleErrors cleanup | deterministic per-file normalization |
| collected error partitions | active, suppressed, deferred, baseline-like policy products |
| Error annotations and quick fixes | labels, related locations, future fixes |
| thread-local TraceSideEffects | solver-local TraceBuffer |
| cold-iteration discard | SCC iteration policy |
| LSP error partition | generation-stamped per-file diagnostic publication |
| perf spans/counters | existing phalcom-lsp perf module |

## 13. Mechanisms not copied

Do not copy:

- Python-specific error suppression syntax;
- baseline file semantics without a Phalcom specification;
- message-text identity;
- global trace sinks shared by concurrent modules;
- publishing every intermediate solver error;
- treating a warning or diagnostic as proof that an answer is invalid;
- the placeholder URL used by the current Phalcom LSP converter;
- CLI formatting structures as semantic data structures.

## 14. Proposed Phalcom data structures

~~~text
DiagnosticFact {
    id: DiagnosticId,
    severity,
    code,
    primary: SourceSpan,
    labels: Vec<DiagnosticLabel>,
    explanation: Option<ExplanationId>,
    fixes: Vec<Fix>,
}

DiagnosticFileProduct {
    source_revision,
    semantic_generation,
    facts: Arc<[DiagnosticFact]>,
}

TraceFact {
    key: TraceKey,
    kind: TraceKind,
    target: SemanticIdentity,
    evidence: EvidenceSet,
}

DiagnosticSnapshot {
    generation,
    files: Arc<BTreeMap<FileId, DiagnosticFileProduct>>,
}
~~~

Diagnostic facts should be immutable once attached to a generation. Explanation payloads may be lazy and separately retained.

## 15. Proposed APIs and module seams

Candidate APIs:

- DiagnosticBuffer::emit(fact)
- DiagnosticBuffer::normalize()
- TraceBuffer::record(key, fact)
- TraceBuffer::commit_for(answer_id)
- DiagnosticStore::publish(generation, affected_files, products)
- DiagnosticStore::for_file(file_id, stamp)
- LspDiagnosticRenderer::render(file_product, source_map)
- ExplanationStore::get(explanation_id, stamp)

Backend integration:

1. publish_engine receives semantic diagnostics and traces from the same candidate generation as semantic facts.
2. publication_effects identifies files whose diagnostic product changed.
3. backend publishes a full diagnostic replacement for each affected open document.
4. request handlers reject file products whose document revision is newer than the product stamp.

The semantic checker remains responsible for facts. The LSP renderer remains responsible for protocol conversion.

## 16. Implementation order

1. Add source-aware span conversion for semantic diagnostics.
2. Add generation and source revision to semantic diagnostic products.
3. Move worker diagnostics into immutable per-file publication.
4. Add deterministic normalization and stable diagnostic IDs.
5. Add stale-result rejection and full replacement publication in the backend.
6. Separate trace facts from typed-expression provenance.
7. Add lazy explanations and related-location rendering.
8. Add suppression/deferred policy only after the core publication path is tested.

## 17. Tests

Required tests:

- duplicate facts at one range normalize deterministically;
- same message at different ranges remains distinct;
- same fact across an unchanged generation retains identity;
- source edit removes old diagnostics;
- stale batch cannot republish old diagnostics;
- cancellation emits no partial diagnostic replacement;
- related labels preserve their source URI and range;
- cold SCC iteration emits no placeholder diagnostic;
- committed traces are visible, discarded traces are not;
- diagnostic order is stable regardless of map insertion or job order;
- LSP backend clears diagnostics when a valid generation contains none.

Current anchors:

- phalcom-semantic/tests/checker.rs;
- phalcom-semantic/tests/phase2_expression_engine.rs;
- phalcom-lsp/tests/stage1_diagnostics.rs;
- phalcom-lsp/tests/semantic_consistency.rs;
- stale revision tests in phalcom-lsp/src/inlay_hints.rs.

## 18. Benchmarks and metrics

Benchmark:

1. one file with many repeated errors;
2. many files with stable diagnostics;
3. rapid edits that invalidate before publication;
4. recursive callable SCC with speculative diagnostics;
5. large trace map and type-at-position queries;
6. full workspace scan with only one changed file.

Acceptance metrics:

- zero stale diagnostic publications;
- deterministic serialized diagnostics;
- bounded memory per retained generation;
- p95 diagnostic publication latency;
- trace allocation avoided when trace demand is off;
- no increase in semantic cache misses caused only by diagnostic retention policy.

## 19. Risks and open questions

- Should diagnostics be part of SemanticSnapshot or a sibling generation store?
- Which diagnostic codes are stable public API?
- How should unknown, dynamic, and unresolved facts affect severity?
- Should explanations be reproducible after source eviction?
- What is the Phalcom equivalent of a baseline, if any?
- Can diagnostic IDs survive harmless source movement, or should movement create new facts?
- Which trace facts are needed by hover, completion, inlay hints, and future debugging?

These are **OPEN / UNVERIFIED**. The current TypeCheckReport and converter do not establish a complete LSP semantic-diagnostic pipeline.

## 20. Final transfer checklist

- [x] Pyrefly error collection, normalization, filtering, and rendering boundaries identified.
- [x] Trace side effects separated from answer publication.
- [x] Cold SCC errors and traces identified as discardable.
- [x] Current Phalcom diagnostic structures and partial LSP integration recorded.
- [x] Diagnostic identity and generation stamps proposed.
- [x] Streaming, stale replacement, and source-aware labels specified.
- [x] Diagnostic cache validity kept separate from semantic cache validity.
- [ ] Semantic diagnostics published from Phalcom backend.
- [ ] Trace store implemented and demand-gated.
- [ ] LSP diagnostic and explanation stress tests pass.

# Testing, Performance, and Observability

## 1. Correctness layers

Semantic systems fail through subtle disagreement, so tests should be layered:

```text
parser/recovery fixtures
 -> surface/scope/identity tests
 -> HIR/lowering tests
 -> CFG/dataflow tests
 -> dispatch resolution tests
 -> interprocedural summary tests
 -> module/incremental tests
 -> consumer integration tests
 -> compiler/runtime conformance
```

A hover snapshot alone cannot tell whether a wrong result came from binding, flow, dispatch, or rendering.

## 2. Golden and structural tests

Use source fixtures with expected:

- semantic IDs/targets;
- scope/binding relationships;
- canonical selectors;
- normalized HIR or CFG structure where stable;
- diagnostics and provenance chains;
- summary dependency edges.

Avoid over-snapshotting Rust debug output. Snapshot stable semantic meaning so representation refactors do not rewrite every test.

## 3. Metamorphic properties

High-value properties include:

```text
format(format(source)) == format(source)
parse(format(parse(source))) preserves intended semantics
alpha-renaming one binding preserves behavior/relations
adding irrelevant whitespace/comments preserves logical semantic identities
inserting a correct explicit inferred type annotation preserves checker result
incremental(final_source) == full_analysis(final_source)
reordering independent declaration storage does not change results
```

Metamorphic tests expose missing dependencies and accidental source-position identity.

## 4. Negative and adversarial tests

Test malformed source, duplicates, missing imports, inheritance cycles, dynamic packs, reflection, recursive summaries, closure captures, and domain widening. Negative tests must assert the *kind* of uncertainty/error, not merely that some diagnostic exists.

## 5. Fuzzing and property testing

Useful fuzz targets:

- parser-produced AST -> semantic surface/scope (never panic);
- random edit sequences -> incremental/full equivalence;
- randomly generated scope trees -> resolver invariants;
- small CFGs -> worklist solver compared with slow reference fixed-point;
- selector syntax -> compiler/runtime/semantic canonicalization consistency;
- module graph mutations -> dependent/provider repair invariants.

For a finite toy domain, differential fixed-point testing is powerful: compare optimized worklist solver against naïve iterate-all-blocks-to-stability implementation.

## 6. Performance methodology

Do not optimize from intuition. Sequence:

```text
state observable semantic invariant
establish baseline correctness + benchmark
profile/counter hot path
change representation/algorithm
re-run correctness/incremental conformance
benchmark again
add regression guard if stable/relevant
```

Measure algorithmic work and wall time. A latency reduction caused by doing less necessary analysis is a correctness regression.

## 7. Semantic counters

Instrument:

```text
files/modules parsed
surfaces/scopes rebuilt
callables analyzed
worklist/SCC rounds
facts joined/widened
contribution slots recomputed
modules/callables in invalidation frontier
candidate-state/COW clones
snapshot publish count
cancelled analyses
query cache hit/miss
hover/completion/reference latency
snapshot memory / retained revisions
```

**CURRENT:** Phalcom semantic code already carries `PerfCounters` and test-visible rebuild traces. Extend observability at the semantic boundary rather than adding ad hoc timestamps in individual LSP handlers.

## 8. Performance budgets by workload

Separate:

- cold project load/indexing;
- single body edit;
- declaration/import edit;
- completion/hover read query;
- references/rename workspace query;
- full batch check;
- recursive/interprocedural pathological case.

A fast hover cannot compensate for a 500ms global rebuild on every keystroke.

## 9. Cache benchmarking

Before/after cache metrics should include memory and invalidation cost. A cache that improves repeated hover but doubles edit invalidation time or retains 100 generations may be a net loss.

Test cold, warm, and churn workloads.

## 10. Conformance regressions

Whenever a semantic bug is caused by runtime mismatch, add a cross-layer fixture. Examples: selector labels, `super`, non-local return, native contract, module cycle. Keep the semantic lock near the feature, not only in a historical bug test.

## 11. Review questions

1. Which semantic invariant does this test lock?
2. Is there a negative case distinguishing uncertainty reasons?
3. Does incremental behavior match full rebuild after edit sequences?
4. Are fixed-point algorithms tested on genuine cycles?
5. Are performance counters measuring semantic work?
6. Does the benchmark include memory/invalidation cost, not only query time?
7. Is a runtime-conformance test needed to prevent semantic drift?

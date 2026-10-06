# Phase 11 Extension API and Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Stabilize the provider, strategy, shrink-pass, database, and reporter extension contracts; add practical alternate implementations and conformance evidence; and remove measured structural inefficiencies without changing observable search semantics.

**Architecture:** Primitive choice supply remains separate from strategy semantics. Public system-random and scripted providers both normalize requests through one canonical path, while replay remains a cursor over immutable examples. Extension conformance is specified by focused Phalcom fixtures and source/static gates; the shared search and shrink kernel remains authoritative. Performance work targets allocation and duplicate-candidate hot spots only, preserving immutable examples, span identity, replay behavior, failure origin, reporting order, and database semantics.

**Tech Stack:** Typed Phalcom source, structural protocols, immutable semantic examples, existing search/shrink/database/reporting slices, Python source/static and mutation verifiers, benchmark fixtures, ZIP/SHA-256 checkpoint tooling.

## Global Constraints

- Implement only Phase 11; do not perform Phase 12 legacy removal or final release cleanup.
- Do not create a second generation, replay, shrink, reporting, or database engine.
- `ChoiceProvider`, `Strategy<out T>`, `ShrinkPass`, `ExampleDatabase`, and `Reporter` remain structural protocol boundaries.
- Alternate providers must return choices normalized to the active request and must not perform shrinking.
- Custom shrink passes may propose candidates, but only the authoritative shrinker decides acceptance under strict `ExampleComplexity` ordering and failure-origin preservation.
- Database adapters treat absence, stale data, corruption, and recoverable storage failures as cache misses rather than counterexamples.
- Reporter event ordering remains synchronous and deterministic; reporter failures are extension failures, never property counterexamples.
- Performance changes must preserve choice order, semantic spans, replay normalization, stable fingerprints, and output schemas.
- Runtime benchmark and `.ph` execution results are reported as unobserved when no `phalcom` executable is available.

---

### Task 1: Phase 11 acceptance, mutation, and benchmark contracts

**Files:**
- Create: `scripts/verify_phase11.py`
- Create: `scripts/verify_phase11_mutations.py`
- Create: `tests/conformance/provider.ph`
- Create: `tests/conformance/strategy.ph`
- Create: `tests/conformance/shrink_pass.ph`
- Create: `tests/conformance/database.ph`
- Create: `tests/conformance/reporter.ph`
- Create: `tests/integration/provider_equivalence.ph`
- Create: `tests/regression/duplicate_shrink_candidates.ph`
- Create: `tests/regression/span_stack_linear.ph`
- Create: `tests/regression/database_merge_on_write.ph`
- Create: `tests/regression/reporter_failure_boundary.ph`
- Create: `tests/regression/database_signature_roundtrip.ph`
- Create: `benchmarks/primitive_generation.ph`
- Create: `benchmarks/nested_lists.ph`
- Create: `benchmarks/replay.ph`
- Create: `benchmarks/integer_shrinking.ph`
- Create: `benchmarks/stateful_shrinking.ph`
- Create: `benchmarks/README.md`

**Interfaces:**
- Consumes: Phase 11 requirements and all Phase 01–10 behavior.
- Produces: a gate that fails on Phase 10, mutation checks proving the gate detects missing extension guarantees, and benchmark workloads for every required hot path.

- [x] Write all focused fixtures and both Python verifiers before production source.
- [x] Run `python3 scripts/verify_phase11.py` and record the expected red state.
- [x] Keep mutation cases deterministic and confined to temporary tree copies.

### Task 2: Public choice-provider implementations and canonical normalization

**Files:**
- Modify: `src/choices/provider.ph`
- Modify: `src/choices/data.ph`
- Modify: `src/engine/evaluator.ph`
- Modify: `src/engine/engine.ph`
- Modify: `src/core/settings.ph`

**Interfaces:**
- Consumes: `ChoiceRequest`, `Choice`, `DrawData`, seeded settings, and replay.
- Produces: public `SystemRandomChoiceProvider`, public `ScriptedChoiceProvider`, canonical `_ChoiceNormalization`, compatibility for the previous private random provider, and explicit provider-factory injection for generation.

- [x] Make system randomness and scripted choices conform to the same request-normalization rules as replay.
- [x] Ensure each generated example receives a fresh provider instance.
- [x] Preserve fixed-seed behavior as the default provider path.
- [x] Reject exhausted scripts and invalid scripted choices as engine overruns, not counterexamples.

### Task 3: Strategy and shrink-pass extension hardening

**Files:**
- Modify: `src/strategies/strategy.ph`
- Modify: `src/strategies/combinators.ph`
- Modify: all concrete strategy modules that extend the shared base
- Modify: `src/engine/shrink_pass.ph`
- Modify: `src/engine/shrinker.ph`

**Interfaces:**
- Consumes: existing structural strategies, semantic examples, and strict complexity ordering.
- Produces: public `StrategyBase<T>` for custom strategies, typed `List<ShrinkPass>` pipelines, duplicate-candidate suppression, and stable custom-pass semantics.

- [x] Keep the structural `Strategy<out T>` protocol authoritative.
- [x] Expose reusable combinator behavior without requiring inheritance for conformance.
- [x] Deduplicate candidate signatures before replay while preserving pass order.
- [x] Continue rejecting equal-or-greater-complexity and wrong-origin candidates centrally.

### Task 4: Database and reporter conformance hardening

**Files:**
- Modify: `src/database/database.ph`
- Modify: `src/database/memory.ph`
- Modify: `src/database/directory.ph`
- Modify: `src/reporting/reporter.ph`
- Modify: `src/engine/engine.ph`
- Modify: `src/core/errors.ph`

**Interfaces:**
- Consumes: immutable examples, canonical database keys, typed report events.
- Produces: adapter-neutral copy semantics, process-local directory exclusion with merge-on-write, reporter failure classification, and deterministic composite forwarding.

- [x] Make save/delete read-modify-write operations mutually exclusive within the process and merge from the latest visible record set.
- [x] Keep recoverable database failures as cache misses.
- [x] Prevent reporter exceptions from being mistaken for generated property failures.
- [x] Preserve event order and exactly-once forwarding to each composite child.

### Task 5: Performance corrections and regression corpus

**Files:**
- Modify: `src/choices/buffer.ph`
- Modify: `src/choices/data.ph`
- Modify: `src/choices/example.ph`
- Modify: `src/database/database.ph`
- Modify: relevant benchmark and regression fixtures

**Interfaces:**
- Consumes: mutable construction stacks, immutable examples, stable signatures.
- Produces: constant-time stack pops, linear span freezing, join-based signature construction, and benchmark-documented complexity expectations.

- [x] Replace list-rebuilding stack pops with tail removal.
- [x] Store closed spans by identifier so freeze ordering is linear.
- [x] Build example and database signatures from parts plus `join` rather than repeated concatenation.
- [x] Verify every optimization is guarded by a regression fixture and does not alter semantics.

### Task 6: Public API, documentation, full verification, and checkpoint

**Files:**
- Modify: `src/hypothesis.ph`
- Complete: `docs/extension-api.md`
- Create: `docs/design/phase-11-hardening.md`
- Modify: `README.md`
- Modify: `CHANGELOG.md`
- Modify: `phalcom.toml`
- Modify: inherited verifiers only where Phase 11 legitimately extends public ownership/version markers
- Modify: `CHECKPOINT.md`
- Modify: `TEST-RESULTS.md`
- Regenerate: `SHA256SUMS`
- Create: `phalcom-hypothesis-phase-11-hardening.zip`

**Interfaces:**
- Produces: documented stable extension contracts, public provider/base/pass exports, benchmark and mutation evidence, and a complete independently verifiable Phase 11 checkpoint.

- [x] Run Python byte-compilation, mutation verification, and every Phase 01–11 verifier in the working tree.
- [x] Run the real Phalcom tests and benchmarks only when an executable exists; otherwise state the runtime boundary precisely.
- [x] Audit internal imports, active-source delimiters, retired syntax, public top-level names, baseline preservation, and placeholder removal.
- [x] Regenerate checksums, build the complete archive, extract it cleanly, and rerun integrity, compilation, mutation, audit, and Phase 01–11 verification.
- [x] Record exact counts, archive SHA-256, and Phase 12 as the next phase.

# Phalcom Hypothesis Phase 07 Reporting Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every public observation visible through typed report events, stable console and JSON renderers, statistics, failure notes, health/flaky classification, and exact in-process reproduction tokens.

**Architecture:** The search engine emits immutable `ReportEvent` values through a `Reporter` protocol and does not format output. `PropertyRunner` owns suite/property lifecycle events and reporters render or retain them. Failure notes are captured from the minimal failing context; event/classification counts are aggregated in immutable statistics. Public targeting is removed from v1 because the current engine has no evidence-backed target optimizer.

**Tech Stack:** Typed Phalcom source, Python source-contract verifiers, golden text/JSON fixtures, ZIP/checksum checkpoint protocol.

## Global Constraints

- Preserve every Phase 01–06 source file and verifier unless an ownership assertion is obsolete.
- Write failing Phase 07 tests and verifier checks before production source.
- Do not implement Phase 08 persistence or Phase 09 stateful reporting early.
- Do not leave public note, event, classification, or targeting APIs as no-ops.
- Retain `Phase.Target` as reserved metadata, but remove `Property.target` from the v1 public surface.
- Keep runtime claims separate from source/static verification when no `phalcom` executable exists.

---

### Task 1: Red Phase 07 contract gate

**Files:**
- Create: `scripts/verify_phase07.py`
- Create: `tests/reporting/event_ordering.ph`
- Create: `tests/reporting/failure_notes.ph`
- Create: `tests/reporting/event_statistics.ph`
- Create: `tests/reporting/health_and_flaky.ph`
- Create: `tests/reporting/json_schema.ph`
- Create: `tests/reporting/reproduction_token.ph`
- Create: `tests/reporting/targeting_removed.ph`
- Create: `tests/golden/reporting/pass.txt`
- Create: `tests/golden/reporting/failure.txt`
- Create: `tests/golden/reporting/health.txt`
- Create: `tests/golden/reporting/flaky.txt`
- Create: `tests/golden/reporting/property.jsonl`

**Interfaces:**
- Consumes: Phase 06 `PropertyRunner`, `PropertyRun`, `SearchEngine`, `Failure`, `Statistics`.
- Produces: an executable source-contract gate requiring all Phase 07 ownership and behavior.

- [ ] Write tests that use `RecordingReporter`, `ConsoleReporter`, `JsonReporter`, `Property.note`, `Property.event`, `Property.classify`, and `ReproductionToken`.
- [ ] Write `verify_phase07.py` to check tests, event variants, reporter implementations, engine emission, note/statistics flow, JSON schema, reproduction support, targeting removal, façade migration, imports, and placeholders.
- [ ] Run `python3 scripts/verify_phase07.py` and record the intended red state caused by reporting placeholders.

### Task 2: Typed event and reporter model

**Files:**
- Replace: `src/reporting/event.ph`
- Replace: `src/reporting/reporter.ph`

**Interfaces:**
- Produces: sealed immutable `ReportEvent`, `Reporter`, `NullReporter`, `RecordingReporter`, and `CompositeReporter`.

- [ ] Define suite, property, phase, example, failure, shrink, health, and completion event variants.
- [ ] Implement recording and composite delivery without output formatting.
- [ ] Run the Phase 07 verifier and confirm only later tasks remain red.

### Task 3: Observation capture and statistics

**Files:**
- Modify: `src/core/context.ph`
- Modify: `src/core/failure.ph`
- Modify: `src/core/statistics.ph`
- Modify: `src/engine/evaluator.ph`
- Modify: `src/property/builder.ph`

**Interfaces:**
- Produces: visible `Property.note`, `Property.event`, and `Property.classify`; failure-local notes; copied event counts.

- [ ] Delegate observation APIs to the active context.
- [ ] Capture context notes in `Failure.from` while retaining the compatibility overload.
- [ ] Aggregate pass/reject/failure context events into `Statistics`.
- [ ] Remove `_targets` and the public `Property.target` method.
- [ ] Run Phase 07 and inherited verifiers.

### Task 4: Engine and runner event delivery

**Files:**
- Modify: `src/engine/engine.ph`
- Modify: `src/engine/shrinker.ph`
- Modify: `src/property/runner.ph`

**Interfaces:**
- Consumes: `Reporter#handle` and `ReportEvent` factories.
- Produces: deterministic suite/property/phase/example/failure/shrink/completion ordering.

- [ ] Preserve `SearchEngine.check(spec)` as a silent overload and add `check(spec, reporter:)`.
- [ ] Emit phase starts, accepted/rejected examples, health checks, failures, and shrink acceptance.
- [ ] Add reporter-aware shrinking without changing candidate semantics.
- [ ] Add reporter overloads to `PropertyRunner.run` and attach settings to each `PropertyRun`.
- [ ] Run the event-ordering contract and all inherited gates.

### Task 5: Reproduction tokens

**Files:**
- Replace: `src/reporting/reproduction.ph`

**Interfaces:**
- Produces: immutable `ReproductionToken`, exact example/settings retention, stable text fingerprint, and replay helpers.

- [ ] Build tokens from a falsified `PropertyRun`.
- [ ] Preserve exact immutable example and settings values in the token.
- [ ] Replay through a reuse-only `PropertySpec` and the authoritative engine.
- [ ] Exclude explicit-case failures from generated-example reproduction tokens.
- [ ] Run Phase 07 tests and inherited gates.

### Task 6: Console and JSON reporters

**Files:**
- Replace: `src/reporting/console.ph`
- Replace: `src/reporting/json.ph`

**Interfaces:**
- Produces: stable console lines and JSON-lines records from `ReportEvent` values.

- [ ] Render pass, fail, inconclusive, health-check, flaky, and engine-error outcomes distinctly.
- [ ] Include named arguments, failure notes, observation counts, statistics, and reproduction text where applicable.
- [ ] Implement stable JSON schema version `1` with deterministic keys and escaped strings.
- [ ] Compare source-declared golden output against all reporting golden fixtures.

### Task 7: Ownership migration and checkpoint

**Files:**
- Modify: `src/hypothesis.ph`
- Modify: `src/_internal/legacy_adapter.ph`
- Modify: `src/_internal/phase01_surface.ph`
- Modify: `phalcom.toml`
- Modify: `README.md`
- Modify: `CHANGELOG.md`
- Modify: `CHECKPOINT.md`
- Modify: `TEST-RESULTS.md`
- Modify inherited verifiers only where ownership assumptions are obsolete.

**Interfaces:**
- Produces: authoritative root exports for all Phase 07 reporting APIs and no duplicate legacy reporter ownership.

- [ ] Move `Reporter`, `ConsoleReporter`, `JsonReporter`, `ReportEvent`, and reproduction exports to authoritative modules.
- [ ] Preserve `PropertyReporter.console` as a compatibility factory in the new console module.
- [ ] Remove reporting placeholders and legacy reporter classes.
- [ ] Set version `0.1.0-phase.07` and document targeting removal.
- [ ] Run Python compilation and Phase 01–07 verifiers.
- [ ] Regenerate `SHA256SUMS`, build `phalcom-hypothesis-phase-07-reporting.zip`, extract it cleanly, and rerun checksum plus every verifier.

# Phase 09 Rule-Based Stateful Testing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Replace the Phase 01 stateful placeholders and temporary legacy state machine with an authoritative, typed, rule-based stateful testing slice that uses the existing DrawData, Example, SearchEngine, Shrinker, Reporter, and ExampleDatabase infrastructure.

**Architecture:** Immutable descriptors (`Bundle`, rule arguments, rule definitions, result references, actions, scenarios, and metadata) are separated from per-evaluation mutable workers. `_StatefulScenarioStrategy` executes a fresh machine incrementally while consuming `DrawData`, places every normal action in a discardable semantic span, and returns the immutable scenario; failures carry the partial scenario while preserving the underlying source-aware `FailureOrigin`. `Stateful.check` delegates to the existing search engine and database key/codec infrastructure.

**Tech Stack:** Typed Phalcom source, passive reflected attributes, Python static/source verifiers, ZIP/SHA-256 checkpoint tooling.

## Global Constraints

- Implement only Phase 09; do not add Phase 10 derivation, Phase 11 provider work, or Phase 12 release cleanup.
- Stateful testing remains a structured client of the shared search kernel.
- Every evaluation creates a fresh machine and fresh bundle storage.
- Initializers run in stable selector order, then invariants once; invariants run after every normal rule.
- Teardown runs exactly once after execution begins on every outcome.
- Normal actions own discardable spans containing selection, argument, bundle-reference, and local choices.
- Invalid dependency replays are engine overruns/cache misses, never counterexamples.
- Runtime behavior is reported as unobserved when no `phalcom` executable is available.

---

### Task 1: Phase 09 acceptance contract

**Files:**
- Create: `scripts/verify_phase09.py`
- Create: `tests/stateful/*.ph`

**Interfaces:**
- Consumes: the handoff’s fourteen required stateful behaviors.
- Produces: a source/static verifier that fails on Phase 08 placeholders and passes only after ownership migration and contract implementation.

- [x] Write all fourteen focused fixtures before production source.
- [x] Implement verifier checks for tests, descriptors, discovery, execution, shrinking, reporting, persistence, facade exports, migration, privacy, and import resolution.
- [x] Run `python3 scripts/verify_phase09.py` and record the expected red result caused by Phase 08 placeholders.

### Task 2: Passive attributes, bundles, and immutable model

**Files:**
- Modify: `src/stateful/attributes.ph`
- Modify: `src/stateful/bundle.ph`
- Modify: `src/stateful/argument.ph`
- Modify: `src/stateful/action.ph`
- Modify: `src/stateful/rule.ph`
- Modify: `src/stateful/scenario.ph`

**Interfaces:**
- Consumes: `Strategy`, reflected `Method`/`Parameter`, immutable data/variant decorators.
- Produces: `Rule`, `Initialize`, `StateInvariant`, `When`, `Teardown`, `Bundle<T>`, `RuleArgument`, `ResultReference`, `StateArgument`, `StateAction`, `StateScenario`, `RuleDefinition`, and `StateMachineMetadata`.

- [x] Implement immutable bundle descriptors with non-consuming selection, consuming selection, and publish-target markers.
- [x] Normalize attribute parts into argument sources and target bundles without wrapping methods.
- [x] Represent semantic kinds with sealed variants rather than strings.
- [x] Render literal and reference arguments differently and generate stable result names.
- [x] Run `python3 scripts/verify_phase09.py`; descriptor checks should pass while execution checks remain red.

### Task 3: Discovery, incremental execution, and teardown

**Files:**
- Modify: `src/stateful/machine.ph`
- Modify: `src/stateful/runner.ph`
- Modify: `src/core/errors.ph`

**Interfaces:**
- Consumes: `DrawData.withSpan`, reflected metadata, `SearchEngine`, `PropertySpec`, `DatabaseKey`, `ExampleCodec`, reporters.
- Produces: fresh per-evaluation context, stable discovery/fingerprint, incremental scenario strategy/executor, stateful target/run wrapper, and `Stateful.check`.

- [x] Discover methods in stable selector order and reject missing rules, duplicate/contradictory attributes, arity mismatches, invalid bundle sources, and multiple teardowns.
- [x] Execute initializers before rules, apply `@When` before rule selection, stop successfully when no rule is applicable, and resolve bundle references against current values.
- [x] Publish rule results to one or more bundles and remove consumed references after selection.
- [x] Guarantee one teardown call by structural result capture, preserving primary and secondary errors.
- [x] Preserve the partial scenario and delegate `failureOrigin` to the underlying user error.
- [x] Run the Phase 09 verifier and all inherited verifiers.

### Task 4: Reporting, persistence, and compatibility migration

**Files:**
- Modify: `src/reporting/console.ph`
- Modify: `src/reporting/json.ph`
- Modify: `src/reporting/reproduction.ph`
- Modify: `src/hypothesis.ph`
- Modify: `src/_internal/legacy_adapter.ph`
- Modify: `src/_internal/phase01_surface.ph`
- Modify minimally: inherited verifiers that hard-code obsolete stateful ownership.
- Modify: `examples/stateful_database.ph`

**Interfaces:**
- Consumes: stateful error/scenario protocol and existing report events/reproduction tokens.
- Produces: executable-style stateful rendering, stateful JSON fields, authoritative root exports, delegate-only compatibility aliases, and a real database model example.

- [x] Render stateful scenario lines before the underlying failure and render secondary teardown failures as context.
- [x] Include scenario text in JSON failure records without creating a second event/reporting system.
- [x] Move facade exports to `src/stateful`, retaining only `RuleBasedStateMachine` and `Invariant` aliases.
- [x] Remove all legacy stateful classes and behavior from `_internal/legacy_adapter.ph`.
- [x] Replace Phase 01 `Bundle`, `When`, and `Teardown` placeholders with compatibility aliases.
- [x] Run Phase 01–09 verifiers.

### Task 5: Checkpoint completion

**Files:**
- Modify: `phalcom.toml`
- Modify: `README.md`
- Modify: `docs/stateful.md`
- Modify: `CHANGELOG.md`
- Modify: `CHECKPOINT.md`
- Modify: `TEST-RESULTS.md`
- Regenerate: `SHA256SUMS`
- Create: `phalcom-hypothesis-phase-09-stateful.zip`

**Interfaces:**
- Produces: a complete independently verifiable Phase 09 project archive.

- [x] Run Python byte-compilation and Phase 01–09 verifiers in the working tree.
- [x] Detect and run the Phalcom test gate only when a `phalcom` executable exists; otherwise record runtime behavior as unobserved.
- [x] Regenerate `SHA256SUMS` over every project file except itself.
- [x] Create the full archive, extract it cleanly, verify checksums, and rerun Phase 01–09 verifiers from the extraction.
- [x] Record exact counts, archive file count, archive SHA-256, and Phase 10 as the next phase.

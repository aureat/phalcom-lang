# Phase 06 Property API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver the canonical reflective property-testing API over the existing Phase 05 search kernel.

**Architecture:** Passive attributes and reflection produce immutable property definitions. Recursive strategy inference resolves parameter types, invocation targets adapt methods and blocks, and both reflective suites and fluent builders delegate to `SearchEngine` without duplicating search or shrinking.

**Tech Stack:** Typed Phalcom source, Python 3 static/source verifiers, ZIP checkpoint packaging, SHA-256 manifest verification.

## Global Constraints

- Preserve every Phase 01–05 project file and verifier guarantee.
- Write Phase 06 fixtures and verifier before production implementation and observe the intended red state.
- Keep type annotations reflective and non-enforcing.
- Do not implement Phase 07 reporting/targeting, Phase 08 persistence, or Phase 09 stateful expansion.
- Use current Phalcom syntax and `_`-prefix implementation-only top-level names.
- Record static verification separately from unavailable Phalcom runtime verification.

---

### Task 1: Phase 06 red fixtures and verifier

**Files:**
- Create: `tests/property/inferred_given.ph`
- Create: `tests/property/inference_error.ph`
- Create: `tests/property/explicit_arity.ph`
- Create: `tests/property/named_overrides.ph`
- Create: `tests/property/explicit_case_names.ph`
- Create: `tests/property/builder_api.ph`
- Create: `tests/property/assertion_origins.ph`
- Create: `tests/property/runner_output.ph`
- Create: `scripts/verify_phase06.py`

**Interfaces:**
- Consumes: Phase 05 `SearchEngine`, `PropertySpec`, `StrategyRegistry`, and immutable result models.
- Produces: executable-design fixtures and a source verifier that fails while property modules are placeholders.

- [ ] Write the eight focused fixtures with exact canonical API use.
- [ ] Write `verify_phase06.py` to validate fixture intent, property ownership, inference, discovery, builder delegation, runner output, adapter migration, privacy, imports, and delimiters.
- [ ] Run `python3 scripts/verify_phase06.py` and record the expected missing-implementation failures.

### Task 2: Attributes and recursive strategy inference

**Files:**
- Modify: `src/property/attributes.ph`
- Modify: `src/property/inference.ph`
- Modify: `src/strategies/registry.ph`
- Modify: `src/core/errors.ph`

**Interfaces:**
- Produces: `Given`, `GivenArgs`, `GivenMode`, `Case`, `WithSettings`, `_ReflectedParameter`, and `_StrategyInference.resolve`.

- [ ] Implement immutable named overrides and exactly three `@Given` modes.
- [ ] Implement reflected parameter extraction and precise diagnostics.
- [ ] Extend `StrategyRegistry.forType` for `Option`, `List`, tuples, `Set`, `Map`, and `Result` applied types.
- [ ] Run the Phase 06 verifier and confirm the attribute/inference gate turns green while later gates remain red.

### Task 3: Invocation targets and source-aware assertions

**Files:**
- Modify: `src/property/target.ph`
- Modify: `src/property/assertion.ph`
- Modify: `src/core/failure.ph`

**Interfaces:**
- Produces: `_InvocationTarget`, `_MethodTarget`, `_BlockTarget`, `_PropertyAssertionError`, `Assert`, and caller-derived `FailureOrigin` values.

- [ ] Implement method/block invocation adapters.
- [ ] Implement assertion methods that capture caller source locations before throwing.
- [ ] Add `PropertySuite` convenience forwarding without changing captured user call sites.
- [ ] Run the assertion-origin source gate.

### Task 4: Builder and property context API

**Files:**
- Modify: `src/property/builder.ph`

**Interfaces:**
- Produces: `Property`, `PropertyBuilder<T...>`, and `PropertySuite`.

- [ ] Implement `Property.given(*strategies)` and immutable `.using(settings)`.
- [ ] Implement `.check(body)` by constructing `PropertySpec` and calling `SearchEngine`.
- [ ] Delegate `Property.forAll` and `Property.find` to the same engine.
- [ ] Keep Phase 07 observation methods explicitly unavailable rather than silent.
- [ ] Run the builder fixture gate.

### Task 5: Reflective discovery and runner

**Files:**
- Modify: `src/property/discovery.ph`
- Modify: `src/property/runner.ph`
- Modify: `src/engine/specification.ph`

**Interfaces:**
- Produces: `PropertyId`, `PropertyDefinition`, `PropertyDiscovery`, `PropertyRun`, `PropertySuiteResult`, and `PropertyRunner`.

- [ ] Discover exactly one `@Given`, at most one `@WithSettings`, all `@Case` values, reflected names/types, and local settings.
- [ ] Resolve strategies according to the selected `GivenMode` and reject invalid metadata before search.
- [ ] Execute definitions through `SearchEngine`, preserve temporary injected reuse behavior, and return named result records.
- [ ] Ensure explicit cases remain engine-level explicit examples and named arguments are derived from reflected names.
- [ ] Run the runner and case-name gates.

### Task 6: Façade and compatibility migration

**Files:**
- Modify: `src/hypothesis.ph`
- Modify: `src/_internal/legacy_adapter.ph`
- Modify: `src/_internal/phase01_surface.ph`
- Modify: `tests/integration/current_acceptance.ph`

**Interfaces:**
- Produces: canonical root exports and compatibility aliases `Check`, `CheckConfig`, `PropertyReporter`, and later-phase stateful names.

- [ ] Export Phase 06 property classes directly from their authoritative modules.
- [ ] Remove compatibility-owned `Given`, `Case`, `Check`, `Assert`, `PropertySuite`, `Property`, and `PropertyRunner` class declarations.
- [ ] Keep only database/reporting/stateful compatibility bridges and make them consume `PropertySuiteResult`.
- [ ] Update inherited verifier expectations only where ownership moved, preserving semantic guarantees.
- [ ] Run Phase 01–06 verifiers.

### Task 7: Documentation, manifest, checksums, and archive

**Files:**
- Modify: `README.md`
- Modify: `docs/inference.md`
- Modify: `CHANGELOG.md`
- Modify: `CHECKPOINT.md`
- Modify: `TEST-RESULTS.md`
- Modify: `phalcom.toml`
- Regenerate: `SHA256SUMS`

**Interfaces:**
- Produces: `phalcom-hypothesis-phase-06-property-api.zip` and checkpoint reports naming Phase 07 next.

- [ ] Document canonical bare/explicit/override `@Given` forms and builder usage.
- [ ] Record the observed red state and fresh green verification outputs.
- [ ] Set the package version to `0.1.0-phase.06`.
- [ ] Regenerate the complete checksum manifest excluding itself.
- [ ] Run Python compilation, Phase 01–06 gates, and checksum verification in the working tree.
- [ ] Build the complete archive, extract it cleanly, and repeat every gate and checksum verification.

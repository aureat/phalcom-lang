# Phalcom Hypothesis Package Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` where available, or `superpowers:executing-plans` to implement this plan phase by phase. Every phase ends with a full-package checkpoint archive.

**Goal:** Build a complete, typed, Phalcom-native property-based testing package with compositional strategies, deterministic replay, structural shrinking, reflective `@Given` discovery, persistent examples, reporting, stateful testing, and derived strategies for typed data and sealed variants.

**Architecture:** The package is organized around an immutable semantic example containing typed primitive choices and spans. Strategies consume choices through `DrawData`; generation records them, replay reuses them, and shrinking transforms the example before rerunning the full property. Ordinary property testing, `find`, stateful testing, databases, and reporters are clients of the same search kernel.

**Tech Stack:** Phalcom source modules, reflective optional type annotations, passive and compile-time attributes, `@constructor`, `@data`, `@immutable`, `@sealed`, `@variant`, contracts, module imports, exact `Int`, `Bytes`, `Random`, filesystem APIs, golden tests, and a minimal `phalcom` project toolchain.

## Global Constraints

- The package is implemented in Phalcom. Native acceleration is an optional backend and must not define observable semantics.
- Constructors use `@constructor`; the retired `construct` syntax is forbidden in active source.
- `if`, `while`, and `for` conditions are written without mandatory parentheses.
- Mechanical arithmetic updates use `+=`, `-=`, `++`, or `--`.
- Type annotations are reflective runtime metadata but do not automatically enforce runtime checks or participate in dispatch.
- Public APIs are typed. Internal code is typed wherever a useful type can be stated without artificial `?` proliferation.
- Attribute-driven tests and explicit property objects use the same engine.
- `@Given` supports full explicit strategies and annotation-based inference.
- String tags are not used for semantic states; use `@sealed` and `@variant`.
- Immutable result and metadata values use `@data @immutable`; active workers remain mutable.
- Public module names are stable. Implementation-only top-level names begin with `_`.
- Every phase begins with failing tests and ends with all tests introduced through that phase passing.
- Every phase returns a complete package archive, not a patch-only artifact.
- Every checkpoint contains `CHECKPOINT.md`, `TEST-RESULTS.md`, `CHANGELOG.md`, and `SHA256SUMS`.
- No public API may silently record information that no engine phase or reporter consumes.
- Failure shrinking preserves a stable failure origin, not merely an exception class.
- Replay and database corruption are normal cache-invalidating conditions, not falsifying examples.
- Property context is scoped with guaranteed cleanup; parallel execution is introduced only after fiber-local context exists.
- The final package includes no incomplete stubs, placeholder methods, or deferred public names.

---

# 1. Target Project Layout

```text
phalcom-hypothesis/
├── phalcom.toml
├── README.md
├── CHANGELOG.md
├── LICENSE
├── CHECKPOINT.md
├── TEST-RESULTS.md
│
├── src/
│   ├── hypothesis.ph
│   ├── core/
│   │   ├── errors.ph
│   │   ├── settings.ph
│   │   ├── phase.ph
│   │   ├── status.ph
│   │   ├── failure.ph
│   │   ├── statistics.ph
│   │   └── context.ph
│   ├── choices/
│   │   ├── choice.ph
│   │   ├── request.ph
│   │   ├── span.ph
│   │   ├── example.ph
│   │   ├── buffer.ph
│   │   ├── provider.ph
│   │   └── data.ph
│   ├── strategies/
│   │   ├── strategy.ph
│   │   ├── combinators.ph
│   │   ├── primitives.ph
│   │   ├── collections.ph
│   │   ├── composite.ph
│   │   ├── registry.ph
│   │   └── gen.ph
│   ├── engine/
│   │   ├── specification.ph
│   │   ├── evaluator.ph
│   │   ├── complexity.ph
│   │   ├── shrink_pass.ph
│   │   ├── shrinker.ph
│   │   ├── search.ph
│   │   └── engine.ph
│   ├── property/
│   │   ├── attributes.ph
│   │   ├── assertion.ph
│   │   ├── target.ph
│   │   ├── inference.ph
│   │   ├── discovery.ph
│   │   ├── builder.ph
│   │   └── runner.ph
│   ├── stateful/
│   │   ├── attributes.ph
│   │   ├── machine.ph
│   │   ├── rule.ph
│   │   ├── bundle.ph
│   │   ├── argument.ph
│   │   ├── action.ph
│   │   ├── scenario.ph
│   │   └── runner.ph
│   ├── database/
│   │   ├── key.ph
│   │   ├── database.ph
│   │   ├── memory.ph
│   │   ├── codec.ph
│   │   └── directory.ph
│   ├── reporting/
│   │   ├── event.ph
│   │   ├── reporter.ph
│   │   ├── console.ph
│   │   ├── json.ph
│   │   └── reproduction.ph
│   └── _internal/
│       ├── sequences.ph
│       ├── fingerprints.ph
│       └── ordering.ph
│
├── tests/
│   ├── core/
│   ├── choices/
│   ├── strategies/
│   ├── engine/
│   ├── property/
│   ├── database/
│   ├── stateful/
│   ├── reporting/
│   ├── integration/
│   └── golden/
│
├── examples/
│   ├── arithmetic.ph
│   ├── collections.ph
│   ├── codec.ph
│   ├── parser_roundtrip.ph
│   ├── derived_data.ph
│   ├── recursive_expression.ph
│   └── stateful_database.ph
│
└── docs/
    ├── concepts.md
    ├── strategies.md
    ├── shrinking.md
    ├── inference.md
    ├── stateful.md
    ├── database.md
    ├── extension-api.md
    └── design/
```

The tree may gain test fixtures and documentation pages, but active source responsibilities must remain within these boundaries unless a later checkpoint records a justified architecture change.

---

# 2. Stable Public Surface

The root `src/hypothesis.ph` façade exports these names:

```phalcom
Given
Case
WithSettings
Settings
Phase

Strategy<T>
Gen
StrategyRegistry

Property
PropertyBuilder<T...>
PropertySuite
Assert

PropertyResult
PropertyId
Failure
Statistics

ExampleDatabase
MemoryDatabase
DirectoryDatabase

StateMachine
Stateful
Rule
Initialize
StateInvariant
When
Teardown
Bundle<T>

Reporter
ConsoleReporter
JsonReporter
```

The canonical property form is:

```phalcom
import Given, PropertySuite, Assert from hypothesis

class ArithmeticProperties is PropertySuite {
  @Given
  additionIsCommutative(a: Int, b: Int) {
    Assert.equal(a + b, b + a)
  }
}
```

The explicit strategy form remains available:

```phalcom
@Given(
  Gen.int(min: -100, max: 100),
  Gen.list(of: Gen.int)
)
property(n: Int, values: List<Int>) {
  ...
}
```

The explicit object API is:

```phalcom
Property
  .given(Gen.int, Gen.int)
  .using(Settings.standard.maxExamples(500))
  .check { a: Int, b: Int =>
    Assert.equal(a + b, b + a)
  }
```

---

# 3. Checkpoint Delivery Protocol

Every completed phase produces a full archive named:

```text
phalcom-hypothesis-phase-NN-<slug>.zip
```

The archive always contains the entire project root. It is never only a patch.

Each checkpoint response includes:

1. A link to the full archive.
2. A link to `CHECKPOINT.md`.
3. A link to `TEST-RESULTS.md`.
4. A concise list of completed public behavior.
5. Honest disclosure of anything not executable or not verified.

`CHECKPOINT.md` records:

```markdown
# Checkpoint NN

- Phase: <name>
- Status: complete
- Source baseline: <previous checkpoint>
- Public APIs added:
- Public APIs changed:
- Tests added:
- Verification commands:
- Known limitations:
- Next phase:
```

`TEST-RESULTS.md` records exact commands and observed outputs. A planned result is never recorded as an observed result.

`SHA256SUMS` covers every file in the archive except itself.

---

# 4. Phase Roadmap

## Phase 00 — Plan and legacy baseline

**Purpose:** Preserve the original monolithic draft and establish the implementation contract.

**Files:**
- Preserve: `legacy/hypothesis-monolith.ph`
- Create: `docs/plans/2026-07-23-phalcom-hypothesis-package.md`
- Create: `docs/architecture/target-project-tree.md`
- Create: `docs/plans/checkpoint-protocol.md`
- Create: `CHECKPOINT.md`

**Acceptance:**
- The original source remains downloadable.
- The target architecture and stable public surface are recorded.
- No active multi-file implementation is claimed.

**Checkpoint:** `phalcom-hypothesis-phase-00-plan.zip`

---

## Phase 01 — Real project scaffold and syntax migration

**Goal:** Produce a valid multi-module Phalcom package whose façade imports and test command work, while preserving the prototype behavior in a temporary compatibility module.

**Files:**
- Create: `phalcom.toml`
- Create: `src/hypothesis.ph`
- Create all source directories
- Create: `src/_internal/legacy_adapter.ph`
- Create: `tests/integration/package_loads.ph`
- Create: `tests/integration/current_acceptance.ph`
- Create: `README.md`, `LICENSE`, `CHANGELOG.md`

**Interfaces produced:**
- Root import module `hypothesis`
- Empty but typed public protocols and façade aliases
- Minimal toolchain commands:
  - `phalcom test`
  - `phalcom test tests/integration/package_loads.ph`
  - `phalcom run examples/arithmetic.ph`

**TDD sequence:**
- [ ] Write `package_loads.ph` importing every public façade name.
- [ ] Run it and confirm module-resolution failures.
- [ ] Add the package manifest and root façade.
- [ ] Split the old monolith into a non-public compatibility module only far enough to load.
- [ ] Migrate all active constructor syntax to `@constructor`.
- [ ] Migrate active control-flow syntax and compound updates.
- [ ] Add typed signatures to public compatibility declarations.
- [ ] Run the package-load test and current acceptance fixture.
- [ ] Confirm no active file contains `construct `.
- [ ] Confirm no active file contains legacy parenthesized `if (` / `while (` / `for (` forms.
- [ ] Commit checkpoint state.

**Acceptance output:**

```text
PASS package imports
PASS legacy acceptance through compatibility adapter

2 passed, 0 failed
```

**Checkpoint:** `phalcom-hypothesis-phase-01-scaffold.zip`

---

## Phase 02 — Typed immutable core model

**Goal:** Replace string-tagged settings, results, phases, statuses, and failures with typed immutable values and sealed variants.

**Files:**
- Create all `src/core/*.ph`
- Create: `tests/core/settings.ph`
- Create: `tests/core/status_match.ph`
- Create: `tests/core/failure_origin.ph`
- Create: `tests/core/context_cleanup.ph`

**Interfaces produced:**

```phalcom
@data @immutable
class Settings

@data @sealed
class Phase

@data @sealed
class ExampleStatus

@data @sealed
class PropertyResult

@data @immutable
class FailureOrigin

@data @immutable
class Failure

@data @immutable
class Statistics
```

`Settings` provides getter/update selector pairs:

```phalcom
settings.maxExamples -> Int
settings.maxExamples(value: Int) -> Settings
```

**TDD sequence:**
- [ ] Test that `Settings.standard` has exact documented defaults.
- [ ] Test each fluent update returns a new value and leaves the receiver unchanged.
- [ ] Test settings contracts reject invalid values.
- [ ] Test exhaustive `.match(...)` over every status and result variant.
- [ ] Test failure origins distinguish two assertion sites with the same error class.
- [ ] Test property context cleanup on normal return and thrown error using `ensure`.
- [ ] Implement the core modules.
- [ ] Replace compatible portions of the legacy adapter with core values.
- [ ] Run core and integration tests.

**Acceptance:** No semantic state in active core source is represented by a free-form string.

**Checkpoint:** `phalcom-hypothesis-phase-02-core.zip`

---

## Phase 03 — Typed choices, spans, examples, providers, and replay

**Goal:** Implement the immutable semantic example model used by generation, replay, serialization, and shrinking.

**Files:**
- Create all `src/choices/*.ph`
- Create: `tests/choices/choice_variants.ph`
- Create: `tests/choices/buffer_freeze.ph`
- Create: `tests/choices/span_tree.ph`
- Create: `tests/choices/generate_replay.ph`
- Create: `tests/choices/overrun.ph`

**Interfaces produced:**

```phalcom
@data @sealed
class Choice

@data @sealed
class ChoiceRequest

@data @immutable
class Span

@data @immutable
class Example

protocol ChoiceProvider

class DrawData
```

`ChoiceBuffer` is mutable during construction and freezes into an immutable `Example`.

**TDD sequence:**
- [ ] Test integer, Boolean, index, and bytes choices.
- [ ] Test request bounds and shrink targets.
- [ ] Test nested spans for list elements and composite strategies.
- [ ] Test generation followed by replay yields identical values and identical normalized choices.
- [ ] Test exhausted replay and choice-budget exhaustion produce `Overrun`, not `Interesting`.
- [ ] Test freezing prevents later mutation from changing an example.
- [ ] Implement providers and data.
- [ ] Run the full suite.

**Acceptance:** A generated example can be replayed deterministically without invoking randomness.

**Checkpoint:** `phalcom-hypothesis-phase-03-choices.zip`

---

## Phase 04 — Typed strategy protocol and standard strategies

**Goal:** Implement the compositional strategy layer entirely over `DrawData`.

**Files:**
- Create all `src/strategies/*.ph`
- Create tests for primitives, collections, combinators, composite draws, recursion, and registry basics.

**Interfaces produced:**

```phalcom
protocol Strategy<out T>

class Gen

class StrategyRegistry
```

Supported strategies at phase completion:

- `Gen.int`
- `Gen.bool`
- `Gen.float`
- `Gen.bytes`
- `Gen.text`
- `Gen.just`
- `Gen.sampledFrom`
- `Gen.oneOf`
- `Gen.option`
- `Gen.result`
- `Gen.list`
- `Gen.set`
- `Gen.map`
- `Gen.tuple`
- `Gen.build`
- deferred and recursive strategies
- `map`, `filter`, `flatMap`, `named`

**TDD sequence:**
- [ ] Test each primitive strategy at minimum, maximum, and default bounds.
- [ ] Test deterministic replay for every standard strategy.
- [ ] Test `map`, `filter`, and `flatMap` composition.
- [ ] Test rejection accounting for filters.
- [ ] Test list spans identify every generated element.
- [ ] Test recursive strategies respect generation size and terminate at size zero.
- [ ] Test invalid strategy construction raises `InvalidStrategy`.
- [ ] Implement registry entries for built-in reflective types.
- [ ] Run strategy and integration suites.

**Acceptance:** The compatibility adapter no longer owns strategy generation.

**Checkpoint:** `phalcom-hypothesis-phase-04-strategies.zip`

---

## Phase 05 — Search engine and structural shrinker

**Goal:** Implement explicit, reuse, generation, shrinking, and verification phases over immutable examples and failure origins.

**Files:**
- Create all `src/engine/*.ph`
- Create tests for complexity ordering, shrink passes, phase ordering, failure preservation, flakiness, and `find`.

**Interfaces produced:**

```phalcom
@data @immutable
class PropertySpec<T...>

class SearchEngine

protocol ShrinkPass

class Shrinker
```

Initial shrink passes:

1. delete discardable spans;
2. shorten trailing choices;
3. minimize branch indices;
4. minimize individual integers;
5. minimize related integer blocks;
6. delete and simplify bytes/text spans;
7. minimize recursive structures.

**TDD sequence:**
- [ ] Write a property whose minimal integer counterexample is `10`.
- [ ] Write a list property requiring deletion of a middle element.
- [ ] Write a recursive-tree property requiring subtree deletion.
- [ ] Test every accepted shrink strictly decreases complexity.
- [ ] Test shrinking preserves the same `FailureOrigin`.
- [ ] Test a non-reproducible failure returns a flaky result.
- [ ] Test explicit examples execute before reuse and generation.
- [ ] Test `find` returns the minimal satisfying value without a fake exception.
- [ ] Implement engine phases and shrink passes.
- [ ] Run engine, strategy, and integration suites.

**Acceptance:** Structural shrinking is no longer limited to changing one integer choice or truncating a prefix.

**Checkpoint:** `phalcom-hypothesis-phase-05-engine.zip`

---

## Phase 06 — Property attributes, reflective inference, builder API, and runner

**Goal:** Deliver the canonical user-facing property-testing workflow.

**Files:**
- Create all `src/property/*.ph`
- Create tests for `@Given`, `@Case`, settings metadata, inferred strategies, explicit overrides, parameter names, and builder usage.

**Interfaces produced:**

```phalcom
@Given
@Case(...)
@WithSettings(settings)

class Property
class PropertyBuilder<T...>
class PropertySuite
class Assert
```

`@Given` modes:

1. no arguments: infer every strategy from reflected parameter types;
2. one strategy per parameter: explicit;
3. `GivenArgs` override object: infer remaining parameters and override named ones.

**TDD sequence:**
- [ ] Test bare `@Given` over `Int`, `Bool`, `String`, `Bytes`, `Option<T>`, `List<T>`, and tuples.
- [ ] Test an unannotated parameter produces a strategy-resolution diagnostic.
- [ ] Test explicit strategies need compatible arity.
- [ ] Test named overrides bind by reflected parameter name.
- [ ] Test `@Case` arguments are named in output and never shrunk.
- [ ] Test `Property.given(...).using(...).check { ... }`.
- [ ] Test two assertions with the same error type remain distinct by source origin.
- [ ] Implement discovery, inference, targets, assertions, and runner.
- [ ] Remove the compatibility runner.
- [ ] Run all suites.

**Acceptance output:**

```text
PASS ArithmeticProperties.additionIsCommutative
PASS CollectionProperties.reverseTwice

2 passed, 0 failed
```

**Checkpoint:** `phalcom-hypothesis-phase-06-property-api.zip`

---

## Phase 07 — Event reporting, statistics, notes, targeting decision, and reproduction

**Goal:** Make every public observation visible and decouple engine events from output formatting.

**Files:**
- Create all `src/reporting/*.ph`
- Create golden tests for console, JSON, notes, events, classifications, health checks, flaky results, and reproduction tokens.

**Interfaces produced:**

```phalcom
@data @sealed
class ReportEvent

protocol Reporter

class ConsoleReporter
class JsonReporter
```

**TDD sequence:**
- [ ] Test event ordering for a passing property.
- [ ] Test notes appear only with the relevant falsifying example.
- [ ] Test event/classification statistics appear in summaries.
- [ ] Test health checks are reported separately from counterexamples.
- [ ] Test JSON output has a stable documented schema.
- [ ] Test reproduction tokens replay exact choices and settings.
- [ ] Decide targeting by implementation evidence:
  - implement a real target phase and expose `Property.target`; or
  - keep targeting private and remove the public method from v1.
- [ ] Implement reporters and reproduction support.
- [ ] Run all golden tests.

**Acceptance:** No public note/event/classification/target API is a no-op.

**Checkpoint:** `phalcom-hypothesis-phase-07-reporting.zip`

---

## Phase 08 — Memory and persistent example databases

**Goal:** Persist minimal failures safely and reuse them before generating new examples.

**Files:**
- Create all `src/database/*.ph`
- Create tests for key stability, codec round trips, atomic writes, corruption recovery, retention limits, and process-to-process reuse.

**Interfaces produced:**

```phalcom
@data @immutable
class DatabaseKey

protocol ExampleDatabase

class MemoryDatabase
class DirectoryDatabase
```

**TDD sequence:**
- [ ] Test database keys include package, module, suite, selector, strategy fingerprint, and engine format version.
- [ ] Test save/fetch/delete for memory storage.
- [ ] Test example codec round-trips choices, spans, size, and failure metadata.
- [ ] Test corrupt files are ignored and quarantined or replaced without failing the property.
- [ ] Test writes use atomic replacement.
- [ ] Test bounded entry count and file size.
- [ ] Run a failing property twice and confirm the second run starts in the reuse phase.
- [ ] Implement database modules and wire settings.
- [ ] Run all suites.

**Acceptance:** A persisted counterexample survives a new process and is invalidated safely when replay no longer matches the strategy.

**Checkpoint:** `phalcom-hypothesis-phase-08-database.zip`

---

## Phase 09 — Rule-based stateful testing

**Goal:** Build realistic model-based testing using the same example and shrink engine.

**Files:**
- Create all `src/stateful/*.ph`
- Create tests for rule discovery, initializers, applicability, bundles, consumed values, invariants, teardown, references, and structural shrinking.

**Interfaces produced:**

```phalcom
class StateMachine
class Stateful

@Rule(...)
@Initialize(...)
@StateInvariant
@When(...)
@Teardown

class Bundle<T>
```

**TDD sequence:**
- [ ] Test a broken counter shrinks to one `decrement()` action.
- [ ] Test `@When` prevents selecting unavailable rules without counting as a rejected example.
- [ ] Test rule return values enter typed bundles.
- [ ] Test later actions reference earlier produced values.
- [ ] Test consumed bundle values cannot be reused.
- [ ] Test invariants run after initialization and every rule according to documented policy.
- [ ] Test teardown runs on pass, failure, rejection, and internal error.
- [ ] Test an irrelevant middle action is deleted during shrinking.
- [ ] Test reproduction output is executable-style and names result references.
- [ ] Implement stateful modules and runner.
- [ ] Run all suites.

**Acceptance:** Stateful testing is not a separate generator; it is a structured client of the shared choice engine.

**Checkpoint:** `phalcom-hypothesis-phase-09-stateful.zip`

---

## Phase 10 — Derived strategies, reflective annotations, data classes, and sealed variants

**Goal:** Make typed domain models effortless to generate while preserving explicit control.

**Files:**
- Extend: `src/strategies/registry.ph`
- Add derivation support modules if needed under `src/strategies/`
- Create tests for `@data`, `@immutable`, `@sealed`, `@variant`, recursive hierarchies, parameter metadata, and custom registrations.

**Interfaces produced:**

```phalcom
@arbitrary
@strategy(Type)

StrategyRegistry.register(type:, strategy:)
StrategyRegistry.forType(type:)
```

**TDD sequence:**
- [ ] Derive a strategy for an immutable `@data` class from typed constructor fields.
- [ ] Derive `oneOf` for a sealed hierarchy.
- [ ] Derive a size-aware recursive expression strategy.
- [ ] Test generic fields such as `Option<Int>` and `List<String>`.
- [ ] Test an unsupported field reports the complete resolution path.
- [ ] Test a constrained constructor refuses unsafe automatic derivation and recommends a custom strategy.
- [ ] Test `@strategy(Type)` overrides automatic derivation.
- [ ] Implement derivation and registry precedence.
- [ ] Run all suites.

**Acceptance:** Bare `@Given` can generate ordinary typed domain models and recursive sealed variants.

**Checkpoint:** `phalcom-hypothesis-phase-10-derivation.zip`

---

## Phase 11 — Extension API, alternate providers, performance, and hardening

**Goal:** Stabilize extension points and prove semantic equivalence across implementations.

**Files:**
- Complete: `docs/extension-api.md`
- Add provider conformance tests
- Add benchmarks and large regression fixtures
- Add package self-tests and mutation/regression corpus

**Interfaces stabilized:**

```phalcom
protocol ChoiceProvider
protocol Strategy<out T>
protocol ShrinkPass
protocol ExampleDatabase
protocol Reporter
```

**TDD and verification sequence:**
- [ ] Create a deterministic scripted provider.
- [ ] Create a system-random provider.
- [ ] Run the same property through both providers where deterministic inputs overlap.
- [ ] Add provider conformance tests.
- [ ] Add database-adapter conformance tests.
- [ ] Add reporter conformance tests.
- [ ] Add regression examples for every bug found during development.
- [ ] Benchmark primitive generation, nested lists, replay, integer shrinking, and stateful shrinking.
- [ ] Remove accidental quadratic behavior identified by benchmarks without changing observable semantics.
- [ ] Audit public API names, types, and documentation.
- [ ] Run the full verification gate.

**Acceptance:** Extension protocols are documented, tested, and have at least two implementations where practical.

**Checkpoint:** `phalcom-hypothesis-phase-11-hardening.zip`

---

## Phase 12 — Final integration and release-complete package

**Goal:** Produce the fully completed package requested by the project.

**Files:**
- Finalize all documentation and examples.
- Remove `legacy/` and compatibility adapters from the release archive.
- Finalize `CHANGELOG.md`, package metadata, and version.
- Add migration notes from the original monolith.

**Verification sequence:**
- [ ] Search active source for retired syntax and forbidden placeholders.
- [ ] Run unit, property, stateful, database, golden, and integration tests.
- [ ] Run examples as executable documentation.
- [ ] Run tests twice with fixed seeds.
- [ ] Run persistence tests across fresh processes.
- [ ] Verify every documented public name imports from `hypothesis`.
- [ ] Verify source archive and installed package produce identical test results.
- [ ] Generate `SHA256SUMS`.
- [ ] Create final release archive.

**Required final command:**

```sh
phalcom test --all
```

**Required final summary:**

```text
All tests passed.
No legacy syntax found.
No placeholder implementations found.
All public façade imports resolved.
```

**Final archive:** `phalcom-hypothesis-complete.zip`

---

# 5. Phase-to-Phase Compatibility Rules

1. A checkpoint archive must be independently understandable and include every source file required for that phase.
2. Later phases may replace internals but must preserve public behavior introduced by earlier phases unless the checkpoint explicitly records an approved API correction.
3. Golden output may evolve only when the reporting phase deliberately introduces richer output.
4. Old checkpoint archives remain immutable; each new archive is a new full snapshot.
5. Every phase’s tests remain in the package and run in all later phases.
6. A phase is not complete merely because files exist. Its acceptance tests must pass or its test limitations must be explicitly disclosed.
7. The final phase removes the legacy monolith from release source, but the migration document preserves its design history.

---

# 6. Completion Definition

The package is fully completed when all of the following are true:

- It is a real multi-file Phalcom project with a manifest, façade, tests, examples, and documentation.
- Active source uses the revised Phalcom syntax.
- Public APIs are typed and type metadata supports `@Given` inference.
- Core statuses and results use sealed variants.
- Generation, replay, shrinking, `find`, databases, and stateful testing share one semantic example model.
- Structural shrinking handles middle-element/action deletion and recursive subtrees.
- Failure identity is source-aware.
- Notes, events, classifications, health checks, and reproduction data are visible.
- Persistent failure reuse works across processes.
- Stateful bundles and applicability predicates work.
- Data and sealed domain models can derive strategies.
- Every extension protocol has conformance tests.
- All package tests and examples pass.
- The final full archive is linked as `phalcom-hypothesis-complete.zip`.

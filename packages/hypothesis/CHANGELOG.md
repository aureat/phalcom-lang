# Changelog

## 0.1.0 — Release-complete package

- Removed the historical monolith, obsolete Phase 01 migration generator, and both temporary compatibility modules from the release tree.
- Made the root `hypothesis` façade depend exclusively on authoritative feature modules.
- Preserved broad-v1 names as direct aliases without a second implementation path.
- Added the normative public API inventory and complete monolith-to-release migration guide.
- Added final release integration fixtures for façade imports, compatibility aliases, examples, and persistence.
- Added `verify_phase12.py` and the repeatable two-pass `verify_release.py` orchestration gate.
- Finalized package metadata at version `0.1.0`.
- Added clean-extraction verification for the complete release archive.

## Checkpoint 11 — Extension API, alternate providers, performance, and hardening

- Stabilized `ChoiceProvider`, `ChoiceProviderFactory`, `Strategy<out T>`, `ShrinkPass`, `ExampleDatabase`, and `Reporter` extension contracts.
- Added public system-random and scripted choice providers plus factories for fresh per-example provider creation.
- Unified scripted and replay request normalization and classified scripted exhaustion as an engine overrun.
- Added public `StrategyBase<T>` and typed custom shrink-pass pipelines.
- Added semantic duplicate-candidate suppression before replay while retaining strict complexity and failure-origin checks.
- Added reporter extension-failure classification through public `ReporterFailure`.
- Added shared process-local directory path exclusion and merge-on-write from the latest visible database bucket.
- Replaced copying stack pops, quadratic span ordering, and repeated signature concatenation with linear operations.
- Corrected codec validation to compare the full database-record signature that was written, preserving the existing on-disk signature format.
- Added five extension conformance fixtures, provider-equivalence integration coverage, five regression fixtures, five benchmark workloads, and a five-case mutation verifier.
- Completed extension documentation and added the Phase 11 design and implementation plan.

## Checkpoint 10 — Derived strategies, reflective annotations, data classes, and sealed variants

- Added passive `@arbitrary` class metadata and passive `@strategy(Type)` provider metadata.
- Added canonical `StrategyRegistry.register(type:, strategy:)` while retaining the Phase 04 `use:` compatibility spelling.
- Added explicit installation of zero-argument annotated strategy-provider methods.
- Added exact/custom/cached/applied/derived registry precedence and derived-cache invalidation.
- Added complete nested resolution-path diagnostics for constructor fields and applied containers.
- Added reflected single-constructor derivation with typed ordered parameters and label-aware fingerprints.
- Rejected missing annotations, rest parameters, ambiguous constructors, and constrained constructors before search.
- Added stable sealed-variant derivation through `Gen.oneOf`.
- Added terminal/recursive variant partitioning and size-aware recursive derivation through `Gen.recursive`.
- Added recursive substitution inside option, list, set, tuple, map, and result fields.
- Added bare `@Given` domain-model coverage, nine Phase 10 fixtures, design documentation, and complete examples.

## Checkpoint 09 — Rule-based stateful testing

- Replaced all eight Phase 01 stateful placeholders with authoritative typed modules.
- Added passive `@Rule`, `@Initialize`, `@StateInvariant`, `@When`, and `@Teardown` metadata.
- Added typed `Bundle<T>` descriptors with non-consuming selection, consuming selection, and result publication.
- Added immutable rule definitions, machine metadata, result references, literal/reference action arguments, actions, and scenarios.
- Added stable reflected discovery with arity, role, predicate, target, invariant, and teardown validation.
- Rejected missing or parameterized `@When` predicates, reflected non-`Bool` predicates, and typed bundle/result mismatches during discovery.
- Added incremental state-machine execution through `DrawData` with fresh machine and bundle state per evaluation.
- Added applicability filtering which does not count unavailable rules as rejected examples.
- Added initializer and invariant ordering, typed result flow, reference resolution, and consumed-value removal.
- Added structural teardown capture with primary and secondary failure context.
- Added source-aware stateful failure wrappers preserving the original assertion or invariant `FailureOrigin`.
- Added discardable normal-action spans under a dedicated step-count span for shared middle-action deletion.
- Added safe invalidation of shrink candidates whose dependencies were deleted.
- Added executable-style scenario reproduction with stable result names to console and JSON reporting.
- Added stateful database identity, reuse, persistence, and stale-example cleanup through the Phase 08 database stack.
- Migrated `Stateful`, `StateMachine`, and all stateful attributes out of `_internal/legacy_adapter.ph`; compatibility aliases now delegate.
- Added fourteen stateful fixtures, a real database example, Phase 09 design documentation, and an eight-lane verifier.
- Updated inherited verifiers only where later stateful ownership or checkpoint versioning superseded earlier assumptions.

## Checkpoint 08 — Memory and persistent example databases

- Added immutable typed `DatabaseKey` values containing package, module, suite, selector, ordered strategy fingerprint, and engine format version.
- Added deterministic canonical key encoding and collision-checked filename digests.
- Added the authoritative typed `ExampleDatabase` protocol and immutable private database record model.
- Added bounded, deduplicating `MemoryDatabase` storage with copied fetch results.
- Added a versioned semantic `ExampleCodec` covering all choice variants, spans, generation size, and optional source-aware failure origins.
- Added codec checksum, key, length, count, choice-contract, span-range, parent-containment, cycle, and trailing-data validation.
- Added bounded `DirectoryDatabase` persistence with temporary writes, flush, close, atomic replacement, corruption quarantine, and fail-soft I/O behavior.
- Added process-to-process reuse and safe stale-entry invalidation through `PropertyRunner`.
- Prevented explicit examples from being persisted as generated counterexamples.
- Replaced compatibility-adapter database ownership and the Phase 01 directory placeholder with authoritative database modules.
- Added eight Phase 08 database fixtures and an eight-lane structural verifier.
- Updated inherited verifiers only where database types, ownership, or later checkpoint markers superseded earlier assumptions.

## Checkpoint 07 — Event reporting, statistics, notes, targeting policy, and reproduction

- Added sealed immutable `ReportEvent` values for suite, property, phase, example, failure, shrink, health, and completion events.
- Added the structural `Reporter` protocol with null, recording, and composite implementations.
- Added reporter-aware `PropertyRunner`, `SearchEngine`, and `Shrinker` overloads while preserving silent compatibility calls.
- Added deterministic suite/property lifecycle ordering and phase/example progress events.
- Implemented `Property.note`, `Property.event`, and `Property.classify` as active context operations.
- Captured notes from the final minimal failing context in immutable `Failure` values.
- Aggregated event and classification counts in immutable `Statistics` snapshots.
- Added distinct console rendering for pass, counterexample, inconclusive, health-check, flaky, and engine-error outcomes.
- Added named counterexample arguments, notes, observations, statistics, and reproduction identifiers to console output.
- Added schema-versioned deterministic JSON-lines reporting with string escaping.
- Added first-class immutable `ReproductionToken` values and authoritative reuse-only replay.
- Removed public `Property.target` rather than retaining a score-recording no-op; `Phase.Target` remains reserved and disabled by default.
- Removed compatibility-owned console reporting, `PropertyReporter`, and Phase 01 reporter/JSON placeholders.
- Expanded the root façade with report events, reporter implementations, reproduction, choice/example, and property-run result values.
- Added seven Phase 07 reporting fixtures, five golden files, the Phase 07 verifier, design documentation, and an implementation plan.
- Updated inherited verifiers only where reporting ownership or checkpoint version advanced.

## Checkpoint 06 — Property attributes, reflective inference, builder API, and runner

- Added passive `Given`, `GivenArgs`, `GivenMode`, `Case`, and `WithSettings` metadata.
- Added bare, explicit, and named-partial `@Given` resolution modes.
- Added reflected parameter names, positions, and type annotations through `_ReflectedParameter`.
- Extended `StrategyRegistry` with recursive `Option`, `List`, tuple, set, map, and result inference.
- Added precise public `StrategyResolutionError` and `PropertyDiscoveryError` diagnostics.
- Added method and block invocation targets for the shared search engine.
- Added source-aware `PropertyAssertionError` and caller-derived `FailureOrigin` values.
- Added `Property.given(...).using(...).check { ... }`, plus shared-kernel `forAll` and `find` delegation.
- Added authoritative `PropertyId`, `PropertyDefinition`, `PropertyDiscovery`, `PropertyRun`, `PropertySuiteResult`, and `PropertyRunner`.
- Added reflected named arguments and explicit-case failure identification.
- Added canonical pass/fail suite summary lines.
- Removed compatibility-owned property attributes, assertions, builders, discovery, and runner behavior.
- Extended the stable root façade with `GivenArgs` and public property diagnostics.
- Added nine Phase 06 property fixtures, the Phase 06 verifier, design notes, and an implementation plan.
- Updated inherited verifiers only where public ownership or checkpoint version advanced.

## Checkpoint 05 — Search engine and structural shrinker

- Added immutable `PropertySpec<T...>` and private find specifications.
- Added the authoritative `SearchEngine` and complete-property `_Evaluator`.
- Added explicit, reuse, generation, structural shrink, and final replay verification phases.
- Added deterministic `ExampleComplexity` ordering and private ordering/fingerprint helpers.
- Added seven ordered structural shrink passes.
- Added immutable `Example.deleteRange` with span contraction, removal, and later-range shifting.
- Added middle list/set/map/text deletion with enclosing length-choice adjustment.
- Added discardable recursive payload spans and recursive subtree collapse.
- Added strict failure-origin preservation and final flaky classification.
- Added value-based `find` over the same engine and shrinker, removing `_LegacyFoundExample`.
- Removed `_LegacyEngine`, `_LegacyPropertySpec`, `_LegacyFailureSignature`, and the greedy compatibility shrinker.
- Delegated explicit properties, reflective runner calls, temporary database reuse, and stateful checks to the authoritative engine.
- Added ten Phase 05 engine fixtures, a structural-shrinking example, and the Phase 05 verifier.
- Updated inherited verifiers for the new ownership boundary without weakening their original contracts.

## Checkpoint 04 — Typed strategy protocol and standard strategies

- Added the public covariant `Strategy<out T>` structural protocol.
- Added `_StrategyBase` with ordinary `map`, `filter`, `flatMap`, and `named` combinators.
- Added typed integer, Boolean, finite quantized float, bytes, text, sampled, and constant strategies.
- Added `oneOf`, option, result, list, set, map, and tuple strategies.
- Added explicit `Draw`-based composite generation through `Gen.build`.
- Added deferred and generation-size-bounded recursive strategies.
- Added scoped default labels and scoped generation sizes to `DrawData`, both with `ensure` cleanup.
- Added filter and uniqueness rejection accounting and invalid-example classification.
- Added semantic strategy, collection, text, tuple, entry, and element spans; list/set elements and map entries are discardable.
- Added exact built-in `StrategyRegistry` entries for `Int`, `Bool`, `Float`, `Bytes`, and `String`.
- Replaced compatibility-adapter strategy ownership with authoritative strategy-module imports.
- Migrated the compatibility stateful scenario strategy onto `_StrategyBase`.
- Added seven Phase 04 strategy fixtures, including generate/replay coverage for every standard strategy.
- Added the Phase 04 structural verifier and updated inherited verifiers to accept later ownership/version checkpoints without weakening their semantic assertions.

## Checkpoint 03 — Typed choices, spans, examples, providers, and replay

- Added sealed immutable `Choice` variants for integer, Boolean, index, and bytes decisions.
- Added sealed immutable `ChoiceRequest` variants with bounds, size, and shrink-target contracts.
- Added immutable `Span` values with stable identifiers, parent relationships, half-open ranges, and discardability.
- Added mutable `ChoiceBuffer` construction with `ensure`-guaranteed nested-span closure.
- Added immutable `Example` snapshots with copied choice/span containers, stable source-order spans, compatibility replacement/prefix operations, and deterministic signatures.
- Added byte copying at normal choice/request API boundaries.
- Added the `ChoiceProvider` protocol, random provider, and deterministic replay provider.
- Added typed `DrawData` primitive methods and generation/replay factories.
- Added replay normalization against current request metadata.
- Classified replay exhaustion, invalid replay, choice-budget exhaustion, and unclosed spans as engine overruns rather than counterexamples.
- Replaced compatibility-adapter choice, tape, generation-data, and replay-data ownership with the authoritative choice modules.
- Corrected the inherited `Property.current` context-stack reference.
- Added five Phase 03 choice/replay tests and a Phase 03 structural verifier.
- Updated the Phase 02 verifier so later package versions preserve its regression gate.

## Checkpoint 02 — Typed immutable core model

- Added immutable `Settings` with exact defaults, fluent copy updates, and contracts.
- Added sealed `Phase`, `ExampleStatus`, and `PropertyResult` families.
- Added source-aware `FailureOrigin` and immutable `Failure` values.
- Added immutable `Statistics` snapshots and a private mutable collector.
- Moved property context state into private core workers with `ensure`-guaranteed cleanup.
- Replaced compatible configuration, evaluation, result, statistics, and context state in the legacy adapter.
- Classified replay overrun and health-check failure as engine errors rather than counterexamples.
- Classified discard exhaustion and rejected explicit cases as inconclusive.
- Classified flaky reproduction as an execution error.
- Updated the compatibility reporter to render passed, falsified, inconclusive, and errored results distinctly.
- Added core tests for defaults, immutable updates, contracts, exhaustive variant dispatch, source-aware failure identity, and context cleanup.
- Added a Phase 02 structural verifier while preserving the Phase 01 regression gate.

## Checkpoint 01 — Project scaffold and syntax migration

- Added `phalcom.toml`, the complete target module tree, tests, examples, documentation, and MIT license.
- Added the stable `src/hypothesis.ph` façade with all approved public names.
- Added typed Phase 01 declarations for APIs owned by later phases.
- Migrated the broad-v1 prototype into the private compatibility adapter.
- Replaced the retired constructor form with `@constructor` in active source.
- Removed mandatory outer control-flow parentheses in active source.
- Replaced mechanical increment assignments with compound updates.
- Added reflective type annotations to public compatibility declarations.
- Added integration fixtures for package imports and inherited broad-v1 behavior.
- Added an observed structural verifier for environments without a Phalcom executable.

## Checkpoint 00 — Planning baseline

- Preserved the original monolithic Hypothesis draft.
- Recorded the approved multi-file architecture.
- Added the complete twelve-phase implementation plan.
- Added the full-package checkpoint delivery protocol.

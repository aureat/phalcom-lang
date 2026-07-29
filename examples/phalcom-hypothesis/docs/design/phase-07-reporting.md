# Phase 07 Design — Reporting, Observations, and Reproduction

## Objective

Phase 07 makes every public observation visible while keeping the search kernel independent from output formatting. The engine emits immutable typed events. Reporters consume those events and decide whether to print, serialize, retain, combine, or ignore them.

## Event boundary

`ReportEvent` is a sealed immutable family covering suite and property lifecycle, phase entry, accepted and rejected examples, discovered failures, accepted shrinks, health-check failures, and completion. The engine and runner depend only on this data model and the one-method `Reporter` protocol:

```phalcom
protocol Reporter {
  handle(event: ReportEvent) -> None
}
```

`NullReporter` preserves silent behavior for existing callers. `RecordingReporter` supports tests and integrations. `CompositeReporter` fans one ordered event stream out to multiple consumers.

`PropertyRunner` owns suite/property start and finish events. `SearchEngine` owns phase and example events. `Shrinker` emits an event only after accepting a strictly smaller candidate. This division prevents duplicate lifecycle ownership.

## Notes and observations

`Property.note(value)` appends to the current example context. Only the final minimal failure copies its context notes into `Failure`; notes from passing, rejected, or superseded failing examples are not rendered as if they belonged to the counterexample.

`Property.event(label)` increments an event count. `Property.classify(condition, as:)` records the label only when the condition is true. Counts from valid, rejected, and initially failing examples are aggregated in `Statistics` and appear in console and JSON summaries.

## Targeting decision

Phase 07 does not expose `Property.target`. The current engine has no evidence-backed target optimizer, score corpus, Pareto policy, or target-phase acceptance tests. Retaining a public method that only stores scores would be a no-op and would violate the checkpoint acceptance criterion.

`Phase.Target` remains reserved in the phase algebra so a future real optimizer can be introduced without reopening the ordered phase model. It is not enabled by `Settings.standard`.

## Result classification

The core `PropertyResult` variants remain stable. Reporters classify `Errored` values by error type:

- `_HealthCheckFailure` renders as `HEALTH CHECK`;
- `_FlakyFailure` renders as `FLAKY`;
- other errors render as `ERROR`;
- falsifying examples alone render `Falsifying example` and may carry reproduction data.

This keeps health and flaky outcomes distinct from ordinary counterexamples without multiplying result variants solely for presentation.

## Console output

`ConsoleReporter` renders deterministic lines from `PropertyFinished` and `SuiteFinished` events. Failure output includes named arguments, failure-local notes, observation counts, source-aware error information, statistics, and a reproduction token when the failure came from generated or reused choices.

Explicit-case failures deliberately omit reproduction tokens because they have no generated semantic example to replay.

## JSON schema

`JsonReporter` emits one JSON object per report event with `schemaVersion: 1` and a stable snake-case `type`. Field construction and key ordering are deterministic. Strings are escaped for backslash, quote, newline, carriage return, and tab.

The JSON event stream is intentionally event-oriented rather than a second result model. Consumers can reconstruct progress or retain only completion records.

## Reproduction tokens

`ReproductionToken` is a first-class immutable value containing:

- the property identity;
- the exact immutable `Example`, including choices, spans, and generation size;
- the original immutable `Settings`;
- a stable human-facing token string.

`Reproduction.replay` uses the authoritative `SearchEngine` and exact recorded example. It restricts phases to reuse and disables further shrinking so replay cannot generate or replace the counterexample.

The text form is a stable diagnostic identifier, not yet a process-independent serialized example codec. Cross-process persistence and durable binary/text encoding belong to Phase 08.

## Compatibility migration

The compatibility adapter no longer owns console reporting or the `PropertyReporter` namespace. The root façade exports the authoritative reporting modules. `PropertyReporter.console` remains as a compatibility factory implemented by `src/reporting/console.ph`.

The Phase 01 reporter and JSON placeholders were removed rather than retained beside the real implementations.

## Verification boundary

Phase 07 adds focused Phalcom fixtures and golden files for ordering, notes, observations, health/flaky classification, JSON schema, reproduction, and targeting removal. In environments without a `phalcom` executable, Python verifiers observe source contracts, ownership, imports, privacy, and golden presence. Runtime behavior remains unobserved until a real toolchain is available.

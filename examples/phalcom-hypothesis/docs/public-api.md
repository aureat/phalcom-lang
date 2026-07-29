# Public API

Hypothesis for Phalcom 0.1.0 exposes the following names from the root `hypothesis` module. This file is the normative export inventory used by the release verifier.

## Property metadata and execution

- `Given`
- `GivenArgs`
- `Case`
- `WithSettings`
- `Settings`
- `Phase`
- `Property`
- `PropertyBuilder`
- `PropertySuite`
- `Assert`
- `PropertyAssertionError`
- `PropertyRunner`
- `PropertyRun`
- `PropertySuiteResult`
- `PropertyResult`
- `PropertyId`
- `Failure`
- `Statistics`
- `PropertyDiscoveryError`
- `HealthCheckFailure`
- `FlakyFailure`

## Choices, strategies, and shrinking

- `Choice`
- `ChoiceRequest`
- `Example`
- `DrawData`
- `ChoiceProvider`
- `ChoiceProviderFactory`
- `SystemRandomChoiceProvider`
- `ScriptedChoiceProvider`
- `SystemRandomProviderFactory`
- `ScriptedProviderFactory`
- `Strategy`
- `StrategyBase`
- `Gen`
- `StrategyRegistry`
- `arbitrary`
- `strategy`
- `StrategyResolutionError`
- `ShrinkPass`
- `Shrinker`

## Reporting and reproduction

- `ReportEvent`
- `Reporter`
- `NullReporter`
- `RecordingReporter`
- `CompositeReporter`
- `ConsoleReporter`
- `PropertyReporter`
- `JsonReporter`
- `ReporterFailure`
- `ReproductionToken`
- `Reproduction`

## Persistence

- `DatabaseKey`
- `ExampleDatabase`
- `MemoryDatabase`
- `DirectoryDatabase`

## Stateful testing

- `StateMachine`
- `Stateful`
- `Rule`
- `Initialize`
- `StateInvariant`
- `When`
- `Teardown`
- `Bundle`

## Compatibility aliases

These names are direct aliases to the authoritative release objects. They do not route through a compatibility adapter.

- `Check`
- `CheckConfig`
- `RuleBasedStateMachine`
- `Invariant`

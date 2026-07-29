// Core error taxonomy shared by the package.
//
// These classes identify engine/configuration failures. Counterexamples are
// represented by Failure and PropertyResult values, not by string tags.

class _HypothesisError is Error {}
class _InvalidSettings is _HypothesisError {}
class _MissingPropertyContext is _HypothesisError {}
class _PropertyContextUnderflow is _HypothesisError {}
class _InconclusiveProperty is _HypothesisError {}
class _EngineOverrun is _HypothesisError {}

// Choice providers signal invalid or exhausted replay as engine overruns.
// These are search-control outcomes, never falsifying examples.
class _ChoiceOverrun is _EngineOverrun {}
class _ReplayExhausted is _ChoiceOverrun {}
class _ScriptedProviderExhausted is _ChoiceOverrun {}
class _InvalidReplayChoice is _ChoiceOverrun {}
class _ChoiceBudgetExceeded is _ChoiceOverrun {}
class _UnclosedSpan is _ChoiceOverrun {}
// Strategy construction and filtering failures. InvalidStrategy is a
// programmer-facing configuration error; RejectedExample is normal search
// control and must never be reported as a counterexample.
class _StrategyError is _HypothesisError {}
class _InvalidStrategy is _StrategyError {}
class StrategyResolutionError is _InvalidStrategy {}
class PropertyDiscoveryError is _HypothesisError {}

// Compatibility alias retained for inherited Phase 04 callers.
const _StrategyResolutionError = StrategyResolutionError
class _RejectedExample is _HypothesisError {}


// Phase 05 search-engine outcomes which are neither counterexamples nor
// strategy construction errors.
class _UnsatisfiedAssumptions is _HypothesisError {}
class _FlakyFailure is _HypothesisError {}
class _NoSuchExample is _HypothesisError {}
class _HealthCheckFailure is _HypothesisError {}
@data
@immutable
class ReporterFailure is _HypothesisError {
  const _cause: Error

  @class
  from(cause: Error) -> ReporterFailure {
    return ReporterFailure.new(cause: cause)
  }

  message -> Option<String> {
    return Some.new(
      "reporter extension failure: " +
      _cause.message.unwrapOr(_cause.toString)
    )
  }
}

// Phase 09 stateful configuration and replay outcomes.
class _StatefulDiscoveryError is _HypothesisError {}
class _InvalidStatefulReplay is _EngineOverrun {}
class _DatabaseLockUnavailable is _HypothesisError {}

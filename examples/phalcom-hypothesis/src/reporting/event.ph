// Immutable data-only events emitted by the runner and search engine.
// Formatting belongs to Reporter implementations, never to the engine.

@data
@immutable
@sealed
class ReportEvent {
  @variant SuiteStarted(total:)
  @variant PropertyStarted(id:)
  @variant PhaseStarted(id:, phase:)
  @variant ExampleAccepted(id:, index:, example:, arguments:, context:)
  @variant ExampleRejected(id:, index:, reason:, example:, arguments:, context:)
  @variant FailureFound(id:, failure:)
  @variant ShrinkAccepted(id:, before:, after:)
  @variant HealthCheckFailed(id:, error:)
  @variant PropertyFinished(run:)
  @variant SuiteFinished(result:)

  @class
  suiteStarted(total: Int) -> ReportEvent {
    return SuiteStarted.new(total: total)
  }

  @class
  propertyStarted(id: Any) -> ReportEvent {
    return PropertyStarted.new(id: id)
  }

  @class
  phaseStarted(id: Any, phase: Any) -> ReportEvent {
    return PhaseStarted.new(id: id, phase: phase)
  }

  @class
  exampleAccepted(
    id: Any,
    index: Int,
    example: Any,
    arguments: List<Any>,
    context: Any
  ) -> ReportEvent {
    return ExampleAccepted.new(
      id: id,
      index: index,
      example: example,
      arguments: arguments,
      context: context
    )
  }

  @class
  exampleRejected(
    id: Any,
    index: Int,
    reason: Any,
    example: Any,
    arguments: List<Any>,
    context: Any
  ) -> ReportEvent {
    return ExampleRejected.new(
      id: id,
      index: index,
      reason: reason,
      example: example,
      arguments: arguments,
      context: context
    )
  }

  @class
  failureFound(id: Any, failure: Any) -> ReportEvent {
    return FailureFound.new(id: id, failure: failure)
  }

  @class
  shrinkAccepted(id: Any, before: Any, after: Any) -> ReportEvent {
    return ShrinkAccepted.new(id: id, before: before, after: after)
  }

  @class
  healthCheckFailed(id: Any, error: Error) -> ReportEvent {
    return HealthCheckFailed.new(id: id, error: error)
  }

  @class
  propertyFinished(run: Any) -> ReportEvent {
    return PropertyFinished.new(run: run)
  }

  @class
  suiteFinished(result: Any) -> ReportEvent {
    return SuiteFinished.new(result: result)
  }

  isSuiteStarted -> Bool {
    return self.match(
      suiteStarted: { _ => true },
      propertyStarted: { _ => false },
      phaseStarted: { _ => false },
      exampleAccepted: { _ => false },
      exampleRejected: { _ => false },
      failureFound: { _ => false },
      shrinkAccepted: { _ => false },
      healthCheckFailed: { _ => false },
      propertyFinished: { _ => false },
      suiteFinished: { _ => false }
    )
  }

  isPropertyStarted -> Bool => self.is(#propertyStarted)
  isPhaseStarted -> Bool => self.is(#phaseStarted)
  isExampleAccepted -> Bool => self.is(#exampleAccepted)
  isExampleRejected -> Bool => self.is(#exampleRejected)
  isFailureFound -> Bool => self.is(#failureFound)
  isShrinkAccepted -> Bool => self.is(#shrinkAccepted)
  isHealthCheckFailed -> Bool => self.is(#healthCheckFailed)
  isPropertyFinished -> Bool => self.is(#propertyFinished)
  isSuiteFinished -> Bool => self.is(#suiteFinished)

  @private
  is(expected: Symbol) -> Bool {
    return self.match(
      suiteStarted: { _ => expected == #suiteStarted },
      propertyStarted: { _ => expected == #propertyStarted },
      phaseStarted: { _ => expected == #phaseStarted },
      exampleAccepted: { _ => expected == #exampleAccepted },
      exampleRejected: { _ => expected == #exampleRejected },
      failureFound: { _ => expected == #failureFound },
      shrinkAccepted: { _ => expected == #shrinkAccepted },
      healthCheckFailed: { _ => expected == #healthCheckFailed },
      propertyFinished: { _ => expected == #propertyFinished },
      suiteFinished: { _ => expected == #suiteFinished }
    )
  }
}

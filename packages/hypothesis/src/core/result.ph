import .failure as Failures
import .statistics as StatisticsModule
import ..choices.example as Examples

/// The outcome returned by `Property.check`.
data PropertyResult<T> {
  /// The property held for every generated example.
  Passed(
    _ value: T,
    _ statistics: StatisticsModule.Statistics
  ),

  /// A stable failure was reproduced and reduced to this example.
  Falsified(
    _ value: T,
    _ failure: Failures.Failure,
    _ example: Examples.Example,
    _ statistics: StatisticsModule.Statistics
  ),

  /// Repeated replay did not produce a stable outcome.
  Flaky(
    _ value: T,
    _ example: Examples.Example,
    _ statistics: StatisticsModule.Statistics
  ),

  /// The search could not evaluate a usable example within its limits.
  GaveUp(
    _ reason: String,
    _ statistics: StatisticsModule.Statistics
  ),
}

export PropertyResult

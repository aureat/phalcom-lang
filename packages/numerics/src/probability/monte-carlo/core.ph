import universe.math as math
from universe.math import NumericalRangeError
from universe.random import Distribution, Generator
from ..errors import UndefinedStatisticError
from ...statistics.moments import RunningMoments, MomentSummary

const maxMonteCarloCount: Int = 9007199254740991

/// Immutable diagnostic estimate over a fixed number of iid callback values.
data MonteCarloEstimate private(
  _ count: Int,
  _ mean: Float,
  _ sampleVariance: Float,
  _ standardError: Float,
  _ summary: MomentSummary
)

impl MonteCarloEstimate {
  @class
  internal fromSummary(_ count: Int, _ summary: MomentSummary) -> MonteCarloEstimate {
    let sampleVariance = summary.sampleVariance.match(
      some: |value| { value },
      none: || { UndefinedStatisticError.new("Monte Carlo estimate requires at least two values").raise() }
    )
    let mean = summary.mean.match(
      some: |value| { value },
      none: || { UndefinedStatisticError.new("Monte Carlo estimate requires an observed value").raise() }
    )
    let standardError = math.sqrt(sampleVariance / Float.new(count))
    if not standardError.isFinite {
      NumericalRangeError.new("Monte Carlo standard error is outside Float range").raise()
    }
    MonteCarloEstimate(count, mean, sampleVariance, standardError, summary)
  }
}

class MonteCarlo {
  @class
  expectation<D, G>(_ distribution: D, count n: Int, using generator: G,
      evaluate f: (<D as Distribution>::Output) -> Float) -> MonteCarloEstimate
    where D |= Distribution, G |= Generator
  {
    if n < 2 { ArgumentError.new("Monte Carlo expectation requires at least two samples").raise() }
    if n > maxMonteCarloCount {
      NumericalRangeError.new("Monte Carlo sample count exceeds exact Float count range").raise()
    }
    let moments = RunningMoments.new()
    let mut remaining = n
    while remaining > 0 {
      let sample = distribution.sample(using: generator)
      let value = f.call(sample)
      if not value.isFinite {
        ArgumentError.new("Monte Carlo callback must return finite Float values").raise()
      }
      moments.push(value)
      remaining = remaining - 1
    }
    let summary = moments.snapshot
    return MonteCarloEstimate.fromSummary(n, summary)
  }
}

export MonteCarlo, MonteCarloEstimate

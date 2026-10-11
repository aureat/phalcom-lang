import universe.math as math
from universe.math import NumericalRangeError
from universe.collections.iterator_laws import Iterable

const maxApproximateCount: Int = 9007199254740991

/// Immutable, mergeable summary. Raw scaled state is not publicly constructible.
data MomentSummary private(
  _ count: Int,
  _ scale: Float,
  _ scaledMean: Float,
  _ scaledM2: Float,
  _ minimumValue: Float,
  _ maximumValue: Float
)

impl MomentSummary {
  /// Constructs a summary from the validated internal scaled state.
  @class
  internal fromScaledState(
    _ count: Int, _ scale: Float, _ scaledMean: Float, _ scaledM2: Float,
    _ minimumValue: Float, _ maximumValue: Float
  ) -> MomentSummary {
    MomentSummary(count, scale, scaledMean, scaledM2, minimumValue, maximumValue)
  }

  mean -> Option<Float> {
    if self.count == 0 { return Option<Float>::None }
    return Option<Float>::Some(self.scale * self.scaledMean)
  }

  minimum -> Option<Float> {
    if self.count == 0 { return Option<Float>::None }
    return Option<Float>::Some(self.minimumValue)
  }

  maximum -> Option<Float> {
    if self.count == 0 { return Option<Float>::None }
    return Option<Float>::Some(self.maximumValue)
  }

  populationVariance -> Option<Float> {
    if self.count == 0 { return Option<Float>::None }
    let fraction = self.scaledM2 / Float.new(self.count)
    let partial = self.scale * fraction
    let result = partial * self.scale
    if not result.isFinite or (fraction > 0.0 and result == 0.0) {
      NumericalRangeError.new("population variance is outside the finite Float range").raise()
    }
    return Option<Float>::Some(result)
  }

  sampleVariance -> Option<Float> {
    if self.count < 2 { return Option<Float>::None }
    let fraction = self.scaledM2 / Float.new(self.count - 1)
    let partial = self.scale * fraction
    let result = partial * self.scale
    if not result.isFinite or (fraction > 0.0 and result == 0.0) {
      NumericalRangeError.new("sample variance is outside the finite Float range").raise()
    }
    return Option<Float>::Some(result)
  }

  populationStddev -> Option<Float> {
    let result: Option<Float> = match self.populationVariance {
      Option::Some(value) => Option<Float>::Some(math.sqrt(value))
      Option::None => Option<Float>::None
    }
    return result
  }

  sampleStddev -> Option<Float> {
    let result: Option<Float> = match self.sampleVariance {
      Option::Some(value) => Option<Float>::Some(math.sqrt(value))
      Option::None => Option<Float>::None
    }
    return result
  }

  merged(_ other: MomentSummary) -> MomentSummary {
    if self.count == 0 { return other }
    if other.count == 0 { return self }
    if self.count > maxApproximateCount - other.count {
      NumericalRangeError.new("merged observation count exceeds the exact Float count range").raise()
    }

    let combinedCount = self.count + other.count
    let combinedScale = if self.scale >= other.scale { self.scale } else { other.scale }
    let leftRatio = if combinedScale == 0.0 { 0.0 } else { self.scale / combinedScale }
    let rightRatio = if combinedScale == 0.0 { 0.0 } else { other.scale / combinedScale }
    let leftMean = self.scaledMean * leftRatio
    let rightMean = other.scaledMean * rightRatio
    let leftM2 = self.scaledM2 * (leftRatio * leftRatio)
    let rightM2 = other.scaledM2 * (rightRatio * rightRatio)
    let delta = rightMean - leftMean
    let rightShare = Float.new(other.count) / Float.new(combinedCount)
    let countFactor = Float.new(self.count) * rightShare
    let combinedMean = leftMean + delta * rightShare
    let combinedM2 = leftM2 + rightM2 + (delta * delta) * countFactor
    let low = if self.minimumValue <= other.minimumValue { self.minimumValue } else { other.minimumValue }
    let high = if self.maximumValue >= other.maximumValue { self.maximumValue } else { other.maximumValue }
    return MomentSummary.fromScaledState(
      combinedCount, combinedScale, combinedMean, combinedM2, low, high
    )
  }
}

/// Mutable online moment accumulator; snapshots are immutable values.
class RunningMoments {
  mut _count
  mut _scale
  mut _scaledMean
  mut _scaledM2
  mut _minimumValue
  mut _maximumValue

  @constructor
  new() {
    _count = 0
    _scale = 0.0
    _scaledMean = 0.0
    _scaledM2 = 0.0
    _minimumValue = 0.0
    _maximumValue = 0.0
  }

  count -> Int { _count }

  push(_ x: Float) -> Unit {
    if not x.isFinite { ArgumentError.new("statistics require finite Float observations").raise() }
    if _count >= maxApproximateCount {
      NumericalRangeError.new("observation count exceeds the exact Float count range").raise()
    }
    let newCount = _count + 1
    let magnitude = x.abs
    let newScale = if magnitude > _scale { magnitude } else { _scale }
    let ratio = if newScale == 0.0 { 0.0 } else { _scale / newScale }
    let previousMean = _scaledMean * ratio
    let previousM2 = _scaledM2 * (ratio * ratio)
    let normalized = if newScale == 0.0 { 0.0 } else { x / newScale }
    let delta = normalized - previousMean
    let newMean = previousMean + delta / Float.new(newCount)
    let deltaAfterMean = normalized - newMean
    let newM2 = previousM2 + delta * deltaAfterMean
    let newMinimum = if _count == 0 or x < _minimumValue { x } else { _minimumValue }
    let newMaximum = if _count == 0 or x > _maximumValue { x } else { _maximumValue }

    // Commit only after validation and all calculations have completed.
    _count = newCount
    _scale = newScale
    _scaledMean = newMean
    _scaledM2 = newM2
    _minimumValue = newMinimum
    _maximumValue = newMaximum
    return ()
  }

  merge(_ other: MomentSummary) -> Unit {
    let merged = self.snapshot.merged(other)
    _count = merged.count
    _scale = merged.scale
    _scaledMean = merged.scaledMean
    _scaledM2 = merged.scaledM2
    _minimumValue = merged.minimumValue
    _maximumValue = merged.maximumValue
    return ()
  }

  snapshot -> MomentSummary {
    MomentSummary.fromScaledState(
      _count, _scale, _scaledMean, _scaledM2, _minimumValue, _maximumValue
    )
  }
}

/// Source-owned scaled Welford update for one finite Float stream.
class MomentKernel {
  @class
  from<I>(_ values: I) -> MomentSummary
    where I |= Iterable, <I as Iterable>::Item == Float
  {
    let mut iterator = values.iterator
    let mut count = 0
    let mut scale = 0.0
    let mut mean = 0.0
    let mut m2 = 0.0
    let mut minimumValue = 0.0
    let mut maximumValue = 0.0
    let mut step = iterator.next
    while step.isSome {
      let x: Float = step.match(some: |value| { value }, none: || { 0.0 })
      if not x.isFinite {
        ArgumentError.new("statistics require finite Float observations").raise()
      }
      if count >= maxApproximateCount {
        NumericalRangeError.new("observation count exceeds the exact Float count range").raise()
      }
      count = count + 1

      if count == 1 {
        minimumValue = x
        maximumValue = x
      } else {
        if x < minimumValue { minimumValue = x }
        if x > maximumValue { maximumValue = x }
      }

      let magnitude = x.abs
      if magnitude > scale {
        if scale != 0.0 {
          let ratio = scale / magnitude
          mean = mean * ratio
          m2 = m2 * (ratio * ratio)
        }
        scale = magnitude
      }

      let normalized = if scale == 0.0 { 0.0 } else { x / scale }
      let delta = normalized - mean
      mean = mean + delta / Float.new(count)
      let deltaAfterMean = normalized - mean
      m2 = m2 + delta * deltaAfterMean
      step = iterator.next
    }
    return MomentSummary.fromScaledState(count, scale, mean, m2, minimumValue, maximumValue)
  }
}

export MomentSummary, MomentKernel, RunningMoments

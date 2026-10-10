import universe.math as math
from universe.math import NumericalRangeError
from universe.collections.iterator_laws import Iterable

const maxApproximateCount: Int = 9007199254740991

/// Immutable paired central-moment summary with merge support.
data CovarianceSummary private(
  _ count: Int,
  _ scaleX: Float,
  _ meanXScaled: Float,
  _ m2XScaled: Float,
  _ scaleY: Float,
  _ meanYScaled: Float,
  _ m2YScaled: Float,
  _ coMomentScaled: Float
)

impl CovarianceSummary {
  /// Constructs a summary from the validated internal scaled state.
  @class
  internal fromScaledState(
    _ count: Int,
    _ scaleX: Float, _ meanXScaled: Float, _ m2XScaled: Float,
    _ scaleY: Float, _ meanYScaled: Float, _ m2YScaled: Float,
    _ coMomentScaled: Float
  ) -> CovarianceSummary {
    CovarianceSummary(
      count, scaleX, meanXScaled, m2XScaled, scaleY, meanYScaled,
      m2YScaled, coMomentScaled
    )
  }

  @class
  private restoreProduct(_ scaleX: Float, _ scaleY: Float, _ fraction: Float) -> Float {
    let magnitude = fraction.abs
    let result = if magnitude >= 1.0 {
      if scaleX <= scaleY { (scaleX * fraction) * scaleY } else { (scaleY * fraction) * scaleX }
    } else {
      if scaleX >= scaleY { (scaleX * fraction) * scaleY } else { (scaleY * fraction) * scaleX }
    }
    if not result.isFinite or (fraction != 0.0 and result == 0.0) {
      NumericalRangeError.new("covariance is outside the finite Float range").raise()
    }
    return result
  }

  meanX -> Option<Float> {
    if self.count == 0 { return Option<Float>::None }
    return Option<Float>::Some(self.scaleX * self.meanXScaled)
  }

  meanY -> Option<Float> {
    if self.count == 0 { return Option<Float>::None }
    return Option<Float>::Some(self.scaleY * self.meanYScaled)
  }

  populationCovariance -> Option<Float> {
    if self.count == 0 { return Option<Float>::None }
    return Option<Float>::Some(CovarianceSummary.restoreProduct(
      self.scaleX, self.scaleY, self.coMomentScaled / Float.new(self.count)
    ))
  }

  sampleCovariance -> Option<Float> {
    if self.count < 2 { return Option<Float>::None }
    return Option<Float>::Some(CovarianceSummary.restoreProduct(
      self.scaleX, self.scaleY, self.coMomentScaled / Float.new(self.count - 1)
    ))
  }

  correlation -> Option<Float> {
    if self.count < 2 or self.m2XScaled == 0.0 or self.m2YScaled == 0.0 {
      return Option<Float>::None
    }
    let denominator = math.sqrt(self.m2XScaled) * math.sqrt(self.m2YScaled)
    return Option<Float>::Some(self.coMomentScaled / denominator)
  }

  merged(_ other: CovarianceSummary) -> CovarianceSummary {
    if self.count == 0 { return other }
    if other.count == 0 { return self }
    if self.count > maxApproximateCount - other.count {
      NumericalRangeError.new("merged pair count exceeds the exact Float count range").raise()
    }

    let combinedCount = self.count + other.count
    let combinedScaleX = if self.scaleX >= other.scaleX { self.scaleX } else { other.scaleX }
    let combinedScaleY = if self.scaleY >= other.scaleY { self.scaleY } else { other.scaleY }
    let leftXRatio = if combinedScaleX == 0.0 { 0.0 } else { self.scaleX / combinedScaleX }
    let rightXRatio = if combinedScaleX == 0.0 { 0.0 } else { other.scaleX / combinedScaleX }
    let leftYRatio = if combinedScaleY == 0.0 { 0.0 } else { self.scaleY / combinedScaleY }
    let rightYRatio = if combinedScaleY == 0.0 { 0.0 } else { other.scaleY / combinedScaleY }
    let leftMeanX = self.meanXScaled * leftXRatio
    let rightMeanX = other.meanXScaled * rightXRatio
    let leftMeanY = self.meanYScaled * leftYRatio
    let rightMeanY = other.meanYScaled * rightYRatio
    let deltaX = rightMeanX - leftMeanX
    let deltaY = rightMeanY - leftMeanY
    let rightShare = Float.new(other.count) / Float.new(combinedCount)
    let countFactor = Float.new(self.count) * rightShare
    let combinedMeanX = leftMeanX + deltaX * rightShare
    let combinedMeanY = leftMeanY + deltaY * rightShare
    let combinedM2X = self.m2XScaled * (leftXRatio * leftXRatio)
      + other.m2XScaled * (rightXRatio * rightXRatio)
      + (deltaX * deltaX) * countFactor
    let combinedM2Y = self.m2YScaled * (leftYRatio * leftYRatio)
      + other.m2YScaled * (rightYRatio * rightYRatio)
      + (deltaY * deltaY) * countFactor
    let combinedCo = self.coMomentScaled * leftXRatio * leftYRatio
      + other.coMomentScaled * rightXRatio * rightYRatio
      + deltaX * deltaY * countFactor
    return CovarianceSummary.fromScaledState(
      combinedCount, combinedScaleX, combinedMeanX, combinedM2X,
      combinedScaleY, combinedMeanY, combinedM2Y, combinedCo
    )
  }
}

/// Scaled Welford update over two strict, single-pass Float streams.
class CovarianceKernel {
  @class
  from<I, J>(_ xs: I, with ys: J) -> CovarianceSummary
    where I |= Iterable, J |= Iterable,
      <I as Iterable>::Item == Float, <J as Iterable>::Item == Float
  {
    let mut iteratorX = xs.iterator
    let mut iteratorY = ys.iterator
    let mut count = 0
    let mut scaleX = 0.0
    let mut meanX = 0.0
    let mut m2X = 0.0
    let mut scaleY = 0.0
    let mut meanY = 0.0
    let mut m2Y = 0.0
    let mut coMoment = 0.0
    let mut stepX = iteratorX.next
    let mut stepY = iteratorY.next
    while stepX.isSome and stepY.isSome {
      let x: Float = stepX.match(some: |value| { value }, none: || { 0.0 })
      let y: Float = stepY.match(some: |value| { value }, none: || { 0.0 })
      if not x.isFinite or not y.isFinite {
        ArgumentError.new("covariance requires finite Float pairs").raise()
      }
      if count >= maxApproximateCount {
        NumericalRangeError.new("pair count exceeds the exact Float count range").raise()
      }
      count = count + 1

      if x.abs > scaleX {
        if scaleX != 0.0 {
          let ratioX = scaleX / x.abs
          meanX = meanX * ratioX
          m2X = m2X * (ratioX * ratioX)
          coMoment = coMoment * ratioX
        }
        scaleX = x.abs
      }
      if y.abs > scaleY {
        if scaleY != 0.0 {
          let ratioY = scaleY / y.abs
          meanY = meanY * ratioY
          m2Y = m2Y * (ratioY * ratioY)
          coMoment = coMoment * ratioY
        }
        scaleY = y.abs
      }

      let normalizedX = if scaleX == 0.0 { 0.0 } else { x / scaleX }
      let normalizedY = if scaleY == 0.0 { 0.0 } else { y / scaleY }
      let deltaX = normalizedX - meanX
      let deltaY = normalizedY - meanY
      meanX = meanX + deltaX / Float.new(count)
      meanY = meanY + deltaY / Float.new(count)
      let deltaAfterMeanX = normalizedX - meanX
      let deltaAfterMeanY = normalizedY - meanY
      m2X = m2X + deltaX * deltaAfterMeanX
      m2Y = m2Y + deltaY * deltaAfterMeanY
      coMoment = coMoment + deltaX * deltaAfterMeanY
      stepX = iteratorX.next
      stepY = iteratorY.next
    }
    if stepX.isSome or stepY.isSome {
      ArgumentError.new("covariance requires equal-length input streams").raise()
    }
    return CovarianceSummary.fromScaledState(
      count, scaleX, meanX, m2X, scaleY, meanY, m2Y, coMoment
    )
  }
}

/// Mutable paired online central-moment accumulator.
class RunningCovariance {
  mut _count
  mut _scaleX
  mut _meanXScaled
  mut _m2XScaled
  mut _scaleY
  mut _meanYScaled
  mut _m2YScaled
  mut _coMomentScaled

  @constructor
  new() {
    _count = 0
    _scaleX = 0.0
    _meanXScaled = 0.0
    _m2XScaled = 0.0
    _scaleY = 0.0
    _meanYScaled = 0.0
    _m2YScaled = 0.0
    _coMomentScaled = 0.0
  }

  count -> Int { _count }

  push(_ x: Float, with y: Float) -> Unit {
    if not x.isFinite or not y.isFinite {
      ArgumentError.new("covariance requires finite Float pairs").raise()
    }
    if _count >= maxApproximateCount {
      NumericalRangeError.new("pair count exceeds the exact Float count range").raise()
    }
    let newCount = _count + 1

    let newScaleX = if x.abs > _scaleX { x.abs } else { _scaleX }
    let ratioX = if newScaleX == 0.0 { 0.0 } else { _scaleX / newScaleX }
    let previousMeanX = _meanXScaled * ratioX
    let previousM2X = _m2XScaled * (ratioX * ratioX)

    let newScaleY = if y.abs > _scaleY { y.abs } else { _scaleY }
    let ratioY = if newScaleY == 0.0 { 0.0 } else { _scaleY / newScaleY }
    let previousMeanY = _meanYScaled * ratioY
    let previousM2Y = _m2YScaled * (ratioY * ratioY)
    let previousCo = _coMomentScaled * ratioX * ratioY

    let normalizedX = if newScaleX == 0.0 { 0.0 } else { x / newScaleX }
    let normalizedY = if newScaleY == 0.0 { 0.0 } else { y / newScaleY }
    let deltaX = normalizedX - previousMeanX
    let deltaY = normalizedY - previousMeanY
    let newMeanX = previousMeanX + deltaX / Float.new(newCount)
    let newMeanY = previousMeanY + deltaY / Float.new(newCount)
    let newM2X = previousM2X + deltaX * (normalizedX - newMeanX)
    let newM2Y = previousM2Y + deltaY * (normalizedY - newMeanY)
    let newCo = previousCo + deltaX * (normalizedY - newMeanY)

    _count = newCount
    _scaleX = newScaleX
    _meanXScaled = newMeanX
    _m2XScaled = newM2X
    _scaleY = newScaleY
    _meanYScaled = newMeanY
    _m2YScaled = newM2Y
    _coMomentScaled = newCo
    return ()
  }

  merge(_ other: CovarianceSummary) -> Unit {
    let merged = self.snapshot.merged(other)
    _count = merged.count
    _scaleX = merged.scaleX
    _meanXScaled = merged.meanXScaled
    _m2XScaled = merged.m2XScaled
    _scaleY = merged.scaleY
    _meanYScaled = merged.meanYScaled
    _m2YScaled = merged.m2YScaled
    _coMomentScaled = merged.coMomentScaled
    return ()
  }

  snapshot -> CovarianceSummary {
    CovarianceSummary.fromScaledState(
      _count, _scaleX, _meanXScaled, _m2XScaled,
      _scaleY, _meanYScaled, _m2YScaled, _coMomentScaled
    )
  }
}

export CovarianceKernel, CovarianceSummary, RunningCovariance

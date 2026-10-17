/// Source-owned native kernels for binary64 mathematical operations.
native class FloatMathKernel is Object {
  @class native sqrt(_ value: Float) -> Float
  @class native cbrt(_ value: Float) -> Float
  @class native exp(_ value: Float) -> Float
  @class native exp2(_ value: Float) -> Float
  @class native expm1(_ value: Float) -> Float
  @class native ln(_ value: Float) -> Float
  @class native ln1p(_ value: Float) -> Float
  @class native log(_ value: Float, base: Float) -> Float
  @class native log2(_ value: Float) -> Float
  @class native log10(_ value: Float) -> Float
  @class native sin(_ value: Float) -> Float
  @class native cos(_ value: Float) -> Float
  @class native tan(_ value: Float) -> Float
  @class native asin(_ value: Float) -> Float
  @class native acos(_ value: Float) -> Float
  @class native atan(_ value: Float) -> Float
  @class native atan2(_ y: Float, _ x: Float) -> Float
  @class native sinh(_ value: Float) -> Float
  @class native cosh(_ value: Float) -> Float
  @class native tanh(_ value: Float) -> Float
  @class native asinh(_ value: Float) -> Float
  @class native acosh(_ value: Float) -> Float
  @class native atanh(_ value: Float) -> Float
  /// Computes hypot(x, y) with the Float backend's binary64 semantics.
  @class native hypot(_ x: Float, _ y: Float) -> Float
}

/// Source-authored angle conversions; arguments and results use radians
/// except for `degrees`' result and `radians`' input.
class FloatAngleHelpers {
  @class
  degrees(_ value: Float) -> Float {
    return value * (180.0 / 3.141592653589793)
  }

  @class
  radians(_ value: Float) -> Float {
    return value * (3.141592653589793 / 180.0)
  }
}

export FloatMathKernel, FloatAngleHelpers

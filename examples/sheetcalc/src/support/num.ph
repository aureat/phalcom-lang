/// Math utilities for numbers missing from the core Number class.
/// All functions here are static and work with f64.
class Num {
  /// Floor of n, rounded down toward negative infinity.
  /// n - (n % 1) truncates toward zero, so negative numbers need correction.
  static floor(n) {
    let t = n - (n % 1)
    if (n < 0 and (n % 1) != 0) {
      return t - 1
    }
    return t
  }

  /// Ceiling of n, rounded up toward positive infinity.
  static ceil(n) {
    let t = n - (n % 1)
    if (n > 0 and (n % 1) != 0) {
      return t + 1
    }
    return t
  }

  /// Round n to nearest integer, ties away from zero.
  static round(n) {
    if (n >= 0) {
      return Num.floor(n + 0.5)
    }
    return Num.ceil(n - 0.5)
  }

  /// Absolute value of n.
  static abs(n) {
    if (n < 0) {
      return n.negated()
    }
    return n
  }

  /// Minimum of all given numbers.
  static min(nums) {
    if (nums.isEmpty) {
      return nil
    }
    var m = nums.at(0)
    for (n in nums) {
      if (n < m) {
        m = n
      }
    }
    return m
  }

  /// Maximum of all given numbers.
  static max(nums) {
    if (nums.isEmpty) {
      return nil
    }
    var m = nums.at(0)
    for (n in nums) {
      if (n > m) {
        m = n
      }
    }
    return m
  }

  /// Test whether n is an integer (has no fractional part).
  static isInt(n) {
    return n == (n - (n % 1))
  }
}

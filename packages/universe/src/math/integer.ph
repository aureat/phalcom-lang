/// Exact integer algorithms using arbitrary-precision Int arithmetic.
class IntegerMath {
  /// Returns the nonnegative greatest common divisor; `gcd(0, 0)` is zero.
  @class
  gcd(_ a: Int, _ b: Int) -> Int {
    let mut x = if a < 0 { -a } else { a }
    let mut y = if b < 0 { -b } else { b }
    while y != 0 {
      let next = x % y
      x = y
      y = next
    }
    return x
  }

  /// Returns the nonnegative least common multiple; zero if either input is zero.
  @class
  lcm(_ a: Int, _ b: Int) -> Int {
    if a == 0 or b == 0 { return 0 }
    let product = (a ~/ IntegerMath.gcd(a, b)) * b
    if product < 0 { return -product }
    return product
  }

  /// Computes the exact product from 1 through n; negative inputs raise ArgumentError.
  @class
  factorial(_ n: Int) -> Int {
    if n < 0 { ArgumentError.new("factorial requires a non-negative Int").raise() }
    let mut result = 1
    let mut i = 2
    while i <= n {
      result = result * i
      i = i + 1
    }
    return result
  }

  /// Returns the exact binomial coefficient; negative n raises ArgumentError, and k < 0 or k > n returns zero.
  @class
  choose(_ n: Int, _ k: Int) -> Int {
    if n < 0 { ArgumentError.new("choose requires a non-negative n").raise() }
    if k < 0 or k > n { return 0 }
    let count = if k < n - k { k } else { n - k }
    let mut result = 1
    let mut i = 1
    while i <= count {
      // Cancel before multiplication to avoid unnecessarily large temporary
      // integers. The remaining denominator divides the previous result.
      let numerator = n - count + i
      let common = IntegerMath.gcd(numerator, i)
      result = (result ~/ (i ~/ common)) * (numerator ~/ common)
      i = i + 1
    }
    return result
  }

  /// Returns the exact falling product; negative arguments raise ArgumentError, and k > n returns zero.
  @class
  fallingFactorial(_ n: Int, _ k: Int) -> Int {
    if n < 0 or k < 0 { ArgumentError.new("fallingFactorial requires non-negative arguments").raise() }
    if k > n { return 0 }
    let mut result = 1
    let mut i = 0
    while i < k {
      result = result * (n - i)
      i = i + 1
    }
    return result
  }

  @class
  risingFactorial(_ n: Int, _ k: Int) -> Int {
    if n < 0 or k < 0 { ArgumentError.new("risingFactorial requires non-negative arguments").raise() }
    if k == 0 { return 1 }
    if n == 0 { return 0 }
    let mut result = 1
    let mut i = 0
    while i < k {
      result = result * (n + i)
      i = i + 1
    }
    return result
  }

  /// Returns the floor of the square root; negative inputs raise `ArgumentError`.
  @class
  isqrt(_ n: Int) -> Int {
    if n < 0 { ArgumentError.new("isqrt requires a non-negative Int").raise() }
    if n < 2 { return n }
    let mut x = 1 << ((n.bitLength + 1) ~/ 2)
    let mut next = (x + n ~/ x) ~/ 2
    while next < x {
      x = next
      next = (x + n ~/ x) ~/ 2
    }
    return x
  }
}

export IntegerMath

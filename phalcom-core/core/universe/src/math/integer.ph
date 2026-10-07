/// Exact integer algorithms using arbitrary-precision Int arithmetic.
class IntegerMath {
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

  @class
  lcm(_ a: Int, _ b: Int) -> Int {
    if a == 0 or b == 0 { return 0 }
    let product = (a ~/ IntegerMath.gcd(a, b)) * b
    if product < 0 { return -product }
    return product
  }

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

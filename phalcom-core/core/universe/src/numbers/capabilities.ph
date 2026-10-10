/// Closed addition; exactness and rounding follow the conforming type.
trait Addable {
  +(_ other: Self) -> Self
}

/// Closed subtraction, independent of addition or ordering.
trait Subtractable {
  -(_ other: Self) -> Self
}

/// Closed multiplication, independent of division or ordering.
trait Multipliable {
  *(_ other: Self) -> Self
}

/// Additive inverse, independent of a numerical order.
trait Negatable {
  - -> Self
}

/// Integer-exponent power. Negative-exponent policy belongs to the conforming type.
trait IntegerPower {
  **(_ exponent: Int) -> Self
}

/// Additive identity, constructed on the conforming type's class side.
trait Zero {
  @class zero -> Self
}

/// Multiplicative identity, constructed on the conforming type's class side.
trait One {
  @class one -> Self
}

impl Addable for Int {}
impl Subtractable for Int {}
impl Multipliable for Int {}
impl Negatable for Int {}
impl Addable for Float {}
impl Subtractable for Float {}
impl Multipliable for Float {}
impl Negatable for Float {}

impl Zero for Int {
  @class zero -> Int { 0 }
}
impl One for Int {
  @class one -> Int { 1 }
}
impl Zero for Float {
  @class zero -> Float { 0.0 }
}
impl One for Float {
  @class one -> Float { 1.0 }
}

export Addable, Subtractable, Multipliable, Negatable, IntegerPower, Zero, One

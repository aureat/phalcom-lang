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

impl Addable for Int {}
impl Subtractable for Int {}
impl Multipliable for Int {}
impl Negatable for Int {}
impl Addable for Float {}
impl Subtractable for Float {}
impl Multipliable for Float {}
impl Negatable for Float {}

export Addable, Subtractable, Multipliable, Negatable, IntegerPower

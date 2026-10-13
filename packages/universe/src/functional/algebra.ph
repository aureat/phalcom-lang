//! Algebraic accumulation with explicit nominal conformance.
from .chain import Chain, NonEmpty
from universe.numbers.capabilities import Addable, Multipliable, Zero, One

/// An associative combination under an explicitly chosen equivalence.
/// Function values can use extensional equivalence; this is not a promise that
/// allocation identity or exact data identity is associative. Laws are not proofs.
trait Semigroup { combine(_ other: Self) -> Self }

/// A semigroup with a two-sided identity. No purity or law proof is implied.
trait Monoid where Self |= Semigroup { @class empty -> Self }

impl<T> Semigroup for Chain<T> {
  combine(_ other: Chain<T>) -> Chain<T> { self.append(other) }
}
impl<T> Monoid for Chain<T> {
  @class empty -> Chain<T> { Chain<T>::Empty }
}
impl<T> Semigroup for NonEmpty<T> {
  combine(_ other: NonEmpty<T>) -> NonEmpty<T> { self.append(other) }
}

/// Distinguishes additive accumulation from multiplicative accumulation.
/// Numeric bounds ensure operations exist; their algebraic laws still depend
/// on T. In particular, floating-point rounding can violate associativity.
data Additive<T>(_ value: T)
impl<T> Semigroup for Additive<T> where T |= Addable {
  combine(_ other: Additive<T>) -> Additive<T> { Additive<T>(self.value + other.value) }
}
impl<T> Monoid for Additive<T> where T |= Addable, T |= Zero {
  @class empty -> Additive<T> { Additive<T>(T.zero) }
}

data Multiplicative<T>(_ value: T)
impl<T> Semigroup for Multiplicative<T> where T |= Multipliable {
  combine(_ other: Multiplicative<T>) -> Multiplicative<T> { Multiplicative<T>(self.value * other.value) }
}
impl<T> Monoid for Multiplicative<T> where T |= Multipliable, T |= One {
  @class empty -> Multiplicative<T> { Multiplicative<T>(T.one) }
}

data AllTrue(_ value: Bool)
impl Semigroup for AllTrue { combine(_ other: AllTrue) -> AllTrue { AllTrue(self.value and other.value) } }
impl Monoid for AllTrue { @class empty -> AllTrue { AllTrue(true) } }

data AnyTrue(_ value: Bool)
impl Semigroup for AnyTrue { combine(_ other: AnyTrue) -> AnyTrue { AnyTrue(self.value or other.value) } }
impl Monoid for AnyTrue { @class empty -> AnyTrue { AnyTrue(false) } }

/// Retains the first present value; absence is the identity.
data First<T>(_ value: Option<T>)
impl<T> Semigroup for First<T> {
  combine(_ other: First<T>) -> First<T> {
    match self.value { Option::Some(_) => self; Option::None => other }
  }
}
impl<T> Monoid for First<T> { @class empty -> First<T> { First<T>(Option::None) } }

/// Retains the last present value; absence is the identity.
data Last<T>(_ value: Option<T>)
impl<T> Semigroup for Last<T> {
  combine(_ other: Last<T>) -> Last<T> {
    match other.value { Option::Some(_) => other; Option::None => self }
  }
}
impl<T> Monoid for Last<T> { @class empty -> Last<T> { Last<T>(Option::None) } }

/// Reverses an existing semigroup's combination order.
data Dual<M>(_ value: M)
impl<M> Semigroup for Dual<M> where M |= Semigroup {
  combine(_ other: Dual<M>) -> Dual<M> { Dual<M>(other.value.combine(self.value)) }
}
impl<M> Monoid for Dual<M> where M |= Semigroup, M |= Monoid {
  @class empty -> Dual<M> { Dual<M>(M.empty) }
}

/// Endomorphisms compose right to left: (f combine g)(x) = f(g(x)).
/// Associativity holds observationally for pure total callbacks.
data Endo<T> private(_ transform: (T) -> T)
impl<T> Endo<T> {
  @class new(_ f: (T) -> T) -> Endo<T> { Endo<T>(f) }
  call(_ value: T) -> T { self.transform.call(value) }
}
impl<T> Semigroup for Endo<T> {
  combine(_ other: Endo<T>) -> Endo<T> { Endo<T>.new(|value| { self.call(other.call(value)) }) }
}
impl<T> Monoid for Endo<T> { @class empty -> Endo<T> { Endo<T>.new(|value| { value }) } }

export Semigroup, Monoid, Additive, Multiplicative, AllTrue, AnyTrue, First, Last, Dual, Endo

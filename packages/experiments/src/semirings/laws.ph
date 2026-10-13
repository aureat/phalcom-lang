/// Alternative is a commutative monoid with zero. Sequential is a monoid with
/// one and need not commute. Sequential distributes over alternative on both
/// sides; zero annihilates sequential on both sides. These are conformance laws,
/// not compiler-proved facts. Neither idempotence nor a Kleene star is assumed.
trait Semiring {
  @class zero -> Self
  @class one -> Self
  alternative(_ other: Self) -> Self
  sequential(_ other: Self) -> Self
}
export Semiring

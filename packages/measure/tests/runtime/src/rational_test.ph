from measure.exact.rational import Rational
from .support import Test

Test.expect(Rational.of(2, over: 4) == Rational.of(1, over: 2), "reduction")
Test.expect(Rational.of(-2, over: -4) == Rational.of(1, over: 2), "sign normalization")
Test.expect(Rational.of(0, over: 999) == Rational.zero, "zero canonicalization")
Test.expect(Rational.of(1, over: 2) + Rational.of(1, over: 3) == Rational.of(5, over: 6), "addition")
Test.expect(Rational.of(2, over: 3) * Rational.of(9, over: 4) == Rational.of(3, over: 2), "multiplication")
Test.expect(Rational.of(2, over: 3) / Rational.of(4, over: 5) == Rational.of(5, over: 6), "division")
Test.expect(Rational.of(2, over: 3) ** -2 == Rational.of(9, over: 4), "negative power")
Test.expect(Rational.decimal(254, places: 4) == Rational.of(127, over: 5000), "decimal exactness")
Test.expect(Rational.of(10 ** 40, over: 3).numerator == 10 ** 40, "large Int numerator")

Test.pass("rational")

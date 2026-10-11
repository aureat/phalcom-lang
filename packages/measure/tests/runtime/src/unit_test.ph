from measure.exact.rational import Rational
from measure.standard.si.base import m, km, h
from measure.standard.si.mechanics import mps, kmph, N
from .support import Test

Test.expect((m / h * h).equivalent(m.expression), "unit cancellation")
Test.expect((km / h * h).scale == Rational.thousand, "scale cancellation")
Test.expect(kmph.expression.scale == Rational.of(5, over: 18), "km/h exact scale")
Test.expect((N * m).dimension.equivalent(N.expression.dimension * m.expression.dimension), "unit product dimension")
Test.expect((m ** 2).scale == Rational.one, "base square scale")

Test.pass("unit")

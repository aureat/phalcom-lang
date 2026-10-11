from measure.exact.rational import Rational
from measure.quantity.measure import Measure, PointMeasure
from measure.standard.si.base import km, m
from measure.standard.si.temperature import K, degC
from .support import Test

let entered = Measure(Rational.whole(5), km)
Test.expect(entered.quantity.in(m) == Rational.whole(5000), "Measure normalizes to Quantity")
Test.expect(entered.toString == "5 km", "Measure retains selected unit")

let temperature = PointMeasure(Rational.whole(20), degC)
Test.expect(
  temperature.point.in(K) == Rational.of(5863, over: 20),
  "PointMeasure normalizes affine coordinate"
)

Test.pass("measure")

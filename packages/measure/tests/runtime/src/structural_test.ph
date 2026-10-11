from measure.exact.rational import Rational
from measure.standard.kinds import Force
from measure.standard.si.base import kg, m, s
from measure.standard.si.mechanics import N
from .support import Test

let forceExpr = kg * m / (s ** 2)
let raw = forceExpr(25)
let force = raw.refined(to: Force())

Test.expect(raw.in(N) == Rational.whole(25), "structural quantity expresses in named unit")
Test.expect(force.in(N) == Rational.whole(25), "UnitExpr callable structural refinement")
Test.expect(raw.dimension.equivalent(Force().dimension), "structural dimension retained")

Test.pass("structural")

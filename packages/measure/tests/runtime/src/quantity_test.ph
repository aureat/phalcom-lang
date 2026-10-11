from measure.exact.rational import Rational
from measure.standard.si.base import m, km, h, kg
from measure.standard.si.mechanics import mps, mps2, N, J, Nm, Mechanics
from .support import Test

let mixed = km(5) + m(300)
Test.expect(mixed.in(m) == Rational.whole(5300), "mixed unit addition")

let distance = mps(100) * h(4)
Test.expect(distance.in(km) == Rational.whole(1440), "velocity × time -> length")

let force = kg(1500) * mps2(2)
Test.expect(force.in(N) == Rational.whole(3000), "mass × acceleration -> force")

let doubled = m(5).scaled(by: Rational.whole(2))
Test.expect(doubled.in(m) == Rational.whole(10), "named scalar scaling")

let work = Mechanics.work(N(10), over: m(3))
let torque = Mechanics.torque(N(10), arm: m(3))
Test.expect(work.in(J) == Rational.whole(30), "named work semantic")
Test.expect(torque.in(Nm) == Rational.whole(30), "named torque semantic")

Test.pass("quantity")

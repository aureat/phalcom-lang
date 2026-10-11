from measure.exact.rational import Rational
from measure.standard.si.temperature import degC, degF, deltaC
from .support import Test

let freezingC = degC(0)
let freezingF = degF(32)
Test.expect(freezingC.in(degF) == Rational.whole(32), "point observation in affine unit")
Test.expect((freezingC - freezingF).in(deltaC) == Rational.zero, "0C == 32F")

let room = degC(20)
let warmer = room.advanced(by: deltaC(5))
Test.expect((warmer - room).in(deltaC) == Rational.whole(5), "point translation")
Test.expect((warmer.retreated(by: deltaC(5)) - room).in(deltaC) == Rational.zero, "point inverse translation")

Test.pass("temperature")

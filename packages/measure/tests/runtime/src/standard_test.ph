from measure.exact.rational import Rational
from measure.standard.accepted import mi
from measure.standard.information import KiB, B, bps
from measure.standard.si.angle import rad, deg, sr
from measure.standard.si.base import m, s, cd
from measure.standard.si.electromagnetism import V, ohm
from measure.standard.si.base import A
from measure.standard.si.mechanics import J, Nm, W
from measure.standard.si.nuclear import Gy, Sv
from measure.standard.si.photometry import lm
from .support import Test

Test.expect(mi(1).in(m) == Rational.of(201168, over: 125), "international mile exactness")
Test.expect(KiB(1).in(B) == Rational.whole(1024), "KiB exact binary scale")
Test.expect((V(12) / A(3)).in(ohm) == Rational.whole(4), "voltage/current resistance")
Test.expect(J.expression.equivalent(Nm.expression), "J and N.m structurally equivalent")
Test.expect(Gy.expression.equivalent(Sv.expression), "Gy and Sv structurally equivalent")
Test.expect(deg(180.0).inApprox(rad) > 3.14159, "degree approximate conversion")
Test.expect((V(12) * A(3)).in(W) == Rational.whole(36), "voltage/current power")
Test.expect((cd(2) * sr(3)).in(lm) == Rational.whole(6), "luminous intensity × solid angle -> flux")
Test.expect((B(100) / bps(20)).in(s) == Rational.whole(40), "information/rate -> elapsed time")

Test.pass("standard")

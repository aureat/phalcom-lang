from measure.exact.rational import Rational
from measure.standard.prefixes import kilo, milli
from measure.standard.si.base import m
from .support import Test

Test.expect(kilo.symbol == "k", "const Prefix retains runtime behavior")
Test.expect(kilo.scale == Rational.thousand, "const Prefix retains Rational payload")
Test.expect(kilo(m).exact(Rational.one).in(m) == Rational.thousand, "const Prefix is callable")
Test.expect(milli(m).exact(Rational.whole(1000)).in(m) == Rational.one, "milli scale")

Test.pass("prefix")

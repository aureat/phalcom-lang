//! Selected exact non-SI units accepted for practical use.

from ..exact.rational import Rational
from ..unit.linear import LinearUnit
from .kinds import Length, Mass
from .si.base import m, kg

/// International inch: exactly 0.0254 m.
let inch = LinearUnit<Length>.define(
  Length(), "in",
  m.expression.scaled(by: Rational.of(127, over: 5000), symbol: "in")
)
let ft = inch.scaled(by: Rational.whole(12), symbol: "ft")
let yd = ft.scaled(by: Rational.whole(3), symbol: "yd")
let mi = yd.scaled(by: Rational.whole(1760), symbol: "mi")

/// Metric tonne: exactly 1000 kg.
let tonne = kg.scaled(by: Rational.thousand, symbol: "t")

export inch, ft, yd, mi, tonne

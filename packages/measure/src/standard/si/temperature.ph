//! Absolute temperature points, linear differences, and temperature rates.

from ...exact.rational import Rational
from ...unit.affine import AffineUnit
from ...unit.linear import LinearUnit
from .base import s, min
from ..kinds import Temperature, TemperatureDifference, TemperatureRate

let K = AffineUnit<Temperature>.define(
  Temperature(), "K",
  scale: Rational.one,
  offset: Rational.zero
)

let degC = AffineUnit<Temperature>.define(
  Temperature(), "°C",
  scale: Rational.one,
  offset: Rational.of(5463, over: 20)
)

let degF = AffineUnit<Temperature>.define(
  Temperature(), "°F",
  scale: Rational.of(5, over: 9),
  offset: Rational.of(45967, over: 180)
)

let deltaC = LinearUnit<TemperatureDifference>.base(
  TemperatureDifference(), "Δ°C"
)

let deltaF = deltaC.scaled(
  by: Rational.of(5, over: 9),
  symbol: "Δ°F"
)

let deltaKps = LinearUnit<TemperatureRate>.define(
  TemperatureRate(), "K/s", deltaK / s
)

let deltaCPerMin = LinearUnit<TemperatureRate>.define(
  TemperatureRate(), "Δ°C/min", deltaC / min
)

export K, degC, degF, deltaC, deltaF, deltaKps, deltaCPerMin

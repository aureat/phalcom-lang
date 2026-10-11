//! SI base/reference units and common exact prefixed/time units.

from ...exact.rational import Rational
from ...unit.linear import LinearUnit
from ..kinds import (
  Length, Mass, ElapsedTime, ElectricCurrent, TemperatureDifference,
  AmountOfSubstance, LuminousIntensity
)
from ..prefixes import milli, centi, kilo, micro, nano

let m = LinearUnit<Length>.base(Length(), "m")
let kg = LinearUnit<Mass>.base(Mass(), "kg")
let g = kg.scaled(by: Rational.thousandth, symbol: "g")
let s = LinearUnit<ElapsedTime>.base(ElapsedTime(), "s")
let A = LinearUnit<ElectricCurrent>.base(ElectricCurrent(), "A")
let deltaK = LinearUnit<TemperatureDifference>.base(TemperatureDifference(), "K")
let mol = LinearUnit<AmountOfSubstance>.base(AmountOfSubstance(), "mol")
let cd = LinearUnit<LuminousIntensity>.base(LuminousIntensity(), "cd")

let mm = milli(m)
let cm = centi(m)
let km = kilo(m)
let um = micro(m)
let nm = nano(m)

let mg = milli(g)
let ug = micro(g)

let ms = milli(s)
let us = micro(s)
let ns = nano(s)
let min = s.scaled(by: Rational.whole(60), symbol: "min")
let h = min.scaled(by: Rational.whole(60), symbol: "h")
let d = h.scaled(by: Rational.whole(24), symbol: "d")

export (
  m, kg, g, s, A, deltaK, mol, cd,
  mm, cm, km, um, nm, mg, ug, ms, us, ns, min, h, d
)

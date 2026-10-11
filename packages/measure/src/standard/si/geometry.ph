//! Exact area, volume, and litre-family units.

from ...exact.rational import Rational
from ...unit.linear import LinearUnit
from ..kinds import Area, Volume
from .base import m, cm, km

let m2 = LinearUnit<Area>.define(Area(), "m²", m ** 2)
let cm2 = LinearUnit<Area>.define(Area(), "cm²", cm ** 2)
let km2 = LinearUnit<Area>.define(Area(), "km²", km ** 2)

let m3 = LinearUnit<Volume>.define(Volume(), "m³", m ** 3)
let cm3 = LinearUnit<Volume>.define(Volume(), "cm³", cm ** 3)

let L = m3.scaled(by: Rational.of(1, over: 1000), symbol: "L")
let mL = L.scaled(by: Rational.of(1, over: 1000), symbol: "mL")

export m2, cm2, km2, m3, cm3, L, mL
